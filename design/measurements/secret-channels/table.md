| input | argv w/o opt-in | `-` / positional `-` | `@env:VAR` | `--X-stdin` | file (`--X-file` / `--in`) | measurement valid |
|---|---|---|---|---|---|---|
| `mnemonic addresses --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic addresses --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic addresses --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic bundle --slot @0.phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --slot @0.entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --slot @0.seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic convert --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from xprv=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from minikey=` | refused | OK | fails closed | n/a | n/a | yes |
| `mnemonic convert --from ms1=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from electrum-phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic convert --bip38-passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic restore --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --from seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --from ms1=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic derive-child --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic derive-child --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic nostr --secret` | refused | fails closed | fails closed | OK | OK (`-file`) | yes |
| `mnemonic silent-payment --secret` | refused | fails closed | fails closed | OK | OK (`-file`) | yes |
| `mnemonic silent-payment --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic final-word --from phrase=` | refused | OK | fails closed | n/a | n/a | yes |
| `mnemonic seed-xor split --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic seed-xor combine --share phrase=` | NOT refused (exit 0) | OK | OK | n/a | n/a | yes |
| `mnemonic seedqr encode --from phrase=` | refused | OK | fails closed | n/a | n/a | yes |
| `mnemonic seedqr decode --from seedqr=` | refused | OK | fails closed | n/a | n/a | yes |
| `mnemonic seedqr decode --digits` | refused | OK | fails closed | — | — | yes |
| `mnemonic repair --ms1` | refused | OK | fails closed | — | — | yes |
| `mnemonic inspect --ms1` | refused | OK | fails closed | — | — | yes |
| `ms encode --phrase` | refused | OK | fails closed | — | OK (`--in`) | yes |
| `ms encode --hex` | refused | OK | fails closed | — | —/fails closed | yes |
| `ms inspect <ms1>` | refused | OK | fails closed | n/a | OK | yes |
| `ms decode <ms1>` | refused | OK | fails closed | n/a | OK | yes |
| `ms verify <ms1>` | refused | OK | fails closed | n/a | OK | yes |
| `ms derive <ms1>` | refused | OK | fails closed | n/a | OK | yes |
| `ms verify --phrase` | refused | OK | fails closed | — | —/fails closed | yes |
| `ms derive --hex` | refused | OK | fails closed | — | —/fails closed | yes |
| `ms derive --phrase` | refused | OK | fails closed | fails closed | —/fails closed | yes |
| `ms derive --passphrase` | refused | OK | OK | OK | —/fails closed | yes |
| `ms repair --ms1` | refused | OK | fails closed | — | OK (`--in`) | yes |
| `mnemonic derive-child --from xprv=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from wif=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --slot @0.phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic verify-bundle --ms1` | refused | OK | OK | — | — | yes |
| `mnemonic import-wallet --ms1` | NOT refused (exit 0) | fails closed | OK | — | — | yes |
| `mnemonic import-wallet --slot @0.phrase=` | refused | fails closed | OK | n/a | n/a | yes |
| `mnemonic electrum-decrypt --decrypt-password` | refused | OK | OK | OK | OK (`-file`) | yes |
| `mnemonic xpub-search path-of-xpub --phrase` | refused | fails closed | OK | OK | — | yes |
| `mnemonic xpub-search path-of-xpub --ms1` | refused | fails closed | fails closed | OK | — | yes |
| `mnemonic xpub-search path-of-xpub --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic xpub-search account-of-descriptor --phrase` | refused | fails closed | OK | OK | — | yes |
| `mnemonic xpub-search account-of-descriptor --ms1` | refused | fails closed | fails closed | OK | — | yes |
| `mnemonic xpub-search account-of-descriptor --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic xpub-search passphrase-of-xpub --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic xpub-search passphrase-of-xpub --phrase` | refused | fails closed | OK | OK | — | yes |
| `mnemonic slip39 split --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic slip39 split --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic slip39 combine --share` | NOT refused (exit 0) | OK | OK | — | — | yes |
| `mnemonic slip39 combine --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic ms-shares split --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic ms-shares combine --share` | NOT refused (exit 0) | OK | OK | — | — | yes |
| `ms split --phrase` | refused | OK | fails closed | — | OK (`--in`) | yes |
| `ms split --hex` | refused | OK | fails closed | — | —/fails closed | yes |
| `ms hashlock --hashlock-phrase` | refused | fails closed | fails closed | OK | —/fails closed | yes |
| `ms hashlock --hex` | refused | OK | fails closed | — | —/fails closed | yes |
| `mnemonic convert --from bip38=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic slip39 split --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic ms-shares split --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic xpub-search passphrase-of-xpub --ms1` | refused | fails closed | fails closed | OK | — | yes |
| `ms hashlock <ms1>` | refused | OK | fails closed | n/a | OK | yes |
| `mnemonic bundle --slot @0.ms1=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --slot @0.wif=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic addresses --from seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic import-wallet --decrypt-password` | refused | OK | OK | OK | OK (`-file`) | yes |
| `mnemonic addresses --from electrum-phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --slot @0.entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --slot @0.seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --slot @0.ms1=` | refused | OK | OK | n/a | n/a | yes |
