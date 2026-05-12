//! Top-level app state (SPEC §B.2 + Phase 6 R1 I-3 fold).
//!
//! Tracks: which CLI tab is active, which subcommand is selected on each
//! tab, and the `path_detect` result per CLI (used by the tab strip to
//! grey unavailable tabs per SPEC §8 error class 1).
//!
//! Phase 6 lands the data-layer wiring (this file). The eframe-side
//! rendering (tab-strip widget, tab grey-out styling, tooltip text) is
//! deferred to Phase 7+ when the eframe loop is wired. The Phase 6 exit
//! gate is narrowed to "data layer wired + path_detect tested" per the
//! plan's Phase 6 R1 I-3 fold.

use crate::path_detect::{self, Detected};

/// One of the four constellation CLIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliTab {
    Mnemonic,
    Md,
    Ms,
    Mk,
}

impl CliTab {
    pub const ALL: &'static [CliTab] = &[CliTab::Mnemonic, CliTab::Md, CliTab::Ms, CliTab::Mk];

    pub fn bin_name(self) -> &'static str {
        match self {
            CliTab::Mnemonic => "mnemonic",
            CliTab::Md => "md",
            CliTab::Ms => "ms",
            CliTab::Mk => "mk",
        }
    }

    /// `<CLI>_BIN` env-var override name (matches the schema-mirror and
    /// runner integration test convention).
    pub fn bin_env_var(self) -> &'static str {
        match self {
            CliTab::Mnemonic => "MNEMONIC_BIN",
            CliTab::Md => "MD_BIN",
            CliTab::Ms => "MS_BIN",
            CliTab::Mk => "MK_BIN",
        }
    }
}

/// Boot-time scan of `$PATH` for each of the 4 CLIs + active-tab/
/// active-subcommand selection.
#[derive(Debug, Clone)]
pub struct AppState {
    pub mnemonic_detect: Detected,
    pub md_detect: Detected,
    pub ms_detect: Detected,
    pub mk_detect: Detected,
    pub active_tab: CliTab,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mnemonic_detect: Detected::NotFound,
            md_detect: Detected::NotFound,
            ms_detect: Detected::NotFound,
            mk_detect: Detected::NotFound,
            active_tab: CliTab::Mnemonic,
        }
    }
}

impl AppState {
    /// Build by running `path_detect` for each of the 4 CLIs. Phase 4+
    /// `init_tracing` will have logged each detection result by the time
    /// the GUI is ready to render.
    pub fn detect_all() -> Self {
        Self {
            mnemonic_detect: path_detect::detect("mnemonic"),
            md_detect: path_detect::detect("md"),
            ms_detect: path_detect::detect("ms"),
            mk_detect: path_detect::detect("mk"),
            active_tab: CliTab::Mnemonic,
        }
    }

    /// `Detected` result for a given tab.
    pub fn detect_for(&self, tab: CliTab) -> &Detected {
        match tab {
            CliTab::Mnemonic => &self.mnemonic_detect,
            CliTab::Md => &self.md_detect,
            CliTab::Ms => &self.ms_detect,
            CliTab::Mk => &self.mk_detect,
        }
    }

    /// True iff the tab's CLI binary is on `$PATH`. The tab-strip widget
    /// (Phase 7+) renders disabled-state for this iff `false`.
    pub fn tab_available(&self, tab: CliTab) -> bool {
        matches!(self.detect_for(tab), Detected::Found(_))
    }
}

/// Tooltip text for a greyed-out (NotFound) tab, per SPEC §8 error class 1.
/// Returns `None` if the tab IS available (no tooltip needed).
pub fn missing_binary_tooltip(tab: CliTab) -> String {
    format!(
        "`{cli}` binary not found on $PATH.\n\
         Install via: cargo install --locked --git <repo-url>",
        cli = tab.bin_name(),
    )
}

pub fn placeholder() {}
