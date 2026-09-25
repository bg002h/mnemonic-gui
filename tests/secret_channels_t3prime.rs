//! T3′ — multi-secret real-runner legs (DESIGN §A9), against the PINNED
//! release binaries (`MNEMONIC_BIN` required — never a skip; T10 names this
//! target). The Rust port of `run_plans.py`: every plan the Rust planner
//! admits is turned into an invocation by the GUI's own `materialize` and run
//! by the GUI's own `runner::run_plan`, and compared with an INDEPENDENT
//! oracle (§A0) built here by hand:
//!
//! - **baseline:** every plannable shape equals the argv baseline
//!   (`--allow-argv-secret`, values verbatim);
//! - **swap:** every source pair swapped must differ from the baseline, except
//!   the shape's symmetric pairs;
//! - **NI1:** each source typed as `@env:USER_SECRET` over the whole corpus
//!   equals the oracle (the CLI's OWN `@env:USER_SECRET`, or known bytes);
//! - **interim (NI3):** the interim plan, executed on Linux, equals the oracle;
//! - **NC1 (NI8):** the variable holds each lookalike; both paths must be
//!   refused by the predicate, never `$OTHER`'s wallet;
//! - **Nm13:** typed `-leading`, `--help`, `-lead  ` equal the oracle, or with
//!   no oracle fail closed / equal the `--flag=VALUE` argv run.
//!
//! A refusal is always safe (the GUI sends nothing).

mod secret_channels_common;

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use mnemonic_gui::form::channels::{
    channel_table, materialize, plan_sources, policy, ChannelKind, PlanOptions, Planned, RunPlan,
    SourceValue,
};
use secret_channels_common::{bin_dir, corpus, env_of, shape_assembled, shapes, Shape};
use zeroize::Zeroizing;

const PW: &str = "hunter2-passphrase";

#[derive(Clone, Debug, PartialEq, Eq)]
struct Out {
    code: Option<i32>,
    stdout: String,
}

fn ok(o: &Out) -> bool {
    matches!(o.code, Some(0) | Some(4))
}

/// An ORACLE run: plain `Command`, the test's own spelling, never the GUI's.
fn run_oracle(argv: &[String], extra_env: &[(String, String)], stdin: Option<&str>) -> Out {
    let prefix = &policy().reserved_env_prefix;
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    for (k, _) in std::env::vars() {
        if k.starts_with(prefix.as_str()) {
            cmd.env_remove(k);
        }
    }
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn oracle");
    {
        let mut h = child.stdin.take().unwrap();
        let _ = h.write_all(stdin.unwrap_or("").as_bytes());
    }
    let o = child.wait_with_output().unwrap();
    Out {
        code: o.status.code(),
        stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
    }
}

/// A GUI run: the plan materialized by the GUI and run by the GUI's runner.
/// `inherited` stands for variables the GUI process itself holds (the child
/// inherits them).
fn run_gui(
    sh: &Shape,
    planned: &Planned,
    values: &[Vec<String>],
    inherited: &[(String, String)],
) -> Out {
    let mut rp: RunPlan = materialize(&shape_assembled(sh, values), planned);
    rp.argv[0] = bin(&sh.cli);
    for (k, v) in inherited {
        rp.env.push((k.clone(), Zeroizing::new(v.clone())));
    }
    let r = mnemonic_gui::runner::run_plan(&rp).expect("runner");
    Out {
        code: r.exit_code,
        stdout: String::from_utf8_lossy(&r.stdout).into_owned(),
    }
}

fn bin(cli: &str) -> String {
    let b: PathBuf = bin_dir().join(cli);
    b.to_string_lossy().into_owned()
}

/// `run_plans.baseline_argv`.
fn baseline_argv(
    sh: &Shape,
    values: &[Vec<String>],
    env_spell: Option<(usize, &str)>,
    toggle: Option<(usize, &str)>,
    eq: &[usize],
) -> Vec<String> {
    let mut argv = vec![bin(&sh.cli)];
    argv.extend(sh.sub.iter().cloned());
    argv.push("--allow-argv-secret".into());
    argv.extend(sh.pre.iter().cloned());
    let mut posit = Vec::new();
    for (i, (s, v)) in sh.sources.iter().zip(values).enumerate() {
        let mut v0 = v[0].clone();
        if let Some((k, name)) = env_spell {
            if k == i {
                v0 = format!("@env:{name}");
            }
        }
        if let Some((k, flag)) = toggle {
            if k == i {
                argv.push(flag.to_string());
                continue;
            }
        }
        match s.form.as_str() {
            "node" => {
                argv.push(s.flag.clone().unwrap());
                argv.push(format!("{}{v0}", s.prefix));
            }
            "value" => {
                if eq.contains(&i) {
                    argv.push(format!("{}={v0}", s.flag.as_ref().unwrap()));
                } else {
                    argv.push(s.flag.clone().unwrap());
                    argv.push(v0);
                }
            }
            "pos" => posit.push(v0),
            _ => posit.extend(v.iter().cloned()),
        }
    }
    argv.extend(sh.post.iter().cloned());
    if !posit.is_empty() {
        argv.push("--".into());
        argv.extend(posit);
    }
    argv
}

/// `run_plans.normalise`: slip39 split is randomised — compare what the shares
/// recover (with the passphrase actually delivered).
fn normalise(sh: &Shape, o: &Out, pw: &str) -> String {
    if !sh.name.starts_with("slip39 split") || o.code != Some(0) {
        return o.stdout.clone();
    }
    let shares: Vec<&str> = o
        .stdout
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .take(2)
        .collect();
    let mut argv = vec![
        bin("mnemonic"),
        "slip39".into(),
        "combine".into(),
        "--allow-argv-secret".into(),
        "--passphrase-stdin".into(),
    ];
    for s in shares {
        argv.push("--share".into());
        argv.push(s.to_string());
    }
    run_oracle(&argv, &[], Some(&format!("{pw}\r\n"))).stdout
}

fn same(sh: &Shape, a: &Out, b: &Out, pw: &str) -> bool {
    a.code == b.code && normalise(sh, a, pw) == normalise(sh, b, pw)
}

fn values_of(sh: &Shape) -> Vec<Vec<String>> {
    sh.sources.iter().map(|s| s.values.clone()).collect()
}

fn resolved_values(p: &Planned) -> Vec<Vec<String>> {
    p.resolved
        .iter()
        .map(|v| match v {
            SourceValue::One(x) => vec![x.to_string()],
            SourceValue::Group(xs) => xs.iter().map(|x| x.to_string()).collect(),
        })
        .collect()
}

/// The oracle for source `i` holding `raw` (`run_plans.measure.oracle`).
fn oracle(sh: &Shape, i: usize, raw: &str, extra: &[(String, String)]) -> Option<Out> {
    let s = &sh.sources[i];
    let chans = &channel_table()[&s.key].channels;
    let values = values_of(sh);
    let mut envx: Vec<(String, String)> = extra.to_vec();
    if chans.iter().any(|c| c.kind == ChannelKind::EnvRef) {
        envx.push(("USER_SECRET".into(), raw.into()));
        return Some(run_oracle(
            &baseline_argv(sh, &values, Some((i, "USER_SECRET")), None, &[]),
            &envx,
            None,
        ));
    }
    if !(raw.starts_with('-') || raw.starts_with("@env:")) {
        let mut vv = values.clone();
        vv[i] = vec![raw.to_string()];
        return Some(run_oracle(
            &baseline_argv(sh, &vv, None, None, &[]),
            &envx,
            None,
        ));
    }
    let tog = chans
        .iter()
        .find(|c| c.kind == ChannelKind::StdinToggle && c.terminator.is_some());
    if let (Some(t), "value") = (tog, s.form.as_str()) {
        return Some(run_oracle(
            &baseline_argv(
                sh,
                &values,
                None,
                Some((i, t.flag.as_deref().unwrap())),
                &[],
            ),
            &envx,
            Some(&format!("{raw}\r\n")),
        ));
    }
    None
}

fn plan(
    sh: &Shape,
    srcs: &[mnemonic_gui::form::channels::PlanSource],
    os: &str,
    env: &[(&str, &str)],
) -> Result<Planned, &'static str> {
    plan_sources(
        srcs,
        channel_table(),
        policy(),
        os,
        &env_of(env),
        PlanOptions::default(),
    )
    .map_err(|e| {
        let _ = sh;
        e.code
    })
}

fn pw_for(sh: &Shape, i: usize, v: &str) -> String {
    if sh.name.starts_with("slip39 split") && i == 1 {
        v.to_string()
    } else {
        PW.to_string()
    }
}

/// Every leg of one shape; returns failure descriptions.
fn measure(sh: &Shape) -> (Vec<String>, HashMap<&'static str, usize>) {
    let mut bad = Vec::new();
    let mut count: HashMap<&'static str, usize> = HashMap::new();
    let values = values_of(sh);
    let linux = match plan(sh, &sh.plan_sources(), "linux", &[]) {
        Ok(p) => p,
        Err(_) => return (bad, count), // a refusal is always safe
    };
    let base = run_oracle(&baseline_argv(sh, &values, None, None, &[]), &[], None);
    // baseline
    let got = run_gui(sh, &linux, &values, &[]);
    if !same(sh, &base, &got, PW) {
        bad.push(format!(
            "{}: planned run != baseline ({:?} vs {:?})",
            sh.name, got.code, base.code
        ));
    }
    *count.entry("baseline").or_default() += 1;
    // swap
    for i in 0..values.len() {
        for j in i + 1..values.len() {
            let mut sw = values.clone();
            sw.swap(i, j);
            let mut p = linux.clone();
            p.resolved.swap(i, j);
            let got = run_gui(sh, &p, &sw, &[]);
            let expect_same = sh.symmetric.contains(&(i, j));
            if same(sh, &got, &base, PW) != expect_same {
                bad.push(format!(
                    "{}: swap {i}<->{j} same_as_baseline={} expected {expect_same}",
                    sh.name, !expect_same
                ));
            }
            *count.entry("swap").or_default() += 1;
        }
    }
    for (i, s) in sh.sources.iter().enumerate() {
        if s.is_group() {
            continue;
        }
        // NI1 (Linux) and interim (NI3), over the whole corpus
        for var in &corpus().variants {
            let raw = var.apply(s.value());
            let srcs = sh.with_value(i, "@env:USER_SECRET");
            for os in ["linux", "macos"] {
                let Ok(p) = plan(sh, &srcs, os, &[("USER_SECRET", &raw)]) else {
                    *count.entry("refused").or_default() += 1;
                    continue;
                };
                let rv = resolved_values(&p);
                // the interim plan runs on Linux too (the OS is only the policy key)
                let got = run_gui(sh, &p, &rv, &[]);
                let orc = oracle(sh, i, &raw, &[]);
                let pw = pw_for(sh, i, &rv[i][0]);
                if let Some(o) = &orc {
                    if !same(sh, &got, o, &pw) {
                        bad.push(format!("{} src{i} {os} {raw:?}: != oracle", sh.name));
                    }
                }
                *count
                    .entry(if os == "linux" { "ni1" } else { "interim" })
                    .or_default() += 1;
            }
        }
        // NC1: the variable holds each lookalike
        for tmpl in &corpus().channel_variants {
            let content = tmpl.replace("{V}", "OTHER");
            let srcs = sh.with_value(i, "@env:USER_SECRET");
            let orc = oracle(sh, i, &content, &[("OTHER".into(), s.value().into())]);
            for os in ["linux", "macos"] {
                match plan(
                    sh,
                    &srcs,
                    os,
                    &[("USER_SECRET", &content), ("OTHER", s.value())],
                ) {
                    Err("value-looks-like-a-channel") => {
                        *count.entry("nc1-refused").or_default() += 1
                    }
                    Err(c) => bad.push(format!(
                        "{} src{i} {os} {content:?}: refused {c}, not by the lookalike predicate",
                        sh.name
                    )),
                    Ok(p) => {
                        let rv = resolved_values(&p);
                        let inh = vec![
                            ("OTHER".to_string(), s.value().to_string()),
                            ("USER_SECRET".to_string(), content.clone()),
                        ];
                        let got = run_gui(sh, &p, &rv, &inh);
                        let other = ok(&got) && same(sh, &got, &base, PW);
                        bad.push(format!(
                            "{} src{i} {os} {content:?}: RAN (other's wallet: {other}; == oracle: {:?})",
                            sh.name,
                            orc.as_ref().map(|o| same(sh, &got, o, &pw_for(sh, i, &content)))
                        ));
                    }
                }
            }
        }
    }
    // Nm13: typed leading-dash values, value-form sources
    for (i, s) in sh.sources.iter().enumerate() {
        if s.form != "value" {
            continue;
        }
        for v in ["-leading", "--help", "-lead  "] {
            let srcs = sh.with_value(i, v);
            let orc = oracle(sh, i, v, &[]);
            let mut vv = values.clone();
            vv[i] = vec![v.to_string()];
            let eqform = run_oracle(&baseline_argv(sh, &vv, None, None, &[i]), &[], None);
            for os in ["linux", "macos"] {
                let Ok(p) = plan(sh, &srcs, os, &[]) else {
                    *count.entry("dash-refused").or_default() += 1;
                    continue;
                };
                let rv = resolved_values(&p);
                let got = run_gui(sh, &p, &rv, &[]);
                match &orc {
                    None => {
                        if ok(&got) && !same(sh, &got, &eqform, PW) {
                            bad.push(format!(
                                "{} src{i} {os} {v:?}: no oracle, ran and != --flag=VALUE",
                                sh.name
                            ));
                        }
                    }
                    Some(o) => {
                        if !same(sh, &got, o, &pw_for(sh, i, v)) {
                            bad.push(format!("{} src{i} {os} {v:?}: != oracle", sh.name));
                        }
                    }
                }
                *count.entry("dash").or_default() += 1;
            }
        }
    }
    (bad, count)
}

#[test]
fn t3prime_every_admitted_plan_matches_its_independent_oracle() {
    let shapes = shapes();
    let results: Vec<(Vec<String>, HashMap<&'static str, usize>)> = std::thread::scope(|s| {
        let hs: Vec<_> = shapes
            .iter()
            .map(|sh| s.spawn(move || measure(sh)))
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut total: HashMap<&'static str, usize> = HashMap::new();
    let mut bad = Vec::new();
    for (b, c) in results {
        bad.extend(b);
        for (k, v) in c {
            *total.entry(k).or_default() += v;
        }
    }
    eprintln!("T3′ legs: {total:?}");
    assert!(
        bad.is_empty(),
        "T3′ FAILURES ({}):\n{}",
        bad.len(),
        bad.join("\n")
    );
    // non-vacuity: every leg ran
    for (k, min) in [
        ("baseline", 25),
        ("swap", 30),
        ("ni1", 400),
        ("interim", 400),
        ("nc1-refused", 2000),
        ("dash", 100),
    ] {
        assert!(
            total.get(k).copied().unwrap_or(0) >= min,
            "leg {k} ran only {:?} times",
            total.get(k)
        );
    }
}
