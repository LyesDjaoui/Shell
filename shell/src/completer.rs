use rustyline::Helper;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use std::borrow::Cow;
use crate::utils;
use std::cell::RefCell;
use std::io::{self, Write};

pub struct ShellCompleter {
    last_line: RefCell<String>,
    tab_count: RefCell<u32>,
}

impl ShellCompleter {
    pub fn new() -> Self {
        ShellCompleter {
            last_line: RefCell::new(String::new()),
            tab_count: RefCell::new(0),
        }
    }

    pub fn get_suggestions(&self , line :&str , pos : usize , _ctx : &rustyline::Context<'_>) -> rustyline::Result<(usize, Vec<String>)> {
        let start_index = line[..pos].rfind(' ').map_or(0, |i| i + 1);
        let word_to_complete = &line[start_index..pos];

        let mut suggestions = Vec::new();

        let builtins = vec!["echo", "exit", "type"];
        for b in builtins {
            if b.starts_with(word_to_complete) {
                suggestions.push(b.to_string());
            }
        }

        let external_exes = utils::get_all_executables(); 
        for exe in external_exes {
            if exe.starts_with(word_to_complete) {
                suggestions.push(exe.to_string());
            }
        }

        suggestions.sort();
        suggestions.dedup();

        Ok((start_index, suggestions))
    }
}

impl Completer for ShellCompleter {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let (start_index, suggestions) = self.get_suggestions(line, pos, _ctx)?;

        if suggestions.is_empty() {
            return Ok((pos, vec![]));
        }

        let word_to_complete = &line[start_index..pos];
        let lcp = utils::longest_common_prefix(&suggestions);

        if lcp.len() > word_to_complete.len() {
            let mut completion = lcp;
            
            if suggestions.len() == 1 {
                completion.push(' ');
            }
            
            *self.tab_count.borrow_mut() = 0;
            return Ok((start_index, vec![completion]));
        }

        let current_line = line[..pos].to_string();
        let is_same_request = *self.last_line.borrow() == current_line;

        if is_same_request {
            *self.tab_count.borrow_mut() += 1;
        } else {
            *self.tab_count.borrow_mut() = 1;
            *self.last_line.borrow_mut() = current_line;
        }

        if *self.tab_count.borrow() == 1 {
            print!("\x07");
            io::stdout().flush().unwrap();
            Ok((pos, vec![]))
        } else {
            *self.tab_count.borrow_mut() = 0;
            println!("\n{}", suggestions.join("  "));
            print!("$ {}", line);
            io::stdout().flush().unwrap();
            Ok((pos, vec![]))
        }
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