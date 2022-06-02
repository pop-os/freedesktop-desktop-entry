// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#[macro_use]
extern crate thiserror;

pub mod exec;
mod iter;

pub use self::iter::Iter;
use std::borrow::Cow;
use std::collections::BTreeMap;

use std::path::{Path, PathBuf};
use xdg::BaseDirectories;

pub type Group<'a> = &'a str;
pub type Groups<'a> = BTreeMap<Group<'a>, KeyMap<'a>>;
pub type Key<'a> = &'a str;
pub type KeyMap<'a> = BTreeMap<Key<'a>, (Value<'a>, LocaleMap<'a>)>;
pub type Locale<'a> = &'a str;
pub type LocaleMap<'a> = BTreeMap<Locale<'a>, Value<'a>>;
pub type Value<'a> = &'a str;

#[derive(Debug, Copy, Clone, Error, PartialEq, Eq)]
pub enum DecodeError {
    #[error("path does not contain a valid app ID")]
    AppID,
}

#[derive(Debug)]
pub struct DesktopEntry<'a> {
    pub appid: &'a str,
    pub groups: Groups<'a>,
    pub path: &'a Path,
    pub ubuntu_gettext_domain: Option<&'a str>,
}

impl<'a> DesktopEntry<'a> {
    pub fn action_entry(&'a self, action: &str, key: &str) -> Option<&'a str> {
        let group = self
            .groups
            .get(["Desktop Action ", action].concat().as_str());

        Self::entry(group, key)
    }

    pub fn action_entry_localized(
        &'a self,
        action: &str,
        key: &str,
        locale: Option<&str>,
    ) -> Option<Cow<'a, str>> {
        let group = self
            .groups
            .get(["Desktop Action ", action].concat().as_str());

        Self::localized_entry(self.ubuntu_gettext_domain, group, key, locale)
    }

    pub fn action_exec(&'a self, action: &str) -> Option<&'a str> {
        self.action_entry(action, "Exec")
    }

    pub fn action_name(&'a self, action: &str, locale: Option<&str>) -> Option<Cow<'a, str>> {
        self.action_entry_localized(action, "Name", locale)
    }

    pub fn actions(&'a self) -> Option<&'a str> {
        self.desktop_entry("Actions")
    }

    pub fn categories(&'a self) -> Option<&'a str> {
        self.desktop_entry("Categories")
    }

    pub fn comment(&'a self, locale: Option<&str>) -> Option<Cow<'a, str>> {
        self.desktop_entry_localized("Comment", locale)
    }

    pub fn decode(path: &'a Path, input: &'a str) -> Result<DesktopEntry<'a>, DecodeError> {
        let appid = path
            .file_stem()
            .ok_or(DecodeError::AppID)?
            .to_str()
            .ok_or(DecodeError::AppID)?;

        let mut groups = Groups::new();

        let mut active_group = "";

        let mut ubuntu_gettext_domain = None;

        for mut line in input.lines() {
            line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let line_bytes = line.as_bytes();

            if line_bytes[0] == b'[' {
                if let Some(end) = memchr::memrchr(b']', &line_bytes[1..]) {
                    active_group = &line[1..end + 1];
                }
            } else if let Some(delimiter) = memchr::memchr(b'=', line_bytes) {
                let key = &line[..delimiter];
                let value = &line[delimiter + 1..];

                if key.as_bytes()[key.len() - 1] == b']' {
                    if let Some(start) = memchr::memchr(b'[', key.as_bytes()) {
                        let key_name = &key[..start];
                        let locale = &key[start + 1..key.len() - 1];
                        groups
                            .entry(active_group)
                            .or_insert_with(Default::default)
                            .entry(key_name)
                            .or_insert_with(|| ("", LocaleMap::new()))
                            .1
                            .insert(locale, value);

                        continue;
                    }
                }

                if key == "X-Ubuntu-Gettext-Domain" {
                    ubuntu_gettext_domain = Some(value);
                    continue;
                }

                groups
                    .entry(active_group)
                    .or_insert_with(Default::default)
                    .entry(key)
                    .or_insert_with(|| ("", BTreeMap::new()))
                    .0 = value;
            }
        }

        Ok(DesktopEntry {
            appid,
            groups,
            path,
            ubuntu_gettext_domain,
        })
    }

    pub fn desktop_entry(&'a self, key: &str) -> Option<&'a str> {
        Self::entry(self.groups.get("Desktop Entry"), key)
    }

    pub fn desktop_entry_localized(
        &'a self,
        key: &str,
        locale: Option<&str>,
    ) -> Option<Cow<'a, str>> {
        Self::localized_entry(
            self.ubuntu_gettext_domain,
            self.groups.get("Desktop Entry"),
            key,
            locale,
        )
    }

    pub fn exec(&'a self) -> Option<&'a str> {
        self.desktop_entry("Exec")
    }

    pub fn flatpak(&'a self) -> Option<&'a str> {
        self.desktop_entry("X-Flatpak")
    }

    pub fn generic_name(&'a self, locale: Option<&str>) -> Option<Cow<'a, str>> {
        self.desktop_entry_localized("GenericName", locale)
    }

    pub fn icon(&'a self) -> Option<&'a str> {
        self.desktop_entry("Icon")
    }

    pub fn id(&'a self) -> &str {
        self.appid
    }

    pub fn keywords(&'a self) -> Option<Cow<'a, str>> {
        self.desktop_entry_localized("Keywords", None)
    }

    pub fn mime_type(&'a self) -> Option<&'a str> {
        self.desktop_entry("MimeType")
    }

    pub fn name(&'a self, locale: Option<&str>) -> Option<Cow<'a, str>> {
        self.desktop_entry_localized("Name", locale)
    }

    pub fn no_display(&'a self) -> bool {
        self.desktop_entry_bool("NoDisplay")
    }

    pub fn only_show_in(&'a self) -> Option<&'a str> {
        self.desktop_entry("OnlyShowIn")
    }

    pub fn prefers_non_default_gpu(&'a self) -> bool {
        self.desktop_entry_bool("PrefersNonDefaultGPU")
    }

    pub fn startup_notify(&'a self) -> bool {
        self.desktop_entry_bool("StartupNotify")
    }

    pub fn startup_wm_class(&'a self) -> Option<&'a str> {
        self.desktop_entry("StartupWMClass")
    }

    pub fn terminal(&'a self) -> bool {
        self.desktop_entry_bool("Terminal")
    }

    pub fn type_(&'a self) -> Option<&'a str> {
        self.desktop_entry("Type")
    }

    fn desktop_entry_bool(&'a self, key: &str) -> bool {
        self.desktop_entry(key).map_or(false, |v| v == "true")
    }

    fn entry(group: Option<&'a KeyMap<'a>>, key: &str) -> Option<&'a str> {
        group.and_then(|group| group.get(key)).map(|key| key.0)
    }

    fn localized_entry(
        ubuntu_gettext_domain: Option<&'a str>,
        group: Option<&'a KeyMap<'a>>,
        key: &str,
        locale: Option<&str>,
    ) -> Option<Cow<'a, str>> {
        group.and_then(|group| group.get(key)).and_then(|key| {
            locale
                .and_then(|locale| match key.1.get(locale).cloned() {
                    Some(value) => Some(value),
                    None => {
                        if let Some(pos) = locale.find('_') {
                            key.1.get(&locale[..pos]).cloned()
                        } else {
                            None
                        }
                    }
                })
                .map(Cow::Borrowed)
                .or_else(|| ubuntu_gettext_domain.map(|domain| Cow::Owned(dgettext(domain, key.0))))
                .or(Some(Cow::Borrowed(key.0)))
        })
    }
}

use std::fmt::{self, Display, Formatter};

impl<'a> Display for DesktopEntry<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        for (group, keymap) in &self.groups {
            let _ = writeln!(formatter, "[{}]", group);

            for (key, (value, localizations)) in keymap {
                let _ = writeln!(formatter, "{}={}", key, value);
                for (locale, localized) in localizations {
                    let _ = writeln!(formatter, "{}[{}]={}", key, locale, localized);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PathSource {
    Local,
    LocalDesktop,
    LocalFlatpak,
    LocalNix,
    Nix,
    System,
    SystemLocal,
    SystemFlatpak,
    SystemSnap,
    Other(String),
}

impl PathSource {
    /// Attempts to determine the PathSource for a given Path.
    /// Note that this is a best-effort guesting function, and its results should be treated as
    /// such (e.g.: non-canonical).
    pub fn guess_from(path: &Path) -> PathSource {
        let base_dirs = BaseDirectories::new().unwrap();
        let data_home = base_dirs.get_data_home();

        if path.starts_with("/usr/share") {
            PathSource::System
        } else if path.starts_with("/usr/local/share") {
            PathSource::SystemLocal
        } else if path.starts_with("/var/lib/flatpak") {
            PathSource::SystemFlatpak
        } else if path.starts_with("/var/lib/snapd") {
            PathSource::SystemSnap
        } else if path.starts_with("/nix/var/nix/profiles/default")
            || path.starts_with("/nix/store")
        {
            PathSource::Nix
        } else if path.to_string_lossy().contains("/flatpak/") {
            PathSource::LocalFlatpak
        } else if path.starts_with(&data_home.as_path()) {
            PathSource::Local
        } else if path.starts_with("/nix/var/nix/profiles/per-user")
            || path.to_string_lossy().contains(".nix")
        {
            PathSource::LocalNix
        } else {
            PathSource::Other(String::from("unknown"))
        }
    }
}

/// Returns the default paths in which desktop entries should be searched for based on the current
/// environment.
///
/// Panics in case determining the current home directory fails.
pub fn default_paths() -> Vec<PathBuf> {
    let base_dirs = BaseDirectories::new().unwrap();
    let mut data_dirs = base_dirs.get_data_dirs();
    data_dirs.push(base_dirs.get_data_home());

    data_dirs.iter().map(|d| d.join("applications")).collect()
}

fn dgettext(domain: &str, message: &str) -> String {
    use gettextrs::{setlocale, LocaleCategory};
    setlocale(LocaleCategory::LcAll, "");
    gettextrs::dgettext(domain, message)
}
