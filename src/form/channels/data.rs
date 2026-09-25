//! The planner's DATA (DESIGN §A0, §A1, §A4.2).
//!
//! Two kinds, kept apart on purpose:
//!
//! - **Derived** by measurement against the pinned binaries, regenerated and
//!   diffed in CI (`regen_check.py`, T4), never hand-edited:
//!   `channel_table.json` (channels, terminators, the per-input `cli_env_rule`,
//!   `argv_eq_exact`) and `reinterpret.json` (the spellings each CLI re-reads
//!   on argv; a measurement the planner's safety does NOT read, R4 NI8).
//! - **Decided** by humans: `channel_policy.json` (per-OS switches, the
//!   lookalike predicate, the trailing-CR/LF refusal, the reserved prefix, the
//!   env-name rule, bounds, the CI test targets). Nothing about CLI behaviour.
//!
//! The files are compiled in with `include_str!` from
//! `design/measurements/secret-channels/`, so there is exactly ONE copy of
//! each: the committed cache the measurement pipeline writes and CI diffs. A
//! bump that re-derives the cache changes the GUI's behaviour with no code
//! edit (DESIGN §A3b).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// `design/measurements/secret-channels/channel_table.json` (derived).
pub const CHANNEL_TABLE_JSON: &str =
    include_str!("../../../design/measurements/secret-channels/channel_table.json");
/// `design/measurements/secret-channels/channel_policy.json` (decided).
pub const CHANNEL_POLICY_JSON: &str =
    include_str!("../../../design/measurements/secret-channels/channel_policy.json");
/// `design/measurements/secret-channels/reinterpret.json` (derived; measurement only).
pub const REINTERPRET_JSON: &str =
    include_str!("../../../design/measurements/secret-channels/reinterpret.json");

/// How a secret reaches the CLI. `Argv` is the interim path's only kind
/// (DESIGN §A6); the others are measured channel cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelKind {
    /// `ms combine -- -`: every share on stdin, one per line.
    StdinMulti,
    /// A `--X-stdin` toggle that replaces `--X VALUE`.
    StdinToggle,
    /// `--X -` or `--X <node>=-` / `@N.<subkey>=-`.
    DashValue,
    /// A positional `-`.
    PosDash,
    /// `@env:NAME` in place of the value.
    EnvRef,
    /// `--X-file /dev/fd/N`.
    FileFlag,
    /// `--in /dev/fd/N`.
    InFile,
    /// Interim path: the resolved bytes on argv with `--allow-argv-secret`.
    Argv,
}

impl ChannelKind {
    /// The stdin channel kinds, in preference order (DESIGN §A4.3).
    pub const STDIN: [ChannelKind; 4] = [
        ChannelKind::StdinMulti,
        ChannelKind::StdinToggle,
        ChannelKind::DashValue,
        ChannelKind::PosDash,
    ];
    /// The pipe-fd channel kinds, in preference order.
    pub const FD: [ChannelKind; 2] = [ChannelKind::FileFlag, ChannelKind::InFile];

    pub fn is_stdin(self) -> bool {
        Self::STDIN.contains(&self)
    }

    pub fn is_fd(self) -> bool {
        Self::FD.contains(&self)
    }
}

/// One measured-OK channel cell.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Channel {
    pub kind: ChannelKind,
    /// The flag spelling the channel needs (`--passphrase-stdin`,
    /// `--secret-file`, `--in`), where it has one.
    #[serde(default)]
    pub flag: Option<String>,
    /// The measured terminator the GUI appends after the target (§A3c).
    /// `None` = lenient: the channel trims whitespace where argv would not,
    /// so it may carry only a CLEAN value.
    pub terminator: Option<String>,
}

/// What the pinned CLI's own `@env:VAR` does to the variable ON THIS INPUT
/// (derived per input; R3 NI7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvRule {
    Verbatim,
    StripOneTrailingNewline,
    /// Matched no known rule: `@env:` on this input refuses
    /// (`C1-env-rule-unknown`).
    Unknown,
}

impl EnvRule {
    /// The ONE implementation of each candidate rule (R4 NI9), mirroring
    /// `plan.RULES`.
    pub fn apply(self, raw: &str) -> String {
        match self {
            EnvRule::Verbatim | EnvRule::Unknown => raw.to_string(),
            EnvRule::StripOneTrailingNewline => {
                if let Some(s) = raw.strip_suffix("\r\n") {
                    s.to_string()
                } else if let Some(s) = raw.strip_suffix('\n') {
                    s.to_string()
                } else {
                    raw.to_string()
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for EnvRule {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.as_str() {
            "verbatim" => EnvRule::Verbatim,
            "strip-one-trailing-newline" => EnvRule::StripOneTrailingNewline,
            // "UNKNOWN", or any rule name this build does not know: refuse.
            _ => EnvRule::Unknown,
        })
    }
}

/// One input's row in `channel_table.json`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TableEntry {
    pub channels: Vec<Channel>,
    #[serde(default)]
    pub measured_in: String,
    /// `None` = the input has no working CLI `@env:`; the GUI then takes the
    /// variable's bytes as typed.
    #[serde(default)]
    pub cli_env_rule: Option<EnvRule>,
    /// Whether `--flag=VALUE` is byte-identical to `--flag VALUE` (R3 Nm13).
    #[serde(default)]
    pub argv_eq_exact: Option<bool>,
}

/// `channel_table.json`: input key → its measured row.
pub type ChannelTable = BTreeMap<String, TableEntry>;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ChannelLookalike {
    pub refuse_if_equals: Vec<String>,
    pub refuse_if_startswith: Vec<String>,
}

/// `channel_policy.json`: DECISIONS only.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Policy {
    pub private_channels_on: Vec<String>,
    pub fd_channel_on: Vec<String>,
    pub channel_lookalike: ChannelLookalike,
    pub refuse_trailing_cr_lf: bool,
    pub reserved_env_prefix: String,
    pub env_name_rule: String,
    pub pipe_payload_max: usize,
    pub real_binary_test_targets: Vec<String>,
}

impl Policy {
    /// The planner's own variables: `MNEMONIC_GUI_S<i>`.
    pub fn env_var_prefix(&self) -> String {
        format!("{}S", self.reserved_env_prefix)
    }

    /// The policy the Copy command is generated under (DESIGN §A7): the
    /// PRIVATE-channel plan on every OS, so a pasted command behaves like
    /// Linux's Run and never carries a secret value.
    pub fn for_copy(&self) -> Policy {
        let mut p = self.clone();
        p.private_channels_on = vec![COPY_OS.to_string()];
        p.fd_channel_on = vec![COPY_OS.to_string()];
        p
    }
}

/// The OS name the Copy plan is computed for (DESIGN §A7).
pub const COPY_OS: &str = "linux";

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Reinterpret {
    pub version: String,
    pub spellings: Vec<String>,
}

/// The committed channel table, parsed once.
pub fn channel_table() -> &'static ChannelTable {
    static T: OnceLock<ChannelTable> = OnceLock::new();
    T.get_or_init(|| serde_json::from_str(CHANNEL_TABLE_JSON).expect("channel_table.json parses"))
}

/// The committed policy, parsed once.
pub fn policy() -> &'static Policy {
    static P: OnceLock<Policy> = OnceLock::new();
    P.get_or_init(|| serde_json::from_str(CHANNEL_POLICY_JSON).expect("channel_policy.json parses"))
}

/// The committed re-read measurement (per CLI). Used by the consistency test
/// and the oracle legs; the planner does not read it (R4 NI8).
pub fn reinterpret() -> BTreeMap<String, Reinterpret> {
    #[derive(Deserialize)]
    struct Row {
        version: String,
        spellings: Vec<String>,
    }
    let raw: BTreeMap<String, Row> =
        serde_json::from_str(REINTERPRET_JSON).expect("reinterpret.json parses");
    raw.into_iter()
        .map(|(k, r)| {
            (
                k,
                Reinterpret {
                    version: r.version,
                    spellings: r.spellings,
                },
            )
        })
        .collect()
}
