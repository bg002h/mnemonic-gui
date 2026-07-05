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
//! **P1.4 scope = PILOTS ONLY** (Chapter-0 orientation + J1 single-sig = 4
//! shots). The machinery is generic over `tutorial::MANIFEST`; P1.5 grows the
//! corpus by extending the manifest, not this harness.

use std::path::{Path, PathBuf};

use egui::accesskit::Role;
use egui_kittest::kittest::{by, Node, Queryable};
use egui_kittest::{Harness, SnapshotOptions};

use mnemonic_gui::app::{AppState, CliTab};
use mnemonic_gui::app_window::MnemonicGuiApp;
use mnemonic_gui::path_detect::Detected;

mod tutorial;
use tutorial::{Drive, Step};

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
/// HARD corpus ceiling (SPEC §5.3 ruling 4).
const BUDGET_HARD_MIB: f64 = 20.0;
/// Corpus target (report-only above this).
const BUDGET_TARGET_MIB: f64 = 15.0;

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

    // ── cwd pinned to the fixture dir for the whole test (SPEC §6.6) ──
    let fixture_dir = manifest_path(FIXTURE_DIR);
    std::env::set_current_dir(&fixture_dir).expect("pin cwd to the fixture dir");

    let update = std::env::var("UPDATE_SNAPSHOTS").is_ok();
    let opts = SnapshotOptions::new().output_path(manifest_path(TUTORIAL_SNAPSHOT_DIR));
    let mut failures: Vec<String> = Vec::new();

    for step in tutorial::MANIFEST {
        execute_step(step, &opts, update, &mut failures);
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

// ─── the generic step executor ───────────────────────────────────────────────

fn execute_step(step: &Step, opts: &SnapshotOptions, update: bool, failures: &mut Vec<String>) {
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
        apply_drive(&mut h, step, drive);
    }

    // (3) driven-field visibility (SPEC §5.4): each driven slot row must
    //     intersect the viewport at the base offset (pilots are single-shot;
    //     P1.5 extends this across recorded scroll offsets).
    for drive in step.drives {
        if let Some((anchor, occ)) = drive.slot_target() {
            let role = slot_value_role(drive);
            let r = on_row_of(&h, anchor, role, occ)
                .raw_bounds()
                .expect("driven slot row has bounds");
            assert!(
                r.y1 > 0.0 && r.y0 < WINDOW_SIZE[1] as f64,
                "{}: driven slot row (anchor {anchor:?}) is clipped out of the base form \
                 shot (rect y0={} y1={}); SPEC §5.4 driven-field-visibility",
                step.stem,
                r.y0,
                r.y1
            );
        }
    }

    // (4) secret-hygiene pre-run guards (SPEC §7): masked-by-construction +
    //     whole-tree no-plaintext for every secret value driven.
    if step.is_secret() {
        assert!(
            has_mask_sentinel(&h),
            "{}: a secret step must render the •••• mask sentinel before Run",
            step.stem
        );
        for val in step.drives.iter().filter_map(Drive::secret_value) {
            assert_no_plaintext(&h, val, &format!("{} filled form", step.stem));
            if let Some(word) = val.split_whitespace().next() {
                assert_no_plaintext(&h, word, &format!("{} filled form (word probe)", step.stem));
            }
        }
    }

    // (5) the base filled-form shot (+ recorded scroll offsets → -formN).
    snapshot(&mut h, &format!("{}-form", step.stem), opts, failures);
    for (i, &delta) in step.scroll.iter().enumerate() {
        wheel_scroll_form(&mut h, delta);
        snapshot(&mut h, &format!("{}-form{}", step.stem, i + 2), opts, failures);
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

    // The populated-pane shot ↔ its transcript, from the SAME RunResult.
    snapshot(&mut h, &format!("{}-run", step.stem), opts, failures);
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
    if step.modal_shot {
        snapshot(h, &format!("{}-modal", step.stem), opts, failures);
    }
    // Modal-Run — single-`step()` semantics through the modal path.
    let modal_run = modal_scoped_run_button(h);
    modal_run.click();
    step_once_same_frame(h, &format!("{} (modal path)", step.stem));
    h.run();
}

/// Interpret one drive against the whole window.
fn apply_drive(h: &mut Harness<'static, MnemonicGuiApp>, step: &Step, drive: &Drive) {
    match *drive {
        Drive::FlipSlotSubkey { anchor, occ, to } => {
            on_row_of(h, anchor, Role::ComboBox, occ).click();
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
            on_row_of(h, anchor, role, occ).type_text(value);
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
    }
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
