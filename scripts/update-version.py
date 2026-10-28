#!/usr/bin/env python3
"""Update the application version in all project-owned package metadata."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


SEMVER = re.compile(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)"
    r"(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
    r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$"
)


def replace_json_version(path: Path, version: str) -> None:
    data = json.loads(path.read_text())
    data["version"] = version
    path.write_text(json.dumps(data, indent=2) + "\n")


def replace_toml_version(path: Path, version: str) -> None:
    text = path.read_text()
    updated, replacements = re.subn(
        r'(?m)^(version\s*=\s*)"[^"]+"',
        rf'\1"{version}"',
        text,
        count=1,
    )
    if replacements != 1:
        raise RuntimeError(f"Could not find the package version in {path}")
    path.write_text(updated)


def replace_lock_package_version(path: Path, package_name: str, version: str) -> None:
    text = path.read_text()
    package = re.compile(
        rf'(?ms)(\[\[package\]\]\nname = "{re.escape(package_name)}"\n'
        r'.*?^version = )"[^"]+"'
    )
    updated, replacements = package.subn(rf'\1"{version}"', text, count=1)
    if replacements != 1:
        raise RuntimeError(f"Could not find {package_name} in {path}")
    path.write_text(updated)


def main() -> int:
    if len(sys.argv) != 2 or not SEMVER.fullmatch(sys.argv[1]):
        print(f"Usage: {Path(sys.argv[0]).name} <version>", file=sys.stderr)
        print("Example: update-version.py 1.2.0", file=sys.stderr)
        return 2

    version = sys.argv[1]
    root = Path(__file__).resolve().parents[1]

    replace_json_version(root / "package.json", version)
    replace_toml_version(root / "Cargo.toml", version)
    replace_toml_version(root / "src-tauri/Cargo.toml", version)
    replace_json_version(root / "src-tauri/tauri.conf.json", version)
    replace_lock_package_version(root / "Cargo.lock", "litestation", version)
    replace_lock_package_version(root / "Cargo.lock", "litestation-lints", version)

    print(f"Updated project version to {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
