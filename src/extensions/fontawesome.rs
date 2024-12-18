use std::sync::Arc;

use minijinja::{
    value::{from_args, Kwargs, Object, ObjectRepr},
    Error, State, Value,
};

use super::Extension;

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
        Ok(Value::from_safe_string(format!("<i class=\"ml-1 mr-0.5 {} fa-{}\"></i>", style, icon)))
    }
}

impl<'a> Extension<'a> for FontAwesome {
    fn name() -> &'static str {
        "fontawesome"
    }

    fn register(env: &mut minijinja::Environment<'a>) {
        env.add_global(Self::name(), Value::from_object(Self::new()));
    }

    fn header() -> &'static str {
        r#"<link rel="stylesheet" href="https://use.fontawesome.com/releases/v5.15.4/css/all.css" integrity="sha384-DyZ88mC6Up2uqS4h/KRgHuoeGwBcD4Ng9SiP4dIRy0EXTlnuz47vAwmeGwVChigm" crossorigin="anonymous" />"#
    }
}
