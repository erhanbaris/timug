use std::sync::Arc;

use minijinja::{
    value::{Enumerator, Object},
    Value,
};
use serde::{Deserialize, Serialize};

use crate::{context::get_context, page::Page, tools::get_files};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pages {
    pub items: Vec<Arc<Page>>,
}

impl Pages {
    pub fn load_base_pages(&mut self) -> crate::Result<()> {
        let ctx = get_context(snafu::location!())?;
        let html_files = get_files(&ctx.template.path, "html")?;

        for html_path in html_files.iter() {
            let page = Page::load_from_path(html_path)?;
            self.items.push(page.into());
            // log::trace!("{}: {}", "Parsed", html_path.display());
        }

        self.items
            .sort_unstable_by_key(|item| (item.title.clone(), item.slug.clone()));

        Ok(())
    }

    pub fn load_custom_pages(&mut self) -> crate::Result<()> {
        let ctx = get_context(snafu::location!())?;
        let html_files = get_files(&ctx.pages_path, "html")?;
        let md_files = get_files(&ctx.pages_path, "md")?;

        for file in html_files {
            let mut page = Page::load_from_path(&file)?;
            page.render = true;
            self.items.push(page.into());
        }

        for file in md_files {
            let mut page = Page::load_from_path(&file)?;
            page.render = true;
            self.items.push(page.into());
        }

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Arc<Page>> {
        self.items
            .iter()
            .find(|page| page.file_name == name)
            .cloned()
    }
}

impl Object for Pages {
    fn repr(self: &Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Seq
    }
    fn get_value(self: &Arc<Self>, index: &Value) -> Option<Value> {
        let item = self.items.get(index.as_usize()?)?;
        Some(Value::from_dyn_object(item.clone()))
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Seq(self.items.len())
    }
}
