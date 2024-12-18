mod application;
mod cli;
mod config;
mod consts;
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
use application::{start_create_new_project, start_deploy_pages, start_server};
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Init { path } => start_create_new_project(cli.silent, path),
        cli::Commands::Deploy { path, draft } => start_deploy_pages(cli.silent, path, draft),
        cli::Commands::Start { path, port, draft } => start_server(path, port, draft),
    }?;

    Ok(())
}
