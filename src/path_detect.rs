//! `$PATH`-based detection of the four constellation CLIs (SPEC §8 class 1).
//!
//! `detect(name)` walks `$PATH` (and `$PATHEXT` on Windows) looking for an
//! executable matching `name`. On hit, returns `Found(absolute_path)`. On
//! miss, returns `NotFound`. The GUI uses this at launch to grey unavailable
//! tabs and to render the "install via …" tooltip.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Detected {
    Found(PathBuf),
    NotFound,
}

/// Resolve `name` against the process's `$PATH`. On Windows, also tries
/// each `$PATHEXT` extension when `name` has no extension.
pub fn detect(name: &str) -> Detected {
    detect_in(
        name,
        std::env::var_os("PATH"),
        std::env::var_os("PATHEXT"),
    )
}

/// Testable variant: resolve `name` against the supplied `$PATH` and
/// `$PATHEXT` env values. Pass `None` for either to behave as if the
/// variable were unset.
pub fn detect_in(name: &str, path_env: Option<OsString>, pathext: Option<OsString>) -> Detected {
    let path_env = path_env.unwrap_or_else(|| OsString::from(""));
    for dir in std::env::split_paths(&path_env) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let candidate = dir.join(name);
        if is_executable_file(&candidate) {
            return Detected::Found(candidate);
        }
        // Windows-style extension search. Gated by cfg!(windows) for
        // production code; tests can exercise it on Unix by passing a
        // pathext Some(...) value with a corresponding fixture file.
        if cfg!(windows) || pathext.is_some() {
            if let Some(ref pe) = pathext {
                for ext in std::env::split_paths(pe) {
                    let mut alt = candidate.clone().into_os_string();
                    alt.push(ext.as_os_str());
                    let alt_path = PathBuf::from(alt);
                    if is_executable_file(&alt_path) {
                        return Detected::Found(alt_path);
                    }
                }
            } else if cfg!(windows) {
                // Windows with no PATHEXT — fall back to .exe.
                let alt = candidate.with_extension("exe");
                if is_executable_file(&alt) {
                    return Detected::Found(alt);
                }
            }
        }
    }
    Detected::NotFound
}

fn is_executable_file(p: &Path) -> bool {
    is_executable_file_impl(p)
}

#[cfg(unix)]
fn is_executable_file_impl(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.is_file()
        && p.metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable_file_impl(p: &Path) -> bool {
    p.is_file()
}
