# GUI-form-renders — Leg-1 P3 R0 review — round 3 (scoped: I1 fold convergence)

**Scope:** Tight scoped round-3 of the per-phase R0 (gate: 0 Critical / 0 Important) of
Leg-1 P3, confirming the round-2 Important (I1 — `seeded_fixture`'s blanket
`flag_is_secret → skip` under-seeded the lone secret-Composite-repeating flag
`seed-xor-combine --share`) was folded with no drift. Round-2 already verified everything
else GREEN; only the I1 predicate-tighten is new. Branch `feat/gui-render-form-emit`,
I1-fold commit `1d82155` (parent / pre-fold `6fbf79f`); master untouched @ `01520a5`.
Round-2 review: `design/agent-reports/gui-form-renders-p3-r0-round-2.md`.

**Reviewer:** opus architect, adversarial. Every claim verified against source + live
`gui-render` runs (pre-fold `6fbf79f` built in a throwaway worktree vs fold `1d82155`) +
the full `-p` suite + both clippy + headless==default. Branch left clean; worktree removed.

---

## VERDICT: RED — 0 Critical / 1 Important

The **behavioral** half of I1 is fully converged and verified: the predicate was tightened
to EXACTLY round-2's prescription, the secret Composite `seed-xor-combine --share` now seeds
one row (mirroring `render_repeating`), the fold is provably byte-inert (pre-fold vs fold
`--emit-all` byte-identical across all 61), every release gate is green (622/0/4, both clippy,
headless==default byte-identical), and secret hygiene holds (`--share` still renders
`<masked>`). **But the fold updated only the INLINE comment (`render_emit.rs:120-128`) and
left the function's DOC-comment (`render_emit.rs:91-95`) asserting "**Secret** flags are NEVER
seeded" and "**Required REPEATING** *non-secret* flags seed ONE row" — both now FALSE**, since
the secret Composite `--share` IS seeded. This is the *same literally-false invariant claim*
that round-2 named as I1 point (c) ("a future maintainer would trust 'no secret flag is ever
in `state.values`'"), now relocated to — and made strictly more wrong in — the function's
primary contract-comment. Round-2's required-work predicted "the doc-comment … become[s]
accurate without further edits" — that prediction is **demonstrably wrong** (tightening moved
the doc-comment from accurate-to-buggy-code → inaccurate-to-correct-code). I1's required-work
is therefore **incompletely satisfied**. Per the strict gate, an unconverged remnant of an
Important — a load-bearing helper's doc-comment contradicting its own code on a
secret-classification property — blocks GREEN. The fix is a 2-line doc-comment edit, no
code/snapshot/test change. **RED — 0C / 1I.**

---

## Verification of the fold (all CONFIRMED)

### 1. The predicate matches round-2's prescription EXACTLY

`git diff 6fbf79f 1d82155` touches one file (`render_emit.rs`, +8/-4). The condition is now:

```rust
if flag_is_secret(flag) && matches!(flag.kind, FlagKind::Text | FlagKind::Boolean) {
    continue;
}
```

— byte-for-byte round-2's prescribed predicate. ✓

### 2. Source-verified flag kinds — the mirror is now exact

I confirmed against `src/schema/mnemonic.rs` (not the round-2 draft) the kind of every
`--share`:

| Flag | Line | `kind` | `secret` | repeating/required |
|---|---|---|---|---|
| `slip39-combine --share` | 1780 | `FlagKind::Text` | true | true / true |
| `ms-shares-combine --share` | 1970 | `FlagKind::Text` | true | true / true |
| `seed-xor-combine --share` | 2075 | `FlagKind::NodeValueComposite(PHRASE_ONLY)` | true | true / true |

`FlagKind` (`schema/mod.rs:142`) has 9 variants; `SECRET_FLAG_NAMES`
(`secrets.rs:142`) adds only `--passphrase`/`--bip38-passphrase` (Text) +
`--passphrase-stdin` (Boolean). So `seed-xor-combine --share` is the unique secret
non-Text/non-Boolean flag, exactly as round-2 enumerated.

Tracing `render_with_dispatch` against source: the secret-Text branch
(`widget.rs:95`, `matches!(kind, FlagKind::Text)`) routes the two **Text** `--share`
flags to `secret_widgets` (NOT `state.values`); the secret-Boolean branch
(`widget.rs:181`) handles `*-stdin`. The **Composite** `seed-xor-combine --share`
matches NEITHER → falls through to `if flag.repeating` (`widget.rs:202`) →
`render_repeating`, whose required-row seed (`widget.rs:150-152` / doc cites
`:309-315`) pushes ONE row into `state.values`. The fold's tightened predicate
reproduces this exactly: secret Text/Boolean → `continue` (not seeded); secret
Composite → falls through → seeds one required-repeating row. The `widget.rs:125`
comment naming only `slip39-combine`/`ms-shares-combine` in the secret-Text branch is
consistent (those are the Text ones) — no contradiction. ✓

### 3. Inert — byte-identical emit, no visibility change

Built `gui-render` at `6fbf79f` (pre-fold, throwaway worktree) and `1d82155` (fold);
`diff -rq` of `--emit-all` (61 forms each) → **byte-identical** (exit 0). `seed-xor-combine`
has `conditional: None` (`schema/mnemonic.rs:4445`), so nothing reads `--share`'s presence and
seeding one row cannot move any `[disabled]`/`(required)` column. The fold changes only
`--share`'s `state.values` seeding, which is invisible at the emit surface. ✓

### 4. Hygiene intact

`mnemonic-seed-xor-combine.gui` renders:
`--share  composite[phrase]  (required, secret, repeating) -> <masked>`. The row is now
seeded but the value column is the `<masked>` sentinel — seeding state ≠ displaying a secret.
Fixtures remain FAKE (`render_fixture`-derived); no secret material is emitted. ✓

### 5. Gates (all green)

| Gate | Command | Result |
|---|---|---|
| Full suite | `cargo test -p mnemonic-gui --jobs 2` | **622 / 0 / 4** ✓ (71 result lines summed) |
| Clippy default | `clippy --all-targets -- -D warnings` | exit **0** ✓ |
| Clippy headless | `clippy --no-default-features -- -D warnings` | exit **0** ✓ |
| Headless build | `build --bin gui-render --no-default-features` | `Finished` ✓ |
| headless==default emit | `--emit-all` (both binaries) `diff -rq` | byte-identical (61) ✓ |
| pre-fold==fold emit | `6fbf79f` vs `1d82155` `--emit-all` `diff -rq` | byte-identical (61) ✓ |
| Branch clean | tracked changes | none (only pre-existing untracked r2 report) ✓ |

### 6. Disabled re-gate still has teeth

The faithfulness gate is unchanged by the fold (the fold touches `render_emit.rs` only, not
`gui_render_faithfulness.rs`); round-2 reproduced its 6-divergence teeth and they remain
intact (full `gui_render_faithfulness` green in the 622/0/4 run). The fold introduces no new
divergence on any gated axis. ✓

---

## Important

### I1-residual — the fold left `seeded_fixture`'s DOC-comment (`render_emit.rs:91-95`) asserting the now-FALSE invariant the fix was supposed to eliminate.

The fold corrected the **inline** comment at the code site (`render_emit.rs:120-128` — now
accurately describes the secret-Composite seed), but left the function's **doc-comment**
(`render_emit.rs:83-95`) untouched. Two of its bullets are now false / contradictory with the
code 30 lines below:

```rust
///   - **Required REPEATING** non-secret flags seed ONE row (the GUI's
///     per-frame required-row seed); **optional repeating** flags seed NOTHING.
///   - **Secret** flags are NEVER seeded (secret Text → `secret_widgets`, secret
///     `*-stdin` Boolean → early return); ...
```

Post-fold, `seed-xor-combine --share` (secret, Composite, required, repeating) IS pushed into
`state.values` (one row). So:

- **Line 93 "Secret flags are NEVER seeded" is FALSE.** The universally-quantified claim
  directly contradicts the code and the corrected inline comment (`:120-128`, which explicitly
  says the secret Composite "falls through to `render_repeating` and IS seeded one row"). The
  parenthetical enumerates only Text → `secret_widgets` and Boolean → early-return; it OMITS
  the Composite case entirely.
- **Line 91 "Required REPEATING *non-secret* flags seed ONE row" is INCOMPLETE** — the
  `non-secret` qualifier now excludes the secret Composite, which also seeds one row.

**Why this is the unconverged tail of I1, not a fresh nit.** Round-2 rated I1 Important on
four grounds; ground (c) was verbatim: *"the helper ships a literally-false 'exactly / does
NOT over-seed' invariant claim that a future maintainer would trust ('no secret flag is ever in
`state.values`')."* Round-2's required-work asserted the fix would make *"the doc-comment …
accurate without further edits"* and only *"optionally"* suggested a note. That assertion is
provably wrong: pre-fold the blanket `flag_is_secret → continue` made "Secret flags are NEVER
seeded" a TRUE description of the (buggy) code; post-fold the code is correct but the
doc-comment is now FALSE. The fold faithfully executed round-2's incomplete prescription (edited
only the inline comment shown in round-2's diff) — so the literally-false invariant claim that
was a named pillar of I1 *survives in the function's primary contract-comment*. I1's
required-work is therefore not fully satisfied.

**Severity rationale — why Important, not Minor.** (a) It is an unconverged remnant of the
Important under review, not a new unrelated finding — the strict gate requires I1 to converge,
and "the doc becomes accurate" was part of the required outcome. (b) The doc-comment is the
function's contract; a maintainer or hygiene-auditor reading "Secret flags are NEVER seeded"
(without scrolling to the inline comment / via rustdoc) is misled into believing no
secret-classified flag's row ever reaches `state.values` — which is false for `--share`. In a
project whose CLAUDE.md makes secret-memory-hygiene a first-class bar and explicitly warns
"folds themselves can introduce drift," a seeding helper whose doc falsely disclaims any secret
in `state.values` is a real (if today benign-valued) audit-trust hazard, not cosmetic. (c) The
fix is a 2-line doc-comment edit, zero code/snapshot/test change, suite stays 622/0/4 — there
is no cost reason to carry it past the gate.

**Required work (trivial):** update `render_emit.rs:91-95` so the doc-comment matches the code
+ the inline comment, e.g. drop the `non-secret` qualifier on the repeating bullet and replace
"Secret flags are NEVER seeded" with the accurate split — secret **Text/Boolean** are not
seeded (→ `secret_widgets` / early-return), while a secret **Composite repeating** flag
(`seed-xor-combine --share`) seeds one row via `render_repeating` exactly like the GUI. Proven
byte-identical emit, so NO snapshot re-pin. Re-run the full `-p` suite + both clippy, then
re-dispatch this scoped review to convergence.

---

## New Critical / Important

- **Critical:** none.
- **Important:** I1-residual (above) — the only blocker.

---

## Bottom line

The behavioral fix is exactly right, source-verified, and provably byte-inert; every release
gate is green and the disabled re-gate keeps its teeth. The one open item is the doc-comment:
the fold corrected the inline comment but left the function's primary doc-comment
(`render_emit.rs:91-95`) asserting "Secret flags are NEVER seeded" / "Required REPEATING
non-secret flags seed ONE row," both of which the very fix made false — the same false invariant
claim round-2 cited as part of I1, which round-2 wrongly predicted would self-correct. Under the
strict 0C/0I gate that is an unconverged tail of I1. **RED — 0C / 1I.** Fold the 2-line
doc-comment correction → re-run suite + clippy → re-dispatch to convergence.
