use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use minijinja::{context, path_loader, Environment, Value};
use snafu::{OptionExt, ResultExt};
use subprocess::{Exec, Redirection};

use crate::{
    consts::POSTS_HTML,
    error::{CurrentDirChangeSnafu, DirectoryCopyFailedSnafu, FileCreationFailedSnafu, FolderCreationFailedSnafu, MarkdownTemplateAddFailedSnafu, NoCurrentDirSnafu, SubProcessSnafu, TemplateReferenceNotFoundSnafu, WriteSnafu},
    extensions::{alertbox::AlertBox, analytics::Analytics, codeblock::Codeblock, contacts::Contacts, gist::Gist, info::Info, projects::Projects, quote::Quote, reading::Reading, social_media_share::SocialMediaShare, stats::Stats},
};

use crate::{
    context::{get_context, get_mut_context},
    extensions::Extension,
    pages::Pages,
    posts::Posts,
    tag::TagContext,
    tools::get_path,
};

pub trait Renderable {
    type Context;
    fn render(&self, engine: &RenderEngine<'_>, ctx: Self::Context) -> crate::error::Result<()>;
}

pub struct RenderEngine<'a> {
    pub env: Environment<'a>,
}

impl<'a> RenderEngine<'a> {
    pub fn new() -> Self {
        let env = Environment::new();

        Self { env }
    }

    pub fn run(&mut self) -> crate::error::Result<()> {
        let ctx = get_context(snafu::location!())?;

        log::debug!("Template path: {}", ctx.template.path.display());
        self.env.set_loader(path_loader(ctx.template.path.clone()));
        drop(ctx);

        self.pre_template_process()?;

        self.build_filters();
        self.build_globals()?;
        self.build_functions();

        self.build_pages()?;
        self.template_process()?;

        Ok(())
    }

    pub fn build_pages(&mut self) -> crate::error::Result<()> {
        self.clear_tags()?;
        self.parse_posts()?;
        self.parse_pages()?;

        self.generate_pages()?;
        self.generate_posts()?;
        self.generate_tags()?;

        self.move_assets()?;

        Ok(())
    }

    pub fn clear_tags(&mut self) -> crate::Result<()> {
        let mut ctx = get_mut_context(snafu::location!())?;
        ctx.tags.clear();
        Ok(())
    }

    pub fn update_status(&self, status: String, message: &str) {
        log::debug!("{}: {}", status, message);
    }

    fn pre_template_process(&mut self) -> crate::error::Result<()> {
        let ctx = get_context(snafu::location!())?;
        let pre_processes = &ctx.template.config.pre_process;
        let current_dir = std::env::current_dir().context(NoCurrentDirSnafu)?;
        std::env::set_current_dir(&ctx.template.path).context(CurrentDirChangeSnafu { path: ctx.template.path.clone() })?;

        for process in pre_processes.iter() {
            log::debug!("Template preprocess: {}", process);
            let command = Exec::shell(process);
            let output = command
                .stdout(Redirection::Pipe)
                .stderr(Redirection::Merge)
                .capture()
                .context(SubProcessSnafu { command: process })?
                .stdout_str();
            log::debug!("Output: {}", output);
        }
        std::env::set_current_dir(&current_dir).context(CurrentDirChangeSnafu { path: current_dir })?;
        drop(ctx);
        Ok(())
    }

    fn template_process(&mut self) -> crate::error::Result<()> {
        let ctx = get_context(snafu::location!())?;
        let processes = &ctx.template.config.process;
        let current_dir = std::env::current_dir().context(NoCurrentDirSnafu)?;
        std::env::set_current_dir(&ctx.template.path).context(CurrentDirChangeSnafu { path: ctx.template.path.clone() })?;

        for process in processes.iter() {
            let command = Exec::shell(process.replace("{publish-folder}", &get_path(&ctx.config.deployment_folder)?));
            log::debug!("Template process: {}", command.to_cmdline_lossy());
            let output = command
                .stdout(Redirection::Pipe)
                .stderr(Redirection::Merge)
                .capture()
                .context(SubProcessSnafu { command: process })?
                .stdout_str();
            log::debug!("Output: {}", output);
        }

        std::env::set_current_dir(&current_dir).context(CurrentDirChangeSnafu { path: current_dir })?;
        drop(ctx);
        Ok(())
    }

    pub fn register_extension<T: Extension<'a>>(&mut self) -> crate::error::Result<()> {
        T::register(&mut self.env);
        let mut ctx = get_mut_context(snafu::location!())?;

        if !T::header().is_empty() {
            ctx.headers.push(T::header());
        }
        T::after_body(&mut ctx);
        drop(ctx);
        Ok(())
    }

    pub fn build_globals(&mut self) -> crate::Result<()> {
        log::debug!("Build globals");
        let ctx = get_context(snafu::location!())?;
        let config = &ctx.config;
        self.env.add_global("author_name", &config.author);
        self.env.add_global("author_email", &config.email);
        self.env.add_global("site_url", &config.site_url);
        self.env.add_global("lang", &config.lang);
        self.env.add_global("description", &config.description);
        self.env.add_global("blog_name", &config.title);
        Ok(())
    }

    pub fn parse_posts(&mut self) -> crate::error::Result<()> {
        log::debug!("Parse posts");
        let posts = Arc::new(Posts::load()?);

        let mut ctx = get_mut_context(snafu::location!())?;
        ctx.posts_value = Value::from_dyn_object(posts.clone());
        ctx.posts = posts;
        Ok(())
    }

    pub fn parse_pages(&mut self) -> crate::error::Result<()> {
        log::debug!("Parse pages");
        let mut pages = Pages::default();
        pages.load_base_pages()?;
        pages.load_custom_pages()?;

        let mut ctx = get_mut_context(snafu::location!())?;

        for page in pages.items.iter() {
            self.env
                .add_template_owned(page.path.clone(), page.content.clone())
                .context(MarkdownTemplateAddFailedSnafu { path: page.path.clone() })?;
        }

        let pages = Arc::new(pages);
        ctx.pages_value = Value::from_dyn_object(pages.clone());
        ctx.pages = pages;
        Ok(())
    }

    pub fn create_context(&self) -> crate::Result<Value> {
        let ctx = get_context(snafu::location!())?;
        Ok(context! {
            config => ctx.config,
            headers => ctx.headers,
            after_bodies => ctx.after_bodies,
            tags => ctx.tags,
            posts => ctx.posts_value,
            pages => ctx.pages_value,
            navs => ctx.config.navs,
        })
    }

    pub fn generate_posts(&mut self) -> crate::error::Result<()> {
        log::debug!("Generate posts");
        let ctx = get_context(snafu::location!())?;
        ctx.posts.render(self, ())?;

        Ok(())
    }

    pub fn generate_tags(&mut self) -> crate::error::Result<()> {
        log::debug!("Generate tags");
        let ctx = get_context(snafu::location!())?;

        let deployment_folder = ctx.config.blog_path.join(&ctx.config.deployment_folder);
        let file_path = deployment_folder.join("tags");
        let posts_page = ctx
            .pages
            .get(POSTS_HTML)
            .context(TemplateReferenceNotFoundSnafu { name: POSTS_HTML.to_string() })?;
        std::fs::create_dir_all(&file_path).context(FolderCreationFailedSnafu { path: file_path.clone() })?;

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

    pub fn move_assets(&mut self) -> crate::error::Result<()> {
        log::debug!("Generate assets");
        let ctx = get_context(snafu::location!())?;

        let deployment_folder = ctx
            .config
            .blog_path
            .join(&ctx.config.deployment_folder)
            .join("assets");
        Self::copy_dir_all(&ctx.statics_path, &deployment_folder).context(DirectoryCopyFailedSnafu {
            from: ctx.statics_path.clone(),
            to: deployment_folder.clone(),
        })
    }

    pub fn generate_pages(&mut self) -> crate::error::Result<()> {
        log::debug!("Generate pages");
        let ctx = get_context(snafu::location!())?;
        for page in ctx.pages.items.iter() {
            page.render(self, ())?;
        }

        Ok(())
    }

    pub fn write(&self, content: String, path: &PathBuf) -> crate::error::Result<()> {
        let mut file = File::create(path).context(FileCreationFailedSnafu { path })?;
        file.write_all(content.as_bytes())
            .context(WriteSnafu { path })
    }
}

pub fn create_engine() -> crate::error::Result<RenderEngine<'static>> {
    let mut engine = RenderEngine::new();
    engine.register_extension::<Codeblock>()?;
    engine.register_extension::<Quote>()?;
    engine.register_extension::<Gist>()?;
    engine.register_extension::<AlertBox>()?;
    engine.register_extension::<Info>()?;
    engine.register_extension::<SocialMediaShare>()?;
    engine.register_extension::<Reading>()?;
    engine.register_extension::<Projects>()?;
    engine.register_extension::<Contacts>()?;
    engine.register_extension::<Stats>()?;
    engine.register_extension::<Analytics>()?;
    Ok(engine)
}
