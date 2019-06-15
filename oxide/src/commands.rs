use core::fmt::Debug;
use std::fs::{DirEntry, Metadata, metadata, read_dir, read_to_string, remove_file};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::DateTime;
use chrono::Local;

#[cfg(target_family = "windows")]
use crate::windows_clear;

pub fn list(filepath: &Vec<String>) {
    if filepath.len() == 0 {
        _list(&std::env::current_dir().unwrap());
        return;
    }

    let filepaths: Vec<&Path> = filepath.iter().map(Path::new).collect();
    if filepaths.len() == 1 {
        _list(filepaths[0]);
        return;
    }

    for path in filepaths {
        println!("{}", path.display());
        _list(path);
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
fn _get_file_size(metadata: &Metadata, path: &Path) -> String {
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


pub fn change_directory(filepath: &Vec<String>) {

    // Copy bash's behaviour
    if filepath.len() == 0 {
        return;
    }

    if filepath.len() > 1 {
        println!("cd: too many arguments");
        return;
    }


    _change_directory(Path::new(filepath.get(0).unwrap()))
}

fn _change_directory(filepath: &Path) {
    let path_string = filepath.to_str().unwrap();

    if filepath.is_dir() {
        std::env::set_current_dir(filepath);
    } else if filepath.is_file() {
        println!("\"{}\" is a file not a directory", path_string);
    } else {
        println!("\"{}\" no such file or directory", path_string);
    }
}


pub fn show(filepath: &Vec<String>) {
    if filepath.len() == 0 {
        println!("Pass in a file and I will SHOW you the contents");
        return;
    }

    let filepaths: Vec<&Path> = filepath.iter().map(Path::new).collect();

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

pub fn clear(filepath: &Vec<String>) {
    _clear();
}

#[cfg(target_family = "unix")]
fn _clear() {
    std::process::Command::new("clear").status().unwrap();
}

#[cfg(target_family = "windows")]
fn _clear() {
    windows_clear::clear_screen_windows();
}

pub fn mkdir(filepath: &Vec<String>) {
    let file_paths: Vec<&Path> = filepath.iter().map(Path::new).collect();

    for path in file_paths {
        _mkdir(path);
    }
}

fn _mkdir(path: &Path) {
    if path.exists() {
        println!("{} already exists", path.display());
        return;
    }
    let result = std::fs::create_dir_all(path);

    if result.is_err() {
        println!("Failed to create directory: {}", path.display());
        println!("{}", result.err().unwrap());
    }
}

pub fn remove(filepath: &Vec<String>) {
    let file_paths: Vec<&Path> = filepath.iter().map(Path::new).collect();

    for path in file_paths {
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
                    _remove_all(path);
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

pub fn remove_all(filepath: &Vec<String>) {
    println!("In remove all!");
}

fn _remove_all(path: &Path) {
    println!("_remove_all");
}

pub fn exit(filepath: &Vec<String>) {
    std::process::exit(0)
}