//! Phase G P1.5 — manual-gui v1.0 cycle: manual_anchor_coverage RED.
//!
//! Track G's bidirectional anchor parity check (SPEC §2.0a #1
//! direction A / §2.1 G4): every (subcommand, flag, optional
//! variant) tuple enumerable from `mnemonic-gui`'s SubcommandSchema
//! arrays MUST resolve to a matching `id="..."` attribute in the
//! rendered HTML manual at `$MANUAL_GUI_HTML_PATH`.
//!
//! Companion gate to the Track M lint
//! `docs/manual-gui/tests/check_gui_schema_coverage.py` (which runs
//! during the `mnemonic-toolkit` repo's `make lint`). Both compute
//! the same SPEC §2.2 kebab-derived anchor set; this Rust-side test
//! is what `cargo test -p mnemonic-gui` exercises in the
//! `mnemonic-gui` PR's CI run.
//!
//! ## Implementation notes
//!
//! - Expected-anchor enumeration walks the four schema modules
//!   directly (mnemonic, md, ms, mk) and computes anchors via the
//!   SPEC §2.2 formula. Two languages (Python in toolkit, Rust here)
//!   implementing the same formula is the bidirectional invariant —
//!   if either side drifts, the cross-repo anchor namespace breaks.
//!
//! - The anchor formula:
//!   subcommand: `<tab>-<kebab(name)>`
//!   flag:       `<sub>-<flag-without-leading-dashes>`
//!   variant:    `<flag>-<kebab(value)>`
//!   where `kebab(s)` = lowercase, non-alphanumeric → `-`, collapse
//!   consecutive `-`, strip leading/trailing `-`.
//!
//! - At P1.5 (NOW): the test compiles cleanly, but fails at runtime
//!   because (1) no HTML manual yet exists at the resolved path, and
//!   (2) even if a stub HTML exists, it has no schema-shaped anchors.
//!   The RED is on the assertion that all expected anchors resolve.
//!
//! - At P2.1 (when `help::url::manual_url_for_*` helpers land):
//!   the test is expected to be migrated to call those helpers
//!   directly (so the test exercises the helpers' surface contract,
//!   not just a duplicate formula). The hand-computed implementation
//!   shipped at P1.5 is a stand-in. SPEC §2.2 anchors-equality is
//!   what gates LOCK either way.
//!
//! - At P2.4 (when manual content lands in Track M): the test goes
//!   GREEN — every expected anchor resolves to an `id="..."` in the
//!   HTML build.
//!
//! ## CI gating
//!
//! Marked `#[ignore]` per
//! `[[feedback-default-cargo-test-runs-sibling-dependent-tests]]`:
//! this test depends on the Track M build output at
//! `$MANUAL_GUI_HTML_PATH`, which is sibling-repo-resolved and not
//! guaranteed present in a developer's local checkout. The dedicated
//! CI job opts back in with `cargo test -- --include-ignored`.

use std::collections::HashSet;
use std::path::PathBuf;

use mnemonic_gui::schema::{self, FlagKind};

/// SPEC §2.2 kebab rule: lowercase, non-alphanumeric → `-`, collapse
/// consecutive `-`, strip leading/trailing `-`.
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

fn expected_anchors() -> Vec<String> {
    let mut out = Vec::new();
    for (tab, schema_ref) in [
        ("mnemonic", &schema::mnemonic::SCHEMA),
        ("md", &schema::md::SCHEMA),
        ("ms", &schema::ms::SCHEMA),
        ("mk", &schema::mk::SCHEMA),
    ] {
        for sub in schema_ref.subcommands {
            let sub_anchor = format!("{}-{}", tab, kebab(sub.name));
            out.push(sub_anchor.clone());
            for flag in sub.flags {
                let flag_name = flag.name.trim_start_matches('-');
                let flag_anchor = format!("{sub_anchor}-{flag_name}");
                out.push(flag_anchor.clone());
                let variants: Option<&'static [&'static str]> = match flag.kind {
                    FlagKind::Dropdown(v)
                    | FlagKind::NodeValueComposite(v)
                    | FlagKind::TaggedOrIndexed(v) => Some(v),
                    _ => None,
                };
                if let Some(vs) = variants {
                    for v in vs {
                        out.push(format!("{flag_anchor}-{}", kebab(v)));
                    }
                }
            }
        }
    }
    out
}

/// Plain-string scan over `id="..."` attribute occurrences. Pandoc
/// HTML5 emits these directly on heading elements (`<h1 id="...">`,
/// `<h2 id="...">`, etc.) — collecting via attribute, not via
/// element type, per SPEC §2.1 G4 emission rule.
fn collect_html_ids(html: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let needle = "id=\"";
    let mut rest = html;
    while let Some(start) = rest.find(needle) {
        let after = &rest[start + needle.len()..];
        if let Some(end) = after.find('"') {
            out.insert(after[..end].to_string());
            rest = &after[end + 1..];
        } else {
            break;
        }
    }
    out
}

#[test]
#[ignore = "Requires Track M manual HTML build at $MANUAL_GUI_HTML_PATH; \
            CI opts in via `cargo test -- --include-ignored` per \
            [[feedback-default-cargo-test-runs-sibling-dependent-tests]]"]
fn cell_manual_anchor_coverage_against_built_html() {
    let html_path: PathBuf = std::env::var("MANUAL_GUI_HTML_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // Sibling-checkout fallback per SPEC §2.5: the GUI manual
            // lives in mnemonic-toolkit, sibling to mnemonic-gui in a
            // shared workspace root.
            PathBuf::from(
                "../mnemonic-toolkit/docs/manual-gui/build/m-format-gui-manual.html",
            )
        });
    assert!(
        html_path.is_file(),
        "manual HTML not found at {html_path:?}; set $MANUAL_GUI_HTML_PATH \
         or build via `make -C ../mnemonic-toolkit/docs/manual-gui html`",
    );

    let html = std::fs::read_to_string(&html_path)
        .unwrap_or_else(|e| panic!("read {html_path:?}: {e}"));
    let found = collect_html_ids(&html);
    let expected = expected_anchors();

    let mut missing: Vec<String> = expected
        .iter()
        .filter(|a| !found.contains(a.as_str()))
        .cloned()
        .collect();
    missing.sort();
    missing.dedup();

    assert!(
        missing.is_empty(),
        "manual_anchor_coverage: {} schema anchor(s) missing from HTML at {:?} \
         (first 10: {:?})",
        missing.len(),
        html_path,
        missing.iter().take(10).collect::<Vec<_>>(),
    );
}

#[cfg(test)]
mod kebab_unit_tests {
    use super::kebab;

    #[test]
    fn kebab_lowercase_ascii_identity() {
        assert_eq!(kebab("convert"), "convert");
        assert_eq!(kebab("derive-child"), "derive-child");
        assert_eq!(kebab("seed-xor-split"), "seed-xor-split");
    }

    #[test]
    fn kebab_uppercase_lowered() {
        assert_eq!(kebab("BIP49"), "bip49");
        assert_eq!(kebab("SLIP39"), "slip39");
    }

    #[test]
    fn kebab_strips_leading_and_trailing_punctuation() {
        assert_eq!(kebab("--from"), "from");
        assert_eq!(kebab("--from-"), "from");
        assert_eq!(kebab("?test?"), "test");
    }

    #[test]
    fn kebab_collapses_runs() {
        assert_eq!(kebab("a/b//c"), "a-b-c");
        assert_eq!(kebab("foo___bar"), "foo-bar");
    }

    #[test]
    fn kebab_empty_after_strip() {
        assert_eq!(kebab("---"), "");
        assert_eq!(kebab(""), "");
    }
}
