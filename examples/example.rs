use std::fs;

use freedesktop_desktop_entry as desktop_entry;

fn main() {
    for entry in desktop_entry::Iter::new(desktop_entry::default_paths()) {
        if let Ok(bytes) = fs::read_to_string(&entry) {
            let entry = desktop_entry::decode(entry, &bytes);
            println!("{}\n---\n{}", entry.path.display(), entry);
        }
    }
}
