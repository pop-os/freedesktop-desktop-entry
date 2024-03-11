# Freedesktop Desktop Entry Specification

This crate provides a library for efficiently parsing and launching [Desktop Entry](https://specifications.freedesktop.org/desktop-entry-spec/latest/index.html) files.

```rust
use std::fs;

use freedesktop_desktop_entry::{default_paths, DesktopEntry, Iter, PathSource};

fn main() {
    for path in Iter::new(default_paths()) {
        let path_src = PathSource::guess_from(&path);
        if let Ok(bytes) = fs::read_to_string(&path) {
            if let Ok(entry) = DesktopEntry::decode(&path, &bytes) {
                println!("{:?}: {}\n---\n{}", path_src, path.display(), entry);
            }
        }
    }
}
```

## License

Licensed under the [Mozilla Public License 2.0](https://choosealicense.com/licenses/mpl-2.0/). Permissions of this copyleft license are conditioned on making available source code of licensed files and modifications of those files under the same license (or in certain cases, one of the GNU licenses). Copyright and license notices must be preserved. Contributors provide an express grant of patent rights. However, a larger work using the licensed work may be distributed under different terms and without source code for files added in the larger work.

### Contribution

Any contribution intentionally submitted for inclusion in the work by you shall be licensed under the Mozilla Public License 2.0 (MPL-2.0). It is required to add a boilerplate copyright notice to the top of each file:

```rs
// Copyright {year} {person OR org} <{email}>
// SPDX-License-Identifier: MPL-2.0
```
