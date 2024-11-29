use std::sync::Arc;

use minijinja::{
    args, context,
    value::{from_args, Kwargs, Object, ObjectRepr},
    Error, ErrorKind, State, Value,
};

use crate::{pages::INFO_HTML, tools::parse_yaml};

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

        let caller: Value = kwargs.get("caller")?;
        let content = caller.call(state, args!())?;

        let content = content.as_str().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidOperation,
                "call block did not return a string",
            )
        })?;


        println!("content: {}", content);
        let mut compiled_content = String::new();
        pulldown_cmark::html::push_html(&mut compiled_content, parse_yaml(content));
        println!("content: {}", compiled_content);

        let template = state.env().get_template(INFO_HTML)?;
        let context = context!(content => compiled_content);
        let content = template.render(context)?;
        println!("content: {}", content);

        Ok(Value::from_safe_string(content))
    }
}
