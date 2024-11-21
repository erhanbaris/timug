mod config;
mod context;
mod error;
mod filters;
mod functions;
mod globals;
mod pages;
mod post;
mod posts;
mod template;

use anyhow::Result;
use context::TimugContext;
use filters::build_filters;
use functions::build_functions;
use globals::build_globals;
use minijinja::Environment;
use template::{
    build_base_templates, generate_base_pages, generate_pages, generate_posts, parse_pages,
    parse_posts,
};

fn main() -> Result<()> {
    let mut context = TimugContext::build(None);
    let mut env = Environment::new();

    build_base_templates(&mut env, &context)?;
    build_filters(&mut env);
    build_globals(&mut env, &mut context);
    build_functions(&mut env);

    parse_posts(&mut context)?;
    parse_pages(&mut env, &mut context)?;

    generate_pages(&mut env, &mut context)?;
    generate_posts(&mut env, &mut context)?;

    generate_base_pages(&mut env, &mut context)?;

    Ok(())
}
