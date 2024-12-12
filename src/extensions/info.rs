use std::sync::Arc;

use minijinja::{
    args, render,
    value::{from_args, Kwargs, Object, ObjectRepr},
    Error, ErrorKind, State, Value,
};

use crate::{context::get_context, tools::parse_yaml};

use super::Extension;

static HTML: &str = include_str!("info.html");

pub struct Info;

impl std::fmt::Debug for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "info")
    }
}

impl Info {
    pub fn new() -> Info {
        Info {}
    }
}

impl Object for Info {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, args: &[Value]) -> Result<Value, Error> {
        let (_, kwargs): (Option<&str>, Kwargs) = from_args(args)?;

        let ctx = get_context();
        let caller: Value = kwargs.get("caller")?;
        let content = caller.call(state, args!())?;

        let content = content.as_str().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidOperation,
                "call block did not return a string",
            )
        })?;

        let mut compiled_content = String::new();
        pulldown_cmark::html::push_html(&mut compiled_content, parse_yaml(content));

        let content = match ctx.get_template_page("info.html") {
            Some(page) => render!(page.content.as_str(), content => compiled_content),
            None => render!(HTML, content => compiled_content),
        };

        Ok(Value::from_safe_string(content))
    }
}

impl<'a> Extension<'a> for Info {
    fn name() -> &'static str {
        "info"
    }

    fn register(env: &mut minijinja::Environment<'a>) {
        env.add_global(Self::name(), Value::from_object(Self::new()));
    }
}
