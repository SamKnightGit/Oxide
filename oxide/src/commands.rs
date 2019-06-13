use std::io;
use std::path::{Path, PathBuf};
use std::fs::{read_dir, read_to_string};

pub fn list(filepath: &Vec<String>, cwd: &mut PathBuf) {
    if filepath.len() == 0 {
        _list(cwd);
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
                    println!("{}", filename);
                }
            }
        }
    }
    else {
        println!("Can't list a file! But the file name is: {}", filepath.display());
    }
    Ok(())
}


pub fn change_directory(filepath: &Vec<String>, cwd: &mut PathBuf) {

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

    if filepath.is_dir(){
        std::env::set_current_dir(filepath);
    }
    else if filepath.is_file(){
        println!("\"{}\" is a file not a directory", path_string);
    }
    else {
        println!("\"{}\" no such file or directory", path_string);
    }

}


pub fn show (filepath: &Vec<String>, cwd: &mut PathBuf) {
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

pub fn exit(filepath: &Vec<String>, cwd: &mut PathBuf){
    std::process::exit(0)
}