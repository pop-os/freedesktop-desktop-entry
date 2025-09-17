use std::path::PathBuf;

use crate::DesktopEntry;

#[test]
fn nautilus_name_french() {
    let locales = &["fr_FR.UTF-8"];

    let de = DesktopEntry::from_path(
        PathBuf::from("tests_entries/org.gnome.Nautilus.desktop"),
        Some(locales),
    )
    .unwrap();

    assert_eq!(de.name(locales).unwrap(), "Fichiers");
}
