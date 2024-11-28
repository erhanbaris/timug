use std::sync::Arc;

use minijinja::{
    args, context,
    value::{from_args, Kwargs, Object, ObjectRepr},
    Error, ErrorKind, State, Value,
};

use crate::pages::QUOTE_HTML;

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

        let template = state.env().get_template(QUOTE_HTML)?;
        let context = context!(content => content, position => position);
        let content = template.render(context)?;

        Ok(Value::from_safe_string(content))
    }
}
