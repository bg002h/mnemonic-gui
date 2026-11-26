//! The exit-code version floor on every run (`SPEC_constellation_exit_codes.md`
//! rev 5, §7j and §9). Every old exit code is a valid new sum with another
//! meaning (old 5 reads as NO + UNVERIFIED), so the GUI asks each binary for
//! its version before every run, secret or not, and decodes the status bit by
//! bit only when the binary is at or above the first version that speaks the
//! constellation's codes. Below the floor the run still happens; its code is
//! shown undecoded. The G1 secret-run refusal is unchanged and uses the same
//! single probe.

use mnemonic_gui::cli_version::{exit_bits_floor, exit_scheme, gate_run, parse_version_line};
use mnemonic_gui::exit_badge::CodeScheme;
use mnemonic_gui::form::channels::RunPlan;

fn ok(s: &str) -> Result<String, String> {
    Ok(s.to_string())
}

#[test]
fn every_cli_has_a_floor_naming_itself() {
    for cli in ["mnemonic", "md", "ms", "mk"] {
        let f = exit_bits_floor(cli).expect("a floor");
        let (name, _) = parse_version_line(f).expect("parses");
        assert_eq!(name, cli, "{f}");
    }
    assert_eq!(exit_bits_floor("mt"), None);
}

#[test]
fn at_or_above_the_floor_decodes() {
    for cli in ["mnemonic", "md", "ms", "mk"] {
        let f = exit_bits_floor(cli).unwrap();
        assert_eq!(
            exit_scheme(cli, &ok(&format!("{f}\n"))),
            CodeScheme::Bits,
            "{f}"
        );
    }
    assert_eq!(exit_scheme("md", &ok("md 0.30.0\n")), CodeScheme::Bits);
    assert_eq!(exit_scheme("ms", &ok("ms 1.0.0\n")), CodeScheme::Bits);
}

#[test]
fn below_the_floor_is_undecoded_and_says_why() {
    // Today's pins speak the old codes.
    let CodeScheme::Undecoded(why) = exit_scheme("md", &ok("md 0.21.0\n")) else {
        panic!("md 0.21.0 must not be decoded");
    };
    assert!(why.contains("md 0.21.0"), "{why}");
    assert!(
        why.contains("predates the constellation exit codes"),
        "{why}"
    );
    assert!(why.contains(exit_bits_floor("md").unwrap()), "{why}");
    assert!(matches!(
        exit_scheme("mnemonic", &ok("mnemonic 0.107.1\n")),
        CodeScheme::Undecoded(_)
    ));
}

#[test]
fn an_unknown_version_is_undecoded() {
    for (bin, rep) in [
        ("md", Err("`md --version` could not run".to_string())),
        ("md", ok("markdown 3.0.0\n")),
        ("md", ok("usage: md\n")),
        ("mt", ok("mt 9.9.9\n")),
    ] {
        assert!(
            matches!(exit_scheme(bin, &rep), CodeScheme::Undecoded(_)),
            "{bin} {rep:?}"
        );
    }
}

#[test]
fn a_path_or_exe_argv0_names_the_cli() {
    let f = exit_bits_floor("ms").unwrap();
    assert_eq!(exit_scheme("/usr/local/bin/ms", &ok(f)), CodeScheme::Bits);
    assert_eq!(exit_scheme("ms.exe", &ok(f)), CodeScheme::Bits);
}

#[test]
fn every_run_is_probed_once_and_a_plain_run_is_never_refused() {
    let p = RunPlan::plain(vec!["md".into(), "decode".into()]);
    let mut calls = 0;
    let r = gate_run(&p, |bin| {
        calls += 1;
        assert_eq!(bin, "md");
        ok("md 0.1.0\n")
    });
    assert_eq!(calls, 1);
    assert!(matches!(r, Ok(CodeScheme::Undecoded(_))), "{r:?}");

    let f = exit_bits_floor("md").unwrap();
    assert_eq!(gate_run(&p, |_| ok(f)), Ok(CodeScheme::Bits));
    // A probe that fails does not refuse a plain run either.
    assert!(matches!(
        gate_run(&p, |_| Err("no".into())),
        Ok(CodeScheme::Undecoded(_))
    ));
}
