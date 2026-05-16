# Changelog

All notable changes to `mnemonic-gui` are recorded here. Follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

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
