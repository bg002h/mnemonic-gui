"""address-of-seed prints the `bundle` command that mints the wallet it found, and that line repeats
the channel spelling the run used (`-`, `@env:VAR`) for each seed and the passphrase. The wallet found
(match, path, slots, descriptor) does not depend on the spelling, so the measurement masks it."""
import re


def AOS_CHECK(o):
    return "\n".join(re.sub(r"(--passphrase )\S+", r"\1<S>", re.sub(r"(--slot @\d+\.[a-z0-9-]+=)\S+", r"\1<S>", l))
                     if l.strip().startswith("mnemonic bundle ") else l for l in o.splitlines())
