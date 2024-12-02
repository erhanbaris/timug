use minijinja::Environment;

pub mod alertbox;
pub mod codeblock;
pub mod fontawesome;
pub mod gist;
pub mod info;
pub mod quote;

pub trait Extension<'a> {
    fn name() -> &'static str;
    fn register(env: &mut Environment<'a>);
    fn header() -> &'static str {
        ""
    }
    fn after_body() -> &'static str {
        ""
    }
}
