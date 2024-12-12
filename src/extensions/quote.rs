use std::sync::Arc;

use minijinja::{
    args, render,
    value::{from_args, Kwargs, Object, ObjectRepr},
    Error, ErrorKind, State, Value,
};

use crate::context::get_context;

use super::Extension;

static HTML: &str = r#"<blockquote class="my-5 {{ position }}">{{ content | safe }}</blockquote>"#;

pub struct Quote;

impl std::fmt::Debug for Quote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "quote")
    }
}

impl Quote {
    pub fn new() -> Quote {
        Quote {}
    }
}

impl Object for Quote {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, args: &[Value]) -> Result<Value, Error> {
        let (position, kwargs): (Option<&str>, Kwargs) = from_args(args)?;

        let ctx = get_context();
        let position = match position {
            Some("left") => "left".to_string(),
            Some("right") => "right".to_string(),
            _ => "center".to_string(),
        };

        let caller: Value = kwargs.get("caller")?;
        let content = caller.call(state, args!())?;

        let content = content.as_str().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidOperation,
                "call block did not return a string",
            )
        })?;

        let content = match ctx.get_template_page("quote.html") {
            Some(page) => render!(page.content.as_str(), content => content, position => position),
            None => render!(HTML, content => content, position => position),
        };

        Ok(Value::from_safe_string(content))
    }
}

impl<'a> Extension<'a> for Quote {
    fn name() -> &'static str {
        "quote"
    }

    fn register(env: &mut minijinja::Environment<'a>) {
        env.add_global(Self::name(), Value::from_object(Self::new()));
    }
}
