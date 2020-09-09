#[macro_use]
extern crate lazy_static;
extern crate rustyline;

use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio, Child};
use std::io;
use std::io::Write;
use std::fs::{File, OpenOptions, read_to_string};

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::{CompletionType, Editor, Config, EditMode, Context};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::error::ReadlineError;
use rustyline::Helper;

mod parser;
use parser::ParseNodeType;
use parser::ParseNode;
use parser::RedirectionOp;
use parser::parse_input;

mod commands;
use commands::change_folder::change_folder;
#[cfg(target_family = "unix")]
use commands::clear::clear;
#[cfg(target_family = "windows")]
use commands::clear_windows::clear;
use commands::exit::exit;

mod config;

const PROMPT: &str = ">> ";
const DEBUG: bool = true;
const EXECUTE_AST: bool = true;

lazy_static! {
    static ref BUILTINS: HashMap<&'static str, fn(Vec<&Path>) -> ()> = {
        let mut builtin_hm = HashMap::new();
        builtin_hm.insert("exit", exit as fn(Vec<&Path>) -> ());
        builtin_hm.insert("cd", change_folder);
        builtin_hm.insert("cf", change_folder);
        builtin_hm.insert("clear", clear);

        builtin_hm
    };

    static ref ALIASES: HashMap<&'static str, &'static str> = {
        let mut alias_hm = HashMap::new();
        alias_hm.insert("list", "ls");
        alias_hm.insert("show", "cat");
        alias_hm.insert("remove", "rm");
        alias_hm.insert("removef", "rm -r");
        alias_hm.insert("create", "touch"); 
        alias_hm.insert("createf", "mkdir");

        alias_hm
    };
        
}

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

struct MyHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for MyHelper {
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}


impl Highlighter for MyHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[38;5;242m".to_owned() + hint + "\x1b[0m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for MyHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

impl Helper for MyHelper {}

fn main() {
    println!("Welcome to Oxide! A shell written entirely in Rust.");
    let rl_config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .output_stream(OutputStreamType::Stdout)
        .build();

    let helper = MyHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        colored_prompt: "".to_owned(),
        validator: MatchingBracketValidator::new(),
    };

    let mut rl = Editor::with_config(rl_config);
    rl.set_helper(Some(helper));

    let oxide_history: (bool, PathBuf) = config::get_oxide_history();
    let history_exists = oxide_history.0;
    let oxide_history_path = oxide_history.1;
    
    if history_exists {
        if rl.load_history(&oxide_history_path).is_err() && DEBUG {
            println!("Could not find history at: {}", oxide_history_path.display());
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
                match rl.save_history(&oxide_history_path) {
                    Ok(_) => {
                        if DEBUG {
                            println!("History saved.")
                        }
                    },
                    Err(err) => println!("Error saving history: {}", err),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
            }
            Err(ReadlineError::Eof) => {
                println!("Exiting!");
                break;
            }
            Err(error) => {
                println!("Error when reading from input: {}", error);
            }
        } 
    }
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

    execute_on_command_list(commands_and_arguments);
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
