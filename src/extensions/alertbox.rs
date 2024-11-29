use std::sync::Arc;

use minijinja::{
    args, context,
    value::{from_args, Kwargs, Object, ObjectRepr},
    Error, ErrorKind, State, Value,
};

use crate::{pages::ALERTBOX_HTML, tools::parse_yaml};

pub struct AlertBox;

impl std::fmt::Debug for AlertBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "info")
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

        let template = state.env().get_template(ALERTBOX_HTML)?;
        let context = context!(content => compiled_content, title => title, style => style);
        let content = template.render(context)?;

        Ok(Value::from_safe_string(content))
    }
}
