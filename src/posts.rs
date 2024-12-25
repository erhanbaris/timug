use std::sync::Arc;

use chrono::Datelike;
use minijinja::{
    value::{Enumerator, Object},
    Value,
};
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};

use crate::{
    consts::POST_HTML,
    context::{get_context, get_mut_context},
    document::{DocumentContext, DocumentType},
    engine::{RenderEngine, Renderable},
    error::{FailedToAddTagSnafu, FolderCreationFailedSnafu},
    post::Post,
    tools::get_files,
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Posts {
    pub posts: Vec<Arc<Post>>,
}

impl Posts {
    pub fn load() -> crate::Result<Self> {
        let mut ctx = get_mut_context(snafu::location!())?;
        let mut posts = Vec::new();
        let files = get_files(&ctx.posts_path, "md")?;

        for file in files {
            let post = Arc::new(Post::load_from_path(&file)?);

            if !ctx.draft && post.draft() {
                continue;
            }

            for tag in post.tags() {
                ctx.tags
                    .add(tag.clone(), post.clone())
                    .context(FailedToAddTagSnafu { tag })?;
            }

            posts.push(post);
            // log::trace!("{}: {}", "Parsed", file.display());
        }

        posts.sort_by_key(|b| std::cmp::Reverse(b.date()));

        Ok(Self { posts })
    }
}

impl Renderable for Posts {
    type Context = ();
    fn render(&self, engine: &RenderEngine<'_>, _: Self::Context) -> crate::Result<()> {
        let general_ctx = get_context(snafu::location!())?;

        for (index, post) in self.posts.iter().enumerate() {
            if !general_ctx.draft && post.draft() {
                return Ok(());
            }

            let source_path = post.path();
            let date = post.date();
            let target_folder = general_ctx
                .config
                .deployment_folder
                .join(date.year().to_string())
                .join(date.month().to_string())
                .join(date.day().to_string());

            std::fs::create_dir_all(&target_folder).context(FolderCreationFailedSnafu { path: target_folder.clone() })?;

            let target_file_path = target_folder.join(format!("{}.html", post.slug()));
            let render_ctx = DocumentContext {
                source_file_path: source_path.clone(),
                target_file_path,
                template: POST_HTML.to_string(),
                title: post.title().clone(),
                index,
                data: Value::from_dyn_object(post.clone()),
            };

            // Render the page
            DocumentType::Markdown.render(engine, render_ctx)?;
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
