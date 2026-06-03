# gui-ms1 catch-up — Phase 1 R0 Review
**Verdict:** GREEN (0C/0I)

Phase 1 diff `git diff 16ba363..2306ff5` (6 files). Gate independently re-confirmed by the controller (below).

## Critical (0) / Important (0) / Minor (2)

## A pins + Cargo.lock — ACCURATE + COHERENT
- `Cargo.toml:42` = `mnemonic-toolkit = { git=…, tag = "mnemonic-toolkit-v0.41.0" }` (sole entry, not in dev/build-deps).
- `pinned-upstream.toml`: `[mnemonic].tag` v0.41.0 (:22), `[md]` md-cli-v0.6.2 (:39), `[ms]` ms-cli-v0.7.0 (:46), `[mk]` mk-cli-v0.7.0 (:53); stale `[md]` comment re-worded (:31-37).
- `Cargo.lock`: mnemonic-toolkit 0.41.0 @ `git+…?tag=mnemonic-toolkit-v0.41.0#d8d0170…3733` — rev byte-matches the remote tag + the SPEC's toolkit SHA. Transitive: md-codec 0.35.0 / mk-codec 0.4.0 / ms-codec 0.4.0 (each exactly once, no dupe/downgrade, crates.io+checksum) — exactly toolkit v0.41.0's declared deps. SAFE: GUI's only toolkit use is `secret_taxonomy::{SECRET_NODE_TYPES,SECRET_SLOT_SUBKEYS}` (`src/secrets.rs:34`); never calls the codecs → transitive churn can't affect it; `secret_taxonomy` still `pub mod` at toolkit lib.rs:82.

## B pin_coherence guard — CORRECT
Typed `toml::Value` parse: `cargo["dependencies"]["mnemonic-toolkit"]["tag"]` (inline table → reaches the toolkit dep, no other CLI git-deps) vs `pinned["mnemonic"]["tag"]` (anchors the `[mnemonic]` table, not the 3 sibling `tag=` lines — the M2 typed-parse fix). `assert_eq!` fires on any mismatch (wrong-tag→red→revert→green demo sound). `.expect()` only panics on a malformed manifest (acceptable). Both files = v0.41.0 → PASS.

## C md repair — FIELD-ACCURATE + correctly placed
`md.rs:467-484`: REPAIR_FLAGS = single `--json` (Boolean, non-secret); REPAIR_POSITIONALS = `md1-strings` (required+repeating). Matches md-cli v0.6.2 `repair.rs:42-49` exactly + the `inspect`/`decode` idiom + `schema/mod.rs` field defs. SubcommandSchema appended LAST after `address` (md.rs:553-560) — correct (schema omits hidden gui-schema). Makes schema md set == binary's surfaced set. Coverage-gap closure (ungated), not a red cell.

## D tests/secrets.rs deviation + sweep — NECESSARY + COMPLETE
`tests/secrets.rs:282` → 6-entry `["phrase","seedqr","entropy","ms1","xprv","wif"]` (exact-set BTreeSet ==, would fail otherwise once the v0.41.0 6-entry re-export is in scope). Matches src/secrets.rs:68 snapshot + toolkit secret_taxonomy.rs:111 + slot_input enum order. SWEEP COMPLETE: grepped every SECRET_SLOT_SUBKEYS consumer — persistence.rs:91/184/304/311/316, secret_taxonomy_pin.rs:35/44/56, argv_assembler_slot.rs:203, widget_interaction.rs:71 all read DYNAMICALLY (contains/iterate ALL via is_secret_bearing) → tolerate +ms1; tests/secrets.rs:282 was the ONLY hardcoded full-set literal. (The lone other 5-entry literal is CHANGELOG.md:226, historical prose.)

## E const-asserts + no Phase-2 leak
Const-asserts HOLD at v0.41.0: NODE_TYPES toolkit `:76-85` (8) == fallback `secrets.rs:42-54` (8); SLOT_SUBKEYS toolkit `:111` (6) == fallback `:68` (6) → lib compiles (proof of the draft + SPEC §3). NO Phase-2 leak: `Cargo.toml:3` still "0.21.3"; banners still stale (mnemonic.rs:3452 "0.38.0", md.rs:565 "0.5.0", mk.rs:476 "0.6.0"); README install pins stale; CHANGELOG top still [0.21.3]. Phase 1 confined to its 6-file scope.

## F gate — CONTROLLER-CONFIRMED (architect's harness couldn't run it)
The reviewing agent could not run the workspace gate (sibling binaries absent in its harness; verified GREEN statically). **Controller independently ran it:** 4 binaries built at correct versions (mnemonic 0.41.0 / ms 0.7.0 / md 0.6.2 / mk 0.7.0); `MNEMONIC_BIN/MS_BIN/MD_BIN/MK_BIN` set; `cargo +1.94.0 test --workspace` → **0 FAILED** (all targets ok, ~354 passed incl. pin_coherence + the new md repair cell + secret gates); `cargo +1.94.0 clippy --all-targets -- -D warnings` → **exit 0, clean**. Gate empirically GREEN.

## Minor (2) — non-blocking
- m1 (cosmetic): `src/secrets.rs:62-64` SLOT_SUBKEYS history comment labels the add "v0.41.0 (toolkit v0.41.0)" — the GUI release is v0.22.0 (toolkit 0.41.0). tests/secrets.rs:277 correctly says "v0.22.0". Optionally align the src comment in Phase 2.
- m2 (documented): pin_coherence guards only the TOOLKIT pin; the 3 sibling pins rely on paired-PR + live schema_mirror (SPEC §6-acknowledged scope, in the test header).

## Verdict rationale
Pins correct on all 4 tags; Cargo.lock coherent (toolkit 0.41.0 @ verified rev d8d0170, transitive codec bumps expected + safe); pin_coherence guard reaches the right tags + fails on drift; md repair field-for-field accurate + correctly placed; the tests/secrets.rs deviation was required + the sweep is complete; const-asserts hold (compile proof); no Phase-2 leak; the workspace gate is empirically GREEN (controller-confirmed: 0 fail + clippy clean). No Critical/Important; 2 cosmetic/documented Minors. **GREEN (0C/0I) — proceed to Phase 2.**
