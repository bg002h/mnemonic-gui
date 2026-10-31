# mnemonic-gui

Paths below are relative to this Git root, including when working inside a crate.

## Start here

Read [CLAUDE.md](CLAUDE.md) explicitly; AGENTS.md does not load it automatically.
Read applicable ancestor guidance too. Existing process rules remain applicable;
verify historical architecture/status statements against source.
Read the shared [guide](https://github.com/bg002h/mnemonic-toolkit/blob/docs/constellation-architecture/docs/architecture/README.md) and [workflow](https://github.com/bg002h/mnemonic-toolkit/blob/docs/constellation-architecture/docs/architecture/WORKFLOW.md).
For sibling checkouts/worktrees, prefer local `../mnemonic-toolkit/docs/architecture/`;
remote links support standalone checkouts while documentation branches await integration.
Check Git status, the current branch, Cargo.toml and Cargo.lock before editing.
Preserve unrelated work and use explicit task-owned paths when staging.

## Where behavior lives

This is the egui utility GUI, distinct from Refugium's mr-gui.

- [app.rs](src/app.rs) selects mnemonic/md/ms/mk; forms construct
  [invocations](src/form/invocation.rs), and [runner.rs](src/runner.rs) executes subprocesses.
- The GUI also links selected toolkit utilities, including
  [secret taxonomy](src/secrets.rs); this is separate from the CLI boundary.
- [tree_form.rs](src/form/tree_form.rs) consumes structured descriptor results.
  A CLI JSON change can break the GUI while all flags remain unchanged.
- [persistence.rs](src/persistence.rs) owns saved/redacted state.
- [pinned-upstream.toml](pinned-upstream.toml) describes CLI schema expectations;
  Cargo.toml/Cargo.lock independently select the linked toolkit library.

## Contracts and checks

Use schema-mirror, pin-coherence, descriptor-builder and persistence tests.
[Build CI](.github/workflows/build.yml) and [schema CI](.github/workflows/schema-mirror.yml)
define current invocations. Check the GUI's own Rust requirements rather than assuming
the core CLI toolchain applies. Coordinate CLI flags, JSON shapes, binary pins,
secret taxonomy and the toolkit's GUI manual in the same change.
Architecture scope includes this utility GUI at lower priority; general mr-gui work
is outside the current guide.
