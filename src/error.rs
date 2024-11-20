use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimugError {
    #[error("'File not found: {0} ({1})'")]
    FileNotFound(String, String),
}
