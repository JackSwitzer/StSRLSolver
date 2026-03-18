"""Prune and archive training data. Safe to run while training is active.

Usage:
    uv run python scripts/prune_data.py                # Run with defaults
    uv run python scripts/prune_data.py --dry-run      # Preview only
    uv run python scripts/prune_data.py --keep 20000   # Keep last 20K episodes
    uv run python scripts/prune_data.py --top 1000     # Keep top 1000 by floor
    uv run python scripts/prune_data.py --run-dir logs/active  # Explicit dir

Safety:
    - Reads episodes.jsonl with append-only semantics (snapshot line count, read that many)
    - Writes archives and pruned files atomically (write tmp -> rename)
    - Never modifies files in-place
    - --dry-run shows what would happen without touching disk
"""

from __future__ import annotations

import argparse
import gzip
import json
import os
import shutil
import sys
import tempfile
import time
from datetime import datetime, timedelta
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
DEFAULT_RUN_DIR = PROJECT_ROOT / "logs" / "active"
ARCHIVE_DIR = PROJECT_ROOT / "logs" / "archive"


# -- Disk usage helpers -------------------------------------------------------

def dir_size_bytes(path: Path) -> int:
    """Total size of all files under path (non-recursive for files, recursive for dirs)."""
    total = 0
    if not path.exists():
        return 0
    if path.is_file():
        return path.stat().st_size
    for entry in path.rglob("*"):
        if entry.is_file():
            total += entry.stat().st_size
    return total


def fmt_size(nbytes: int) -> str:
    """Human-readable file size."""
    for unit in ("B", "KB", "MB", "GB"):
        if abs(nbytes) < 1024:
            return f"{nbytes:.1f} {unit}"
        nbytes /= 1024  # type: ignore[assignment]
    return f"{nbytes:.1f} TB"


def file_age_hours(path: Path) -> float:
    """Hours since file was last modified."""
    return (time.time() - path.stat().st_mtime) / 3600


# -- Core operations ----------------------------------------------------------

def count_lines(path: Path) -> int:
    """Count lines in a file without reading it all into memory."""
    count = 0
    with open(path, "rb") as f:
        for _ in f:
            count += 1
    return count


def read_episodes_safe(path: Path, max_lines: int | None = None) -> list[dict]:
    """Read episodes.jsonl, skipping malformed lines. Reads at most max_lines."""
    episodes = []
    with open(path) as f:
        for i, line in enumerate(f):
            if max_lines is not None and i >= max_lines:
                break
            line = line.strip()
            if not line:
                continue
            try:
                episodes.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return episodes


def consolidate_top_runs(
    run_dir: Path,
    top_n: int,
    dry_run: bool,
) -> int:
    """Read episodes.jsonl, extract top N by floor (+ all wins), write to top_episodes.json.

    Returns number of episodes selected.
    """
    episodes_path = run_dir / "episodes.jsonl"
    if not episodes_path.exists():
        print("  [skip] episodes.jsonl not found")
        return 0

    total_lines = count_lines(episodes_path)
    print(f"  Reading {total_lines:,} episodes from episodes.jsonl ...")

    # Snapshot: read exactly total_lines to avoid racing with appender
    all_eps = read_episodes_safe(episodes_path, max_lines=total_lines)
    print(f"  Parsed {len(all_eps):,} valid episodes")

    # Separate wins and losses
    wins = [ep for ep in all_eps if ep.get("won")]
    losses = [ep for ep in all_eps if not ep.get("won")]

    # Sort losses by floor descending, take top N minus wins count
    losses.sort(key=lambda ep: ep.get("floor", 0), reverse=True)
    slots_for_losses = max(0, top_n - len(wins))
    top_losses = losses[:slots_for_losses]

    selected = wins + top_losses
    selected.sort(key=lambda ep: ep.get("floor", 0), reverse=True)

    min_floor = selected[-1].get("floor", 0) if selected else 0
    max_floor = selected[0].get("floor", 0) if selected else 0
    avg_floor = sum(ep.get("floor", 0) for ep in selected) / len(selected) if selected else 0

    print(f"  Selected {len(selected):,} episodes ({len(wins)} wins, {len(top_losses)} top losses)")
    print(f"  Floor range: {min_floor}-{max_floor}, avg: {avg_floor:.1f}")

    if dry_run:
        print("  [dry-run] Would write top_episodes.json")
        return len(selected)

    # Atomic write: tmp file -> rename
    out_path = run_dir / "top_episodes.json"
    output = {
        "meta": {
            "total_episodes": len(selected),
            "total_source_episodes": len(all_eps),
            "wins": len(wins),
            "avg_floor": round(avg_floor, 2),
            "floor_range": [min_floor, max_floor],
            "pruned_at": datetime.now().isoformat(),
        },
        "episodes": selected,
    }
    tmp_path = out_path.with_suffix(".json.tmp")
    with open(tmp_path, "w") as f:
        json.dump(output, f, indent=2)
    tmp_path.rename(out_path)
    print(f"  Wrote {fmt_size(out_path.stat().st_size)} to top_episodes.json")
    return len(selected)


def compress_episodes(
    run_dir: Path,
    keep_last: int,
    dry_run: bool,
) -> int:
    """Keep last N episodes in episodes.jsonl, archive the rest as gzipped JSONL.

    Returns bytes saved.
    """
    episodes_path = run_dir / "episodes.jsonl"
    if not episodes_path.exists():
        print("  [skip] episodes.jsonl not found")
        return 0

    total_lines = count_lines(episodes_path)
    original_size = episodes_path.stat().st_size

    if total_lines <= keep_last:
        print(f"  [skip] Only {total_lines:,} episodes, below threshold of {keep_last:,}")
        return 0

    archive_count = total_lines - keep_last
    print(f"  Total: {total_lines:,} episodes ({fmt_size(original_size)})")
    print(f"  Archiving oldest {archive_count:,}, keeping newest {keep_last:,}")

    if dry_run:
        print("  [dry-run] Would archive and compress")
        return 0

    # Create archive directory
    ARCHIVE_DIR.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    archive_path = ARCHIVE_DIR / f"episodes_{timestamp}.jsonl.gz"

    # Read the file in two passes:
    # Pass 1: write old lines to gzip archive
    # Pass 2: write recent lines to tmp file, then rename
    bytes_archived = 0
    with open(episodes_path, "rb") as src, gzip.open(archive_path, "wb", compresslevel=6) as gz:
        for i, line in enumerate(src):
            if i >= archive_count:
                break
            gz.write(line)
            bytes_archived += len(line)

    print(f"  Archived {archive_count:,} episodes to {archive_path.name} ({fmt_size(archive_path.stat().st_size)} compressed)")

    # Write the tail (keep_last lines) to a temp file, then atomic rename
    tmp_path = episodes_path.with_suffix(".jsonl.tmp")
    lines_written = 0
    with open(episodes_path, "rb") as src, open(tmp_path, "wb") as dst:
        for i, line in enumerate(src):
            if i >= archive_count:
                dst.write(line)
                lines_written += 1

    # Atomic rename
    tmp_path.rename(episodes_path)
    new_size = episodes_path.stat().st_size
    saved = original_size - new_size
    print(f"  episodes.jsonl: {fmt_size(original_size)} -> {fmt_size(new_size)} (saved {fmt_size(saved)})")
    print(f"  Kept {lines_written:,} episodes")
    return saved


def clean_worker_status(
    run_dir: Path,
    max_age_hours: float,
    dry_run: bool,
) -> int:
    """Remove stale worker status files older than max_age_hours.

    Returns number of files removed.
    """
    workers_dir = run_dir / "workers"
    if not workers_dir.exists():
        print("  [skip] No workers/ directory")
        return 0

    removed = 0
    for f in workers_dir.iterdir():
        if not f.is_file():
            continue
        age = file_age_hours(f)
        if age > max_age_hours:
            if dry_run:
                print(f"  [dry-run] Would remove {f.name} (age: {age:.1f}h)")
            else:
                f.unlink()
                print(f"  Removed {f.name} (age: {age:.1f}h)")
            removed += 1

    if removed == 0:
        print(f"  No stale worker files (all < {max_age_hours}h old)")
    return removed


def report_disk_usage(run_dir: Path, label: str) -> int:
    """Print disk usage breakdown. Returns total bytes."""
    print(f"\n{'=' * 50}")
    print(f"Disk usage ({label})")
    print(f"{'=' * 50}")

    total = 0
    items = [
        ("episodes.jsonl", run_dir / "episodes.jsonl"),
        ("top_episodes.json", run_dir / "top_episodes.json"),
        ("recent_episodes.json", run_dir / "recent_episodes.json"),
        ("floor_curve.json", run_dir / "floor_curve.json"),
        ("status.json", run_dir / "status.json"),
        ("checkpoints (*.pt)", None),  # special
        ("nohup logs", None),  # special
        ("workers/", run_dir / "workers"),
        ("best_trajectories/", run_dir / "best_trajectories"),
        ("archive/", ARCHIVE_DIR),
    ]

    for name, path in items:
        if name == "checkpoints (*.pt)":
            size = sum(f.stat().st_size for f in run_dir.glob("*.pt") if f.is_file())
        elif name == "nohup logs":
            size = sum(f.stat().st_size for f in run_dir.glob("nohup*.log") if f.is_file())
        elif path is None:
            continue
        elif path.is_file():
            size = path.stat().st_size if path.exists() else 0
        elif path.is_dir():
            size = dir_size_bytes(path)
        else:
            size = 0

        if size > 0:
            print(f"  {name:<30s} {fmt_size(size):>10s}")
            total += size

    print(f"  {'TOTAL':<30s} {fmt_size(total):>10s}")
    return total


# -- Main ---------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Prune and archive STS RL training data.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument(
        "--run-dir",
        type=Path,
        default=DEFAULT_RUN_DIR,
        help=f"Training run directory (default: {DEFAULT_RUN_DIR.relative_to(PROJECT_ROOT)})",
    )
    parser.add_argument(
        "--top",
        type=int,
        default=500,
        help="Number of top episodes to keep in top_episodes.json (default: 500)",
    )
    parser.add_argument(
        "--keep",
        type=int,
        default=10_000,
        help="Number of recent episodes to keep in episodes.jsonl (default: 10000)",
    )
    parser.add_argument(
        "--worker-age",
        type=float,
        default=1.0,
        help="Remove worker status files older than N hours (default: 1.0)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview changes without modifying any files",
    )
    parser.add_argument(
        "--skip-compress",
        action="store_true",
        help="Skip episode compression (only consolidate top runs + clean workers)",
    )
    parser.add_argument(
        "--skip-top",
        action="store_true",
        help="Skip top episode consolidation",
    )
    args = parser.parse_args()

    run_dir: Path = args.run_dir.resolve()
    if not run_dir.exists():
        print(f"ERROR: Run directory does not exist: {run_dir}", file=sys.stderr)
        sys.exit(1)

    if args.dry_run:
        print("*** DRY RUN — no files will be modified ***\n")

    # -- Pre-prune disk usage --
    size_before = report_disk_usage(run_dir, "BEFORE")

    # -- Step 1: Consolidate top runs --
    if not args.skip_top:
        print(f"\n--- Consolidating top {args.top} episodes ---")
        consolidate_top_runs(run_dir, args.top, args.dry_run)

    # -- Step 2: Compress episodes.jsonl --
    if not args.skip_compress:
        print(f"\n--- Compressing episodes.jsonl (keeping last {args.keep:,}) ---")
        compress_episodes(run_dir, args.keep, args.dry_run)

    # -- Step 3: Clean stale worker files --
    print(f"\n--- Cleaning stale worker files (>{args.worker_age}h) ---")
    clean_worker_status(run_dir, args.worker_age, args.dry_run)

    # -- Post-prune disk usage --
    if not args.dry_run:
        size_after = report_disk_usage(run_dir, "AFTER")
        saved = size_before - size_after
        # Archive dir is new space used, account for it
        archive_size = dir_size_bytes(ARCHIVE_DIR)
        net_saved = size_before - size_after
        print(f"\nNet change: {fmt_size(abs(net_saved))} {'freed' if net_saved > 0 else 'added'}")
        if archive_size > 0:
            print(f"Archive dir total: {fmt_size(archive_size)}")
    else:
        print("\n[dry-run] No files modified.")


if __name__ == "__main__":
    main()
