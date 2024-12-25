use std::{path::PathBuf, sync::Arc};

use console::style;
use minijinja::{context, value::Object, Value};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use unidecode::unidecode;

use crate::{
    engine::{RenderEngine, Renderable},
    error::{MarkdownRenderFailedSnafu, MarkdownTemplateNotFoundSnafu},
    post::Post,
    posts::Posts,
    tools::get_file_name,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub posts: Vec<Arc<Post>>,
}

pub struct TagContext {
    pub folder: PathBuf,
    pub index: usize,
    pub template_path: String,
}

impl Renderable for Tag {
    type Context = TagContext;
    fn render(&self, engine: &RenderEngine<'_>, ctx: TagContext) -> crate::Result<()> {
        let name = unidecode(&self.name).replace([' ', '\r', '\n', '\t'], "-");
        let context = engine.create_context()?;
        let file_name = ctx.folder.join(format!("{}.html", name.to_lowercase()));
        engine.update_status(style("Rendering tag").bold().cyan().to_string(), get_file_name(&file_name)?.as_str());

        let posts = Value::from_object(Posts { posts: self.posts.clone() });

        let template = engine
            .env
            .get_template(&ctx.template_path)
            .context(MarkdownTemplateNotFoundSnafu { template: &ctx.template_path })?;
        let context = context! {
            ..context! { index => ctx.index, posts => posts },
            ..context.clone()
        };

        let content = template
            .render(context)
            .context(MarkdownRenderFailedSnafu { file_name: file_name.clone() })?;
        engine.write(content, &file_name)?;
        engine.update_status(style("Generated tag").bold().green().to_string(), get_file_name(&file_name)?.as_str());
        Ok(())
    }
}

impl Object for Tag {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let key = key.as_str()?;
        match key {
            "name" => Some(Value::from(self.name.as_str())),
            "items" => Some(Value::from_iter(self.posts.iter().cloned().map(Value::from_dyn_object))),
            _ => None,
        }
    }
}
