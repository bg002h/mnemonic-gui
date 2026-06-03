# mnemonic-gui v0.22.0 Implementation Plan — toolkit-v0.41.0 pin catch-up + ms1 slot picker + pin-coherence guard

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`).

**Goal:** Bring the GUI's stale pins current (Cargo lib v0.37.3→v0.41.0 + 4 pinned-upstream tags), land the prepared ms1 slot-editor picker, add the missing `md repair` schema entry, fix stale README/banner artifacts, add a `pin_coherence` recurrence guard. Restores `schema_mirror` GREEN (CI bug-fix) + ships the ms1 picker. GUI 0.21.3 → **0.22.0** (MINOR).

**Architecture:** No new GUI logic — this is a pin/schema lockstep cycle. The only behavioral add is the `Ms1` slot-editor picker variant (already drafted) + its secret-redaction (via the bumped toolkit lib re-export). `md repair` is a declarative schema entry. The guard is a pure-logic test.

**Tech Stack:** Rust (GUI needs ≥1.88 transitively → build/test with `+1.94.0`; GUI CI uses `@stable`). `toml` crate (already a dep + dev-dep). 4 sibling CLI binaries for the schema_mirror gate.

**Source of truth:** `design/SPEC_gui_v0_22_0_pin_catchup_ms1.md` (R0-GREEN, `80fd012`). Branch: continue on `bundle-slot-ms1-gui` (carries the draft `d04bad9`). Re-grep all line numbers before editing.

**Gate per phase:** all four CLI binaries built at current versions (mnemonic 0.41.0, ms 0.7.0, md 0.6.2, mk 0.7.0); `MNEMONIC_BIN/MS_BIN/MD_BIN/MK_BIN`=abs paths; `cargo +1.94.0 test --workspace` 0-fail + `cargo +1.94.0 clippy --all-targets -- -D warnings` clean. Mandatory opus R0 per phase + end-of-cycle (0C/0I; persist to `design/agent-reports/`; re-dispatch after every fold).

---

## Phase 0 — pre-flight (no edits)

- [ ] **Confirm the toolkit tag is on the remote** (the Cargo git-dep resolves against GitHub, R0-M4): `git ls-remote --tags https://github.com/bg002h/mnemonic-toolkit mnemonic-toolkit-v0.41.0` → must print a ref. (It was pushed when toolkit v0.41.0 shipped.) Also confirm `ms-cli-v0.7.0` / `descriptor-mnemonic-md-cli-v0.6.2` / `mk-cli-v0.7.0` exist on their remotes (for CI installs).
- [ ] **Build the 4 current binaries** (for the gate): mnemonic (`cargo build -p mnemonic-toolkit --bin mnemonic` in toolkit master → `target/debug/mnemonic`, v0.41.0); ms (`--manifest-path .../mnemonic-secret/Cargo.toml -p ms-cli` → ms 0.7.0); md (`--manifest-path .../descriptor-mnemonic/Cargo.toml -p md-cli --features cli-compiler` → md 0.6.2); mk (`--manifest-path .../mnemonic-key/Cargo.toml -p mk-cli` → mk 0.7.0). Record abs paths + confirm `--version`.
- [ ] **Sizing re-confirm** (SPEC §7, R0-m2): with the 4 `*_BIN` set, `cargo +1.94.0 test --test schema_mirror` from GUI master — confirm **zero flag-NAME drift on every schema-declared cell** (all per-CLI cells PASS). NOTE: `md repair`'s absence is real-but-**ungated** — `schema_mirror` (schema_mirror.rs:91-121) iterates only schema-declared subcommands, so a binary-only subcommand is invisible; it is closed declaratively in Task 1.3, not surfaced as a red cell here. This run empirically re-confirms the SPEC's "schemas already current; redness is purely stale pins" finding.

---

## Phase 1 — pins + guard + ms1 draft + md repair (the code)

**Files:** `Cargo.toml`, `Cargo.lock`, `pinned-upstream.toml`, `tests/pin_coherence.rs` (new), `src/schema/md.rs`. (Draft already carries `src/form/slot_editor.rs` + `src/secrets.rs`.)

### Task 1.1 — pin-coherence guard (TDD: red against current tree)

- [ ] **Step 1: Write `tests/pin_coherence.rs`** — typed `toml`-crate parse (R0-M2; `toml` is a dev-dep at `Cargo.toml:73`):

```rust
//! Guards the bug class "Cargo toolkit pin and pinned-upstream.toml drift apart"
//! (CHANGELOG v0.22.0). pinned-upstream.toml:20-21 already declares they bump in
//! lockstep; this promotes that prose to a gate. Pure-logic; no binary, no network.
use std::fs;
use std::path::Path;

fn read(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(name)).unwrap()
}

#[test]
fn cargo_toolkit_pin_matches_pinned_upstream_mnemonic_tag() {
    let cargo: toml::Value = toml::from_str(&read("Cargo.toml")).unwrap();
    let cargo_tag = cargo["dependencies"]["mnemonic-toolkit"]["tag"]
        .as_str().expect("Cargo.toml [dependencies].mnemonic-toolkit.tag");
    let pinned: toml::Value = toml::from_str(&read("pinned-upstream.toml")).unwrap();
    let pinned_tag = pinned["mnemonic"]["tag"]
        .as_str().expect("pinned-upstream.toml [mnemonic].tag");
    assert_eq!(
        cargo_tag, pinned_tag,
        "pin drift: Cargo.toml toolkit tag {cargo_tag:?} != pinned-upstream [mnemonic].tag \
         {pinned_tag:?}; bump BOTH in lockstep (CHANGELOG v0.22.0 bug class)"
    );
}
```
(Re-grep the real Cargo.toml dep-table shape; if `mnemonic-toolkit` is under `[dependencies]` as an inline table with `tag`, the indexing above holds. Adjust if it's `[dependencies.mnemonic-toolkit]`.)

- [ ] **Step 2: Run — verify it FAILS against the current tree.** `cargo +1.94.0 test --test pin_coherence 2>&1 | tail`. Expected: FAIL — Cargo tag `mnemonic-toolkit-v0.37.3` != pinned-upstream `mnemonic-toolkit-v0.38.0`. (Confirms it's a real guard catching the existing drift.)
- [ ] (Do NOT commit yet — the fix is Task 1.2; commit after green to keep HEAD coherent.)

### Task 1.2 — bump the pins (the fix)

- [ ] **Step 1: Bump `Cargo.toml`** — `mnemonic-toolkit` git-dep `tag = "mnemonic-toolkit-v0.37.3"` → `"mnemonic-toolkit-v0.41.0"` (`:42`). (Leave `version` for Phase 2.)
- [ ] **Step 2: Bump `pinned-upstream.toml`** — `[mnemonic].tag` → `mnemonic-toolkit-v0.41.0`; `[ms].tag` → `ms-cli-v0.7.0`; `[mk].tag` → `mk-cli-v0.7.0`; `[md].tag` → `descriptor-mnemonic-md-cli-v0.6.2`; re-word the stale `[md]` "v0.4.3" comment (`:32-36`).
- [ ] **Step 3: Relock.** `cargo +1.94.0 build 2>&1 | tail` (resolves the new toolkit rev; updates `Cargo.lock`). Expected: COMPILES — the draft's `SECRET_SLOT_SUBKEYS` const-assert now passes (6-entry re-export == 6-entry snapshot; SECRET_NODE_TYPES unchanged per SPEC §3).
- [ ] **Step 4: Run — pin_coherence + secret gates GREEN.** `cargo +1.94.0 test --test pin_coherence --test secret_taxonomy_pin --test schema_mirror_secret_drift 2>&1 | tail`. Expected: PASS (pin_coherence now coherent; the const-asserts compiled).
- [ ] **Step 5: Commit** — `git add Cargo.toml Cargo.lock pinned-upstream.toml tests/pin_coherence.rs && git commit -m "feat(pins): lockstep bump to toolkit v0.41.0 + siblings current + pin_coherence guard (P1.1-1.2)"`.

### Task 1.3 — add the `md repair` schema entry

- [ ] **Step 1: Write the SubcommandSchema** in `src/schema/md.rs` per SPEC §5 (field-accurate vs `src/schema/mod.rs` defs — re-grep `inspect`/`decode` to match idiom): add `REPAIR_FLAGS` (single `--json` Boolean, non-secret) + `REPAIR_POSITIONALS` (`md1-strings`, required+repeating) + a `SubcommandSchema { name: "repair", human_name: "Repair (BCH error-correction)", flags: REPAIR_FLAGS, positional_args: REPAIR_POSITIONALS, allows_slots: false, conditional: None }` appended to `SUBCOMMANDS` (after `address`).
- [ ] **Step 2: Run — schema_mirror still GREEN.** With `MD_BIN`=md 0.6.2: `MNEMONIC_BIN=… MD_BIN=… MS_BIN=… MK_BIN=… cargo +1.94.0 test --test schema_mirror 2>&1 | tail`. Expected: still GREEN (R0-m1) — the `md` cell was ALREADY green (`md repair`'s absence is ungated; schema_mirror iterates only schema-declared subcommands). Adding the `repair` entry CLOSES the schema/binary subcommand-coverage gap that no automated gate catches, and the new `repair` cell now verifies `--json` matches the md 0.6.2 binary. This is a coverage-gap closure, NOT a previously-red cell flipping.
- [ ] **Step 3: Commit** — `git add src/schema/md.rs && git commit -m "schema(md): add md repair SubcommandSchema (P1.3)"`.

### Phase 1 gate
- [ ] Full suite + clippy with the 4 `*_BIN` set: `cargo +1.94.0 test --workspace 2>&1 | grep -cE '^test .* FAILED'` → `0`; `cargo +1.94.0 clippy --all-targets -- -D warnings` clean. The compile-time const-asserts (`src/secrets.rs:78-99`) compiled (proven by any successful build). **Persist opus R0** to `design/agent-reports/gui-ms1-phase-1-R0-review.md` BEFORE proceeding; loop to 0C/0I.

---

## Phase 2 — version + docs

**Files:** `Cargo.toml`, `src/schema/{mnemonic,md,mk}.rs` (banners + doc headers), `README.md`, `CHANGELOG.md`, `tests/schema_mirror.rs` (the `:402` comment), `mnemonic-toolkit/design/FOLLOWUPS.md`.

### Task 2.1 — version + pinned_version banners + module-doc headers

- [ ] **Step 1:** `Cargo.toml:3` `version = "0.21.3"` → `"0.22.0"`.
- [ ] **Step 2: `pinned_version` const banners** (SPEC §4 item 8): `src/schema/mnemonic.rs:~3452` "mnemonic 0.38.0" → "mnemonic 0.41.0"; `src/schema/md.rs:~532` "md 0.5.0" → "md 0.6.2"; `src/schema/mk.rs:~476` "mk 0.6.0" → "mk 0.7.0". (ms.rs already "ms 0.7.0".)
- [ ] **Step 3 (R0-M3, cosmetic):** module-doc headers `src/schema/mnemonic.rs:1` → v0.41.0; `src/schema/md.rs:1` → v0.6.2; `src/schema/mk.rs:1` → v0.7.0; + the `tests/schema_mirror.rs:402` inline pin-set comment.
- [ ] **Step 4: Commit** — `git add -- Cargo.toml src/schema/mnemonic.rs src/schema/md.rs src/schema/mk.rs tests/schema_mirror.rs && git commit -m "release(gui): v0.22.0 + pinned_version banners + doc headers (P2.1)"`.

### Task 2.2 — README install block + CHANGELOG + FOLLOWUP flip

- [ ] **Step 1: README** (`:42,50-53`): bump the manual-install `cargo install --tag` block to the current set — mnemonic-gui v0.22.0, mnemonic-toolkit v0.41.0, descriptor-mnemonic-md-cli v0.6.2, ms-cli v0.7.0, mk-cli v0.7.0 — so `:47`'s "match pinned-upstream.toml" is true.
- [ ] **Step 2: CHANGELOG** — new `[0.22.0]` entry (Keep-a-Changelog prose, matching prior entries) per SPEC §9: lands the ms1 slot picker + SECRET_SLOT_SUBKEYS snapshot at toolkit pin v0.41.0; catch-up pin bump across all 4 CLIs — RESTORES schema_mirror green (CI bug-fix, pins had lagged the schemas since v0.21.3); adds the `md repair` schema entry; the new `pin_coherence` guard + names the "schema-ahead-of-pins, masked by local-binary run" bug class; SECRET_NODE_TYPES unchanged so only SECRET_SLOT_SUBKEYS moved.
- [ ] **Step 3: Flip the toolkit FOLLOWUP** — in `/scratch/code/shibboleth/mnemonic-toolkit/design/FOLLOWUPS.md`, set `gui-ms1-slot-subkey-pending-pin-bump` Status `open` → `resolved <gui-commit-sha>` (will finalize the SHA at ship). (Committed in the toolkit repo separately at ship.)
- [ ] **Step 4: Commit (GUI)** — `git add -- README.md CHANGELOG.md && git commit -m "docs(gui): README install pins + CHANGELOG v0.22.0 (P2.2)"`.

### Phase 2 gate
- [ ] Full suite + clippy green (as Phase 1). **Persist opus R0** to `design/agent-reports/gui-ms1-phase-2-R0-review.md`; loop to 0C/0I.

---

## End-of-cycle + ship (authorized: autonomous through tag)

- [ ] **End-of-cycle opus R0** over the full branch diff (`master..HEAD`) → `design/agent-reports/gui-ms1-end-of-cycle-R0-review.md`; loop to 0C/0I.
- [ ] **Ship** (GUI tag-only): clean tree → `git checkout master && git merge --ff-only bundle-slot-ms1-gui` → tag `mnemonic-gui-v0.22.0` (annotated) → `git push origin master` + push tag. Then in the toolkit repo: commit the FOLLOWUP flip (`gui-ms1-slot-subkey-pending-pin-bump` → resolved <gui-sha>) → push toolkit master. Update CONTINUITY.md + save a memory record.

---

## Self-review (spec coverage)
- SPEC §4 items 1-3 (pins) → Tasks 1.2; item 4 (md repair) → 1.3; items 5-6 (draft) → already on branch, kept (1.2 Step 3 verifies the const-assert); items 7 (README) → 2.2; item 8 (banners + M3 headers) → 2.1; item 9 (guard) → 1.1; items 10-11 (CHANGELOG + FOLLOWUP) → 2.2. §3 (SECRET_NODE_TYPES unchanged) → 1.2 Step 3 (compile proof). §5 (md repair code) → 1.3. §6 (guard code) → 1.1. §7 (gate + pre-flight) → Phase 0 + per-phase gates. §9 (SemVer/CHANGELOG) → 2.1/2.2. No placeholders; all commands + code present.
