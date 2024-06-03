use std::path::Path;

use freedesktop_desktop_entry::DesktopEntry;

fn main() {
    let path = Path::new("tests/org.mozilla.firefox.desktop");
    let locales = &["fr_FR", "en", "it"];

    // if let Ok(bytes) = fs::read_to_string(path) {
    //     if let Ok(entry) = DesktopEntry::decode_from_str(path, &bytes, locales) {
    //         println!("{}\n---\n{}", path.display(), entry);
    //     }
    // }

    if let Ok(entry) = DesktopEntry::decode_from_path(path.to_path_buf(), locales) {
        println!("{}\n---\n{}", path.display(), entry);

        dbg!(entry.comment(locales));

        dbg!(entry.actions());
        

        dbg!(entry.action_entry("new-window", "Name"));
    }
}
