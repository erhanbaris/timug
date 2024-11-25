mod config;
mod context;
mod error;
mod filters;
mod functions;
mod page;
mod pages;
mod post;
mod posts;
mod template;
mod tools;

use anyhow::Result;
use context::TimugContext;

use template::RenderEngine;

fn main() -> Result<()> {
    let context = TimugContext::build(None);
    let mut engine = RenderEngine::new(context);
    engine.run()?;
    Ok(())
}
