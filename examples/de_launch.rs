use freedesktop_desktop_entry::{get_languages_from_env, DesktopEntry};
use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args.get(1).expect("Not enough arguments");
    let path = PathBuf::from(path);
    let locales = get_languages_from_env();
    let input = fs::read_to_string(&path).expect("Failed to read file");
    let de = DesktopEntry::from_path(path, &locales).expect("Error decoding desktop entry");
    de.launch(&[], false, &locales)
        .expect("Failed to run desktop entry");
}
