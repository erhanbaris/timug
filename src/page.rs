use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use minijinja::{value::Object, Value};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    consts::PAGE_HTML,
    context::get_context,
    document::{DocumentContext, DocumentType},
    engine::{RenderEngine, Renderable},
    error::{PathBufParseSnafu, YamlDeserializationFailedSnafu},
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
    pub draft: bool,

    #[serde(skip)]
    pub page_type: DocumentType,

    #[serde(default)]
    pub render: bool,

    #[serde(flatten)]
    other: HashMap<String, serde_yaml::value::Value>,
}

impl Page {
    pub fn load_from_path(path: &PathBuf) -> crate::Result<Self> {
        let content: String = get_file_content(path)?;
        Self::load_from_str(&content, path)
    }

    pub fn load_from_str(content: &str, path: &Path) -> crate::Result<Self> {
        let front_matter = parse_yaml_front_matter(content);
        let metadata = front_matter.metadata.unwrap_or_default();
        let mut page = serde_yaml::from_str::<'_, Page>(metadata).context(YamlDeserializationFailedSnafu { content: metadata })?;

        // Page details
        page.content = front_matter.content.to_string();
        page.file_name = get_file_name(path)?;
        page.path = get_path(path)?;

        if page.title.is_empty() {
            page.title = page.file_name.clone();
        }

        page.page_type = match page.file_name.to_lowercase().ends_with(".html") || page.file_name.to_lowercase().ends_with(".htm") {
            true => DocumentType::Html,
            false => DocumentType::Markdown,
        };

        if page.slug.is_empty() {
            page.slug = match page.page_type {
                DocumentType::Html => page
                    .file_name
                    .to_lowercase()
                    .replace(".html", "")
                    .replace(".htm", ""),
                DocumentType::Markdown => page.file_name.to_lowercase().replace(".md", ""),
            };
        }
        Ok(page)
    }

    fn inner_render(&self, engine: &RenderEngine<'_>) -> crate::Result<()> {
        let ctx = get_context(snafu::location!())?;
        if !ctx.draft && self.draft {
            return Ok(());
        }

        let source_path = PathBuf::from_str(self.path.as_str()).context(PathBufParseSnafu { path: self.path.clone() })?;
        let publish_path = ctx
            .config
            .blog_path
            .join(ctx.config.deployment_folder.clone());

        let render_ctx = DocumentContext {
            source_file_path: source_path.clone(),
            target_file_path: publish_path.join(self.file_name.replace(".md", ".html")),
            template: PAGE_HTML.to_string(),
            title: self.title.clone(),
            index: 0,
            data: (),
        };

        // Render the page
        self.page_type.render(engine, render_ctx)
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
    fn render(&self, engine: &RenderEngine<'_>, _: Self::Context) -> crate::Result<()> {
        if !self.render {
            return Ok(());
        }

        match self.inner_render(engine) {
            Err(err) => {
                log::error!("Failed to render page: {}", self.path);
                log::error!("{}", err);
                Err(err)
            }
            _ => Ok(()),
        }
    }
}
