use std::path::Path;
use std::fs::File;
use std::io;

pub fn create(filepaths: Vec<&Path>) {
    if filepaths.len() == 0 {
        println!("Please provide a filename to create!");
    }
    else {
        for path in filepaths {
            if path.is_file() {
                println!("File: {} already exists. Would you like to overwrite the file? (y/n)", path.display());
                let mut overwrite_file_confirm = String::new();
                match io::stdin().read_line(&mut overwrite_file_confirm) {
                    Ok(_) => {
                        if overwrite_file_confirm.trim() == "y"
                        {
                            _create(path);
                        }
                    }
                    Err(err) => println!("Error while reading input: {}", err)
                }
            }
            else if path.is_dir() {
                println!("Folder: {} already exists!", path.display());
            }
            else {
                _create(path);
            }
            
        }
    }
}

pub fn touch(filepaths: Vec<&Path>) {
    if filepaths.len() == 0 {
        println!("Please provide a filename to create!");
    }
    else {
        for path in filepaths {
            if path.exists() {
                match File::open(path) {
                    Ok(_) => println!("File: {} times updated.", path.display()),
                    Err(err) => println!("Error when updating file {}", err),
                }
            }
            else {
                _create(path);
            }
        }
    }
}

fn _create(filepath: &Path) {
    match File::create(filepath) {
        Ok(_) => println!("File: {} has been created.", filepath.display()),
        Err(err) => println!("Could not create file. Error: {}", err),
    }
}

pub fn create_folder(filepaths: Vec<&Path>) {
    for path in filepaths {
        _create_folder(path);
    }
}

fn _create_folder(path: &Path) {
    if path.exists() {
        println!("{} already exists", path.display());
        return;
    }
    let result = std::fs::create_dir_all(path);

    if result.is_err() {
        println!("Failed to create folder: {}", path.display());
        println!("{}", result.err().unwrap());
    }
}