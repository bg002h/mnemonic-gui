//! Guards the bug class "mnemonic-gui README install-command `--tag` pins
//! drift from `pinned-upstream.toml` / `Cargo.toml`" (FOLLOWUP
//! `gui-readme-install-pin-coherence-guard`).
//!
//! The README's `cargo install … --tag <X>` block claims the pinned tags
//! "match `pinned-upstream.toml`" — but nothing enforced it, and the pins
//! drifted THREE versions (self `mnemonic-gui-v0.22.0`, toolkit
//! `mnemonic-toolkit-v0.41.0`) before the v0.25.0 cycle backfilled them.
//! This pure-logic test (no binary, no network) promotes that prose to a
//! gate, mirroring `pin_coherence.rs` (Cargo↔pinned-upstream) and the
//! toolkit's `readme_version_current.rs`.
//!
//! Each README `cargo install … --tag <TAG> <pkg>` line must satisfy:
//!   - the `mnemonic-gui` self-line: `<TAG>` == `mnemonic-gui-v{Cargo.toml version}`
//!   - `mnemonic-toolkit` / `md-cli` / `ms-cli` / `mk-cli`: `<TAG>` == the
//!     matching `pinned-upstream.toml` `[mnemonic|md|ms|mk].tag`.
//!
//! The install lines use column-alignment padding (runs of spaces), so the
//! parser tokenizes on whitespace (`split_whitespace`), not single spaces.
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn read(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(name)).unwrap()
}

/// Extract `package -> tag` from every `cargo install … --tag <TAG> <pkg>`
/// line in the README. The package is the trailing token; the tag is the
/// token after `--tag`.
fn readme_install_pins() -> HashMap<String, String> {
    let readme = read("README.md");
    let mut out = HashMap::new();
    for line in readme.lines() {
        let line = line.trim();
        if !line.starts_with("cargo install") || !line.contains("--tag") {
            continue;
        }
        let toks: Vec<&str> = line.split_whitespace().collect();
        let tag_idx = toks
            .iter()
            .position(|t| *t == "--tag")
            .expect("a `--tag` token in a cargo-install line");
        let tag = (*toks.get(tag_idx + 1).expect("a tag after `--tag`")).to_string();
        let pkg = (*toks.last().expect("a trailing package name")).to_string();
        out.insert(pkg, tag);
    }
    out
}

fn pinned_upstream_tag(section: &str) -> String {
    let pinned: toml::Value = toml::from_str(&read("pinned-upstream.toml")).unwrap();
    pinned[section]["tag"]
        .as_str()
        .unwrap_or_else(|| panic!("pinned-upstream.toml [{section}].tag"))
        .to_string()
}

fn cargo_version() -> String {
    let cargo: toml::Value = toml::from_str(&read("Cargo.toml")).unwrap();
    cargo["package"]["version"]
        .as_str()
        .expect("Cargo.toml [package].version")
        .to_string()
}

#[test]
fn readme_install_tags_match_pins() {
    let pins = readme_install_pins();

    // Drive from the five expected install targets: each MUST be present and
    // its `--tag` MUST equal its source of truth (Cargo version for the GUI
    // self-tag; pinned-upstream.toml for the four siblings).
    let expectations: [(&str, String); 5] = [
        ("mnemonic-gui", format!("mnemonic-gui-v{}", cargo_version())),
        ("mnemonic-toolkit", pinned_upstream_tag("mnemonic")),
        ("md-cli", pinned_upstream_tag("md")),
        ("ms-cli", pinned_upstream_tag("ms")),
        ("mk-cli", pinned_upstream_tag("mk")),
    ];

    for (pkg, expected) in expectations {
        let actual = pins.get(pkg).unwrap_or_else(|| {
            panic!(
                "README is missing a `cargo install … --tag … {pkg}` line; \
                 found install targets: {:?}",
                pins.keys().collect::<Vec<_>>()
            )
        });
        assert_eq!(
            *actual, expected,
            "README install-pin drift for `{pkg}`: README `--tag {actual}` != \
             expected `{expected}`. Bump the README `cargo install` line in \
             lockstep with the Cargo.toml version / pinned-upstream.toml tag."
        );
    }
}
