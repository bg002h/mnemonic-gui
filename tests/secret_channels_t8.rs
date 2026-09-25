//! T8 — plan parity (DESIGN §A9). The Rust planner equals `plans_pure.json`
//! (`gen_plans.py`, regenerated from `plan.py` by `check_design_tables.py`) on
//! every shape × OS: bindings AND the exact payload bytes each binding
//! delivers, for the typed case and for each source typed as
//! `@env:USER_SECRET` (holding the value plus two spaces).
//!
//! The shapes' argv order is the prototype's; the pure planner is fed that
//! order, so source indices and `MNEMONIC_GUI_S<i>` names are comparable. The
//! form-level mapping (the GUI's own flag order) is pinned separately in
//! `secret_channels_t6.rs`.

mod secret_channels_common;

use mnemonic_gui::form::channels::{
    channel_table, plan_sources, policy, PlanOptions, Planned, Provenance, SourceValue,
};
use secret_channels_common::{env_of, measurement_dir, shapes, PLATFORMS};

fn plans_pure() -> serde_json::Value {
    let raw = std::fs::read_to_string(measurement_dir().join("plans_pure.json")).unwrap();
    serde_json::from_str(&raw).unwrap()
}

fn payloads(p: &Planned) -> Vec<String> {
    p.bindings
        .iter()
        .zip(&p.resolved)
        .map(|(b, v)| match v {
            SourceValue::Group(vs) => vs.iter().map(|x| x.as_str()).collect::<Vec<_>>().join("\n"),
            SourceValue::One(x) => {
                if b.kind == mnemonic_gui::form::channels::ChannelKind::Argv {
                    x.to_string()
                } else {
                    format!("{}{}", x.as_str(), b.terminator)
                }
            }
        })
        .collect()
}

fn prov_json(p: &[Vec<Provenance>], groups: &[bool]) -> serde_json::Value {
    serde_json::Value::Array(
        p.iter()
            .zip(groups)
            .map(|(v, &g)| {
                let s: Vec<serde_json::Value> = v.iter().map(|x| x.to_string().into()).collect();
                if g {
                    serde_json::Value::Array(s)
                } else {
                    s.into_iter().next().unwrap()
                }
            })
            .collect(),
    )
}

#[test]
fn t8_rust_plan_equals_plans_pure_on_every_shape_and_os() {
    let pure = plans_pure();
    let rows = pure.as_array().unwrap();
    let shapes = shapes();
    assert_eq!(
        rows.len(),
        shapes.len(),
        "plans_pure.json and shapes.py disagree on the shape count"
    );
    let table = channel_table();
    let mut once = policy().clone();
    once.private_channels_on = PLATFORMS.iter().map(|s| s.to_string()).collect();
    let none = env_of(&[]);
    let mut compared = 0;
    for (sh, row) in shapes.iter().zip(rows) {
        assert_eq!(sh.name, row["name"]);
        let srcs = sh.plan_sources();
        for (label, pol, os) in [
            ("linux", policy(), "linux"),
            ("macos", policy(), "macos"),
            ("windows", policy(), "windows"),
            ("other_once_enabled", &once, "macos"),
        ] {
            let want = &row["plans"][label];
            match plan_sources(&srcs, table, pol, os, &none, PlanOptions::default()) {
                Ok(p) => {
                    let got: Vec<serde_json::Value> =
                        p.bindings.iter().map(|b| b.to_json()).collect();
                    assert_eq!(
                        serde_json::Value::Array(got),
                        want["bindings"],
                        "{} {label}: bindings differ",
                        sh.name
                    );
                }
                Err(r) => assert_eq!(
                    r.code, want["refusal"],
                    "{} {label}: refusal {} vs {}",
                    sh.name, r.code, want
                ),
            }
            compared += 1;
        }
        // value cases: payload BYTES (R2 NI3)
        let groups: Vec<bool> = sh.sources.iter().map(|s| s.is_group()).collect();
        for case in row["value_cases"].as_array().unwrap() {
            let name = case["case"].as_str().unwrap();
            let os = case["os"].as_str().unwrap();
            let (srcs, env): (Vec<_>, Vec<(String, String)>) = if name == "typed" {
                (sh.plan_sources(), vec![])
            } else {
                let i: usize = name
                    .strip_prefix("src")
                    .and_then(|r| r.split(' ').next())
                    .unwrap()
                    .parse()
                    .unwrap();
                (
                    sh.with_value(i, "@env:USER_SECRET"),
                    vec![("USER_SECRET".into(), format!("{}  ", sh.sources[i].value()))],
                )
            };
            let pairs: Vec<(&str, &str)> =
                env.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
            let uenv = env_of(&pairs);
            match plan_sources(&srcs, table, policy(), os, &uenv, PlanOptions::default()) {
                Ok(p) => {
                    let kinds: Vec<serde_json::Value> = p
                        .bindings
                        .iter()
                        .map(|b| format!("{:?}", b.kind).into())
                        .collect();
                    assert_eq!(
                        serde_json::Value::Array(kinds),
                        case["kinds"],
                        "{} {name} {os}: kinds",
                        sh.name
                    );
                    assert_eq!(
                        prov_json(&p.provenance, &groups),
                        case["provenance"],
                        "{} {name} {os}: provenance",
                        sh.name
                    );
                    let pl: Vec<serde_json::Value> =
                        payloads(&p).into_iter().map(Into::into).collect();
                    assert_eq!(
                        serde_json::Value::Array(pl),
                        case["payloads"],
                        "{} {name} {os}: payload bytes",
                        sh.name
                    );
                }
                Err(r) => assert_eq!(r.code, case["refusal"], "{} {name} {os}: refusal", sh.name),
            }
            compared += 1;
        }
    }
    assert!(compared > 300, "T8 compared only {compared} plans");
}

#[test]
fn t8_describe_matches_the_design_a5_linux_column() {
    // §A5's Linux cells are `plan.describe(...)`; the GUI's Preview uses the
    // same phrasing.
    let doc = std::fs::read_to_string(
        measurement_dir().join("../../DESIGN_secret_channels_and_new_forms.md"),
    )
    .unwrap();
    let none = env_of(&[]);
    for sh in shapes() {
        if let Ok(p) = plan_sources(
            &sh.plan_sources(),
            channel_table(),
            policy(),
            "linux",
            &none,
            PlanOptions::default(),
        ) {
            let cell = format!(
                "| {} | {} |",
                sh.name,
                mnemonic_gui::form::channels::describe(&p.bindings)
            );
            assert!(doc.contains(&cell), "§A5 lacks {cell}");
        }
    }
}
