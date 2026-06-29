//! Phase P4 — I4 **curated real pinned-CLI functional cells** (plan
//! `design/IMPLEMENTATION_PLAN_gui_ui_test_harness.md` P4; spec
//! `design/SPEC_gui_automated_ui_test_harness.md` §5 I4).
//!
//! ## The invariant (the EXPENSIVE integration oracle — small + curated)
//! A handful of happy-path end-to-end cells, a few per CLI: build a VALID
//! minimal form, **drive `--json` through the real P1 harness widget**,
//! `assemble_argv` via the GUI's OWN assembler (never a hand-rolled argv —
//! the whole point is to hand the real CLI exactly what the GUI would emit),
//! run against the **pinned** binary, and assert **exit 0 + `--json` stdout
//! parses + a key field or two**. Catches gross functional breakage of the
//! GUI's argv against the real CLI (spec §9).
//!
//! ## Cells (one deterministic decode/inspect-style read per CLI)
//! | CLI       | subcommand       | fixed input (public)                    | asserted fields                       |
//! |-----------|------------------|-----------------------------------------|---------------------------------------|
//! | mnemonic  | `decode-address` | `bc1qw508…f3t4` (BIP-173 P2WPKH)        | `valid`, `script_type`, `networks`    |
//! | md        | `decode`         | `md1yqpqqxqq8xtwhw4xwn4qh` (md `--help`)| `schema`, `descriptor.tree.tag`       |
//! | ms        | `decode`         | `ms10entrs…34v7f` (all-zero `ms vectors`)| `entropy_hex`, `word_count`, `language` |
//! | mk        | `decode`         | the 2-string V1 card (`mk vectors`)     | `xpub`, `origin_fingerprint`, `chunks`|
//!
//! Every input is a **fixed public test vector** (no real secrets) and every
//! asserted field is a **pure function of the input string** — a decode, with
//! NO RNG and NO timestamp anywhere in the asserted path, so the cells are
//! deterministic. (The all-zero `ms1` vector recovers the canonical published
//! BIP-39 "abandon … about" phrase — a well-known vector, not a real secret —
//! and it routes through the SECRET-positional path because the schema marks
//! `ms decode`'s `ms1` positional `secret: true`; we still treat its stdout as
//! sensitive for failure-message hygiene.)
//!
//! ## Env-gating (CRITICAL — plan P4 m5): EARLY-RETURN-SKIP, never `#[ignore]`
//! Each cell resolves its pinned binary from `MNEMONIC_BIN` / `MD_BIN` /
//! `MS_BIN` / `MK_BIN` (the schema-mirror pin convention,
//! `schema-mirror.yml`). When the env var is UNSET the cell prints a skip note
//! and `return`s — it does NOT use `#[ignore]`, which would make I4 never run
//! under `cargo test --workspace` even when the schema-mirror CI job has
//! installed all four pins (zeroing I4's CI teeth). So: CI-without-binaries
//! passes (skips); CI-with-pins actually exercises the GUI's argv vs the real
//! CLI. (Mirrors `tests/runner_integration.rs`'s env-var binary resolution +
//! its `cell_3` early-return-skip shape, combined per m5.)

mod ui_harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::runner;
use mnemonic_gui::schema::FormState;

use ui_harness::{drive, flag_of, render_flag_harness, schema_for, sub_of, IdentityKind, Injected};

// ─── fixed PUBLIC test vectors (no real secrets) ────────────────────────────

/// BIP-173 canonical P2WPKH mainnet address (pure-public — just an address).
const ADDR_BC1Q_P2WPKH: &str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";

/// `md decode --help` EXAMPLES vector → `wpkh(@0/<0;1>/*)` (keyless template).
const MD1_WPKH_TEMPLATE: &str = "md1yqpqqxqq8xtwhw4xwn4qh";

/// `ms vectors` first entry: 12-word all-zero entropy (BIP-39 `[0; 16]`). The
/// recovered phrase is the published "abandon … about" canonical vector — a
/// well-known value, NOT a real secret.
const MS1_ALL_ZERO: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const MS1_ALL_ZERO_ENTROPY_HEX: &str = "00000000000000000000000000000000";

/// `mk vectors` V1 (`V1_bip48_mainnet_1_stub_with_fp`): a 2-string mainnet
/// BIP-48 multisig recovery card. Watch-only (public xpub only).
const MK1_V1_STRING_A: &str = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf";
const MK1_V1_STRING_B: &str =
    "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";
const MK1_V1_XPUB: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";
const MK1_V1_FINGERPRINT: &str = "aabbccdd";

// ─── env-gating ─────────────────────────────────────────────────────────────

/// Resolve a pinned-binary path/name from `env_var`, or `None` when it is
/// unset/blank — the EARLY-RETURN-SKIP signal (plan P4 m5). A bare name (the
/// CI convention, `schema-mirror.yml` sets `MNEMONIC_BIN: mnemonic` …) is
/// returned verbatim and resolved by `$PATH` at spawn; an absolute path (the
/// local convention, `MNEMONIC_BIN=$HOME/.cargo/bin/mnemonic`) is spawned
/// directly. A SET-but-bogus value deliberately does NOT skip — `runner::run`
/// then errors loudly (a misconfigured pin must fail, not silently pass).
fn pinned_bin(env_var: &str) -> Option<String> {
    match std::env::var(env_var) {
        Ok(p) if !p.trim().is_empty() => Some(p),
        _ => None,
    }
}

// ─── the shared drive→assemble→run→parse path ───────────────────────────────

/// Drive `--json` ON through the real P1 harness widget on `(tab, sub_name)`,
/// `assemble_argv` from the driven + positional-seeded `base` state, spawn the
/// pinned binary, assert exit 0, and parse stdout as JSON. Returns `None` iff
/// `env_var` is unset (the cell then skips). FAILURE MESSAGES ARE
/// COORDINATE-ONLY for stdout — `ms decode`'s stdout carries the recovered
/// phrase, so a parse-failure never echoes the payload (uniform hygiene; stderr
/// carries only non-secret warnings/notes and IS surfaced).
fn drive_json_and_decode(
    tab: CliTab,
    sub_name: &str,
    env_var: &str,
    base: FormState,
) -> Option<serde_json::Value> {
    let Some(bin) = pinned_bin(env_var) else {
        eprintln!(
            "i4 [{}/{sub_name}]: {env_var} unset — skipping real-CLI cell \
             (pinned binary absent; NOT a failure)",
            tab.bin_name()
        );
        return None;
    };

    let schema = schema_for(tab);
    let sub = sub_of(tab, sub_name);
    let json_flag = flag_of(sub, "--json");

    // Reuse the P1 harness: render the `--json` Boolean and drive it ON via the
    // REAL widget dispatch (NOT a hand-set `state.values` entry), so the argv we
    // hand the CLI is genuinely the GUI's assembled output. `decode-address` /
    // `md decode` / `ms decode` / `mk decode` all declare `conditional: None`,
    // so no visibility gate can suppress `--json`.
    let mut h = render_flag_harness(tab, sub, json_flag, base);
    drive(&mut h, IdentityKind::Boolean, &Injected::Boolean(true));

    // GUI's OWN assembler — no hand-rolled argv (the I4 point).
    let mut argv = assemble_argv(schema, sub, h.state());

    // Non-vacuity: the driven checkbox actually reached argv. A no-op drive
    // would omit `--json`, the CLI would emit TEXT, and the JSON parse below
    // would mis-attribute the failure to the CLI rather than the harness.
    assert!(
        argv.iter().any(|t| t == "--json"),
        "i4 [{}/{sub_name}]: driving the --json checkbox did not reach the \
         assembled argv (harness drive no-op)",
        tab.bin_name()
    );

    // Run the configured/pinned binary rather than a $PATH guess.
    argv[0] = bin;
    let result = runner::run(argv).unwrap_or_else(|e| {
        panic!(
            "i4 [{}/{sub_name}]: runner spawn failed: {e}",
            tab.bin_name()
        )
    });

    // stderr is non-secret (decode warnings / notes only) — safe to surface.
    let stderr = String::from_utf8_lossy(&result.stderr).into_owned();
    assert_eq!(
        result.exit_code,
        Some(0),
        "i4 [{}/{sub_name}]: expected exit 0 from the pinned binary; stderr:\n{stderr}",
        tab.bin_name()
    );

    // RunResult impls Drop (whole-holder zeroize) so stdout cannot be moved out
    // (E0509) — clone the bytes (the holder still drops scrubbed). Parse from
    // the bytes; on failure report COORDINATES ONLY (payload withheld — stdout
    // may carry sensitive material for `ms decode`).
    let stdout = result.stdout.clone();
    let value: serde_json::Value = serde_json::from_slice(&stdout).unwrap_or_else(|e| {
        panic!(
            "i4 [{}/{sub_name}]: --json stdout did not parse as JSON: {e} \
             (payload withheld — see coordinates)",
            tab.bin_name()
        )
    });
    Some(value)
}

// ════════════════════════════════════════════════════════════════════════
// The curated cells (one per CLI)
// ════════════════════════════════════════════════════════════════════════

/// `mnemonic decode-address <ADDRESS> --json` — a pure address decode (no
/// secret, no RNG, no timestamp). Asserts the validity + script-type +
/// network classification of the BIP-173 P2WPKH vector.
#[test]
fn i4_mnemonic_decode_address_p2wpkh() {
    let base = FormState::default().with_positionals([ADDR_BC1Q_P2WPKH]);
    let Some(v) = drive_json_and_decode(CliTab::Mnemonic, "decode-address", "MNEMONIC_BIN", base)
    else {
        return; // env-gated skip (note already printed)
    };
    assert_eq!(v["valid"].as_bool(), Some(true), "address must be valid");
    assert_eq!(
        v["script_type"].as_str(),
        Some("p2wpkh"),
        "BIP-173 vector is a P2WPKH"
    );
    let networks = v["networks"]
        .as_array()
        .expect("--json carries a networks array");
    assert!(
        networks.iter().any(|n| n.as_str() == Some("mainnet")),
        "the bc1… vector decodes to mainnet"
    );
}

/// `md decode <MD1> --json` — a pure md1 → BIP-388 template decode. Asserts the
/// CLI's JSON schema tag + the decoded descriptor's top-level node tag.
#[test]
fn i4_md_decode_wpkh_template() {
    let base = FormState::default().with_positionals([MD1_WPKH_TEMPLATE]);
    let Some(v) = drive_json_and_decode(CliTab::Md, "decode", "MD_BIN", base) else {
        return;
    };
    assert_eq!(
        v["schema"].as_str(),
        Some("md-cli/1"),
        "md decode --json carries the md-cli/1 schema tag"
    );
    assert_eq!(
        v["descriptor"]["tree"]["tag"].as_str(),
        Some("Wpkh"),
        "the md1 vector decodes to a wpkh(...) template"
    );
}

/// `ms decode <MS1> --json` — a pure ms1 → entropy/phrase decode. The `ms1`
/// positional is schema-`secret: true`, so it routes through the secret-widget
/// argv path (`secret_widgets["positional:ms1"]`). Asserts the recovered
/// entropy/word-count/language (the all-zero vector — public).
#[test]
fn i4_ms_decode_all_zero_vector() {
    // SECRET positional → seed the SecretLineEdit store the assembler reads
    // (`positional:<name>`), NOT `state.positionals`. The all-zero vector is a
    // published test vector, not a real secret.
    let mut base = FormState::default();
    base.secret_widgets.insert(
        "positional:ms1".to_string(),
        vec![SecretLineEdit::from_text(MS1_ALL_ZERO)],
    );
    let Some(v) = drive_json_and_decode(CliTab::Ms, "decode", "MS_BIN", base) else {
        return;
    };
    assert_eq!(
        v["entropy_hex"].as_str(),
        Some(MS1_ALL_ZERO_ENTROPY_HEX),
        "all-zero ms1 vector recovers 16 zero bytes"
    );
    assert_eq!(
        v["word_count"].as_u64(),
        Some(12),
        "16 bytes of entropy ⇒ a 12-word phrase"
    );
    assert_eq!(
        v["language"].as_str(),
        Some("english"),
        "--language defaults to english"
    );
}

/// `mk decode <MK1>... --json` — a pure mk1 → xpub/origin decode over a
/// repeating positional (the 2-string V1 card). Asserts the recovered xpub +
/// origin fingerprint + chunk count (all watch-only / public).
#[test]
fn i4_mk_decode_v1_two_string_card() {
    let base = FormState::default().with_positionals([MK1_V1_STRING_A, MK1_V1_STRING_B]);
    let Some(v) = drive_json_and_decode(CliTab::Mk, "decode", "MK_BIN", base) else {
        return;
    };
    assert_eq!(
        v["xpub"].as_str(),
        Some(MK1_V1_XPUB),
        "the V1 card decodes to the pinned account xpub"
    );
    assert_eq!(
        v["origin_fingerprint"].as_str(),
        Some(MK1_V1_FINGERPRINT),
        "origin fingerprint round-trips"
    );
    assert_eq!(
        v["chunks"].as_u64(),
        Some(2),
        "the V1 card is a 2-string (2-chunk) card"
    );
}
