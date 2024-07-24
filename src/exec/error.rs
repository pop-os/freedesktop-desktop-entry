// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::env::VarError;
use std::io;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExecError<'a> {
    #[error("{0}")]
    WrongFormat(String),

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

    #[error("Action '{action}' not found for desktop entry '{desktop_entry:?}'")]
    ActionNotFound {
        action: String,
        desktop_entry: &'a Path,
    },

    #[error("Exec key not found for action :'{action}' in desktop entry '{desktop_entry:?}'")]
    ActionExecKeyNotFound {
        action: String,
        desktop_entry: &'a Path,
    },

    #[error("Failed to launch aplication via dbus: {0}")]
    DBusError(#[from] zbus::Error),
}
