use freedesktop_desktop_entry::DesktopEntry;
use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args.get(1).expect("Not enough arguments");
    let path = PathBuf::from(path);
    let input = fs::read_to_string(&path).expect("Failed to read file");
    let de = DesktopEntry::decode(path.as_path(), &input).expect("Error decoding desktop entry");
    de.launch(&[]).expect("Failed to run desktop entry");
}
