# GUI-form-renders — Leg-1 P3 R0 review — round 4 (scoped: I1-residual doc-comment fold)

**Scope:** Narrowest-possible round-4 of the per-phase R0 (gate: 0 Critical / 0 Important)
of Leg-1 P3, confirming the round-3 Important (I1-residual — the `seeded_fixture`
doc-comment at `render_emit.rs:91-95` still asserting "**Secret** flags are NEVER seeded" +
"**Required REPEATING** *non-secret* flags seed ONE row," both made FALSE by the r2
behavioral fold that seeds the secret Composite `seed-xor-combine --share`) is now corrected
with no drift. Round-3 already verified ALL behavior converged (predicate exact, byte-inert
emit, gates 622/0/4, both clippy, headless==default, hygiene, faithfulness teeth); the ONLY
open tail was the doc-comment. Branch `feat/gui-render-form-emit`, doc-fold commit `2c5ffb1`
(parent / behavioral fold `1d82155`); master untouched @ `01520a5`.
Round-3 review: `design/agent-reports/gui-form-renders-p3-r0-round-3.md`.

**Reviewer:** opus architect, adversarial. Doc-comment re-read against the live code +
inline comment; diff confirmed doc-only by line-class filter; `gui-render` bin rebuilt to
confirm compilation integrity. Branch left clean.

---

## VERDICT: GREEN — 0 Critical / 0 Important

The round-3 Important is fully resolved. The doc-comment now accurately describes the code,
agrees with the corrected inline comment, and the false universally-quantified "Secret flags
are NEVER seeded" claim is gone. The fold is doc-comment-only (every changed line is a `///`
line), so it provably cannot alter runtime behavior — round-3's full behavioral verification
(622/0/4, byte-identical `--emit-all`, both clippy clean, hygiene, faithfulness teeth) carries
forward unchanged, and the incremental build confirms the edit did not break compilation.
**P3 is now fully converged — ready for the leg post-impl.**

---

## Verification (all CONFIRMED)

### 1. The doc-comment is now ACCURATE vs the code + inline comment

Post-fold `render_emit.rs:91-98`:

```rust
///   - **Required REPEATING** flags seed ONE row (the GUI's per-frame
///     required-row seed) — including a secret repeating flag of a non-Text/Boolean
///     kind (e.g. the secret Composite `seed-xor-combine --share`, which the widget
///     dispatch routes to `render_repeating`); **optional repeating** flags seed NOTHING.
///   - **Secret Text/Boolean** flags are NEVER seeded (secret Text → `secret_widgets`,
///     secret `*-stdin` Boolean → early return) — but ONLY those two kinds, per the
///     widget dispatch; **mode-suppressed** + conditional-`Hidden` flags are not
///     rendered, hence not seeded.
```

Checked claim-by-claim against the code body (`render_emit.rs:114-152`):

- **"Secret Text/Boolean flags are NEVER seeded … but ONLY those two kinds"** — matches the
  skip predicate exactly: `if flag_is_secret(flag) && matches!(flag.kind, FlagKind::Text |
  FlagKind::Boolean) { continue; }` (`:130`). The qualifier is now narrowed to Text/Boolean,
  so the former universal-negative is gone. ✓
- **"Required REPEATING flags seed ONE row … including a secret repeating flag of a
  non-Text/Boolean kind (e.g. seed-xor-combine --share … routed to render_repeating); optional
  repeating flags seed NOTHING"** — matches the repeating branch (`:137-145`): `if
  flag.repeating { if flag.required { …push(default_flag_value_for_flag(flag)); } continue; }`.
  The secret Composite `--share` is NOT caught by the Text/Boolean skip at `:130`, so it falls
  through to this branch and (being required) seeds one row; optional-repeating pushes nothing.
  The dropped `non-secret` qualifier is the exact correction round-3 prescribed. ✓
- **"mode-suppressed + conditional-`Hidden` flags are not rendered, hence not seeded"** —
  matches the two `continue`s at `:117-122`. ✓
- The doc-comment now agrees with the corrected **inline** comment (`:123-129`, "a secret flag
  of any other kind … falls through to `render_repeating` and IS seeded one row") — no
  remaining doc-vs-inline contradiction. ✓
- The monotone-fixed-point tail (`:100-101`, "`state.values` only GROWS … converges in ≤
  `sub.flags.len()` passes") is unchanged and still accurate (the loop only ever pushes; the
  `changed` flag drives termination at `:153-155`). ✓

The intro sentence (`:75`, "auto-seeds every CURRENTLY-RENDERED non-secret flag's schema
default") was deliberately re-examined: it is a positive description of the dominant mechanism
for non-secret flags, not a universal-negative about secrets (it never claims secret flags are
never seeded), the bullets below refine it precisely, it is unchanged by this fold, and
round-2/round-3's full behavioral passes did not flag it. Not a defect; not in scope.

### 2. Diff is doc-only — no behavioral drift possible

`git diff 1d82155..2c5ffb1` touches one file (`src/form/render_emit.rs`, +8/-5). Filtering the
changed lines to anything that is NOT a `///` doc-comment line returns EMPTY — every added and
removed line is a doc-comment line inside `seeded_fixture`'s `///` block. A rustdoc comment is
not compiled into behavior, so:
- `--emit-all` is necessarily byte-identical to `1d82155` (round-3 already proved `1d82155`
  byte-identical to pre-fold `6fbf79f` across all 61 forms) — no snapshot re-pin.
- The full suite (round-3: **622 / 0 / 4**), both clippy lanes (default + `--no-default-features`,
  both exit 0), and headless==default emit are all unaffected by a comment-only edit.
- Compilation integrity confirmed live: `cargo build --bin gui-render` → `Finished` (clean
  incremental, no warnings) — a malformed `///` block is the only way a doc edit could break,
  and it did not. ✓

### 3. No new Critical / Important

- **Critical:** none.
- **Important:** none. The sole round-3 blocker (I1-residual) is resolved; no fresh finding.

### 4. Branch clean

`git status --porcelain` shows only the pre-existing untracked round-2/round-3 reports (and
this round-4 report). No tracked-file modifications, no stray worktree. ✓

---

## Bottom line

The doc-only fix is exactly what round-3 prescribed: the false "Secret flags are NEVER seeded"
universal-negative is replaced by the accurate Text/Boolean split, the `non-secret` qualifier
is dropped from the repeating bullet, and the doc-comment now agrees with both the inline
comment and the code 30 lines below. The diff is provably doc-comment-only (zero non-`///`
changed lines), so round-3's behavioral verification stands unchanged and the bin still
compiles clean. The strict 0C/0I gate is met. **GREEN — 0C / 0I. P3 is fully converged and
ready for the Leg-1 post-impl review.**
