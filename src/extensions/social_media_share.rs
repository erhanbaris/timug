use std::sync::Arc;

use minijinja::{
    render,
    value::{from_args, Kwargs, Object, ObjectRepr},
    Error, ErrorKind, State, Value,
};

use crate::context::get_context;

use super::Extension;

static HTML: &str = include_str!("social_media_share.html");

pub struct SocialMediaShare;

impl std::fmt::Debug for SocialMediaShare {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "social_media_share")
    }
}

impl SocialMediaShare {
    pub fn new() -> SocialMediaShare {
        SocialMediaShare {}
    }
}

impl Object for SocialMediaShare {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, args: &[Value]) -> Result<Value, Error> {
        let (data, _): (Value, Kwargs) = from_args(args)?;

        let env = state.env();
        let ctx = get_context(snafu::location!()).map_err(|err| Error::new(ErrorKind::InvalidOperation, err.to_string()))?;

        let content = match ctx.get_template_page("social_media_share.html") {
            Some(page) => {
                render!(in env, page.content.as_str(), data => data, posts => ctx.posts_value, pages => ctx.pages_value)
            }
            None => {
                render!(in env, HTML, data => data, posts => ctx.posts_value, pages => ctx.pages_value)
            }
        };

        Ok(Value::from_safe_string(content))
    }
}

impl<'a> Extension<'a> for SocialMediaShare {
    fn name() -> &'static str {
        "social_media_share"
    }

    fn register(env: &mut minijinja::Environment<'a>) {
        env.add_global(Self::name(), Value::from_object(Self::new()));
    }
}
