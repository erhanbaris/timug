use std::borrow::Cow;
use std::env::current_dir;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, OnceLock};

use console::style;
use minijinja::Value;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::de::DeserializeOwned;
use serde_yaml::from_str;
use snafu::{OptionExt, ResultExt};

use crate::config::TimugConfig;
use crate::consts::{ASSETS_PATH, CONFIG_FILE_NAME, PAGES_PATH, POSTS_PATH, TEMPLATES_PATH};
use crate::error::{CanonicalizeSnafu, ContextNotInitializedSnafu, CurrentDirChangeSnafu, FileNotFoundSnafu, NoCurrentDirSnafu, YamlDeserializationFailedSnafu};
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
    pub after_bodies: Vec<Cow<'static, str>>,
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
    fn build(timug_path: Option<PathBuf>, draft: bool) -> crate::Result<Self> {
        let timug_path = match timug_path {
            Some(path) => match path.is_absolute() {
                true => path,
                false => {
                    let path = current_dir().context(NoCurrentDirSnafu)?.join(path);
                    path.canonicalize().context(CanonicalizeSnafu { path })?
                }
            },
            None => current_dir().context(NoCurrentDirSnafu)?,
        };

        let config_path = timug_path.join(CONFIG_FILE_NAME);
        log::debug!("{}: {}", style("Reading config file from").yellow().bold(), config_path.display());

        let content = read_to_string(&config_path).context(FileNotFoundSnafu { path: config_path })?;
        let mut config: TimugConfig = from_str(&content).context(YamlDeserializationFailedSnafu { content })?;

        if !config.blog_path.is_absolute() {
            let tmp_path = timug_path.join(config.blog_path);
            config.blog_path = tmp_path
                .canonicalize()
                .context(CanonicalizeSnafu { path: tmp_path })?;
            log::debug!("Blog path: {:?}", style(&config.blog_path).yellow());
        }

        if !config.deployment_folder.is_absolute() {
            let tmp_path = timug_path.join(config.deployment_folder);
            config.deployment_folder = tmp_path
                .canonicalize()
                .context(CanonicalizeSnafu { path: tmp_path })?;
            log::debug!("Deployment path: {:?}", style(&config.deployment_folder).yellow());
        }

        let templates_path = Self::get_path(&config, TEMPLATES_PATH).join(config.theme.clone());
        let template = Template::new(templates_path.clone())?;

        let posts_path = Self::get_path(&config, POSTS_PATH);
        let pages_path = Self::get_path(&config, PAGES_PATH);
        let statics_path = Self::get_path(&config, ASSETS_PATH);

        let current_dir = std::env::current_dir().context(NoCurrentDirSnafu)?;
        std::env::set_current_dir(&timug_path).context(CurrentDirChangeSnafu { path: timug_path })?;

        let git_folder: Option<PathBuf> = match std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .stdout(std::process::Stdio::piped())
            .output()
            .map(|output| String::from_utf8(output.stdout))
        {
            Ok(Ok(output)) => match PathBuf::from_str(output.trim()) {
                Ok(path) => {
                    log::debug!("Git path: {:?}", style(&path).yellow());
                    Some(path)
                }
                _ => None,
            },
            _ => None,
        };

        std::env::set_current_dir(&current_dir).context(CurrentDirChangeSnafu { path: current_dir })?;

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

pub fn build_context(config_path: Option<PathBuf>, draft: bool) -> crate::Result<()> {
    let context = TimugContext::build(config_path, draft)?;
    let _ = CONTEXT.set(context.into());
    Ok(())
}

pub fn get_context(loc: snafu::Location) -> crate::Result<RwLockReadGuard<'static, TimugContext>> {
    Ok(CONTEXT
        .get()
        .context(ContextNotInitializedSnafu { loc })?
        .read())
}

pub fn get_mut_context(loc: snafu::Location) -> crate::Result<RwLockWriteGuard<'static, TimugContext>> {
    Ok(CONTEXT
        .get()
        .context(ContextNotInitializedSnafu { loc })?
        .write())
}
