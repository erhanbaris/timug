use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::Datelike;
use colored::Colorize;
use minijinja::{context, path_loader, Environment, Value};
use run_shell::cmd;
use serde::{Deserialize, Serialize};

use crate::{
    context::{get_context, get_mut_context},
    extensions::Extension,
    pages::{Pages, POST_HTML},
    posts::Posts,
    tools::parse_yaml,
};

pub struct RenderEngine<'a> {
    pub env: Environment<'a>,
    pub posts: Posts,
}

impl<'a> RenderEngine<'a> {
    pub fn new() -> Self {
        let env = Environment::new();
        Self {
            env,
            posts: Posts::default(),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let ctx = get_context();
        self.env.set_loader(path_loader(ctx.templates_path.clone()));
        drop(ctx);

        self.pre_template_process();

        self.build_filters();
        self.build_globals();
        self.build_functions();

        self.parse_posts()?;
        self.parse_pages()?;

        self.generate_pages()?;
        self.generate_posts()?;

        self.move_statics()?;
        self.template_process();

        Ok(())
    }

    fn pre_template_process(&mut self) {
        let ctx = get_context();
        let pre_processes = &ctx.template_config.pre_process;
        let current_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&ctx.templates_path).unwrap();
        for process in pre_processes.iter() {
            cmd!(process).run().unwrap();
        }
        std::env::set_current_dir(&current_dir).unwrap();
        drop(ctx);
    }

    fn template_process(&mut self) {
        let ctx = get_context();
        let processes = &ctx.template_config.process;
        let current_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&ctx.templates_path).unwrap();
        for process in processes.iter() {
            cmd!(process).run().unwrap();
        }
        std::env::set_current_dir(&current_dir).unwrap();
        drop(ctx);
    }

    pub fn register_extension<T: Extension<'a>>(&mut self) -> anyhow::Result<()> {
        T::register(&mut self.env);
        let mut ctx = get_mut_context();

        if !T::header().is_empty() {
            ctx.headers.push(T::header());
        }

        if !T::after_body().is_empty() {
            ctx.after_bodies.push(T::after_body());
        }

        drop(ctx);
        Ok(())
    }

    pub fn build_globals(&mut self) {
        let ctx = get_context();
        let config = &ctx.config;
        self.env.add_global("author_name", &config.author);
        self.env.add_global("author_email", &config.email);
        self.env.add_global("site_url", &config.site_url);
        self.env.add_global("lang", &config.lang);
        self.env.add_global("description", &config.description);
        self.env.add_global("blog_name", &config.title);
    }

    pub fn parse_posts(&mut self) -> anyhow::Result<()> {
        let ctx = get_context();
        self.posts = Posts::load(&ctx.posts_path)?;
        drop(ctx);

        let mut ctx = get_mut_context();
        ctx.posts_value = Value::from_object(self.posts.clone());
        Ok(())
    }

    pub fn parse_pages(&mut self) -> anyhow::Result<()> {
        let mut ctx = get_mut_context();
        let mut pages = Pages::default();
        pages.load_base_pages(&ctx.templates_path)?;
        pages.load_custom_pages(&ctx.pages_path)?;

        for page in pages.items.iter() {
            self.env
                .add_template_owned(page.path.clone(), page.content.clone())?;
        }
        ctx.pages_value = Value::from_object(pages);
        Ok(())
    }

    pub fn generate_posts(&mut self) -> anyhow::Result<()> {
        let ctx = get_context();
        let deployment_folder = ctx.config.blog_path.join(&ctx.config.deployment_folder);

        let mut created_paths = Vec::new();
        let mut generate_path = |path: &PathBuf| -> anyhow::Result<()> {
            if !created_paths.contains(path) {
                std::fs::create_dir_all(path)?;
                created_paths.push(path.clone());
            }

            Ok(())
        };

        let posts = ctx
            .posts_value
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Posts>())
            .context("'posts' is not a Posts type".to_string())?;

        for (index, post) in posts.items.iter().enumerate() {
            if post.draft() {
                continue;
            }

            let mut post = post.clone();
            let date = post.date();
            let file_path = deployment_folder
                .join(date.year().to_string())
                .join(date.month().to_string())
                .join(date.day().to_string());
            let file_name = file_path.join(format!("{}.html", post.slug()));

            println!("{}: {}", "Compiling".yellow(), post.slug());

            if post.content().contains("{%") {
                let context = context!(config => ctx.config, post => Value::from_object(post.clone()), posts => ctx.posts_value, pages => ctx.pages_value, active_page => "posts");
                let content = self.env.render_str(post.content(), &context)?;
                post.set_content(content);
            }

            let mut content = String::new();
            pulldown_cmark::html::push_html(&mut content, parse_yaml(post.content()));
            post.set_content(content);

            let template = self.env.get_template(POST_HTML)?;
            let context = context!(config => ctx.config, post => Value::from_object(post.clone()), headers => ctx.headers, after_bodies => ctx.after_bodies, posts => ctx.posts_value, pages => ctx.pages_value, active_page => "posts", index => index);
            let content = template.render(context)?;
            generate_path(&file_path)?;
            self.compress_and_write(content, &file_name)?;

            println!("{}: {}", "Generated".green(), file_name.display());
        }

        Ok(())
    }

    fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
        std::fs::create_dir_all(&dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                Self::copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }

    pub fn move_statics(&mut self) -> anyhow::Result<()> {
        let ctx = get_context();

        let deployment_folder = ctx
            .config
            .blog_path
            .join(&ctx.config.deployment_folder)
            .join("assets");
        Ok(Self::copy_dir_all(&ctx.statics_path, &deployment_folder)?)
    }

    pub fn generate_pages(&mut self) -> anyhow::Result<()> {
        let ctx = get_context();
        let pages = ctx
            .pages_value
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Pages>())
            .context("'pages' is not a Pages type".to_string())?;

        for page in pages.items.iter() {
            if !page.render {
                continue;
            }

            let template = self.env.get_template(&page.path)?;
            println!("{}: {}", "Rendering".yellow(), page.path);
            let content = template.render(context!(config => ctx.config, headers => ctx.headers, after_bodies => ctx.after_bodies, posts => ctx.posts_value, pages => ctx.pages_value, active_page => page))?;

            let file_path = ctx
                .config
                .blog_path
                .join(ctx.config.deployment_folder.clone());
            let file_name = file_path.join(&page.file_name);

            self.compress_and_write(content, &file_name)?;
            println!("{}: {}", "Generated".green(), file_name.display());
        }

        Ok(())
    }

    fn compress_and_write(&self, content: String, path: &PathBuf) -> anyhow::Result<()> {
        let mut file = File::create(path)?;
        //let content = Self::compress_html(content);
        Ok(file.write_all(content.as_bytes())?)
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub name: String,
    #[serde(rename = "pre-process")]
    pub pre_process: Vec<String>,
    #[serde(rename = "process")]
    pub process: Vec<String>,
    #[serde(rename = "post-process")]
    pub post_process: Vec<String>,
}
