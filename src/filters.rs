use chrono::NaiveDateTime;
use minijinja::{Error, ErrorKind, Value};
use unidecode::unidecode;

use crate::template::RenderEngine;
const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

impl<'a> RenderEngine<'a> {
    pub fn build_filters(&mut self) {
        self.env.add_filter("formatdatetime", Self::format_date);
        self.env.add_filter("url_encode", Self::url_encode);
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

    fn url_encode(url: String) -> Result<Value, Error> {
        use urlencoding::encode;
        let url = unidecode(&url.to_lowercase()).replace([' ', '\r', '\n', '\t'], "-");
        let encoded = encode(&url);
        Ok(Value::from_safe_string(encoded.to_string()))
    }
}
