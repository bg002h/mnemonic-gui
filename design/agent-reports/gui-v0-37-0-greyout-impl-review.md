# Implementation review — GUI v0.37.0 grey-out stdin toggles (2026-06-10)

Reviewer: Fable 5 implementation-review agent (post-impl, pre-commit). Spec @ design/SPEC_gui_v0_37_0_greyout_stdin_toggles.md (R0 GREEN r1). Verdict: GREEN (0 Critical / 0 Important / 2 non-action Minor). Review verbatim below.

---

## Critical
None.
## Important
None.

## Minor
1. **Converse-closure is a leading gate once the flag exists in a schema (by design).** T1's converse-closure iterates flags present in the 4 SCHEMA consts; a future secret Path/Number trips RED when added to a schema. Verified the assertion fires by constructing a synthetic secret `Path` satisfying `secret && !Text && !Composite && !Boolean`. Intended semantics; no change.
2. **`render_help_icon` in the new branch is a guaranteed no-op for these flags** (`needs_help_icon` returns true only for Dropdown/Composite/TaggedOrIndexed/repeating; these are Boolean+repeating:false). Harmless, consistent with every other render path. The disabled-checkbox tooltip is the help affordance.

## Verdict
**GREEN (0 Critical / 0 Important).**
- **Render branch (widget.rs:165-188):** correctly after the secret-Text gate (:89) + before repeating(:196)/scalar(:214); predicate `flag_is_secret && Boolean`; `add_enabled(false, Checkbox::new(&mut unchecked, flag.name))` + tooltip + early return (no state.values writeback). Non-secret Booleans + secret Text untouched.
- **Predicate-mirror EXACT (empirical census):** SECRET_TOTAL=64 = TEXT=39 + COMPOSITE=1 + BOOL=24 + **OTHER=0** → converse-closure non-vacuously satisfied today. 24 secret Booleans / 6 names matches CHANGELOG+FOLLOWUPS (passphrase-stdin×13, ms1-stdin×3, phrase-stdin×3, decrypt-password-stdin×2, secret-stdin×2, bip38-passphrase-stdin×1). Suppression at invocation.rs:255/278.
- **T1 non-vacuous** (visits all 24 secret Booleans); converse-closure RED-on-secret-Path proven. **T2 discriminates** (scratch-revert → t2 FAILED at :108, t1 stayed green; widget restored md5-identical).
- **Ritual:** CHANGELOG [0.37.0] accurate; version coherent at Cargo.toml + Cargo.lock + README pin; FOLLOWUPS resolved with the user grey-out decision + predicate-mirror record; secrets.rs 5→6 Boolean comment correct (exactly 6 distinct secret:true Boolean names). **invocation.rs byte-identical** → assemble_argv unchanged (emitted nothing before AND after); schema_mirror + drift gates green.
- **Full suite GREEN** (release + pinned BINs), clippy `-D warnings` clean. Tree left exactly as found.
