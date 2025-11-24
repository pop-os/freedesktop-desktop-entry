use std::path::PathBuf;

use mime_guess::Mime;

use crate::{Groups, Value};

pub struct MimeValue<'a>(&'a Value);

impl<'a> MimeValue<'a> {
    /// Returns an iterator over the individual desktop file names.
    pub fn filenames(&self) -> impl Iterator<Item = &str> {
        self.0.split_terminator(';')
    }
}

/// Parse a mimeapps.list file and provide convenience methods for getting MIME types
#[derive(Debug, Clone)]
pub struct MimeApps {
    pub path: PathBuf,
    pub groups: Groups,
}

impl MimeApps {
    fn entries(&self, group: &str) -> impl Iterator<Item = (Mime, MimeValue<'_>)> {
        self.groups.group(group).into_iter().flat_map(|group| {
            group.0.iter().filter_map(|(key, (value, _locale))| {
                match key.parse::<Mime>() {
                    Ok(mime) => Some((mime, MimeValue(value))),
                    Err(_) => None, // skip malformed MIME identifiers
                }
            })
        })
    }

    #[inline]
    pub fn default_applications(&self) -> impl Iterator<Item = (Mime, MimeValue<'_>)> {
        self.entries("Default Applications")
    }

    #[inline]
    pub fn added_associations(&self) -> impl Iterator<Item = (Mime, MimeValue<'_>)> {
        self.entries("Added Associations")
    }

    #[inline]
    pub fn removed_associations(&self) -> impl Iterator<Item = (Mime, MimeValue<'_>)> {
        self.entries("Removed Associations")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIMEAPPS_PATH: &str = "tests_entries/mimeapps.list";
    const BAD_MIMEAPPS_PATH: &str = "tests_entries/bad.mimeapps.list";

    #[test]
    fn can_load_mimeapps_file() {
        let path = PathBuf::from(MIMEAPPS_PATH);

        // `from_path` should succeed and give us a `MimeApps`.
        let m = MimeApps::from_path(&path).expect("failed to parse mimeapps.list");
        // Should contain exactly three groups
        assert_eq!(m.groups.0.len(), 3);
        assert_eq!(m.path, path)
    }

    #[test]
    fn default_applications_iterator() {
        let path = PathBuf::from(MIMEAPPS_PATH);

        let m = MimeApps::from_path(&path).unwrap();

        let mut apps: Vec<_> = m.default_applications().collect();
        assert_eq!(apps.len(), 2);

        // Sort for deterministic comparison.
        apps.sort_by_key(|(mime, _)| mime.essence_str().to_owned());

        assert_eq!(apps[0].0.essence_str(), "image/png");
        assert_eq!(apps[0].1.filenames().next(), Some("org.gnome.eog.desktop"));

        assert_eq!(apps[1].0.essence_str(), "text/plain");
        assert_eq!(
            apps[1].1.filenames().next(),
            Some("com.system76.CosmicEdit.desktop")
        );
    }

    #[test]
    fn added_associations_filenames_iteration() {
        let path = PathBuf::from(MIMEAPPS_PATH);

        let m = MimeApps::from_path(&path).unwrap();

        let mut assoc: Vec<_> = m.added_associations().collect();
        assert_eq!(assoc.len(), 1);

        let (_, mime_val) = assoc.pop().unwrap();
        let files: Vec<_> = mime_val.filenames().collect();
        assert_eq!(files, vec!["evince.desktop", "okular.desktop"]);
    }

    #[test]
    fn removed_associations_parses_mime_correctly() {
        let path = PathBuf::from(MIMEAPPS_PATH);

        let m = MimeApps::from_path(&path).unwrap();

        let mut removed: Vec<_> = m.removed_associations().collect();
        assert_eq!(removed.len(), 1);

        let (mime, val) = removed.pop().unwrap();
        assert_eq!(mime.essence_str(), "audio/mpeg");
        assert_eq!(val.filenames().next(), Some("some-old-player.desktop"));
    }

    #[test]
    fn malformed_mime_is_skipped() {
        let path = PathBuf::from(BAD_MIMEAPPS_PATH);

        let m = MimeApps::from_path(&path).unwrap();
        let apps: Vec<_> = m.default_applications().collect();

        // Only the well‑formed entry should survive.
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].0.essence_str(), "text/html");
    }

    #[test]
    fn filenames_iterator_handles_trailing_semicolon() {
        let value = MimeValue(&"a.desktop;b.desktop;c.desktop;".to_string());
        let collected: Vec<_> = value.filenames().collect();
        assert_eq!(collected, vec!["a.desktop", "b.desktop", "c.desktop"]);
    }
}
