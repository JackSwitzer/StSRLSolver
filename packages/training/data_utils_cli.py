"""CLI for data pipeline utilities.

Usage:
    uv run python -m packages.training.data_utils_cli inventory
    uv run python -m packages.training.data_utils_cli quality
    uv run python -m packages.training.data_utils_cli organize [--dry-run]
    uv run python -m packages.training.data_utils_cli tier
    uv run python -m packages.training.data_utils_cli replay [--seed SEED]
    uv run python -m packages.training.data_utils_cli prune [--dry-run]
    uv run python -m packages.training.data_utils_cli compress [--dry-run]

Or via training.sh:
    bash scripts/training.sh data inventory
    bash scripts/training.sh data quality
    bash scripts/training.sh data organize
    bash scripts/training.sh data tier
    bash scripts/training.sh data replay [seed]
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


def cmd_tier():
    """Run the tiering pipeline and report stats per tier."""
    from .data_tiers import TIER_CONFIGS, run_tier_pipeline, score_trajectories

    dirs = _all_traj_dirs()
    print("\n=== DATA TIER REPORT ===\n")

    results = run_tier_pipeline(dirs)
    for tier in TIER_CONFIGS:
        r = results[tier]
        print(f"  [{tier.value.upper():>8}] {len(r.accepted):4d} files | "
              f"{r.total_transitions:7d} transitions | {r.rejected:3d} rejected")
        print(f"            {TIER_CONFIGS[tier].description}")

    print("\n=== TOP 10 TRAJECTORIES (by quality score) ===\n")
    top = score_trajectories(dirs, top_n=10)
    for i, q in enumerate(top, 1):
        print(f"  {i:2d}. F{q.floor:02d} score={q.composite_score:.3f} "
              f"(floor={q.floor_score:.2f} hp={q.hp_preservation:.2f} "
              f"deck={q.deck_quality:.2f} decisions={q.decisions_count})")
        print(f"      {q.path.name}")

    print()


def cmd_replay(seed: str = "", index: int = -1):
    """Show game replay for a specific episode."""
    from .data_tiers import format_replay, replay_episode

    episodes_path = None
    active = Path("logs/active/episodes.jsonl")
    if active.exists():
        episodes_path = active
    else:
        for run in sorted(Path("logs/runs").glob("run_*"), reverse=True):
            ep = run / "episodes.jsonl"
            if ep.exists():
                episodes_path = ep
                break

    if episodes_path is None:
        print("No episodes.jsonl found.")
        return

    episode = replay_episode(episodes_path, episode_id=seed if seed else None, index=index)
    if episode is None:
        print(f"Episode not found (seed={seed}, index={index}).")
        return

    print(format_replay(episode))


def cmd_prune(dry_run: bool = False):
    """Prune old checkpoints across all runs."""
    from .data_tiers import prune_checkpoints

    total_pruned = 0
    for run_dir in sorted(Path("logs/runs").glob("run_*")):
        deleted = prune_checkpoints(run_dir, keep_latest=5, dry_run=dry_run)
        if deleted:
            action = "Would prune" if dry_run else "Pruned"
            print(f"  {action} {len(deleted)} checkpoints from {run_dir.name}")
            total_pruned += len(deleted)

    if total_pruned == 0:
        print("  No checkpoints to prune.")
    else:
        action = "Would prune" if dry_run else "Pruned"
        print(f"\n  {action} {total_pruned} total checkpoints.")


def cmd_compress(dry_run: bool = False):
    """Compress old episodes.jsonl files."""
    from .data_tiers import compress_old_jsonl

    compressed = compress_old_jsonl(Path("logs/runs"), dry_run=dry_run)
    if compressed:
        action = "Would compress" if dry_run else "Compressed"
        print(f"  {action} {len(compressed)} files.")
    else:
        print("  No files to compress.")


def main():
    parser = argparse.ArgumentParser(description="Data pipeline utilities")
    parser.add_argument(
        "command",
        choices=["inventory", "quality", "organize", "tier", "replay", "prune", "compress"],
    )
    parser.add_argument("--dry-run", action="store_true", help="Preview without writing")
    parser.add_argument("--seed", default="", help="Episode seed for replay")
    parser.add_argument("--index", type=int, default=-1, help="Episode index for replay")
    args = parser.parse_args()

    if args.command == "inventory":
        cmd_inventory()
    elif args.command == "quality":
        cmd_quality()
    elif args.command == "organize":
        cmd_organize(dry_run=args.dry_run)
    elif args.command == "tier":
        cmd_tier()
    elif args.command == "replay":
        cmd_replay(seed=args.seed, index=args.index)
    elif args.command == "prune":
        cmd_prune(dry_run=args.dry_run)
    elif args.command == "compress":
        cmd_compress(dry_run=args.dry_run)


if __name__ == "__main__":
    main()
