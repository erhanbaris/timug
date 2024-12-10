use std::{path::PathBuf, sync::Arc};

use colored::Colorize;
use minijinja::{context, value::Object, Value};
use serde::{Deserialize, Serialize};
use unidecode::unidecode;

use crate::{
    engine::{RenderEngine, Renderable},
    post::Post,
    posts::Posts,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub posts: Vec<Post>,
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
        println!("{}: {}", "Rendering".yellow(), file_name.display());

        let posts = Value::from_object(Posts {
            items: self.posts.clone().into(),
        });

        let template = engine.env.get_template(&ctx.template_path)?;
        let context = context! {
            ..context! { index => ctx.index, posts => posts },
            ..context.clone()
        };

        let content = template.render(context)?;
        engine.compress_and_write(content, &file_name)?;
        Ok(())
    }
}

impl Object for Tag {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let key = key.as_str()?;
        match key {
            "name" => Some(Value::from(self.name.as_str())),
            "items" => Some(Value::from_iter(
                self.posts.iter().cloned().map(Value::from_object),
            )),
            _ => None,
        }
    }
}
