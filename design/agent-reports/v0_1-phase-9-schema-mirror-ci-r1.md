# Phase 9 Schema-Mirror CI Review — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Phase 9 landing commit
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §B.11 + §B.12 + §C Phase 9

## Verdict

**0C / 1I — fold needed** (folded inline)

CI workflows technically correct. One Phase 9 exit-gate violation: sibling-repo PRs not opened. Folded by actually opening the 4 PRs.

---

## Important findings

### I-1 — 4 sibling-repo FOLLOWUPS PRs not opened (exit-gate violation)

**Confidence:** 85
**File:** SPEC §C Phase 9 exit gate

Pre-fold: GUI FOLLOWUPS.md shipped a copy-paste body and deferred actual sibling PR creation to manual action. SPEC §C Phase 9 exit gate explicitly requires "4 sibling-repo FOLLOWUPS PRs opened" so Phase 10 step 7 has something to gate against. R1 I-2 fold of the plan also pinned: "sibling-repo entries land in the same cycle as the schema-mirror CI gate (Phase 9), NOT deferred to release."

**Fold:** Opened 4 PRs autonomously (user pre-authorized via the "All bash and git and gh permissions granted" directive):

| Repo | PR | Branch | Pinned tag |
|------|-----|--------|------------|
| bg002h/mnemonic-toolkit | [#13](https://github.com/bg002h/mnemonic-toolkit/pull/13) | followups-mnemonic-gui-schema-mirror | mnemonic-toolkit-v0.8.1 |
| bg002h/descriptor-mnemonic | [#28](https://github.com/bg002h/descriptor-mnemonic/pull/28) | followups-mnemonic-gui-schema-mirror | descriptor-mnemonic-md-cli-v0.4.3 |
| bg002h/mnemonic-secret | [#4](https://github.com/bg002h/mnemonic-secret/pull/4) | followups-mnemonic-gui-schema-mirror | ms-cli-v0.1.0 |
| bg002h/mnemonic-key | [#7](https://github.com/bg002h/mnemonic-key/pull/7) | followups-mnemonic-gui-schema-mirror | mk-cli-v0.2.0 |

Each PR adds an entry to `design/FOLLOWUPS.md` with the slug `mnemonic-gui-schema-mirror`, citing the GUI's same-slug entry and the appropriate pinned tag. The four entries follow the existing `### slug — title` heading format used in each sibling's FOLLOWUPS.md.

GUI `FOLLOWUPS.md` updated to record the four PR URLs in the companion-entries table.

Phase 10 step 7 can now gate on these PRs merging before tag-push.

---

## Hot-spot resolution

| # | Hot spot | Verdict |
|---|----------|---------|
| 1 | schema-mirror.yml install tags vs pinned-upstream.toml | MATCH (4/4) |
| 2 | Workflow injection hardening | No github.event.* usage |
| 3 | build.yml 5-target matrix | matches SPEC §B.12 |
| 4 | REF_NAME env-var injection (POSIX + PowerShell) | correct syntax both shells |
| 5 | Artifact naming non-tag path | gated by startsWith refs/tags/...; safe |
| 6 | package-windows $env:ARTIFACT | correct PowerShell |
| 7 | FOLLOWUPS.md structure | now includes 4 PR URLs (I-1 fold) |
| 8 | ci_workflow_snapshot regression test | 6 step names + 4 tags + env var |
| 9 | build.yml pull_request trigger | fires full 5-target smoke per Phase 10 R1 I-3 |
| 10 | Sibling PRs not auto-opened | **→ I-1, FOLDED** |
| 11 | aarch64-apple-darwin on macos-latest | Apple Silicon runner → native; correct |
| 12 | cargo-test-full-suite step | runs cargo test --workspace |

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|------------|-------------|
| build.yml shipped in Phase 9 vs Phase 10 per SPEC | 72 | Functionally correct; Phase 10's PR-CI gate requires it; pragmatic |
| package-windows `mkdir dist` vs `mkdir -p` | 45 | dist never pre-exists on fresh runner |

---

## Post-fold deliverables

  - GUI repo: `.github/workflows/{schema-mirror,build}.yml` + `FOLLOWUPS.md` + `ci_workflow_snapshot` cell.
  - Sibling repos: 4 open PRs adding mirror-invariant entries to each sibling's `design/FOLLOWUPS.md`.
  - 93 tests across 9 binaries; CI workflow snapshot covers 6 steps + 4 tags + env var.
