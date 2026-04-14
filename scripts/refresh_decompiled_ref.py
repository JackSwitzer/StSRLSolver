#!/usr/bin/env python3
"""Refresh the local Slay the Spire decompile cache.

This script builds a workspace-local reference tree at:
  decompiled/java-src

It uses the installed game jar by default, filters it down to the
`com/megacrit/cardcrawl` namespace, decompiles that filtered jar with CFR,
writes a manifest, and refreshes the compatibility symlink:
  /tmp/sts-decompiled -> decompiled/java-src

The cache lives under an already-ignored root and is intended as a local
parity/audit reference, not something we commit.
"""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from typing import Iterable
import urllib.request
import zipfile


REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_GAME_JAR = Path(
    "/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/"
    "SlayTheSpire/SlayTheSpire.app/Contents/Resources/desktop-1.0.jar"
)
DEFAULT_OUTPUT_ROOT = REPO_ROOT / "decompiled"
DEFAULT_OUTPUT_DIR = DEFAULT_OUTPUT_ROOT / "java-src"
DEFAULT_MANIFEST = DEFAULT_OUTPUT_ROOT / "manifest.json"
DEFAULT_CFR_JAR = DEFAULT_OUTPUT_ROOT / ".tools" / "cfr-0.152.jar"
DEFAULT_CFR_URL = "https://repo.maven.apache.org/maven2/org/benf/cfr/0.152/cfr-0.152.jar"
DEFAULT_TMP_SYMLINK = Path("/tmp/sts-decompiled")
DEFAULT_FILTER_PREFIX = "com/megacrit/cardcrawl/"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, default=DEFAULT_GAME_JAR, help="Path to desktop-1.0.jar")
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=DEFAULT_OUTPUT_DIR,
        help="Destination directory for decompiled java sources",
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        default=DEFAULT_MANIFEST,
        help="Manifest JSON written alongside the cache",
    )
    parser.add_argument(
        "--cfr-jar",
        type=Path,
        default=DEFAULT_CFR_JAR,
        help="Location of the cached CFR decompiler jar",
    )
    parser.add_argument(
        "--cfr-url",
        default=DEFAULT_CFR_URL,
        help="Download URL for CFR when the local cached jar is missing",
    )
    parser.add_argument(
        "--filter-prefix",
        default=DEFAULT_FILTER_PREFIX,
        help="Only decompile classes beneath this jar prefix",
    )
    parser.add_argument(
        "--java-bin",
        type=Path,
        default=None,
        help="Optional java binary override; defaults to the bundled JRE when present",
    )
    parser.add_argument(
        "--tmp-symlink",
        type=Path,
        default=DEFAULT_TMP_SYMLINK,
        help="Compatibility symlink refreshed to point at the cache",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Rebuild even if the manifest already matches the source jar",
    )
    return parser.parse_args()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def load_manifest(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        return json.loads(path.read_text())
    except json.JSONDecodeError:
        return None


def bundled_java_for(jar_path: Path) -> Path | None:
    candidate = jar_path.parent / "jre" / "bin" / "java"
    return candidate if candidate.exists() else None


def resolve_java_bin(cli_java: Path | None, jar_path: Path) -> Path:
    if cli_java is not None:
        return cli_java
    bundled = bundled_java_for(jar_path)
    if bundled is not None:
        return bundled
    return Path("java")


def ensure_cfr_jar(cfr_jar: Path, cfr_url: str) -> None:
    if cfr_jar.exists():
        return
    ensure_parent(cfr_jar)
    with urllib.request.urlopen(cfr_url) as response, cfr_jar.open("wb") as out:
        shutil.copyfileobj(response, out)


def build_filtered_jar(source_jar: Path, filtered_jar: Path, prefix: str) -> int:
    count = 0
    with zipfile.ZipFile(source_jar) as zin, zipfile.ZipFile(filtered_jar, "w", compression=zipfile.ZIP_DEFLATED) as zout:
        for info in zin.infolist():
            if not info.filename.startswith(prefix):
                continue
            if not info.filename.endswith(".class"):
                continue
            zout.writestr(info, zin.read(info.filename))
            count += 1
    return count


def run_cfr(java_bin: Path, cfr_jar: Path, filtered_jar: Path, output_dir: Path) -> None:
    cmd = [
        str(java_bin),
        "-jar",
        str(cfr_jar),
        str(filtered_jar),
        "--outputdir",
        str(output_dir),
        "--caseinsensitivefs",
        "true",
        "--silent",
        "true",
    ]
    subprocess.run(cmd, check=True)


def replace_tree(src: Path, dst: Path) -> None:
    if dst.exists():
        shutil.rmtree(dst)
    ensure_parent(dst)
    os.replace(src, dst)


def refresh_symlink(link_path: Path, target: Path) -> None:
    if link_path.exists() or link_path.is_symlink():
        if link_path.is_dir() and not link_path.is_symlink():
            shutil.rmtree(link_path)
        else:
            link_path.unlink()
    link_path.symlink_to(target)


def manifest_payload(
    *,
    source_jar: Path,
    source_sha256: str,
    filtered_class_count: int,
    cfr_jar: Path,
    cfr_sha256: str,
    output_dir: Path,
    java_bin: Path,
    filter_prefix: str,
) -> dict:
    return {
        "source_jar": str(source_jar),
        "source_jar_sha256": source_sha256,
        "cfr_jar": str(cfr_jar),
        "cfr_jar_sha256": cfr_sha256,
        "java_bin": str(java_bin),
        "output_dir": str(output_dir),
        "filter_prefix": filter_prefix,
        "filtered_class_count": filtered_class_count,
    }


def manifest_is_current(manifest: dict | None, payload: dict, output_dir: Path) -> bool:
    if manifest is None or not output_dir.exists():
        return False
    return all(manifest.get(key) == value for key, value in payload.items())


def main() -> int:
    args = parse_args()
    source_jar = args.jar.expanduser().resolve()
    output_dir = args.output_dir.expanduser().resolve()
    manifest_path = args.manifest.expanduser().resolve()
    cfr_jar = args.cfr_jar.expanduser().resolve()
    java_bin = resolve_java_bin(args.java_bin.expanduser().resolve() if args.java_bin else None, source_jar)

    if not source_jar.exists():
        print(f"error: source jar not found: {source_jar}", file=sys.stderr)
        return 1

    ensure_parent(manifest_path)
    ensure_parent(output_dir)
    ensure_cfr_jar(cfr_jar, args.cfr_url)

    source_sha = sha256_file(source_jar)
    cfr_sha = sha256_file(cfr_jar)

    current_manifest = load_manifest(manifest_path)

    with tempfile.TemporaryDirectory(prefix="sts-decompile-", dir=str(DEFAULT_OUTPUT_ROOT)) as tmp_root_str:
        tmp_root = Path(tmp_root_str)
        filtered_jar = tmp_root / "cardcrawl-only.jar"
        filtered_class_count = build_filtered_jar(source_jar, filtered_jar, args.filter_prefix)
        if filtered_class_count == 0:
            print(
                f"error: no classes matched prefix {args.filter_prefix!r} inside {source_jar}",
                file=sys.stderr,
            )
            return 1

        payload = manifest_payload(
            source_jar=source_jar,
            source_sha256=source_sha,
            filtered_class_count=filtered_class_count,
            cfr_jar=cfr_jar,
            cfr_sha256=cfr_sha,
            output_dir=output_dir,
            java_bin=java_bin,
            filter_prefix=args.filter_prefix,
        )
        if not args.force and manifest_is_current(current_manifest, payload, output_dir):
            refresh_symlink(args.tmp_symlink, output_dir)
            print(f"up to date: {output_dir}")
            return 0

        tmp_output = tmp_root / "java-src"
        run_cfr(java_bin, cfr_jar, filtered_jar, tmp_output)
        if not (tmp_output / "com" / "megacrit" / "cardcrawl").exists():
            print("error: CFR output did not include com/megacrit/cardcrawl", file=sys.stderr)
            return 1

        replace_tree(tmp_output, output_dir)
        manifest_path.write_text(
            json.dumps(
                {
                    **payload,
                    "generated_at": datetime.now(timezone.utc).isoformat(),
                },
                indent=2,
                sort_keys=True,
            )
            + "\n"
        )
        refresh_symlink(args.tmp_symlink, output_dir)

    print(f"refreshed: {output_dir}")
    print(f"manifest: {manifest_path}")
    print(f"symlink:  {args.tmp_symlink} -> {output_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
