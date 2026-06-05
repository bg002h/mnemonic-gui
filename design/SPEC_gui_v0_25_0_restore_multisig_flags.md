# SPEC — mnemonic-gui v0.25.0 — `restore` multisig-cosigner flags + toolkit-v0.44.0 pin catch-up

**Status:** R0 gate (pre-implementation). MUST converge to 0 Critical / 0 Important before any code.
**Resolves:** `mnemonic-toolkit` FOLLOWUP `gui-restore-multisig-flags-pending-pin-bump`.
**Toolkit source SHA cited:** `mnemonic-toolkit` `4d0523a` (tag `mnemonic-toolkit-v0.44.0`); GUI base `48a3a0f` (tag `mnemonic-gui-v0.24.0`).
**SemVer:** MINOR (additive `schema_mirror` flag-name parity delta → `0.24.0 → 0.25.0`; mirrors the v0.22/v0.23/v0.24 catch-up precedent).

---

## 1. Summary

`mnemonic-toolkit-v0.44.0` added two flags to `mnemonic restore` (multisig-cosigner restore) and made `--from` conditionally-required:

- `--md1 <MD1>` — repeating; the shared wallet-policy md1 card chunk(s). Non-secret (watch-only).
- `--cosigner <COSIGNER>` — repeating; `@N=<mk1|xpub>` cross-check assertion. Non-secret (watch-only).
- `--from` — was unconditionally `required: true`; is now `required_unless_present = "md1"` (toolkit `RestoreArgs.from: Option<String>`). The v0.44.0 `gui-schema` emits `--from` with `required: false`.

The GUI's `RESTORE_FLAGS` (added at v0.24.0, pinned to toolkit v0.43.0) therefore drifts: `schema_mirror` fails with `only in upstream: ["--cosigner", "--md1"]` (RED baseline captured against the v0.44.0 binary). This cycle bumps the toolkit pin v0.43.0 → v0.44.0, mirrors the two flags, models the conditional-requiredness, and ships v0.25.0.

**This is a GUI-repo-only cycle.** No toolkit changes. (See §8 for what is explicitly OUT of scope — notably the v0.3.0-pinned `manual-gui`.)

## 2. Empirical baseline (captured pre-implementation)

- **`schema_mirror` (`+1.94.0`, `MNEMONIC_BIN=`v0.44.0):** `mnemonic_schema_flag_names_match_help_text` FAILS — `schema-mirror drift for mnemonic restore: only in upstream: ["--cosigner", "--md1"]`. This is the whole-mnemonic-surface empirical check: **restore is the ONLY drifted subcommand** between v0.43.0 and v0.44.0 (v0.43.1 was behavior-only/no-flag; v0.44.0 = restore only). No new subcommands, no dropdown-value-enum deltas.
- The `mk`/`ms` `schema_mirror` cells fail ONLY due to stale `$PATH` binaries in the dev shell (the test shells `mk address`/`ms …`); they are not affected by this change. The final GREEN check uses the four pinned binaries (mnemonic 0.44.0 / md 0.6.2 / ms 0.7.0 / mk 0.7.0) via `MNEMONIC_BIN`/`MD_BIN`/`MS_BIN`/`MK_BIN`.
- **Toolchain:** CI uses `dtolnay/rust-toolchain@stable`; the local default `nightly` (1.97.0-nightly) ICEs in codegen — run all GUI builds/tests with `+1.94.0` (or stable). Not a code issue.

## 3. Schema change — `src/schema/mnemonic.rs`

### 3.1 `RESTORE_FLAGS` — add two flags

Insert two `FlagSchema` entries (immediately before the trailing `NO_AUTO_REPAIR_FLAG,` so the global flag stays last, matching every other subcommand array):

```rust
FlagSchema {
    name: "--md1",
    kind: FlagKind::Text,
    required: false,
    repeating: true,
    help: "Multisig-cosigner restore: the shared wallet-policy `md1` card \
           chunk(s). Reconstructs the watch-only multisig descriptor from \
           the md1 alone; wsh / sh(wsh) only. Repeat for chunked cards.",
    secret: false,
    default_value: None,
    global: false,
},
FlagSchema {
    name: "--cosigner",
    kind: FlagKind::Text,
    required: false,
    repeating: true,
    help: "Cross-check assertion (multisig mode): `@N=<mk1-chunk|xpub>` — \
           cosigner at position N is this public key. A mismatch is a hard \
           error (exit 4) unless --allow-mismatch. Watch-only (non-secret).",
    secret: false,
    default_value: None,
    global: false,
},
```

Both `secret: false` (toolkit `flag_is_secret` → false; watch-only). The only secret-bearing restore flags remain `--passphrase`/`--passphrase-stdin` → **no `SECRET_NODE_TYPES`/`SECRET_SLOT_SUBKEYS` delta**; `schema_mirror_secret_drift` unchanged. `repeating: true` mirrors the toolkit `Vec<String>` clap args (same shape as the existing repeating card flags `--ms1`/`--mk1`/`--md1` in verify-bundle/repair, which are `required:false, repeating:true`).

### 3.2 `RESTORE_FLAGS` — flip `--from`

`--from`: `required: true` → `required: false` (the toolkit v0.44.0 `gui-schema` emits `--from required:false`; `required_unless_present="md1"`).

### 3.3 `restore` `SubcommandSchema` — wire the conditional fn

`conditional: None` → `conditional: Some(crate::form::conditional::restore)`. Update the adjacent comment (currently *"gui-schema emits `conditional_rules: []` for restore → conditional: None"*) to record the v0.25.0 GUI-authored at-least-one rule (see §4 + the repair/inspect precedent).

## 4. Conditional engine — `src/form/conditional.rs` (DECISION: faithful mirror, "Option B")

**Decision.** Model `--from required_unless_present="md1"` as a GUI-authored at-least-one rule, rather than leaving `--from` flat-optional. Add:

```rust
/// `restore` subcommand conditionals (v0.25.0 / toolkit v0.44.0).
///
/// Mirrors `RestoreArgs.from` `required_unless_present = "md1"`
/// (`crates/mnemonic-toolkit/src/cmd/restore.rs`): `--from` is required
/// UNLESS `--md1` is supplied (multisig mode), where `--from` is an
/// optional own-cosigner cross-check. When neither is present the toolkit
/// errors at run time; the GUI surfaces the requirement up-front.
///
/// NB: the toolkit `gui-schema` `conditional_rules` projection is a
/// hand-encoded allowlist (`build_subcommand_conditional_rules`,
/// `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:336-345`) with arms only
/// for bundle/verify-bundle/export-wallet/convert/derive-child/compare-cost;
/// restore falls through to `_ => Vec::new()`, so it emits
/// `conditional_rules: []` despite carrying a real `required_unless_present`
/// clap attribute. This rule is therefore GUI-authored and NOT covered by
/// the `gui_schema_conditional_drift` gate (which `continue`s on empty-rule
/// subcommands). The closest mirrored precedent is `verify-bundle`'s
/// `required_unless_present` modeling (`conditional.rs:421-425`, Required on
/// Not(present)); the `repair`/`inspect` at-least-one rules are the broader
/// GUI-authored-without-toolkit-emission precedent. Promotion to a
/// toolkit-emitted + drift-gated rule is tracked as a toolkit FOLLOWUP (§7).
pub fn restore(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    if !state.has_value("--md1") {
        vis.push(("--from", Visibility::Required));
    }
    vis
}
```

**Rationale / why this is gate-safe and not Option A (flat `required:false`, `conditional:None`):**

- `required`/`Visibility::Required` drives a **visual marker only** (`main.rs:593`, `form/widget.rs:81,386`), not a hard submit-block. So this is purely better up-front guidance; it never blocks a valid invocation, and the toolkit remains the authoritative gate.
- **Prior art exists** (the advisor's decision criterion): `repair`/`inspect` (`conditional.rs::three_way_card_at_least_one`) are GUI-authored at-least-one rules for which the toolkit `gui-schema` emits `conditional_rules: []`. `restore` is the same shape (at-least-one of {`--from`, `--md1`}).
- **Drift gate is satisfied:** `gui_schema_conditional_drift.rs` iterates the toolkit's emitted rules and `continue`s on empty (`:228`); restore emits `[]` → skipped. There is **no orphan-direction check** (a GUI fn without a toolkit rule is permitted — `repair`/`inspect` prove it). Verified against the v0.44.0 binary's `gui-schema`.

## 5. Pin + version bump

- `Cargo.toml` `[dependencies].mnemonic-toolkit.tag`: `mnemonic-toolkit-v0.43.0` → `v0.44.0`.
- `pinned-upstream.toml` `[mnemonic].tag`: `mnemonic-toolkit-v0.43.0` → `v0.44.0` (+ refresh the documentary `[md]/[ms]/[mk]` comment lines only if stale — md 0.6.2 / ms 0.7.0 / mk 0.7.0 are current; no bump). `pin_coherence` asserts the two toolkit pins agree.
- `Cargo.lock`: regenerated (`cargo +1.94.0 update -p mnemonic-toolkit --precise` via the git tag; in practice `cargo build` rewrites it) — **stage it**.
- `Cargo.toml` `version`: `0.24.0` → `0.25.0`.
- `src/schema/mnemonic.rs` module-doc header + any `pinned_version` banner string: bump `0.43.0`/`v0.43.0` → `0.44.0`/`v0.44.0` (grep for the literal; the v0.24.0 entry bumped these).
- `CHANGELOG.md`: new `## mnemonic-gui [0.25.0] — <date>` entry (extends the v0.24.0 restore entry).

## 6. Tests

- **`tests/conditional_visibility.rs`:** add restore cells mirroring the `repair`/`inspect` cell shape:
  - `restore` + empty `FormState` → `--from` is `Visibility::Required`.
  - `restore` + `--md1` populated → `--from` NOT in the override map (falls through to `Visible`).
  - **(R0 Minor 1)** add `restore` to the `coverage_all_constrained_subcommands_have_conditional_fn` `is_some()` allowlist (`:316-345`) — restore is now a constrained subcommand, so the coverage guard must list it (suite stays GREEN either way, but this keeps the guard meaningful).
- **`tests/schema_mirror.rs`:** no test edit; goes GREEN once §3 lands (run with all four pinned `*_BIN`).
- **`tests/gui_schema_conditional_drift.rs`:** no edit; restore stays skipped (empty toolkit rules). Run it (with `MNEMONIC_BIN`) to confirm no regression.
- **`tests/pin_coherence.rs`:** GREEN after §5 (both pins v0.44.0).
- Full `cargo +1.94.0 test -p mnemonic-gui --no-fail-fast` with all four `*_BIN` → GREEN. Plus `cargo +1.94.0 clippy -p mnemonic-gui --all-targets`.
- **Widget check:** `--md1`/`--cosigner` are repeating-`Text` flags → auto-rendered by the schema-driven repeating-field widget (same as existing repeating card flags); confirm no kittest restore-form snapshot pins the flag list. `--cosigner @N=` is a plain text value (no slot-editor coupling — slot-editor is `--slot`-only; `SECRET_SLOT_SUBKEYS` untouched).

## 7. Lockstep / cross-repo ship steps

- **Toolkit repo (`mnemonic-toolkit`):** flip FOLLOWUP `gui-restore-multisig-flags-pending-pin-bump` `Status: open → resolved (mnemonic-gui-v0.25.0)`. This is a separate toolkit-repo commit (the FOLLOWUP registry lives there) — explicit in the ship checklist (do not forget the cross-repo status flip).
- **Toolkit FOLLOWUP (NEW):** file `gui-schema-restore-required-unless-md1-projection` — the toolkit `gui-schema` `conditional_rules` projection does not emit restore's `--from required_unless_present="md1"`, so the GUI rule (§4) is GUI-authored/ungated (like `repair`/`inspect`). Promote to a toolkit-emitted + drift-gated rule in a future toolkit cycle. (Records the advisor's "don't let it be silent" consequence as a tracked item.)
- **`mnemonic-gui/CLAUDE.md`:** add a companion note if the existing schema-mirror-lockstep section calls for one (check at ship; the v0.22/23/24 cycles set the convention).

## 8. Out of scope (explicit)

- **`manual-gui`** (`mnemonic-toolkit/docs/manual-gui/`): pinned to `mnemonic-gui-v0.3.0` (single commit `a17dfbd`) — a separate, long-deferred "v1.1" manual track. The prior catch-ups (v0.22/v0.23/v0.24) did not touch it. Its coverage test `manual_anchor_coverage.rs` is `#[ignore]`-gated (needs `$MANUAL_GUI_HTML_PATH`), and the toolkit-side `check_gui_schema_coverage.py` runs against the v0.3.0 pin — neither fires on this change. Adding restore `--md1`/`--cosigner` manual-gui anchors belongs to a future manual-gui re-pin cycle, not here.
- **Bespoke multisig restore widgets** (a position-picker for `@N=`, an md1-chunk repeater UX): the schema-driven repeating-`Text` widget is sufficient and matches the FOLLOWUP. UX polish → future.
- **Toolkit changes** of any kind (other than the two ship-time FOLLOWUP edits in §7).

## 9. Phased plan

- **Phase 1 (RED):** add the two `conditional_visibility.rs` restore cells (assert `--from` Required when no `--md1`; not-Required when `--md1` set). They fail (no restore conditional fn yet). Confirm RED for the right reason. (`schema_mirror` RED baseline already captured, §2.)
- **Phase 2 (GREEN):** §3 schema (two flags + `--from` flip + `conditional: Some(restore)`), §4 `restore` conditional fn. Run `conditional_visibility` + `schema_mirror` (4 pinned bins) + `gui_schema_conditional_drift` → GREEN. Per-phase opus review; persist to `design/agent-reports/`.
- **Phase 3 (pin/version/release):** §5 pin + version + CHANGELOG; `pin_coherence` GREEN; full suite + clippy GREEN. Per-phase review.
- **Phase 4 (ship):** §7 cross-repo FOLLOWUP flips (toolkit repo) + new FOLLOWUP + CLAUDE.md note; tag `mnemonic-gui-v0.25.0`; push; watch CI (`build`, `schema-mirror`).

## 10. Risk / SemVer

Low-risk, additive catch-up (two flags + one at-least-one visual rule + pin). MINOR per the new-flag/`schema_mirror`-delta precedent (v0.22/v0.23/v0.24). No breaking change; no secret-projection change; no new dropdown enum.
