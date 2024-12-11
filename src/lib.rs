// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod decoder;
mod iter;

mod exec;
use cached::proc_macro::cached;
pub use exec::ExecError;

pub mod matching;
pub use decoder::DecodeError;

pub use self::iter::Iter;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use std::path::{Path, PathBuf};
use xdg::BaseDirectories;

pub type GroupName = String;
#[derive(Debug, Clone, Default)]
pub struct Groups(pub BTreeMap<GroupName, Group>);

impl Groups {
    pub fn desktop_entry(&self) -> Option<&Group> {
        self.0.get("Desktop Entry")
    }

    pub fn group(&self, key: &str) -> Option<&Group> {
        self.0.get(key)
    }
}

pub type Key = String;
#[derive(Debug, Clone, Default)]
pub struct Group(pub BTreeMap<Key, (Value, LocaleMap)>);

impl Group {
    pub fn localized_entry<L: AsRef<str>>(&self, key: &str, locales: &[L]) -> Option<&str> {
        let (default_value, locale_map) = self.0.get(key)?;

        for locale in locales {
            match locale_map.get(locale.as_ref()) {
                Some(value) => return Some(value),
                None => {
                    if let Some(pos) = memchr::memchr(b'_', locale.as_ref().as_bytes()) {
                        if let Some(value) = locale_map.get(&locale.as_ref()[..pos]) {
                            return Some(value);
                        }
                    }
                }
            }
        }

        Some(default_value)
    }

    pub fn entry(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|key| key.0.as_ref())
    }

    pub fn entry_bool(&self, key: &str) -> Option<bool> {
        match self.entry(key)? {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }
}

pub type Locale = String;
pub type LocaleMap = BTreeMap<Locale, Value>;
pub type Value = String;

#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub appid: String,
    pub groups: Groups,
    pub path: PathBuf,
    pub ubuntu_gettext_domain: Option<String>,
}

impl Eq for DesktopEntry {}

impl Hash for DesktopEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.appid.hash(state);
    }
}

impl PartialEq for DesktopEntry {
    fn eq(&self, other: &Self) -> bool {
        self.appid == other.appid
    }
}

impl DesktopEntry {
    /// Construct a new [`DesktopEntry`] from an appid. The name field will be
    /// set to that appid.
    pub fn from_appid(appid: String) -> DesktopEntry {
        let name = appid.split('.').next_back().unwrap_or(&appid).to_string();

        let mut de = DesktopEntry {
            appid,
            groups: Groups::default(),
            path: PathBuf::from(""),
            ubuntu_gettext_domain: None,
        };
        de.add_desktop_entry("Name".to_string(), name);
        de
    }
}

impl DesktopEntry {
    pub fn id(&self) -> &str {
        self.appid.as_ref()
    }

    /// A desktop entry field if any field under the `[Desktop Entry]` section.
    pub fn desktop_entry(&self, key: &str) -> Option<&str> {
        self.groups.desktop_entry()?.entry(key)
    }

    pub fn desktop_entry_localized<'a, L: AsRef<str>>(
        &'a self,
        key: &str,
        locales: &[L],
    ) -> Option<Cow<'a, str>> {
        Self::localized_entry(
            self.ubuntu_gettext_domain.as_deref(),
            self.groups.desktop_entry(),
            key,
            locales,
        )
    }

    /// Insert a new field to this [`DesktopEntry`], in the `[Desktop Entry]` section, removing
    /// the previous value and locales in any.
    pub fn add_desktop_entry(&mut self, key: String, value: String) {
        let action_key = "Desktop Entry";
        let value = (value, LocaleMap::default());

        match self.groups.0.get_mut(action_key) {
            Some(keymap) => {
                keymap.0.insert(key, value);
            }
            None => {
                let mut keymap = Group::default();
                keymap.0.insert(key, value);
                self.groups.0.insert(action_key.to_string(), keymap);
            }
        }
    }

    pub fn name<L: AsRef<str>>(&self, locales: &[L]) -> Option<Cow<'_, str>> {
        self.desktop_entry_localized("Name", locales)
    }

    pub fn generic_name<L: AsRef<str>>(&self, locales: &[L]) -> Option<Cow<'_, str>> {
        self.desktop_entry_localized("GenericName", locales)
    }

    pub fn icon(&self) -> Option<&str> {
        self.desktop_entry("Icon")
    }

    /// This is an human readable description of the desktop file.
    pub fn comment<L: AsRef<str>>(&self, locales: &[L]) -> Option<Cow<str>> {
        self.desktop_entry_localized("Comment", locales)
    }

    pub fn exec(&self) -> Option<&str> {
        self.desktop_entry("Exec")
    }

    /// Return categories
    pub fn categories(&self) -> Option<Vec<&str>> {
        self.desktop_entry("Categories")
            .map(|e| e.split(';').collect())
    }

    /// Return keywords
    pub fn keywords<L: AsRef<str>>(&self, locales: &[L]) -> Option<Vec<Cow<str>>> {
        self.localized_entry_splitted(self.groups.desktop_entry(), "Keywords", locales)
    }

    /// Return mime types
    pub fn mime_type(&self) -> Option<Vec<&str>> {
        self.desktop_entry("MimeType")
            .map(|e| e.split(';').collect())
    }

    pub fn no_display(&self) -> bool {
        self.desktop_entry_bool("NoDisplay")
    }

    pub fn only_show_in(&self) -> Option<Vec<&str>> {
        self.desktop_entry("OnlyShowIn")
            .map(|e| e.split(';').collect())
    }

    pub fn not_show_in(&self) -> Option<Vec<&str>> {
        self.desktop_entry("NotShowIn")
            .map(|e| e.split(';').collect())
    }

    pub fn flatpak(&self) -> Option<&str> {
        self.desktop_entry("X-Flatpak")
    }

    pub fn prefers_non_default_gpu(&self) -> bool {
        self.desktop_entry_bool("PrefersNonDefaultGPU")
    }

    pub fn startup_notify(&self) -> bool {
        self.desktop_entry_bool("StartupNotify")
    }

    pub fn startup_wm_class(&self) -> Option<&str> {
        self.desktop_entry("StartupWMClass")
    }

    pub fn terminal(&self) -> bool {
        self.desktop_entry_bool("Terminal")
    }

    pub fn type_(&self) -> Option<&str> {
        self.desktop_entry("Type")
    }

    pub fn actions(&self) -> Option<Vec<&str>> {
        self.desktop_entry("Actions")
            .map(|e| e.split(';').collect())
    }

    /// An action is defined as `[Desktop Action actions-name]` where `action-name`
    /// is defined in the `Actions` field of `[Desktop Entry]`.
    /// Example: to get the `Name` field of this `new-window` action
    /// ```txt
    /// [Desktop Action new-window]
    /// Name=Open a New Window
    /// ```
    /// you will need to call
    /// ```ignore
    /// entry.action_entry("new-window", "Name")
    /// ```
    pub fn action_entry(&self, action: &str, key: &str) -> Option<&str> {
        self.groups
            .group(["Desktop Action ", action].concat().as_str())?
            .entry(key)
    }

    pub fn action_entry_localized<L: AsRef<str>>(
        &self,
        action: &str,
        key: &str,
        locales: &[L],
    ) -> Option<Cow<'_, str>> {
        let group = self
            .groups
            .group(["Desktop Action ", action].concat().as_str());

        Self::localized_entry(self.ubuntu_gettext_domain.as_deref(), group, key, locales)
    }

    pub fn action_name<L: AsRef<str>>(&self, action: &str, locales: &[L]) -> Option<Cow<str>> {
        self.action_entry_localized(action, "Name", locales)
    }

    pub fn action_exec(&self, action: &str) -> Option<&str> {
        self.action_entry(action, "Exec")
    }

    fn desktop_entry_bool(&self, key: &str) -> bool {
        self.desktop_entry(key).map_or(false, |v| v == "true")
    }

    pub(crate) fn localized_entry<'a, L: AsRef<str>>(
        ubuntu_gettext_domain: Option<&str>,
        group: Option<&'a Group>,
        key: &str,
        locales: &[L],
    ) -> Option<Cow<'a, str>> {
        let (default_value, locale_map) = group?.0.get(key)?;

        for locale in locales {
            match locale_map.get(locale.as_ref()) {
                Some(value) => return Some(Cow::Borrowed(value)),
                None => {
                    if let Some(pos) = memchr::memchr(b'_', locale.as_ref().as_bytes()) {
                        if let Some(value) = locale_map.get(&locale.as_ref()[..pos]) {
                            return Some(Cow::Borrowed(value));
                        }
                    }
                }
            }
        }
        if let Some(domain) = ubuntu_gettext_domain {
            return Some(Cow::Owned(dgettext(domain, default_value)));
        }
        Some(Cow::Borrowed(default_value))
    }

    pub fn localized_entry_splitted<'a, L: AsRef<str>>(
        &self,
        group: Option<&'a Group>,
        key: &str,
        locales: &[L],
    ) -> Option<Vec<Cow<'a, str>>> {
        let (default_value, locale_map) = group?.0.get(key)?;

        for locale in locales {
            match locale_map.get(locale.as_ref()) {
                Some(value) => {
                    return Some(value.split(';').map(Cow::Borrowed).collect());
                }
                None => {
                    if let Some(pos) = memchr::memchr(b'_', locale.as_ref().as_bytes()) {
                        if let Some(value) = locale_map.get(&locale.as_ref()[..pos]) {
                            return Some(value.split(';').map(Cow::Borrowed).collect());
                        }
                    }
                }
            }
        }
        if let Some(domain) = &self.ubuntu_gettext_domain {
            return Some(
                dgettext(domain, default_value)
                    .split(';')
                    .map(|e| Cow::Owned(e.to_string()))
                    .collect(),
            );
        }

        Some(default_value.split(';').map(Cow::Borrowed).collect())
    }
}

use std::fmt::{self, Display, Formatter};

impl Display for DesktopEntry {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        for (group_name, group) in &self.groups.0 {
            let _ = writeln!(formatter, "[{}]", group_name);

            for (key, (value, localizations)) in &group.0 {
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
/// Paths are sorted by priority.
///
/// Panics in case determining the current home directory fails.
pub fn default_paths() -> impl Iterator<Item = PathBuf> {
    let base_dirs = BaseDirectories::new().unwrap();
    let mut data_dirs: Vec<PathBuf> = vec![];
    data_dirs.push(base_dirs.get_data_home());
    data_dirs.append(&mut base_dirs.get_data_dirs());

    data_dirs.into_iter().map(|d| d.join("applications"))
}

pub(crate) fn dgettext(domain: &str, message: &str) -> String {
    use gettextrs::{setlocale, LocaleCategory};
    setlocale(LocaleCategory::LcAll, "");
    gettextrs::dgettext(domain, message)
}

/// Get the configured user language env variables.
/// See https://wiki.archlinux.org/title/Locale#LANG:_default_locale for more information
#[cached]
pub fn get_languages_from_env() -> Vec<String> {
    let mut l = Vec::new();

    if let Ok(lang) = std::env::var("LANG") {
        l.push(lang);
    }

    if let Ok(lang) = std::env::var("LANGUAGES") {
        lang.split(':').for_each(|lang| {
            l.push(lang.to_owned());
        })
    }

    l
}

pub fn current_desktop() -> Option<Vec<String>> {
    std::env::var("XDG_CURRENT_DESKTOP").ok().map(|x| {
        let x = x.to_ascii_lowercase();
        if x == "unity" {
            vec!["gnome".to_string()]
        } else {
            x.split(':').map(|e| e.to_string()).collect()
        }
    })
}

#[test]
fn add_field() {
    let appid = "appid";
    let de = DesktopEntry::from_appid(appid.to_string());

    assert_eq!(de.appid, appid);
    assert_eq!(de.name(&[] as &[&str]).unwrap(), appid);

    let s = get_languages_from_env();

    println!("{:?}", s);
}

#[test]
fn env_with_locale() {
    let locales = &["fr_FR"];

    let de = DesktopEntry::from_path(
        PathBuf::from("tests_entries/org.mozilla.firefox.desktop"),
        Some(locales),
    )
    .unwrap();

    assert_eq!(de.generic_name(locales).unwrap(), "Navigateur Web");

    let locales = &["nb"];

    assert_eq!(de.generic_name(locales).unwrap(), "Web Browser");
}
