# Phase A.1 Doubled-Prefix Artifact Fix — R1

**Date:** 2026-05-13
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit d86524e on branch v0_2
**Plan ref:** /home/bcg/.claude/plans/v0_2-mnemonic-gui.md §C Phase A.1; SPEC §9

## Verdict

**0C / 0I — converge**

Env-passthrough deviation is a strict improvement over the plan's literal snippet. All seven hot spots pass. One sub-threshold naming note; does not block.

---

## Critical findings

None.

## Important findings

None.

## Sub-threshold notes

### N-1 — Step-local `REF_NAME` binding shares a name with the deleted job-level var

**Confidence:** 35
**File:** `.github/workflows/build.yml` lines 39-40

The commit removes the job-level `env: REF_NAME:` block (the doubled-prefix source) and introduces a step-level `env: REF_NAME:` binding inside `compute-version`. A future maintainer scanning the file for `REF_NAME` will find it, see it bound to `${{ github.ref_name }}`, and might incorrectly assume that `${{ env.REF_NAME }}` remains a valid template expression in sibling steps. It is not — step-level env bindings are not promoted to the `env.` context available to other steps — but the visual similarity could mislead.

The regression cell `ci_build_version_step_present` closes this operationally: `!body.contains("env.REF_NAME")` would catch any mistaken reintroduction. The risk is bounded.

Renaming the binding to `GIT_REF_NAME` inside `compute-version` would eliminate the surface resemblance entirely. Not required; N-grade.

---

## Hot-spot resolution

| # | Hot spot | Verdict |
|---|----------|---------|
| 1 | Cross-platform correctness of `compute-version` | PASS — git-bash on `windows-latest` ships bash 4.x and fully supports POSIX `${VAR#prefix}` parameter expansion. `>> "$GITHUB_ENV"` is the GitHub Actions standard env-file protocol, honoured identically across all shells. Quoting `"$GITHUB_ENV"` is correct. No special-char risk: `REF_NAME` is bound at the YAML-template layer (safe); `$REF_NAME` is consumed only as a bash variable in the run block, never re-expanded by the template engine. |
| 2 | Plan-deviation acceptance | ACCEPTED — env-passthrough is strictly safer than the plan's literal snippet. The plan's `VERSION="${{ github.ref_name }}"` inlines a template expression that expands before bash sees the script, creating a script-injection surface if a ref name ever contains shell metacharacters (e.g., a branch named with backticks or `$(…)`). Binding via step-level `env:` and consuming as `$REF_NAME` eliminates that vector. The exit-gate clause "no `REF_NAME` references" was clearly aimed at `env.REF_NAME` template accesses; the regression cell correctly operationalises the meaningful invariant with `!body.contains("env.REF_NAME")`. The body contains `REF_NAME:` (the step-env key) and `$REF_NAME` (a bash variable) — neither matches `env.REF_NAME`. Invariant holds. |
| 3 | Regression-cell coverage | ADEQUATE — `ci_build_version_step_present` catches: (a) removal of the `compute-version` step (`name: compute-version` absent); (b) removal of `shell: bash`; (c) reintroduction of `env.REF_NAME` in any template expression; (d) `env.VERSION` count falling below or above 4 (renaming VERSION to VER drops count to 0; adding a fifth site forces an intentional count-bump with documented rationale). The cell does not validate that the four `env.VERSION` occurrences are in the correct YAML keys vs. comments, which is the same accepted posture as the v0.1 `ci_workflow_snapshot` (substring containment throughout). Coverage is proportionate to the risk. |
| 4 | Edge cases — PR / master / prefixed branch | PASS — On `pull_request`, `github.ref_name` is the HEAD branch name (e.g., `v0_2`), not `refs/pull/N/merge`; the strip is a no-op unless the branch is named with a `mnemonic-gui-` prefix. `upload-artifact` is gated by `startsWith(github.ref, 'refs/tags/mnemonic-gui-v')`, which is false for all PR and master-push triggers; no upload occurs. Package steps do run, placing an artifact in the local `dist/` directory, but that is discarded at job end. On master push `github.ref_name` = `master`; strip → `master`; package creates `mnemonic-gui-master-…` locally; no upload. On a branch named `mnemonic-gui-feature`, strip → `feature`; no upload. All safe. |
| 5 | YAML correctness | PASS — `compute-version` is correctly sequenced between `actions/checkout@v4` and `install-rust` within the `steps:` list. `env:` and `shell: bash` are step-level keys at the correct indent. The YAML parses cleanly (commit message notes pre-commit `ruby -ryaml` verification). No orphaned keys. |
| 6 | Test integration — right file | PASS — `ci_build_version_step_present` belongs in `tests/schema_mirror.rs`. `ci_workflow_snapshot` already lives there (Phase 9 precedent); Phase A.1 declared the same post-implementation snapshot exception; both cells follow an identical pattern. No basis to split into a dedicated `tests/ci_build.rs` at this scope. |
| 7 | v0.1 Phase-9 catch-alls | Plan item 4 ("update README.md if any reference the doubled-prefix artifact name"): README.md uses `cargo install` (no artifact download URL); onboarding docs (`macos-gatekeeper-walkthrough.md`, `windows-smartscreen-walkthrough.md`) already reference single-prefix names (`mnemonic-gui-v0.1.0-x86_64-macos.tar.gz`, `mnemonic-gui-v0.1.0-x86_64-windows.zip`). No update needed; plan item satisfied by verified absence. Pre-existing `mkdir dist` vs `mkdir -p dist` discrepancy (`package-windows` vs `package-unix`) is not introduced by this commit and was previously assessed at confidence 45 (sub-threshold). |

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|------------|-------------|
| `== 4` count too rigid | 30 | Any future 5th site requires updating the assertion and its named rationale — appropriate self-documentation, not a real trap |
| `shell: bash` assertion not scoped to `compute-version` specifically | 25 | build.yml has exactly one `shell: bash` occurrence; combined with the `name: compute-version` check the coverage is adequate |

---

Phase A.1 exit gate: satisfied. `build.yml` valid YAML; `compute-version` step present with `shell: bash`; four `env.VERSION` substitutions applied; no `env.REF_NAME` references; `README.md` unchanged (no doubled-prefix references found to fix); regression snapshot cell `ci_build_version_step_present` authored in `tests/schema_mirror.rs`. 0C / 0I.
