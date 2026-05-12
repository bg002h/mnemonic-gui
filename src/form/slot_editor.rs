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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SlotSubkey {
    Phrase,
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
                | SlotSubkey::Entropy
                | SlotSubkey::Wif
                | SlotSubkey::Xprv
        )
    }
}

/// One row in the SlotEditor — a (slot index, subkey, value) triple.
/// The repeating argv form is `--slot @<index>.<subkey>=<value>`.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Default)]
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

/// Render the SlotEditor inside a vertical scroll area (SPEC §B.4 R1 L-2:
/// row-height ~32px, no virtualization in v0.1 — N ≤ 16 cosigners bounds
/// the row count below the threshold where virtualization matters).
pub fn render(ui: &mut egui::Ui, state: &mut SlotState) {
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
                    ui.text_edit_singleline(&mut row.value);
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
        });
}
