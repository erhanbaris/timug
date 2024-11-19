use std::path::PathBuf;

use crate::{config::TimugConfig, error::TimugError, post::Post};

const TEMPLATES_PATH: &str = "templates";
const POSTS_PATH: &str = "posts";
const DRAFTS_PATH: &str = "drafts";
const CONFIG_FILE_NAME: &str = "timug.yaml";

pub struct TimugContext {
    pub config: TimugConfig,
    pub templates_path: PathBuf,
    pub posts_path: PathBuf,
    pub drafts_path: PathBuf,
    pub posts: Vec<Post>,
}

impl TimugContext {
    pub fn build(timug_path: Option<String>) -> Self {
        let current_path = std::env::current_dir().expect("Failed to get current directory");
        let config_path = match timug_path {
            Some(path) => path.into(),
            None => current_path.join(CONFIG_FILE_NAME),
        };

        let config_content =
            std::fs::read_to_string(&config_path).expect("Failed to read config file");
        let config = serde_yaml::from_str(&config_content).expect("Failed to parse config file");

        let templates_path = Self::get_path(&config, TEMPLATES_PATH).join(config.theme.clone());
        let posts_path = Self::get_path(&config, POSTS_PATH);
        let drafts_path = Self::get_path(&config, DRAFTS_PATH);

        Self {
            config,
            templates_path,
            posts_path,
            drafts_path,
            posts: vec![],
        }
    }

    fn get_path(config: &TimugConfig, name: &str) -> PathBuf {
        config.blog_path.join(name)
    }

    pub fn get_template_file_path(&self, name: &str) -> PathBuf {
        self.templates_path.join(name)
    }

    pub fn get_post_file_path(&self, name: &str) -> PathBuf {
        self.posts_path.join(name)
    }

    pub fn get_templates_path(&self) -> PathBuf {
        self.templates_path.clone()
    }

    pub fn get_posts_path(&self) -> Result<String, TimugError> {
        Ok(self
            .config
            .blog_path
            .join(POSTS_PATH)
            .to_str()
            .ok_or_else(|| TimugError::PathCouldNotGenerated(POSTS_PATH.to_string()))?
            .to_string())
    }

    pub fn get_template_file_content(&self, name: &str) -> Result<String, TimugError> {
        match std::fs::read_to_string(self.get_template_file_path(name)) {
            Ok(content) => Ok(content),
            Err(error) => Err(TimugError::FileNotFound(
                name.to_string(),
                error.to_string(),
            )),
        }
    }

    pub fn get_blog_file_content(&self, name: &str) -> Result<String, TimugError> {
        match std::fs::read_to_string(self.get_post_file_path(name)) {
            Ok(content) => Ok(content),
            Err(error) => Err(TimugError::FileNotFound(
                name.to_string(),
                error.to_string(),
            )),
        }
    }
}
