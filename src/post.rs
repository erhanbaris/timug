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

    #[serde(default, with = "my_date_format")]
    pub date: Option<DateTime<Utc>>,

    #[serde(default)]
    pub slug: String,

    #[serde(default)]
    pub tags: Vec<String>,
}

mod my_date_format {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

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
    pub fn load(context: &TimugContext, path: &str) -> Result<Self, TimugError> {
        let content: String = context.get_blog_file_content(path)?;
        let mut opts = Options::all();
        opts.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        opts.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);

        let parser = pulldown_cmark::Parser::new_ext(&content, opts);
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

        let mut post: Post = serde_yaml::from_str(&metadata).expect("Failed to parse config file");

        pulldown_cmark::html::push_html(&mut post.content, body_items.into_iter());
        Ok(post)
    }
}
