use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use minijinja::{context, path_loader, Environment, Value};
use run_shell::cmd;

use crate::{
    context::{get_context, get_mut_context},
    extensions::Extension,
    pages::{Pages, POSTS_HTML},
    post::PostContext,
    posts::Posts,
    tag::TagContext,
};

pub trait Renderable {
    type Context;
    fn render(&self, engine: &RenderEngine<'_>, ctx: Self::Context) -> anyhow::Result<()>;
}

pub struct RenderEngine<'a> {
    pub env: Environment<'a>,
}

impl<'a> RenderEngine<'a> {
    pub fn new() -> Self {
        let env = Environment::new();

        Self { env }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let ctx = get_context();
        self.env.set_loader(path_loader(ctx.template.path.clone()));
        drop(ctx);

        self.pre_template_process();

        self.build_filters();
        self.build_globals();
        self.build_functions();

        self.parse_posts()?;
        self.parse_pages()?;

        self.generate_pages()?;
        self.generate_posts()?;
        self.generate_tags()?;

        self.move_statics()?;
        self.template_process();

        Ok(())
    }

    pub fn update_status(&self, status: String, message: &str) {
        println!("{}: {}", status, message);
    }

    fn pre_template_process(&mut self) {
        let ctx = get_context();
        let pre_processes = &ctx.template.config.pre_process;
        let current_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&ctx.template.path).unwrap();
        for process in pre_processes.iter() {
            cmd!(process).run().unwrap();
        }
        std::env::set_current_dir(&current_dir).unwrap();
        drop(ctx);
    }

    fn template_process(&mut self) {
        let ctx = get_context();
        let processes = &ctx.template.config.process;
        let current_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&ctx.template.path).unwrap();
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
        let posts = Posts::load()?;

        let mut ctx = get_mut_context();
        ctx.posts_value = Value::from_object(posts.clone());
        ctx.posts = posts;
        Ok(())
    }

    pub fn parse_pages(&mut self) -> anyhow::Result<()> {
        let mut pages = Pages::default();
        pages.load_base_pages()?;
        pages.load_custom_pages()?;

        let mut ctx = get_mut_context();

        for page in pages.items.iter() {
            self.env
                .add_template_owned(page.path.clone(), page.content.clone())?;
        }
        ctx.pages_value = Value::from_object(pages.clone());
        ctx.pages = pages;
        Ok(())
    }

    pub fn create_context(&self) -> Value {
        let ctx = get_context();
        context! {
            config => ctx.config,
            headers => ctx.headers,
            after_bodies => ctx.after_bodies,
            tags => ctx.tags,
            posts => ctx.posts_value,
            pages => ctx.pages_value
        }
    }

    pub fn generate_posts(&mut self) -> anyhow::Result<()> {
        let ctx = get_context();
        let deployment_folder = ctx.config.blog_path.join(&ctx.config.deployment_folder);

        for (index, post) in ctx.posts.items.iter().enumerate() {
            post.render(
                self,
                PostContext {
                    index,
                    deployment_folder: deployment_folder.clone(),
                },
            )?;
        }

        Ok(())
    }

    pub fn generate_tags(&mut self) -> anyhow::Result<()> {
        let ctx = get_context();

        let deployment_folder = ctx.config.blog_path.join(&ctx.config.deployment_folder);
        let file_path = deployment_folder.join("tags");
        let posts_page = ctx
            .pages
            .get(POSTS_HTML)
            .context("posts.html could not found")?;
        std::fs::create_dir_all(&file_path)?;

        for (index, tag) in ctx.tags.iter().enumerate() {
            tag.render(
                self,
                TagContext {
                    folder: deployment_folder.clone(),
                    index,
                    template_path: posts_page.path.clone(),
                },
            )?;
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
        for page in ctx.pages.items.iter() {
            page.render(self, ())?;
        }

        Ok(())
    }

    pub fn compress_and_write(&self, content: String, path: &PathBuf) -> anyhow::Result<()> {
        let mut file = File::create(path)?;
        //let content = Self::compress_html(content);
        Ok(file.write_all(content.as_bytes())?)
    }
}
