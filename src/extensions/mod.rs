use minijinja::Environment;
use parking_lot::RwLockWriteGuard;

use crate::context::TimugContext;

pub mod alertbox;
pub mod analytics;
pub mod codeblock;
pub mod contacts;
pub mod fontawesome;
pub mod gist;
pub mod info;
pub mod projects;
pub mod quote;
pub mod reading;
pub mod social_media_share;
pub mod stats;

pub trait Extension<'a> {
    fn name() -> &'static str;
    fn register(env: &mut Environment<'a>);
    fn header() -> &'static str {
        ""
    }
    fn after_body(_: &'_ mut RwLockWriteGuard<'static, TimugContext>) {}
}
