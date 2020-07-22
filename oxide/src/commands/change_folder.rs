use std::path::Path;

pub fn change_folder(filepaths: Vec<&Path>) {

    // Copy bash's behaviour
    if filepaths.len() == 0 {
        return;
    }

    if filepaths.len() > 1 {
        println!("cd: too many arguments");
        return;
    }


    _change_folder(Path::new(filepaths.get(0).unwrap()))
}

fn _change_folder(filepath: &Path) {
    let path_string = filepath.to_str().unwrap();

    if filepath.is_dir() {
        match std::env::set_current_dir(filepath) 
        {
            Ok(_) => (),
            Err(err) => println!("Failed to change folder with error: {}", err), 
        }
    } else if filepath.is_file() {
        println!("\"{}\" is a file not a directory", path_string);
    } else {
        println!("\"{}\" no such file or directory", path_string);
    }
}

