# GUI-form-renders — Leg-1 P3 R0 review — round 2

**Scope:** Round-2 of the per-phase R0 (gate: 0 Critical / 0 Important) of Leg-1 P3, after
the round-1 Important (ruling A — seed flag defaults before `conditional()`) was folded.
Confirm convergence + hunt for fold-introduced drift. Branch `feat/gui-render-form-emit` @
`6fbf79f` (fold `2b38399` + docs `6fbf79f`); master untouched @ `01520a5`. Round-1 review:
`design/agent-reports/gui-form-renders-p3-r0-round-1.md`.

**Reviewer:** opus architect, adversarial. Every claim verified against source + live
`gui-render` runs (pre-fold `99dc48a` vs post-fold built in a throwaway worktree) + a teeth
re-run + a proposed-fix experiment + the full `-p` suite. Branch left clean; worktree removed.

---

## VERDICT: RED — 0 Critical / 1 Important

The fold is *almost* converged: the seeded fixed point is the right architecture, the
disabled re-gate has genuine teeth (reproduced exactly — 6 divergences on exactly the 6
flags), the 3-form/6-flag delta is GUI-faithful + complete, all release gates are green
(622/0/4, both clippy clean, headless==default byte-identical, deterministic), the deleted
"construction frame" comment is correctly replaced, and secret hygiene holds. **But the
load-bearing `seeded_fixture` does NOT mirror the GUI's auto-seed EXACTLY** — it under-seeds
one flag (`seed-xor-combine --share`, the lone secret-Composite-repeating flag) that the real
GUI *does* seed into `state.values`. This is the exact "vice-versa seed divergence" the round-2
charter pre-classifies as a NEW latent divergence, and the helper's own doc-comment / commit
message assert "mirrors … exactly; does NOT over-seed" — literally false (it under-seeds). It
is provably **inert today** (byte-identical emit across all 61, gates green — both proven),
and the re-gate is self-guarding (any future load-bearing case REDs loudly, never silent), so
the fix is a one-line condition tighten with NO re-pin. Per the strict gate ("currently inert"
is the rationalization the standard overrides), an open exact-mirror divergence in the
load-bearing fold blocks GREEN. **RED — 0C / 1I.**

---

## Important

### I1 — `seeded_fixture`'s blanket `flag_is_secret → skip` UNDER-seeds `seed-xor-combine --share` (the only secret-Composite-repeating flag), diverging from the real GUI auto-seed it claims to mirror "exactly."

**Source (`render_emit.rs:120-125`):**

```rust
// Secret flags (incl. secret REPEATING) never reach `state.values`
// — the secret check precedes the repeating check in the widget
// dispatch (`widget.rs:181`/`:202`).
if flag_is_secret(flag) {
    continue;
}
```

**Why it is wrong.** The GUI's `render_with_dispatch` only routes a secret flag *away* from
`state.values` for two kinds: secret **Text** (→ `secret_widgets`, `widget.rs:95-168`) and
secret **Boolean** `*-stdin` (→ early return, `widget.rs:181-194`). A secret flag of ANY
OTHER kind falls through to `if flag.repeating { render_repeating(...) }` (`widget.rs:202`) or
the non-secret scalar push (`widget.rs:220-229`) — and IS seeded into `state.values`.

I enumerated every secret flag across all 61 forms (`flag_is_secret` over `all_forms()`):
**64 secret flags; exactly ONE is neither Text nor Boolean** —
`Mnemonic/seed-xor-combine/--share`, a secret `NodeValueComposite`, `repeating=true`,
`required=true`. In the real GUI it routes to `render_repeating`, whose required-row seed
(`widget.rs:309-315`: `if flag.required && !has_rows → push(default_flag_value_for_flag)`)
pushes ONE `--share` row into `state.values` on the first frame. `seeded_fixture`'s blanket
`flag_is_secret → continue` pushes **nothing** → it under-seeds `--share` relative to the GUI.

This is a **vice-versa** of the over-seed trap the round-2 charter names, and it is the same
class the round-1 model itself missed: round-1 over-generalized "Secret flags are NEVER seeded
into `state.values` (Text → `secret_widgets`; `*-stdin` Boolean → early return)" — that
enumeration omitted secret Composite, and the implementer faithfully encoded the (incomplete)
model. The fold's doc-comment (`render_emit.rs:88-101` "deliberately does NOT over-seed … Secret
flags are NEVER seeded"), the commit message ("Mirrors the GUI's per-frame auto-seed exactly;
does NOT over-seed"), and the new test comment (`gui_render_faithfulness.rs:234` "seeds every
rendered non-secret flag's default") all assert an EXACT mirror that the code does not deliver.

**Severity rationale — why Important, not Minor.** I verified the impact empirically:

- `seed-xor-combine` has `conditional: None` (`schema/mnemonic.rs:4450`) — nothing reads
  `--share`'s presence — and `--share`'s value column is the `MASKED` sentinel regardless of
  seeding. So the under-seed is **inert today**: I applied the precise fix
  (`flag_is_secret && matches!(flag.kind, FlagKind::Text | FlagKind::Boolean)`), re-emitted all
  61 forms, and they are **byte-identical** to the shipped emit; `gui_render_faithfulness` +
  `gui_render_emit` stay green. (Reverted; tree clean.)
- The disabled/presence re-gate IS self-guarding: if `--share` (or any future secret
  non-Text/non-Boolean flag) ever feeds a conditional, the emit's under-seeded `conditional`
  outcome diverges from the settled GUI and the axis REDs — loud, never silent.

So the SHIPPED artifact is correct and the gate is sound. I nonetheless rate it **Important**
because (a) the round-2 charter explicitly pre-classifies "any kind it seeds that the GUI
doesn't (or vice-versa)" as a NEW latent divergence to block on; (b) `seeded_fixture` is THE
load-bearing fold artifact and ruling A's whole mandate was an EXACT mirror — `--share` is in
the required-repeating class round-1 itself flagged as load-bearing (`convert --to`); (c) the
helper ships a literally-false "exactly / does NOT over-seed" invariant claim that a future
maintainer would trust ("no secret flag is ever in `state.values`"); and (d) the fix is a
one-line, provably byte-inert condition tighten — there is no cost reason to defer it past the
gate.

**Required work (trivial):** replace the blanket skip with the precise GUI predicate —

```rust
if flag_is_secret(flag) && matches!(flag.kind, FlagKind::Text | FlagKind::Boolean) {
    continue;
}
```

— which makes the mirror exact and the "exactly / does NOT over-seed" claims TRUE (so the
doc-comment / test-comment become accurate without further edits; optionally add a one-line
note that secret-Composite-repeating routes through `render_repeating` like the GUI). Proven
byte-identical, so **NO snapshot re-pin** and the suite stays 622/0/4. Re-run the full `-p`
suite + both clippy after the fold, then re-dispatch this review to convergence.

---

## Everything else — VERIFIED GREEN

### 1. `seeded_fixture` — render-gated monotone fixed point (correct except I1)

- **Algorithm matches the prescribed model:** loop {`vis_map = conditional(state)`; for each
  flag skip if `is_render_suppressed` → skip if `Hidden` → skip if secret → skip if `already`
  present → `repeating`: required ⇒ one row, optional ⇒ nothing → else seed scalar default;
  break when no flag was added}. Numeric defaults seed `Unset` (`has_value` reads absent — no
  fabricated number). ✓
- **Secret-before-repeating order matches `widget.rs:181`/`:202`:** the secret check
  (`render_emit.rs:123`) precedes the repeating check (`:130`), so a secret repeating flag is
  caught by the secret arm, not the repeating arm — mirroring the GUI dispatch order. ✓ (The
  ONLY defect is that the secret arm is too broad — I1.)
- **Disabled flags ARE seeded (correctly):** the GUI greys a `Visibility::Disabled` flag but
  still calls its widget → seeds it; `seeded_fixture` skips only `Hidden`/suppressed/secret, so
  it seeds `Disabled` flags too. This subtle point is RIGHT.
- **Convergence:** `state.values` only grows; the `already` guard means each non-secret rendered
  flag pushes at most once → bounded by `sub.flags.len()`, monotone, terminates. The GUI's
  `state.values` is likewise monotone (a later-Hidden flag keeps its stored value), so the fixed
  points coincide. ✓
- **Mode suppression handled:** `is_render_suppressed` covers `--slot` + build-descriptor
  tree/archetype suppression, so mode-suppressed flags are not seeded — matching the GUI's
  `continue` before the widget. ✓

### 2. The re-gate is NON-TAUTOLOGICAL + has teeth

- **Base is the BLANK fixture, not the seed:** `wf = render_extended_form_harness(tab, sub,
  render_fixture(tab, sub.name))` (`gui_render_faithfulness.rs:230`) — the harness is fed the
  pristine `FormState::default()` and SETTLES through the GUI's OWN auto-seed (egui_kittest's
  `Harness` constructor `run_ok()`). The emit side seeds via `seeded_fixture`. So the disabled
  axis compares **emit-seed-sim vs GUI-actual-settle**, NOT two copies of one seed. ✓
- **Teeth reproduced EXACTLY:** I reverted `project_form`'s state to the unseeded
  `render_fixture` and ran the gate → **6 divergences, on exactly the 6 flags**, every one
  `real disabled=true, emit disabled=false`:
  `bundle/{--multisig-path-family,--threshold}`, `verify-bundle/--threshold`,
  `export-wallet/{--descriptor,--threshold,--multisig-path-family}`. This proves the gate reads
  the real settled `is_disabled()` off the AccessKit label node independently, AND that the
  3-form/6-flag delta is complete (no other form diverges). Reverted; tree clean.

### 3. The 3-form/6-flag delta is GUI-faithful + complete

- **Exactly 3 forms changed pre-fold (`99dc48a`) → post-fold (`2b38399`)**, 58 byte-identical
  (`diff -rq` of `--emit-all`): `bundle`, `verify-bundle`, `export-wallet`. The diffs:
  - `bundle`: `--multisig-path-family`, `--threshold` gain `[disabled]`.
  - `verify-bundle`: `--threshold` gains `[disabled]`.
  - `export-wallet`: `--descriptor`, `--threshold`, `--multisig-path-family` gain `[disabled]`;
    `--template` and `--descriptor` DROP `(required)`.
- **GUI-faithful, verified against the conditional body** (`conditional.rs:589-624`
  `export_wallet`): seeded `--template = bip44` ⇒ `has_template = true`,
  `template_is_single_sig = true`, `--descriptor` empty ⇒ `has_descriptor = false`. So
  `has_template` ⇒ `--descriptor` Disabled; `!has_descriptor && !has_template` is FALSE ⇒ the
  Required arm does NOT fire ⇒ `(required)` correctly drops; single-sig ⇒ `--threshold` +
  `--multisig-path-family` Disabled. This is exactly the real GUI's settled on-load vis_map.
- **`(required)` change is correct + m4-consistent:** the dropped markers are conditional-sourced
  (`Visibility::Required`), not `flag.required`; the real GUI (template seeded) does not paint
  the red `*`, so dropping it is faithful. P3 does NOT gate `(required)` (m4 carve-out — not
  AccessKit-recoverable); the ASCII marker is regenerated from the SAME seeded conditional that
  drives the gated disabled axis, so it stays internally consistent with the carve-out.
- **Committed export-wallet snapshot is byte-identical** to the live `--emit-all` output
  (checked programmatically). The `slot_form_renders_slot_editor_placeholder` re-pins on
  `bundle` add the two newly-disabled lines.
- **No form that should have changed but didn't:** the teeth run REDs on exactly 6 flags and
  the re-gate covers all 61, so every form whose settled GUI disables a flag is accounted for
  (e.g. `convert`'s seeded required-repeating `--to` produces no disabled delta → correctly
  unchanged).

### 4. No new divergence on any gated axis across 61

Full `gui_render_faithfulness` is green: presence + control-class + secret-masking + positional
+ action-bar + the new disabled axis, all read off the real AccessKit tree. Anti-tautology
intact: the round-1 103-divergence control-class teeth plus the round-2 6-divergence disabled
teeth both fire when the respective side is mutated. The only latent (inert) divergence is the
I1 `--share` under-seed, which manifests on no gated axis today.

### 5. Gates

| Gate | Command | Result |
|---|---|---|
| Full suite | `cargo test -p mnemonic-gui --jobs 2` | **622 / 0 / 4** ✓ (matches claim) |
| Clippy default | `clippy --all-targets -- -D warnings` | exit **0** ✓ |
| Clippy headless | `clippy --no-default-features -- -D warnings` | exit **0** ✓ |
| Headless build | `build -p mnemonic-gui --no-default-features` | `Finished` ✓ |
| headless==default emit | `--emit-all` (both binaries) `diff -rq` | byte-identical (61) ✓ |
| Determinism | re-emit default, `diff -rq` | identical ✓ |
| fmt / mlock | fold touches 3 files; `mlock.rs` untouched; no mass-reformat | clean ✓ |
| Secret hygiene | fixtures = FAKE `FormState::default()`; secrets = `MASKED`; divergences coordinate-only | ✓ |
| Deleted-comment | false "construction frame (no run())" replaced with correct "Harness SETTLES in its constructor (run_ok)" | ✓ |

No committed `.gui` files exist in this repo (Leg-2 generates the manual renders downstream),
so nothing is stale.

### 6. New Critical / Important

- **Critical:** none.
- **Important:** I1 (above).

---

## Bottom line

The fold correctly adopts ruling A's seeded fixed point: the 3-form/6-flag delta is exactly
right and GUI-faithful, the disabled re-gate has reproduced teeth and is self-guarding, the
`(required)` drop is correct, the wrong settling-comment is fixed, and every release gate is
green (622/0/4, both clippy, headless==default byte-identical, deterministic, no broad fmt,
secret-safe). The one open item is I1: `seeded_fixture`'s blanket `flag_is_secret` skip
under-seeds the lone secret-Composite-repeating flag (`seed-xor-combine --share`) that the real
GUI seeds via `render_repeating`, so the helper does NOT mirror the auto-seed "exactly" as its
own comment and the commit claim. It is provably inert today and self-guarded against silent
future regression, and the fix is a one-line, byte-identical condition tighten — but an open
exact-mirror divergence in the load-bearing fold blocks the 0C/0I gate. **RED — 0C / 1I.** Fold
the precise predicate → re-run the full `-p` suite + clippy → re-dispatch this review to
convergence before proceeding.
