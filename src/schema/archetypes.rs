//! v0.31.0 (SPEC §1) — static mirror of the toolkit's `build-descriptor
//! --spec-schema` `archetypes` section (the per-archetype field-specs the
//! data-driven archetype param form consumes).
//!
//! Transcribed from the pinned v0.52.0 binary's `--spec-schema` output
//! (probed 2026-06-09; the `archetypes` schema section shipped in toolkit
//! v0.51.0 — the pin is already past it, NO pin change this cycle). Wire
//! keys per archetype: `id` / `summary` / `params`; per param: `flag` /
//! `kind` / `required` / `repeatable` / `min` (toolkit `min_count`).
//!
//! Drift gate: `tests/archetype_schema_mirror.rs` runs the pinned binary's
//! `build-descriptor --spec-schema` and compares this table **field-by-field
//! including param ORDER** (load-bearing — the form renders params in schema
//! order) and `summary` verbatim. Any toolkit-side preset change (new
//! archetype, param change, summary reword) fails the gate at the next pin
//! bump — the lagging-gate posture `schema_mirror` established; the leading
//! discipline is the paired-PR rule.

/// One archetype parameter field-spec (e.g. `--key` for `kofn-recovery`).
pub struct ArchetypeParamSpec {
    /// Argv form (e.g. `"--key"`).
    pub flag: &'static str,
    /// Param kind vocabulary: `"key"` | `"threshold"` | `"blocks"` |
    /// `"absolute_locktime"` | `"hex_digest"` (toolkit
    /// `descriptor_builder/schema.rs` kind strings).
    pub kind: &'static str,
    /// True if the param is required for this archetype (wire `required`).
    pub required: bool,
    /// True if the param accepts multiple occurrences UNDER THIS ARCHETYPE
    /// (wire `repeatable`). NOTE: this is per-archetype arity — the
    /// underlying `FlagSchema.repeating` for `--key`/`--recovery-key` stays
    /// `true` (clap Append); the archetypes diverge (R0-r1 C1).
    pub repeatable: bool,
    /// Wire key `min` (toolkit `min_count`) — the minimum OCCURRENCE count.
    /// Feeds ONLY the `(min N)` header annotation (R0-r2 m3: NOT a value
    /// bound — the Number widget `min` comes from the static `FlagSchema`).
    pub min: u32,
}

/// One archetype preset (id + summary + ordered param field-specs).
pub struct ArchetypeSpec {
    /// Archetype id — equals an `ARCHETYPES` const value (minus the GUI's
    /// `""` UNSET sentinel).
    pub id: &'static str,
    /// Toolkit registry summary, shown under the `--archetype` dropdown.
    pub summary: &'static str,
    /// Params in toolkit schema order (render order is load-bearing).
    pub params: &'static [ArchetypeParamSpec],
}

/// The 9 archetype parameter flags (union of every archetype's `params`
/// across `ARCHETYPE_SPECS`). In archetype mode the generic flag loop skips
/// these by NAME-SET (plus `--spec`); the conditional marks the ones NOT
/// declared by the selected archetype `Hidden` so they never emit
/// (SPEC §3/§5). Order: `BUILD_DESCRIPTOR_FLAGS` declaration order.
pub const ARCHETYPE_PARAM_FLAGS: &[&str] = &[
    "--key",
    "--threshold",
    "--recovery-key",
    "--recovery-threshold",
    "--final-key",
    "--older",
    "--recovery-older",
    "--after",
    "--hash",
];

/// The 5 archetypes, toolkit order (== `CliArchetype` / the binary's
/// `archetypes` array order). Transcribed verbatim from the v0.52.0
/// binary's `build-descriptor --spec-schema`.
pub const ARCHETYPE_SPECS: &[ArchetypeSpec] = &[
    ArchetypeSpec {
        id: "decaying-multisig",
        summary: "k-of-n multisig that decays to a smaller recovery quorum \
                  and finally a single key as timelocks expire",
        params: &[
            ArchetypeParamSpec {
                flag: "--key",
                kind: "key",
                required: true,
                repeatable: true,
                min: 2,
            },
            ArchetypeParamSpec {
                flag: "--threshold",
                kind: "threshold",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--older",
                kind: "blocks",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--recovery-key",
                kind: "key",
                required: true,
                repeatable: true,
                min: 2,
            },
            ArchetypeParamSpec {
                flag: "--recovery-threshold",
                kind: "threshold",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--recovery-older",
                kind: "blocks",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--final-key",
                kind: "key",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--after",
                kind: "absolute_locktime",
                required: true,
                repeatable: false,
                min: 1,
            },
        ],
    },
    ArchetypeSpec {
        id: "hashlock-gated",
        summary: "primary key spends with a SHA-256 preimage; a recovery key \
                  takes over after a relative timelock",
        params: &[
            ArchetypeParamSpec {
                flag: "--key",
                kind: "key",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--hash",
                kind: "hex_digest",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--recovery-key",
                kind: "key",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--older",
                kind: "blocks",
                required: true,
                repeatable: false,
                min: 1,
            },
        ],
    },
    ArchetypeSpec {
        id: "kofn-recovery",
        summary: "k-of-n multisig with a single timelocked recovery key",
        params: &[
            ArchetypeParamSpec {
                flag: "--key",
                kind: "key",
                required: true,
                repeatable: true,
                min: 2,
            },
            ArchetypeParamSpec {
                flag: "--threshold",
                kind: "threshold",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--recovery-key",
                kind: "key",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--older",
                kind: "blocks",
                required: true,
                repeatable: false,
                min: 1,
            },
        ],
    },
    ArchetypeSpec {
        id: "simple-timelocked-inheritance",
        summary: "owner key spends anytime; an heir key unlocks after a \
                  relative timelock",
        params: &[
            ArchetypeParamSpec {
                flag: "--key",
                kind: "key",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--recovery-key",
                kind: "key",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--older",
                kind: "blocks",
                required: true,
                repeatable: false,
                min: 1,
            },
        ],
    },
    ArchetypeSpec {
        id: "tiered-recovery",
        summary: "primary sorted multisig OR a timelocked recovery threshold \
                  of distinct keys",
        params: &[
            ArchetypeParamSpec {
                flag: "--key",
                kind: "key",
                required: true,
                repeatable: true,
                min: 2,
            },
            ArchetypeParamSpec {
                flag: "--threshold",
                kind: "threshold",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--recovery-key",
                kind: "key",
                required: true,
                repeatable: true,
                min: 2,
            },
            ArchetypeParamSpec {
                flag: "--recovery-threshold",
                kind: "threshold",
                required: true,
                repeatable: false,
                min: 1,
            },
            ArchetypeParamSpec {
                flag: "--older",
                kind: "blocks",
                required: true,
                repeatable: false,
                min: 1,
            },
        ],
    },
];

/// Look up an `ArchetypeSpec` by id. Returns `None` for the `""` UNSET
/// sentinel and any non-archetype string.
pub fn find(id: &str) -> Option<&'static ArchetypeSpec> {
    ARCHETYPE_SPECS.iter().find(|a| a.id == id)
}

// ─── SPEC §1 self-consistency units (the public-SCHEMA route — R0-r1 M2:
//     `ARCHETYPES` / `BUILD_DESCRIPTOR_FLAGS` are module-private in
//     `schema/mnemonic.rs`; we reach both through the public `SCHEMA`
//     just like `tests/build_descriptor_schema.rs`). ─────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{FlagKind, SubcommandSchema};

    fn build_descriptor_sub() -> &'static SubcommandSchema {
        crate::schema::mnemonic::SCHEMA
            .subcommands
            .iter()
            .find(|s| s.name == "build-descriptor")
            .expect("build-descriptor must appear in mnemonic SUBCOMMANDS")
    }

    /// The `--archetype` Dropdown options minus the leading `""` UNSET
    /// sentinel — the public route to the private `ARCHETYPES` const.
    fn archetype_dropdown_values() -> &'static [&'static str] {
        let flag = build_descriptor_sub()
            .flags
            .iter()
            .find(|f| f.name == "--archetype")
            .expect("--archetype flag");
        let FlagKind::Dropdown(opts) = flag.kind else {
            panic!("--archetype must be a Dropdown");
        };
        assert_eq!(opts.first(), Some(&""), "ARCHETYPES[0] is the sentinel");
        &opts[1..]
    }

    #[test]
    fn archetype_spec_ids_match_archetypes_const_order_and_set() {
        // SPEC §1: every ARCHETYPE_SPECS id ∈ ARCHETYPES (sentinel
        // excluded) — and the orders match (the dropdown and the spec
        // table iterate in toolkit CliArchetype order).
        let ids: Vec<&str> = ARCHETYPE_SPECS.iter().map(|a| a.id).collect();
        assert_eq!(
            ids,
            archetype_dropdown_values(),
            "ARCHETYPE_SPECS ids must equal ARCHETYPES minus the \"\" \
             sentinel, in order"
        );
    }

    #[test]
    fn every_param_flag_exists_in_build_descriptor_flags() {
        // SPEC §1: every `flag` ∈ BUILD_DESCRIPTOR_FLAGS names.
        let names: Vec<&str> = build_descriptor_sub()
            .flags
            .iter()
            .map(|f| f.name)
            .collect();
        for spec in ARCHETYPE_SPECS {
            for p in spec.params {
                assert!(
                    names.contains(&p.flag),
                    "{}'s param {} is not a BUILD_DESCRIPTOR_FLAGS name",
                    spec.id,
                    p.flag,
                );
            }
        }
        for flag in ARCHETYPE_PARAM_FLAGS {
            assert!(
                names.contains(flag),
                "ARCHETYPE_PARAM_FLAGS entry {flag} is not a \
                 BUILD_DESCRIPTOR_FLAGS name",
            );
        }
    }

    #[test]
    fn archetype_param_flags_is_exactly_the_union_of_declared_params() {
        // The name-set the host loop skips (§3) and the conditional hides
        // from (§5) must be EXACTLY the union of every archetype's params —
        // a missing member would leave a param flag double-rendered, an
        // extra member would suppress a generic flag for no reason.
        use std::collections::BTreeSet;
        let union: BTreeSet<&str> = ARCHETYPE_SPECS
            .iter()
            .flat_map(|a| a.params.iter().map(|p| p.flag))
            .collect();
        let listed: BTreeSet<&str> = ARCHETYPE_PARAM_FLAGS.iter().copied().collect();
        assert_eq!(
            union, listed,
            "ARCHETYPE_PARAM_FLAGS must equal the union of all declared \
             archetype params"
        );
    }

    #[test]
    fn param_kinds_use_the_known_vocabulary() {
        // The §4 widget map has 5 arms; an unknown kind string would fall
        // through to the defensive generic arm. Pin the vocabulary so a
        // toolkit-side kind addition is a deliberate decision here.
        const KINDS: &[&str] =
            &["key", "threshold", "blocks", "absolute_locktime", "hex_digest"];
        for spec in ARCHETYPE_SPECS {
            for p in spec.params {
                assert!(
                    KINDS.contains(&p.kind),
                    "{}'s param {} has unknown kind {:?}",
                    spec.id,
                    p.flag,
                    p.kind,
                );
            }
        }
    }
}
