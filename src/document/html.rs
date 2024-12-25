use std::{path::PathBuf, str::FromStr};

use console::style;
use minijinja::context;
use serde::Serialize;
use snafu::ResultExt;

use crate::{
    engine::RenderEngine,
    error::{MarkdownRenderFailedSnafu, MarkdownTemplateNotFoundSnafu, PathBufParseSnafu},
    tools::{get_file_name, get_path},
};

use super::{Document, DocumentContext};

pub struct HtmlDocument;

impl<T> Document<T> for HtmlDocument
where
    T: Serialize,
{
    fn render(&self, engine: &RenderEngine<'_>, ctx: DocumentContext<T>) -> crate::Result<()> {
        let filename: String = get_file_name(ctx.source_file_path.as_path())?;
        let source_file_path_str = get_path(ctx.source_file_path.as_path())?;

        let template = engine
            .env
            .get_template(&source_file_path_str)
            .context(MarkdownTemplateNotFoundSnafu { template: source_file_path_str })?;
        engine.update_status(style("Rendering as HTML").bold().cyan().to_string(), &filename);

        let context = engine.create_context()?;
        let context = context! {
            ..context! {
                title => ctx.title.as_str(),
                index => ctx.index,
                data => ctx.data,
            },
            ..context.clone()
        };
        let content: String = template
            .render(context)
            .context(MarkdownRenderFailedSnafu {
                file_name: PathBuf::from_str(&filename).context(PathBufParseSnafu { path: filename.clone() })?,
            })?;

        engine.write(content, &ctx.target_file_path)?;
        engine.update_status(style("Generated as HTML").bold().green().to_string(), filename.as_str());

        Ok(())
    }
}
