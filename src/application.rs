use crate::cli::TemplateCommand;
use crate::config::TimugConfig;
use crate::consts::{ExamplesAssets, TemplateAssets, ASSETS_PATH, CONFIG_FILE_NAME, DEFAULT_DEPLOYMENT_FOLDER, DEFAULT_LANGUAGE, DEFAULT_THEME, PAGES_PATH, POSTS_PATH, ROCKET, TEMPLATES_PATH};
use crate::context::{build_context, get_context};
use crate::error::{CanceledByTheUserSnafu, CannotOverwriteConfigSnafu, CouldNotGetAbsolutePathSnafu, FileCreationFailedSnafu, NoCurrentDirSnafu, WriteSnafu, YamlSerializationFailedSnafu};
use crate::server::start_webserver;
use crate::tools::{get_slug, inner_deploy_pages};

use std::{
    fs::{create_dir, create_dir_all, File},
    io::Write,
    path::PathBuf,
};

use console::{style, Term};
use snafu::{ensure, ResultExt};

fn initialize(path: Option<PathBuf>, draft: bool) -> crate::Result<()> {
    build_context(path, draft)?;
    Ok(())
}

pub fn start_create_new_project(project_path: Option<PathBuf>) -> crate::Result<()> {
    let project_path = if let Some(path) = project_path {
        path
    } else {
        std::env::current_dir().context(NoCurrentDirSnafu)?
    };

    let terminal = Term::stdout();
    let mut config = TimugConfig::default();

    // Get full path
    let path = std::path::absolute(project_path.clone()).context(CouldNotGetAbsolutePathSnafu { path: project_path.clone() })?;

    config.lang = DEFAULT_LANGUAGE.to_string();
    config.theme = DEFAULT_THEME.to_string();
    config.deployment_folder = path.join(DEFAULT_DEPLOYMENT_FOLDER);
    config.blog_path = path.clone();

    let config_path = config.blog_path.join(CONFIG_FILE_NAME);
    if std::fs::exists(&config_path).unwrap_or_default() {
        // Config is already exists

        ensure!(log::max_level() != log::LevelFilter::Off, CannotOverwriteConfigSnafu);

        log::info!("\"{}\" already created.\r\n", config_path.display());
        log::info!("Do you want to overwrite it?");
        log::info!("[y/n] ");
        let answer = terminal.read_char();

        match answer {
            Ok('Y') | Ok('y') => (),
            _ => {
                return CanceledByTheUserSnafu
                    .fail()
                    .map_err(::core::convert::Into::into)
            }
        }
    }

    // Create path, if need it
    let _ = create_dir(&config.blog_path);
    let mut config_file = File::create(&config_path).context(FileCreationFailedSnafu { path: config_path.clone() })?;

    let config_string = serde_yaml::to_string(&config).context(YamlSerializationFailedSnafu)?;

    config_file
        .write_all(config_string.as_bytes())
        .context(WriteSnafu { path: config_path })?;

    let template_path = config.blog_path.join(TEMPLATES_PATH).join(DEFAULT_THEME);
    let posts_path = config.blog_path.join(POSTS_PATH);

    let _ = create_dir_all(&template_path);
    let _ = create_dir(&posts_path);
    let _ = create_dir(config.blog_path.join(PAGES_PATH));
    let _ = create_dir(config.blog_path.join(ASSETS_PATH));
    let _ = create_dir(config.blog_path.join(DEFAULT_DEPLOYMENT_FOLDER));

    for file_name in TemplateAssets::iter() {
        if let Some(file_content) = TemplateAssets::get(&file_name) {
            let mut file = File::create(template_path.join(file_name.to_string())).context(FileCreationFailedSnafu { path: template_path.join(file_name.to_string()) })?;
            file.write_all(&file_content.data)
                .context(WriteSnafu { path: template_path.join(file_name.to_string()) })?;
        }
    }

    for file_name in ExamplesAssets::iter() {
        if let Some(file_content) = ExamplesAssets::get(&file_name) {
            let mut file = File::create(posts_path.join(file_name.to_string())).context(FileCreationFailedSnafu { path: posts_path.join(file_name.to_string()) })?;
            file.write_all(&file_content.data)
                .context(WriteSnafu { path: posts_path.join(file_name.to_string()) })?;
        }
    }

    log::info!("{} New Timug project created.", ROCKET);
    log::info!("Execute following command to see your new site.");
    log::info!("{} {}", style("timug deploy {}").yellow(), path.display());
    log::info!("{} {}", style("timug start").yellow(), path.display());

    Ok(())
}

pub fn start_server(path: Option<PathBuf>, port: Option<u16>, draft: bool) -> crate::Result<()> {
    initialize(path.clone(), draft)?;
    log::info!("Building...");
    inner_deploy_pages()?;
    log::info!("Starting webserver...");
    start_webserver(port)?;
    Ok(())
}

pub fn start_deploy_pages(path: Option<PathBuf>, draft: bool) -> crate::Result<()> {
    initialize(path.clone(), draft)?;
    inner_deploy_pages()
}

pub fn create_page(path: Option<PathBuf>, title: String, draft: bool) -> crate::Result<()> {
    create_new(path, title, draft, PAGES_PATH)
}

pub fn create_post(path: Option<PathBuf>, title: String, draft: bool) -> crate::Result<()> {
    create_new(path, title, draft, POSTS_PATH)
}

fn create_new(path: Option<PathBuf>, title: String, draft: bool, folder: &str) -> crate::Result<()> {
    initialize(path.clone(), draft)?;
    let ctx = get_context(snafu::location!())?;
    let slug = get_slug(&title);
    let date = chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S");
    let path = ctx
        .config
        .blog_path
        .join(folder)
        .join(format!("{}.md", slug));

    log::debug!("Creating new {}...", path.display());
    log::debug!("Title: {}", title);
    log::debug!("Date: {}", date);
    log::debug!("Draft: {}", draft);

    let content = format!(
        r#"---
title: {}
date: {}{}
tags: 
---"#,
        title,
        date,
        match draft {
            true => "\ndraft: true",
            false => "",
        }
    );
    let mut file = File::create(&path).context(FileCreationFailedSnafu { path: path.clone() })?;
    file.write_all(content.as_bytes())
        .context(WriteSnafu { path })?;
    Ok(())
}

pub fn execute_template(path: Option<PathBuf>, command: TemplateCommand) -> crate::Result<()> {
    initialize(path.clone(), false)?;
    let ctx = get_context(snafu::location!())?;
    let template_path = ctx
        .config
        .blog_path
        .join(TEMPLATES_PATH)
        .join(DEFAULT_THEME);

    match command {
        TemplateCommand::Upgrade => {
            for file_name in TemplateAssets::iter() {
                if let Some(file_content) = TemplateAssets::get(&file_name) {
                    let mut file = File::create(template_path.join(file_name.to_string())).context(FileCreationFailedSnafu { path: template_path.join(file_name.to_string()) })?;
                    file.write_all(&file_content.data)
                        .context(WriteSnafu { path: template_path.join(file_name.to_string()) })?;
                }
            }

            log::warn!("Template updated.");
        }
        TemplateCommand::Deploy => {}
    };
    Ok(())
}
