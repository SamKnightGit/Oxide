#[macro_use]
extern crate lazy_static;

use std::io::{self, Write};
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
mod commands;


#[cfg(target_family = "windows")]
mod windows_clear;

use commands::{list, show, exit, change_directory, clear, mkdir, remove};


const PROMPT: &str = ">> ";
const DEBUG: bool = false;

lazy_static! {
    static ref COMMANDS: HashMap<String, fn(&Vec<String>) -> ()> = {
        let mut command_hm = HashMap::new();
        command_hm.insert("ls".to_string(), list as fn(&Vec<String>) -> ());
        command_hm.insert("cat".to_string(), show);
        command_hm.insert("exit".to_string(), exit);
        command_hm.insert("cd".to_string(), change_directory);
        command_hm.insert("clear".to_string(), clear);
        command_hm.insert("mkdir".to_string(), mkdir);
        command_hm.insert("remove".to_string(), remove);

        command_hm
    };
}

fn main() {
    println!("Welcome to Oxide! A shell written entirely in Rust.");

    loop {
        print!("{0} {1}", std::env::current_dir().unwrap().to_str().unwrap(), PROMPT);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if DEBUG {
                    println!("Read the following: {}", input);
                }
                execute_command(&mut input);
            }
            Err(error) => {
                println!("Error when reading from input: {}", error);
            }
        } 
    }
}


fn parse_command(input: &mut String) -> Vec<String> {
    let command_vector = input.split_whitespace();
    let commands: Vec<String> = Vec::from_iter(command_vector.map(String::from));
    commands
}

fn execute_command(input: &mut String) {
    if DEBUG{
        println!("Executing on following string: {}", input);
    }

    // TODO: Change this in future to return an iterator of (command, args)
    let commands: Vec<String> = parse_command(input);
    let command = &commands[0];
    let arguments = &commands[1..].to_vec();
    
    match COMMANDS.get(command) {
        None => {
            println!("Command {} not understood", command);
        }
        Some(comm) => {
            comm(&arguments);
        }
    } 
}
