use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use colored::Colorize;
use minijinja::{
    value::{Enumerator, Object},
    Value,
};
use serde::{Deserialize, Serialize};

use crate::{page::Page, tools::get_files};

const BASE_HTML: &str = "base.html";
const INDEX_HTML: &str = "index.html";
const FOOTER_HTML: &str = "footer.html";
const HEADER_HTML: &str = "header.html";
pub const POST_HTML: &str = "post.html";
const POSTS_HTML: &str = "posts.html";

const TEMPLATE_HTMLS: [&str; 6] = [
    BASE_HTML,
    INDEX_HTML,
    FOOTER_HTML,
    HEADER_HTML,
    POST_HTML,
    POSTS_HTML,
];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pages {
    pub items: Vec<Page>,
}

impl Pages {
    pub fn load_base_pages(&mut self, template_path: &Path) -> anyhow::Result<()> {
        self.build_base_template(template_path, INDEX_HTML, true)?;
        self.build_base_template(template_path, POSTS_HTML, true)?;

        TEMPLATE_HTMLS.iter().for_each(|name| {
            self.build_base_template(template_path, name, false)
                .unwrap();
        });

        self.items
            .sort_unstable_by_key(|item| (item.title.clone(), item.slug.clone()));

        Ok(())
    }

    fn build_base_template(
        &mut self,
        templates_path: &Path,
        name: &str,
        render: bool,
    ) -> anyhow::Result<()> {
        let file = templates_path.join(name);
        let mut page = Page::load_from_path(&file)?;
        page.render = page.render || render;
        self.items.push(page);
        println!("{}: {}", "Parsed".green(), file.display());
        Ok(())
    }

    pub fn load_custom_pages(&mut self, path: &PathBuf) -> anyhow::Result<()> {
        let files = get_files(path, "html")?;

        for file in files {
            let mut page = Page::load_from_path(&file)?;
            page.render = true;
            self.items.push(page);
            println!("{}: {}", "Parsed".green(), file.display());
        }

        self.items
            .sort_unstable_by_key(|item| (item.order, item.title.clone()));

        Ok(())
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
