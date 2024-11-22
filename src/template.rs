use std::{fs::File, io::Write, path::PathBuf};

use anyhow::{Context, Ok};
use chrono::Datelike;
use colored::Colorize;
use minify_html::minify;
use minijinja::{context, path_loader, Environment, Value};

use crate::{
    context::TimugContext, pages::Page, posts::Posts, tools::{get_file_content, get_file_name, get_files}
};
const BASE_HTML: &str = "base.html";
pub const INDEX_HTML: &str = "index.html";
const FOOTER_HTML: &str = "footer.html";
const HEADER_HTML: &str = "header.html";
const POST_HTML: &str = "post.html";
pub const POSTS_HTML: &str = "posts.html";

pub struct RenderEngine<'a> {
    pub env: Environment<'a>,
    pub ctx: TimugContext,
    posts_value: Value,
}

impl<'a> RenderEngine<'a> {
    pub fn new(ctx: TimugContext) -> Self {
        let env = Environment::new();
        Self {
            env,
            ctx,
            posts_value: Value::default(),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.build_base_templates()?;
        self.build_filters();
        self.build_globals();
        self.build_functions();

        self.parse_posts()?;
        self.parse_pages()?;

        self.generate_pages()?;
        self.generate_posts()?;

        self.generate_base_pages()?;

        Ok(())
    }

    fn build_base_template(&mut self, name: &str) -> anyhow::Result<()> {
        let content = get_file_content(&self.ctx.templates_path.join(name))?;
        self.env.add_template_owned(name.to_string(), content)?;
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

    pub fn build_base_templates(&mut self) -> anyhow::Result<()> {
        let template_path = self.ctx.templates_path.clone();
        self.env.set_loader(path_loader(template_path));
        self.build_base_template(INDEX_HTML)?;
        self.build_base_template(BASE_HTML)?;
        self.build_base_template(FOOTER_HTML)?;
        self.build_base_template(HEADER_HTML)?;
        self.build_base_template(POST_HTML)?;
        self.build_base_template(POSTS_HTML)?;

        Ok(())
    }

    pub fn parse_posts(&mut self) -> anyhow::Result<()> {
        let posts = Posts::load(&self.ctx.posts_path)?;
        self.posts_value = Value::from_object(posts);
        Ok(())
    }

    pub fn generate_base_pages(&mut self) -> anyhow::Result<()> {
        self.generate_page(INDEX_HTML)?;
        self.generate_page(POSTS_HTML)?;
        Ok(())
    }

    pub fn parse_pages(&mut self) -> anyhow::Result<()> {
        let files = get_files(&self.ctx.pages_path, "html")?;

        for file in files {
            let _page = Page::load_from_path(&file)?;

            let content = get_file_content(&file)?;
            let file_name = get_file_name(file)?;
            self.env
                .add_template_owned(file_name.to_string(), content)?;

            println!("{}: {}", "Parsed".green(), &file_name);
            self.ctx.pages.push(file_name);
        }

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

            let file_path = deployment_folder
                .join(post.date.year().to_string())
                .join(post.date.month().to_string())
                .join(post.date.day().to_string());
            let file_name = file_path.join(format!("{}.html", post.slug));

            let template = self.env.get_template(POST_HTML)?;
            let content = template.render(context!(config => self.ctx.config, post => post))?;
            generate_path(&file_path)?;
            self.compress_and_write(content, &file_name)?;

            println!("{}: {}", "Generated".green(), file_name.display());
        }

        Ok(())
    }

    pub fn generate_pages(&mut self) -> anyhow::Result<()> {
        for page in self.ctx.pages.clone().iter() {
            self.generate_page(page)?;
            println!("{}: {}", "Generated".green(), &page);
        }

        Ok(())
    }

    pub fn generate_page(&mut self, page_name: &str) -> anyhow::Result<()> {
        let template = self.env.get_template(page_name)?;
        let content =
            template.render(context!(config => self.ctx.config, posts => self.posts_value))?;

        let file_path = self
            .ctx
            .config
            .blog_path
            .join(self.ctx.config.deployment_folder.clone());
        let file_name = file_path.join(page_name);

        self.compress_and_write(content, &file_name)?;
        println!("{}: {}", "Generated".green(), file_name.display());

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
