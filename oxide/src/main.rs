#[macro_use]
extern crate lazy_static;
extern crate rustyline;

use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::{CompletionType, Editor, Config, EditMode, Context};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::error::ReadlineError;
use rustyline::Helper;

use commands::change_folder::change_folder;
#[cfg(target_family = "unix")]
use commands::clear::clear;
#[cfg(target_family = "windows")]
use commands::clear_windows::clear;
use commands::create::create;
use commands::create::touch;
use commands::create::create_folder;
use commands::exit::exit;
use commands::list::list;
use commands::remove::remove;
use commands::remove::remove_folder;
use commands::show::show;

mod config;

mod commands;

const PROMPT: &str = ">> ";
const DEBUG: bool = false;

lazy_static! {
    static ref COMMANDS: HashMap<&'static str, fn(Vec<&Path>) -> ()> = {
        let mut command_hm = HashMap::new();
        command_hm.insert("ls", list as fn(Vec<&Path>) -> ());
        command_hm.insert("list", list);

        command_hm.insert("cat", show);
        command_hm.insert("show", show);

        command_hm.insert("exit", exit);

        command_hm.insert("cd", change_folder);
        command_hm.insert("cf", change_folder);

        command_hm.insert("clear", clear);

        command_hm.insert("mkdir", create_folder);
        command_hm.insert("createf", create_folder);

        command_hm.insert("rm", remove);
        command_hm.insert("remove", remove);

        command_hm.insert("rmf", remove_folder);
        command_hm.insert("removef", remove_folder);

        command_hm.insert("create", create);
        command_hm.insert("touch", touch);

        command_hm
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
                execute_command(&mut input);
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


fn parse_command(input: &mut String) -> Vec<&str> {
    let command_vector = input.split_whitespace();
    let commands: Vec<&str> = Vec::from_iter(command_vector);
    commands
}

fn execute_command(input: &mut String) {
    if DEBUG{
        println!("Executing on following string: {}", input);
    }

    // TODO: Change this in future to return an iterator of (command, args)
    let commands: Vec<&str> = parse_command(input);
    let command = &commands[0];
    let arguments = commands[1..].iter().map(Path::new).collect::<Vec<&Path>>();
    
    match COMMANDS.get(command) {
        None => {
            println!("Command {} not understood", command);
        }
        Some(comm) => {
            comm(arguments);
        }
    } 
}
