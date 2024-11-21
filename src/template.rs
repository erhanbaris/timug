use std::{fs::File, io::Write, path::PathBuf};

use chrono::Datelike;
use colored::Colorize;
use minify_html::minify;
use minijinja::{context, path_loader, Environment, Value};

use crate::{context::TimugContext, post::Post};
const BASE_HTML: &str = "base.html";
pub const INDEX_HTML: &str = "index.html";
const FOOTER_HTML: &str = "footer.html";
const HEADER_HTML: &str = "header.html";
const POST_HTML: &str = "post.html";
pub const POSTS_HTML: &str = "posts.html";

pub fn build_base_templates(
    env: &mut Environment<'_>,
    context: &TimugContext,
) -> anyhow::Result<()> {
    let index = context.get_template_file_content(INDEX_HTML)?;
    let base = context.get_template_file_content(BASE_HTML)?;
    let footer = context.get_template_file_content(FOOTER_HTML)?;
    let header = context.get_template_file_content(HEADER_HTML)?;
    let post = context.get_template_file_content(POST_HTML)?;
    let posts = context.get_template_file_content(POSTS_HTML)?;

    env.set_loader(path_loader(context.get_templates_path()));
    env.add_template_owned(INDEX_HTML, index)?;
    env.add_template_owned(HEADER_HTML, header)?;
    env.add_template_owned(FOOTER_HTML, footer)?;
    env.add_template_owned(BASE_HTML, base)?;
    env.add_template_owned(POST_HTML, post)?;
    env.add_template_owned(POSTS_HTML, posts)?;

    Ok(())
}

pub fn parse_posts(context: &mut TimugContext) -> anyhow::Result<()> {
    let paths = std::fs::read_dir(&context.posts_path)?;

    for path in paths.flatten() {
        if let Some(filename) = path.path().file_name().and_then(|name| name.to_str()) {
            if filename.to_lowercase().ends_with(".md") {
                let post = Post::load_from_path(context, filename)?;
                context.posts.items.push(post);
                println!("{}: {}", "Parsed".green(), filename);
            }
        } else {
            println!("Path name could not parsed ({})", path.path().display());
        }
    }

    context
        .posts
        .items
        .sort_unstable_by_key(|item| (item.date, item.slug.clone()));

    Ok(())
}

pub fn parse_pages(context: &mut TimugContext) -> anyhow::Result<()> {
    let paths = std::fs::read_dir(&context.pages_path)?;

    for path in paths.flatten() {
        if let Some(filename) = path.path().file_name().and_then(|name| name.to_str()) {
            if filename.to_lowercase().ends_with(".html") {
                let post = Post::load_from_path(context, filename)?;
                context.pages.items.push(post);
                println!("{}: {}", "Parsed".green(), filename);
            }
        } else {
            println!("Path name could not parsed ({})", path.path().display());
        }
    }

    context
        .pages
        .items
        .sort_unstable_by_key(|item| (item.date, item.slug.clone()));

    Ok(())
}

pub fn generate_posts(env: &mut Environment<'_>, context: &mut TimugContext) -> anyhow::Result<()> {
    let deployment_folder = context
        .config
        .blog_path
        .join(&context.config.deployment_folder);

    let mut created_paths = Vec::new();
    let mut generate_path = |path: &PathBuf| -> anyhow::Result<()> {
        if !created_paths.contains(path) {
            std::fs::create_dir_all(path)?;
            created_paths.push(path.clone());
        }

        Ok(())
    };

    for post in context.posts.items.iter() {
        if post.draft {
            continue;
        }

        let file_path = deployment_folder
            .join(&post.lang)
            .join(post.date.year().to_string())
            .join(post.date.month().to_string())
            .join(post.date.day().to_string());
        let file_name = file_path.join(format!("{}.html", post.slug));

        let template = env.get_template(POST_HTML)?;
        let content = template.render(context!(config => context.config, post => post))?;
        generate_path(&file_path)?;
        compress_and_write(content, &file_name)?;

        println!("{}: {}", "Generated".green(), file_name.display());
    }

    Ok(())
}

pub fn generate_pages(env: &mut Environment<'_>, context: &mut TimugContext) -> anyhow::Result<()> {
    let deployment_folder = context
        .config
        .blog_path
        .join(&context.config.deployment_folder);

    for page in context.pages.items.iter() {
        if page.draft {
            continue;
        }

        let file_name = deployment_folder.join(format!("{}.html", page.slug));

        let template = env.get_template(POST_HTML)?;
        let content = template.render(context!(config => context.config, page => page))?;
        compress_and_write(content, &file_name)?;

        println!("{}: {}", "Generated".green(), file_name.display());
    }

    Ok(())
}

pub fn generate_posts_page(
    env: &mut Environment<'_>,
    context: &mut TimugContext,
) -> anyhow::Result<()> {
    let template = env.get_template(POSTS_HTML)?;
    let content = template.render(context!(config => context.config, posts => Value::from_object(context.posts.clone())))?;

    let file_path = context
        .config
        .blog_path
        .join(context.config.deployment_folder.clone());
    let file_name = file_path.join(POSTS_HTML);

    compress_and_write(content, &file_name)?;
    println!("{}: {}", "Generated".green(), file_name.display());

    Ok(())
}

pub fn generate_page(env: &mut Environment<'_>, context: &mut TimugContext, page_name: &str) -> anyhow::Result<()> {
    let template = env.get_template(page_name)?;
    let content = template.render(context!(config => context.config, posts => Value::from_object(context.posts.clone())))?;

    let file_path = context
        .config
        .blog_path
        .join(context.config.deployment_folder.clone());
    let file_name = file_path.join(page_name);

    compress_and_write(content, &file_name)?;
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

fn compress_and_write(content: String, path: &PathBuf) -> anyhow::Result<()> {
    let mut file = File::create(path)?;
    let content = compress_html(content);
    Ok(file.write_all(&content)?)
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
        build_base_templates(&mut env, &context).unwrap();
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

        generate_posts(&mut env, &mut context).unwrap();

        let file_path = context
            .config
            .blog_path
            .join(context.config.deployment_folder.join("en"))
            .join("test.html");
        assert!(file_path.exists());
    }
}
