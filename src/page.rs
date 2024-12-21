use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use console::style;
use minijinja::{context, value::Object, Value};
use serde::{Deserialize, Serialize};

use crate::{
    context::get_context,
    engine::{RenderEngine, Renderable},
    pages::PAGE_HTML,
    tools::{get_file_content, get_file_name, get_path, parse_yaml, parse_yaml_front_matter},
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum PageType {
    Html,

    #[default]
    Markdown,
}

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
    pub draft: bool,

    #[serde(skip)]
    pub page_type: PageType,

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

        page.page_type = match page.file_name.to_lowercase().ends_with(".html") || page.file_name.to_lowercase().ends_with(".htm") {
            true => PageType::Html,
            false => PageType::Markdown,
        };

        if page.slug.is_empty() {
            page.slug = match page.page_type {
                PageType::Html => page
                    .file_name
                    .to_lowercase()
                    .replace(".html", "")
                    .replace(".htm", ""),
                PageType::Markdown => page.file_name.to_lowercase().replace(".md", ""),
            };
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
        serde_yaml::Value::Sequence(vec) => minijinja::Value::from(vec.iter().map(from).collect::<Vec<_>>()),
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
            "draft" => Some(Value::from(self.draft)),
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
        if !ctx.draft && self.draft {
            return Ok(());
        }

        let source_path = PathBuf::from_str(self.path.as_str())?;
        let publish_path = ctx
            .config
            .blog_path
            .join(ctx.config.deployment_folder.clone());

        if let PageType::Html = self.page_type {
            let target_file_path = publish_path.join(&self.file_name);
            let template = engine.env.get_template(&self.path)?;
            engine.update_status(style("Rendering page").bold().cyan().to_string(), self.file_name.as_str());

            let context = engine.create_context();
            let content = template.render(context)?;

            engine.compress_and_write(content, &target_file_path)?;
            engine.update_status(style("Generated page").bold().green().to_string(), self.file_name.as_str());
            Ok(())
        } else {
            let target_file_path = publish_path.join(self.file_name.to_lowercase().replace("md", "html"));
            let context = engine.create_context();
            let mut content: String = get_file_content(&source_path)?;

            engine.update_status(style("Rendering page").bold().cyan().to_string(), get_file_name(&source_path)?.as_str());

            if content.contains("{%") {
                let content_tmp = engine.env.render_str(content.as_str(), &context)?;
                content = content_tmp;
            }

            let mut content_tmp = String::new();
            let parsed = parse_yaml(content.as_str());
            pulldown_cmark::html::push_html(&mut content_tmp, parsed);

            let template = engine.env.get_template(PAGE_HTML)?;
            let context = context! {
                ..context! {
                    title => self.title.clone(),
                    content => content_tmp,
                },
                ..context.clone()
            };

            let content: String = template.render(context)?;
            engine.compress_and_write(content, &target_file_path)?;
            engine.update_status(style("Generated page").bold().green().to_string(), get_file_name(&target_file_path)?.as_str());
            Ok(())
        }
    }
}
