use std::path::PathBuf;

use crate::Groups;

/// Parse a thumbnailer file and provide convenience methods for entries
#[derive(Debug, Clone)]
pub struct Thumbnail {
    pub path: PathBuf,
    pub groups: Groups,
}

impl Thumbnail {
    #[inline]
    pub fn thumbnailer_entry(&self, key: &str) -> Option<&str> {
        self.groups.thumbnailer_entry()?.entry(key)
    }

    #[inline]
    pub fn exec(&self) -> Option<&str> {
        self.thumbnailer_entry("Exec")
    }

    #[inline]
    pub fn try_exec(&self) -> Option<&str> {
        self.thumbnailer_entry("TryExec")
    }

    /// Return mime types
    #[inline]
    pub fn mime_type(&self) -> Option<Vec<&str>> {
        self.thumbnailer_entry("MimeType")
            .map(|e| e.split_terminator(';').collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THUMBNAILER_PATH: &str = "tests_entries/com.system76.CosmicPlayer.thumbnailer";

    #[test]
    fn can_load_thumbnailer_file() {
        let path = PathBuf::from(THUMBNAILER_PATH);

        // `from_path` should succeed and give us a `Thumbnailer`.
        let t = Thumbnail::from_path(&path).expect("failed to parse mimeapps.list");
        // Should contain exactly three groups
        assert_eq!(t.groups.0.len(), 1);
        assert_eq!(t.path, path)
    }

    #[test]
    fn can_get_entries() {
        let path = PathBuf::from(THUMBNAILER_PATH);

        let t = Thumbnail::from_path(&path).expect("failed to parse mimeapps.list");

        assert_eq!(t.exec(), Some("cosmic-player --thumbnail %o --size %s %u"));
        assert_eq!(t.try_exec(), Some("cosmic-player"));
        assert_eq!(
            t.mime_type(),
            Some(vec!["application/mxf", "application/ram"])
        );
    }
}
