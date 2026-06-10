//! v0.31.0 (archetype forms SPEC §2) — `archetype_schema_mirror` drift gate.
//!
//! Runs the pinned toolkit binary's `build-descriptor --spec-schema`,
//! parses the `archetypes` section, and asserts **field-by-field equality**
//! with the GUI's static mirror `schema::archetypes::ARCHETYPE_SPECS`:
//! ids (set + order), per-param `flag`/`kind`/`required`/`repeatable`/`min`
//! **in declared param ORDER** (load-bearing — SPEC §3 renders params in
//! schema order; R0-r1 M5c), and `summary` verbatim.
//!
//! Any toolkit-side preset change (new archetype, param change, summary
//! reword) fails this gate at the next pin bump — the lagging-gate posture
//! `schema_mirror` established; the leading discipline is the paired-PR
//! rule.
//!
//! Skip discipline (R0-r1 I4): this is a const-mirror-vs-binary parity
//! test, so it uses the **skip-if-absent** pattern from
//! `tests/schema_mirror.rs::single_sig_templates_const_matches_meta_template_groups`
//! (eprintln + return when no binary) — NOT the main mirror gate's
//! fail-loud. CI still runs it because the workflow exports `MNEMONIC_BIN`.

use std::process::Command;

use mnemonic_gui::schema::archetypes::ARCHETYPE_SPECS;

#[test]
fn archetype_specs_match_pinned_binary_spec_schema() {
    // Skipped when MNEMONIC_BIN is unset AND `mnemonic` is not on PATH
    // (matches the const-vs-binary parity tests' runtime-skip discipline).
    let bin: String = match std::env::var("MNEMONIC_BIN").ok() {
        Some(b) => b,
        None => match Command::new("mnemonic").arg("--help").output() {
            Ok(_) => "mnemonic".into(),
            Err(_) => {
                eprintln!(
                    "MNEMONIC_BIN unset + PATH lookup failed; \
                     skipping archetype_specs const-vs-binary parity"
                );
                return;
            }
        },
    };
    let out = Command::new(&bin)
        .args(["build-descriptor", "--spec-schema"])
        .output()
        .expect("failed to spawn `mnemonic build-descriptor --spec-schema`");
    assert!(
        out.status.success(),
        "`mnemonic build-descriptor --spec-schema` exited non-zero: {:?}",
        out.status
    );
    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("--spec-schema stdout must be JSON");
    let archetypes = json["archetypes"]
        .as_array()
        .expect("--spec-schema must carry an `archetypes` array");

    // Count + id ORDER (set equality follows from order equality).
    assert_eq!(
        archetypes.len(),
        ARCHETYPE_SPECS.len(),
        "archetype count drift: binary has {}, ARCHETYPE_SPECS has {} — \
         update src/schema/archetypes.rs in lockstep with the toolkit pin",
        archetypes.len(),
        ARCHETYPE_SPECS.len(),
    );
    for (i, (wire, spec)) in archetypes.iter().zip(ARCHETYPE_SPECS).enumerate() {
        assert_eq!(
            wire["id"].as_str(),
            Some(spec.id),
            "archetype id drift at index {i} (id order is the dropdown \
             order — compare in declared order, not as a set)",
        );
        assert_eq!(
            wire["summary"].as_str(),
            Some(spec.summary),
            "summary drift for `{}` (the summary renders verbatim under \
             the --archetype dropdown)",
            spec.id,
        );
        let params = wire["params"]
            .as_array()
            .unwrap_or_else(|| panic!("`{}` must carry a params array", spec.id));
        assert_eq!(
            params.len(),
            spec.params.len(),
            "param count drift for `{}`",
            spec.id,
        );
        for (j, (wp, sp)) in params.iter().zip(spec.params).enumerate() {
            let ctx = format!("`{}` param {j} (`{}`)", spec.id, sp.flag);
            assert_eq!(
                wp["flag"].as_str(),
                Some(sp.flag),
                "{ctx}: flag drift — param ORDER is load-bearing (SPEC §3 \
                 renders in schema order; R0-r1 M5c)",
            );
            assert_eq!(wp["kind"].as_str(), Some(sp.kind), "{ctx}: kind drift");
            assert_eq!(
                wp["required"].as_bool(),
                Some(sp.required),
                "{ctx}: required drift",
            );
            assert_eq!(
                wp["repeatable"].as_bool(),
                Some(sp.repeatable),
                "{ctx}: repeatable drift (per-archetype arity — drives the \
                 §4 widget-map arm selection)",
            );
            assert_eq!(
                wp["min"].as_u64(),
                Some(u64::from(sp.min)),
                "{ctx}: min drift (feeds the `(min N)` header annotation)",
            );
        }
    }
}
