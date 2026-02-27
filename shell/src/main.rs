mod utils;
mod commands;
mod exec;
use std::io::{self, Write};
use crate::exec::execute_command;
mod completer;
use crate::completer::ShellCompleter;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;


fn main() -> rustyline::Result<()> {
    let mut rl: Editor<ShellCompleter, rustyline::history::FileHistory> = 
    Editor::with_config(rustyline::Config::builder().build())?;
    let h = ShellCompleter::new();
    rl.set_helper(Some(h));

loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(input) => {
                let _ = rl.add_history_entry(input.as_str());

                let args: Vec<String> = input.trim().split_whitespace().map(String::from).collect();
                
                if args.is_empty() { continue; }

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