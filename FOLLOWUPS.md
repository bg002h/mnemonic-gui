# mnemonic-gui FOLLOWUPS

Cross-repo coordination items + deferred v0.2 work. Per the constellation's
mirror-invariant discipline, every entry that affects a sibling repo carries
a `Companion:` cross-cite, and the corresponding entry in the sibling repo
mirrors it.

## Active

### gui-conditional-applicability-drift-fix

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle. Motivating bug: GUI bundle form default state (template = `bip84`, single-sig) emitted `--threshold 1 --multisig-path-family bip48` which the CLI rejected with SPEC §6.6 byte-exact errors (`crates/mnemonic-toolkit/src/cmd/bundle.rs:120, 207-220`).
- **Where:** `src/form/conditional.rs` (P2 — ~14 NEW rules across `bundle` / `verify-bundle` / `export-wallet` / `derive-child`); `src/form/invocation.rs` (P3 — visibility gate at top of per-flag loop; both Hidden + Disabled suppress emission, Required does not); `tests/gui_schema_conditional_drift.rs` (P4 — NEW drift gate consuming toolkit `mnemonic gui-schema` JSON v2 `conditional_rules`); `src/main.rs:197-206` (P5 — removed `--multisig-path-family bip87` default seed); `src/schema_check.rs` (P1 lockstep — `parse_gui_schema_conditional_rules` + relax `parse_gui_schema_json` version gate from `!= 1` to `< 1`); `.github/workflows/schema-mirror.yml:60-69` (CI smoke-step gate relaxed from `==1` to `>=1` per SPEC §6.10.6 additive-bump policy).
- **What:** Cross-repo mechanism + comprehensive rule coverage. Consumes toolkit-emitted `conditional_rules` JSON v2 (SPEC §6.10 Predicate AST + Effect grammar + drift invariant). Adds ~14 NEW per-frame visibility rules. Extends `assemble_argv` with visibility gate. Latent-bug fix: typed-then-mutex-disabled secret values (e.g., user types `--passphrase=foo` then sets `--passphrase-stdin`) are now suppressed at argv emission per the visibility gate.
- **Status:** `resolved 7b7e07d` — shipped at `mnemonic-gui v0.5.0` (2026-05-16). All P1–P5 surfaces landed; drift gate green; 187/187 GUI tests green against toolkit `v0.16.0` (commit `519bcfc`). End-of-cycle opus reviewer-loop R1 FOLD → R2 PASS (0C / 0I). One post-tag CI surface (`schema-mirror.yml` install-tag was stale) folded in `<next-commit-SHA>`; tracked separately at `schema-mirror-yml-toolkit-pin-tracks-pinned-upstream`.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `gui-schema-conditional-rules-v1` (resolved at toolkit v0.16.0 commit `519bcfc`).

### schema-mirror-yml-toolkit-pin-tracks-pinned-upstream

- **Surfaced:** 2026-05-16, end of v0.5.0 cycle. After `mnemonic-toolkit-v0.16.0` + `mnemonic-gui-v0.5.0` tag-push, the master-branch `schema-mirror` CI failed at `tests/gui_schema_conditional_drift.rs` with `drift gate must exercise at least one rule; got 0`. Root cause: `.github/workflows/schema-mirror.yml:28` hardcoded `--tag mnemonic-toolkit-v0.14.0` (stale since the v0.14.0 release; harmless until v0.5.0 because prior `schema_mirror` tests only consumed flag-name extraction, which is version-agnostic). The v0.5.0 drift gate requires v2 `conditional_rules` emission. Tag CI for `mnemonic-gui-v0.5.0` itself was green (workflow scoped to `branches: [master]`); only master CI failed.
- **Where:** `.github/workflows/schema-mirror.yml` install-mnemonic-toolkit step (line ~30 post-fold). Same drift class applies to the md / ms / mk install steps below it (lines ~36/43/49) which still hardcode `md-cli-v0.5.0` / `ms-cli-v0.2.1` / `mk-cli-v0.3.1` — currently NOT stale vs `pinned-upstream.toml`, but the same drift-detection pattern would prevent future divergence.
- **What:** v1 fold (lands in the same v0.5.0 cycle): bump install-mnemonic-toolkit tag to `mnemonic-toolkit-v0.16.0`. v2 cleanup (future cycle): parameterize all four install steps' tag values from `pinned-upstream.toml` so they auto-track future bumps. Two options for the v2 implementation — (a) a workflow-pre step that parses `pinned-upstream.toml` and exports `MNEMONIC_TOOLKIT_TAG` / `MD_TAG` / `MS_TAG` / `MK_TAG` env vars, or (b) a per-CLI matrix that reads the pin via `dasel` / `jq` per step. (a) is simpler; (b) is more granular.
- **Why deferred:** v1 fold is mechanically trivial and ships this cycle; v2 cleanup is a UX-grade improvement that wasn't in the v0.5.0 cycle's scope.
- **Status:** `resolved 93c862a` — v2 cleanup shipped at `mnemonic-gui v0.5.1` (2026-05-16). Tag points at `93c862a`; the cycle split across two commits: workflow surgery + Cargo bump + CHANGELOG at `a445277`; snapshot-test refactor + CHANGELOG amendment at `93c862a` (the latter folded a latent bug — see below). Mechanism: new `parse-pinned-upstream` workflow-pre step loads `pinned-upstream.toml` via Python 3.11+ stdlib `tomllib` and exports per-CLI tag values; each install step consumes the matching `${{ steps.pins.outputs.<cli>_tag }}` via the `env:` → `$TAG` pattern (per GitHub's hardening guidance). Master CI green at `93c862a`: schema-mirror (run `25973805125`) + build (run `25973805129`). Tag CI green: build (run `25973933383`). Latent-bug fix folded: `tests/schema_mirror.rs::ci_workflow_snapshot` had been passing on an incidental v0.14.0 comment substring (the v0.5.0 fix-commit `54865a7` bumped the real install-step pin from v0.14.0 → v0.16.0 but left a surrounding comment mentioning v0.14.0); the v0.5.1 workflow surgery removed the comment and surfaced the gap. Refactored to assert v2 wiring directly: `parse-pinned-upstream` step present + four `steps.pins.outputs.<cli>_tag` references.
- **Tier:** `v0.6+`
- **Companion:** None — gui-only.

### `gui-schema-runtime-conditional-projection` — project SPEC §6.6 slot-count-dependent + runtime rules into gui-schema JSON

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle. Filed at cycle close per plan §7 item 1.
- **Where:** `src/form/conditional.rs` (gui side — slot-count signal from FormState to conditional engine); `src/schema_check.rs` (Predicate AST extension for `slot_count_op` / `slot_count_min` etc. when toolkit adds them).
- **What:** v1 cycle deferred slot-count-dependent + post-binding rules because the GUI's conditional engine consumes FormState snapshots without slot-count exposure. A future cycle will plumb a slot-count signal through FormState + extend the Predicate AST. Concrete rules to add: SPEC §6.6 row 9 (T-in-range vs N), row 10 (single-sig with N > 1), row 11 (multisig with N == 1), row 13 (BIP-388 distinct-key), row 14 (per-`@N` annotation inconsistency).
- **Why deferred:** Per plan §1.4 — runtime rules surface at Run time via the CLI's typed error. v1 ships argv-level submission.
- **Status:** `open`
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `gui-schema-runtime-conditional-projection`.

### `gui-number-widget-unset-sentinel` — Number/Range/Timestamp/TaggedOrIndexed widgets lack a "no value" sentinel

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, plan §7 item 2.
- **Where:** `src/schema/mod.rs:263-268` (`flag_value_is_present` always returns true for Number/Range/Timestamp/TaggedOrIndexed); `src/form/widget.rs:101-126` (`default_flag_value_for` seeds Number widgets to `min` regardless of user interaction).
- **What:** Numeric / Range / Timestamp / TaggedOrIndexed widgets have no "no value" sentinel — once `default_flag_value_for` seeds them, the value is always-present per `flag_value_is_present`. The v0.5.0 §6.10 visibility gate sidesteps this for the common case (Hidden/Disabled flags don't emit regardless of widget value). A future cycle may add an explicit unset state for UX clarity (e.g., a "clear" affordance next to numeric widgets so users can explicitly opt out of supplying a numeric flag).
- **Why deferred:** Per plan §1.4 — the visibility gate makes this unnecessary for the motivating bug. UX-quality improvement, not a correctness gap.
- **Status:** `open`
- **Tier:** `v0.6+`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` cross-reference entry `gui-number-widget-unset-sentinel` (toolkit-side bookkeeping only — gui-impact-only).

### `gui-default-form-state-template-aware-seed` — replace static default-state seed with template-aware seed

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, plan §7 item 3. Natural successor to P5 (the v0.5.0 cycle's static seed cleanup at `src/main.rs:203`).
- **Where:** `src/main.rs:197-211` (default form-state seed; v0.5.0's P5 removed the `--multisig-path-family bip87` line but left the static structure intact).
- **What:** Replace the static screenshot-mode default seed with a template-aware default. When the user picks a multisig template (e.g., `wsh-sortedmulti`), the form auto-seeds multisig defaults (e.g., `--multisig-path-family bip87`, `--threshold` to a reasonable default); when the user picks single-sig, the form omits those flags entirely.
- **Why deferred:** Out of v0.5.0 cycle scope per plan §7 — optional follow-on. The v0.5.0 P5 cleanup removes the unconditionally-wrong seed; the template-aware version is a UX enhancement.
- **Status:** `open`
- **Tier:** `v0.6+`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` cross-reference entry `gui-default-form-state-template-aware-seed` (toolkit-side bookkeeping only — gui-impact-only).

### `gui-schema-numeric-flag-value-pin-effect` — add `pin_value` Effect variant for SPEC §6.6 row 12 projection

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, R1 I3 reviewer fold. Plan §7 item 4.
- **Where:** `mnemonic-toolkit/design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.3 (Effect vocabulary); `mnemonic-toolkit/src/cmd/gui_schema.rs` (Effect enum + serializer); `mnemonic-toolkit/src/cmd/bundle.rs:200-205` (the rule the projection would encode — `DESCRIPTOR_WITH_NONZERO_ACCOUNT`); `src/form/conditional.rs` (consumer — Number widget value-coerce-to-zero handler).
- **What:** Add a `pin_value: { flag, value }` Effect variant to SPEC §6.10.3 vocabulary so the GUI can coerce `--account` to 0 (or any pinned numeric value) when `--descriptor` is present, mirroring SPEC §6.6 row 12's CLI rejection at `bundle.rs:200-205`. v0.5.0's Number widget for `--account` defaults to `0` (per `default_flag_value_for`) — the safe value; the rule only fires when the user actively types a nonzero value, in which case the CLI's byte-exact error suffices for v0.5.0.
- **Why deferred:** Per R1 I3 reviewer fold — the GUI default of 0 makes this rare misuse; the CLI error is informative. Adding a `pin_value` Effect requires SPEC §6.10.3 expansion + GUI Number-widget coercion semantics not warranted by user evidence.
- **Status:** `open`
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `gui-schema-numeric-flag-value-pin-effect`.

### `gui-schema-template-groups-meta-field` — emit per-subcommand `meta.template_groups` to retire `SINGLE_SIG_TEMPLATES` const

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, R1 I4 reviewer fold. Plan §7 item 5.
- **Where:** `mnemonic-toolkit/src/cmd/gui_schema.rs` (toolkit side — emit `meta.template_groups: { single_sig: [..], multisig: [..] }` block sourced from `Template::is_multisig()`); `src/form/conditional.rs:23` (gui side — replace module-level `SINGLE_SIG_TEMPLATES: &[&str] = &["bip44", "bip49", "bip84", "bip86"]` with parse from JSON `meta.template_groups`); `mnemonic-toolkit/src/template.rs:46-56` (`is_multisig()` source-of-truth — unchanged).
- **What:** v0.5.0 cycle replicates the single-sig template set client-side as a module-level `SINGLE_SIG_TEMPLATES` const in `conditional.rs`. The drift gate test detects divergence, but a future cleanup cycle can collapse the const by having the toolkit emit `meta.template_groups` in the gui-schema JSON.
- **Why deferred:** Out of v0.5.0 cycle scope — the drift gate suffices for parity enforcement. Cleanup-class change.
- **Status:** `open`
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `gui-schema-template-groups-meta-field`.

### mnemonic-gui-schema-mirror

**What:** The `mnemonic-gui` GUI maintains a schema (per-CLI flag surface
description) that mirrors the four constellation CLIs (`mnemonic`, `md`,
`ms`, `mk`). Drift between the GUI schema and any CLI's clap-derive flag
set is enforced at CI time via `.github/workflows/schema-mirror.yml`,
which installs each pinned upstream binary and runs the in-process
`tests/schema_mirror.rs` cells.

Additionally, Phase 7's `build.rs` codegen reads the upstream
`NodeType::is_secret_bearing()` and `SlotSubkey::is_secret_bearing()`
impls to generate the `SECRET_NODE_TYPES` + `SECRET_SLOT_SUBKEYS`
constants. The runtime `source_audit_*` tests re-parse the upstream
files and assert set-equality.

**Companion entries (per CLAUDE.md cross-repo discipline):**

| Sibling repo | Companion file | Current pinned tag | gui-schema PR (Phase C.2) |
|--------------|----------------|--------------------|---------------------------|
| `bg002h/mnemonic-toolkit` | `design/FOLLOWUPS.md` | `mnemonic-toolkit-v0.13.0` (v0.3); was `v0.9.0` at v0.2 | [#14](https://github.com/bg002h/mnemonic-toolkit/pull/14) |
| `bg002h/descriptor-mnemonic` | `design/FOLLOWUPS.md` | `descriptor-mnemonic-md-cli-v0.5.0` | [#29](https://github.com/bg002h/descriptor-mnemonic/pull/29) |
| `bg002h/mnemonic-secret` | `design/FOLLOWUPS.md` | `ms-cli-v0.2.0` | [#5](https://github.com/bg002h/mnemonic-secret/pull/5) |
| `bg002h/mnemonic-key` | `design/FOLLOWUPS.md` | `mk-cli-v0.3.0` | [#8](https://github.com/bg002h/mnemonic-key/pull/8) |

Each sibling-repo entry must cross-cite this entry + the
`mnemonic-gui` repo URL + this `mnemonic-gui-schema-mirror`
workflow URL. When the sibling CLI's flag surface changes (flag
add/remove/rename, conflict_with addition, etc.), both the
sibling-repo PR AND a companion `mnemonic-gui` PR (bumping the
schema + pinned-upstream.toml tag) land in lockstep — matching the
mnemonic-toolkit ↔ docs/manual mirror-invariant pattern.

**Suggested sibling-repo FOLLOWUPS body** (copy-paste into each sibling's
`design/FOLLOWUPS.md`):

```markdown
### mnemonic-gui-schema-mirror

**Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry
`mnemonic-gui-schema-mirror`; CI gate at
`.github/workflows/schema-mirror.yml`.

The `mnemonic-gui` GUI mirrors this CLI's clap-derive flag surface at
the pinned tag `<TAG>`. Any flag add / remove / rename / conflict_with
change in this repo's CLI surface must land in lockstep with a
companion `mnemonic-gui` PR that bumps the schema + the
`pinned-upstream.toml` tag for this CLI.
```

### slip39-gui-schema-flattening-companion

**Companion:** `bg002h/mnemonic-toolkit` `design/PLAN_v0_13_0_p2.md` §4.2 + `design/FOLLOWUPS.md` entry `slip39-shamir-secret-sharing`; toolkit P2.1 RED commit bumps `tests/cli_gui_schema.rs` from 7 → 10 subcommands.

**What:** v0.13.0 P2.1 GREEN lands a `cmd/gui_schema.rs` flattening fix in `mnemonic-toolkit`: nested clap subcommands now emit flattened hyphenated entries in the `gui-schema` JSON output. Specifically:

- `seed-xor` → `seed-xor-split` + `seed-xor-combine`
- `slip39` → `slip39-split` + `slip39-combine`

Schema `version` stays at `1` (additive: existing nested-parent names disappear; new hyphenated names appear; the schema document shape is unchanged).

**Pre-RED probe (executed at toolkit `81488e3`):** confirmed `mnemonic gui-schema | jq '.subcommands[] | select(.name == "seed-xor")'` returns `{name: "seed-xor", flags: [], positionals: []}` — i.e. `mnemonic-gui` v0.2 cannot see `seed-xor split` / `seed-xor combine` as discoverable subcommands. **This is a pre-existing v0.12.0 gap, NOT a v0.13.0 regression.** The toolkit-side flattening fix repairs both v0.12.0 (seed-xor) AND v0.13.0 (slip39) at the same patch.

**GUI-side companion work (gated on `mnemonic-toolkit-v0.13.0` shipping):**

1. Bump `pinned-upstream.toml` `mnemonic-toolkit` tag to `mnemonic-toolkit-v0.13.0` (toolkit PE rollup tag).
2. Refresh the schema-mirror tests (`tests/schema_mirror.rs`) to reflect the new flattened subcommand-name set — the test fixture pins `subcommands[]` names.
3. Audit any GUI surface that dispatched on the now-removed `seed-xor` name. The GUI's v0.2 release predates this fix; the seed-xor surface may have been an empty / unreachable code path (the upstream schema returned `flags: []` so per-flag dispatch had nothing to render). Verify before assuming a no-op.
4. Add `slip39-split` + `slip39-combine` GUI surfaces (new subcommand pair shipped at toolkit v0.13.0).

**Status:** `resolved at mnemonic-gui-v0.3.0` — all 4 GUI-side work items shipped 2026-05-14 in cycle v0.3. The bumped `mnemonic-toolkit-v0.13.0` pin + 4 v0.10..v0.13 drift flags (bundle/verify-bundle/convert/derive-child `*-stdin` adds, closes the `mnemonic-gui-schema-mirror` invariant breach) + 5 new subcommand surfaces (`slip39-{split,combine}`, `seed-xor-{split,combine}`, `final-word`) landed under release tag `mnemonic-gui-v0.3.0`. The latent v0.2 repeating-secret bug in `assemble_argv` was also surfaced and fixed in lockstep. See `design/PLAN_v0_3.md` for the 3-section reviewer-LOCKed plan + P0 drift-fold amendment.

**Tier:** shipped at `mnemonic-gui-v0.3.0`.

### gui-accesskit-production-side-effect (accepted in v0.2 Phase A.3)

**What:** v0.2 Phase A.3 introduced `egui_kittest = "0.31"` as a
dev-dependency (the egui-driven integration test harness). Cargo
feature unification then activates `egui/accesskit` globally because
`egui_kittest 0.31.1 → kittest 0.1.0 → accesskit 0.17.1` requires it,
and `egui-winit 0.31.1`'s `PlatformOutput` is destructured
exhaustively — without the matching feature on egui-winit, the build
fails. The minimal fix was to add `"accesskit"` to eframe's feature
list in `Cargo.toml`, which propagates the feature to both
`egui/accesskit` and `egui-winit/accesskit` (per eframe 0.31
`[features]`).

**Production-binary consequence:** the GUI binary now links the
accesskit family on all platforms (`accesskit_winit` 0.23.1,
`accesskit_unix` 0.13.1 + `atspi-*` transitive on Linux,
`accesskit_macos` 0.18.1, `accesskit_windows` 0.24.1). The
accessibility tree is active at runtime — screen readers and
accessibility tools can traverse the GUI's widgets.

**Disposition: accepted.** No cargo mechanism scopes a feature
activation to dev/test builds only (features are strictly additive
across the dep graph). No accesskit-free egui-0.31 testing harness
exists. The side effect is behaviorally benign (active accessibility
support is a positive externality), not a security concern.

**Revisit triggers:**

- If egui_kittest 0.32+ decouples the kittest/accesskit dep and a
  future GUI version drops the harness, the accesskit feature could
  be removed from eframe.
- If the accessibility tree exposure of mnemonic input fields becomes
  a threat-model concern (e.g., a screen-reader API leaks the secret
  buffer), revisit and audit the accesskit_winit accessible-name
  surface on `SecretLineEdit`.

**Trace:** v0.2 plan Phase A.3 R5 fold N-2 in
`/home/bcg/.claude/plans/v0_2-mnemonic-gui.md` Section C iterative
review log; report at
`design/agent-reports/v0_2-phase-A3-kittest-scaffold-r1.md`.

### `secret-taxonomy-public-api-consumption` — retire `build.rs` source-walker; consume `mnemonic-toolkit::secret_taxonomy` directly

**Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry
`secret-taxonomy-public-api-promotion`. Architect-vetted long-term fix
for the codegen pattern that caused the v0.3.0..v0.3.2 BIP-39-persistence
leak (HIGH-severity; tactically patched in v0.3.3 at commit `6851d1b`).

**Surfaced:** 2026-05-16, post-v0.3.3 emergency security fix.

**Where:** `build.rs` (the entire syn-based upstream-source walker is
the deletion target); `src/secrets.rs` (consumes `SECRET_*` via
`include!(concat!(env!("OUT_DIR"), ...))`; switches to `use
mnemonic_toolkit::secret_taxonomy::*`); `tests/secrets_canonical_fallback.rs`
(the v0.3.3 drift gate — deleted after one-cycle overlap);
`pinned-upstream.toml` (`[mnemonic]` `tag` becomes documentary;
load-bearing pin moves to `Cargo.toml`'s `[dependencies]` table);
`.github/workflows/schema-mirror.yml` (drop the
`cargo-test-secrets-canonical-fallback` step).

**What:** Today the GUI scrapes the toolkit's *private* `cmd/convert.rs`
+ `slot_input.rs` modules via `syn::parse_file` at build time. This is
the workaround for the toolkit's lack of a versioned, addressable
public contract for the secret-class taxonomy. Every fragility of the
codegen path descends from that contract gap — the cargo-install
sandbox stub-fallback bug (v0.3.0..v0.3.2 empty `&[]` arrays leaking
BIP-39 phrases to `state.json`) was a direct consequence. The
toolkit-side companion entry adds a new `pub mod secret_taxonomy` in
`mnemonic-toolkit v0.14.0`; this entry tracks the GUI-side switch to
that contract in `mnemonic-gui v0.4.0`.

**Why deferred:** v0.3.3 tactical patch is shipped + verified +
released; install path is no longer leaking secrets. Long-term fix
requires coordinated minor bump on both sides
(`mnemonic-toolkit v0.14.0` + `mnemonic-gui v0.4.0` lockstep). Filed
for the v0.4.x GUI cycle.

**One-cycle overlap recommended:** in GUI v0.4.0, retain the v0.3.3
`CANONICAL_FALLBACK_*` arrays + the `committed_fallback_is_non_empty`
backstop test, AND add a compile-time `const _: () = assert!(...)`
that they equal `mnemonic_toolkit::secret_taxonomy::SECRET_*`. Drop
the fallback in v0.5.0 once the new contract has been exercised
through one release cycle.

**Status:** `resolved 6fe44b6` (mnemonic-gui v0.4.0, 2026-05-16). Cargo.toml gains `mnemonic-toolkit = { git, tag = "mnemonic-toolkit-v0.14.0" }`; `build.rs` deleted; `src/secrets.rs` switches to `pub use mnemonic_toolkit::secret_taxonomy::*` + compile-time supply-chain guard against drift from v0.3.3's committed snapshot. R1 opus review caught a Critical (incomplete deletion sweep — `tests/schema_mirror.rs::source_audit` mod survived) + 5 Importants; all folded in the same commit before tag. Toolkit half closed at `bg002h/mnemonic-toolkit@1a52612` (mnemonic-toolkit v0.14.0).

**Tier:** `cross-repo / v0.4.0-coordinated`

**Architect's full evaluation** (Options A–E, recommendation A, migration
sketch, 6 non-obvious risks) is in the toolkit-side companion entry —
read that for the deeper rationale.

**Risks to surface at v0.4.0 planning time:**
1. Toolkit dep tree (bitcoin, miniscript, bip39, clap, etc.) gets linked
   into the GUI's cargo graph — ~30-60s cold compile cost increase.
   Mitigation: optional `cli` default-on feature-gate on the toolkit
   side; GUI depends with `default-features = false, features =
   ["secret-taxonomy"]`. Defer if compile cost is acceptable.
2. Toolkit's `secret_taxonomy` module becomes load-bearing semver
   surface — rename/relocate now requires a minor bump.
3. The GUI's pinned `mnemonic-toolkit` tag must stay current; a future
   toolkit-side `is_secret_bearing()` widening (e.g., new node type
   added) without a GUI bump means the GUI silently lacks the new
   secret class. Mitigation: future `mnemonic gui-schema` extension
   emitting the live taxonomy + GUI runtime cross-check against the
   installed `mnemonic` binary.
4. Re-export choice: `pub const &[&str]` (recommended) vs.
   `pub use NodeType` / `pub use SlotSubkey`. Stick with string slices;
   smaller semver surface; decouples GUI from toolkit's internal enum
   shape.
5. `mnemonic-toolkit` lib must build cleanly on GUI's full platform
   matrix (macOS, Windows, Linux × x86_64 + aarch64). `mlock.rs` uses
   `libc` and needs cfg-gating audit (likely already correct, but
   revisit during v0.14.0 release).
6. Lockstep release discipline (mirrors the manual-gui v1.0 cycle
   pattern): toolkit v0.14.0 PR + GUI v0.4.0 PR coordinated; both
   `Companion:` lines updated as each side closes.

### `mnemonic-gui-cratesio-publish` — re-enable `cargo install mnemonic-gui` from crates.io (blocked by toolkit publish)

**Companion:** `bg002h/mnemonic-toolkit/design/FOLLOWUPS.md` entry
`mnemonic-toolkit-cratesio-publish` (blocking).

**Surfaced:** 2026-05-16, post-v0.4.2 crates.io publish audit. v0.3.0
and v0.3.1 were published to crates.io and SHIPPED THE BIP-39
PERSISTENCE LEAK to any direct `cargo install mnemonic-gui` user;
both versions are now yanked (2026-05-16 17:36 UTC, cargo audit
records `bg002h` as the yanker). v0.3.2 / v0.3.3 / v0.4.0 / v0.4.1 /
v0.4.2 were tagged but never published.

**Where:** `Cargo.toml` line 36: `mnemonic-toolkit = { git = "...",
tag = "mnemonic-toolkit-v0.14.2" }` is the publish-blocking dep.
crates.io requires version-or-version+git/path; pure-git deps are
forbidden in published crates.

**What:** Once `mnemonic-toolkit` is on crates.io (toolkit-side
FOLLOWUP), this entry's work is:
1. Change the Cargo.toml dep from `{ git, tag }` to `{ version = "0.14" }` (or whatever the published version is).
2. Verify the v0.3.3 supply-chain guard's `v0_3_canonical_fallback` snapshot still equals the crates.io toolkit's `SECRET_*` (it should, since the toolkit-version pin determines both).
3. `cargo publish --dry-run` then `cargo publish` from `mnemonic-gui`.
4. Toolkit `install.sh` flips `mnemonic-gui` from `cratesio=no` back to `cratesio=yes` so direct `cargo install mnemonic-gui` users get a binary that's structurally incapable of the v0.3.x leak class.

**Why deferred:** Blocked by toolkit publish work; not blocking
install-script users (`./scripts/install.sh mnemonic-gui --from-git
--force` resolves through git+tag and already gets the latest fix).

**Status:** `open` (blocked by `mnemonic-toolkit-cratesio-publish`).

**Tier:** `v1+ / nice-to-have`.

## Deferred to v0.3+

Named for explicit closure per SPEC §14. Carried forward from v0.1
because not in v0.2 scope, or carried forward from v0.2 because
shipped partially.

### `gui-help-icon-per-flag-affordance` — extend help-icon coverage to every flag if Option C selective placement proves insufficient

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle planning. Filed per `mnemonic-toolkit/design/PLAN_manual_gui_v1.md` §2.7 (in-flight; archived to design/ at PE close).
- **Where:** `src/form/widget.rs` widget render. v1.0 ships Option C: per-subcommand `?` button + per-dropdown/NodeValueComposite/TaggedOrIndexed `?` button + per-repeating-field-flag `?` button (28+43+20=91 buttons). Per-flag `?` buttons would add ~100 more buttons across all 28 form views.
- **What:** If user feedback after v1.0 ships surfaces that hover-tooltip alone is insufficient for non-dropdown flags (e.g., users want click-through deep-links for `--passphrase`, `--json-out`, secret-bearing flags), extend Option C to Option A: per-flag `?` buttons on every FlagSchema.
- **Why deferred:** v1.0 ships Option C to balance UX-budget vs visual clutter (91 buttons / ~3 per visible form is sustainable; 200 buttons / ~7 per visible form is chaos). Wait for user feedback.
- **Status:** `open`
- **Tier:** `v1.1+`
- **Companion:** `mnemonic-toolkit/design/PLAN_manual_gui_v1.md` §1.6.

### `gui-manual-base-url-runtime-override` — `--manual-base-url` runtime flag if build-time env-var override proves insufficient

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle planning. Filed per `mnemonic-toolkit/design/PLAN_manual_gui_v1.md` §2.7.
- **Where:** `src/help/url.rs` MANUAL_BASE_URL constant + `src/main.rs` CLI argument parsing. v1.0 ships build-time env-var override `MNEMONIC_GUI_MANUAL_BASE_URL` via `option_env!` (CI staging vs prod). No runtime flag.
- **What:** If users in air-gapped environments need to point the GUI's help icons at a locally-hosted mirror (e.g., a corporate intranet copy of the manual), add a `--manual-base-url <URL>` runtime flag that overrides the compile-time default. Runtime override would also help self-hosting users without rebuilding from source.
- **Why deferred:** v1.0 ships with a stable GitHub Pages URL. Self-hosting / air-gap is a niche use case; defer until concrete demand surfaces.
- **Status:** `open`
- **Tier:** `v1.1+`
- **Companion:** `mnemonic-toolkit/design/PLAN_manual_gui_v1.md` §1.5 + §2.4.

- `gui-code-signing-mac-developer-id` — v0.1.x and v0.2.0 ship
  unsigned macOS binaries; users need to right-click → Open or
  `xattr -d com.apple.quarantine` on first launch (see
  `docs/onboarding/macos-gatekeeper-walkthrough.md`). v0.3+ plan:
  paid Apple Developer ID + notarization roundtrip.
- `gui-code-signing-windows` — v0.1.x and v0.2.0 ship unsigned
  Windows binaries; users need to click SmartScreen "More info →
  Run anyway" on first launch (see
  `docs/onboarding/windows-smartscreen-walkthrough.md`). v0.3+ plan:
  Authenticode certificate (EV variant for SmartScreen reputation).
- `gui-os-snapshot-secret-occlusion-linux` — v0.2 Phase B.2 shipped
  macOS (`NSWindowSharingType::NSWindowSharingNone` via
  `objc2-app-kit`) and Windows
  (`SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` via
  `windows-rs`) occlusion. Linux has no compositor API for this at
  v0.2 — see `src/platform.rs` cfg-not-any branch for the deferral
  notice and the paste-warn modal copy that surfaces the gap to
  users. Tracking entry kept open for the Linux-specific
  follow-up.

### `gui-bundle-multisig-flags-conditional` — `--multisig-path-family` and `--threshold` should be Disabled (conditional-visibility) under single-sig templates

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle M-P2.4 sub-batch 5b R0 fold (worked example for `mnemonic bundle` single-sig had to add an explicit "clear `--multisig-path-family`" step because the field is seeded to `bip87` by default at `src/main.rs:188-211`, and leaving it set under `--template bip84` triggers the `mode_text::PATH_FAMILY_WITHOUT_MULTISIG` refusal).
- **Where:** `src/form/conditional.rs::bundle` (line 21-45). The current rules enforce `--template`-required-unless-descriptor, `--descriptor`/`--descriptor-file` XOR, and `--passphrase`/`--passphrase-stdin` XOR. They do NOT disable `--multisig-path-family` or `--threshold` when the active template is in the single-sig set (`bip44`, `bip49`, `bip84`, `bip86`).
- **What:** Extend `pub fn bundle(state: &FormState) -> FlagVisibility` to disable `--multisig-path-family` and `--threshold` when `state.dropdown_value("--template")` is in the single-sig template set. Mirror the same fix in `verify_bundle` (same constraint applies). The argv assembler will then skip these fields (per `form/invocation.rs::emit_one`'s "empty / false / absent values are NOT emitted" rule at the schema docstring) and the user no longer needs to manually clear the seeded default.
- **Why deferred:** Surfaced AFTER v0.3.0 ship; a reasonable fix but not blocking the manual-gui v1.0 cycle. v1.0 manual instead documents the manual-clear workaround.
- **Status:** `resolved 6c2d019` — closed by the GUI conditional-applicability v1 cycle (mnemonic-gui v0.5.0 + mnemonic-toolkit v0.16.0 lockstep, in-flight). P2 (`16b15de`) extended `bundle()` + `verify_bundle()` + `export_wallet()` with single-sig-template Disabled rules + single-sig-template + descriptor-mode mutexes. P3 (`f2a985b`) added the `assemble_argv` visibility gate that suppresses Hidden/Disabled flags from argv emission. P5 (`2afd603`) removed the `--multisig-path-family bip87` default seed at `main.rs:203` (the root of the surfacing). The manual workaround documented in the worked example may now be retired in a future manual cycle.
- **Tier:** `v0.4`
- **Companion:** `mnemonic-toolkit/docs/manual-gui/src/40-mnemonic/42-bundle.md` worked-example step 3 documents the workaround and cites this FOLLOWUP; superseded by the v1 cycle.
- **Successor:** `gui-conditional-applicability-drift-fix` (this file, above) is the mechanism + drift-gate generalization of which this entry is the originating specific case.

### `gui-run-confirm-modal-secret-redaction` — run-confirm modal renders secret-bearing argv tokens in plaintext (security-relevant gap)

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle M-P2.4 batch 4 R0 source-grep. The `mnemonic-toolkit/docs/manual-gui/src/10-foundations/14-secret-handling.md` Defense-2 prose (LOCKed in M-P2.4 batch 2) claims the run-confirm modal "shows the assembled argv with secret values replaced by `***`". `src/main.rs:512-535` shows the modal renders each argv token verbatim in monospace via `ui.monospace(format!("  {}", tok))`; no redaction step exists anywhere in the source tree (`grep -rn "redact" src/` returns only `persistence.rs` on-disk-save paths).
- **Where:** `src/main.rs:512-535` (modal render block); `src/secrets.rs:65-66` (`RUN_CONFIRM_MODAL_PREFIX` const has no continuation that would substitute a redacted argv); `src/form/invocation.rs:42-100` (`assemble_argv` returns the full plaintext argv including secret-class flag values).
- **What:** Add a redaction step that mirrors `persistence::redact_for_persistence`'s flag-class logic so the modal displays e.g. `--passphrase ***` instead of `--passphrase the-actual-secret-mnemonic`. Two implementation options: (a) build a parallel `redact_argv_for_display(sub, state, &argv)` in `secrets.rs` and call it from the modal site only — preserves the actual `argv` that's passed to `spawn_and_capture` after Run-confirm; (b) inline a per-token check in the modal render loop using `secrets::flag_is_secret` against the preceding flag-name token. Option (a) is cleaner; option (b) is smaller-LOC. Either way the secret-class boundary already exists.
- **Why deferred:** Surfaced AFTER v0.3.0 ship; remediation requires (i) a new `mnemonic-gui` cycle (`mnemonic-gui-v0.4.0` or a v0.3.1 patch) and (ii) lockstep manual prose patch landing in the `manual-gui-v1.0` PR's batch-4 commit so the v1.0 manual ships consistent with what shipped GUI v0.3.0 actually does. Until the GUI fix lands, the manual MUST describe the actual (undesired) behavior plus an operational mitigation: only run the GUI on a cold/airgapped machine where on-screen secret display does not constitute a network-exfiltration vector. Compromise: the v1.0 manual ships honestly-broken; v1.1 ships fixed. Severity is high but not P0-block-v1.0 for the manual cycle because the manual cannot fix the GUI behavior — only describe it.
- **Status:** `open`
- **Tier:** `v0.4-cross-repo`
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md` `gui-run-confirm-modal-secret-redaction-manual-companion`; `mnemonic-toolkit/docs/manual-gui/src/10-foundations/14-secret-handling.md` Defense-2 prose patch in M-P2.4 batch 4 commit. Closure requires: (i) GUI source patch implementing redaction; (ii) manual prose patch undoing the v1.0 honest-broken framing and restoring the `***` claim; (iii) `pinned-upstream.toml` bump in this manual to whatever GUI tag ships the fix.

## Resolved in v0.2

- `gui-secret-buffer-allocator-residue` — **shipped Phase B.1.**
  `SecretLineEdit` widget backed by `Zeroizing<Vec<u8>>` replaces
  the v0.1 best-effort-on-`String` `SecretBuffer`. Buffer zeroes on
  drop / form reset / app exit. Excluded from `Serialize` /
  `Debug` derives; never persisted to disk via
  `redact_for_persistence`. See `src/form/secret_widget.rs`.
- `gui-os-snapshot-secret-occlusion` (macOS + Windows) —
  **shipped Phase B.2.** macOS uses
  `NSWindowSharingType::NSWindowSharingNone` via `objc2-app-kit`;
  Windows uses `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`
  via `windows-rs`. Both applied from `MnemonicGuiApp::new()`.
  Linux gap moved to a separate entry (above) since the platform
  has no compositor API for this.
- `gui-headless-test-harness-evaluation` — **shipped Phase A.3.**
  `egui_kittest` v0.31.1 dev-dep + `accesskit` feature on
  `eframe`. Five widget-driving cells across
  `tests/widget_interaction.rs` (slot editor, conditional
  visibility, `ms encode` argv, `md encode` dropdown) and
  `tests/widget_secret.rs` (paste-warn modal). See
  `gui-accesskit-production-side-effect` (above, Active) for the
  production-side-effect note that accepting `egui_kittest`
  introduced.
- `gui-schema-json-subcommand-evaluation` — **shipped Phase
  C.1 / C.2 / C.3.** `<cli> gui-schema` subcommand on each of the
  four sibling CLIs emits a SPEC §7 JSON envelope
  (`{version:1, cli, subcommands:[{name, flags, positionals}]}`).
  GUI consumes via `src/schema_check.rs::json_flag_names`. Falls
  back to v0.1 regex-on-`--help` if the binary lacks `gui-schema`
  or exits non-zero. Schema-mirror CI gate now runs
  `<cli> gui-schema | python3 -c 'json.load...'` smoke for each
  CLI before the in-process test suite.
- **15 sibling-CLI subcommands** — **shipped Phase D.1 / D.2 /
  D.3 / D.4.** D.1 audited `--help` across `ms` (×4) + `mk` (×4)
  + `md` (×7). D.2 + D.3 added the schema entries to
  `src/schema/{ms,mk,md}.rs`. D.4 added two egui_kittest cells
  (`ms encode` argv-assembly + `md encode` dropdown
  value-inspect) covering representative new surface. All 15
  subcommand tabs render in the GUI at v0.2.

## Process notes

### v0.2: enforce PR-CI gate before tag-push

**Phase 10 R1 I-2 finding (confidence 85).** v0.1.0 was tagged via direct
push to master on a fresh repo, bypassing the `pull_request` build.yml
trigger that SPEC §B.12 R1 I-3 fold explicitly required ("PR must pass
full matrix BEFORE tag"). For v0.1.0 on a fresh repo with no prior master
history, this was mechanically the only path. For v0.2 and beyond — when
master has history and PRs are the normal flow — feature work must land
via PR with full 5-target CI green before tagging. This entry exists so
the v0.2 release prep doesn't repeat the v0.1 deviation.

## Resolved

### gui-combobox-id-collision (resolved in v0.1.2 by from_id_salt switch)

**Symptom (reported 2026-05-12, post-v0.1.1):**

> "There is a bug involving every dropdown list. No list opens and
> sometimes every list on the page gets highlighted when one list is
> clicked on."

**Root cause:** The three `egui::ComboBox` instances in
`src/form/widget.rs` (the `FlagKind::Dropdown` selector at line 26, the
`FlagKind::NodeValueComposite` node selector at line 60, and the
`FlagKind::TaggedOrIndexed` tag selector at line 84) all used
`ComboBox::from_label("")` or `from_label(" ")`.
`ComboBox::from_label(label)` derives the egui widget ID from `label`,
and egui keys popup open-state, hover-state, and selection-state by ID.
All ComboBoxes sharing the same `""`/`" "` label thus shared an ID:

- "no list opens" — egui couldn't disambiguate which popup-state to
  drive when the click landed on a widget with a non-unique ID.
- "every list on the page gets highlighted when one is clicked" — the
  hover and selection state propagated to every widget sharing the ID.

**Fix:** Switched each of the three sites to
`ComboBox::from_id_salt((const, flag.name))` — the
`flag.name: &'static str` field is unique per `FlagSchema`, so each
ComboBox gets a unique egui widget ID. This matches the convention
already used by `src/form/slot_editor.rs:160`, which had been correct
since v0.1.0 (`from_id_salt(("slot_subkey", i))`).

**Audit pinned at `tests/dropdown_id_salt.rs`:** the test reads
`src/form/widget.rs` and asserts (a) no `ComboBox::from_label` calls
remain and (b) `ComboBox::from_id_salt` is used. Future regressions —
e.g., someone reaching for the quicker-typing `from_label("")` again —
fail the audit at test-time.

**Out of scope (left intentionally):** `src/main.rs:291` uses
`ComboBox::from_label("subcommand")`. The label is non-empty and
unique, and there is only one such ComboBox in the application, so no
ID collision occurs. Not touched by this hotfix; the
`from_id_salt`-everywhere stylistic sweep can be a v0.2+ janitorial
follow-up if desired.

**Files changed in v0.1.2:** `src/form/widget.rs` (3 `from_label` →
`from_id_salt` swaps), `tests/dropdown_id_salt.rs` (new audit),
`Cargo.toml` (version bump 0.1.1 → 0.1.2), `CHANGELOG.md` (`[0.1.2]`
entry), this `FOLLOWUPS.md` (Resolved entry).

### gui-glow-wayland-loop-broken (resolved in v0.1.1 by renderer swap)

**Symptom:** With `eframe = "0.29"` + `egui_glow` renderer on KDE/KWin
Wayland, the eframe event loop went stuck after the first 1-2 paint
cycles. Cross-thread `Context::request_repaint()` and
`Context::send_viewport_cmd(ViewportCommand::Close)` calls were silently
dropped — they didn't wake winit's event loop. Symptoms observed during
v0.1.1 dev:

- `update()` called 2 times at startup, never again over 90+ seconds of
  runtime (despite a background keepalive thread calling
  `request_repaint()` at perfect 1 Hz cadence).
- KWin sent `xdg_toplevel.close` via the wayland protocol after a
  Scripting `closeWindow()` call — the GUI process did not process the
  close, did not call `on_exit()`, and stayed alive until SIGKILL.
- Signal-hook handler thread sent `ViewportCommand::Close` on SIGINT —
  ignored the same way; only a `process::exit(130)` fallback after 3 s
  could terminate the process.
- KDE's title bar marked the window "Not Responding" because the
  surface stopped committing frames between input events.

**Root cause:** Bug in the `egui_glow`/`egui_winit` wayland integration's
cross-thread wakeup. Verified across `eframe = "0.29"`, `"0.30"`, and
`"0.31"` — same broken behavior in all three.

**Fix:** Switched eframe to the `wgpu` renderer (Vulkan via Mesa) by
configuring `eframe = { version = "0.31", default-features = false,
features = ["wgpu", "default_fonts", "wayland", "x11"] }` in Cargo.toml.
With wgpu:

- `update()` runs at the keepalive's 1 Hz cadence (CPU still ~0 % at idle)
- Cross-thread `request_repaint()` works
- Cross-thread `send_viewport_cmd(Close)` works
- SIGINT/SIGTERM → handler → `ViewportCommand::Close` → `on_exit()`
  fires cleanly within ~2.5 s (well under the 3 s timeout grace)

A residual cosmetic issue: `egui_wgpu` logs `Dropped frame with error:
A timeout was encountered while trying to acquire the next frame` at the
1 Hz keepalive cadence. These are suppressed at the default WARN level
via the `init_tracing` filter (`wgpu_hal=error,egui_wgpu=error`); only
visible under `--debug` / `RUST_LOG=info`. They don't affect
functionality.

**Files changed in v0.1.1:** `Cargo.toml` (eframe feature flags +
signal-hook), `src/main.rs` (signal-hook handler, keepalive thread,
on_exit signature for wgpu renderer, tracing filter for wgpu warnings).
