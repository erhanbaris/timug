use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::consts::{DEFAULT_DEPLOYMENT_FOLDER, DEFAULT_LANGUAGE, DEFAULT_THEME};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavItem {
    pub name: String,
    pub link: String,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TimugConfig {
    pub title: String,
    pub description: String,

    #[serde(default = "default_blog_path", rename = "blog-path")]
    pub blog_path: PathBuf,

    #[serde(default = "default_language")]
    pub lang: String,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_deployment_folder", rename = "deployment-folder")]
    pub deployment_folder: PathBuf,

    #[serde(default, rename = "site-url")]
    pub site_url: String,
    pub author: String,
    pub email: String,

    #[serde(default)]
    pub contacts: Vec<Contact>,

    #[serde(default)]
    pub navs: Vec<NavItem>,

    #[serde(flatten)]
    pub other: HashMap<String, serde_yaml::value::Value>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Contact {
    pub icon: String,
    pub name: String,
    pub address: String,
}

fn default_blog_path() -> PathBuf {
    std::env::current_dir().expect("Failed to get current directory")
}

fn default_theme() -> String {
    DEFAULT_THEME.to_string()
}

fn default_language() -> String {
    DEFAULT_LANGUAGE.to_string()
}

fn default_deployment_folder() -> PathBuf {
    default_blog_path().join(DEFAULT_DEPLOYMENT_FOLDER)
}
