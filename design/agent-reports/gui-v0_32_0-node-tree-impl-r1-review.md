# Impl review — GUI v0.32.0 node-tree builder (P1+P2+P3) — round 1
**Verdict: YELLOW** (0C / 1I / 5M — one missing SPEC-§5 cell; everything load-bearing verified green against the real v0.52.0 binary)

## Critical
None.
## Important
**I1 —** SPEC §5's "surplus-children flag renders + does not emit": the does-not-emit half is covered three ways, but NO test renders a surplus tree and asserts the amber label — deleting the render line leaves the suite green while a user sees a branch that silently doesn't emit (the v0.31.0-C1 funds-safety inversion surface). One kittest cell closes it.
## Minor
**M1** tree-mode Preview shows bare argv vs the pipeline the POSIX button copies (cosmetic). **M2** Edit-as-tree silently overwrites a pre-existing hand-built tree (FOLLOWUP). **M3** strip wire-absence untested (one token). **M4** xprv-like content in hex/w persists unredacted (free belt-and-suspenders; FOLLOWUP). **M5** the filed tracing flake reproduced once — fold the serialization fix into this ship.

## SPEC-conformance checklist (adjudications)
Mode mutex PASS (2 Disabled + 10 Hidden; 12-name partition pinned; stale-everything + restore cells). Parse contract: **descriptor-first discriminator DEVIATION ACCEPTED** (probed: the success envelope carries BOTH descriptor and diagnostics:[] — the SPEC's literal wording would mis-route success; intent preserved: stdout-parse-keyed, never exit-code-keyed; SPEC erratum required). stdin discipline PASS (write→drop→wait; BrokenPipe degrades; run() byte-identical; cells). Grammar mirror PASS (both version pins; ran green). Model PASS (preserve-and-flag; structural-only completeness; both round-trip laws; depth 64 at all 3 surfaces). **Keystone PASS, RED-on-drift verified** (serialization never consults child_path → a drift fails both assertions). next_id invariant PASS at both import sites. Persistence + redaction PASS (recursive incl. surplus; migration cell). mark_dirty structural-diff: **"provably not" CONFIRMED** (all writes mutate root; collapse lives in egui memory keyed by push_id — cannot trip a PartialEq diff; refusals append post-diff). Completeness gates Validate/Run/Copy PASS. Validate PASS (fixed argv + --allow pass-through; fail-soft strip cells; structurally cannot write last_run — the lib has no App access). **P3 purpose-built emit-spec argv DEVIATION ACCEPTED** (probed: --emit-spec clap-conflicts with --json AND --format — assemble_argv reuse would break the feature; pinned incl. the exclusion cell). POSIX pipeline PASS (shared posix_quote; shlex round-trip; real /bin/sh exec leg).

## Empirical probes run
Full suite 459/0 (one filed-flake repro, 5/5 isolated); clippy -D warnings clean; the 3 gates + tree_form verbose (1/1, 7/7, 13/13, 21/21); the success-envelope probe; the --emit-spec conflict probe; fixtures byte-equal upstream; origin == d2fe58b.

## Ship checklist
Fold I1 (+M3/M5) → CHANGELOG [0.32.0] → version bump + README self-tag → SPEC erratum for the two accepted deviations → push → CI → tag → toolkit install.sh pin + BRAINSTORM §5 annotation → file M2/M4 FOLLOWUPs.
