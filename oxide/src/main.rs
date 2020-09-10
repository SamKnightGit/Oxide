#[macro_use]
extern crate lazy_static;

use std::borrow::Cow::{self, Borrowed, Owned};
use std::process;
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

mod config;
mod parser;
mod commands;
mod logic;

use crate::commands::change_folder::change_folder;
#[cfg(target_family = "unix")]
use crate::commands::clear::clear;
#[cfg(target_family = "windows")]
use crate::commands::clear_windows::clear;
use crate::commands::exit::exit;

use config::OxideHistory;
use logic::run;

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

pub struct MyHelper {
    pub completer: FilenameCompleter,
    pub highlighter: MatchingBracketHighlighter,
    pub validator: MatchingBracketValidator,
    pub hinter: HistoryHinter,
    pub colored_prompt: String,
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

    let oxide_history = OxideHistory::new();

    if let Err(err) = run(rl, oxide_history)
    {
        eprintln!("Error when running shell {}", err);
        process::exit(1);
    }
}
