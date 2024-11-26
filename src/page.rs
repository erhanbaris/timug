use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use minijinja::{value::Object, Value};
use serde::{Deserialize, Serialize};

use crate::tools::{get_file_content, get_file_name, yaml_front_matter};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Page {
    #[serde(default)]
    pub title: String,

    #[serde(skip)]
    pub file_name: String,

    #[serde(skip)]
    pub path: String,

    #[serde(default)]
    pub slug: String,

    #[serde(default)]
    pub custom_page: bool,

    #[serde(default)]
    pub content: String,

    #[serde(default)]
    pub render: bool,

    #[serde(flatten)]
    other: HashMap<String, serde_yaml::value::Value>,
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

fn from(value: &serde_yaml::Value) -> minijinja::Value {
    match value {
        serde_yaml::Value::Null => minijinja::Value::UNDEFINED,
        serde_yaml::Value::Bool(val) => minijinja::Value::from(*val),
        serde_yaml::Value::Number(val) => minijinja::Value::from(val.as_f64()),
        serde_yaml::Value::String(val) => minijinja::Value::from(val),
        serde_yaml::Value::Sequence(vec) => {
            minijinja::Value::from(vec.iter().map(from).collect::<Vec<_>>())
        }
        serde_yaml::Value::Mapping(mapping) => minijinja::Value::from(
            mapping
                .into_iter()
                .map(|(key, value)| (key.as_str().unwrap_or_default().to_string(), from(value)))
                .collect::<HashMap<_, _>>(),
        ),
        serde_yaml::Value::Tagged(_) => minijinja::Value::UNDEFINED,
    }
}

impl Object for Page {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let key = key.as_str()?;

        if let Some(value) = self.other.get(key) {
            return Some(from(value));
        }

        match key {
            "title" => Some(Value::from(&self.title)),
            "slug" => Some(Value::from(&self.slug)),
            "path" => Some(Value::from(&self.path)),
            "render" => Some(Value::from(self.render)),
            "custom_page" => Some(Value::from(self.custom_page)),
            _ => None,
        }
    }
}
