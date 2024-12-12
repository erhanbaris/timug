use std::env::current_dir;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::OnceLock;

use console::style;
use minijinja::Value;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::de::DeserializeOwned;
use serde_yaml::from_str;

use crate::config::TimugConfig;
use crate::pages::Pages;
use crate::posts::Posts;
use crate::tags::Tags;
use crate::template::Template;

const TEMPLATES_PATH: &str = "templates";
const POSTS_PATH: &str = "posts";
const PAGES_PATH: &str = "pages";
const ASSETS_PATH: &str = "assets";
const CONFIG_FILE_NAME: &str = "timug.yaml";

static CONTEXT: OnceLock<RwLock<TimugContext>> = OnceLock::new();

#[derive(Debug, Default)]
pub struct TimugContext {
    pub config: TimugConfig,
    pub posts_path: PathBuf,
    pub pages_path: PathBuf,
    pub statics_path: PathBuf,
    pub headers: Vec<&'static str>,
    pub after_bodies: Vec<&'static str>,
    pub posts_value: Value,
    pub pages_value: Value,
    pub pages: Pages,
    pub posts: Posts,
    pub tags: Tags,
    pub template: Template,
}

impl TimugContext {
    fn build(timug_path: Option<String>) -> Self {
        let current_path = current_dir().expect("Failed to get current directory");
        let config_path = match timug_path {
            Some(path) => path.into(),
            None => current_path.join(CONFIG_FILE_NAME),
        };

        println!(
            "{}: {}",
            style("Reading config file from").yellow().bold(),
            config_path.display()
        );
        let config_content = read_to_string(&config_path).expect("Failed to read config file");
        let config = from_str(&config_content).expect("Failed to parse config file");
        let templates_path = Self::get_path(&config, TEMPLATES_PATH).join(config.theme.clone());
        let template = Template::new(templates_path.clone()).unwrap();

        let posts_path = Self::get_path(&config, POSTS_PATH);
        let pages_path = Self::get_path(&config, PAGES_PATH);
        let statics_path = Self::get_path(&config, ASSETS_PATH);

        Self {
            config,
            template,
            posts_path,
            pages_path,
            statics_path,
            headers: Default::default(),
            after_bodies: Default::default(),
            posts_value: Default::default(),
            pages_value: Default::default(),
            tags: Default::default(),
            pages: Default::default(),
            posts: Default::default(),
        }
    }

    fn get_path(config: &TimugConfig, name: &str) -> PathBuf {
        config.blog_path.join(name)
    }

    pub fn get_config<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let value = self.config.other.get(key)?;
        if let Ok(config) = serde_yaml::from_value(value.to_owned()) {
            return Some(config);
        }
        None
    }

    pub fn get_template_page<'a>(&'a self, name: &str, default_html: &'a str) -> &'a str {
        if let Some(page) = self.pages.get(name) {
            &page.content
        } else {
            default_html
        }
    }
}

pub fn get_context() -> RwLockReadGuard<'static, TimugContext> {
    CONTEXT
        .get_or_init(|| TimugContext::build(None).into())
        .read()
}

pub fn get_mut_context() -> RwLockWriteGuard<'static, TimugContext> {
    CONTEXT
        .get_or_init(|| TimugContext::build(None).into())
        .write()
}
