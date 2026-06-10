//! v0.33.0 (audit I4 GUI half) — pins for the secret flips that ride the
//! toolkit pin bump v0.52.0 → v0.53.1.
//!
//! Three cells (SPEC §6, `design/SPEC_gui_v0_33_0_secret_flips_pin_bump.md`):
//!
//! - T1 — `ms repair --ms1` is `secret: true` by DELIBERATE GUI-side
//!   override (no automated gate covers ms.rs secret bits; the drift gate
//!   walks `schema::mnemonic::SCHEMA` only). Guards against a future
//!   "reconcile ms.rs to upstream" sweep silently reverting it — ms-cli has
//!   no gui-schema surface to reconcile against, and the value is
//!   master-secret material (BCH-corrupted BIP-39 entropy). See
//!   `FOLLOWUPS.md::ms-repair-ms1-not-secret-classified`.
//! - T2 — a checked secret Boolean stdin toggle emits NOTHING from
//!   `assemble_argv` (the secret branch `continue`s before the generic
//!   Boolean emission; the GUI runner has no stdin feed channel). First
//!   cell anywhere to pin the no-emit mechanism shared by all 24 census
//!   sites (`FOLLOWUPS.md::boolean-stdin-secret-toggles-never-emit`).
//! - T3 — `--phrase` on the three xpub-search modes is secret per
//!   `secrets::flag_is_secret` → routes to the masked `SecretLineEdit`
//!   widget branch (pure-predicate form of the widget-dispatch check).

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::schema::{self, FlagValue, FormState};
use mnemonic_gui::secrets::flag_is_secret;

fn mnemonic_sub(name: &str) -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand `{name}` not in mnemonic schema"))
}

const XPUB_SEARCH_MODES: &[&str] = &[
    "xpub-search-path-of-xpub",
    "xpub-search-account-of-descriptor",
    "xpub-search-passphrase-of-xpub",
];

/// T1 — the ms.rs override pin.
#[test]
fn t1_ms_repair_ms1_is_secret_by_deliberate_override() {
    let repair = schema::ms::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "repair")
        .expect("ms repair in schema");
    let ms1 = repair
        .flags
        .iter()
        .find(|f| f.name == "--ms1")
        .expect("--ms1 on ms repair");
    assert!(
        ms1.secret,
        "ms repair --ms1 must stay secret: true — this is a DELIBERATE \
         GUI-side override (recorded v0.33.0; see the module doc + \
         FOLLOWUPS.md::ms-repair-ms1-not-secret-classified). Do not \
         'reconcile' it to a non-existent ms-cli gui-schema."
    );
}

/// T2 — checked secret Boolean stdin toggles emit nothing.
#[test]
fn t2_checked_secret_stdin_toggles_emit_nothing() {
    for mode in XPUB_SEARCH_MODES {
        for toggle in ["--phrase-stdin", "--ms1-stdin"] {
            let mut state = FormState::default();
            state
                .values
                .push((toggle.to_string(), FlagValue::Boolean(true)));
            let argv = assemble_argv(&schema::mnemonic::SCHEMA, mnemonic_sub(mode), &state);
            assert!(
                !argv.contains(&toggle.to_string()),
                "({mode}, {toggle}): checked secret stdin toggle must NOT \
                 emit (the GUI runner has no stdin feed; the assembler's \
                 secret branch suppresses secret Booleans — see \
                 boolean-stdin-secret-toggles-never-emit)"
            );
        }
    }
}

/// T3 — `--phrase` is secret on all three modes (widget-dispatch predicate).
#[test]
fn t3_phrase_is_secret_on_all_three_xpub_search_modes() {
    for mode in XPUB_SEARCH_MODES {
        let phrase = mnemonic_sub(mode)
            .flags
            .iter()
            .find(|f| f.name == "--phrase")
            .unwrap_or_else(|| panic!("--phrase missing on {mode}"));
        assert!(
            flag_is_secret(phrase),
            "({mode}, --phrase): a raw master BIP-39 phrase must classify \
             secret (masked widget + run-confirm + redaction union)"
        );
        assert!(phrase.secret, "({mode}, --phrase): FlagSchema.secret bit");
    }
}
