mod decoder;
mod iter;
mod owned;

pub use self::decoder::decode;
pub use self::iter::Iter;
pub use self::owned::DesktopEntryBuf;

use std::collections::BTreeMap;
use std::path::PathBuf;

pub type Group<'a> = &'a str;
pub type Groups<'a> = BTreeMap<Group<'a>, KeyMap<'a>>;
pub type Key<'a> = &'a str;
pub type KeyMap<'a> = BTreeMap<Key<'a>, (Value<'a>, LocaleMap<'a>)>;
pub type Locale<'a> = &'a str;
pub type LocaleMap<'a> = BTreeMap<Locale<'a>, Value<'a>>;
pub type Value<'a> = &'a str;

#[derive(Debug)]
pub struct DesktopEntry<'a> {
    pub path: PathBuf,
    pub groups: Groups<'a>,
}

impl<'a> DesktopEntry<'a> {
    pub fn action_entry(&self, action: &str, key: &str, locale: Option<&str>) -> Option<&'a str> {
        let group = self
            .groups
            .get(["Desktop Action ", action].concat().as_str());

        Self::localized_entry(group, key, locale)
    }

    pub fn action_exec(&self, action: &str) -> Option<&'a str> {
        self.action_entry(action, "Exec", None)
    }

    pub fn action_name(&self, action: &str, locale: Option<&str>) -> Option<&'a str> {
        self.action_entry(action, "Name", locale)
    }

    pub fn actions(&self) -> Option<&'a str> {
        self.desktop_entry("Actions", None)
    }

    pub fn categories(&self) -> Option<&'a str> {
        self.desktop_entry("Categories", None)
    }

    pub fn comment(&self, locale: Option<&str>) -> Option<&'a str> {
        self.desktop_entry("Comment", locale)
    }

    pub fn desktop_entry(&self, key: &str, locale: Option<&str>) -> Option<&'a str> {
        Self::localized_entry(self.groups.get("Desktop Entry"), key, locale)
    }

    pub fn exec(&self) -> Option<&'a str> {
        self.desktop_entry("Exec", None)
    }

    pub fn icon(&self) -> Option<&'a str> {
        self.desktop_entry("Icon", None)
    }

    pub fn id(&self) -> &str {
        self.path
            .file_name()
            .expect("desktop entry without a file name")
            .to_str()
            .expect("desktop entry has a filename that is not UTF-8")
    }

    pub fn keywords(&self) -> Option<&'a str> {
        self.desktop_entry("Keywords", None)
    }

    pub fn mime_type(&self) -> Option<&'a str> {
        self.desktop_entry("MimeType", None)
    }

    pub fn name(&self, locale: Option<&str>) -> Option<&'a str> {
        self.desktop_entry("Name", locale)
    }

    pub fn no_display(&self) -> bool {
        self.desktop_entry_bool("NoDisplay")
    }

    pub fn prefers_non_default_gpu(&self) -> bool {
        self.desktop_entry_bool("PrefersNonDefaultGPU")
    }

    pub fn startup_notify(&self) -> bool {
        self.desktop_entry_bool("StartupNotify")
    }

    pub fn terminal(&self) -> bool {
        self.desktop_entry_bool("Terminal")
    }

    pub fn type_(&self) -> Option<&'a str> {
        self.desktop_entry("Type", None)
    }

    fn desktop_entry_bool(&self, key: &str) -> bool {
        self.desktop_entry(key, None).map_or(false, |v| v == "true")
    }

    fn localized_entry(
        group: Option<&KeyMap<'a>>,
        key: &str,
        locale: Option<&str>,
    ) -> Option<&'a str> {
        group.and_then(|group| group.get(key)).and_then(|key| {
            locale
                .and_then(|locale| key.1.get(locale).cloned())
                .or_else(|| Some(key.0))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathSource {
    Local,
    LocalDesktop,
    LocalFlatpak,
    System,
    SystemFlatpak,
    SystemSnap,
}

pub fn default_paths() -> Vec<(PathSource, PathBuf)> {
    let home_dir = dirs::home_dir().unwrap();

    vec![
        (PathSource::LocalDesktop, home_dir.join("Desktop")),
        (
            PathSource::LocalFlatpak,
            home_dir.join(".local/share/flatpak/exports/share/applications"),
        ),
        (
            PathSource::Local,
            home_dir.join(".local/share/applications"),
        ),
        (
            PathSource::SystemSnap,
            PathBuf::from("/var/lib/snapd/desktop/applications"),
        ),
        (
            PathSource::SystemFlatpak,
            PathBuf::from("/var/lib/flatpak/exports/share/applications"),
        ),
        (PathSource::System, PathBuf::from("/usr/share/applications")),
    ]
}
