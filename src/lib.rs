// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod decoder;
mod iter;

pub mod matching;

#[cfg(test)]
mod test;

pub use self::iter::Iter;
use std::borrow::Cow;
use std::collections::BTreeMap;

use std::path::{Path, PathBuf};
use xdg::BaseDirectories;

pub type Group<'a> = Cow<'a, str>;
pub type Groups<'a> = BTreeMap<Group<'a>, KeyMap<'a>>;
pub type Key<'a> = Cow<'a, str>;
pub type KeyMap<'a> = BTreeMap<Key<'a>, (Value<'a>, LocaleMap<'a>)>;
pub type Locale<'a> = Cow<'a, str>;
pub type LocaleMap<'a> = BTreeMap<Locale<'a>, Value<'a>>;
pub type Value<'a> = Cow<'a, str>;

#[derive(Debug)]
pub struct DesktopEntry<'a> {
    pub appid: Cow<'a, str>,
    pub groups: Groups<'a>,
    pub path: Cow<'a, Path>,
    pub ubuntu_gettext_domain: Option<Cow<'a, str>>,
}

impl<'a> DesktopEntry<'a> {
    pub fn into_owned(self) -> DesktopEntry<'static> {
        let mut new_groups = Groups::new();

        for (group, key_map) in self.groups {
            let mut new_key_map = KeyMap::new();

            for (key, (value, locale_map)) in key_map {
                let mut new_locale_map = LocaleMap::new();

                for (locale, value) in locale_map {
                    new_locale_map.insert(
                        Cow::Owned(locale.into_owned()),
                        Cow::Owned(value.into_owned()),
                    );
                }

                new_key_map.insert(
                    Cow::Owned(key.into_owned()),
                    (Cow::Owned(value.into_owned()), new_locale_map),
                );
            }

            new_groups.insert(Cow::Owned(group.into_owned()), new_key_map);
        }

        DesktopEntry {
            appid: Cow::Owned(self.appid.into_owned()),
            groups: new_groups,
            ubuntu_gettext_domain: self
                .ubuntu_gettext_domain
                .map(|e| Cow::Owned(e.into_owned())),
            path: Cow::Owned(self.path.into_owned()),
        }
    }
}

impl<'a> DesktopEntry<'a> {
    pub fn action_entry(&'a self, action: &str, key: &str) -> Option<&'a Cow<'a, str>> {
        let group = self
            .groups
            .get(["Desktop Action ", action].concat().as_str());

        Self::entry(group, key)
    }

    pub fn action_entry_localized<L: AsRef<str>>(
        &'a self,
        action: &str,
        key: &str,
        locales: &[L],
    ) -> Option<Cow<'a, str>> {
        let group = self
            .groups
            .get(["Desktop Action ", action].concat().as_str());

        Self::localized_entry(self.ubuntu_gettext_domain.as_deref(), group, key, locales)
    }

    pub fn action_exec(&'a self, action: &str) -> Option<&'a Cow<'a, str>> {
        self.action_entry(action, "Exec")
    }

    pub fn action_name<L: AsRef<str>>(
        &'a self,
        action: &str,
        locales: &[L],
    ) -> Option<Cow<'a, str>> {
        self.action_entry_localized(action, "Name", locales)
    }

    pub fn actions(&'a self) -> Option<&'a str> {
        self.desktop_entry("Actions")
    }

    pub fn categories(&'a self) -> Option<&'a str> {
        self.desktop_entry("Categories")
    }

    pub fn comment<L: AsRef<str>>(&'a self, locales: &[L]) -> Option<Cow<'a, str>> {
        self.desktop_entry_localized("Comment", locales)
    }

    pub fn desktop_entry(&'a self, key: &str) -> Option<&'a str> {
        Self::entry(self.groups.get("Desktop Entry"), key).map(|e| e.as_ref())
    }

    pub fn desktop_entry_localized<L: AsRef<str>>(
        &'a self,
        key: &str,
        locales: &[L],
    ) -> Option<Cow<'a, str>> {
        Self::localized_entry(
            self.ubuntu_gettext_domain.as_deref(),
            self.groups.get("Desktop Entry"),
            key,
            locales,
        )
    }

    pub fn exec(&'a self) -> Option<&'a str> {
        self.desktop_entry("Exec")
    }

    pub fn flatpak(&'a self) -> Option<&'a str> {
        self.desktop_entry("X-Flatpak")
    }

    pub fn generic_name<L: AsRef<str>>(&'a self, locales: &[L]) -> Option<Cow<'a, str>> {
        self.desktop_entry_localized("GenericName", locales)
    }

    pub fn icon(&'a self) -> Option<&'a str> {
        self.desktop_entry("Icon")
    }

    pub fn id(&'a self) -> &'a str {
        self.appid.as_ref()
    }

    pub fn keywords<L: AsRef<str>>(&'a self, locales: &[L]) -> Option<Cow<'a, str>> {
        self.desktop_entry_localized("Keywords", locales)
    }

    pub fn mime_type(&'a self) -> Option<&'a str> {
        self.desktop_entry("MimeType")
    }

    pub fn name<L: AsRef<str>>(&'a self, locales: &[L]) -> Option<Cow<'a, str>> {
        self.desktop_entry_localized("Name", locales)
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

    fn entry(group: Option<&'a KeyMap<'a>>, key: &str) -> Option<&'a Cow<'a, str>> {
        group.and_then(|group| group.get(key)).map(|key| &key.0)
    }

    pub(crate) fn localized_entry<L: AsRef<str>>(
        ubuntu_gettext_domain: Option<&'a str>,
        group: Option<&'a KeyMap<'a>>,
        key: &str,
        locales: &[L],
    ) -> Option<Cow<'a, str>> {
        let Some(group) = group else {
            return None;
        };

        let Some((default_value, locale_map)) = group.get(key) else {
            return None;
        };

        for locale in locales {
            match locale_map.get(locale.as_ref()) {
                Some(value) => return Some(value.clone()),
                None => {
                    if let Some(pos) = locale.as_ref().find('_') {
                        if let Some(value) = locale_map.get(&locale.as_ref()[..pos]) {
                            return Some(value.clone());
                        }
                    }
                }
            }
        }
        if let Some(domain) = ubuntu_gettext_domain {
            return Some(Cow::Owned(dgettext(domain, &default_value)));
        }
        return Some(default_value.clone());
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
            writeln!(formatter)?;
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
        } else if path.starts_with(data_home.as_path()) {
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
    let mut data_dirs: Vec<PathBuf> = vec![];
    data_dirs.push(base_dirs.get_data_home());
    data_dirs.append(&mut base_dirs.get_data_dirs());

    data_dirs.iter().map(|d| d.join("applications")).collect()
}

pub (crate) fn dgettext(domain: &str, message: &str) -> String {
    use gettextrs::{setlocale, LocaleCategory};
    setlocale(LocaleCategory::LcAll, "");
    gettextrs::dgettext(domain, message)
}

// todo: support more variable syntax like fr_FR.
// This will require some work in decode and values query
// for now, just remove the _* part, cause it seems more common

/// Get the configured user language env variables.
/// See https://wiki.archlinux.org/title/Locale#LANG:_default_locale for more information
pub fn get_languages_from_env() -> Vec<String> {
    let mut l = Vec::new();

    if let Ok(mut lang) = std::env::var("LANG") {
        if let Some(start) = memchr::memchr(b'_', lang.as_bytes()) {
            lang.truncate(start);
            l.push(lang)
        } else {
            l.push(lang);
        }
    }

    if let Ok(lang) = std::env::var("LANGUAGES") {
        lang.split(':').for_each(|lang| {
            if let Some(start) = memchr::memchr(b'_', lang.as_bytes()) {
                l.push(lang.split_at(start).0.to_owned())
            } else {
                l.push(lang.to_owned());
            }
        })
    }

    l
}

#[test]
fn locales_env_test() {
    let s = get_languages_from_env();

    println!("{:?}", s);
}
