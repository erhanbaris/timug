use std::sync::Arc;

use minijinja::{
    value::{from_args, Kwargs, Object, ObjectRepr},
    Error, State, Value,
};

pub struct FontAwesome;

impl std::fmt::Debug for FontAwesome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fontawesome")
    }
}

impl FontAwesome {
    pub fn new() -> FontAwesome {
        FontAwesome {}
    }
}

impl Object for FontAwesome {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(self: &Arc<Self>, _: &State<'_, '_>, args: &[Value]) -> Result<Value, Error> {
        let (style, icon, _): (&str, &str, Kwargs) = from_args(args)?;
        Ok(Value::from_safe_string(format!(
            "<i class=\"{} fa-{}\"></i>",
            style, icon
        )))
    }
}
