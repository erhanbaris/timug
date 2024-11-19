use pulldown_cmark::Options;

use crate::{context::TimugContext, error::TimugError};

pub struct Post {
    pub title: String,
    pub content: String,
    pub date: String,
    pub slug: String,
    pub tags: Vec<String>,
}

impl Post {
    pub fn load(context: &TimugContext, path: &str) -> Result<String, TimugError> {
        let content: String = context.get_blog_file_content(path)?;
        let mut html_output = String::new();
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        opts.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);

        let parser = pulldown_cmark::Parser::new_ext(&content, opts);

        pulldown_cmark::html::push_html(&mut html_output, parser);
        Ok(html_output)
    }
}
