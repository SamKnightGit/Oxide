use std::fs::remove_file;
use std::io;
use std::path::Path;

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
        println!("Found a folder, would you like to remove all contents in: \n {} ? (y,n)", path.display());
        let mut remove_dir_confirm = String::new();
        match io::stdin().read_line(&mut remove_dir_confirm) {
            Ok(_) => {
                println!("input: {}", remove_dir_confirm);
                // must trim because newline char is present in string read by read_line
                if remove_dir_confirm.trim() == "y".to_string() {
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
