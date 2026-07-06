//! The `gui_example.pdf` tutorial CAPTURE HARNESS + its always-run gates
//! (`gui_example_tutorial` cycle, plan P1.4).
//!
//! Authority: `mnemonic-toolkit/docs/manual-gui/design/SPEC_gui_example_tutorial.md`
//! §3.1(b) (harness + the two named gates), §5 (manifest + shots), §6
//! (determinism), §7 (secret hygiene); `IMPLEMENTATION_PLAN_gui_example_tutorial.md`
//! P1.4; ratifications in `agent-reports/gui-example-p0-spike.md` (the proven
//! mechanics this file re-authors fresh) + ruling 9 (single-`step()` click
//! semantics) + ruling 2 (injected-MouseWheel scroll).
//!
//! Two test surfaces live here:
//!   1. **Always-run unit gates** (plain `cargo test`, no rasterizer): the
//!      manifest-stems regen-diff census, the secret-allowlist checker, the
//!      fixture watch-only scan, the corpus-budget ceiling, and the two
//!      gate NEGATIVES (each proves its gate BITES) — all drive off the
//!      egui-free `tutorial` manifest module.
//!   2. **The env-gated capture harness** `gui_tutorial_snapshots`
//!      (`GUI_TUTORIAL_SNAPSHOTS=1`, early-return-skip): drives the REAL
//!      whole-window app (`app_window` shell) at 920×720 @ ppp 2.0, fills each
//!      manifest step, captures the filled-form + (secret) modal + populated
//!      pane shots, and byte-persists the transcripts. The `build.yml`
//!      `tutorial-snapshots` job (P1.6) is the enforcing consumer.
//!
//! P1.5 grows the corpus to the FULL journey set (J1–J5) by EXTENDING both the
//! manifest AND this harness: additive `Drive` interpreter arms (dropdown /
//! text / path / composite / number / `+ Add slot` / `--md1` rows), the
//! intra-journey md1 chain, and `capture: false` transcript-only steps — all
//! over the SAME de-risked spine (whole-window `app.ui`, same-frame runner,
//! secret masking, byte-reproducibility) proven at P1.4. No spine surgery.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use egui::accesskit::Role;
use egui_kittest::kittest::{by, Node, Queryable};
use egui_kittest::{Harness, SnapshotOptions};

use mnemonic_gui::app::{AppState, CliTab};
use mnemonic_gui::app_window::MnemonicGuiApp;
use mnemonic_gui::form::secret_widget::{revealed_field, REVEAL_EYE_GLYPH};
use mnemonic_gui::form::slot_editor::SlotSubkey;
use mnemonic_gui::path_detect::Detected;

mod tutorial;
use tutorial::{Drive, Step};

/// The intra-journey chain carry: a source step's parsed md1 chunks, keyed by
/// that step's stem. A `Drive::TypeMd1Chain { chain_from }` in a LATER step
/// reads this (the `&'static` manifest can't hold runtime-derived text, so the
/// carry lives here — R0 item 2 / m5). Populated after every run in
/// [`execute_step`]; read by [`apply_drive`].
type ChainStore = HashMap<&'static str, Vec<String>>;

// ─── shared constants ────────────────────────────────────────────────────────

const PPP: f32 = 2.0;
/// The single global window size (spike S4-ratified: the production default
/// window seed, `main.rs:52` / `app_window.rs`).
const WINDOW_SIZE: [f32; 2] = [920.0, 720.0];
/// The committed tutorial corpus dir (leg-2 `verify-tutorial-figures` /
/// `verify-tutorial-transcripts` byte-copy from here).
const TUTORIAL_SNAPSHOT_DIR: &str = "tests/snapshots/tutorial";
/// The fixture dir the whole test pins cwd to (SPEC §6.6).
const FIXTURE_DIR: &str = "tests/tutorial/fixtures";
/// The committed corpus census (regenerated + diffed by the census gate).
const MANIFEST_STEMS: &str = "tests/tutorial/manifest-stems.txt";
/// HARD corpus ceiling. Originally 20 MiB (SPEC §5.3 ruling 4), set when the
/// concern was a *committed* PDF. The 2026-07-05 user decision made the PDF
/// RELEASE-ATTACH-ONLY (spec §3.2c), resolving that rationale; the full
/// all-shots corpus (50 committed; 51 nominal, 1 modal trimmed) measured
/// 27.1 MiB (24 real populated-pane shots). USER DECISION 2026-07-05: raise the
/// ceiling, keep all shots. 32 MiB = ~18%
/// regen-drift headroom over the measured 27.1.
const BUDGET_HARD_MIB: f64 = 32.0;
/// Corpus target (report-only above this).
const BUDGET_TARGET_MIB: f64 = 28.0;

fn manifest_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

// ═════════════════════════════════════════════════════════════════════════════
//  ALWAYS-RUN GATES (no rasterizer, no pinned CLI — plain `cargo test`)
// ═════════════════════════════════════════════════════════════════════════════

/// Census: the manifest-derived corpus artifact list is unique, sorted, and
/// byte-matches the committed `manifest-stems.txt` (the single source both
/// repos' figure/transcript censuses read). Regenerate with
/// `UPDATE_TUTORIAL_STEMS=1`.
#[test]
fn manifest_stems_regen_diff() {
    // Uniqueness: no artifact basename appears twice across the manifest.
    let raw = tutorial::corpus_manifest_raw();
    let mut deduped = raw.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(
        raw.len(),
        deduped.len(),
        "duplicate corpus artifact basename(s) in the manifest — every step's stems must \
         be unique:\n{raw:#?}"
    );

    // Ordering: the emitted census is sorted.
    let census = tutorial::corpus_manifest();
    let mut sorted = census.clone();
    sorted.sort();
    assert_eq!(census, sorted, "corpus_manifest() must be emitted sorted");

    // Regen-diff against the committed file.
    let expected = tutorial::manifest_stems_txt();
    let path = manifest_path(MANIFEST_STEMS);
    if std::env::var("UPDATE_TUTORIAL_STEMS").as_deref() == Ok("1") {
        std::fs::write(&path, &expected).expect("write manifest-stems.txt");
        eprintln!("TUTORIAL-STEMS: regenerated {}", path.display());
        return;
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "manifest-stems.txt missing/unreadable ({e}); regenerate with \
             UPDATE_TUTORIAL_STEMS=1 cargo test --test gui_tutorial_snapshots \
             manifest_stems_regen_diff"
        )
    });
    assert_eq!(
        committed, expected,
        "manifest-stems.txt drift — the committed census no longer matches the manifest; \
         regenerate with UPDATE_TUTORIAL_STEMS=1"
    );
}

/// Secret-allowlist (SPEC §7): every value the manifest routes to a
/// secret-classified widget is one of the three published phrases; the sweep
/// is non-vacuous (at least one secret drive exists — J1).
#[test]
fn secret_values_are_allowlisted() {
    let violations = tutorial::secret_allowlist_violations();
    assert!(
        violations.is_empty(),
        "secret-allowlist violation(s) ({}):\n{}",
        violations.len(),
        violations.join("\n")
    );
    assert!(
        tutorial::secret_drive_count() > 0,
        "non-vacuous sweep: the manifest must drive at least one secret value (J1's \
         phrase slot) — a green-but-empty allowlist check is a hygiene blind spot"
    );
    assert!(
        tutorial::node_secret_taxonomy_nonempty(),
        "the SECRET_NODE_TYPES_ARGV / SECRET_FLAG_NAMES taxonomies must be reachable"
    );
}

/// P1.4 RULING 1 — the reveal marker set is RULE-derived (`Drive::secret_value()
/// .is_some()` over `capture: true` steps), fail-closed both ways, and is
/// EXACTLY the four designated steps.
#[test]
fn reveal_markers_are_rule_derived() {
    let violations = tutorial::reveal_marker_violations();
    assert!(
        violations.is_empty(),
        "reveal-marker census violation(s) ({}):\n{}",
        violations.len(),
        violations.join("\n")
    );
    let mut marked = tutorial::reveal_marked_stems();
    marked.sort();
    assert_eq!(
        marked,
        vec![
            "tut-j1-01-bundle-single-sig",
            "tut-j2-02-convert-fingerprint",
            "tut-j2-03-convert-xpub",
            "tut-j2-07-bundle-all-seeds",
        ],
        "the reveal-marked set must be EXACTLY the four capture:true secret-value steps \
         (the six restore steps type a PUBLIC md1 card → no marker; the J2 devices-1/2 \
         converts are capture:false → no marker)"
    );
}

/// P1.4 RULING 2 — the ⊆-agreement: any field the widget masks (hence CAN reveal)
/// is classified secret by the allowlist checker's taxonomy, so nothing
/// revealable escapes the allowlist gate.
#[test]
fn reveal_in_scope_fields_are_checker_classified() {
    // Every reveal-marked step reveals a checker-classified secret value.
    for s in tutorial::MANIFEST.iter().filter(|s| s.reveal) {
        assert!(
            s.revealed_value().is_some(),
            "{}: a reveal-marked step must reveal a checker-classified secret value",
            s.stem
        );
    }
    // Slots: the widget-mask gate (`is_secret_bearing`) EQUALS the checker
    // taxonomy (`slot_subkey_is_secret`) for EVERY subkey. Composites mask on
    // `node_type_is_argv_secret` — the SAME fn `Drive::secret_value` classifies
    // on — so agreement there is by construction.
    for sk in SlotSubkey::ALL {
        assert_eq!(
            sk.is_secret_bearing(),
            mnemonic_gui::secrets::slot_subkey_is_secret(*sk),
            "slot subkey {sk:?}: widget-mask gate != checker taxonomy — a revealable field \
             could escape the allowlist"
        );
    }
}

/// A minimal synthetic step for the allowlist-checker negative.
fn synthetic_step(stem: &'static str, drives: &'static [Drive]) -> Step {
    Step {
        stem,
        tab: CliTab::Mnemonic,
        subcommand: "convert",
        select: None,
        drives,
        scroll: &[],
        runs: false,
        secret_modal: false,
        modal_shot: false,
        capture: true,
        expect_exit: None,
        expect_stderr: false,
        reveal: false,
    }
}

/// P1.4 RULING 2 negative — the allowlist checker BITES: a NON-allowlisted secret
/// value routed to a secret widget is RED-flagged (whether or not the step is
/// reveal-marked), and an allowlisted one is clean.
#[test]
fn allowlist_checker_bites_on_non_allowlisted_secret() {
    const LEAK: &str = "not a real seed but classified secret via the phrase node abc";
    const BAD_DRIVES: &[Drive] = &[Drive::TypeComposite {
        flag: "--from",
        node: "phrase",
        value: LEAK,
    }];
    const OK_DRIVES: &[Drive] = &[Drive::TypeSlot {
        anchor: "@",
        occ: 0,
        subkey: SlotSubkey::Phrase,
        value: tutorial::S0,
    }];

    // Negative: a non-allowlisted secret drive is flagged (proves the gate BITES).
    let bad = synthetic_step("synthetic-leak", BAD_DRIVES);
    let v = tutorial::check_allowlist(std::slice::from_ref(&bad));
    assert!(
        v.iter().any(|m| m.contains("synthetic-leak")),
        "the allowlist checker must RED-flag a non-allowlisted secret drive: {v:?}"
    );
    // A reveal-marked step with a non-allowlisted revealed value is caught by the
    // SAME checker (the filled-form parameterization permits ONLY the field's own
    // value, which check_allowlist gates first — so the reveal opens no hole).
    let bad_reveal = Step { reveal: true, ..bad };
    assert!(
        !tutorial::check_allowlist(std::slice::from_ref(&bad_reveal)).is_empty(),
        "a reveal-marked step with a non-allowlisted revealed value must still RED"
    );

    // Positive control: an allowlisted secret (S0) is clean.
    let ok = synthetic_step("synthetic-ok", OK_DRIVES);
    assert!(
        tutorial::check_allowlist(std::slice::from_ref(&ok)).is_empty(),
        "an allowlisted secret value must NOT be flagged"
    );
}

/// Fixtures are watch-only by construction: no fixture file inlines any secret
/// phrase (SPEC §7 — J3/J4/J5 are xpub-only; secrets live as manifest literals,
/// never in a committed fixture).
#[test]
fn fixtures_carry_no_secret_material() {
    let dir = manifest_path(FIXTURE_DIR);
    let mut scanned = 0usize;
    for entry in std::fs::read_dir(&dir).expect("read fixtures dir") {
        let p = entry.expect("dir entry").path();
        // The README documents provenance in prose; not a consumed fixture.
        if p.extension().and_then(|e| e.to_str()) == Some("md") {
            continue;
        }
        if !p.is_file() {
            continue;
        }
        let bytes = std::fs::read(&p).expect("read fixture");
        let text = String::from_utf8_lossy(&bytes);
        for phrase in tutorial::SECRET_ALLOWLIST {
            assert!(
                !text.contains(phrase),
                "fixture {p:?} inlines a BIP-39 phrase — fixtures MUST stay watch-only; \
                 secret material may only appear as a manifest literal (SPEC §7)"
            );
        }
        // Watch-only descriptors never carry private-key prefixes.
        for needle in ["xprv", "tprv", " wif ", "-----BEGIN"] {
            assert!(
                !text.contains(needle),
                "fixture {p:?} contains {needle:?} — private-key material is forbidden in \
                 a fixture (SPEC §7)"
            );
        }
        scanned += 1;
    }
    assert!(
        scanned >= 4,
        "expected >=4 watch-only fixtures (policy.desc, taproot.desc, taproot-4leaf.desc, \
         policy.json), scanned {scanned}"
    );
}

/// The two named gates BITE — suite-pinned, not just spike-history (plan P1.4
/// "Include a NEGATIVE test proving each gate BITES").
#[test]
fn pinned_tier_version_gate_bites() {
    // Negative: a wrong-tier probe result is rejected.
    let err = tutorial::version_matches("mnemonic", "mnemonic 0.74.0", "mnemonic 0.75.0")
        .expect_err("version gate must reject a wrong-tier string");
    assert!(
        err.contains("wrong tier"),
        "the version-gate diagnostic must name the wrong-tier refusal: {err}"
    );
    // Positive: the matching tier passes.
    assert!(tutorial::version_matches("mnemonic", "mnemonic 0.75.0", "mnemonic 0.75.0").is_ok());
}

#[test]
fn same_frame_completion_gate_bites() {
    // Negative: a run that did NOT land in the click frame (an async runner)
    // fires the tripwire by name.
    let err = tutorial::same_frame_completion(false, "unit BITE")
        .expect_err("SAME-FRAME-COMPLETION must fire when the run did not land in-frame");
    assert!(
        err.contains("SAME-FRAME-COMPLETION violated"),
        "the tripwire diagnostic must name the invariant: {err}"
    );
    // Positive: an in-frame completion passes.
    assert!(tutorial::same_frame_completion(true, "unit ok").is_ok());
}

/// HARD corpus-budget ceiling (SPEC §5.3 ruling 4). Sums the committed
/// tutorial PNGs; panics above 20 MiB, reports above the 15 MiB target. Wired
/// now (pilots only) and re-measures at the full corpus in P1.5.
#[test]
fn corpus_budget_under_ceiling() {
    let (total, count) = tutorial_png_total();
    let mib = total as f64 / (1024.0 * 1024.0);
    eprintln!("TUTORIAL-BUDGET: {count} committed PNG(s), {total} bytes ({mib:.3} MiB)");
    if mib > BUDGET_TARGET_MIB {
        eprintln!(
            "TUTORIAL-BUDGET: over the {BUDGET_TARGET_MIB} MiB target (under the \
             {BUDGET_HARD_MIB} MiB HARD ceiling) — apply the §5.3 trim levers at P1.5"
        );
    }
    assert!(
        mib <= BUDGET_HARD_MIB,
        "tutorial PNG corpus {mib:.3} MiB exceeds the HARD {BUDGET_HARD_MIB} MiB ceiling \
         (SPEC §5.3 ruling 4) — STOP: the locked all-journeys scope vs budget is a USER \
         decision"
    );
}

/// The committed PNG count matches the manifest shot count (a non-rasterizer
/// completeness guard; the in-CI harness census is the authoritative one).
/// Skips cleanly before the corpus is captured.
#[test]
fn corpus_png_count_matches_manifest() {
    let (_, count) = tutorial_png_total();
    if count == 0 {
        eprintln!("TUTORIAL-COUNT: no committed PNGs yet (pre-capture); harness census is authoritative");
        return;
    }
    assert_eq!(
        count,
        tutorial::total_shots(),
        "committed tutorial PNG count ({count}) != manifest shot count ({})",
        tutorial::total_shots()
    );
}

/// Sum + count of committed corpus PNGs (`*.png`, excluding the gitignored
/// `.new/.diff/.old` litter). `(0, 0)` if the dir does not exist yet.
fn tutorial_png_total() -> (u64, usize) {
    let dir = manifest_path(TUTORIAL_SNAPSHOT_DIR);
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return (0, 0);
    };
    let mut total = 0u64;
    let mut count = 0usize;
    for entry in rd.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.ends_with(".png") {
            continue;
        }
        if name.ends_with(".new.png") || name.ends_with(".diff.png") || name.ends_with(".old.png") {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            total += meta.len();
            count += 1;
        }
    }
    (total, count)
}

// ═════════════════════════════════════════════════════════════════════════════
//  THE CAPTURE HARNESS (GUI_TUTORIAL_SNAPSHOTS=1; software rasterizer + pinned CLI)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn gui_tutorial_snapshots() {
    if std::env::var("GUI_TUTORIAL_SNAPSHOTS").as_deref() != Ok("1") {
        eprintln!(
            "TUTORIAL-SKIP: GUI_TUTORIAL_SNAPSHOTS != 1 — gui_tutorial_snapshots skipped \
             (needs a software rasterizer + the pinned CLIs on $PATH; the build.yml \
             `tutorial-snapshots` job is the enforcing consumer)"
        );
        return;
    }

    adapter_guard();
    run_pinned_tier_version_gate();
    assert_demo_seed_baseline();

    // ── cwd pinned to the fixture dir for the whole test (SPEC §6.6) ──
    let fixture_dir = manifest_path(FIXTURE_DIR);
    std::env::set_current_dir(&fixture_dir).expect("pin cwd to the fixture dir");

    let update = std::env::var("UPDATE_SNAPSHOTS").is_ok();
    let opts = SnapshotOptions::new().output_path(manifest_path(TUTORIAL_SNAPSHOT_DIR));
    let mut failures: Vec<String> = Vec::new();
    let mut chain: ChainStore = HashMap::new();

    for step in tutorial::MANIFEST {
        execute_step(step, &opts, update, &mut failures, &mut chain);
    }

    // ── corpus-budget report over the just-captured shots (the always-run
    //    `corpus_budget_under_ceiling` test is the hard assertion) ──
    let (total, count) = tutorial_png_total();
    eprintln!(
        "TUTORIAL-BUDGET: {count} PNG(s), {:.3} MiB (HARD ceiling {BUDGET_HARD_MIB} MiB)",
        total as f64 / (1024.0 * 1024.0)
    );

    assert!(
        failures.is_empty(),
        "tutorial capture failures ({}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ─── the demo-seed baseline (m1 / SPEC §6.3) ─────────────────────────────────

/// Assert the fresh headless app's determinism baseline EXPLICITLY (not just
/// implicitly via the ch0 snapshot byte-compare + row-0 drive dependency): the
/// active subcommand is `mnemonic:bundle`, and its form carries the demo seed —
/// `--template=bip84`, `--network=mainnet`, and exactly ONE (empty, Xpub) slot
/// row. A future demo-seed change then breaks HERE by name, not as an
/// undiagnosed pixel diff or a `slots.rows[occ]` index panic (R0 m1;
/// `src/app_window.rs` `new_headless` demo-seed block).
fn assert_demo_seed_baseline() {
    let h = app_harness();
    let st = h.state();
    assert_eq!(
        st.active_subcommand
            .get(&CliTab::Mnemonic)
            .map(String::as_str),
        Some("bundle"),
        "demo-seed baseline: the fresh app must open on mnemonic:bundle (SPEC §6.3)"
    );
    let form = &st.form_state["mnemonic:bundle"];
    assert_eq!(
        form.dropdown_value("--template"),
        Some("bip84"),
        "demo-seed baseline: bundle --template must pre-fill bip84 (SPEC §6.3)"
    );
    assert_eq!(
        form.dropdown_value("--network"),
        Some("mainnet"),
        "demo-seed baseline: bundle --network must pre-fill mainnet (SPEC §6.3)"
    );
    assert_eq!(
        form.slots.rows.len(),
        1,
        "demo-seed baseline: bundle must seed exactly one slot row (SPEC §6.3)"
    );
    assert_eq!(
        form.slots.rows[0].subkey,
        SlotSubkey::Xpub,
        "demo-seed baseline: the seeded slot row must default to the Xpub subkey (SPEC §6.3)"
    );
    assert!(
        form.slots.rows[0].value.is_empty(),
        "demo-seed baseline: the seeded slot row must be empty (SPEC §6.3)"
    );
}

// ─── the generic step executor ───────────────────────────────────────────────

fn execute_step(
    step: &Step,
    opts: &SnapshotOptions,
    update: bool,
    failures: &mut Vec<String>,
    chain: &mut ChainStore,
) {
    let mut h = app_harness();

    // (1) subcommand selection — `None` rides the fresh-app default.
    if let Some(human) = step.select {
        combo_select_subcommand(&mut h, human, step.subcommand);
    }
    assert_eq!(
        h.state()
            .active_subcommand
            .get(&step.tab)
            .map(String::as_str)
            .unwrap_or_default(),
        step.subcommand,
        "{}: expected the active subcommand to be {:?}",
        step.stem,
        step.subcommand
    );

    // (2) apply the drives.
    for drive in step.drives {
        apply_drive(&mut h, step, drive, chain);
    }

    // (3) driven-field visibility (SPEC §5.4): each driven field must intersect
    //     the viewport in AT LEAST ONE captured offset (base + recorded
    //     scrolls) — m4 extends the pilot's base-only check. Skipped for
    //     `capture: false` transcript-only steps (no shot → visibility moot).
    if step.capture {
        assert_driven_fields_visible(&mut h, step);
    }

    // (3.5) reveal marker (P1.4 RULING 1): actuate the LAST-driven secret field's
    //       eye (AccessKit Click = the LATCH arm) so the `-form` teaching shot
    //       shows the PUBLIC demo phrase. After the drives + visibility, BEFORE
    //       the filled-form hygiene checkpoint + capture. The Run click (step 6)
    //       auto-hides it, so the `-modal`/`-run` shots stay masked.
    if step.reveal {
        reveal_last_secret_field(&mut h, step);
    }

    // (4) secret-hygiene pre-run guards (SPEC §7): masked-by-construction +
    //     whole-tree no-plaintext for every secret value driven (held for
    //     `capture: false` secret steps too — masking must hold with or without
    //     a shot). PARAMETERIZED at the FILLED-FORM checkpoint (RULING 2): for a
    //     reveal-marked step the revealed field's OWN allowlisted value is
    //     permitted HERE (the deliberate teaching reveal); every OTHER secret
    //     stays strict (word-disjointness of S0/S1/S2 keeps the other probes
    //     from false-matching the revealed text). The populated-pane (step 7) and
    //     confirm-modal (`run_via_modal`) checkpoints stay UNCONDITIONALLY strict.
    if step.is_secret() {
        assert!(
            has_mask_sentinel(&h),
            "{}: a secret step must render the •••• mask sentinel before Run",
            step.stem
        );
        let revealed = step.revealed_value();
        for val in step.drives.iter().filter_map(Drive::secret_value) {
            if Some(val) == revealed {
                continue; // the deliberately-revealed field's own allowlisted value
            }
            assert_no_plaintext(&h, val, &format!("{} filled form", step.stem));
            if let Some(word) = val.split_whitespace().next() {
                assert_no_plaintext(&h, word, &format!("{} filled form (word probe)", step.stem));
            }
        }
    }

    // (5) the base filled-form shot (+ recorded scroll offsets → -formN). A
    //     `capture: false` step skips PNG capture but still runs + transcripts.
    if step.capture {
        snapshot(&mut h, &format!("{}-form", step.stem), opts, failures);
        for (i, &delta) in step.scroll.iter().enumerate() {
            wheel_scroll_form(&mut h, delta);
            snapshot(&mut h, &format!("{}-form{}", step.stem, i + 2), opts, failures);
        }
        // Restore the base scroll offset before the Run so the `-run` pane is
        // captured at a stable, top-anchored position regardless of the
        // `-formN` scroll excursions above.
        if !step.scroll.is_empty() {
            reset_scroll(&mut h);
        }
    }

    // (6) Run (if any) — direct one-click, or the secret-confirm two-click.
    if !step.runs {
        return;
    }
    if step.secret_modal {
        run_via_modal(&mut h, step, opts, failures);
    } else {
        run_direct(&mut h);
    }

    // (7) post-run assertions + transcript persistence.
    {
        let run = h.state().last_run.as_ref().expect("last_run after a Run");
        assert_eq!(
            run.argv.first().map(String::as_str),
            Some(step.tab.bin_name()),
            "{}: argv[0] must be the bare CLI name",
            step.stem
        );
        assert_eq!(
            run.exit_code, step.expect_exit,
            "{}: exit code mismatch; argv={:?} stderr={}",
            step.stem,
            run.argv,
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            !run.stderr.is_empty(),
            step.expect_stderr,
            "{}: expect_stderr={} but the run's stderr was {} — the manifest declaration \
             and the census must match reality",
            step.stem,
            step.expect_stderr,
            if run.stderr.is_empty() { "empty" } else { "non-empty" }
        );
        // Every secret value that reached argv (cleartext, as spawned) must
        // carry its display-mask bit (SPEC §7 — masked argv echo).
        for val in step.drives.iter().filter_map(Drive::secret_value) {
            if let Some(idx) = run.argv.iter().position(|t| t.contains(val)) {
                assert!(
                    run.mask[idx],
                    "{}: the secret argv token must be display-masked",
                    step.stem
                );
            }
        }
    }
    if step.is_secret() {
        for val in step.drives.iter().filter_map(Drive::secret_value) {
            if let Some(word) = val.split_whitespace().next() {
                assert_no_plaintext(&h, word, &format!("{} populated pane", step.stem));
            }
        }
    }

    // Capture the chain payload: parse this run's md1 chunks and store them
    // keyed by the step stem, so a LATER step's `TypeMd1Chain { chain_from }`
    // can type them into the `--md1` rows (SPEC §3.1b). Every run contributes
    // (empty for non-bundle runs) — the sink names its source stem.
    {
        let run = h.state().last_run.as_ref().expect("last_run");
        chain.insert(step.stem, tutorial::parse_md1_chunks(&run.stdout));
    }

    // The populated-pane shot ↔ its transcript, from the SAME RunResult. A
    // `capture: false` transcript-only step skips the PNG but still byte-gates
    // its transcript.
    if step.capture {
        snapshot(&mut h, &format!("{}-run", step.stem), opts, failures);
    }
    if let Err(e) = persist_transcripts(&h, step, update) {
        failures.push(e);
    }
}

/// One-click (non-secret) Run — ruling-9 single-`step()` semantics.
fn run_direct(h: &mut Harness<'static, MnemonicGuiApp>) {
    assert!(h.state().last_run.is_none(), "pre-click: no run yet");
    h.get_by_label("Run").click();
    assert!(
        h.state().last_run.is_none(),
        "click queued but not yet stepped: last_run must still be None (proves the \
         same-frame assertion is not vacuous)"
    );
    step_once_same_frame(h, "direct run");
    h.run();
}

/// Two-click secret Run: Run → confirm modal (deferred) → modal-Run.
fn run_via_modal(
    h: &mut Harness<'static, MnemonicGuiApp>,
    step: &Step,
    opts: &SnapshotOptions,
    failures: &mut Vec<String>,
) {
    assert!(h.state().last_run.is_none(), "pre-click: no run yet");
    h.get_by_label("Run").click();
    h.step(); // ONE frame: the click + the modal render
    assert!(
        h.state().last_run.is_none(),
        "{}: the first Run click on a secret form must DEFER to the confirm modal, not spawn",
        step.stem
    );
    h.run(); // settle (modal open)
    assert!(
        h.query_all_by_label("Confirm secret-bearing run").next().is_some(),
        "{}: the confirm modal must be visible after the first Run click",
        step.stem
    );
    // Label-collision demo → window-subtree scoping (the ratified discipline).
    let runs: Vec<Node<'_>> = h.query_all(by().role(Role::Button).label("Run")).collect();
    assert_eq!(
        runs.len(),
        2,
        "{}: expected exactly 2 Run buttons with the modal open (action bar + modal)",
        step.stem
    );
    // The modal's own masked token list carries no plaintext (SPEC §7(b)).
    for val in step.drives.iter().filter_map(Drive::secret_value) {
        if let Some(word) = val.split_whitespace().next() {
            assert_no_plaintext(h, word, &format!("{} confirm modal", step.stem));
        }
    }
    if step.capture && step.modal_shot {
        snapshot(h, &format!("{}-modal", step.stem), opts, failures);
    }
    // Modal-Run — single-`step()` semantics through the modal path.
    let modal_run = modal_scoped_run_button(h);
    modal_run.click();
    step_once_same_frame(h, &format!("{} (modal path)", step.stem));
    h.run();
}

/// P1.4 reveal marker (RULING 1): actuate the LAST-driven secret field's reveal
/// (👁) eye via an AccessKit `Click` (the LATCH arm — the only harness-drivable
/// reveal path). Under the single-revealed-field invariant exactly ONE field
/// reveals; the eye persists (no blur — the AccessKit click does not steal the
/// field's focus, verified) until the Run click auto-hides it. The target is the
/// last drive whose `secret_value()` is `Some` — a masked slot row (its eye is
/// the leftmost Button on the row) or the composite value field (its eye is the
/// `👁`-labelled Button on the flag row).
fn reveal_last_secret_field(h: &mut Harness<'static, MnemonicGuiApp>, step: &Step) {
    let last = step
        .drives
        .iter()
        .rev()
        .find(|d| d.secret_value().is_some())
        .unwrap_or_else(|| panic!("{}: reveal-marked step has no secret-value drive", step.stem));
    {
        let eye = match *last {
            Drive::TypeSlot { anchor, occ, .. } => {
                // Slot row layout: `@ N . combo = 👁 field ✕` — the eye is the
                // leftmost Button; assert the label so a layout drift fails loud.
                let btn = slot_row_widget(h, anchor, occ, Role::Button);
                assert_eq!(
                    btn.label().as_deref(),
                    Some(REVEAL_EYE_GLYPH),
                    "{}: the slot row's leftmost Button must be the reveal eye",
                    step.stem
                );
                btn
            }
            Drive::TypeComposite { flag, .. } => {
                on_row_of_opt(h, flag, Role::Button, REVEAL_EYE_GLYPH).unwrap_or_else(|| {
                    panic!("{}: no reveal eye (👁) on the {flag} composite row", step.stem)
                })
            }
            other => panic!(
                "{}: reveal marker on an unsupported drive {other:?} — extend reveal_last_secret_field",
                step.stem
            ),
        };
        eye.click();
    }
    h.run();
    assert!(
        revealed_field(&h.ctx).is_some(),
        "{}: the reveal marker must latch a revealed secret field",
        step.stem
    );
}

/// Interpret one drive against the whole window — the additive P1.5 interpreter
/// arms over the P1.4 slot spine. Each reuses the proven AccessKit-lookup
/// discipline (row-anchored on the flag-name label for scalar flags; the
/// `render_repeating` "+ add" for md1 rows).
fn apply_drive(
    h: &mut Harness<'static, MnemonicGuiApp>,
    step: &Step,
    drive: &Drive,
    chain: &ChainStore,
) {
    match *drive {
        Drive::FlipSlotSubkey { anchor, occ, to } => {
            slot_row_widget(h, anchor, occ, Role::ComboBox).click();
            h.run();
            h.get_by_label(to.as_str()).click();
            h.run();
            close_popup(h);
            let row = &h.state().form_state[&step.form_key()].slots.rows[occ];
            assert_eq!(
                row.subkey, to,
                "{}: the slot row subkey must flip to {:?} via the popup drive",
                step.stem, to
            );
        }
        Drive::TypeSlot {
            anchor,
            occ,
            subkey,
            value,
        } => {
            let role = slot_value_role(drive);
            slot_row_widget(h, anchor, occ, role).type_text(value);
            h.run();
            h.run(); // settle write-back (buffer lands at frame end)
            let row = &h.state().form_state[&step.form_key()].slots.rows[occ];
            assert_eq!(
                row.subkey, subkey,
                "{}: TypeSlot expected the row to be on subkey {:?}",
                step.stem, subkey
            );
            assert_eq!(
                row.value, value,
                "{}: the typed value must land in the slot row",
                step.stem
            );
        }
        Drive::AddSlots { count } => {
            for _ in 0..count {
                h.get_by_label("+ Add slot").click();
                h.run();
            }
        }
        Drive::SetSlotIndex { occ, index } => {
            let id = slot_row_widget(h, "@", occ, Role::SpinButton).id();
            set_value_action(h, id, index);
            let got = h.state().form_state[&step.form_key()].slots.rows[occ].index;
            assert_eq!(
                got as i64, index,
                "{}: slot row {occ} index must be {index}",
                step.stem
            );
        }
        Drive::SelectDropdown { flag, value } => {
            // Row-anchored open (a per-flag combo is unlabeled — the flag-name
            // label sits to its left). Guard the open (egui ComboBox does not
            // auto-close on a `selectable_value` click).
            let display = if value.is_empty() { "(none)" } else { value };
            if h.query_all_by_label(display).next().is_none() {
                on_row_of(h, flag, Role::ComboBox, 0).click();
                h.run();
                h.run();
            }
            h.get_by_label(display).click();
            h.run();
            close_popup(h);
            assert_eq!(
                h.state().form_state[&step.form_key()].dropdown_value(flag),
                Some(value),
                "{}: SelectDropdown {flag} must land {value:?}",
                step.stem
            );
        }
        Drive::SelectRowDropdown {
            flag,
            next_flag,
            value,
        } => {
            if h.query_all_by_label(value).next().is_none() {
                block_row_combo(h, flag, next_flag)
                    .unwrap_or_else(|| panic!("{}: no row combo in the {flag} block", step.stem))
                    .click();
                h.run();
                h.run();
            }
            h.get_by_label(value).click();
            h.run();
            close_popup(h);
        }
        Drive::ToggleBoolean { flag } => {
            on_row_of(h, flag, Role::CheckBox, 0).click();
            h.run();
            h.run();
            let form = &h.state().form_state[&step.form_key()];
            assert!(
                form.has_value(flag),
                "{}: ToggleBoolean {flag} must set the flag true",
                step.stem
            );
        }
        Drive::TypeText { flag, value } => {
            type_into_text_flag(h, flag, value);
            assert_eq!(
                h.state().form_state[&step.form_key()].text_value(flag),
                Some(value),
                "{}: TypeText {flag} value must land",
                step.stem
            );
        }
        Drive::TypeTextFromFixture { flag, fixture } => {
            // cwd is the fixture dir (SPEC §6.6); read the committed watch-only
            // descriptor and type it as `--descriptor` TEXT (the F2 route-around
            // — ballooned argv echo is viewport-faithful, F7).
            let text = std::fs::read_to_string(fixture)
                .unwrap_or_else(|e| panic!("{}: read fixture {fixture:?}: {e}", step.stem));
            let text = text.trim_end_matches(['\n', '\r']);
            type_into_text_flag(h, flag, text);
            assert_eq!(
                h.state().form_state[&step.form_key()].text_value(flag),
                Some(text),
                "{}: TypeTextFromFixture {flag} value must land",
                step.stem
            );
        }
        Drive::TypePath { flag, value } => {
            on_row_of(h, flag, Role::TextInput, 0).type_text(value);
            h.run();
            h.run();
        }
        Drive::TypeComposite { flag, node, value } => {
            // Select the node type in the composite combo (row-anchored), then
            // type the value (masked PasswordInput for a secret node such as
            // `phrase`; plain TextInput for a public node).
            if h.query_all_by_label(node).next().is_none() {
                on_row_of(h, flag, Role::ComboBox, 0).click();
                h.run();
                h.run();
            }
            h.get_by_label(node).click();
            h.run();
            close_popup(h);
            let role = if drive.secret_value().is_some() {
                Role::PasswordInput
            } else {
                Role::TextInput
            };
            on_row_of(h, flag, role, 0).type_text(value);
            h.run();
            h.run();
            let form = &h.state().form_state[&step.form_key()];
            assert_eq!(
                form.composite_node(flag),
                Some(node),
                "{}: TypeComposite {flag} node must land {node:?}",
                step.stem
            );
            assert_eq!(
                composite_value(form, flag).as_deref(),
                Some(value),
                "{}: TypeComposite {flag} value must land",
                step.stem
            );
        }
        Drive::SetNumber { flag, value } => {
            set_number_flag(h, flag, value);
            assert_eq!(
                h.state().form_state[&step.form_key()].number_value(flag),
                Some(value),
                "{}: SetNumber {flag} must land {value}",
                step.stem
            );
        }
        Drive::TypeMd1Chain { flag, chain_from } => {
            let chunks = chain.get(chain_from).unwrap_or_else(|| {
                panic!(
                    "{}: TypeMd1Chain source {chain_from:?} has no captured md1 chunks — \
                     the chain-source step must run BEFORE this step",
                    step.stem
                )
            });
            assert!(
                !chunks.is_empty(),
                "{}: chain source {chain_from:?} produced ZERO md1 chunks",
                step.stem
            );
            for (i, chunk) in chunks.iter().enumerate() {
                // Append a row via the repeating-flag header "+ add", then type
                // the chunk into the newly-created (last) row.
                add_row_button_of(h, flag).click();
                h.run();
                h.run();
                let node = md1_row_inputs(h).into_iter().last().unwrap_or_else(|| {
                    panic!("{}: no --md1 row TextInput after + add", step.stem)
                });
                node.type_text(chunk);
                h.run();
                h.run();
                let rows: Vec<String> = md1_row_values(h, flag);
                assert_eq!(
                    rows.get(i).map(String::as_str),
                    Some(chunk.as_str()),
                    "{}: md1 chunk {i} did not land in its row",
                    step.stem
                );
            }
            let rows = md1_row_values(h, flag);
            assert_eq!(
                rows.len(),
                chunks.len(),
                "{}: expected {} --md1 rows, got {}",
                step.stem,
                chunks.len(),
                rows.len()
            );
        }
    }
}

/// Type into a scalar `Text`/`Path` flag's input (row-anchored on the flag-name
/// label). Two `run()`s settle the buffer write-back.
fn type_into_text_flag(h: &mut Harness<'static, MnemonicGuiApp>, flag: &'static str, value: &str) {
    on_row_of(h, flag, Role::TextInput, 0).type_text(value);
    h.run();
    h.run();
}

/// Set a `Number` flag: click its `Set` affordance if present (initial state is
/// `Unset`), then push an AccessKit `SetValue` action onto the SpinButton (the
/// DragValue gotcha — kittest `Node` has no `SetValue` helper).
fn set_number_flag(h: &mut Harness<'static, MnemonicGuiApp>, flag: &'static str, value: i64) {
    if let Some(set) = on_row_of_opt(h, flag, Role::Button, "Set") {
        set.click();
        h.run();
        h.run();
    }
    let id = on_row_of(h, flag, Role::SpinButton, 0).id();
    set_value_action(h, id, value);
}

/// Push an AccessKit `SetValue` numeric action at `id` (the DragValue gotcha —
/// kittest `Node` exposes no `SetValue` helper). Read `id` BEFORE `input_mut()`.
fn set_value_action(h: &mut Harness<'static, MnemonicGuiApp>, id: egui::accesskit::NodeId, value: i64) {
    h.input_mut()
        .events
        .push(egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::SetValue,
                target: id,
                data: Some(egui::accesskit::ActionData::NumericValue(value as f64)),
            },
        ));
    h.run();
    h.run();
}

/// The `+ add` button belonging to `flag`'s repeating-row header (row-anchored
/// on the flag-name label so the correct header is picked among several).
fn add_row_button_of<'t>(
    h: &'t Harness<'static, MnemonicGuiApp>,
    flag: &'static str,
) -> Node<'t> {
    on_row_of_opt(h, flag, Role::Button, "+ add")
        .unwrap_or_else(|| panic!("no '+ add' button on the {flag} header row"))
}

/// The `--md1` repeating-row `TextInput` nodes, in top-to-bottom order. Bounded
/// below by the sibling `--cosigner` repeating flag (0 seeded rows) so no other
/// flag's input is mistaken for an md1 row.
fn md1_row_inputs<'t>(h: &'t Harness<'static, MnemonicGuiApp>) -> Vec<Node<'t>> {
    let top = label_y(h, "--md1").expect("--md1 header label present");
    let bottom = label_y(h, "--cosigner").unwrap_or(f64::INFINITY);
    let mut hits: Vec<Node<'t>> = h
        .query_all(by().role(Role::TextInput))
        .filter(|n| {
            n.raw_bounds()
                .map(|r| {
                    let c = (r.y0 + r.y1) / 2.0;
                    c > top && c < bottom
                })
                .unwrap_or(false)
        })
        .collect();
    hits.sort_by(|a, b| {
        a.raw_bounds()
            .unwrap()
            .y0
            .partial_cmp(&b.raw_bounds().unwrap().y0)
            .unwrap()
    });
    hits
}

/// The first row `ComboBox` of a repeating Dropdown flag (e.g. convert `--to`),
/// sitting BELOW the `flag` header and ABOVE the `next_flag` label.
fn block_row_combo<'t>(
    h: &'t Harness<'static, MnemonicGuiApp>,
    flag: &'static str,
    next_flag: &'static str,
) -> Option<Node<'t>> {
    let top = label_y(h, flag)?;
    let bottom = label_y(h, next_flag).unwrap_or(f64::INFINITY);
    let mut hits: Vec<Node<'t>> = h
        .query_all(by().role(Role::ComboBox))
        .filter(|n| {
            n.raw_bounds()
                .map(|r| {
                    let c = (r.y0 + r.y1) / 2.0;
                    c > top && c < bottom
                })
                .unwrap_or(false)
        })
        .collect();
    hits.sort_by(|a, b| {
        a.raw_bounds()
            .unwrap()
            .y0
            .partial_cmp(&b.raw_bounds().unwrap().y0)
            .unwrap()
    });
    hits.into_iter().next()
}

/// The current `--md1` row values from form-state (assertion oracle).
fn md1_row_values(h: &Harness<'static, MnemonicGuiApp>, flag: &str) -> Vec<String> {
    h.state()
        .form_state
        .values()
        .find_map(|f| {
            let rows: Vec<String> = f
                .values
                .iter()
                .filter(|(k, _)| k == flag)
                .filter_map(|(_, v)| match v {
                    mnemonic_gui::schema::FlagValue::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();
            if rows.is_empty() {
                None
            } else {
                Some(rows)
            }
        })
        .unwrap_or_default()
}

/// The value string of a `NodeValueComposite` flag from form-state.
fn composite_value(form: &mnemonic_gui::schema::FormState, flag: &str) -> Option<String> {
    form.values.iter().find_map(|(k, v)| match v {
        mnemonic_gui::schema::FlagValue::NodeValueComposite { value, .. } if k == flag => {
            Some(value.clone())
        }
        _ => None,
    })
}

/// The y-center of a label node (first match), if present.
fn label_y(h: &Harness<'static, MnemonicGuiApp>, label: &str) -> Option<f64> {
    h.query_all_by_label(label)
        .next()
        .and_then(|n| n.raw_bounds())
        .map(|r| (r.y0 + r.y1) / 2.0)
}

/// Like [`on_row_of`] but returns `None` instead of panicking, and matches a
/// widget of `role` on the anchor's row that ALSO carries `want_label`.
fn on_row_of_opt<'t>(
    h: &'t Harness<'static, MnemonicGuiApp>,
    anchor: &'static str,
    role: Role,
    want_label: &'static str,
) -> Option<Node<'t>> {
    let a = h.query_all_by_label(anchor).next()?;
    let (ax0, ay0, _ax1, ay1) = rect_of(&a);
    h.query_all(by().role(role).label(want_label)).find(|n| {
        n.raw_bounds()
            .map(|r| {
                let c = (r.y0 + r.y1) / 2.0;
                c >= ay0 - 4.0 && c <= ay1 + 4.0 && r.x0 >= ax0
            })
            .unwrap_or(false)
    })
}

/// SPEC §5.4 driven-field visibility (m4): each drive's representative field
/// must intersect the viewport in AT LEAST ONE captured offset. Checks the base
/// offset first; if clipped, wheel-scrolls through each recorded offset looking
/// for it. Restores the base offset afterwards. Slot value drives use the slot
/// row rect; scalar/repeating flag drives use the flag-name label rect (which
/// sits on the field's row; for the `--md1` chain it is the header, whose
/// visibility means "the user sees WHERE chunks are entered" — the full chunk
/// list rides the gated transcript, §5.4 viewport-faithful contract).
fn assert_driven_fields_visible(h: &mut Harness<'static, MnemonicGuiApp>, step: &Step) {
    for drive in step.drives {
        let visible_at_base = driven_field_visible(h, drive);
        if visible_at_base {
            continue;
        }
        // Not visible at base — scan the recorded scroll offsets.
        let mut found = false;
        for &delta in step.scroll {
            wheel_scroll_form(h, delta);
            if driven_field_visible(h, drive) {
                found = true;
                break;
            }
        }
        if !step.scroll.is_empty() {
            reset_scroll(h);
        }
        assert!(
            found,
            "{}: driven field for {drive:?} is clipped out of EVERY captured offset \
             (SPEC §5.4 driven-field-visibility) — add a scroll offset",
            step.stem
        );
    }
}

/// True iff the drive's representative field rect intersects the viewport.
fn driven_field_visible(h: &Harness<'static, MnemonicGuiApp>, drive: &Drive) -> bool {
    let rect = if let Some((anchor, occ)) = drive.slot_target() {
        let role = slot_value_role(drive);
        slot_row_widget(h, anchor, occ, role).raw_bounds()
    } else if let Some(label) = drive.label_anchor() {
        h.query_all_by_label(label).next().and_then(|n| n.raw_bounds())
    } else {
        // AddSlots / FlipSlotSubkey: no distinct field to keep visible.
        return true;
    };
    match rect {
        Some(r) => r.y1 > 0.0 && r.y0 < WINDOW_SIZE[1] as f64,
        None => false,
    }
}

/// Reset the central form `ScrollArea` to the top by wheel-scrolling up by a
/// large delta (deterministic clamp at offset 0).
fn reset_scroll(h: &mut Harness<'static, MnemonicGuiApp>) {
    wheel_scroll_form(h, 100_000.0);
}

/// The expected value-editor role for a slot drive: `PasswordInput` for a
/// secret subkey (masked-by-construction), `TextInput` for a public one.
fn slot_value_role(drive: &Drive) -> Role {
    match drive.secret_value() {
        Some(_) => Role::PasswordInput,
        None => Role::TextInput,
    }
}

// ─── the 5-rule lookup discipline (spike-ratified; authored fresh) ───────────

/// Injected AppState — all four CLIs `Found` (SPEC §6 item 3: no
/// `$PATH`-dependent tab grey-out). The display paths are never rendered;
/// `spawn_and_capture` re-probes the real `$PATH` at click time regardless.
fn fixed_appstate_all_found() -> AppState {
    AppState {
        mnemonic_detect: Detected::Found(PathBuf::from("/pinned/mnemonic")),
        md_detect: Detected::Found(PathBuf::from("/pinned/md")),
        ms_detect: Detected::Found(PathBuf::from("/pinned/ms")),
        mk_detect: Detected::Found(PathBuf::from("/pinned/mk")),
        active_tab: CliTab::Mnemonic,
    }
}

/// The whole-window harness over the REAL app (`new_headless`, no persistence).
/// `with_max_steps(64)` — the default 4 is too tight for the smooth-scroll
/// animation (ruling 2); `run()` still settles deterministically.
fn app_harness() -> Harness<'static, MnemonicGuiApp> {
    let app = MnemonicGuiApp::new_headless(fixed_appstate_all_found(), None, None);
    Harness::builder()
        .with_size(egui::Vec2::new(WINDOW_SIZE[0], WINDOW_SIZE[1]))
        .with_pixels_per_point(PPP)
        .with_max_steps(64)
        .build_state(|ctx, app: &mut MnemonicGuiApp| app.ui(ctx), app)
}

fn rect_of(n: &Node<'_>) -> (f64, f64, f64, f64) {
    let r = n.raw_bounds().expect("node has no bounds");
    (r.x0, r.y0, r.x1, r.y1)
}

/// Rule 2 — row-anchored geometric lookup for unlabelled inputs: the widget of
/// `role` on the same horizontal band as the exact-label `anchor`, `occ`-th
/// from the left. Deterministic under the fixed window size + ppp.
fn on_row_of<'t>(
    h: &'t Harness<'static, MnemonicGuiApp>,
    anchor: &'static str,
    role: Role,
    occ: usize,
) -> Node<'t> {
    let a = h
        .query_all_by_label(anchor)
        .next()
        .unwrap_or_else(|| panic!("row anchor label {anchor:?} not found"));
    let (ax0, ay0, _ax1, ay1) = rect_of(&a);
    let mut hits: Vec<Node<'t>> = h
        .query_all(by().role(role))
        .filter(|n| {
            let Some(r) = n.raw_bounds() else {
                return false;
            };
            let c = (r.y0 + r.y1) / 2.0;
            c >= ay0 - 2.0 && c <= ay1 + 2.0 && r.x0 >= ax0
        })
        .collect();
    hits.sort_by(|a, b| {
        a.raw_bounds()
            .unwrap()
            .x0
            .partial_cmp(&b.raw_bounds().unwrap().x0)
            .unwrap()
    });
    *hits.get(occ).unwrap_or_else(|| {
        panic!(
            "on_row_of({anchor:?}, {role:?}, occ={occ}): only {} match(es) on the row",
            hits.len()
        )
    })
}

/// Slot-row lookup: the widget of `role` on the `occ`-th slot row. Each slot
/// row carries exactly ONE `anchor` (`@`) label, so rows are distinguished by
/// the `occ`-th `anchor` (top-to-bottom), THEN the first `role` widget on that
/// row (`on_row_of` anchors on the FIRST label only, which collapses multi-row
/// slot grids — this variant selects the row first).
fn slot_row_widget<'t>(
    h: &'t Harness<'static, MnemonicGuiApp>,
    anchor: &'static str,
    occ: usize,
    role: Role,
) -> Node<'t> {
    let mut anchors: Vec<Node<'t>> = h.query_all_by_label(anchor).collect();
    anchors.sort_by(|a, b| {
        a.raw_bounds()
            .unwrap()
            .y0
            .partial_cmp(&b.raw_bounds().unwrap().y0)
            .unwrap()
    });
    let a = anchors.get(occ).unwrap_or_else(|| {
        panic!(
            "slot_row_widget({anchor:?}, occ={occ}, {role:?}): only {} slot row(s)",
            anchors.len()
        )
    });
    let (ax0, ay0, _ax1, ay1) = rect_of(a);
    let mut hits: Vec<Node<'t>> = h
        .query_all(by().role(role))
        .filter(|n| {
            n.raw_bounds()
                .map(|r| {
                    let c = (r.y0 + r.y1) / 2.0;
                    c >= ay0 - 2.0 && c <= ay1 + 2.0 && r.x0 >= ax0
                })
                .unwrap_or(false)
        })
        .collect();
    hits.sort_by(|a, b| {
        a.raw_bounds()
            .unwrap()
            .x0
            .partial_cmp(&b.raw_bounds().unwrap().x0)
            .unwrap()
    });
    hits.into_iter().next().unwrap_or_else(|| {
        panic!("slot_row_widget({anchor:?}, occ={occ}, {role:?}): no {role:?} on row {occ}")
    })
}

/// Rule 5 — Escape after every AccessKit popup option click (an AccessKit
/// `Action::Click` has no pointer, so `clicked_elsewhere()` never fires and the
/// popup would linger into the next shot; egui closes any popup on Escape).
fn close_popup(h: &mut Harness<'static, MnemonicGuiApp>) {
    h.press_key(egui::Key::Escape);
    h.run();
}

/// Rule 1 — select a subcommand through the real ComboBox popup (the combo is
/// unique by role+label "subcommand"; option rows by human_name), then Escape.
fn combo_select_subcommand(
    h: &mut Harness<'static, MnemonicGuiApp>,
    human_name: &'static str,
    expect_name: &str,
) {
    h.get_by_role_and_label(Role::ComboBox, "subcommand").click();
    h.run();
    h.get_by_label(human_name).click();
    h.run();
    close_popup(h);
    assert_eq!(
        h.state()
            .active_subcommand
            .get(&CliTab::Mnemonic)
            .map(String::as_str)
            .unwrap_or_default(),
        expect_name,
        "the combo popup drive must land the subcommand selection"
    );
}

/// Rule 4 — scope the Run-button query to the modal Window subtree.
fn modal_scoped_run_button<'t>(h: &'t Harness<'static, MnemonicGuiApp>) -> Node<'t> {
    let window = h
        .query_all(by().role(Role::Window).label("Confirm secret-bearing run"))
        .next()
        .unwrap_or_else(|| panic!("modal Window node not found by role+title"));
    window
        .query_all(by().role(Role::Button).label("Run"))
        .next()
        .expect("Run button inside the modal subtree")
}

/// Ruling 2 — injected `PointerMoved` + `MouseWheel{unit: Point}` + `run()`.
/// (Referenced by the executor's scroll loop; pilots record no offsets, so it
/// is exercised first by P1.5's tall steps.)
fn wheel_scroll_form(h: &mut Harness<'static, MnemonicGuiApp>, delta_y: f32) {
    // Hover a point inside the central form scroll region. Prefer the first
    // flag-name anchor; fall back to the window center.
    let pos = h
        .query_all(by().role(Role::ComboBox).label("subcommand"))
        .next()
        .and_then(|n| n.raw_bounds())
        .map(|r| {
            egui::pos2(
                ((r.x0 + r.x1) / 2.0) as f32,
                ((r.y0 + r.y1) / 2.0 + 120.0) as f32,
            )
        })
        .unwrap_or_else(|| egui::pos2(WINDOW_SIZE[0] / 2.0, WINDOW_SIZE[1] / 2.0));
    h.input_mut().events.push(egui::Event::PointerMoved(pos));
    h.input_mut().events.push(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, delta_y),
        modifiers: egui::Modifiers::default(),
    });
    h.run();
}

// ─── the two named per-step gates + secret guards ────────────────────────────

/// SAME-FRAME-COMPLETION (SPEC §3.1b / §6.5, ruling 9): deliver the already
/// queued click in exactly ONE `harness.step()`, then assert the run landed
/// BEFORE any further stepping. An async runner fails HERE, by name.
fn step_once_same_frame(h: &mut Harness<'static, MnemonicGuiApp>, what: &str) {
    h.step(); // exactly ONE frame: the click frame
    tutorial::same_frame_completion(h.state().last_run.is_some(), what)
        .unwrap_or_else(|e| panic!("{e}"));
}

/// True iff any node in the tree renders the `••••` mask sentinel.
fn has_mask_sentinel(h: &Harness<'static, MnemonicGuiApp>) -> bool {
    h.query_all_by(|n: &Node<'_>| {
        n.label().map(|l| l.contains("••••")).unwrap_or(false)
            || n.value().map(|v| v.contains("••••")).unwrap_or(false)
    })
    .next()
    .is_some()
}

/// Whole-tree no-plaintext assertion (SPEC §7): no AccessKit node label OR
/// value may contain the secret substring. egui masks password values before
/// AccessKit, so this is global with zero exclusions.
fn assert_no_plaintext(h: &Harness<'static, MnemonicGuiApp>, needle: &str, ctx_msg: &str) {
    let needle = needle.to_string();
    let hits: Vec<String> = h
        .query_all_by(move |n: &Node<'_>| {
            n.label().map(|l| l.contains(&needle)).unwrap_or(false)
                || n.value().map(|v| v.contains(&needle)).unwrap_or(false)
        })
        .map(|n| format!("{:?} {:?}/{:?}", n.role(), n.label(), n.value()))
        .collect();
    assert!(
        hits.is_empty(),
        "no-plaintext violation ({ctx_msg}): {} node(s) expose the secret:\n{hits:#?}",
        hits.len()
    );
}

// ─── snapshots + transcripts ─────────────────────────────────────────────────

fn snapshot(
    h: &mut Harness<'static, MnemonicGuiApp>,
    name: &str,
    opts: &SnapshotOptions,
    failures: &mut Vec<String>,
) {
    if let Err(e) = h.try_snapshot_options(name, opts) {
        failures.push(format!("{name}: {e}"));
    }
}

/// Byte-persist the RunResult transcripts (SPEC §3.1b): `<stem>.stdout.txt` +
/// `<stem>.exit.txt` always; `<stem>.stderr.txt` iff non-empty. Update mode
/// (`UPDATE_SNAPSHOTS`) writes; else byte-compares against the committed corpus
/// and records a `.new`-written failure on drift.
fn persist_transcripts(
    h: &Harness<'static, MnemonicGuiApp>,
    step: &Step,
    update: bool,
) -> Result<(), String> {
    let run = h.state().last_run.as_ref().expect("last_run");
    let dir = manifest_path(TUTORIAL_SNAPSHOT_DIR);
    let exit_txt = match run.exit_code {
        Some(n) => format!("{n}\n"),
        None => "signal\n".to_string(),
    };
    let mut items: Vec<(String, Vec<u8>)> = vec![
        (format!("{}.stdout.txt", step.stem), run.stdout.clone()),
        (format!("{}.exit.txt", step.stem), exit_txt.into_bytes()),
    ];
    if !run.stderr.is_empty() {
        items.push((format!("{}.stderr.txt", step.stem), run.stderr.clone()));
    }
    let mut drift: Vec<String> = Vec::new();
    for (name, bytes) in items {
        let path = dir.join(&name);
        if update {
            std::fs::write(&path, &bytes).map_err(|e| format!("write {name}: {e}"))?;
            continue;
        }
        match std::fs::read(&path) {
            Ok(committed) if committed == bytes => {}
            Ok(_) => {
                let newp = dir.join(format!("{name}.new"));
                let _ = std::fs::write(&newp, &bytes);
                drift.push(format!("{name}: byte-diff vs committed (wrote {name}.new)"));
            }
            Err(e) => {
                let newp = dir.join(format!("{name}.new"));
                let _ = std::fs::write(&newp, &bytes);
                drift.push(format!("{name}: missing committed transcript ({e}); wrote {name}.new"));
            }
        }
    }
    if drift.is_empty() {
        Ok(())
    } else {
        Err(drift.join("\n"))
    }
}

// ─── shared preflight (adapter + version gate) ───────────────────────────────

/// Renders MUST come from a software rasterizer (`device_type == Cpu`) and
/// honor `WGPU_BACKEND` when set (the gui_form_snapshots A1 pattern).
fn adapter_guard() {
    let info = {
        let render_state =
            egui_kittest::wgpu::create_render_state(egui_kittest::wgpu::default_wgpu_setup());
        render_state.adapter.get_info()
    };
    eprintln!("TUTORIAL-ADAPTER: {info:?}");
    assert_eq!(
        info.device_type,
        eframe::wgpu::DeviceType::Cpu,
        "GUI_TUTORIAL_SNAPSHOTS renders MUST come from a software rasterizer \
         (device_type Cpu) — got {info:?}. Use lavapipe (mesa-vulkan-drivers + \
         WGPU_BACKEND=vulkan) or llvmpipe (WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1)."
    );
    if let Ok(env) = std::env::var("WGPU_BACKEND") {
        if let Some(expected) = expected_backend(&env) {
            assert_eq!(
                info.backend, expected,
                "adapter backend does not honor WGPU_BACKEND={env} — got {info:?}"
            );
        }
    }
}

/// `pinned-tier-version-gate` — BEFORE any render or spawn (SPEC §3.1b / §6 item
/// 4; the gen.sh:22 pattern). Probe EVERY manifest-spawned CLI's `--version`
/// against the pinned tier and HARD-fail on any mismatch, so a wrong-tier local
/// regen can never produce honest-looking corpus bytes.
fn run_pinned_tier_version_gate() {
    for cli in tutorial::spawned_clis() {
        let expected = expected_pinned_version(cli);
        let got = probe_version(cli);
        tutorial::version_matches(cli, &got, expected).unwrap_or_else(|e| panic!("{e}"));
        eprintln!("TUTORIAL-GATE-OK: {got}");
    }
}

// ─── SAME-FRAME-COMPLETION at the DIRECT (one-click, non-secret) click class ──

/// A world-known public bip84 account-0 xpub (S0-derived; watch-only material —
/// NOT a secret, so it drives through a plain `TextInput` and its Run is a
/// single click with no confirm modal).
const PUBLIC_BIP84_XPUB: &str =
    "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

/// Live demonstration of `SAME-FRAME-COMPLETION` at the **direct one-click**
/// class (J1 exercises the modal two-click class; the plan's P1.4 gate asks for
/// both). This is a harness GATE PROOF, not tutorial content: it drives a
/// reachable non-secret bundle run (a public xpub in the demo-seeded slot →
/// watch-only mode, no confirm modal), asserts the run lands in the click frame
/// via the SAME shared `step_once_same_frame` helper the corpus uses, and
/// captures NO snapshot / NO transcript (nothing enters the manifest-derived
/// census). Env-gated like the corpus harness (needs a rasterizer + the pinned
/// CLI; the version gate machine-guards the tier).
#[test]
fn same_frame_completion_direct_click_class() {
    if std::env::var("GUI_TUTORIAL_SNAPSHOTS").as_deref() != Ok("1") {
        eprintln!(
            "TUTORIAL-SKIP: GUI_TUTORIAL_SNAPSHOTS != 1 — direct-click SAME-FRAME demo skipped"
        );
        return;
    }
    adapter_guard();
    run_pinned_tier_version_gate();

    let mut h = app_harness();
    // Fresh app is on the demo-seeded mnemonic:bundle with one empty Xpub slot
    // row. Type a PUBLIC xpub into it (no subkey flip — it is already Xpub, a
    // watch-only public subkey → a plain TextInput, no secret modal).
    on_row_of(&h, "@", Role::TextInput, 0).type_text(PUBLIC_BIP84_XPUB);
    h.run();
    h.run();
    assert_eq!(
        h.state().form_state["mnemonic:bundle"].slots.rows[0].value,
        PUBLIC_BIP84_XPUB,
        "the public xpub must land in the seeded slot row"
    );

    // Direct one-click Run — the SAME single-`step()` tripwire the corpus uses.
    run_direct(&mut h);

    let run = h.state().last_run.as_ref().expect("last_run after a direct Run");
    assert!(
        h.query_all_by_label("Confirm secret-bearing run").next().is_none(),
        "a public (non-secret) bundle run must NOT raise the confirm modal — this is the \
         direct click class"
    );
    assert_eq!(run.exit_code, Some(0), "watch-only bundle must exit 0");
    assert_eq!(run.argv.first().map(String::as_str), Some("mnemonic"));
    eprintln!("TUTORIAL-SAMEFRAME-DIRECT: direct-click class holds (exit 0, no modal)");
}

// ─── version-gate plumbing ───────────────────────────────────────────────────

/// The expected `<cli> --version` string at the tutorial's pinned tier. For
/// `mnemonic` this is the schema constant that renders the window's `Pinned:`
/// line (`schema/mnemonic.rs`), so the gate machine-guards that label's honesty
/// in every shot. (md/ms/mk land in P1.5 with the first step that spawns them —
/// no tutorial journey does today; §5.3 is all `mnemonic <subcommand>`.)
fn expected_pinned_version(cli: &str) -> &'static str {
    match cli {
        "mnemonic" => mnemonic_gui::schema::mnemonic::SCHEMA.pinned_version,
        other => panic!(
            "pinned-tier-version-gate: no expected version wired for {other:?} — P1.5 wires \
             md/ms/mk if a step ever spawns them"
        ),
    }
}

fn probe_version(cli: &str) -> String {
    let out = std::process::Command::new(cli)
        .arg("--version")
        .output()
        .unwrap_or_else(|e| panic!("pinned-tier-version-gate: failed to spawn `{cli} --version`: {e}"));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Map a `WGPU_BACKEND` spelling to its single backend (skip the assert for
/// multi-backend/unknown spellings — the `device_type == Cpu` assert never is).
fn expected_backend(env: &str) -> Option<eframe::wgpu::Backend> {
    match env.to_ascii_lowercase().as_str() {
        "vulkan" | "vk" => Some(eframe::wgpu::Backend::Vulkan),
        "gl" | "opengl" | "gles" => Some(eframe::wgpu::Backend::Gl),
        "metal" | "mtl" => Some(eframe::wgpu::Backend::Metal),
        "dx12" | "d3d12" => Some(eframe::wgpu::Backend::Dx12),
        "webgpu" => Some(eframe::wgpu::Backend::BrowserWebGpu),
        _ => None,
    }
}
