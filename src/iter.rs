// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::{collections::VecDeque, fs, path::PathBuf};

use crate::DesktopEntry;

pub struct Iter {
    directories_to_walk: VecDeque<PathBuf>,
    actively_walking: Option<VecDeque<PathBuf>>,
}

impl Iter {
    /// Directories will be processed in order.
    #[inline]
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

                            // Skip directories_to_walk which could not be read
                            Err(_) => continue,
                        }
                    }

                    return None;
                }
            };

            'inner: while let Some(mut path) = paths.pop_front() {
                path = match path.canonicalize() {
                    Ok(canonicalized) => canonicalized,
                    Err(_) => continue 'inner,
                };

                if let Ok(metadata) = path.metadata() {
                    if metadata.is_dir() {
                        self.directories_to_walk.push_front(path);
                    } else if metadata.is_file()
                        && path.extension().map_or(false, |ext| ext == "desktop")
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
