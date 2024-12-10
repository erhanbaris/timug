use chrono::Datelike;
use minijinja::{Error, ErrorKind, State, Value};

use crate::{context::get_context, engine::RenderEngine, pages::Pages, posts::Posts};

impl<'a> RenderEngine<'a> {
    pub fn build_functions(&mut self) {
        self.env.add_function("current_year", Self::current_year);
        self.env.add_function("post_url", Self::post_url);
        self.env.add_function("page_url", Self::page_url);
    }

    fn current_year() -> Result<Value, Error> {
        let current_date = chrono::Utc::now();
        Ok(current_date.year().into())
    }

    fn post_url(lang: String, slug: String, state: &State) -> Result<Value, Error> {
        let posts = match state.lookup("posts") {
            Some(posts) => posts,
            None => {
                return Err(Error::new(
                    ErrorKind::UndefinedError,
                    "'posts' not found".to_string(),
                ))
            }
        };

        let posts = match posts
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Posts>())
        {
            Some(posts) => posts,
            None => {
                return Err(Error::new(
                    ErrorKind::UndefinedError,
                    "'posts' is not a Posts type".to_string(),
                ))
            }
        };

        let post = match posts.items.iter().find(|post| post.slug().as_str() == slug) {
            Some(post) => post,
            None => {
                return Err(Error::new(
                    ErrorKind::UndefinedError,
                    format!("Post (lang: '{}', slug: '{}') could not found", lang, slug),
                ))
            }
        };

        let ctx = get_context();
        let date = post.date();
        Ok(Value::from_safe_string(format!(
            "{}/{}/{}/{}/{}.html",
            ctx.config.site_url,
            date.year(),
            date.month(),
            date.day(),
            slug
        )))
    }

    fn page_url(slug: String, state: &State) -> Result<Value, Error> {
        let pages = match state.lookup("pages") {
            Some(pages) => pages,
            None => {
                return Err(Error::new(
                    ErrorKind::UndefinedError,
                    "'pages' not found".to_string(),
                ))
            }
        };

        let pages = match pages
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Pages>())
        {
            Some(pages) => pages,
            None => {
                return Err(Error::new(
                    ErrorKind::UndefinedError,
                    "'pages' is not a Posts type".to_string(),
                ))
            }
        };

        let page = match pages.items.iter().find(|page| page.slug == slug) {
            Some(page) => page,
            None => {
                return Err(Error::new(
                    ErrorKind::UndefinedError,
                    format!("Page (slug: '{}') could not found", slug),
                ))
            }
        };

        let ctx = get_context();
        Ok(Value::from_safe_string(format!(
            "{}/{}.html",
            ctx.config.site_url, page.slug
        )))
    }
}
