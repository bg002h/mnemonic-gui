| shape | Linux plan | macOS | Windows |
|---|---|---|---|
| addresses phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| restore phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| restore ms1+passphrase | --from ms1= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| derive-child phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| bundle slot+passphrase | --slot @N.phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| bundle wsh-multi 2 slots | --slot @N.phrase= ← env MNEMONIC_GUI_S0; --slot @N.phrase= ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| bundle wsh-multi 2 slots+passphrase | --slot @N.phrase= ← env MNEMONIC_GUI_S0; --slot @N.ms1= ← env MNEMONIC_GUI_S1; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| convert phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| convert wif+bip38-passphrase | --from wif= ← env MNEMONIC_GUI_S0; --bip38-passphrase ← stdin via --bip38-passphrase-stdin | same as Linux | same as Linux |
| convert bip38+bip38-passphrase | --from bip38= ← env MNEMONIC_GUI_S0; --bip38-passphrase ← stdin via --bip38-passphrase-stdin | same as Linux | same as Linux |
| xpub-search path-of-xpub phrase+passphrase | path-of-xpub --phrase ← stdin via --phrase-stdin; path-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| xpub-search path-of-xpub ms1+passphrase | path-of-xpub --ms1 ← stdin via --ms1-stdin; path-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| xpub-search passphrase-of-xpub phrase+passphrase | passphrase-of-xpub --phrase ← stdin via --phrase-stdin; passphrase-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| xpub-search passphrase-of-xpub ms1+passphrase | passphrase-of-xpub --ms1 ← stdin via --ms1-stdin; passphrase-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| xpub-search account-of-descriptor phrase+passphrase | account-of-descriptor --phrase ← stdin via --phrase-stdin; account-of-descriptor --passphrase ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| xpub-search account-of-descriptor ms1+passphrase | account-of-descriptor --ms1 ← stdin via --ms1-stdin; account-of-descriptor --passphrase ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| silent-payment secret+passphrase | --secret ← pipe fd via --secret-file; --passphrase ← stdin via --passphrase-stdin | **refuse** (fd-not-on-platform) | **refuse** (fd-not-on-platform) |
| slip39 split phrase+passphrase | split --from phrase= ← env MNEMONIC_GUI_S0; split --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| slip39 combine 2 shares+passphrase | combine --share ← env MNEMONIC_GUI_S0; combine --share ← env MNEMONIC_GUI_S1; combine --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| slip39 combine 2 shares | combine --share ← env MNEMONIC_GUI_S0; combine --share ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| seed-xor combine 2 shares | combine --share phrase= ← env MNEMONIC_GUI_S0; combine --share phrase= ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| ms-shares combine 2 shares | combine --share ← env MNEMONIC_GUI_S0; combine --share ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| import-wallet 2 cosigner ms1 | --ms1 ← env MNEMONIC_GUI_S0; --ms1 ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| verify-bundle ms1+slot+passphrase | --ms1 ← env MNEMONIC_GUI_S0; --slot @N.phrase= ← env MNEMONIC_GUI_S1; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| ms combine share group | <shares> ← stdin via one `-` (all, one per line) | same as Linux | same as Linux |
| ms verify phrase+ms1 | --phrase ← stdin via `-`; <ms1> ← pipe fd via --in | **refuse** (fd-not-on-platform) | **refuse** (fd-not-on-platform) |
| ms derive ms1+passphrase | --passphrase ← stdin via --passphrase-stdin; <ms1> ← pipe fd via --in | **refuse** (fd-not-on-platform) | **refuse** (fd-not-on-platform) |
| ms derive phrase+passphrase | **refuse** (two-stdin) | same as Linux | same as Linux |
| ms derive hex+passphrase | **refuse** (two-stdin) | same as Linux | same as Linux |

| shape | baseline exit | planned exit | planned == baseline | effect (baseline) | source values swapped (i↔j) vs baseline | T1 | C1: `-` refused; `@env:VAR` resolved run == baseline |
|---|---|---|---|---|---|---|---|
| addresses phrase+passphrase | 0 | 0 | **yes** | `0  bc1qrm3qju2002wmwly8x2ee7ghdaunexsndwgedwv` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| restore phrase+passphrase | 0 | 0 | **yes** | `master fingerprint: 45fbfbe6  (passphrase: applied)` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| restore ms1+passphrase | 0 | 0 | **yes** | `master fingerprint: 45fbfbe6  (passphrase: applied)` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| derive-child phrase+passphrase | 0 | 0 | **yes** | `target biology midnight canal glass common include trophy glimpse north castle dove` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| bundle slot+passphrase | 0 | 0 | **yes** | `mk1qpd2y2pqqsqk4z99gdzlh7lxqvzg3vs7vs57ls3u2nlnjvzn90ffnjpcsauf2eggmpdquu02l9k7dpjhxhs3yaa` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| bundle wsh-multi 2 slots | 0 | 0 | **yes** | `mk1qpm6gzpqqspdayp9s00fqfvrw0za5zs8qjyty8kskx54hpzzjwjpds9j69su6hyzpkdq32t74e44wnhpg9dj4y5` | 0↔1: differs (exit 0) | ok | ok; 2/2 |
| bundle wsh-multi 2 slots+passphrase | 0 | 0 | **yes** | `mk1qpgaqcpqqspywsvg03r5rzrughalhes8qjyty83nr0wscquatny8cq3ctkcnc7w7lklfeq66dhl2ml4aacj89mq` | 0↔1: differs (exit 1); 0↔2: differs (exit 1); 1↔2: differs (exit 1) | ok | ok; 3/3 |
| convert phrase+passphrase | 0 | 0 | **yes** | `xpub: xpub6CPUCVp94gpNs3bS1eGiwSWwZMLNmfr2Uo1t5v8YtY4XVoxhUraBH7sRyVfgwSNCxRVpX1bDREtc5Kri` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| convert wif+bip38-passphrase | 0 | 0 | **yes** | `bip38: 6PYP8fdoVaE3ThLmEnYcGo3nJeBqd8PvB7CRvTz3TX5L9ojoPHKCg7QXG6` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| convert bip38+bip38-passphrase | 0 | 0 | **yes** | `wif: KyZpNDKnfs94vbrwhJneDi77V6jF64PWPF8x5cdJb8ifgg2DUc9d` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| xpub-search path-of-xpub phrase+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| xpub-search path-of-xpub ms1+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 2) | ok | ok; 2/2 |
| xpub-search passphrase-of-xpub phrase+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| xpub-search passphrase-of-xpub ms1+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 2) | ok | ok; 2/2 |
| xpub-search account-of-descriptor phrase+passphrase | 0 | 0 | **yes** | `match: cosigner @0  m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| xpub-search account-of-descriptor ms1+passphrase | 0 | 0 | **yes** | `match: cosigner @0  m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 2) | ok | ok; 2/2 |
| silent-payment secret+passphrase | 0 | 0 | **yes** | `address:      sp1qq2d73kpx36h7r08gmawe6slzxkntu2tw0as7pkqe0hvv3k38mkvckqsp6dmrwxvd6mumfqj9` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| slip39 split phrase+passphrase | 0 | 0 | **yes** | `00000000000000000000000000000000` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| slip39 combine 2 shares+passphrase | 0 | 0 | **yes** | `8ab7aa39cd427130e225ed43c34f5b5a` | 0↔1: **same** (symmetric); 0↔2: differs (exit 1); 1↔2: differs (exit 1) | ok | ok; 3/3 |
| slip39 combine 2 shares | 0 | 0 | **yes** | `00000000000000000000000000000000` | 0↔1: **same** (symmetric) | ok | ok; 2/2 |
| seed-xor combine 2 shares | 0 | 0 | **yes** | `zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong` | 0↔1: **same** (symmetric) | ok | ok; 2/2 |
| ms-shares combine 2 shares | 0 | 0 | **yes** | `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ab` | 0↔1: **same** (symmetric) | ok | ok; 2/2 |
| import-wallet 2 cosigner ms1 | 0 | 0 | **yes** | `"descriptor": "wsh(sortedmulti(2,[5436d724/48'/0'/0'/2']xpub6E79FaRWLSJCAgA2jDHRvyrWKwT6aS` | 0↔1: differs (exit 4) | ok | ok; 2/2 |
| verify-bundle ms1+slot+passphrase | 0 | 0 | **yes** | `result: ok` | 0↔1: differs (exit 2); 0↔2: differs (exit 2); 1↔2: differs (exit 1) | ok | ok; 3/3 |
| ms combine share group | 0 | 0 | **yes** | `entropy: 00000000000000000000000000000000` | n/a (one source) | ok | ok; 1/1 |
| ms verify phrase+ms1 | 0 | 0 | **yes** | `OK: round-trip valid (12 words, language=english)` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| ms derive ms1+passphrase | 0 | 0 | **yes** | `master_fingerprint:  45fbfbe6` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| ms derive phrase+passphrase | — | — | refused (expected: refuse) | — | — | — | ok; n/a |
| ms derive hex+passphrase | — | — | refused (expected: refuse) | — | — | — | ok; n/a |
