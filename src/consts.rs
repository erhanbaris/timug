use console::Emoji;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "templates/default"]
pub struct TemplateAssets;

#[derive(Embed)]
#[folder = "example"]
pub struct ExampleAssets;

pub const DEFAULT_DEPLOYMENT_FOLDER: &str = "public";
pub const DEFAULT_LANGUAGE: &str = "en";
pub const DEFAULT_THEME: &str = "default";
pub const DEFAULT_WEBSERVER_PORT: u16 = 8080;

pub const TEMPLATES_PATH: &str = "templates";
pub const POSTS_PATH: &str = "posts";
pub const PAGES_PATH: &str = "pages";
pub const ASSETS_PATH: &str = "assets";
pub const CONFIG_FILE_NAME: &str = "timug.yaml";

pub const ROCKET: Emoji<'_, '_> = Emoji("🚀 ", ":-)");
pub const SPARKLE: Emoji<'_, '_> = Emoji("✨ ", "#");
