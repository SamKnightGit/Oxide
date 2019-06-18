#[macro_use]
extern crate lazy_static;
extern crate rustyline;

use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashMap;
use std::io::{self, Write};
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::{CompletionType, Editor, Config, EditMode, Context, Helper};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::error::ReadlineError;

use commands::change_folder::change_folder;
#[cfg(target_family = "unix")]
use commands::clear::clear;
#[cfg(target_family = "windows")]
use commands::clear_windows::clear;
use commands::create::create;
use commands::create_folder::create_folder;
use commands::exit::exit;
use commands::list::list;
use commands::remove::remove;
use commands::remove_folder::remove_folder;
use commands::show::show;

mod config;

mod commands;

const PROMPT: &str = ">> ";
const DEBUG: bool = false;

lazy_static! {
    static ref COMMANDS: HashMap<String, fn(Vec<&Path>) -> ()> = {
        let mut command_hm = HashMap::new();
        command_hm.insert("ls".to_string(), list as fn(Vec<&Path>) -> ());
        command_hm.insert("list".to_string(), list);

        command_hm.insert("cat".to_string(), show);
        command_hm.insert("show".to_string(), show);

        command_hm.insert("exit".to_string(), exit);

        command_hm.insert("cd".to_string(), change_folder);
        command_hm.insert("cf".to_string(), change_folder);

        command_hm.insert("clear".to_string(), clear);

        command_hm.insert("mkdir".to_string(), create_folder);
        command_hm.insert("createf".to_string(), create_folder);

        command_hm.insert("rm".to_string(), remove);
        command_hm.insert("remove".to_string(), remove);

        command_hm.insert("touch".to_string(), create);
        command_hm.insert("create".to_string(), create);

        command_hm
    };
}


struct MyHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
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

impl Highlighter for MyHelper {}

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
        let readline = rl.readline(&prompt);

        let mut input = String::new();
        match readline {
            Ok(mut input) => {
                if DEBUG {
                    println!("Read the following: {}", input);
                }
                rl.add_history_entry(input.as_str().trim());
                execute_command(&mut input);
                rl.save_history(&oxide_history_path);
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
