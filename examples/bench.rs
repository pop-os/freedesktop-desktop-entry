// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::{fs, time::Duration};

use freedesktop_desktop_entry::{DesktopEntry, Iter, default_paths, get_languages_from_env};

use std::time::Instant;

fn main() {
    let it = 500;

    bench(it);
}

fn bench(it: u32) {
    let mut total_time = Duration::ZERO;

    for _ in 0..it {
        let locale = get_languages_from_env();
        let paths = Iter::new(default_paths());

        let now = Instant::now();

        for path in paths {
            if let Ok(bytes) = fs::read_to_string(&path) {
                if let Ok(_entry) = DesktopEntry::from_str(&path, &bytes, Some(&locale)) {}
            }
        }

        total_time += now.elapsed();
    }

    println!("time to parse all .desktop files: {:.2?}", total_time / it);
}
