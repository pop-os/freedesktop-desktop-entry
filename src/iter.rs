use std::{fs, path::PathBuf};
pub struct Iter {
    directories_to_walk: Vec<PathBuf>,
    actively_walking: Option<fs::ReadDir>,
}

impl Iter {
    pub fn new(directories_to_walk: Vec<PathBuf>) -> Self {
        Self {
            directories_to_walk,
            actively_walking: None,
        }
    }
}

impl Iterator for Iter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<PathBuf> {
        'outer: loop {
            let mut iterator = match self.actively_walking.take() {
                Some(dir) => dir,
                None => {
                    while let Some(path) = self.directories_to_walk.pop() {
                        match fs::read_dir(path) {
                            Ok(directory) => {
                                self.actively_walking = Some(directory);
                                continue 'outer;
                            }

                            // Skip directories_to_walk which could not be read
                            Err(_) => continue,
                        }
                    }

                    return None;
                }
            };

            while let Some(entry) = iterator.next() {
                if let Ok(entry) = entry {
                    let path = entry.path();

                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            self.directories_to_walk.push(path);
                        } else if file_type.is_file() {
                            if path.extension().map_or(false, |ext| ext == "desktop") {
                                self.actively_walking = Some(iterator);
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }
    }
}
