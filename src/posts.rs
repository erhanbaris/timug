use std::{path::PathBuf, sync::Arc};

use chrono::Datelike;
use console::style;
use minijinja::{
    context,
    value::{Enumerator, Object},
    Value,
};
use serde::{Deserialize, Serialize};

use crate::{
    context::get_mut_context,
    engine::{RenderEngine, Renderable},
    pages::POST_HTML,
    post::Post,
    tools::{get_file_name, get_files, parse_yaml},
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Posts {
    pub posts: Vec<Arc<Post>>,
}

impl Posts {
    pub fn load() -> anyhow::Result<Self> {
        let mut ctx = get_mut_context();
        let mut posts = Vec::new();
        let files = get_files(&ctx.posts_path, "md")?;

        for file in files {
            let post = Arc::new(Post::load_from_path(&file)?);

            if post.draft() {
                continue;
            }

            for tag in post.tags() {
                ctx.tags.add(tag, post.clone());
            }

            posts.push(post);
            // println!("{}: {}", "Parsed", file.display());
        }

        posts.sort_by_key(|b| std::cmp::Reverse(b.date()));

        Ok(Self { posts })
    }
}

pub struct PostsContext {
    pub deployment_folder: PathBuf,
}

impl Renderable for Posts {
    type Context = PostsContext;
    fn render(&self, engine: &RenderEngine<'_>, ctx: PostsContext) -> anyhow::Result<()> {
        for (index, post) in self.posts.iter().enumerate() {
            if post.draft() {
                return Ok(());
            }

            let context = engine.create_context();
            let date = post.date();
            let file_path = ctx
                .deployment_folder
                .join(date.year().to_string())
                .join(date.month().to_string())
                .join(date.day().to_string());
            let file_name = file_path.join(format!("{}.html", post.slug()));

            engine.update_status(
                style("Rendering post").bold().cyan().to_string(),
                get_file_name(&file_name)?.as_str(),
            );

            if post.content().contains("{%") {
                let content = engine.env.render_str(post.content().as_str(), &context)?;
                post.set_content(content);
            }

            let mut content = String::new();
            pulldown_cmark::html::push_html(&mut content, parse_yaml(post.content().as_str()));
            post.set_content(content);

            let template = engine.env.get_template(POST_HTML)?;

            let context = context! {
                ..context! {
                    post => Value::from_dyn_object(post.clone()),
                    index => index,
                },
                ..context.clone()
            };

            let content: String = template.render(context)?;
            std::fs::create_dir_all(file_path)?;
            engine.compress_and_write(content, &file_name)?;
            engine.update_status(
                style("Generated post").bold().green().to_string(),
                get_file_name(&file_name)?.as_str(),
            );
        }

        Ok(())
    }
}

impl Object for Posts {
    fn repr(self: &Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Seq
    }

    fn get_value(self: &Arc<Self>, index: &Value) -> Option<Value> {
        let item = self.posts.get(index.as_usize()?)?;
        Some(Value::from_dyn_object(item.clone()))
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Seq(self.posts.len())
    }
}
