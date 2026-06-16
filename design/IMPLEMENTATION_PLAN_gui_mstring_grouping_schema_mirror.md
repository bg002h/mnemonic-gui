# IMPLEMENTATION PLAN — mnemonic-gui schema-mirror for mstring display-grouping (P5)

**Cycle:** mnemonic-gui v0.40.0 → **v0.41.0 (SemVer MINOR)** · final phase (P5) of the
cross-repo mstring display-grouping cycle.
**Source SHAs:** GUI `master@c5e3434` (v0.40.0, clean). Upstream binaries (all RELEASED,
on crates.io / tagged): toolkit `mnemonic-toolkit-v0.56.0` (= commit
`a1dcff82393a21c24857887ca8475c07d1f2a2ea`), `descriptor-mnemonic-md-cli-v0.7.0`,
`ms-cli-v0.8.0`, `mk-cli-v0.9.0`.
**Companion:** toolkit/sibling `design/FOLLOWUPS.md` `display-grouping-render-strip-v1`;
spec `mnemonic-toolkit/design/SPEC_mstring_display_grouping.md` §11 (lockstep) + §I7
(keyword-dropdown constraint).

---

## 1. What this phase does

P1–P4 added the uniform display-grouping flags `--group-size <u16>` (default 5, `0`=unbroken)
and `--separator` (default `space`; keyword `space|hyphen|comma` or literal) to the emit
subcommands of all four CLIs. The GUI maintains a clap-flag **schema mirror**
(`src/schema/{mnemonic,md,ms,mk}.rs`), gated by `tests/schema_mirror.rs` (flag-NAME set
equality per declared subcommand, run against the binary resolved from `*_BIN` /
`pinned-upstream.toml`). This phase:

1. Bumps the four upstream pins to the grouping-enabled releases.
2. Adds `--group-size` + `--separator` to the exact set of mirrored subcommands that gained
   them, with `--separator` rendered as an I7 **keyword dropdown** (`space|hyphen|comma`).
3. Ships GUI v0.41.0 (paired-PR rule satisfied in-repo).

## 2. Measured delta (authoritative work-list — empirically confirmed, NOT inferred)

Measured by running each GUI `Schema`'s declared subcommands against the **v0.56.0 / 0.7.0 /
0.8.0 / 0.9.0 binaries** via `mnemonic_gui::schema_check::json_flag_names` (the same
`gui-schema` JSON path `schema_mirror` uses). The drift is **purely the grouping flags** —
NO accumulated lagging-gate drift on any other declared subcommand (the v0.53.2–v0.55.3 and
sibling cycles touched no GUI-mirrored subcommand's flag NAMES):

| Schema file | Subcommand | flags to ADD (`only_in_upstream`) |
|---|---|---|
| `mnemonic.rs` | `bundle` | `--group-size`, `--separator` |
| `mnemonic.rs` | `convert` | `--group-size`, `--separator` |
| `mnemonic.rs` | `ms-shares-split` | `--group-size`, `--separator` |
| `mnemonic.rs` | `ms-shares-combine` | `--group-size`, `--separator` |
| `md.rs` | `encode` | `--group-size`, `--separator` |
| `ms.rs` | `encode` | `--group-size`, `--separator` |
| `ms.rs` | `split` | `--group-size`, `--separator` |
| `mk.rs` | `encode` | `--group-size`, `--separator` |

Every diff is `only_in_schema=[]`, `only_in_upstream=["--group-size","--separator"]` — purely
additive. (`mnemonic verify-bundle` did NOT gain them — its forensic output stays unbroken;
`ms combine` did NOT gain them standalone — confirmed absent from the drift.) This is the
COMPLETE set; nothing else to backfill.

## 3. Flag declarations (identical shape every site)

`--group-size` mirrors the existing `--import-json-index` Number pattern (u16 → `Static(65535)`);
`--separator` is an I7 keyword dropdown. Toolkit `gui-schema` reports `--separator` as kind
`text` (custom `value_parser` accepts keyword OR literal, so clap exposes no `choices`); the
GUI **deliberately narrows it to a Dropdown** per spec §I7 (emit keywords only, never a literal
space, to avoid argv/whitespace ambiguity). `schema_mirror` compares flag NAMES only, so the
Dropdown-vs-text kind divergence is invariant-safe (same precedent as the archetype/build-format
dropdowns, whose value-enums `schema_mirror` also does not gate).

Add a per-file keyword const (each schema file defines its own dropdown consts — there is no
shared list module):

```rust
// I7 (SPEC §I7 / §4): the GUI MUST emit the separator as a KEYWORD
// (space|hyphen|comma), never a literal space, to avoid argv/whitespace
// ambiguity through the GUI→argv path. The toolkit's gui-schema reports
// `--separator` as kind `text` (keyword-or-literal value_parser); the GUI
// narrows it to this dropdown. schema_mirror gates flag NAMES only.
const SEPARATORS: &[&str] = &["space", "hyphen", "comma"];
```

Append these two `FlagSchema` entries to each affected subcommand's `*_FLAGS` const (order is
irrelevant — `schema_mirror` uses a `BTreeSet`):

```rust
FlagSchema {
    name: "--group-size",
    kind: FlagKind::Number { min: 0, max: NumberMax::Static(65535) },
    required: false,
    repeating: false,
    help: "Display grouping: break the emitted card into groups of N \
           characters (default 5; 0 = unbroken single line). Cosmetic — \
           intake strips separators, so any grouping re-ingests.",
    secret: false,
    default_value: Some("5"),
    global: false,
},
FlagSchema {
    name: "--separator",
    kind: FlagKind::Dropdown(SEPARATORS),
    required: false,
    repeating: false,
    help: "Display-grouping separator keyword (space|hyphen|comma; \
           default space). Cosmetic — non-load-bearing.",
    secret: false,
    default_value: Some("space"),
    global: false,
},
```

`default_value: Some("5")` / `Some("space")` mirror the toolkit's `gui-schema` `default_value`
(5 / "space") so `form/invocation::is_at_default` suppresses the flag when unchanged (no
spurious `--group-size 5` / `--separator space` in argv). No conditional rules, no slot/secret
interactions; the generic widget path renders both.

## 4. Exact edit sites

### 4.1 `src/schema/mnemonic.rs`
- Add `const SEPARATORS` to the "Shared dropdown option lists" block (after `NODE_TYPES`/etc.).
- Append the two flags to `BUNDLE_FLAGS` (`:183`), `CONVERT_FLAGS` (`:771`),
  `MS_SHARES_SPLIT_FLAGS` (`:1419`), `MS_SHARES_COMBINE_FLAGS` (`:1483`). (Subcommand structs:
  `bundle`→`BUNDLE_FLAGS` `:3664`, `convert`→`CONVERT_FLAGS` `:3680`,
  `ms-shares-split`→`MS_SHARES_SPLIT_FLAGS` `:3843`, `ms-shares-combine`→`MS_SHARES_COMBINE_FLAGS`
  `:3851`.)
- `:1` module-doc `mnemonic-toolkit-v0.53.1` → `-v0.56.0`.
- `:3950` `pinned_version: "mnemonic 0.53.1"` → `"mnemonic 0.56.0"` (verified: `mnemonic
  --version` = `mnemonic 0.56.0`).
- `NumberMax` already imported (`:20`).

### 4.2 `src/schema/md.rs`
- Add `const SEPARATORS`; append the two flags to `ENCODE_FLAGS` (`:59`).
- `:1` module-doc `descriptor-mnemonic-md-cli-v0.6.2` → `-v0.7.0`.
- `:573` `pinned_version: "md 0.6.2"` → `"md 0.7.0"`.
- `NumberMax` already imported (`:14`).

### 4.3 `src/schema/ms.rs`
- **Add `NumberMax` to the `use super::{…}` import (`:10`)** — currently absent.
- Add `const SEPARATORS`; append the two flags to `ENCODE_FLAGS` (`:60`) AND `SPLIT_FLAGS`
  (`:356`). (NOT `COMBINE_FLAGS` — `ms combine` did not gain them.)
- `:1` module-doc `(ms-cli-v0.7.0)` → `(ms-cli-v0.8.0)`.
- `:540` `pinned_version: "ms 0.7.0"` → `"ms 0.8.0"`.

### 4.4 `src/schema/mk.rs`
- **Add `NumberMax` to the `use super::{…}` import (`:10`)** — currently absent.
- Add `const SEPARATORS` (mk.rs has no named dropdown consts today — this is the first; matches
  the per-file pattern); append the two flags to `ENCODE_FLAGS` (`:45`).
- `:1` module-doc `(mk-cli-v0.7.0)` → `(mk-cli-v0.9.0)`.
- `:482` `pinned_version: "mk 0.7.0"` → `"mk 0.9.0"`.

### 4.5 Pin bumps (six edit categories + Cargo.lock; ~12 concrete sites)
- `Cargo.toml:42` toolkit dep `tag = "mnemonic-toolkit-v0.53.1"` → `-v0.56.0`.
- `pinned-upstream.toml`: `[mnemonic].tag :22` → `-v0.56.0`; `[md].tag :39` →
  `descriptor-mnemonic-md-cli-v0.7.0`; `[ms].tag :46` → `ms-cli-v0.8.0`; `[mk].tag :53` →
  `mk-cli-v0.9.0`. Re-word any stale version comments.
- `README.md:50-53` four install lines → the four new tags.
- `Cargo.lock`: toolkit `source`/rev → `a1dcff82393a21c24857887ca8475c07d1f2a2ea`
  (= `mnemonic-toolkit-v0.56.0^{commit}`) + gui `version` 0.40.0→0.41.0. Regenerate via
  `cargo update -p mnemonic-toolkit --precise <rev>` or a plain `cargo build` after the
  Cargo.toml tag edit; confirm the diff is exactly the two stanzas (gui version + toolkit
  source/version), no collateral dep churn.

### 4.6 Version + changelog
- `Cargo.toml:3` `version = "0.40.0"` → `"0.41.0"`.
- `CHANGELOG.md`: new `## mnemonic-gui [0.41.0] — 2026-06-15` section (MINOR — additive flags +
  new keyword dropdown).
- **`README.md:42` self-pin `mnemonic-gui-v0.40.0` → `mnemonic-gui-v0.41.0` (MANDATORY, M1).**
  `tests/readme_pin_coherence.rs::readme_install_tags_match_pins` is a no-skip pure-logic gate
  that HARD-asserts the README `mnemonic-gui` self-tag == `mnemonic-gui-v{Cargo.toml version}`
  AND the four sibling install-tags (README:50-53) == `pinned-upstream.toml` tags. So §4.5's
  README edits + this self-pin together satisfy `readme_pin_coherence`.

## 5. TDD / RED→GREEN sequence (per-commit-green discipline)

The natural RED is already demonstrated: with pins bumped (or `*_BIN` pointed at the new
binaries) and the schema unedited, `schema_mirror` is RED on exactly the 8 instances in §2.

1. **Commit 1 (RED→GREEN, atomic): schema flags + pin bumps together.** Adding the flags
   WITHOUT bumping pins would make `schema_mirror` RED the OTHER direction (`only_in_schema`)
   against the still-pinned pre-grouping CI binaries; bumping pins WITHOUT the flags is RED as
   in §2. They MUST land together so every committed state is green against its own pinned
   binaries. Single commit: §4.1–§4.5.
   - Gate: `MNEMONIC_BIN=… MD_BIN=… MS_BIN=… MK_BIN=… cargo test --test schema_mirror` → all
     4 CLIs GREEN.
2. **Commit 2: version bump + CHANGELOG (§4.6).**
3. Full verification (below) before tag.

(No new GUI unit test is required: `schema_mirror` IS the RED-driver and the GREEN gate — it
already fails on the missing flags and passes once added. A bespoke "grouping flags present"
assertion would duplicate it. Optionally add a focused regression cell asserting
`bundle`/`encode` schema flag-sets contain the two names, if the architect wants belt-and-braces.)

## 6. Full verification (before tag)

**Step 0 (M3 — do FIRST, before staging Commit 1):** edit `Cargo.toml:42` tag → v0.56.0 and
`cargo build`. The compile-time secret_taxonomy guard at `src/secrets.rs:78-99` is a
`const _: () = assert!(secret_slice_eq(...))` — a drift between the committed snapshot and the
v0.56.0 `secret_taxonomy` constants fails the BUILD (not a test). Given the v0.53.1→v0.56.0 jump
(multiple intervening cycles), this is load-bearing. Grouping touched no secret taxonomy so it's
EXPECTED clean; if it fails, treat as a separate audit-style reconcile (out-of-scope-but-blocking)
and surface to the user before proceeding.

Then, with all four `*_BIN` env vars pointed at the new local binaries:
- `cargo test` (whole suite) GREEN — especially `schema_mirror` (4 CLIs),
  `readme_pin_coherence` (no-skip — README self-tag + sibling install-tags; M2),
  `schema_mirror_secret_drift` (toolkit-v5 `secret=true` set vs GUI hand-code; new flags are
  `secret:false` both sides → green but exercised; M2),
  `gui_schema_conditional_drift` (no new conditional rules — pre-confirmed 5/5 vs v0.56.0),
  `archetype_schema_mirror` (build-descriptor value-enums — pre-confirmed clean vs v0.56.0),
  `pin_coherence` (Cargo.toml tag == pinned-upstream `[mnemonic].tag`),
  `canonicity_drift`, `non_canonical_descriptor_account_pin`, `secret_taxonomy_pin`.
- `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check` (GUI uses stable fmt;
  no mlock exemption here).

## 7. Risks / gotchas

- **R1 — pins resolve against the REMOTE.** The Cargo git-dep + CI installs fetch tags from
  GitHub. All four tags are pushed (toolkit v0.56.0 force-moved to a1dcff8; siblings released).
  Pre-flight: `git ls-remote --tags <repo> <tag>` for each.
- **R2 — `--separator` kind: Dropdown (GUI) vs text (toolkit).** Intentional per I7; `schema_mirror`
  is flag-NAME-only → safe. Do NOT "fix" the GUI to `Text` to match the binary.
- **R3 — `ms.rs`/`mk.rs` missing `NumberMax` import** → compile error if forgotten. Add to the
  `use super::{…}` line in both.
- **R4 — `pinned_version` strings are the literal `<bin> --version` output.** Verified:
  `mnemonic 0.56.0` / `md 0.7.0` / `ms 0.8.0` / `mk 0.9.0`. Wrong strings would mislead the
  action-bar label (and any version-pin gate).
- **R5 — Cargo.lock churn.** Bound the diff to the two expected stanzas; reject collateral.
- **R6 — secret_taxonomy compile-time guard** (see §6) — only fires on the Cargo bump.

## 8. Release ritual

- `schema-mirror.yml` fires on master push AND `mnemonic-gui-v*` tags; it installs the pinned
  tags and runs the gate — so the bumped `pinned-upstream.toml` is validated in CI.
- Tag `mnemonic-gui-v0.41.0` after master is green. Version-marker gates (README + CHANGELOG
  `## mnemonic-gui [0.41.0]`). GUI ships its own namespace; nothing flows back to the toolkit.
- Stage paths explicitly (no `git add -A`). Commit trailer:
  `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.

## 9. R0 gate

This plan-doc passes opus architect R0 (0C/0I) BEFORE any edit; review persisted verbatim to
`design/agent-reports/` then re-dispatched after each fold until GREEN, per the toolkit
CLAUDE.md convention this cycle has followed at every level.
