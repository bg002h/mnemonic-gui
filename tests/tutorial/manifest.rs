//! The tutorial step table (SPEC §5.3). Split from `mod.rs` for readability —
//! `mod.rs` owns the types + censuses; this file is pure data.
//!
//! 25 shot-bearing steps / 50 committed shots (51 nominal; 1 modal trimmed) (Ch 0 + J1–J5) plus `capture: false`
//! transcript-only steps (J2 devices-1/2 converts, per-journey `bundle --json`
//! chain feeds, the J4 NUMS bundle/restore). Every census reads this table.
//!
//! Two route-arounds worth flagging (documented at their steps):
//!   - **F2**: engrave-shape bundle + export-wallet descriptor inputs ride
//!     `--descriptor` TEXT, not `--descriptor-file` (ruling 6).
//!   - **restore-form single-sig `--template` leak** (a real GUI papercut found
//!     at P1.5): the restore form materializes single-sig `--template=bip44`,
//!     rejected in md1 mode; we select the wallet's MULTISIG template (inert in
//!     md1 mode, output byte-identical) to route around it.

use super::{
    Drive, Step, COMPARE_FOLDED, MULTISIG_DESC, S0, S1, S2, TAPROOT_MULTI_DESC,
};
use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::slot_editor::SlotSubkey;

// Subcommand ComboBox human-names (schema `human_name`, `schema/mnemonic.rs`).
const HN_CONVERT: &str = "Convert (between formats)";
const HN_EXPORT: &str = "Export Wallet (watch-only)";
const HN_RESTORE: &str = "Restore (re-derive a wallet export from a source)";
const HN_COMPARE: &str = "Compare Cost (wsh vs tr per spending condition)";
const HN_BUILD: &str = "Build Descriptor (policy-tree spec → wsh descriptor + BIP-388)";

// ─── terse builders (const fn — Step is Copy) ────────────────────────────────

/// Shot-bearing running step, exit 0, non-empty stderr, non-secret.
const fn run_step(
    stem: &'static str,
    subcommand: &'static str,
    select: Option<&'static str>,
    drives: &'static [Drive],
) -> Step {
    Step {
        stem,
        tab: CliTab::Mnemonic,
        subcommand,
        select,
        drives,
        scroll: &[],
        runs: true,
        secret_modal: false,
        modal_shot: false,
        capture: true,
        expect_exit: Some(0),
        expect_stderr: true,
    }
}

/// A `capture: false` transcript-only running step (chain feed / devices
/// convert / NUMS). `secret` marks the modal path (no modal shot).
const fn feed_step(
    stem: &'static str,
    subcommand: &'static str,
    select: Option<&'static str>,
    drives: &'static [Drive],
    secret: bool,
    expect_stderr: bool,
) -> Step {
    Step {
        stem,
        tab: CliTab::Mnemonic,
        subcommand,
        select,
        drives,
        scroll: &[],
        runs: true,
        secret_modal: secret,
        modal_shot: false,
        capture: false,
        expect_exit: Some(0),
        expect_stderr,
    }
}

/// A refusal step (non-zero exit, empty stdout, stderr diagnostic).
const fn refusal_step(
    stem: &'static str,
    subcommand: &'static str,
    select: Option<&'static str>,
    drives: &'static [Drive],
) -> Step {
    Step {
        stem,
        tab: CliTab::Mnemonic,
        subcommand,
        select,
        drives,
        scroll: &[],
        runs: true,
        secret_modal: false,
        modal_shot: false,
        capture: true,
        expect_exit: Some(2),
        expect_stderr: true,
    }
}

// ─── shared drive fragments ──────────────────────────────────────────────────

/// export-wallet descriptor-mode prefix: F1 `(none)` unlock, then `--descriptor`.
macro_rules! export_descriptor_text {
    ($val:expr, $fmt:expr) => {
        &[
            Drive::SelectDropdown { flag: "--template", value: "" },
            Drive::TypeText { flag: "--descriptor", value: $val },
            Drive::SelectDropdown { flag: "--format", value: $fmt },
        ]
    };
}
macro_rules! export_descriptor_fixture {
    ($fix:expr, $fmt:expr) => {
        &[
            Drive::SelectDropdown { flag: "--template", value: "" },
            Drive::TypeTextFromFixture { flag: "--descriptor", fixture: $fix },
            Drive::SelectDropdown { flag: "--format", value: $fmt },
        ]
    };
}

/// convert phrase-derivation drives (`--from phrase=<seed>`, `--to <node>`).
macro_rules! convert_drives {
    ($seed:expr, $to:expr) => {
        &[
            Drive::TypeComposite { flag: "--from", node: "phrase", value: $seed },
            Drive::SelectRowDropdown { flag: "--to", next_flag: "--network", value: $to },
            Drive::SelectDropdown { flag: "--template", value: "wsh-sortedmulti" },
        ]
    };
}

/// restore-from-md1 drives (multisig-`--template` route-around + md1 chain).
macro_rules! restore_drives {
    ($template:expr, $feed:expr) => {
        &[
            Drive::SelectDropdown { flag: "--template", value: $template },
            Drive::TypeMd1Chain { flag: "--md1", chain_from: $feed },
        ]
    };
}

pub const MANIFEST: &[Step] = &[
    // ══ Chapter 0 — orientation: the fresh app window, demo-seed baseline. ══
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
        capture: true,
        expect_exit: None,
        expect_stderr: false,
    },
    // ══ Journey 1 — single-sig card set (§2): bundle bip84 + masked S0. ══
    Step {
        stem: "tut-j1-01-bundle-single-sig",
        tab: CliTab::Mnemonic,
        subcommand: "bundle",
        select: None,
        drives: &[
            Drive::FlipSlotSubkey { anchor: "@", occ: 0, to: SlotSubkey::Phrase },
            Drive::TypeSlot { anchor: "@", occ: 0, subkey: SlotSubkey::Phrase, value: S0 },
        ],
        scroll: &[],
        runs: true,
        secret_modal: true,
        modal_shot: true,
        capture: true,
        expect_exit: Some(0),
        expect_stderr: true,
    },
    // ══ Journey 2 — conventional 2-of-3 multisig (§3, §3.4). ══
    // 02. per-device fingerprint (device 0 / S0) — secret phrase, no modal shot.
    Step {
        stem: "tut-j2-02-convert-fingerprint",
        tab: CliTab::Mnemonic,
        subcommand: "convert",
        select: Some(HN_CONVERT),
        drives: convert_drives!(S0, "fingerprint"),
        scroll: &[],
        runs: true,
        secret_modal: true,
        modal_shot: false,
        capture: true,
        expect_exit: Some(0),
        expect_stderr: true,
    },
    // 03. per-device xpub (device 0 / S0).
    Step {
        stem: "tut-j2-03-convert-xpub",
        tab: CliTab::Mnemonic,
        subcommand: "convert",
        select: Some(HN_CONVERT),
        drives: convert_drives!(S0, "xpub"),
        scroll: &[],
        runs: true,
        secret_modal: true,
        modal_shot: false,
        capture: true,
        expect_exit: Some(0),
        expect_stderr: true,
    },
    // devices 1/2 converts (S1, S2) — transcript-only (identical interaction).
    feed_step("tut-j2-dev1-convert-fingerprint", "convert", Some(HN_CONVERT),
        convert_drives!(S1, "fingerprint"), true, true),
    feed_step("tut-j2-dev1-convert-xpub", "convert", Some(HN_CONVERT),
        convert_drives!(S1, "xpub"), true, true),
    feed_step("tut-j2-dev2-convert-fingerprint", "convert", Some(HN_CONVERT),
        convert_drives!(S2, "fingerprint"), true, true),
    feed_step("tut-j2-dev2-convert-xpub", "convert", Some(HN_CONVERT),
        convert_drives!(S2, "xpub"), true, true),
    // 04. canonicalise the assembled 2-of-3 descriptor.
    run_step("tut-j2-04-canonicalise", "export-wallet", Some(HN_EXPORT),
        export_descriptor_text!(MULTISIG_DESC, "descriptor")),
    // 05. first receive address via the BSMS record.
    run_step("tut-j2-05-bsms", "export-wallet", Some(HN_EXPORT),
        export_descriptor_text!(MULTISIG_DESC, "bsms")),
    // 06. engrave the shared watch-only card set (F2: --descriptor TEXT).
    run_step("tut-j2-06-bundle-watch-only", "bundle", None,
        &[Drive::TypeText { flag: "--descriptor", value: MULTISIG_DESC }]),
    // 07. all-seeds bundle (§3.4): 3 slots, masked phrases, confirm-modal shot.
    Step {
        stem: "tut-j2-07-bundle-all-seeds",
        tab: CliTab::Mnemonic,
        subcommand: "bundle",
        select: None,
        drives: &[
            Drive::SelectDropdown { flag: "--template", value: "wsh-sortedmulti" },
            Drive::SelectDropdown { flag: "--multisig-path-family", value: "bip87" },
            Drive::AddSlots { count: 2 },
            Drive::SetSlotIndex { occ: 1, index: 1 },
            Drive::SetSlotIndex { occ: 2, index: 2 },
            // Explicit threshold (max = slot count = 3 now): the template-defaults
            // hook's seed can be clamped to 1 while only 1 slot exists.
            Drive::SetNumber { flag: "--threshold", value: 2 },
            Drive::FlipSlotSubkey { anchor: "@", occ: 0, to: SlotSubkey::Phrase },
            Drive::TypeSlot { anchor: "@", occ: 0, subkey: SlotSubkey::Phrase, value: S0 },
            Drive::FlipSlotSubkey { anchor: "@", occ: 1, to: SlotSubkey::Phrase },
            Drive::TypeSlot { anchor: "@", occ: 1, subkey: SlotSubkey::Phrase, value: S1 },
            Drive::FlipSlotSubkey { anchor: "@", occ: 2, to: SlotSubkey::Phrase },
            Drive::TypeSlot { anchor: "@", occ: 2, subkey: SlotSubkey::Phrase, value: S2 },
        ],
        scroll: &[],
        runs: true,
        secret_modal: true,
        // Pre-committed budget trim (ruling 4): keep ONE representative modal
        // shot (J1's); the all-seeds 3-slot modal runs through the confirm path
        // but captures no modal PNG.
        modal_shot: false,
        capture: true,
        expect_exit: Some(0),
        expect_stderr: true,
    },
    // chain feed for the J2 restore (multisig md1).
    feed_step("tut-j2-restore-feed-bundle-json", "bundle", None,
        &[Drive::TypeText { flag: "--descriptor", value: MULTISIG_DESC },
          Drive::ToggleBoolean { flag: "--json" }], false, true),
    // 08. restore the 2-of-3 from its md1 card alone (default text form).
    run_step("tut-j2-08-restore", "restore", Some(HN_RESTORE),
        restore_drives!("wsh-sortedmulti", "tut-j2-restore-feed-bundle-json")),

    // ══ Journey 3 — pathological 4-tier degrading wsh vault (§5). ══
    // 09. the guided builder REFUSES the 11-key policy (--spec file path).
    refusal_step("tut-j3-09-build-descriptor-refusal", "build-descriptor", Some(HN_BUILD),
        &[Drive::TypePath { flag: "--spec", value: "policy.json" }]),
    // 10. canonicalise the hand-written miniscript descriptor.
    run_step("tut-j3-10-canonicalise", "export-wallet", Some(HN_EXPORT),
        export_descriptor_fixture!("policy.desc", "descriptor")),
    // 11. first receive address (BSMS).
    run_step("tut-j3-11-bsms", "export-wallet", Some(HN_EXPORT),
        export_descriptor_fixture!("policy.desc", "bsms")),
    // 12. engrave the watch-only card set (F2: --descriptor TEXT).
    run_step("tut-j3-12-engrave", "bundle", None,
        &[Drive::TypeTextFromFixture { flag: "--descriptor", fixture: "policy.desc" }]),
    // chain feed for the J3 restore (11-key vault md1).
    feed_step("tut-j3-restore-feed-bundle-json", "bundle", None,
        &[Drive::TypeTextFromFixture { flag: "--descriptor", fixture: "policy.desc" },
          Drive::ToggleBoolean { flag: "--json" }], false, true),
    // 13. restore round-trip — reconstructs the same first address.
    run_step("tut-j3-13-restore", "restore", Some(HN_RESTORE),
        restore_drives!("wsh-sortedmulti", "tut-j3-restore-feed-bundle-json")),

    // ══ Journey 4 — taproot twin (§6). ══
    // 14. depth-2 taptree REFUSAL (the PR-#953 guard).
    refusal_step("tut-j4-14-depth2-refusal", "export-wallet", Some(HN_EXPORT),
        export_descriptor_fixture!("taproot-4leaf.desc", "descriptor")),
    // 15. canonicalise the depth-1 tr(...) descriptor.
    run_step("tut-j4-15-canonicalise", "export-wallet", Some(HN_EXPORT),
        export_descriptor_fixture!("taproot.desc", "descriptor")),
    // 16. engrave the taproot card set (F2: --descriptor TEXT).
    run_step("tut-j4-16-engrave", "bundle", None,
        &[Drive::TypeTextFromFixture { flag: "--descriptor", fixture: "taproot.desc" }]),
    // chain feed for the J4 taproot restore.
    feed_step("tut-j4-restore-feed-bundle-json", "bundle", None,
        &[Drive::TypeTextFromFixture { flag: "--descriptor", fixture: "taproot.desc" },
          Drive::ToggleBoolean { flag: "--json" }], false, true),
    // 17. restore + first address (proves the real internal key round-trips).
    run_step("tut-j4-17-restore", "restore", Some(HN_RESTORE),
        restore_drives!("tr-sortedmulti-a", "tut-j4-restore-feed-bundle-json")),
    // 18. Bitcoin Core importdescriptors payload.
    run_step("tut-j4-18-core-export", "export-wallet", Some(HN_EXPORT),
        export_descriptor_fixture!("taproot.desc", "bitcoin-core")),
    // 19. BSMS is UNSUPPORTED for taproot (a teaching refusal).
    refusal_step("tut-j4-19-bsms-unsupported", "export-wallet", Some(HN_EXPORT),
        export_descriptor_fixture!("taproot.desc", "bsms")),
    // 20. spending-cost comparison (§6.5; no stderr).
    Step {
        stem: "tut-j4-20-compare-cost",
        tab: CliTab::Mnemonic,
        subcommand: "compare-cost",
        select: Some(HN_COMPARE),
        drives: &[Drive::TypeText { flag: "--miniscript", value: COMPARE_FOLDED }],
        scroll: &[],
        runs: true,
        secret_modal: false,
        modal_shot: false,
        capture: true,
        expect_exit: Some(0),
        expect_stderr: false,
    },
    // 21. NUMS taproot multisig — canonicalise (§6.6).
    run_step("tut-j4-21-nums-export", "export-wallet", Some(HN_EXPORT),
        export_descriptor_text!(TAPROOT_MULTI_DESC, "descriptor")),
    // NUMS bundle/restore — transcript-only (§5.3).
    feed_step("tut-j4-nums-feed-bundle-json", "bundle", None,
        &[Drive::TypeText { flag: "--descriptor", value: TAPROOT_MULTI_DESC },
          Drive::ToggleBoolean { flag: "--json" }], false, true),
    feed_step("tut-j4-nums-restore", "restore", Some(HN_RESTORE),
        restore_drives!("tr-sortedmulti-a", "tut-j4-nums-feed-bundle-json"), false, true),

    // ══ Journey 5 — watch-only export (§4): the md1 chain, shot-bearing. ══
    // 22. bundle --json (multisig) → produces the md1 the restores consume.
    run_step("tut-j5-22-bundle-json", "bundle", None,
        &[Drive::TypeText { flag: "--descriptor", value: MULTISIG_DESC },
          Drive::ToggleBoolean { flag: "--json" }]),
    // 23. restore --format descriptor (chained md1 → bare canonical descriptor).
    run_step("tut-j5-23-restore-descriptor", "restore", Some(HN_RESTORE),
        &[Drive::SelectDropdown { flag: "--template", value: "wsh-sortedmulti" },
          Drive::SelectDropdown { flag: "--format", value: "descriptor" },
          Drive::TypeMd1Chain { flag: "--md1", chain_from: "tut-j5-22-bundle-json" }]),
    // 24. restore --format bitcoin-core (chained md1 → importdescriptors array).
    run_step("tut-j5-24-restore-core", "restore", Some(HN_RESTORE),
        &[Drive::SelectDropdown { flag: "--template", value: "wsh-sortedmulti" },
          Drive::SelectDropdown { flag: "--format", value: "bitcoin-core" },
          Drive::TypeMd1Chain { flag: "--md1", chain_from: "tut-j5-22-bundle-json" }]),
];
