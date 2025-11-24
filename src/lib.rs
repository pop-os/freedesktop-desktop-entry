// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod decoder;
mod desktop_entry;
mod exec;
mod iter;
mod mime_apps;
mod thumbnail;

pub use self::iter::Iter;
pub use decoder::DecodeError;
pub use desktop_entry::DesktopEntry;
pub use exec::ExecError;
pub use mime_apps::MimeApps;
use std::collections::BTreeMap;
use std::hash::Hash;
use std::path::{Path, PathBuf};
pub use thumbnail::Thumbnail;
pub use unicase;
use unicase::Ascii;
use xdg::BaseDirectories;

/// Read all desktop entries on disk into a Vec, with only the given locales retained.
pub fn desktop_entries(locales: &[String]) -> Vec<DesktopEntry> {
    Iter::new(default_paths())
        .filter_map(|p| DesktopEntry::from_path(p, Some(&locales)).ok())
        .collect::<Vec<_>>()
}

/// Case-insensitive search of desktop entries for the given app ID.
///
/// Requires using the `unicase` crate for its `Ascii` case support.
///
/// Searches by name if an ID match could not be found.
pub fn find_app_by_id<'a>(
    entries: &'a [DesktopEntry],
    app_id: Ascii<&str>,
) -> Option<&'a DesktopEntry> {
    // NOTE: Use `cargo run --example find_appid {{wm_app_id}}` to check if the match works.

    // Prefer desktop entries whose startup wm class is a perfect match.
    let match_by_wm_class = entries.iter().find(|entry| entry.matches_wm_class(app_id));

    match_by_wm_class
        // If no suitable wm class was found, search by entry file name.
        .or_else(|| entries.iter().find(|entry| entry.matches_id(app_id)))
        // Otherwise by name specified in the desktop entry.
        .or_else(|| entries.iter().find(|entry| entry.matches_name(app_id)))
        // Or match by the exact exec command
        .or_else(|| {
            entries
                .iter()
                .find(|entry| entry.exec().is_some_and(|exec| exec == app_id))
        })
        // Or match by the first command in the exec
        .or_else(|| {
            entries.iter().find(|entry| {
                entry.exec().is_some_and(|exec| {
                    exec.split_ascii_whitespace()
                        .next()
                        .is_some_and(|exec| exec == app_id)
                })
            })
        })
}

#[derive(Debug, Clone, Default)]
pub struct Groups(pub BTreeMap<GroupName, Group>);
pub type GroupName = String;

impl Groups {
    #[inline]
    pub fn desktop_entry(&self) -> Option<&Group> {
        self.0.get("Desktop Entry")
    }

    #[inline]
    pub fn thumbnailer_entry(&self) -> Option<&Group> {
        self.0.get("Thumbnailer Entry")
    }

    #[inline]
    pub fn group(&self, key: &str) -> Option<&Group> {
        self.0.get(key)
    }
}

pub type Key = String;
#[derive(Debug, Clone, Default)]
pub struct Group(pub BTreeMap<Key, (Value, LocaleMap)>);

impl Group {
    pub fn localized_entry<L: AsRef<str>>(&self, key: &str, locales: &[L]) -> Option<&str> {
        #[inline(never)]
        fn inner<'a>(
            this: &'a Group,
            key: &str,
            locales: &mut dyn Iterator<Item = &str>,
        ) -> Option<&'a str> {
            let (default_value, locale_map) = this.0.get(key)?;

            for locale in locales {
                match locale_map.get(locale) {
                    Some(value) => return Some(value),
                    None => {
                        if let Some(pos) = memchr::memchr(b'_', locale.as_bytes()) {
                            if let Some(value) = locale_map.get(&locale[..pos]) {
                                return Some(value);
                            }
                        }
                    }
                }
            }

            Some(default_value)
        }

        inner(self, key, &mut locales.iter().map(AsRef::as_ref))
    }

    #[inline]
    pub fn entry(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|key| key.0.as_ref())
    }

    #[inline]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconSource {
    Name(String),
    Path(PathBuf),
}

impl IconSource {
    pub fn from_unknown(icon: &str) -> Self {
        let icon_path = Path::new(icon);
        if icon_path.is_absolute() && icon_path.exists() {
            Self::Path(icon_path.into())
        } else {
            Self::Name(icon.into())
        }
    }
}

impl Default for IconSource {
    #[inline]
    fn default() -> Self {
        Self::Name("application-default".to_string())
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
        let base_dirs = BaseDirectories::new();
        let data_home = base_dirs.get_data_home().unwrap();
        let mut nix_state = base_dirs.get_state_home().unwrap();
        nix_state.push("nix");

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
            || path.starts_with("/run/current-system/sw")
        {
            PathSource::Nix
        } else if path.to_string_lossy().contains("/flatpak/") {
            PathSource::LocalFlatpak
        } else if path.starts_with(data_home.as_path()) {
            PathSource::Local
        } else if path.starts_with("/nix/var/nix/profiles/per-user")
            || path.to_string_lossy().contains(".nix")
            || path.starts_with(nix_state.as_path())
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
#[cold]
pub fn default_paths() -> impl Iterator<Item = PathBuf> {
    let base_dirs = BaseDirectories::new();
    let mut data_dirs: Vec<PathBuf> = vec![];
    data_dirs.push(base_dirs.get_data_home().unwrap());
    data_dirs.append(&mut base_dirs.get_data_dirs());

    data_dirs.into_iter().map(|d| d.join("applications"))
}

#[cfg(feature = "gettext")]
#[inline]
pub(crate) fn dgettext(domain: &str, message: &str) -> String {
    use gettextrs::{setlocale, LocaleCategory};
    setlocale(LocaleCategory::LcAll, "");
    gettextrs::dgettext(domain, message)
}

/// Get the configured user language env variables.
/// See https://wiki.archlinux.org/title/Locale#LANG:_default_locale for more information
#[cold]
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

// #[cfg(test)]
// mod mime_list_tests {
//     use super::*;
//     use std::fs::File;
//     use std::io::Write;
//     use tempfile::tempdir;

//     /// Helper that writes a minimal `mimeapps.list` file containing the three
//     /// MIME‑related groups we care about.
//     fn write_test_mimeapps(dir: &std::path::Path) -> PathBuf {
//         let path = dir.join("mimeapps.list");
//         let mut f = File::create(&path).expect("could not create test file");

//         // Header comment – optional but realistic.
//         writeln!(f, "# Test mimeapps.list generated by unit tests").unwrap();

//         // The format follows the freedesktop spec.  Each key is a MIME type,
//         // the value is a semicolon‑separated list of desktop‑file IDs.
//         writeln!(f, "[Default Applications]").unwrap();
//         writeln!(f, "text/plain=com.system76.CosmicEdit.desktop;").unwrap();
//         writeln!(f, "image/png=org.gnome.eog.desktop;").unwrap();

//         writeln!(f, "[Added Associations]").unwrap();
//         writeln!(f, "application/pdf=evince.desktop;okular.desktop;").unwrap();

//         writeln!(f, "[Removed Associations]").unwrap();
//         writeln!(f, "audio/mpeg=some-old-player.desktop;").unwrap();

//         path
//     }

//     #[test]
//     fn can_load_mimeapps_file() {
//         let tmp = tempdir().unwrap();
//         let path = write_test_mimeapps(tmp.path());

//         // `from_path` should succeed and give us a `MimeList`.
//         let ml = MimeList::from_path(&path).expect("failed to parse mimeapps.list");
//         // Should contain exactly three groups
//         assert_eq!(ml.groups.0.len(), 3);
//         assert_eq!(ml.path, path)
//     }

//     #[test]
//     fn default_applications_iterator() {
//         let tmp = tempdir().unwrap();
//         let path = write_test_mimeapps(tmp.path());
//         let ml = MimeList::from_path(&path).unwrap();

//         let mut apps: Vec<_> = ml.default_applications().collect();
//         assert_eq!(apps.len(), 2);

//         // Sort for deterministic comparison.
//         apps.sort_by_key(|(mime, _)| mime.essence_str().to_owned());

//         assert_eq!(apps[0].0.essence_str(), "image/png");
//         assert_eq!(apps[0].1.filenames().next(), Some("org.gnome.eog.desktop"));

//         assert_eq!(apps[1].0.essence_str(), "text/plain");
//         assert_eq!(
//             apps[1].1.filenames().next(),
//             Some("com.system76.CosmicEdit.desktop")
//         );
//     }

//     #[test]
//     fn added_associations_filenames_iteration() {
//         let tmp = tempdir().unwrap();
//         let path = write_test_mimeapps(tmp.path());
//         let ml = MimeList::from_path(&path).unwrap();

//         let mut assoc: Vec<_> = ml.added_associations().collect();
//         assert_eq!(assoc.len(), 1);

//         let (_, mime_val) = assoc.pop().unwrap();
//         let files: Vec<_> = mime_val.filenames().collect();
//         assert_eq!(files, vec!["evince.desktop", "okular.desktop"]);
//     }

//     #[test]
//     fn removed_associations_parses_mime_correctly() {
//         let tmp = tempdir().unwrap();
//         let path = write_test_mimeapps(tmp.path());
//         let ml = MimeList::from_path(&path).unwrap();

//         let mut removed: Vec<_> = ml.removed_associations().collect();
//         assert_eq!(removed.len(), 1);

//         let (mime, val) = removed.pop().unwrap();
//         assert_eq!(mime.essence_str(), "audio/mpeg");
//         assert_eq!(val.filenames().next(), Some("some-old-player.desktop"));
//     }

//     #[test]
//     fn malformed_mime_is_skipped() {
//         // Build a mimeapps.list where one key is not a valid MIME identifier.
//         let tmp = tempdir().unwrap();
//         let path = tmp.path().join("bad.mimeapps.list");
//         let mut f = File::create(&path).unwrap();
//         writeln!(f, "[Default Applications]").unwrap();
//         writeln!(f, "not-a-mime=fo.desktop;").unwrap(); // invalid
//         writeln!(f, "text/html =browser.desktop;").unwrap(); // handles white space

//         let ml = MimeList::from_path(&path).unwrap();
//         let apps: Vec<_> = ml.default_applications().collect();

//         // Only the well‑formed entry should survive.
//         assert_eq!(apps.len(), 1);
//         assert_eq!(apps[0].0.essence_str(), "text/html");
//     }

//     #[test]
//     fn filenames_iterator_handles_trailing_semicolon() {
//         let value = MimeValue(&"a.desktop;b.desktop;c.desktop;".to_string());
//         let collected: Vec<_> = value.filenames().collect();
//         assert_eq!(collected, vec!["a.desktop", "b.desktop", "c.desktop"]);
//     }
// }
