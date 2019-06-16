use std::fs::{read_to_string};
use std::path::Path;

pub fn show(filepaths: Vec<&Path>) {
    if filepaths.len() == 0 {
        println!("Pass in a file and I will SHOW you the contents");
        return;
    }

    for path in filepaths {
        _show(path);
    }
}

fn _show(filepath: &Path) {
    if !filepath.is_file() {
        println!("Cannot print out non-file contents.");
        return;
    }
    println!("Showing file {}", filepath.display());
    let file_data = read_to_string(filepath);
    match file_data {
        Err(error) => { println!("Could not read file {} \n Error {}", filepath.display(), error); }
        Ok(file_text) => { println!("{}", file_text); }
    }
}

