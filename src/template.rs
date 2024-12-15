use std::{fs::read_to_string, path::PathBuf};

use anyhow::{anyhow, Result};
use console::style;
use serde::{Deserialize, Serialize};
use serde_yaml::from_str;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub name: String,
    #[serde(rename = "pre-process")]
    pub pre_process: Vec<String>,
    #[serde(rename = "process")]
    pub process: Vec<String>,
    #[serde(rename = "post-process")]
    pub post_process: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Template {
    pub config: TemplateConfig,
    pub path: PathBuf,
}

impl Template {
    pub fn new(path: PathBuf, silent: bool) -> Result<Self> {
        let config_path = path.join("template.yaml");

        if !silent {
            println!(
                "{}: {}",
                style("Reading template file from").yellow().bold(),
                config_path.display()
            );
        }

        let content = read_to_string(&config_path)
            .map_err(|_| anyhow!("'{}' not found", config_path.display()))?;
        let config: TemplateConfig = from_str(&content)
            .map_err(|_| anyhow!("'{}' is not valid yaml format", config_path.display()))?;

        Ok(Self {
            config,
            path: path.clone(),
        })
    }
}
