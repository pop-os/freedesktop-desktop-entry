use crate::exec::error::ExecError;
use crate::exec::graphics::Gpus;
use crate::DesktopEntry;
use std::convert::TryFrom;
use std::path::PathBuf;
use std::process::Command;

pub mod error;
mod graphics;

impl DesktopEntry<'_> {
    /// Execute the given desktop entry `Exec` key with either the default gpu or
    /// the alternative one if available.
    pub fn launch(
        &self,
        filename: Option<&str>,
        filenames: &[&str],
        url: Option<&str>,
        urls: &[&str],
        prefer_non_default_gpu: bool,
    ) -> Result<(), ExecError> {
        let exec = self.exec();
        if exec.is_none() {
            return Err(ExecError::MissingExecKey(&self.path));
        }

        let exec = exec.unwrap();
        let exec = if let Some(unquoted_exec) = exec.strip_prefix('\"') {
            unquoted_exec
                .strip_suffix('\"')
                .ok_or(ExecError::UnmatchedQuote {
                    exec: exec.to_string(),
                })?
        } else {
            exec
        };

        let mut exec_args = vec![];

        for arg in exec.split_ascii_whitespace() {
            let arg = ArgOrFieldCode::try_from(arg)?;
            exec_args.push(arg);
        }

        let exec_args = self.get_args(filename, filenames, url, urls, exec_args);

        if exec_args.is_empty() {
            return Err(ExecError::EmptyExecString);
        }

        let exec_args = exec_args.join(" ");
        let shell = std::env::var("SHELL")?;

        let status = if self.terminal() {
            let (terminal, separator) = detect_terminal();
            let terminal = terminal.to_string_lossy();
            let args = format!("{terminal} {separator} {exec_args}");
            let args = ["-c", &args];
            let mut cmd = Command::new(shell);
            if prefer_non_default_gpu {
                with_non_default_gpu(cmd)
            } else {
                cmd
            }
            .args(args)
            .output()?
            .status
        } else {
            let mut cmd = Command::new(shell);

            if prefer_non_default_gpu {
                with_non_default_gpu(cmd)
            } else {
                cmd
            }
            .args(&["-c", &exec_args])
            .output()?
            .status
        };

        if !status.success() {
            return Err(ExecError::NonZeroStatusCode {
                status: status.code(),
                exec: exec.to_string(),
            });
        }

        Ok(())
    }

    // Replace field code with their values and ignore deprecated and unknown field codes
    fn get_args(
        &self,
        filename: Option<&str>,
        filenames: &[&str],
        url: Option<&str>,
        urls: &[&str],
        exec_args: Vec<ArgOrFieldCode>,
    ) -> Vec<String> {
        exec_args
            .iter()
            .filter_map(|arg| match arg {
                ArgOrFieldCode::SingleFileName => filename.map(|filename| filename.to_string()),
                ArgOrFieldCode::FileList => {
                    if !filenames.is_empty() {
                        Some(filenames.join(" "))
                    } else {
                        None
                    }
                }
                ArgOrFieldCode::SingleUrl => url.map(|url| url.to_string()),
                ArgOrFieldCode::UrlList => {
                    if !urls.is_empty() {
                        Some(urls.join(" "))
                    } else {
                        None
                    }
                }
                ArgOrFieldCode::IconKey => self.icon().map(ToString::to_string),
                ArgOrFieldCode::TranslatedName => {
                    let locale = std::env::var("LANG").ok();
                    if let Some(locale) = locale {
                        let locale = locale.split_once('.').map(|(locale, _)| locale);
                        self.name(locale).map(|locale| locale.to_string())
                    } else {
                        None
                    }
                }
                ArgOrFieldCode::DesktopFileLocation => {
                    Some(self.path.to_string_lossy().to_string())
                }
                // Ignore deprecated field-codes
                ArgOrFieldCode::Arg(arg) => Some(arg.to_string()),
            })
            .collect()
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
    use crate::DesktopEntry;
    use speculoos::prelude::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    #[test]
    fn should_return_unmatched_quote_error() {
        let path = PathBuf::from("tests/entries/unmatched-quotes.desktop");
        let input = fs::read_to_string(&path).unwrap();
        let de = DesktopEntry::decode(path.as_path(), &input).unwrap();
        let result = de.launch(None, &[], None, &[], false);

        assert_that!(result)
            .is_err()
            .matches(|err| matches!(err, ExecError::UnmatchedQuote { exec: _ }));
    }

    #[test]
    fn should_fail_if_exec_string_is_empty() {
        let path = PathBuf::from("tests/entries/empty-exec.desktop");
        let input = fs::read_to_string(&path).unwrap();
        let de = DesktopEntry::decode(Path::new(path.as_path()), &input).unwrap();
        let result = de.launch(None, &[], None, &[], false);

        assert_that!(result)
            .is_err()
            .matches(|err| matches!(err, ExecError::EmptyExecString));
    }

    #[test]
    #[ignore = "Needs a desktop environment and alacritty installed, run locally only"]
    fn should_exec_simple_command() {
        let path = PathBuf::from("tests/entries/alacritty-simple.desktop");
        let input = fs::read_to_string(&path).unwrap();
        let de = DesktopEntry::decode(path.as_path(), &input).unwrap();
        let result = de.launch(None, &[], None, &[], false);

        assert_that!(result).is_ok();
    }

    #[test]
    #[ignore = "Needs a desktop environment and alacritty and mesa-utils installed, run locally only"]
    fn should_exec_complex_command() {
        let path = PathBuf::from("tests/entries/non-terminal-cmd.desktop");
        let input = fs::read_to_string(&path).unwrap();
        let de = DesktopEntry::decode(path.as_path(), &input).unwrap();
        let result = de.launch(None, &[], None, &[], false);

        assert_that!(result).is_ok();
    }

    #[test]
    #[ignore = "Needs a desktop environment and alacritty and mesa-utils installed, run locally only"]
    fn should_exec_terminal_command() {
        let path = PathBuf::from("tests/entries/non-terminal-cmd.desktop");
        let input = fs::read_to_string(&path).unwrap();
        let de = DesktopEntry::decode(path.as_path(), &input).unwrap();
        let result = de.launch(None, &[], None, &[], false);

        assert_that!(result).is_ok();
    }

    #[test]
    fn should_build_command_with_gpu() {
        let cmd = with_non_default_gpu(Command::new("glxgears"));
        assert_that!(cmd.get_envs().collect::<Vec<(_, _)>>()).is_not_empty();
    }
}
