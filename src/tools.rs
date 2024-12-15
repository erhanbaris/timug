use std::path::{Path, PathBuf};

use anyhow::Context;
use pulldown_cmark::{Options, Parser};
use unidecode::unidecode;

use crate::error::TimugError;

pub fn get_file_name(path: &Path) -> anyhow::Result<String> {
    Ok(path
        .file_name()
        .context("Could not get filename")?
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

pub fn parse_yaml(content: &'_ str) -> Parser<'_> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    opts.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);

    pulldown_cmark::Parser::new_ext(content, opts)
}

pub fn url_encode(url: String) -> String {
    use urlencoding::encode;
    let url = unidecode(&url.to_lowercase()).replace([' ', '\r', '\n', '\t'], "-");
    let encoded = encode(&url);
    encoded.to_string()
}

#[derive(Debug)]
pub struct FrontMatterInfo<'a> {
    pub metadata: Option<&'a str>,
    pub content: &'a str,
}

pub fn parse_yaml_front_matter(content: &'_ str) -> FrontMatterInfo<'_> {
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
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_get_file_name() {
        let path = Path::new("/some/path/filename.txt");
        let file_name = get_file_name(path).unwrap();
        assert_eq!(file_name, "filename.txt");
    }

    #[test]
    fn test_get_path() {
        let path = Path::new("/some/path/filename.txt");
        let path_str = get_path(path).unwrap();
        assert_eq!(path_str, "/some/path/filename.txt");
    }

    #[test]
    fn test_get_files() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.md");
        File::create(&file_path).unwrap();

        let files = get_files(&dir.path().to_path_buf(), "md").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file_path);
    }

    #[test]
    fn test_get_file_content() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let content = get_file_content(&file_path).unwrap();
        assert_eq!(content, "Hello, world!\n");
    }

    #[test]
    fn test_parse_yaml() {
        let content = "---\nkey: value\n---\n# Heading\n";
        let parser = parse_yaml(content);
        let events: Vec<_> = parser.collect();
        assert!(events.len() > 0);
    }

    #[test]
    fn test_yaml_front_matter() {
        let content = "---\nkey: value\n---\n# Heading\n";
        let front_matter_info = parse_yaml_front_matter(content);
        assert_eq!(front_matter_info.metadata, Some("key: value"));
        assert_eq!(front_matter_info.content, "# Heading\n");
    }

    #[test]
    fn test_yaml_front_matter_no_metadata() {
        let content = "# Heading\n";
        let front_matter_info = parse_yaml_front_matter(content);
        assert_eq!(front_matter_info.metadata, None);
        assert_eq!(front_matter_info.content, "# Heading\n");
    }
}
