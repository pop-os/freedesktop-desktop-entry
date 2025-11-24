use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use unicase::Ascii;

#[cfg(feature = "gettext")]
use crate::dgettext;

use crate::{Group, Groups, LocaleMap};

#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub appid: String,
    pub groups: Groups,
    pub path: PathBuf,
    pub ubuntu_gettext_domain: Option<String>,
}

impl Ord for DesktopEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (&self.path, &self.appid).cmp(&(&other.path, &other.appid))
    }
}

impl PartialOrd for DesktopEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.path.cmp(&other.path))
    }
}

impl PartialEq for DesktopEntry {
    fn eq(&self, other: &Self) -> bool {
        (&self.path, &self.appid) == (&other.path, &other.appid)
    }
}

impl Eq for DesktopEntry {}

impl Hash for DesktopEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.appid.hash(state);
    }
}

impl DesktopEntry {
    /// Construct a new [`DesktopEntry`] from an appid. The name field will be
    /// set to that appid.
    #[inline]
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

    /// Entries with a matching `StartupWMClass` should be preferred over those that do not.
    #[inline]
    pub fn matches_wm_class(&self, id: Ascii<&str>) -> bool {
        self.startup_wm_class()
            .is_some_and(|wm_class| wm_class == id)
    }

    /// Match entry by desktop entry file name
    #[inline]
    pub fn matches_id(&self, id: Ascii<&str>) -> bool {
        // If the desktop entry appid matches
        id == self.id()
            // or the path itself matches
            || self.path.file_stem()
                .and_then(|os_str| os_str.to_str())
                .is_some_and(|name| {
                    name == id
                        // Or match by last part of app ID
                        || id.split('.').rev().next().is_some_and(|id| id == name)
                })
    }

    // Match by name specified in desktop entry, which should only be used if a match by ID failed.
    #[inline]
    pub fn matches_name(&self, name: Ascii<&str>) -> bool {
        self.name::<&str>(&[])
            .map(|n| n.as_ref() == name)
            .unwrap_or_default()
    }
}

impl DesktopEntry {
    #[inline]
    pub fn id(&self) -> &str {
        self.appid.as_ref()
    }

    /// A desktop entry field if any field under the `[Desktop Entry]` section.
    #[inline]
    pub fn desktop_entry(&self, key: &str) -> Option<&str> {
        self.groups.desktop_entry()?.entry(key)
    }

    #[inline]
    pub fn desktop_entry_localized<'a, L: AsRef<str>>(
        &'a self,
        key: &str,
        locales: &[L],
    ) -> Option<Cow<'a, str>> {
        Self::localized_entry(
            self.ubuntu_gettext_domain.as_deref(),
            self.groups.desktop_entry(),
            key,
            &mut locales.iter().map(AsRef::as_ref),
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

    #[inline]
    pub fn name<L: AsRef<str>>(&self, locales: &[L]) -> Option<Cow<'_, str>> {
        self.desktop_entry_localized("Name", locales)
    }

    #[inline]
    pub fn generic_name<L: AsRef<str>>(&self, locales: &[L]) -> Option<Cow<'_, str>> {
        self.desktop_entry_localized("GenericName", locales)
    }

    /// Get the full name of an application, and fall back to the name if that fails.
    #[inline]
    pub fn full_name<L: AsRef<str>>(&self, locales: &[L]) -> Option<Cow<'_, str>> {
        self.desktop_entry_localized("X-GNOME-FullName", locales)
            .filter(|name| !name.as_ref().is_empty())
            .or_else(|| self.name(locales))
    }

    #[inline]
    pub fn icon(&self) -> Option<&str> {
        self.desktop_entry("Icon")
    }

    /// This is an human readable description of the desktop file.
    #[inline]
    pub fn comment<'a, L: AsRef<str>>(&'a self, locales: &[L]) -> Option<Cow<'a, str>> {
        self.desktop_entry_localized("Comment", locales)
    }

    #[inline]
    pub fn exec(&self) -> Option<&str> {
        self.desktop_entry("Exec")
    }

    /// Path or name of an executable to check if app is really installed
    #[inline]
    pub fn try_exec(&self) -> Option<&str> {
        self.desktop_entry("TryExec")
    }

    #[inline]
    pub fn dbus_activatable(&self) -> bool {
        self.desktop_entry_bool("DBusActivatable")
    }

    /// Return categories
    #[inline]
    pub fn categories(&self) -> Option<Vec<&str>> {
        self.desktop_entry("Categories")
            .map(|e| e.split(';').collect())
    }

    /// Return keywords
    #[inline]
    pub fn keywords<'a, L: AsRef<str>>(&'a self, locales: &[L]) -> Option<Vec<Cow<'a, str>>> {
        self.localized_entry_splitted(self.groups.desktop_entry(), "Keywords", locales)
    }

    /// Return mime types
    #[inline]
    pub fn mime_type(&self) -> Option<Vec<&str>> {
        self.desktop_entry("MimeType")
            .map(|e| e.split_terminator(';').collect())
    }

    /// List of D-Bus interfaces supported by this application
    #[inline]
    pub fn implements(&self) -> Option<Vec<&str>> {
        self.desktop_entry("Implements")
            .map(|e| e.split(';').collect())
    }

    /// Application exists but shouldn't be shown in menus
    #[inline]
    pub fn no_display(&self) -> bool {
        self.desktop_entry_bool("NoDisplay")
    }

    /// Desktop environments that should display this application
    #[inline]
    pub fn only_show_in(&self) -> Option<Vec<&str>> {
        self.desktop_entry("OnlyShowIn")
            .map(|e| e.split(';').collect())
    }

    /// Desktop environments that should not display this application
    #[inline]
    pub fn not_show_in(&self) -> Option<Vec<&str>> {
        self.desktop_entry("NotShowIn")
            .map(|e| e.split(';').collect())
    }

    /// Treat application as if it does not exist
    #[inline]
    pub fn hidden(&self) -> bool {
        self.desktop_entry_bool("Hidden")
    }

    #[inline]
    pub fn flatpak(&self) -> Option<&str> {
        self.desktop_entry("X-Flatpak")
    }

    #[inline]
    pub fn prefers_non_default_gpu(&self) -> bool {
        self.desktop_entry_bool("PrefersNonDefaultGPU")
    }

    #[inline]
    pub fn startup_notify(&self) -> bool {
        self.desktop_entry_bool("StartupNotify")
    }

    #[inline]
    pub fn startup_wm_class(&self) -> Option<&str> {
        self.desktop_entry("StartupWMClass")
    }

    #[inline]
    pub fn terminal(&self) -> bool {
        self.desktop_entry_bool("Terminal")
    }

    /// The app has a single main window only
    #[inline]
    pub fn single_main_window(&self) -> bool {
        self.desktop_entry_bool("SingleMainWindow")
    }

    /// Working directory to run program in
    #[inline]
    pub fn path(&self) -> Option<&str> {
        self.desktop_entry("Path")
    }

    #[inline]
    pub fn type_(&self) -> Option<&str> {
        self.desktop_entry("Type")
    }

    /// URL to access if entry type is Link
    pub fn url(&self) -> Option<&str> {
        self.desktop_entry("URL")
    }
    /// Supported version of the Desktop Entry Specification
    pub fn version(&self) -> Option<&str> {
        self.desktop_entry("Version")
    }

    #[inline]
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
    #[inline]
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
        #[inline(never)]
        fn inner<'a>(
            this: &'a DesktopEntry,
            action: &str,
            key: &str,
            locales: &mut dyn Iterator<Item = &str>,
        ) -> Option<Cow<'a, str>> {
            let group = this
                .groups
                .group(["Desktop Action ", action].concat().as_str());

            DesktopEntry::localized_entry(
                this.ubuntu_gettext_domain.as_deref(),
                group,
                key,
                locales,
            )
        }

        inner(self, action, key, &mut locales.iter().map(AsRef::as_ref))
    }

    #[inline]
    pub fn action_name<'a, L: AsRef<str>>(
        &'a self,
        action: &str,
        locales: &[L],
    ) -> Option<Cow<'a, str>> {
        self.action_entry_localized(action, "Name", locales)
    }

    #[inline]
    pub fn action_exec(&self, action: &str) -> Option<&str> {
        self.action_entry(action, "Exec")
    }

    #[inline]
    fn desktop_entry_bool(&self, key: &str) -> bool {
        self.desktop_entry(key).map_or(false, |v| v == "true")
    }

    #[inline(never)]
    pub(crate) fn localized_entry<'a>(
        #[cfg_attr(not(feature = "gettext"), allow(unused_variables))]
        ubuntu_gettext_domain: Option<&str>,
        group: Option<&'a Group>,
        key: &str,
        locales: &mut dyn Iterator<Item = &str>,
    ) -> Option<Cow<'a, str>> {
        let (default_value, locale_map) = group?.0.get(key)?;

        for locale in locales {
            match locale_map.get(locale) {
                Some(value) => return Some(Cow::Borrowed(value)),
                None => {
                    if let Some(pos) = memchr::memchr(b'_', locale.as_bytes()) {
                        if let Some(value) = locale_map.get(&locale[..pos]) {
                            return Some(Cow::Borrowed(value));
                        }
                    }
                }
            }
        }
        #[cfg(feature = "gettext")]
        if let Some(domain) = ubuntu_gettext_domain {
            return Some(Cow::Owned(dgettext(domain, default_value)));
        }
        Some(Cow::Borrowed(default_value))
    }

    #[inline(never)]
    pub fn localized_entry_splitted<'a, L: AsRef<str>>(
        &'a self,
        group: Option<&'a Group>,
        key: &str,
        locales: &[L],
    ) -> Option<Vec<Cow<'a, str>>> {
        #[inline(never)]
        fn inner<'a>(
            #[cfg_attr(not(feature = "gettext"), allow(unused_variables))] this: &'a DesktopEntry,
            group: Option<&'a Group>,
            key: &str,
            locales: &mut dyn Iterator<Item = &str>,
        ) -> Option<Vec<Cow<'a, str>>> {
            let (default_value, locale_map) = group?.0.get(key)?;

            for locale in locales {
                match locale_map.get(locale) {
                    Some(value) => {
                        return Some(value.split(';').map(Cow::Borrowed).collect());
                    }
                    None => {
                        if let Some(pos) = memchr::memchr(b'_', locale.as_bytes()) {
                            if let Some(value) = locale_map.get(&locale[..pos]) {
                                return Some(value.split(';').map(Cow::Borrowed).collect());
                            }
                        }
                    }
                }
            }
            #[cfg(feature = "gettext")]
            if let Some(domain) = &this.ubuntu_gettext_domain {
                return Some(
                    dgettext(domain, default_value)
                        .split(';')
                        .map(|e| Cow::Owned(e.to_string()))
                        .collect(),
                );
            }

            Some(default_value.split(';').map(Cow::Borrowed).collect())
        }

        inner(self, group, key, &mut locales.iter().map(AsRef::as_ref))
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nautilus_name_french() {
        let locales = &["fr_FR.UTF-8"];

        let de = DesktopEntry::from_path(
            PathBuf::from("tests_entries/org.gnome.Nautilus.desktop"),
            Some(locales),
        )
        .unwrap();

        assert_eq!(de.name(locales).unwrap(), "Fichiers");
    }
}
