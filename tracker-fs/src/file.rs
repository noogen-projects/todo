use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub fn find_file_in_dir_and_parents(dir: impl AsRef<Path>, file_name: impl AsRef<str>) -> Option<PathBuf> {
    let mut current_dir = dir.as_ref();
    loop {
        let file = current_dir.join(file_name.as_ref());
        if file.exists() {
            break Some(file);
        } else {
            current_dir = current_dir.parent()?;
        }
    }
}

pub fn find_in_dir_by_name_part(dir: impl AsRef<Path>, name_part: impl AsRef<str>) -> Option<PathBuf> {
    for entry in WalkDir::new(dir).follow_links(true).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if let Some(file_name) = path.file_name() {
            if file_name.to_string_lossy().contains(name_part.as_ref()) {
                return Some(path.to_path_buf());
            }
        }
    }
    None
}
