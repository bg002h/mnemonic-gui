//! GUI help-icon URL helpers (manual-gui v1.0 cycle, SPEC §2.4).
//!
//! Builds anchor URLs into the rendered HTML user manual under
//! `https://bg002h.github.io/mnemonic-toolkit/manual-gui/` (or the
//! build-time-overridable `MNEMONIC_GUI_MANUAL_BASE_URL` env var). The
//! per-`?`-button click sites in `crate::form::widget` and `main.rs`
//! (P2.2) call these to compose `ctx.open_url(OpenUrl::new_tab(...))`.
//!
//! The three `manual_url_for_*` fns mirror SPEC §2.2's anchor formula:
//!
//! - subcommand: `<tab>-<kebab(name)>`
//! - flag:       `<sub-anchor>-<flag-without-leading-dashes>`
//! - variant:    `<flag-anchor>-<kebab(value)>`
//!
//! Callers pass raw schema strings (e.g. `flag.name == "--from"`); the
//! helpers strip leading dashes + kebab-fold internally so the call
//! shape mirrors the schema as-is.
//!
//! Bidirectional parity note: `tests/manual_anchor_coverage.rs` (P1.5)
//! ships its own local `kebab()` and anchor-derivation; that test is
//! scheduled to migrate to call these helpers at P2.4 GREEN so the two
//! sides share one source of truth. Until then, the formula MUST stay
//! byte-identical between the two implementations.

use crate::app::CliTab;

/// Manual base URL. Default points at GitHub Pages; override at build
/// time via `MNEMONIC_GUI_MANUAL_BASE_URL` (the PE.5 gh-pages bootstrap
/// enables the default URL; downstream packagers can re-point the
/// manual to a locally-hosted mirror by setting the env var at build).
pub const MANUAL_BASE_URL: &str = match option_env!("MNEMONIC_GUI_MANUAL_BASE_URL") {
    Some(s) => s,
    None => "https://bg002h.github.io/mnemonic-toolkit/manual-gui/",
};

/// SPEC §2.2 kebab rule: lowercase, non-alphanumeric → `-`, collapse
/// consecutive `-`, strip trailing `-`. (Leading `-` cannot occur in
/// the output because `prev_dash` starts `true`.)
fn kebab(value: &str) -> String {
    let mut prev_dash = true;
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            for lc in ch.to_lowercase() {
                out.push(lc);
            }
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn anchor_for_subcommand(tab: CliTab, subcommand: &str) -> String {
    format!("{}-{}", tab.bin_name(), kebab(subcommand))
}

fn anchor_for_flag(tab: CliTab, subcommand: &str, flag: &str) -> String {
    format!(
        "{}-{}",
        anchor_for_subcommand(tab, subcommand),
        flag.trim_start_matches('-'),
    )
}

fn anchor_for_variant(tab: CliTab, subcommand: &str, flag: &str, variant: &str) -> String {
    format!(
        "{}-{}",
        anchor_for_flag(tab, subcommand, flag),
        kebab(variant),
    )
}

/// Returns `<MANUAL_BASE_URL>#<tab>-<kebab(subcommand)>`. Example:
/// `manual_url_for_subcommand(CliTab::Mnemonic, "convert")` →
/// `https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-convert`.
pub fn manual_url_for_subcommand(tab: CliTab, subcommand: &str) -> String {
    format!(
        "{MANUAL_BASE_URL}#{}",
        anchor_for_subcommand(tab, subcommand),
    )
}

/// Returns `<MANUAL_BASE_URL>#<sub-anchor>-<flag-no-dashes>`. Example:
/// `manual_url_for_flag(CliTab::Mnemonic, "convert", "--from")` →
/// `https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-convert-from`.
///
/// SPEC §2.2 specifies "flag-name-without-leading-dashes" — only the
/// leading `-`s are stripped; the flag body is NOT kebab-folded because
/// flag names are already constrained to lowercase-ASCII + `-`.
pub fn manual_url_for_flag(tab: CliTab, subcommand: &str, flag: &str) -> String {
    format!(
        "{MANUAL_BASE_URL}#{}",
        anchor_for_flag(tab, subcommand, flag),
    )
}

/// Returns `<MANUAL_BASE_URL>#<flag-anchor>-<kebab(variant)>`. Example:
/// `manual_url_for_variant(CliTab::Mnemonic, "convert", "--from", "phrase")` →
/// `https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-convert-from-phrase`.
pub fn manual_url_for_variant(
    tab: CliTab,
    subcommand: &str,
    flag: &str,
    variant: &str,
) -> String {
    format!(
        "{MANUAL_BASE_URL}#{}",
        anchor_for_variant(tab, subcommand, flag, variant),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // The SPEC §2.4 example URL pinned into both the P1.4 kittest and
    // this module's docstring. If MANUAL_BASE_URL ever changes default,
    // these tests + the P1.4 cell update in lockstep.
    const EXAMPLE_BASE: &str = "https://bg002h.github.io/mnemonic-toolkit/manual-gui/";

    #[test]
    fn kebab_lowercase_ascii_identity() {
        assert_eq!(kebab("convert"), "convert");
        assert_eq!(kebab("seed-xor-split"), "seed-xor-split");
    }

    #[test]
    fn kebab_uppercase_lowered() {
        assert_eq!(kebab("BIP49"), "bip49");
    }

    #[test]
    fn kebab_strips_punctuation_and_trailing_dashes() {
        assert_eq!(kebab("--from"), "from");
        assert_eq!(kebab("--from-"), "from");
        assert_eq!(kebab("a/b//c"), "a-b-c");
        assert_eq!(kebab("---"), "");
        assert_eq!(kebab(""), "");
    }

    #[test]
    fn subcommand_anchor_matches_spec_2_2_example() {
        assert_eq!(
            manual_url_for_subcommand(CliTab::Mnemonic, "convert"),
            format!("{EXAMPLE_BASE}#mnemonic-convert"),
        );
        assert_eq!(
            manual_url_for_subcommand(CliTab::Md, "encode"),
            format!("{EXAMPLE_BASE}#md-encode"),
        );
        assert_eq!(
            manual_url_for_subcommand(CliTab::Ms, "vectors"),
            format!("{EXAMPLE_BASE}#ms-vectors"),
        );
        assert_eq!(
            manual_url_for_subcommand(CliTab::Mk, "inspect"),
            format!("{EXAMPLE_BASE}#mk-inspect"),
        );
    }

    #[test]
    fn flag_anchor_matches_p1_4_kittest_pinned_url() {
        // This is the exact URL the P1.4 kittest cell
        // (`cell_help_icon_emits_open_url_for_mnemonic_convert_from`)
        // asserts against — keep in lockstep.
        assert_eq!(
            manual_url_for_flag(CliTab::Mnemonic, "convert", "--from"),
            format!("{EXAMPLE_BASE}#mnemonic-convert-from"),
        );
    }

    #[test]
    fn flag_anchor_strips_leading_dashes_only() {
        // Flag bodies are not kebab-folded: SPEC §2.2 says
        // "flag-name-without-leading-dashes". A hypothetical
        // multi-dash internal name like `--key-origin` keeps its
        // body intact.
        assert_eq!(
            manual_url_for_flag(CliTab::Md, "encode", "--key-origin"),
            format!("{EXAMPLE_BASE}#md-encode-key-origin"),
        );
    }

    #[test]
    fn variant_anchor_kebabs_value() {
        assert_eq!(
            manual_url_for_variant(CliTab::Mnemonic, "convert", "--from", "phrase"),
            format!("{EXAMPLE_BASE}#mnemonic-convert-from-phrase"),
        );
        assert_eq!(
            manual_url_for_variant(CliTab::Mnemonic, "slip39-split", "--from", "entropy"),
            format!("{EXAMPLE_BASE}#mnemonic-slip39-split-from-entropy"),
        );
    }
}
