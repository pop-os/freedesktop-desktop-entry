use std::path::Path;

use crate::DesktopEntry;

#[test]
fn test() {
    let path = Path::new("tests/org.mozilla.firefox.desktop");
    let locales = &["fr", "en"];

    if let Ok(entry) = DesktopEntry::decode_from_path(path.to_path_buf(), locales) {
        let e = DesktopEntry::localized_entry(
            None,
            entry.groups.get("Desktop Entry"),
            "GenericName",
            Some("fr"),
        )
        .unwrap();

        println!("{e}");
        // println!("{}\n---\n{}", path.display(), entry);
    }
}
