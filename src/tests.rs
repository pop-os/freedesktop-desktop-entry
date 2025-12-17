use std::path::PathBuf;

use crate::{desktop_entry_from_path, group_entry_from_path, DesktopEntry};

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

#[test]
fn translatable_inverted_key_at_the_end() {
    let locales = &["fr_FR.UTF-8"];

    let de = DesktopEntry::from_path(
        PathBuf::from("tests_entries/org.gnome.SystemMonitor.desktop"),
        Some(locales),
    )
    .unwrap();

    assert!(de.keywords(&[] as &[&str]).is_some());
}

#[test]
fn get_single_group_entry() {
    let entry = group_entry_from_path(
        PathBuf::from("tests_entries/org.gnome.Nautilus.desktop"),
        "Desktop Entry",
        "Exec",
    )
    .unwrap();

    assert_eq!(entry.unwrap(), "nautilus --new-window %U");
}

#[test]
fn get_single_desktop_entry() {
    let entry = desktop_entry_from_path(
        PathBuf::from("tests_entries/org.gnome.Nautilus.desktop"),
        "Exec",
    )
    .unwrap();

    assert_eq!(entry.unwrap(), "nautilus --new-window %U");
}

#[test]
fn get_invalid_single_desktop_entry() {
    let entry = desktop_entry_from_path(
        PathBuf::from("tests_entries/org.gnome.Nautilus.desktop"),
        "Invalid",
    )
    .unwrap();

    assert!(entry.is_none());
}
