use std::process::{Command, Stdio};
use std::io::Write;
use std::fs::OpenOptions;
use crate::utils::{find_executable, is_builtin_command, get_file_writer};

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

pub fn handle_pipe(
    args: &[String],
    stdout_path: Option<String>,
    stderr_path: Option<String>,
    stdout_append: bool,
    stderr_append: bool,
) -> Option<String> {
    let blocks: Vec<Vec<String>> = args
        .split(|s| s == "|")
        .map(|b| b.to_vec())
        .collect();

    let mut prev_reader: Option<os_pipe::PipeReader> = None;
    let mut children: Vec<std::process::Child> = Vec::new();

    for (i, cmd_args) in blocks.iter().enumerate() {
        if cmd_args.is_empty() {
            continue;
        }

        let is_last = i == blocks.len() - 1;

        let (current_stdout, next_reader) = if is_last {
            let stdio = if let Some(path) = &stdout_path {
                Stdio::from(get_file_writer(path, stdout_append).ok()?)
            } else {
                Stdio::inherit()
            };
            (stdio, None)
        } else {
            let (r, w) = os_pipe::pipe().ok()?;
            (Stdio::from(w), Some(r))
        };

        let stdin = prev_reader.take().map(Stdio::from).unwrap_or(Stdio::inherit());

        if is_builtin_command(&cmd_args[0]) {
            let cmd_args = cmd_args.clone();

            let (r, w) = os_pipe::pipe().ok()?;

            std::thread::spawn(move || {
                let mut writer = w;
                match cmd_args[0].as_str() {
                    "echo" => handle_echo(&cmd_args[1..], &mut writer),
                    "type" => handle_type(&cmd_args[1..], &mut writer),
                    _ => {}
                }
            });

            if is_last {
                let mut src = r;
                if let Some(path) = &stdout_path {
                    if let Ok(mut file) = get_file_writer(path, stdout_append) {
                        std::io::copy(&mut src, &mut file).ok();
                    }
                } else {
                    std::io::copy(&mut src, &mut std::io::stdout()).ok();
                }
            } else {
                prev_reader = Some(r);
            }

            continue;
        }

        if let Some(exe) = find_executable(&cmd_args[0]) {
            let mut cmd = Command::new(exe);
            cmd.args(&cmd_args[1..])
                .stdin(stdin)
                .stdout(current_stdout);

            if is_last {
                if let Some(path) = &stderr_path {
                    if let Ok(f) = get_file_writer(path, stderr_append) {
                        cmd.stderr(Stdio::from(f));
                    }
                }
            }

            if let Ok(child) = cmd.spawn() {
                children.push(child);
            } else {
                eprintln!("{}: failed to spawn", cmd_args[0]);
            }
        } else {
            eprintln!("{}: command not found", cmd_args[0]);
        }

        prev_reader = next_reader;
    }

    drop(prev_reader);

    for mut child in children {
        let _ = child.wait();
    }

    std::io::stdout().flush().ok();
    std::io::stderr().flush().ok();

    None
}