# Phase 7 Secrets + build.rs Codegen Review — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Phase 7 landing commit
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §9 + §10 + §B.11 + §C Phase 7

## Verdict

**1C / 1I — fold needed** (both folded inline)

15 hot spots evaluated. 1 critical (rerun-if-changed gap), 1 important (test-side parser fence weaker than build.rs).

---

## Critical findings

### C-1 — `build.rs` missing `rerun-if-changed` on resolved upstream source files

**Confidence:** 91
**File:** `build.rs:33-36` (pre-fold rerun directives) + post-resolution gap

Pre-fold rerun directives covered only `pinned-upstream.toml`, `build.rs`, and two env-var sentinels. The resolved `<upstream_root>/<workspace_member>/src/cmd/convert.rs` and `.../slot_input.rs` were NOT registered.

**Impact:** Under the primary dev workflow (`MNEMONIC_GUI_UPSTREAM_ROOT` pointing at a local mnemonic-toolkit checkout), editing the upstream source files would NOT invalidate cached `secrets_generated.rs`. `cargo build` would silently ship stale `SECRET_*` constants. The source-audit test catches drift at `cargo test` time, but not at build time.

**Fold:** After resolving + existence-checking both file paths, emit:
```rust
println!("cargo:rerun-if-changed={}", convert_path.display());
println!("cargo:rerun-if-changed={}", slot_input_path.display());
```
Placed inside the post-resolution path (before the codegen logic) so the stub-fallback path correctly skips (no path to watch).

---

## Important findings

### I-1 — `source_audit::collect_variants` lacked the target-type fence build.rs uses

**Confidence:** 82
**File:** `tests/schema_mirror.rs::source_audit::collect_variants` (pre-fold)

Pre-fold test-side walker accepted any `Pat::Path` / `Pat::TupleStruct`, taking the last segment unconditionally. `build.rs::extract_variant_ident` filters to two-segment paths of shape `target_type::Variant` or `Self::Variant`.

**Impact:** Currently harmless (upstream `is_secret_bearing` bodies use only `Self::Variant` patterns). But the two parsers have different acceptance criteria; a future upstream change introducing a foreign path could pass through the test-side walker while build.rs rejects it. The cross-check invariant weakens silently.

**Fold:** Thread `target_type: &str` through `collect_variants_filtered` (the renamed function takes an `accept: impl Fn(&syn::Path) -> bool` closure that mirrors build.rs's guard). `two_segment_guard()` helper applies the same filter as build.rs.

---

## Hot-spot resolution

| # | Hot spot | Verdict |
|---|----------|---------|
| 1 | Step-1 fall-through on non-dir env var | Lenient but spec-consistent; not a bug |
| 2 | syn AST multi-line `matches!` extraction | Token-stream flattening works; correct |
| 3 | R4 C-1 panic contract | Load-bearing, names variant, repo URL; correct |
| 4 | should_confirm_run positionals gap | All mnemonic subcommands have NO_POSITIONALS; not reachable |
| 5 | SECRET_FLAG_NAMES completeness | --ms1 uses inline `secret: true`; --mk1/--md1 correctly non-secret |
| 6 | zeroize non-string FlagValues | Skipped correctly (no secret payload) |
| 7 | dev-deps duplication of build-deps | Correct (integration tests need dev-deps) |
| 8 | mutated_convert.rs fixture integrity | Phrase in as_str + omitted from is_secret_bearing; correct |
| 9 | quote_to_string for syn::Type | Correct (ToTokens impl present) |
| 10 | PASTE_WARN_MODAL_TEXT byte-exact | Spot-checks adequate; below threshold |
| 11 | Avoid-reclone branch on existing target dir | Correct |
| 12 | run_confirm_silent path | Correct |
| 13 | String::zeroize clears length | Correct |
| 14 | SecretBuffer Drop pattern | Equivalent to Zeroizing<String>; correct |
| 15 | rerun-if-changed for upstream source | **→ C-1, folded** |
| extra | source_audit fence | **→ I-1, folded** |

---

## Post-fold test totals

  argv_assembler         10/10
  argv_assembler_slot     5/5
  conditional_visibility 13/13
  copy_command           15/15
  path_detect             9/9
  runner_integration      3/3
  schema_mirror           8/8  (3 source-audit cells + 5 flag-name)
  secrets                18/18

= 81 total tests across 8 binaries. No warnings.
