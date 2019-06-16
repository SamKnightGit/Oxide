use std::fs::remove_file;
use std::io;
use std::path::Path;

use crate::commands::remove_folder::remove_folder;

use super::remove_folder::_remove_folder;

pub fn remove(filepaths: Vec<&Path>) {
    for path in filepaths {
        _remove(path);
    }
}

fn _remove(path: &Path) {
    if !path.exists() {
        println!("Could not find the file: {}", path.display());
        return
    }

    if path.is_dir() {
        _remove_folder(path)
    }

    else {
        remove_file(path);
        _print_remove(path);
    }
}

fn _print_remove(path: &Path) {
    println!("Removed file: {}", path.display());
}
