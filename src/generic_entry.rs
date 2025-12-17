use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    fs,
    path::PathBuf,
};

use crate::{
    decoder::{format_value, parse_line, Line},
    DecodeError,
};

#[derive(Debug, Clone, Default)]
pub struct Group(pub BTreeMap<Key, Value>);
pub type Key = String;
pub type Value = String;

impl Group {
    #[inline]
    pub fn entry(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|key| key.as_ref())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Groups(pub BTreeMap<GroupName, Group>);
pub type GroupName = String;

impl Groups {
    #[inline]
    pub fn group(&self, key: &str) -> Option<&Group> {
        self.0.get(key)
    }
}

/// Parse files based on the desktop entry spec. Any duplicate groups or keys
/// will be overridden by the last parsed value and any entries without a group will be ignored.
#[derive(Debug, Clone)]
pub struct GenericEntry {
    pub path: PathBuf,
    pub groups: Groups,
}

impl GenericEntry {
    pub fn from_str(path: impl Into<PathBuf>, input: &str) -> Result<GenericEntry, DecodeError> {
        #[inline(never)]
        fn inner<'a>(path: PathBuf, input: &'a str) -> Result<GenericEntry, DecodeError> {
            let path: PathBuf = path.into();

            let mut groups = Groups::default();
            let mut active_group: Option<(&str, Group)> = None;

            for line in input.lines() {
                match parse_line(line)? {
                    Line::Group(key) => {
                        if let Some((prev_key, prev_group)) =
                            active_group.replace((key, Group::default()))
                        {
                            groups.0.insert(prev_key.to_string(), prev_group);
                        }
                    }
                    Line::Entry(key, value) => {
                        if let Some((_, group)) = active_group.as_mut() {
                            group.0.insert(key.to_string(), format_value(value)?);
                        }
                    }
                    _ => (),
                }
            }

            if let Some((prev_key, prev_group)) = active_group.take() {
                groups.0.insert(prev_key.to_string(), prev_group);
            }

            Ok(GenericEntry { groups, path })
        }

        inner(path.into(), input)
    }

    /// Return an owned [`GenericEntry`]
    #[inline]
    pub fn from_path(path: impl Into<PathBuf>) -> Result<GenericEntry, DecodeError> {
        let path: PathBuf = path.into();
        let input = fs::read_to_string(&path)?;
        Self::from_str(path, &input)
    }

    #[inline]
    pub fn group(&self, key: &str) -> Option<&Group> {
        self.groups.group(key)
    }
}

impl Display for GenericEntry {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        for (group_name, group) in &self.groups.0 {
            let _ = writeln!(formatter, "[{}]", group_name);

            for (key, value) in &group.0 {
                let _ = writeln!(formatter, "{}={}", key, value);
            }
            writeln!(formatter)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GENERIC_ENTRY_PATH: &str = "tests_entries/generic.entry";

    #[test]
    fn can_load_file() {
        let path = PathBuf::from(GENERIC_ENTRY_PATH);

        let entry = GenericEntry::from_path(&path).expect("failed to parse file");
        assert_eq!(entry.groups.0.len(), 1);
        assert_eq!(entry.path, path)
    }

    #[test]
    fn can_get_entries() {
        let path = PathBuf::from(GENERIC_ENTRY_PATH);

        let entry = GenericEntry::from_path(&path).expect("failed to parse file");

        let group = entry.group("Thumbnailer Entry").unwrap();

        assert_eq!(
            group.entry("Exec"),
            Some("cosmic-player --thumbnail %o --size %s %u")
        );
        assert_eq!(group.entry("TryExec"), Some("cosmic-player"));
        assert_eq!(
            group.entry("MimeType"),
            Some("application/mxf;application/ram")
        );
    }
}
