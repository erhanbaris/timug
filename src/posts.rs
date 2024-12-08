use std::sync::Arc;

use colored::Colorize;
use minijinja::{
    value::{Enumerator, Object},
    Value,
};
use serde::{Deserialize, Serialize};

use crate::{context::get_mut_context, post::Post, tools::get_files};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Posts {
    pub items: Arc<Vec<Post>>,
}

impl Posts {
    pub fn load() -> anyhow::Result<Self> {
        let mut ctx = get_mut_context();
        let mut items = Vec::new();
        let files = get_files(&ctx.posts_path, "md")?;

        for file in files {
            let post = Post::load_from_path(&file)?;

            for tag in post.tags() {
                if let Some(tag_data) = ctx.tag_posts.get_mut(&tag) {
                    tag_data.push(post.clone());
                } else {
                    ctx.tag_posts.insert(tag.clone(), vec![post.clone()]);
                }
                ctx.tags.insert(tag);
            }

            items.push(post);
            println!("{}: {}", "Parsed".green(), file.display());
        }

        items.sort_by_key(|b| std::cmp::Reverse(b.date()));

        Ok(Self {
            items: Arc::new(items),
        })
    }
}

impl Object for Posts {
    fn repr(self: &Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Seq
    }

    fn get_value(self: &Arc<Self>, index: &Value) -> Option<Value> {
        let item = self.items.get(index.as_usize()?)?;
        Some(Value::from_object(item.clone()))
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Seq(self.items.len())
    }
}
