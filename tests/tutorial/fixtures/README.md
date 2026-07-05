# Tutorial fixtures (`gui_example_tutorial` cycle)

Byte-copies of the four tracked `.examples-build/` primitives from
**mnemonic-toolkit `master@80c65ac9`** (the v0.75.0 tier), under their
Examples-display names, per `SPEC_gui_example_tutorial.md` §3.1(c). These are
the watch-only descriptor / policy inputs the J3 (degrading vault) and J4
(taproot twin) journeys drive through `Path` form fields.

**All fixtures are watch-only — they contain public xpubs + public hashlock
digests only. No secret (BIP-39 phrase / entropy / xprv / WIF / ms1 / seedqr)
material appears in any fixture** (the machine-asserted secret-allowlist in
`tests/gui_tutorial_snapshots.rs` enforces this: the only secret-class values
the harness ever drives are the three published test phrases S0/S1/S2, and they
live as manifest literals, never in a fixture file — SPEC §7).

| Fixture | Source (`.examples-build/`) | Transform | Source SHA-256 |
|---|---|---|---|
| `policy.desc` | `degrade2.desc` | byte-copy | `842249079066b5e53e8b5064177c73a972871eb0634b6d7a0ad3fa627fb7febe` |
| `taproot.desc` | `tr2.desc` | byte-copy | `9c493990717173fec0c9985f302642b455402acf2e419563e94b080c366cd022` |
| `taproot-4leaf.desc` | `tr4.desc` | byte-copy | `b1dcb1b370baedbe89fa61db05e2b50524e3e755924ba2fb17f98fbe8c6861f3` |
| `policy.json` | `degrade2-spec.json` | `opensessame`-digest rewrite (`gen.sh:33-37`) once at authoring | `f79181a73989fd0bbce346a83a87ff1edc22d38ec9eef4e37e8b9667c1fc0221` (pre-rewrite) |

`policy.json` transform (applied ONCE here, exactly `gen.sh:33-37`): the 11-key
spec's placeholder hashlock digest
`68100fc1…3b17bc9d` → the `opensessame` digest
`a84dce40…9b9a08ad`, so the file is self-consistent if a reader inspects it.
The `build-descriptor` guided-builder refuses on key-count before the hash
matters, so the rewrite is cosmetic (the J3 refusal teaching moment survives).

Provenance note: the `.desc` copies are byte-identical to their sources (SHA
match verified at authoring). The pilot phase (Ch 0 + J1) does not consume any
of these files — J1 rides the fresh-app demo seed + the typed S0 phrase — but
they are authored here so the full-manifest journeys (P1.5) reuse this
provenance record rather than re-establishing it.
