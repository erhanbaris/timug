use std::env::current_dir;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use anyhow::{anyhow, Context};
use console::style;
use minijinja::Value;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::de::DeserializeOwned;
use serde_yaml::from_str;

use crate::config::TimugConfig;
use crate::page::Page;
use crate::pages::Pages;
use crate::posts::Posts;
use crate::tags::Tags;
use crate::template::Template;

pub const TEMPLATES_PATH: &str = "templates";
pub const POSTS_PATH: &str = "posts";
pub const PAGES_PATH: &str = "pages";
pub const ASSETS_PATH: &str = "assets";
pub const CONFIG_FILE_NAME: &str = "timug.yaml";

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
    pub pages: Arc<Pages>,
    pub posts: Arc<Posts>,
    pub tags: Tags,
    pub template: Template,
    pub silent: bool,
}

impl TimugContext {
    fn build(timug_path: Option<PathBuf>, silent: bool) -> anyhow::Result<Self> {
        let current_path =
            current_dir().map_err(|_| anyhow::anyhow!("Failed to get current directory"))?;
        let config_path = match timug_path {
            Some(path) => path.join(CONFIG_FILE_NAME),
            None => current_path.join(CONFIG_FILE_NAME),
        };

        if !silent {
            println!(
                "{}: {}",
                style("Reading config file from").yellow().bold(),
                config_path.display()
            );
        }

        let config_content =
            read_to_string(&config_path).map_err(|_| anyhow!("Failed to read config file"))?;
        let config =
            from_str(&config_content).map_err(|_| anyhow!("Failed to parse config file"))?;
        let templates_path = Self::get_path(&config, TEMPLATES_PATH).join(config.theme.clone());
        let template = Template::new(templates_path.clone(), silent)?;

        let posts_path = Self::get_path(&config, POSTS_PATH);
        let pages_path = Self::get_path(&config, PAGES_PATH);
        let statics_path = Self::get_path(&config, ASSETS_PATH);

        Ok(Self {
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
            silent,
        })
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

    pub fn get_template_page(&self, name: &str) -> Option<Arc<Page>> {
        self.pages.get(name)
    }
}

pub fn build_context(silent: bool, config_path: Option<PathBuf>) -> anyhow::Result<()> {
    let context = TimugContext::build(config_path, silent)?;
    let _ = CONTEXT.set(context.into());
    Ok(())
}

pub fn get_context() -> RwLockReadGuard<'static, TimugContext> {
    CONTEXT
        .get()
        .context("Context not initialized")
        .unwrap()
        .read()
}

pub fn get_mut_context() -> RwLockWriteGuard<'static, TimugContext> {
    CONTEXT
        .get()
        .context("Context not initialized")
        .unwrap()
        .write()
}
