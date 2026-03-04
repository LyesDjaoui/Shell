use std::process::{Command, Stdio};
use crate::utils::find_executable;
use crate::utils::is_builtin_command;
use crate::utils::get_file_writer;
use crate::utils;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::io::FromRawFd;
use std::fs::File;

pub fn handle_external_command(args: &[String], stdout_file: Option<String>, stderr_file: Option<String>, append: bool , stderr_append: bool) -> Option<String> {
    if args.is_empty() {
        return None;
    }

    if let Some(_) = find_executable(&args[0]) {
        let stdout: Stdio = match &stdout_file {
            Some(file_path) => match OpenOptions::new()
                .write(true)
                .append(append)
                .create(true)
                .truncate(!append)
                .open(file_path)
            {
                Ok(file) => Stdio::from(file),
                Err(e) => {
                    eprintln!("Cannot create file {}: {}", file_path, e);
                    return None;
                }
            },
            None => Stdio::inherit(),
        };

        let stderr: Stdio = match &stderr_file {
            Some(file_path) => match OpenOptions::new()
                .write(true)
                .append(stderr_append)
                .create(true)
                .truncate(!stderr_append)
                .open(file_path)
            {
                Ok(file) => Stdio::from(file),
                Err(e) => {
                    eprintln!("Cannot create file {}: {}", file_path, e);
                    return None;
                }
            },
            None => Stdio::inherit(),
        };

        match Command::new(&args[0])
            .args(&args[1..])
            .stdout(stdout)
            .stderr(stderr)
            .status()
        {
            Ok(_) => None,
            Err(_) => None,
        }
    } else {
        let msg = format!("{}: command not found\n", args[0]);
        Some(msg)
    }
}

pub fn handle_echo(args: &[String], writer: &mut dyn Write) {
    let output = args.iter()
        .map(|a| a.trim_matches('\'').trim_matches('"'))
        .collect::<Vec<_>>()
        .join(" ");

    if let Err(e) = writeln!(writer, "{}", output) {
        eprintln!("Error writing echo: {}", e);
    }
}
pub fn handle_type(args: &[String], writer: &mut dyn Write) {
    if args.is_empty() {
        return;
    }
    let target = &args[0];
    let message = if ["exit", "echo", "type"].contains(&target.as_str()) {
        format!("{} is a shell builtin\n", target)
    } else if let Some(path) = find_executable(target) {
        format!("{} is {}\n", target, path.display())
    } else {
        format!("{}: not found\n", target)
    };

    let _ = write!(writer, "{}", message);
}

pub fn handle_execute_builtin_command(args: &[String]) -> Option<String> {
    match args[0].as_str() {
        "exit" => std::process::exit(0),
        "echo" => {
            handle_echo(&args[1..], &mut std::io::stdout());
            None
        },
        "type" => {
            handle_type(&args[1..], &mut std::io::stdout());
            None
        },
        _ => None,
    }
}

pub fn handle_pipe(args: &[String], stdout_path: Option<String>, stderr_path: Option<String>,stdout_append: bool,stderr_append: bool) -> Option<String> {
    let commands_blocks: Vec<Vec<String>> = args
        .split(|s| s == "|")
        .map(|m| m.to_vec())
        .collect();

    let mut prev_reader: Option<os_pipe::PipeReader> = None;
    let mut children = Vec::new();

    for (i, cmd_args) in commands_blocks.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == commands_blocks.len() - 1;

        let (reader, writer) = os_pipe::pipe().ok()?;

        let stdin = if is_first {
            Stdio::inherit()
        } else {
            Stdio::from(prev_reader.take().unwrap()) 
        };

        let stdout = if is_last {
            if let Some(path) = &stdout_path {
                Stdio::from(utils::get_file_writer(path, stdout_append).ok()?)
            } else {
                Stdio::inherit()
            }
        } else {
            Stdio::from(writer)
        };

        if let Some(exe) = find_executable(&cmd_args[0]) {
            let mut cmd = Command::new(exe);
            cmd.args(&cmd_args[1..]).stdin(stdin).stdout(stdout);

            if is_last {
                if let Some(path) = &stderr_path {
                    cmd.stderr(Stdio::from(get_file_writer(path, stderr_append).ok()?));
                }
            }

            let child = cmd.spawn().ok()?;
            children.push(child);
        }

        prev_reader = Some(reader);
    }

    for mut child in children {
        let _ = child.wait();
    }
    None
}
