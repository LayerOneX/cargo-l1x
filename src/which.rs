use std::env;
use std::fs;
use std::path::PathBuf;

pub fn which(program: String) -> Option<PathBuf> {
    if let Some(path) = try_which_from_path(&program) {
        return Some(path);
    }

    let paths = env::var_os("PATH").unwrap_or_default();
    for path in env::split_paths(&paths) {
        let candidate = path.join(&program);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }

    None
}

fn try_which_from_path(program: &String) -> Option<PathBuf> {
    let path = PathBuf::from(program);
    if path.is_absolute() && is_executable(&path) {
        return Some(path);
    }

    None
}

fn is_executable(path: &PathBuf) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && has_executable_permissions(&metadata))
        .unwrap_or(false)
}

fn has_executable_permissions(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;

    metadata.permissions().mode() & 0o111 != 0
}
