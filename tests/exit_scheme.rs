//! Which exit-code scheme a CLI speaks (`SPEC_constellation_exit_codes.md`
//! rev 5, R11, R12, §7j, §9; review 2026-11-26 C1, I1, I2, M1, M2).
//!
//! Every old code is a valid new sum with another meaning, so the GUI must
//! not decode an old binary bit by bit. Version numbers cannot tell the two
//! apart (`mk 0.18.0-dev` exists both before and after the renumber), so the
//! scheme is read from the binary itself: R11 makes every renumbered CLI's
//! root `--help` END with the legend. A binary whose `--help` does not is
//! read by the old per-command meanings; one that cannot be asked is shown
//! undecoded. The answer is cached per binary (path, mtime, size), and every
//! probe has a timeout.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use mnemonic_gui::cli_version::{exit_scheme_cached, probe_scheme, scheme_from_help};
use mnemonic_gui::exit_badge::CodeScheme;
use mnemonic_gui::exit_code::HELP_LEGEND;
use mnemonic_gui::form::channels::RunPlan;
use mnemonic_gui::runner::run_plan_decoded;

/// A fresh directory for one test's fake CLIs.
fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "mnemonic-gui-exit-scheme-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A fake CLI `name` in `d`: `--help` prints `help` (exit `help_rc`),
/// `--version` prints `version`, anything else exits `rc`. Each `--help`
/// appends a line to `<d>/<name>.probes`.
fn fake(d: &Path, name: &str, help: &str, help_rc: i32, version: &str, rc: i32) -> PathBuf {
    let p = d.join(name);
    let help_file = d.join(format!("{name}.help"));
    std::fs::write(&help_file, help).unwrap();
    let script = format!(
        "#!/bin/sh\ncase \"$1\" in\n  --help) echo x >> '{probes}'; cat '{help}'; exit {help_rc};;\n  \
         --version) echo '{version}';;\n  *) echo out; exit {rc};;\nesac\n",
        probes = d.join(format!("{name}.probes")).display(),
        help = help_file.display(),
    );
    std::fs::write(&p, script).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    p
}

fn probes(d: &Path, name: &str) -> usize {
    std::fs::read_to_string(d.join(format!("{name}.probes")))
        .map(|s| s.lines().count())
        .unwrap_or(0)
}

fn help_with_legend() -> String {
    format!("Usage: mk <COMMAND>\n\nOptions:\n  -h, --help  Print help\n\n{HELP_LEGEND}\n")
}

/// The legend as `ms` prints it: clap re-wraps it at 80 columns.
fn help_with_wrapped_legend() -> String {
    let mut out = String::from("Usage: ms <COMMAND>\n\n");
    let mut line = String::new();
    for w in HELP_LEGEND.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + w.len() > 72 {
            out.push_str(&line);
            out.push('\n');
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(w);
    }
    out.push_str(&line);
    out.push('\n');
    out
}

fn undecoded(help: &str) -> String {
    match scheme_from_help(help) {
        CodeScheme::Undecoded(why) => why,
        other => panic!("expected Undecoded, got {other:?} for {help:?}"),
    }
}

#[test]
fn the_exact_legend_at_the_end_of_help_means_bits() {
    assert_eq!(scheme_from_help(&help_with_legend()), CodeScheme::Bits);
    assert_eq!(
        scheme_from_help(&help_with_wrapped_legend()),
        CodeScheme::Bits
    );
}

#[test]
fn no_exit_status_paragraph_at_all_means_legacy() {
    assert_eq!(
        scheme_from_help("Usage: md <COMMAND>\n"),
        CodeScheme::Legacy
    );
    assert_eq!(scheme_from_help(""), CodeScheme::Legacy);
    // The words inside a sentence are not a paragraph beginning with them.
    assert_eq!(
        scheme_from_help("Usage: md\n\n  Prints the exit status: see the manual.\n"),
        CodeScheme::Legacy
    );
}

/// Review I4 / M14: a legend that drifted is neither scheme.
#[test]
fn a_legend_that_is_not_last_is_undecoded() {
    let why = undecoded(&format!("{}\nMore help after it.\n", help_with_legend()));
    assert!(why.contains("Exit status:"), "{why}");
}

#[test]
fn a_legend_with_one_word_changed_is_undecoded() {
    let changed = help_with_legend().replace("a check said no", "a check said nay");
    assert_ne!(changed, help_with_legend());
    let why = undecoded(&changed);
    assert!(why.contains("Exit status:"), "{why}");
    // Wrapped the way ms and mnemonic wrap it, too.
    let changed = help_with_wrapped_legend().replace("refused", "declined");
    undecoded(&changed);
}

#[test]
fn a_fake_cli_with_a_drifted_legend_is_undecoded() {
    let d = dir("drift");
    let help = help_with_legend().replace("16 a correction", "16 the correction");
    let p = fake(&d, "md", &help, 0, "md 0.22.0", 5);
    assert!(matches!(
        probe_scheme(p.to_str().unwrap(), Duration::from_secs(5)),
        CodeScheme::Undecoded(_)
    ));
}

#[test]
fn a_fake_cli_with_the_legend_decodes() {
    let d = dir("bits");
    let p = fake(&d, "mk", &help_with_legend(), 0, "mk 0.18.0-dev", 20);
    assert_eq!(
        probe_scheme(p.to_str().unwrap(), Duration::from_secs(5)),
        CodeScheme::Bits
    );
}

#[test]
fn an_old_mk_0_18_0_dev_without_the_legend_is_read_the_old_way() {
    // mnemonic-key main before the renumber also says `mk 0.18.0-dev`.
    let d = dir("legacy");
    let p = fake(&d, "mk", "Usage: mk <COMMAND>\n", 0, "mk 0.18.0-dev", 4);
    assert_eq!(
        probe_scheme(p.to_str().unwrap(), Duration::from_secs(5)),
        CodeScheme::Legacy
    );
}

#[test]
fn a_help_that_fails_or_hangs_is_undecoded_within_the_timeout() {
    let d = dir("undecoded");
    let p = fake(&d, "md", &help_with_legend(), 3, "md 1.0.0", 0);
    assert!(matches!(
        probe_scheme(p.to_str().unwrap(), Duration::from_secs(5)),
        CodeScheme::Undecoded(_)
    ));
    let hang = d.join("hang");
    std::fs::write(&hang, "#!/bin/sh\nsleep 30\n").unwrap();
    std::fs::set_permissions(&hang, std::fs::Permissions::from_mode(0o755)).unwrap();
    let t = Instant::now();
    let s = probe_scheme(hang.to_str().unwrap(), Duration::from_millis(300));
    assert!(t.elapsed() < Duration::from_secs(5), "{:?}", t.elapsed());
    let CodeScheme::Undecoded(why) = s else {
        panic!("a hung probe must be undecoded")
    };
    assert!(why.contains("did not finish"), "{why}");
    assert!(matches!(
        probe_scheme("/definitely/not/a/binary", Duration::from_secs(1)),
        CodeScheme::Undecoded(_)
    ));
}

#[test]
fn the_answer_is_cached_per_binary_and_reprobed_when_it_changes() {
    let d = dir("cache");
    let p = fake(&d, "ms", "Usage: ms\n", 0, "ms 0.21.0", 0);
    let bin = p.to_str().unwrap();
    assert_eq!(exit_scheme_cached(bin), CodeScheme::Legacy);
    assert_eq!(exit_scheme_cached(bin), CodeScheme::Legacy);
    assert_eq!(probes(&d, "ms"), 1, "probed once, then cached");
    // An upgrade in place (different size) is probed again.
    std::fs::write(d.join("ms.help"), help_with_legend()).unwrap();
    let script = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, format!("{script}# upgraded\n")).unwrap();
    assert_eq!(exit_scheme_cached(bin), CodeScheme::Bits);
    assert_eq!(probes(&d, "ms"), 2);
}

#[test]
fn an_undecoded_answer_is_not_cached() {
    let d = dir("nocache");
    let p = fake(&d, "md", "Usage: md\n", 2, "md 0.21.0", 0);
    let bin = p.to_str().unwrap();
    assert!(matches!(exit_scheme_cached(bin), CodeScheme::Undecoded(_)));
    assert!(matches!(exit_scheme_cached(bin), CodeScheme::Undecoded(_)));
    assert_eq!(probes(&d, "md"), 2, "a failed probe is retried next run");
}

/// Review M1: the scheme reaches the result the badge reads.
#[test]
fn run_plan_decoded_stamps_the_scheme_on_the_result() {
    let d = dir("stamp");
    let p = fake(&d, "mk", &help_with_legend(), 0, "mk 0.18.0", 20);
    let r = run_plan_decoded(
        &RunPlan::plain(vec![p.to_str().unwrap().to_string(), "verify".into()]),
        &p,
    )
    .expect("runs");
    assert_eq!(r.exit_code, Some(20));
    assert_eq!(r.exit_scheme, CodeScheme::Bits);

    let q = fake(&d, "old", "Usage: md\n", 0, "md 0.21.0", 5);
    let r = run_plan_decoded(&RunPlan::plain(vec![q.to_str().unwrap().to_string()]), &q).unwrap();
    assert_eq!(
        (r.exit_code, r.exit_scheme.clone()),
        (Some(5), CodeScheme::Legacy)
    );
}

/// Review M12: the run executes the very file whose `--help` was read, even
/// when `argv[0]` is a bare name that would resolve elsewhere (or nowhere),
/// and the displayed argv is the plan's.
#[test]
fn the_run_executes_the_probed_file_not_a_fresh_lookup() {
    let d = dir("samefile");
    let p = fake(&d, "mk", &help_with_legend(), 0, "mk 0.18.0", 20);
    let plan = RunPlan::plain(vec!["mk-not-on-path-anywhere".into(), "verify".into()]);
    let r = run_plan_decoded(&plan, &p).expect("runs the resolved file");
    assert_eq!(r.exit_code, Some(20));
    assert_eq!(r.exit_scheme, CodeScheme::Bits);
    assert_eq!(r.argv, plan.argv);
    assert_eq!(probes(&d, "mk"), 1);
}

/// Review M1 / M12: the app's single run site resolves the binary once and
/// hands that file to the G1 `--version` probe and to the run (whose scheme
/// probe reads the same file), after the G1 gate.
#[test]
fn spawn_and_capture_runs_the_resolved_file_after_the_gate() {
    let src = include_str!("../src/app_window.rs");
    let body = &src[src.find("fn spawn_and_capture(").unwrap()..];
    let body = &body[..body.find("\n}\n").unwrap()];
    let gate = body.find("cli_version::gate_secret_run(").expect("G1 gate");
    assert!(
        body.contains("crate::cli_version::probe(&program_str)"),
        "the G1 probe reads the resolved file"
    );
    let run = body
        .find("runner::run_plan_decoded(&plan, &program)")
        .expect("spawn_and_capture must run the resolved file through run_plan_decoded");
    assert!(gate < run);
    assert!(
        !body.contains("runner::run_plan(&"),
        "the undecorated runner is not used"
    );
}
