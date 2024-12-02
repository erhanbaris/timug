use std::{path::PathBuf, sync::Arc};

use colored::Colorize;
use minijinja::{
    value::{Enumerator, Object},
    Value,
};
use serde::{Deserialize, Serialize};

use crate::{post::Post, tools::get_files};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Posts {
    pub items: Arc<Vec<Post>>,
}

impl Posts {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let mut items = Vec::new();
        let files = get_files(path, "md")?;

        for file in files {
            let post = Post::load_from_path(&file)?;
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
