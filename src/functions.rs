use chrono::Datelike;
use minijinja::{Environment, Error, Value};

pub fn build_functions(env: &mut Environment<'_>) {
    env.add_function("current_year", current_year);
    env.add_function("post_url", post_url);
}

fn current_year() -> Result<Value, Error> {
    let current_date = chrono::Utc::now();
    Ok(current_date.year().into())
}

fn post_url() -> Result<Value, Error> {
    let current_date = chrono::Utc::now();
    Ok(current_date.year().into())
}
