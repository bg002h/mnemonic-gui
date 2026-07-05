//! The tutorial STEP MANIFEST — the single source of truth for the
//! `gui_example.pdf` corpus (`gui_example_tutorial` cycle).
//!
//! Authority: `mnemonic-toolkit/docs/manual-gui/design/SPEC_gui_example_tutorial.md`
//! §5 (naming + chapter/shot budget), §5.3 (the authoritative step table), §6
//! (determinism), §7 (secret hygiene); `IMPLEMENTATION_PLAN_gui_example_tutorial.md`
//! P1.4 (spine) + P1.5 (this full corpus).
//!
//! **Everything is manifest-derived — nothing hardcodes 25 or 51.** The step
//! table below drives:
//!   - the capture harness (`tests/gui_tutorial_snapshots.rs`): per-step drives,
//!     shots (via `capture`), run mode, expected exit, intra-journey md1 chains;
//!   - the corpus census `manifest-stems.txt` (every committed PNG + transcript
//!     basename, emitted from [`corpus_manifest`]);
//!   - the machine-asserted secret-allowlist ([`SECRET_ALLOWLIST`]).
//!
//! **This module is egui-FREE** (no rasterizer, no `gui` feature needed) so the
//! always-run census / allowlist / gate-negative unit tests compile and run
//! under plain `cargo test`. Only the harness itself pulls the egui stack.
//!
//! **P1.5 scope = the FULL corpus** (Ch 0 orientation + J1–J5). Shot-bearing
//! steps total 25 / 50 committed shots (51 nominal; 1 modal trimmed, §5.3); additional `capture: false` transcript-only
//! steps (J2 devices-1/2 converts, J3/J4 `bundle --json` chain feeds, the J4
//! NUMS bundle/restore) run the identical GUI-driven Run path but capture no PNG.
//! Every census reads THIS table; the counts grow from the data.

#![allow(dead_code)] // some helpers are harness-only; kept for the manifest surface.

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::slot_editor::SlotSubkey;
use mnemonic_gui::secrets::{
    node_type_is_argv_secret, slot_subkey_is_secret, SECRET_FLAG_NAMES,
};

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

// ─── public assembled descriptors (NOT tracked `.examples-build` primitives) ──
//
// `multisig.desc` (§3) and `taproot-multi.desc` (§6.6) are NOT among the four
// tracked `.examples-build` primitives (`degrade2.desc`/`tr2.desc`/`tr4.desc`/
// `degrade2-spec.json` — recon C); Examples' `gen.sh` assembles them INLINE
// from the three cosigners' live `mnemonic convert` fp+xpub outputs (`gen.sh:
// 252-261, 617-619`). They carry ONLY public watch-only material (origin-
// annotated xpubs + the BIP-341 NUMS H-point) — the secret-allowlist is not
// engaged — and are a pure function of the fixed public seeds S0/S1/S2. They
// live here as reviewed public constants (NOT fixtures — the fixture-provenance
// contract is scoped to the four tracked primitives; NOT chained — assembling
// them from six convert runs would add bespoke fp+xpub string-templating for
// zero determinism benefit). The J2 convert steps still RUN and DISPLAY each
// cosigner's real derivation, and the harness spawns the real pinned CLI on
// these descriptors, so the canonical/BSMS/bundle/restore outputs are the real
// Examples wallet outputs. (The md1 chunks a restore consumes ARE per-run
// generated and so ARE real chained output — see `Drive::TypeMd1Chain`.)
pub const MULTISIG_DESC: &str = "wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*,[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*))";

pub const TAPROOT_MULTI_DESC: &str = "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*,[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*))";

/// §6.5 compare-cost miniscripts (public, key-agnostic abstract labels).
pub const COMPARE_FOLDED: &str = "or_i(and_v(v:older(65535),multi(2,A,B)),or_i(and_v(v:older(4255898),multi(1,C,D,E)),and_v(v:pk(F),ripemd160(06d05e2f02fb90ddf98d8cd95d806ba12b27aff4))))";

// ─── drive vocabulary ────────────────────────────────────────────────────────

/// One form-fill action against the real whole-window app. The harness
/// interprets these via the spike-ratified 5-rule lookup discipline (each
/// variant a named interpreter arm in `gui_tutorial_snapshots.rs::apply_drive`).
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
    /// Click the `+ Add slot` button `count` times (J2 §3.4 grows to 3 slots).
    AddSlots { count: usize },
    /// Set the `occ`-th slot row's `@N` index DragValue to `index` (new slots
    /// default to `@0`; a multisig bundle needs contiguous `@0/@1/@2`).
    SetSlotIndex { occ: usize, index: i64 },
    /// Select `value` in a per-flag scalar `Dropdown` via its popup (row-anchored
    /// on the flag-name label). Handles the F1 export-wallet `(none)` sentinel
    /// (`value == ""` renders "(none)").
    SelectDropdown {
        flag: &'static str,
        value: &'static str,
    },
    /// Select `value` in the FIRST row of a REPEATING Dropdown flag (convert
    /// `--to` seeds one row whose combo sits BELOW the header, not on the label's
    /// row). Bounded below by `next_flag`'s label so the block lookup can't stray
    /// into a later flag's widget.
    SelectRowDropdown {
        flag: &'static str,
        next_flag: &'static str,
        value: &'static str,
    },
    /// Toggle a `Boolean` flag's checkbox true (row-anchored on the flag label).
    ToggleBoolean { flag: &'static str },
    /// Type literal `value` into a `Text` flag's input (row-anchored on the
    /// flag label). Public data only (descriptors / miniscripts).
    TypeText {
        flag: &'static str,
        value: &'static str,
    },
    /// Type the contents of a committed watch-only fixture file (read at
    /// runtime — cwd is the fixture dir) into a `Text` flag. The F2 route-around
    /// for the big fixture descriptors (`policy.desc` / `taproot.desc` /
    /// `taproot-4leaf.desc`) rides `--descriptor` TEXT, not `--descriptor-file`.
    TypeTextFromFixture {
        flag: &'static str,
        fixture: &'static str,
    },
    /// Type `value` into a `Path` flag's input (e.g. build-descriptor `--spec`).
    TypePath {
        flag: &'static str,
        value: &'static str,
    },
    /// Drive a `NodeValueComposite` flag (convert `--from`): select the `node`
    /// type in the composite combo, then type `value`. Secret-classified when
    /// `node_type_is_argv_secret(node)` (e.g. `phrase`) → masked `PasswordInput`.
    TypeComposite {
        flag: &'static str,
        node: &'static str,
        value: &'static str,
    },
    /// Set a `Number` flag (click its `Set` affordance, then AccessKit
    /// `SetValue`); e.g. `--threshold 2`.
    SetNumber { flag: &'static str, value: i64 },
    /// Append `--md1` repeating rows from the parsed md1 chunks captured out of
    /// a PRIOR step's `RunResult.stdout` (`chain_from` = that step's stem), and
    /// type each chunk into its row. The intra-journey chain replacing Examples'
    /// `jq -r ".md1[]" | sed "s/^/--md1 /"` glue (SPEC §3.1b).
    TypeMd1Chain {
        flag: &'static str,
        chain_from: &'static str,
    },
}

impl Drive {
    /// The secret-class value this drive routes to a secret-classified widget,
    /// if any (the allowlist checker's classification seam — rides the
    /// `SECRET_SLOT_SUBKEYS` / `SECRET_NODE_TYPES_ARGV` taxonomies, NOT the I3
    /// flag census; SPEC §7).
    pub fn secret_value(&self) -> Option<&'static str> {
        match self {
            Drive::TypeSlot { subkey, value, .. } if slot_subkey_is_secret(*subkey) => Some(value),
            Drive::TypeComposite { node, value, .. } if node_type_is_argv_secret(node) => {
                Some(value)
            }
            _ => None,
        }
    }

    /// The `(anchor, occ)` of a VALUE-typing slot drive (the slot row the §5.4
    /// visibility check locates). `None` for a subkey-flip (its sibling
    /// `TypeSlot` covers the row) and for non-slot drives.
    pub fn slot_target(&self) -> Option<(&'static str, usize)> {
        match *self {
            Drive::TypeSlot { anchor, occ, .. } => Some((anchor, occ)),
            _ => None,
        }
    }

    /// The flag-name LABEL whose on-screen rect stands in for this drive's
    /// field in the §5.4 driven-field-visibility check (the label sits on the
    /// same row as the input for scalar flags; for the repeating `--md1` chain
    /// it is the header, whose visibility means "the user can see WHERE chunks
    /// go" — the full chunk list always accompanies the shot as the gated
    /// transcript, per the §5.4 viewport-faithful contract). `None` for drives
    /// with no distinct value field to keep visible (slot drives use
    /// [`slot_target`]; `AddSlots` mutates the slot block a `TypeSlot` covers).
    pub fn label_anchor(&self) -> Option<&'static str> {
        match *self {
            Drive::SelectDropdown { flag, .. }
            | Drive::SelectRowDropdown { flag, .. }
            | Drive::ToggleBoolean { flag }
            | Drive::TypeText { flag, .. }
            | Drive::TypeTextFromFixture { flag, .. }
            | Drive::TypePath { flag, .. }
            | Drive::TypeComposite { flag, .. }
            | Drive::SetNumber { flag, .. }
            | Drive::TypeMd1Chain { flag, .. } => Some(flag),
            Drive::FlipSlotSubkey { .. }
            | Drive::TypeSlot { .. }
            | Drive::AddSlots { .. }
            | Drive::SetSlotIndex { .. } => None,
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
    /// Capture the confirm modal as a `-modal` shot (secret_modal steps only —
    /// J1 + J2 all-seeds keep theirs; other secret steps run through the modal
    /// but capture no modal shot, SPEC §5.3).
    pub modal_shot: bool,
    /// **Capture-gate (SPEC §5.3 `shots: 0`).** `true` = a shot-bearing step
    /// (emits `-form`/`-modal`/`-run` PNGs). `false` = a transcript-only step:
    /// runs the identical GUI-driven path and byte-gates its transcript, but
    /// captures NO PNG (chain feeds, devices-1/2 converts, NUMS bundle/restore).
    /// `capture: false` ⇒ `runs: true` (a no-PNG no-run step would be inert).
    pub capture: bool,
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
    /// `-run` (if it runs). A `capture: false` step commits ZERO PNGs.
    pub fn figure_stems(&self) -> Vec<String> {
        if !self.capture {
            return Vec::new();
        }
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

    /// The transcript filenames this step commits (empty for a no-run step;
    /// UNAFFECTED by `capture` — a `shots: 0` transcript-only step still gates
    /// its `.stdout`/`.exit`/`.stderr`). `.stdout.txt` + `.exit.txt` always for
    /// a run; `.stderr.txt` iff the run emits stderr.
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

// ─── the manifest ────────────────────────────────────────────────────────────

mod manifest;
pub use manifest::MANIFEST;

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

/// Total PNG shot count across the manifest (50 committed; the "51" nominal, but
/// DERIVED — `capture: false` steps contribute zero).
pub fn total_shots() -> usize {
    MANIFEST.iter().map(|s| s.figure_stems().len()).sum()
}

/// Count of shot-bearing steps (§5.3 "25 steps"; DERIVED).
pub fn shot_bearing_step_count() -> usize {
    MANIFEST.iter().filter(|s| s.capture).count()
}

/// Count of manifest steps that Run (shot-bearing + `shots: 0` transcript-only)
/// — the transcript census's expected run count.
pub fn run_step_count() -> usize {
    MANIFEST.iter().filter(|s| s.runs).count()
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
/// Classification rides the `SECRET_SLOT_SUBKEYS` / `SECRET_NODE_TYPES_ARGV`
/// taxonomies (recon C; NOT the I3 flag census).
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

/// Sanity: the argv-node secret taxonomy is reachable.
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

/// Parse the md1 chunks out of a `bundle --json` (or text-form) `RunResult`
/// stdout — the intra-journey chain payload (SPEC §3.1b). Tries the `--json`
/// `md1` array first (the shape every chain feed uses); falls back to scanning
/// ungrouped `md1…` lines. Returns the chunk strings in order.
pub fn parse_md1_chunks(stdout: &[u8]) -> Vec<String> {
    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(stdout) {
        if let Some(arr) = val.get("md1").and_then(|m| m.as_array()) {
            return arr
                .iter()
                .filter_map(|c| c.as_str().map(str::to_string))
                .collect();
        }
    }
    // Text-form fallback: ungrouped chunks are lines of `md1` + base32 (no
    // dashes / spaces). Skip the grouped (`md1fg-…`) and comment (`# …`) lines.
    String::from_utf8_lossy(stdout)
        .lines()
        .map(str::trim)
        .filter(|l| {
            l.starts_with("md1")
                && !l.contains('-')
                && !l.contains(' ')
                && l.len() > 40
        })
        .map(str::to_string)
        .collect()
}
