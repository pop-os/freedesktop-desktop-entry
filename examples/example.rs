// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::fs;

use freedesktop_desktop_entry::{default_paths, DesktopEntry, Iter};

fn main() {
    for (path_src, path) in Iter::new(default_paths()) {
        if let Ok(bytes) = fs::read_to_string(&path) {
            if let Ok(entry) = DesktopEntry::decode(&path, &bytes) {
                println!("{:?}: {}\n---\n{}", path_src, path.display(), entry);
            }
        }
    }
}
