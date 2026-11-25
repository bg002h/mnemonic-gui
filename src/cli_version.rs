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
/// pinned one; otherwise `Err` with a message for the user.
pub fn check_reported(pinned: &str, reported: &str) -> Result<(), String> {
    let (want_name, want) = parse_version_line(pinned)
        .ok_or_else(|| format!("internal error: unparseable pinned version {pinned:?}"))?;
    let refuse = |what: String| {
        Err(format!(
            "{what}. This GUI sends secrets to `{want_name}` over private channels \
             (@env:, stdin, pipes) that were checked against {pinned}; an older or \
             different binary could read a channel reference as the secret itself \
             and derive the wrong wallet. Install {pinned} or newer first on $PATH \
             (or move the old one out of the way), then retry."
        ))
    };
    let Some((got_name, got)) = parse_version_line(reported) else {
        let first = reported.lines().next().unwrap_or("").trim();
        return refuse(format!(
            "`{want_name} --version` printed {first:?}, not a version this GUI recognises"
        ));
    };
    if got_name != want_name {
        return refuse(format!(
            "the `{want_name}` on $PATH reports itself as `{got_name}`, not `{want_name}`"
        ));
    }
    if got < want {
        return refuse(format!(
            "the `{want_name}` on $PATH is version {}.{}.{}, older than the pinned {pinned}",
            got.0, got.1, got.2
        ));
    }
    Ok(())
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
    let reported = probe(bin)?;
    check_reported(pinned, &reported)
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
