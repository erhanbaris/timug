mod config;
mod context;
mod error;
mod post;

use anyhow::Result;
use context::TimugContext;
use minijinja::{context, path_loader, Environment};
use post::Post;

fn main() -> Result<()> {
    let context = TimugContext::build(None);
    let post = Post::load(&context, "How-I-Identify-Top-Talent-for-a-Small-Agile-Team.md")?;
    println!("Post: {:?}", post);

    let base = context.get_template_file_content("base.html")?;
    let footer = context.get_template_file_content("footer.html")?;
    let header = context.get_template_file_content("header.html")?;
    let post = context.get_template_file_content("post.html")?;

    let markdown_input = "hello world";
    let parser = pulldown_cmark::Parser::new(markdown_input);

    // Write to a new String buffer.
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);

    assert_eq!(&html_output, "<p>hello world</p>\n");
    println!("Hello, world!");

    let mut env = Environment::new();
    env.set_loader(path_loader(context.get_templates_path()));
    env.add_template("header.html", &header)?;
    env.add_template("footer.html", &footer)?;
    env.add_template("base.html", &base)?;
    env.add_template("post.html", &post)?;
    let template = env.get_template("post.html")?;
    println!(
        "{}",
        template.render(context!(config => context.config)).unwrap()
    );

    Ok(())
}
