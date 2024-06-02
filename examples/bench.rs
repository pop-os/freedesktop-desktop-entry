// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::{fs, time::Duration};

use freedesktop_desktop_entry::{default_paths, get_languages_from_env, DesktopEntry, Iter};

use std::time::Instant;

fn main() {
    let it = 500;

    bench_borrowed(it);
    bench_owned(it);
    bench_owned_optimized(it);
}

fn bench_borrowed(it: u32) {
    let mut total_time = Duration::ZERO;

    for _ in 0..it {
        let locale = get_languages_from_env();
        let paths = Iter::new(default_paths());

        let now = Instant::now();

        for path in paths {
            if let Ok(bytes) = fs::read_to_string(&path) {
                if let Ok(_entry) = DesktopEntry::decode_from_str(&path, &bytes, &locale) {}
            }
        }

        total_time += now.elapsed();
    }

    println!("bench_borrowed: {:.2?}", total_time / it);
}

fn bench_owned(it: u32) {
    let mut total_time = Duration::ZERO;

    for _ in 0..it {
        let locale = get_languages_from_env();
        let paths = Iter::new(default_paths());

        let now = Instant::now();

        for path in paths {
            if let Ok(_entry) = DesktopEntry::decode_from_path(path, &locale) {}
        }

        total_time += now.elapsed();
    }

    println!("bench_owned: {:.2?}", total_time / it);
}

fn bench_owned_optimized(it: u32) {
    let mut total_time = Duration::ZERO;

    for _ in 0..it {
        let locale = get_languages_from_env();
        let paths = Iter::new(default_paths());

        let now = Instant::now();

        let _ = DesktopEntry::decode_from_paths(paths, &locale)
            .filter_map(|e| e.ok())
            .collect::<Vec<_>>();

        total_time += now.elapsed();
    }

    println!("bench_owned_optimized: {:.2?}", total_time / it);
}
