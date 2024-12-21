use clap::*;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Set terminal log level
    #[arg(short, long)]
    pub log: Option<LogLevel>,

    /// Path to the project folder
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum LogLevel {
    /// Disable all output.
    #[value()]
    Off,
    /// Set log level to `error`.
    #[value()]
    Error,
    /// Set log level to `warn`.
    #[value()]
    Warn,
    /// Set log level to `info`.
    #[value()]
    Info,
    /// Set log level to `debug`.
    #[value()]
    Debug,
    /// Set log level to `trace`.
    #[value()]
    Trace,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum CreateType {
    /// Create new post.
    #[value()]
    Post,
    /// Create new page.
    #[value()]
    Page,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum TemplateCommand {
    /// Update template
    #[value()]
    Update,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create new timug project in the current folder
    Init,

    /// Create new static post or page
    Create {
        #[clap(name = "type")]
        _type: CreateType,

        title: String,
        /// Create as draft
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        draft: bool,
    },

    /// Generate static pages
    Deploy {
        /// Deploy draft posts
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        draft: bool,
    },

    /// Start development server with live update
    Start {
        port: Option<u16>,

        /// Render draft posts
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        draft: bool,
    },

    /// Template related commands
    Template {
        #[arg(value_enum)]
        command: TemplateCommand,
    },
}
