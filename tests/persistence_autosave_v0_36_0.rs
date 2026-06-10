//! v0.36.0 — atomic `save()` + change-gated `save_if_changed` pins.
//!
//! SPEC: `design/SPEC_gui_v0_36_0_autosave_atomic.md`.
//!
//! - T1 — `save()` is ATOMIC: writes a per-PID sibling temp
//!   (`state.json.<pid>.tmp` — the PID suffix is LOAD-BEARING: a shared
//!   fixed name lets two instances atomically install TORN content) then
//!   `fs::rename`s over the destination. Pre-impl RED: `save()` was a
//!   direct `fs::write`, so the pre-created PID-named garbage survives.
//! - T2 — `save_if_changed` debounce-by-content: first call writes,
//!   identical state skips (and must NOT recreate a user-deleted file),
//!   mutated state writes again; the cache updates ONLY after Ok.

use std::path::PathBuf;

use mnemonic_gui::persistence::{PersistedState, load, save, save_if_changed};

fn dest(dir: &tempfile::TempDir) -> PathBuf {
    dir.path().join("state.json")
}

// ── T1 — atomic save ────────────────────────────────────────────────────────

#[test]
fn t1_save_goes_through_the_pid_tmp_and_leaves_no_sibling() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dest(&dir);
    // The EXACT name save() must use (test and save() share a process).
    // Pre-creating a generic `state.json.tmp` would be red against a
    // CORRECT implementation (R0-r2 I3).
    let tmp = dir
        .path()
        .join(format!("state.json.{}.tmp", std::process::id()));
    std::fs::write(&tmp, "garbage-not-json").unwrap();

    let state = PersistedState {
        last_cli_tab: "md".into(),
        ..Default::default()
    };
    save(&state, &path).unwrap();

    assert!(
        !tmp.exists(),
        "save() must write through the PID-suffixed temp and rename it \
         away — the pre-created garbage temp survived (non-atomic write?)"
    );
    // No *.tmp sibling of any name remains (same-process glob).
    let leftover: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".tmp"))
        .collect();
    assert!(leftover.is_empty(), "stray temp files: {leftover:?}");
    let loaded = load(&path).expect("saved state must load");
    assert_eq!(loaded.last_cli_tab, "md");
}

#[test]
fn t1b_save_replaces_an_existing_destination() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dest(&dir);
    let first = PersistedState {
        last_cli_tab: "ms".into(),
        ..Default::default()
    };
    save(&first, &path).unwrap();
    let second = PersistedState {
        last_cli_tab: "mk".into(),
        ..Default::default()
    };
    save(&second, &path).unwrap();
    assert_eq!(
        load(&path).unwrap().last_cli_tab,
        "mk",
        "rename-over-existing must replace the destination"
    );
}

// ── T2 — change-gated save ──────────────────────────────────────────────────

#[test]
fn t2_save_if_changed_writes_then_skips_then_writes_on_change() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dest(&dir);
    let mut cache: Option<String> = None;

    let mut state = PersistedState {
        show_stdout: false,
        ..Default::default()
    };

    let wrote = save_if_changed(&state, &path, &mut cache).unwrap();
    assert!(wrote, "first call must write");
    assert!(path.exists());
    assert!(cache.is_some(), "cache must be set after a successful write");

    // Identical content → skip. Pin the skip by deleting the file: a
    // skip must NOT recreate it (the unconditional on_exit save covers
    // the user-deleted-mid-session case).
    std::fs::remove_file(&path).unwrap();
    let wrote = save_if_changed(&state, &path, &mut cache).unwrap();
    assert!(!wrote, "identical state must skip");
    assert!(
        !path.exists(),
        "a content-skip must not touch the filesystem"
    );

    // Mutated state → writes again.
    state.last_cli_tab = "md".into();
    let wrote = save_if_changed(&state, &path, &mut cache).unwrap();
    assert!(wrote, "changed state must write");
    assert_eq!(load(&path).unwrap().last_cli_tab, "md");
}

#[cfg(unix)]
#[test]
fn t2b_failed_write_does_not_poison_the_cache() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::TempDir::new().unwrap();
    let sub = dir.path().join("ro");
    std::fs::create_dir(&sub).unwrap();
    let path = sub.join("state.json");
    let mut cache: Option<String> = None;
    let state = PersistedState::default();

    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o555)).unwrap();
    let res = save_if_changed(&state, &path, &mut cache);
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755)).unwrap();

    assert!(res.is_err(), "write into a read-only dir must fail");
    assert!(
        cache.is_none(),
        "the cache updates ONLY after Ok — a failed write must leave it \
         untouched so the next interval retries"
    );
}
