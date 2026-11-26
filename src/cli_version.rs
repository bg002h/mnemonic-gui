//! Runtime version gate for secret-bearing runs (bug hunt 2026-11-25, G1).
//!
//! The secret-channel planner (`form::channels`) sends a secret as
//! `--passphrase @env:MNEMONIC_GUI_S<i>`, as `-` / `--X-stdin`, or as
//! `/dev/fd/N`. That table was measured against the PINNED binaries
//! (`Schema::pinned_version`). The binary actually spawned is whatever
//! `mnemonic` / `md` / `ms` / `mk` comes first on `$PATH`, and an older CLI
//! can read such a reference as the secret itself — for example an ms-cli
//! before the argv guard takes `@env:MNEMONIC_GUI_S0` literally as the BIP-39
//! passphrase and exits 0 with a different wallet.
//!
//! So before any run that carries a secret, the GUI asks the binary for
//! `--version` and refuses the run unless the binary names itself as the
//! expected CLI at a version at least the pinned one. The gate never changes
//! the plan: a run it admits is byte-identical to a run without it.

use crate::exit_badge::CodeScheme;
use crate::form::channels::RunPlan;

/// A `major.minor.patch` version. Pre-release and build suffixes
/// (`0.1.0-dev`, `+abc`) are ignored: only the numeric triple is compared.
pub type Version = (u64, u64, u64);

/// Parse the leading numeric `X.Y.Z` of a version token (an optional leading
/// `v` is accepted). `None` unless all three components are present.
pub fn parse_version(token: &str) -> Option<Version> {
    let t = token.trim().trim_start_matches('v');
    let core = t.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

/// Parse clap's `--version` line (`"<name> <version>"`, e.g. `"ms 0.21.0"`)
/// into the program name and its version. Only the first non-empty line is
/// read.
pub fn parse_version_line(text: &str) -> Option<(&str, Version)> {
    let line = text.lines().map(str::trim).find(|l| !l.is_empty())?;
    let mut toks = line.split_whitespace();
    let name = toks.next()?;
    let version = parse_version(toks.next()?)?;
    Some((name, version))
}

/// The pinned `--version` string for one of the four CLIs
/// (`Schema::pinned_version`), or `None` for an unknown name.
pub fn pinned_version_for(cli_name: &str) -> Option<&'static str> {
    use crate::schema;
    let s: &schema::Schema = match cli_name {
        "mnemonic" => &schema::mnemonic::SCHEMA,
        "md" => &schema::md::SCHEMA,
        "ms" => &schema::ms::SCHEMA,
        "mk" => &schema::mk::SCHEMA,
        _ => return None,
    };
    Some(s.pinned_version)
}

/// Judge a binary's `--version` output against the pinned string. `Ok(())`
/// iff the output names the same program as `pinned` at a version `>=` the
/// pinned one; otherwise `Err` with the short reason (the gate adds the
/// platform-accurate context, [`refusal_text`]).
pub fn check_reported(pinned: &str, reported: &str) -> Result<(), String> {
    let (want_name, want) = parse_version_line(pinned)
        .ok_or_else(|| format!("internal error: unparseable pinned version {pinned:?}"))?;
    let Some((got_name, got)) = parse_version_line(reported) else {
        let first = reported.lines().next().unwrap_or("").trim();
        return Err(format!(
            "`{want_name} --version` printed {first:?}, not a version this GUI recognises"
        ));
    };
    if got_name != want_name {
        return Err(format!(
            "the `{want_name}` this GUI runs reports itself as `{got_name}`, not `{want_name}`"
        ));
    }
    if got < want {
        return Err(format!(
            "the `{want_name}` this GUI runs is version {}.{}.{}, older than the pinned {pinned}",
            got.0, got.1, got.2
        ));
    }
    Ok(())
}

/// The user-facing refusal: `reason` plus how this OS's Run sends the secret
/// (`interim` = the argv path of macOS/Windows; else the private channels of
/// Linux — the same split as the run-confirm dialog and the paste warning),
/// a warning that the Copy command is not safe with this binary either (it
/// always uses the private-channel spelling), and the remedy.
pub fn refusal_text(reason: &str, pinned: &str, interim: bool) -> String {
    let cli = pinned.split_whitespace().next().unwrap_or(pinned);
    let how = if interim {
        format!(
            "On this OS the GUI passes secrets to `{cli}` on its command line with \
             --allow-argv-secret, checked against {pinned}; an older or different \
             binary may refuse or misread them."
        )
    } else {
        format!(
            "This GUI sends secrets to `{cli}` over private channels (@env:, stdin or \
             a pipe) checked against {pinned}; an older or different binary can read \
             a reference such as @env:MNEMONIC_GUI_S0 as the secret itself and derive \
             the wrong wallet."
        )
    };
    format!(
        "{reason}. {how} The copied command (Copy buttons) is not safe with this \
         binary either: an older `{cli}` can take its @env: or stdin references \
         literally as the secret. Install {pinned} or newer (README, \"Install\") so \
         that it is the `{cli}` found first on the search path, then retry."
    )
}

/// Ask `bin` for its version: stdout of `<bin> --version` on exit 0.
pub fn probe(bin: &str) -> Result<String, String> {
    match crate::runner::run([bin, "--version"]) {
        Ok(r) if r.exit_code == Some(0) => Ok(String::from_utf8_lossy(&r.stdout).into_owned()),
        Ok(r) => Err(format!(
            "`{bin} --version` failed (exit {:?}); cannot confirm the binary's version",
            r.exit_code
        )),
        Err(e) => Err(format!("`{bin} --version` could not run: {e}")),
    }
}

/// The gate: a plan with no secret binding passes unprobed; a plan with one
/// passes only if `probe(argv[0])` reports a version accepted by
/// [`check_reported`]. `probe` is injectable for tests; production passes
/// [`probe`].
pub fn gate_secret_run(
    plan: &RunPlan,
    probe: impl FnOnce(&str) -> Result<String, String>,
) -> Result<(), String> {
    if !plan.has_secrets() {
        return Ok(());
    }
    let bin = plan.argv.first().map(String::as_str).unwrap_or("");
    let pinned = pinned_version_for(bin)
        .ok_or_else(|| format!("refusing a secret-bearing run of unknown binary `{bin}`"))?;
    let reported = probe(bin).map_err(|e| refusal_text(&e, pinned, plan.interim))?;
    check_reported(pinned, &reported).map_err(|e| refusal_text(&e, pinned, plan.interim))
}

/// The first version of each CLI that exits with the constellation's codes
/// (`SPEC_constellation_exit_codes.md` rev 5, §3; floor per §7j and §9), in
/// the `--version` form, or `None` for a binary the GUI does not run.
///
/// Every old code is a valid new sum with another meaning, so a status from a
/// binary below its floor is never decoded bit by bit. These are the next
/// releases after the pins of 2026-11-26 (`md 0.21.0`, `ms 0.21.0`,
/// `mk 0.17.0`, `mnemonic 0.107.1`); the cutover release of each CLI must
/// carry at least this version. A `-dev` build reports its numeric triple
/// only, so a pre-cutover `mk 0.18.0-dev` would be taken as speaking the new
/// codes.
pub fn exit_bits_floor(cli_name: &str) -> Option<&'static str> {
    match cli_name {
        "mnemonic" => Some("mnemonic 0.108.0"),
        "md" => Some("md 0.22.0"),
        "ms" => Some("ms 0.22.0"),
        "mk" => Some("mk 0.18.0"),
        _ => None,
    }
}

/// `argv[0]`'s file name without a `.exe` suffix.
fn cli_name(bin: &str) -> &str {
    let name = std::path::Path::new(bin)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(bin);
    name.strip_suffix(".exe").unwrap_or(name)
}

/// How to read the exit status of a run of `bin`, given its `--version`
/// answer: decode the bits only when it names the same CLI at a version at or
/// above [`exit_bits_floor`]; otherwise say why the status is not decoded.
pub fn exit_scheme(bin: &str, reported: &Result<String, String>) -> CodeScheme {
    let cli = cli_name(bin);
    let Some(floor) = exit_bits_floor(cli) else {
        return CodeScheme::Undecoded(format!("`{cli}` is not a CLI this GUI decodes"));
    };
    let (_, want) = parse_version_line(floor).expect("floors parse (unit-tested)");
    let reported = match reported {
        Ok(r) => r,
        Err(e) => return CodeScheme::Undecoded(e.clone()),
    };
    let Some((got_name, got)) = parse_version_line(reported) else {
        return CodeScheme::Undecoded(format!(
            "`{cli} --version` did not name a version this GUI recognises"
        ));
    };
    if got_name != cli {
        return CodeScheme::Undecoded(format!(
            "the `{cli}` this GUI runs reports itself as `{got_name}`"
        ));
    }
    if got < want {
        return CodeScheme::Undecoded(format!(
            "`{cli} {}.{}.{}` predates the constellation exit codes, which start at {floor}",
            got.0, got.1, got.2
        ));
    }
    CodeScheme::Bits
}

/// The gate before EVERY run (exit-code spec §7j): ask the binary for its
/// version once, refuse a secret-bearing run exactly as [`gate_secret_run`]
/// does, and otherwise return how the run's exit status may be read. A run
/// without a secret is never refused, whatever the probe says.
pub fn gate_run(
    plan: &RunPlan,
    probe: impl FnOnce(&str) -> Result<String, String>,
) -> Result<CodeScheme, String> {
    let bin = plan.argv.first().map(String::as_str).unwrap_or("");
    let reported = probe(bin);
    gate_secret_run(plan, |_| reported.clone())?;
    Ok(exit_scheme(bin, &reported))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clap_version_lines() {
        assert_eq!(parse_version_line("ms 0.21.0\n"), Some(("ms", (0, 21, 0))));
        assert_eq!(
            parse_version_line("mnemonic 0.107.1-dev (abc123)\n"),
            Some(("mnemonic", (0, 107, 1)))
        );
        assert_eq!(parse_version_line("md v1.2.3"), Some(("md", (1, 2, 3))));
        assert_eq!(parse_version_line("ms"), None);
        assert_eq!(parse_version_line("ms 0.21"), None);
        assert_eq!(parse_version_line(""), None);
    }

    #[test]
    fn every_pinned_version_parses_and_names_its_cli() {
        for cli in ["mnemonic", "md", "ms", "mk"] {
            let p = pinned_version_for(cli).unwrap();
            let (name, _) = parse_version_line(p).unwrap();
            assert_eq!(name, cli, "pinned_version {p:?}");
            assert_eq!(check_reported(p, p), Ok(()));
        }
    }

    #[test]
    fn older_version_is_refused_newer_accepted() {
        assert!(check_reported("ms 0.21.0", "ms 0.16.0\n").is_err());
        assert!(check_reported("ms 0.21.0", "ms 0.20.9\n").is_err());
        assert!(check_reported("mnemonic 0.107.1", "mnemonic 0.107.0\n").is_err());
        assert_eq!(check_reported("ms 0.21.0", "ms 0.21.0\n"), Ok(()));
        assert_eq!(check_reported("ms 0.21.0", "ms 0.22.0-dev\n"), Ok(()));
        assert_eq!(check_reported("ms 0.21.0", "ms 1.0.0\n"), Ok(()));
        // 0.100.0 > 0.21.0 numerically (not lexically).
        assert_eq!(check_reported("ms 0.21.0", "ms 0.100.0\n"), Ok(()));
    }

    #[test]
    fn wrong_program_or_garbage_is_refused() {
        let e = check_reported("md 0.21.0", "md 9.9.9-other-tool\n");
        assert_eq!(e, Ok(()), "same name, newer version passes");
        assert!(check_reported("md 0.21.0", "markdown 3.0.0\n").is_err());
        assert!(check_reported("md 0.21.0", "usage: md [options]\n").is_err());
        assert!(check_reported("md 0.21.0", "").is_err());
    }

    #[test]
    fn refusal_text_is_platform_accurate_and_warns_about_copy() {
        let private = refusal_text("r", "ms 0.21.0", false);
        assert!(
            private.contains("over private channels (@env:"),
            "{private}"
        );
        assert!(!private.contains("--allow-argv-secret"), "{private}");
        let interim = refusal_text("r", "ms 0.21.0", true);
        assert!(
            interim.contains("on its command line with --allow-argv-secret"),
            "{interim}"
        );
        assert!(!interim.contains("private channels"), "{interim}");
        for t in [&private, &interim] {
            assert!(t.starts_with("r. "), "{t}");
            assert!(
                t.contains("The copied command (Copy buttons) is not safe with this binary either"),
                "{t}"
            );
            assert!(t.contains("Install ms 0.21.0 or newer"), "{t}");
            assert!(!t.contains("$PATH"), "{t}");
        }
    }

    #[test]
    fn exit_floors_parse_and_name_their_cli() {
        for cli in ["mnemonic", "md", "ms", "mk"] {
            let (name, _) = parse_version_line(exit_bits_floor(cli).unwrap()).unwrap();
            assert_eq!(name, cli);
        }
    }

    #[test]
    fn plan_without_secrets_is_not_probed() {
        let p = RunPlan::plain(vec!["ms".into(), "decode".into()]);
        assert_eq!(gate_secret_run(&p, |_| panic!("must not probe")), Ok(()));
    }

    #[cfg(unix)]
    #[test]
    fn probe_reads_a_real_binary() {
        // `true` exits 0 with empty stdout → probe Ok(""), which check_reported refuses.
        let out = probe("true").unwrap();
        assert!(check_reported("ms 0.21.0", &out).is_err());
        assert!(probe("definitely-not-a-binary-mnemonic-gui").is_err());
    }
}
