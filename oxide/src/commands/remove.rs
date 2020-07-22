use std::fs::{read_dir, remove_dir, remove_dir_all, remove_file};
use std::io;
use std::path::Path;


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
        match remove_file(path) {
            Ok(_) => println!("Removed file: {}", path.display()),
            Err(err) => println!("Failed to remove file with error: {}", err)
        }
    }
}

pub fn remove_folder(filepaths: Vec<&Path>) {
    for path in filepaths {
        _remove_folder(path);
    }
}

pub fn _remove_folder(path: &Path) {
    let num_files_in_dir = read_dir(path).unwrap().count();

    if num_files_in_dir == 0 {
        match remove_dir(path) {
            Ok(_) => println!("Removed directory: {}", path.display()),
            Err(err) => println!("Failed to remove directory with error: {}", err)
        }
    } else {
        println!("This directory is not empty, would you like to remove all contents in: \n {} ? (y,n)", path.display());
        let mut remove_dir_confirm = String::new();

        match io::stdin().read_line(&mut remove_dir_confirm) {
            Ok(_) => {
                if remove_dir_confirm.trim() == "y" {
                    match remove_dir_all(path)
                    {
                        Ok(_) => println!("Removed directory: {}", path.display()),
                        Err(err) => println!("Failed to remove directory with error: {}", err),
                    };
                }
            }
            Err(error) => println!("Error reading input: {}", error),
        }
    }
}
