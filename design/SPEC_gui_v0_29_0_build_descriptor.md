# SPEC — mnemonic-gui v0.29.0: bump toolkit pin v0.47.3 → v0.50.0 + surface `build-descriptor`

**Resolves (toolkit FOLLOWUP, GUI companion):** `gui-build-descriptor-schema-mirror-pending-pin-bump`
(`mnemonic-toolkit/design/FOLLOWUPS.md`).
**Toolkit release at pin target:** `mnemonic-toolkit-v0.50.0` (commit `ecba644`; descriptor-builder engine Release A).
**GUI source SHA at recon:** `bdecfff` (master = HEAD, up-to-date).
**Recon:** `mnemonic-toolkit/cycle-prep-recon-gui-build-descriptor-schema-mirror-pending-pin-bump.md`.
**Model:** `design/SPEC_gui_v0_28_0_pin_bump_v0_47_3.md` (the prior pin-bump cycle; same 6-site lockstep).

---

## 0. Measured-clean drift (load-bearing)

`MNEMONIC_BIN=<local mnemonic built from v0.50.0> cargo test --test schema_mirror mnemonic_schema_flag_names_match_help_text` → **PASS**. The GUI's current v0.47.3-modeled hand schema already matches the **v0.50.0** binary's `gui-schema` for every declared subcommand → **zero accumulated flag-NAME / dropdown value-enum / conditional drift across v0.47.3 → v0.50.0** (the five-release lagging-gate worst case does NOT bite). Per-release: v0.47.4 (self-check ms1, no surface), v0.48.0 (NUMS internal-key WIRE flip), v0.49.0 (BIP-388 wallet-policy JSON as an *input format* on the existing `--descriptor`, not a new flag), v0.49.1 (taproot NUMS restore). None added/renamed a flag or value-enum. **So the pin bump is mechanically clean; the only schema *content* change this cycle is the new `build-descriptor` entry.**

`schema_mirror` loops only the hand schema's **declared** subcommands (`tests/schema_mirror.rs:93`) and asserts per-subcommand flag-NAME **set-equality**; there is no subcommand-completeness gate (`src/schema/mod.rs:22`: the schema is a deliberate "Subset of upstream"). So adding `build-descriptor` is the CLAUDE.md **mirror-invariant** obligation (it is the lone uncovered subcommand of 30), surfaced as a working basic form — NOT a red-gate fix.

---

## 1. The change (two parts)

**(A) Toolkit pin bump v0.47.3 → v0.50.0** — 6 lockstep sites (per the v0.28.0 precedent §3a).
**(B) Surface `build-descriptor`** — one `SubcommandSchema` + its `FlagSchema` array + one dropdown const, plus the GUI self-version bump.

---

## 2. Part A — pin bump (6 sites)

| # | Site | Change | Gate |
|---|---|---|---|
| 1 | `Cargo.toml:42` | `tag = "mnemonic-toolkit-v0.47.3"` → `…-v0.50.0` | `pin_coherence` |
| 2 | `Cargo.lock:2296-2297` | `version` `0.50.0` + `source` rev = the **actual v0.50.0 tag commit** | (resolved) |
| 3 | `pinned-upstream.toml:22` | `[mnemonic].tag = "mnemonic-toolkit-v0.47.3"` → `…-v0.50.0` | `pin_coherence` (CI `schema-mirror.yml` installs this) |
| 4 | `README.md:50` | toolkit install `--tag mnemonic-toolkit-v0.47.3` → `…-v0.50.0` | `readme_pin_coherence` |
| 5 | `src/schema/mnemonic.rs:3688` | `pinned_version: "mnemonic 0.47.3"` → `"mnemonic 0.50.0"` | **UNGATED** (action-bar banner) |
| 6 | `src/schema/mnemonic.rs:1` | module-doc `…-v0.47.3` → `…-v0.50.0` | **UNGATED** (module-doc) |

- **(R0 M4 precedent)** site 2: regenerate via `cargo +1.94.0 update -p mnemonic-toolkit`; **let cargo resolve the tag commit** — do NOT hand-paste the recon SHA `ecba644` into `Cargo.lock` `source` (it must be the tag-object's commit, which is `ecba644` here since the v0.50.0 annotated tag points at it — but resolve it, don't assume).
- Sites 5 + 6 are not gated by any test → must be done by hand (every prior pin-bump cycle bumps them; v0.28.0 R0 I1).
- The `v0.47.3` mentions in `tests/argv_assembler.rs`, `src/form/invocation.rs`, `tests/canonicity_drift.rs`, `design/`, `CHANGELOG.md`, `FOLLOWUPS.md` are **historical version-stamped notes** — NOT pin sites; leave them.

---

## 3. Part B — surface `build-descriptor`

### 3a. New dropdown const (near the other value-enum consts, `src/schema/mnemonic.rs` ~line 29-130)
```rust
const BUILD_FORMATS: &[&str] = &["descriptor", "bip388"];
```

### 3b. New flags array (`BUILD_DESCRIPTOR_FLAGS`)
The flag-NAME set MUST equal the v0.50.0 `mnemonic gui-schema` `build-descriptor` set (verified): `{--format, --json, --network, --no-auto-repair, --spec, --spec-schema}`.
```rust
const BUILD_DESCRIPTOR_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--spec",
        // Toolkit `--spec` is a FILE PATH (or `-` = stdin; omitted ⇒ stdin when
        // not a TTY) — NEVER inline JSON. Model as Path (like `--blob`) so the
        // widget emits a valid path/`-`, not raw JSON (which the toolkit would
        // treat as a path → ENOENT). `schema_mirror` is flag-NAME-only, so the
        // Path-vs-Text kind (gui-schema reports "text") does not drift the gate.
        kind: FlagKind::Path { stdio_sentinel: true },
        required: false,
        repeating: false,
        help: "Path to the JSON policy-tree spec (`-` reads from stdin; omitted \
               reads stdin when not a TTY).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--spec-schema",
        kind: FlagKind::Boolean,
        required: false, repeating: false,
        help: "Dump the versioned node-tree grammar JSON and exit (ignores other inputs).",
        secret: false, default_value: None, global: false,
    },
    FlagSchema {
        name: "--format",
        kind: FlagKind::Dropdown(BUILD_FORMATS),
        required: false, repeating: false,
        help: "Output payload format: `descriptor` (raw) or `bip388` (wallet policy JSON).",
        secret: false, default_value: None, global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false, repeating: false,
        help: "Network (default mainnet).",
        secret: false, default_value: None, global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false, repeating: false,
        help: "Emit the full envelope (descriptor + bip388 + cost + diagnostics) as JSON.",
        secret: false, default_value: None, global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];
```
- `--network` mirrors the existing convention (`Dropdown(NETWORKS), required:false, default_value:None`, line 392-399) — the GUI does NOT model the toolkit's runtime mainnet default as a clap `default_value`.
- `--no-auto-repair` is the shared `NO_AUTO_REPAIR_FLAG` const (`global:true`).
- **(R0 M2) Default-form emission is non-empty, by established convention.** With `default_value: None`, a `Dropdown` widget seeds to `opts.first()` (`src/form/widget.rs:133-135`, reached via the `None` fallback at `:167-168`). So the *default* build-descriptor form emits `--format descriptor --network mainnet` — both valid (`descriptor` = the bare concrete descriptor; mainnet = the toolkit's own default), identical to every existing `--network`/`--format` Dropdown form (e.g. import-wallet `--format`). This is expected, not a defect; `--spec` (Path, empty ⇒ omitted) is the only required user input.

### 3c. New SubcommandSchema entry (in `SUBCOMMANDS`, after `export-wallet` — descriptor family)
```rust
    SubcommandSchema {
        name: "build-descriptor",
        human_name: "Build Descriptor (policy-tree spec → wsh descriptor + BIP-388)",
        flags: BUILD_DESCRIPTOR_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
```
- `conditional: None` — build-descriptor has no clap conflicts/`required_unless`; it is NOT in `conditional_visibility`'s constrained list and needs no conditional fn, and has no `SUBCOMMAND_FLOORS` entry (`gui_schema_conditional_drift` floor is a `>=` minimum → unaffected by a 0-rule subcommand).
- `allows_slots: false` (no `--slot` grammar).
- Placement is cosmetic (`schema_mirror` is set-based); descriptor-family grouping after `export-wallet`.

---

## 4. GUI self-version bump (MINOR)

New surfaced subcommand = capability addition ⇒ **MINOR**. `v0.28.0 → v0.29.0`.
- `Cargo.toml:3` `version = "0.28.0"` → `"0.29.0"`.
- `Cargo.lock` self `mnemonic-gui` `0.28.0` → `0.29.0` (cargo update).
- `README.md:42` self-install pin `mnemonic-gui-v0.28.0` → `…-v0.29.0` (`readme_pin_coherence`).
- `README.md:15` "Released mnemonic-gui-v0.3.0…" is historical prose — leave.
- `CHANGELOG.md` — add a `## mnemonic-gui [0.29.0]` section (the GUI repo DOES maintain a per-release CHANGELOG; the prior entry is `[0.28.0]`).

---

## 5. Verification / test plan (TDD: a failing characterization first)

**RED (new test, fails before 3b/3c land):** add `tests/` assertion (or extend an existing schema test) `build_descriptor_surfaced_with_path_spec`:
- `SUBCOMMANDS` contains `name == "build-descriptor"`;
- its flag-NAME set == `{--format,--json,--network,--no-auto-repair,--spec,--spec-schema}`;
- its `--spec` flag `kind` is `FlagKind::Path { stdio_sentinel: true }` (pins the working-form correctness — a `Text` regression would re-break argv routing).

**GREEN gates (all must pass; `cargo +1.94.0`, `MNEMONIC_BIN`=v0.50.0 binary + MD/MS/MK debug bins for their mirror tests):**
1. `schema_mirror::mnemonic_schema_flag_names_match_help_text` — now also validates `build-descriptor`'s 6 flags vs the v0.50.0 binary (set-equality).
2. `r7_no_auto_repair_removal` — build-descriptor carries the shared `NO_AUTO_REPAIR_FLAG`; the per-subcommand iteration must stay green.
3. `conditional_visibility::coverage_all_constrained_subcommands_have_conditional_fn` — build-descriptor is unconstrained (`conditional:None`); not in either named list → unaffected (verify no exhaustiveness assertion trips).
4. `schema_mirror_secret_drift` — build-descriptor has no `secret:true` flag → unaffected.
5. `gui_schema_conditional_drift` — 0 rules for build-descriptor; `>=35` floor unaffected.
6. `pin_coherence` + `readme_pin_coherence` — all 4 gated pin sites + self-pin lockstep.
7. **Full `cargo +1.94.0 test --workspace`** green (catch any other subcommand-iterating test: `widget_*`, `runner_integration`, `secrets`).

`manual_anchor_coverage` is `#[ignore]`'d and unset in CI (`build.yml`/`schema-mirror.yml` set no `MANUAL_GUI_HTML_PATH`) → does NOT run here. See §7 latent debt.

---

## 6. Cross-repo lockstep / FOLLOWUPs

- **Resolve** the toolkit FOLLOWUP `gui-build-descriptor-schema-mirror-pending-pin-bump` → flip to `resolved` in lockstep (a small `mnemonic-toolkit` doc commit, cross-citing GUI v0.29.0; or the v0.28.0 precedent's deferred-with-M1-note if cross-repo authoring lags).
- **File (GUI root `FOLLOWUPS.md`)** `manual-gui-build-descriptor-anchors-pending-pin-bump`: adding build-descriptor to the live schema creates a latent obligation that the NEXT `docs/manual-gui` pin bump (in the toolkit repo, currently pinned `mnemonic-gui-v0.3.0`) to a build-descriptor-containing GUI version MUST add the `mnemonic-build-descriptor[-…]` anchors, or `manual_anchor_coverage --ignored` + the toolkit `check_gui_schema_coverage.py` lint fail then.

---

## 7. Out of scope

- **The build-descriptor WIZARD** (recursive node-tree builder UI). This cycle adds the **flag-level schema** only → a basic form: a file-picker/stdin for `--spec`, dropdowns for `--format`/`--network`, booleans for `--json`/`--spec-schema`. The structured tree-builder UI is a later GUI cycle (brainstorm "archetype forms first; recursive node-tree builder deferred").
- **`docs/manual-gui` anchors** — pinned to `mnemonic-gui-v0.3.0` (toolkit repo, separate cadence); tracked by the §6 FOLLOWUP.
- **Toolkit-side changes** — none (the toolkit v0.50.0 is shipped; only the FOLLOWUP flip).

---

## 8. SemVer + ship

**MINOR — `mnemonic-gui v0.29.0`.** Release sequence (GUI cadence): bump all version/pin sites → `cargo +1.94.0 test --workspace` green → push master → CI (`build.yml` + `schema-mirror.yml`) green → tag `mnemonic-gui-v0.29.0` (annotated; `^{commit}` resolvable) → push tag.

---

## 9. Source SHAs (for future readers)
- Toolkit pin target: `mnemonic-toolkit-v0.50.0` = `ecba644` (annotated tag).
- GUI base: `bdecfff` (master).
- Recon: `mnemonic-toolkit/cycle-prep-recon-gui-build-descriptor-schema-mirror-pending-pin-bump.md`.
