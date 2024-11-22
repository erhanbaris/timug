use std::path::PathBuf;

use anyhow::Context;
use pulldown_cmark::{Event, Options, Tag, TagEnd};

use crate::error::TimugError;

pub fn get_file_name(path: PathBuf) -> anyhow::Result<String> {
    Ok(path
        .file_name()
        .context("Could not convert to string")?
        .to_str()
        .context("Could not convert to string")?
        .to_lowercase())
}

pub fn get_files(path: &PathBuf, extension: &str) -> anyhow::Result<Vec<PathBuf>> {
    let paths = std::fs::read_dir(path)?
        .flatten()
        .map(|dir_entry| dir_entry.path())
        .filter_map(|path| {
            if path
                .extension()
                .map_or(false, |ext| ext.to_ascii_lowercase() == extension)
            {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    Ok(paths)
}

pub fn get_file_content(path: &PathBuf) -> Result<String, TimugError> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(error) => Err(TimugError::FileNotFound(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            error.to_string(),
        )),
    }
}

pub struct YamlData<'a> {
    pub metadata: String,
    pub body_items: Vec<Event<'a>>,
}

pub fn parse_yaml(content: &'_ str) -> YamlData<'_> {
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

    YamlData {
        metadata,
        body_items,
    }
}
