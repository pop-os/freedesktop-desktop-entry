use std::{fs, path::Path};

use freedesktop_desktop_entry::DesktopEntry;

fn main() {
    let path = Path::new("tests/org.mozilla.firefox.desktop");
    let locales = &["fr", "en"];

    if let Ok(bytes) = fs::read_to_string(path) {
        if let Ok(entry) = DesktopEntry::decode_from_str(path, &bytes, locales) {
            println!("{}\n---\n{}", path.display(), entry);
        }
    }
}
