// Produces secret_sources.txt. Build as a scratch crate depending on mnemonic-gui
// (path dep, default-features = false) with the toolkit's [patch.crates-io] miniscript rev;
// `cargo run > secret_sources.txt`. T4 replaces this with an in-repo Rust test.
// Enumerate every secret SOURCE the GUI's schema mirror can produce (DESIGN §A4 definition),
// spelled as channel-table keys. Throwaway; its output is committed as secret_sources.txt.
use mnemonic_gui::schema::{self, FlagKind};
use mnemonic_gui::secrets::{self, SECRET_NODE_TYPES_ARGV, SECRET_SLOT_SUBKEYS};
const NESTED: &[&str] = &["seed-xor", "seedqr", "slip39", "ms-shares", "xpub-search"];
fn sub_tokens(name: &str) -> String {
    for p in NESTED {
        if let Some(rest) = name.strip_prefix(&format!("{p}-")) { return format!("{p} {rest}"); }
    }
    name.to_string()
}
fn main() {
    for sch in [&schema::mnemonic::SCHEMA, &schema::md::SCHEMA, &schema::ms::SCHEMA, &schema::mk::SCHEMA] {
        for sub in sch.subcommands {
            let base = format!("{} {}", sch.cli_name, sub_tokens(sub.name));
            for f in sub.flags {
                if f.name == "--slot" && sub.allows_slots {
                    for sk in SECRET_SLOT_SUBKEYS { println!("{base} --slot @N.{sk}="); }
                    continue;
                }
                match f.kind {
                    FlagKind::NodeValueComposite(nodes) => {
                        for n in nodes.iter().filter(|n| secrets::node_type_is_argv_secret(n)) {
                            println!("{base} {} {n}=", f.name);
                        }
                    }
                    FlagKind::Text if secrets::flag_is_secret(f) => println!("{base} {}", f.name),
                    // a non-secret Text flag whose value may name a secret node (restore --from …)
                    FlagKind::Text if f.name == "--from" => {
                        for n in SECRET_NODE_TYPES_ARGV { println!("{base} --from {n}="); }
                    }
                    _ => {}
                }
            }
            for p in sub.positional_args.iter().filter(|p| p.secret) {
                if p.repeating { println!("{base} <{}> [group]", p.name); } else { println!("{base} <{}>", p.name); }
            }
        }
    }
}
