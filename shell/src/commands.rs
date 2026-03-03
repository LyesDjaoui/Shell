use std::process::{Command, Stdio};
use crate::utils;
use crate::utils::find_executable;
use std::fs::OpenOptions;

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

pub fn handle_echo(args: &[String], output_file: Option<String> , append: bool) -> Option<String> {
    let output = format!("{}\n", args.iter()
        .map(|a| a.trim_matches('\'').trim_matches('"'))
        .collect::<Vec<_>>()
        .join(" "));
    if let Some(path) = output_file {
        if let Err(_) = utils::write_in_file(&path, &output, append) {
            eprintln!("Error writing to file: {}", &path);
            None
        } else {
            None
        }
    } else {
        Some(output)
    }
}

pub fn handle_pipe(left_cmd: &[String], right_cmd: &[String]) -> Option<String> {
    if left_cmd.is_empty() || right_cmd.is_empty() {
        return None;
    }

    let left_exe = find_executable(&left_cmd[0])?;
    let right_exe = find_executable(&right_cmd[0])?;

    let mut left_child = Command::new(left_exe)
        .args(&left_cmd[1..])
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;

    let left_stdout = left_child.stdout.take()?;

    let mut right_child = Command::new(right_exe)
        .args(&right_cmd[1..])
        .stdin(Stdio::from(left_stdout))
        .stdout(Stdio::inherit())
        .spawn()
        .ok()?;

    let _ = left_child.wait();
    let _ = right_child.wait();

    None
}