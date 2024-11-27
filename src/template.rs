use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::Datelike;
use colored::Colorize;
use minify_html::minify;
use minijinja::{context, path_loader, Environment, Value};

use crate::{
    context::TimugContext,
    pages::{Pages, POST_HTML},
    posts::Posts,
};

pub struct RenderEngine<'a> {
    pub env: Environment<'a>,
    pub ctx: TimugContext,
    posts_value: Value,
    pages_value: Value,
}

impl<'a> RenderEngine<'a> {
    pub fn new(ctx: TimugContext) -> Self {
        let env = Environment::new();
        Self {
            env,
            ctx,
            posts_value: Value::default(),
            pages_value: Value::default(),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.env
            .set_loader(path_loader(self.ctx.templates_path.clone()));
        self.build_filters();
        self.build_globals();
        self.build_functions();

        self.parse_posts()?;
        self.parse_pages()?;

        self.generate_pages()?;
        self.generate_posts()?;

        self.move_statics()?;

        Ok(())
    }

    pub fn build_globals(&mut self) {
        let config = &self.ctx.config;
        self.env.add_global("author_name", &config.author);
        self.env.add_global("author_email", &config.email);
        self.env.add_global("site_url", &config.site_url);
        self.env.add_global("lang", &config.lang);
        self.env.add_global("description", &config.description);
        self.env.add_global("blog_name", &config.title);
    }

    pub fn parse_posts(&mut self) -> anyhow::Result<()> {
        let posts = Posts::load(&self.ctx.posts_path)?;
        self.posts_value = Value::from_object(posts);
        Ok(())
    }

    pub fn parse_pages(&mut self) -> anyhow::Result<()> {
        let mut pages = Pages::default();
        pages.load_base_pages(&self.ctx.templates_path)?;
        pages.load_custom_pages(&self.ctx.pages_path)?;

        for page in pages.items.iter() {
            self.env
                .add_template_owned(page.path.clone(), page.content.clone())?;
        }
        self.pages_value = Value::from_object(pages);
        Ok(())
    }

    pub fn generate_posts(&mut self) -> anyhow::Result<()> {
        let deployment_folder = self
            .ctx
            .config
            .blog_path
            .join(&self.ctx.config.deployment_folder);

        let mut created_paths = Vec::new();
        let mut generate_path = |path: &PathBuf| -> anyhow::Result<()> {
            if !created_paths.contains(path) {
                std::fs::create_dir_all(path)?;
                created_paths.push(path.clone());
            }

            Ok(())
        };

        let posts = self
            .posts_value
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Posts>())
            .context("'posts' is not a Posts type".to_string())?;

        for post in posts.items.iter() {
            if post.draft {
                continue;
            }

            let context = context!(config => self.ctx.config, post => post, posts => self.posts_value, pages => self.pages_value, active_page => "posts");

            let file_path = deployment_folder
                .join(post.date.year().to_string())
                .join(post.date.month().to_string())
                .join(post.date.day().to_string());
            let file_name = file_path.join(format!("{}.html", post.slug));

            let template = self.env.get_template(POST_HTML)?;

            if post.content.contains("{%") {
                let compiled = self.env.render_str(&post.content, &context)?;
                println!("{}: {}", "Compiled".yellow(), compiled);
            }

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
        Ok(Self::copy_dir_all(
            &self.ctx.statics_path,
            &self.ctx.config.deployment_folder,
        )?)
    }

    pub fn generate_pages(&mut self) -> anyhow::Result<()> {
        let pages = self
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
            let content = template.render(context!(config => self.ctx.config, posts => self.posts_value, pages => self.pages_value, active_page => page))?;

            let file_path = self
                .ctx
                .config
                .blog_path
                .join(self.ctx.config.deployment_folder.clone());
            let file_name = file_path.join(&page.file_name);

            self.compress_and_write(content, &file_name)?;
            println!("{}: {}", "Generated".green(), file_name.display());
        }

        Ok(())
    }

    fn compress_html(content: String) -> Vec<u8> {
        let cfg = minify_html::Cfg {
            minify_css: true,
            minify_js: true,
            do_not_minify_doctype: true,
            ..Default::default()
        };
        minify(content.as_bytes(), &cfg)
    }

    fn compress_and_write(&self, content: String, path: &PathBuf) -> anyhow::Result<()> {
        let mut file = File::create(path)?;
        let content = Self::compress_html(content);
        Ok(file.write_all(&content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_build_base_templates() {
        let temp_dir = tempdir().unwrap();
        let posts_path = temp_dir.path().join("posts");
        let templates_path = temp_dir.path().join("templates").join("default");

        std::fs::create_dir_all(&posts_path).unwrap();
        std::fs::create_dir_all(&templates_path).unwrap();

        std::fs::write(templates_path.join(BASE_HTML), "").unwrap();
        std::fs::write(templates_path.join(FOOTER_HTML), "").unwrap();
        std::fs::write(templates_path.join(HEADER_HTML), "").unwrap();
        std::fs::write(templates_path.join(POST_HTML), "").unwrap();

        std::fs::write(
            temp_dir.path().join("timug.yaml"),
            format!(
                r#"name: Test Blog
description: Test Blog
language: tr
theme: default
deployment_folder: public
blog_path: {}
author_name: Test Author
author_email: test@gmail.com
"#,
                temp_dir.path().display()
            ),
        )
        .unwrap();

        let mut env = Environment::new();
        let context = TimugContext::build(Some(
            temp_dir
                .path()
                .join("timug.yaml")
                .to_str()
                .unwrap()
                .to_string(),
        ));
        self.build_base_templates(&mut env, &context).unwrap();
    }

    #[test]
    fn test_parse_posts() {
        let temp_dir = tempdir().unwrap();
        let posts_path = temp_dir.path().join("posts");
        let templates_path = temp_dir.path().join("templates").join("default");

        std::fs::create_dir_all(&posts_path).unwrap();
        std::fs::create_dir_all(&templates_path).unwrap();

        std::fs::write(templates_path.join(BASE_HTML), "").unwrap();
        std::fs::write(templates_path.join(FOOTER_HTML), "").unwrap();
        std::fs::write(templates_path.join(HEADER_HTML), "").unwrap();
        std::fs::write(templates_path.join(POST_HTML), "").unwrap();

        std::fs::write(
            temp_dir.path().join("timug.yaml"),
            format!(
                r#"name: Test Blog
description: Test Blog
language: tr
theme: default
deployment_folder: public
blog_path: {}
author_name: Test Author
author_email: test@gmail.com
"#,
                temp_dir.path().display()
            ),
        )
        .unwrap();

        std::fs::write(
            posts_path.join("test.md"),
            "---
title: Test Post
---",
        )
        .unwrap();

        let mut context = TimugContext::build(Some(
            temp_dir
                .path()
                .join("timug.yaml")
                .to_str()
                .unwrap()
                .to_string(),
        ));
        let result = parse_posts(&mut context);
        assert!(result.is_ok());
        assert_eq!(context.posts.items.len(), 1);
        assert_eq!(context.posts.items[0].title, "Test Post");
    }

    #[test]
    fn test_generate_posts() {
        let temp_dir = tempdir().unwrap();
        let posts_path = temp_dir.path().join("posts");
        let templates_path = temp_dir.path().join("templates").join("default");

        std::fs::create_dir_all(&posts_path).unwrap();
        std::fs::create_dir_all(&templates_path).unwrap();

        std::fs::write(templates_path.join(BASE_HTML), "").unwrap();
        std::fs::write(templates_path.join(FOOTER_HTML), "").unwrap();
        std::fs::write(templates_path.join(HEADER_HTML), "").unwrap();
        std::fs::write(templates_path.join(POST_HTML), "").unwrap();

        std::fs::write(
            temp_dir.path().join("timug.yaml"),
            format!(
                r#"name: Test Blog
description: Test Blog
language: tr
theme: default
deployment_folder: public
blog_path: {}
author_name: Test Author
author_email: test@gmail.com
"#,
                temp_dir.path().display()
            ),
        )
        .unwrap();

        std::fs::write(
            posts_path.join("test.md"),
            "---
title: Test Post
---",
        )
        .unwrap();

        let mut context = TimugContext::build(Some(
            temp_dir
                .path()
                .join("timug.yaml")
                .to_str()
                .unwrap()
                .to_string(),
        ));

        let mut env = Environment::new();
        build_base_templates(&mut env, &context).unwrap();
        parse_posts(&mut context).unwrap();

        generate_posts().unwrap();

        let file_path = context
            .config
            .blog_path
            .join(context.config.deployment_folder.join("en"))
            .join("test.html");
        assert!(file_path.exists());
    }
}
