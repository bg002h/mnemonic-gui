//! Phase P3 — I3 **classified-secret never-persist / never-leak REGRESSION
//! NET** (plan `design/IMPLEMENTATION_PLAN_gui_ui_test_harness.md` P3; spec
//! `design/SPEC_gui_automated_ui_test_harness.md` §5 I3 — the CORRECTED scope).
//!
//! ## Honest scope (CORRECTED — IMP-1; stated, not a co-headline class-catcher)
//! This file is a **regression net for *CLASSIFIED* secrets only**: it iterates
//! the `flag_is_secret(flag)` set (schema `secret: true` ∪ the hand-maintained
//! [`SECRET_FLAG_NAMES`]) and proves each classified secret's fixture cannot
//! reach the persistence / preview surfaces. It therefore **CANNOT** catch an
//! *UNclassified* secret rendering as a normal Text widget (the actual v0.31.1
//! leak class) — that class is owned by `tests/schema_mirror_secret_drift.rs`
//! + `tests/secret_taxonomy_pin.rs`, which this harness **does NOT replace**.
//!
//! ## The invariant
//! For every classified secret flag × subcommand: widget-inject a **FAKE**
//! fixture (a distinctive `FAKE_SECRET_FIXTURE_*` sentinel — NEVER a real key),
//! then assert the fixture is ABSENT from:
//!   1. **Persisted state** — `redact_for_persistence` → serialize (NOT merely
//!      `#[serde(skip)]`; the build-descriptor tree `key`/`keys` persist then
//!      redact — asserted POST-redaction in [`i3_tree_key_persist_then_redact`]).
//!   2. **Masked-argv confirm-modal / preview** — `assemble_argv_with_secret_mask`
//!      → `render_copy_command_masked`. The fixture must be MASKED (`••••`) in
//!      the preview, and every raw-argv token bearing it must carry `mask == true`.
//!   3. **`--spec -` stdin** path (build-descriptor tree) — the one place a
//!      secret *node* key flows via stdin rather than argv ([`i3_tree_secret_key`]).
//!
//! ## Harness hygiene (MANDATORY — self-custody first-class bar)
//! - **FAKE fixtures ONLY.** Never inject or log a real key.
//! - **On assertion FAILURE, emit ONLY the flag/subcommand COORDINATES** — the
//!   helpers below (`leak_msg`) never embed the FormState, the AccessKit tree,
//!   the serialized blob, the argv, or the fixture value (the undo-ring / held
//!   buffers can hold plaintext; a message that prints the leaked value would
//!   itself be a leak). See [`coord`].
//! - Respect `Zeroizing` — secret reads go through `SecretLineEdit::as_string()`
//!   (which returns `Zeroizing<String>`); we never copy a secret into a plain
//!   long-lived `String`.
//!
//! ## Non-vacuity (proves the net BITES, not vacuously green)
//! - Per flag: the DRIVE-landed assertion proves the fixture actually reached
//!   the widget store (an injection that no-op'd into the void would RED here).
//! - Globally: [`i3_negative_*`] are deliberate-leak cells — they construct a
//!   state where a FAKE fixture WOULD persist / WOULD render unmasked and assert
//!   the harness's OWN checks RED-flag it (without ever persisting a real secret).

mod ui_harness;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::invocation::{
    assemble_argv, assemble_argv_with_secret_mask, render_copy_command_masked, ShellFlavor,
};
use mnemonic_gui::form::tree_form::spec_stdin_bytes;
use mnemonic_gui::form::tree_model::{TreeNode, TreeState};
use mnemonic_gui::form::widget::render_with_dispatch;
use mnemonic_gui::persistence::redact_for_persistence;
use mnemonic_gui::runner::PendingConfirm;
use mnemonic_gui::schema::{
    FlagKind, FlagSchema, FlagValue, FormState, Schema, SubcommandSchema,
};
use mnemonic_gui::secrets::{flag_is_secret, schema_secret_flag_names, SECRET_FLAG_NAMES};

use ui_harness::schema_for;

use zeroize::Zeroize;

// ════════════════════════════════════════════════════════════════════════
// Enumeration + classification
// ════════════════════════════════════════════════════════════════════════

/// Every `(tab, sub, flag)` where `flag_is_secret(flag)` — the CLASSIFIED
/// secret set, FIELD-extracted from the schema STRUCTS (never a text grep;
/// comment lines in mnemonic.rs carry the literal `secret: true`).
fn classified_secret_flags(
) -> impl Iterator<Item = (CliTab, &'static SubcommandSchema, &'static FlagSchema)> {
    CliTab::ALL.iter().copied().flat_map(|tab| {
        schema_for(tab).subcommands.iter().flat_map(move |sub| {
            sub.flags
                .iter()
                .filter(|f| flag_is_secret(f))
                .map(move |f| (tab, sub, f))
        })
    })
}

/// A secret flag is **value-bearing** (a FAKE fixture can be injected) iff its
/// kind is `Text` or `NodeValueComposite` — those carry a user-typed secret
/// value (Text → `secret_widgets`; NodeValueComposite → `state.values`,
/// masked-by-`flag_is_secret`). `Boolean` secret flags are the `*-stdin`
/// toggles: a disabled presence-checkbox the assembler SUPPRESSES, carrying NO
/// value buffer — they are NARROWED (see [`i3_narrowed_boolean_stdin_toggles`]).
fn is_value_bearing(flag: &FlagSchema) -> bool {
    matches!(flag.kind, FlagKind::Text | FlagKind::NodeValueComposite(_))
}

/// Stable `tab/sub/flag` coordinate string for COORDINATE-ONLY failure
/// messages (never embeds state / argv / blob / the fixture value).
fn coord(tab: CliTab, sub: &SubcommandSchema, flag: &FlagSchema) -> String {
    format!("{}/{}/{}", tab.bin_name(), sub.name, flag.name)
}

/// A coordinate-only leak message (no payload — hygiene rule).
fn leak_msg(tab: CliTab, sub: &SubcommandSchema, flag: &FlagSchema, surface: &str) -> String {
    format!(
        "I3 LEAK [{}]: a classified-secret fixture reached the {surface} surface \
         (payload withheld — see coordinates only)",
        coord(tab, sub, flag)
    )
}

// ════════════════════════════════════════════════════════════════════════
// Fixtures (FAKE sentinels — never real keys)
// ════════════════════════════════════════════════════════════════════════

/// A distinctive, per-flag-unique FAKE secret value. Alphanumeric + `_` so it
/// survives any shell-quoting intact (`.contains` locates it whether or not
/// `posix_quote` wraps the token).
fn fixture(idx: usize) -> String {
    format!("FAKE_SECRET_FIXTURE_{idx:03}")
}

// ════════════════════════════════════════════════════════════════════════
// Drive: widget-inject a fixture into a single classified secret flag
// ════════════════════════════════════════════════════════════════════════

/// Render exactly ONE flag through the REAL `render_with_dispatch` (the same
/// dispatch the live form runs — `tests/repeating_secret_rows.rs` precedent),
/// so the secret routes through its production path (`SecretLineEdit` →
/// `secret_widgets` for Text; `render_repeating` → `state.values` for the
/// secret NodeValueComposite `--share`). Rendering one flag in isolation keeps
/// the secret input uniquely role-targetable and bypasses NO routing.
fn single_flag_harness(
    tab: CliTab,
    sub: &'static SubcommandSchema,
    flag: &'static FlagSchema,
) -> Harness<'static, FormState> {
    Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            render_with_dispatch(ui, tab, sub.name, flag, state, &[]);
        },
        FormState::default(),
    )
}

/// Widget-inject `fixture` into the rendered single secret flag.
///
/// Secret Text flags render a password-masked TextEdit (egui `Role::PasswordInput`,
/// NOT `TextInput`); the secret NodeValueComposite `--share` (node `phrase`,
/// argv-secret) ALSO renders its value field passworded. Optional repeating
/// secret Text (`--ms1`) seeds ZERO rows — we click the header `+ add` first.
/// Followed by run-to-stable (never a fixed frame count — SPEC §10).
fn drive_secret(h: &mut Harness<'static, FormState>, fixture: &str) {
    h.run();
    // Optional-repeating secret Text seeds no rows: add one before typing.
    let has_input = h.query_by_role(Role::PasswordInput).is_some()
        || h.query_by_role(Role::TextInput).is_some();
    if !has_input && h.query_by_label("+ add").is_some() {
        h.get_by_label("+ add").click();
        h.run();
    }
    // Exactly one secret input exists for a single-flag render (the value
    // field). Prefer the password role; fall back to plain TextInput for any
    // non-secret-node composite value field (defensive — `--share` is `phrase`).
    if h.query_by_role(Role::PasswordInput).is_some() {
        h.get_by_role(Role::PasswordInput).type_text(fixture);
    } else {
        h.get_by_role(Role::TextInput).type_text(fixture);
    }
    h.run();
    h.run(); // settle: buffer write-back lands at frame end
}

/// True iff `fixture` actually landed in the widget store after the drive —
/// the per-flag NON-VACUITY proof (an injection that no-op'd RED-flags here).
/// Reads the Text path (`secret_widgets`, via the `Zeroizing` `as_string()`)
/// and the composite path (`state.values`). Returns a bool ONLY (never echoes
/// the value).
fn fixture_landed(state: &FormState, flag: &FlagSchema, fixture: &str) -> bool {
    if let Some(rows) = state.secret_widgets.get(flag.name) {
        if rows.iter().any(|w| w.as_string().as_str() == fixture) {
            return true;
        }
    }
    state.values.iter().any(|(k, v)| {
        k == flag.name
            && match v {
                FlagValue::NodeValueComposite { value, .. } => value == fixture,
                FlagValue::Text(s) => s == fixture,
                _ => false,
            }
    })
}

// ════════════════════════════════════════════════════════════════════════
// Surface checks (return a bool ONLY — callers print coordinates, not payload)
// ════════════════════════════════════════════════════════════════════════

/// Surface 1 — does the redacted+serialized persisted state contain `fixture`?
/// Drives the REAL persist walk (`redact_for_persistence`, NOT bare
/// `serde(skip)`), so a Text fixture mis-routed into `state.values` OR a tree
/// `key`/`keys` not blanked would be detected.
fn persist_leaks(state: &FormState, fixture: &str) -> bool {
    let redacted = redact_for_persistence(state);
    let json = serde_json::to_string(&redacted).expect("FormState serializes");
    json.contains(fixture)
}

/// Surface 2 — does the masked confirm-modal / preview render contain `fixture`?
/// (`render_copy_command_masked` over BOTH shell flavors — the secret-bearing
/// run-confirm surface.) A masked token renders as `SECRET_MASK` (`••••`), so a
/// correctly classified secret cannot appear here.
fn masked_preview_leaks(schema: &Schema, sub: &SubcommandSchema, state: &FormState, fixture: &str) -> bool {
    let (argv, mask) = assemble_argv_with_secret_mask(schema, sub, state);
    let posix = render_copy_command_masked(&argv, &mask, ShellFlavor::Posix);
    let win = render_copy_command_masked(&argv, &mask, ShellFlavor::WindowsCmd);
    posix.contains(fixture) || win.contains(fixture)
}

/// Mask-correctness on the RAW argv: `(reached, all_masked)` — `reached` iff a
/// raw-argv token bears `fixture`; `all_masked` iff every such token carries
/// `mask == true`. `reached` is the masking-path NON-VACUITY signal (counted
/// globally); `all_masked` is the per-flag classification correctness.
fn argv_mask_status(
    schema: &Schema,
    sub: &SubcommandSchema,
    state: &FormState,
    fixture: &str,
) -> (bool, bool) {
    let (argv, mask) = assemble_argv_with_secret_mask(schema, sub, state);
    let idxs: Vec<usize> = argv
        .iter()
        .enumerate()
        .filter(|(_, t)| t.contains(fixture))
        .map(|(i, _)| i)
        .collect();
    let reached = !idxs.is_empty();
    let all_masked = idxs.iter().all(|&i| mask[i]);
    (reached, all_masked)
}

// ════════════════════════════════════════════════════════════════════════
// CENSUS — pin the classified-secret partition (drift gate)
// ════════════════════════════════════════════════════════════════════════

#[test]
fn i3_classified_secret_partition_census() {
    let mut value_bearing = 0usize;
    let mut narrowed_boolean = 0usize;
    for (tab, sub, flag) in classified_secret_flags() {
        // Every classified secret flag must be a kind this net knows how to
        // handle. A future secret Number/Path/Dropdown would trip this and
        // force the harness to gain a driver (logged, not silently uncovered).
        assert!(
            matches!(
                flag.kind,
                FlagKind::Text | FlagKind::NodeValueComposite(_) | FlagKind::Boolean
            ),
            "unhandled classified-secret kind at {} — extend the I3 net",
            coord(tab, sub, flag)
        );
        if is_value_bearing(flag) {
            value_bearing += 1;
        } else {
            narrowed_boolean += 1;
        }
    }
    let total = value_bearing + narrowed_boolean;

    // Frozen census (2026-06; field-extracted). On drift: a NEW classified
    // secret flag landed — extend the value-bearing sweep (Text/Composite) or
    // the narrowed-Boolean list, then bump these counts deliberately.
    assert_eq!(
        (value_bearing, narrowed_boolean, total),
        (40, 24, 64),
        "classified-secret partition drifted (value-bearing Text/Composite, \
         narrowed Boolean *-stdin toggles, total)"
    );

    // Secret POSITIONALS (5) are a SEPARATE surface (`pos.secret`, not a
    // `secret: true` flag) routed through the IDENTICAL `SecretLineEdit` →
    // `secret_widgets["positional:<name>"]` mechanism this net drives for Text
    // flags, but rendered by a bin-private block in `main.rs` (not reachable
    // via the integration harness). They are NARROWED here and covered by
    // `tests/persist_redaction_v0_34_0.rs` (T1b serialize / T2 emit / T4
    // census). Pin the count so a 6th secret positional is a deliberate change.
    let secret_positionals: usize = CliTab::ALL
        .iter()
        .copied()
        .flat_map(|t| schema_for(t).subcommands.iter())
        .flat_map(|s| s.positional_args.iter())
        .filter(|p| p.secret)
        .count();
    assert_eq!(
        secret_positionals, 5,
        "secret-positional census drifted — see persist_redaction_v0_34_0.rs"
    );
}

// ════════════════════════════════════════════════════════════════════════
// THE SWEEP — every value-bearing classified secret flag, 3 surfaces
// ════════════════════════════════════════════════════════════════════════

#[test]
fn i3_value_bearing_secret_flags_never_leak() {
    let mut covered = 0usize;
    let mut reached_argv_masked = 0usize;

    for (idx, (tab, sub, flag)) in classified_secret_flags()
        .filter(|(_, _, f)| is_value_bearing(f))
        .enumerate()
    {
        let fix = fixture(idx);

        // ── DRIVE: widget-inject the fixture through the real dispatch ──────
        let mut h = single_flag_harness(tab, sub, flag);
        drive_secret(&mut h, &fix);
        let state = h.state();

        // Non-vacuity: the fixture actually landed in the widget store.
        assert!(
            fixture_landed(state, flag, &fix),
            "I3 DRIVE-NOOP [{}]: the fixture never reached the widget store \
             (the cell would be vacuously green)",
            coord(tab, sub, flag)
        );

        // ── Surface 1 — persisted state (redact_for_persistence walk) ───────
        assert!(
            !persist_leaks(state, &fix),
            "{}",
            leak_msg(tab, sub, flag, "persisted-state")
        );

        // ── Surface 2 — masked-argv confirm-modal / preview ─────────────────
        let schema = schema_for(tab);
        assert!(
            !masked_preview_leaks(schema, sub, state, &fix),
            "{}",
            leak_msg(tab, sub, flag, "masked-argv-preview")
        );
        // Classification correctness on raw argv (where the secret emits): the
        // token must carry mask == true. (Some secret flags are conditionally
        // suppressed in a bare single-secret state — `reached == false` is then
        // a no-leak no-op, NOT a failure.)
        let (reached, all_masked) = argv_mask_status(schema, sub, state, &fix);
        assert!(
            all_masked,
            "I3 MASK-MISCLASSIFY [{}]: a raw-argv token bearing the fixture \
             is NOT mask=true (would render cleartext in the preview)",
            coord(tab, sub, flag)
        );
        if reached {
            reached_argv_masked += 1;
        }

        // ── Surface 3 — `--spec -` stdin (build-descriptor tree) ────────────
        // A classified secret FLAG routes to `secret_widgets` / `state.values`,
        // NEVER `state.tree`; `spec_stdin_bytes` reads ONLY `state.tree`, so a
        // flag fixture is structurally incapable of reaching the tree-spec
        // stdin. Assert that disjointness (no tree state ⇒ no spec stdin).
        assert!(
            spec_stdin_bytes(state).is_none(),
            "I3 [{}]: a non-tree form must produce no `--spec -` stdin",
            coord(tab, sub, flag)
        );

        covered += 1;
    }

    assert_eq!(covered, 40, "expected to drive all 40 value-bearing secrets");
    // The masking path was genuinely exercised (most secrets emit + mask in a
    // bare state). A regression that stopped masking would crater this.
    assert!(
        reached_argv_masked >= 35,
        "the masked-argv path was under-exercised ({reached_argv_masked}/40 reached \
         argv with a mask bit) — investigate conditional suppression drift"
    );
}

// ════════════════════════════════════════════════════════════════════════
// NARROWED — Boolean `*-stdin` toggles (no value buffer; documented)
// ════════════════════════════════════════════════════════════════════════

#[test]
fn i3_narrowed_boolean_stdin_toggles() {
    // A classified secret `Boolean` is a `*-stdin` toggle: rendered as an
    // ALWAYS-DISABLED checkbox with NO value buffer, and the assembler's
    // secret-branch `continue`s it (no emission) — so there is no secret VALUE
    // to inject (the value-injection invariant is N/A). The residual risk is a
    // STALE persisted checkbox state; this cell pins that (a) the persist
    // name-net covers every such toggle, so a stale `Boolean(true)` is dropped,
    // and (b) the assembler never emits it.
    let mut narrowed = 0usize;
    for (tab, sub, flag) in classified_secret_flags().filter(|(_, _, f)| !is_value_bearing(f)) {
        assert!(
            matches!(flag.kind, FlagKind::Boolean),
            "non-value-bearing classified secret that is not a Boolean toggle at {}",
            coord(tab, sub, flag)
        );

        // (a) the persist name-net covers the toggle (so a stale persisted
        //     checkbox is redacted on the next save).
        let covered = schema_secret_flag_names().contains(flag.name)
            || SECRET_FLAG_NAMES.contains(&flag.name);
        assert!(
            covered,
            "I3 [{}]: a secret Boolean *-stdin toggle is NOT covered by the \
             persist name-net (a stale checkbox could persist)",
            coord(tab, sub, flag)
        );

        // Force a STALE `Boolean(true)` under the toggle's name and prove both
        // surfaces fail-closed.
        let mut state = FormState::default();
        state
            .values
            .push((flag.name.to_string(), FlagValue::Boolean(true)));

        let redacted = redact_for_persistence(&state);
        assert!(
            !redacted.values.iter().any(|(k, _)| k == flag.name),
            "I3 [{}]: a stale secret-Boolean toggle survived persist redaction",
            coord(tab, sub, flag)
        );

        let (argv, _mask) = assemble_argv_with_secret_mask(schema_for(tab), sub, &state);
        assert!(
            !argv.iter().any(|t| t == flag.name),
            "I3 [{}]: a suppressed secret-Boolean toggle leaked into argv",
            coord(tab, sub, flag)
        );

        narrowed += 1;
    }
    assert_eq!(narrowed, 24, "expected exactly 24 narrowed secret-Boolean toggles");
}

// ════════════════════════════════════════════════════════════════════════
// TREE — build-descriptor `key`/`keys` persist-then-redact (POST-redaction)
// ════════════════════════════════════════════════════════════════════════

/// A FAKE private-key fixture that is NOT extended-PUBLIC-like, so the persist
/// allowlist (`is_extended_public_like`) BLANKS it. Never a real key.
const TREE_KEY_FIXTURE: &str = "FAKE_SECRET_FIXTURE_TREE_KEY";
const TREE_KEYS_FIXTURE: &str = "FAKE_SECRET_FIXTURE_TREE_KEYS";
/// A watch-only xpub (extended-public) — SURVIVES persist (the redactor
/// discriminates; proves it is not a blanket wipe).
const SURVIVING_XPUB: &str =
    "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet";

/// Build a `FormState` whose tree carries `key` (a `pk` leaf) and `keys` (a
/// `multi` quorum) fixtures plus a surviving xpub, under an `and_v` root.
fn tree_state_with_secret_keys() -> FormState {
    let pk = TreeNode {
        id: 1,
        kind: "pk".into(),
        key: TREE_KEY_FIXTURE.into(),
        ..Default::default()
    };
    let multi = TreeNode {
        id: 2,
        kind: "multi".into(),
        k: 1,
        keys: vec![TREE_KEYS_FIXTURE.into(), SURVIVING_XPUB.into()],
        ..Default::default()
    };
    let root = TreeNode {
        id: 0,
        kind: "and_v".into(),
        children: vec![pk, multi],
        ..Default::default()
    };
    let mut ts = TreeState::fresh();
    ts.enabled = true;
    ts.root = root;
    ts.recompute_next_id();
    FormState {
        tree: Some(ts),
        ..Default::default()
    }
}

#[test]
fn i3_tree_key_persist_then_redact() {
    let state = tree_state_with_secret_keys();

    // PRE-redaction: a bare serialize WOULD leak both fixtures — proving the
    // redaction walk (NOT `serde(skip)`; `tree` IS persisted) is load-bearing.
    let pre = serde_json::to_string(&state).expect("FormState serializes");
    assert!(
        pre.contains(TREE_KEY_FIXTURE) && pre.contains(TREE_KEYS_FIXTURE),
        "I3 SETUP: the tree fixtures must be present PRE-redaction (else the \
         post-redaction assertion is vacuous)"
    );

    // POST-redaction: `redact_for_persistence` → `TreeState::redacted_for_persistence`
    // → `blank_non_extended_public_keys` clears the non-xpub `key`/`keys`.
    assert!(
        !persist_leaks(&state, TREE_KEY_FIXTURE),
        "I3 LEAK [build-descriptor tree key]: a private-key fixture survived \
         persist redaction (payload withheld)"
    );
    assert!(
        !persist_leaks(&state, TREE_KEYS_FIXTURE),
        "I3 LEAK [build-descriptor tree keys]: a quorum private-key fixture \
         survived persist redaction (payload withheld)"
    );

    // The redactor DISCRIMINATES (not a blanket wipe): the watch-only xpub
    // SURVIVES — non-vacuity that it is the redactor at work.
    let redacted = redact_for_persistence(&state);
    let json = serde_json::to_string(&redacted).expect("serializes");
    assert!(
        json.contains(SURVIVING_XPUB),
        "I3 SETUP: a watch-only xpub must survive persist redaction (the \
         redactor must discriminate, not blanket-wipe)"
    );
}

// ════════════════════════════════════════════════════════════════════════
// TREE — secret-node `--spec -` stdin: off the masked preview + scrubbed
// ════════════════════════════════════════════════════════════════════════

#[test]
fn i3_tree_secret_key_off_preview_and_spec_stdin_scrubbed() {
    // A COMPLETE tree (single `pk` leaf carrying the secret key fixture) so
    // `spec_stdin_bytes` is `Some`.
    let mut ts = TreeState::fresh();
    ts.enabled = true;
    ts.root = TreeNode {
        id: 0,
        kind: "pk".into(),
        key: TREE_KEY_FIXTURE.into(),
        ..Default::default()
    };
    ts.recompute_next_id();
    let state = FormState {
        tree: Some(ts),
        ..Default::default()
    };

    let sub = schema_for(CliTab::Mnemonic)
        .subcommands
        .iter()
        .find(|s| s.name == "build-descriptor")
        .expect("build-descriptor in mnemonic schema");

    // The spec stdin is the FUNCTIONAL channel — it MUST carry the key (the
    // binary derives from it; the analogue of a secret VALUE token in argv).
    // Assert it carries it (so the surfaces below are non-vacuous), then prove
    // the key does NOT leak to the PREVIEW and the held stdin SCRUBS on drop.
    let stdin = spec_stdin_bytes(&state).expect("a complete tree yields spec stdin");
    assert!(
        String::from_utf8_lossy(&stdin).contains(TREE_KEY_FIXTURE),
        "I3 SETUP: the spec-stdin functional channel must carry the tree key \
         (else the preview/scrub assertions are vacuous)"
    );

    // ── Surface 2 (tree leg) — the masked confirm-modal / preview ───────────
    // In tree mode the host appends `--spec -` and pipes the spec via STDIN;
    // the key is NOT an argv token, so the masked preview (which renders argv
    // only) cannot contain it.
    let mut argv = assemble_argv(schema_for(CliTab::Mnemonic), sub, &state);
    let mut mask: Vec<bool> = vec![false; argv.len()];
    argv.push("--spec".into());
    mask.push(false);
    argv.push("-".into());
    mask.push(false);
    let preview = render_copy_command_masked(&argv, &mask, ShellFlavor::Posix);
    assert!(
        !preview.contains(TREE_KEY_FIXTURE),
        "I3 LEAK [build-descriptor tree --spec - preview]: the tree key \
         appeared in the masked confirm-modal preview (payload withheld)"
    );

    // ── Surface 3 — the `--spec -` stdin is held in a SCRUBBING holder ──────
    // The cleartext stdin lives only in the `PendingConfirm` run-holder, which
    // `Zeroize`s (and `Drop`-scrubs) the bytes — no cleartext residue persists
    // past the confirm. (Whole-holder scrub seam: `run_holder_zeroize.rs`.)
    let mut pending = PendingConfirm {
        argv,
        mask,
        stdin: Some(stdin),
    };
    pending.zeroize();
    let residue = pending
        .stdin
        .as_ref()
        .map(|b| String::from_utf8_lossy(b).contains(TREE_KEY_FIXTURE))
        .unwrap_or(false);
    assert!(
        !residue,
        "I3 LEAK [build-descriptor tree --spec - stdin holder]: the spec-stdin \
         key survived the PendingConfirm scrub (payload withheld)"
    );
}

// ════════════════════════════════════════════════════════════════════════
// NEGATIVE — prove the harness's own checks BITE (non-vacuous)
// ════════════════════════════════════════════════════════════════════════

#[test]
fn i3_negative_persist_check_bites() {
    let leak = "FAKE_SECRET_FIXTURE_NEG_PERSIST";

    // POSITIVE control: the fixture correctly held in `secret_widgets` (the
    // real secret-Text store) does NOT persist — `persist_leaks` is false.
    let mut clean = FormState::default();
    clean.secret_widgets.insert(
        "--passphrase".into(),
        vec![mnemonic_gui::form::secret_widget::SecretLineEdit::from_text(leak)],
    );
    assert!(
        !persist_leaks(&clean, leak),
        "control: a secret_widgets-held fixture must not persist"
    );

    // DELIBERATE LEAK A (values misroute): the fixture forced into
    // `state.values` under a NON-secret name escapes the name-net and SURVIVES
    // serialization — `persist_leaks` MUST detect it. (No real secret used.)
    let mut leaky = FormState::default();
    leaky.values.push((
        "--not-a-classified-secret-name".into(),
        FlagValue::Text(leak.into()),
    ));
    assert!(
        persist_leaks(&leaky, leak),
        "the persist check is VACUOUS — it failed to detect a fixture surviving \
         redaction under a non-secret name"
    );

    // DELIBERATE LEAK B (pre-redaction tree): a tree `key` serialized WITHOUT
    // redaction leaks — proving `redact_for_persistence` is the load-bearing
    // step (a regression that dropped the tree-redact call would surface here).
    let mut ts = TreeState::fresh();
    ts.root = TreeNode {
        id: 0,
        kind: "pk".into(),
        key: leak.into(),
        ..Default::default()
    };
    let tree = FormState {
        tree: Some(ts),
        ..Default::default()
    };
    let unredacted = serde_json::to_string(&tree).expect("serializes");
    assert!(
        unredacted.contains(leak),
        "the pre-redaction tree serialize should contain the key — the \
         post-redaction assertion would otherwise be vacuous"
    );
}

#[test]
fn i3_negative_masked_preview_check_bites() {
    let leak = "FAKE_SECRET_FIXTURE_NEG_PREVIEW";
    let argv = vec!["mnemonic".to_string(), "--passphrase".to_string(), leak.to_string()];

    // Misclassified (mask=false): the fixture renders CLEARTEXT in the masked
    // preview — `render_copy_command_masked` must surface it, proving the
    // surface-2 check is non-vacuous.
    let unmasked = render_copy_command_masked(&argv, &[false, false, false], ShellFlavor::Posix);
    assert!(
        unmasked.contains(leak),
        "the masked-preview check is VACUOUS — an unmasked secret token did not \
         render in the preview"
    );

    // Correctly classified (mask=true): the token is replaced with the mask
    // sentinel — the fixture is ABSENT.
    let masked = render_copy_command_masked(&argv, &[false, false, true], ShellFlavor::Posix);
    assert!(
        !masked.contains(leak),
        "a mask=true secret token must be redacted (`••••`) in the preview"
    );
}
