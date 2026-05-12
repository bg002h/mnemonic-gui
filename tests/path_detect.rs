//! `path_detect` tests (SPEC §8 class 1 + IMPL_PLAN §C Phase 6).
//!
//! Uses `detect_in()` (Phase 6 refactor) with explicit PATH + PATHEXT
//! overrides instead of mutating process environment variables (which
//! would race with parallel tests).
//!
//! Coverage:
//!   - all four binaries present in a mock PATH
//!   - mnemonic-only (md / ms / mk all missing)
//!   - md-missing-only (mnemonic / ms / mk present)
//!   - Windows-style `.exe` suffix via PATHEXT (exercised on Unix using
//!     the testable PATHEXT override path)

use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use mnemonic_gui::path_detect::{detect_in, Detected};
use tempfile::TempDir;

fn make_executable(dir: &std::path::Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    {
        let mut f = File::create(&p).expect("create");
        writeln!(f, "#!/bin/sh\nexit 0").unwrap();
    }
    let mut perms = std::fs::metadata(&p).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&p, perms).unwrap();
    p
}

fn make_non_executable(dir: &std::path::Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    File::create(&p).unwrap();
    let mut perms = std::fs::metadata(&p).unwrap().permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(&p, perms).unwrap();
    p
}

fn path_env_for(dirs: &[&std::path::Path]) -> OsString {
    let joined = dirs
        .iter()
        .map(|d| d.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(":");
    OsString::from(joined)
}

#[test]
fn cell_1_all_four_binaries_present() {
    let dir = TempDir::new().unwrap();
    for name in ["mnemonic", "md", "ms", "mk"] {
        make_executable(dir.path(), name);
    }
    let path_env = path_env_for(&[dir.path()]);

    for name in ["mnemonic", "md", "ms", "mk"] {
        let result = detect_in(name, Some(path_env.clone()), None);
        match result {
            Detected::Found(p) => assert_eq!(p, dir.path().join(name)),
            Detected::NotFound => panic!("expected {} to be Found", name),
        }
    }
}

#[test]
fn cell_2_mnemonic_only_others_missing() {
    let dir = TempDir::new().unwrap();
    make_executable(dir.path(), "mnemonic");
    let path_env = path_env_for(&[dir.path()]);

    matches!(
        detect_in("mnemonic", Some(path_env.clone()), None),
        Detected::Found(_)
    );
    for name in ["md", "ms", "mk"] {
        assert_eq!(
            detect_in(name, Some(path_env.clone()), None),
            Detected::NotFound,
            "{} should be NotFound",
            name
        );
    }
}

#[test]
fn cell_3_partial_availability_md_missing() {
    let dir = TempDir::new().unwrap();
    for name in ["mnemonic", "ms", "mk"] {
        make_executable(dir.path(), name);
    }
    let path_env = path_env_for(&[dir.path()]);

    assert_eq!(
        detect_in("md", Some(path_env.clone()), None),
        Detected::NotFound,
        "md must be NotFound"
    );
    for name in ["mnemonic", "ms", "mk"] {
        assert!(
            matches!(
                detect_in(name, Some(path_env.clone()), None),
                Detected::Found(_)
            ),
            "{} must be Found",
            name
        );
    }
}

#[test]
fn cell_4_windows_pathext_exe_suffix() {
    // Place a file named `md.exe` in the dir; pass `PATHEXT=.EXE` so the
    // detect_in fallback discovers it as `md`. Exercises the Windows
    // code path on Unix via the testable override.
    let dir = TempDir::new().unwrap();
    make_executable(dir.path(), "md.exe");
    // The bare `md` file does NOT exist in the dir.
    let path_env = path_env_for(&[dir.path()]);
    // PATHEXT is uppercase on real Windows (`.COM;.EXE;.BAT;...`), but
    // Windows file systems are case-insensitive so the lookup matches
    // regardless. Unix is case-sensitive: use `.exe` here to match the
    // lowercase fixture filename we created above.
    let pathext = OsString::from(".exe");

    let result = detect_in("md", Some(path_env), Some(pathext));
    match result {
        Detected::Found(p) => assert_eq!(p, dir.path().join("md.exe")),
        Detected::NotFound => panic!("expected md.exe to be Found via PATHEXT"),
    }
}

#[test]
fn cell_5_non_executable_file_is_not_found() {
    // A regular file (mode 0o644) in PATH must not match. SPEC §8 class 1
    // implies executability is part of the resolution contract on Unix.
    let dir = TempDir::new().unwrap();
    make_non_executable(dir.path(), "mnemonic");
    let path_env = path_env_for(&[dir.path()]);

    assert_eq!(
        detect_in("mnemonic", Some(path_env), None),
        Detected::NotFound,
        "non-executable file must not be Found"
    );
}

#[test]
fn cell_6_first_path_entry_wins() {
    // PATH precedence: first matching dir wins (matches POSIX semantics).
    let dir_a = TempDir::new().unwrap();
    let dir_b = TempDir::new().unwrap();
    let bin_a = make_executable(dir_a.path(), "mnemonic");
    make_executable(dir_b.path(), "mnemonic");
    let path_env = path_env_for(&[dir_a.path(), dir_b.path()]);

    let result = detect_in("mnemonic", Some(path_env), None);
    match result {
        Detected::Found(p) => assert_eq!(p, bin_a, "first PATH entry must win"),
        Detected::NotFound => panic!("expected Found"),
    }
}

#[test]
fn cell_7_empty_or_missing_path_yields_notfound() {
    assert_eq!(detect_in("mnemonic", None, None), Detected::NotFound);
    assert_eq!(
        detect_in("mnemonic", Some(OsString::from("")), None),
        Detected::NotFound
    );
}
