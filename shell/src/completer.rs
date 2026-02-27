use rustyline::Helper;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use std::borrow::Cow;

pub struct ShellCompleter;

impl ShellCompleter {
    pub fn new() -> Self {
        ShellCompleter
    }
}

impl Completer for ShellCompleter {
    type Candidate = String;

    fn complete(&self , line : &str , pos : usize , _ctx : &rustyline::Context<'_>) -> rustyline::Result<(usize , Vec<Self::Candidate>)> {

        let slice = &line[..pos];

        if slice == "ech" && pos == line.len() {
            Ok((0 , vec!["echo ".to_string()]))
        } else if slice == "exi" && pos == line.len() {
            Ok((0 , vec!["exit ".to_string()]))
        } else if slice == "typ" && pos == line.len() {
            Ok((0 , vec!["type ".to_string()]))
        } else {
            Ok((0 , Vec::new()))
        }
    }
}

impl Hinter for ShellCompleter {
    type Hint = String; 

    fn hint(&self , _line : &str , _pos : usize , _ctx : &rustyline::Context<'_>) -> Option<Self::Hint> {
        None
    }
}

impl Highlighter for ShellCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }
}

impl Validator for ShellCompleter {}

impl Helper for ShellCompleter {}