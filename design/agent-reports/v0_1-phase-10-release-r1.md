# Phase 10 Release Roll-up Review — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Phase 10 commit + 4 sibling PR merges + GitHub repo creation + `mnemonic-gui-v0.1.0` tag-push
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §C Phase 10 + §B.14

## Verdict

**2C / 3I — fold needed** (all folded inline post-tag)

Release shipped. Two critical README defects + three important docs/process gaps. Folded as post-tag patches on master; v0.1.0 tag itself untouched.

---

## Critical findings

### C-1 — README.md `md` install tag stale (v0.3.0 vs v0.4.3)

**Confidence:** 100
**File:** `README.md` line 28 (pre-fold)

Pre-fold: `cargo install … --tag descriptor-mnemonic-md-cli-v0.3.0`. Every other authoritative source pins v0.4.3 (pinned-upstream.toml, CHANGELOG, schema-mirror.yml, src/schema/md.rs). A user following the primary install instructions would install the wrong version and trigger the runtime version-mismatch banner.

**Fold:** Updated README to `v0.4.3`.

### C-2 — README has no link to onboarding walkthroughs (SPEC §B.14 / Phase 10 step 3)

**Confidence:** 97
**File:** `README.md` (pre-fold)

SPEC §B.14 R1-N-2 fold: "Each linked from the top-level repo README and the `mnemonic-gui-vX.Y.Z` GitHub release notes." CHANGELOG referenced both; README did not.

**Fold:** Added "First launch (unsigned binaries)" section to README linking both walkthroughs + the v0.2 code-signing FOLLOWUPS slugs.

---

## Important findings

### I-1 — README Status section reads "v0.1.0 in development" after tagging

**Confidence:** 88
**File:** `README.md` line 15 (pre-fold)

**Fold:** Updated to "Released `mnemonic-gui-v0.1.0` on 2026-05-12."

### I-2 — Phase 10 step 6 deviation: direct-to-master without PR CI gate

**Confidence:** 85
**Process** (no code-level fix)

The plan required pushing through a PR that passes the 5-target build matrix before tagging. v0.1.0 was tagged via direct push to master on a fresh repo. Mechanically the only path for a brand-new repo (no prior master history) but sets a precedent.

**Fold:** Added "Process notes" section to `FOLLOWUPS.md` recording the v0.1 deviation + v0.2 enforcement requirement.

### I-3 — macos-gatekeeper walkthrough cites wrong FOLLOWUPS slug in opening

**Confidence:** 80
**File:** `docs/onboarding/macos-gatekeeper-walkthrough.md` line 6 (pre-fold)

Pre-fold opening cited `gui-secret-buffer-allocator-residue` (the zeroize followup) as the code-signing deferral reason. The code-signing slug is `gui-code-signing-mac-developer-id`.

**Fold:** Corrected opening to cite the right slug; added both code-signing slugs (mac + windows) to `FOLLOWUPS.md` deferred-v0.2 list.

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|------------|-------------|
| build.yml tag-push trigger filter `tags: ['mnemonic-gui-v*']` | n/a | Correct syntax for the pushed tag |
| CHANGELOG completeness | n/a | All sections present |
| pinned-upstream.toml ↔ CHANGELOG tag consistency | n/a | All 4 match |
| Onboarding doc content quality | n/a | Clear + technically accurate (modulo I-3 fold) |
| Cargo gauntlet -D warnings | n/a | Verified clean before tag |
| Tag existence on remote | n/a | Pushed and confirmed |
| External GitHub repo creation | n/a | Within user's broad permissions |

---

## Post-fold deliverables

  - README updated: correct `md` tag, onboarding links, current release status
  - macOS walkthrough: correct FOLLOWUPS slug
  - FOLLOWUPS.md: 2 new deferred slugs (code-signing mac + windows); "Process notes" section recording v0.2 PR-CI-gate requirement
  - v0.1.0 tag preserved as shipped; fixes land on master post-tag (acceptable for alpha-stage release)

Build clean; all 93 tests still green.
