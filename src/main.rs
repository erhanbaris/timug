mod application;
mod cli;
mod config;
mod consts;
mod context;
mod document;
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

use application::{create_page, create_post, execute_template, start_create_new_project, start_deploy_pages, start_server};
use clap::Parser;
use cli::{CreateType, LogLevel};
use env_logger::fmt::style;
use log::{Level, LevelFilter};
use std::io::Write;

pub use crate::error::Error;
pub use crate::error::Result;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    let level_filter = match cli.log {
        Some(ref log) => match log {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        },
        None => LevelFilter::Info,
    };

    env_logger::Builder::new()
        .format(|buf, record| {
            let start_color = match record.level() {
                Level::Trace => style::AnsiColor::Cyan.on_default(),
                Level::Debug => style::AnsiColor::Blue.on_default(),
                Level::Info => style::AnsiColor::Green.on_default(),
                Level::Warn => style::AnsiColor::Yellow.on_default(),
                Level::Error => style::AnsiColor::Red
                    .on_default()
                    .effects(style::Effects::BOLD),
            };

            let level_info = format!("{}{}{}", start_color, record.level(), style::Reset);

            writeln!(buf, "{} [{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), level_info, record.args())
        })
        .filter(Some("timug"), level_filter)
        .init();

    let result = match cli.command {
        cli::Commands::Init => start_create_new_project(cli.path),
        cli::Commands::Deploy { draft } => start_deploy_pages(cli.path, draft),
        cli::Commands::Server { port, draft } => start_server(cli.path, port, draft),
        cli::Commands::Create { _type, title, draft } => match _type {
            CreateType::Post => create_post(cli.path, title, draft),
            CreateType::Page => create_page(cli.path, title, draft),
        },
        cli::Commands::Template { command } => execute_template(cli.path, command),
    };

    if let Err(ref e) = result {
        log::error!("{}", e);
    }

    Ok(())
}
