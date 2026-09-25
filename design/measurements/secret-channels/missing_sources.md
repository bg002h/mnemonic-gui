| input with no table entry | argv run (+opt-in) | verdict |
|---|---|---|
| `mnemonic addresses --from xprv=` | exit 1: error: --from Xprv is not supported by `addresses` (use xpub/phrase/entropy/seedqr/electrum-phrase) | CLI rejects this input itself |
| `mnemonic addresses --from wif=` | exit 1: error: --from Wif is not supported by `addresses` (use xpub/phrase/entropy/seedqr/electrum-phrase) | CLI rejects this input itself |
| `mnemonic addresses --from ms1=` | exit 1: error: --from Ms1 is not supported by `addresses` (use xpub/phrase/entropy/seedqr/electrum-phrase) | CLI rejects this input itself |
| `mnemonic addresses --from bip38=` | exit 1: error: --from Bip38 is not supported by `addresses` (use xpub/phrase/entropy/seedqr/electrum-phrase) | CLI rejects this input itself |
| `mnemonic addresses --from minikey=` | exit 1: error: --from MiniKey is not supported by `addresses` (use xpub/phrase/entropy/seedqr/electrum-phrase) | CLI rejects this input itself |
| `mnemonic bundle --slot @N.xprv=` | exit 1: error: --slot @0.xprv not supported in v0.4.2; deferred to v0.5+ pending ms-codec XPRV-tag extension. See FOLLOWUP `unified-slot-xprv-resolution-needs | CLI rejects this input itself |
| `mnemonic verify-bundle --from phrase=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from entropy=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from xprv=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from wif=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from ms1=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from bip38=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from electrum-phrase=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from seedqr=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from minikey=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --slot @N.xprv=` | exit 1: error: --slot @0.xprv not supported in v0.4.2; deferred to v0.5+ pending ms-codec XPRV-tag extension. See FOLLOWUP `unified-slot-xprv-resolution-needs | CLI rejects this input itself |
| `mnemonic verify-bundle --slot @N.wif=` | exit 4: (no error line) | **unmeasured** |
| `mnemonic export-wallet --slot @N.phrase=` | exit 2: error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret materi | CLI rejects this input itself |
| `mnemonic export-wallet --slot @N.seedqr=` | exit 2: error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret materi | CLI rejects this input itself |
| `mnemonic export-wallet --slot @N.entropy=` | exit 2: error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret materi | CLI rejects this input itself |
| `mnemonic export-wallet --slot @N.ms1=` | exit 2: error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret materi | CLI rejects this input itself |
| `mnemonic export-wallet --slot @N.xprv=` | exit 2: error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret materi | CLI rejects this input itself |
| `mnemonic export-wallet --slot @N.wif=` | exit 2: error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret materi | CLI rejects this input itself |
| `mnemonic restore --from xprv=` | exit 1: error: --from xprv is not a seed source for restore (use ms1/phrase/entropy/seedqr) | CLI rejects this input itself |
| `mnemonic restore --from wif=` | exit 1: error: --from wif is not a seed source for restore (use ms1/phrase/entropy/seedqr) | CLI rejects this input itself |
| `mnemonic restore --from bip38=` | exit 1: error: --from bip38 is not a seed source for restore (use ms1/phrase/entropy/seedqr) | CLI rejects this input itself |
| `mnemonic restore --from electrum-phrase=` | exit 1: error: --from electrum-phrase is not a seed source for restore (use ms1/phrase/entropy/seedqr) | CLI rejects this input itself |
| `mnemonic restore --from minikey=` | exit 1: error: --from minikey is not a seed source for restore (use ms1/phrase/entropy/seedqr) | CLI rejects this input itself |
| `mnemonic import-wallet --slot @N.seedqr=` | exit 1: error: import-wallet: --slot @0.seedqr=: only the `phrase` subkey is supported by import-wallet | CLI rejects this input itself |
| `mnemonic import-wallet --slot @N.entropy=` | exit 1: error: import-wallet: --slot @0.entropy=: only the `phrase` subkey is supported by import-wallet | CLI rejects this input itself |
| `mnemonic import-wallet --slot @N.ms1=` | exit 1: error: import-wallet: --slot @0.ms1=: only the `phrase` subkey is supported by import-wallet | CLI rejects this input itself |
| `mnemonic import-wallet --slot @N.xprv=` | exit 1: error: import-wallet: --slot @0.xprv=: only the `phrase` subkey is supported by import-wallet | CLI rejects this input itself |
| `mnemonic import-wallet --slot @N.wif=` | exit 1: error: import-wallet: --slot @0.wif=: only the `phrase` subkey is supported by import-wallet | CLI rejects this input itself |
| `mnemonic word-card --from phrase=` | exit 2: error: positional argument 'phrase=aband…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from entropy=` | exit 2: error: positional argument 'entropy=0000…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from xprv=` | exit 2: error: positional argument 'xprv=xprv9s2…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from wif=` | exit 2: error: positional argument 'wif=KyZpNDKn…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from ms1=` | exit 2: error: positional argument 'ms1=ms10entr…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from bip38=` | exit 2: error: positional argument 'bip38=6PYP8f…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from electrum-phrase=` | exit 2: error: positional argument 'electrum-phr…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from seedqr=` | exit 2: error: positional argument 'seedqr=00000…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from minikey=` | exit 2: error: positional argument 'minikey=S6c5…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
