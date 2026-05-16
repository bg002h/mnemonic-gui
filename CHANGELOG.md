# Changelog

All notable changes to `mnemonic-gui` are recorded here. Follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

## [0.5.0] — 2026-05-16

### Added — SPEC §6.10 conditional-applicability consumer + drift gate

Consumes `mnemonic-toolkit v0.16.0`'s new
`mnemonic gui-schema` JSON v2 `conditional_rules` projection.
The toolkit emits machine-readable per-subcommand mutex/conditional
rules; the GUI maps them onto its per-frame visibility computation
and enforces parity via a drift-gate test.

#### Motivating bug closed

The GUI bundle form's default state (template `bip84`, single-sig)
previously emitted `--threshold 1 --multisig-path-family bip48` —
the CLI rejected the argv with the SPEC §6.6 byte-exact errors
`THRESHOLD_WITHOUT_MULTISIG` + `PATH_FAMILY_WITHOUT_MULTISIG`.
Three stacked defects:

1. `main.rs:203` pre-seeded `--multisig-path-family = bip87`
   unconditionally.
2. `assemble_argv` ignored the existing `Visibility` infrastructure.
3. No machine-readable conditional-applicability metadata flowed
   from toolkit to GUI; the GUI's hand-coded `conditional.rs` was
   the only source-of-truth and had drifted behind the CLI rule
   surface.

All three closed in v0.5.0.

#### Implementation surfaces (this cycle)

- **`src/schema_check.rs`** — `parse_gui_schema_conditional_rules`
  fn requiring `version >= 2`; relaxed `parse_gui_schema_json`
  version gate from `!= 1` to `< 1` (additive bump policy).
- **`src/form/conditional.rs`** — ~14 new visibility rules
  across `bundle` / `verify-bundle` / `export-wallet` /
  `derive-child`. Module-level `SINGLE_SIG_TEMPLATES` and
  `TAPROOT_INTERNAL_KEY_TEMPLATES` constants mirror the
  toolkit's `CliTemplate::is_multisig()` source-of-truth at
  `mnemonic-toolkit/src/template.rs:46-56` (parity enforced by
  the drift gate).
- **`src/form/invocation.rs`** — visibility gate at the TOP of
  the per-flag iteration loop. Both `Hidden` and `Disabled`
  suppress emission; `Required` does not (decorative marker
  only). Slot emission is exempt per SPEC §6.10 v1 scope.
- **`src/main.rs`** — removed `--multisig-path-family = bip87`
  default seed (bug-class default). A future cycle may
  re-introduce a template-aware seed; tracked at FOLLOWUP
  `gui-default-form-state-template-aware-seed`.
- **`.github/workflows/schema-mirror.yml`** — CI smoke steps
  for all four CLIs relaxed from `version == 1` to
  `version >= 1`.

#### Latent-bug fix

The visibility gate at `assemble_argv` also closes a pre-v0.5.0
latent bug: typed-then-mutex-disabled secret values (e.g., user
types `--passphrase=foo` then sets `--passphrase-stdin`) are now
suppressed at argv emission, preventing clap's `conflicts_with`
rejection downstream.

#### Drift gate

`tests/gui_schema_conditional_drift.rs` (NEW): shells out to
`<MNEMONIC_BIN> gui-schema`, parses the v2 `conditional_rules`,
synthesizes an exemplar `FormState` per rule's predicate,
invokes the corresponding hand-coded `SubcommandSchema.conditional`
fn, asserts the returned `FlagVisibility` contains the rule's
declared `(flag, visibility)`. Failure messages cite the rule's
`rationale` + `spec_ref` for forensic clarity.

Drift-gate cross-validation (§5.3): planting a divergence (e.g.,
commenting out the derive-child `--dice-sides Required` rule)
produces a failure of the form

    assertion `left == right` failed: drift in subcommand `derive-child`:
      rule rationale: --dice-sides is required when --application is set to dice.
      spec_ref: cmd/derive_child.rs clap-derive required_if_eq
      predicate: DropdownValueIn { flag: "--application", values: ["dice"] }
      target flag: --dice-sides
      expected visibility: Required
      actual visibility:   Visible

### Verification

- `cargo test --release` with all `*_BIN` env vars: 187 passed,
  0 failed, 1 ignored (the v0.4.3 baseline +30 new cells across
  P1/P2/P3/P4/P5).
- `default_bundle_form_state_cli_accepts` smoke test: passes
  (CLI exit 0 against the v0.16.0 toolkit binary; the motivating
  bug no longer reproduces).
- End-of-cycle opus reviewer-loop: R1 returned FOLD with 1
  Critical (CI workflow version gate) + 1 Important (missing
  companion FOLLOWUP entry); both folded in
  `6c2d019`. R2 returned PASS (0C / 0I).

### Companion / lockstep

- `bg002h/mnemonic-toolkit v0.16.0` (commit `519bcfc`, tag
  `mnemonic-toolkit-v0.16.0`) ships the producer side. The GUI's
  toolkit-dep pin bumps from `v0.15.0` (post-v0.4.3 catchup) to
  `v0.16.0` in this release.
- Plan + SPEC: `mnemonic-toolkit/design/
  IMPLEMENTATION_PLAN_gui_conditional_applicability_v1.md`
  + `mnemonic-toolkit/design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.

### Predecessor

- `mnemonic-gui v0.4.3` (toolkit v0.15.0 wire-format catchup;
  scope-isolated). v0.5.0 builds atop v0.4.3.

## [0.4.3] — 2026-05-16

### Scope-isolation catchup — bump toolkit dep to v0.15.0

`Cargo.toml` + `pinned-upstream.toml` pins bumped from
`mnemonic-toolkit-v0.14.2` to `mnemonic-toolkit-v0.15.0`. v0.15.0
was the toolkit's md-codec catchup release (md-codec 0.16.1 →
0.33.1 + mk-codec 0.2.1 → 0.3.0 + ms-codec git → 0.1.3); its
release commit (`5d92768`) describes the change as a wire-format
clean break (v0.14.x bundles forward-incompatible).

This release is a scope-isolated prerequisite for the in-flight
GUI conditional-applicability v1 cycle (toolkit v0.16.0 + GUI
v0.5.0 lockstep). Cutting v0.4.3 ahead of v0.5.0 ensures that any
§5.1 manual-reproduction failure in the v0.5.0 cycle is
attributable to v0.16.0 conditional-applicability work, not to
v0.15.0 wire-format drift. Architect-review rationale recorded in
the toolkit's plan-doc at
`design/IMPLEMENTATION_PLAN_gui_conditional_applicability_v1.md`
(top revision note + §4 prerequisite gate).

### Verification

- `cargo build --release` clean at v0.15.0 pin.
- Full `cargo test --release` green (156 passed, 0 failed,
  1 `#[ignore]`-gated sibling-dep test as expected) with all
  `*_BIN` env vars set (`MNEMONIC_BIN`, `MD_BIN`, `MS_BIN`,
  `MK_BIN`).
- Coldcard fixture (`tests/fixtures/coldcard_generic_bip84_mainnet.json`)
  verified **byte-identical** between v0.14.0 vendored copy and
  v0.15.0 master via `diff -q`; no re-vendor needed.

### Companion

`mnemonic-toolkit v0.15.0` (commit `5d92768`, tag
`mnemonic-toolkit-v0.15.0`) is the catchup target. No
toolkit-side change in this release.

## [0.4.2] — 2026-05-16

### Bug fix — bump toolkit dep to v0.14.2 (slip39 lib-internal mlock cfg-gate)

v0.4.1 bumped the toolkit dep to v0.14.1 expecting Windows builds
to pass. They didn't: v0.14.1's lib-cross-platform Windows CI job
caught FOUR additional `crate::mlock::*` call sites inside
`src/slip39/mod.rs` that v0.14.1's `lib.rs` cfg-gate missed.
mnemonic-gui v0.4.1 Windows CI (run 25952017502) inherited the
same failure: `error[E0433]: failed to resolve: could not find
'mlock' in the crate root` at slip39/mod.rs:159 + :314.

v0.14.2 cfg-gates the four slip39 call sites; this release just
pulls in the fix.

## [0.4.1] — 2026-05-16

### Bug fix — bump toolkit dep to v0.14.1 (Windows-compatible lib)

Cargo.toml + pinned-upstream.toml pins bumped from
`mnemonic-toolkit-v0.14.0` to `mnemonic-toolkit-v0.14.1`. v0.14.1
cfg-gates `pub mod mlock` behind `#[cfg(unix)]`, unblocking the
GUI's Windows build matrix. v0.4.0 CI run 25951528124 failed on
`x86_64-pc-windows-msvc` with `cannot find function 'sysconf' /
'mlock' / 'munlock' in crate 'libc'` — the v0.14.1 fix lands the
gate; this release just pulls it in.

### Docs / hygiene — architect-audit Important fixes

The v0.4.0 architect-audit (post-release LOCK pass) surfaced 2
Criticals (resolved at toolkit v0.14.1 + install.sh follow-up patch)
and 5 Importants. Folding the GUI-side Importants now:

- **NEW**: `tests/fixtures/SOURCE.md` — documents the provenance of
  the vendored Coldcard fixture (originally vendored from toolkit
  v0.14.0 in v0.4.0 to decouple from `MNEMONIC_GUI_UPSTREAM_ROOT`).
  Records re-vendor procedure for future toolkit cycles that revise
  Coldcard emission output.
- **DOCS**: `CHANGELOG.md` v0.4.0 section — replace the misnamed
  `CANONICAL_FALLBACK_*` reference (which exists only in v0.3.3
  historical context) with the live module path
  `v0_3_canonical_fallback::SECRET_NODE_TYPES` /
  `SECRET_SLOT_SUBKEYS`.

### Companion

`mnemonic-toolkit v0.14.1` (commit `bf54505`, tag
`mnemonic-toolkit-v0.14.1`) lands the actual Windows fix.
`scripts/install.sh` on the toolkit side will flip
`mnemonic-gui`'s `cratesio=yes` to `no` so the install path no
longer resolves `cargo install mnemonic-gui` to the leaky v0.3.1
on crates.io.

## [0.4.0] — 2026-05-16

### Structural fix — retire `build.rs` source-walker; consume `mnemonic_toolkit::secret_taxonomy`

Architect-recommended Option A from the cross-repo FOLLOWUP entries
(`secret-taxonomy-public-api-promotion` on the toolkit side,
`secret-taxonomy-public-api-consumption` here). Replaces v0.3.3's
tactical patch (committed `CANONICAL_FALLBACK_*` arrays in
`build.rs`) with the durable architectural fix.

### What changed

- **NEW**: `mnemonic-toolkit = { git = "...", tag =
  "mnemonic-toolkit-v0.14.0" }` added to `Cargo.toml` `[dependencies]`.
  Pulls in the toolkit's new `pub mod secret_taxonomy` module which
  exports `SECRET_NODE_TYPES` + `SECRET_SLOT_SUBKEYS` as compile-time
  contracts.

- **DELETED**: `build.rs` (the entire syn-based upstream-source walker).
  Was previously the source of the v0.3.0..v0.3.2 HIGH-severity bug:
  cargo-install sandboxes had no adjacent toolkit checkout, so the
  walker fell back to a `write_stub()` that emitted empty `&[]`
  arrays, silently disabling `persistence::redact_for_persistence`
  and leaking BIP-39 phrases to `state.json` in plaintext.

- **DELETED**: `tests/secrets_canonical_fallback.rs` (the v0.3.3 drift
  gate). Superseded by the toolkit's own per-variant parity tests at
  build time (`mnemonic-toolkit-v0.14.0` `secret_taxonomy_parity_tests`).

- **DELETED**: `[build-dependencies]` block in `Cargo.toml`. The
  `syn`/`quote`/`proc-macro2` crates are still in `[dev-dependencies]`
  because `tests/schema_mirror.rs` continues to re-parse upstream
  clap-derive flag surfaces for the flag-name parity gate.

- **CHANGED**: `src/secrets.rs`. The `include!(concat!(env!("OUT_DIR"),
  "/secrets_generated.rs"));` codegen line is replaced by:
  ```rust
  pub use mnemonic_toolkit::secret_taxonomy::{SECRET_NODE_TYPES, SECRET_SLOT_SUBKEYS};
  ```
  Downstream consumers (`persistence::redact_for_persistence`,
  `secrets::slot_subkey_is_secret`, etc.) consume the toolkit-imported
  constants unchanged.

- **NEW**: compile-time supply-chain guard. A `const _: () =
  assert!(...)` block in `src/secrets.rs` asserts that the imported
  `SECRET_*` arrays equal the v0.3.3-committed snapshot (preserved as
  `v0_3_canonical_fallback::SECRET_NODE_TYPES` /
  `v0_3_canonical_fallback::SECRET_SLOT_SUBKEYS`). Catches a
  supply-chain class of regression where the toolkit dep tag could
  resolve to a build with different `SECRET_*` arrays. Maintainers
  who deliberately bump the SECRET_* set must also update the
  snapshot (or the build fails). One-cycle belt-and-suspenders per the
  architect's recommendation; will be removed in v0.5.0.

- **NEW**: `tests/secret_taxonomy_pin.rs` — four runtime backstop
  tests asserting `SECRET_*` non-empty and contains the four
  BIP-39-class entries (`phrase`, `entropy`, `xprv`, `wif`). Runs
  under default `cargo test` with no env-var requirements; replaces
  the `secrets_canonical_fallback.rs` `#[ignore]`-gated drift gate.

- **CHANGED**: `.github/workflows/schema-mirror.yml` — the
  `cargo-test-secrets-canonical-fallback` step is retired. Backstop
  testing now runs as part of `cargo-test-full-suite`.

- **CHANGED**: `pinned-upstream.toml` — `[mnemonic].tag` is now
  documentary only. Load-bearing toolkit version pin lives in
  `Cargo.toml`'s `[dependencies]` table; both should bump in lockstep.

### Why this matters

End users running `cargo install --git mnemonic-gui` get a binary
that **correctly redacts secret slot rows and node-value composites
before persisting session state**, by compile-time guarantee. There
is no longer a code path through which the redaction filter can
silently become a no-op due to missing build-time environment setup.
The v0.3.0..v0.3.2 stub-fallback class of bug is structurally
impossible in v0.4.0+.

### User action

Re-install via:
```
./scripts/install.sh mnemonic-gui --from-git --force
```
Existing `~/.config/mnemonic-gui/state.json` files written by v0.3.3
are safe (v0.3.3 also redacted correctly). Files written by
v0.3.0..v0.3.2 may contain secret material — delete them if you used
any of those versions with `--slot @N.<phrase|entropy|xprv|wif>=…` or
`--from <phrase|entropy|xprv|wif|ms1|bip38|electrum-phrase>=…`:
```
rm -i ~/.config/mnemonic-gui/state.json
```

### Closes

`secret-taxonomy-public-api-consumption` (GUI half of the cross-repo
lockstep work). Companion: `mnemonic-toolkit` v0.14.0 (tag
`mnemonic-toolkit-v0.14.0`, commit `1a52612`) closed
`secret-taxonomy-public-api-promotion`.

### Reviewer trail

- Architect dispatch (opus): produced the Option A migration sketch
  in the cross-repo FOLLOWUP entries.
- R1 toolkit review (opus): caught a Critical (the original
  `every_*_variant` closure+driver pattern was not load-bearing —
  arm-only extension could escape parity tests).
- R2 toolkit review (opus): LOCK with-1-folded; the v0.14.0 fix was
  restructured around a declarative macro pattern that ties the
  variant array and exhaustiveness check to a single input.
- R1 GUI review (opus): caught a Critical missed deletion —
  `tests/schema_mirror.rs::source_audit` mod (250 LOC) +
  `tests/fixtures/mutated_convert.rs` were the OLDER Phase 7 audit
  re-doing the same source-walker pattern that this release's
  CHANGELOG claimed to retire, against a stale v0.13.0 toolkit
  clone. Plus five Important findings (stale comments in
  `src/persistence.rs` / `src/secrets.rs:138` / `tests/persistence.rs:303` /
  `tests/secrets.rs:241`; dead `[upstream]` table in
  `pinned-upstream.toml`; orphan `MNEMONIC_GUI_UPSTREAM_ROOT` coupling
  in `tests/runner_integration.rs` — fixture is now vendored at
  `tests/fixtures/coldcard_generic_bip84_mainnet.json`; misleading
  initial reviewer-trail framing in this CHANGELOG; lingering env
  var on `cargo-test-full-suite`). All six folded in the same commit.

## [0.3.3] — 2026-05-15

### Security fix — persistence-redaction bypass in cargo-install builds

**Severity: HIGH.** Affects v0.3.0, v0.3.1, v0.3.2.

`build.rs` generates `SECRET_NODE_TYPES` and `SECRET_SLOT_SUBKEYS` by
parsing the upstream `mnemonic-toolkit` source tree's
`NodeType::is_secret_bearing()` and
`SlotSubkey::is_secret_bearing()` impls. When the upstream source is
unresolvable at build time (Step 4 of the SPEC §B.11 resolution chain),
the prior `write_stub` emitted **empty arrays**:

```rust
pub const SECRET_NODE_TYPES: &[&str] = &[];
pub const SECRET_SLOT_SUBKEYS: &[&str] = &[];
```

This is the default outcome for **every `cargo install --git
mnemonic-gui` invocation** — `cargo install` runs `build.rs` in an
isolated sandbox with no adjacent toolkit checkout, so resolution
chain Steps 1/2/3 all fail and the stub fallback is used unless the
user explicitly sets `MNEMONIC_GUI_UPSTREAM_ROOT` or
`MNEMONIC_GUI_ALLOW_UPSTREAM_CLONE=1` before running cargo install.

`persistence::redact_for_persistence` filters slot rows by
`SECRET_SLOT_SUBKEYS.contains(...)` and `NodeValueComposite` entries
by `SECRET_NODE_TYPES.contains(...)`. With both arrays empty, the
filter is a no-op. The result: a BIP-39 seed phrase typed into a slot
field (e.g., `--slot @0.phrase=<phrase>`) or into a NodeValueComposite
field (e.g., `--from phrase=<phrase>`) **persists to `state.json` in
plaintext** when the GUI saves session state on exit.

The hand-maintained `SECRET_FLAG_NAMES` constant (`--passphrase`,
`--bip38-passphrase`, `--passphrase-stdin`) was NOT affected; only the
build-generated arrays were empty under the install-path build.

### Fix

`build.rs::write_stub` now emits the **canonical** secret-class sets
as a committed-in-source fallback, NOT empty arrays:

```rust
const CANONICAL_FALLBACK_NODE_TYPES: &[&str] = &[
    "phrase", "entropy", "xprv", "wif",
    "ms1", "bip38", "electrum-phrase",
];

const CANONICAL_FALLBACK_SLOT_SUBKEYS: &[&str] =
    &["phrase", "entropy", "xprv", "wif"];
```

These mirror the upstream `is_secret_bearing()` match-arm sets at
`mnemonic-toolkit@mnemonic-toolkit-v0.13.1`
(`crates/mnemonic-toolkit/src/cmd/convert.rs:85` +
`crates/mnemonic-toolkit/src/slot_input.rs:60`). When upstream source
IS resolvable, build.rs regenerates from source as before; when not,
the fallback ships canonical values rather than empty placeholders.

### Drift gate

New `tests/secrets_canonical_fallback.rs` re-parses the upstream
`is_secret_bearing()` impls and asserts set-equality against the
committed fallback arrays. `#[ignore]`-gated by default (needs
`MNEMONIC_GUI_UPSTREAM_ROOT`). Schema-mirror CI workflow runs it via
`cargo test --test secrets_canonical_fallback -- --include-ignored`;
any upstream change to the secret-class sets that isn't mirrored here
fails CI immediately.

A second (always-on) test asserts both fallback arrays are non-empty
and contain the four BIP-39-class items (`phrase`, `entropy`, `xprv`,
`wif`) — a backstop against future regression to `&[]`.

### Verification

Reproduction (before fix, post-`cargo install --git` build):

```
SlotRow { subkey: Phrase, value: "abandon abandon ..." }
→ save() → state.json contains "subkey": "Phrase",
                                "value": "abandon abandon ..."
```

After fix:

```
SlotRow { subkey: Phrase, value: "abandon abandon ..." } → save()
→ state.json contains "rows": []   ← phrase row stripped
```

### User action

Run `./scripts/install.sh mnemonic-gui --from-git --force` (or
re-install via your preferred path) to pick up v0.3.3. If you have an
existing `~/.config/mnemonic-gui/state.json` written by v0.3.0..v0.3.2
that may contain secret material, delete it manually before launching
v0.3.3:

```
rm -i ~/.config/mnemonic-gui/state.json
```

### CVE / GHSA

Recommend filing a GHSA advisory after release tag is live. Affected
versions: v0.3.0, v0.3.1, v0.3.2. Fixed in: v0.3.3.

## [0.3.2] — 2026-05-15

Patch: replace 4 non-ASCII glyphs in user-visible schema strings with
ASCII equivalents — matches the project's existing ASCII-first
convention (per `src/form/widget.rs:36` on the `?` button). User
reported a missing-glyph (open-square) render in the `mnemonic
final-word` subcommand dropdown; defensive sweep covers the other
three same-class chars that may also lack font support on some
systems.

Replacements:
- `—` (U+2014 EM DASH) -> `--`
- `→` (U+2192 RIGHTWARDS ARROW) -> `->`
- `↔` (U+2194 LEFT RIGHT ARROW) -> `<->`
- `≤` (U+2264 LESS-THAN OR EQUAL TO) -> `<=`

Affected user-visible strings (14 lines across 4 schema files):
- `mnemonic`: 5 help/human_name strings (final-word, slot threshold
  ×2, slot help paragraph, output-path help)
- `md`: 4 human_names (Encode/Decode/Verify/Compile)
- `ms`: 3 human_names (Encode/Decode/Verify)
- `mk`: 2 human_names (Encode/Decode)

Schema-mirror invariant: help-text content is NOT gated by the
bidirectional mirror (only flag presence/absence is), so this patch
ships independently of the toolkit. All 16 schema_mirror tests + full
suite green with proper sibling-binary env setup
(MNEMONIC_GUI_UPSTREAM_ROOT + MNEMONIC_BIN / MD_BIN / MS_BIN /
MK_BIN).

## [0.3.1] — 2026-05-15

(Backfill — v0.3.1 shipped at commit 407c5ef but its CHANGELOG section
was omitted from that release commit.)

Patch: GUI-side track of the manual-gui v1.0 cycle (lockstep with
`mnemonic-toolkit-v0.13.0` + manual-gui-v1.0.0 / v1.0.1 tags).

### Added

- `src/help/` module — `manual_url_for_subcommand` /
  `manual_url_for_dropdown` / `manual_url_for_composite` helpers, with
  `MANUAL_BASE_URL` build-time overridable via the
  `MNEMONIC_GUI_MANUAL_BASE_URL` env var (default
  `https://bg002h.github.io/mnemonic-toolkit/manual-gui/`).
- 91 `?` help-icon buttons across the form scaffolding
  (per-subcommand, per-Dropdown, per-NodeValueComposite,
  per-repeating-field) — SPEC §1.6 Option C selective placement.
- `tests/widget_help_icon.rs` kittest cell: clicks the `?` next to a
  flag, asserts the right tab/manual URL is targeted via
  `ctx.open_url`.

## [0.3.0] — 2026-05-14

v0.3 catches the GUI up to `mnemonic-toolkit-v0.13.0`. Five new
`mnemonic` subcommand surfaces (`slip39-split`, `slip39-combine`,
`seed-xor-split`, `seed-xor-combine`, `final-word`) close the
v0.11..v0.13 toolkit feature gap. A v0.10..v0.13 flag-drift
correction closes the schema-mirror-invariant breach for `bundle` /
`verify-bundle` / `convert` / `derive-child` (4 `*-stdin` flags
that shipped toolkit-side without companion GUI PRs). The
schema-mirror gate now prefers `gui-schema` JSON over `--help`
regex (required for flattened nested-subcommand names like
`seed-xor-split` that clap doesn't expose at the top level), and
`assemble_argv` now correctly emits repeating-secret flags (latent
v0.2 bug that affected `--ms1`/`--mk1`/`--md1` and was surfaced by
`--share`).

### Added — 5 new mnemonic subcommand surfaces

- `slip39-split` (8 flags) + `slip39-combine` (6 flags) — Trezor
  SLIP-39 K-of-N share splitter (toolkit v0.13.0 unblocks the
  GUI-side companion FOLLOWUP).
- `seed-xor-split` (5 flags) + `seed-xor-combine` (4 flags) —
  Coldcard all-or-nothing BIP-39 ↔ BIP-39 XOR splitter (toolkit
  v0.12.0).
- `final-word` (3 flags) — BIP-39 N-1 phrase → candidate Nth-word
  set (toolkit v0.11.0).
- 3 new schema constants (`SLIP39_FROM_NODES`, `SLIP39_TO_SHAPES`,
  `PHRASE_ONLY`).
- New `FormState::composite_node()` helper for NodeValueComposite
  value-inspect (slip39-split `--language` hide-when-entropy).
- 2 new conditional fns (`slip39_split`, `slip39_combine`).
- 5 new egui_kittest cells (one per new subcommand) +
  14 new conditional-visibility cells.

### Added — v0.10..v0.13 drift correction

Closes the schema-mirror-invariant breach (FOLLOWUPS.md
`mnemonic-gui-schema-mirror`): 4 toolkit cycles shipped new
`*-stdin` flags without companion `mnemonic-gui` PRs.

- `BUNDLE_FLAGS` / `VERIFY_BUNDLE_FLAGS` / `DERIVE_CHILD_FLAGS`:
  `--passphrase-stdin` (clap `conflicts_with = "passphrase"`).
- `CONVERT_FLAGS`: `--bip38-passphrase-stdin` (clap
  `conflicts_with = "bip38_passphrase"`).
- `src/form/conditional.rs`: `bundle` / `verify_bundle` / `convert`
  fns extended with passphrase XOR clauses; NEW `derive_child` fn
  (was `None` — gained conflict at v0.13.0); stale "no conditional
  fn needed" comment deleted.
- `derive-child` SubcommandSchema `conditional: None` →
  `Some(crate::form::conditional::derive_child)`.

### Changed — pinned-upstream + workflow

- `pinned-upstream.toml`: `[mnemonic].tag`
  `mnemonic-toolkit-v0.9.0` → `mnemonic-toolkit-v0.13.0` (sibling
  pins `md` / `ms` / `mk` unchanged).
- `.github/workflows/schema-mirror.yml`: install + clone steps
  bump to `v0.13.0`.
- `src/schema/mnemonic.rs`: `pinned_version: "mnemonic 0.9.0"` →
  `"mnemonic 0.13.0"`.
- `tests/schema_mirror.rs::ci_workflow_snapshot::required_tags`
  first entry bumped.

### Fixed — latent v0.2 bugs

- `src/form/invocation.rs::assemble_argv`: repeating-secret flags
  (`--ms1` / `--mk1` / `--md1` from v0.2; `--share` from v0.3) now
  emit N occurrences via `state.values` iteration. The pre-v0.3
  path only consulted `state.secret_widgets` (singular BTreeMap
  lookup) and silently emitted at most one token. Trade-off:
  zeroize-on-drop still applies per-widget; the values-map
  `String` copies during emission are plain heap allocations.
- `tests/schema_mirror.rs::assert_schema_matches_help`: prefer
  `gui-schema` JSON over `--help` regex (required for flattened
  nested-subcommand names like `seed-xor-split` that clap's
  top-level `--help` doesn't recognize). Falls back to `--help`
  regex for `gui-schema-capable = false` CLIs.

### Design

- `design/PLAN_v0_3.md` — 3-section reviewer-LOCKed plan
  (brainstorming + SPEC + implementation plan) with P0 drift-fold
  amendment, persisted from the plan-mode artifact directory to the
  in-repo design archive (matches v0.1 / v0.2 convention).

## [0.2.0] — 2026-05-12

v0.2 expands the GUI surface from the v0.1 baseline (one CLI's
`mnemonic` subcommand surface) to all four sibling CLIs of the
m-format constellation: `mnemonic`, `md`, `ms`, `mk` — 15 additional
subcommands across 4 binaries. The release also lands the SPEC §7
machine-readable schema contract (`<cli> gui-schema`), an OS-snapshot
occlusion baseline (Phase B.2), a secret-buffer abstraction
(Phase B.1), an egui_kittest harness scaffold (Phase A.3), and the
doubled-prefix release-artifact fix (Phase A.1).

### Added — sibling-CLI surface (Phase D)

- `ms` subcommands: `decode`, `encode`, `mnemonic`, `wordlist`
  (4 new schemas in `src/schema/ms.rs`).
- `mk` subcommands: `decode`, `encode`, `derive`, `fingerprint`
  (4 new schemas in `src/schema/mk.rs`).
- `md` subcommands: `decode`, `encode`, `compile`, `derive`,
  `address`, `policy`, `id` (7 new schemas in `src/schema/md.rs`).
- `src/form/conditional.rs` — per-subcommand conditional-visibility
  functions for `ms encode`, `mk encode`, `md encode`, `md compile`,
  `md address` (SPEC §8 enumeration-discipline conformance).

### Added — schema contract (Phase C)

- `<cli> gui-schema` JSON contract (SPEC §7). All four sibling CLIs
  now expose a `gui-schema` subcommand that walks their clap
  `Command` tree and emits a machine-readable schema
  (`{version: 1, cli, subcommands: [{name, flags, positionals}]}`).
- `src/schema_check.rs::parse_gui_schema_json` + `json_flag_names`
  — runtime consumer that shells out to `<cli> gui-schema`, parses
  the JSON, and exposes the canonical per-subcommand flag-name set
  to the schema-mirror gate. Falls back to v0.1
  regex-on-`--help` if the binary lacks `gui-schema` or exits
  non-zero.
- `.github/workflows/schema-mirror.yml` — 4 new
  `smoke-gui-schema-*` steps that validate each installed sibling
  binary emits the SPEC §7 envelope on CI before the test suite runs.

### Added — secret buffer + OS occlusion (Phase B)

- `src/form/secret_widget.rs` — `SecretLineEdit` widget backed by
  `Zeroizing<Vec<u8>>` for `--passphrase` and other secret-flag
  fields. Buffer zeroes on drop, on form reset, and on app exit.
  Excluded from `Serialize` / `Debug` derives; never persisted to
  disk via `redact_for_persistence`.
- `src/platform.rs` — first `unsafe` module in the codebase;
  cfg-gated macOS / Windows / Linux occlusion impl: macOS uses
  `NSWindowSharingType::NSWindowSharingNone` via `objc2` /
  `objc2-app-kit`, Windows uses
  `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` via
  `windows-rs`, Linux logs that no compositor API is available.
- `ctrlc` crate (windows-only) — Ctrl-C handler that runs the
  same `on_exit()` cleanup path as the unix `signal_hook::iterator`
  branch (Phase A.2).

### Added — testing infrastructure (Phase A.3 + D.4)

- `egui_kittest` (v0.31.1) dev-dependency; `accesskit` feature on
  `eframe`.
- `tests/widget_interaction.rs` — 4 cells driving real egui forms:
  slot editor (cell 1), conditional visibility (cell 2),
  `ms encode` argv assembly (cell 4), `md encode` dropdown
  value-inspection (cell 5).
- `tests/widget_secret.rs` — `cell_paste_warn_modal_trigger`
  validates the paste-warn modal text and behavior on
  `SecretLineEdit` paste events.
- `tests/dropdown_id_salt.rs` — source-audit regression backstop
  for the v0.1.2 ComboBox ID-collision hotfix.

### Fixed

- `.github/workflows/build.yml` — `compute-version` step strips
  the `mnemonic-gui-` prefix from `github.ref_name` into a
  `VERSION` env var; 4 artifact-name template sites now reference
  `env.VERSION` instead of `env.REF_NAME`. Pre-fix this produced
  doubled-prefix artifacts like
  `mnemonic-gui-mnemonic-gui-v0.1.0-x86_64-linux.tar.gz`.

### Pinned

- `pinned-upstream.toml` — bumped all four sibling tags in
  lockstep with the Phase C.2 PR merges:
  `mnemonic-toolkit-v0.9.0`, `descriptor-mnemonic-md-cli-v0.5.0`,
  `ms-cli-v0.2.0`, `mk-cli-v0.3.0`. All four
  `gui-schema-capable = true` (Phase C.3).

### Internal

- `FormState` lost its `Clone` derive (the new `secret_widgets:
  BTreeMap<String, SecretLineEdit>` field is intentionally
  non-cloneable to prevent accidental secret duplication).
- `PersistedState` lost its `Clone` derive for the same reason.
- `Schema::pinned_version` strings bumped per CLI to match the
  bumped sibling-binary `--version` output.
- 122 tests pass across 15 binaries at the v0.2 release commit
  (vs. ~65 at v0.1.2).

## [0.1.2] — 2026-05-12

Dropdown-bug hotfix. The three `egui::ComboBox` instances in
`src/form/widget.rs` (the `FlagKind::Dropdown` selector, the
`NodeValueComposite` node selector, and the `TaggedOrIndexed` tag
selector) all used `ComboBox::from_label("")` or `from_label(" ")`.
`from_label(label)` derives the egui widget ID from `label`; egui then
keys popup open-state, hover-state, and selection-state by that ID.
With multiple ComboBoxes on the same page all using `""` (or `" "`)
they shared an ID, so:

- The popup failed to open ("no list opens" — egui couldn't
  disambiguate which popup state to drive), and
- Hover and selection state leaked across every ComboBox on the page
  ("every list on the page gets highlighted when one list is clicked
  on" — they all shared interaction state via the shared ID).

v0.1.2 switches each ComboBox to `ComboBox::from_id_salt((const,
flag.name))`, where `flag.name: &'static str` is unique per flag. This
matches the convention already used by `src/form/slot_editor.rs:160`
(`from_id_salt(("slot_subkey", i))`).

### Fixed

- `src/form/widget.rs:26` — `Dropdown` selector now
  `from_id_salt(("flag_dropdown", flag.name))`.
- `src/form/widget.rs:60` — `NodeValueComposite` node selector now
  `from_id_salt(("flag_nodevalue", flag.name))`.
- `src/form/widget.rs:84` — `TaggedOrIndexed` tag selector now
  `from_id_salt(("flag_tagged", flag.name))`.

### Added

- `tests/dropdown_id_salt.rs` — source-audit regression that fails if
  any future edit reintroduces `ComboBox::from_label` in `widget.rs` or
  removes the `from_id_salt` invariant. Pattern follows the existing
  Phase 7 source-audit tests.

### Unchanged

- `src/main.rs:291` uses `ComboBox::from_label("subcommand")` — the
  literal `"subcommand"` label is unique and there is only one such
  ComboBox in the application, so no ID collision. Not affected by the
  bug; not touched by the fix (scope discipline).
- `src/form/slot_editor.rs:160` was already correct
  (`from_id_salt(("slot_subkey", i))`).

## [0.1.1] — 2026-05-12

First functional GUI release. v0.1.0 shipped the full architecture
(schema, form widgets, slot editor, runner, secrets, persistence, CI
gates) but its eframe loop was wired against the `egui_glow` renderer,
which is broken on KDE/KWin Wayland: after the initial 1–2 paint
cycles, the wayland event loop went stuck, ignoring cross-thread
`request_repaint()` and `send_viewport_cmd(Close)`, ignoring KWin's
`xdg_toplevel.close` events, and never reaching `on_exit()` for clean
shutdown. KDE marked the window "Not Responding" in its title bar.

v0.1.1 swaps `egui_glow` → `egui_wgpu` (Vulkan via Mesa). With the
wgpu renderer, every cross-thread wake mechanism works: `update()`
runs at the 1 Hz keepalive cadence (CPU still ~0 % at idle), KWin
sees regular surface commits (no "Not Responding" label), SIGINT /
SIGTERM route through `ViewportCommand::Close` to `on_exit()` in
~2.5 s. Real user clicks would have failed under v0.1.0; they work
under v0.1.1.

### Changed

- `eframe = "0.31"` with `default-features = false` and explicit
  `features = ["wgpu", "default_fonts", "wayland", "x11"]` (was
  `eframe = "0.29"` with default glow renderer).
- `egui = "0.31"` (was `egui = "0.29"`).

### Added

- `signal-hook = "0.3"` dependency.
- SIGINT / SIGTERM handler thread that routes through
  `ViewportCommand::Close` for graceful shutdown (zeroize sweep +
  `on_exit()`), with `process::exit(130)` fallback after 3 s if the
  event loop is unresponsive.
- 1 Hz `wayland-keepalive` background thread keeping the surface
  alive for compositor liveness heuristics.
- `init_tracing` filter suppresses `egui_wgpu` / `wgpu_hal`
  swap-chain timeout warnings at default WARN level; visible under
  `--debug` / `RUST_LOG=info`.
- 3 demo screenshots in `screenshots/` against the working wgpu
  build (replacing the v0.1.0 captures that showed the
  frozen-at-first-paint "Not Responding" GUI).

### Fixed

- The event-loop-stuck bug described above (see
  `FOLLOWUPS.md` → Resolved → `gui-glow-wayland-loop-broken`).
- `on_exit()` signature updated to match the wgpu integration's
  `fn(&mut Self)` (was `fn(&mut Self, Option<&glow::Context>)`).

### Removed

- Diagnostic instrumentation added during v0.1.1 dev (TICK label,
  HEARTBEAT dialog, update() counter, keepalive tracing log) — kept
  only the working production code paths.
- Unused `with_position` viewport hint (Wayland ignores absolute
  window positioning by protocol design).

## [0.1.0] — 2026-05-12

First release. Cross-platform GUI overlay for the m-format constellation
CLIs (`mnemonic`, `md`, `ms`, `mk`), built with `egui` in Rust.

### Phases shipped

The build followed a 10-phase converged plan
(`/home/bcg/.claude/plans/declarative-tumbling-shell.md`, 3 sections —
Brainstorm + SPEC + IMPL_PLAN — all 0C/0I after iterative architect
review). Per-phase agent-report artifacts under `design/agent-reports/`.

- **Phase 0** — Repo scaffolding + dependency pin.
- **Phase 1** — Schema types + `mnemonic` schema (5 subcommands).
- **Phase 2** — Form widget renderer + argv assembler +
  CommandLineToArgvW-compatible copy-command shell-quoting.
- **Phase 3** — SlotEditor composite widget for the
  `--slot @N.<subkey>=<value>` repeating grammar.
- **Phase 4** — Subprocess runner (deadlock-safe via
  `wait_with_output`) + tracing init + integration test.
- **Phase 5** — Conditional visibility engine (12 clap-level and
  runtime constraints from upstream `mnemonic-toolkit-v0.8.1`).
- **Phase 6** — Sibling CLI schemas (`md`, `ms`, `mk`) + path-detect
  data layer.
- **Phase 7** — Secret-handling modals + `Zeroize` + `build.rs` codegen
  of `SECRET_NODE_TYPES` / `SECRET_SLOT_SUBKEYS` from upstream
  `is_secret_bearing()` impls via `syn::parse_file` + mutation-
  detection regression fixture.
- **Phase 8** — `state.json` persistence + never-persist redaction +
  schema-version migration.
- **Phase 9** — Schema-mirror CI workflow + cross-platform build
  matrix (5 targets) + 4 sibling-repo `FOLLOWUPS.md` PRs activating
  the mirror-invariant.
- **Phase 10** — This release roll-up.

### Test corpus

93 integration tests across 9 binaries:

| Binary | Cells |
|--------|-------|
| `argv_assembler` | 10 |
| `argv_assembler_slot` | 5 |
| `conditional_visibility` | 13 |
| `copy_command` | 15 |
| `path_detect` | 9 |
| `persistence` | 11 |
| `runner_integration` | 3 |
| `schema_mirror` | 9 (incl. CI workflow snapshot) |
| `secrets` | 18 |

### Pinned upstream tags

| CLI | Tag | --version output (runtime soft-check) |
|-----|-----|----------------------------------------|
| `mnemonic` | `mnemonic-toolkit-v0.8.1` | `mnemonic 0.8.0` |
| `md` | `descriptor-mnemonic-md-cli-v0.4.3` | `md 0.4.3` |
| `ms` | `ms-cli-v0.1.0` | `ms 0.1.0` |
| `mk` | `mk-cli-v0.2.0` | `mk 0.2.0` |

### v0.1 scope

5 subcommands of `mnemonic` (bundle / verify-bundle / convert /
export-wallet / derive-child) plus 1 subcommand each for the three
siblings (md / ms / mk inspect). See SPEC §A coverage table for the
15 subcommands deferred to v0.2.

### Explicit non-mitigations (v0.2 deferrals per SPEC §14)

- Code-signing (Mac Developer ID, Windows Authenticode): unsigned in
  v0.1; see `docs/onboarding/macos-gatekeeper-walkthrough.md` and
  `docs/onboarding/windows-smartscreen-walkthrough.md` for
  first-launch workarounds.
- Installer packages (AppImage, Flatpak, .dmg, .msi): tarball/zip only.
- Package-manager taps (Homebrew, winget): cargo install + release
  binaries only.
- Custom `SecretLineEdit` with `Zeroizing<Vec<u8>>`: best-effort
  `String`-level zeroize only. FOLLOWUPS
  `gui-secret-buffer-allocator-residue`.
- OS-snapshot occlusion (`NSWindowSharingNone` /
  `WDA_EXCLUDEFROMCAPTURE`): not yet mitigated; paste-warn modal
  copy mentions this. FOLLOWUPS `gui-os-snapshot-secret-occlusion`.
- Headless egui test harness: widget rendering paths unexercised by
  tests in v0.1. FOLLOWUPS `gui-headless-test-harness-evaluation`.

### Cross-repo mirror-invariant

Activated via 4 sibling-repo FOLLOWUPS PRs merged 2026-05-12:

- bg002h/mnemonic-toolkit#13 (tag `mnemonic-toolkit-v0.8.1`)
- bg002h/descriptor-mnemonic#28 (tag `descriptor-mnemonic-md-cli-v0.4.3`)
- bg002h/mnemonic-secret#4 (tag `ms-cli-v0.1.0`)
- bg002h/mnemonic-key#7 (tag `mk-cli-v0.2.0`)

Any flag add/remove/rename or new `conflicts_with` /
`required_unless_present_any` in those upstream CLIs lands in lockstep
with a companion `mnemonic-gui` PR that bumps the schema +
`pinned-upstream.toml` tag for that CLI.
