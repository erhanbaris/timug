use std::sync::Arc;

use minijinja::{
    value::{from_args, Kwargs, Object, ObjectRepr},
    Error, State, Value,
};

pub struct Gist;

impl std::fmt::Debug for Gist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gist")
    }
}

impl Gist {
    pub fn new() -> Gist {
        Gist {}
    }
}

impl Object for Gist {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(self: &Arc<Self>, _: &State<'_, '_>, args: &[Value]) -> Result<Value, Error> {
        let (gist, filename, _): (&str, &str, Kwargs) = from_args(args)?;
        Ok(Value::from_safe_string(format!(
            "<script src=\"https://gist.github.com/{}.js?file={}\"></script>",
            gist, filename
        )))
    }
}
