use std::env::VarError;
use std::io;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExecError<'a> {
    #[error("Unmatched quote delimiter: '{exec}'")]
    UnmatchedQuote { exec: String },

    #[error("Exec string is empty")]
    EmptyExecString,

    #[error("$SHELL environment variable is not set")]
    ShellNotFound(#[from] VarError),

    #[error("Failed to run Exec command")]
    IoError(#[from] io::Error),

    #[error("Exec command '{exec}' exited with status code '{status:?}'")]
    NonZeroStatusCode { status: Option<i32>, exec: String },

    #[error("Unknown field code: '{0}'")]
    UnknownFieldCode(String),

    #[error("Deprecated field code: '{0}'")]
    DeprecatedFieldCode(String),

    #[error("Exec key not found in desktop entry '{0:?}'")]
    MissingExecKey(&'a Path),

    #[error("Failed to launch aplication via dbus: {0}")]
    DBusError(#[from] zbus::Error),
}
