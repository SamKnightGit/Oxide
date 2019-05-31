use std::io::{self, Write};
use std::iter::FromIterator;
mod commands;

const PROMPT: &str = ">> ";
const DEBUG: bool = true;

fn main() {
    println!("Welcome to Oxide! A shell written entirely in Rust.");

    loop {
        print!("{}", PROMPT);
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

fn execute_command(input: &mut String) {
    println!("Executing on following string: {}", input);
    let command_vector = input.split_whitespace();
    let commands: Vec<String> = Vec::from_iter(command_vector.map(String::from));
    process_command(&commands[0], &commands[1..].to_vec());
}

fn process_command(command: &String, inputs: &Vec<String>) {
    println!("Command: {}", command);
    println!("Inputs: {:?}", inputs);   
}
