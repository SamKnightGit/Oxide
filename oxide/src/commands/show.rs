use std::fs::{File, read_to_string};
use std::io::{self, BufReader, BufRead};
use std::path::Path;

pub fn show(filepaths: Vec<&Path>) {
    if filepaths.len() == 0 {
        println!("Pass in a file and I will SHOW you the contents");
        return;
    }

    for path in &filepaths {
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

/*
_show_buffer may be faster for large files? 
On small files has been tested to be slower than _show

fn _show_buffer(filepath: &Path) -> io::Result<()> {
    if !filepath.is_file() {
        println!("Cannot print out non-file contents.");
        return Ok(());
    }
    println!("Showing file {}", filepath.display());
    let file = File::open(filepath)?;
    let file = BufReader::new(file);
    for line in file.lines() {
        println!("{}", line.unwrap());
    }

    Ok(())
}


*/
