mod utils;
mod commands;
mod exec;
mod completer;

use crate::completer::ShellCompleter;
use crate::exec::execute_command;
use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};
use rustyline::history::MemHistory;

fn main() -> rustyline::Result<()> {
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

                let _ = rl.add_history_entry(trimmed);

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