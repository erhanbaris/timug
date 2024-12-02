use std::path::PathBuf;
use std::sync::OnceLock;

use colored::Colorize;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::config::TimugConfig;

const TEMPLATES_PATH: &str = "templates";
const POSTS_PATH: &str = "posts";
const PAGES_PATH: &str = "pages";
const STATICS_PATH: &str = "statics";
const CONFIG_FILE_NAME: &str = "timug.yaml";

static CONTEXT: OnceLock<RwLock<TimugContext>> = OnceLock::new();

#[derive(Debug, Default)]
pub struct TimugContext {
    pub config: TimugConfig,
    pub templates_path: PathBuf,
    pub posts_path: PathBuf,
    pub pages_path: PathBuf,
    pub statics_path: PathBuf,
    pub headers: Vec<&'static str>,
    pub after_bodies: Vec<&'static str>,
}

impl TimugContext {
    fn build(timug_path: Option<String>) -> Self {
        let current_path = std::env::current_dir().expect("Failed to get current directory");
        let config_path = match timug_path {
            Some(path) => path.into(),
            None => current_path.join(CONFIG_FILE_NAME),
        };

        println!("{}: {:?}", "Reading config file from".purple(), config_path);
        let config_content =
            std::fs::read_to_string(&config_path).expect("Failed to read config file");
        let config = serde_yaml::from_str(&config_content).expect("Failed to parse config file");

        let templates_path = Self::get_path(&config, TEMPLATES_PATH).join(config.theme.clone());
        let posts_path = Self::get_path(&config, POSTS_PATH);
        let pages_path = Self::get_path(&config, PAGES_PATH);
        let statics_path = Self::get_path(&config, STATICS_PATH);

        Self {
            config,
            templates_path,
            posts_path,
            pages_path,
            statics_path,
            headers: Default::default(),
            after_bodies: Default::default(),
        }
    }

    fn get_path(config: &TimugConfig, name: &str) -> PathBuf {
        config.blog_path.join(name)
    }
}

pub fn get_context() -> RwLockReadGuard<'static, TimugContext> {
    CONTEXT
        .get_or_init(|| TimugContext::build(None).into())
        .read()
        .unwrap()
}

pub fn get_mut_context() -> RwLockWriteGuard<'static, TimugContext> {
    CONTEXT
        .get_or_init(|| TimugContext::build(None).into())
        .write()
        .unwrap()
}
