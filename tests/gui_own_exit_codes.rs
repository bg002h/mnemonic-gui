//! The GUI's own process exit statuses (`SPEC_constellation_exit_codes.md`
//! rev 5, R12): a bad command line is 64 (not clap's 2), `--help` and
//! `--version` are 0, and `--help` states the GUI's codes. Neither case opens
//! a window.
#![cfg(feature = "gui")]

use std::process::Command;

fn gui(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mnemonic-gui"))
        .args(args)
        .output()
        .expect("mnemonic-gui runs")
}

#[test]
fn a_bad_flag_exits_64() {
    let o = gui(&["--no-such-flag"]);
    assert_eq!(o.status.code(), Some(64), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stderr).contains("--no-such-flag"));
}

#[test]
fn help_and_version_exit_0_and_help_states_the_codes() {
    let v = gui(&["--version"]);
    assert_eq!(v.status.code(), Some(0), "{v:?}");
    let h = gui(&["--help"]);
    assert_eq!(h.status.code(), Some(0), "{h:?}");
    let text = String::from_utf8_lossy(&h.stdout)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        text.contains(
            "Exit status: 0 = closed normally. 32 = cannot start (no display, graphics \
             initialisation failed). 64 = bad command line. 101 = internal error. 130 = \
             stopped by a signal and the clean shutdown did not finish."
        ),
        "{text}"
    );
}

/// Review M5: a window that cannot open (no display at all) exits 32,
/// "cannot start", with a message. Linux only: elsewhere a display backend
/// does not depend on these variables.
#[cfg(target_os = "linux")]
#[test]
fn no_display_exits_32_cannot_start() {
    let home = std::env::temp_dir().join(format!("mnemonic-gui-own-exit-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_mnemonic-gui"))
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("WAYLAND_SOCKET")
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_DATA_HOME", home.join("data"))
        .env("XDG_STATE_HOME", home.join("state"))
        .env("XDG_RUNTIME_DIR", &home)
        .output()
        .expect("mnemonic-gui runs");
    assert_eq!(o.status.code(), Some(32), "{o:?}");
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("mnemonic-gui: cannot start:"),
        "{o:?}"
    );
}
