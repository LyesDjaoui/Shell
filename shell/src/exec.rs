use crate::utils;
use crate::commands;
use std::fs::File;

pub fn execute_command(args: &[String]) -> Option<String> {
    let mut output_file: Option<String> = None;
    let mut op: String = String::new();
    let mut clean_args: Vec<String> = Vec::new();
    let mut i = 0;


    while i < args.len() {
        if args[i] == ">" || args[i] == "1>" || args[i] == "2>" || args[i] == ">>" || args[i] == "1>>" || args[i] == "2>>" {
            op = args[i].clone();
            if i + 1 < args.len() {
                output_file = Some(args[i + 1].clone());
                i += 2;
            }
        } else if args[i] == "|" {
                let left_cmd = args[..i].to_vec();
                let right_cmd = args[i+1..].to_vec();
                return commands::handle_pipe(&left_cmd, &right_cmd);
        } else {
            clean_args.push(args[i].clone());
            i += 1;
        }
    }

    let stdout_file = if op == ">" || op == "1>" || op == ">>" || op == "1>>" { output_file.clone() } else { None };
    let stderr_file = if op == "2>" || op == "2>>" { output_file.clone() } else { None };
    let append = op == ">>" || op == "1>>";
    let stderr_append = op == "2>>";

    let result = match clean_args.as_slice() {
        [cmd] if cmd == "exit" => std::process::exit(0),

        [cmd, rest @ ..] if cmd == "echo" => {
            if let Some(path) = &stderr_file {
                File::create(path).ok();
            }
            commands::handle_echo(rest, stdout_file.clone(), append)
        }

        [cmd, rest @ ..] if cmd == "type" => {
            if let Some(target) = rest.first() {
                if ["exit", "echo", "type"].contains(&target.as_str()) {
                    Some(format!("{} is a shell builtin\n", target))
                } else if let Some(path) = utils::find_executable(target) {
                    Some(format!("{} is {}\n", target, path.display()))
                } else {
                    Some(format!("{}: not found\n", target))
                }
            } else {
                None
            }
        }

        [cmd, rest @ ..] => {
            let full_cmd: Vec<String> = std::iter::once(cmd.clone())
                .chain(rest.iter().cloned())
                .collect();
            crate::commands::handle_external_command(&full_cmd, stdout_file.clone(), stderr_file.clone() ,append, stderr_append)
        }

        [] => None,
    };

    result
}
