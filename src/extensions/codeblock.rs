use std::sync::Arc;

use minijinja::{
    args, render,
    value::{from_args, Kwargs, Object, ObjectRepr},
    Error, ErrorKind, State, Value,
};

use crate::context::get_context;

use super::Extension;

static HTML: &str = "<pre>
  <code class=\"language-{{lang}}\">{{ content | safe }}</code>
</pre>";

pub struct Codeblock;

impl std::fmt::Debug for Codeblock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "codeblock")
    }
}

impl Codeblock {
    pub fn new() -> Codeblock {
        Codeblock {}
    }
}

impl Object for Codeblock {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, args: &[Value]) -> Result<Value, Error> {
        let (lang, kwargs): (&str, Kwargs) = from_args(args)?;

        let ctx = get_context();
        let caller: Value = kwargs.get("caller")?;
        let content = caller.call(state, args!())?;

        let content = content
            .as_str()
            .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "call block did not return a string"))?
            .trim();

        let content = match ctx.get_template_page("codeblock.html") {
            Some(page) => render!(page.content.as_str(), content => content, lang => lang),
            None => render!(HTML, content => content, lang => lang),
        };

        Ok(Value::from_safe_string(content))
    }
}

impl<'a> Extension<'a> for Codeblock {
    fn name() -> &'static str {
        "codeblock"
    }

    fn register(env: &mut minijinja::Environment<'a>) {
        env.add_global(Self::name(), Value::from_object(Self::new()));
    }

    fn header() -> &'static str {
        r#"<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/default.min.css">
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
<link rel="stylesheet" href="https://unpkg.com/@highlightjs/cdn-assets@11.9.0/styles/atom-one-dark.min.css" />"#
    }

    fn after_body() -> &'static str {
        r#"<script>
document.addEventListener('DOMContentLoaded', (event) => {
    document.querySelectorAll('pre code').forEach((block) => {
        hljs.highlightBlock(block);
    });
});
</script>"#
    }
}
