//! T2 / T2b — single-input real-binary equivalence and byte fidelity (DESIGN
//! §A9), against the PINNED release binaries (`MNEMONIC_BIN` required — never
//! a skip; T10 names this target).
//!
//! For every input in `channel_table.json` and every measured channel cell,
//! the GUI's own `materialize` + `runner::run_plan` deliver the secret through
//! that cell, and the result must equal the argv baseline carrying the SAME
//! bytes (`--allow-argv-secret`, verbatim) — the independent oracle:
//!
//! - **T2:** the fixture value, with DEPENDENCE (a different secret changes the
//!   baseline, so equality is not vacuous);
//! - **T2b:** every corpus variant (`corpus.VARIANTS`) with the cell's measured
//!   terminator; a LENIENT cell (terminator null) carries only a clean value.
//!
//! The cases (argv templates, fixture values) are the measurement's own
//! (`cases3.py`), read through python — one copy.

mod secret_channels_common;

use std::collections::BTreeMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use mnemonic_gui::form::channels::{
    channel_table, materialize, Assembled, Binding, Channel, ChannelKind, Planned, Provenance,
    SourceForm, SourceSite, SourceValue,
};
use secret_channels_common::{bin_dir, corpus, python_json};

#[derive(Clone, Debug)]
struct Case {
    label: String,
    argv: Vec<String>,
    mode: String,
    s: String,
    c: String,
    multi: bool,
    has_check: bool,
}

fn cases() -> Vec<Case> {
    let b = bin_dir();
    let j = python_json(
        "import json\nfrom cases3 import C, C2, C3\nd={}\nfor c in C+C2+C3: d[c['label']]=c\n\
         print(json.dumps([{'label':l,'argv':c['argv'],'mode':c['mode'],'S':c['S'],'C':c['C'],\
         'multi':bool(c.get('multi')),'has_check':'check' in c} for l,c in d.items()]))",
        &[("BIN_DIR", b.to_str().unwrap())],
    );
    j.as_array()
        .unwrap()
        .iter()
        .map(|c| Case {
            label: c["label"].as_str().unwrap().into(),
            argv: c["argv"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().into())
                .collect(),
            mode: c["mode"].as_str().unwrap().into(),
            s: c["S"].as_str().unwrap().into(),
            c: c["C"].as_str().map(String::from).unwrap_or_default(),
            multi: c["multi"].as_bool().unwrap(),
            has_check: c["has_check"].as_bool().unwrap(),
        })
        .collect()
}

fn norm_key(label: &str) -> String {
    let re = regex::Regex::new(r"@\d+\.").unwrap();
    re.replace_all(label, "@N.").into_owned()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Out {
    code: Option<i32>,
    stdout: String,
}

fn run_argv(argv: &[String], stdin: Option<&str>) -> Out {
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut ch = cmd.spawn().expect("spawn");
    {
        let mut h = ch.stdin.take().unwrap();
        let _ = h.write_all(stdin.unwrap_or("").as_bytes());
    }
    let o = ch.wait_with_output().unwrap();
    Out {
        code: o.status.code(),
        stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
    }
}

fn bin(cli: &str) -> String {
    bin_dir().join(cli).to_string_lossy().into_owned()
}

/// The measurement's `check` normalisers for randomised outputs, by label.
/// A case with a check and no normaliser here fails the test (no silent gap).
fn normalise(label: &str, value: &str, o: &Out) -> String {
    if !matches!(o.code, Some(0) | Some(4)) {
        return String::new();
    }
    let lines = |pred: &dyn Fn(&str) -> bool| -> Vec<String> {
        o.stdout
            .lines()
            .filter(|l| pred(l))
            .map(|l| l.to_string())
            .take(2)
            .collect()
    };
    match label {
        "mnemonic slip39 split --from phrase="
        | "mnemonic slip39 split --from entropy="
        | "mnemonic slip39 split --passphrase" => {
            let sh = lines(&|l: &str| !l.trim().is_empty() && !l.starts_with('#'));
            if sh.len() < 2 {
                return o.stdout.clone();
            }
            let mut a = vec![
                bin("mnemonic"),
                "slip39".into(),
                "combine".into(),
                "--allow-argv-secret".into(),
            ];
            if label.ends_with("--passphrase") {
                a.push("--passphrase".into());
                a.push(value.into());
            }
            for s in sh {
                a.push("--share".into());
                a.push(s);
            }
            run_argv(&a, None).stdout
        }
        "mnemonic ms-shares split --from phrase=" | "mnemonic ms-shares split --from entropy=" => {
            let sh: Vec<String> = o
                .stdout
                .lines()
                .map(str::trim)
                .filter(|l| l.starts_with("ms1"))
                .take(2)
                .map(String::from)
                .collect();
            if sh.len() < 2 {
                return o.stdout.clone();
            }
            let a = vec![
                bin("mnemonic"),
                "ms-shares".into(),
                "combine".into(),
                "--allow-argv-secret".into(),
                "--share".into(),
                sh[0].clone(),
                "--share".into(),
                sh[1].clone(),
                "--to".into(),
                "phrase".into(),
            ];
            run_argv(&a, None).stdout
        }
        "ms split --phrase" | "ms split --hex" => {
            let sh: Vec<String> = o
                .stdout
                .lines()
                .filter(|l| l.starts_with("ms1"))
                .take(2)
                .map(|l| l.replace(' ', ""))
                .collect();
            if sh.len() < 2 {
                return o.stdout.clone();
            }
            let mut a = vec![
                bin("ms"),
                "combine".into(),
                "--allow-argv-secret".into(),
                "--".into(),
            ];
            a.extend(sh);
            run_argv(&a, None).stdout
        }
        _ => o.stdout.clone(),
    }
}

const NORMALISED: &[&str] = &[
    "mnemonic slip39 split --from phrase=",
    "mnemonic slip39 split --from entropy=",
    "mnemonic slip39 split --passphrase",
    "mnemonic ms-shares split --from phrase=",
    "mnemonic ms-shares split --from entropy=",
    "ms split --phrase",
    "ms split --hex",
];

fn result(label: &str, value: &str, o: &Out) -> (Option<i32>, String) {
    (o.code, normalise(label, value, o))
}

/// The case argv with `{S}` → `value`, the opt-in dropped unless other
/// secrets ride argv (`multi`), and the source's site.
fn assembled(case: &Case, value: &str) -> Assembled {
    let mut argv: Vec<String> = case.argv.clone();
    if !case.multi {
        argv.retain(|a| a != "--allow-argv-secret");
    }
    let i = argv.iter().position(|a| a.contains("{S}")).unwrap();
    let tok = argv[i].clone();
    let (form, flag_at) = match case.mode.as_str() {
        "node" => (
            SourceForm::Node {
                prefix: tok.split("{S}").next().unwrap().to_string(),
            },
            Some(i - 1),
        ),
        "value" => (SourceForm::Value, Some(i - 1)),
        _ => (SourceForm::Pos, None),
    };
    argv[i] = tok.replace("{S}", value);
    let eoo_at = if flag_at.is_none() {
        argv[..i].iter().rposition(|a| a == "--")
    } else {
        argv.iter().position(|a| a == "--")
    };
    Assembled {
        argv,
        sub_tokens: 1,
        eoo_at,
        sites: vec![SourceSite {
            key: norm_key(&case.label),
            form,
            flag_at,
            value_at: vec![i],
        }],
        declares_allow_argv_secret: false,
    }
}

/// Deliver `value` through `cell` with the GUI's own materialize + runner.
fn via_cell(case: &Case, cell: &Channel, value: &str) -> Out {
    let b = Binding {
        source: 0,
        key: norm_key(&case.label),
        kind: cell.kind,
        terminator: cell.terminator.clone().unwrap_or_default(),
        flag: cell.flag.clone(),
        env: (cell.kind == ChannelKind::EnvRef).then(|| "MNEMONIC_GUI_S0".to_string()),
        fd: cell.kind.is_fd().then_some(3),
        argv_form: None,
    };
    let planned = Planned {
        bindings: vec![b],
        provenance: vec![vec![Provenance::Typed]],
        resolved: vec![SourceValue::one(value)],
    };
    let rp = materialize(&assembled(case, value), &planned);
    let r = mnemonic_gui::runner::run_plan(&rp).expect("runner");
    Out {
        code: r.exit_code,
        stdout: String::from_utf8_lossy(&r.stdout).into_owned(),
    }
}

fn exact(case: &Case, value: &str) -> Out {
    run_argv(
        &case
            .argv
            .iter()
            .map(|a| a.replace("{S}", value))
            .collect::<Vec<_>>(),
        None,
    )
}

#[test]
fn t2_every_cell_equals_the_argv_baseline_with_dependence_and_byte_fidelity() {
    let table = channel_table();
    let cases = cases();
    for c in &cases {
        assert!(
            !c.has_check || NORMALISED.contains(&c.label.as_str()),
            "case {} has a check with no Rust normaliser",
            c.label
        );
    }
    let by_key: BTreeMap<String, &Case> = cases.iter().map(|c| (norm_key(&c.label), c)).collect();
    let keys: Vec<&String> = table
        .keys()
        .filter(|k| *k != "ms combine <shares>")
        .collect();
    for k in &keys {
        assert!(
            by_key.contains_key(*k),
            "table input {k} has no measurement case"
        );
    }
    let next = AtomicUsize::new(0);
    let bad = Mutex::new(Vec::<String>::new());
    let cells_run = AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..16 {
            s.spawn(|| loop {
                let n = next.fetch_add(1, Ordering::SeqCst);
                let Some(key) = keys.get(n) else { break };
                let case = by_key[*key];
                let entry = &table[*key];
                // T2: dependence, then every cell == baseline
                let base = exact(case, &case.s);
                let other = exact(case, &case.c);
                // dependence normalises BOTH with the fixture value (as
                // cases.py's check does): slip39 split+combine under the SAME
                // passphrase recovers the same seed whatever it is.
                if result(&case.label, &case.s, &base) == result(&case.label, &case.s, &other) {
                    bad.lock()
                        .unwrap()
                        .push(format!("{key}: baseline does not depend on the secret"));
                }
                if !matches!(base.code, Some(0) | Some(4)) {
                    bad.lock()
                        .unwrap()
                        .push(format!("{key}: baseline exit {:?}", base.code));
                }
                for cell in &entry.channels {
                    let got = via_cell(case, cell, &case.s);
                    if result(&case.label, &case.s, &got) != result(&case.label, &case.s, &base) {
                        bad.lock().unwrap().push(format!(
                            "{key} {:?}: != baseline (exit {:?})",
                            cell.kind, got.code
                        ));
                    }
                    cells_run.fetch_add(1, Ordering::SeqCst);
                }
                // T2b: every corpus variant, with the measured terminator
                for var in &corpus().variants {
                    let v = var.apply(&case.s);
                    let want = exact(case, &v);
                    for cell in &entry.channels {
                        if cell.terminator.is_none() {
                            continue; // lenient: clean values only (T2 above)
                        }
                        let got = via_cell(case, cell, &v);
                        if result(&case.label, &v, &got) != result(&case.label, &v, &want) {
                            bad.lock()
                                .unwrap()
                                .push(format!("{key} {:?} {v:?}: != argv-exact", cell.kind));
                        }
                    }
                }
            });
        }
    });
    let bad = bad.into_inner().unwrap();
    assert!(
        bad.is_empty(),
        "T2 failures ({}):\n{}",
        bad.len(),
        bad.join("\n")
    );
    let n = cells_run.load(Ordering::SeqCst);
    assert!(n >= 140, "only {n} cells ran");
}

#[test]
fn t2_ms_combine_share_group_cells_equal_the_baseline() {
    let sh = [
        "ms12ec7pqsf34k729gl6yrsfk9q244kzsx62s9krxwywxv4jav",
        "ms12ec7pp3txgcymeun2ws3tceqmggc83f2m3l2cj299w9l9vr",
    ];
    let other = [
        "ms12wah2q74tl23u0cm5dzxvrkdx67hu7cj2qm9xglv9ajf4vr",
        "ms12wah2pymqlh20wp3nft97u5f9xgr0gp6hpthlstfqk23dmh",
    ];
    let base_argv = |s: &[&str]| {
        let mut a = vec![
            bin("ms"),
            "combine".into(),
            "--allow-argv-secret".into(),
            "--".into(),
        ];
        a.extend(s.iter().map(|x| x.to_string()));
        a
    };
    let base = run_argv(&base_argv(&sh), None);
    assert_eq!(base.code, Some(0));
    assert_ne!(
        base.stdout,
        run_argv(&base_argv(&other), None).stdout,
        "dependence"
    );
    let entry = &channel_table()["ms combine <shares>"];
    assert!(!entry.channels.is_empty());
    for cell in &entry.channels {
        let argv = vec![
            bin("ms"),
            "combine".into(),
            "--".into(),
            sh[0].into(),
            sh[1].into(),
        ];
        let asm = Assembled {
            argv,
            sub_tokens: 1,
            eoo_at: Some(2),
            sites: vec![SourceSite {
                key: "ms combine <shares>".into(),
                form: SourceForm::Group,
                flag_at: None,
                value_at: vec![3, 4],
            }],
            declares_allow_argv_secret: true,
        };
        let planned = Planned {
            bindings: vec![Binding {
                source: 0,
                key: "ms combine <shares>".into(),
                kind: cell.kind,
                terminator: cell.terminator.clone().unwrap_or_default(),
                flag: cell.flag.clone(),
                env: None,
                fd: cell.kind.is_fd().then_some(3),
                argv_form: None,
            }],
            provenance: vec![vec![Provenance::Typed, Provenance::Typed]],
            resolved: vec![SourceValue::group(sh)],
        };
        let rp = materialize(&asm, &planned);
        assert!(
            !rp.argv.iter().any(|t| sh.contains(&t.as_str())),
            "a share on argv: {:?}",
            rp.argv
        );
        let r = mnemonic_gui::runner::run_plan(&rp).unwrap();
        assert_eq!(
            (r.exit_code, String::from_utf8_lossy(&r.stdout).into_owned()),
            (base.code, base.stdout.clone()),
            "{:?}",
            cell.kind
        );
    }
}
