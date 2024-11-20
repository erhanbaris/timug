use chrono::{DateTime, Utc};
use pulldown_cmark::{Event, Options, Tag, TagEnd};
use serde::{Deserialize, Serialize};

use crate::{context::TimugContext, error::TimugError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    #[serde(default)]
    pub title: String,

    #[serde(default)]
    pub content: String,

    #[serde(default, with = "date_format")]
    pub date: Option<DateTime<Utc>>,

    #[serde(default)]
    pub slug: String,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub author_name: String,

    #[serde(default)]
    pub author_email: String,

    #[serde(default)]
    pub lang: String,
}

pub mod date_format {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.unwrap_or_default().format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)))
    }
}

impl Post {
    pub fn load_from_path(context: &TimugContext, path: &str) -> Result<Self, TimugError> {
        let content: String = context.get_blog_file_content(path)?;
        Self::load_from_str(context, &content, path)
    }

    pub fn load_from_str(
        context: &TimugContext,
        content: &str,
        path: &str,
    ) -> Result<Self, TimugError> {
        let mut opts = Options::all();
        opts.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        opts.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);

        let parser = pulldown_cmark::Parser::new_ext(content, opts);
        let mut metacontent_started = false;
        let mut metadata = String::new();
        let mut body_items = Vec::new();

        for event in parser {
            if let Event::Start(Tag::MetadataBlock(_)) = event {
                metacontent_started = true;
                continue;
            }

            if let Event::End(TagEnd::MetadataBlock(_)) = event {
                metacontent_started = false;
                continue;
            }

            if metacontent_started {
                if let Event::Text(text) = event {
                    metadata.push_str(&text);
                }
            } else {
                body_items.push(event);
            }
        }

        let mut post: Post = serde_yaml::from_str(&metadata)
            .unwrap_or_else(|_| panic!("Failed to parse post metadata information ({})", path));

        if post.author_name.is_empty() {
            post.author_name = context.config.author_name.clone();
        }

        if post.author_email.is_empty() {
            post.author_email = context.config.author_email.clone();
        }

        if post.lang.is_empty() {
            post.lang = context.config.lang.clone();
        }

        if post.slug.is_empty() {
            post.slug = path.to_lowercase().replace(".md", "");
        }

        pulldown_cmark::html::push_html(&mut post.content, body_items.into_iter());
        Ok(post)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_context() -> TimugContext {
        TimugContext {
            config: crate::config::TimugConfig {
                author_name: "Default Author".to_string(),
                author_email: "author@example.com".to_string(),
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
        assert_eq!(post.date.unwrap().to_string(), "2023-10-01 12:00:00 UTC");
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
        assert!(post.date.is_none());
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
        assert!(post.date.is_none());
        assert!(post.tags.is_empty());
        assert_eq!(post.content, "<p>This is the content of the post.</p>\n");
        assert_eq!(post.author_name, "Default Author");
        assert_eq!(post.author_email, "author@example.com");
        assert_eq!(post.lang, "en");
        assert_eq!(post.slug, "test_post");
    }
}
