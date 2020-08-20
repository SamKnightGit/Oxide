#[macro_use]
extern crate lazy_static;
extern crate rustyline;

use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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
const DEBUG: bool = false;

lazy_static! {
    static ref COMMANDS: HashMap<&'static str, fn(Vec<&Path>) -> ()> = {
        let mut command_hm = HashMap::new();
        command_hm.insert("exit", exit as fn(Vec<&Path>) -> ());
        command_hm.insert("cd", change_folder);
        command_hm.insert("cf", change_folder);
        command_hm.insert("clear", clear);

        command_hm
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
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
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
                if DEBUG {
                    println!("Read the following: {}", input);
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
    read_ast_and_execute(&ast_root);
    println!("Parse Tree: {:?}", ast_root);
}

fn read_ast_and_execute(ast_root: &ParseNode) {
    let expr_children = ast_root.children.as_ref().unwrap();
    
    let command_expr_children = expr_children[0].children.as_ref().unwrap();
    let (command, arguments) = read_command_expr(command_expr_children);
    
    if expr_children.len() == 1
    {
        // Single command expression, print out result
        match execute_command(command, arguments) {
            Some(output) => print!("{}", output),
            None => println!("Command {} not understood", command),
        }
    }
    else
    {
        // Command expression, redirect expression || Command expression, pipe expression
        if expr_children[1].entry == ParseNodeType::PipeExpr
        {
            let pipe_expr_children = expr_children[1].children.as_ref().unwrap();
            let pipe_command_expr_children = pipe_expr_children[1].children.as_ref().unwrap();
            let (pipe_command, pipe_arguments) = read_command_expr(pipe_command_expr_children);
            redirect_pipe(command, arguments, pipe_command, pipe_arguments);
        }
        else
        {
            let redirection_expr_children = expr_children[1].children.as_ref().unwrap();
            let (redirection_op, filelist) = read_redirection_expr(redirection_expr_children);
            match redirection_op {
                ">" => redirect_output(command, arguments, filelist, true),
                ">>" => redirect_output(command, arguments, filelist, false),
                "<" => redirect_input(command, arguments, filelist),
                _ => println!("Unexpected redirection operation: {}", redirection_op), 
            }
        }
    }
}

fn read_command_expr(command_expr_children: &Vec<ParseNode>) -> (&str, Vec<&str>)
{
    let mut command: &str = "";
    let mut arguments: Vec<&str> = Vec::new();
    for node in command_expr_children.iter() 
    {
        match &node.entry
        {
            ParseNodeType::Command(command_name) => command = command_name,
            ParseNodeType::File(filename) => arguments.push(filename),
            _ => eprintln!("Unexpected parsenode in command expression!")
        } 
    }
    return (command, arguments)
}

fn read_redirection_expr(redirection_expr_children: &Vec<ParseNode>) -> (&str, Vec<&Path>)
{
    let mut redirection_op: &str = "";
    let mut filelist: Vec<&Path> = Vec::new();
    for node in redirection_expr_children.iter()
    {
        match &node.entry
        {
            ParseNodeType::RedirectionOp(redirection_op_name) => redirection_op = redirection_op_name,
            ParseNodeType::File(filename) => filelist.push(Path::new(filename)),
            _ => eprintln!("Unexpected parsenode in redirection expression!")
        }
    }
    return (redirection_op, filelist)
}

fn redirect_output(command: &str, arguments: Vec<&str>, filelist: Vec<&Path>, overwrite: bool)
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

fn redirect_input(command: &str, arguments: Vec<&str>, filelist: Vec<&Path>)
{
    //TODO: Right now we do input redirection like bash (only first file is used as input)
    //      This can be changed if we want to be opinionated:
    //          Concat all data in files and use resulting blob as input
    //          Run command separately for each file and output each result
    match execute_command_with_input(command, arguments, filelist)
    {
        Some(output) => {
            print!("{}", output)
        }
        None => println!("Command {} not understood", command),

    }
}

fn redirect_pipe(command: &str, arguments: Vec<&str>, pipe_command: &str, pipe_arguments: Vec<&str>)
{
    let command_process = 
        match Command::new(command).args(arguments).stdout(Stdio::piped()).spawn()
        {
            Ok(child) => {
                child
            }
            Err(err) => {
                println!("Could not execute command {}", command);
                return
            }
        };
        
    match Command::new(pipe_command).args(pipe_arguments).stdin(command_process.stdout.unwrap()).output()
    {
        Ok(pipe_command_output) => {
            let output_data = pipe_command_output.stdout;
            print!("{}", String::from_utf8(output_data).unwrap());
        }
        Err(err) => {
            println!("Could not execute command {}", pipe_command);
        }
    }
}

fn execute_command(command: &str, arguments: Vec<&str>) -> Option<String>
{
    match COMMANDS.get(command) {
        Some(comm) => {
            comm(arguments.iter().map(Path::new).collect::<Vec<&Path>>());
            // Custom commands will always return emptystring
            return Some(String::from(""));
        }
        None => ()
    }
    // Make a mutable copy of command so we can modify it if its an alias
    let mut command = command;

    if let Some(comm) = ALIASES.get(command) {
        command = comm;
    }
    
    match Command::new(command).args(arguments).output() 
    {
        Ok(command_output) => {
            let output_data = command_output.stdout;
            return Some(String::from_utf8(output_data).unwrap())
        }
        Err(_) => return None,
    }
}

fn execute_command_with_input(command: &str, arguments: Vec<&str>, input_filelist: Vec<&Path>) -> Option<String>
{
    match COMMANDS.get(command) {
        Some(comm) => {
            comm(arguments.iter().map(Path::new).collect::<Vec<&Path>>());
            // Custom commands will always return emptystring
            return Some(String::from(""));
        }
        None => ()
    }

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
    
    let file_contents = read_to_string(input_filelist[0]).expect("Could not read input file");

    {
        let stdin = process.stdin.as_mut().expect("Failed to get input handle");
        stdin.write_all(file_contents.as_bytes());
    }

    match process.wait_with_output()
    {
        Ok(command_output) => {
            let output_data = command_output.stdout;
            return Some(String::from_utf8(output_data).unwrap())
        }
        Err(_) => return None
    }
}
