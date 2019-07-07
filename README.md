# Freedesktop Desktop Entry Specification

This crate provides a library for efficiently generating valid desktop entries.

- [Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/index.html)

## Example

This could be added to your `build.rs`, or as a workspace member:

```rust
use freedesktop_desktop_entry::{Application, DesktopEntry, DesktopType};
use std::{
    env,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

const APPID: &str = "com.system76.Popsicle";

fn main() {
    let exec_path = Path::new("/usr").join("bin").join(APPID);
    let exec = exec_path.as_os_str().to_str().expect("prefix is not UTF-8");

    let mut desktop = File::create(["target/", APPID, ".desktop"].concat().as_str())
        .expect("failed to create desktop entry file");

    let entry = DesktopEntry::new(
        "Popsicle",
        APPID,
        DesktopType::Application(
            Application::new(&["System", "GTK"], exec)
                .keywords(&["usb", "flash" ,"drive", "image"])
                .startup_notify(),
        ),
    )
    .comment("Multiple USB image flasher")
    .generic_name("USB Flasher");

    desktop.write_all(entry.to_string().as_bytes());
}
```
