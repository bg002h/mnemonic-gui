"""T10 prototype (R2 Nm8, hardened for R3 Nm10): may an OS be in private_channels_on?

Only if some workflow, triggered on `push` or `pull_request`, has a job that
  1. has no `if:` and no `continue-on-error` (job or the qualifying step);
  2. runs on that OS: a literal `runs-on`, or `${{ matrix.<k> }}` resolved through
     strategy.matrix — list values plus `include`, minus `exclude`;
  3. has a step whose `run:` contains a SHELL COMMAND LINE (comments stripped, leading
     `VAR=value` assignments skipped) whose program is `cargo` and whose subcommand is `test` or
     `nextest run`, naming every target in real_binary_test_targets with `--test <t>` — with no
     `||`/`|`/`;`/`&` masking it, no `--no-run`, and no test-name filter or skip option (R4 Nm14);
  3b. and whose `needs:` chain reaches no `if:` job (R4 Nm14);
  4. has a NON-EMPTY `MNEMONIC_BIN` in that step's, the job's or the workflow's `env:`.
Parsed YAML throughout, so a comment can satisfy nothing.

Chosen over a signed-artifact scheme: flipping an OS is a reviewed one-line edit, and a signed
artifact would add key management to a gate whose decoys are accidents, not attacks."""
import glob, os, re, shlex
import yaml

RUNNER = {"linux": "ubuntu", "macos": "macos", "windows": "windows"}


def _triggers(doc):
    on = doc.get("on", doc.get(True))           # YAML 1.1 reads a bare `on:` key as boolean True
    if isinstance(on, str):
        return {on}
    if isinstance(on, list):
        return set(on)
    return set(on or {})


def _runners(job):
    ro = job.get("runs-on", "")
    ro = ro if isinstance(ro, list) else [ro]
    matrix = (job.get("strategy") or {}).get("matrix") or {}
    out = []
    for r in ro:
        m = re.fullmatch(r"\$\{\{\s*matrix\.([\w-]+)\s*\}\}", str(r).strip())
        if not m:
            out.append(str(r))
            continue
        key = m.group(1)
        vals = matrix.get(key, [])
        vals = [str(v) for v in (vals if isinstance(vals, list) else [vals])]
        vals += [str(inc[key]) for inc in matrix.get("include", []) or [] if isinstance(inc, dict) and key in inc]
        excl = {str(ex[key]) for ex in matrix.get("exclude", []) or [] if isinstance(ex, dict) and key in ex}
        out += [v for v in vals if v not in excl]
    return out


# cargo / nextest options that take a VALUE (so the next word is not a test-name filter)
_VALUE_OPTS = {"--test", "-p", "--package", "--features", "-F", "--target", "--profile", "-j", "--jobs",
               "--manifest-path", "--config", "-Z", "--bin", "--example", "--bench", "--color",
               "--message-format", "--target-dir", "--cargo-profile", "--partition"}
# options that FILTER or SKIP tests, or build without running (R4 Nm14)
_REJECT_OPTS = {"--no-run", "-E", "--filterset", "--filter-expr", "--skip", "--exact", "--ignored", "--run-ignored"}


def _cargo_test_lines(run):
    """Yield the argument words of each `cargo test` / `cargo nextest run` command that is the
    whole shell command line apart from leading VAR=value assignments and trailing `&& …`.
    Rejected (R4 Nm14): `||`, `|`, `;`, `&` anywhere on the line (masks the exit status or
    detaches it), `--no-run` and filter/skip options, and any bare word that would be a test-name
    filter (before `--`: not an option or an option's value; after `--`: not a harness flag)."""
    for line in run.splitlines():
        line = line.split(" #")[0].strip()
        if not line or line.startswith("#"):
            continue
        try:
            lex = shlex.shlex(line, posix=True, punctuation_chars=True)
            lex.whitespace_split = True
            words = list(lex)
        except ValueError:
            continue
        while words and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*=.*", words[0]):
            words = words[1:]
        if "&&" in words:
            words = words[:words.index("&&")]          # what follows a successful run is irrelevant
        if any(w in ("||", "|", ";", "&", "|&") for w in words):
            continue
        if not (len(words) >= 2 and words[0] == "cargo" and (words[1] == "test" or words[1:3] == ["nextest", "run"])):
            continue
        args = words[2:] if words[1] == "test" else words[3:]
        if any(a.split("=")[0] in _REJECT_OPTS for a in args):
            continue
        before, after = (args[:args.index("--")], args[args.index("--") + 1:]) if "--" in args else (args, [])
        bad, k = False, 0
        while k < len(before):
            a = before[k]
            if a in _VALUE_OPTS:
                k += 2
                continue
            if not a.startswith("-"):
                bad = True                                 # a test-name filter
                break
            k += 1
        if bad or any(not (a.startswith("-") or a.isdigit()) for a in after):
            continue
        yield words


def _has_bin(*envs):
    return any(str((e or {}).get("MNEMONIC_BIN", "")).strip() for e in envs)


def _needs_guarded(name, jobs, seen=None):
    """True if the job's `needs:` chain reaches a job with `if:` (it may be skipped, and then so is
    this one) or a job that does not exist (R4 Nm14)."""
    seen = seen or set()
    needs = jobs.get(name, {}).get("needs") or []
    needs = [needs] if isinstance(needs, str) else needs
    for n in needs:
        if n in seen:
            continue
        seen.add(n)
        if n not in jobs or "if" in jobs[n] or _needs_guarded(n, jobs, seen):
            return True
    return False


def job_qualifies(job, osname, targets, wf_env, name=None, jobs=None):
    if "if" in job or job.get("continue-on-error"):
        return False
    if jobs is not None and _needs_guarded(name, jobs):
        return False
    if not any(RUNNER[osname] in r for r in _runners(job)):
        return False
    for step in job.get("steps", []) or []:
        if "if" in step or step.get("continue-on-error"):
            continue
        for words in _cargo_test_lines(step.get("run") or ""):
            named = {words[k + 1] for k in range(len(words) - 1) if words[k] == "--test"}
            if all(t in named for t in targets) and _has_bin(step.get("env"), job.get("env"), wf_env):
                return True
    return False


def gate(workflow_paths, osname, targets):
    for p in workflow_paths:
        doc = yaml.safe_load(open(p)) or {}
        if not _triggers(doc) & {"push", "pull_request"}:
            continue
        jobs = {k: v for k, v in (doc.get("jobs") or {}).items() if isinstance(v, dict)}
        for name, job in jobs.items():
            if job_qualifies(job, osname, targets, doc.get("env"), name, jobs):
                return True
    return False


def repo_workflows(here):
    return sorted(glob.glob(os.path.join(here, "..", "..", "..", ".github", "workflows", "*.yml")))
