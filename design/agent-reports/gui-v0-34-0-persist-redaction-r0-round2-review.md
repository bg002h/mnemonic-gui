# R0 round-2 architect review — SPEC_gui_v0_34_0_persist_redaction_hardening (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 2, post-fold verification). GUI 4b5bf46. Verdict: GREEN (0 Critical / 0 Important / 3 Minor incl. one partial fold-drift — folded post-review before Phase 1). Review verbatim below.

---

## Critical

None.

## Important

None.

## Minor

- **M-r2-1 (§4 — residual stale hedge from the M5 fold; see Fold-verification M5).** §4's Phase-8 precondition note still reads "slot VALUES are already redacted… **verify that claim in-cycle before writing it; if wrong, file accordingly**" while the Non-goals section now states the claim is VERIFIED (R0-r1 M5, `persistence.rs:103-110`). The spec contradicts itself: one site affirms, the other still orders an in-cycle verification of the same claim. Delete the hedge in §4 (or replace with a pointer to the Non-goals VERIFIED sentence). No design impact — both readings land at the same non-goal.
- **M-r2-2 (§4/Non-goals — secret-capable count is understated; affects the FOLLOWUP-to-be-filed, not this cycle's design).** Measured against the v0.53.1 binary: **6 of the 7** toolkit positionals are secret-capable, not "at least 4". All three xpub-search modes' positionals are `[MS1]...` — `account-of-descriptor --help` reads "Positional ms1 card (HRP-autodetect)" verbatim, identical to `passphrase-of-xpub`/`path-of-xpub` — plus the 3 HRP-autodetect `extra_strings` (`inspect`/`repair`/`verify-bundle` each route ms1). Only `decode-address` `address` is non-secret. "At least 4" is literally true (hedged), so the spec text is not false, but the new FOLLOWUP `toolkit-secret-capable-positionals-unmirrored` should record the accurate census ("6 of 7 — all except decode-address") so it doesn't ship with a born-stale count (the recurring FOLLOWUP-self-mis-cite failure mode).
- **M-r2-3 (§2 — allowlist over-acceptance is harmless; record one sentence).** The 10-prefix list admits SLIP-132 forms (`ypub`/`zpub`/`Upub`…) that the toolkit's step-2 `Xpub::from_str` would refuse anyway (rust-bitcoin accepts only the xpub/tpub version bytes), so a persisted `zpub` is a value the gate will later reject. That is the correct trade — they are PUBLIC material (no leak direction), and keeping them future-proofs against SLIP-132 intake — but worth one sentence at the new fn so nobody later "tightens" the list to {xpub,tpub} and back-doors a behavior change, or conversely mistakes the allowlist for a validity check.

## Fold-verification

- **I1 — FOLDED-OK.** §4 re-derives no-companion from the three measured facts (a) no toolkit-side `secret` field — re-verified: v0.53.1 `gui-schema` positional objects carry exactly `{name, required, repeating}`; (b) schema_mirror flag-name-only; (c) GUI mirrors only decode-address — re-verified 30 `NO_POSITIONALS` use-sites (31 grep lines − 1 definition) + `DECODE_ADDRESS_POSITIONALS` at `mnemonic.rs:3215`. The 7-subcommand enumeration matches the binary exactly (`decode-address` `address` required; `inspect`/`repair`/`verify-bundle` `extra_strings` repeating; 3 xpub-search `positional` repeating). The stale `mod.rs:40-42` doc ("mnemonic-toolkit's subcommands have zero positionals") confirmed present at 4b5bf46; in-cycle errand + FOLLOWUP filing (with the mixed-secrecy/conservative-`true` wrinkle) both in §4; Non-goals reworded. Only residue: the census count (M-r2-2).
- **I2 — FOLDED-OK.** §2 now mandates the full 4-byte literal list with the byte-0 rationale, T5 gains the `Kpub<base58…>` row, and §2 carries the panic-free idiom (M8). **List completeness re-verified against the toolkit gate's complement** (`gate.rs:270-292`): the doc enumerates extended-PRIVATE `xprv/tprv/yprv/zprv/uprv/vprv` "(+ capital variants)" = 10 prv prefixes, whose public complement is exactly the spec's `xpub/tpub/ypub/Ypub/zpub/Zpub/upub/Upub/vpub/Vpub` — and that is the complete SLIP-132 extended-public registration set for Bitcoin (mainnet xpub/ypub/Ypub/zpub/Zpub + testnet tpub/upub/Upub/vpub/Vpub; no `Xpub`/`Tpub` exist in SLIP-132). First-char set {x,t,y,Y,z,Z,u,U,v,V} is disjoint from WIF first chars {5,9,K,L,c} as claimed. The prefix check operates on the origin-stripped part, so suffixed legit forms (`[fp/…]xpub…/<0;1>/*`) survive — correct direction.
- **M1 — FOLDED-OK.** §3 final bullet states VERIFIED + "Don't hunt for inversions". Re-verified: `grep positional tests/persistence.rs` → zero hits.
- **M2 — FOLDED-OK.** §1.2 records the verification (2 non-secret callers, Run not gated) and the permanent invariant with the `secret_widgets["positional:<name>"]` consult rule.
- **M3 — FOLDED-OK.** §2 bullet: `is_xprv_like` doc-pointer + `xprv_hint` widening note naming the silently-blanked classes; render-side hint extension correctly scoped non-blocking.
- **M4 — FOLDED-OK.** §2 bullet documents the `rsplit(']')` `xprv…]xpub` caveat at the new fn; tightening optional.
- **M5 — FOLD-DRIFT (partial).** Non-goals correctly affirms VERIFIED with the `persistence.rs:103-110` citation (re-verified: the `SECRET_SLOT_SUBKEYS` row filter is exactly there), but §4 retains the pre-fold "verify that claim in-cycle" hedge → internal contradiction. See M-r2-1.
- **M6 — FOLDED-OK.** §1.4 serde-shape sentence with `:34` (SCHEMA_VERSION=1), `:168`/`:185` (`save`/`load`) — re-verified, including zero `save(`/`load(` callers in `src/`.
- **M7 — FOLDED-OK.** §1.2 names the seam (direct `SecretLineEdit::show`, do NOT bend `flag_is_secret`/`render_with_dispatch`), chrome keyed off `pos.required`/`pos.repeating`, `FormState.positionals` doc lockstep sentence (`mod.rs:281-284` re-verified), paste-warn mootness noted.
- **M8 — FOLDED-OK.** §2 "panic-free `as_bytes().get(..4)` idiom".

**Round-2 source re-verification (spot-checks beyond the folds, all PASS):** `persistence.rs:115` verbatim clone; `secrets.rs:200-225` `should_confirm_run` has no positional loop; `main.rs:664-677` plain `text_edit_singleline` rows; `invocation.rs:292-296` argv-end emit; `tree_model.rs:176-187`/`:650-653` `redacted_for_persistence`→`blank_xprv_keys`→`is_xprv_like`; `blank_xprv_keys` covers `key` + `keys[i]` + recursive children (the port surface for T5 is as described); `is_xprv_like` callers = exactly {`tree_model.rs` walk, `tree_form.rs:779` hint}; `COMBINE_POSITIONALS` `shares` `required:true, repeating:true` "Secret-equivalent"; flag names carry `--` prefix → `positional:` keying collision-proof; README self-pin at `:42`; FOLLOWUPS.md `positional-secrets-not-redacted-at-persist` entry open + I6 index line at `:18` as cited.

**Whole-spec re-scan:** no new Critical/Important. Checked specifically: required-but-all-empty secret positional (`combine shares`) emits nothing → toolkit clap error, identical to today's empty-positional behavior; ≤1-positional-per-table claim holds so render-loop index alignment can't mix paths; belt-blanked non-secret positionals can't break the two `has_positional` conditionals across a restart (persistence unwired, and the conditionals read live state); MINOR bump is right (bin crate, behavior+schema-struct change); T4's frozen-literal census is non-circular per the I1/I2 mk-cli lesson.

## Verdict

**GREEN — 0 Critical / 0 Important / 3 Minor (incl. one partial fold-drift, M5→M-r2-1).** Implementation may begin. The three minors are prose-level: delete §4's stale "verify in-cycle" hedge, record the accurate 6-of-7 secret-capable census in the FOLLOWUP when filing it, and add the over-acceptance sentence at the new allowlist fn. None blocks Phase 1 RED.
