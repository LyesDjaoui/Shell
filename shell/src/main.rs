mod utils;
mod commands;
mod exec;
mod completer;

use crate::completer::ShellCompleter;
use crate::exec::execute_command;
use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};

fn main() -> rustyline::Result<()> {
    let config = Config::builder()
        .auto_add_history(true)
        .build();

    let mut rl: Editor<ShellCompleter, rustyline::history::FileHistory> = 
        Editor::with_config(config)?;

    let h = ShellCompleter::new();
    rl.set_helper(Some(h));

    loop {
        let readline = rl.readline("$ ");

        match readline {
            Ok(input) => {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let args: Vec<String> = trimmed
                    .split_whitespace()
                    .map(String::from)
                    .collect();

                if let Some(output) = execute_command(&args) {
                    print!("{}", output);
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