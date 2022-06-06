// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::exec::error::ExecError;
use crate::exec::graphics::Gpus;
use crate::DesktopEntry;
<<<<<<< HEAD
use std::borrow::Cow;
=======
use fork::{daemon, Fork};
>>>>>>> eaa21ff (fix: fork process instead of spawning with std::process::Command)
use std::convert::TryFrom;
use std::os::unix::prelude::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use zbus::blocking::Connection;

mod dbus;
pub mod error;
mod graphics;

impl<'a> DesktopEntry<'a> {
    /// Execute the given desktop entry `Exec` key with either the default gpu or
    /// the alternative one if available.
<<<<<<< HEAD
    pub fn launch<L>(
        &self,
        uris: &[&'a str],
        prefer_non_default_gpu: bool,
        locales: &[L],
    ) -> Result<(), ExecError>
    where
        L: AsRef<str>,
    {
=======
    pub fn launch(&self, uris: &[&str]) -> Result<(), ExecError> {
>>>>>>> 8a6e90a (refactor: remove prefer_non_default arg gpu and use desktop entry attribute instead)
        match Connection::session() {
            Ok(conn) => {
                if self.is_bus_actionable(&conn) {
                    self.dbus_launch(&conn, uris)
                } else {
<<<<<<< HEAD
                    self.shell_launch(uris, prefer_non_default_gpu, locales)
                }
            }
            Err(_) => self.shell_launch(uris, prefer_non_default_gpu, locales),
        }
    }

    fn shell_launch<L>(
        &self,
        uris: &[&str],
        prefer_non_default_gpu: bool,
        locales: &[L],
    ) -> Result<(), ExecError>
    where
        L: AsRef<str>,
    {
=======
                    self.shell_launch(uris)
                }
            }
            Err(_) => self.shell_launch(uris),
        }
    }

    fn shell_launch(&self, uris: &[&str]) -> Result<(), ExecError> {
>>>>>>> 8a6e90a (refactor: remove prefer_non_default arg gpu and use desktop entry attribute instead)
        let exec = self.exec();
        if exec.is_none() {
            return Err(ExecError::MissingExecKey(&self.path));
        }

        let exec = exec.unwrap();

        let mut exec_args = vec![];

        for arg in exec.split_ascii_whitespace() {
            let arg = ArgOrFieldCode::try_from(arg)?;
            exec_args.push(arg);
        }

        let exec_args = self.get_args(uris, exec_args, locales);

        if exec_args.is_empty() {
            return Err(ExecError::EmptyExecString);
        }

        let exec_args = exec_args.join(" ");
        let shell = std::env::var("SHELL")?;

        if let Ok(Fork::Child) = daemon(true, false) {
            if self.terminal() {
                let (terminal, separator) = detect_terminal();
                let terminal = terminal.to_string_lossy();
                let args = format!("{terminal} {separator} {exec_args}");
                let args = ["-c", &args];
                let mut cmd = Command::new(shell);

                if self.prefers_non_default_gpu() {
                    with_non_default_gpu(cmd)
                } else {
                    cmd
                }
                .args(args)
                .exec()
            } else {
                let mut cmd = Command::new(shell);

                if self.prefers_non_default_gpu() {
                    with_non_default_gpu(cmd)
                } else {
                    cmd
                }
                .args(&["-c", &exec_args])
                .exec()
            };
        }

        Ok(())
    }

    // Replace field code with their values and ignore deprecated and unknown field codes
    fn get_args<L>(
        &'a self,
        uris: &[&'a str],
        exec_args: Vec<ArgOrFieldCode<'a>>,
        locales: &[L],
    ) -> Vec<Cow<'a, str>>
    where
        L: AsRef<str>,
    {
        let mut final_args: Vec<Cow<str>> = Vec::new();

        for arg in exec_args {
            match arg {
                ArgOrFieldCode::SingleFileName | ArgOrFieldCode::SingleUrl => {
                    if let Some(arg) = uris.get(0) {
                        final_args.push(Cow::Borrowed(arg));
                    }
                }
                ArgOrFieldCode::FileList | ArgOrFieldCode::UrlList => {
                    uris.into_iter()
                        .for_each(|uri| final_args.push(Cow::Borrowed(uri)));
                }
                ArgOrFieldCode::IconKey => {
                    if let Some(icon) = self.icon() {
                        final_args.push(Cow::Borrowed(&icon));
                    }
                }
                ArgOrFieldCode::TranslatedName => {
                    if let Some(name) = self.name(locales) {
                        final_args.push(name.clone());
                    }
                }
                ArgOrFieldCode::DesktopFileLocation => {
                    final_args.push(self.path.to_string_lossy());
                }
                ArgOrFieldCode::Arg(arg) => {
                    final_args.push(Cow::Borrowed(&arg));
                }
            }
        }

        final_args
    }
}

fn with_non_default_gpu(mut cmd: Command) -> Command {
    let gpus = Gpus::load();
    let gpu = if gpus.is_switchable() {
        gpus.non_default()
    } else {
        gpus.get_default()
    };

    if let Some(gpu) = gpu {
        for (opt, value) in gpu.launch_options() {
            cmd.env(opt, value);
        }
    }

    cmd
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

impl<'a> TryFrom<&'a str> for ArgOrFieldCode<'a> {
    type Error = ExecError<'a>;

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
                Err(ExecError::DeprecatedFieldCode(value.to_string()))
            }
            other if other.starts_with('%') => Err(ExecError::UnknownFieldCode(other.to_string())),
            other => Ok(ArgOrFieldCode::Arg(other)),
        }
    }
}

// Returns the default terminal emulator linked to `/usr/bin/x-terminal-emulator`
// or fallback to gnome terminal, then konsole
fn detect_terminal() -> (PathBuf, &'static str) {
    use std::fs::read_link;

    const SYMLINK: &str = "/usr/bin/x-terminal-emulator";

    if let Ok(found) = read_link(SYMLINK) {
        let arg = if found.to_string_lossy().contains("gnome-terminal") {
            "--"
        } else {
            "-e"
        };

        return (read_link(&found).unwrap_or(found), arg);
    }

    let gnome_terminal = PathBuf::from("/usr/bin/gnome-terminal");
    if gnome_terminal.exists() {
        (gnome_terminal, "--")
    } else {
        (PathBuf::from("/usr/bin/konsole"), "-e")
    }
}

#[cfg(test)]
mod test {
    use crate::exec::error::ExecError;
    use crate::exec::with_non_default_gpu;
    use crate::{get_languages_from_env, DesktopEntry};
    use speculoos::prelude::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    #[test]
<<<<<<< HEAD
    fn should_return_unmatched_quote_error() {
        let path = PathBuf::from("tests/entries/unmatched-quotes.desktop");
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, &locales).unwrap();
        let result = de.launch(&[], false, &locales);

        assert_that!(result)
            .is_err()
            .matches(|err| matches!(err, ExecError::UnmatchedQuote { exec: _ }));
    }

    #[test]
    fn should_fail_if_exec_string_is_empty() {
        let path = PathBuf::from("tests/entries/empty-exec.desktop");
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, &locales).unwrap();
        let result = de.launch(&[], false, &locales);
=======
    fn should_fail_if_exec_string_is_empty() {
        let path = PathBuf::from("tests/entries/empty-exec.desktop");
        let input = fs::read_to_string(&path).unwrap();
        let de = DesktopEntry::decode(Path::new(path.as_path()), &input).unwrap();
        let result = de.launch(&[]);
>>>>>>> 8a6e90a (refactor: remove prefer_non_default arg gpu and use desktop entry attribute instead)

        assert_that!(result)
            .is_err()
            .matches(|err| matches!(err, ExecError::EmptyExecString));
    }

    #[test]
    #[ignore = "Needs a desktop environment and alacritty installed, run locally only"]
    fn should_exec_simple_command() {
        let path = PathBuf::from("tests/entries/alacritty-simple.desktop");
<<<<<<< HEAD
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, &locales).unwrap();
        let result = de.launch(&[], false, &locales);
=======
        let input = fs::read_to_string(&path).unwrap();
        let de = DesktopEntry::decode(path.as_path(), &input).unwrap();
        let result = de.launch(&[]);
>>>>>>> 8a6e90a (refactor: remove prefer_non_default arg gpu and use desktop entry attribute instead)

        assert_that!(result).is_ok();
    }

    #[test]
    #[ignore = "Needs a desktop environment and alacritty and mesa-utils installed, run locally only"]
    fn should_exec_complex_command() {
        let path = PathBuf::from("tests/entries/non-terminal-cmd.desktop");
<<<<<<< HEAD
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, &locales).unwrap();
        let result = de.launch(&[], false, &locales);
=======
        let input = fs::read_to_string(&path).unwrap();
        let de = DesktopEntry::decode(path.as_path(), &input).unwrap();
        let result = de.launch(&[]);
>>>>>>> 8a6e90a (refactor: remove prefer_non_default arg gpu and use desktop entry attribute instead)

        assert_that!(result).is_ok();
    }

    #[test]
    #[ignore = "Needs a desktop environment and alacritty and mesa-utils installed, run locally only"]
    fn should_exec_terminal_command() {
        let path = PathBuf::from("tests/entries/non-terminal-cmd.desktop");
<<<<<<< HEAD
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, &locales).unwrap();
        let result = de.launch(&[], false, &locales);
=======
        let input = fs::read_to_string(&path).unwrap();
        let de = DesktopEntry::decode(path.as_path(), &input).unwrap();
        let result = de.launch(&[]);
>>>>>>> 8a6e90a (refactor: remove prefer_non_default arg gpu and use desktop entry attribute instead)

        assert_that!(result).is_ok();
    }

    #[test]
    #[ignore = "Needs a desktop environment with nvim installed, run locally only"]
    fn should_launch_with_field_codes() {
        let path = PathBuf::from("/usr/share/applications/nvim.desktop");
<<<<<<< HEAD
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, &locales).unwrap();
        let result = de.launch(&["src/lib.rs"], false, &locales);
=======
        let input = fs::read_to_string(&path).unwrap();
        let de = DesktopEntry::decode(path.as_path(), &input).unwrap();
        let result = de.launch(&["src/lib.rs"]);
>>>>>>> 8a6e90a (refactor: remove prefer_non_default arg gpu and use desktop entry attribute instead)

        assert_that!(result).is_ok();
    }

    #[test]
    #[ignore = "Needs a desktop environment with gnome Books installed, run locally only"]
    fn should_launch_with_dbus() {
        let path = PathBuf::from("/usr/share/applications/org.gnome.Books.desktop");
<<<<<<< HEAD
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, &locales).unwrap();
        let result = de.launch(&["src/lib.rs"], false, &locales);
=======
        let input = fs::read_to_string(&path).unwrap();
        let de = DesktopEntry::decode(path.as_path(), &input).unwrap();
        let result = de.launch(&[]);
>>>>>>> 8a6e90a (refactor: remove prefer_non_default arg gpu and use desktop entry attribute instead)

        assert_that!(result).is_ok();
    }

    #[test]
    #[ignore = "Needs a desktop environment with Nautilus installed, run locally only"]
    fn should_launch_with_dbus_and_field_codes() {
        let path = PathBuf::from("/usr/share/applications/org.gnome.Nautilus.desktop");
        let locales = get_languages_from_env();
        let de = DesktopEntry::from_path(path, &locales).unwrap();
        let _result = de.launch(&[], false, &locales);
        let path = std::env::current_dir().unwrap();
        let path = path.to_string_lossy();
        let path = format!("file:///{path}");
<<<<<<< HEAD
        let result = de.launch(&[path.as_str()], false, &locales);
=======
        let result = de.launch(&[path.as_str()]);
>>>>>>> eaa21ff (fix: fork process instead of spawning with std::process::Command)

        assert_that!(result).is_ok();
    }

    #[test]
    fn should_build_command_with_gpu() {
        let cmd = with_non_default_gpu(Command::new("glxgears"));
        assert_that!(cmd.get_envs().collect::<Vec<(_, _)>>()).is_not_empty();
    }
}
