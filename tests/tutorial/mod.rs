//! The tutorial STEP MANIFEST — the single source of truth for the
//! `gui_example.pdf` corpus (`gui_example_tutorial` cycle).
//!
//! Authority: `mnemonic-toolkit/docs/manual-gui/design/SPEC_gui_example_tutorial.md`
//! §5 (naming + chapter/shot budget), §6 (determinism), §7 (secret hygiene),
//! and `IMPLEMENTATION_PLAN_gui_example_tutorial.md` P1.4.
//!
//! **Everything is manifest-derived — nothing hardcodes 25 or 51.** The step
//! table below drives:
//!   - the capture harness (`tests/gui_tutorial_snapshots.rs`): per-step drives,
//!     shots, run mode, expected exit;
//!   - the corpus census `manifest-stems.txt` (every committed PNG + transcript
//!     basename, emitted from [`corpus_manifest`]);
//!   - the machine-asserted secret-allowlist ([`SECRET_ALLOWLIST`]).
//!
//! **This module is egui-FREE** (no rasterizer, no `gui` feature needed) so the
//! always-run census / allowlist / gate-negative unit tests compile and run
//! under plain `cargo test`. Only the harness itself pulls the egui stack.
//!
//! **P1.4 scope = PILOTS ONLY** (Chapter-0 orientation + Journey-1 single-sig).
//! The full 25-step / 51-shot manifest is P1.5; it EXTENDS `MANIFEST` (and the
//! `Drive` vocabulary) — the counts grow from the data, never from an edit to a
//! hardcoded literal.

#![allow(dead_code)] // P1.5 extends the Drive/Step surface; pilot-unused helpers stay.

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::slot_editor::SlotSubkey;
use mnemonic_gui::secrets::{node_type_is_argv_secret, slot_subkey_is_secret, SECRET_FLAG_NAMES};

// ─── the secret allowlist (SPEC §7) ──────────────────────────────────────────

/// S0 — the world-known all-`abandon` BIP-39 test vector (Examples.md:209; fp
/// `73c5da0a`).
pub const S0: &str = "abandon abandon abandon abandon abandon abandon \
                      abandon abandon abandon abandon abandon about";
/// S1 — Examples.md:279 (fp `b8688df1`).
pub const S1: &str = "legal winner thank year wave sausage worth useful \
                      legal winner thank yellow";
/// S2 — Examples.md:284 (fp `28645006`).
pub const S2: &str = "letter advice cage absurd amount doctor acoustic avoid \
                      letter advice cage above";

/// The ONLY secret-class values the harness may ever drive (SPEC §7 —
/// demo-data-only rule). Machine-enforced by [`secret_allowlist_violations`]
/// over the manifest's secret-classified drives.
pub const SECRET_ALLOWLIST: &[&str] = &[S0, S1, S2];

// ─── drive vocabulary ────────────────────────────────────────────────────────

/// One form-fill action against the real whole-window app. The harness
/// interprets these via the spike-ratified 5-rule lookup discipline. The
/// pilot vocabulary below is a subset; P1.5 adds variants as its steps need
/// them (each new variant = a named harness interpreter arm, no hardcoded
/// counts touched).
#[derive(Clone, Copy, Debug)]
pub enum Drive {
    /// Flip a slot-editor row's subkey via its subkey `ComboBox` popup
    /// (row-anchored by `anchor`, `occ`-th match), landing on `to`.
    FlipSlotSubkey {
        anchor: &'static str,
        occ: usize,
        to: SlotSubkey,
    },
    /// Type `value` into a slot-editor row's value editor. `subkey` fixes both
    /// the secret classification (`slot_subkey_is_secret`) and the expected
    /// widget role (`PasswordInput` for secret subkeys, `TextInput` else) —
    /// asserted at runtime.
    TypeSlot {
        anchor: &'static str,
        occ: usize,
        subkey: SlotSubkey,
        value: &'static str,
    },
}

impl Drive {
    /// The secret-class value this drive routes to a secret-classified widget,
    /// if any (the allowlist checker's classification seam — rides the
    /// `SECRET_SLOT_SUBKEYS` / `SECRET_NODE_TYPES_ARGV` / `SECRET_FLAG_NAMES`
    /// taxonomies, NOT the I3 flag census; SPEC §7).
    pub fn secret_value(&self) -> Option<&'static str> {
        match self {
            Drive::TypeSlot { subkey, value, .. } if slot_subkey_is_secret(*subkey) => Some(value),
            _ => None,
        }
    }

    /// The `(anchor, occ)` of a VALUE-typing slot drive (the driven-field the
    /// §5.4 visibility check locates). `None` for a subkey-flip (no value; the
    /// same row's `TypeSlot` covers visibility) — this keeps the harness from
    /// looking up a value editor whose role isn't yet fixed.
    pub fn slot_target(&self) -> Option<(&'static str, usize)> {
        match *self {
            Drive::TypeSlot { anchor, occ, .. } => Some((anchor, occ)),
            _ => None,
        }
    }
}

// ─── the step ────────────────────────────────────────────────────────────────

/// One tutorial step: a real whole-window drive → shots → optional Run.
#[derive(Clone, Copy, Debug)]
pub struct Step {
    /// `tut-<j>-<nn>-<slug>` (SPEC §5.1). Shared across GUI corpus, toolkit
    /// figures/transcripts, and the chapter anchor.
    pub stem: &'static str,
    pub tab: CliTab,
    /// The subcommand this step drives (the `form_state` key is
    /// `"<bin>:<subcommand>"`).
    pub subcommand: &'static str,
    /// `Some(human_name)` drives the subcommand ComboBox popup; `None` rides
    /// the fresh-app default selection (`mnemonic:bundle`).
    pub select: Option<&'static str>,
    pub drives: &'static [Drive],
    /// Additional wheel-scroll deltas (points), each producing one extra
    /// `-formN` shot at a recorded offset (SPEC §5.4). Empty = single `-form`.
    pub scroll: &'static [f32],
    /// Does this step click Run and spawn the pinned CLI?
    pub runs: bool,
    /// Is the Run gated by the "Confirm secret-bearing run" modal (two-click
    /// secret path)? Implies the step is secret-bearing.
    pub secret_modal: bool,
    /// Capture the confirm modal as a `-modal` shot (secret_modal steps only).
    pub modal_shot: bool,
    /// Expected process exit (`Some(0)` normal, `Some(n!=0)` refusal). `None`
    /// for a no-run step (Chapter 0).
    pub expect_exit: Option<i32>,
    /// The run produced non-empty stderr (drift-checked against the captured
    /// `RunResult` at runtime, and governs whether a `.stderr.txt` transcript
    /// is committed).
    pub expect_stderr: bool,
}

impl Step {
    /// `"<bin>:<subcommand>"` — the `form_state` key.
    pub fn form_key(&self) -> String {
        format!("{}:{}", self.tab.bin_name(), self.subcommand)
    }

    /// This step is secret-bearing (drives a secret-class value into a secret
    /// widget, or its Run is gated by the secret-confirm modal).
    pub fn is_secret(&self) -> bool {
        self.secret_modal || self.drives.iter().any(|d| d.secret_value().is_some())
    }

    /// The PNG figure basenames (no extension) this step commits, in capture
    /// order: `-form`, `-form2..` per scroll offset, `-modal` (if captured),
    /// `-run` (if it runs).
    pub fn figure_stems(&self) -> Vec<String> {
        let mut v = vec![format!("{}-form", self.stem)];
        for i in 0..self.scroll.len() {
            v.push(format!("{}-form{}", self.stem, i + 2));
        }
        if self.secret_modal && self.modal_shot {
            v.push(format!("{}-modal", self.stem));
        }
        if self.runs {
            v.push(format!("{}-run", self.stem));
        }
        v
    }

    /// The transcript filenames this step commits (empty for a no-run step).
    /// `.stdout.txt` + `.exit.txt` always for a run; `.stderr.txt` iff the run
    /// emits stderr.
    pub fn transcript_files(&self) -> Vec<String> {
        if !self.runs {
            return Vec::new();
        }
        let mut v = vec![
            format!("{}.stdout.txt", self.stem),
            format!("{}.exit.txt", self.stem),
        ];
        if self.expect_stderr {
            v.push(format!("{}.stderr.txt", self.stem));
        }
        v
    }
}

// ─── the manifest (PILOTS: Chapter 0 + Journey 1) ────────────────────────────

/// SPEC §5.3 chapter plan. **P1.4 populates the two pilot steps** (Ch 0
/// orientation, 1 shot; J1 single-sig, 3 shots — form/modal/run); P1.5 appends
/// the remaining journeys (J2–J5 + the `shots: 0` transcript runs). Every
/// census reads THIS table.
pub const MANIFEST: &[Step] = &[
    // ── Chapter 0 — orientation (SPEC §5.3): the fresh app window, tabs,
    //    subcommand combo, output panel showing "(no run yet)". The demo-seed
    //    baseline (bundle pre-filled, one empty Xpub slot row — SPEC §6.3),
    //    rendered as-is. No drive, no Run: one `-form` shot.
    Step {
        stem: "tut-ch0-00-orientation",
        tab: CliTab::Mnemonic,
        subcommand: "bundle",
        select: None,
        drives: &[],
        scroll: &[],
        runs: false,
        secret_modal: false,
        modal_shot: false,
        expect_exit: None,
        expect_stderr: false,
    },
    // ── Journey 1 — single-sig card set (Examples.md §2): bundle bip84 with a
    //    typed, masked BIP-39 phrase (S0). Rides the demo seed — flip the
    //    seeded Xpub slot row to `phrase`, type S0 (masked-by-construction:
    //    SECRET_SLOT_SUBKEYS taxonomy → PasswordInput). Run is secret-gated →
    //    the confirm modal (captured), then modal-Run → the populated card-set
    //    pane. Three shots: form, modal, run.
    Step {
        stem: "tut-j1-01-bundle-single-sig",
        tab: CliTab::Mnemonic,
        subcommand: "bundle",
        select: None,
        drives: &[
            Drive::FlipSlotSubkey {
                anchor: "@",
                occ: 0,
                to: SlotSubkey::Phrase,
            },
            Drive::TypeSlot {
                anchor: "@",
                occ: 0,
                subkey: SlotSubkey::Phrase,
                value: S0,
            },
        ],
        scroll: &[],
        runs: true,
        secret_modal: true,
        modal_shot: true,
        expect_exit: Some(0),
        expect_stderr: true,
    },
];

// ─── manifest-derived censuses ───────────────────────────────────────────────

/// Every committed corpus artifact filename (figures + transcripts), in
/// manifest order (NOT yet sorted/deduped — [`corpus_manifest`] does that; the
/// raw form feeds the uniqueness census).
pub fn corpus_manifest_raw() -> Vec<String> {
    let mut v = Vec::new();
    for s in MANIFEST {
        for f in s.figure_stems() {
            v.push(format!("{f}.png"));
        }
        v.extend(s.transcript_files());
    }
    v
}

/// The `manifest-stems.txt` payload: every corpus artifact basename, sorted +
/// deduped. Both repos' censuses read this (the toolkit gate copies the file
/// from the pinned clone).
pub fn corpus_manifest() -> Vec<String> {
    let mut v = corpus_manifest_raw();
    v.sort();
    v.dedup();
    v
}

/// The exact bytes of `manifest-stems.txt` (trailing newline; LF).
pub fn manifest_stems_txt() -> String {
    let mut s = corpus_manifest().join("\n");
    s.push('\n');
    s
}

/// Total PNG shot count across the manifest (the "51-shot" number, but
/// DERIVED). Pilots = 4.
pub fn total_shots() -> usize {
    MANIFEST.iter().map(|s| s.figure_stems().len()).sum()
}

/// The distinct CLI binary names the manifest spawns (the version gate's probe
/// set). Only steps that Run contribute.
pub fn spawned_clis() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = MANIFEST
        .iter()
        .filter(|s| s.runs)
        .map(|s| s.tab.bin_name())
        .collect();
    v.sort();
    v.dedup();
    v
}

// ─── secret-allowlist checker (SPEC §7) ──────────────────────────────────────

/// Every value the manifest routes to a secret-classified widget MUST be in
/// [`SECRET_ALLOWLIST`]. Returns the list of violations (empty = clean).
/// Classification rides the `SECRET_SLOT_SUBKEYS` / `SECRET_NODE_TYPES_ARGV` /
/// `SECRET_FLAG_NAMES` taxonomies (recon C; NOT the I3 flag census).
pub fn secret_allowlist_violations() -> Vec<String> {
    let mut v = Vec::new();
    for step in MANIFEST {
        for drive in step.drives {
            if let Some(val) = drive.secret_value() {
                if !SECRET_ALLOWLIST.contains(&val) {
                    v.push(format!(
                        "{}: a secret-classified drive carries a NON-allowlisted value \
                         ({:?}); only the published test phrases S0/S1/S2 are permitted \
                         (SPEC §7)",
                        step.stem, val
                    ));
                }
            }
        }
    }
    v
}

/// Count of secret-classified drives across the manifest (non-vacuity guard for
/// the allowlist test).
pub fn secret_drive_count() -> usize {
    MANIFEST
        .iter()
        .flat_map(|s| s.drives.iter())
        .filter(|d| d.secret_value().is_some())
        .count()
}

/// Sanity: the argv-node secret taxonomy is reachable (keeps the import
/// honest even before P1.5 adds a composite-`phrase=` drive).
pub fn node_secret_taxonomy_nonempty() -> bool {
    node_type_is_argv_secret("phrase") && !SECRET_FLAG_NAMES.is_empty()
}

// ─── the two named gate PREDICATES (pure — unit-testable BITE) ────────────────

/// `pinned-tier-version-gate` comparison (SPEC §3.1b / §6 item 4 — the
/// `gen.sh:22` pattern). Pure so the negative BITE is suite-pinned, not just
/// spike history: the harness spawns `<cli> --version` and feeds the result
/// here; a unit test feeds a wrong-tier string and asserts `Err`.
pub fn version_matches(cli: &str, got: &str, expected: &str) -> Result<(), String> {
    if got == expected {
        Ok(())
    } else {
        Err(format!(
            "pinned-tier-version-gate: `{cli} --version` = {got:?}, expected {expected:?} — \
             refusing to render or spawn from a wrong tier (SPEC §3.1b; a wrong-tier local \
             regen must never produce honest-looking corpus bytes)"
        ))
    }
}

/// `SAME-FRAME-COMPLETION` predicate (SPEC §3.1b / §6.5, ruling-9 single-`step()`
/// semantics). `run_landed` = `app.last_run.is_some()` observed after exactly
/// ONE `harness.step()` delivered the queued Run/modal-Run click. Pure so the
/// negative BITE (async-runner → `run_landed == false`) is suite-pinned.
pub fn same_frame_completion(run_landed: bool, what: &str) -> Result<(), String> {
    if run_landed {
        Ok(())
    } else {
        Err(format!(
            "SAME-FRAME-COMPLETION violated ({what}): the runner must complete in the \
             Run-click frame — populated-pane contract, SPEC §6.5; any async-runner change \
             is a USER-decision downgrade, never an implementation choice"
        ))
    }
}
