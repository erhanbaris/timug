mod config;
mod context;
mod error;
mod filters;
mod functions;
mod post;
mod template;

use anyhow::Result;
use context::TimugContext;
use filters::build_filters;
use functions::build_functions;
use minijinja::Environment;
use template::{build_base_templates, generate_posts, parse_posts};

fn main() -> Result<()> {
    let mut context = TimugContext::build(None);
    let mut env = Environment::new();

    build_base_templates(&mut env, &context)?;
    build_filters(&mut env);
    build_functions(&mut env);
    parse_posts(&mut context)?;
    generate_posts(&mut env, &mut context)?;

    Ok(())
}
