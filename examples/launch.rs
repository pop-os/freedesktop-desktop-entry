use freedesktop_desktop_entry::DesktopEntry;
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("tests/org.mozilla.firefox.desktop");

    let de = DesktopEntry::from_path::<&str>(path, None).expect("Error decoding desktop entry");

    de.launch_with_uris::<&str>(&[], false, &[])
        .expect("Failed to run desktop entry");
}
