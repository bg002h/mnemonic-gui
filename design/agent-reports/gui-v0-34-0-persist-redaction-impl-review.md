# Implementation review — GUI v0.34.0 persist-redaction hardening (2026-06-10)

Reviewer: Fable 5 implementation-review agent (post-impl, pre-commit). Spec @ design/SPEC_gui_v0_34_0_persist_redaction_hardening.md (R0 GREEN r2). Verdict: YELLOW (0 Critical / 1 Important / 4 Minor) — I-1 + M-1..M-4 ALL FOLDED post-review (Phase-8 unblocked sentences ×2, lockstep doc moved to the positionals field, T5 coverage attribution reworded, ≤1-positional pin added to T4, Tier fixed); suite + clippy re-verified green after folds. Review verbatim below.

---

## Critical

None.

## Important

**I-1 — Spec §4's required "Phase-8 precondition note" is absent from BOTH FOLLOWUPS resolutions.**
Spec §4 (design/SPEC_gui_v0_34_0_persist_redaction_hardening.md:48): *"Phase-8 precondition note: both entries' resolutions state persistence wiring is now UNBLOCKED on these two counts (I4 resolved in v0.33.0; remaining Phase-8 gates tracked elsewhere…)"*. Neither resolution carries it:
- `positional-secrets-not-redacted-at-persist` Status line (FOLLOWUPS.md:98) covers fix shapes + companion reasoning but never states the Phase-8 wiring precondition is now cleared.
- `tree-wif-hex-privkey-in-key-fields-unredacted` (FOLLOWUPS.md:101-106) mentions "(latent — persistence unwired)" in the What, but no unblocked statement either.

Both entries were the stated **hard preconditions for Phase-8** ("Persistence MUST NOT be wired until this lands" — spec line 4); the whole point of the note is that a future Phase-8 cycle greps these entries and learns the gate is cleared. Fix: one sentence in each resolution, e.g. *"Phase-8 persistence wiring is now UNBLOCKED on this count (I4 cleared in v0.33.0; remaining Phase-8 gates tracked at `persistence-unwired-redaction-never-runs` / `slot-secret-values-rendered-unmasked` [render-side only])."* Docs-only; no code or test impact.

## Minor

**M-1 — FormState lockstep doc landed on the wrong field.** Spec §1.2 (line 19): "Give `FormState.positionals`'s doc (mod.rs:281-284) a lockstep sentence." The sentence was attached to `pub values` instead (src/schema/mod.rs:292-295) while the `positionals` doc (mod.rs:297-300) is unchanged — a rustdoc reader of `positionals` itself won't see it, and the `values` doc now opens with a sentence about a different field. Move (or duplicate) the sentence onto mod.rs:297.

**M-2 — FOLLOWUPS overstates T5's own coverage.** Tree entry Fix line (FOLLOWUPS.md:105): "14-row table test incl. recursive + surplus-children legs." T5 (tests/persist_redaction_v0_34_0.rs:206-216) covers the recursive leg via one child; the explicit **surplus**-children leg lives in the pre-existing `tests/tree_round_trip.rs:234 redaction_blanks_xprv_keys_keeps_xpub_and_hex` (id-3 surplus `pk` node, :256/:277), which now exercises the new walk and stays green — this is how spec §3's "port the existing cells" was satisfied (acceptable), but the FOLLOWUPS sentence attributes it to the 14-row table. Reword.

**M-3 — the ≤1-positional-per-table assumption is verified but unpinned.** Verified empirically: 20 non-empty `*_POSITIONALS` tables, each exactly one literal. The assembler's structure depends on it (invocation.rs:297-312: when any secret positional exists, the `else` branch never runs — a hypothetical future 2-entry table mixing secret + non-secret would silently DROP the non-secret at emit). Cheap hardening: add `assert!(sc.positional_args.len() <= 1)` to the T4 walk.

**M-4 — `Tier: resolved` is a category error.** FOLLOWUPS.md:106: the Tier line conventionally records locality. Use `Tier: GUI-local`.

## Verdict

**YELLOW — 0 Critical / 1 Important / 4 Minor.** Fix I-1 (one sentence × 2 entries, docs-only) and this is GREEN; M-1..M-4 are non-blocking polish.

### Verification record (all checks performed)

1. **Schema field:** `secret: true` on exactly the 5 ms.rs sites (ms.rs:502/510/518/526/534 — inspect/decode/verify/derive ms1 + combine shares), `false` on the other 15 (md 8, mk 6, mnemonic 1); 20/20 literal sites compile-enforced. T4 census cell pins the set non-circularly. `has_positional` invariant doc at mod.rs:74 + `PositionalArgSchema.secret` doc mirrors `FlagSchema.secret`; its only 2 callers confirmed non-secret (conditional.rs:779/830). Stale "zero positionals" doc corrected (mod.rs:37-44) and matches the live binary.
2. **Assembler** (invocation.rs:297-312): widget rows emit in row order at argv end, blank rows skipped (T2), stale `state.positionals` ignored (T2b), non-secret path unchanged (T2c). `find()` consistent with the verified ≤1 invariant (M-3 for pinning).
3. **Render** (main.rs:655-705): direct `SecretLineEdit::show`, no `render_with_dispatch`/`flag_is_secret`; `or_insert_with` one-row seed (matches flag-side scalar discipline; `shares` is required so repeating-required-seed parity holds); ✕ only when `repeating && n>1`, `+ add` only when repeating; removed rows `zeroize()`d (main.rs:680-682); `continue` preserves the enumerate `i`, and the while-pad (main.rs:713-715) keeps non-secret index alignment correct even for a hypothetical mixed table. Reserved-key rows ride the existing `zeroize_form_state` `values_mut()` sweep.
4. **Belt + confirm:** persistence.rs:121 `positionals: Vec::new()` with the honest no-subcommand-context comment; secrets.rs:218-227 positional loop (T3 green).
5. **Allowlist** (tree_model.rs:677-707): full 10× 4-byte literals, panic-free `get(..4)`, byte-0/Kpub rationale + over-acceptance note + rsplit caveat all in the doc; walk covers key + keys + recursive children (surplus via the kind-agnostic recursion — old surplus cell still green); `is_xprv_like` retained for tree_form.rs:786 with bidirectional cross-pointers (tree_model.rs:651-655, tree_form.rs:779-784 widening note); pre-existing `is_xprv_like_matches_gate_heuristic` pin unchanged (tree_model.rs:807).
6. **Tests:** 8 cells = T1/T1b/T2/T2b/T2c/T3/T4/T5 per spec §3; T5 = 4 KEPT + 10 BLANKED = 14 rows incl. `Kpub…` (line 200) and the fail-closed hex comment. **TDD probe:** reverted `positionals: Vec::new()` → `state.positionals.clone()` → exactly `t1_redact_drops_all_positionals` FAILED (7 pass/1 fail); restored byte-identical — sha256 `d3942e38…` matches pre-probe, `git diff --stat` unchanged (16 files, +205/−23), re-run green.
7. **Docs/ritual:** CHANGELOG [0.34.0] claims all verified true of the diff; the 6-of-7 census verified **empirically against the pinned v0.53.1 binary** (`gui-schema` emits positionals for exactly the 7 named subcommands; objects carry only `{name, required, repeating}` — no `secret` field, so the no-companion reasoning holds); both audit index lines flipped (FOLLOWUPS.md:56/:59); version 0.34.0 in Cargo.toml:3, Cargo.lock, README:42; SPEC + both R0 reports present in design/.
8. **Suite + lint:** full `cargo test` with `MNEMONIC_BIN=…/mnemonic-toolkit/target/release/mnemonic` (0.53.1) `MS_BIN=/tmp/pinned-sib/bin/ms` (0.7.0) `MK_BIN=/tmp/pinned-sib/bin/mk` (0.7.0): **471 tests across 50 binaries, 0 failures** (incl. schema_mirror, archetype_schema_mirror, xpub_search gates); `cargo clippy --workspace --all-targets -- -D warnings` clean.
