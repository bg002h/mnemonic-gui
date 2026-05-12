//! Synthetic minimal `impl NodeType` fixture for Phase 7's mutation
//! detection test (R4 I-2 fold). NOT a vendored copy of upstream
//! `convert.rs` — only ~30 LOC mirroring upstream `is_secret_bearing()`
//! MINUS `Phrase` (intentional mutation). The audit test asserts that
//! the build.rs codegen + parser disagrees on the `is_secret_bearing`
//! set when this fixture is passed via `MNEMONIC_GUI_AUDIT_OVERRIDE_PATH`.

pub enum NodeType {
    Phrase,
    Entropy,
    Xpub,
    Xprv,
    Wif,
    Ms1,
    Bip38,
    ElectrumPhrase,
}

impl NodeType {
    pub fn is_secret_bearing(self) -> bool {
        // R4 I-2 fold: `Phrase` intentionally OMITTED to trigger audit
        // mismatch. If upstream ADDS a new NodeType variant in the
        // future, Phase 1's schema-mirror test catches it first (this
        // fixture only needs to update on additions).
        matches!(
            self,
            Self::Entropy | Self::Xprv | Self::Wif | Self::Ms1 | Self::Bip38 | Self::ElectrumPhrase
        )
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Phrase => "phrase",
            Self::Entropy => "entropy",
            Self::Xpub => "xpub",
            Self::Xprv => "xprv",
            Self::Wif => "wif",
            Self::Ms1 => "ms1",
            Self::Bip38 => "bip38",
            Self::ElectrumPhrase => "electrum-phrase",
        }
    }
}
