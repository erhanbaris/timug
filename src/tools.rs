use std::path::{Path, PathBuf};

use anyhow::Context;
use pulldown_cmark::{Event, Options, Tag, TagEnd};

use crate::error::TimugError;

pub fn get_file_name(path: &Path) -> anyhow::Result<String> {
    Ok(path
        .file_name()
        .context("Could not convert to string")?
        .to_str()
        .context("Could not convert to string")?
        .to_lowercase())
}

pub fn get_path(path: &Path) -> anyhow::Result<String> {
    Ok(path
        .to_str()
        .context("Could not convert to string")?
        .to_string())
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
            path.to_string_lossy().to_string(),
            error.to_string(),
        )),
    }
}

pub struct YamlData<'a> {
    pub metadata: String,
    pub body_items: Vec<Event<'a>>,
}

pub fn parse_yaml(content: &'_ str) -> YamlData<'_> {
    let mut heading_indexer = 0;
    let mut opts = Options::all();
    opts.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    opts.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);

    let parser = pulldown_cmark::Parser::new_ext(content, opts);
    let mut metacontent_started = false;
    let mut metadata = String::new();
    let mut body_items = Vec::new();

    for event in parser.into_iter() {
        match event {
            Event::Start(Tag::MetadataBlock(_)) => {
                metacontent_started = true;
                continue;
            }
            Event::Start(Tag::Heading {
                level,
                id: _,
                classes,
                attrs,
            }) => {
                let id = Some(format!("heading-{}", heading_indexer).into());
                heading_indexer += 1;
                body_items.push(Event::Start(Tag::Heading {
                    level,
                    id,
                    classes,
                    attrs,
                }));
                continue;
            }
            Event::End(TagEnd::MetadataBlock(_)) => {
                metacontent_started = false;
                continue;
            }
            _ => {}
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

#[derive(Debug)]
pub struct FrontMatterInfo<'a> {
    pub metadata: Option<&'a str>,
    pub content: &'a str,
}

pub fn yaml_front_matter(content: &'_ str) -> FrontMatterInfo<'_> {
    let mut front_matter_started = false;
    let mut front_matter_found = false;
    let mut front_matter_start_position = 0;
    let mut front_matter_end_position = 0;
    let mut content_start_position = 0;
    let mut start = 0;

    for (index, ch) in content.chars().enumerate() {
        if ch == '\n' {
            if content[start..index].trim() == "---" && front_matter_started {
                front_matter_end_position = start;
                content_start_position = index + 1;
                front_matter_found = true;
                break;
            } else if content[start..index].trim() == "---" && !front_matter_started {
                front_matter_started = true;
                front_matter_start_position = index + 1;
            }

            start = index;
        }
    }

    if front_matter_found {
        FrontMatterInfo {
            metadata: Some(&content[front_matter_start_position..front_matter_end_position]),
            content: &content[content_start_position..],
        }
    } else {
        FrontMatterInfo {
            metadata: None,
            content,
        }
    }
}
