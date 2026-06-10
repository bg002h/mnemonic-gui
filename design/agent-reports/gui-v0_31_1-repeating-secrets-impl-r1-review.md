# Impl review — GUI v0.31.1 repeating secrets — round 1
**Verdict: GREEN** (0C / 0I)

## Critical
None.
## Important
None.
## Minor
1. Pre-existing test flake observed once under full-suite load (cell_2_tracing_init_logs_subprocess_spawn, runner_integration.rs:140-168 — thread-local set_default under parallel threads; passed 5/5 isolated + full rerun green). NOT commit-related; FOLLOWUP note recommended.
2. boolean-stdin FOLLOWUP under-counts: ms.rs:275-281 carries an 18th --passphrase-stdin site (secret:false but name-matched via SECRET_FLAG_NAMES) — also suppressed. Amend to 18.
3. No direct live-form cell for run-confirm-silent-on-blank-required---share; covered by the faithful unit negative (the exact has_value route, code-traced false). Cheap future insurance.
4. Cosmetic: per-row empty-label show renders an empty hover frame.

## SPEC-conformance checklist
§2 assembler == pseudocode token-for-token (kind-gated; Text unconditional continue :272; NVC fall-through :274-277 → generic loop :281-287; Boolean continue :278; supersession comment without the false paste-warn line). §4 all 6 sites (has_value per-row :355-361 — 1-element-empty vec returns FALSE; zeroize flatten :296; scalar entry :86-95; split :83; persistence :120). Deviations both conformant (defensive scalar guard; header-owned chrome). §1 branch order/seed/add/removal/header all verified. §3 field-extracted OnceLock union seeded from SECRET_FLAG_NAMES; redaction at persistence.rs:75/:88-90; both drift tests non-vacuous. §5 THE live-path cell is REAL (clicks add ×2, type_text into Role::PasswordInput nodes, asserts values-free, asserts row-order argv — would have FAILED pre-fix); seed-xor pin byte-unchanged (zero diff lines); migrations + faithful :171 negative + dead-path both-assertions all verified. CHANGELOG/version/README ✓. 4 FOLLOWUPs' citations verified (8-site --ms1 census exact); main FOLLOWUP still open (correct pre-ship).

## Empirical probes run
Full suite ×2 (one pre-existing flake, triaged 5/5 isolated; rerun 405/0); clippy 0; repeating_secret_rows 8/8 verbose; run-confirm regression 8/8; citation greps for all 4 FOLLOWUPs.
