//! T1 — plan properties and refusals, pure (DESIGN §A9). The Rust port of
//! `test_plan.py`'s legs, run against the Rust planner:
//!
//! - the plan property with DISTINCT sentinels (no argv token holds one; each
//!   binding at its own source's flag; each channel carries its own source's
//!   target + terminator; env names unique; ≤ 1 stdin);
//! - C1: the seven refusal spellings on every source of every shape, on all
//!   three OSes;
//! - resolution under both rules and per input; no input `UNKNOWN`;
//! - the pass-through guard; `no-table-entry` everywhere; the interim switch;
//! - M5 permutations; the lenient rule; `payload-too-large`;
//! - NI3 interim values; NI8 lookalikes (+ consistency with the measured
//!   re-reads, + ordinary secrets pass); NI9; Nm13; env-empty; NUL;
//! - the pin identity check and the per-input Copy gate;
//! - the lookalike normalization's exhaustive parity with the reference.
//!
//! Expected answers never come from the data under test (§A0 oracle rule):
//! the refusal codes and target bytes here are the design's decisions.

mod secret_channels_common;

use std::collections::BTreeSet;

use mnemonic_gui::form::channels::{
    channel_table, guard_passthrough, looks_like_channel, materialize, plan_sources, policy,
    resolve, ChannelKind, EnvRule, PlanOptions, PlanSource, Policy, SourceForm, SourceValue,
};
use secret_channels_common::{
    corpus, env_of, measurement_dir, python_json, shape_assembled, shapes, PLATFORMS,
};

fn code(r: Result<impl Sized, mnemonic_gui::form::channels::Refusal>) -> Option<&'static str> {
    r.err().map(|e| e.code)
}

fn plan(
    srcs: &[PlanSource],
    os: &str,
    env: &[(&str, &str)],
    rule: Option<EnvRule>,
) -> Result<mnemonic_gui::form::channels::Planned, mnemonic_gui::form::channels::Refusal> {
    plan_sources(
        srcs,
        channel_table(),
        policy(),
        os,
        &env_of(env),
        PlanOptions {
            rule_override: rule,
        },
    )
}

#[test]
fn t1_plan_property_with_distinct_sentinels() {
    let mut legs = 0;
    for sh in shapes() {
        // every source filled with a distinct sentinel (grouped: one per member)
        let mut values: Vec<Vec<String>> = Vec::new();
        for (i, s) in sh.sources.iter().enumerate() {
            let n = s.values.len();
            values.push((0..n).map(|k| format!("SENTINEL{i}X{k}")).collect());
        }
        let srcs: Vec<PlanSource> = sh
            .sources
            .iter()
            .zip(&values)
            .map(|(s, v)| {
                let mut p = s.plan_source();
                p.value = if s.is_group() {
                    SourceValue::group(v.clone())
                } else {
                    SourceValue::one(v[0].clone())
                };
                p
            })
            .collect();
        let Ok(planned) = plan(&srcs, "linux", &[], None) else {
            continue;
        };
        let asm = shape_assembled(sh, &values);
        let rp = materialize(&asm, &planned);
        let all: Vec<&String> = values.iter().flatten().collect();
        for t in &rp.argv {
            assert!(
                !all.iter().any(|s| t.contains(s.as_str())),
                "{}: sentinel on argv: {t}",
                sh.name
            );
        }
        let mut envs = BTreeSet::new();
        let mut stdins = 0;
        for (i, bv) in rp.bindings.iter().enumerate() {
            let b = &bv.binding;
            let s = &sh.sources[i];
            let want = if s.is_group() {
                values[i].join("\n")
            } else {
                format!("{}{}", values[i][0], b.terminator)
            };
            match b.kind {
                ChannelKind::EnvRef => {
                    let name = b.env.clone().unwrap();
                    assert!(envs.insert(name.clone()), "{}: env name reused", sh.name);
                    let got = rp
                        .env
                        .iter()
                        .find(|(k, _)| *k == name)
                        .map(|(_, v)| v.to_string());
                    assert_eq!(
                        got.as_deref(),
                        Some(want.as_str()),
                        "{} src{i}: env carries the wrong bytes",
                        sh.name
                    );
                    let tok = &rp.argv[bv.argv_index.unwrap()];
                    assert_eq!(
                        tok,
                        &format!("{}@env:{name}", s.prefix),
                        "{} src{i}: env ref token",
                        sh.name
                    );
                    if let Some(f) = &s.flag {
                        assert_eq!(
                            &rp.argv[bv.argv_index.unwrap() - 1],
                            f,
                            "{} src{i}: not at its flag",
                            sh.name
                        );
                    }
                }
                k if k.is_stdin() => {
                    stdins += 1;
                    assert_eq!(
                        rp.stdin
                            .as_deref()
                            .map(|b| String::from_utf8_lossy(b).to_string()),
                        Some(want.clone()),
                        "{} src{i}: stdin bytes",
                        sh.name
                    );
                    let tok = &rp.argv[bv.argv_index.unwrap()];
                    match k {
                        ChannelKind::StdinToggle => {
                            assert_eq!(Some(tok), b.flag.as_ref(), "{} src{i}", sh.name)
                        }
                        ChannelKind::DashValue => {
                            assert_eq!(tok, &format!("{}-", s.prefix));
                            assert_eq!(
                                &rp.argv[bv.argv_index.unwrap() - 1],
                                s.flag.as_ref().unwrap(),
                                "{} src{i}: not at its flag",
                                sh.name
                            );
                        }
                        _ => assert_eq!(tok, "-"),
                    }
                }
                k if k.is_fd() => {
                    let fd = b.fd.unwrap();
                    let (_, bytes) = rp.fds.iter().find(|(n, _)| *n == fd).expect("fd payload");
                    let want_fd = if s.is_group() {
                        format!("{want}\n")
                    } else {
                        want.clone()
                    };
                    assert_eq!(
                        String::from_utf8_lossy(bytes),
                        want_fd,
                        "{} src{i}: fd bytes",
                        sh.name
                    );
                    assert_eq!(rp.argv[bv.argv_index.unwrap() + 1], format!("/dev/fd/{fd}"));
                }
                other => panic!("{}: unexpected {other:?} on linux", sh.name),
            }
            legs += 1;
        }
        assert!(stdins <= 1, "{}: two stdin bindings", sh.name);
    }
    assert!(legs > 50, "only {legs} legs");
}

/// (typed spelling, the user's environment, the refusal code)
type C1Leg = (
    &'static str,
    &'static [(&'static str, &'static str)],
    &'static str,
);

const C1: &[C1Leg] = &[
    ("-", &[], "C1-dash"),
    (
        "@env:MNEMONIC_GUI_S0",
        &[("MNEMONIC_GUI_S0", "x")],
        "C1-reserved-name",
    ),
    (
        "@env:MNEMONIC_GUI_OTHER",
        &[("MNEMONIC_GUI_OTHER", "x")],
        "C1-reserved-name",
    ),
    ("@env:lower_case", &[("lower_case", "x")], "C1-bad-name"),
    ("@env:9LEAD", &[("9LEAD", "x")], "C1-bad-name"),
    ("@env:NOT_SET_ANYWHERE", &[], "C1-env-unset"),
    ("@env:EMPTY", &[("EMPTY", "")], "C1-env-empty"),
];

#[test]
fn t1_c1_refusals_on_every_source_and_os() {
    let mut n = 0;
    for sh in shapes() {
        for i in 0..sh.sources.len() {
            for (spelling, env, want) in C1 {
                for os in PLATFORMS {
                    n += 1;
                    let got = code(plan(&sh.with_value(i, spelling), os, env, None));
                    assert_eq!(got, Some(*want), "C1 {} src{i} {spelling:?} {os}", sh.name);
                }
            }
        }
    }
    assert_eq!(n, 1260, "the design pins 1260 C1 legs");
}

fn pw_source(v: &str) -> Vec<PlanSource> {
    vec![PlanSource {
        key: "mnemonic restore --passphrase".into(),
        form: SourceForm::Value,
        value: SourceValue::one(v),
    }]
}

#[test]
fn t1_resolution_under_both_rules_and_per_input() {
    for (raw, rule, want) in [
        ("pw\n", EnvRule::Verbatim, "pw\n"),
        ("pw\r\n", EnvRule::Verbatim, "pw\r\n"),
        ("pw\n", EnvRule::StripOneTrailingNewline, "pw"),
        ("pw\r\n", EnvRule::StripOneTrailingNewline, "pw"),
        ("pw\n\n", EnvRule::StripOneTrailingNewline, "pw\n"),
        ("a\nb", EnvRule::StripOneTrailingNewline, "a\nb"),
        ("pw  ", EnvRule::Verbatim, "pw  "),
    ] {
        let (res, prov) = resolve(
            &pw_source("@env:MY_PW"),
            &env_of(&[("MY_PW", raw)]),
            None,
            policy(),
            PlanOptions {
                rule_override: Some(rule),
            },
        )
        .unwrap();
        assert_eq!(
            res[0].value.values(),
            vec![want],
            "resolve {raw:?} {rule:?}"
        );
        assert_eq!(prov[0][0].to_string(), "$MY_PW");
    }
    // Per input (R3 NI7): with no override, THIS input's derived rule applies.
    for (key, e) in channel_table() {
        let one = vec![PlanSource {
            key: key.clone(),
            form: SourceForm::Value,
            value: SourceValue::one("@env:MY_PW"),
        }];
        let (res, _) = resolve(
            &one,
            &env_of(&[("MY_PW", "pw\n")]),
            Some(channel_table()),
            policy(),
            PlanOptions::default(),
        )
        .unwrap();
        let want = match e.cli_env_rule {
            None | Some(EnvRule::Verbatim) => "pw\n",
            Some(EnvRule::StripOneTrailingNewline) => "pw",
            Some(EnvRule::Unknown) => unreachable!(),
        };
        assert_eq!(res[0].value.values(), vec![want], "per-input rule {key}");
    }
    assert!(
        !channel_table()
            .values()
            .any(|e| e.cli_env_rule == Some(EnvRule::Unknown)),
        "an input's CLI @env: rule measured UNKNOWN: an unmodelled rule must stop the bump"
    );
}

#[test]
fn t1_unknown_rule_refuses() {
    let mut t = channel_table().clone();
    t.get_mut("mnemonic restore --passphrase")
        .unwrap()
        .cli_env_rule = Some(EnvRule::Unknown);
    let got = plan_sources(
        &pw_source("@env:MY_PW"),
        &t,
        policy(),
        "linux",
        &env_of(&[("MY_PW", "pw")]),
        PlanOptions::default(),
    );
    assert_eq!(code(got), Some("C1-env-rule-unknown"));
}

#[test]
fn t1_passthrough_guard() {
    assert_eq!(
        code(guard_passthrough(["xpub=@env:MNEMONIC_GUI_S0"], policy())),
        Some("C1-reserved-name")
    );
    assert_eq!(
        code(guard_passthrough(["xpub=@env:MY_XPUB", "bip84"], policy())),
        None
    );
}

#[test]
fn t1_no_table_entry_on_every_os() {
    let fake = vec![PlanSource {
        key: "mnemonic verify-bundle --from phrase=".into(),
        form: SourceForm::Node {
            prefix: "phrase=".into(),
        },
        value: SourceValue::one("x y"),
    }];
    for os in PLATFORMS {
        assert_eq!(
            code(plan(&fake, os, &[], None)),
            Some("no-table-entry"),
            "{os}"
        );
    }
}

#[test]
fn t1_interim_exactly_off_private_channels_on() {
    for sh in shapes() {
        for os in PLATFORMS {
            let Ok(p) = plan(&sh.plan_sources(), os, &[], None) else {
                continue;
            };
            let interim = !policy().private_channels_on.iter().any(|o| o == os);
            assert_eq!(
                p.bindings.iter().all(|b| b.kind == ChannelKind::Argv),
                interim,
                "{} {os}",
                sh.name
            );
        }
    }
}

fn permutations<T: Clone>(v: &[T]) -> Vec<Vec<T>> {
    if v.len() <= 1 {
        return vec![v.to_vec()];
    }
    let mut out = Vec::new();
    for i in 0..v.len() {
        let mut rest = v.to_vec();
        let x = rest.remove(i);
        for mut p in permutations(&rest) {
            p.insert(0, x.clone());
            out.push(p);
        }
    }
    out
}

#[test]
fn t1_m5_permutations_never_change_plan_vs_refuse() {
    let mut enabled = policy().clone();
    enabled.private_channels_on = PLATFORMS.iter().map(|s| s.to_string()).collect();
    let none = env_of(&[]);
    for (os, pol) in [("linux", policy()), ("macos", &enabled)] {
        for sh in shapes() {
            let base = plan_sources(
                &sh.plan_sources(),
                channel_table(),
                pol,
                os,
                &none,
                PlanOptions::default(),
            )
            .is_ok();
            for perm in permutations(&sh.plan_sources()) {
                let ok = plan_sources(
                    &perm,
                    channel_table(),
                    pol,
                    os,
                    &none,
                    PlanOptions::default(),
                )
                .is_ok();
                assert_eq!(ok, base, "M5 {} {os}", sh.name);
            }
        }
    }
}

#[test]
fn t1_lenient_rule_and_payload_bound() {
    let ms1 = |v: &str| {
        vec![PlanSource {
            key: "mnemonic xpub-search path-of-xpub --ms1".into(),
            form: SourceForm::Value,
            value: SourceValue::one(v),
        }]
    };
    assert_eq!(
        code(plan(&ms1("ms10abc "), "linux", &[], None)),
        Some("value-not-byte-exact")
    );
    assert_eq!(code(plan(&ms1("ms10abc"), "linux", &[], None)), None);
    let big = vec![
        PlanSource {
            key: "ms verify --phrase".into(),
            form: SourceForm::Value,
            value: SourceValue::one("a b"),
        },
        PlanSource {
            key: "ms verify <ms1>".into(),
            form: SourceForm::Pos,
            value: SourceValue::one("m".repeat(5000)),
        },
    ];
    assert_eq!(
        code(plan(&big, "linux", &[], None)),
        Some("payload-too-large")
    );
}

#[test]
fn t1_ni3_interim_carries_the_resolved_target_never_the_typed_text() {
    for sh in shapes() {
        for (i, s) in sh.sources.iter().enumerate() {
            if s.is_group() {
                continue;
            }
            for rule in [EnvRule::Verbatim, EnvRule::StripOneTrailingNewline] {
                let raw = format!(
                    "{}{}",
                    s.value(),
                    if rule == EnvRule::Verbatim {
                        "  "
                    } else {
                        "\n"
                    }
                );
                for os in ["macos", "windows"] {
                    let p = plan(
                        &sh.with_value(i, "@env:USER_SECRET"),
                        os,
                        &[("USER_SECRET", &raw)],
                        Some(rule),
                    )
                    .unwrap_or_else(|e| {
                        panic!("NI3 {} src{i} {os} {rule:?}: refused {e}", sh.name)
                    });
                    assert_eq!(
                        p.resolved[i].values(),
                        vec![rule.apply(&raw).as_str()],
                        "NI3 {} src{i} {os}",
                        sh.name
                    );
                    assert_eq!(p.bindings[i].kind, ChannelKind::Argv);
                    assert_eq!(p.provenance[i][0].to_string(), "$USER_SECRET");
                }
            }
        }
    }
}

#[test]
fn t1_ni8_every_lookalike_refused_on_every_path() {
    let mut n = 0;
    for sh in shapes() {
        for (i, s) in sh.sources.iter().enumerate() {
            if s.is_group() {
                continue;
            }
            for tmpl in &corpus().channel_variants {
                let content = tmpl.replace("{V}", "OTHER");
                for os in PLATFORMS {
                    n += 1;
                    let got = code(plan(
                        &sh.with_value(i, "@env:USER_SECRET"),
                        os,
                        &[("USER_SECRET", &content), ("OTHER", "hunter2")],
                        None,
                    ));
                    assert_eq!(
                        got,
                        Some("value-looks-like-a-channel"),
                        "NI8 {} src{i} {content:?} {os}",
                        sh.name
                    );
                }
            }
        }
    }
    assert_eq!(n, 3717, "the design pins 3717 NI8 legs");
    // consistency with the MEASUREMENT: every re-read spelling is refused
    for (cli, row) in mnemonic_gui::form::channels::data::reinterpret() {
        for sp in row.spellings {
            assert!(
                looks_like_channel(policy(), &sp.replace("{V}", "X")),
                "{cli} re-reads {sp:?}, which the predicate must refuse"
            );
        }
    }
    for ok in [
        "hunter2",
        "--",
        "-leading",
        "a-b",
        "env:X",
        "@en v",
        "pass@env:X",
    ] {
        assert!(
            !looks_like_channel(policy(), ok),
            "NI8 must not refuse {ok:?}"
        );
    }
    // the design's examples (§A0.1)
    for bad in [
        " @env:X",
        "@ENV:X",
        "\u{200b}@env:X",
        "\u{ff20}env:X",
        "- ",
        "\u{a0}-",
        "@envelope",
    ] {
        assert!(looks_like_channel(policy(), bad), "NI8 must refuse {bad:?}");
    }
}

#[test]
fn t1_ni9_trailing_cr_lf_refused_on_every_path() {
    for os in PLATFORMS {
        for (raw, rule) in [
            ("pw\n", EnvRule::Verbatim),
            ("pw\r", EnvRule::Verbatim),
            ("pw\n\n", EnvRule::StripOneTrailingNewline),
            ("pw\r\n\r\n", EnvRule::StripOneTrailingNewline),
            ("pw\r\r\n", EnvRule::StripOneTrailingNewline),
        ] {
            let got = code(plan(
                &pw_source("@env:MY_PW"),
                os,
                &[("MY_PW", raw)],
                Some(rule),
            ));
            assert_eq!(
                got,
                Some("value-ends-in-newline"),
                "NI9 {raw:?} {rule:?} {os}"
            );
        }
        assert_eq!(
            code(plan(
                &pw_source("@env:MY_PW"),
                os,
                &[("MY_PW", "pw\n")],
                Some(EnvRule::StripOneTrailingNewline)
            )),
            None
        );
        // typed, too
        assert_eq!(
            code(plan(&pw_source("pw\n"), os, &[], None)),
            Some("value-ends-in-newline"),
            "typed {os}"
        );
    }
}

#[test]
fn t1_nm13_leading_dash_on_the_interim_path() {
    for sh in shapes() {
        for (i, s) in sh.sources.iter().enumerate() {
            if s.form != "value" {
                continue;
            }
            for os in ["macos", "windows"] {
                let got = plan(&sh.with_value(i, "-lead  "), os, &[], None);
                if channel_table()[&s.key].argv_eq_exact == Some(true) {
                    let p = got.unwrap_or_else(|e| panic!("Nm13 {} src{i} {os}: {e}", sh.name));
                    assert_eq!(
                        p.bindings[i].argv_form,
                        Some(mnemonic_gui::form::channels::ArgvForm::Eq),
                        "Nm13 {} src{i}",
                        sh.name
                    );
                } else {
                    assert_eq!(
                        code(got),
                        Some("value-starts-with-dash"),
                        "Nm13 {} src{i} {os}",
                        sh.name
                    );
                }
            }
        }
    }
}

#[test]
fn t1_env_empty_is_judged_on_the_target_and_nul_refuses() {
    let r = resolve(
        &pw_source("@env:MY_PW"),
        &env_of(&[("MY_PW", "\n")]),
        None,
        policy(),
        PlanOptions {
            rule_override: Some(EnvRule::StripOneTrailingNewline),
        },
    );
    assert_eq!(code(r), Some("C1-env-empty"));
    let r = resolve(
        &pw_source("@env:MY_PW"),
        &env_of(&[("MY_PW", "\n")]),
        None,
        policy(),
        PlanOptions {
            rule_override: Some(EnvRule::Verbatim),
        },
    );
    assert_eq!(code(r), None);
    for os in PLATFORMS {
        assert_eq!(
            code(plan(&pw_source("a\0b"), os, &[], None)),
            Some("nul-in-value"),
            "{os}"
        );
    }
}

#[test]
fn t1_pin_identity_and_copy_gate() {
    let pins: toml::Table = toml::from_str(
        &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/pinned-upstream.toml"))
            .unwrap(),
    )
    .unwrap();
    let mw: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(measurement_dir().join("measured_with.json")).unwrap(),
    )
    .unwrap();
    for (cli, v) in &pins {
        let Some(tag) = v.get("tag").and_then(|t| t.as_str()) else {
            continue;
        };
        assert_eq!(
            mw[cli]["pinned_tag"], tag,
            "pin: {cli} measured under {}, pinned {tag}",
            mw[cli]["pinned_tag"]
        );
        assert_eq!(
            mw[cli]["sha256"].as_str().map(str::len),
            Some(64),
            "pin: {cli} sha256 recorded"
        );
    }
    // Copy gate (R3 NI7): an input whose CLI rule is not verbatim/none needs
    // an EnvRef cell, where Copy spells the CLI's own `@env:VAR`.
    for (k, e) in channel_table() {
        if !matches!(e.cli_env_rule, None | Some(EnvRule::Verbatim)) {
            assert!(
                e.channels.iter().any(|c| c.kind == ChannelKind::EnvRef),
                "copy gate: {k}"
            );
        }
    }
}

#[test]
fn t1_policy_holds_decisions_the_design_names() {
    let p: &Policy = policy();
    assert_eq!(p.reserved_env_prefix, "MNEMONIC_GUI_");
    assert!(p.refuse_trailing_cr_lf);
    assert_eq!(p.pipe_payload_max, 4096);
    assert_eq!(
        p.real_binary_test_targets,
        [
            "secret_channels_t2",
            "secret_channels_t3prime",
            "secret_channels_t7"
        ]
    );
}

/// The lookalike normalization agrees with the reference's over EVERY code
/// point, in every position the predicate can see (alone; before `-`, `@env`;
/// after; and inside `@env`). A divergence in NFKC, Cc/Cf, whitespace or
/// case-folding tables shows up here.
#[test]
fn t1_lookalike_predicate_matches_the_reference_exhaustively() {
    let patterns = [
        "{c}",
        "{c}-",
        "-{c}",
        "{c}@env",
        "@env{c}",
        "@{c}nv",
        "@e{c}v",
        "@en{c}",
        "{c}{c}-{c}",
    ];
    let py = python_json(
        &format!(
            "import json, plan\nP={patterns:?}\nout=[]\nfor cp in range(0x110000):\n    if 0xD800<=cp<=0xDFFF: continue\n    c=chr(cp)\n    for k,p in enumerate(P):\n        if plan.looks_like_channel(p.replace('{{c}}',c)): out.append([cp,k])\nprint(json.dumps(out))"
        ),
        &[],
    );
    let want: BTreeSet<(u32, usize)> = py
        .as_array()
        .unwrap()
        .iter()
        .map(|x| {
            (
                x[0].as_u64().unwrap() as u32,
                x[1].as_u64().unwrap() as usize,
            )
        })
        .collect();
    let mut got = BTreeSet::new();
    for cp in 0..0x110000u32 {
        let Some(c) = char::from_u32(cp) else {
            continue;
        };
        let cs = c.to_string();
        for (k, p) in patterns.iter().enumerate() {
            if looks_like_channel(policy(), &p.replace("{c}", &cs)) {
                got.insert((cp, k));
            }
        }
    }
    let only_rust: Vec<_> = got.difference(&want).take(10).collect();
    let only_py: Vec<_> = want.difference(&got).take(10).collect();
    assert!(
        only_rust.is_empty() && only_py.is_empty(),
        "lookalike parity: rust-only {only_rust:x?}, python-only {only_py:x?}"
    );
    assert!(
        want.len() > 1000,
        "the reference refused only {} probes",
        want.len()
    );
}
