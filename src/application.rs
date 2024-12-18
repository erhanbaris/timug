use crate::config::TimugConfig;
use crate::consts::{ExampleAssets, TemplateAssets, ASSETS_PATH, CONFIG_FILE_NAME, DEFAULT_DEPLOYMENT_FOLDER, DEFAULT_LANGUAGE, DEFAULT_THEME, PAGES_PATH, POSTS_PATH, ROCKET, TEMPLATES_PATH};
use crate::context::build_context;
use crate::server::start_webserver;
use crate::tools::inner_deploy_pages;

use std::{
    fs::{create_dir, create_dir_all, File},
    io::Write,
    path::PathBuf,
};

use console::{style, Term};

fn initialize(silent: bool, path: Option<PathBuf>, draft: bool) -> anyhow::Result<()> {
    build_context(silent, path, draft)?;
    Ok(())
}

pub fn start_create_new_project(silent: bool, project_path: PathBuf) -> anyhow::Result<()> {
    let terminal = Term::stdout();
    let mut config = TimugConfig::default();

    // Get full path
    let path = std::path::absolute(project_path.clone())?;

    config.lang = DEFAULT_LANGUAGE.to_string();
    config.theme = DEFAULT_THEME.to_string();
    config.deployment_folder = path.join(DEFAULT_DEPLOYMENT_FOLDER);
    config.blog_path = path.clone();

    let config_path = config.blog_path.join(CONFIG_FILE_NAME);
    if std::fs::exists(&config_path).unwrap_or_default() {
        // Config is already exists
        if silent {
            return Err(anyhow::anyhow!(""));
        }

        terminal.write_line(&format!("\"{}\" already created.\r\n", config_path.display()))?;
        terminal.write_line("Do you want to overwrite it?")?;
        terminal.write_line("[y/n] ")?;
        let answer = terminal.read_char();

        match answer {
            Ok('Y') | Ok('y') => (),
            _ => return Err(anyhow::anyhow!("Canceled by the user")),
        }
    }

    // Create path, if need it
    let _ = create_dir(&config.blog_path);
    let mut config_file = File::create(&config_path)?;

    let config_string = serde_yaml::to_string(&config)?;

    config_file.write_all(config_string.as_bytes())?;

    let template_path = config.blog_path.join(TEMPLATES_PATH).join(DEFAULT_THEME);
    let posts_path = config.blog_path.join(POSTS_PATH);

    let _ = create_dir_all(&template_path);
    let _ = create_dir(&posts_path);
    let _ = create_dir(config.blog_path.join(PAGES_PATH));
    let _ = create_dir(config.blog_path.join(ASSETS_PATH));
    let _ = create_dir(config.blog_path.join(DEFAULT_DEPLOYMENT_FOLDER));

    for file_name in TemplateAssets::iter() {
        if let Some(file_content) = TemplateAssets::get(&file_name) {
            let mut file = File::create(template_path.join(file_name.to_string()))?;
            file.write_all(&file_content.data)?;
        }
    }

    for file_name in ExampleAssets::iter() {
        if let Some(file_content) = ExampleAssets::get(&file_name) {
            let mut file = File::create(posts_path.join(file_name.to_string()))?;
            file.write_all(&file_content.data)?;
        }
    }

    if !silent {
        terminal.write_line(&format!("{} New Timug project created.", ROCKET))?;
        terminal.write_line("Execute following command to see your new site.")?;
        terminal.write_line(&format!("{} {}", style("timug deploy {}").yellow(), path.display()))?;
        terminal.write_line(&format!("{} {}", style("timug start").yellow(), path.display()))?;
    }

    Ok(())
}

pub fn start_server(path: Option<PathBuf>, port: Option<u16>, draft: bool) -> anyhow::Result<()> {
    initialize(true, path.clone(), draft)?;
    inner_deploy_pages(true)?;
    start_webserver(port)?;
    Ok(())
}

pub fn start_deploy_pages(silent: bool, path: Option<PathBuf>, draft: bool) -> anyhow::Result<()> {
    initialize(silent, path.clone(), draft)?;
    inner_deploy_pages(silent)
}
