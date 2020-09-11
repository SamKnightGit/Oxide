#[macro_use]
//extern crate rustyline;

use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio, Child};
use std::io;
use std::io::Write;
use std::fs::{File, OpenOptions, read_to_string};
use std::error::Error;

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::{CompletionType, Editor, Config, EditMode, Context};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::error::ReadlineError;
use rustyline::Helper;

use crate::parser::ParseNodeType;
use crate::parser::ParseNode;
use crate::parser::RedirectionOp;
use crate::parser::parse_input;

use crate::commands::change_folder::change_folder;
#[cfg(target_family = "unix")]
use crate::commands::clear::clear;
#[cfg(target_family = "windows")]
use crate::commands::clear_windows::clear;
use crate::commands::exit::exit;

use crate::config::OxideHistory;

use crate::ALIASES;
use crate::BUILTINS;
use crate::MyHelper;


const PROMPT: &str = ">> ";
const DEBUG: bool = true;
const EXECUTE_AST: bool = true;

#[derive(Debug)]
struct CommandData <'a> {
    command: String,
    arguments: Vec<String>,
    redirection_op: Option<RedirectionOp>,
    redirection_files: Vec<&'a Path>,
}

impl CommandData <'_> {
    fn new() -> CommandData<'static> {
        CommandData {
            command: "".to_string(),
            arguments: Vec::new(),
            redirection_op: None,
            redirection_files: Vec::new(),
        }
    }

    // Do we really need this???
    fn has_redirection(&self) -> bool {
        match self.redirection_op
        {
            Some(_) => return true,
            None => return false,
        }
    }
}



pub fn run(mut rl: Editor<MyHelper>, oxide_history: OxideHistory) -> Result<(), Box<dyn Error>> {
    if oxide_history.exists {
        if rl.load_history(&oxide_history.path).is_err() && DEBUG {
            println!("Could not find history at: {}", oxide_history.path.display());
        }
    }

    loop {
        let prompt = format!("{0} {1}", std::env::current_dir().unwrap().to_str().unwrap(), PROMPT);
        rl.helper_mut().unwrap().colored_prompt = format!("\x1b[1;32m{}\x1b[0m", prompt);
        let readline = rl.readline(&prompt);

        //let mut input = String::new();
        match readline {
            Ok(mut input) => {
                if DEBUG 
                {
                    println!("Read the following: {}", input);
                }
                if input.is_empty()
                {
                    print!("");
                    continue
                }

                rl.add_history_entry(input.as_str().trim());
                execute_input(&mut input);
                match rl.save_history(&oxide_history.path) {
                    Ok(_) => {
                        if DEBUG {
                            println!("History saved.")
                        }
                    },
                    Err(err) => {
                        if oxide_history.exists
                        {
                            println!("Error saving history: {:?}", err);
                            return Err(Box::new(err))
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
            }
            Err(ReadlineError::Eof) => {
                println!("Exiting!");
                break;
            }
            Err(err) => {
                println!("Error when reading from input: {:?}", err);
                return Err(Box::new(err))
            }
        } 
    }

    return Ok(())
}


fn split_input(input: &mut String) -> Vec<&str> {
    let command_vector = input.split_whitespace();
    let commands: Vec<&str> = Vec::from_iter(command_vector);
    commands
}

fn execute_input(input: &mut String) {
    if DEBUG{
        println!("Executing on following string: {}", input);
    }

    let commands: Vec<&str> = split_input(input);
    let ast_root;
    match parse_input(&commands) {
        Ok(ast) => ast_root = ast,
        Err(e) => {
            println!("{}", e);
            return
        }
    }
    if EXECUTE_AST
    {
        read_ast_and_execute(&ast_root);
    }
    if DEBUG
    {
        println!("Parse Tree: {:#?}", ast_root);
    }
}

fn read_ast_and_execute(ast_root: &ParseNode) {
    let mut expr_children = ast_root.children.as_ref().unwrap();
    
    let commands_and_arguments = accumulate_commands_and_arguments(&mut expr_children);

    execute_on_command_list(commands_and_arguments);
}

fn accumulate_commands_and_arguments(mut expr_children: &Vec<ParseNode>) -> Vec<CommandData> {
    let mut commands_and_arguments: Vec<CommandData> = Vec::new();
    let mut child_index = 0;
    let mut command_data = CommandData::new();
    while child_index < expr_children.len()
    {
        match expr_children[child_index].entry {
            ParseNodeType::CommandExpr => {
                let command_expr_children = expr_children[child_index].children.as_ref().unwrap();
                read_command_expr(
                    &mut command_data.command, 
                    &mut command_data.arguments, 
                    command_expr_children
                );
                
                child_index += 1;
            }
            ParseNodeType::RedirectionExpr => {
                let redirection_expr_children = expr_children[child_index].children.as_ref().unwrap();
                read_redirection_expr(
                    &mut command_data.redirection_op,
                    &mut command_data.redirection_files,
                    redirection_expr_children
                );
                
                child_index += 1;
            }
            ParseNodeType::PipeExpr => {
                let pipe_expr_children = expr_children[child_index].children.as_ref().unwrap();
                
                // Add command struct to the vector and reset struct
                commands_and_arguments.push(command_data);
                command_data = CommandData::new();

                // Set new children to recurse down the PipeExpr
                expr_children = pipe_expr_children;
                // Set child index to avoid Pipe node.
                child_index = 1;
                continue
            }
            _ => break,
        }
    }

    // TODO: Is there a smarter way to check if the struct is empty?
    if !(command_data.command == "".to_string())
    {
        commands_and_arguments.push(command_data);
    }

    return commands_and_arguments
}

fn read_command_expr(
    command: &mut String, 
    arguments: &mut Vec<String>,
    command_expr_children: &Vec<ParseNode>
)
{
    for node in command_expr_children.iter() 
    {
        
        match &node.entry
        {
            ParseNodeType::Command(command_name) => {
                *command = command_name.clone()
            }
            ParseNodeType::File(filename) => {
                arguments.push(filename.to_string())
            }
            _ => eprintln!("Unexpected parsenode in command expression!")
        } 
    }
}

fn read_redirection_expr<'a>(
    redirection_op: &mut Option<RedirectionOp>, 
    redirection_files: &mut Vec<&'a Path>,
    redirection_expr_children: &'a Vec<ParseNode>
)
{ 
    for node in redirection_expr_children.iter()
    {
        match &node.entry
        {
            ParseNodeType::RedirectionOp(redirection_op_name) => {
                *redirection_op = Some(redirection_op_name.clone())
            }
            ParseNodeType::File(filename) => {
                redirection_files.push(Path::new(filename))
            }
            _ => eprintln!("Unexpected parsenode in redirection expression!")
        }
    }
}

fn execute_on_command_list(commands_and_arguments: Vec<CommandData>)
{
    if DEBUG
    {
        println!("{:?}", commands_and_arguments);
    }

    // Execute separately so builtins take effect
    if commands_and_arguments.len() == 1
    {
        let command_data = &commands_and_arguments[0];
        execute_on_command_data(&command_data);         
        return
    }

    let mut previous_process: Option<Child> = None;
    for command_data in commands_and_arguments
    {
        // If BUILTIN start the piping again after the builtin command
        if let Some(_) = BUILTINS.get(&command_data.command[..])
        {
            println!("Builtin command: {} encountered during piping, consider removing", command_data.command);
            previous_process = None;
        }
        else
        {
            match previous_process
            {
                // TODO: Move these into methods on the command_data struct
                Some(prev_process) => {
                    let next_process = 
                        match Command::new(command_data.command)
                                      .args(command_data.arguments)
                                      .stdout(Stdio::piped())
                                      .stdin(prev_process.stdout.unwrap())
                                      .spawn()
                        {
                            Ok(process) => process,
                            Err(err) => {
                                eprintln!("{}", err);
                                return
                            }
                        };
                    
                    previous_process = Some(next_process);
                }
                None => {
                    let next_process = 
                        match Command::new(command_data.command)
                                      .args(command_data.arguments)
                                      .stdout(Stdio::piped())
                                      .stdin(Stdio::piped())
                                      .spawn()
                        {
                            Ok(process) => process,
                            Err(err) => {
                                eprintln!("{}", err);
                                return
                            }
                        };
                    
                    previous_process = Some(next_process);
                }
            }
        }

        if let Some(redirection_op) = command_data.redirection_op
        { 
            match redirection_op  
            {
                RedirectionOp::Output => {
                    redirect_output_process(previous_process, &command_data.redirection_files, true);
                    previous_process = None;
                }
                RedirectionOp::Append => {
                    redirect_output_process(previous_process, &command_data.redirection_files, false);
                    previous_process = None;
                }
                RedirectionOp::Input => {
                    //redirect_input_process(&mut previous_process.as_mut(), &command_data.redirection_files);

                    redirect_input_process(previous_process.as_mut(), &command_data.redirection_files);
                }
            }    
        }
        else
        {
            if let Some(ref process) = previous_process 
            {
                // Should drop handle to stdin stopping process hanging on input
                let _input = process.stdin.as_ref();
            }
        }
    }

    // Just print the results of the pipe expression
    if let Some(process) = previous_process 
    {
        match process.wait_with_output()
        {
            Ok(command_output) => {
                let output_data = command_output.stdout;
                println!("{}", String::from_utf8(output_data).unwrap())
            }
            Err(_) => println!("Could not get output from final command!"),
        }
    } 
}


// Output redirection on piped processes
fn redirect_output_process(process: Option<Child>, filelist: &Vec<&Path>, overwrite: bool)
{
    if DEBUG 
    {
        println!("Printing output to files: {:?}", filelist);
    }

    let mut output = String::new();
    if let Some(real_process) = process
    {
        match real_process.wait_with_output()
        {
            Ok(command_output) => {
                let output_data = command_output.stdout;
                output = 
                    match String::from_utf8(output_data)
                    {
                        Ok(output_string) => output_string,
                        Err(_) => {
                            println!("Could not read data from process into string!");
                            return
                        }
                    };
            }
            Err(_) => {
                println!("Could not get output from process during output redirection");
                return
            }
        }
    }

    for file in filelist.iter()
    {
        // TODO: Do we want to warn people before we overwrite with > operator?
        match OpenOptions::new()
                          .write(true)
                          .create(true)
                          .truncate(overwrite)
                          .append(!overwrite)
                          .open(file)
        {
            Ok(mut fp) => {
                match fp.write_all(output.as_bytes()) 
                {
                    Err(err) => eprintln!("{}", err),
                    _ => (),
                }
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}

// Output redirection on individual command
fn redirect_output(command: &str, arguments: &Vec<String>, filelist: &Vec<&Path>, overwrite: bool)
{
    match execute_command(command, arguments) 
    {
        Some(output) => {
            for file in filelist.iter()
            {
                // TODO: Do we want to warn people before we overwrite with > operator?
                match OpenOptions::new()
                                  .write(true)
                                  .create(true)
                                  .truncate(overwrite)
                                  .append(!overwrite)
                                  .open(file)
                {
                    Ok(mut fp) => {
                        match fp.write_all(output.as_bytes()) 
                        {
                            Err(err) => eprintln!("{}", err),
                            _ => (),
                        }
                    }
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
        None => println!("Command {} not understood", command), 
    }
}

// Input redirection on piped processes
// TODO: Change expects to handle errors.
fn redirect_input_process(process: Option<&mut Child>, filelist: &Vec<&Path>)
{
    if let Some(real_process) = process 
    {
        let file_contents = read_to_string(filelist[0]).expect("Could not read input file");

        {
            let stdin = real_process.stdin.as_mut().expect("Failed to get input handle");
            stdin.write_all(file_contents.as_bytes());
        }
    }
}

// Input redirection on individual command
fn redirect_input(command: &str, arguments: &Vec<String>, filelist: &Vec<&Path>)
{
    //TODO: Right now we do input redirection like bash (only first file is used as input)
    //      This can be changed if we want to be opinionated:
    //          Concat all data in files and use resulting blob as input
    //          Run command separately for each file and output each result
    match BUILTINS.get(command) {
        Some(comm) => {
            comm(arguments.iter().map(Path::new).collect::<Vec<&Path>>()); 
        }
        None => {
            // Make a mutable copy of command so we can modify it if its an alias
            let mut command = command;

            if let Some(comm) = ALIASES.get(command) {
                command = comm;
            }

            let mut process = Command::new(command)
                                      .args(arguments)
                                      .stdin(Stdio::piped())
                                      .stdout(Stdio::piped())
                                      .spawn()
                                      .expect("Failed to spawn child process to execute command");
            //redirect_input_process(&mut Some(&process), filelist);
            redirect_input_process(Some(&mut process), filelist);
            
            match process.wait_with_output()
            {
                Ok(command_output) => {
                    let output_data = command_output.stdout;
                    println!("{}", String::from_utf8(output_data).unwrap());
                }
                Err(err) => eprintln!("{}", err)
            }
        }
    }
}

fn execute_on_command_data(command_data: &CommandData)
{
    if let Some(redirection_op) = &command_data.redirection_op
    { 
        match redirection_op  
        {
            RedirectionOp::Output => {
                redirect_output(
                    &command_data.command,
                    &command_data.arguments,
                    &command_data.redirection_files, 
                    true
                );
            }
            RedirectionOp::Append => {
                redirect_output(
                    &command_data.command,
                    &command_data.arguments,
                    &command_data.redirection_files, 
                    false
                );
            }
            RedirectionOp::Input => {
                redirect_input(
                    &command_data.command,
                    &command_data.arguments,
                    &command_data.redirection_files, 
                );
            }
        }    
    }
    else
    {
        if let Some(output) = execute_command(&command_data.command, &command_data.arguments)
        {
            println!("{}", output);
        }
    }
        
            
}

// TODO: Return result, String if Ok and Error if Err
fn execute_command(command: &str, arguments: &Vec<String>) -> Option<String>
{
    if let Some(comm) = BUILTINS.get(command) {
        comm(arguments.iter().map(Path::new).collect::<Vec<&Path>>());
        // Custom BUILTINS will always return emptystring
        return Some(String::from(""));
    }
    
    // Make a mutable copy of command so we can modify it if its an alias
    let mut command = command;

    if let Some(comm) = ALIASES.get(command) {
        command = comm;
    }
    
    let process = Command::new(command).args(arguments).spawn();

    if let Ok(running_process) = process
    {
        match running_process.wait_with_output()
        {
            Ok(command_output) => {
                let output_data = command_output.stdout;
                return Some(String::from_utf8(output_data).unwrap())
            }
            Err(_) => return None,
        }
    }
    else
    {
 eprintln!("Could not get output from command: {}", command);
        return None;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_split_input()
    {
        let mut input = String::from("ls .. . wow");
        let expected_split = vec!("ls", "..", ".", "wow");
        assert_eq!(split_input(&mut input), expected_split);
    }

    #[test]
    fn test_accumulate_commands_and_arguments()
    {
        let mut input = String::from("ls . | sort > test.txt");
        let commands = split_input(&mut input);
        let ast_root = parse_input(&commands).unwrap();
        let mut expr_children = ast_root.children.as_ref().unwrap();
        
        let commands_and_arguments = accumulate_commands_and_arguments(&mut expr_children);
        let first_command = &commands_and_arguments[0];
        let second_command = &commands_and_arguments[1];
        
        assert_eq!(first_command.command, "ls");
        assert_eq!(first_command.arguments, vec!(String::from(".")));
        assert_eq!(first_command.redirection_op, None);
        assert_eq!(first_command.redirection_files, Vec::<&Path>::new());

        assert_eq!(second_command.command, "sort");
        assert_eq!(second_command.arguments, Vec::<String>::new());
        assert_eq!(second_command.redirection_op, Some(RedirectionOp::Output));
        assert_eq!(second_command.redirection_files, vec!(Path::new("test.txt")));
    }
    
    #[test]
    fn test_read_command_expr()
    {
        let mut command_data = CommandData::new();
        let mut command_expr_children = vec!(
            ParseNode {
                entry: ParseNodeType::Command(String::from("ls")),
                children: None,
            },
            ParseNode {
                entry: ParseNodeType::File(String::from(".")),
                children: None,
            },
            ParseNode {
                entry: ParseNodeType::File(String::from("..")),
                children: None,
            },
            ParseNode {
                entry: ParseNodeType::RedirectionOp(RedirectionOp::Output),
                children: None,
            }
        );

        read_command_expr(
            &mut command_data.command, 
            &mut command_data.arguments, 
            &command_expr_children
        );

        assert_eq!(command_data.command, String::from("ls"));
        assert_eq!(command_data.arguments, vec!(String::from("."), String::from("..")));
        assert_eq!(command_data.redirection_op, None); 
        assert_eq!(command_data.redirection_files, Vec::<&Path>::new());
    }

    #[test]
    fn test_read_redirection_expr()
    {
        let mut command_data = CommandData::new();
        let mut command_expr_children = vec!(
            ParseNode {
                entry: ParseNodeType::RedirectionOp(RedirectionOp::Input),
                children: None,
            },
            ParseNode {
                entry: ParseNodeType::File(String::from("test1.txt")),
                children: None,
            },
            ParseNode {
                entry: ParseNodeType::File(String::from("test2.txt")),
                children: None,
            },
            ParseNode {
                entry: ParseNodeType::Command(String::from("ls")),
                children: None,
            }
        );

        read_redirection_expr(
            &mut command_data.redirection_op, 
            &mut command_data.redirection_files, 
            &command_expr_children
        );

        assert_eq!(command_data.command, String::from(""));
        assert_eq!(command_data.arguments, Vec::<String>::new());
        assert_eq!(command_data.redirection_op, Some(RedirectionOp::Input)); 
        assert_eq!(command_data.redirection_files, vec!(Path::new("test1.txt"), Path::new("test2.txt")));

    }
}
