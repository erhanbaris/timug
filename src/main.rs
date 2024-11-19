mod config;
mod context;
mod error;
mod post;

use std::{fs::File, io::Write};

use anyhow::Result;
use context::TimugContext;
use minijinja::{context, path_loader, Environment};
use post::Post;

fn main() -> Result<()> {
    let context = TimugContext::build(None);
    // println!("Post: {:?}", post);

    let base = context.get_template_file_content("base.html")?;
    let footer = context.get_template_file_content("footer.html")?;
    let header = context.get_template_file_content("header.html")?;
    let post = context.get_template_file_content("post.html")?;

    let mut env = Environment::new();
    env.set_loader(path_loader(context.get_templates_path()));
    env.add_template("header.html", &header)?;
    env.add_template("footer.html", &footer)?;
    env.add_template("base.html", &base)?;
    env.add_template("post.html", &post)?;
    let template = env.get_template("post.html")?;
    let post = Post::load(
        &context,
        "How-I-Identify-Top-Talent-for-a-Small-Agile-Team.md",
    )?;

    let content = template
        .render(context!(config => context.config, post => post))
        .unwrap();

    println!("{:?}", &post.date);

    let mut file = File::create("foo.html")?;
    file.write_all(content.as_bytes())?;

    Ok(())
}
