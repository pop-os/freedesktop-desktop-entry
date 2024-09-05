// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::{collections::VecDeque, fs, path::PathBuf};

use crate::{
    decoder::{add_generic_locales, decode_from_path_with_buf},
    DesktopEntry,
};

pub struct Iter {
    directories_to_walk: VecDeque<PathBuf>,
    actively_walking: Option<fs::ReadDir>,
}

impl Iter {
    /// Directories will be processed in order.
    pub fn new<I: Iterator<Item = PathBuf>>(directories_to_walk: I) -> Self {
        Self {
            directories_to_walk: directories_to_walk.collect(),
            actively_walking: None,
        }
    }
}

impl Iterator for Iter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            let mut iterator = match self.actively_walking.take() {
                Some(dir) => dir,
                None => {
                    while let Some(path) = self.directories_to_walk.pop_front() {
                        match fs::read_dir(&path) {
                            Ok(directory) => {
                                self.actively_walking = Some(directory);
                                continue 'outer;
                            }

                            // Skip directories_to_walk which could not be read
                            Err(_) => continue,
                        }
                    }

                    return None;
                }
            };

            'inner: while let Some(entry) = iterator.next() {
                if let Ok(entry) = entry {
                    let mut path = entry.path();

                    path = match path.canonicalize() {
                        Ok(canonicalized) => canonicalized,
                        Err(_) => continue 'inner,
                    };

                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            self.directories_to_walk.push_front(path);
                        } else if file_type.is_file()
                            && path.extension().map_or(false, |ext| ext == "desktop")
                        {
                            self.actively_walking = Some(iterator);
                            return Some(path);
                        }
                    }
                }
            }
        }
    }
}

impl Iter {
    pub fn entries<'i, 'l: 'i, L>(
        self,
        locales_filter: Option<&'l [L]>,
    ) -> impl Iterator<Item = DesktopEntry<'static>> + 'i
    where
        L: AsRef<str>,
    {
        let mut buf = String::new();
        let locales_filter = locales_filter.map(add_generic_locales);

        self.map(move |path| decode_from_path_with_buf(path, locales_filter.as_deref(), &mut buf))
            .filter_map(|e| e.ok())
    }
}
