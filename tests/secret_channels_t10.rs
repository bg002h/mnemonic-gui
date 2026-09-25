//! T10 — the OS gate (DESIGN §A6, §A9), run by `os_gate.py` (parsed YAML).
//!
//! An OS may be in `channel_policy.json` `private_channels_on` only if a
//! workflow triggered on `push`/`pull_request` has an unguarded job on that OS
//! whose step runs `cargo test`/`cargo nextest run` naming every
//! `real_binary_test_targets` target with a non-empty `MNEMONIC_BIN`.
//!
//! **Acceptance criterion (§A9, R3 Nit 3):** from the change that implements
//! Part A on, the repo's OWN workflows must pass the gate for every listed OS.
//! "Pending" is a HARD FAILURE here, never a report. The gate itself is pinned
//! against 14 decoys and 4 genuine fixtures on all three OSes.

mod secret_channels_common;

use mnemonic_gui::form::channels::policy;
use secret_channels_common::{measurement_dir, python_json};

fn gate(paths: &[String], os: &str) -> bool {
    let targets = serde_json::to_string(&policy().real_binary_test_targets).unwrap();
    let paths = serde_json::to_string(paths).unwrap();
    python_json(
        &format!(
            "import json, os_gate\nprint(json.dumps(os_gate.gate({paths}, {os:?}, {targets})))"
        ),
        &[],
    )
    .as_bool()
    .unwrap()
}

#[test]
fn t10_the_repos_own_workflows_pass_the_gate_for_every_enabled_os() {
    let wf: Vec<String> = python_json(
        "import json, os, os_gate\nprint(json.dumps(os_gate.repo_workflows(os.getcwd())))",
        &[],
    )
    .as_array()
    .unwrap()
    .iter()
    .map(|p| p.as_str().unwrap().to_string())
    .collect();
    assert!(!wf.is_empty(), "no workflows found");
    assert!(!policy().private_channels_on.is_empty());
    for os in &policy().private_channels_on {
        assert!(
            gate(&wf, os),
            "T10: `{os}` is in private_channels_on but no workflow runs {:?} on it with MNEMONIC_BIN \
             (the implementing change must add that job; 'pending' is a failure)",
            policy().real_binary_test_targets
        );
    }
}

#[test]
fn t10_the_gate_is_pinned_against_its_decoys_and_genuine_fixtures() {
    let ci = measurement_dir().join("fixtures/ci");
    let fixtures: &[(&str, &[&str])] = &[
        ("decoy_comment", &[]),
        ("decoy_if_false", &[]),
        ("decoy_missing_target", &[]),
        ("decoy_continue_on_error", &[]),
        ("decoy_echo", &[]),
        ("decoy_build_only", &[]),
        ("decoy_shell_comment", &[]),
        ("decoy_dispatch_only", &[]),
        ("decoy_matrix_exclude", &["linux"]),
        ("decoy_empty_bin", &[]),
        ("decoy_or_true", &[]),
        ("decoy_no_run", &[]),
        ("decoy_needs_skipped", &[]),
        ("decoy_filter_nothing", &[]),
        ("genuine_matrix", &["linux", "macos", "windows"]),
        ("genuine_linux", &["linux"]),
        ("genuine_workflow_env", &["macos"]),
        ("genuine_nocapture", &["macos"]),
    ];
    let decoys = fixtures
        .iter()
        .filter(|(n, _)| n.starts_with("decoy"))
        .count();
    assert_eq!(decoys, 14);
    assert_eq!(fixtures.len() - decoys, 4);
    // one python call for all 54 cells
    let mut cells = Vec::new();
    for (name, _) in fixtures {
        for os in ["linux", "macos", "windows"] {
            cells.push((
                ci.join(format!("{name}.yml"))
                    .to_string_lossy()
                    .into_owned(),
                os,
            ));
        }
    }
    let targets = serde_json::to_string(&policy().real_binary_test_targets).unwrap();
    let got = python_json(
        &format!(
            "import json, os_gate\nC={}\nprint(json.dumps([os_gate.gate([p], o, {targets}) for p, o in C]))",
            serde_json::to_string(&cells).unwrap()
        ),
        &[],
    );
    let got: Vec<bool> = got
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b.as_bool().unwrap())
        .collect();
    let mut k = 0;
    for (name, want) in fixtures {
        for os in ["linux", "macos", "windows"] {
            assert_eq!(got[k], want.contains(&os), "os gate fixture {name} {os}");
            k += 1;
        }
    }
}
