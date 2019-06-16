use std::path::Path;
use std::fs::{remove_file};
use std::io;

use super::remove_folder::{_remove_folder};

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
        println!("Found a folder, would you like to remove all contents in: \n {} ? (y,n)", path.display());
        let mut remove_dir_confirm = String::new();
        match io::stdin().read_line(&mut remove_dir_confirm) {
            Ok(_) => {
                if remove_dir_confirm == "y" {
                    _remove_folder(path);
                }
            }
            Err(error) => println!("Error reading input: {}", error),
        }
    }

    else {
        remove_file(path);
        _print_remove(path);
    }
}

fn _print_remove(path: &Path) {
    println!("Removed file: {}", path.display());
}
