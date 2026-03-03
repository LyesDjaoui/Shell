use crate::commands;
use std::fs::OpenOptions;
use std::io::Write;

fn get_file_writer(path: &str, append: bool) -> std::io::Result<std::fs::File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .append(append)
        .truncate(!append)
        .open(path)
}

pub fn execute_command(args: &[String]) -> Option<String> {
    let mut output_file: Option<String> = None;
    let mut op = String::new();
    let mut clean_args = Vec::new();
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
            let right_cmd = args[i + 1..].to_vec();
            return commands::handle_pipe(&left_cmd, &right_cmd);
        } else {
            clean_args.push(args[i].clone());
            i += 1;
        }
    }

    let stdout_path = if op == ">" || op == "1>" || op == ">>" || op == "1>>" { output_file.clone() } else { None };
    let stderr_path = if op == "2>" || op == "2>>" { output_file.clone() } else { None };
    let stdout_append = op == ">>" || op == "1>>";
    let stderr_append = op == "2>>";

    match clean_args.as_slice() {
        [cmd] if cmd == "exit" => std::process::exit(0),

        [cmd, rest @ ..] if cmd == "echo" || cmd == "type" => {
            if let Some(path) = &stderr_path {
                let _ = get_file_writer(path, stderr_append);
            }

            if let Some(path) = &stdout_path {
                match get_file_writer(path, stdout_append) {
                    Ok(mut file) => {
                        if cmd == "echo" { commands::handle_echo(rest, &mut file); }
                        else { commands::handle_type(rest, &mut file); }
                    },
                    Err(e) => eprintln!("Error opening file {}: {}", path, e),
                }
            } else {
                if cmd == "echo" { commands::handle_echo(rest, &mut std::io::stdout()); }
                else { commands::handle_type(rest, &mut std::io::stdout()); }
            }
            None
        }

        [cmd, rest @ ..] => {
            let full_cmd: Vec<String> = std::iter::once(cmd.clone())
                .chain(rest.iter().cloned())
                .collect();
            commands::handle_external_command(
                &full_cmd, 
                stdout_path, 
                stderr_path, 
                stdout_append, 
                stderr_append
            )
        }

        [] => None,
    }
}