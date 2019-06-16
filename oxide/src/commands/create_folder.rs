use std::path::Path;

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
        println!("Failed to create directory: {}", path.display());
        println!("{}", result.err().unwrap());
    }
}

