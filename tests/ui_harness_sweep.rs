//! Phase P5 — the **one-time coverage SWEEP** (the bug-finder).
//!
//! Plan: `design/IMPLEMENTATION_PLAN_gui_ui_test_harness.md` Phase P5.
//! Spec:  `design/SPEC_gui_automated_ui_test_harness.md` §6 / §7.
//!
//! P1–P4 proved the invariants on a vertical slice (I1), the 17 conditional
//! subcommands (I2), every classified secret (I3) and a curated real-CLI set
//! (I4). P5 **forces the rest**: it runs the I1 form→argv wiring round-trip
//! over the identity flags of **all 61 subcommands** (mnemonic 32 + md 10 +
//! ms 10 + mk 9), plus bounded-`proptest` depth fuzzers for I1 leaf values,
//! I2 renderer↔rule faithfulness over random states, and I3 secret fixtures.
//! **This phase EXPECTS to find bugs — that is its job.**
//!
//! ## Run modes (stated per plan P5)
//! - [`i1_wiring_sweep_all_61`] + [`sweep_census`] are **NORMALLY-RUN**:
//!   deterministic, no proptest randomness, no flake/shrink, fast (~one harness
//!   render per identity flag). They are the coverage GATE + the always-on
//!   bug-finder, safe for `cargo test --jobs 2`.
//! - The three `*_proptest_*` fuzzers are **`#[ignore]`-marked** (run on demand:
//!   `cargo test --test ui_harness_sweep -- --ignored`). They carry proptest
//!   randomness — exactly the flake/shrink/regression-file noise the plan says
//!   to keep OUT of every-commit CI (P6 owns the permanent deterministic gate).
//!   `failure_persistence: None` keeps them from writing a `proptest-regressions`
//!   file into the repo. P5 ran them this cycle to hunt bugs; CI does not.
//!
//! ## The I1 sweep oracle (conditional-aware → false-positive-free)
//! For each `(subcommand, identity-flag)`: pick the first [candidate base]
//! (`ui_harness::sweep_candidate_bases`) under which the flag renders
//! **Visible/Required** and is NOT [render-suppressed](ui_harness::is_render_suppressed)
//! (M1 discipline), strip the under-test flag, **widget-inject** a guaranteed
//! NON-default value, run-to-stable, then assert the injected value is argv-bound
//! to its flag. If injecting the value SELF-gates the flag to Hidden/Disabled/
//! PinValue (rare; e.g. a flag conditional on its own value), the cell SKIPS
//! (that surface is owned by I2 — `ui_harness_i2_conditional.rs`), so the sweep
//! never false-fails on legitimate conditional behavior. Flags with no possible
//! non-default injection (single-option dropdowns, degenerate `min==max==default`
//! numbers) are NARROWED + reported, never faked.
//!
//! ## Triage bar (m7) — bounds the fix loop
//! In-cycle, FIX ONLY funds- or secret-Critical findings (and loudly flag any
//! `src/` change + add a regression cell). Everything else is FILED as a
//! FOLLOWUP. The harness + its census ship regardless.

mod ui_harness;

use std::collections::BTreeMap;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use proptest::prelude::*;
use proptest::test_runner::{Config, TestCaseError, TestRunner};

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::invocation::{
    assemble_argv, assemble_argv_with_secret_mask, render_copy_command_masked, ShellFlavor,
};
use mnemonic_gui::form::widget::render_with_dispatch;
use mnemonic_gui::persistence::redact_for_persistence;
use mnemonic_gui::schema::{
    FlagKind, FlagSchema, FlagValue, FormState, SubcommandSchema, Visibility,
};
use mnemonic_gui::secrets::flag_is_secret;

use ui_harness::{
    drive, effect_of, identity_flags, is_render_suppressed, present_value, render_flag_harness,
    render_whole_form_harness, schema_for, sweep_candidate_bases, IdentityKind, Injected,
};

// ════════════════════════════════════════════════════════════════════════
// Non-default leaf injection (per identity kind)
// ════════════════════════════════════════════════════════════════════════

/// A pool of FAKE, shell-safe (`[A-Za-z0-9_./-]`) Text/Path fixtures. The first
/// entry that differs from the flag's schema default is used, so the injected
/// value is guaranteed NON-default (else `is_at_default` would suppress it and
/// false-RED the round-trip).
const TEXT_FIXTURES: &[&str] = &[
    "SWEEP_FIXTURE_ALPHA",
    "SWEEP_FIXTURE_BRAVO",
    "/tmp/sweep-fixture-charlie",
];

fn text_fixture(default: Option<&str>) -> Option<&'static str> {
    TEXT_FIXTURES.iter().copied().find(|s| Some(*s) != default)
}

/// A number within `[min, resolved_max]` that differs from the flag's schema
/// default, or `None` for a degenerate `min==max==default` range (NARROWED).
fn number_injection(flag: &FlagSchema, state: &FormState) -> Option<i64> {
    let FlagKind::Number { min, max } = flag.kind else {
        return None;
    };
    let hi = max.resolve(state);
    if hi < min {
        return None;
    }
    let default = flag.default_value.and_then(|s| s.parse::<i64>().ok());
    let mid = min + (hi - min) / 2;
    [min, min + 1, hi, hi - 1, mid]
        .into_iter()
        .find(|&c| c >= min && c <= hi && Some(c) != default)
}

/// A non-empty Dropdown option distinct from the rendered-selected default
/// (= `default_value` or `opts[0]`), or `None` for a single-option dropdown
/// (NARROWED — no observably-distinct option to select).
fn dropdown_injection(flag: &FlagSchema) -> Option<&'static str> {
    let FlagKind::Dropdown(opts) = flag.kind else {
        return None;
    };
    let selected = flag
        .default_value
        .unwrap_or_else(|| opts.first().copied().unwrap_or(""));
    opts.iter().copied().find(|o| !o.is_empty() && *o != selected)
}

/// The canonical NON-default injection for `flag` under `state`, or `None` if
/// the flag has no possible non-default injection (NARROWED).
fn injected_for(flag: &FlagSchema, state: &FormState, kind: IdentityKind) -> Option<Injected> {
    match kind {
        IdentityKind::Text | IdentityKind::Path => {
            text_fixture(flag.default_value).map(Injected::Text)
        }
        IdentityKind::Number => number_injection(flag, state).map(Injected::Number),
        IdentityKind::Dropdown => dropdown_injection(flag).map(Injected::Dropdown),
        // Boolean default is always-false unless the schema declares "true"
        // (then driving ON is at-default → suppressed → no injection possible).
        IdentityKind::Boolean => {
            (flag.default_value != Some("true")).then_some(Injected::Boolean(true))
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// The I1 cell + its outcome taxonomy
// ════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
enum Cell {
    /// The round-trip was asserted GREEN (the injected value bound to argv).
    Checked,
    /// No candidate base renders the flag Visible/Required (it is gate-hidden in
    /// every seed state — owned by I2). Reason string for the census.
    SkippedSuppressed,
    /// The flag has no possible NON-default injection (single-option dropdown /
    /// degenerate number range). NARROWED — never faked.
    Narrowed,
    /// Injecting the value SELF-gated the flag to Hidden/Disabled/PinValue —
    /// legitimate conditional behavior owned by I2; the round-trip is N/A.
    SelfGated,
    /// A real I1 invariant violation (the injected value did NOT reach argv
    /// where it should have). The string is a coordinate-only description.
    Finding(String),
}

/// The first candidate base under which `flag` renders Visible/Required and is
/// NOT mode-suppressed, **prepared for injection**: the under-test flag is
/// stripped (§5 IMP-3 — its value is solely widget-injected) and, for `Text` /
/// `Path` kinds, re-seeded with an EMPTY starting buffer.
///
/// ## Why the empty re-seed (harness artifact fix, not a GUI behavior change)
/// `render_with_dispatch` pre-fills an ABSENT Text/Path flag with its schema
/// `default_value` (e.g. `--feerate` → "1.0", `--output` → "-"). `type_text`
/// then APPENDS to that pre-filled text, so a naive strip+type yields
/// "1.0SWEEP_FIXTURE…" — a HARNESS artifact (the production widget is correct;
/// the assertion was simply not modeling a user clearing the default first).
/// Seeding `Text("")` / `Path("")` starts the field empty (absent-equivalent
/// for the conditional gate — `has_value` is false either way), so the injected
/// value is asserted PURE. Number (Unset→Set→SetValue) and Dropdown (select)
/// REPLACE rather than append, so they need no empty seed.
fn prepared_eligible_base(
    tab: CliTab,
    sub: &'static SubcommandSchema,
    flag: &'static FlagSchema,
    kind: IdentityKind,
) -> Option<FormState> {
    for mut base in sweep_candidate_bases(tab, sub.name) {
        base.values.retain(|(k, _)| k != flag.name);
        base.secret_widgets.remove(flag.name);
        if is_render_suppressed(sub, flag.name, &base) {
            continue;
        }
        if !matches!(
            effect_of(sub, &base, flag.name),
            Visibility::Visible | Visibility::Required
        ) {
            continue;
        }
        match kind {
            IdentityKind::Text => base
                .values
                .push((flag.name.to_string(), FlagValue::Text(String::new()))),
            IdentityKind::Path => base
                .values
                .push((flag.name.to_string(), FlagValue::Path(String::new()))),
            _ => {}
        }
        return Some(base);
    }
    None
}

/// Run the I1 round-trip for one `(sub, identity-flag)`: find the first
/// candidate base exposing the flag Visible/Required, inject a non-default
/// value, settle, and assert the conditional-aware round-trip.
fn i1_cell(tab: CliTab, sub: &'static SubcommandSchema, flag: &'static FlagSchema, kind: IdentityKind) -> Cell {
    let coord = format!("{}/{}/{}", tab.bin_name(), sub.name, flag.name);

    let Some(base) = prepared_eligible_base(tab, sub, flag, kind) else {
        return Cell::SkippedSuppressed;
    };

    // Produce a guaranteed-non-default injection (borrow ends before move).
    let Some(injected) = injected_for(flag, &base, kind) else {
        return Cell::Narrowed;
    };

    let mut h = render_flag_harness(tab, sub, flag, base);
    h.run();
    drive(&mut h, kind, &injected);
    let state = h.state();

    // Settled-effect-aware: if injecting self-gated the flag, defer to I2.
    if !matches!(
        effect_of(sub, state, flag.name),
        Visibility::Visible | Visibility::Required
    ) {
        return Cell::SelfGated;
    }

    let argv = assemble_argv(schema_for(tab), sub, state);
    match injected.expected_token() {
        // Value kinds: the injected token must bind immediately after the flag.
        Some(expected) => match argv.iter().position(|t| t == flag.name) {
            Some(pos) if argv.get(pos + 1) == Some(&expected) => Cell::Checked,
            _ => Cell::Finding(format!(
                "I1 WIRING [{coord}]: the widget-injected value did not bind to its \
                 flag in argv (Visible/Required, expected `{} <value>`)",
                flag.name
            )),
        },
        // Boolean: presence-only emission.
        None => {
            if argv.iter().any(|t| t == flag.name) {
                Cell::Checked
            } else {
                Cell::Finding(format!(
                    "I1 WIRING [{coord}]: driving the checkbox ON did not emit the flag"
                ))
            }
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// THE SWEEP — deterministic, all 61, normally-run (the coverage gate)
// ════════════════════════════════════════════════════════════════════════

#[derive(Default)]
struct Tally {
    checked: usize,
    skipped_suppressed: usize,
    narrowed: usize,
    self_gated: usize,
    findings: Vec<String>,
    /// subcommand → (identity-flag count, checked count)
    per_sub: BTreeMap<String, (usize, usize)>,
    /// subcommands with ZERO identity flags (no I1 surface at all).
    no_identity_surface: Vec<String>,
}

fn run_full_sweep() -> Tally {
    let mut t = Tally::default();
    for tab in CliTab::ALL.iter().copied() {
        for sub in schema_for(tab).subcommands {
            let idents: Vec<_> = identity_flags(sub).collect();
            let key = format!("{}/{}", tab.bin_name(), sub.name);
            if idents.is_empty() {
                t.no_identity_surface.push(key.clone());
            }
            let mut checked_here = 0usize;
            for (flag, kind) in idents.iter().copied() {
                match i1_cell(tab, sub, flag, kind) {
                    Cell::Checked => {
                        t.checked += 1;
                        checked_here += 1;
                    }
                    Cell::SkippedSuppressed => t.skipped_suppressed += 1,
                    Cell::Narrowed => t.narrowed += 1,
                    Cell::SelfGated => t.self_gated += 1,
                    Cell::Finding(msg) => t.findings.push(msg),
                }
            }
            t.per_sub.insert(key, (idents.len(), checked_here));
        }
    }
    t
}

/// The headline coverage gate: run the I1 wiring round-trip over every identity
/// flag of all 61 subcommands and assert NO real wiring findings. (Skips /
/// narrows are expected and reported by [`sweep_census`].)
#[test]
fn i1_wiring_sweep_all_61() {
    let t = run_full_sweep();
    assert!(
        t.findings.is_empty(),
        "I1 wiring sweep found {} real finding(s):\n{}",
        t.findings.len(),
        t.findings.join("\n")
    );
    // Non-vacuity: the sweep actually drove a broad set of round-trips.
    assert!(
        t.checked >= 80,
        "the sweep checked only {} identity round-trips — coverage regressed",
        t.checked
    );
}

/// Coverage census — prints the per-subcommand I1 coverage table (run with
/// `--nocapture` to see it) and pins the structural coverage facts so a future
/// schema change that silently drops I1 surface is caught.
#[test]
fn sweep_census() {
    let t = run_full_sweep();
    let n_subs = t.per_sub.len();
    let subs_with_cover = t.per_sub.values().filter(|(_, c)| *c > 0).count();

    eprintln!("\n──────── P5 I1 WIRING SWEEP CENSUS ────────");
    eprintln!(
        "subcommands: {n_subs}/61 | with ≥1 round-tripped identity flag: {subs_with_cover}"
    );
    eprintln!(
        "identity round-trips: checked={} skipped(suppressed)={} narrowed={} self-gated={} findings={}",
        t.checked, t.skipped_suppressed, t.narrowed, t.self_gated, t.findings.len()
    );
    eprintln!("subcommands with NO identity-flag surface (positional/secret/repeating/transform only):");
    for s in &t.no_identity_surface {
        eprintln!("    {s}");
    }
    eprintln!("per-subcommand (identity-flags / checked):");
    for (k, (n, c)) in &t.per_sub {
        if *n > 0 {
            eprintln!("    {k:50} {c}/{n}");
        }
    }
    eprintln!("───────────────────────────────────────────\n");

    assert_eq!(n_subs, 61, "expected exactly 61 subcommands across the 4 CLIs");
    // The 4 CLIs each surface ≥1 round-trippable subcommand (coverage floor).
    assert!(
        subs_with_cover >= 40,
        "only {subs_with_cover} subcommands round-tripped an identity flag — \
         the seed table under-covers"
    );
}

// ════════════════════════════════════════════════════════════════════════
// PROPTEST DEPTH FUZZERS (#[ignore] — the one-time finders; run on demand)
// ════════════════════════════════════════════════════════════════════════

/// Build the flat list of `(tab, sub, flag, kind)` eligible for an I1 round-trip
/// (≥1 candidate base exposes them Visible/Required with a non-default
/// injection). Used by the proptest depth fuzzer to pick a target.
fn eligible_i1_targets() -> Vec<(CliTab, &'static SubcommandSchema, &'static FlagSchema, IdentityKind)> {
    let mut v = Vec::new();
    for tab in CliTab::ALL.iter().copied() {
        for sub in schema_for(tab).subcommands {
            for (flag, kind) in identity_flags(sub) {
                if matches!(i1_cell(tab, sub, flag, kind), Cell::Checked) {
                    v.push((tab, sub, flag, kind));
                }
            }
        }
    }
    v
}

fn proptest_config(cases: u32) -> Config {
    Config {
        cases,
        // Do NOT write a proptest-regressions file into the repo (P5 is a
        // one-time finder; the permanent gate is P6's deterministic table).
        failure_persistence: None,
        ..Config::default()
    }
}

/// I1 leaf-value depth fuzzer: over the eligible identity targets, vary the
/// injected leaf (random in-range Number / random non-default Dropdown option /
/// random alphanumeric Text) and assert the round-trip still binds. Numbers are
/// the genuine value (boundary fuzzing); Text/Dropdown variation is breadth.
#[test]
#[ignore = "P5 one-time proptest finder; run with --ignored"]
fn i1_leaf_value_proptest() {
    let targets = eligible_i1_targets();
    assert!(!targets.is_empty(), "no eligible I1 targets");
    let mut runner = TestRunner::new(proptest_config(96));
    runner
        .run(
            &(0..targets.len(), 0u32..1_000_000, "[A-Za-z0-9_]{1,24}"),
            |(idx, num_seed, text)| {
                let (tab, sub, flag, kind) = targets[idx];
                // The prepared base (stripped + Text/Path empty-seeded — same
                // path i1_cell uses).
                let Some(base) = prepared_eligible_base(tab, sub, flag, kind) else {
                    return Ok(()); // raced out of eligibility — skip
                };

                // A randomized but VALID, non-default injection for this kind.
                let injected = match kind {
                    IdentityKind::Number => {
                        let FlagKind::Number { min, max } = flag.kind else {
                            return Ok(());
                        };
                        let hi = max.resolve(&base);
                        if hi < min {
                            return Ok(());
                        }
                        let span = (hi - min + 1) as u64;
                        let mut n = min + (num_seed as i64 % span as i64);
                        let default = flag.default_value.and_then(|s| s.parse::<i64>().ok());
                        if Some(n) == default {
                            n = if n < hi { n + 1 } else { n - 1 };
                        }
                        if n < min || n > hi {
                            return Ok(());
                        }
                        Injected::Number(n)
                    }
                    // Text/Path: a fresh random alphanumeric (leaked into a
                    // 'static via Box::leak — test-only, bounded by `cases`).
                    IdentityKind::Text | IdentityKind::Path => {
                        Injected::Text(Box::leak(text.into_boxed_str()))
                    }
                    // Dropdown / Boolean have a tiny fixed space; reuse the
                    // canonical injection (the deterministic sweep covers them).
                    _ => match injected_for(flag, &base, kind) {
                        Some(i) => i,
                        None => return Ok(()),
                    },
                };

                let expected = injected.expected_token();
                let mut h = render_flag_harness(tab, sub, flag, base);
                h.run();
                drive(&mut h, kind, &injected);
                let state = h.state();
                if !matches!(
                    effect_of(sub, state, flag.name),
                    Visibility::Visible | Visibility::Required
                ) {
                    return Ok(()); // self-gated — owned by I2
                }
                let argv = assemble_argv(schema_for(tab), sub, state);
                match expected {
                    Some(exp) => {
                        let bound = argv
                            .iter()
                            .position(|t| t == flag.name)
                            .is_some_and(|p| argv.get(p + 1) == Some(&exp));
                        if !bound {
                            return Err(TestCaseError::fail(format!(
                                "I1 leaf round-trip failed for {}/{}/{}",
                                tab.bin_name(),
                                sub.name,
                                flag.name
                            )));
                        }
                    }
                    None => {
                        if !argv.iter().any(|t| t == flag.name) {
                            return Err(TestCaseError::fail(format!(
                                "I1 boolean emission failed for {}/{}/{}",
                                tab.bin_name(),
                                sub.name,
                                flag.name
                            )));
                        }
                    }
                }
                Ok(())
            },
        )
        .unwrap();
}

/// I2 renderer↔rule faithfulness over RANDOM states: for each of the 17
/// conditional subs, set a random subset of flags "present", render the WHOLE
/// form (the mode-aware path — M1), and assert every eligible flag's rendered
/// presence/disabled state equals `conditional(state)`'s effect. Catches a
/// renderer↔rule desync under state combinations the curated P2 cells never hit.
#[test]
#[ignore = "P5 one-time proptest finder; run with --ignored"]
fn i2_render_faithfulness_proptest() {
    let conditional_subs: Vec<(CliTab, &'static SubcommandSchema)> = CliTab::ALL
        .iter()
        .copied()
        .flat_map(|tab| {
            schema_for(tab)
                .subcommands
                .iter()
                .filter(|s| s.conditional.is_some())
                .map(move |s| (tab, s))
        })
        .collect();
    assert_eq!(conditional_subs.len(), 17, "expected 17 conditional subs");

    for (tab, sub) in conditional_subs {
        let n = sub.flags.len();
        let mut runner = TestRunner::new(proptest_config(24));
        runner
            .run(&proptest::collection::vec(any::<bool>(), n), |mask| {
                let mut state = FormState::default();
                for (flag, present) in sub.flags.iter().zip(mask.iter()) {
                    if *present {
                        state
                            .values
                            .push((flag.name.to_string(), present_value(flag, "x")));
                    }
                }
                let mut h = render_whole_form_harness(tab, sub, state);
                h.run();
                let st = h.state();
                for flag in sub.flags {
                    if !ui_harness::eligible_for_label_check(sub, flag, st) {
                        continue;
                    }
                    let effect = effect_of(sub, st, flag.name);
                    let present = ui_harness::label_present(&h, flag.name);
                    let disabled = ui_harness::label_disabled(&h, flag.name);
                    let ok = match &effect {
                        Visibility::Hidden => !present,
                        Visibility::Disabled => disabled == Some(true),
                        _ => disabled == Some(false), // Visible/Required/PinValue
                    };
                    if !ok {
                        return Err(TestCaseError::fail(format!(
                            "I2 renderer↔rule desync at {}/{}/{} (effect={effect:?}, \
                             present={present}, disabled={disabled:?})",
                            tab.bin_name(),
                            sub.name,
                            flag.name
                        )));
                    }
                }
                Ok(())
            })
            .unwrap();
    }
}

/// I3 secret never-leak over RANDOM fixtures: for each value-bearing classified
/// secret, widget-inject a random FAKE fixture and assert it never reaches the
/// persisted state nor the masked preview. (FAKE fixtures only; failure messages
/// are coordinate-only — no payload echo, per the I3 hygiene rule.)
#[test]
#[ignore = "P5 one-time proptest finder; run with --ignored"]
fn i3_secret_fixture_proptest() {
    let secrets: Vec<(CliTab, &'static SubcommandSchema, &'static FlagSchema)> = CliTab::ALL
        .iter()
        .copied()
        .flat_map(|tab| {
            schema_for(tab).subcommands.iter().flat_map(move |sub| {
                sub.flags
                    .iter()
                    .filter(|f| {
                        flag_is_secret(f)
                            && matches!(f.kind, FlagKind::Text | FlagKind::NodeValueComposite(_))
                    })
                    .map(move |f| (tab, sub, f))
            })
        })
        .collect();
    assert!(!secrets.is_empty(), "no value-bearing classified secrets");

    let mut runner = TestRunner::new(proptest_config(64));
    runner
        .run(
            &(0..secrets.len(), "FAKE_[A-Z0-9_]{4,20}"),
            |(idx, fixture)| {
                let (tab, sub, flag) = secrets[idx];
                let mut h = Harness::new_ui_state(
                    move |ui, state: &mut FormState| {
                        render_with_dispatch(ui, tab, sub.name, flag, state, &[]);
                    },
                    FormState::default(),
                );
                h.run();
                // Add a repeating-secret row if needed, then type the fixture.
                let has_input = h.query_by_role(Role::PasswordInput).is_some()
                    || h.query_by_role(Role::TextInput).is_some();
                if !has_input && h.query_by_label("+ add").is_some() {
                    h.get_by_label("+ add").click();
                    h.run();
                }
                if h.query_by_role(Role::PasswordInput).is_some() {
                    h.get_by_role(Role::PasswordInput).type_text(&fixture);
                } else if h.query_by_role(Role::TextInput).is_some() {
                    h.get_by_role(Role::TextInput).type_text(&fixture);
                } else {
                    return Ok(()); // no input rendered in a bare state — skip
                }
                h.run();
                h.run();
                let state = h.state();

                // Surface 1 — persisted state (redact walk).
                let redacted = redact_for_persistence(state);
                let json = serde_json::to_string(&redacted).expect("serializes");
                if json.contains(fixture.as_str()) {
                    return Err(TestCaseError::fail(format!(
                        "I3 PERSIST LEAK at {}/{}/{} (payload withheld)",
                        tab.bin_name(),
                        sub.name,
                        flag.name
                    )));
                }
                // Surface 2 — masked preview.
                let (argv, mask) = assemble_argv_with_secret_mask(schema_for(tab), sub, state);
                let posix = render_copy_command_masked(&argv, &mask, ShellFlavor::Posix);
                if posix.contains(fixture.as_str()) {
                    return Err(TestCaseError::fail(format!(
                        "I3 PREVIEW LEAK at {}/{}/{} (payload withheld)",
                        tab.bin_name(),
                        sub.name,
                        flag.name
                    )));
                }
                Ok(())
            },
        )
        .unwrap();
}
