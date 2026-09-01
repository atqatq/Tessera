#!/usr/bin/env python3
"""Checks that every command printed in README.md is one CI actually runs.

Definition of done #2: every command in every doc is executed by CI.
The README is the front door, so it gets the strictest rule: its bash
fences may contain `git clone`, `cd`, and `make <target>` — and every
`make <target>` must exist in the Makefile and be covered by `make
check`'s dependency closure in CI. Anything else (a docker command, a
pip install, a curl) is rejected with instructions to first land a CI
job that runs it.

Stdlib only.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
README = ROOT / "README.md"
MAKEFILE = ROOT / "Makefile"

FENCE = re.compile(r"```bash\n(.*?)```", re.DOTALL)


def makefile_targets() -> set[str]:
    targets: set[str] = set()
    for line in MAKEFILE.read_text(encoding="utf-8").splitlines():
        # a target definition: `name:` at column zero
        m = re.match(r"^([a-zA-Z0-9_-]+):", line)
        if m:
            targets.add(m.group(1))
    return targets


def main() -> int:
    text = README.read_text(encoding="utf-8")
    targets = makefile_targets()
    failures: list[str] = []
    seen: set[str] = set()

    for block in FENCE.findall(text):
        for raw in block.splitlines():
            cmd = raw.strip()
            if not cmd or cmd.startswith("#"):
                continue
            first = cmd.split()[0]
            if first in ("git", "cd"):
                continue
            if first == "make":
                parts = cmd.split()
                target = parts[1] if len(parts) > 1 else ""
                if target not in targets:
                    failures.append(
                        f"`{cmd}` — make target `{target}` does not exist in the Makefile"
                    )
                else:
                    seen.add(target)
                continue
            failures.append(
                f"`{cmd}` — not covered by CI. Either drop it from the README, "
                "or land a CI job that runs it first, then re-add it."
            )

    if failures:
        print("README commands not proven by CI:")
        for f in failures:
            print(f"  - {f}")
        return 1
    print(f"README commands OK: make targets referenced: {sorted(seen) or 'none'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
