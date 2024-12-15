use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Turn off all terminal outputs
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub silent: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create new timug project in the current folder
    Init { path: PathBuf },

    /// Generate static pages
    Deploy { config_path: Option<PathBuf> },

    /// Start development server with live update
    Start { config_path: Option<PathBuf> },
}
