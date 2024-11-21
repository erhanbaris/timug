mod config;
mod context;
mod error;
mod filters;
mod functions;
mod globals;
mod post;
mod posts;
mod pages;
mod template;

use anyhow::Result;
use context::TimugContext;
use filters::build_filters;
use functions::build_functions;
use globals::build_globals;
use minijinja::Environment;
use template::{build_base_templates, generate_page, generate_pages, generate_posts, generate_posts_page, parse_pages, parse_posts, INDEX_HTML, POSTS_HTML};

fn main() -> Result<()> {
    let mut context = TimugContext::build(None);
    let mut env = Environment::new();

    build_base_templates(&mut env, &context)?;
    build_filters(&mut env);
    build_globals(&mut env, &mut context);
    build_functions(&mut env);
    parse_posts(&mut context)?;
    parse_pages(&mut context)?;
    generate_pages(&mut env, &mut context)?;
    generate_posts(&mut env, &mut context)?;
    generate_posts_page(&mut env, &mut context)?;
    generate_page(&mut env, &mut context, INDEX_HTML)?;
    generate_page(&mut env, &mut context, POSTS_HTML)?;

    Ok(())
}
