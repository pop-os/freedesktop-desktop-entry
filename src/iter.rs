// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::{
    collections::{BTreeSet, VecDeque},
    fs,
    path::PathBuf,
};

use crate::DesktopEntry;

pub struct Iter {
    directories_to_walk: VecDeque<PathBuf>,
    actively_walking: Option<VecDeque<PathBuf>>,
    visited: BTreeSet<PathBuf>,
}

impl Iter {
    /// Directories will be processed in order.
    #[inline]
    pub fn new<I: Iterator<Item = PathBuf>>(directories_to_walk: I) -> Self {
        Self {
            directories_to_walk: directories_to_walk.collect(),
            actively_walking: None,
            visited: BTreeSet::default(),
        }
    }
}

impl Iterator for Iter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            let mut paths = match self.actively_walking.take() {
                Some(dir) => dir,
                None => {
                    while let Some(path) = self.directories_to_walk.pop_front() {
                        match fs::read_dir(&path) {
                            Ok(dir) => {
                                self.actively_walking = Some({
                                    // Pre-sort the walked directories as order of parsing affects appid matches.
                                    let mut entries = dir
                                        .filter_map(Result::ok)
                                        .map(|entry| entry.path())
                                        .collect::<VecDeque<_>>();
                                    entries.make_contiguous().sort_unstable();
                                    entries
                                });

                                continue 'outer;
                            }

                            // Skip directories_to_walk which could not be read or that were
                            // already visited
                            _ => continue,
                        }
                    }

                    return None;
                }
            };

            'inner: while let Some(mut path) = paths.pop_front() {
                if !path.exists() {
                    continue 'inner;
                }

                if path.is_dir() {
                    path = match path.canonicalize() {
                        Ok(canonicalized) => canonicalized,
                        Err(_) => continue 'inner,
                    };
                }

                if let Ok(metadata) = path.metadata() {
                    if metadata.is_dir() {
                        // Skip visited directories to mitigate against file system loops
                        if self.visited.insert(path.clone()) {
                            self.directories_to_walk.push_front(path);
                        }
                    } else if metadata.is_file()
                        && path.extension().is_some_and(|ext| ext == "desktop")
                    {
                        self.actively_walking = Some(paths);
                        return Some(path);
                    }
                }
            }
        }
    }
}

impl Iter {
    #[inline]
    pub fn entries<'i, 'l: 'i, L>(
        self,
        locales_filter: Option<&'l [L]>,
    ) -> impl Iterator<Item = DesktopEntry> + 'i
    where
        L: AsRef<str>,
    {
        self.map(move |path| DesktopEntry::from_path(path, locales_filter))
            .filter_map(|e| e.ok())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix};

    use super::{DesktopEntry, Iter};

    #[test]
    fn iter_yields_all_entries() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        // File hierarchy
        // Directory 'a'
        let dir_a = root.join("a");
        let dir_a_a = dir_a.join("aa");
        fs::create_dir_all(&dir_a_a).unwrap();
        let file_a = dir_a.join("a.desktop");
        let file_b = dir_a.join("b.desktop");
        let file_c = dir_a_a.join("c.desktop");

        // Directory 'b'
        let dir_b_bb_bbb = root.join("b/bb/bbb");
        fs::create_dir_all(&dir_b_bb_bbb).unwrap();
        let file_d = dir_b_bb_bbb.join("d.desktop");

        // Files in root
        let file_e = root.join("e.desktop");

        // Write entries for each file
        let all_files = [file_a, file_b, file_c, file_d, file_e];
        for file in &all_files {
            let (name, _) = file
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .split_once('.')
                .unwrap();
            fs::write(file, DesktopEntry::from_appid(name.to_string()).to_string()).unwrap();
        }

        let mut iter = Iter::new(
            fs::read_dir(root)
                .unwrap()
                .map(|entry| entry.unwrap().path()),
        );
        for (expected, actual) in all_files.iter().zip(&mut iter) {
            assert_eq!(*expected, actual);
        }

        assert_eq!(None, iter.next());
    }

    #[test]
    fn iter_no_infinite_loop() {
        // Hierarchy with an infinite loop
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let dir = root.join("loop");
        unix::fs::symlink(root, &dir).expect("Linking {dir:?} to {root:?}");

        // Sanity check that we have a loop
        assert_eq!(
            fs::canonicalize(root).unwrap(),
            fs::canonicalize(&dir).unwrap(),
            "Expected a loop where {dir:?} points to {root:?}"
        );

        // Now we need a fake desktop entry that will be yielded endlessly with a broken iter
        let entry = DesktopEntry::from_appid("joshfakeapp123".into());
        let entry_path = root.join("joshfakeapp123.desktop");
        fs::write(&entry_path, entry.to_string()).expect("Writing entry: {entry_path:?}");

        // Finally, check that the iterator is eventually exhausted
        for (i, de) in Iter::new(
            fs::read_dir(root)
                .unwrap()
                .map(|entry| entry.unwrap().path()),
        )
        .entries(Option::<&[&str]>::None)
        .enumerate()
        {
            assert_eq!(entry.appid, de.appid);
            if i > 0 {
                panic!("Infinite loop");
            }
        }
    }
}
