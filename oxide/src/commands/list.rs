use std::path::Path;
use std::fs::{Metadata, DirEntry, read_dir};
use std::io;
use chrono::DateTime;
use chrono::Local;

pub fn list(filepaths: Vec<&Path>) {
    if filepaths.len() == 0 {
        match _list(&std::env::current_dir().unwrap()) {
            Ok(_) => {},
            Err(err) => println!("Error in list operation {}", err),
        }
        return;
    }

    if filepaths.len() == 1 {
        match _list(filepaths[0]) {
            Ok(_) => {},
            Err(err) => println!("Error in list operation {}", err),
        }
        return;
    }

    for path in filepaths {
        println!("{}", path.display());
        match _list(path) {
            Ok(_) => {},
            Err(err) => println!("Error in list operation {}", err),
        }
    }
}

fn _list(filepath: &Path) -> io::Result<()> {
    if filepath.is_dir() {
        for entry in read_dir(filepath)? {
            let entry = entry?;
            match entry.file_name().to_str() {
                None => {
                    println!("Could not convert path to utf-8 string. What funky OS are you using?");
                }
                Some(filename) => {
                    println!("{}", _form_list_string(&entry, &filename));
                }
            }
        }
    } else if filepath.is_file() {
        println!("Can't list a file! But the file name is: {}", filepath.display());
    } else {
        println!("Not a valid directory: {}", filepath.display());
    }
    Ok(())
}

fn _form_list_string(file: &DirEntry, filename: &str) -> String {
    let metadata_result = file.metadata();

    if metadata_result.is_ok() {
        let metadata = metadata_result.unwrap();
        return format!("{0: <6}  {1}  {2: <9}  {3}", _get_file_type_string(file), _get_file_modified(&metadata), _get_file_size(&metadata, &file.path()), filename);
    } else {
        return format!("{}", filename);
    }
}

#[cfg(target_family = "unix")]
fn _get_file_size(metadata: &Metadata, _path: &Path) -> String {
    return _convert_bytes_to_string(metadata.len());
}

#[cfg(target_family = "windows")]
fn _get_file_size(metadata: &Metadata, path: &Path) -> String {
    let total_size = walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .fold(0, |acc, m| acc + m.len());

    return _convert_bytes_to_string(total_size);
}

fn _convert_bytes_to_string(size: u64) -> String {
    // TODO: decide on system for displaying sizes in what units
    let mut size: f64 = size as f64;
    size /= 1e3;
    return format!("{:.2} KB", size);
}

fn _get_file_modified(metadata: &Metadata) -> String {
    if metadata.modified().is_ok() {
        let time = DateTime::<Local>::from(metadata.modified().unwrap());
        return format!("{}", time.format("%d/%m/%Y %H:%M"));
    }

    return format!("");
}

fn _get_file_type_string(file: &DirEntry) -> &str {
    let file_type = file.file_type().unwrap();
    let mut string = "";

    if file_type.is_file() {
        string = "<file>";
    } else if file_type.is_dir() {
        string = "<dir>";
    } else if file_type.is_symlink() {
        string = "<sym>";
    }
    return string;
}
