use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use console::style;
use minijinja::{value::Object, Value};
use serde::{Deserialize, Serialize};

use crate::{
    context::get_context,
    engine::{RenderEngine, Renderable},
    tools::{get_file_content, get_file_name, get_path, parse_yaml_front_matter},
};

#[derive(Debug, Default, Serialize, Deserialize)]
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
    pub content: String,

    #[serde(default)]
    pub render: bool,

    #[serde(default)]
    pub order: i32,

    #[serde(default, rename = "no-listing")]
    pub no_listing: bool,

    #[serde(flatten)]
    other: HashMap<String, serde_yaml::value::Value>,
}

impl Page {
    pub fn load_from_path(path: &PathBuf) -> anyhow::Result<Self> {
        let content: String = get_file_content(path)?;
        Self::load_from_str(&content, path)
    }

    pub fn load_from_str(content: &str, path: &Path) -> anyhow::Result<Self> {
        let front_matter = parse_yaml_front_matter(content);
        let metadata = front_matter.metadata.unwrap_or_default();
        let mut page = serde_yaml::from_str::<'_, Page>(metadata)?;

        // Page details
        page.content = front_matter.content.to_string();
        page.file_name = get_file_name(path)?;
        page.path = get_path(path)?;

        if page.title.is_empty() {
            page.title = page.file_name.clone();
        }

        if page.slug.is_empty() {
            page.slug = page.file_name.replace(".html", "");
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
            "no_listing" => Some(Value::from(self.no_listing)),
            _ => None,
        }
    }
}

impl Renderable for Page {
    type Context = ();
    fn render(&self, engine: &RenderEngine<'_>, _: Self::Context) -> anyhow::Result<()> {
        if !self.render {
            return Ok(());
        }

        let ctx = get_context();

        let template = engine.env.get_template(&self.path)?;
        engine.update_status(
            style("Rendering page").bold().cyan().to_string(),
            self.file_name.as_str(),
        );

        let context = engine.create_context();
        let content = template.render(context)?;

        let file_path = ctx
            .config
            .blog_path
            .join(ctx.config.deployment_folder.clone());
        let file_path = file_path.join(&self.file_name);

        engine.compress_and_write(content, &file_path)?;
        engine.update_status(
            style("Generated page").bold().green().to_string(),
            self.file_name.as_str(),
        );
        Ok(())
    }
}
