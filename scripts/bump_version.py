#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Automatic CalVer (Year.Month.Day.Number) version bumper for rcal.

Format in Cargo.toml (SemVer compliant): YYYY.MMDD.N (e.g. 2026.1101.1)
Format for humans & CLI (rcal --version): YYYY.MM.DD.N (e.g. 2026.11.01.1)

Usage:
    python3 scripts/bump_version.py             # Auto-bump for today
    python3 scripts/bump_version.py --set 2026.11.01.2
"""

import sys
import re
import datetime
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

def parse_calver(v_str):
    """
    Parses either '2026.11.01.1' or '2026.1101.1' into (year, month, day, num)
    """
    m1 = re.match(r"^(\d{4})\.(\d{1,2})\.(\d{1,2})\.(\d+)$", v_str)
    if m1:
        return int(m1.group(1)), int(m1.group(2)), int(m1.group(3)), int(m1.group(4))
    
    m2 = re.match(r"^(\d{4})\.(\d{4})\.(\d+)$", v_str)
    if m2:
        year = int(m2.group(1))
        mmdd = m2.group(2)
        month = int(mmdd[:2])
        day = int(mmdd[2:])
        num = int(m2.group(3))
        return year, month, day, num

    return None

def format_semver(year, month, day, num):
    return f"{year:04d}.{month:02d}{day:02d}.{num}"

def format_human(year, month, day, num):
    return f"{year:04d}.{month:02d}.{day:02d}.{num}"

def main():
    cargo_toml_path = REPO_ROOT / "Cargo.toml"
    content = cargo_toml_path.read_text(encoding="utf-8")

    m = re.search(r'^version\s*=\s*"([^"]+)"', content, re.MULTILINE)
    if not m:
        print("Error: Could not find version in Cargo.toml", file=sys.stderr)
        sys.exit(1)

    cur_version_str = m.group(1)
    parsed = parse_calver(cur_version_str)

    today = datetime.date.today()
    ty, tm, td = today.year, today.month, today.day

    # Check CLI args for --set
    if len(sys.argv) >= 3 and sys.argv[1] == "--set":
        explicit = parse_calver(sys.argv[2])
        if not explicit:
            print(f"Error: Invalid version format '{sys.argv[2]}'. Expected YYYY.MM.DD.N or YYYY.MMDD.N", file=sys.stderr)
            sys.exit(1)
        ny, nm, nd, nnum = explicit
    else:
        if parsed and (parsed[0], parsed[1], parsed[2]) == (ty, tm, td):
            # Same day, bump number
            ny, nm, nd, nnum = ty, tm, td, parsed[3] + 1
        else:
            # New day or first calver
            ny, nm, nd, nnum = ty, tm, td, 1

    new_semver = format_semver(ny, nm, nd, nnum)
    new_human = format_human(ny, nm, nd, nnum)

    print(f"Bumping version:")
    print(f"  Old version: {cur_version_str}")
    print(f"  New SemVer (Cargo.toml): {new_semver}")
    print(f"  New Human CalVer:        {new_human}")
    print(f"  New Git Tag:             v{new_human}")

    # 1. Update Cargo.toml
    new_content = re.sub(
        r'^version\s*=\s*"[^"]+"',
        f'version = "{new_semver}"',
        content,
        flags=re.MULTILINE
    )
    cargo_toml_path.write_text(new_content, encoding="utf-8")

    # 2. Update Cargo.lock via cargo check
    subprocess.run(["cargo", "check", "--quiet"], cwd=REPO_ROOT, check=True)

    # 3. Update READMEs UI mockups
    old_banner_re = re.compile(r"rcal v[0-9\.]+")
    readme_path = REPO_ROOT / "README.md"
    if readme_path.exists():
        r_text = readme_path.read_text(encoding="utf-8")
        r_text = old_banner_re.sub(f"rcal v{new_human}", r_text)
        readme_path.write_text(r_text, encoding="utf-8")

    locales_dir = REPO_ROOT / "docs" / "locales"
    if locales_dir.exists():
        for f in locales_dir.glob("README.*.md"):
            t = f.read_text(encoding="utf-8")
            t = old_banner_re.sub(f"rcal v{new_human}", t)
            f.write_text(t, encoding="utf-8")

    print("\n✓ Version bump completed successfully!")

if __name__ == "__main__":
    main()
