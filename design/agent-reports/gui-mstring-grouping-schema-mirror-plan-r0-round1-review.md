# Plan-R0 (GUI P5 schema-mirror) round 1 — architect review (verbatim)

> Persisted verbatim per CLAUDE.md BEFORE the fold. Dispatched via Agent
> (feature-dev:code-architect, opus). **Verdict: GREEN (0C/0I)** — 4 Minors
> (M1–M4), all non-blocking doc/sequencing refinements. Plan SHA at review:
> GUI `master@c5e3434`; plan-doc `design/IMPLEMENTATION_PLAN_gui_mstring_grouping_schema_mirror.md`.
> The architect re-verified every cited line/const/import/pin against live source.

---

## Verbatim agent output

This confirms the version-marker sites that must move from `0.40.0`→`0.41.0`:
- `Cargo.toml:3` `version = "0.40.0"` (plan §4.6 ✓)
- `README.md:42` self-pin `mnemonic-gui-v0.40.0` (plan §4.6 flags it conditionally; `readme_pin_coherence.rs` makes it mandatory)
- `Cargo.lock` gui `version` (plan §4.5 ✓)
- CHANGELOG new entry (plan §4.6 ✓)

The `0.40.0` references in CHANGELOG history, FOLLOWUPS, design docs, src comments are all historical/documentary — NOT version markers that need bumping. Good.

I now have full coverage. Let me write the verdict. I found no Critical and no Important blockers. All edit sites, const names, line numbers, import lines, pin sites, the Cargo.lock rev, the per-commit-green ordering logic, the I7 dropdown invariant-safety, and the gate coverage are correct. I have a handful of Minor refinements.

---

## R0 Architect Review — `IMPLEMENTATION_PLAN_gui_mstring_grouping_schema_mirror.md`

# GREEN (0C/0I)

The plan is correct, complete, and per-commit-green. Every cited line number, const name, import line, pin site, and the Cargo.lock rev verified against live source. The I7 Dropdown-vs-text divergence is invariant-safe under the flag-NAME-only gate. No Critical or Important findings.

## Verification log (what I confirmed against live source)

**Edit-site accuracy — ALL EXACT:**
- `FlagSchema` field set in `src/schema/mod.rs:87-133` = `name, kind, required, repeating, help, secret, default_value, global` — byte-matches the plan's two literals (§3). No missing/extra field; will compile.
- `src/schema/mnemonic.rs`: `BUNDLE_FLAGS:183`, `CONVERT_FLAGS:771`, `MS_SHARES_SPLIT_FLAGS:1419`, `MS_SHARES_COMBINE_FLAGS:1483` ✓; subcommand structs `bundle:3664→flags:3666`, `ms-shares-split:3843→3845`, `ms-shares-combine:3851→3853` ✓; module-doc `:1`=v0.53.1 ✓; `pinned_version "mnemonic 0.53.1":3950` ✓; `NumberMax` imported at `:20` ✓.
- `src/schema/md.rs`: `ENCODE_FLAGS:59` ✓; `:1`=md-cli-v0.6.2 ✓; `pinned_version "md 0.6.2":573` ✓; `NumberMax` imported at `:14` ✓.
- `src/schema/ms.rs`: `ENCODE_FLAGS:60`, `SPLIT_FLAGS:356` ✓; `COMBINE_FLAGS:426` correctly EXCLUDED; `:1`=ms-cli-v0.7.0 ✓; `pinned_version "ms 0.7.0":540` ✓; `use super::{…}` at `:10` lacks `NumberMax` (existing uses are `super::NumberMax` at :379/:389) — plan's add is correct and conflict-free.
- `src/schema/mk.rs`: `ENCODE_FLAGS:45` ✓; `:1`=mk-cli-v0.7.0 ✓; `pinned_version "mk 0.7.0":482` ✓; `use super::{…}` at `:10` lacks `NumberMax`, and mk.rs ENCODE_FLAGS uses zero NumberMax today (all Text/Boolean) — import add is genuinely required.
- `--group-size`/`--separator` appear NOWHERE in the schema today (additive, not duplicate). The `--import-json-index` precedent (`mnemonic.rs:243-246`) is `Number { min: 0, max: NumberMax::Static(65535) }` — byte-matches the plan's `--group-size`. `SEPARATORS` is not an existing const (no collision); the "Shared dropdown option lists" block at `mnemonic.rs:27` is the correct home.

**Completeness vs measured delta:** The plan touches exactly the 8 drifted (Schema, subcommand) pairs and only those. `ms combine` (COMBINE_FLAGS, declared at `ms.rs:529→531`) is correctly excluded; `verify-bundle` is correctly excluded. No flag is added that the binary lacks (no `only_in_schema` inversion).

**Pin-bump completeness — ALL sites enumerated.** Live grep of the old pins across non-`design/` source returns: `Cargo.toml:42`, `pinned-upstream.toml:{22,32(comment),39,46,53}`, `README.md:{50,51,52,53}`, `Cargo.lock:2297`, and the four `schema/*.rs:1` module-docs — every one is covered by §4.1–§4.5 (the `:32` stale comment is caught by "re-word any stale version comments"). No hardcoded tag in `.github/workflows/` (the workflow reads `pinned-upstream.toml` via `steps.pins.outputs.<cli>_tag`, pinned by `schema_mirror.rs::ci_workflow_snapshot`). `pin_coherence.rs` is satisfied by the lockstep `Cargo.toml`↔`pinned-upstream [mnemonic].tag` bump. **Cargo.lock rev confirmed:** `mnemonic-toolkit-v0.56.0` → `a1dcff82393a21c24857887ca8475c07d1f2a2ea` (verified against the GitHub release page); current lock rev is `87c33c5…` (v0.53.1), so the precise update is right.

**Per-commit-green ordering — CORRECT.** schema-flag-adds and pin-bumps MUST be one atomic commit: adding flags without bumping pins → RED `only_in_schema` against the still-pinned pre-grouping binaries; bumping pins without flags → RED `only_in_upstream`. The plan's Commit-1 atomicity is sound; no intermediate-RED hazard within Commit 1.

**Gate coverage:** `schema_mirror.rs` (line 53) compares `f.name` only — it IS both RED-driver and GREEN gate; a bespoke test is genuinely redundant. I read every gate the bump could trip:
- `gui_schema_conditional_drift.rs` — conditional-rules only; grouping flags have none → green.
- `archetype_schema_mirror.rs` / `spec_nodes_mirror.rs` / `build_descriptor_schema.rs` — `build-descriptor --spec-schema` archetypes/nodes/value-enums; mstring cycle touched none → green. `SEPARATORS` is not a build-descriptor enum, so it's never value-gate-checked anywhere.
- `xpub_search_schema_mirror.rs` — iterates only the four xpub-search subcommands; none gain grouping → green.
- `secret_taxonomy_pin.rs` — min-membership runtime pin; grouping added no secret taxonomy → green.
- `default_form_state.rs::default_bundle_form_state_cli_accepts` — emits argv WITHOUT the grouping flags (they're not in the explicit `from_pairs` seed); v0.56.0 accepts the optional-defaulted flags → green.

**I7 correctness — invariant-safe.** Confirmed via `schema_mirror.rs:52-53` that only names are compared, so Dropdown(GUI)-vs-text(toolkit) never trips the gate (same precedent as `BUILD_FORMATS`/`ARCHETYPES`). `SEPARATORS=["space","hyphen","comma"]` with **no** empty sentinel is correct: `is_at_default` (`invocation.rs:84`) suppresses a Dropdown whose value equals `default_str`, and `emit_one` (`invocation.rs:413`) skips an empty Dropdown — so `default_value: Some("space")` cleanly suppresses the default with no `""` sentinel needed. The archetype `""` sentinel exists for a different reason (its dropdown has NO toolkit default and auto-seeding to the first real value would emit a guaranteed-refusal `--archetype …`); `--separator`'s toolkit default IS `"space"`, so the suppression path differs and a sentinel is unnecessary.

## Minor (non-blocking — fold at author's discretion)

- **M1 — `README.md:42` self-pin is MANDATORY, not conditional.** §4.6 hedges ("if the README pins its own `mnemonic-gui-v*`… check `:42`"). It does (`mnemonic-gui-v0.40.0`), and `tests/readme_pin_coherence.rs::readme_install_tags_match_pins` (a no-skip pure-logic gate) HARD-asserts `mnemonic-gui` self-tag == `mnemonic-gui-v{Cargo.toml version}` AND the four sibling install-tags == `pinned-upstream.toml` tags. De-hedge §4.6 to "bump `README.md:42` → `mnemonic-gui-v0.41.0`" and explicitly name `readme_pin_coherence` as a gate satisfied by §4.5 (README:50-53) + this self-pin. This is the test most likely to fire if the version-marker step is skipped.

- **M2 — Name `tests/readme_pin_coherence.rs` and `tests/schema_mirror_secret_drift.rs` in §6's pre-clear list.** §6 enumerates the gates to run but omits these two. `readme_pin_coherence` is no-skip and WILL run on every `cargo test`. `schema_mirror_secret_drift` (skip-capable) WILL execute when `MNEMONIC_BIN`=v0.56.0 per §6's own setup, and asserts the toolkit-v5 `secret=true` set equals the GUI hand-code set; the new flags are `secret:false` on both sides, so it stays green — but it's exercised, not skipped, so it belongs in the list.

- **M3 — Strengthen R6 / §6's secret_taxonomy framing from "verify by building" to "MUST `cargo build` before Commit 1."** The guard at `src/secrets.rs:78-99` is a compile-time `const _: () = assert!(secret_slice_eq(...))` — it fails the BUILD (not a test), so a drift makes Commit 1 un-compilable. Given the toolkit jumped v0.53.1→v0.56.0 (multiple intervening cycles), the verify-by-build is genuinely load-bearing. The plan already calls this out correctly; just elevate it to an explicit ordered step *before* staging Commit 1 (a `cargo build` after the Cargo.toml tag edit, before the schema edits even, surfaces it earliest).

- **M4 — §4.5 header "the 6-site discipline" undercounts.** The actual non-`design/` pin sites are ~12 (Cargo.toml ×1, pinned-upstream ×5 incl. the `:32` comment, README ×4, Cargo.lock ×1, plus the 4 module-docs enumerated in §4.1–§4.4). The body is complete; only the "6-site" label is a slight misnomer (it's 6 *edit categories*). Cosmetic — re-word to avoid a future reader treating "6" as a checklist count.

All Minors are documentation/sequencing refinements; none gates implementation. The plan may proceed to implementation as written (folding M1–M3 is recommended for release safety).
