mod utils;
mod commands;
mod exec;
use std::io::{self, Write};
use crate::exec::execute_command;


fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        let args: Vec<String> = input.trim().split_whitespace().map(String::from).collect();
        if args.is_empty() { continue; }


        if let Some(output) = execute_command(&args) {
        print!("{}", output);
    }
    }
}