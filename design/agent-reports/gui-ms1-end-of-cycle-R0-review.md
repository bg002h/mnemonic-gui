# gui-ms1 catch-up — End-of-Cycle R0 Review
**Verdict:** GREEN (0C/0I)

Full cycle diff `git diff ec9f00b..1cbf13d` (subsumes the Phase-2 docs/version review). Last gate before merge→master + tag `mnemonic-gui-v0.22.0`.

## Critical (0) / Important (0) / Minor (1)

## 1 release-correctness — ACCURATE
- Version coherence: `Cargo.toml:3` = "0.22.0"; `Cargo.lock` mnemonic-gui self-version 0.22.0; toolkit lib @ v0.41.0 tag rev `d8d0170…3733` (byte-matches remote tag + SPEC SHA); transitive md-codec 0.35.0 / mk-codec 0.4.0 / ms-codec 0.4.0 = toolkit v0.41.0's declared deps (inert — GUI only uses secret_taxonomy). No unintended lock change.
- `pinned_version` banners current: mnemonic.rs:3452 "mnemonic 0.41.0", md.rs:565 "md 0.6.2", mk.rs:476 "mk 0.7.0", ms.rs:529 "ms 0.7.0". Module-doc headers (line 1) bumped on all 4. tests/schema_mirror.rs:401-402 pin-set comment current.
- README install block (`:42,50-53`) cross-checked against live pinned-upstream.toml (`:22,39,46,53`): mnemonic-gui v0.22.0 / mnemonic-toolkit-v0.41.0 / md-cli-v0.6.2 / ms-cli-v0.7.0 / mk-cli-v0.7.0 — all match → README:47 "match pinned-upstream.toml" claim now TRUE. Markdown preserved.
- CHANGELOG `[0.22.0]` (CHANGELOG.md:6), dated 2026-06-03, SemVer-MINOR, covers SPEC §9 (a)-(f) accurately (ms1 picker+snapshot; 4-CLI catch-up RESTORES schema_mirror green as bug-fix; md repair; pin_coherence guard + named bug class; SECRET_NODE_TYPES unchanged). "CI was effectively red" framing accurate (CI installs at stale pins). Not overclaiming.

## 2 integration — END-TO-END SOUND (runtime cross-checked vs upstream v0.41.0)
- Upstream SECRET_SLOT_SUBKEYS @ v0.41.0 = `["phrase","seedqr","entropy","ms1","xprv","wif"]` — byte+order-identical to GUI snapshot secrets.rs:67-68 → positional secret_slice_eq const-assert (:89-99) holds.
- Upstream SECRET_NODE_TYPES @ v0.41.0 = 8 entries byte+order-identical to secrets.rs:42-54 (confirms SPEC §3) → const-assert (:78-88) holds. Both compile-time guards = the load-bearing proof.
- SlotSubkey enum order (Phrase,Seedqr,Entropy,Ms1,Xpub,MasterXpub,Fingerprint,Path,Wif,Xprv) mirrored exactly by slot_editor.rs:27-65 (enum+ALL); as_str all match; is_secret_bearing (:82-92) = {Phrase,Seedqr,Entropy,Ms1,Wif,Xprv} = upstream. Ms1 fully wired, nothing half-wired.
- Redaction complete: persistence.rs:91 filters via `!SECRET_SLOT_SUBKEYS.contains(...)` → Ms1 slot row redacted from state.json; run-confirm (secrets.rs:211-212) + zeroize (:281-283) iterate dynamically.
- md repair: md-cli v0.6.2 repair.rs = positional md1_strings + --json only; schema md.rs:467-560 mirrors field-for-field, appended after address → schema_mirror md cell green.

## 3 regressions-scope — CLEAN
Diff is exactly the cycle's files. tests/secrets.rs:281-287 expects the 6-entry set incl ms1 (coherent). tests/pin_coherence.rs typed-parse anchors the right tags (both v0.41.0 → pass). No Phase-2 edit broke a Phase-1 gate (Phase 2 changed only version/banner/doc/comment strings — value-inert to the secrets/schema_mirror/pin_coherence assertions). Workspace gate was controller-confirmed GREEN at Phase-1 R0 (0 fail + clippy clean).

## 4 ship-readiness — READY
No blocking defect; all versions consistent (Cargo.toml↔lock↔banners↔README↔CHANGELOG). Tag `mnemonic-gui-v0.22.0` matches convention. FOLLOWUP split correct: `gui-ms1-slot-subkey-pending-pin-bump` (toolkit FOLLOWUPS.md:58, Status open) is flipped to resolved <gui-sha> at ship in the TOOLKIT repo — correctly not in this GUI diff. Confirm GUI tracked tree clean before checkout→ff→tag→push.

## Minor (1) — non-blocking (FIXED post-review)
- m-residual (cosmetic, carried from Phase-1 m1): slot_editor.rs:37 doc comment labeled "v0.41.0 (toolkit v0.41.0)" where the GUI release is v0.22.0. **Fixed → "v0.22.0 (toolkit v0.41.0)"** for consistency with the secrets.rs:62 fix. No functional/test impact.

## Verdict rationale
All pins coherent + load-bearing-correct (Cargo.lock byte-verified @ d8d0170); banners/headers/README/CHANGELOG current + accurate; README "matches pinned-upstream" true. Whole-cycle ms1 integration proven by runtime cross-check vs toolkit v0.41.0 (both SECRET_* consts byte+order-identical → both const-asserts compile; SlotSubkey mirrors; redaction/run-confirm/zeroize consume +ms1; md repair field-accurate; pin_coherence reaches the right tags). No stray edits; no Phase-2 edit disturbed a Phase-1 gate; workspace gate empirically GREEN (controller-confirmed). Sole open item was one cosmetic doc-comment label, now fixed. **GREEN (0C/0I) — cleared to merge→master + tag `mnemonic-gui-v0.22.0`** (flip the toolkit FOLLOWUP at ship; confirm clean tracked tree first).
