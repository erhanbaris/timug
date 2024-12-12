use std::sync::Arc;

use minijinja::{
    value::{Enumerator, Object},
    Value,
};
use serde::{Deserialize, Serialize};

use crate::{post::Post, tag::Tag};

pub struct TagsIterator<'a> {
    tags: &'a Tags,
    index: usize,
}

impl<'a> Iterator for TagsIterator<'a> {
    type Item = &'a Tag;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.tags.tags.len() {
            let result = Some(&self.tags.tags[self.index]);
            self.index += 1;
            result
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Tags {
    pub tags: Vec<Tag>,
}

impl Tags {
    pub fn add(&mut self, name: String, post: Arc<Post>) {
        let (tag, sort) = match self.tags.iter_mut().find(|item| item.name == name) {
            Some(tag) => (tag, false),
            None => {
                let tag = Tag {
                    name,
                    posts: Vec::new(),
                };

                self.tags.push(tag);
                (self.tags.last_mut().unwrap(), true)
            }
        };

        tag.posts.push(post);

        if sort {
            self.tags.sort_by(|x, y| x.name.cmp(&y.name));
        }
    }

    pub fn iter(&self) -> TagsIterator {
        TagsIterator {
            tags: self,
            index: 0,
        }
    }
}

impl Object for Tags {
    fn repr(self: &Arc<Self>) -> minijinja::value::ObjectRepr {
        minijinja::value::ObjectRepr::Iterable
    }

    fn get_value(self: &Arc<Self>, index: &Value) -> Option<Value> {
        let item = self.tags.get(index.as_usize()?)?;
        Some(Value::from_object(item.clone()))
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Seq(self.tags.len())
    }
}
