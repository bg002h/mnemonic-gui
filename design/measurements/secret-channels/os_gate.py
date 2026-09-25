"""T10 prototype (R2 Nm8): may an OS be in private_channels_on? Only if some workflow has a job
that (1) is not `if:`-guarded, (2) runs on that OS — literal `runs-on`, or `${{ matrix.<k> }}`
resolved through strategy.matrix (include entries too), (3) has a step whose `run:` invokes every
target in real_binary_test_targets through cargo (`--test <t>`), with (4) MNEMONIC_BIN set in that
step's or the job's `env:` (parsed YAML, so a comment cannot satisfy it)."""
import glob, os, re
import yaml

RUNNER = {"linux": "ubuntu", "macos": "macos", "windows": "windows"}


def _runners(job):
    ro = job.get("runs-on", "")
    ro = ro if isinstance(ro, list) else [ro]
    out = []
    matrix = (job.get("strategy") or {}).get("matrix") or {}
    for r in ro:
        m = re.fullmatch(r"\$\{\{\s*matrix\.([\w-]+)\s*\}\}", str(r).strip())
        if m:
            key = m.group(1)
            vals = matrix.get(key, [])
            out += [str(v) for v in (vals if isinstance(vals, list) else [vals])]
            out += [str(inc[key]) for inc in matrix.get("include", []) or [] if isinstance(inc, dict) and key in inc]
        else:
            out.append(str(r))
    return out


def job_qualifies(job, osname, targets):
    if "if" in job:
        return False
    if not any(RUNNER[osname] in r for r in _runners(job)):
        return False
    job_env = job.get("env") or {}
    for step in job.get("steps", []) or []:
        run = step.get("run") or ""
        if "if" in step or "cargo" not in run:
            continue
        if not all(re.search(rf"--test\s+{re.escape(t)}\b", run) for t in targets):
            continue
        if "MNEMONIC_BIN" in (step.get("env") or {}) or "MNEMONIC_BIN" in job_env:
            return True
    return False


def gate(workflow_paths, osname, targets):
    for p in workflow_paths:
        doc = yaml.safe_load(open(p)) or {}
        for job in (doc.get("jobs") or {}).values():
            if isinstance(job, dict) and job_qualifies(job, osname, targets):
                return True
    return False


def repo_workflows(here):
    return sorted(glob.glob(os.path.join(here, "..", "..", "..", ".github", "workflows", "*.yml")))
