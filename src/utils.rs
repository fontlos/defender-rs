use std::path::{Path, PathBuf};

pub fn path<P: AsRef<Path>>(file: P) -> PathBuf {
    let path = std::env::current_exe().unwrap();
    path.parent().unwrap().join(file)
}