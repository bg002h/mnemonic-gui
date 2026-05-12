# Phase 4 Runner Review — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `0dee4b3 Phase 4: subprocess runner + tracing init + integration test`
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §B.7 + §B.8 + §C Phase 4

## Verdict

**0C / 0I — converge**

All 10 hot spots evaluated; none reached the 80-confidence threshold. Phase 4 deliverable is sound.

---

## Hot-spot evaluations

| # | Hot spot | Confidence | Disposition |
|---|----------|------------|-------------|
| HS-1 | argv[0] absolute-path posture | 30 | `assemble_argv` always emits bare cli_name; cell_1's test-only replacement is documented. `Command::new` accepts both. Not a defect. |
| HS-2 | Stdio::null() makes --passphrase-stdin unusable | 40 | Defensive default; Phase 5 will revisit when wiring convert UI. Current scope doesn't exercise stdin-fed paths. |
| HS-3 | `debug!(argv = ?argv)` leaks secrets under --debug | 72 | Below threshold; explicitly Phase 7's concern (SECRET_NODE_TYPES constants will inform a secret-aware logging filter). User must explicitly enable debug mode for exposure. |
| HS-4 | init_tracing / test subscriber conflict | 10 | init_tracing lives in main.rs (bin only); tests link against [lib]; cell_2 uses scoped set_default() guard. No conflict possible. |
| HS-5 | cell_1 argv flag order vs comment | 15 | assemble_argv follows schema order (SPEC §6.3); clap accepts flags in any order; test asserts stdout content vs fixture. Not a bug. |
| HS-6 | cell_3 deadlock coverage at 2 MiB | n/a | Well above Linux 64 KiB pipe buffer; wait_with_output parallel-drain contract holds. |
| HS-7 | PATHEXT split_paths on Windows | 25 | `;` is the correct Windows separator (matches PATHEXT). Each token appended as extension. cfg!(windows) gates the block. |
| HS-8 | Unix execute-bit check | n/a | `mode() & 0o111 != 0` standard. |
| HS-9 | Error class 1 vs 4 distinction | 50 | Runner returns plain io::Error; caller inspects `err.kind() == NotFound` per SPEC §8. Correct design boundary. |
| HS-10 | Cargo.lock retained | n/a | git status clean. No defect. |

---

## Notes for downstream phases

- **HS-3 → Phase 7:** when `secrets::SECRET_NODE_TYPES` and `secrets::SECRET_SLOT_SUBKEYS` land via the `build.rs` codegen, integrate a `runner` hook that redacts values for those node/subkey tokens before the `debug!` argv event fires. Phase 7 review will flag if this integration is missed.
- **HS-2 → Phase 5:** if `--passphrase-stdin` becomes wireable in v0.1, the runner needs a stdin-pipe variant. Currently the convert UX would refuse with a clear non-zero exit, which is acceptable for v0.1.

---

## Confidence-filtered: omitted

All 10 hot spots; none above threshold.
