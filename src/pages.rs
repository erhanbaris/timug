use std::sync::Arc;

use minijinja::{
    value::{Enumerator, Object},
    Value,
};use serde::{Deserialize, Serialize};

use crate::post::Post;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pages {
    pub items: Vec<Post>,
}

impl Object for Pages {
    fn repr(self: &Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Seq
    }
    fn get_value(self: &Arc<Self>, index: &Value) -> Option<Value> {
        let item = self.items.get(index.as_usize()?)?;
        Some(Value::from_object(item.clone()))
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Seq(self.items.len())
    }
}
