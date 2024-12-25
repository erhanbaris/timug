#![allow(dead_code, unreachable_code)]

use std::path::PathBuf;

use snafu::Snafu;
use subprocess::PopenError;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(visibility(pub), display("File not found, path: {}, at {loc}", path.display()))]
    FileNotFound {
        #[snafu(source)]
        source: std::io::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        path: PathBuf,
    },

    #[snafu(visibility(pub), display("Template reference found, path: {name}, at {loc}"))]
    TemplateReferenceNotFound {
        #[snafu(implicit)]
        loc: snafu::Location,
        name: String,
    },

    #[snafu(visibility(pub), display("Context not initialized, at {loc}"))]
    ContextNotInitialized { loc: snafu::Location },

    #[snafu(visibility(pub), display("Template reference found, at {loc}"))]
    CannotOverwriteConfig {
        #[snafu(implicit)]
        loc: snafu::Location,
    },

    #[snafu(visibility(pub), display("Canceled by the user, at {loc}"))]
    CanceledByTheUser {
        #[snafu(implicit)]
        loc: snafu::Location,
    },

    #[snafu(visibility(pub), display("Failed to add tag, tag: {tag}, at {loc}"))]
    FailedToAddTag {
        #[snafu(implicit)]
        loc: snafu::Location,
        tag: String,
    },

    #[snafu(visibility(pub), display("Could not get file name, path: {}, at {loc}", path.display()))]
    CouldNotGetFilename {
        #[snafu(implicit)]
        loc: snafu::Location,
        path: PathBuf,
    },

    #[snafu(visibility(pub), display("Could not convert OsStr to string, path: {}, at {loc}", path.display()))]
    CouldNotConvertOsStr {
        #[snafu(implicit)]
        loc: snafu::Location,
        path: PathBuf,
    },

    #[snafu(visibility(pub), display("Could not absolute path, path: {}, at {loc}", path.display()))]
    CouldNotGetAbsolutePath {
        #[snafu(source)]
        source: std::io::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        path: PathBuf,
    },

    #[snafu(visibility(pub), display("No current dir, at {loc}"))]
    NoCurrentDir {
        #[snafu(source)]
        source: std::io::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
    },

    #[snafu(visibility(pub), display("Path canonicalize failed, path: {}, at {loc}", path.display()))]
    CanonicalizeError {
        #[snafu(source)]
        source: std::io::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        path: PathBuf,
    },

    #[snafu(visibility(pub), display("Current directory could not changed, path: {}, at {loc}", path.display()))]
    CurrentDirChangeError {
        #[snafu(source)]
        source: std::io::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        path: PathBuf,
    },

    #[snafu(visibility(pub), display("Reading directory failed, path: {}, at {loc}", path.display()))]
    ReadingDirectoryFailed {
        #[snafu(source)]
        source: std::io::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        path: PathBuf,
    },

    #[snafu(visibility(pub), display("Yaml serializetion failed, at {loc}"))]
    YamlSerializationFailed {
        #[snafu(source)]
        source: serde_yaml::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
    },

    #[snafu(visibility(pub), display("Yaml deserializetion failed, content: {content}, at {loc}"))]
    YamlDeserializationFailed {
        #[snafu(source)]
        source: serde_yaml::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        content: String,
    },

    #[snafu(visibility(pub), display("Markdown template not found, template: {template}, at {loc}"))]
    MarkdownTemplateNotFound {
        #[snafu(source)]
        source: minijinja::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        template: String,
    },

    #[snafu(visibility(pub), display("Markdown render failed, file: {}, error: {source}, at {loc}", file_name.display()))]
    MarkdownRenderFailed {
        #[snafu(source)]
        source: minijinja::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        file_name: PathBuf,
    },

    #[snafu(visibility(pub), display("Markdown template adding failed, path: {path}, error: {source}, at {loc}"))]
    MarkdownTemplateAddFailed {
        #[snafu(source)]
        source: minijinja::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        path: String,
    },

    #[snafu(visibility(pub), display("File creation failed, path: {}, at {loc}", path.display()))]
    FileCreationFailed {
        #[snafu(source)]
        source: std::io::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        path: PathBuf,
    },

    #[snafu(visibility(pub), display("Folder creation failed, path: {}, at {loc}", path.display()))]
    FolderCreationFailed {
        #[snafu(source)]
        source: std::io::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        path: PathBuf,
    },

    #[snafu(visibility(pub), display("Directory copy failed, from: {} to: {}, at {loc}", from.display(), to.display()))]
    DirectoryCopyFailed {
        #[snafu(source)]
        source: std::io::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        from: PathBuf,
        to: PathBuf,
    },

    #[snafu(visibility(pub), display("Write error, path: {}, at {loc}", path.display()))]
    WriteError {
        #[snafu(source)]
        source: std::io::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        path: PathBuf,
    },

    #[snafu(visibility(pub), display("Sub-process error, command: {command}, error: {source}, at {loc}"))]
    SubProcessError {
        #[snafu(source)]
        source: PopenError,
        #[snafu(implicit)]
        loc: snafu::Location,
        command: String,
    },

    #[snafu(visibility(pub), display("Path could not parsed, path: {path}, error: {source}, at {loc}"))]
    PathBufParseError {
        #[snafu(source)]
        source: core::convert::Infallible,
        #[snafu(implicit)]
        loc: snafu::Location,
        path: String,
    },

    #[snafu(visibility(pub), display("Filesystem watcher failed, at {loc}"))]
    FileSystemWatcherFailed {
        #[snafu(source)]
        source: notify::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
    },

    #[snafu(visibility(pub), display("Filesystem watcher failed, path: {} at {loc}", path.display()))]
    CouldNotWatchFilesystem {
        #[snafu(source)]
        source: notify::Error,
        #[snafu(implicit)]
        loc: snafu::Location,
        path: PathBuf,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
