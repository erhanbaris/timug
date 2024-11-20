use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TimugConfig {
    pub name: String,
    pub description: String,

    #[serde(default = "default_blog_path")]
    pub blog_path: PathBuf,

    #[serde(default = "default_language")]
    pub lang: String,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_deployment_folder")]
    pub deployment_folder: PathBuf,

    #[serde(default)]
    pub site_url: String,
    pub author_name: String,
    pub author_email: String,
}

fn default_blog_path() -> PathBuf {
    std::env::current_dir().expect("Failed to get current directory")
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

fn default_deployment_folder() -> PathBuf {
    default_blog_path().join("public")
}
