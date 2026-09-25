//! Pinned schema for the `ms` CLI (ms-cli-v0.8.0).
//!
//! Scope: `inspect`/`encode`/`decode`/`verify`/`vectors` plus `repair`
//! (backfilled into the mirror at v0.5 — it shipped in v0.4 but was never
//! mirrored), the v0.5 read-only `derive` (master fingerprint + account
//! xpub), and the v0.7.0 BIP-93 codex32 K-of-N `split`/`combine` pair. See
//! `design/agent-reports/v0_2-phase-D1-help-audit-r1.md` for the original
//! per-flag provenance.

use super::{FlagKind, FlagSchema, NumberMax, PositionalArgSchema, Schema, SubcommandSchema};

/// BIP-39 wordlist tokens accepted by the `ms` CLI. Hyphenated
/// Chinese variants (NOT the fused tokens used by mnemonic.rs:
/// `simplifiedchinese` / `traditionalchinese`). Using the wrong
/// tokens silently emits argv rejected by the binary — see D.1
/// audit R1 finding #1.
pub const LANG_MS: &[&str] = &[
    "english",
    "japanese",
    "korean",
    "spanish",
    "chinese-simplified",
    "chinese-traditional",
    "french",
    "italian",
    "czech",
    "portuguese",
];

// mstring display-grouping (ms-cli v0.8.0): `--separator` keyword values.
// SPEC §I7 — keyword dropdown; the CLI reports `--separator` as kind `text`,
// the GUI narrows it. Names-only gate.
// F-679 (ms-cli v0.19.0): `hyphen` and `comma` are RETIRED — ms refuses them
// ("separator \"hyphen\" is no longer offered: `ms` emits whitespace grouping
// only"). The drift gate cannot see this (the CLI reports `text`, no choices),
// so it was measured against the release binary. Offering them would be a
// dropdown value that always fails.
const SEPARATORS: &[&str] = &["space"];

// F-679 (ms-cli v0.19.0): ms REFUSES secret material on argv unless
// `--allow-argv-secret` is present, on the eight material verbs. GUI-managed,
// exactly as `schema::mnemonic::ALLOW_ARGV_SECRET_FLAG`: mirrored for parity,
// never rendered, added only by the Run path when the argv carries a
// secret-masked token. ms declares it non-global (its gui-schema carries no
// `global` key).
const ALLOW_ARGV_SECRET_FLAG: FlagSchema = FlagSchema {
    global: false,
    ..super::mnemonic::ALLOW_ARGV_SECRET_FLAG
};

// ─── inspect ─────────────────────────────────────────────────────────────

// `ms inspect [MS1] [--json]`
const INSPECT_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON instead of text verdict + fields.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read the ms1 string from FILE instead of argv (a private channel: \
               the secret never reaches the command line).",
        secret: false,
        default_value: None,
        global: false,
    },
    ALLOW_ARGV_SECRET_FLAG,
];

const INSPECT_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "ms1",
    required: false,
    repeating: false,
    help: "ms1 string to inspect. Type the value, or `@env:VAR` (read by the GUI; a value that is `-` or starts with `@env` is refused).",
    secret: true,
}];

// ─── encode ──────────────────────────────────────────────────────────────

// `ms encode --phrase|--hex [--language] [--no-engraving-card] [--json]`
//
// Upstream: `--phrase` XOR `--hex` (required_one_of with mutual exclusion).
// Both are secret-bearing (BIP-39 mnemonic / raw entropy bytes). When
// `--hex` is supplied, `--language` is ignored (upstream help). Conditional
// fn wires this in `form::conditional::ms_encode`.
const ENCODE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--group-size",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65535),
        },
        required: false,
        repeating: false,
        help: "Display grouping: break the emitted card into groups of N \
               characters (default 5; 0 = unbroken single line). Cosmetic — \
               intake strips separators, so any grouping re-ingests.",
        secret: false,
        default_value: Some("5"),
        global: false,
    },
    FlagSchema {
        name: "--separator",
        kind: FlagKind::Dropdown(SEPARATORS),
        required: false,
        repeating: false,
        help: "Display-grouping separator: `space` only (hyphen and comma \
               were retired). Cosmetic — non-load-bearing.",
        secret: false,
        default_value: Some("space"),
        global: false,
    },
    FlagSchema {
        name: "--phrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 mnemonic phrase. XOR with --hex.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--hex",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Raw entropy as hex. XOR with --phrase. --language is ignored.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANG_MS),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (default english). Ignored when --hex is set.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--no-engraving-card",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Suppress the human-readable engraving-card panel.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON instead of text output.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read the BIP-39 PHRASE from FILE (never hex). A private channel: \
               the phrase never reaches the command line.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--out",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Write the canonical artifact to FILE, owner-only (0600), instead \
               of stdout. OVERWRITES an existing file.",
        secret: false,
        default_value: None,
        global: false,
    },
    ALLOW_ARGV_SECRET_FLAG,
];

const ENCODE_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── decode ──────────────────────────────────────────────────────────────

// `ms decode [MS1] [--language] [--json]`
const DECODE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANG_MS),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (default english) used to render the phrase.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON instead of text output.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read the ms1 string from FILE instead of argv (a private channel: \
               the secret never reaches the command line).",
        secret: false,
        default_value: None,
        global: false,
    },
    ALLOW_ARGV_SECRET_FLAG,
];

const DECODE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "ms1",
    required: false,
    repeating: false,
    help: "ms1 string to decode. Type the value, or `@env:VAR` (read by the GUI; a value that is `-` or starts with `@env` is refused).",
    secret: true,
}];

// ─── verify ──────────────────────────────────────────────────────────────

// `ms verify [MS1] --phrase [--language] [--json]`
//
// `--phrase` is secret-bearing (round-trip check against the supplied ms1).
const VERIFY_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--phrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 mnemonic phrase to round-trip against the ms1.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANG_MS),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (default english).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON instead of text output.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read the ms1 string from FILE instead of argv (a private channel: \
               the secret never reaches the command line).",
        secret: false,
        default_value: None,
        global: false,
    },
    ALLOW_ARGV_SECRET_FLAG,
];

const VERIFY_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "ms1",
    required: false,
    repeating: false,
    help: "ms1 string to verify. Type the value, or `@env:VAR` (read by the GUI; a value that is `-` or starts with `@env` is refused).",
    secret: true,
}];

// ─── vectors ─────────────────────────────────────────────────────────────

// `ms vectors [--pretty]` — maintainer tool, emits test-vector JSON.
const VECTORS_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--pretty",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Pretty-print JSON output.",
    secret: false,
    default_value: None,
    global: false,
}];

const VECTORS_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── derive (v0.5) ───────────────────────────────────────────────────────
// `ms derive [ms1] [--hex|--phrase] [--template] [--account] [--network]
//            [--passphrase|--passphrase-stdin] [--language] [--json]`.
// `--template`/`--network`/`--language` reflect as ValueEnum dropdowns;
// `--account` reflects as text (ms-cli gui-schema classifies u32 as text).
const DERIVE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--hex",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Raw entropy hex (alternative to the ms1).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--phrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 phrase (alternative to the ms1).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--template",
        // Order is upstream's, verified against `ms gui-schema` (the CLI's own
        // JSON) rather than its --help prose. The three bip48-* entries are
        // MULTISIG account paths at depth 4; the first four are single-sig at
        // depth 3.
        kind: FlagKind::Dropdown(&[
            "bip44",
            "bip49",
            "bip84",
            "bip86",
            "bip48-p2wsh",
            "bip48-p2sh-p2wsh",
            "bip48",
            // F-679 (ms-cli v0.19.0): three templates added upstream.
            "bip48-p2tr",
            "bg002h-tr",
            "bg002h-wsh",
        ]),
        required: false,
        repeating: false,
        help: "Account-path template; emits an account xpub. bip44/49/84/86 are \
               single-sig account paths; the bip48-* variants are multisig \
               account paths (m/48'/coin'/account'/script'); bip48-p2tr is the \
               taproot-multisig convention (script 3'); bg002h-tr / bg002h-wsh \
               are the constellation's own paths (m/270028'/coin'/account'/0' \
               and /1').",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--account",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Account index for --template (default 0).",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(&["mainnet", "testnet"]),
        required: false,
        repeating: false,
        help: "Network for the account xpub serialization + coin-type.",
        secret: false,
        default_value: Some("mainnet"),
        global: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 passphrase.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read the BIP-39 passphrase from stdin.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANG_MS),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (load-bearing; default english).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON instead of text output.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read the ms1 string from FILE instead of argv (a private channel: \
               the secret never reaches the command line).",
        secret: false,
        default_value: None,
        global: false,
    },
    ALLOW_ARGV_SECRET_FLAG,
];

const DERIVE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "ms1",
    required: false,
    repeating: false,
    help: "ms1 string. Type the value, or `@env:VAR` (read by the GUI; a value that is `-` or starts with `@env` is refused).",
    secret: true,
}];

// ─── repair (v0.4; backfilled into the mirror at v0.5) ───────────────────
const REPAIR_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--ms1",
        kind: FlagKind::Text,
        // F-679: no longer clap-required upstream (ms-cli v0.19.0 gui-schema
        // `required: false`) — `--in FILE` is the private alternative.
        required: false,
        repeating: false,
        help: "ms1 string to repair via BCH error correction. Type the value, or `@env:VAR` (read by the GUI; a value that is `-` or starts with `@env` is refused).",
        // v0.33.0 deliberate GUI-side override (audit I4 ms.rs half): the
        // to-be-repaired ms1 IS master-secret material (BCH-corrupted BIP-39
        // entropy; the lone false twin of the 8-site --ms1 census). No
        // automated gate covers ms.rs secret bits (schema_mirror_secret_drift
        // walks schema::mnemonic only), and ms-cli has no gui-schema surface
        // to mirror — see FOLLOWUPS.md::ms-repair-ms1-not-secret-classified.
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON instead of text output.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read the ms1 string from FILE instead of argv (a private channel: \
               the secret never reaches the command line).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--out",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Write the canonical artifact to FILE, owner-only (0600), instead \
               of stdout. OVERWRITES an existing file.",
        secret: false,
        default_value: None,
        global: false,
    },
    ALLOW_ARGV_SECRET_FLAG,
];

const REPAIR_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── split (v0.7.0) ──────────────────────────────────────────────────────
//
// `ms split (--phrase|--hex) --threshold K --shares N [--language] [--json]`
// BIP-93 codex32 K-of-N share splitter. `--phrase` / `--hex` are both
// secret-bearing (BIP-39 mnemonic / raw entropy). schema_mirror gates the
// flag-NAME set; the `--json` wire-shape is NOT gated (FOLLOWUP
// `ms-kofn-json-wire-shape-ungated`).
const SPLIT_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--group-size",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65535),
        },
        required: false,
        repeating: false,
        help: "Display grouping: break each emitted share into groups of N \
               characters (default 5; 0 = unbroken single line). Cosmetic — \
               intake strips separators, so any grouping re-ingests.",
        secret: false,
        default_value: Some("5"),
        global: false,
    },
    FlagSchema {
        name: "--separator",
        kind: FlagKind::Dropdown(SEPARATORS),
        required: false,
        repeating: false,
        help: "Display-grouping separator: `space` only (hyphen and comma \
               were retired). Cosmetic — non-load-bearing.",
        secret: false,
        default_value: Some("space"),
        global: false,
    },
    FlagSchema {
        name: "--phrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 mnemonic to split. Type the value, or `@env:VAR` (read by the GUI; a value that is `-` or starts with `@env` is refused). XOR with --hex.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--hex",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Hex-encoded entropy to split (16/20/24/28/32 B). XOR with --phrase.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--threshold",
        kind: FlagKind::Number { min: 2, max: super::NumberMax::Static(9) },
        required: true,
        repeating: false,
        help: "Threshold K — minimum shares needed to recombine (2..=9).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--shares",
        kind: FlagKind::Number { min: 2, max: super::NumberMax::Static(31) },
        required: true,
        repeating: false,
        help: "Total shares N to produce (K <= N <= 31).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANG_MS),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist for the input phrase; ignored under --hex. \
               A non-English wordlist produces a `mnem` share-set (default english).",
        secret: false,
        default_value: Some("english"),
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a single JSON object on stdout instead of multi-line text.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read the BIP-39 PHRASE from FILE (never hex). A private channel: \
               the phrase never reaches the command line.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--out",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Write the canonical artifact to FILE, owner-only (0600), instead \
               of stdout. OVERWRITES an existing file.",
        secret: false,
        default_value: None,
        global: false,
    },
    ALLOW_ARGV_SECRET_FLAG,
];

const SPLIT_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── combine (v0.7.0) ────────────────────────────────────────────────────
//
// `ms combine <SHARES>... [--to phrase|entropy|ms1] [--json]`
// The shares are POSITIONAL (secret-equivalent — flagged on the positional).
const COMBINE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--to",
        kind: FlagKind::Dropdown(COMBINE_TO_SHAPES),
        required: false,
        repeating: false,
        help: "Output form (default phrase): phrase (mnem -> wire language, entr -> english), \
               entropy (hex), or ms1 (single unshared string).",
        secret: false,
        default_value: Some("phrase"),
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a single JSON object on stdout instead of text.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read the shares from FILE, one per line, instead of argv. A \
               private channel: the shares never reach the command line.",
        secret: false,
        default_value: None,
        global: false,
    },
    ALLOW_ARGV_SECRET_FLAG,
];

const COMBINE_TO_SHAPES: &[&str] = &["phrase", "entropy", "ms1"];

const COMBINE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "shares",
    required: true,
    repeating: true,
    help: "Distributed share strings to recombine (K or more, distinct indices). Secret-equivalent.",
    secret: true,
}];

// ─── gen-man ─────────────────────────────────────────────────────────────
//
// man-pages cycle (ms-cli v0.13.0): emit roff man pages for the whole CLI
// tree into `--out <DIR>`. Single required directory-path flag; no
// positionals, no global flags. The ms-cli gui-schema emits `--out` with
// `kind: "text"`, so the GUI mirror uses `FlagKind::Text` for parity.
const GEN_MAN_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--out",
    kind: FlagKind::Text,
    required: true,
    repeating: false,
    help: "Directory to write the `*.1` man pages into (created if absent). \
           One page per (nested) subcommand, hyphen-joined parent→child.",
    secret: false,
    default_value: None,
    global: false,
}];

const GEN_MAN_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── hashlock (DESIGN secret channels Part B, B5) ────────────────────────────

// `ms hashlock` (ms-cli v0.19.1): a hashlock preimage/digest from exactly ONE
// source — a hashlock phrase, `--hex` (a 32-byte preimage), an `<ms1>`
// preimage plate, `--in FILE`, or `--random` (which needs `--out`). ms's
// v1 gui-schema carries no secret bit; the phrase, `--hex` and the `<ms1>`
// positional are HAND-MARKED secret here (DESIGN §A4.1, §B6).
//
// - The phrase's only private channel is `--hashlock-phrase-stdin` (measured:
//   `ms hashlock --hashlock-phrase` has no `-`/`@env:` cell), which the
//   planner emits; the toggle itself stays rendered disabled (§A4.5). The
//   phrase is BYTE-VERBATIM: no trim (T9).
// - `--kind` starts at "(choose)" and Run is disabled until a kind is chosen:
//   omitting --kind puts a sha256 record on stdout (F-553). "all kinds —
//   lookup only" omits the flag deliberately, with a banner.
// - `--separator`: the pinned ms 0.19.1 accepts only `space` (hyphen/comma
//   exit 64: "ms emits whitespace grouping only"), so the shared ms list.
// Conditional fn at `form::conditional::ms_hashlock`.

/// The GUI-only `--kind` value that omits the flag on purpose (every kind's
/// digest listed; stdout is the sha256 record). Never emitted.
pub const HASHLOCK_KIND_ALL: &str = "all kinds — lookup only";
/// `--kind`: `""` is the "(choose)" sentinel — Run stays disabled on it.
pub const HASHLOCK_KINDS: &[&str] = &["", "sha256", "hash256", "ripemd160", "hash160", HASHLOCK_KIND_ALL];
const HASHLOCK_METHODS: &[&str] = &["hardened", "sha256"];

const HASHLOCK_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--hashlock-phrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "The hashlock phrase, byte for byte (never trimmed). Sent privately \
               over --hashlock-phrase-stdin. Type the value, or `@env:VAR` (read by \
               the GUI; a value that is `-` or starts with `@env` is refused). One source only.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--hashlock-phrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read the phrase from stdin. GUI-managed: the Run path uses it for the \
               phrase field.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--kind",
        kind: FlagKind::Dropdown(HASHLOCK_KINDS),
        required: false,
        repeating: false,
        help: "Which hash the SCRIPT commits to: sha256, hash256, ripemd160, hash160. \
               Run stays disabled until one is chosen. \"all kinds — lookup only\" omits \
               the flag: every kind's digest is listed and stdout is the sha256 record.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--hex",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "A 32-byte preimage as 64 hex characters. Type the value, or `@env:VAR` \
               (read by the GUI; a value that is `-` or starts with `@env` is refused). \
               One source only.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read the ms1 (preimage plate) string from FILE. One source only.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--random",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "32 bytes from the OS random source. Requires --out FILE. One source only.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--method",
        kind: FlagKind::Dropdown(HASHLOCK_METHODS),
        required: false,
        repeating: false,
        help: "Phrase -> preimage method (phrase sources only). Default hardened.",
        secret: false,
        default_value: Some("hardened"),
        global: false,
    },
    FlagSchema {
        name: "--out",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Write the preimage ms1 string to FILE, owner-only. Never suppresses stdout.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "One JSON object on stdout in place of the record line. stdout then CARRIES THE SECRET.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--no-engraving-card",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Suppress the engraving card.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--emit-record",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Also print a `phrase:` record on the card (for `me sysw pack \
               --pack-preimage`). Phrase source only.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--group-size",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65_535),
        },
        required: false,
        repeating: false,
        help: "Group the ms1 on the card every N characters (0 = no grouping). Default 5.",
        secret: false,
        default_value: Some("5"),
        global: false,
    },
    FlagSchema {
        name: "--separator",
        kind: FlagKind::Dropdown(SEPARATORS),
        required: false,
        repeating: false,
        help: "Separator for the card: space (ms emits whitespace grouping only).",
        secret: false,
        default_value: Some("space"),
        global: false,
    },
    ALLOW_ARGV_SECRET_FLAG,
    FlagSchema {
        name: "--phrase-looks-like-digest-ok",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Accept a phrase that looks like a hex digest (normally refused as a \
               likely paste mistake).",
        secret: false,
        default_value: None,
        global: false,
    },
];

const HASHLOCK_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "ms1",
    required: false,
    repeating: false,
    help: "A preimage-kind ms1 string (a preimage plate), to re-derive the digest. \
           Type the value, or `@env:VAR` (read by the GUI; a value that is `-` or \
           starts with `@env` is refused). One source only.",
    secret: true,
}];

// ─── SCHEMA constant ─────────────────────────────────────────────────────

const SUBCOMMANDS: &[SubcommandSchema] = &[
    SubcommandSchema {
        name: "inspect",
        human_name: "Inspect (verdict + fields)",
        flags: INSPECT_FLAGS,
        positional_args: INSPECT_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::ms_ms1_or_in),
    },
    SubcommandSchema {
        name: "encode",
        human_name: "Encode (phrase/hex -> ms1)",
        flags: ENCODE_FLAGS,
        positional_args: ENCODE_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::ms_encode),
    },
    SubcommandSchema {
        name: "decode",
        human_name: "Decode (ms1 -> phrase)",
        flags: DECODE_FLAGS,
        positional_args: DECODE_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::ms_ms1_or_in),
    },
    SubcommandSchema {
        name: "verify",
        human_name: "Verify (phrase <-> ms1 round-trip)",
        flags: VERIFY_FLAGS,
        positional_args: VERIFY_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::ms_ms1_or_in),
    },
    SubcommandSchema {
        name: "vectors",
        human_name: "Vectors (test-vector dump)",
        flags: VECTORS_FLAGS,
        positional_args: VECTORS_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "derive",
        human_name: "Derive (master fingerprint + account xpub)",
        flags: DERIVE_FLAGS,
        positional_args: DERIVE_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::ms_derive),
    },
    SubcommandSchema {
        name: "repair",
        human_name: "Repair (BCH error correction)",
        flags: REPAIR_FLAGS,
        positional_args: REPAIR_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::ms_repair),
    },
    // v0.7.0: BIP-93 codex32 K-of-N share split / combine.
    SubcommandSchema {
        name: "split",
        human_name: "Split (BIP-93 codex32 K-of-N share splitter)",
        flags: SPLIT_FLAGS,
        positional_args: SPLIT_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "combine",
        human_name: "Combine (recombine >=K codex32 shares)",
        flags: COMBINE_FLAGS,
        positional_args: COMBINE_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::ms_combine),
    },
    // DESIGN secret channels Part B (B5), built on Part A's StdinToggle channel.
    SubcommandSchema {
        name: "hashlock",
        human_name: "Hashlock (phrase/preimage -> hashlock digest)",
        flags: HASHLOCK_FLAGS,
        positional_args: HASHLOCK_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::ms_hashlock),
    },
    SubcommandSchema {
        name: "gen-man",
        human_name: "Gen Man (emit roff man pages for the whole CLI tree)",
        flags: GEN_MAN_FLAGS,
        positional_args: GEN_MAN_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
];

pub const SCHEMA: Schema = Schema {
    cli_name: "ms",
    pinned_version: "ms 0.19.1",
    subcommands: SUBCOMMANDS,
};
