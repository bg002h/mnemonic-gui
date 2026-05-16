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
        // v0.3: prefer `gui-schema` JSON path when the CLI is
        // `gui-schema-capable = true` (all 4 CLIs are at Phase C.3).
        // The JSON path handles **flattened** nested-subcommand names
        // like `seed-xor-split` / `slip39-split` that clap's `--help`
        // doesn't recognize at the top-level (`mnemonic seed-xor-split
        // --help` errors with "unrecognized subcommand"). Fall back to
        // `--help` regex for non-capable CLIs or if the JSON path
        // returns `None` for an unexpected reason.
        let from_upstream = match mnemonic_gui::schema_check::json_flag_names(
            schema.cli_name,
            sub.name,
        ) {
            Some(names) => names,
            None => help_text_flag_names(&bin, sub.name),
        };
        let only_in_schema: Vec<_> = from_schema.difference(&from_upstream).collect();
        let only_in_upstream: Vec<_> = from_upstream.difference(&from_schema).collect();
        assert!(
            only_in_schema.is_empty() && only_in_upstream.is_empty(),
            "schema-mirror drift for `{} {}`:\n  only in schema: {:?}\n  only in upstream: {:?}",
            schema.cli_name,
            sub.name,
            only_in_schema,
            only_in_upstream,
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

// v0.4.0: The Phase 7 source-level audit (`mod source_audit`) was
// deleted in lockstep with `build.rs`. The taxonomy is now a public
// crate API on `mnemonic-toolkit` (v0.14.0+ `pub mod secret_taxonomy`),
// with per-variant parity tests enforced inside the toolkit at toolkit
// test time. The GUI consumes the constants via `pub use` in
// `src/secrets.rs`, with an additional compile-time supply-chain
// guard against drift from the v0.3.3 committed snapshot. Runtime
// backstop in `tests/secret_taxonomy_pin.rs`.

// ── Phase 9: CI workflow snapshot regression guard (continued below) ───


// ── Phase 9: CI workflow snapshot regression guard ─────────────────────
//
// Phase 9 IMPL_PLAN: tests/schema_mirror.rs::ci_workflow_snapshot reads
// .github/workflows/schema-mirror.yml and asserts the YAML's required
// step names are present. Authored AFTER the workflow YAML is on disk
// (post-implementation snapshot, NOT RED-driver) — guards against
// accidental workflow regression.

#[test]
fn ci_workflow_snapshot() {
    let workflow_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/schema-mirror.yml");
    let body = std::fs::read_to_string(&workflow_path)
        .unwrap_or_else(|e| panic!("read {:?}: {}", workflow_path, e));

    // Required step names (substring assertion — simpler + more
    // diff-resistant than full YAML parsing).
    let required_steps = [
        "install-mnemonic-toolkit",
        "install-md-cli",
        "install-ms-cli",
        "install-mk-cli",
        "cargo-test-schema-mirror",
        // Phase C.3 v0.2: per-CLI gui-schema smoke steps assert that
        // each installed binary exposes a SPEC §7 `gui-schema` subcommand
        // emitting `{version:1, cli:<name>, ...}`. If the smoke step is
        // dropped the CI gate stops noticing a regression at the
        // subcommand boundary even when the rest of the workflow
        // passes.
        "smoke-gui-schema-mnemonic",
        "smoke-gui-schema-md",
        "smoke-gui-schema-ms",
        "smoke-gui-schema-mk",
    ];
    for step in &required_steps {
        assert!(
            body.contains(step),
            "schema-mirror.yml must contain step `{}`",
            step
        );
    }

    // Required pinned tags — the workflow installs the binaries that
    // tests/schema_mirror.rs cells run against. If the pin drifts
    // here without the schema also drifting, the CI gate fails.
    //
    // v0.4.0 bump: mnemonic-toolkit pin → v0.14.0 (the release with
    // `pub mod secret_taxonomy` consumed by `src/secrets.rs`). All
    // other pins unchanged from v0.3.x.
    let required_tags = [
        "mnemonic-toolkit-v0.14.0",
        "descriptor-mnemonic-md-cli-v0.5.0",
        "ms-cli-v0.2.1",
        "mk-cli-v0.3.1",
    ];
    for tag in &required_tags {
        assert!(
            body.contains(tag),
            "schema-mirror.yml must pin tag `{}`",
            tag
        );
    }

    // v0.4.0: `MNEMONIC_GUI_UPSTREAM_ROOT` assertion was retired with
    // the source_audit module. The env var is no longer set by the
    // workflow (no consumer in the test tree after source_audit
    // deletion).
}

// ── Phase A.1 (v0.2): doubled-prefix artifact fix snapshot ─────────────
//
// SPEC §9 / Phase A.1: build.yml computes a `VERSION` env var by
// stripping the `mnemonic-gui-` prefix from `github.ref_name`, and
// templates artifact filenames against `${{ env.VERSION }}`. Pre-fix,
// the workflow templated against `${{ env.REF_NAME }}` (raw ref name),
// producing doubled-prefix artifacts like
// `mnemonic-gui-mnemonic-gui-v0.1.0-x86_64-linux.tar.gz`. This cell
// pins the fix: future edits that drop `compute-version` or reintroduce
// `env.REF_NAME` in any artifact-name template fail this snapshot.
//
// Authored AFTER the workflow YAML is on disk (post-implementation
// snapshot, NOT RED-driver) — Phase A.1 declared a RED-test exception
// because CI YAML correctness is not reachable from `cargo test`.

#[test]
fn ci_build_version_step_present() {
    let workflow_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/build.yml");
    let body = std::fs::read_to_string(&workflow_path)
        .unwrap_or_else(|e| panic!("read {:?}: {}", workflow_path, e));

    // Required: a `compute-version` step exists and uses `shell: bash`
    // (Windows runners default to PowerShell; bash makes the strip
    // portable across all five matrix targets).
    assert!(
        body.contains("name: compute-version"),
        "build.yml must contain a `compute-version` step that strips \
         the `mnemonic-gui-` prefix from github.ref_name into a \
         `VERSION` env var (Phase A.1 / SPEC §9)"
    );
    assert!(
        body.contains("shell: bash"),
        "build.yml `compute-version` step must declare `shell: bash` \
         so the prefix-strip parameter expansion works on the Windows \
         runner (PowerShell default would break it)"
    );

    // Forbidden: any GitHub Actions template expression accessing the
    // raw `env.REF_NAME` value (which carries the doubled prefix).
    // Artifact-name templates must use `env.VERSION` instead.
    assert!(
        !body.contains("env.REF_NAME"),
        "build.yml must not reference `env.REF_NAME` — that's the \
         pre-Phase-A.1 raw ref name that produced doubled-prefix \
         artifacts. Use `env.VERSION` (computed by the \
         `compute-version` step) in artifact-name templates."
    );

    // Required (Phase E follow-up to Phase A.1): the compute-version
    // step must sanitize slashes in the stripped VERSION. PR refs
    // are `<N>/merge` (head merge-ref); interpolating that raw into
    // `mnemonic-gui-${VERSION}-<suffix>.tar.gz` makes tar/7z try to
    // create a path with a directory separator in the basename and
    // fail with "Failed to open". The fix replaces `/` with `-` so
    // any ref (PR or tag) yields a single-segment filename. The
    // exact bash form `${VERSION//\//-}` is searched for here so a
    // future maintainer accidentally dropping the sanitize fails the
    // snapshot rather than the macOS package step.
    assert!(
        body.contains("${VERSION//\\//-}"),
        "build.yml `compute-version` step must sanitize slashes in \
         VERSION via the bash `${{VERSION//\\//-}}` form. PR refs \
         like `1/merge` would otherwise break tar/7z when \
         interpolated into the artifact filename."
    );

    // Required: artifact-name templates use `env.VERSION`. Four sites
    // mirror the four template locations in build.yml: `package-unix`
    // ARTIFACT, `package-windows` ARTIFACT, `upload-artifact` name,
    // `upload-artifact` path.
    let env_version_count = body.matches("env.VERSION").count();
    assert_eq!(
        env_version_count, 4,
        "build.yml must reference `env.VERSION` exactly 4 times (one \
         each for package-unix ARTIFACT, package-windows ARTIFACT, \
         upload-artifact name, upload-artifact path) — found {}",
        env_version_count
    );
}

// ── Phase C.1 (v0.2): `--gui-schema` JSON consumer ─────────────────────

#[test]
fn schema_check_json_parse_returns_flag_names() {
    use mnemonic_gui::schema_check::parse_gui_schema_json;
    // SPEC §7 sample: minimal JSON with one subcommand + one flag.
    let json = r#"{
        "version": 1,
        "cli": "md",
        "subcommands": [
            {
                "name": "encode",
                "flags": [
                    { "name": "--output", "required": false, "kind": "path", "choices": null }
                ],
                "positionals": []
            }
        ]
    }"#;
    let names = parse_gui_schema_json(json, "encode")
        .expect("encode subcommand present + version 1");
    assert!(names.contains("--output"));
}

#[test]
fn schema_check_json_accepts_version_one_and_two() {
    use mnemonic_gui::schema_check::parse_gui_schema_json;
    // SPEC §6.10.6 (v0.16.0 v2 bump): the flag-name extractor accepts both
    // v1 and v2 docs (v2 is additive: the new `conditional_rules` field is
    // ignored by this function). Only version < 1 is rejected.
    let v1 = r#"{"version":1,"cli":"md","subcommands":[
        {"name":"encode","flags":[{"name":"--out","required":false,"kind":"path","choices":null}],"positionals":[]}
    ]}"#;
    let v2 = r#"{"version":2,"cli":"md","subcommands":[
        {"name":"encode","flags":[{"name":"--out","required":false,"kind":"path","choices":null}],"positionals":[],"conditional_rules":[]}
    ]}"#;
    assert!(parse_gui_schema_json(v1, "encode").is_some(), "v1 accepted");
    assert!(parse_gui_schema_json(v2, "encode").is_some(), "v2 accepted (additive)");
    // Version 0 is illegal — fall back path.
    let v0 = r#"{"version":0,"cli":"md","subcommands":[]}"#;
    assert!(parse_gui_schema_json(v0, "encode").is_none(), "version < 1 rejected");
}

#[test]
fn schema_check_conditional_rules_requires_v2() {
    use mnemonic_gui::schema_check::parse_gui_schema_conditional_rules;
    // SPEC §6.10.6: parse_gui_schema_conditional_rules requires version >= 2
    // (the `conditional_rules` field is only emitted by v2+ producers).
    let v1 = r#"{"version":1,"cli":"mnemonic","subcommands":[
        {"name":"bundle","flags":[],"positionals":[]}
    ]}"#;
    assert!(
        parse_gui_schema_conditional_rules(v1, "bundle").is_none(),
        "v1 docs return None — predates SPEC §6.10"
    );
    let v2 = r#"{"version":2,"cli":"mnemonic","subcommands":[
        {"name":"bundle","flags":[],"positionals":[],"conditional_rules":[
            {"rationale":"r","spec_ref":"s","when":{"kind":"flag_present","flag":"--x"},
             "effect":{"flag":"--y","visibility":"disabled"}}
        ]}
    ]}"#;
    let rules = parse_gui_schema_conditional_rules(v2, "bundle")
        .expect("v2 docs yield Some");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].effect.flag, "--y");
}

#[test]
fn schema_check_json_returns_none_for_missing_subcommand() {
    use mnemonic_gui::schema_check::parse_gui_schema_json;
    let json = r#"{"version":1,"cli":"md","subcommands":[{"name":"decode","flags":[]}]}"#;
    assert!(parse_gui_schema_json(json, "encode").is_none());
}

#[test]
fn schema_check_json_invokes_gui_schema_on_capable_cli() {
    // Phase C.3 v0.2: all four CLIs are now `gui-schema-capable = true`
    // in pinned-upstream.toml, in lockstep with the bumped tags
    // (mnemonic-toolkit-v0.13.0 / md-v0.5.0 / ms-v0.2.1 / mk-v0.3.1).
    // `json_flag_names` should exec the installed binary's
    // `gui-schema` subcommand and return `Some(...)` for an existing
    // subcommand.
    //
    // The binary lookup matches schema_check.rs::json_flag_names: it
    // honours the per-CLI *_BIN env var (set by CI), falling back to
    // bare-name PATH lookup. Skip the cell if a binary cannot be
    // located so dev-laptop runs without the sibling CLIs installed
    // don't spuriously fail.
    for (cli, sub) in [
        ("mnemonic", "bundle"),
        ("md", "encode"),
        ("ms", "encode"),
        ("mk", "encode"),
    ] {
        let env_var = format!("{}_BIN", cli.to_uppercase());
        let candidate = std::env::var(&env_var).ok().unwrap_or_else(|| cli.to_string());
        let probe = std::process::Command::new(&candidate)
            .arg("gui-schema")
            .output();
        match probe {
            Ok(o) if o.status.success() => {}
            Ok(o) => {
                eprintln!(
                    "skipping {cli}: gui-schema exited {:?} stderr={}",
                    o.status,
                    String::from_utf8_lossy(&o.stderr)
                );
                continue;
            }
            Err(e) => {
                eprintln!("skipping {cli}: cannot exec {candidate:?}: {e}");
                continue;
            }
        }
        let result = mnemonic_gui::schema_check::json_flag_names(cli, sub);
        assert!(
            result.is_some(),
            "gui-schema-capable=true expects Some(_) for {cli} {sub}; got None"
        );
        assert!(
            !result.as_ref().unwrap().is_empty(),
            "{cli} {sub} should expose at least one flag via gui-schema"
        );
    }
}

#[test]
fn pinned_upstream_gui_schema_capable_all_true_at_c3() {
    // Phase C.3 v0.2 invariant: all four CLI sections carry
    // `gui-schema-capable = true` once the sibling-repo PRs land and
    // the tag pins move forward (which happens together at C.3). If
    // any CLI is still `false` here, either (a) the sibling-repo PR
    // didn't ship the subcommand, or (b) the tag bump was applied
    // without flipping the capability flag — both leave the schema
    // gate stuck on the regex-on-`--help` fallback path.
    let body = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pinned-upstream.toml"),
    )
    .expect("read pinned-upstream.toml");
    let occurrences = body.matches("gui-schema-capable = true").count();
    assert_eq!(
        occurrences, 4,
        "expected 4 'gui-schema-capable = true' occurrences in pinned-upstream.toml; got {}",
        occurrences
    );
    assert!(
        !body.contains("gui-schema-capable = false"),
        "no CLI should be gui-schema-capable = false at Phase C.3"
    );
}

// ── Phase B.2 (v0.2): platform module compile gate ─────────────────────

#[test]
fn platform_module_compiles_linux() {
    // SPEC §4 / Phase B.2 plan line 801: assert the platform module
    // signature is importable and exposes the expected function. The
    // test cannot drive a live WindowHandle in a unit-test context;
    // it asserts the public function type-checks. Linux CI implicitly
    // exercises the cfg-not-any branch; macOS/Windows CI implicitly
    // exercise their cfg-gated branches.
    use raw_window_handle::WindowHandle;
    let f: fn(WindowHandle<'_>) = mnemonic_gui::platform::apply_window_capture_protection;
    // Use `f` so the binding doesn't get dead-code-eliminated. (The
    // function is never invoked here — no live WindowHandle exists.)
    let _ = f;

    // Linux build must not emit any `unsafe` block from this module
    // (cfg-not-any branch is pure tracing::debug). Spot-check via
    // source-substring: there is no `unsafe` token inside the
    // cfg-not-any branch.
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/platform.rs"),
    )
    .expect("read src/platform.rs");
    let linux_branch_start = src
        .find("#[cfg(not(any(target_os = \"macos\", windows)))]")
        .expect("Linux cfg-not-any branch present");
    let linux_branch_end = src[linux_branch_start..]
        .find("\n}")
        .map(|n| linux_branch_start + n)
        .unwrap_or(src.len());
    let linux_branch = &src[linux_branch_start..linux_branch_end];
    assert!(
        !linux_branch.contains("unsafe"),
        "Linux branch of src/platform.rs must contain no `unsafe`"
    );
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
