use std::process::{Command, Stdio};
use crate::utils::find_executable;
use crate::utils::is_builtin_command;
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

pub fn handle_pipe(left_cmd: &[String], right_cmd: &[String]) -> Option<String> {
    if left_cmd.is_empty() || right_cmd.is_empty() {
        return None;
    }

    let (reader, mut writer) = os_pipe::pipe().ok()?;

    if is_builtin_command(&left_cmd[0]) {
        match left_cmd[0].as_str() {
            "echo" => handle_echo(&left_cmd[1..], &mut writer),
            "type" => handle_type(&left_cmd[1..], &mut writer),
            _ => {}
        }
        drop(writer);
    } else if let Some(exe) = find_executable(&left_cmd[0]) {
        let mut left_child = Command::new(exe)
            .args(&left_cmd[1..])
            .stdout(writer)
            .spawn()
            .ok()?;
    }

    if is_builtin_command(&right_cmd[0]) {
        let mut buffer = Vec::new();
        use std::io::Read;
        let _ = std::io::BufReader::new(reader).read_to_end(&mut buffer);

        handle_execute_builtin_command(right_cmd);
    } else if let Some(exe) = find_executable(&right_cmd[0]) {
        let mut right_child = Command::new(exe)
            .args(&right_cmd[1..])
            .stdin(reader)
            .stdout(Stdio::inherit())
            .spawn()
            .ok()?;
        
        let _ = right_child.wait();
    }

    None
}
