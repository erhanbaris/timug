use std::sync::Arc;

use minijinja::{
    render,
    value::{Object, ObjectRepr},
    Error, State, Value,
};
use serde::{Deserialize, Serialize};

use crate::context::get_context;

use super::Extension;

static HTML: &str = include_str!("projects.html");

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ProjectsInfo {
    name: String,
    link: String,
    description: Option<String>,
}

pub struct Projects;

impl std::fmt::Debug for Projects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "projects")
    }
}

impl Projects {
    pub fn new() -> Projects {
        Projects {}
    }
}

impl Object for Projects {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, _: &[Value]) -> Result<Value, Error> {
        let ctx = get_context();
        if let Some(projects) = ctx.get_config::<Vec<ProjectsInfo>>(Self::name()) {
            let env = state.env();
            let content = render!(in env, HTML, projects => projects);
            return Ok(Value::from_safe_string(content));
        }

        Ok(Value::UNDEFINED)
    }
}

impl<'a> Extension<'a> for Projects {
    fn name() -> &'static str {
        "projects"
    }

    fn register(env: &mut minijinja::Environment<'a>) {
        env.add_global(Self::name(), Value::from_object(Self::new()));
    }
}
