# Phase A.2 ctrlc Windows Ctrl-C — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit f8a7cb2 on branch v0_2
**Plan ref:** /home/bcg/.claude/plans/v0_2-mnemonic-gui.md §C Phase A.2; SPEC §5, §2

## Verdict
**0C / 0I — ship**
All eight hot spots resolve clean. The Windows ctrlc block is correctly shaped, additive, and properly gated. Unix block is byte-identical to master. Cargo dep is Windows-only. No critical or important findings.

---

## Critical findings
None.

---

## Important findings
None.

---

## Sub-threshold notes

### N-1 — `on_exit()` log message is Wayland-centric
**Confidence:** 35
**File:** `src/main.rs` line 478

The `on_exit()` log says `"on_exit() called — clean shutdown via wayland close event"`. With the Windows ctrlc path now routing through `ViewportCommand::Close`, `on_exit()` is also reachable on Windows, where the message is inaccurate (no Wayland involved). This is pre-existing code untouched by f8a7cb2; the commit is not the regression origin. A follow-up pass on `on_exit()` could generalize the message (e.g., `"on_exit() called — clean shutdown"`), but does not block Phase A.2.

### N-2 — Duplicate wayland-keepalive comment block
**Confidence:** 20
**File:** `src/main.rs` lines 88-113

The wayland-keepalive explanation is present twice in `MnemonicGuiApp::new()` — an abbreviated version (lines 88-98) followed by a more detailed version (lines 98-113). This is pre-existing and untouched by f8a7cb2. Worth a cleanup pass but not Phase A.2 scope.

### N-3 — `ctrlc` lock churn includes dispatch2 and nix for non-Windows targets
**Confidence:** 20
**File:** `Cargo.lock`

The lock file records `ctrlc = "3.5.2"` with three transitive entries: `nix = "0.31.3"` (unix-gated within ctrlc), `dispatch2 = "0.3.1"` (apple-gated within ctrlc), and `windows-sys = "0.61.2"` (windows-gated within ctrlc). None of these compile on Linux or macOS because ctrlc itself is excluded by the project's `[target.'cfg(windows)'.dependencies]` gate. The lock churn (~63 lines) is expected for a new platform-gated dep with 3 target-differentiated transitive deps. No action needed.

---

## Hot-spot resolution

| # | Hot spot | Verdict |
|---|----------|---------|
| 1 | Compilation gate — trust CI for Windows compile | Acceptable. Same posture as v0.1 Unix signal-hook. No local Windows toolchain available; CI matrix on x86_64-pc-windows-msvc is the correct gate. No alternative local assertion is reachable without a Windows target. |
| 2 | `Send + 'static` for `egui::Context` | Confirmed. egui 0.31 docs explicitly list `Send` + `Sync` in auto-trait impls; `Context` is `Arc<RwLock<...>>` internally. The existing `std::thread::spawn` in the Unix block is an in-repo compile proof; the Windows `ctrlc::set_handler` (which requires `FnMut() + 'static + Send`) uses an identical capture pattern. |
| 3 | ctrlc version and API stability | `ctrlc = "3"` → v3.5.2. `set_handler` signature is `FnMut() + 'static + Send`; the commit's `move || { ... }` closure satisfies this. The API has been stable across all 3.x releases. No known issues at v3.5.2. |
| 4 | Comment block accuracy | New comment (lines 115-127) accurately describes both platforms. Explicitly notes "SIGTERM has no Windows equivalent — Ctrl-C only." The v0.1.1 "v0.2 candidate" language is correctly removed. |
| 5 | Additive discipline — Unix block unchanged | Verified byte-identical: lines 128-148 of the committed `src/main.rs` match lines 124-144 of `master/src/main.rs` exactly. The `#[cfg(windows)]` block (lines 150-161) is appended immediately after with no intervening changes. |
| 6 | Cargo.lock churn | ctrlc v3.5.2 adds nix 0.31.3 + dispatch2 0.3.1 + windows-sys 0.61.2 to the lock. All three are target-gated within ctrlc's own Cargo.toml (nix=unix, dispatch2=apple, windows-sys=windows). On Linux/macOS, none compile. No duplicate nix version conflict (nix 0.31.3 is the sole nix entry; signal-hook uses libc directly, not nix). Churn is proportionate and clean. |
| 7 | Cross-platform handler parity | Verified: both blocks clone `egui::Context`, call `send_viewport_cmd(ViewportCommand::Close)`, sleep 3 s, then `process::exit(130)`. Logging shape matches: `tracing::info!` on receipt, `tracing::warn!` on grace timeout. Exit code 130 (128 + SIGINT) is the Unix convention; Windows mirrors it. |
| 8 | "No RED test" exception | Correctly declared per Phase A.2 spec (lines 735-736). Signal-handler installation requires live process delivery; no `cargo test` harness can exercise this. A compile-only check would require a Windows toolchain absent from this machine. The CI `x86_64-pc-windows-msvc` build matrix job is the canonical gate. Exception is sound and precedented by v0.1 signal-hook posture. |

---

## Exit gate checklist

| Gate item | Status |
|-----------|--------|
| `#[cfg(windows)]` block mirrors Unix signal-hook structure | PASS |
| `signal-hook` Unix block unchanged | PASS — byte-identical to master |
| `ctrlc` dep is Windows-only in Cargo.toml | PASS — `[target.'cfg(windows)'.dependencies]` |
| `cargo build` clean on all 5 matrix targets | DEFERRED to CI — no Windows toolchain locally |
| 0C / 0I | PASS |
