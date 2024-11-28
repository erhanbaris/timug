use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use minijinja::{value::Object, Value};
use serde::{Deserialize, Serialize};

use crate::{
    error::TimugError,
    tools::{get_file_content, parse_yaml},
};
const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
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
        let yaml_data = parse_yaml(content);
        let mut post: Post = serde_yaml::from_str(&yaml_data.metadata).unwrap_or_else(|_| {
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

        pulldown_cmark::html::push_html(&mut post.content, yaml_data.body_items.into_iter());
        Ok(post)
    }
}

impl Object for Post {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let key = key.as_str()?;
        match key {
            "title" => Some(Value::from(&self.title)),
            "content" => Some(Value::from(&self.content)),
            "date" => Some(Value::from(self.date.format(DATE_FORMAT).to_string())),
            "slug" => Some(Value::from(&self.slug)),
            "tags" => Some(Value::from(self.tags.clone())),
            "draft" => Some(Value::from(self.draft)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_context() -> TimugContext {
        TimugContext {
            config: crate::config::TimugConfig {
                author: "Default Author".to_string(),
                email: "author@example.com".to_string(),
                lang: "en".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_load_post_with_metadata() {
        let context = setup_context();
        let path = "test_post.md";
        let content = r#"---
title: Test Post
date: 2023-10-01 12:00:00
tags:
    - rust
    - programming
---
This is the content of the post.
"#;

        let post = Post::load_from_str(&context, content, path).expect("Failed to load post");

        assert_eq!(post.title, "Test Post");
        assert_eq!(post.date.to_string(), "2023-10-01 12:00:00 UTC");
        assert_eq!(post.tags, vec!["rust", "programming"]);
        assert_eq!(post.content, "<p>This is the content of the post.</p>\n");
    }

    #[test]
    fn test_load_post_without_metadata() {
        let context = setup_context();
        let path = "test_post.md";
        let content = "This is the content of the post.";

        let post = Post::load_from_str(&context, content, path).expect("Failed to load post");

        assert_eq!(post.title, "");
        assert!(post.tags.is_empty());
        assert_eq!(post.content, "<p>This is the content of the post.</p>\n");
        assert_eq!(post.author_name, "Default Author");
        assert_eq!(post.author_email, "author@example.com");
        assert_eq!(post.lang, "en");
        assert_eq!(post.slug, "test_post");
    }

    #[test]
    fn test_load_post_with_partial_metadata() {
        let context = setup_context();
        let path = "test_post.md";
        let content = r#"---
title: Test Post
---
This is the content of the post.
"#;

        let post = Post::load_from_str(&context, content, path).expect("Failed to load post");

        assert_eq!(post.title, "Test Post");
        assert!(post.tags.is_empty());
        assert_eq!(post.content, "<p>This is the content of the post.</p>\n");
        assert_eq!(post.author_name, "Default Author");
        assert_eq!(post.author_email, "author@example.com");
        assert_eq!(post.lang, "en");
        assert_eq!(post.slug, "test_post");
    }
}
