use std::sync::Arc;

use minijinja::{
    args, render,
    value::{from_args, Kwargs, Object, ObjectRepr},
    Error, ErrorKind, State, Value,
};

use crate::{context::get_context, tools::parse_yaml};

use super::Extension;

static HTML: &str = include_str!("alertbox.html");

pub struct AlertBox;

impl std::fmt::Debug for AlertBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "alertbox")
    }
}

impl AlertBox {
    pub fn new() -> AlertBox {
        AlertBox {}
    }
}

impl Object for AlertBox {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, args: &[Value]) -> Result<Value, Error> {
        let (style, title, kwargs): (&str, &str, Kwargs) = from_args(args)?;
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
        let html = &ctx.get_template_page("alertbox.html", HTML);

        let content = render!(html, content => compiled_content, title => title, style => style);
        Ok(Value::from_safe_string(content))
    }
}

impl<'a> Extension<'a> for AlertBox {
    fn name() -> &'static str {
        "alertbox"
    }

    fn register(env: &mut minijinja::Environment<'a>) {
        env.add_global(Self::name(), Value::from_object(Self::new()));
    }
}
