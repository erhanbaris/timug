use std::{path::PathBuf, sync::Arc};

use console::style;
use minijinja::{context, value::Object, Value};
use serde::{Deserialize, Serialize};
use unidecode::unidecode;

use crate::{
    engine::{RenderEngine, Renderable},
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
    fn render(&self, engine: &RenderEngine<'_>, ctx: TagContext) -> anyhow::Result<()> {
        let name = unidecode(&self.name).replace([' ', '\r', '\n', '\t'], "-");
        let context = engine.create_context();
        let file_name = ctx.folder.join(format!("{}.html", name.to_lowercase()));
        engine.update_status(style("Rendering tag").bold().cyan().to_string(), get_file_name(&file_name)?.as_str());

        let posts = Value::from_object(Posts { posts: self.posts.clone() });

        let template = engine.env.get_template(&ctx.template_path)?;
        let context = context! {
            ..context! { index => ctx.index, posts => posts },
            ..context.clone()
        };

        let content = template.render(context)?;
        engine.compress_and_write(content, &file_name)?;
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
