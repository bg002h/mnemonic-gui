//! Argv assembler + copy-command shell-quoting (SPEC §6).
//!
//! `assemble_argv` produces a `Vec<String>` from a (Schema, SubcommandSchema,
//! FormState) triple per the byte-exact SPEC §6.7 emission rules.
//! `render_copy_command` formats an argv vector for display-only copy-paste
//! under either POSIX or Windows shell flavors.
//!
//! The argv is passed directly to `std::process::Command::args()` (no shell
//! escaping needed); the copy-command output is for the user's eyes only
//! and is NEVER used to spawn the subprocess.

use crate::schema::{
    FlagKind, FlagSchema, FlagValue, Schema, SubcommandSchema, TaggedOrIndexedValue,
    TimestampValue, Visibility,
};

/// v0.10.0 B.3 (D33) — compare the user-typed `value` against the flag's
/// schema-declared `default_value` per the D33 per-FlagKind compare-predicate
/// table. Returns `true` iff the value equals the schema default; in that
/// case the argv assembler suppresses the flag from emission (the user
/// hasn't deviated from the toolkit's own default, so adding it to argv
/// would be noise).
///
/// Flags without a declared `default_value` (`default_value == None`) always
/// return `false` — those flags emit whenever Set per the existing
/// `emit_one` rules.
///
/// D33 compare-predicate table:
///
/// | FlagKind            | Predicate                                                          |
/// |---------------------|---------------------------------------------------------------------|
/// | Boolean             | `value == default` (trivially: false-default already suppressed)    |
/// | Text                | `value.is_empty() OR value == default_str`                          |
/// | Path                | `value == default_str` (sentinels like `"-"` matched literally)     |
/// | Number              | `value == default_int` (decimal parse of default_str)               |
/// | Range               | `"<a>,<b>" == default_str` (range serialization preserved)          |
/// | Timestamp           | `Now` matches `"now"`; `Epoch(n)` never matches `"now"`             |
/// | Dropdown            | `value == default_str`                                              |
/// | TaggedOrIndexed     | per-variant default rare; falls back to "always emit if Set"        |
/// | NodeValueComposite  | empty-value already trivially suppressed by `emit_one`              |
///
/// Defensive parse failures (e.g., garbage default_str on a Number flag)
/// return `false` so the flag emits — never silently drop a user-typed
/// value due to a schema typo.
pub fn is_at_default(flag: &FlagSchema, value: &FlagValue) -> bool {
    let Some(default_str) = flag.default_value else {
        return false;
    };
    match (&flag.kind, value) {
        // Boolean: presence-only emission. Boolean(false) is already
        // trivially suppressed by emit_one (no token emitted). Boolean(true)
        // when default is "true" (rare; toolkit v5 doesn't emit Boolean
        // defaults) is suppressed.
        (FlagKind::Boolean, FlagValue::Boolean(b)) => match default_str {
            "true" => *b,
            "false" => !*b,
            _ => false,
        },
        // Text: empty is already trivially suppressed; non-empty matches
        // default_str suppress.
        (FlagKind::Text, FlagValue::Text(s)) => s.is_empty() || s == default_str,
        // Path: empty already trivially suppressed; non-empty matches
        // default_str (e.g., "-" sentinel) suppress.
        (FlagKind::Path { .. }, FlagValue::Path(p)) => p == default_str,
        // Number: parse default_str as i64; compare. Parse failure → false
        // (defensive; emit user-typed value).
        (FlagKind::Number { .. }, FlagValue::Number(n)) => {
            default_str.parse::<i64>().is_ok_and(|d| d == *n)
        }
        // Range: format the user-typed pair the same way emit_one does
        // ("<a>,<b>") and compare against the schema default_str.
        (FlagKind::Range, FlagValue::Range(a, b)) => {
            format!("{},{}", a, b) == default_str
        }
        // Timestamp: `Now` is at-default only when the schema default_str is
        // literally "now" (export-wallet's default is "0" since toolkit
        // v0.47.3, so an explicit `Now` emits `--timestamp now`); `Unix(n)`
        // always emits per D33 (the default form seeds Unset, not a number).
        (FlagKind::Timestamp, FlagValue::Timestamp(t)) => match t {
            TimestampValue::Now => default_str == "now",
            TimestampValue::Unix(_) => false,
        },
        // Dropdown: literal string equality against default_str.
        (FlagKind::Dropdown(_), FlagValue::Dropdown(s)) => s == default_str,
        // TaggedOrIndexed: per-variant default is rare. Default to false
        // (always emit if Set).
        (FlagKind::TaggedOrIndexed(_), FlagValue::TaggedOrIndexed(_)) => false,
        // NodeValueComposite: empty value already trivially suppressed by
        // emit_one; non-empty composite values have no toolkit default
        // emission per v5 schema observation.
        (FlagKind::NodeValueComposite(_), FlagValue::NodeValueComposite { .. }) => false,
        // Unset / type-shape mismatch: not at default.
        _ => false,
    }
}

/// Shell flavor for `render_copy_command`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellFlavor {
    /// POSIX shells (bash, zsh, fish). `shlex::try_quote` does the heavy
    /// lifting; multi-token argv joined with single spaces.
    Posix,
    /// Windows `cmd.exe` + `CommandLineToArgvW`. Double-quote each arg;
    /// embedded `"` is encoded as `\"` per the `ArgvQuote` odd-backslash
    /// rule (see `cmd_quote` doc-comment for full rules — `""` is NOT a
    /// valid literal-`"` escape under `CommandLineToArgvW`). Lines joined
    /// with ` ^\r\n  ` for shell-side line continuation.
    /// R2 C-1 / R3 I-1 fold.
    WindowsCmd,
}

/// Assemble the argv for `<cli> <subcommand> [flags...]`.
///
/// Invariants per SPEC §6:
///   1. `argv[0]` = `schema.cli_name`. No absolute path.
///   2. `argv[1]` = `subcommand.name`.
///   3. Flag emission order = `subcommand.flags` declaration order.
///   4. Repeating flags emit one argv pair per FormState entry in form-
///      state order (slot-index ascending is the SlotEditor's responsibility;
///      Phase 3 wires it).
///   5. Empty / false / absent values are NOT emitted.
///   6. (v0.16.0 SPEC §6.10) Flags whose effective visibility is Hidden OR
///      Disabled are suppressed from emission. Required does not affect
///      emission (decorative marker only). Slot emission is unaffected by
///      visibility (slot values are not gated by §6.6/§6.9 rules in v1).
pub fn assemble_argv(
    schema: &Schema,
    subcommand: &SubcommandSchema,
    state: &crate::schema::FormState,
) -> Vec<String> {
    assemble_argv_with_secret_mask(schema, subcommand, state).0
}

/// Fixed redaction placeholder substituted for every secret VALUE token in
/// `render_copy_command_masked`. NOT shell-quoted (it is a display sentinel,
/// never run). Four `\u{2022}` bullets.
pub const SECRET_MASK: &str = "••••";

/// Like [`assemble_argv`], but also returns a parallel `mask: Vec<bool>` where
/// `mask[i] == true` iff `argv[i]` is a SECRET VALUE token. The mask is
/// correct-by-construction: every `argv.push` is paired with exactly one
/// `mask.push`, so `mask.len() == argv.len()` structurally.
///
/// A token is masked `true` at exactly the four secret-VALUE sources — the
/// same four `secrets::should_confirm_run` classifies: (1) secret Text flag
/// value; (2) secret slot row value token (`@N.subkey=value`, subkey
/// secret-bearing); (3) secret positional value; (4) `NodeValueComposite`
/// value token whose flag is secret-bearing OR whose node is secret-classed
/// (`node_type_is_secret`). All other tokens (cli/subcommand names, flag
/// names, PinValue tokens, non-secret values, sentinels) are masked `false`.
pub fn assemble_argv_with_secret_mask(
    schema: &Schema,
    subcommand: &SubcommandSchema,
    state: &crate::schema::FormState,
) -> (Vec<String>, Vec<bool>) {
    let mut argv: Vec<String> = Vec::new();
    let mut mask: Vec<bool> = Vec::new();
    argv.push(schema.cli_name.to_string());
    mask.push(false);
    argv.push(subcommand.name.to_string());
    mask.push(false);

    // v0.16.0 SPEC §6.10 visibility gate. Compute the per-frame visibility
    // override map once. `subcommand.conditional` is `Option<fn(&FormState)
    // -> FlagVisibility>`; absent fn → empty Vec (no overrides; every flag
    // defaults to Visible). First-rule-wins per `main.rs:391-394` semantics.
    //
    // v0.6.0 (Visibility no longer Copy due to v3 PinValue carrying
    // serde_json::Value): `visibility_of` clones rather than derefs.
    let vis: Vec<(&'static str, Visibility)> = subcommand
        .conditional
        .map(|f| f(state))
        .unwrap_or_default();
    let visibility_of = |name: &str| -> Visibility {
        vis.iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.clone())
            .unwrap_or(Visibility::Visible)
    };
    // v0.7.0 SPEC §6.10.4 v4-cycle: `Visibility::DisableOptions` is
    // SCHEMA-TIME ONLY — it greys out specific Dropdown options at
    // render time but does NOT join the suppress set. If `state.values`
    // already holds a now-disabled option value (carried over from a
    // prior frame), argv emits it; CLI rows 10/11 catch the residual.
    // `Visibility::PinValue` is also intentionally NOT in this set: it
    // REPLACES the user-typed value with the pinned value and emits
    // the pair (handled below in the per-flag emit branch).
    let suppresses = |v: &Visibility| matches!(v, Visibility::Hidden | Visibility::Disabled);

    for flag in subcommand.flags {
        // SPEC §6.10: Hidden AND Disabled suppress emission. Slot emission
        // (next branch) is exempt — slot values are not gated by §6.6/§6.9
        // rules in v1. The gate fires BEFORE the secret-flag + values
        // branches so a typed-then-mutex-disabled secret value (e.g., user
        // types --passphrase=foo then sets --passphrase-stdin) is NOT
        // emitted — fixing a pre-v0.16.0 latent bug where the value would
        // emit and trigger clap's `conflicts_with` rejection downstream.
        //
        // v0.6.0 §6.10.4 v3: PinValue REPLACES the user-typed value before
        // emission. Handled below the suppress check so PinValue beats
        // a stale state.values entry for the same flag.
        let flag_vis = visibility_of(flag.name);
        if flag.name != "--slot" || !subcommand.allows_slots {
            if suppresses(&flag_vis) {
                continue;
            }
            if let Visibility::PinValue { value } = &flag_vis {
                if let Some(rendered) = pin_value_to_argv_token(value) {
                    argv.push(flag.name.to_string());
                    mask.push(false);
                    argv.push(rendered);
                    mask.push(false); // pinned values are non-secret (only --account=0 lives today)
                }
                // PinValue is exclusive with the normal emit path — even
                // when the JSON value can't be rendered we suppress the
                // user's stale state.values entry.
                continue;
            }
        }
        // SPEC §6.4: when allows_slots == true, the `--slot` flag is
        // emitted from SlotState (not from `values`), in slot-index
        // ascending order. The schema still carries a `--slot` FlagSchema
        // entry so the schema-mirror flag-name test sees it.
        if flag.name == "--slot" && subcommand.allows_slots {
            // v0.6.1 P3 #5A defense-in-depth: the visibility gate above is
            // wrapped in `if flag.name != "--slot" || !subcommand.allows_slots`
            // and so does NOT run for --slot on slot-bearing subcommands. A
            // future toolkit rule that targets --slot with PinValue would
            // silently fall through to this slot-emission branch and emit
            // malformed argv (pin_value's single-value emission semantic
            // doesn't map onto the multi-row @N.subkey=value slot grammar).
            // debug_assert fails loud in dev / CI debug-profile; release-
            // mode `if-suppress` is the defensive net. A future cycle that
            // legitimately wants pin_value-on-slot semantics MUST remove
            // this debug_assert and replace with the new design.
            // Tracks FOLLOWUP `gui-pin-value-effect-on-slot-flag-gap`.
            debug_assert!(
                !matches!(flag_vis, Visibility::PinValue { .. }),
                "pin_value on --slot is unspecified — see FOLLOWUP \
                 gui-pin-value-effect-on-slot-flag-gap. Encountered \
                 pin_value for --slot on subcommand `{}`",
                subcommand.name,
            );
            if matches!(flag_vis, Visibility::PinValue { .. }) {
                continue;
            }
            // Slot tokens come in pairs ["--slot", "@N.subkey=value"]; the
            // value token is secret iff its subkey is secret-bearing (Phrase /
            // Seedqr / Entropy / Ms1 / Wif / Xprv). `to_slot_argv_masked`
            // carries the per-token bit (the "--slot" token is always false).
            for (token, secret) in state.slots.to_slot_argv_masked() {
                argv.push(token);
                mask.push(secret);
            }
            continue;
        }
        // SPEC §3 / v0.2 Phase B.1 + v0.31.1 repeating-secrets fold:
        // secret-flag branch, KIND-GATED to mirror the widget dispatch.
        //
        // SUPERSESSION NOTE (v0.31.1): the v0.3 fold documented here an
        // intent to route REPEATING secrets through `state.values` like
        // non-secret repeating — but the widget layer never wrote secret
        // Text rows into `state.values` (its secret dispatch routed every
        // secret Text flag, repeating or not, to a single `secret_widgets`
        // entry), so the values-read below was a DEAD source and live
        // forms emitted NOTHING for `--ms1` / `--share` (FOLLOWUP
        // `repeating-secret-flags-never-reach-argv`). v0.31.1 inverts the
        // fix direction: `secret_widgets` became `BTreeMap<String,
        // Vec<SecretLineEdit>>` (scalar = the 1-element vec) and the
        // assembler reads the rows from it — per-row secrets STAY in
        // `secret_widgets`, never entering `state.values`, so the
        // never-persist invariant remains TYPE-level (#[serde(skip)]).
        // Trade-off posture unchanged: the transient plain `String` copies
        // pushed into argv exist exactly as the pre-v0.31.1 scalar
        // `as_string()` path always produced.
        //
        // R0-r1 C1: the branch MIRRORS the widget dispatch (kind-gated),
        // because flag_is_secret is kind-BLIND while the widget routes only
        // Text secrets to secret_widgets:
        // - Text → the secret_widgets vec, unconditional `continue` (a
        //   values-synthesized Text-secret entry emits NOTHING).
        // - NodeValueComposite (seed-xor-combine --share) → falls through
        //   to the generic values paths (render_repeating writes
        //   state.values; emit_one's composite arm emits; values-routed
        //   composites are redaction-covered at persist — SPEC §3).
        // - Boolean *-stdin secrets → `continue`, preserving today's
        //   no-emit (the old kind-blind branch ate them; FOLLOWUP
        //   `boolean-stdin-secret-toggles-never-emit` tracks whether they
        //   should emit).
        if crate::secrets::flag_is_secret(flag) {
            if matches!(flag.kind, FlagKind::Text) {
                if let Some(rows) = state.secret_widgets.get(flag.name) {
                    for w in rows {
                        // scalar = 1-element vec; row order = vec order =
                        // visual order (argv order preserved). Empty rows
                        // (added-but-blank) emit nothing.
                        if !w.is_empty() {
                            // B.1 R1 I-2 fold: as_string() returns
                            // Zeroizing<String> directly; the wrap is
                            // type-level rather than caller-applied.
                            let value = w.as_string();
                            argv.push(flag.name.to_string());
                            mask.push(false);
                            argv.push(value.as_str().to_string());
                            mask.push(true); // secret Text value
                        }
                    }
                }
                continue;
            }
            if matches!(flag.kind, FlagKind::NodeValueComposite(_)) {
                // fall through to the generic values paths (seed-xor
                // --share keeps emitting).
            } else {
                continue; // Boolean *-stdin secrets: preserve today's no-emit.
            }
        }
        if flag.repeating {
            for (_, value) in state.values.iter().filter(|(k, _)| k == flag.name) {
                emit_one(flag, value, &mut argv, &mut mask);
            }
        } else if let Some((_, value)) = state.values.iter().find(|(k, _)| k == flag.name) {
            emit_one(flag, value, &mut argv, &mut mask);
        }
    }

    // Positional args (Phase 6; v0.34.0 audit-I5 split) — emit at the end
    // of argv. SECRET positionals (every table has ≤1 entry) emit from
    // their `secret_widgets["positional:<name>"]` rows in row order; any
    // stale `state.positionals` content is IGNORED for them (the widget
    // path is authoritative — mirrors the v0.31.1 kind-gated flag
    // discipline). Non-secret positionals keep the `state.positionals`
    // path, skipping empty strings (SPEC §6.7 parity).
    if let Some(pos) = subcommand.positional_args.iter().find(|p| p.secret) {
        if let Some(rows) = state.secret_widgets.get(&format!("positional:{}", pos.name)) {
            for w in rows {
                if !w.is_empty() {
                    let value = w.as_string();
                    argv.push(value.as_str().to_string());
                    mask.push(true); // secret positional value
                }
            }
        }
    } else {
        for pos in &state.positionals {
            if !pos.is_empty() {
                argv.push(pos.clone());
                mask.push(false);
            }
        }
    }

    debug_assert_eq!(
        argv.len(),
        mask.len(),
        "secret-mask length must track argv length — a push site is missing its mask.push"
    );
    (argv, mask)
}

/// SPEC §6.10.4 v3 PinValue emission helper. Renders the pinned
/// `serde_json::Value` as the argv string. Returns `None` for value shapes
/// that have no clean string representation for the current FlagKind
/// vocabulary (Object / Array / Null); the caller then suppresses emission
/// entirely. v0.6.0 ships the row-12 use case (`pin_value(0)` on
/// `--account`, a Number flag); String / Bool primitives are handled for
/// future pin coercions over Dropdown / Text per the toolkit-side
/// "permissive value" doc.
fn pin_value_to_argv_token(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::String(s) => Some(s.clone()),
        // Bool: emit "true"/"false" as the literal token. Boolean FlagKind
        // pins are uncommon (booleans typically use presence semantics) but
        // the toolkit grammar permits them.
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Null
        | serde_json::Value::Object(_)
        | serde_json::Value::Array(_) => None,
    }
}

// v0.39.0: `mask` tracks `argv` 1:1 — every `argv.push` here pairs a
// `mask.push`. Only the `NodeValueComposite` value token can be secret in
// this function (secret Text + secret positionals are handled in the caller's
// secret branch BEFORE reaching emit_one; secret non-Text/non-Composite flags
// are Boolean-suppressed). Its bit is `flag_is_secret(flag) ||
// node_type_is_secret(node)` — covering both the secret flag `--share` and the
// value-dependent `--from phrase=<seed>` (flag non-secret, NODE secret).
fn emit_one(flag: &FlagSchema, value: &FlagValue, argv: &mut Vec<String>, mask: &mut Vec<bool>) {
    // v0.10.0 B.3 (D33): default-value suppression. When the user's typed
    // value equals the toolkit-declared default for this flag, omit the
    // flag from argv entirely — the toolkit will pick up the same value
    // from its own defaults at parse time, so emitting it explicitly is
    // noise. See `is_at_default` doc for the per-FlagKind compare table.
    if is_at_default(flag, value) {
        return;
    }
    match (&flag.kind, value) {
        (FlagKind::Text, FlagValue::Text(v))
            if !v.is_empty() => {
                argv.push(flag.name.to_string());
                mask.push(false);
                argv.push(v.clone());
                mask.push(false);
            }
        (FlagKind::Number { .. }, FlagValue::Number(n)) => {
            argv.push(flag.name.to_string());
            mask.push(false);
            argv.push(n.to_string());
            mask.push(false);
        }
        (FlagKind::Dropdown(_), FlagValue::Dropdown(v))
            if !v.is_empty() => {
                argv.push(flag.name.to_string());
                mask.push(false);
                argv.push(v.clone());
                mask.push(false);
            }
        (FlagKind::Boolean, FlagValue::Boolean(true)) => {
            argv.push(flag.name.to_string());
            mask.push(false);
        }
        (FlagKind::Boolean, FlagValue::Boolean(false)) => {
            // Omitted.
        }
        (FlagKind::Range, FlagValue::Range(a, b)) => {
            argv.push(flag.name.to_string());
            mask.push(false);
            argv.push(format!("{},{}", a, b));
            mask.push(false);
        }
        (FlagKind::Timestamp, FlagValue::Timestamp(t)) => {
            argv.push(flag.name.to_string());
            mask.push(false);
            argv.push(match t {
                TimestampValue::Now => "now".to_string(),
                TimestampValue::Unix(n) => n.to_string(),
            });
            mask.push(false);
        }
        (
            FlagKind::NodeValueComposite(_),
            FlagValue::NodeValueComposite { node, value },
        )
            // SPEC §6.7 R3 I-3 fold: empty value → omit (matches Text/Path
            // empty-value rule and avoids upstream's "value is empty"
            // rejection from `parse_from_input` at convert.rs:128-132).
            if !value.is_empty() => {
                argv.push(flag.name.to_string());
                mask.push(false);
                argv.push(format!("{}={}", node, value));
                // v0.39.0: secret iff the flag is secret-bearing (--share) OR
                // the node is a secret class (--from phrase=<seed>).
                mask.push(
                    crate::secrets::flag_is_secret(flag)
                        || crate::secrets::node_type_is_secret(node),
                );
            }
        (FlagKind::TaggedOrIndexed(_), FlagValue::TaggedOrIndexed(tv)) => {
            argv.push(flag.name.to_string());
            mask.push(false);
            argv.push(match tv {
                TaggedOrIndexedValue::Tag(t) => t.clone(),
                TaggedOrIndexedValue::Indexed(n) => format!("@{}", n),
            });
            mask.push(false);
        }
        (FlagKind::Path { stdio_sentinel }, FlagValue::Path(p)) => {
            if p.is_empty() {
                return;
            }
            if p == "-" && !stdio_sentinel {
                // Schema explicitly disallows stdin sentinel for this flag;
                // the GUI form should have rejected it at validation time.
                // Defensive guard: skip emission.
                return;
            }
            argv.push(flag.name.to_string());
            mask.push(false);
            argv.push(p.clone());
            mask.push(false);
        }
        // Type-shape mismatch: state carries the wrong FlagValue for this
        // flag's FlagKind. Phase 2 trusts the form widget to maintain the
        // invariant; we silently skip rather than panic. Phase 5+ may add
        // a debug assertion if useful.
        _ => {}
    }
}

/// Render a shell-quoted single-line command for display copy-paste.
/// Per SPEC §6.6 this output is for the user's eyes only — it is NEVER
/// re-parsed or used to spawn the subprocess.
pub fn render_copy_command(argv: &[String], flavor: ShellFlavor) -> String {
    match flavor {
        ShellFlavor::Posix => argv
            .iter()
            .map(|s| posix_quote(s))
            .collect::<Vec<_>>()
            .join(" "),
        ShellFlavor::WindowsCmd => argv
            .iter()
            .map(|s| cmd_quote(s))
            .collect::<Vec<_>>()
            .join(" ^\r\n  "),
    }
}

/// v0.39.0 — DISPLAY-ONLY variant of [`render_copy_command`] that substitutes
/// the fixed [`SECRET_MASK`] placeholder (un-quoted — it is a sentinel, never
/// run) for every token whose `mask` bit is `true`, shell-quoting the rest as
/// usual. The masked output is NEVER copied to the clipboard or run — only the
/// real [`render_copy_command`] feeds Run and the deliberate-reveal copy.
///
/// `mask` is the parallel vector from [`assemble_argv_with_secret_mask`];
/// `mask.len()` is expected to equal `argv.len()` (a shorter mask defaults the
/// tail to non-secret — fail-open is acceptable here only because the assembler
/// guarantees equal length, asserted in debug).
pub fn render_copy_command_masked(argv: &[String], mask: &[bool], flavor: ShellFlavor) -> String {
    debug_assert_eq!(
        argv.len(),
        mask.len(),
        "render_copy_command_masked: mask/argv length mismatch — a secret token could render cleartext"
    );
    let render = |i: usize, s: &String| -> String {
        if mask.get(i).copied().unwrap_or(false) {
            SECRET_MASK.to_string()
        } else {
            match flavor {
                ShellFlavor::Posix => posix_quote(s),
                ShellFlavor::WindowsCmd => cmd_quote(s),
            }
        }
    };
    let parts: Vec<String> = argv.iter().enumerate().map(|(i, s)| render(i, s)).collect();
    match flavor {
        ShellFlavor::Posix => parts.join(" "),
        ShellFlavor::WindowsCmd => parts.join(" ^\r\n  "),
    }
}

/// POSIX shell quoting. Wraps `shlex::try_quote` and falls back to a manual
/// single-quote encoding for the (rare) inputs `shlex` rejects (interior
/// NULs — not expected in clap argv but defended against).
///
/// `pub` since v0.32.0 P3: the tree-mode POSIX pipeline copy
/// (`tree_form::posix_pipeline_command`) quotes the spec JSON with the
/// same machinery the argv copy uses — one quoting implementation, no
/// hand-rolled second path.
pub fn posix_quote(s: &str) -> String {
    match shlex::try_quote(s) {
        Ok(cow) => cow.into_owned(),
        Err(_) => {
            // Fallback: encode each `'` as `'\''` and wrap in single quotes.
            let escaped = s.replace('\'', "'\\''");
            format!("'{}'", escaped)
        }
    }
}

/// Windows quoting compatible with `CommandLineToArgvW` (the universal
/// Win32 parser used by cmd.exe → CreateProcess → target-binary startup).
/// Implements Daniel Colascione's canonical `ArgvQuote` rules from the
/// Microsoft "Everyone quotes command line arguments the wrong way" post:
///
/// 1. Wrap the token in `"…"`.
/// 2. For each run of `n` consecutive `\` followed by a literal `"`
///    (interior): emit `2n+1` `\` + `"`. The odd-count rule produces a
///    literal `"` and preserves in-quotes mode.
/// 3. For each run of `n` consecutive `\` at end-of-string (before the
///    close-`"`): emit `2n` `\`. The even-count rule produces `n`
///    literal `\` and toggles in-quotes mode (closing the wrapper).
/// 4. For each interior run of `n` consecutive `\` (not followed by a
///    `"`): emit `n` `\` (pass-through).
/// 5. A bare `"` (no preceding `\`) is encoded as `\"` (n=0 → 1 `\`).
///
/// R1 I-1 / R2 C-1 fold: prior implementations used `""` for embedded
/// `"`, which `CommandLineToArgvW` does NOT recognize as a literal-`"`
/// escape — it parses as (close-quote, reopen-quote) and the literal `"`
/// is lost from the resulting argv. The odd-backslash rule is the only
/// universal encoding.
fn cmd_quote(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' {
            let mut n = 0usize;
            while i < chars.len() && chars[i] == '\\' {
                n += 1;
                i += 1;
            }
            if i == chars.len() {
                // 2b: end of input; double the run so close-`"` is
                // unambiguously the quote-close, not the escape target.
                for _ in 0..(n * 2) {
                    out.push('\\');
                }
            } else if chars[i] == '"' {
                // 2a: emit 2n+1 backslashes + the literal `"`.
                for _ in 0..(n * 2 + 1) {
                    out.push('\\');
                }
                out.push('"');
                i += 1;
            } else {
                // 2c: interior; pass through.
                for _ in 0..n {
                    out.push('\\');
                }
            }
        } else if chars[i] == '"' {
            // Rule 3: lone literal `"` → `\"`.
            out.push('\\');
            out.push('"');
            i += 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out.push('"');
    out
}
