use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use minijinja::{
    value::{Enumerator, Object},
    Value,
};
use serde::{Deserialize, Serialize};

use crate::tools::{get_file_content, get_file_name, yaml_front_matter};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Page {
    #[serde(default)]
    pub title: String,
    
    #[serde(skip)]
    pub file_name: String,
    
    #[serde(skip)]
    pub path: PathBuf,

    #[serde(default)]
    pub slug: String,

    #[serde(default)]
    pub custom_page: bool,

    #[serde(default)]
    pub content: String,
}

impl Page {
    pub fn load_from_path(path: &PathBuf) -> anyhow::Result<Self> {
        let content: String = get_file_content(path)?;
        Self::load_from_str(&content, path)
    }

    pub fn load_from_str(content: &str, path: &Path) -> anyhow::Result<Self> {
        let front_matter = yaml_front_matter(content);

        let mut page: Page = serde_yaml::from_str(front_matter.metadata.unwrap_or_default())
            .unwrap_or_else(|error| {
                panic!(
                    "Failed to parse page metadata information ({}, {:?})",
                    path.display(),
                    error
                )
            });

        page.content = front_matter.content.to_string();
        page.file_name = get_file_name(path)?;

        if page.title.is_empty() {
            page.title = page.file_name.clone();
        }

        if page.slug.is_empty() {
            page.slug = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
                .replace(".html", "");
        }
        Ok(page)
    }
}

impl Object for Page {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let key = key.as_str()?;
        match key {
            "title" => Some(Value::from(&self.title)),
            "slug" => Some(Value::from(&self.slug)),
            "custom_page" => Some(Value::from(self.custom_page)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pages {
    pub items: Vec<Page>,
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
