# Phase B.2 OS-Snapshot Occlusion — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit 2066046 on branch v0_2
**Plan ref:** /home/bcg/.claude/plans/v0_2-mnemonic-gui.md §C Phase B.2; SPEC §4

## Verdict

**1 Critical / 1 Important / 2 Sub-threshold (N) — MUST FIX C-1 before merge**

The Windows branch is correct: `HWND(*mut c_void)` constructor, `SetWindowDisplayAffinity` returning `Result<()>`, and `WDA_EXCLUDEFROMCAPTURE` import path all confirmed against windows-docs-rs. The Linux branch is correct: no `unsafe`, compile-gate test in `schema_mirror.rs::platform_module_compiles_linux` is sound. `FOLLOWUPS.md`, `PASTE_WARN_MODAL_TEXT`, `tests/secrets.rs` byte-exact assertions, `Cargo.toml` dependency additions, and the `main.rs` call site are all correct.

The macOS branch has one certain compile-break (C-1) and one SAFETY-comment and pattern concern (I-1).

---

## Critical findings

### C-1 — `NSWindowSharingType::None` does not exist; compile error on all macOS targets

**Confidence:** 100
**File:** `src/platform.rs` lines 6, 66, 68, 74

objc2-app-kit 0.2 defines `NSWindowSharingType` with three associated constants: `NSWindowSharingNone`, `NSWindowSharingReadOnly`, `NSWindowSharingReadWrite`. There is no `::None`. The identifier `NSWindowSharingType::None` will produce `error[E0599]` on `aarch64-apple-darwin` / `x86_64-apple-darwin`, failing the exit gate.

**Fix:** Replace `NSWindowSharingType::None` → `NSWindowSharingType::NSWindowSharingNone` at all four locations.

---

## Important findings

### I-1 — Two spurious `unsafe` blocks + SAFETY comment missing load-bearing assumption

**Confidence:** 85
**File:** `src/platform.rs` lines 55-58, 65-69, 45-48

`NSView::window()` and `NSWindow::setSharingType()` are both `pub fn` (safe) in objc2-app-kit 0.2. The code wraps them in `unsafe` blocks with SAFETY comments that acknowledge they are safe — a contradiction. Remove both blocks.

The genuinely `unsafe` cast at lines 49-53 (`NonNull<c_void>` → `&NSView`) has a SAFETY comment that omits its two load-bearing invariants: (a) the NSView is retained by the eframe window system, (b) the call runs on the main thread (from `MnemonicGuiApp::new(cc)`). Extend the comment to state both.

---

## Sub-threshold notes

### N-1 — `AnyObject` import + `type_name` suppression line not justified

**Confidence:** 35
**File:** `src/platform.rs:34, 72`

The `use objc2::runtime::AnyObject;` import and `let _ = std::any::type_name::<AnyObject>();` workaround serve no observable purpose in the current code. Remove both as part of the I-1 fold.

### N-2 — B.1 R1 N-1 / A.3 R2 N-1 carry-forwards resolved before B.2 landed

**Confidence:** 95
**File:** Plan line 529

Both carry-forwards were folded in B.1 (plan line 529 now reads "B.1 R1 N-1 fold: aligned with the A.3 R5 fold at line 751"). No outstanding obligation on B.2.

---

## Hot-spot resolution

| # | Hot spot | Verdict |
|---|----------|---------|
| 1 | macOS `NSWindowSharingType::None` | C-1 — constant name wrong |
| 2 | macOS `&NSView` cast soundness | Defensibly sound; SAFETY comment incomplete (I-1) |
| 3 | `view.window()` method | Correct return type; spurious `unsafe` (I-1) |
| 4 | `window.setSharingType()` method | Correct; spurious `unsafe` (I-1) |
| 5 | Windows `HWND` constructor | CORRECT |
| 6 | `SetWindowDisplayAffinity` return type `Result<()>` | CORRECT |
| 7 | `WDA_EXCLUDEFROMCAPTURE` path | CORRECT |
| 8 | Cargo.toml feature flags | CORRECT |
| 9 | Linux no-`unsafe` compile-gate test | CORRECT |
| 10 | `_signature_check()` test affordance | ACCEPTABLE — harmless redundancy |
| 11 | `cc.window_handle()` warn-and-skip failure mode | CORRECT |
| 12 | Race/timing on `new(cc)` | CORRECT — eframe creates native window first |
| 13 | `PASTE_WARN_MODAL_TEXT` content | CORRECT |
| 14 | `tests/secrets.rs` byte-exact assertions | CORRECT |
| 15 | `FOLLOWUPS.md` `gui-os-snapshot-secret-occlusion` | CORRECT |
| 16 | B.1 I-1 / I-2 resolved before B.2 | CORRECT |

---

## Unsafe soundness ruling

After C-1 fixed: macOS branch is defensibly sound. The cast invariants (external retain by eframe, main-thread call site) hold in practice; the SAFETY comment should state them per I-1. Windows branch is sound. Linux branch has no `unsafe`.

## API-call correctness (blind-write constraint)

Windows: HWND, SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE all independently verified against windows-docs-rs — correct.

macOS: method signatures (`NSView::window`, `NSWindow::setSharingType`) correct. Constant name `NSWindowSharingType::None` wrong; should be `NSWindowSharingNone` (C-1).

---

## Exit gate checklist

| Gate item | Status |
|-----------|--------|
| `cargo build` clean on all 5 matrix targets | FAIL — macOS C-1 |
| Manual macOS smoke | BLOCKED by C-1 |
| Manual Windows smoke | NOT YET — user smoke |
| Manual Linux smoke | NOT YET — user smoke |
| `PASTE_WARN_MODAL_TEXT` + byte-exact assertions | PASS |
| 0C / 0I | NOT MET — C-1 + I-1 open |

---

Sources:
- [NSWindowSharingType — docs.rs objc2-app-kit 0.2.2](https://docs.rs/objc2-app-kit/0.2.2/objc2_app_kit/struct.NSWindowSharingType.html)
- [SetWindowDisplayAffinity — microsoft.github.io windows-docs-rs](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/WindowsAndMessaging/fn.SetWindowDisplayAffinity.html)
- [AppKitWindowHandle — raw-window-handle 0.6](https://docs.rs/raw-window-handle/0.6.2/raw_window_handle/struct.AppKitWindowHandle.html)

Next action: Fix C-1 + fold I-1 + N-1 (remove `AnyObject` import and workaround); re-review as R2.
