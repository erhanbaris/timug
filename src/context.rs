use std::collections::HashSet;
use std::env::current_dir;
use std::fs::read_to_string;
use std::sync::OnceLock;
use std::{collections::HashMap, path::PathBuf};

use colored::Colorize;
use minijinja::Value;
use serde::de::DeserializeOwned;
use serde_yaml::from_str;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::pages::Pages;
use crate::post::Post;
use crate::posts::Posts;
use crate::{config::TimugConfig, template::TemplateConfig};

const TEMPLATES_PATH: &str = "templates";
const POSTS_PATH: &str = "posts";
const PAGES_PATH: &str = "pages";
const ASSETS_PATH: &str = "assets";
const CONFIG_FILE_NAME: &str = "timug.yaml";

static CONTEXT: OnceLock<RwLock<TimugContext>> = OnceLock::new();

#[derive(Debug, Default)]
pub struct TimugContext {
    pub config: TimugConfig,
    pub template_config: TemplateConfig,
    pub templates_path: PathBuf,
    pub posts_path: PathBuf,
    pub pages_path: PathBuf,
    pub statics_path: PathBuf,
    pub headers: Vec<&'static str>,
    pub after_bodies: Vec<&'static str>,
    pub posts_value: Value,
    pub pages_value: Value,
    pub pages: Pages,
    pub posts: Posts,
    pub tags: HashSet<String>,
    pub tag_posts: HashMap<String, Vec<Post>>,
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
            "Reading config file from".purple(),
            config_path.display()
        );
        let config_content = read_to_string(&config_path).expect("Failed to read config file");
        let config = from_str(&config_content).expect("Failed to parse config file");
        let templates_path = Self::get_path(&config, TEMPLATES_PATH).join(config.theme.clone());
        let template_config_path = templates_path.join("template.yaml");

        println!(
            "{}: {}",
            "Reading template config from".purple(),
            template_config_path.display()
        );
        let template_content =
            read_to_string(&template_config_path).expect("Failed to read config file");
        let template_config: TemplateConfig =
            from_str(&template_content).expect("Failed to parse template file");

        println!("{:?}", &template_config);
        let posts_path = Self::get_path(&config, POSTS_PATH);
        let pages_path = Self::get_path(&config, PAGES_PATH);
        let statics_path = Self::get_path(&config, ASSETS_PATH);

        Self {
            config,
            templates_path,
            template_config,
            posts_path,
            pages_path,
            statics_path,
            headers: Default::default(),
            after_bodies: Default::default(),
            posts_value: Default::default(),
            pages_value: Default::default(),
            tags: Default::default(),
            tag_posts: Default::default(),
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
