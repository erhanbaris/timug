mod config;
mod context;
mod error;
mod extensions;
mod filters;
mod functions;
mod page;
mod pages;
mod post;
mod posts;
mod template;
mod tools;

use anyhow::Result;

use template::RenderEngine;

fn main() -> Result<()> {
    let mut engine = RenderEngine::new();
    engine.run()?;
    Ok(())
}
