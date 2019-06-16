use std::path::Path;
use std::fs::File;

pub fn create(filepaths: Vec<&Path>) {
    if filepaths.len() == 0 {
        println!("Please provide a filename to create!");
    }
    else {
        for path in filepaths {
            if path.is_file() {
                println!("File: {} already exists!", path.display());
            }
            else if path.is_dir() {
                println!("Folder: {} already exists!", path.display());
            }
            _create(path);
        }
    }
}

fn _create(filepath: &Path) -> std::io::Result<()> {
    match File::create(filepath) {
        Ok(file) => println!("File: {} has been created.", filepath.display()),
        Err(err) => println!("Could not create file. Error: {}", err),
    }
    Ok(())
}