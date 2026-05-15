# mnemonic-gui v0.3 — Plan (brainstorming + SPEC + implementation plan)

Plan-mode artifact. Mirrors the in-repo `design/SPEC_*` + `design/PLAN_*`
style per `[[feedback-plan-artifact-mirror-project-convention]]`. Three
sections each reviewer-iterated to LOCK (0C/0I) with
`feature-dev:code-reviewer` + `model: "opus"`
(`[[feedback-opus-primary-review-agent]]`) before ExitPlanMode.

Working tree: `/scratch/code/shibboleth/mnemonic-gui` (DIFFERENT from
this session's primary cwd `/scratch/code/shibboleth/mnemonic-toolkit`).

Companion FOLLOWUP: `mnemonic-gui/FOLLOWUPS.md` entry
`slip39-gui-schema-flattening-companion` (status `unblocked`, filed
2026-05-14 at `fd64e1b`).

## Table of contents

- [Context](#context)
- [Section 1 — Brainstorming](#section-1--brainstorming)
- [Section 2 — SPEC](#section-2--spec)
- [Section 3 — Implementation plan](#section-3--implementation-plan)

---

## Context

`mnemonic-gui v0.2.0` shipped 2026-05-12 with 4 CLI surfaces
(`mnemonic` / `md` / `ms` / `mk`), the SPEC §7 `gui-schema` JSON
contract (Phase C.1–C.3), and 5 of the 8 user-facing top-level
`mnemonic` subcommands wired into `src/schema/mnemonic.rs`: `bundle`,
`verify-bundle`, `convert`, `export-wallet`, `derive-child`. (The
toolkit also exposes a 9th top-level `gui-schema` self-introspection
subcommand, filtered from its own JSON output at
`crates/mnemonic-toolkit/tests/cli_gui_schema.rs`.) The mnemonic-toolkit
pin was `mnemonic-toolkit-v0.9.0`.

Three toolkit cycles have shipped since v0.2.0 — adding 5 new
`mnemonic` subcommand surfaces (post-flattening):

| Toolkit cycle | Tag                          | NEW `mnemonic` surface(s)                                      |
|---------------|------------------------------|-----------------------------------------------------------------|
| v0.11.0       | `mnemonic-toolkit-v0.11.0`   | `final-word` (BIP-39 N-1 → checksum-valid Nth word set)         |
| v0.12.0       | `mnemonic-toolkit-v0.12.0`   | `seed-xor split` / `seed-xor combine` (Coldcard XOR splitter)   |
| v0.13.0       | `mnemonic-toolkit-v0.13.0`   | `slip39 split` / `slip39 combine` (Trezor SLIP-39 K-of-N)       |

v0.13.0 P2.1 also fixed the `gui-schema` JSON emitter so nested
subcommands flatten to hyphenated names: `seed-xor split` →
`seed-xor-split`, `seed-xor combine` → `seed-xor-combine` (and
likewise for `slip39`). That fix repairs both the v0.12.0 (seed-xor)
and v0.13.0 (slip39) discoverability gaps in the same commit. The
toolkit's `tests/cli_gui_schema.rs` was bumped from 7 → 10
subcommands at P2.1 RED. The pre-RED probe at toolkit `81488e3`
confirmed `mnemonic-gui` v0.2 cannot see nested subcommands today.

The v0.3 cycle picks up the now-unblocked
`slip39-gui-schema-flattening-companion` FOLLOWUP. Goal: bump the
pinned toolkit tag, refresh the schema-mirror test fixture, and add
GUI surfaces for the still-missing subcommands.

---

## Section 1 — Brainstorming

**Status:** R0 ITERATE folded → R1 PENDING.

### 1.1 Scope decision — what subcommands to add to the schema

The FOLLOWUP enumerates 4 sub-items and explicitly names only
`slip39-split` + `slip39-combine` as new GUI surfaces. But the GUI's
v0.2 schema is also missing `final-word` (v0.11.0) and the
seed-xor pair (v0.12.0). The schema-mirror CI gate doesn't fail on
missing subcommands (it only checks set-equality of flag names for
subcommands the schema lists — see `tests/schema_mirror.rs:90-106`),
so absence is not a regression; it's a UX gap.

Four scoping options were presented:

| Option | What ships in v0.3 schema/mnemonic.rs                                          | Net new surfaces | LOC est.* |
|--------|---------------------------------------------------------------------------------|------------------|-----------|
| A. Minimum (FOLLOWUP-as-written) | `slip39-split` + `slip39-combine` | 2 | ~150 |
| B. Standard (catch up share-splitting) | adds `seed-xor-split` + `seed-xor-combine` to A | 4 | ~250 |
| **C. Comprehensive (selected)** | adds `final-word` to B | **5** | **~350** |
| D. Maintenance-only | none — just bump pin + tag in schema_mirror | 0 | ~30 |

*LOC estimates are coarse (R0 N-2 fold + amended-plan R1 n-1
fold). Per-component: each `FlagSchema` block runs ~120 LOC per
~14-flag subcommand in `schema/mnemonic.rs`. New surfaces here
range 3–8 flags so per-subcommand LOC is smaller (~40–80) plus
conditional fn (~20) plus kittest cell (~30) plus
SubcommandSchema entry (~10). **Drift fix adds ~220 LOC** (4 new
FlagSchema entries ~32, 1 new conditional fn ~15, 3 fn extensions
~15, 8 new conditional cells ~160, -3 from stale-comment
deletion). Revised Option C estimate: **~570 LOC**, not ~350.
Section 3 plan tightens these.

**Decision (user 2026-05-14): Option C — Comprehensive.**

Rationale (folds R0 architect-note "Option B is well-reasoned" but
extends to C per the user's pick):

- C reaches full parity with toolkit v0.13.0's user-facing CLI
  surface. No mnemonic subcommand is silently missing from the GUI.
- C bundles 3 deferred surfaces (`final-word` from v0.11.0,
  seed-xor pair from v0.12.0, slip39 pair from v0.13.0) into one
  reviewer-loop pass instead of fragmenting across 2-3 mini-cycles.
- The kittest convention from v0.2 D.4 (one cell per new surface
  class) extends cleanly: 5 cells, one per subcommand.
- `final-word` is a different workflow shape from split/combine
  (one phrase in, set of candidate words out — no share-set
  semantics). Carrying it in C does add surface heterogeneity, but
  the schema-mirror discipline (clap-flag-name set-equality)
  handles all three shapes uniformly.

### 1.2 The "audit dispatch" item is vacuous

The FOLLOWUP's sub-item 3 says "audit any GUI surface that
dispatched on the now-removed `seed-xor` name." Grep
of `mnemonic-gui/src/**` and `mnemonic-gui/tests/**` returns ZERO
hits for `seed-xor` / `seed_xor` / `slip39` / `final-word` /
`final_word`. Pre-RED probe at toolkit `81488e3` confirmed the
upstream schema returned `{flags: [], positionals: []}` for the
nested-parent `seed-xor` name, so the GUI's `schema_check.rs::
json_flag_names` consumer would have rendered an empty flag set
(no widgets, no dispatch path). The FOLLOWUP itself flags this
("**Verify before assuming a no-op**" at FOLLOWUPS.md line 76).
R0 reproduced the grep — zero hits in `src/**` and `tests/**`;
matches only inside `FOLLOWUPS.md` itself.

**Disposition:** verification step folds into Section 3 P0 as a
single grep cell (no-op confirmed); no dispatch fix needed. The
"audit dispatch" sub-item is closed at P0 verification, not as a
separate phase.

### 1.3 Substantive design — five new GUI surfaces

All five surfaces share core infrastructure (R0 architect-note):

- `--from phrase=<v-or->` composite (FlagKind::NodeValueComposite),
  re-using the widget at `src/schema/mnemonic.rs:393` (`convert
  --from`). `seed-xor split` and `final-word` restrict to `phrase`
  only; `slip39 split` accepts `phrase` OR `entropy`.
- `--passphrase` XOR `--passphrase-stdin` (slip39 only) — same
  conditional shape as `convert` (see `form/conditional.rs:89-101`).
- `--json-out` path flag — emits a SPEC §2.6 world-readable
  advisory via the toolkit's `secret_advisory::warn_if_world_readable`
  (`crates/mnemonic-toolkit/src/secret_advisory.rs:47`). The advisory
  is stderr-side; the GUI's existing runner already surfaces
  stderr as warnings, no GUI-side wiring needed.
- `--language` dropdown using the existing `LANGUAGES` constant at
  `src/schema/mnemonic.rs:38-49` (10 BIP-39 languages).
- Multi-secret stdout for `*-split` outputs is already handled by
  the v0.2 runner (same pattern the bundle subcommand uses for
  three-card stdout).

#### 1.3.1 `slip39-split` (8 flags)

Source: `cmd/slip39.rs:80-139`.

| Flag                       | Kind                                | Required | Notes                                                                                |
|----------------------------|-------------------------------------|----------|--------------------------------------------------------------------------------------|
| `--from`                   | NodeValueComposite(`phrase`,`entropy`) | yes   | `phrase=<v-or->` OR `entropy=<hex-or->`. `=-` reads from stdin. Secret VALUE.        |
| `--passphrase`             | Text                                | no       | SLIP-39 passphrase. `conflicts_with passphrase_stdin`. Secret.                       |
| `--passphrase-stdin`       | Boolean                             | no       | XOR with `--passphrase`.                                                              |
| `--group-threshold`        | Number                              | yes      | K of the group layer. No clap range; library-enforced 1..=group_count.                |
| `--group`                  | Text, REPEATING                     | yes      | `N,T` composite. Repeating; argv position == `group_idx` in `BadGroupSpec` refusals.  |
| `--iteration-exponent`     | Number { min:0, max:15 }            | no       | E. Default 0. Library-enforced 0..=15 (R0 N-1 fold: NOT clap-parser-enforced; GUI's `Number { max:15 }` is defensive mirror). G9 advisory at E≥5. |
| `--language`               | Dropdown(LANGUAGES)                 | no       | Default `english`. Selects BIP-39 wordlist for parsing `--from phrase=…`; ignored when `--from entropy=…` (R0 I-2 fold — INPUT side, NOT round-trip). Hidden when `--from` node != phrase. |
| `--json-out`               | Path { stdio_sentinel: false }      | no       | World-readable-path advisory at `secret_advisory::warn_if_world_readable`.            |

#### 1.3.2 `slip39-combine` (6 flags)

Source: `cmd/slip39.rs:141-178`.

| Flag                       | Kind                                | Required | Notes                                                                                |
|----------------------------|-------------------------------------|----------|--------------------------------------------------------------------------------------|
| `--share`                  | Text, REPEATING                     | yes      | One per share; ≥K total required. At most ONE may be `-` (stdin). Secret VALUE.       |
| `--passphrase`             | Text                                | no       | SLIP-39 passphrase used at split time. `conflicts_with passphrase_stdin`. Secret.    |
| `--passphrase-stdin`       | Boolean                             | no       | XOR with `--passphrase`.                                                              |
| `--to`                     | Dropdown(`entropy`,`phrase`)        | **no**   | Default `entropy` (R0 I-1 fold — has `default_value = "entropy"` at `slip39.rs:167`; variant order `Entropy`→`Phrase`). |
| `--language`               | Dropdown(LANGUAGES)                 | no       | Default `english`. Used only when `--to phrase`; ignored for `--to entropy`. Hidden when `--to entropy`. |
| `--json-out`               | Path { stdio_sentinel: false }      | no       | Same advisory.                                                                       |

#### 1.3.3 `seed-xor-split` (5 flags)

Source: `cmd/seed_xor.rs:40-70`. `slip39.rs` line-7 docstring
documents the structural mirror; the differences here:

| Flag                       | Kind                                | Required | Notes                                                                                |
|----------------------------|-------------------------------------|----------|--------------------------------------------------------------------------------------|
| `--from`                   | NodeValueComposite(`phrase` only)   | yes      | Refuses non-phrase nodes at runtime (`seed_xor.rs:115-119`). Schema can model as restricted-choices NodeValueComposite or plain Text + runtime check. |
| `--shares`                 | Number { min:2, max:255 }           | yes      | Count of XOR shares. Library-enforced ≥ 2.                                            |
| `--language`               | Dropdown(LANGUAGES)                 | no       | Default `english`. Used for both input parse + output share emit.                     |
| `--deterministic-from-master` | Boolean                          | no       | Coldcard SHA256d-deterministic share generation. SPEC §2.6 row 5: 15/21-word + this flag emits a Coldcard-interop advisory. |
| `--json-out`               | Path { stdio_sentinel: false }      | no       | Same advisory.                                                                       |

#### 1.3.4 `seed-xor-combine` (4 flags)

Source: `cmd/seed_xor.rs:72-95`.

| Flag                       | Kind                                | Required | Notes                                                                                |
|----------------------------|-------------------------------------|----------|--------------------------------------------------------------------------------------|
| `--share`                  | NodeValueComposite(`phrase` only), REPEATING | yes | `phrase=<v>` or `phrase=-`. At most ONE may be stdin. Secret VALUE.                  |
| `--shares`                 | Number { min:2, max:255 }           | yes      | Asserted count; library-enforced equal to actual `--share` count.                     |
| `--language`               | Dropdown(LANGUAGES)                 | no       | Default `english`.                                                                    |
| `--json-out`               | Path { stdio_sentinel: false }      | no       | Same advisory.                                                                       |

#### 1.3.5 `final-word` (3 flags)

Source: `cmd/final_word.rs:22-48`.

| Flag                       | Kind                                | Required | Notes                                                                                |
|----------------------------|-------------------------------------|----------|--------------------------------------------------------------------------------------|
| `--from`                   | NodeValueComposite(`phrase` only)   | yes      | `phrase=<n-1 words>` or `phrase=-`. Partial must be 11/14/17/20/23 words.            |
| `--language`               | Dropdown(LANGUAGES)                 | no       | Default `english`.                                                                    |
| `--json-out`               | Path { stdio_sentinel: false }      | no       | Same advisory. Plain candidate list is still emitted to stdout.                       |

**`--from` modeling note (R1-pending):** three of the five surfaces
restrict `--from` to `phrase` only (`seed-xor-split`,
`seed-xor-combine`, `final-word`). Two modeling choices:

- **(a)** Re-use `NodeValueComposite(&["phrase"])` — schema-level
  restricted-choice dropdown. Pro: GUI presents a clear single-choice
  affordance. Con: minor schema-enum gymnastics; the existing
  `NodeValueComposite` variant takes a `&[&str]` of allowed nodes.
- **(b)** Plain `FlagKind::Text` with the value-format string
  documented in `help`. Pro: less schema work. Con: GUI rendering
  is a free-form text field, harder UX.

Recommendation: **(a)** — single-element `NodeValueComposite(&["phrase"])`
mirrors how `derive-child --from` restricts to `["xprv", "phrase"]`
(`schema/mnemonic.rs:661`). Section 2 SPEC will codify.

### 1.4 Reviewer-loop / convergence rails

Each of the three sections gets its own reviewer-loop. R0
preconditions per `[[feedback-r0-must-read-source-off-by-n]]`:

- R0 of Section 1: source-grep verify every claim about
  toolkit/GUI source (file paths, function names, flag names, line
  numbers). Off-by-N narrative pattern is durable. **[Done — see
  §1.6 R0 fold log; ITERATE 0C/2I/2N/2n converged to R1.]**
- R0 of Section 2: verify SPEC-style claims (acceptance gates,
  refusal classes citation, schema enum variants) against actual
  source on disk. Cannot reference a refusal class that does not
  exist in `crates/mnemonic-toolkit/src/cmd/slip39.rs`.
- R0 of Section 3: verify phase boundaries against TDD discipline
  (RED tests can RUN; GREEN tests can pass; LOCK tests gate
  regression). Per `[[feedback-r2-blocking-vs-cosmetic-gate]]`,
  anything that prevents a test from running is Important.

Reviewer dispatch invariant: `model: "opus"` (Sonnet was demoted
to trivial fold-verify per `[[feedback-opus-primary-review-agent]]`).
Architect of each section MUST run the prose's commands end-to-end
per `[[feedback-architect-must-run-prose-commands]]` — for Section
2 (SPEC) and Section 3 (plan) this means the reviewer should be
able to enumerate every test cell + every flag-add + every fixture
diff from the prose alone and find no surprises against source.

**Plan-mode reviewer-output convention (new):** when the
architect-reviewer runs in plan mode, it cannot write
`design/agent-reports/v0_3_*-r*.md` to disk (only the plan file is
writable). Reports are delivered inline in the agent's final message
and folded into the plan file's section-specific R*n* review log. Post-
ExitPlanMode, the architect can optionally persist them under
`design/agent-reports/` for archival parity with prior cycles.

### 1.5 Resolved scope answers (Q1–Q4)

- **Q1 (scope):** **Option C selected** — add `slip39-split` +
  `slip39-combine` + `seed-xor-split` + `seed-xor-combine` +
  `final-word` (5 new surfaces).
- **Q2 (egui_kittest coverage):** **One cell per subcommand** — 5
  new kittest cells covering split-archetype (slip39-split,
  seed-xor-split) + combine-archetype (slip39-combine,
  seed-xor-combine) + final-word's distinct shape.
- **Q3 (release cadence):** **Ship `mnemonic-gui-v0.3.0`** — bump
  Cargo.toml, CHANGELOG entry, git tag, GitHub release with
  binaries. Mirrors v0.2.0 PR-CI-then-tag-push discipline (see
  FOLLOWUPS.md "Process notes" §"v0.2: enforce PR-CI gate before
  tag-push").
- **Q4 (cross-repo loose ends):** at cycle close, the GUI-side
  FOLLOWUP `slip39-gui-schema-flattening-companion` marks
  `resolved <commit>`, the GUI-side `mnemonic-gui-schema-mirror`
  pinned-tag table row for `mnemonic-toolkit` bumps from
  `mnemonic-toolkit-v0.9.0` to `mnemonic-toolkit-v0.13.0`, and the
  toolkit-side `slip39-shamir-secret-sharing` FOLLOWUP (already
  marked resolved at toolkit v0.13.0) needs no further change.
  No NEW cross-repo loose ends are anticipated; if Section 2 SPEC
  drafting surfaces any, fold there.

### 1.6 R*n* review log (Section 1)

**R0 (opus, 2026-05-14):** ITERATE 0C/2I/2N/2n.

- **I-1 folded** (§1.3.2): `slip39-combine --to` corrected to
  required=no, default `entropy`, variant order `Entropy`→`Phrase`.
  Ground truth: `cmd/slip39.rs:167` `default_value = "entropy"`.
- **I-2 folded** (§1.3.1): `slip39-split --language` corrected to
  INPUT-side only ("Selects BIP-39 wordlist for parsing `--from
  phrase=…`; ignored when `--from entropy=…`. Hidden when `--from`
  node != phrase"). Ground truth: `cmd/slip39.rs:131-133` doc-
  comment "BIP-39 language of input phrase; ignored for `entropy=`
  inputs."
- **N-1 folded** (§1.3.1): `--iteration-exponent` note tightened to
  "library-enforced 0..=15 (NOT clap-parser-enforced); GUI's
  `Number { max:15 }` is defensive mirror."
- **N-2 folded** (§1.1): LOC-estimate caveat added below the
  options table; tighter per-surface estimates noted; "Section 3
  plan tightens these" forward-reference.
- **n-1 folded** (Context): "5 of the 8 top-level subcommands"
  clarified to "user-facing top-level subcommands" with
  parenthetical noting the 9th `gui-schema` self-introspection
  command is filtered.
- **n-2 folded** (§1.3): split table now has 8 rows
  (full enumeration of `cmd/slip39.rs:80-139`); combine table has 6
  rows (full enumeration of `cmd/slip39.rs:141-178`); "(more —
  verify in P0)" placeholder removed.

**Plan-mode reviewer-output convention (R0 architect-note):** R0
delivered inline because plan mode forbids writes outside the plan
file. Captured in §1.4 as the new convention. R1 onward follows
the same pattern.

**Scope flip (user 2026-05-14, post-R0):** R0 reviewed against
Option B recommendation; user picked Option C. §1.1, §1.3, §1.5
updated accordingly. R1 should re-verify the three new surface
tables (§1.3.3 seed-xor-split, §1.3.4 seed-xor-combine, §1.3.5
final-word) against `cmd/seed_xor.rs` and `cmd/final_word.rs`.

**R1 (opus, 2026-05-14):** **LOCK 0C/0I/0N/0n.**

All 6 R0 folds verified PASS (I-1, I-2, N-1, N-2, n-1, n-2 — see
reviewer report). All 3 Option-C-new tables source-truth verified
PASS against `cmd/slip39.rs`, `cmd/seed_xor.rs`, `cmd/final_word.rs`.
Infrastructure-sharing claims (NodeValueComposite at
schema/mnemonic.rs:393, `warn_if_world_readable` at
secret_advisory.rs:47, LANGUAGES at schema/mnemonic.rs:38-49,
`derive-child --from` 2-element NodeValueComposite at line 661) all
verified.

**R1 architect-notes carried forward into Section 2 SPEC drafting:**

- **Terminology micro-precision:** §1.3.4 row `--shares` says
  "library-enforced equal to actual `--share` count," but the
  cardinality check at `cmd/seed_xor.rs:228-234` is handler-side
  (CLI), not in `mnemonic_toolkit::seed_xor` library proper.
  Section 2 SPEC for `seed-xor-combine` schema entry: phrase as
  "handler-side runtime check"; GUI mirror = `Number { min:2 }`
  defensive + "verify cardinality" remark.
- **Section 2 R0 must source-truth** the enum-variant ordering AND
  help-text byte-faithfulness of all five new FlagSchema blocks
  against `crates/mnemonic-toolkit/src/cmd/*.rs` doc-comments. New
  help-text strings the SPEC codifies are at maximum off-by-N risk
  per `[[feedback-r0-must-read-source-off-by-n]]`.
- **Section 3 R*n* must run a real kittest cell scaffold** (or at
  minimum a `cargo test --no-run -p mnemonic-gui` parse-check)
  before LOCK; per `[[feedback-r2-blocking-vs-cosmetic-gate]]` a
  parse-failing test is Important regardless of test-outcome
  reasoning.

**Section 1 LOCKED. Architect proceeds to Section 2 SPEC drafting.**

---

## Section 2 — SPEC

**Status:** DRAFT — pending R0 dispatch.

This SPEC pins the v0.3 contract for `mnemonic-gui`. It is an
amendment to (not replacement of) the v0.2 in-repo conventions
(`tests/schema_mirror.rs` shape, `src/schema/` module organization,
`src/form/conditional.rs` fn-pointer pattern, `pinned-upstream.toml`
resolution order). Five new mnemonic subcommand surfaces land per
Option C; one toolkit pin bumps three minor versions; no platform
or widget-layer changes.

### §2.0 Scope

**In scope (v0.3):**

1. Add 5 new `SubcommandSchema` entries to
   `src/schema/mnemonic.rs::SUBCOMMANDS`: `slip39-split`,
   `slip39-combine`, `seed-xor-split`, `seed-xor-combine`,
   `final-word`. Flag tables: see §1.3.1–§1.3.5.
2. Add 2 new conditional-visibility fns to
   `src/form/conditional.rs`: `slip39_split` and `slip39_combine`
   (seed-xor and final-word have no clap conflicts; they take
   `conditional: None` in the SubcommandSchema).
3. Bump `pinned-upstream.toml` `[mnemonic].tag` from
   `mnemonic-toolkit-v0.9.0` → `mnemonic-toolkit-v0.13.0` and
   `pinned_version` constant in `schema/mnemonic.rs` from
   `"mnemonic 0.9.0"` → `"mnemonic 0.13.0"`.
4. Bump `tests/schema_mirror.rs::ci_workflow_snapshot::required_tags`
   first entry to match (3).
5. Add 5 egui_kittest cells (one per new subcommand) in either
   `tests/widget_interaction.rs` or a new `tests/widget_slip39.rs`
   (TBD at Section 3 P1).
6. Update `.github/workflows/schema-mirror.yml` to install
   `mnemonic-toolkit-v0.13.0` instead of `v0.9.0`.
7. CHANGELOG entry + Cargo.toml `version = "0.3.0"` bump + git tag
   `mnemonic-gui-v0.3.0` + GitHub release (Q3 = ship).

**Out of scope (v0.3 — deferred to v0.4 or filed as FOLLOWUPS):**

- BIP-85 application enumeration drift, if any v0.13.0 toolkit work
  changed the BIP85_APPLICATIONS list. (Not expected per the cycle
  changelog; verify at P0.)
- New `FlagKind` variants — all five new surfaces use existing
  variants (NodeValueComposite, Text, Number, Boolean, Path,
  Dropdown). The `--group N,T` composite is modeled as plain Text
  with `repeating: true`; a polished GroupSpec widget is deferred
  to a v0.4 FOLLOWUP (per §1.3 design discussion).
- BIP-39 final-word candidate-list output secret-handling: the
  toolkit emits candidate words to stdout one per line; the GUI's
  runner already treats stdout as secret-bearing for `bundle` /
  `seed-xor split`. Same pattern applies; no new widget needed.
- v0.4+ items listed in `FOLLOWUPS.md` "Deferred to v0.3+" section
  (`gui-code-signing-mac-developer-id`,
  `gui-code-signing-windows`,
  `gui-os-snapshot-secret-occlusion-linux`) remain deferred.

### §2.1 Acceptance gates

LOCK criteria for the cycle. Each gate is a discrete test artifact.

- **G1 (schema-mirror flag-set parity):**
  `cargo test --test schema_mirror mnemonic_schema_flag_names_match_help_text`
  passes against a locally-installed
  `mnemonic-toolkit-v0.13.0` binary (sets `MNEMONIC_BIN` env-var).
  Set-equality check at `tests/schema_mirror.rs:90-106` succeeds
  for all 10 mnemonic SubcommandSchema entries (5 existing + 5
  new). Existing 4 sibling-CLI tests (md/ms/mk) remain green —
  their pinned tags do not change.

- **G2 (CI workflow snapshot):**
  `cargo test --test schema_mirror ci_workflow_snapshot` passes.
  `tests/schema_mirror.rs::ci_workflow_snapshot` (lines 391-453)
  asserts the workflow YAML contains the new tag string
  `mnemonic-toolkit-v0.13.0` in `required_tags`.

- **G3 (conditional-visibility cells):**
  `cargo test --test conditional_visibility` adds new cells
  covering `slip39_split`, `slip39_combine`, AND the 4 drift-fix
  conditionals (amended-plan R1 I-3 fold — XOR cells are
  bidirectional, so each XOR pair = 2 cells):
    - slip39-split: passphrase XOR passphrase-stdin (2 cells,
      both directions) + `--language` Hidden when `--from` node
      == entropy (1 cell) → 3 cells.
    - slip39-combine: passphrase XOR passphrase-stdin (2 cells)
      + `--language` Hidden when `--to` == "entropy" (1 cell)
      → 3 cells.
    - bundle: passphrase XOR passphrase-stdin (2 cells, both
      directions).
    - verify-bundle: passphrase XOR passphrase-stdin (2 cells).
    - convert: bip38-passphrase XOR bip38-passphrase-stdin (2
      cells — NEW pair; the existing passphrase XOR cells are
      pre-existing and unchanged).
    - derive-child: passphrase XOR passphrase-stdin (2 cells;
      was `conditional: None`).
  **Total: 14 new conditional-visibility cells.**

- **G4 (egui_kittest widget-driving cells):**
  5 new cells, one per subcommand, exercising the SUBCOMMAND-tab
  rendering + at least one widget interaction (argv-assembly inspection):
    - `widget_slip39_split_argv_assembles` (or in
      widget_interaction.rs): set required flags, inspect argv.
    - `widget_slip39_combine_argv_assembles`.
    - `widget_seed_xor_split_argv_assembles`.
    - `widget_seed_xor_combine_argv_assembles`.
    - `widget_final_word_argv_assembles`.

- **G5 (pinned-upstream invariant):** the existing test at
  `tests/schema_mirror.rs:627-650`
  (`pinned_upstream_gui_schema_capable_all_true_at_c3`) continues
  to pass — toolkit pin is still `gui-schema-capable = true`.

- **G6 (source-audit secret-* regen):** the existing tests at
  `tests/schema_mirror.rs:322-378`
  (`source_audit_secret_node_types_matches_generated` +
  `source_audit_secret_slot_subkeys_matches_generated` +
  `source_audit_detects_mutation`) continue to pass with the
  bumped pin. If `crates/mnemonic-toolkit/src/cmd/convert.rs` or
  `src/slot_input.rs` changed the `is_secret_bearing()` variant
  set between v0.9.0 and v0.13.0, build.rs regenerates the
  SECRET_* arrays from v0.13.0 source automatically (provided
  `MNEMONIC_GUI_UPSTREAM_ROOT` resolves to a v0.13.0 checkout, or
  `MNEMONIC_GUI_ALLOW_UPSTREAM_CLONE=1`). Verify at Section 3 P0
  probe.

- **G7 (release artifact integrity):** the v0.2 fix at
  `tests/schema_mirror.rs::ci_build_version_step_present` (lines
  471-534) continues to pass; the build.yml `compute-version`
  step is preserved; artifact-name templates use `env.VERSION`
  exactly 4 times. No changes expected to build.yml beyond what
  G2 requires.

### §2.2 Schema additions (SubcommandSchema metadata)

Five new entries appended to `SUBCOMMANDS` in
`src/schema/mnemonic.rs` (after `derive-child` row). Per-entry
metadata:

| `name` | `human_name` | `allows_slots` | `conditional` |
|--------|--------------|----------------|---------------|
| `slip39-split` | `SLIP-39 Split (K-of-N share splitter)` | `false` | `Some(crate::form::conditional::slip39_split)` |
| `slip39-combine` | `SLIP-39 Combine (reconstruct from shares)` | `false` | `Some(crate::form::conditional::slip39_combine)` |
| `seed-xor-split` | `Seed XOR Split (Coldcard all-or-nothing splitter)` | `false` | `None` |
| `seed-xor-combine` | `Seed XOR Combine (reconstruct from XOR shares)` | `false` | `None` |
| `final-word` | `Final Word (BIP-39 N-1 → candidate Nth words)` | `false` | `None` |

`allows_slots: false` for all five — none of the new subcommands
take the v0.4 unified `--slot @N.<subkey>=<value>` input grammar
(only `bundle`, `verify-bundle`, `export-wallet` do). `positional_args:
NO_POSITIONALS` for all five (none take positionals; all input
flows via flags).

Flag tables: §1.3.1 (slip39-split, 8 flags), §1.3.2
(slip39-combine, 6 flags), §1.3.3 (seed-xor-split, 5 flags),
§1.3.4 (seed-xor-combine, 4 flags), §1.3.5 (final-word, 3 flags).
**Help-text strings** in each FlagSchema's `help` field are
informative GUI tooltips, NOT byte-faithful upstream `--help`
mirrors — `tests/schema_mirror.rs:90-106` only checks
flag-name set-equality (see extracted `--<flag-name>` token
regex at lines 19-43), not help-text equality.

### §2.3 Conditional-visibility fn specs

Two new fns in `src/form/conditional.rs`:

```rust
/// `slip39-split` conditionals.
///
/// Upstream (`crates/mnemonic-toolkit/src/cmd/slip39.rs`):
///   :101 `--passphrase` conflicts_with = "passphrase_stdin"
///   :131 `--language`   doc: "BIP-39 language of input phrase; ignored for `entropy=` inputs"
pub fn slip39_split(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_passphrase = state.has_value("--passphrase");
    let has_passphrase_stdin = state.has_value("--passphrase-stdin");
    let from_node = state.composite_node("--from");  // returns Option<&str>

    // passphrase XOR passphrase-stdin (mirrors convert.rs conditional shape).
    if has_passphrase {
        vis.push(("--passphrase-stdin", Visibility::Disabled));
    }
    if has_passphrase_stdin {
        vis.push(("--passphrase", Visibility::Disabled));
    }

    // --language ignored when --from node is entropy.
    if from_node == Some("entropy") {
        vis.push(("--language", Visibility::Hidden));
    }
    vis
}

/// `slip39-combine` conditionals.
///
/// Upstream (`cmd/slip39.rs`):
///   :157 `--passphrase` conflicts_with = "passphrase_stdin"
///   :170 `--language`   doc: "BIP-39 language for `--to phrase`; ignored for `--to entropy`"
pub fn slip39_combine(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_passphrase = state.has_value("--passphrase");
    let has_passphrase_stdin = state.has_value("--passphrase-stdin");
    let to_value = state.dropdown_value("--to");  // existing helper

    if has_passphrase {
        vis.push(("--passphrase-stdin", Visibility::Disabled));
    }
    if has_passphrase_stdin {
        vis.push(("--passphrase", Visibility::Disabled));
    }
    // --language ignored when --to == "entropy" (the default).
    // Use `Hidden` (not `Disabled`) to match the `md_encode`
    // precedent at conditional.rs:153 ("--language hidden when
    // --hex supplied").
    if to_value == Some("entropy") || to_value.is_none() {
        vis.push(("--language", Visibility::Hidden));
    }
    vis
}
```

**`FormState::composite_node` helper:** `FormState` lives at
`src/schema/mod.rs:142` (struct) + `impl` block at line 164.
Existing methods: `has_value` (line 205), `has_positional` (line
218), `dropdown_value` (line 226). **`composite_node` does NOT
exist** (grep-confirmed; no method with that name in the impl
block). Two implementation options:

- **(a)** Add `pub fn composite_node(&self, flag: &str) -> Option<&str>`
  to `FormState`. It reads the NodeValueComposite slot for the
  given flag and returns the node-token (e.g., `Some("phrase")`).
  Mirrors `dropdown_value`'s shape.
- **(b)** Defer the `--language` Hidden-when condition for split
  (§2.3 slip39_split) to v0.4. Simpler v0.3; user sees `--language`
  visible even when entropy mode (harmless — just unused).

**Recommendation: (a)** — small helper, ~10 LOC, gives the GUI
clean conditional support for any future NodeValueComposite-driven
visibility. Section 3 P0 verifies the FormState API; if
`composite_node` doesn't exist, Section 3 P1 adds it as a RED-test
driver before the `slip39_split` fn is written.

`seed_xor_split`, `seed_xor_combine`, `final_word` have NO
clap-conflicts and NO node-dependent flag visibility per source
inspection (§1.3.3–§1.3.5). They map to `conditional: None` in the
SubcommandSchema.

**Handler-side cardinality check note (R1 architect-note fold):**
`seed-xor-combine --shares` asserts `args.share.len() == args.shares`
at `cmd/seed_xor.rs:228-234` — a **handler-side runtime check**,
not a clap or library constraint. The GUI's `FlagKind::Number { min:2,
max:255 }` is a defensive mirror; the cardinality coherence is
verified at runtime (CLI invocation, error surfaces on stderr).
No GUI-side pre-check is required at v0.3.

### §2.4 Pinned-upstream + schema-mirror fixture changes

Three file edits:

1. **`pinned-upstream.toml`:** `[mnemonic].tag` field
   `mnemonic-toolkit-v0.9.0` → `mnemonic-toolkit-v0.13.0`. All
   other fields unchanged. (md/ms/mk pins do NOT bump — those
   sibling-repo cycles haven't shipped since v0.2; only the
   mnemonic-toolkit pin bumps.)

2. **`src/schema/mnemonic.rs`:**
   - Top-of-file doc-comment line 1 (`Pinned schema for the
     `mnemonic` CLI from mnemonic-toolkit-v0.9.0.`) updates to
     `…from mnemonic-toolkit-v0.13.0.`.
   - `pub const SCHEMA: Schema { pinned_version: "mnemonic 0.9.0", … }`
     at line 791 → `"mnemonic 0.13.0"`. Ground truth verified:
     `git show mnemonic-toolkit-v0.13.0:crates/mnemonic-toolkit/Cargo.toml`
     → `version = "0.13.0"` (matches HEAD). `mnemonic --version`
     emits `mnemonic 0.13.0`.
   - 5 new `SubcommandSchema` entries appended to `SUBCOMMANDS`
     after the `derive-child` row (lines 770-777).
   - New `const` definitions above the schema as needed (e.g.,
     `SLIP39_FROM_NODES: &[&str] = &["phrase", "entropy"];`,
     `SLIP39_TO_SHAPES: &[&str] = &["entropy", "phrase"];`,
     `SEED_XOR_FROM_NODES: &[&str] = &["phrase"];`,
     `FINAL_WORD_FROM_NODES: &[&str] = &["phrase"];`).

3. **`tests/schema_mirror.rs::ci_workflow_snapshot`** (lines
   432-444): `required_tags` array first entry
   `"mnemonic-toolkit-v0.9.0"` → `"mnemonic-toolkit-v0.13.0"`.
   Comment at lines 430-431 updates the cycle reference.
   Stale doc-comment at line 579 (parenthetical "(mnemonic-toolkit-
   v0.9.0 / md-v0.5.0 / ms-v0.2.0 / mk-v0.3.0)") — refresh per
   amended-plan R1 N-1.

### §2.5 CI workflow + ancillary

- **`.github/workflows/schema-mirror.yml`:** install step for
  `mnemonic-toolkit` bumps to v0.13.0 (path-version + repo-tag
  references; the `tests/schema_mirror.rs::ci_workflow_snapshot`
  assertion at G2 enforces the string match).

- **Build.rs invariance:** `build.rs` reads upstream source via
  `MNEMONIC_GUI_UPSTREAM_ROOT` or `pinned-upstream.toml` checkout-
  root path; the generated `SECRET_NODE_TYPES` / `SECRET_SLOT_SUBKEYS`
  arrays should regenerate from v0.13.0 source without code change
  (the `syn` walk at `tests/schema_mirror.rs:153-193` is variant-
  set agnostic). G6 verifies.

### §2.6 Cross-refs + FOLLOWUPS at cycle close

At cycle close (PE):

- **GUI `FOLLOWUPS.md`:**
  - `slip39-gui-schema-flattening-companion` (currently
    `unblocked` at FOLLOWUPS.md line 79) → `resolved <commit>`
    with the v0.3 PE commit hash. The 4 sub-items + the implicit
    4 net-new surfaces are all addressed.
  - `mnemonic-gui-schema-mirror` (FOLLOWUPS.md lines 10-57): the
    pinned-tag table row for `mnemonic-toolkit` bumps from
    `mnemonic-toolkit-v0.9.0` to `mnemonic-toolkit-v0.13.0`.
    (Other rows for md/ms/mk unchanged.)

- **Toolkit `design/FOLLOWUPS.md`:**
  - `slip39-shamir-secret-sharing` (toolkit) is already
    `resolved` at toolkit `mnemonic-toolkit-v0.13.0` per
    `[[project-v0-13-0-slip39-closed]]`. No change.
  - `slip39-cli-extendable-flag` (toolkit) remains `open`,
    `v0.14-feature`. No change.

- **Companion-FOLLOWUP discipline (CLAUDE.md cross-repo invariant):**
  No NEW cross-repo FOLLOWUPS are created in v0.3. The GUI
  cycle closes a sibling-repo loop; no new siblings are touched.

### §2.7 Reviewer-loop convergence rails (Section 2 specifics)

- **R0 must source-truth** the conditional-fn shapes against
  `cmd/slip39.rs` clap attributes (line 101 conflicts_with;
  line 157 conflicts_with; line 131 + 170 doc-comment `ignored
  for ...`). Off-by-N narrative pattern is at maximum risk in the
  conditional-fn comments — verify line numbers exactly.
- **R0 must verify** the conditional-fn `cmd/slip39.rs` line
  citations are exact (line 101 + line 157 conflicts_with; line 131
  + line 170 doc-comments). Architect pre-verified the Cargo.toml
  version (`v0.13.0` tag and master both `version = "0.13.0"`) and
  the `FormState` API surface (struct at `src/schema/mod.rs:142`;
  `has_value` 205, `has_positional` 218, `dropdown_value` 226;
  `composite_node` confirmed non-existent — new helper).
- LOCK criterion: 0C/0I, same as Section 1.

### §2.8 R*n* review log (Section 2)

**R0 (opus, 2026-05-14):** **LOCK 0C/0I/0N/0n.**

All 10 verification surfaces (A–J) PASS clean on first round:

- **A** — cmd/slip39.rs:101/131/157/170 line citations byte-exact.
- **B** — schema/mnemonic.rs line 1 / 791 / 770-777 byte-exact.
- **C** — schema_mirror.rs lines 432-444 / 627-650 / 322-378 / 471-534
  ranges exact.
- **D** — FormState API pre-verification confirmed (struct 142,
  impl 164, has_value 205, has_positional 218, dropdown_value 226;
  `composite_node` absent — grep ZERO).
- **E** — Cargo.toml `version = "0.13.0"` at v0.13.0 tag confirmed.
- **F** — human_name parenthetical-summary style consistent; all 5
  new entries faithful. `allows_slots: false` correct (no `--slot`
  flag in any of cmd/slip39.rs, cmd/seed_xor.rs, cmd/final_word.rs).
- **G** — SECRET_* sets identical between v0.9.0 and v0.13.0
  (NodeType: 13 variants, is_secret_bearing: 7 variants; SlotSubkey:
  8 variants, is_secret_bearing: 4 variants). Build.rs regen needs
  no code change; §2.5 claim accurate.
- **H** — FOLLOWUPS.md line 79 + lines 10-57 exact.
- **I** — G1-G7 internal-consistency counts (6 conditional cells,
  5 kittest cells, 10 SubcommandSchema total) correct.
- **J** — BIP85_APPLICATIONS no-drift v0.9.0..v0.13.0 (identical
  9-element set on both ends).

**Section 2 LOCKED. Architect proceeds to Section 3 implementation
plan drafting.**

---

## Section 3 — Implementation plan

**Status:** DRAFT — pending R0 dispatch.

Mirrors prior cycles' P0 → P1 (RED) → P2 (GREEN) → P3 (LOCK) → PE
(release rollup) structure (cf. toolkit
`design/PLAN_v0_13_0_p2.md` / `PLAN_v0_13_0_p3.md` shape; GUI v0.2
agent-reports phase A.1..D.4 + Phase 10 release). All work occurs
in the **mnemonic-gui** working tree
(`/scratch/code/shibboleth/mnemonic-gui`); the toolkit working tree
is read-only during this cycle.

### §3.0 P0 — Pre-RED probe (read-only verification)

**P0 finding fold (2026-05-14, user-approved scope expansion;
amended-plan R1 fold of I-2):**
P0.8 baseline against the v0.13.0 binary surfaced **4-flag drift**
in existing subcommands that accumulated across v0.10–v0.13
toolkit cycles without companion `mnemonic-gui` PRs (mirror-
invariant breach per FOLLOWUPS.md:36-40). Drift folded into v0.3
scope with per-subcommand structural shape:

| Subcommand | Drift flag | Conditional-fn impact | Stale-comment cleanup |
|---|---|---|---|
| `bundle` | `--passphrase-stdin` (secret) | EXTEND existing `bundle` fn (`conditional.rs:20-35`) — add passphrase XOR passphrase-stdin clause; existing descriptor/template logic stays | — |
| `verify-bundle` | `--passphrase-stdin` (secret) | EXTEND existing `verify_bundle` fn (`conditional.rs:46-83`) — add passphrase XOR clause | — |
| `convert` | `--bip38-passphrase-stdin` (secret) | EXTEND existing `convert` fn (`conditional.rs:89-101`) — add SECOND XOR pair for `--bip38-passphrase` / `--bip38-passphrase-stdin` (additive; existing passphrase XOR stays) | — |
| `derive-child` | `--passphrase-stdin` (secret) | **NEW** `derive_child` fn; flip `schema/mnemonic.rs:773` `conditional: None` → `Some(crate::form::conditional::derive_child)` | DELETE stale comment at `conditional.rs:135-137` ("derive-child has no clap conflicts_with ... No conditional fn needed") — invalidated by v0.13.0 drift |

Clap-XOR ground truth (upstream `crates/mnemonic-toolkit/src/cmd/`,
verified by amended-plan R1 reviewer):

- `bundle.rs:51` — `conflicts_with = "passphrase"`
- `verify_bundle.rs:51` — `conflicts_with = "passphrase"`
- `convert.rs:203` — `conflicts_with = "bip38_passphrase"` (the bip38-stdin variant)
- `derive_child.rs:68` — `conflicts_with = "passphrase"`

All 4 are upstream-enforced XORs; GUI conditional fns mirror the
existing `convert` precedent at `conditional.rs:89-101` without
semantic translation.

**Sibling-CLI clean** (source-truth-verified by amended-plan R1):
`MNEMONIC_BIN=<v0.13.0> MD_BIN=<descriptor-mnemonic-md-cli-v0.5.0>
MS_BIN=<ms-cli-v0.2.1> MK_BIN=<mk-cli-v0.3.1> cargo test --test
schema_mirror` → md/ms/mk cells all PASS at master HEAD against the
pinned schema. Only mnemonic-toolkit drift to fix.

**Additional cleanup folded into P2.* (amended-plan R1 N-1):**

- `schema/mnemonic.rs:780-788` — R1 I-1 fold comment about
  `mnemonic 0.8.0` ≠ `mnemonic-toolkit-v0.8.1` is now stale (v0.13.0
  tag matches `version = "0.13.0"`). Refresh at P2.1.
- `tests/schema_mirror.rs:579` — parenthetical "(mnemonic-toolkit-
  v0.9.0 / md-v0.5.0 / ms-v0.2.0 / mk-v0.3.0)" is stale post-bump.
  Refresh at P2.3.

Goals: confirm Section 2 source-truth claims hold at cycle start,
install the new pinned binary locally, audit "dispatch vacuous"
claim end-to-end. Branch creation also happens here.

| # | Action | Pass criterion |
|---|--------|----------------|
| P0.1 | `git switch -c v0.3-feature` from `mnemonic-gui` HEAD `fd64e1b` | clean branch off the FOLLOWUP-filing commit |
| P0.2 | Install `mnemonic-toolkit-v0.13.0` binary locally (preferred form mirrors existing CI install at `.github/workflows/schema-mirror.yml`: `cargo install --git ... --tag mnemonic-toolkit-v0.13.0 mnemonic-toolkit` (positional package-name); OR symlink from a v0.13.0 checkout's `target/release/mnemonic`) | `mnemonic --version` → `mnemonic 0.13.0` |
| P0.3 | `MNEMONIC_BIN=$(which mnemonic) mnemonic gui-schema \| jq '.subcommands \| length'` | returns `10` (per `cli_gui_schema.rs:43-69` post-flattening) |
| P0.4 | Re-grep `src/**` + `tests/**` for `seed-xor`, `seed_xor`, `slip39`, `final-word`, `final_word` | ZERO hits (R1-confirmed in §1.2) |
| P0.5 | Confirm `composite_node` absence: `grep -rn 'composite_node' src/ tests/` | ZERO hits |
| P0.6 | Confirm BIP85_APPLICATIONS no-drift: `mnemonic derive-child --help` enumerates 9 applications matching `schema/mnemonic.rs:92-102` | match exact |
| P0.7 | Confirm SECRET_* regen no-drift: `MNEMONIC_GUI_UPSTREAM_ROOT=/scratch/code/shibboleth/mnemonic-toolkit cargo test --test schema_mirror source_audit` (3 cells) at the bumped pin | all 3 cells green |
| P0.8 | Baseline `cargo test -p mnemonic-gui --no-fail-fast` against unchanged code at v0.13.0 binary | schema_mirror's `mnemonic_schema_flag_names_match_help_text` PASSES (the existing 5 subcommands still have stable flag surfaces v0.9.0..v0.13.0) — verify at P0; if it FAILS, surface drift as a P0 finding |

Per R1 architect-note, P0.8 is also where the `cargo test --no-run`
parse-check rail lands (parse-failure = Important per
`[[feedback-r2-blocking-vs-cosmetic-gate]]`).

### §3.1 P1 — RED tests

Three sub-phases, each ending with a per-phase reviewer-loop until
that phase's RED tests cleanly fail in the expected way (i.e., FAIL
for the documented reason, not for unrelated reasons).

#### P1.1 — Schema-mirror RED (5 new SubcommandSchema entries, EMPTY flag arrays)

Drive `tests/schema_mirror.rs::mnemonic_schema_flag_names_match_help_text`
to fail on the 5 new subcommands.

- **Edit:** `src/schema/mnemonic.rs` — append 5 SubcommandSchema
  entries to SUBCOMMANDS (after `derive-child` at line 777), each
  with:
    - `name`: `"slip39-split"` / `"slip39-combine"` /
      `"seed-xor-split"` / `"seed-xor-combine"` / `"final-word"`.
    - `human_name`: per §2.2 table.
    - `flags`: `&[]` (EMPTY — RED-driver).
    - `positional_args`: `NO_POSITIONALS`.
    - `allows_slots`: `false`.
    - `conditional`: `None` (placeholder; replaced at P2).
- **No bump** to `pinned_version` or `pinned-upstream.toml` yet —
  that's P1.2's RED-driver.
- **Expected RED:** test fails 5×, each with
  `only in upstream --help: ["--from", "--passphrase", ...]`. The
  test still iterates all 10 entries; the existing 5 stay green.

**LOCK criterion for P1.1:** the 5 RED failures match the expected
flag-set diff. No spurious failures.

#### P1.2 — Pinned-upstream + workflow snapshot RED

Drive `tests/schema_mirror.rs::ci_workflow_snapshot` to RED.

- **Edit:** `tests/schema_mirror.rs:432-444` `required_tags` array
  → first entry `"mnemonic-toolkit-v0.13.0"` (was `"...v0.9.0"`).
- **Expected RED:** `ci_workflow_snapshot` fails because the
  workflow YAML at `.github/workflows/schema-mirror.yml` still
  pins `mnemonic-toolkit-v0.9.0` (the snapshot checks YAML body
  contains every required tag string).

**LOCK criterion for P1.2:** RED reproduces with the expected
"schema-mirror.yml must pin tag `mnemonic-toolkit-v0.13.0`" panic
message. No spurious failures.

#### P1.3 — Conditional-visibility RED (14 cells; amended-plan R1 I-3 fold)

Drive `tests/conditional_visibility.rs` to fail 14×.

- **Edit:** add 14 new test cells:
    - **slip39-split (3):**
        - `slip39_split_passphrase_disables_passphrase_stdin`
        - `slip39_split_passphrase_stdin_disables_passphrase`
        - `slip39_split_language_hidden_when_from_entropy`
    - **slip39-combine (3):**
        - `slip39_combine_passphrase_disables_passphrase_stdin`
        - `slip39_combine_passphrase_stdin_disables_passphrase`
        - `slip39_combine_language_hidden_when_to_entropy`
    - **bundle drift (2):**
        - `bundle_passphrase_disables_passphrase_stdin`
        - `bundle_passphrase_stdin_disables_passphrase`
    - **verify-bundle drift (2):**
        - `verify_bundle_passphrase_disables_passphrase_stdin`
        - `verify_bundle_passphrase_stdin_disables_passphrase`
    - **convert bip38 drift (2):**
        - `convert_bip38_passphrase_disables_bip38_passphrase_stdin`
        - `convert_bip38_passphrase_stdin_disables_bip38_passphrase`
    - **derive-child drift (2):**
        - `derive_child_passphrase_disables_passphrase_stdin`
        - `derive_child_passphrase_stdin_disables_passphrase`
- **Stub:** add `slip39_split`, `slip39_combine`, AND `derive_child`
  fns to `src/form/conditional.rs` with `Vec::new()` returns. The
  existing `bundle`, `verify_bundle`, `convert` fns are EXTENDED
  in P2.2 with the drift XOR clauses (no stub needed — they
  already exist and pass their pre-existing cells).
- **Expected RED:** all 14 cells fail at `assert!(visibilities
  .contains(&("--passphrase-stdin", Visibility::Disabled)))` style
  assertions, because the new fns return empty AND the extended
  fns don't yet carry the drift clause.

**LOCK criterion for P1.3:** 14 RED failures match expected
assertion fail; no compile errors.

#### P1.4 — Kittest RED (5 cells)

Drive `tests/widget_interaction.rs` (or new `tests/widget_slip39.rs`
— **decide at P1.4 R0**) to fail 5×.

**Cell-construction pattern (R0 I-1 fold):** mirrors v0.2 D.4 cells
4 + 5 at `tests/widget_interaction.rs:176-288`, NOT the
`MnemonicGuiApp` boot pattern. Each cell:

1. Builds `Harness::new_ui_state` with an inline probe closure
   carrying `set-<flag>` stub buttons that push the canned value
   into `FormState.values` on click. (`MnemonicGuiApp` is not
   reachable from integration tests — its main-mod state is
   private; see `tests/widget_secret.rs:18-24` documented
   limitation.)
2. The harness's `step()` clicks the probe buttons in sequence to
   populate the form state.
3. The cell then calls
   `mnemonic_gui::form::invocation::assemble_argv(&schema::mnemonic::SCHEMA, "slip39-split", harness.state())`
   directly (pure-logic call — no widget render needed for argv
   assembly).
4. Asserts on the returned `Vec<String>`: expected `--<flag>` tokens
   present, in the expected position.

`tests/argv_assembler.rs:20-46` is the **pure-logic** argv-assembly
test pattern (no Harness); P1.4 cells INSTEAD use the v0.2 D.4
Harness-with-probe-buttons pattern so the cell ALSO exercises the
widget-population path (the value of kittest coverage). Both
patterns end in a direct `assemble_argv` call on `state()`.

- **Expected RED:** because P1.1 entries have empty `flags: &[]`
  arrays, `assemble_argv` iterates the empty subcommand-flag list
  and emits only `["mnemonic", "<subcommand>"]` — missing every
  `--<flag>` token. Assertions fail with `expected to find
  "--from" in argv; got: ["mnemonic", "slip39-split"]`.

**LOCK criterion for P1.4:** 5 RED failures match expected token-
missing message; cells parse + run cleanly under
`cargo test --no-run -p mnemonic-gui` (per
`[[feedback-r2-blocking-vs-cosmetic-gate]]`, parse-failure on cells
is Important).

### §3.2 P2 — GREEN impl

Five sub-phases. Each ends with a phase-specific reviewer-loop.

**Line-number convention:** all "line N" references below are at the
**pre-P2.* edit state** of the file (= current `mnemonic-gui`
HEAD `fd64e1b`). The file grows as entries are added; downstream
edits within the same sub-phase use anchor-string matching, not
line numbers.

#### P2.1 — Schema fill (5 SubcommandSchema entries fully populated)

- **Edit:** `src/schema/mnemonic.rs` — fill the 5 SubcommandSchema
  `flags:` arrays per the §1.3.1–§1.3.5 tables. Add new local
  constants as needed:
    - `SLIP39_FROM_NODES: &[&str] = &["phrase", "entropy"];`
    - `SLIP39_TO_SHAPES: &[&str] = &["entropy", "phrase"];`
    - `SEED_XOR_PHRASE_ONLY: &[&str] = &["phrase"];` (re-used by
      final-word too).
- **Updates:** bump `pinned_version: "mnemonic 0.13.0"` at line 791
  and the top-of-file doc-comment at line 1.
- **Result:** P1.1 RED → GREEN. P1.4 partial (form now renders
  widgets; argv assembly still fails on conditional-visibility-
  dependent assertions, which P2.2 closes).

#### P2.2 — Conditional-visibility fn impls (amended-plan R1 I-2 fold)

Five distinct conditional-fn changes:

- **NEW `slip39_split`** (per §2.3 spec): full impl with passphrase
  XOR + `--language` Hidden-when-entropy. P2.1's `None`
  placeholder in slip39-split SubcommandSchema flips to `Some(...)`.
- **NEW `slip39_combine`** (per §2.3 spec): full impl with
  passphrase XOR + `--language` Hidden-when-entropy. Flip.
- **NEW `derive_child`** (drift fix): full impl with passphrase
  XOR passphrase-stdin. Mirrors `convert.rs:89-101` precedent. Flip
  `schema/mnemonic.rs:773` `conditional: None` → `Some(...)`.
  DELETE the stale "no conditional fn needed" comment at
  `conditional.rs:135-137`.
- **EXTEND existing `bundle`** (drift fix): add passphrase XOR
  passphrase-stdin clause to the existing fn. Descriptor/template
  logic stays. No schema-side flip needed (bundle's
  SubcommandSchema already has `conditional: Some(...)`).
- **EXTEND existing `verify_bundle`** (drift fix): add passphrase
  XOR passphrase-stdin clause. No flip needed.
- **EXTEND existing `convert`** (drift fix): ADD second XOR pair
  for `--bip38-passphrase` / `--bip38-passphrase-stdin`. The
  existing `--passphrase` XOR stays. Both XORs coexist; no
  replacement. No flip needed.

- **Edit:** `src/schema/mod.rs` — add `pub fn composite_node(&self,
  flag: &str) -> Option<&str>` to `FormState::impl` block (after
  `dropdown_value` at line 226). Mirrors the `dropdown_value`
  pattern but reads from the NodeValueComposite slot.
- **Edit:** `src/schema/mnemonic.rs:780-788` — refresh the R1 I-1
  fold comment that explains why `pinned_version = "mnemonic 0.8.0"`
  ≠ `mnemonic-toolkit-v0.8.1` tag. That rationale doesn't apply
  in v0.13.0 (tag matches crate version). Either delete the
  comment or update it to note the v0.13.0 lockstep.
- **Result:** P1.3 14 RED → GREEN. P1.4 conditionally-driven argv
  assertions now pass.

#### P2.3 — Pinned-upstream + workflow YAML + CI bump

- **Edit:** `pinned-upstream.toml` — `[mnemonic].tag` →
  `"mnemonic-toolkit-v0.13.0"`.
- **Edit:** `.github/workflows/schema-mirror.yml` — bump the
  install step + any tag-string interpolations. Must contain
  `mnemonic-toolkit-v0.13.0` somewhere the
  `ci_workflow_snapshot` substring-check sees.
- **Result:** P1.2 RED → GREEN.

#### P2.4 — Kittest fill (5 cells argv-assertions GREEN)

Same Harness-with-probe-buttons construction pattern as P1.4 (R0
I-1 fold). Each probe closure for P2.4 carries the full
button-set; `step()`-click sequence populates the form; final
`assemble_argv` call returns a Vec<String> with all the
expected tokens.

**Canned-value discipline (R1 architect-discretion note):** cells
inspect argv-assembly, not toolkit semantics — canned phrase /
share / partial values need not be cryptographically valid BIP-39
or SLIP-39 phrases. Placeholder strings like `"abandon abandon ..."`
(mirroring v0.2 D.4 cell 4) are sufficient.

- **Edit:** kittest cells — flesh out the canned-input flows and
  argv assertions. Specifically:
    - `slip39-split`: set `--from phrase=<valid 12w>`,
      `--group-threshold 1`, `--group 2,2` (one entry), assert
      argv contains all three.
    - `slip39-combine`: set `--share` ×2 with valid SLIP-39
      mnemonics from the toolkit's vectors fixture (or hard-coded
      test vectors), `--to entropy`, assert.
    - `seed-xor-split`: set `--from phrase=<12w>`, `--shares 2`,
      assert.
    - `seed-xor-combine`: set `--share` ×2, `--shares 2`, assert.
    - `final-word`: set `--from phrase=<11w>`, assert.
- **Result:** P1.4 5 RED → GREEN.

#### P2.5 — Full-cycle gate run

- Run `MNEMONIC_BIN=$(which mnemonic) cargo test -p mnemonic-gui`
  end-to-end. All tests green: G1 (schema-mirror), G2 (workflow
  snapshot), G3 (conditional-visibility 6 new + existing),
  G4 (kittest 5 new + existing), G5 (gui-schema-capable
  invariant), G6 (SECRET_* regen), G7 (build.yml version step).
- Run `cargo build --release` to confirm no rustc warnings beyond
  baseline.
- Optionally run the workflow locally via `act` for a smoke; not
  required if CI matrix path is the source of truth.

### §3.3 P3 — Cycle LOCK reviewer-loop

Whole-cycle reviewer pass.

- **R0..R*n*:** dispatch `feature-dev:code-reviewer` + `model: "opus"`
  with scope "all P2 changes since master fork-point." Reviewer:
    - Verifies G1–G7 all pass against the actual feature branch.
    - Checks for off-by-N drift in any introduced strings (help-
      text labels, conditional-fn doc-comment line refs, schema
      consts).
    - Verifies the FOLLOWUPS resolution-strings at PE are
      structured correctly (R2 architect-note from prior cycles:
      `resolved <commit>` must reference the PE commit, NOT P3).
- LOCK criterion: 0C/0I.

### §3.4 PE — Release rollup

Final commit + tag + release.

| # | Action | Notes |
|---|--------|-------|
| PE.1 | Bump `Cargo.toml [package].version` `0.2.0` → `0.3.0` | sibling bump for any internal members if any |
| PE.2 | Add CHANGELOG.md entry `## [0.3.0] - 2026-MM-DD` | mirror v0.2.0 entry shape; cover the 5 new surfaces + pin bump + drift fix (schema-mirror-invariant restoration: 4 missing flags across bundle/verify-bundle/convert/derive-child added post-v0.10..v0.13 — closes mirror-invariant breach per FOLLOWUPS.md:36-40) per amended-plan R1 n-2 |
| PE.3 | Update FOLLOWUPS.md: `slip39-gui-schema-flattening-companion` → `resolved <commit>` (the PE rollup commit hash) | per §2.6 |
| PE.4 | Update FOLLOWUPS.md: `mnemonic-gui-schema-mirror` pinned-tag table row for mnemonic-toolkit → `mnemonic-toolkit-v0.13.0` | per §2.6 |
| PE.5 | Open PR from `v0.3-feature` → `master` | enforce PR-CI matrix BEFORE tag-push per FOLLOWUPS "Process notes" |
| PE.6 | After PR merge: `git tag mnemonic-gui-v0.3.0 <merge-commit>` + push | tag the merge commit after PR-CI green per FOLLOWUPS.md "Process notes" §`v0.2: enforce PR-CI gate before tag-push` (lines 193-202) |
| PE.7 | `gh release create mnemonic-gui-v0.3.0` with the workflow-produced artifacts (built by `build.yml` matrix) | binaries for 5 matrix targets |
| PE.8 | Save a `.v0_3-shipped-handoff.md` scratch file at repo root | mirrors `.v0_13_0-shipped-handoff.md` shape; pre-flight checks + memory update reminder |

### §3.5 Reviewer-loop discipline (per phase)

- Each P1.* and P2.* sub-phase dispatches a `feature-dev:code-
  reviewer` + `model: "opus"` round until LOCK (0C/0I).
- R0 of each phase MUST verify source-truth (line numbers, function
  names, flag names) per `[[feedback-r0-must-read-source-off-by-n]]`.
- R0 of each phase MUST run the prose's commands end-to-end per
  `[[feedback-architect-must-run-prose-commands]]`: e.g., for P2.4
  the reviewer must actually execute `cargo test -p mnemonic-gui
  widget_slip39_split_argv_assembles -- --nocapture` and confirm
  GREEN, not just read the cell prose.
- Per-phase reviewer reports persist to
  `design/agent-reports/v0_3-phase-<phase>-r<n>.md` (mirrors v0.1
  / v0.2 cycle archive convention).

### §3.6 Cycle-execution outline (handoff between sessions)

This plan-file is for a single cycle that may span multiple
sessions. Per-session resumption:

- **Resume token:** read `/home/bcg/.claude/plans/eager-giggling-
  castle.md` (this file) + the latest `design/agent-reports/v0_3-
  phase-*-r*.md` in the GUI repo.
- **State snapshot:** at any point, the cycle's current phase is
  the lowest-numbered phase NOT yet LOCKed in §3.7 R*n* log.
- **Memory updates:** at PE close, write a new project-memory
  `project_v0_3_mnemonic_gui_closed.md` mirroring
  `[[project-v0-13-0-slip39-closed]]` shape.

### §3.7 R*n* review log (Section 3)

**R0 (opus, 2026-05-14):** ITERATE 0C/1I/0N/2n.

- **I-1 folded** (§3.1 P1.4 + §3.2 P2.4): kittest cell-construction
  prose corrected from "boot MnemonicGuiApp + select subcommand
  tab" to the v0.2 D.4 Harness-with-probe-buttons pattern at
  `tests/widget_interaction.rs:176-288`. Cells use inline probe
  closures with `set-<flag>` stub buttons that mutate
  `FormState.values` on click, then call `assemble_argv` directly.
  Expected-RED rationale unchanged (empty flags array → argv emits
  only `["mnemonic", "<subcommand>"]`).
- **n-1 folded** (§3.4 PE.6): citation softened to "tag the merge
  commit after PR-CI green per FOLLOWUPS.md 'Process notes'
  §`v0.2: enforce PR-CI gate before tag-push` (lines 193-202)" —
  no longer claims merge-commit-vs-HEAD specificity beyond what
  the Process notes document.
- **n-2 folded** (§3.0 P0.2): install-recipe form now mirrors the
  existing CI install at `.github/workflows/schema-mirror.yml`
  (positional package-name `mnemonic-toolkit`); `--bin mnemonic`
  is still mentioned as an alternative.

12 of 13 verification-matrix items (A–F, H–M) PASS clean on R0;
G FAILED only on the cell-construction prose, not on the
underlying RED-driver expectation.

**R1 (opus, 2026-05-14):** **LOCK 0C/0I/0N/0n.**

All 3 R0 folds verified PASS:

- **I-1 PASS** — §3.1 P1.4 4-step cell pattern matches v0.2 D.4
  cells 4+5 at `tests/widget_interaction.rs:176-288` byte-faithfully.
  `Harness::new_ui_state` syntax confirmed at lines 185-204 (cell 4)
  and 237-259 (cell 5). `assemble_argv` location confirmed at
  `src/form/invocation.rs:42`. §3.2 P2.4 fold-back-reference
  faithful.
- **n-1 PASS** — §3.4 PE.6 citation lines 193-202 verified against
  `FOLLOWUPS.md`: line 193 is `### v0.2: enforce PR-CI gate before
  tag-push` heading; line 202 ends the entry; range exact.
- **n-2 PASS** — §3.0 P0.2 install-recipe form mirrors
  `.github/workflows/schema-mirror.yml:23-28` (positional
  package-name `mnemonic-toolkit`).

Regression check **PASS**: phase numbering preserved (P0.1-P0.8,
P1.1-P1.4, P2.1-P2.5, P3, PE.1-PE.8); cross-citations to §1.3 /
§2.1 / §2.4 ground truth intact; gate references G1-G7 consistent.

**R1 architect-discretion note folded** (§3.2 P2.4): canned-value
discipline clarified — cells inspect argv-assembly, not toolkit
semantics; placeholder strings (e.g., `"abandon abandon ..."`)
suffice.

**Section 3 LOCKED. All three sections at LOCK. Architect calls
ExitPlanMode.**
