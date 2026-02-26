use std::env;
use std::fs;
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;
use std::io::Write;
use std::fs::OpenOptions;

pub fn find_executable(name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;

    for directory in env::split_paths(&path_var) {
        let full_path = directory.join(name);

        if let Ok(metadata) = fs::metadata(&full_path) {
            if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                return Some(full_path);
            }
        }
    }
    None
}

pub fn write_in_file(path: &str, text: &str , append : bool) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(append)
        .create(true)
        .truncate(!append)
        .open(path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}