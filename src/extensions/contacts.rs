use std::sync::Arc;

use minijinja::{
    render,
    value::{Object, ObjectRepr},
    Error, State, Value,
};
use serde::{Deserialize, Serialize};

use crate::context::get_context;

use super::Extension;

static HTML: &str = include_str!("contacts.html");

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ContactInfo {
    icon: String,
    link: String,
    description: Option<String>,
}

pub struct Contacts;

impl std::fmt::Debug for Contacts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "contacts")
    }
}

impl Contacts {
    pub fn new() -> Contacts {
        Contacts {}
    }
}

impl Object for Contacts {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, _: &[Value]) -> Result<Value, Error> {
        let ctx = get_context();
        let html = &ctx.get_template_page("contacts.html", HTML);
        let env = state.env();
        let content = render!(in env, html, contacts => ctx.config.contacts);
        Ok(Value::from_safe_string(content))
    }
}

impl<'a> Extension<'a> for Contacts {
    fn name() -> &'static str {
        "contacts"
    }

    fn register(env: &mut minijinja::Environment<'a>) {
        env.add_global(Self::name(), Value::from_object(Self::new()));
    }
}
