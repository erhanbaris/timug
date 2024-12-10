use console::style;
use parking_lot::{
    lock_api::{MappedRwLockReadGuard, RwLockReadGuard},
    RawRwLock, RwLock,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::{DateTime, Datelike, Utc};
use minijinja::{context, value::Object, Value};
use serde::{Deserialize, Serialize};

use crate::{
    engine::{RenderEngine, Renderable},
    error::TimugError,
    pages::POST_HTML,
    tools::{get_file_content, get_file_name, parse_yaml, parse_yaml_front_matter},
};
const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    inner: Arc<RwLock<InnerPost>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InnerPost {
    #[serde(default)]
    pub title: String,

    #[serde(default)]
    pub content: String,

    #[serde(default, with = "date_format")]
    pub date: DateTime<Utc>,

    #[serde(default)]
    pub slug: String,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub draft: bool,
}

pub mod date_format {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::DATE_FORMAT;

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(DATE_FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt =
            NaiveDateTime::parse_from_str(&s, DATE_FORMAT).map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}

impl Post {
    pub fn load_from_path(path: &PathBuf) -> Result<Self, TimugError> {
        let content: String = get_file_content(path)?;
        Self::load_from_str(&content, path)
    }

    pub fn load_from_str(content: &str, path: &Path) -> Result<Self, TimugError> {
        let front_matter = parse_yaml_front_matter(content);
        let mut post: InnerPost = serde_yaml::from_str(front_matter.metadata.unwrap_or_default())
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to parse post metadata information ({})",
                    path.display()
                )
            });

        if post.slug.is_empty() {
            post.slug = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
                .replace(".md", "");
        }

        post.content = front_matter.content.to_string();
        Ok(Post {
            inner: Arc::new(post.into()),
        })
    }

    pub fn title(&self) -> MappedRwLockReadGuard<'_, RawRwLock, String> {
        RwLockReadGuard::map(self.inner.read(), |item| &item.title)
    }

    pub fn content(&self) -> MappedRwLockReadGuard<'_, RawRwLock, String> {
        RwLockReadGuard::map(self.inner.read(), |item| &item.content)
    }

    pub fn set_content(&self, content: String) {
        self.inner.write().content = content;
    }

    pub fn slug(&self) -> MappedRwLockReadGuard<'_, RawRwLock, String> {
        RwLockReadGuard::map(self.inner.read(), |item| &item.slug)
    }

    pub fn draft(&self) -> bool {
        self.inner.read().draft
    }

    pub fn date(&self) -> DateTime<Utc> {
        self.inner.read().date
    }

    pub fn tags(&self) -> Vec<String> {
        self.inner.read().tags.clone()
    }
}

impl Object for Post {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let key = key.as_str()?;
        match key {
            "title" => Some(Value::from(self.title().as_str())),
            "content" => Some(Value::from(self.content().as_str())),
            "date" => Some(Value::from(self.date().format(DATE_FORMAT).to_string())),
            "slug" => Some(Value::from(self.slug().as_str())),
            "tags" => Some(Value::from(self.tags())),
            "draft" => Some(Value::from(self.draft())),
            _ => None,
        }
    }
}

pub struct PostContext {
    pub deployment_folder: PathBuf,
    pub index: usize,
}

impl Renderable for Post {
    type Context = PostContext;
    fn render(&self, engine: &RenderEngine<'_>, ctx: PostContext) -> anyhow::Result<()> {
        if self.draft() {
            return Ok(());
        }

        let context = engine.create_context();
        let date = self.date();
        let file_path = ctx
            .deployment_folder
            .join(date.year().to_string())
            .join(date.month().to_string())
            .join(date.day().to_string());
        let file_name = file_path.join(format!("{}.html", self.slug()));

        engine.update_status(style("Rendering post").bold().cyan().to_string(), get_file_name(&file_name)?.as_str());

        if self.content().contains("{%") {
            let content = engine.env.render_str(self.content().as_str(), &context)?;
            self.set_content(content);
        }

        let mut content = String::new();
        pulldown_cmark::html::push_html(&mut content, parse_yaml(self.content().as_str()));
        self.set_content(content);

        let template = engine.env.get_template(POST_HTML)?;

        let context = context! {
            ..context! {
                post => Value::from_object(self.clone()),
                index => ctx.index,
            },
            ..context.clone()
        };

        let content: String = template.render(context)?;
        std::fs::create_dir_all(file_path)?;
        engine.compress_and_write(content, &file_name)?;
        engine.update_status(style("Generated post").bold().green().to_string(), get_file_name(&file_name)?.as_str());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_from_path() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_post.md");
        let mut file = File::create(&file_path).unwrap();
        writeln!(
            file,
            "---\ntitle: Test Post\ndate: 2023-10-01 12:00:00\nslug: test-post\ntags: [\"rust\", \"test\"]\ndraft: false\n---\nThis is a test post."
        )
        .unwrap();

        let post = Post::load_from_path(&file_path).unwrap();
        assert_eq!(post.title().as_str(), "Test Post");
        assert_eq!(
            post.date().format(DATE_FORMAT).to_string(),
            "2023-10-01 12:00:00"
        );
        assert_eq!(post.slug().as_str(), "test-post");
        assert_eq!(post.tags(), vec!["rust", "test"]);
        assert_eq!(post.draft(), false);
        assert_eq!(post.content().as_str(), "This is a test post.\n");
    }

    #[test]
    fn test_load_from_str() {
        let content = "---\ntitle: Test Post\ndate: 2023-10-01 12:00:00\nslug: test-post\ntags: [\"rust\", \"test\"]\ndraft: false\n---\nThis is a test post.";
        let path = Path::new("test_post.md");
        let post = Post::load_from_str(content, path).unwrap();
        assert_eq!(post.title().as_str(), "Test Post");
        assert_eq!(
            post.date().format(DATE_FORMAT).to_string(),
            "2023-10-01 12:00:00"
        );
        assert_eq!(post.slug().as_str(), "test-post");
        assert_eq!(post.tags(), vec!["rust", "test"]);
        assert_eq!(post.draft(), false);
        assert_eq!(post.content().as_str(), "This is a test post.");
    }

    #[test]
    fn test_set_content() {
        let content = "---\ntitle: Test Post\ndate: 2023-10-01 12:00:00\nslug: test-post\ntags: [\"rust\", \"test\"]\ndraft: false\n---\nThis is a test post.";
        let path = Path::new("test_post.md");
        let post = Post::load_from_str(content, path).unwrap();
        post.set_content("Updated content.".to_string());
        assert_eq!(post.content().as_str(), "Updated content.");
    }

    #[test]
    fn test_object_get_value() {
        let content = "---\ntitle: Test Post\ndate: 2023-10-01 12:00:00\nslug: test-post\ntags: [\"rust\", \"test\"]\ndraft: false\n---\nThis is a test post.";
        let path = Path::new("test_post.md");
        let post = Arc::new(Post::load_from_str(content, path).unwrap());

        assert_eq!(
            post.get_value(&Value::from("title")).unwrap(),
            Value::from("Test Post")
        );
        assert_eq!(
            post.get_value(&Value::from("content")).unwrap(),
            Value::from("This is a test post.")
        );
        assert_eq!(
            post.get_value(&Value::from("date")).unwrap(),
            Value::from("2023-10-01 12:00:00")
        );
        assert_eq!(
            post.get_value(&Value::from("slug")).unwrap(),
            Value::from("test-post")
        );
        assert_eq!(
            post.get_value(&Value::from("tags")).unwrap(),
            Value::from(vec!["rust", "test"])
        );
        assert_eq!(
            post.get_value(&Value::from("draft")).unwrap(),
            Value::from(false)
        );
    }
}
