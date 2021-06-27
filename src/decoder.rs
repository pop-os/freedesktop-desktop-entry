use super::*;
use std::path::PathBuf;

pub fn decode<'a>(path: PathBuf, input: &'a str) -> DesktopEntry<'a> {
    let mut groups = Groups::new();

    let mut active_group = "";

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

            groups
                .entry(active_group)
                .or_insert_with(Default::default)
                .entry(key)
                .or_insert_with(|| ("", BTreeMap::new()))
                .0 = value;
        }
    }

    DesktopEntry { path, groups }
}
