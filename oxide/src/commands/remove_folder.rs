use std::fs::{read_dir, remove_dir, remove_dir_all};
use std::io;
use std::path::Path;

pub fn remove_folder(filepaths: Vec<&Path>) {
    println!("In remove_folder!");

    for path in filepaths {
        _remove_folder(path);
    }
}

pub fn _remove_folder(path: &Path) {
    println!("_remove_folder");
    let num_files_in_dir = read_dir(path).unwrap().count();

    if num_files_in_dir == 0 {
        remove_dir(path);
        println!("Removed directory: {}", path.display());
    } else {
        println!("This directory is not empty, would you like to remove all contents in: \n {} ? (y,n)", path.display());
        let mut remove_dir_confirm = String::new();

        match io::stdin().read_line(&mut remove_dir_confirm) {
            Ok(_) => {
                if remove_dir_confirm.trim() == "y" {
                    remove_dir_all(path);

                    if !path.exists() {
                        println!("Removed directory: {}", path.display());
                    } else {
                        println!("Failed to remove directory: {}", path.display());
                    }
                }
            }
            Err(error) => println!("Error reading input: {}", error),
        }
    }
}

//pub fn _remove_folder(path: &Path) {
//    println!("_remove_folder");
//
//    println!("Found a folder, would you like to remove all contents in: \n {} ? (y,n)", path.display());
//    let mut remove_dir_confirm = String::new();
//    match io::stdin().read_line(&mut remove_dir_confirm) {
//        Ok(_) => {
//            // must trim because newline char is present in string read by read_line
//            if remove_dir_confirm.trim() == "y".to_string() {
//                _remove_folder(path);
//            }
//        }
//        Err(error) => println!("Error reading input: {}", error),
//    }
//}

