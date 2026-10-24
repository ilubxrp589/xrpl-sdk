#!/usr/bin/env python3
"""Check that every relative link in the crates' READMEs works on crates.io
and on GitHub.

    check_readme_links.py

`cargo package` copies a crate's README (the `readme` key of its
Cargo.toml) into the root of the package, and crates.io resolves the
README's relative links from the crate's own directory in the repository
(`crates/xrpl-core/...`). Core, client and SDK ship the repository's
root README, so a link such as `docs/PARITY.md`, right on GitHub, points
crates.io at `crates/xrpl-core/docs/PARITY.md`, which does not exist.

For each crate in `crates/`, each relative link in its README (Markdown
links, reference definitions, and HTML `href`/`src`) must name a file or
directory that exists both from the crate's directory (crates.io) and
from the README's own directory (GitHub). Links to other sites and
`#anchors` are not checked. The exit code is 1 when any link fails. CI
runs it on every change.
"""

import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent

LINK = re.compile(
    r"\]\(\s*<?([^)\s>]+)>?(?:\s+\"[^\"]*\")?\s*\)"  # [text](target "title")
    r"|^\s*\[[^\]]+\]:\s*<?(\S+?)>?(?:\s|$)"  # [ref]: target
    r"|(?:href|src)\s*=\s*\"([^\"]+)\"",  # <a href="target">
    re.MULTILINE,
)
EXTERNAL = re.compile(r"^(?:[a-z][a-z0-9+.-]*:|//|#)", re.IGNORECASE)


def relative_links(text):
    """Yield (line number, target without its #anchor) for each relative link."""
    for m in LINK.finditer(text):
        target = next(g for g in m.groups() if g)
        if EXTERNAL.match(target):
            continue
        yield text.count("\n", 0, m.start()) + 1, target.split("#", 1)[0]


def main():
    failures = {}  # (README, line, target, site) -> crates whose page breaks
    for manifest in sorted(ROOT.glob("crates/*/Cargo.toml")):
        crate_dir = manifest.parent
        package = tomllib.loads(manifest.read_text())["package"]
        readme = package.get("readme", "README.md")
        if readme is False or not (crate_dir / readme).is_file():
            continue
        readme_path = (crate_dir / readme).resolve()
        shown = readme_path.relative_to(ROOT)
        for line, target in relative_links(readme_path.read_text()):
            for where, base in (("crates.io", crate_dir), ("GitHub", readme_path.parent)):
                if not (base / target).resolve().exists():
                    failures.setdefault((shown, line, target, where), []).append(package["name"])
    for (shown, line, target, where), crates in failures.items():
        print(f"{shown}:{line}: `{target}` does not resolve on {where} for {', '.join(crates)}")
    if failures:
        print(
            "\nUse an absolute link (https://github.com/ilubxrp589/xrpl-sdk/blob/main/...) "
            "for anything outside the crate's directory.",
            file=sys.stderr,
        )
        return 1
    print("every relative README link resolves on crates.io and GitHub")
    return 0


if __name__ == "__main__":
    sys.exit(main())
