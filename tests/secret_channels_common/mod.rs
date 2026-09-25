//! Shared helpers for the secret-channel tests (DESIGN §A9).
//!
//! The SHAPES (`shapes.py`) and the value CORPUS (`corpus.py`) exist once, in
//! the measurement folder; these helpers read them through `python3` rather
//! than keeping a second copy here. `python3` is required: a missing
//! interpreter is a FAILURE, never a skip.
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use mnemonic_gui::form::channels::{Assembled, PlanSource, SourceForm, SourceSite, SourceValue};

pub fn measurement_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("design/measurements/secret-channels")
}

/// Run a python snippet in the measurement folder and parse its stdout as JSON.
pub fn python_json(code: &str, extra_env: &[(&str, &str)]) -> serde_json::Value {
    let mut cmd = Command::new("python3");
    cmd.arg("-c").arg(code).current_dir(measurement_dir());
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    let out = cmd
        .output()
        .expect("python3 is required by the secret-channel tests (not a skip)");
    assert!(
        out.status.success(),
        "python3 failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("python emitted JSON")
}

#[derive(Clone, Debug)]
pub struct ShapeSource {
    pub key: String,
    pub form: String,
    pub flag: Option<String>,
    pub prefix: String,
    /// One value, or a group's values.
    pub values: Vec<String>,
}

impl ShapeSource {
    pub fn is_group(&self) -> bool {
        self.form == "group"
    }

    pub fn value(&self) -> &str {
        &self.values[0]
    }

    pub fn plan_source(&self) -> PlanSource {
        PlanSource {
            key: self.key.clone(),
            form: match self.form.as_str() {
                "node" => SourceForm::Node {
                    prefix: self.prefix.clone(),
                },
                "value" => SourceForm::Value,
                "pos" => SourceForm::Pos,
                "group" => SourceForm::Group,
                other => panic!("unknown form {other}"),
            },
            value: if self.is_group() {
                SourceValue::group(self.values.clone())
            } else {
                SourceValue::one(self.values[0].clone())
            },
        }
    }

    pub fn with_value(&self, v: &str) -> ShapeSource {
        let mut s = self.clone();
        if s.is_group() {
            s.values[0] = v.to_string();
        } else {
            s.values = vec![v.to_string()];
        }
        s
    }
}

#[derive(Clone, Debug)]
pub struct Shape {
    pub name: String,
    pub cli: String,
    pub sub: Vec<String>,
    pub pre: Vec<String>,
    pub sources: Vec<ShapeSource>,
    pub post: Vec<String>,
    pub symmetric: Vec<(usize, usize)>,
    pub expect: String,
}

impl Shape {
    pub fn plan_sources(&self) -> Vec<PlanSource> {
        self.sources.iter().map(|s| s.plan_source()).collect()
    }

    pub fn with_value(&self, i: usize, v: &str) -> Vec<PlanSource> {
        self.sources
            .iter()
            .enumerate()
            .map(|(k, s)| if k == i { s.with_value(v) } else { s.clone() }.plan_source())
            .collect()
    }
}

fn strs(v: &serde_json::Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

/// `shapes.SHAPES`, read from the reference.
pub fn shapes() -> &'static Vec<Shape> {
    static S: OnceLock<Vec<Shape>> = OnceLock::new();
    S.get_or_init(|| {
        let j = python_json(
            "import json, shapes; print(json.dumps([{k: s[k] for k in ('name','cli','sub','pre',\
             'sources','post','symmetric','expect')} for s in shapes.SHAPES]))",
            &[],
        );
        j.as_array()
            .unwrap()
            .iter()
            .map(|s| Shape {
                name: s["name"].as_str().unwrap().into(),
                cli: s["cli"].as_str().unwrap().into(),
                sub: strs(&s["sub"]),
                pre: strs(&s["pre"]),
                post: strs(&s["post"]),
                expect: s["expect"].as_str().unwrap().into(),
                symmetric: s["symmetric"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| {
                        (
                            p[0].as_u64().unwrap() as usize,
                            p[1].as_u64().unwrap() as usize,
                        )
                    })
                    .collect(),
                sources: s["sources"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| ShapeSource {
                        key: x["key"].as_str().unwrap().into(),
                        form: x["form"].as_str().unwrap().into(),
                        flag: x["flag"].as_str().map(String::from),
                        prefix: x
                            .get("prefix")
                            .and_then(|p| p.as_str())
                            .unwrap_or("")
                            .into(),
                        values: match &x["value"] {
                            serde_json::Value::String(v) => vec![v.clone()],
                            a => strs(a),
                        },
                    })
                    .collect(),
            })
            .collect()
    })
}

/// A value variant from `corpus.VARIANTS`: (kind, text), kind `suffix`/`prefix`.
#[derive(Clone, Debug)]
pub struct Variant {
    pub kind: String,
    pub text: String,
}

impl Variant {
    pub fn apply(&self, v: &str) -> String {
        if self.kind == "suffix" {
            format!("{v}{}", self.text)
        } else {
            format!("{}{v}", self.text)
        }
    }
}

pub struct Corpus {
    pub variants: Vec<Variant>,
    /// `corpus.CHANNEL_VARIANTS`, `{V}` unexpanded.
    pub channel_variants: Vec<String>,
}

pub fn corpus() -> &'static Corpus {
    static C: OnceLock<Corpus> = OnceLock::new();
    C.get_or_init(|| {
        let j = python_json(
            "import json, corpus; print(json.dumps({'v': corpus.VARIANTS, 'c': corpus.CHANNEL_VARIANTS}))",
            &[],
        );
        Corpus {
            variants: j["v"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| Variant {
                    kind: p[0].as_str().unwrap().into(),
                    text: p[1].as_str().unwrap().into(),
                })
                .collect(),
            channel_variants: strs(&j["c"]),
        }
    })
}

/// A user environment from pairs.
pub fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let m: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    move |k: &str| m.get(k).cloned()
}

pub const PLATFORMS: [&str; 3] = ["linux", "macos", "windows"];

/// The pinned-binary directory for the real-binary legs: `$MNEMONIC_BIN`'s
/// directory when it is a path, else the directory holding `mnemonic` on
/// `$PATH`. REQUIRED (never a skip): these tests are the gate T10 names.
pub fn bin_dir() -> PathBuf {
    let m = std::env::var("MNEMONIC_BIN").unwrap_or_else(|_| {
        panic!("MNEMONIC_BIN must be set for the secret-channel real-binary tests")
    });
    let p = PathBuf::from(&m);
    let resolved = if p.components().count() > 1 {
        p
    } else {
        let path = std::env::var_os("PATH").expect("PATH");
        std::env::split_paths(&path)
            .map(|d| d.join(&m))
            .find(|c| c.is_file())
            .unwrap_or_else(|| panic!("{m} not found on PATH"))
    };
    resolved.parent().unwrap().to_path_buf()
}

/// An argv for a shape the way `run_plans.baseline_argv` lays it out (no
/// `--allow-argv-secret`), with its source sites — to drive `materialize`.
pub fn shape_assembled(sh: &Shape, values: &[Vec<String>]) -> Assembled {
    let mut argv = vec![sh.cli.clone()];
    argv.extend(sh.sub.iter().cloned());
    argv.extend(sh.pre.iter().cloned());
    let mut sites = Vec::new();
    let mut posit: Vec<(usize, String)> = Vec::new();
    for (i, (s, v)) in sh.sources.iter().zip(values).enumerate() {
        match s.form.as_str() {
            "node" => {
                argv.push(s.flag.clone().unwrap());
                argv.push(format!("{}{}", s.prefix, v[0]));
                sites.push(SourceSite {
                    key: s.key.clone(),
                    form: SourceForm::Node {
                        prefix: s.prefix.clone(),
                    },
                    flag_at: Some(argv.len() - 2),
                    value_at: vec![argv.len() - 1],
                });
            }
            "value" => {
                argv.push(s.flag.clone().unwrap());
                argv.push(v[0].clone());
                sites.push(SourceSite {
                    key: s.key.clone(),
                    form: SourceForm::Value,
                    flag_at: Some(argv.len() - 2),
                    value_at: vec![argv.len() - 1],
                });
            }
            _ => {
                for x in v {
                    posit.push((i, x.clone()));
                }
                sites.push(SourceSite {
                    key: s.key.clone(),
                    form: if s.is_group() {
                        SourceForm::Group
                    } else {
                        SourceForm::Pos
                    },
                    flag_at: None,
                    value_at: vec![],
                });
            }
        }
    }
    argv.extend(sh.post.iter().cloned());
    let mut eoo_at = None;
    if !posit.is_empty() {
        eoo_at = Some(argv.len());
        argv.push("--".into());
        for (i, x) in posit {
            sites[i].value_at.push(argv.len());
            argv.push(x);
        }
    }
    Assembled {
        argv,
        sub_tokens: sh.sub.len(),
        eoo_at,
        sites,
        declares_allow_argv_secret: true,
    }
}
