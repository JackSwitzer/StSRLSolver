"""CLI for data pipeline utilities.

Usage:
    uv run python -m packages.training.data_utils_cli inventory
    uv run python -m packages.training.data_utils_cli quality
    uv run python -m packages.training.data_utils_cli organize [--dry-run]

Or via training.sh:
    bash scripts/training.sh data inventory
    bash scripts/training.sh data quality
    bash scripts/training.sh data organize
"""

from __future__ import annotations

import argparse
import logging
import shutil
import sys
from pathlib import Path

import numpy as np

from .data_utils import (
    check_trajectory_quality,
    find_combat_files,
    find_trajectory_files,
    load_combat_data,
    load_trajectory_file,
    parse_floor_from_filename,
)

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [data] %(levelname)s %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("data_cli")

# All known data directories (relative to project root)
TRAJECTORY_DIRS = [
    Path("logs/pretrain_data"),
    Path("logs/v3_collect/all_trajectories"),
    Path("logs/v3_collect/best_trajectories"),
]
# Also search all run best_trajectories
RUN_DIR = Path("logs/runs")
ARCHIVE_DIR = Path("logs/archive")
COMBAT_SEARCH_DIRS = [Path("logs")]
DATA_ROOT = Path("logs/data")


def _all_traj_dirs() -> list[Path]:
    """Collect all directories that may contain trajectory files."""
    dirs = list(TRAJECTORY_DIRS)
    if RUN_DIR.exists():
        for run in RUN_DIR.iterdir():
            bt = run / "best_trajectories"
            if bt.exists():
                dirs.append(bt)
    if ARCHIVE_DIR.exists():
        for arch in ARCHIVE_DIR.rglob("*trajectories"):
            if arch.is_dir():
                dirs.append(arch)
    return dirs


def cmd_inventory():
    """Print a complete inventory of all training data."""
    dirs = _all_traj_dirs()
    files = find_trajectory_files(dirs)

    # Group by parent directory
    by_dir: dict[str, list[Path]] = {}
    for f in files:
        key = str(f.parent)
        by_dir.setdefault(key, []).append(f)

    total_size = 0
    total_files = 0
    print("\n=== TRAJECTORY DATA INVENTORY ===\n")
    for d in sorted(by_dir.keys()):
        flist = by_dir[d]
        size = sum(f.stat().st_size for f in flist)
        total_size += size
        total_files += len(flist)
        print(f"  {d}: {len(flist)} files ({size / 1024 / 1024:.1f} MB)")

    print(f"\n  Total: {total_files} trajectory files ({total_size / 1024 / 1024:.1f} MB)")

    # Combat data
    combat_files = find_combat_files(COMBAT_SEARCH_DIRS)
    combat_size = sum(f.stat().st_size for f in combat_files)
    print(f"\n=== COMBAT DATA ===\n")
    print(f"  {len(combat_files)} combat files ({combat_size / 1024 / 1024:.1f} MB)")

    # Checkpoints
    ckpt_dir = Path("logs/strategic_checkpoints")
    if ckpt_dir.exists():
        ckpts = list(ckpt_dir.glob("*.pt"))
        ckpt_size = sum(f.stat().st_size for f in ckpts)
        print(f"\n=== CHECKPOINTS ===\n")
        print(f"  {len(ckpts)} checkpoints ({ckpt_size / 1024 / 1024:.1f} MB)")

    print()


def cmd_quality():
    """Run quality checks and print report."""
    from .training_config import MODEL_ACTION_DIM

    dirs = _all_traj_dirs()
    print("\n=== DATA QUALITY REPORT ===\n")

    report = check_trajectory_quality(
        dirs,
        expected_obs_dim=480,
        expected_action_dim=MODEL_ACTION_DIM,
    )

    print(f"  Total files:           {report.total_files}")
    print(f"  Total transitions:     {report.total_transitions}")
    print(f"  Valid files (480-dim): {report.valid_files}")
    print(f"  Valid transitions:     {report.valid_transitions}")
    print(f"  Usable:               {report.usable_pct:.1f}%")
    print(f"  Dim mismatches:        {report.dim_mismatch_files}")
    print(f"  NaN rewards:           {report.nan_reward_files}")
    print(f"  Extreme rewards:       {report.extreme_reward_files}")
    print(f"  Invalid actions:       {report.invalid_action_transitions}")

    print(f"\n  Obs dim distribution:")
    for dim, count in sorted(report.obs_dim_distribution.items()):
        marker = " <-- current" if dim == 480 else " (stale)"
        print(f"    {dim}: {count} files{marker}")

    print(f"\n  Floor distribution:")
    for floor, count in sorted(report.floor_distribution.items()):
        bar = "#" * min(count // 5, 40)
        print(f"    F{floor:02d}: {count:4d} {bar}")

    print()


def cmd_organize(dry_run: bool = False):
    """Organize data into tiered directory structure.

    logs/data/raw/      -- all valid trajectory files (symlinked)
    logs/data/filtered/ -- dimension-consistent (480-dim) files (copied)
    logs/data/curated/  -- high-quality games (floor 20+, correct masking) (copied)
    """
    DATA_ROOT.mkdir(parents=True, exist_ok=True)
    for sub in ("raw", "filtered", "curated"):
        (DATA_ROOT / sub).mkdir(exist_ok=True)

    dirs = _all_traj_dirs()
    files = find_trajectory_files(dirs)

    raw_count = 0
    filtered_count = 0
    curated_count = 0

    for f in files:
        # Raw: symlink everything
        raw_dest = DATA_ROOT / "raw" / f.name
        if not raw_dest.exists():
            if dry_run:
                raw_count += 1
            else:
                # Use unique name to avoid collisions
                unique_name = f"{f.parent.name}_{f.name}"
                raw_dest = DATA_ROOT / "raw" / unique_name
                if not raw_dest.exists():
                    raw_dest.symlink_to(f.resolve())
                    raw_count += 1

        # Filtered: copy 480-dim files
        traj = load_trajectory_file(f, expected_obs_dim=480, expected_action_dim=512)
        if traj is None:
            continue

        filtered_dest = DATA_ROOT / "filtered" / f.name
        if not filtered_dest.exists():
            unique_name = f"{f.parent.name}_{f.name}"
            filtered_dest = DATA_ROOT / "filtered" / unique_name
            if not filtered_dest.exists():
                if dry_run:
                    filtered_count += 1
                else:
                    shutil.copy2(f, filtered_dest)
                    filtered_count += 1

        # Curated: floor 20+ only
        if traj.floor >= 20:
            curated_dest = DATA_ROOT / "curated" / f.name
            if not curated_dest.exists():
                unique_name = f"{f.parent.name}_{f.name}"
                curated_dest = DATA_ROOT / "curated" / unique_name
                if not curated_dest.exists():
                    if dry_run:
                        curated_count += 1
                    else:
                        shutil.copy2(f, curated_dest)
                        curated_count += 1

    action = "Would copy" if dry_run else "Organized"
    print(f"\n{action}:")
    print(f"  raw/      {raw_count} new symlinks")
    print(f"  filtered/ {filtered_count} new files (480-dim, valid)")
    print(f"  curated/  {curated_count} new files (floor 20+)")
    print()


def main():
    parser = argparse.ArgumentParser(description="Data pipeline utilities")
    parser.add_argument("command", choices=["inventory", "quality", "organize"])
    parser.add_argument("--dry-run", action="store_true", help="Preview without writing")
    args = parser.parse_args()

    if args.command == "inventory":
        cmd_inventory()
    elif args.command == "quality":
        cmd_quality()
    elif args.command == "organize":
        cmd_organize(dry_run=args.dry_run)


if __name__ == "__main__":
    main()
