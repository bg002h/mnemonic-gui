# CLAUDE.md — mnemonic-gui repo notes

This file is auto-loaded by Claude Code when starting a session in this repository.

## What this is

`mnemonic-gui` is the cross-platform GUI overlay for the **m-format constellation** CLIs (`mnemonic`, `md`, `ms`, `mk`). Built with [`egui`](https://github.com/emilk/egui); single statically-linked binary per platform (Linux x86_64/aarch64, macOS x86_64/ARM, Windows x86_64).

The GUI does not implement CLI logic itself — it assembles flag-argv arrays and spawns the toolkit/codec binaries pinned in `pinned-upstream.toml`. The constellation source repos are:

- [`mnemonic-toolkit`](https://github.com/bg002h/mnemonic-toolkit) — top-level integration crate + `mnemonic` CLI; the primary upstream.
- [`md-codec`](https://github.com/bg002h/descriptor-mnemonic) — wallet descriptor codec; CLI `md`.
- [`mk-codec`](https://github.com/bg002h/mnemonic-key) — xpub codec; CLI `mk`.
- [`ms-codec`](https://github.com/bg002h/mnemonic-secret) — BIP-39 entropy codec; CLI `ms`.

## Cross-repo follow-ups

When GUI work surfaces an action item that affects a sibling codec or the toolkit, mirror an entry in BOTH repos' `FOLLOWUPS.md` / `design/FOLLOWUPS.md` with cross-citing `Companion:` lines. When the action ships, both entries update in lockstep.

## GUI schema-mirror coverage (toolkit lockstep)

`src/schema/mnemonic.rs` is the hand-maintained clap-flag schema mirror of the `mnemonic-toolkit` CLI surface (subcommand-by-subcommand flag listings + dropdown value enums), enforced by the `schema_mirror` integration test which runs `mnemonic gui-schema` against the pinned toolkit binary and compares against the hand-maintained schema.

**Mirror invariant (companion to toolkit `CLAUDE.md`):** any toolkit CLI surface change (clap flag / option / subcommand / dropdown value add / remove / rename) MUST update `src/schema/mnemonic.rs` in lockstep — same PR if cross-repo authoring is feasible, otherwise a paired sibling PR. The `schema_mirror` test fires on pin bumps (`pinned-upstream.toml` bump) as a **lagging indicator**; the leading discipline is the paired-PR rule.

When ingesting a toolkit pin bump, run `cargo test --test schema_mirror` FIRST. If it fails with missing flags, those represent accumulated lockstep gaps — address them inline in the pin-bump PR.

Historical case study (v0.27.0 + v0.27.1): neither toolkit cycle paired its CLI additions with a GUI schema-mirror update. The v0.11.1 pin bump v0.26.0 → v0.27.2 fired the drift gate against 8 accumulated missing flags (`bundle --import-json`, `--import-json-index`; `export-wallet --bsms-form`, `--from-import-json`, `--from-import-json-index`, `bsms` format, `BSMS_FORMS` enum; `import-wallet --bsms-round1`, `--bsms-verify-strict`). Phase 3 of v0.11.1 had to backfill all 8 in one go.

See toolkit `design/FOLLOWUPS.md` entry `gui-schema-mirror-lockstep-discipline` for the canonical record. Companion convention lives in `mnemonic-toolkit/CLAUDE.md`.

## Conventions

- **Default to ultracode (multi-agent orchestration) — refined policy** (2026-06-17, after an architect panel; verdict: keep default-ON, refine per-phase). Standing user directive, project-wide across the m-format constellation + seedhammer fork; does NOT require the per-turn `ultracode` keyword. **Default ON for every *substantial* task; token cost is not a constraint.** Trivial one-line/mechanical edits, version bumps, and plain Q&A run solo. **Per-phase pattern:** (1) **research/recon** — fan out parallel subagents; any agent handling **external protocol facts** (BIP-39, BCH/codec semantics, NDEF, RP2350 OTP, SDK behavior) MUST verify them against **authoritative source text**, not just the draft doc (guards against false-consensus on plausible-but-wrong facts — the "1 valid last word" class). (2) **design/spec/plan** — single author + the mandatory R0 loop. (3) **implementation** — a *single* subagent executes the GREEN plan in a worktree (NOT parallel re-implementations); TDD. (4) **post-implementation** — a **mandatory, non-deferrable** independent adversarial execution review over the whole diff (R0 = plan correctness; this catches implementation-introduced regressions TDD misses). (5) if Agent-API dispatch fails mid-session, **flag it explicitly** and defer the formal review to API recovery — never silently substitute inline self-review. Composes with — does not replace — the R0 gate; verbatim agent reports persist to `design/agent-reports/`.
- Source in `src/`; schema mirror at `src/schema/mnemonic.rs`.
- Design artifacts in `design/`: plan docs, session handoffs, `FOLLOWUPS.md` at repo root.
- Per-phase TDD: tests written before impl. Per-phase reviewer-loop until 0 critical / 0 important.
- Stage paths explicitly (no `git add -A`).
- Pin bumps: update `pinned-upstream.toml` AND run `cargo test --test schema_mirror` before committing.

## Parallel execution — this machine has 24 CPU cores

**Standing directive (2026-08-19): consider parallel execution for ALL tests,
cache generation and long calculations.** The defaults use almost none of the
box. Measured constellation-wide the same day: **824s → 204s (~4×)**.

- **Rust — `cargo nextest run --locked`**, not `cargo test`. `cargo test` runs
  each test *binary* serially; nextest spreads them over all cores. Per-repo
  measurements: mnemonic-toolkit 256s→49s, descriptor-mnemonic 40s→27s,
  mnemonic-engrave 33s→16s, mnemonic-secret 2s→0.3s. `cargo-nextest` 0.9.140 is
  installed.
- **Go — shard the package.** `-parallel` does NOTHING unless tests call
  `t.Parallel()`; the fork's `gui` package has 886 test funcs and zero of them.
  `mnemonic-engrave/scripts/gui-shard-test.sh <pkg> 24` took `./gui/` from 493s
  to 112s. It enumerates its partition from `go test -list` and **asserts the
  union is exhaustive before running**, so it cannot silently drop a test — any
  replacement must do the same.
- **Long independent work** — cache/corpus generation, fixture derivation, batch
  rendering — is a candidate too. Ask whether it is CPU-bound and independent
  before running it in a loop.

**Speed WITHOUT dropping debug_assertions.** Do NOT reach for `--release` to
speed tests up — it drops `debug_assertions` and overflow checks, so mutation
tests and invariant panics stop detecting things while still reporting green.
Raise the optimisation level instead and keep them:

```toml
[profile.test]
opt-level = 2

[profile.dev]
opt-level = 2
```

`debug-assertions` defaults to **true** on both profiles, so this is pure gain.
Measured on descriptor-mnemonic: execution **25.4s → 0.765s**, versus 0.775s for
`--release` — the same speed, with the checks intact. Verified empirically, not
inferred: at `opt-level = 2` both `cfg!(debug_assertions)` and an
`attempt to add with overflow` panic still fire. Cost is a slower first build of
dependencies, cached thereafter.

**Check what `/tmp` is before building there.** On this box it is a 32 GB tmpfs,
and a scratch worktree's `target/` filled it and killed a running test.

**Never run the same suite twice** to collect counts and failures separately.
Capture once to a file, then grep it — otherwise every measurement costs double.
