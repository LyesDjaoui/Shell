use rustyline::Helper;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use std::borrow::Cow;
use crate::utils; 

pub struct ShellCompleter;

impl ShellCompleter {
    pub fn new() -> Self {
        ShellCompleter
    }
}

impl Completer for ShellCompleter {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let start_index = line[..pos].rfind(' ').map_or(0, |i| i + 1);
        let word_to_complete = &line[start_index..pos];

        let mut suggestions = Vec::new();

        let builtins = vec!["echo", "exit", "type"];
        for b in builtins {
            if b.starts_with(word_to_complete) {
                suggestions.push(format!("{} ", b));
            }
        }

        let external_exes = utils::get_all_executables(); 
        for exe in external_exes {
            if exe.starts_with(word_to_complete) {
                suggestions.push(format!("{} ", exe));
            }
        }

        suggestions.sort();
        suggestions.dedup();

        Ok((start_index, suggestions))
    }
}

impl Hinter for ShellCompleter {
    type Hint = String; 
    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> { None }
}
impl Highlighter for ShellCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> { Cow::Borrowed(line) }
}
impl Validator for ShellCompleter {}

impl Helper for ShellCompleter {}