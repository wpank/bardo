#!/usr/bin/env bash
# Generate plans/context/workspace-map.md and plans/context/preflight-snapshot.md
# (same information as bardo-ctl's write_preflight_files, without running the TUI).
#
# Usage: ./scripts/bardo-sync-context.sh [REPO_ROOT]
# Env:
#   SKIP_CARGO_CHECK=1  — omit cargo check (faster; snapshot still has git sections)

set -euo pipefail

REPO_ROOT="$(cd "${1:-.}" && pwd)"
cd "$REPO_ROOT"

mkdir -p plans/context

export SKIP_CARGO_CHECK="${SKIP_CARGO_CHECK:-}"

python3 <<'PY'
import os
import subprocess
from pathlib import Path

root = Path(".").resolve()
ctx = root / "plans" / "context"
ctx.mkdir(parents=True, exist_ok=True)

lines: list[str] = ["# Workspace Map", ""]

def collect_rs(crate_src: Path) -> list[str]:
    out: list[str] = []
    for p in sorted(crate_src.rglob("*.rs")):
        rel = p.relative_to(crate_src)
        out.append(f"- `{rel}`")
    return out

for heading, base_name in (
    ("## Library Crates", "crates"),
    ("## Application Crates", "apps"),
):
    base = root / base_name
    if not base.is_dir():
        continue
    lines.append(heading)
    lines.append("")
    for entry in sorted(base.iterdir()):
        if not entry.is_dir():
            continue
        src = entry / "src"
        if not src.is_dir():
            continue
        lines.append(f"### {entry.name}")
        lines.append("")
        lines.extend(collect_rs(src))
        lines.append("")

(ctx / "workspace-map.md").write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")

# Preflight snapshot
snap: list[str] = ["# Preflight Snapshot", "", "## Recent Commits", ""]
try:
    log = subprocess.run(
        ["git", "log", "--oneline", "-10"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    snap.append("```")
    snap.append(log.stdout.strip() or "(no commits)")
    snap.append("```")
    snap.append("")
except Exception as e:
    snap.append(f"(git log failed: {e})")
    snap.append("")

snap.append("## Git Status")
snap.append("")
try:
    st = subprocess.run(
        ["git", "status", "--short"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    snap.append("```")
    snap.append(st.stdout.strip() or "(clean)")
    snap.append("```")
    snap.append("")
except Exception as e:
    snap.append(f"(git status failed: {e})")
    snap.append("")

snap.append("## Cargo Check")
snap.append("")
if os.environ.get("SKIP_CARGO_CHECK") == "1":
    snap.append("Status: SKIPPED (SKIP_CARGO_CHECK=1)")
    snap.append("")
else:
    chk = subprocess.run(
        ["cargo", "check", "--workspace"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    snap.append(f"Status: {'OK' if chk.returncode == 0 else 'FAILED'}")
    snap.append("")
    if chk.returncode != 0:
        tail = (chk.stderr or chk.stdout or "").splitlines()[-40:]
        snap.append("```")
        snap.extend(tail)
        snap.append("```")
        snap.append("")

(ctx / "preflight-snapshot.md").write_text("\n".join(snap), encoding="utf-8")
print(f"Wrote {ctx / 'workspace-map.md'} and {ctx / 'preflight-snapshot.md'}")
PY
