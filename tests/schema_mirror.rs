//! Schema-mirror invariant test (SPEC §11).
//!
//! For each (CLI, subcommand) listed in the in-process `Schema`, shell out
//! to `<cli> <subcommand> --help`, regex-extract every `--<flag-name>`
//! token, and assert set-equality with the schema's flag-name set
//! (excluding the auto-injected `--help`).
//!
//! Binary lookup: `<CLI_uppercase>_BIN` env var (e.g. `MNEMONIC_BIN`) wins;
//! otherwise the literal binary name is invoked through `$PATH`.

use std::collections::BTreeSet;
use std::process::Command;

use mnemonic_gui::schema;

/// Extract every `--[a-z][a-z0-9-]+` token from `text`. Mirrors the
/// `grep -oE -- '--[a-z][a-z0-9-]+'` extractor used by
/// `mnemonic-toolkit/docs/manual/tests/lint.sh` (SPEC §11 prior-art).
fn extract_flag_names(text: &str) -> BTreeSet<String> {
    let bytes = text.as_bytes();
    let mut out = BTreeSet::new();
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b'-' && bytes[i + 1] == b'-' && bytes[i + 2].is_ascii_lowercase() {
            let start = i;
            i += 2;
            while i < bytes.len()
                && (bytes[i].is_ascii_lowercase() || bytes[i].is_ascii_digit() || bytes[i] == b'-')
            {
                i += 1;
            }
            let name = std::str::from_utf8(&bytes[start..i]).unwrap().to_string();
            // Defensive: trailing `--` (from `cargo test -- ...`) would
            // produce an empty name; the byte test above filters it.
            if name.len() > 2 {
                out.insert(name);
            }
        } else {
            i += 1;
        }
    }
    out
}

/// Resolve the binary name. `MNEMONIC_BIN` etc. override $PATH.
fn resolve_bin(cli_name: &str) -> String {
    let env_var = format!("{}_BIN", cli_name.to_ascii_uppercase().replace('-', "_"));
    std::env::var(&env_var).unwrap_or_else(|_| cli_name.to_string())
}

fn schema_flag_names(sub: &schema::SubcommandSchema) -> BTreeSet<String> {
    sub.flags.iter().map(|f| f.name.to_string()).collect()
}

fn help_text_flag_names(bin: &str, subcommand: &str) -> BTreeSet<String> {
    let output = Command::new(bin)
        .args([subcommand, "--help"])
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "failed to spawn `{} {} --help`: {} (set {}_BIN env var to override $PATH lookup)",
                bin,
                subcommand,
                e,
                bin.to_ascii_uppercase().replace('-', "_")
            )
        });
    // Some sibling CLIs (ms-cli, mk-cli) exit non-zero even on --help due
    // to a sysexits-style main wrapper (FOLLOWUPS candidate: convince
    // upstream to use clap's default `ExitCode::SUCCESS` for --help).
    // Stdout still carries the help text correctly, so we don't require
    // a zero exit code — only that stdout is non-empty.
    let text = if !output.stdout.is_empty() {
        String::from_utf8(output.stdout).expect("help-text stdout must be UTF-8")
    } else {
        panic!(
            "`{} {} --help` produced empty stdout (exit {:?})\nstderr:\n{}",
            bin,
            subcommand,
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    };
    let mut names = extract_flag_names(&text);
    // clap auto-injects `--help`; the schema deliberately omits it.
    names.remove("--help");
    names
}

fn assert_schema_matches_help(schema: &schema::Schema) {
    let bin = resolve_bin(schema.cli_name);
    for sub in schema.subcommands {
        let from_schema = schema_flag_names(sub);
        let from_help = help_text_flag_names(&bin, sub.name);
        let only_in_schema: Vec<_> = from_schema.difference(&from_help).collect();
        let only_in_help: Vec<_> = from_help.difference(&from_schema).collect();
        assert!(
            only_in_schema.is_empty() && only_in_help.is_empty(),
            "schema-mirror drift for `{} {}`:\n  only in schema: {:?}\n  only in upstream --help: {:?}",
            schema.cli_name,
            sub.name,
            only_in_schema,
            only_in_help,
        );
    }
}

#[test]
fn mnemonic_schema_flag_names_match_help_text() {
    assert_schema_matches_help(&schema::mnemonic::SCHEMA);
}

#[test]
fn md_schema_flag_names_match_help_text() {
    assert_schema_matches_help(&schema::md::SCHEMA);
}

#[test]
fn ms_schema_flag_names_match_help_text() {
    assert_schema_matches_help(&schema::ms::SCHEMA);
}

#[test]
fn mk_schema_flag_names_match_help_text() {
    assert_schema_matches_help(&schema::mk::SCHEMA);
}

#[test]
fn extract_flag_names_handles_basic_help_text() {
    let sample = "Options:\n  --network <NETWORK>  [possible values: mainnet]\n  --template <T>\n  -h, --help\n";
    let names = extract_flag_names(sample);
    assert!(names.contains("--network"));
    assert!(names.contains("--template"));
    assert!(names.contains("--help"));
    // Negative: dashes alone, single-dash short opts, uppercase starts.
    assert!(!names.contains("--"));
    assert!(!names.contains("--NETWORK"));
}
