# R0 review — SPEC_gui_v0_31_1_repeating_secrets — round 3

**Verdict: YELLOW** (0C / 1I / 2m — both round-2 folds applied cleanly; the confirm-sweep found ONE more cross-CLI collision the disclosure misses)

## Round-2 fold verification
- **I-NEW1 fold — APPLIED, accurate as far as it goes** (all three xpub-search `--phrase` sites verified verbatim secret:false Text; the plaintext-persistence claim correct; the name-level-net correction present).
- **m1 fold — APPLIED** (`invocation.rs:238-253` verified).

## Critical
None.

## Important
**I-NEW2 — "all inert EXCEPT `--phrase`" is FALSE: `--ms1` has a second undisclosed secret:false twin.** `ms repair` (`src/schema/ms.rs:314-324`) declares `--ms1` Text/secret:false/required — colliding with mnemonic.rs's 7 secret:true `--ms1` sites. Values-routed → the to-be-repaired ms1 string (master-secret material, merely BCH-corrupted) **persists to state.json in plaintext TODAY**; the §3 union silently stops that. Same class + adjudication as `--phrase` (safety-positive, second live leak closed). Blind-spotted because `--ms1` sat in the union's HEADLINE so its own twin was never checked. Fold: extend the disclosure (`--phrase` AND `--ms1`); second FOLLOWUP `ms-repair-ms1-not-secret-classified` at ship.

## Minor
- **m-NEW1 —** adjacent family leak OUTSIDE the flag-name net: the codex32 combine `shares` POSITIONAL (`ms.rs:441-448`, "Secret-equivalent") rides `state.positionals`, cloned unredacted (`persistence.rs:98`). Pre-existing; file a third FOLLOWUP at ship, don't fold.
- **m-NEW2 —** census-method note: raw greps trip on 2 comment lines (`mnemonic.rs:2263/:2268`); the §3 drift test must extract the FIELD, not grep text. All 15 `--network` sites verified secret:false (no Dropdown joins the union).

## Empirical probes run
Full 4-schema (name,kind,secret) tuple census + twin-check of EVERY union name (--decrypt-password/--secret/--digits/--hex/--share/--*-stdin: no false twin, inert confirmed; --passphrase-stdin's ms.rs Boolean twin already name-dropped today → inert; --phrase ×3 disclosed; --ms1 ×1 at ms.rs:316-321 NOT disclosed → I-NEW2); md/mk zero secrets; verbatim reads of all cited sites; git tree = dabbdfe + 3 untracked docs.
