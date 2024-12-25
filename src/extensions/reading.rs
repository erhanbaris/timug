use std::sync::Arc;

use minijinja::{
    render,
    value::{Object, ObjectRepr},
    Error, ErrorKind, State, Value,
};
use serde::{Deserialize, Serialize};

use crate::context::get_context;

use super::Extension;

static HTML: &str = include_str!("reading.html");

#[derive(Debug, Default, Serialize, Deserialize)]
struct ReadingInfo {
    image: Option<String>,
    name: String,
    series_name: Option<String>,
    author: String,
    link: String,
}

pub struct Reading;

impl std::fmt::Debug for Reading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "reading")
    }
}

impl Reading {
    pub fn new() -> Reading {
        Reading {}
    }
}

impl Object for Reading {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, _: &[Value]) -> Result<Value, Error> {
        let ctx = get_context(snafu::location!()).map_err(|err| Error::new(ErrorKind::InvalidOperation, err.to_string()))?;
        if let Some(config) = ctx.get_config::<ReadingInfo>(Self::name()) {
            let env = state.env();

            let content = match ctx.get_template_page("reading.html") {
                Some(page) => {
                    render!(in env, page.content.as_str(), image => config.image, name => config.name, series_name => config.series_name, author => config.author, link => config.link)
                }
                None => {
                    render!(in env, HTML, image => config.image, name => config.name, series_name => config.series_name, author => config.author, link => config.link)
                }
            };

            return Ok(Value::from_safe_string(content));
        }

        Ok(Value::UNDEFINED)
    }
}

impl<'a> Extension<'a> for Reading {
    fn name() -> &'static str {
        "reading"
    }

    fn register(env: &mut minijinja::Environment<'a>) {
        env.add_global(Self::name(), Value::from_object(Self::new()));
    }
}
