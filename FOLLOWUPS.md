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

| Sibling repo | Companion file | Pinned tag for v0.1 | Activation PR |
|--------------|----------------|---------------------|---------------|
| `bg002h/mnemonic-toolkit` | `design/FOLLOWUPS.md` | `mnemonic-toolkit-v0.8.1` | [#13](https://github.com/bg002h/mnemonic-toolkit/pull/13) |
| `bg002h/descriptor-mnemonic` | `design/FOLLOWUPS.md` | `descriptor-mnemonic-md-cli-v0.4.3` | [#28](https://github.com/bg002h/descriptor-mnemonic/pull/28) |
| `bg002h/mnemonic-secret` | `design/FOLLOWUPS.md` | `ms-cli-v0.1.0` | [#4](https://github.com/bg002h/mnemonic-secret/pull/4) |
| `bg002h/mnemonic-key` | `design/FOLLOWUPS.md` | `mk-cli-v0.2.0` | [#7](https://github.com/bg002h/mnemonic-key/pull/7) |

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

## Deferred v0.2

Named for explicit closure per SPEC §14:

- `gui-code-signing-mac-developer-id` — v0.1.0 ships unsigned macOS
  binaries; users need to right-click → Open or `xattr -d com.apple.quarantine`
  on first launch (see `docs/onboarding/macos-gatekeeper-walkthrough.md`).
  v0.2 plan: paid Apple Developer ID + notarization roundtrip.
- `gui-code-signing-windows` — v0.1.0 ships unsigned Windows binaries;
  users need to click SmartScreen "More info → Run anyway" on first launch
  (see `docs/onboarding/windows-smartscreen-walkthrough.md`). v0.2 plan:
  Authenticode certificate (EV variant for SmartScreen reputation).
- `gui-secret-buffer-allocator-residue` — `SecretBuffer` is best-effort
  on `String`; full `Zeroizing<Vec<u8>>` requires custom widget +
  manual buffer management. Phase 7 ships v0.1 zeroize on String.
- `gui-os-snapshot-secret-occlusion` — Mac App Switcher /
  Windows Task View may snapshot the visible window. v0.1 acknowledges
  the risk via paste-warn modal copy; mitigation
  (`NSWindowSharingNone` / `WDA_EXCLUDEFROMCAPTURE`) deferred to v0.2.
- `gui-headless-test-harness-evaluation` — Phase 2/3 widget rendering
  is unexercised by tests; evaluate egui headless harness for v0.2.
- `gui-schema-json-subcommand-evaluation` — v0.1 uses regex flag-name
  extraction from `--help` (load-bearing prior-art: `lint.sh`). A
  `--gui-schema` JSON subcommand on each CLI would be more robust;
  evaluate for v0.2.
- 15 subcommands not in v0.1 coverage (`md encode/decode/verify/...`,
  `ms encode/decode/...`, `mk encode/decode/...`) — Section A coverage
  table v0.1 scope.

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
