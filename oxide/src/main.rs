#[macro_use]
extern crate lazy_static;

use std::io::{self, Write};
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
mod commands;
use commands::{list, show};

const PROMPT: &str = ">> ";
const DEBUG: bool = true;


lazy_static! {
    static ref COMMANDS: HashMap<String, fn(&Vec<String>, &mut PathBuf) -> ()> = {
        let mut command_hm = HashMap::new();
        command_hm.insert("ls".to_string(), list as fn(&Vec<String>, &mut PathBuf) -> ());
        command_hm.insert("cat".to_string(), show);
        //command_hm.insert("cd".to_string(), change_directory);

        command_hm
    };
}

fn main() {
    println!("Welcome to Oxide! A shell written entirely in Rust.");

    let mut cwd = PathBuf::from(".");

    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if DEBUG {
                    println!("Read the following: {}", input);
                }
                execute_command(&mut input, &mut cwd);
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

fn execute_command(input: &mut String, cwd: &mut PathBuf) {
    println!("Executing on following string: {}", input);
    // Change this in future to return an iterator of (command, args)
    let commands: Vec<String> = parse_command(input);
    process_command(&commands[0], &commands[1..].to_vec(), cwd);
}

fn process_command(command: &String, inputs: &Vec<String>, cwd: &mut PathBuf) {
    println!("Command: {}", command);
    println!("Inputs: {:?}", inputs); 
    match COMMANDS.get(command) {
        None => {
            println!("Command {} not understood", command);
        }
        Some(comm) => {
            comm(&inputs, cwd);
        }
    } 
}
