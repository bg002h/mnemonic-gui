//! Every secret over a private channel (DESIGN Part A).
//!
//! The planner decides, for each **secret source** the form carries (§A4.1),
//! which channel delivers its bytes to the CLI: a planner-owned
//! `@env:MNEMONIC_GUI_S<i>` variable, the child's stdin (a `--X-stdin` toggle,
//! `--X -`, a positional `-`, or `ms combine`'s one `-` for a share group), or
//! an inherited pipe (`--X-file /dev/fd/N`, `--in /dev/fd/N`). On an OS outside
//! `private_channels_on` it returns the INTERIM plan instead: the resolved
//! bytes on argv with `--allow-argv-secret` (§A6).
//!
//! `plan.py` (`design/measurements/secret-channels/`) is the executable
//! specification. [`plan_sources`] mirrors its `plan()` branch for branch,
//! and T8 (`tests/secret_channels_t8.rs`) pins the two equal on every shape ×
//! OS, bindings AND payload bytes. The data the rule reads is the committed
//! cache in [`data`]; nothing about CLI behaviour lives in this file.
//!
//! Before either path (§A3a, C1): a secret field holding `-` or `@env:VAR`
//! means that channel, never those characters. `@env:VAR` is resolved from
//! the GUI's own environment under this input's measured `cli_env_rule`;
//! `-` refuses (the GUI has no stdin to forward). Then, on EVERY path and OS
//! (§A0.1): a value that looks like a channel refuses, and a value ending in
//! CR or LF refuses. The GUI never falls back to argv (§A8).

pub mod copy;
pub mod data;
pub mod lookalike;

use std::fmt;

use zeroize::{Zeroize, Zeroizing};

pub use data::{
    channel_table, policy, Channel, ChannelKind, ChannelTable, EnvRule, Policy, TableEntry,
};
pub use lookalike::{looks_like_channel, normalize_for_lookalike};

use crate::schema::{FormState, Schema, SubcommandSchema};

// ── Sources ────────────────────────────────────────────────────────────────

/// How a source's value sits in argv (`plan.py`'s `form`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceForm {
    /// `--flag <prefix><value>`: a node-valued Text or composite
    /// (`--from phrase=…`), or a slot row (`--slot @0.phrase=…`).
    Node { prefix: String },
    /// `--flag <value>`.
    Value,
    /// A secret positional (one value).
    Pos,
    /// A repeating secret positional carried as ONE source (`ms combine`'s
    /// shares).
    Group,
}

impl SourceForm {
    fn is_group(&self) -> bool {
        matches!(self, SourceForm::Group)
    }
}

/// A source's text: one value, or a group's values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceValue {
    One(Zeroizing<String>),
    Group(Vec<Zeroizing<String>>),
}

impl SourceValue {
    pub fn one(s: impl Into<String>) -> Self {
        SourceValue::One(Zeroizing::new(s.into()))
    }

    pub fn group<I: IntoIterator<Item = S>, S: Into<String>>(items: I) -> Self {
        SourceValue::Group(
            items
                .into_iter()
                .map(|s| Zeroizing::new(s.into()))
                .collect(),
        )
    }

    /// Every value (a group's members, or the one value).
    pub fn values(&self) -> Vec<&str> {
        match self {
            SourceValue::One(v) => vec![v.as_str()],
            SourceValue::Group(vs) => vs.iter().map(|v| v.as_str()).collect(),
        }
    }

    /// The bytes a channel carries: a group is its members one per line
    /// (`"\n".join`), a single value is itself plus the channel terminator.
    fn channel_data(&self, terminator: &str) -> Zeroizing<String> {
        match self {
            SourceValue::One(v) => Zeroizing::new(format!("{}{}", v.as_str(), terminator)),
            SourceValue::Group(vs) => {
                Zeroizing::new(vs.iter().map(|v| v.as_str()).collect::<Vec<_>>().join("\n"))
            }
        }
    }
}

/// One secret input the user filled (`plan.py`'s source dict).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanSource {
    /// The channel-table key, e.g. `"mnemonic restore --passphrase"`.
    pub key: String,
    pub form: SourceForm,
    pub value: SourceValue,
}

/// Where a value came from, per value (§A3a provenance).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Provenance {
    Typed,
    /// The GUI resolved `@env:<name>` from its own environment.
    Env(String),
}

impl fmt::Display for Provenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provenance::Typed => f.write_str("typed"),
            Provenance::Env(n) => write!(f, "${n}"),
        }
    }
}

// ── Refusals ───────────────────────────────────────────────────────────────

/// A refusal (§A8). The GUI sends nothing; it never falls back to argv.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    /// The stable code (`C1-dash`, `two-stdin`, …).
    pub code: &'static str,
    /// The source (channel-table key) or field it concerns.
    pub source: String,
    /// A human explanation.
    pub why: String,
}

impl Refusal {
    fn new(code: &'static str, source: impl Into<String>, why: impl Into<String>) -> Self {
        Refusal {
            code,
            source: source.into(),
            why: why.into(),
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}: {}", self.code, self.source, self.why)
    }
}

/// The design's name for a refusal (§A4.2 signature).
pub type ChannelRefusal = Refusal;

// ── Bindings ───────────────────────────────────────────────────────────────

/// On the interim path, how a value goes on argv (R3 Nm13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgvForm {
    /// `--flag VALUE` (separate words).
    Sep,
    /// `--flag=VALUE`, only where that form measured byte-exact.
    Eq,
}

/// One source's channel (`plan.py`'s binding dict).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub source: usize,
    pub key: String,
    pub kind: ChannelKind,
    /// The measured terminator appended after the target (`""` for lenient
    /// cells, `EnvRef`, and `Argv`).
    pub terminator: String,
    /// The channel's own flag (`--passphrase-stdin`, `--secret-file`, `--in`).
    pub flag: Option<String>,
    /// `MNEMONIC_GUI_S<i>` for `EnvRef`.
    pub env: Option<String>,
    /// The child fd number for a pipe channel.
    pub fd: Option<i32>,
    /// Interim path only.
    pub argv_form: Option<ArgvForm>,
}

impl Binding {
    /// The same JSON shape as `plans_pure.json`'s bindings (T8).
    pub fn to_json(&self) -> serde_json::Value {
        let mut m = serde_json::Map::new();
        m.insert("source".into(), self.source.into());
        m.insert("key".into(), self.key.clone().into());
        m.insert("kind".into(), format!("{:?}", self.kind).into());
        m.insert("terminator".into(), self.terminator.clone().into());
        if let Some(f) = &self.flag {
            m.insert("flag".into(), f.clone().into());
        }
        if let Some(e) = &self.env {
            m.insert("env".into(), e.clone().into());
        }
        if let Some(fd) = self.fd {
            m.insert("fd".into(), fd.into());
        }
        if let Some(a) = self.argv_form {
            m.insert(
                "argv_form".into(),
                match a {
                    ArgvForm::Sep => "sep",
                    ArgvForm::Eq => "eq",
                }
                .into(),
            );
        }
        serde_json::Value::Object(m)
    }

    /// `plan.describe`'s channel phrase for this binding.
    pub fn channel_phrase(&self) -> String {
        let mut w = match self.kind {
            ChannelKind::EnvRef => format!("env {}", self.env.as_deref().unwrap_or("?")),
            ChannelKind::StdinToggle => {
                format!("stdin via {}", self.flag.as_deref().unwrap_or("?"))
            }
            ChannelKind::DashValue => "stdin via `-`".to_string(),
            ChannelKind::PosDash => "stdin via positional `-`".to_string(),
            ChannelKind::StdinMulti => "stdin via one `-` (all, one per line)".to_string(),
            ChannelKind::Argv => "argv + --allow-argv-secret (interim)".to_string(),
            ChannelKind::FileFlag => {
                format!("pipe fd via {}", self.flag.as_deref().unwrap_or("?"))
            }
            ChannelKind::InFile => "pipe fd via --in".to_string(),
        };
        if !self.terminator.is_empty() && self.kind != ChannelKind::StdinMulti {
            w.push_str(&format!(" + {}", py_repr(&self.terminator)));
        }
        w
    }

    /// The label `plan.describe` gives the source: the key minus its CLI and
    /// first subcommand token.
    pub fn source_label(&self) -> String {
        self.key
            .splitn(3, ' ')
            .last()
            .unwrap_or(&self.key)
            .to_string()
    }
}

/// Python `repr` of a short terminator string, as `plan.describe` prints it.
fn py_repr(s: &str) -> String {
    let mut out = String::from("'");
    for c in s.chars() {
        match c {
            '\r' => out.push_str("\\r"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\'' => out.push_str("\\'"),
            '\\' => out.push_str("\\\\"),
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}

/// `plan.describe`: one `<source> ← <channel>` per binding.
pub fn describe(bindings: &[Binding]) -> String {
    bindings
        .iter()
        .map(|b| format!("{} ← {}", b.source_label(), b.channel_phrase()))
        .collect::<Vec<_>>()
        .join("; ")
}

/// A plan before it is turned into an invocation: bindings, per-source
/// provenance, and the RESOLVED source values (the target bytes).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Planned {
    pub bindings: Vec<Binding>,
    pub provenance: Vec<Vec<Provenance>>,
    pub resolved: Vec<SourceValue>,
}

/// Test hooks for the pure planner.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlanOptions {
    /// Resolve every `@env:` under this rule instead of the input's measured
    /// `cli_env_rule` (`plan.py`'s `rule=` argument).
    pub rule_override: Option<EnvRule>,
}

// ── C1 (§A3a) ──────────────────────────────────────────────────────────────

/// `plan.resolve`: C1, on every OS, before either Run path.
pub fn resolve(
    sources: &[PlanSource],
    user_env: &dyn Fn(&str) -> Option<String>,
    table: Option<&ChannelTable>,
    policy: &Policy,
    opts: PlanOptions,
) -> Result<(Vec<PlanSource>, Vec<Vec<Provenance>>), Refusal> {
    let name_rule = regex::Regex::new(&policy.env_name_rule).expect("env_name_rule is a regex");
    let prefix = policy.reserved_env_prefix.as_str();
    let mut out = Vec::with_capacity(sources.len());
    let mut prov = Vec::with_capacity(sources.len());
    for s in sources {
        let mut got: Vec<Zeroizing<String>> = Vec::new();
        let mut where_: Vec<Provenance> = Vec::new();
        for v in s.value.values() {
            if v == "-" {
                return Err(Refusal::new(
                    "C1-dash",
                    &s.key,
                    "the GUI has no stdin of its own to forward; type the value, or use @env:VAR",
                ));
            }
            if let Some(name) = v.strip_prefix("@env:") {
                if name.starts_with(prefix) {
                    return Err(Refusal::new(
                        "C1-reserved-name",
                        &s.key,
                        format!("{prefix}* names are the GUI's own"),
                    ));
                }
                if !name_rule.is_match(name) {
                    return Err(Refusal::new(
                        "C1-bad-name",
                        &s.key,
                        format!("{name:?} is not a valid name ([A-Z_][A-Z0-9_]*)"),
                    ));
                }
                let Some(raw) = user_env(name).map(Zeroizing::new) else {
                    return Err(Refusal::new(
                        "C1-env-unset",
                        &s.key,
                        format!("${name} is not set in the GUI's environment"),
                    ));
                };
                let rule = match opts.rule_override {
                    Some(r) => Some(r),
                    None => table
                        .and_then(|t| t.get(&s.key))
                        .and_then(|e| e.cli_env_rule),
                };
                if rule == Some(EnvRule::Unknown) {
                    return Err(Refusal::new(
                        "C1-env-rule-unknown",
                        &s.key,
                        "the CLI's @env: rule for this input did not measure as any known rule",
                    ));
                }
                let target = Zeroizing::new(rule.unwrap_or(EnvRule::Verbatim).apply(&raw));
                // Judged on the TARGET, after the rule (R2 Nit 1).
                if target.is_empty() {
                    return Err(Refusal::new(
                        "C1-env-empty",
                        &s.key,
                        format!("${name} is empty (after the CLI's @env: rule)"),
                    ));
                }
                got.push(target);
                where_.push(Provenance::Env(name.to_string()));
            } else {
                got.push(Zeroizing::new(v.to_string()));
                where_.push(Provenance::Typed);
            }
        }
        let value = if s.form.is_group() {
            SourceValue::Group(got)
        } else {
            SourceValue::One(got.into_iter().next().unwrap_or_default())
        };
        out.push(PlanSource {
            key: s.key.clone(),
            form: s.form.clone(),
            value,
        });
        prov.push(where_);
    }
    Ok((out, prov))
}

/// `plan.guard_passthrough` (R1 Nm1): no user-typed token in ANY field may
/// name the GUI's reserved variables.
pub fn guard_passthrough<'a, I: IntoIterator<Item = &'a str>>(
    tokens: I,
    policy: &Policy,
) -> Result<(), Refusal> {
    let needle = format!("@env:{}", policy.reserved_env_prefix);
    for t in tokens {
        if t.contains(&needle) {
            let field = t
                .split("@env:")
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or("field");
            return Err(Refusal::new(
                "C1-reserved-name",
                field,
                "reserved @env: name in a pass-through field",
            ));
        }
    }
    Ok(())
}

/// A value a LENIENT channel (terminator null) carries exactly: no CR/LF and
/// no edge whitespace (Python `str.strip()` whitespace).
fn is_clean(v: &str) -> bool {
    !v.contains('\r') && !v.contains('\n') && v == py_strip(v)
}

/// Python `str.strip()`: strips characters for which `str.isspace()` holds
/// (Unicode White_Space plus U+001C..U+001F).
fn py_strip(v: &str) -> &str {
    let ws = |c: char| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c);
    v.trim_matches(ws)
}

// ── §A4.3: the rule ─────────────────────────────────────────────────────────

/// `plan.plan`: resolve (C1), refuse on every path what §A0.1 refuses, then
/// the interim plan on an OS outside `private_channels_on`, else the private
/// channels.
pub fn plan_sources(
    sources: &[PlanSource],
    table: &ChannelTable,
    policy: &Policy,
    os: &str,
    user_env: &dyn Fn(&str) -> Option<String>,
    opts: PlanOptions,
) -> Result<Planned, Refusal> {
    let (res, prov) = resolve(sources, user_env, Some(table), policy, opts)?;
    for s in &res {
        if !table.contains_key(&s.key) {
            return Err(Refusal::new("no-table-entry", &s.key, "input not measured"));
        }
        let vals = s.value.values();
        // R2 Nit 1: argv and env cannot carry NUL; one message on every OS.
        if vals.iter().any(|v| v.contains('\0')) {
            return Err(Refusal::new(
                "nul-in-value",
                &s.key,
                "the value contains a NUL byte",
            ));
        }
        // R4 NI8 / NI9: decisions that make delivery independent of CLI
        // behaviour outside any probe set.
        for v in vals {
            if looks_like_channel(policy, v) {
                return Err(Refusal::new(
                    "value-looks-like-a-channel",
                    &s.key,
                    "after trimming and case-folding this value reads like `-` or `@env…`, which a \
                     CLI may treat as a channel; nobody wants that as a secret",
                ));
            }
            if policy.refuse_trailing_cr_lf && (v.ends_with('\r') || v.ends_with('\n')) {
                return Err(Refusal::new(
                    "value-ends-in-newline",
                    &s.key,
                    "the value ends in a newline or CR (check how the variable was set); CLIs \
                     differ in how many they strip, so the GUI does not send it",
                ));
            }
        }
    }
    if !policy.private_channels_on.iter().any(|o| o == os) {
        // INTERIM path (§A6): resolved bytes on argv. R3 Nm13: a value
        // starting with `-` goes as `--flag=VALUE` where that form MEASURED
        // byte-exact for this input; otherwise it refuses.
        let mut bindings = Vec::with_capacity(res.len());
        for (i, s) in res.iter().enumerate() {
            let dash = matches!(s.form, SourceForm::Value)
                && matches!(&s.value, SourceValue::One(v) if v.starts_with('-'));
            if dash && table[&s.key].argv_eq_exact != Some(true) {
                return Err(Refusal::new(
                    "value-starts-with-dash",
                    &s.key,
                    "on this OS the value goes on the command line, where a leading `-` reads as \
                     a flag and `--flag=VALUE` is not byte-exact for this input",
                ));
            }
            bindings.push(Binding {
                source: i,
                key: s.key.clone(),
                kind: ChannelKind::Argv,
                terminator: String::new(),
                flag: None,
                env: None,
                fd: None,
                argv_form: Some(if dash { ArgvForm::Eq } else { ArgvForm::Sep }),
            });
        }
        let resolved = res.into_iter().map(|s| s.value).collect();
        return Ok(Planned {
            bindings,
            provenance: prov,
            resolved,
        });
    }
    match plan_private(&res, table, policy, os) {
        Ok(bindings) => Ok(Planned {
            bindings,
            provenance: prov,
            resolved: res.into_iter().map(|s| s.value).collect(),
        }),
        Err(e) => {
            let fd_on = policy.fd_channel_on.iter().any(|o| o == os);
            if !fd_on
                && matches!(
                    e.code,
                    "two-stdin" | "no-channel-left" | "no-channel-on-platform"
                )
            {
                if let Some(fd_os) = policy.fd_channel_on.first() {
                    if plan_private(&res, table, policy, fd_os).is_ok() {
                        return Err(Refusal::new(
                            "fd-not-on-platform",
                            e.source,
                            format!("needs a pipe fd; not enabled on {os}"),
                        ));
                    }
                }
            }
            Err(e)
        }
    }
}

/// `plan._plan`: the private-channel assignment (steps 1–3 of §A4.3).
fn plan_private(
    sources: &[PlanSource],
    table: &ChannelTable,
    policy: &Policy,
    os: &str,
) -> Result<Vec<Binding>, Refusal> {
    let fd_on = policy.fd_channel_on.iter().any(|o| o == os);
    let mut avail: Vec<Vec<&Channel>> = Vec::with_capacity(sources.len());
    for s in sources {
        let mut chans: Vec<&Channel> = table[&s.key].channels.iter().collect();
        if !fd_on {
            chans.retain(|c| !c.kind.is_fd());
        }
        if chans.is_empty() {
            return Err(Refusal::new("no-channel-on-platform", &s.key, os));
        }
        if !s.value.values().iter().all(|v| is_clean(v)) {
            chans.retain(|c| c.terminator.is_some());
            if chans.is_empty() {
                return Err(Refusal::new(
                    "value-not-byte-exact",
                    &s.key,
                    "every channel for this input trims whitespace; the value has CR/LF or edge \
                     whitespace",
                ));
            }
        }
        avail.push(chans);
    }

    fn pick<'a>(chans: &[&'a Channel], kinds: &[ChannelKind]) -> Option<&'a Channel> {
        for k in kinds {
            for c in chans {
                if c.kind == *k {
                    return Some(c);
                }
            }
        }
        None
    }

    let mut assigned: Vec<Option<&Channel>> = vec![None; sources.len()];
    let mut stdin_owner: Option<usize> = None;
    // Step 1 — forced stdin: sources whose every channel is a stdin channel.
    let forced: Vec<usize> = avail
        .iter()
        .enumerate()
        .filter(|(_, ch)| ch.iter().all(|c| c.kind.is_stdin()))
        .map(|(i, _)| i)
        .collect();
    if forced.len() > 1 {
        return Err(Refusal::new(
            "two-stdin",
            forced
                .iter()
                .map(|&i| sources[i].key.as_str())
                .collect::<Vec<_>>()
                .join(" + "),
            "each has stdin as its only channel",
        ));
    }
    if let Some(&i) = forced.first() {
        assigned[i] = pick(&avail[i], &ChannelKind::STDIN);
        stdin_owner = Some(i);
    }
    // Step 2 — if stdin is still free, the first source (argv order) with a
    // --X-stdin toggle.
    if stdin_owner.is_none() {
        for (i, ch) in avail.iter().enumerate() {
            if let Some(c) = pick(ch, &[ChannelKind::StdinToggle]) {
                assigned[i] = Some(c);
                stdin_owner = Some(i);
                break;
            }
        }
    }
    // Step 3 — the rest, argv order: env, else stdin if free, else fd, else
    // refuse.
    for (i, ch) in avail.iter().enumerate() {
        if assigned[i].is_some() {
            continue;
        }
        let mut c = pick(ch, &[ChannelKind::EnvRef]);
        if c.is_none() && stdin_owner.is_none() {
            c = pick(ch, &ChannelKind::STDIN);
            if c.is_some() {
                stdin_owner = Some(i);
            }
        }
        if c.is_none() {
            c = pick(ch, &ChannelKind::FD);
        }
        match c {
            Some(c) => assigned[i] = Some(c),
            None => {
                return Err(Refusal::new(
                    "no-channel-left",
                    &sources[i].key,
                    "stdin already used and no env/fd channel",
                ))
            }
        }
    }

    let env_prefix = policy.env_var_prefix();
    let mut bindings = Vec::with_capacity(sources.len());
    let mut fd_next = 3;
    for (i, c) in assigned.into_iter().enumerate() {
        let c = c.expect("every source is assigned or refused above");
        let terminator = c.terminator.clone().unwrap_or_default();
        let mut b = Binding {
            source: i,
            key: sources[i].key.clone(),
            kind: c.kind,
            terminator,
            flag: c.flag.clone(),
            env: None,
            fd: None,
            argv_form: None,
        };
        if c.kind == ChannelKind::EnvRef {
            b.env = Some(format!("{env_prefix}{i}"));
        }
        if c.kind.is_fd() {
            b.fd = Some(fd_next);
            fd_next += 1;
            let payload = sources[i].value.channel_data(&b.terminator);
            // a group's pipe payload is its lines plus the terminator
            let len = match &sources[i].value {
                SourceValue::Group(_) => payload.len() + b.terminator.len(),
                SourceValue::One(_) => payload.len(),
            };
            if len > policy.pipe_payload_max {
                return Err(Refusal::new(
                    "payload-too-large",
                    &sources[i].key,
                    format!("> {} bytes", policy.pipe_payload_max),
                ));
            }
        }
        bindings.push(b);
    }
    Ok(bindings)
}

// ── The invocation ─────────────────────────────────────────────────────────

/// Where a source sits in the assembled argv.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceSite {
    pub key: String,
    pub form: SourceForm,
    /// The flag token (`--passphrase`, `--from`, `--slot`), if any.
    pub flag_at: Option<usize>,
    /// The value token(s): one, or a group's members.
    pub value_at: Vec<usize>,
}

/// An assembled argv plus the secret sources inside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assembled {
    pub argv: Vec<String>,
    /// The assembler's display mask (`true` = masked in every display),
    /// parallel to `argv`; empty = all false. Kept tokens keep their bit
    /// (e.g. a pasted xprv in a public md field, DESIGN §B2–B4).
    pub mask: Vec<bool>,
    /// How many subcommand tokens follow `argv[0]` (1, or 2 when nested).
    pub sub_tokens: usize,
    /// The end-of-options `--` token before positionals, if any.
    pub eoo_at: Option<usize>,
    pub sites: Vec<SourceSite>,
    /// Whether the subcommand declares `--allow-argv-secret`.
    pub declares_allow_argv_secret: bool,
}

impl Assembled {
    /// The planner's sources: each site with its typed value.
    pub fn sources(&self) -> Vec<PlanSource> {
        self.sites
            .iter()
            .map(|s| {
                let text = |i: usize| -> String {
                    let t = &self.argv[i];
                    match &s.form {
                        SourceForm::Node { prefix } => {
                            t.strip_prefix(prefix.as_str()).unwrap_or(t).to_string()
                        }
                        _ => t.clone(),
                    }
                };
                let value = if s.form.is_group() {
                    SourceValue::Group(
                        s.value_at
                            .iter()
                            .map(|&i| Zeroizing::new(text(i)))
                            .collect(),
                    )
                } else {
                    SourceValue::One(Zeroizing::new(text(s.value_at[0])))
                };
                PlanSource {
                    key: s.key.clone(),
                    form: s.form.clone(),
                    value,
                }
            })
            .collect()
    }
}

impl Drop for Assembled {
    fn drop(&mut self) {
        self.argv.zeroize();
    }
}

/// One binding as the user sees it (Preview, confirm dialog, Copy).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingView {
    pub binding: Binding,
    /// Per value: typed, or `$VAR`.
    pub provenance: Vec<Provenance>,
    /// The index in [`RunPlan::argv`] of the token that names this binding's
    /// channel, if one does.
    pub argv_index: Option<usize>,
}

impl BindingView {
    /// e.g. `--passphrase ← stdin via --passphrase-stdin + '\r\n' (value of $MY_PW)`.
    pub fn line(&self) -> String {
        let prov = if self.provenance.iter().all(|p| *p == Provenance::Typed) {
            "typed".to_string()
        } else {
            format!(
                "value of {}",
                self.provenance
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        format!(
            "{} ← {} ({prov})",
            self.binding.source_label(),
            self.binding.channel_phrase()
        )
    }
}

/// What the runner executes (§A4.2).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunPlan {
    /// No secret byte on the private path; the interim path's resolved values
    /// are masked in `mask`.
    pub argv: Vec<String>,
    pub mask: Vec<bool>,
    /// Target + terminator of the stdin-bound source.
    pub stdin: Option<Zeroizing<Vec<u8>>>,
    /// `MNEMONIC_GUI_S<i>` → target.
    pub env: Vec<(String, Zeroizing<String>)>,
    /// Linux: child fd N ← a pipe holding these bytes; argv says `/dev/fd/N`.
    pub fds: Vec<(i32, Zeroizing<Vec<u8>>)>,
    pub bindings: Vec<BindingView>,
    /// True on the interim (argv) path.
    pub interim: bool,
}

impl RunPlan {
    /// A plan with no secret sources: the argv as assembled.
    pub fn plain(argv: Vec<String>) -> Self {
        let mut p = RunPlan::default();
        p.mask = vec![false; argv.len()];
        p.argv = argv;
        p
    }

    /// Any secret source in this run.
    pub fn has_secrets(&self) -> bool {
        !self.bindings.is_empty()
    }
}

impl Zeroize for RunPlan {
    fn zeroize(&mut self) {
        self.argv.zeroize();
        self.mask.zeroize();
        if let Some(s) = self.stdin.as_mut() {
            s.zeroize();
        }
        self.stdin = None;
        for (k, v) in self.env.iter_mut() {
            k.zeroize();
            v.zeroize();
        }
        self.env.clear();
        for (_, v) in self.fds.iter_mut() {
            v.zeroize();
        }
        self.fds.clear();
    }
}

impl Drop for RunPlan {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// `--allow-argv-secret` (the interim path's opt-in).
pub const ALLOW_ARGV_SECRET: &str = crate::form::invocation::ALLOW_ARGV_SECRET;

/// Turn a plan into an invocation (`run_plans.build`, and the interim
/// `baseline_argv`): the argv with each source's tokens replaced by its
/// channel spelling, the env, stdin and pipe payloads.
pub fn materialize(assembled: &Assembled, planned: &Planned) -> RunPlan {
    debug_assert_eq!(assembled.sites.len(), planned.bindings.len());
    let n = assembled.argv.len();
    // Per token: its replacement (None = keep).
    let mut replace: Vec<Option<Vec<(String, bool)>>> = vec![None; n];
    // Which binding a replaced token names (for `argv_index`).
    let mut names: Vec<Option<usize>> = vec![None; n];
    let mut extra_flags: Vec<(String, usize)> = Vec::new();
    let mut env = Vec::new();
    let mut stdin: Option<Zeroizing<Vec<u8>>> = None;
    let mut fds = Vec::new();
    let interim = planned.bindings.iter().any(|b| b.kind == ChannelKind::Argv);

    for (i, b) in planned.bindings.iter().enumerate() {
        let site = &assembled.sites[i];
        let val = &planned.resolved[i];
        let data = val.channel_data(&b.terminator);
        let prefix = match &site.form {
            SourceForm::Node { prefix } => prefix.as_str(),
            _ => "",
        };
        let first = site.value_at[0];
        let set = |replace: &mut Vec<Option<Vec<(String, bool)>>>,
                   at: usize,
                   toks: Vec<(String, bool)>| {
            replace[at] = Some(toks);
        };
        match b.kind {
            ChannelKind::EnvRef => {
                let name = b.env.clone().expect("EnvRef binding has an env name");
                set(
                    &mut replace,
                    first,
                    vec![(format!("{prefix}@env:{name}"), false)],
                );
                names[first] = Some(i);
                env.push((name, data));
            }
            ChannelKind::DashValue => {
                set(&mut replace, first, vec![(format!("{prefix}-"), false)]);
                names[first] = Some(i);
                stdin = Some(Zeroizing::new(data.as_bytes().to_vec()));
            }
            ChannelKind::StdinToggle => {
                let flag_at = site.flag_at.expect("a toggle replaces a flag");
                set(
                    &mut replace,
                    flag_at,
                    vec![(b.flag.clone().expect("toggle has a flag"), false)],
                );
                names[flag_at] = Some(i);
                set(&mut replace, first, vec![]);
                stdin = Some(Zeroizing::new(data.as_bytes().to_vec()));
            }
            ChannelKind::PosDash | ChannelKind::StdinMulti => {
                set(&mut replace, first, vec![("-".to_string(), false)]);
                names[first] = Some(i);
                for &at in &site.value_at[1..] {
                    set(&mut replace, at, vec![]);
                }
                stdin = Some(Zeroizing::new(data.as_bytes().to_vec()));
            }
            ChannelKind::FileFlag => {
                let fd = b.fd.expect("fd binding has an fd");
                let flag_at = site.flag_at.expect("a file flag replaces a flag");
                set(
                    &mut replace,
                    flag_at,
                    vec![(b.flag.clone().expect("file flag has a flag"), false)],
                );
                set(&mut replace, first, vec![(format!("/dev/fd/{fd}"), false)]);
                names[flag_at] = Some(i);
                fds.push((fd, Zeroizing::new(data.as_bytes().to_vec())));
            }
            ChannelKind::InFile => {
                let fd = b.fd.expect("fd binding has an fd");
                let mut payload = data.as_bytes().to_vec();
                if site.form.is_group() {
                    payload.push(b'\n');
                }
                match site.flag_at {
                    Some(flag_at) => {
                        set(&mut replace, flag_at, vec![("--in".to_string(), false)]);
                        set(&mut replace, first, vec![(format!("/dev/fd/{fd}"), false)]);
                        names[flag_at] = Some(i);
                    }
                    None => {
                        for &at in &site.value_at {
                            set(&mut replace, at, vec![]);
                        }
                        extra_flags.push((format!("/dev/fd/{fd}"), i));
                    }
                }
                fds.push((fd, Zeroizing::new(payload)));
            }
            ChannelKind::Argv => match (&site.form, b.argv_form, val) {
                (SourceForm::Value, Some(ArgvForm::Eq), SourceValue::One(v)) => {
                    let flag_at = site.flag_at.expect("value form has a flag");
                    let flag = assembled.argv[flag_at].clone();
                    set(&mut replace, flag_at, vec![]);
                    set(
                        &mut replace,
                        first,
                        vec![(format!("{flag}={}", v.as_str()), true)],
                    );
                }
                (_, _, SourceValue::One(v)) => {
                    set(
                        &mut replace,
                        first,
                        vec![(format!("{prefix}{}", v.as_str()), true)],
                    );
                }
                (_, _, SourceValue::Group(vs)) => {
                    for (k, &at) in site.value_at.iter().enumerate() {
                        set(&mut replace, at, vec![(vs[k].as_str().to_string(), true)]);
                    }
                }
            },
        }
    }

    // Rebuild. Pipe `--in` flags for positionals go just before `--`; a `--`
    // with no positional left after it is dropped.
    let mut argv: Vec<String> = Vec::with_capacity(n + 2);
    let mut mask: Vec<bool> = Vec::with_capacity(n + 2);
    let mut index_of: Vec<Option<usize>> = vec![None; planned.bindings.len()];
    let eoo = assembled.eoo_at;
    let positional_left = match eoo {
        Some(e) => (e + 1..n).any(|k| !matches!(&replace[k], Some(t) if t.is_empty())),
        None => false,
    };
    for k in 0..n {
        if Some(k) == eoo {
            for (tok, b) in &extra_flags {
                argv.push("--in".into());
                mask.push(false);
                index_of[*b] = Some(argv.len() - 1);
                argv.push(tok.clone());
                mask.push(false);
            }
            if !positional_left {
                continue;
            }
        }
        match &replace[k] {
            None => {
                argv.push(assembled.argv[k].clone());
                mask.push(assembled.mask.get(k).copied().unwrap_or(false));
            }
            Some(toks) => {
                for (t, m) in toks {
                    if let Some(b) = names[k] {
                        index_of[b].get_or_insert(argv.len());
                    }
                    argv.push(t.clone());
                    mask.push(*m);
                }
            }
        }
    }
    if eoo.is_none() && !extra_flags.is_empty() {
        for (tok, b) in &extra_flags {
            argv.push("--in".into());
            mask.push(false);
            index_of[*b] = Some(argv.len() - 1);
            argv.push(tok.clone());
            mask.push(false);
        }
    }
    if interim && assembled.declares_allow_argv_secret && !planned.bindings.is_empty() {
        let at = 1 + assembled.sub_tokens;
        argv.insert(at, ALLOW_ARGV_SECRET.to_string());
        mask.insert(at, false);
        for x in index_of.iter_mut().flatten() {
            if *x >= at {
                *x += 1;
            }
        }
    }
    let bindings = planned
        .bindings
        .iter()
        .enumerate()
        .map(|(i, b)| BindingView {
            binding: b.clone(),
            provenance: planned.provenance[i].clone(),
            argv_index: index_of[i],
        })
        .collect();
    RunPlan {
        argv,
        mask,
        stdin,
        env,
        fds,
        bindings,
        interim,
    }
}

// ── The form ───────────────────────────────────────────────────────────────

/// `"<cli> <subcommand tokens>"`, the channel-table key prefix.
pub fn key_base(schema: &Schema, sub: &SubcommandSchema) -> String {
    let mut s = schema.cli_name.to_string();
    for t in crate::form::invocation::subcommand_tokens(sub.name) {
        s.push(' ');
        s.push_str(t);
    }
    s
}

/// The secret-source classifier for a plain Text value (§A4.1): `<node>=<v>`
/// whose node is an argv-secret node, whatever `<v>` is. What `<v>` MEANS
/// (`-`, `@env:VAR`, or the secret itself) is C1's business, not the
/// classifier's.
pub fn text_value_names_secret_node(value: &str) -> Option<(&str, &str)> {
    let (node, v) = value.split_once('=')?;
    crate::secrets::node_type_is_argv_secret(node).then_some((node, v))
}

/// The form's argv and secret sources (§A4.1).
pub fn assemble(schema: &Schema, sub: &SubcommandSchema, state: &FormState) -> Assembled {
    let (argv, mask, sites, eoo_at) =
        crate::form::invocation::assemble_argv_with_sources(schema, sub, state);
    let sub_tokens = crate::form::invocation::subcommand_tokens(sub.name).len();
    Assembled {
        argv,
        mask,
        sub_tokens,
        eoo_at,
        sites,
        declares_allow_argv_secret: sub
            .flags
            .iter()
            .any(|f| f.name == crate::form::invocation::ALLOW_ARGV_SECRET),
    }
}

/// §A4.2: `plan(schema, sub, state, user_env, os) -> Result<RunPlan, _>`
/// under the committed table and policy.
pub fn plan(
    schema: &Schema,
    sub: &SubcommandSchema,
    state: &FormState,
    user_env: &dyn Fn(&str) -> Option<String>,
    os: &str,
) -> Result<RunPlan, ChannelRefusal> {
    plan_with(schema, sub, state, user_env, os, channel_table(), policy())
}

/// [`plan`] under an explicit table and policy (tests; the Copy plan).
pub fn plan_with(
    schema: &Schema,
    sub: &SubcommandSchema,
    state: &FormState,
    user_env: &dyn Fn(&str) -> Option<String>,
    os: &str,
    table: &ChannelTable,
    policy: &Policy,
) -> Result<RunPlan, ChannelRefusal> {
    let assembled = assemble(schema, sub, state);
    let source_tokens: std::collections::BTreeSet<usize> = assembled
        .sites
        .iter()
        .flat_map(|s| s.value_at.iter().copied())
        .collect();
    guard_passthrough(
        assembled
            .argv
            .iter()
            .enumerate()
            .skip(1)
            .filter(|(i, _)| !source_tokens.contains(i))
            .map(|(_, t)| t.as_str()),
        policy,
    )?;
    if assembled.sites.is_empty() {
        let mut p = RunPlan::plain(assembled.argv.clone());
        p.mask = assembled.mask.clone();
        return Ok(p);
    }
    let sources = assembled.sources();
    let planned = plan_sources(
        &sources,
        table,
        policy,
        os,
        user_env,
        PlanOptions::default(),
    )?;
    Ok(materialize(&assembled, &planned))
}

/// What the GUI's Run button executes for this form on this OS: [`plan`]
/// under the process environment and [`current_os`]. Tests that model "what
/// the GUI runs" against a real CLI use this with `runner::run_plan`.
pub fn plan_for_run(
    schema: &Schema,
    sub: &SubcommandSchema,
    state: &FormState,
) -> Result<RunPlan, ChannelRefusal> {
    plan(schema, sub, state, &process_env, current_os())
}

/// The GUI's own environment, as the planner reads it for `@env:VAR`. A
/// variable that is not valid UTF-8 reads as unset.
pub fn process_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// The OS name the policy is keyed by (`linux`, `macos`, `windows`, …).
pub fn current_os() -> &'static str {
    std::env::consts::OS
}
