//! Drift gate: the committed canonical-fallback `SECRET_*` arrays in
//! `build.rs` MUST stay equal to the upstream `is_secret_bearing()` impl
//! match-arm sets.
//!
//! Why this exists: when `cargo install --git mnemonic-gui` is run by an
//! end user, `build.rs` cannot resolve the upstream mnemonic-toolkit
//! source tree (it's not adjacent in the cargo-install sandbox). In
//! v0.3.0..v0.3.2 the fallback emitted `&[]` for both arrays, causing
//! `persistence::redact_for_persistence` to silently no-op on slot rows
//! and NodeValueComposite entries — BIP-39 phrases persisted to disk in
//! plaintext. v0.3.3 fixes this by embedding canonical fallback values
//! in `build.rs` (`CANONICAL_FALLBACK_NODE_TYPES` +
//! `CANONICAL_FALLBACK_SLOT_SUBKEYS`).
//!
//! This test asserts those two committed arrays match the upstream
//! `is_secret_bearing()` impls byte-for-byte (modulo set-equality). The
//! test is `#[ignore]`-gated by default: it requires
//! `MNEMONIC_GUI_UPSTREAM_ROOT` (the same env var `build.rs` uses) to
//! locate the upstream files. Schema-mirror CI provides it; default
//! `cargo test` skips it; running `cargo test --test
//! secrets_canonical_fallback -- --include-ignored` from a checkout
//! with the env var set runs it.

use std::collections::HashSet;
use std::path::PathBuf;

/// Re-parse the committed canonical-fallback arrays from `build.rs` by
/// reading the source file at compile time and extracting the relevant
/// `const` items. We can't `include!()` build.rs (it has a `fn main`),
/// so we parse it the same way build.rs parses upstream.
fn read_committed_fallback() -> (HashSet<String>, HashSet<String>) {
    let build_rs_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("build.rs");
    let src = std::fs::read_to_string(&build_rs_path)
        .expect("build.rs must be readable from CARGO_MANIFEST_DIR");

    let nodes = extract_str_const(&src, "CANONICAL_FALLBACK_NODE_TYPES");
    let subkeys = extract_str_const(&src, "CANONICAL_FALLBACK_SLOT_SUBKEYS");
    (nodes, subkeys)
}

/// Extract a `const NAME: &[&str] = &["a", "b", ...]` array from a Rust
/// source string by simple bracket-balanced scanning. Not a full Rust
/// parser, but precise enough for the two declarations we control.
fn extract_str_const(src: &str, name: &str) -> HashSet<String> {
    let needle = format!("const {} : &[&str] = &[", name).replace(' ', "");
    // Normalize whitespace in src to make the match position-independent.
    let needle_alt = format!("const {}: &[&str] = &[", name);
    let start = src
        .find(&needle_alt)
        .or_else(|| {
            let cmp: String = src.chars().filter(|c| !c.is_whitespace()).collect();
            cmp.find(&needle).map(|_| {
                // Fall back to brute-force: locate `const NAME` and then `[`.
                let n = src.find(&format!("const {}", name)).unwrap();
                src[n..].find('[').map(|i| n + i + 1).unwrap()
            })
        })
        .unwrap_or_else(|| panic!("could not find `const {}` in build.rs", name));
    let body_start = start + needle_alt.len();
    let body_end = body_start
        + src[body_start..]
            .find(']')
            .expect("array body must close with ]");
    let body = &src[body_start..body_end];

    let mut out = HashSet::new();
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            let mut s = String::new();
            for ch in chars.by_ref() {
                if ch == '"' {
                    break;
                }
                s.push(ch);
            }
            out.insert(s);
        }
    }
    out
}

/// Walk an upstream `impl` block for `target_type` and return the
/// `as_str()` values of every variant in the `is_secret_bearing()`
/// match-arm set. Simple line-based parse; matches the shape the
/// upstream sources actually use.
fn extract_upstream_secret_set(file: &str, target_type: &str) -> HashSet<String> {
    let src = std::fs::read_to_string(file)
        .unwrap_or_else(|e| panic!("read {}: {}", file, e));

    // Step 1: extract variant idents from the
    // `is_secret_bearing(...) -> bool { matches!(self, A | B | ...) }`
    // shape. The upstream uses `matches!`, not the multi-arm form.
    let head = format!("impl {} {{", target_type);
    let impl_start = src.find(&head).expect("impl block must exist");
    let impl_end = impl_start
        + src[impl_start..]
            .find("\n}\n")
            .expect("impl block close must exist");
    let impl_body = &src[impl_start..impl_end];

    // Find the `pub fn is_secret_bearing` fn body (not `is_argv_secret_bearing`,
    // which is named with a non-space prefix so the literal needle does not
    // match it). Then walk forward to the next `{...}` block — that's the
    // function body. The body contains `matches!(self, A | B | ...)`; we
    // tolerate whitespace between `matches!(` and `self,` (the upstream uses
    // a multi-line layout, e.g. `matches!(\n    self,\n    Self::Phrase...`).
    let needle = "pub fn is_secret_bearing";
    let fn_start = impl_body.find(needle).expect("is_secret_bearing fn");
    let after = &impl_body[fn_start..];
    let body_open = after.find('{').expect("fn body open brace");
    let body_close = find_matching_close(&after[body_open..])
        .expect("fn body close brace");
    let body = &after[body_open..body_open + body_close];
    let matches_open = body
        .find("matches!(")
        .expect("matches! call in is_secret_bearing");
    let after_matches_open = matches_open + "matches!(".len();
    // The first comma after `matches!(` separates `self` from the patterns.
    let comma_idx = body[after_matches_open..]
        .find(',')
        .expect("comma after self in matches!");
    let alts_start = after_matches_open + comma_idx + 1;
    let alts_close = find_matching_close_paren(&body[matches_open..])
        .expect("matches! close paren");
    let alts_end = matches_open + alts_close;
    let alts = &body[alts_start..alts_end];

    let mut variants: Vec<String> = Vec::new();
    for tok in alts.split('|') {
        // tok looks like " Self::Phrase " or " NodeType::Wif ".
        let tok = tok.trim();
        if let Some(idx) = tok.rfind("::") {
            let v = tok[idx + 2..].trim().to_string();
            variants.push(v);
        }
    }

    // Step 2: extract the `as_str()` map.
    let as_start = impl_body.find("pub fn as_str").expect("as_str fn");
    let as_body = &impl_body[as_start..];
    let mut as_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for line in as_body.lines() {
        let line = line.trim();
        if let Some(arrow_idx) = line.find("=>") {
            let pat = line[..arrow_idx].trim();
            if let Some(idx) = pat.rfind("::") {
                let v = pat[idx + 2..].trim().to_string();
                let rhs = line[arrow_idx + 2..].trim();
                // rhs looks like `"phrase",`
                let rhs = rhs.trim_end_matches(',').trim();
                if rhs.starts_with('"') && rhs.ends_with('"') {
                    let s = rhs[1..rhs.len() - 1].to_string();
                    as_map.insert(v, s);
                }
            }
        }
    }

    variants
        .into_iter()
        .map(|v| {
            as_map
                .get(&v)
                .cloned()
                .unwrap_or_else(|| panic!("variant {} missing from as_str", v))
        })
        .collect()
}

fn upstream_root() -> Option<PathBuf> {
    std::env::var_os("MNEMONIC_GUI_UPSTREAM_ROOT").map(PathBuf::from)
}

/// Given a string starting with `{`, return the index (relative to the
/// start of `s`) of the matching `}`, accounting for nested braces.
/// Returns `None` if no matching close exists.
fn find_matching_close(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    if bytes.first() != Some(&b'{') {
        return None;
    }
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// Given a string starting with `(`, return the index (relative to the
/// start of `s`) of the matching `)`, accounting for nesting.
fn find_matching_close_paren(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let open_pos = bytes.iter().position(|&b| b == b'(')?;
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate().skip(open_pos) {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[test]
#[ignore = "drift gate: needs MNEMONIC_GUI_UPSTREAM_ROOT; schema-mirror CI runs with --include-ignored"]
fn committed_fallback_node_types_match_upstream() {
    let Some(root) = upstream_root() else {
        panic!("MNEMONIC_GUI_UPSTREAM_ROOT not set — cannot validate");
    };
    let convert_rs = root.join("crates/mnemonic-toolkit/src/cmd/convert.rs");
    let upstream =
        extract_upstream_secret_set(convert_rs.to_str().unwrap(), "NodeType");
    let (committed, _) = read_committed_fallback();
    assert_eq!(
        upstream, committed,
        "drift: committed CANONICAL_FALLBACK_NODE_TYPES in build.rs differs \
         from upstream NodeType::is_secret_bearing()"
    );
}

#[test]
#[ignore = "drift gate: needs MNEMONIC_GUI_UPSTREAM_ROOT; schema-mirror CI runs with --include-ignored"]
fn committed_fallback_slot_subkeys_match_upstream() {
    let Some(root) = upstream_root() else {
        panic!("MNEMONIC_GUI_UPSTREAM_ROOT not set — cannot validate");
    };
    let slot_rs = root.join("crates/mnemonic-toolkit/src/slot_input.rs");
    let upstream =
        extract_upstream_secret_set(slot_rs.to_str().unwrap(), "SlotSubkey");
    let (_, committed) = read_committed_fallback();
    assert_eq!(
        upstream, committed,
        "drift: committed CANONICAL_FALLBACK_SLOT_SUBKEYS in build.rs differs \
         from upstream SlotSubkey::is_secret_bearing()"
    );
}

/// The redaction tests in `tests/persistence.rs` assume `SECRET_*` is
/// populated. This vacuous test asserts the committed fallback is itself
/// non-empty — a sanity check that build.rs's `write_stub` won't
/// regress to `&[]`.
#[test]
fn committed_fallback_is_non_empty() {
    let (nodes, subkeys) = read_committed_fallback();
    assert!(
        !nodes.is_empty(),
        "CANONICAL_FALLBACK_NODE_TYPES must not be empty — empty set silently \
         disables redact_for_persistence (CVE-class persistence leak)"
    );
    assert!(
        !subkeys.is_empty(),
        "CANONICAL_FALLBACK_SLOT_SUBKEYS must not be empty — empty set silently \
         disables redact_for_persistence (CVE-class persistence leak)"
    );
    // Pin minimum membership: the four BIP-39-class subkeys MUST be present.
    for required in &["phrase", "entropy", "xprv", "wif"] {
        assert!(
            subkeys.contains(&required.to_string()),
            "CANONICAL_FALLBACK_SLOT_SUBKEYS must contain {:?}",
            required
        );
        assert!(
            nodes.contains(&required.to_string()),
            "CANONICAL_FALLBACK_NODE_TYPES must contain {:?}",
            required
        );
    }
}
