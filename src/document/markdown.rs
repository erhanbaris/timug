use std::{path::PathBuf, str::FromStr};

use console::style;
use minijinja::context;
use serde::Serialize;
use snafu::ResultExt;

use crate::{
    engine::RenderEngine,
    error::{MarkdownRenderFailedSnafu, MarkdownTemplateNotFoundSnafu, PathBufParseSnafu},
    tools::{get_file_content, get_file_name, parse_yaml},
};

use super::{Document, DocumentContext};

pub struct MarkdownDocument;

impl<T> Document<T> for MarkdownDocument
where
    T: Serialize,
{
    fn render(&self, engine: &RenderEngine<'_>, ctx: DocumentContext<T>) -> crate::Result<()> {
        let file_name = get_file_name(ctx.source_file_path.as_path())?;

        let context = engine.create_context()?;
        let mut content: String = get_file_content(&ctx.source_file_path)?;

        engine.update_status(style("Rendering as Markdown").bold().cyan().to_string(), file_name.as_str());

        if content.contains("{%") {
            let content_tmp = engine
                .env
                .render_str(content.as_str(), &context)
                .context(MarkdownRenderFailedSnafu { file_name: file_name.clone() })?;
            content = content_tmp;
        }

        let mut content_tmp = String::new();
        let parsed = parse_yaml(content.as_str());
        pulldown_cmark::html::push_html(&mut content_tmp, parsed);

        let template = engine
            .env
            .get_template(&ctx.template)
            .context(MarkdownTemplateNotFoundSnafu { template: &ctx.template })?;
        let context = context! {
            ..context! {
                title => ctx.title.as_str(),
                content => content_tmp,
                index => ctx.index,
                data => ctx.data,
            },
            ..context.clone()
        };

        let content: String = template
            .render(context)
            .context(MarkdownRenderFailedSnafu {
                file_name: PathBuf::from_str(&file_name).context(PathBufParseSnafu { path: file_name.clone() })?,
            })?;
        engine.write(content, &ctx.target_file_path)?;
        engine.update_status(style("Generated as Markdown").bold().green().to_string(), file_name.as_str());
        Ok(())
    }
}
