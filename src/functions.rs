use chrono::Datelike;
use minijinja::{Environment, Error, ErrorKind, State, Value};

use crate::posts::Posts;

pub fn build_functions(env: &mut Environment<'_>) {
    env.add_function("current_year", current_year);
    env.add_function("post_url", post_url);
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

    let post = match posts
        .items
        .iter()
        .find(|post| post.slug == slug && post.lang == lang)
    {
        Some(post) => post,
        None => {
            return Err(Error::new(
                ErrorKind::UndefinedError,
                format!("Post (lang: '{}', slug: '{}') could not found", lang, slug),
            ))
        }
    };

    Ok(Value::from_safe_string(format!(
        "/{}/{}/{}/{}/{}.html",
        lang,
        post.date.year(),
        post.date.month(),
        post.date.day(),
        slug
    )))
}
