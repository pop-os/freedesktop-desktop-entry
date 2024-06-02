// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::fs;

use freedesktop_desktop_entry::{default_paths, get_languages_from_env, DesktopEntry, Iter};

use std::time::Instant;

fn main() {
    bench_borrowed();
    bench_owned();
    bench_owned_optimized();
}

fn bench_borrowed() {
    let locale = get_languages_from_env();
    let paths = Iter::new(default_paths());

    let mut c = 0;

    let now = Instant::now();

    for path in paths {
        if let Ok(bytes) = fs::read_to_string(&path) {
            if let Ok(_entry) = DesktopEntry::decode_from_str(&path, &bytes, &locale) {
                c += 1;
            }
        }
    }

    let elapsed = now.elapsed();
    println!("bench_borrowed {}: {:.2?}", c, elapsed);
}

fn bench_owned() {
    let locale = get_languages_from_env();
    let paths = Iter::new(default_paths());

    let mut c = 0;

    let now = Instant::now();

    for path in paths {
        if let Ok(_entry) = DesktopEntry::decode_from_path(path, &locale) {
            c += 1;
        }
    }

    let elapsed = now.elapsed();
    println!("bench_owned {}: {:.2?}", c, elapsed);
}

fn bench_owned_optimized() {
    let locale = get_languages_from_env();
    let paths = Iter::new(default_paths());

    let now = Instant::now();

    let de = DesktopEntry::decode_from_paths(paths, &locale)
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();

    let elapsed = now.elapsed();
    println!("bench_owned_optimized {}: {:.2?}", de.len(), elapsed);
}
