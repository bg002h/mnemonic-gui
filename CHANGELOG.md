# Changelog

All notable changes to `mnemonic-gui` are recorded here. Follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

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
