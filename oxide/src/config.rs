extern crate dirs;
use std::path::{PathBuf};
use std::fs::{create_dir_all, File};
use std::io;


pub fn get_oxide_history() -> (bool, PathBuf) {
    match dirs::config_dir() {
        Some(config_folder) => {
            let oxide_history_path: PathBuf = [config_folder.to_str().unwrap(), "oxide", "history.txt"].iter().collect();

            if oxide_history_path.exists() {
                return (true, oxide_history_path);
            }
            
            match create_oxide_history(config_folder) {
                Ok(_) => {
                    return (true, oxide_history_path);
                }
                Err(e) => {
                    println!("Could not create oxide configuration file due to error: {}
                             Oxide will have reduced functionality.", e);
                    
                    return (false, oxide_history_path);
                }
            }
        }
        None => {
            println!("Could not find configuration folder. Oxide will have reduced functionality.");
            
            return (false, PathBuf::new());
        }
    }
} 

fn create_oxide_history(config_folder_path: PathBuf) -> io::Result<()> {
    let oxide_conf_folder: PathBuf = [config_folder_path.to_str().unwrap(), "oxide"].iter().collect();
    create_dir_all(oxide_conf_folder.as_path())?;
    let history_file_path: PathBuf = [oxide_conf_folder.to_str().unwrap(), "history.txt"].iter().collect();
    File::create(history_file_path.as_path())?;
    Ok(())      
}
        
   