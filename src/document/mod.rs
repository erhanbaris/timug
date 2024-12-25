mod html;
mod markdown;

use crate::engine::RenderEngine;
use html::HtmlDocument;
use markdown::MarkdownDocument;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct DocumentContext<T>
where
    T: Serialize,
{
    pub source_file_path: PathBuf,
    pub target_file_path: PathBuf,
    pub template: String,
    pub title: String,
    pub index: usize,
    pub data: T,
}

pub trait Document<T>
where
    T: Serialize,
{
    fn render(&self, engine: &RenderEngine<'_>, ctx: DocumentContext<T>) -> crate::Result<()>;
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum DocumentType {
    Html,

    #[default]
    Markdown,
}

impl DocumentType {
    pub fn render<T>(&self, engine: &RenderEngine<'_>, ctx: DocumentContext<T>) -> crate::Result<()>
    where
        T: Serialize,
    {
        let document: Box<dyn Document<T>> = match self {
            DocumentType::Html => Box::new(HtmlDocument),
            DocumentType::Markdown => Box::new(MarkdownDocument),
        };

        document.render(engine, ctx)
    }
}
