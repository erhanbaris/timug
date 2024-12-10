use std::sync::Arc;

use minijinja::{
    value::{Enumerator, Object},
    Value,
};
use serde::{Deserialize, Serialize};

use crate::{context::get_context, page::Page, tools::get_files};

pub const POST_HTML: &str = "post.html";
pub const POSTS_HTML: &str = "posts.html";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pages {
    pub items: Vec<Page>,
}

impl Pages {
    pub fn load_base_pages(&mut self) -> anyhow::Result<()> {
        let ctx = get_context();
        let html_files = get_files(&ctx.template.path, "html")?;

        for html_path in html_files.iter() {
            self.items.push(Page::load_from_path(html_path)?);
            // println!("{}: {}", "Parsed", html_path.display());
        }

        self.items
            .sort_unstable_by_key(|item| (item.title.clone(), item.slug.clone()));

        Ok(())
    }

    pub fn load_custom_pages(&mut self) -> anyhow::Result<()> {
        let ctx = get_context();
        let files = get_files(&ctx.pages_path, "html")?;

        for file in files {
            let mut page = Page::load_from_path(&file)?;
            page.render = true;
            self.items.push(page);
            // println!("{}: {}", "Parsed", file.display());
        }

        self.items
            .sort_unstable_by_key(|item| (item.order, item.title.clone()));

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Page> {
        self.items.iter().find(|page| page.file_name == name)
    }
}

impl Object for Pages {
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
