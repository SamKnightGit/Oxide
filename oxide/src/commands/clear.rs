use std::path::Path;

pub fn clear(_filepaths: Vec<&Path>) {
    _clear();
}

fn _clear() {
    std::process::Command::new("clear").status().unwrap();
}