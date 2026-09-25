#!/usr/bin/env python3
"""T5 for the Rust implementation (DESIGN §A9): mutate the planner, the lookalike
predicate, the CR/LF refusal and the runner's delivery, and show a test goes red.

A mutation counts as KILLED only when:
  - it APPLIED (the exact snippet occurs once in the file),
  - the tree still BUILDS (a compile error is reported as INVALID, never a kill),
  - the mutated line RAN (each mutation carries a marker that writes $MUTATION_MARK),
  - and a test FAILED BY ASSERTION (nextest reports FAIL; no build error).
A no-op CONTROL (marker only) must SURVIVE, or the harness is broken and this exits 1.

Each file is restored byte-for-byte after its mutation (checked by sha256). Run from the
repo root with the pinned binaries on PATH and MNEMONIC_BIN set:

  MNEMONIC_BIN=mnemonic python3 scripts/secret-channels-mutations.py [NAME-FILTER]

Writes design/measurements/secret-channels/rust_mutations.out."""
import hashlib, os, subprocess, sys, tempfile

MARK = 'if let Some(p) = std::env::var_os("MUTATION_MARK") { let _ = std::fs::write(p, b"ran"); }'
PLANNER, LOOK, DATA, RUNNER = ("src/form/channels/mod.rs", "src/form/channels/lookalike.rs",
                               "src/form/channels/data.rs", "src/runner.rs")
PURE = ["secret_channels_t1", "secret_channels_t6", "secret_channels_t8"]
PARTB = ["part_b_forms", "secret_channels_t9", "f679_argv_secret_admission"]
COND, SCHEMA_MOD = "src/form/conditional.rs", "src/schema/mod.rs"
ALL = PURE + ["secret_channels_t7", "secret_channels_t2", "secret_channels_t3prime"]
SKIP_SLOW = "not test(exhaustively)"

# (name, file, old, new, targets, filterset, expect_killed)
M = [
    ("CONTROL: no-op (must SURVIVE)", PLANNER,
     "let fd_on = policy.fd_channel_on.iter().any(|o| o == os);\n    let mut avail",
     "let fd_on = { " + MARK + " policy.fd_channel_on.iter().any(|o| o == os) };\n    let mut avail",
     ALL, None, False),
    # ── the planner's assignment ──
    ("assignment: step 3 never picks EnvRef", PLANNER,
     "let mut c = pick(ch, &[ChannelKind::EnvRef]);",
     "let mut c: Option<&Channel> = { " + MARK + " None };", ALL, SKIP_SLOW, True),
    ("assignment: step 2 takes the LAST toggle", PLANNER,
     "        for (i, ch) in avail.iter().enumerate() {\n            if let Some(c) = pick(ch, &[ChannelKind::StdinToggle]) {",
     "        for (i, ch) in { " + MARK + " avail.iter().enumerate().rev() } {\n            if let Some(c) = pick(ch, &[ChannelKind::StdinToggle]) {",
     ALL, SKIP_SLOW, True),
    ("assignment: two-stdin refusal off", PLANNER,
     "if forced.len() > 1 {", "if { " + MARK + " false } {", ALL, SKIP_SLOW, True),
    ("assignment: env name off by one", PLANNER,
     'b.env = Some(format!("{env_prefix}{i}"));',
     'b.env = Some({ ' + MARK + ' format!("{env_prefix}{}", i + 1) });', ALL, SKIP_SLOW, True),
    ("assignment: lenient filter off", PLANNER,
     "if !s.value.values().iter().all(|v| is_clean(v)) {", "if { " + MARK + " false } {", ALL, SKIP_SLOW, True),
    ("assignment: no-table-entry off", PLANNER,
     "if !table.contains_key(&s.key) {",
     "if { " + MARK + " false } && !table.contains_key(&s.key) {", PURE, SKIP_SLOW, True),
    # ── the lookalike predicate (R4 NI8) ──
    ("NI8: lookalike refusal off", PLANNER,
     "if looks_like_channel(policy, v) {", "if { " + MARK + " false } {", ALL, SKIP_SLOW, True),
    ("NI8: normalization without whitespace strip", LOOK,
     "casefold(kept.trim())", "{ " + MARK + " casefold(&kept) }", PURE, None, True),
    ("NI8: normalization without case-fold", LOOK,
     "casefold(kept.trim())", "{ " + MARK + " kept.trim().to_string() }", PURE, None, True),
    ("NI8: normalization without Cc/Cf removal", LOOK,
     ".filter(|&c| !is_cc(c) && !is_cf(c))", ".filter(|&_c| { " + MARK + " true })", PURE, None, True),
    ("NI8: normalization without NFKC", LOOK,
     "let nfkc: String = v.nfkc().collect();",
     "let nfkc: String = { " + MARK + " let _ = v.nfkc(); v.to_string() };", PURE, None, True),
    # ── the CR/LF refusal (R4 NI9) ──
    ("NI9: trailing CR/LF refusal off", PLANNER,
     "if policy.refuse_trailing_cr_lf && (v.ends_with('\\r') || v.ends_with('\\n')) {",
     "if { " + MARK + " false } {", ALL, SKIP_SLOW, True),
    ("NI9: only LF refused (CR admitted)", PLANNER,
     "(v.ends_with('\\r') || v.ends_with('\\n'))", "({ " + MARK + " v.ends_with('\\n') })", PURE, SKIP_SLOW, True),
    ("NI9: the ONE strip-one rule becomes strip-all", DATA,
     'if let Some(s) = raw.strip_suffix("\\r\\n") {\n                    s.to_string()',
     'if raw.ends_with(\'\\n\') {\n                    ' + MARK + ' raw.trim_end_matches([\'\\r\', \'\\n\']).to_string()',
     PURE, SKIP_SLOW, True),
    # ── C1 ──
    ("C1: `-` not refused", PLANNER, 'if v == "-" {', "if { " + MARK + " false } {", PURE, SKIP_SLOW, True),
    ("C1: per-input rule ignored (always verbatim)", PLANNER,
     "let target = Zeroizing::new(rule.unwrap_or(EnvRule::Verbatim).apply(&raw));",
     "let target = { " + MARK + " let _ = rule; Zeroizing::new(EnvRule::Verbatim.apply(&raw)) };",
     PURE, SKIP_SLOW, True),
    ("Nit: NUL allowed", PLANNER, "if vals.iter().any(|v| v.contains('\\0')) {",
     "if { " + MARK + " false } {", PURE, SKIP_SLOW, True),
    # ── the runner's delivery ──
    ("delivery: swap two secrets (data from the next source)", PLANNER,
     "let val = &planned.resolved[i];",
     "let val = { " + MARK + " &planned.resolved[(i + 1) % planned.resolved.len()] };", ALL, SKIP_SLOW, True),
    ("delivery: private path sends the TYPED text, not the resolved bytes", PLANNER,
     "        Ok(bindings) => Ok(Planned {\n            bindings,\n            provenance: prov,\n            resolved: res.into_iter().map(|s| s.value).collect(),",
     "        Ok(bindings) => Ok(Planned {\n            bindings,\n            provenance: prov,\n            resolved: { " + MARK + " let _ = res; sources.iter().map(|s| s.value.clone()).collect() },",
     ALL, SKIP_SLOW, True),
    ("delivery: interim path sends the TYPED text (R2's mutation)", PLANNER,
     "let resolved = res.into_iter().map(|s| s.value).collect();",
     "let resolved = { " + MARK + " let _ = res; sources.iter().map(|s| s.value.clone()).collect() };",
     ALL, SKIP_SLOW, True),
    ("delivery: terminator dropped", PLANNER,
     'SourceValue::One(v) => Zeroizing::new(format!("{}{}", v.as_str(), terminator)),',
     'SourceValue::One(v) => { ' + MARK + ' let _ = terminator; Zeroizing::new(v.as_str().to_string()) }',
     ALL, SKIP_SLOW, True),
    ("delivery: stdin written from the wrong binding", PLANNER,
     "                set(&mut replace, first, vec![]);\n                stdin = Some(Zeroizing::new(data.as_bytes().to_vec()));",
     "                set(&mut replace, first, vec![]);\n                stdin = Some({ " + MARK + " Zeroizing::new(planned.resolved[(i + 1) % planned.resolved.len()].channel_data(&b.terminator).as_bytes().to_vec()) });",
     ALL, SKIP_SLOW, True),
    ("delivery: env scrub dropped", RUNNER,
     "if k.as_encoded_bytes().starts_with(reserved_prefix.as_bytes()) {",
     "if { " + MARK + " false } && k.as_encoded_bytes().starts_with(reserved_prefix.as_bytes()) {",
     ["secret_channels_t7"], None, True),
    ("delivery: pipe write end left open", RUNNER,
     "drop(writer); // the write end is closed BEFORE spawn",
     "{ " + MARK + " std::mem::forget(writer); }", ["secret_channels_t7"], None, True),
    ("delivery: the argv admission restored on the private path", PLANNER,
     "if interim && assembled.declares_allow_argv_secret && !planned.bindings.is_empty() {",
     "if { " + MARK + " true } && assembled.declares_allow_argv_secret && !planned.bindings.is_empty() {",
     ALL, SKIP_SLOW, True),
    # ── Part B (the five forms) ──
    ("B5: `all kinds` emits its label as --kind", SCHEMA_MOD,
     "value == crate::schema::ms::HASHLOCK_KIND_ALL",
     "({ " + MARK + " false }) && value == crate::schema::ms::HASHLOCK_KIND_ALL", PARTB, None, True),
    ("B5: Run no longer blocked on (choose)", COND,
     'let kind = state.dropdown_value("--kind").unwrap_or("");',
     'let kind = { ' + MARK + ' let _ = state; "sha256" };', PARTB, None, True),
    ("B5: source exclusivity off", COND,
     '&["positional:ms1", "--hashlock-phrase", "--hex", "--in", "--random"],',
     '&{ ' + MARK + ' ["positional:ms1"] },', PARTB, None, True),
    ("M1: Copy no longer gated on (choose)", "src/form/channels/copy.rs",
     "if let Some(b) = crate::form::conditional::run_blocker(schema.cli_name, sub.name, state) {",
     "if let Some(b) = ({ " + MARK + " None::<&str> }) {", PARTB + ["fold1_review"], None, True),
    ("M2: Copy reveal label never raised", "src/form/channels/copy.rs",
     ".map(|p| p.mask.iter().any(|&m| m))",
     ".map(|p| { " + MARK + " let _ = p; false })", ["fold1_review"], None, True),
    ("B2-B4: pasted xprv no longer masked", "src/secrets.rs",
     ".any(crate::form::tree_model::is_xprv_like)",
     ".any(|t| { " + MARK + " let _ = t; false })", PARTB, None, True),
]


def sha(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest()


def main():
    flt = sys.argv[1] if len(sys.argv) > 1 else ""
    rows, bad = [], []
    for name, f, old, new, targets, fs, expect in M:
        if flt and flt not in name:
            continue
        orig = open(f).read()
        before = sha(f)
        n = orig.count(old)
        if n != 1:
            rows.append((name, f"NOT APPLIED ({n} matches)", "-"))
            bad.append(name)
            continue
        mark = tempfile.mktemp(prefix="mutmark-")
        try:
            open(f, "w").write(orig.replace(old, new))
            b = subprocess.run(["cargo", "build", "--tests", "-q"], capture_output=True, text=True)
            if b.returncode:
                rows.append((name, "INVALID (does not build)", "-"))
                bad.append(name)
                continue
            cmd = ["cargo", "nextest", "run", "--no-fail-fast"] + sum([["--test", t] for t in targets], [])
            if fs:
                cmd += ["-E", fs]
            r = subprocess.run(cmd, capture_output=True, text=True, env=dict(os.environ, MUTATION_MARK=mark),
                               timeout=1800)
            out = r.stdout + r.stderr
            failed = sorted({l.split()[-1] for l in out.splitlines() if l.strip().startswith("FAIL [")})
            ran = os.path.exists(mark)
            killed = r.returncode != 0 and bool(failed) and "error[E" not in out
            verdict = "KILLED" if killed else "survived"
            rows.append((name, verdict + ("" if ran else " (mutated line NEVER RAN)"), ", ".join(failed[:3])))
            if killed != expect or not ran:
                bad.append(name)
        finally:
            open(f, "w").write(orig)
            if os.path.exists(mark):
                os.remove(mark)
            assert sha(f) == before, f"{f} not restored"
    lines = ["| mutation | verdict | first failing tests |", "|---|---|---|"]
    lines += [f"| {a} | {b} | {c} |" for a, b, c in rows]
    killed = sum(1 for _, v, _ in rows if v.startswith("KILLED"))
    # review N2: the denominator counts the non-CONTROL rows actually run
    ran_mutations = sum(1 for n, _, _ in rows if not n.startswith("CONTROL"))
    lines.append("")
    lines.append(f"{killed}/{ran_mutations} mutations killed by assertion with the mutated line run; "
                 f"control {'not run (filtered)' if len(rows) == ran_mutations else ('survived' if not any(n.startswith('CONTROL') for n in bad) else 'DID NOT SURVIVE')}; "
                 f"problems: {bad or 'none'}")
    text = "\n".join(lines) + "\n"
    print(text)
    if not flt:
        open("design/measurements/secret-channels/rust_mutations.out", "w").write(text)
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
