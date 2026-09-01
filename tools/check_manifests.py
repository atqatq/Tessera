# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0

#!/usr/bin/env python3
"""CI gate for spoke manifests.

Exit code 0 means every manifest is valid and the independence rule holds.
Any other exit code fails the build and prints every finding, not just the
first, so a developer fixes the whole manifest in one pass.
"""

from __future__ import annotations

import argparse
import glob
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "reference" / "python"))

from scor_ref.manifest import validate_all  # noqa: E402


def load(path: Path) -> dict:
    text = path.read_text()
    if path.suffix in (".json",):
        return json.loads(text)
    try:
        import yaml
    except ImportError:
        sys.exit("pyyaml is required to read yaml manifests: pip install pyyaml")
    return yaml.safe_load(text)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="+")
    parser.add_argument(
        "--rule",
        choices=["all", "independence"],
        default="all",
        help="'independence' reports only the hard-dependency rule",
    )
    args = parser.parse_args()

    files = sorted({Path(p) for pattern in args.paths for p in glob.glob(pattern)})
    if not files:
        print("no manifests matched", file=sys.stderr)
        return 2

    manifests = [load(f) for f in files]
    results = validate_all(manifests)

    failed = False
    for spoke, result in sorted(results.items()):
        findings = result.findings
        if args.rule == "independence":
            findings = [f for f in findings if f.code == "requires.spoke_dependency"]
        for finding in findings:
            print(f"{spoke}: {finding}")
            if finding.severity == "error":
                failed = True

    print(f"\nchecked {len(files)} manifest(s); {'FAILED' if failed else 'ok'}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
