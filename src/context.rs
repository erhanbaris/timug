use std::env::current_dir;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, OnceLock};

use anyhow::{anyhow, Context};
use console::style;
use minijinja::Value;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::de::DeserializeOwned;
use serde_yaml::from_str;

use crate::config::TimugConfig;
use crate::consts::{ASSETS_PATH, CONFIG_FILE_NAME, PAGES_PATH, POSTS_PATH, TEMPLATES_PATH};
use crate::page::Page;
use crate::pages::Pages;
use crate::posts::Posts;
use crate::tags::Tags;
use crate::template::Template;

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
    pub git_folder: Option<PathBuf>,
    pub draft: bool,
}

impl TimugContext {
    fn build(timug_path: Option<PathBuf>, silent: bool, draft: bool) -> anyhow::Result<Self> {
        let timug_path = match timug_path {
            Some(path) => match path.is_absolute() {
                true => path,
                false => current_dir()
                    .map_err(|_| anyhow!("Failed to get current directory"))?
                    .join(path)
                    .canonicalize()?,
            },
            None => current_dir().map_err(|_| anyhow!("Failed to get current directory"))?,
        };

        let config_path = timug_path.join(CONFIG_FILE_NAME);
        if !silent {
            println!("{}: {}", style("Reading config file from").yellow().bold(), config_path.display());
        }

        let config_content = read_to_string(&config_path).map_err(|_| anyhow!("Failed to read config file"))?;
        let mut config: TimugConfig = from_str(&config_content).map_err(|_| anyhow!("Failed to parse config file"))?;

        if !config.blog_path.is_absolute() {
            config.blog_path = timug_path.join(config.blog_path).canonicalize()?;
            println!("Blog path: {:?}", style(&config.blog_path).yellow());
        }

        if !config.deployment_folder.is_absolute() {
            config.deployment_folder = timug_path.join(config.deployment_folder).canonicalize()?;
            println!("Deployment path: {:?}", style(&config.deployment_folder).yellow());
        }

        let templates_path = Self::get_path(&config, TEMPLATES_PATH).join(config.theme.clone());
        let template = Template::new(templates_path.clone(), silent)?;

        let posts_path = Self::get_path(&config, POSTS_PATH);
        let pages_path = Self::get_path(&config, PAGES_PATH);
        let statics_path = Self::get_path(&config, ASSETS_PATH);

        let current_dir = std::env::current_dir()?;
        std::env::set_current_dir(&timug_path)?;

        let git_folder: Option<PathBuf> = match std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .stdout(std::process::Stdio::piped())
            .output()
            .map(|output| String::from_utf8(output.stdout))
        {
            Ok(Ok(output)) => match PathBuf::from_str(output.trim()) {
                Ok(path) => {
                    println!("Git path: {:?}", style(&path).yellow());
                    Some(path)
                }
                _ => None,
            },
            _ => None,
        };

        std::env::set_current_dir(&current_dir)?;

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
            git_folder,
            draft,
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

pub fn build_context(silent: bool, config_path: Option<PathBuf>, draft: bool) -> anyhow::Result<()> {
    let context = TimugContext::build(config_path, silent, draft)?;
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
