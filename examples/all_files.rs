// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::fs;

use freedesktop_desktop_entry::{
    default_paths, get_languages_from_env, DesktopEntry, Iter, PathSource,
};

fn main() {
    let locales = get_languages_from_env();

    for path in Iter::new(default_paths()) {
        let path_src = PathSource::guess_from(&path);
        if let Ok(bytes) = fs::read_to_string(&path) {
            if let Ok(entry) = DesktopEntry::from_str(&path, &bytes, &locales) {
                println!("{:?}: {}\n---\n{}", path_src, path.display(), entry);
            }
        }
    }
}
