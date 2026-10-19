// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::DesktopEntry;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExecError {
    #[error("{0}")]
    WrongFormat(String),

    #[error("Exec field is empty")]
    ExecFieldIsEmpty,

    #[error("Exec key was not found")]
    ExecFieldNotFound,
}

impl DesktopEntry {
    #[inline]
    pub fn parse_exec(&self) -> Result<Vec<String>, ExecError> {
        self.get_args(self.exec(), &[], &[] as &[&str])
    }

    /// Macros like `%f` (cf [.desktop spec](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#exec-variables)) will be subtitued using the `uris` parameter.
    #[inline]
    pub fn parse_exec_with_uris<L>(
        &self,
        uris: &[&str],
        locales: &[L],
    ) -> Result<Vec<String>, ExecError>
    where
        L: AsRef<str>,
    {
        self.get_args(self.exec(), uris, locales)
    }

    #[inline]
    pub fn parse_exec_action(&self, action_name: &str) -> Result<Vec<String>, ExecError> {
        self.get_args(self.action_exec(action_name), &[], &[] as &[&str])
    }

    #[inline]
    pub fn parse_exec_action_with_uris<L>(
        &self,
        action_name: &str,
        uris: &[&str],
        locales: &[L],
    ) -> Result<Vec<String>, ExecError>
    where
        L: AsRef<str>,
    {
        self.get_args(self.action_exec(action_name), uris, locales)
    }

    fn get_args<L>(
        &self,
        exec: Option<&str>,
        uris: &[&str],
        locales: &[L],
    ) -> Result<Vec<String>, ExecError>
    where
        L: AsRef<str>,
    {
        #[inline(never)]
        fn inner<'a>(
            this: &'a DesktopEntry,
            exec: Option<&str>,
            uris: &[&str],
            locales: &mut dyn Iterator<Item = &str>,
        ) -> Result<Vec<String>, ExecError> {
            let Some(exec) = exec else {
                return Err(ExecError::ExecFieldNotFound);
            };

            let mut args: Vec<String> = Vec::new();

            for (arg, quoted) in split_exec(exec)? {
                // Field codes must not be used inside a quoted argument
                if quoted {
                    args.push(arg);
                    continue;
                }

                match ArgOrFieldCode::try_from(arg.as_str()) {
                    Ok(arg) => match arg {
                        ArgOrFieldCode::SingleFileName | ArgOrFieldCode::SingleUrl => {
                            if let Some(arg) = uris.first() {
                                args.push(arg.to_string());
                            }
                        }
                        ArgOrFieldCode::FileList | ArgOrFieldCode::UrlList => {
                            uris.iter().for_each(|uri| args.push(uri.to_string()));
                        }
                        ArgOrFieldCode::IconKey => {
                            if let Some(icon) = this.icon() {
                                args.push(icon.to_string());
                            }
                        }
                        ArgOrFieldCode::TranslatedName => {
                            if let Some(name) = DesktopEntry::localized_entry(
                                this.ubuntu_gettext_domain.as_deref(),
                                this.groups.desktop_entry(),
                                "Name",
                                locales,
                            ) {
                                args.push(name.to_string());
                            }
                        }
                        ArgOrFieldCode::DesktopFileLocation => {
                            args.push(this.path.to_string_lossy().to_string());
                        }
                        ArgOrFieldCode::Arg(arg) => {
                            args.push(arg.to_string());
                        }
                    },
                    Err(e) => {
                        log::error!("{}", e);
                    }
                }
            }

            if args.is_empty() {
                return Err(ExecError::ExecFieldIsEmpty);
            }

            if args.first().unwrap().contains('=') {
                return Err(ExecError::WrongFormat("equal sign detected".into()));
            }

            Ok(args)
        }

        inner(self, exec, uris, &mut locales.iter().map(AsRef::as_ref))
    }
}

// Splits an Exec value into arguments following the quoting rules described in
// https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html
//
// The general string escapes (\s, \\, ...) must already be undone, which the
// decoder does for every value. Each argument is returned with a flag telling
// whether any part of it was quoted.
fn split_exec(exec: &str) -> Result<Vec<(String, bool)>, ExecError> {
    let unmatched_quote = || ExecError::WrongFormat("unmatched quote".into());

    let mut args = Vec::new();
    let mut arg = String::new();
    let mut in_arg = false;
    let mut quoted = false;
    let mut chars = exec.chars();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                in_arg = true;
                quoted = true;

                loop {
                    match chars.next().ok_or_else(unmatched_quote)? {
                        '"' => break,
                        '\\' => match chars.next().ok_or_else(unmatched_quote)? {
                            c @ ('"' | '`' | '$' | '\\') => arg.push(c),
                            c => {
                                arg.push('\\');
                                arg.push(c);
                            }
                        },
                        c => arg.push(c),
                    }
                }
            }
            // not defined by the spec outside of quotes, so behave like GLib and KDE
            // which follow shell rules and treat it as escaping the next character
            '\\' => {
                in_arg = true;
                arg.push(chars.next().unwrap_or('\\'));
            }
            c if c.is_ascii_whitespace() => {
                if in_arg {
                    args.push((std::mem::take(&mut arg), quoted));
                    in_arg = false;
                    quoted = false;
                }
            }
            c => {
                in_arg = true;
                arg.push(c);
            }
        }
    }

    if in_arg {
        args.push((arg, quoted));
    }

    Ok(args)
}

// either a command line argument or a field-code as described
// in https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#exec-variables
enum ArgOrFieldCode<'a> {
    SingleFileName,
    FileList,
    SingleUrl,
    UrlList,
    IconKey,
    TranslatedName,
    DesktopFileLocation,
    Arg(&'a str),
}

#[derive(Debug, Error)]
enum ExecErrorInternal<'a> {
    #[error("Unknown field code: '{0}'")]
    UnknownFieldCode(&'a str),

    #[error("Deprecated field code: '{0}'")]
    DeprecatedFieldCode(&'a str),
}

impl<'a> TryFrom<&'a str> for ArgOrFieldCode<'a> {
    type Error = ExecErrorInternal<'a>;

    // todo: handle escaping
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "%f" => Ok(ArgOrFieldCode::SingleFileName),
            "%F" => Ok(ArgOrFieldCode::FileList),
            "%u" => Ok(ArgOrFieldCode::SingleUrl),
            "%U" => Ok(ArgOrFieldCode::UrlList),
            "%i" => Ok(ArgOrFieldCode::IconKey),
            "%c" => Ok(ArgOrFieldCode::TranslatedName),
            "%k" => Ok(ArgOrFieldCode::DesktopFileLocation),
            "%d" | "%D" | "%n" | "%N" | "%v" | "%m" => {
                Err(ExecErrorInternal::DeprecatedFieldCode(value))
            }
            other if other.starts_with('%') => Err(ExecErrorInternal::UnknownFieldCode(other)),
            other => Ok(ArgOrFieldCode::Arg(other)),
        }
    }
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use crate::{DesktopEntry, get_languages_from_env};

    use super::ExecError;

    #[test]
    fn should_return_unmatched_quote_error() {
        let path = PathBuf::from("tests_entries/exec/unmatched-quotes.desktop");
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, Some(&locales)).unwrap();
        let result = de.parse_exec_with_uris(&[], &locales);

        assert!(matches!(result.unwrap_err(), ExecError::WrongFormat(..)));
    }

    #[test]
    fn should_fail_if_exec_string_is_empty() {
        let path = PathBuf::from("tests_entries/exec/empty-exec.desktop");
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, Some(&locales)).unwrap();
        let result = de.parse_exec_with_uris(&[], &locales);

        assert!(matches!(result.unwrap_err(), ExecError::ExecFieldIsEmpty));
    }

    #[test]
    fn should_keep_quoted_path_with_spaces() {
        let path = PathBuf::from("tests_entries/exec/gog_com-Dead_Cells_1.desktop");
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, Some(&locales)).unwrap();
        let result = de.parse_exec();

        assert_eq!(
            result.unwrap(),
            vec![
                "/home/abc/GOG Games/Dead Cells/start.sh".to_string(),
                "".to_string()
            ]
        );

        assert_eq!(
            de.icon(),
            Some("/home/abc/GOG Games/Dead Cells/support/icon.png")
        );

        assert_eq!(de.path(), Some("/home/abc/GOG Games/Dead Cells"));
    }

    #[test]
    fn should_unescape_before_quoting() {
        let path = PathBuf::from("tests_entries/exec/emacsclient.desktop");
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, Some(&locales)).unwrap();
        let script = "if [ -n \"$*\" ]; then exec /usr/bin/emacsclient --alternate-editor= --reuse-frame \"$@\"; else exec /usr/bin/emacsclient --alternate-editor= --create-frame; fi";

        assert_eq!(
            de.parse_exec_with_uris(&[], &locales).unwrap(),
            vec!["sh", "-c", script, "sh"]
        );

        assert_eq!(
            de.parse_exec_with_uris(&["a.txt", "b.txt"], &locales)
                .unwrap(),
            vec!["sh", "-c", script, "sh", "a.txt", "b.txt"]
        );
    }

    #[test]
    fn should_exec_simple_command() {
        let path = PathBuf::from("tests_entries/exec/alacritty-simple.desktop");
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, Some(&locales)).unwrap();
        let result = de.parse_exec_with_uris(&[], &locales);

        assert!(result.is_ok());
    }

    #[test]
    fn should_exec_complex_command() {
        let path = PathBuf::from("tests_entries/exec/non-terminal-cmd.desktop");
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, Some(&locales)).unwrap();
        let result = de.parse_exec_with_uris(&[], &locales);

        assert!(result.is_ok());
    }

    #[test]
    fn should_exec_terminal_command() {
        let path = PathBuf::from("tests_entries/exec/non-terminal-cmd.desktop");
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, Some(&locales)).unwrap();
        let result = de.parse_exec_with_uris(&[], &locales);

        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Needs a desktop environment with nvim installed, run locally only"]
    fn should_parse_exec_with_field_codes() {
        let path = PathBuf::from("/usr/share/applications/nvim.desktop");
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, Some(&locales)).unwrap();
        let result = de.parse_exec_with_uris(&["src/lib.rs"], &locales);

        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Needs a desktop environment with gnome Books installed, run locally only"]
    fn should_parse_exec_with_dbus() {
        let path = PathBuf::from("/usr/share/applications/org.gnome.Books.desktop");
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, Some(&locales)).unwrap();
        let result = de.parse_exec_with_uris(&["src/lib.rs"], &locales);

        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Needs a desktop environment with Nautilus installed, run locally only"]
    fn should_parse_exec_with_dbus_and_field_codes() {
        let path = PathBuf::from("/usr/share/applications/org.gnome.Nautilus.desktop");
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, Some(&locales)).unwrap();
        let _result = de.parse_exec_with_uris(&[], &locales);
        let path = std::env::current_dir().unwrap();
        let path = path.to_string_lossy();
        let path = format!("file:///{path}");
        let result = de.parse_exec_with_uris(&[path.as_str()], &locales);

        assert!(result.is_ok());
    }
}
