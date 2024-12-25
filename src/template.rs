use std::{fs::read_to_string, path::PathBuf};

use console::style;
use serde::{Deserialize, Serialize};
use serde_yaml::from_str;
use snafu::ResultExt;

use crate::{
    consts::TEMPLATE_FILE_NAME,
    error::{FileNotFoundSnafu, YamlDeserializationFailedSnafu},
    Result,
};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub name: String,

    #[serde(rename = "pre-process")]
    pub pre_process: Vec<String>,

    #[serde(rename = "process")]
    pub process: Vec<String>,

    #[serde(rename = "post-process")]
    pub post_process: Vec<String>,

    #[serde(default)]
    pub git_repo: Option<String>,

    pub version: String,
}

#[derive(Debug, Default)]
pub struct Template {
    pub config: TemplateConfig,
    pub path: PathBuf,
}

impl Template {
    pub fn new(path: PathBuf) -> Result<Self> {
        let config_path = path.join(TEMPLATE_FILE_NAME);
        log::debug!("{}: {}", style("Reading template file from").yellow().bold(), config_path.display());

        let content = read_to_string(&config_path).context(FileNotFoundSnafu { path: config_path })?;
        let config: TemplateConfig = from_str(&content).context(YamlDeserializationFailedSnafu { content })?;

        Ok(Self { config, path: path.clone() })
    }
}
