use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimugError {
    #[error("'File not found: {0} ({1})'")]
    FileNotFound(String, String),

    #[error("'Path could not generated ({0})'")]
    PathCouldNotGenerated(String),
}
