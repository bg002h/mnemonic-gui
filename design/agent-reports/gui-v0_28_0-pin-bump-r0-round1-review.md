# R0 Architect Review — mnemonic-gui v0.28.0 pin-bump-v0.47.3 — Round 1

> Persisted verbatim from the opus `feature-dev:code-architect` agent
> (`agentId: a55ec206c4fa63eac`). Had Read/Glob/Grep; verified against both repos.

---

**VERDICT: 0 Critical / 1 Important (+ 4 Minor)**

### Critical (0) — None.

### Important (1)

**I1 — §3a change-list is incomplete: two ungated lockstep updates omitted (`pinned_version` banner + module-doc header).** SPEC §3a lists four files; two standard lockstep updates are missing and neither is gated by any test:
- `src/schema/mnemonic.rs:3688` — `pinned_version: "mnemonic 0.46.2"` → `"mnemonic 0.47.3"` (the action-bar "Pinned:" label rendered in the running GUI). Every prior cycle's CHANGELOG calls this out.
- `src/schema/mnemonic.rs:1` (module-doc header) — `mnemonic-toolkit-v0.46.2` → `v0.47.3`.
**Fix:** add both to §3a. After this fold the SPEC is complete.

### Minor (4)

**M1 — README self-install tag (line 42) not in §3a** — `--tag mnemonic-gui-v0.27.0` → `v0.28.0`. GATED by `tests/readme_pin_coherence.rs` (self-tag vs Cargo.toml version) → immediate RED if forgotten. Add to §3a for completeness.

**M2 — `src/form/invocation.rs:75-76` comment becomes mildly stale** (`// Timestamp: Now matches "now"; Epoch(n) never matches "now"`). Logic stays correct (arm hardcodes `default_str == "now"`); only the comment's spirit drifts. No code change required for correctness.

**M3 — `cell_3_export_wallet_range_timestamp_argv` comment at line 97** ("emits regardless of the `--timestamp now` schema default") drifts — default is now `"0"`. Assertion stays correct; only the comment.

**M4 — Cargo.lock source rev** — `cargo update` resolves the actual `mnemonic-toolkit-v0.47.3` tag commit; do NOT hand-paste the recon SHA `d509361` into Cargo.lock's `source`.

### Verified Clean
1. **Minimal fix correct + complete.** toolkit `export_wallet.rs:212` `default_value = "0"`; `parse_timestamp:311` still accepts `"now"`. `widget.rs:166-188` seeds `FlagKind::Timestamp` → `FlagValue::Unset` (default form emits nothing → toolkit applies `0`). `seeded_value_for:201` → `Now` on click. `is_at_default:78` `Now => "0"=="now"` = false → explicit Now emits `--timestamp now` (bug fixed); `Unix(_) => false` always emits. Schema `default_value` change ONLY; NO `is_at_default` change. §2a sound.
2. **Test blast radius complete.** The only `Some("now")` in the codebase is `src/schema/mnemonic.rs:1044`. The 4 called-out tests are the complete set; nostr `--timestamp` is a separate `FlagKind::Text` already at `"0"`.
3. **schema_mirror does not gate default_value.** `GuiSchemaFlag` deserializes `name` only (`schema_check.rs:99-104`); `schema_flag_names` collects `f.name` only (`schema_mirror.rs:52-54`). Fix is for correctness, not a gate.
4. **No manual-gui change in the GUI repo.** The GUI repo has no `docs/` at all. (Operator note: `docs/manual-gui/` lives in the TOOLKIT repo and DOES carry stale `now`-default prose at `45-export-wallet.md:30,340-343` — a separate toolkit-repo loose end, see fold below.)
5. **Pin-bump completeness.** `Cargo.toml:42`, `Cargo.lock:2296-2297`, `pinned-upstream.toml:22`, `README.md:50`. `pin_coherence.rs` asserts Cargo.toml tag == pinned-upstream tag; `readme_pin_coherence.rs` asserts README `cargo install --tag` lines match. md/ms/mk pins current.
6. **SemVer MINOR** v0.27.0→v0.28.0 — consistent with the v0.22–v0.27 pin-bump convention; conservative for a behavior-correctness fix.
7. **Help string update in scope** (§3b) — correct.

**After folding I1 (+ M1): 0 Critical / 0 Important → GREEN.**

---

## Operator note (cross-repo manual-gui finding)
The architect's "GUI repo has no `docs/`" surfaced that `docs/manual-gui/` is a **TOOLKIT-repo** doc tree (the GUI user manual, separate pinned cadence). It carries stale `now`-default prose: toolkit `docs/manual-gui/src/40-mnemonic/45-export-wallet.md:30` ("default `now`") + `:340-343` ("`now` (the default; emits the literal string `"now"`…)"). The `expected_gui_schema_inventory.json` `--timestamp` entry has NO `default_value` field (not stale, not gated). This is OUT of scope for this GUI-repo cycle → file a TOOLKIT FOLLOWUP (`manual-gui-export-wallet-timestamp-default-now-stale`) for the next manual-gui cadence touch. SPEC §3d updated to clarify.
