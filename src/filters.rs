use chrono::NaiveDateTime;
use minijinja::{Environment, Error, ErrorKind, Value};
const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn build_filters(env: &mut Environment<'_>) {
    env.add_filter("formatdatetime", format_date);
}

fn format_date(value: Value, format: Option<Value>) -> Result<Value, Error> {
    let format = match format {
        Some(format) => format.as_str().unwrap_or("%B %d, %Y").to_string(),
        _ => "%B %d, %Y".into(),
    };

    if let Some(value) = value.as_str() {
        let date_info = NaiveDateTime::parse_from_str(value, FORMAT)
            .map_err(|_| {
                Error::new(
                    ErrorKind::BadSerialization,
                    format!("{} could converted into datetime", value),
                )
            })?
            .format(&format);
        let formated_datetime = format!("{}", date_info);
        Ok(formated_datetime.into())
    } else {
        Ok("N/A".into())
    }
}
