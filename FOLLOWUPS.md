# mnemonic-gui FOLLOWUPS

Cross-repo coordination items + deferred v0.2 work. Per the constellation's
mirror-invariant discipline, every entry that affects a sibling repo carries
a `Companion:` cross-cite, and the corresponding entry in the sibling repo
mirrors it.

## Active

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

| Sibling repo | Companion file | Pinned tag for v0.2 | gui-schema PR (Phase C.2) |
|--------------|----------------|---------------------|---------------------------|
| `bg002h/mnemonic-toolkit` | `design/FOLLOWUPS.md` | `mnemonic-toolkit-v0.9.0` | [#14](https://github.com/bg002h/mnemonic-toolkit/pull/14) |
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

**Status:** `unblocked` — toolkit `mnemonic-toolkit-v0.13.0` tag shipped 2026-05-14 (commit `6a80343`; <https://github.com/bg002h/mnemonic-toolkit/releases/tag/mnemonic-toolkit-v0.13.0>). The 4 GUI-side work items above can now be picked up; the FOLLOWUP itself stays open until that cycle lands.

**Tier:** next mnemonic-gui cycle (likely `v0.3-feature`).

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

## Deferred to v0.3+

Named for explicit closure per SPEC §14. Carried forward from v0.1
because not in v0.2 scope, or carried forward from v0.2 because
shipped partially.

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
