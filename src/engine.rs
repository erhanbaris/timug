use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use minijinja::{context, path_loader, Environment, Value};
use subprocess::{Exec, Redirection};

use crate::extensions::{alertbox::AlertBox, analytics::Analytics, codeblock::Codeblock, contacts::Contacts, fontawesome::FontAwesome, gist::Gist, info::Info, projects::Projects, quote::Quote, reading::Reading, social_media_share::SocialMediaShare, stats::Stats};

use crate::{
    context::{get_context, get_mut_context},
    extensions::Extension,
    pages::{Pages, POSTS_HTML},
    posts::{Posts, PostsContext},
    tag::TagContext,
    tools::get_path,
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

        log::debug!("Template path: {}", ctx.template.path.display());
        self.env.set_loader(path_loader(ctx.template.path.clone()));
        drop(ctx);

        self.pre_template_process()?;

        self.build_filters();
        self.build_globals();
        self.build_functions();

        self.build_pages()?;
        self.template_process()?;

        Ok(())
    }

    pub fn build_pages(&mut self) -> anyhow::Result<()> {
        self.clear_tags();
        self.parse_posts()?;
        self.parse_pages()?;

        self.generate_pages()?;
        self.generate_posts()?;
        self.generate_tags()?;

        self.move_assets()?;

        Ok(())
    }

    pub fn clear_tags(&mut self) {
        let mut ctx = get_mut_context();
        ctx.tags.clear();
    }

    pub fn update_status(&self, status: String, message: &str) {
        log::debug!("{}: {}", status, message);
    }

    fn pre_template_process(&mut self) -> anyhow::Result<()> {
        let ctx = get_context();
        let pre_processes = &ctx.template.config.pre_process;
        let current_dir = std::env::current_dir()?;
        std::env::set_current_dir(&ctx.template.path)?;

        for process in pre_processes.iter() {
            log::debug!("Template Preprocess: {}", process);
            let command = Exec::shell(process);
            let output = command
                .stdout(Redirection::Pipe)
                .stderr(Redirection::Merge)
                .capture()?
                .stdout_str();
            log::debug!("Output: {}", output);
        }
        std::env::set_current_dir(&current_dir)?;
        drop(ctx);
        Ok(())
    }

    fn template_process(&mut self) -> anyhow::Result<()> {
        let ctx = get_context();
        let processes = &ctx.template.config.process;
        let current_dir = std::env::current_dir()?;
        std::env::set_current_dir(&ctx.template.path)?;
        for process in processes.iter() {
            let command = Exec::shell(process.replace("{publish-folder}", &get_path(&ctx.config.deployment_folder)?));
            log::debug!("Template Preprocess: {}", command.to_cmdline_lossy());
            let output = command
                .stdout(Redirection::Pipe)
                .stderr(Redirection::Merge)
                .capture()?
                .stdout_str();
            log::debug!("Output: {}", output);
        }
        std::env::set_current_dir(&current_dir)?;
        drop(ctx);
        Ok(())
    }

    pub fn register_extension<T: Extension<'a>>(&mut self) -> anyhow::Result<()> {
        T::register(&mut self.env);
        let mut ctx = get_mut_context();

        if !T::header().is_empty() {
            ctx.headers.push(T::header());
        }
        T::after_body(&mut ctx);
        drop(ctx);
        Ok(())
    }

    pub fn build_globals(&mut self) {
        log::debug!("Build globals");
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
        log::debug!("Parse posts");
        let posts = Arc::new(Posts::load()?);

        let mut ctx = get_mut_context();
        ctx.posts_value = Value::from_dyn_object(posts.clone());
        ctx.posts = posts;
        Ok(())
    }

    pub fn parse_pages(&mut self) -> anyhow::Result<()> {
        log::debug!("Parse pages");
        let mut pages = Pages::default();
        pages.load_base_pages()?;
        pages.load_custom_pages()?;

        let mut ctx = get_mut_context();

        for page in pages.items.iter() {
            self.env
                .add_template_owned(page.path.clone(), page.content.clone())?;
        }

        let pages = Arc::new(pages);
        ctx.pages_value = Value::from_dyn_object(pages.clone());
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
            pages => ctx.pages_value,
            navs => ctx.config.navs,
        }
    }

    pub fn generate_posts(&mut self) -> anyhow::Result<()> {
        log::debug!("Generate posts");
        let ctx = get_context();
        let deployment_folder = ctx.config.blog_path.join(&ctx.config.deployment_folder);

        let posts_ctx = PostsContext { deployment_folder };
        ctx.posts.render(self, posts_ctx)?;

        Ok(())
    }

    pub fn generate_tags(&mut self) -> anyhow::Result<()> {
        log::debug!("Generate tags");
        let ctx = get_context();

        let deployment_folder = ctx.config.blog_path.join(&ctx.config.deployment_folder);
        let file_path = deployment_folder.join("tags");
        let posts_page = ctx
            .pages
            .get(POSTS_HTML)
            .context("posts.html could not found")?;
        std::fs::create_dir_all(&file_path)?;

        for (index, tag) in ctx.tags.iter().enumerate() {
            let ctx = TagContext {
                folder: file_path.clone(),
                index,
                template_path: posts_page.path.clone(),
            };

            tag.render(self, ctx)?;
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

    pub fn move_assets(&mut self) -> anyhow::Result<()> {
        log::debug!("Generate assets");
        let ctx = get_context();

        let deployment_folder = ctx
            .config
            .blog_path
            .join(&ctx.config.deployment_folder)
            .join("assets");
        Ok(Self::copy_dir_all(&ctx.statics_path, &deployment_folder)?)
    }

    pub fn generate_pages(&mut self) -> anyhow::Result<()> {
        log::debug!("Generate pages");
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

pub fn create_engine() -> anyhow::Result<RenderEngine<'static>> {
    let mut engine = RenderEngine::new();
    engine.register_extension::<Codeblock>()?;
    engine.register_extension::<Quote>()?;
    engine.register_extension::<Gist>()?;
    engine.register_extension::<AlertBox>()?;
    engine.register_extension::<FontAwesome>()?;
    engine.register_extension::<Info>()?;
    engine.register_extension::<SocialMediaShare>()?;
    engine.register_extension::<Reading>()?;
    engine.register_extension::<Projects>()?;
    engine.register_extension::<Contacts>()?;
    engine.register_extension::<Stats>()?;
    engine.register_extension::<Analytics>()?;
    Ok(engine)
}
