mod cli;
mod config;
mod context;
mod engine;
mod error;
mod extensions;
mod filters;
mod functions;
mod page;
mod pages;
mod post;
mod posts;
mod server;
mod tag;
mod tags;
mod template;
mod tools;

use anyhow::Result;
use clap::Parser;
use config::TimugConfig;
use context::{
    build_context, ASSETS_PATH, CONFIG_FILE_NAME, PAGES_PATH, POSTS_PATH, TEMPLATES_PATH,
};
use rust_embed::Embed;
use server::start;

use std::{
    fs::{create_dir, create_dir_all, File},
    io::Write,
    path::PathBuf,
    time::Instant,
};

use console::{Emoji, Term};

use engine::RenderEngine;
use extensions::{
    alertbox::AlertBox, codeblock::Codeblock, contacts::Contacts, fontawesome::FontAwesome,
    gist::Gist, info::Info, projects::Projects, quote::Quote, reading::Reading,
    social_media_share::SocialMediaShare, stats::Stats,
};

#[derive(Embed)]
#[folder = "templates/default"]
struct TemplateAssets;

#[derive(Embed)]
#[folder = "example"]
struct ExampleAssets;

static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");
static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", ":-)");

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Init { path } => create_new_project(cli.silent, path),
        cli::Commands::Deploy { config_path } => deploy_pages(cli.silent, config_path),
        cli::Commands::Start { config_path } => start_development(cli.silent, config_path),
    }?;

    Ok(())
}

fn create_new_project(silent: bool, path: PathBuf) -> anyhow::Result<()> {
    let terminal = Term::stdout();
    let mut config = TimugConfig::default();

    // Get full path
    let path = std::path::absolute(path.clone())?;

    config.lang = "en".to_string();
    config.theme = "default".to_string();
    config.deployment_folder = path.join("public");
    config.blog_path = path.clone();

    let config_path = config.blog_path.join(CONFIG_FILE_NAME);
    if std::fs::exists(&config_path).unwrap_or_default() {
        // Config is already exists
        if silent {
            return Err(anyhow::anyhow!(""));
        }

        terminal.write_line(&format!(
            "\"{}\" already created.\r\n",
            config_path.display()
        ))?;
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

    let template_path = config.blog_path.join(TEMPLATES_PATH).join("default");
    let posts_path = config.blog_path.join(POSTS_PATH);

    let _ = create_dir_all(&template_path);
    let _ = create_dir(&posts_path);
    let _ = create_dir(config.blog_path.join(PAGES_PATH));
    let _ = create_dir(config.blog_path.join(ASSETS_PATH));
    let _ = create_dir(config.blog_path.join("public"));

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
        terminal.write_line(&format!("Execute following command to see new site."))?;
        terminal.write_line(&format!("timug deploy {}", path.display()))?;
        terminal.write_line(&format!("timug start {}", path.display()))?;
    }

    Ok(())
}

fn deploy_pages(silent: bool, config_path: Option<PathBuf>) -> anyhow::Result<()> {
    build_context(silent, config_path)?;
    let started = Instant::now();
    let mut engine = RenderEngine::new(silent);
    engine.register_extension::<Codeblock>()?;
    engine.register_extension::<Quote>()?;
    engine.register_extension::<Gist>()?;
    engine.register_extension::<AlertBox>()?;
    engine.register_extension::<FontAwesome>()?;
    engine.register_extension::<Info>()?;
    engine.register_extension::<SocialMediaShare>()?;
    engine.register_extension::<Reading>()?;
    engine.register_extension::<Projects>()?;
    engine.register_extension::<Contacts>()?;
    engine.register_extension::<Stats>()?;
    engine.run()?;

    if !silent {
        println!(
            "{} Done in {:?} seconds",
            SPARKLE,
            started.elapsed().as_secs_f32()
        );
    }

    Ok(())
}

fn start_development(silent: bool, config_path: Option<PathBuf>) -> anyhow::Result<()> {
    build_context(silent, config_path)?;
    start();
    Ok(())
}
