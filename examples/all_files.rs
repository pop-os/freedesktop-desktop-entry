// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter, PathSource};

fn main() {
    let locales = get_languages_from_env();

    let entries = Iter::new(default_paths())
        .entries(Some(&locales))
        .collect::<Vec<_>>();

    for entry in entries {
        let path_src = PathSource::guess_from(&entry.path);

        println!("{:?}: {}\n---\n{}", path_src, entry.path.display(), entry);
    }
}
