//! SlotEditor composite widget for the `--slot @N.<subkey>=<value>`
//! repeating grammar (SPEC §B.4).
//!
//! `SlotSubkey` mirrors `mnemonic-toolkit::slot_input::SlotSubkey` exactly —
//! 8 variants, 4 secret-bearing (Phrase / Entropy / Wif / Xprv), 4
//! watch-only (Xpub / MasterXpub / Fingerprint / Path). Phase 7 source-audit
//! verifies this set against the upstream `is_secret_bearing()` true-arm
//! match. Phase 1 R1 fold pinned the pattern of mirroring upstream sets
//! source-side.

use eframe::egui;

/// Per-row slot subkey selector. Variants + `as_str()` ordering match
/// `crates/mnemonic-toolkit/src/slot_input.rs:13-28, 44-55` exactly.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum SlotSubkey {
    Phrase,
    /// v0.16.1 — mirrors `mnemonic-toolkit::slot_input::SlotSubkey::Seedqr`
    /// (added in toolkit v0.31.3). Secret-bearing (decodes to a BIP-39
    /// phrase at slot-emit time via `seedqr::decode`). Position-critical:
    /// declared at index 1 (after Phrase, before Entropy) to mirror the
    /// upstream enum-order so derived `Ord` produces ascending-sorted
    /// legal-set patterns that match the toolkit's `is_legal_set`.
    Seedqr,
    Entropy,
    Xpub,
    MasterXpub,
    Fingerprint,
    Path,
    Wif,
    Xprv,
}

impl SlotSubkey {
    pub const ALL: &'static [SlotSubkey] = &[
        SlotSubkey::Phrase,
        SlotSubkey::Seedqr,
        SlotSubkey::Entropy,
        SlotSubkey::Xpub,
        SlotSubkey::MasterXpub,
        SlotSubkey::Fingerprint,
        SlotSubkey::Path,
        SlotSubkey::Wif,
        SlotSubkey::Xprv,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            SlotSubkey::Phrase => "phrase",
            SlotSubkey::Seedqr => "seedqr",
            SlotSubkey::Entropy => "entropy",
            SlotSubkey::Xpub => "xpub",
            SlotSubkey::MasterXpub => "master_xpub",
            SlotSubkey::Fingerprint => "fingerprint",
            SlotSubkey::Path => "path",
            SlotSubkey::Wif => "wif",
            SlotSubkey::Xprv => "xprv",
        }
    }

    pub fn is_secret_bearing(self) -> bool {
        matches!(
            self,
            SlotSubkey::Phrase
                | SlotSubkey::Seedqr
                | SlotSubkey::Entropy
                | SlotSubkey::Wif
                | SlotSubkey::Xprv
        )
    }
}

/// One row in the SlotEditor — a (slot index, subkey, value) triple.
/// The repeating argv form is `--slot @<index>.<subkey>=<value>`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SlotRow {
    pub index: u8,
    pub subkey: SlotSubkey,
    pub value: String,
}

impl Default for SlotRow {
    fn default() -> Self {
        Self {
            index: 0,
            subkey: SlotSubkey::Xpub,
            value: String::new(),
        }
    }
}

/// Multi-row slot state. Owned by `FormState` for subcommands where
/// `allows_slots == true`. Rows are stored in user-add order; emission
/// re-sorts by `index` ascending (SPEC §6.4 + upstream
/// `cmd::bundle::resolve_slots` BTreeMap iteration).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SlotState {
    pub rows: Vec<SlotRow>,
}

impl SlotState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Iterator over rows in slot-index ascending order. Stable sort —
    /// rows with the same `index` preserve insertion order. Upstream
    /// `cmd::bundle::resolve_slots` keys its `BTreeMap<u8, Vec<&SlotInput>>`
    /// by `u8` index alone; duplicate `(index, subkey)` pairs for the same
    /// slot are rejected by `validate_slot_set` (`duplicate-subkey` error),
    /// NOT by BTreeMap-key collision. R1 I-1 fold (clarifies prior comment).
    pub fn rows_sorted(&self) -> Vec<&SlotRow> {
        let mut v: Vec<&SlotRow> = self.rows.iter().collect();
        v.sort_by_key(|r| r.index);
        v
    }

    /// Subset of rows whose subkey is NOT secret-bearing. Phase 8's
    /// `persistence::serialize_form_state` will use this to filter slot
    /// rows before round-tripping through `state.json` per SPEC §10.
    pub fn persistable_rows(&self) -> impl Iterator<Item = &SlotRow> {
        self.rows.iter().filter(|r| !r.subkey.is_secret_bearing())
    }

    /// Emit `--slot @N.subkey=value` argv pairs. Rows with empty `value`
    /// are skipped (matches the SPEC §6.7 empty-value omission rule for
    /// other FlagKind variants).
    pub fn to_slot_argv(&self) -> Vec<String> {
        let mut out = Vec::new();
        for row in self.rows_sorted() {
            if row.value.is_empty() {
                continue;
            }
            out.push("--slot".to_string());
            out.push(format!(
                "@{}.{}={}",
                row.index,
                row.subkey.as_str(),
                row.value
            ));
        }
        out
    }
}

/// v0.7.1 SPEC §6.6 row 8 — detect non-contiguous slot indices.
///
/// Returns the sorted list of missing indices (`Vec<u8>`) that would cause
/// the CLI to reject the bundle with `error: slot indices must be
/// contiguous starting at @0; missing @{i}`. Empty result means all slots
/// are contiguous starting at @0 (or the slot set is empty).
///
/// Duplicate-index rows are NOT a contiguity violation (multiple subkeys
/// per slot index — e.g., `@0.phrase + @0.path` — are a legitimate row
/// shape). The detector operates on UNIQUE indices.
///
/// Option A pattern (mirrors v0.7.0 `NumberMax::FromSlotCount` for row 9):
/// pure GUI-internal pre-check; no toolkit wire-format change. The CLI
/// still authoritatively rejects non-contiguous bundles at runtime.
pub fn detect_slot_index_gaps(rows: &[SlotRow]) -> Vec<u8> {
    if rows.is_empty() {
        return Vec::new();
    }
    let unique: std::collections::BTreeSet<u8> = rows.iter().map(|r| r.index).collect();
    let max_index = *unique.iter().max().expect("non-empty set has a max");
    (0..=max_index).filter(|i| !unique.contains(i)).collect()
}

/// Render the SlotEditor inside a vertical scroll area (SPEC §B.4 R1 L-2:
/// row-height ~32px, no virtualization in v0.1 — N ≤ 16 cosigners bounds
/// the row count below the threshold where virtualization matters).
///
/// **v0.8.1 F3 — `path_hint`:** when Some, the per-row text-edit widget
/// renders the hint string as a placeholder (`egui::TextEdit::hint_text`)
/// whenever `row.subkey == SlotSubkey::Path` AND `row.value.is_empty()`.
/// Pass None to preserve pre-v0.8.1 rendering. Main.rs computes the hint
/// from `descriptor_non_canonical_default_path_notice`'s underlying
/// machinery and threads it through here.
pub fn render(ui: &mut egui::Ui, state: &mut SlotState, path_hint: Option<&str>) {
    egui::ScrollArea::vertical()
        .max_height(320.0) // ~10 rows at default row-height before scroll
        .show(ui, |ui| {
            let mut remove_idx: Option<usize> = None;
            for (i, row) in state.rows.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label("@");
                    ui.add(egui::DragValue::new(&mut row.index).range(0u8..=15));
                    ui.label(".");
                    egui::ComboBox::from_id_salt(("slot_subkey", i))
                        .selected_text(row.subkey.as_str())
                        .show_ui(ui, |ui| {
                            for opt in SlotSubkey::ALL {
                                ui.selectable_value(&mut row.subkey, *opt, opt.as_str());
                            }
                        });
                    ui.label("=");
                    match (row.subkey, path_hint) {
                        (SlotSubkey::Path, Some(hint)) if row.value.is_empty() => {
                            ui.add(
                                egui::TextEdit::singleline(&mut row.value).hint_text(hint),
                            );
                        }
                        _ => {
                            ui.text_edit_singleline(&mut row.value);
                        }
                    }
                    if ui.button("✕").clicked() {
                        remove_idx = Some(i);
                    }
                });
            }
            if let Some(i) = remove_idx {
                state.rows.remove(i);
            }
            if ui.button("+ Add slot").clicked() {
                state.rows.push(SlotRow::default());
            }
            // v0.7.1 SPEC §6.6 row 8 — inline contiguity warning. Pre-checks
            // the CLI's mode-violation row 8 (`error: slot indices must be
            // contiguous starting at @0; missing @{i}`) so the user sees
            // the issue before hitting the CLI error. Renders nothing when
            // the slot set is contiguous (or empty). Option A pattern: no
            // toolkit wire-format change; the CLI is still authoritative.
            let gaps = detect_slot_index_gaps(&state.rows);
            if !gaps.is_empty() {
                let missing = gaps
                    .iter()
                    .map(|i| format!("@{i}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                ui.colored_label(
                    egui::Color32::from_rgb(220, 165, 0),
                    format!(
                        "⚠ slot indices must be contiguous starting at @0; missing {missing}"
                    ),
                );
            }
        });
}
