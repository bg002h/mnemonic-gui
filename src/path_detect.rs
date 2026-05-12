//! `$PATH`-based detection of the four constellation CLIs (SPEC §8 class 1).
//!
//! `detect(name)` walks `$PATH` (and `$PATHEXT` on Windows) looking for an
//! executable matching `name`. On hit, returns `Found(absolute_path)`. On
//! miss, returns `NotFound`. The GUI uses this at launch to grey unavailable
//! tabs and to render the "install via …" tooltip.

use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Detected {
    Found(PathBuf),
    NotFound,
}

/// Resolve `name` against `$PATH`. On Windows, also tries each
/// `$PATHEXT` extension when `name` has no extension (so `detect("md")`
/// finds `md.exe`).
pub fn detect(name: &str) -> Detected {
    let path_env = std::env::var_os("PATH").unwrap_or_else(|| OsString::from(""));
    for dir in std::env::split_paths(&path_env) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let candidate = dir.join(name);
        if is_executable_file(&candidate) {
            return Detected::Found(candidate);
        }
        // Windows-style extensions: try each $PATHEXT suffix when the
        // candidate name lacks an extension.
        if cfg!(windows) {
            if let Some(pathext) = std::env::var_os("PATHEXT") {
                for ext in std::env::split_paths(&pathext) {
                    let mut alt = candidate.clone().into_os_string();
                    alt.push(ext);
                    let alt_path = PathBuf::from(alt);
                    if is_executable_file(&alt_path) {
                        return Detected::Found(alt_path);
                    }
                }
            } else {
                // No PATHEXT — fall back to .exe.
                let alt = candidate.with_extension("exe");
                if is_executable_file(&alt) {
                    return Detected::Found(alt);
                }
            }
        }
    }
    Detected::NotFound
}

#[cfg(unix)]
fn is_executable_file(p: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.is_file()
        && p.metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable_file(p: &std::path::Path) -> bool {
    p.is_file()
}
