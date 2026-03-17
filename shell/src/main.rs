mod utils;
mod commands;
mod exec;
mod completer;

use crate::completer::ShellCompleter;
use crate::exec::execute_command;
use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};
use rustyline::history::MemHistory;
use std::io::{self, Write};


fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                while let Some(&inner) = chars.peek() {
                    chars.next();
                    if inner == '"' {
                        break;
                    }
                    if inner == '\\' {
                        if let Some(&escaped) = chars.peek() {
                            chars.next();
                            match escaped {
                                '"' | '\\' | '$' | '`' | '\n' => current.push(escaped),
                                _ => { current.push('\\'); current.push(escaped); }
                            }
                        }
                    } else {
                        current.push(inner);
                    }
                }
            }
            '\'' => {
                while let Some(&inner) = chars.peek() {
                    chars.next();
                    if inner == '\'' {
                        break;
                    }
                    current.push(inner);
                }
            }
            '\\' => {
                if let Some(&escaped) = chars.peek() {
                    chars.next();
                    current.push(escaped);
                }
            }
            '|' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push("|".to_string());
            }
            '>' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                let mut redir = ">".to_string();
                if chars.peek() == Some(&'>') {
                    chars.next();
                    redir = ">>".to_string();
                }
                tokens.push(redir);
            }
            ' ' | '\t' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                if (c == '1' || c == '2') && chars.peek() == Some(&'>') {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                    chars.next();
                    let mut redir = format!("{}>", c);
                    if chars.peek() == Some(&'>') {
                        chars.next();
                        redir = format!("{}>>", c);
                    }
                    tokens.push(redir);
                } else {
                    current.push(c);
                }
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn main() -> rustyline::Result<()> {
    let mut history : Vec<String> = Vec::new();
    let config = Config::builder().build();

    let mut rl: Editor<ShellCompleter, MemHistory> = 
        Editor::with_history(config, MemHistory::new())?;

    let h = ShellCompleter::new();
    rl.set_helper(Some(h));

    loop {
        let readline: Result<String, ReadlineError> = rl.readline("$ ");

        match readline {
            Ok(input) => {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    continue;
                }
                history.push(trimmed.to_string());
                let _ = rl.add_history_entry(trimmed);

                let args = tokenize(trimmed);

                if let Some(output) = execute_command(&args , &history) {
                    print!("{}", output);
                    let _ = io::stdout().flush();
                }else{
                    let _ = io::stdout().flush();
                }
            },
            Err(ReadlineError::Interrupted) => {
                continue;
            },
            Err(ReadlineError::Eof) => {
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}