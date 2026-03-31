"""Data tiering, quality scoring, and deck analysis for trajectory data.

Provides:
- DataTier enum with RAW/FILTERED/CURATED/EXPERT levels
- TierConfig dataclass for configurable filtering per tier
- Quality scoring per trajectory (floor, HP preservation, deck quality)
- Deck composition analysis and archetype detection
- Checkpoint pruning utilities
"""

from __future__ import annotations

import enum
import gzip
import json
import logging
import shutil
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Protocol, Sequence

import numpy as np

from .data_utils import (
    TrajectoryData,
    find_trajectory_files,
    load_trajectory_file,
    parse_floor_from_filename,
)

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Data Tier Enum
# ---------------------------------------------------------------------------

class DataTier(enum.Enum):
    RAW = "raw"
    FILTERED = "filtered"
    CURATED = "curated"
    EXPERT = "expert"


# ---------------------------------------------------------------------------
# Tier Configuration
# ---------------------------------------------------------------------------

@dataclass
class TierConfig:
    """Configuration for a data tier's filtering criteria."""
    tier: DataTier
    min_floor: int = 0
    min_hp_pct: float = 0.0
    min_deck_size: int = 0
    max_deck_size: int = 999
    expected_obs_dim: Optional[int] = None
    expected_mask_dim: Optional[int] = None
    no_nan_rewards: bool = False
    max_reward: Optional[float] = None
    description: str = ""


TIER_CONFIGS: Dict[DataTier, TierConfig] = {
    DataTier.RAW: TierConfig(
        tier=DataTier.RAW,
        description="All trajectories, no filtering",
    ),
    DataTier.FILTERED: TierConfig(
        tier=DataTier.FILTERED,
        expected_obs_dim=480,
        expected_mask_dim=512,
        no_nan_rewards=True,
        max_reward=100.0,
        description="Dimension-consistent, no NaN, no extreme rewards",
    ),
    DataTier.CURATED: TierConfig(
        tier=DataTier.CURATED,
        min_floor=10,
        min_hp_pct=0.30,
        min_deck_size=10,
        max_deck_size=30,
        expected_obs_dim=480,
        expected_mask_dim=512,
        no_nan_rewards=True,
        max_reward=100.0,
        description="Floor>=10, HP>30%, deck 10-30 cards",
    ),
    DataTier.EXPERT: TierConfig(
        tier=DataTier.EXPERT,
        min_floor=17,
        expected_obs_dim=480,
        expected_mask_dim=512,
        no_nan_rewards=True,
        max_reward=100.0,
        description="Floor>=17 (beat Act 1 boss) or known-good seeds",
    ),
}


# ---------------------------------------------------------------------------
# Tier Filter Protocol
# ---------------------------------------------------------------------------

class TierFilter(Protocol):
    def accepts(self, path: Path, data: dict[str, np.ndarray]) -> bool: ...


class DefaultTierFilter:
    """Default filter that applies TierConfig rules to a trajectory file."""

    def __init__(self, config: TierConfig):
        self.config = config

    def accepts(self, path: Path, data: dict[str, np.ndarray]) -> bool:
        c = self.config

        if "obs" in data:
            if c.expected_obs_dim is not None and data["obs"].shape[1] != c.expected_obs_dim:
                return False

        if "masks" in data:
            if c.expected_mask_dim is not None and data["masks"].shape[1] != c.expected_mask_dim:
                return False

        if c.no_nan_rewards and "rewards" in data:
            if np.any(np.isnan(data["rewards"])):
                return False

        if c.max_reward is not None and "rewards" in data:
            if np.any(np.abs(data["rewards"]) > c.max_reward):
                return False

        floor = parse_floor_from_filename(path)
        if "floor" in data and len(data["floor"]) > 0:
            floor = int(data["floor"][0])
        if floor < c.min_floor:
            return False

        return True


# ---------------------------------------------------------------------------
# Tier Pipeline
# ---------------------------------------------------------------------------

@dataclass
class TierResult:
    tier: DataTier
    accepted: List[Path]
    rejected: int
    total_transitions: int


def run_tier_pipeline(
    dirs: Sequence[Path],
    configs: Optional[Dict[DataTier, TierConfig]] = None,
) -> Dict[DataTier, TierResult]:
    """Run the full tiering pipeline, classifying files into tiers."""
    if configs is None:
        configs = TIER_CONFIGS

    files = find_trajectory_files(dirs)
    results: Dict[DataTier, TierResult] = {}

    for tier, config in configs.items():
        filt = DefaultTierFilter(config)
        accepted: List[Path] = []
        rejected = 0
        total_transitions = 0

        for f in files:
            try:
                data = dict(np.load(f))
                if filt.accepts(f, data):
                    accepted.append(f)
                    if "obs" in data:
                        total_transitions += len(data["obs"])
                else:
                    rejected += 1
            except Exception as e:
                logger.warning("Tier %s: failed to load %s: %s", tier.value, f.name, e)
                rejected += 1

        results[tier] = TierResult(
            tier=tier,
            accepted=accepted,
            rejected=rejected,
            total_transitions=total_transitions,
        )

    return results


# ---------------------------------------------------------------------------
# Quality Scoring
# ---------------------------------------------------------------------------

@dataclass
class TrajectoryQuality:
    path: Path
    floor: int
    composite_score: float
    floor_score: float
    hp_preservation: float
    deck_quality: float
    decisions_count: int


def _estimate_deck_quality(traj: TrajectoryData) -> float:
    """Estimate deck quality from trajectory data.
    Uses mask-based deck size proxy. Scoring: 12-25 cards -> 1.0, <12 -> 0.7,
    >25 -> max(0.4, 1.0-(size-25)*0.03). Bonus +0.1 for upgrade proxy, +0.1 for scaling.
    """
    if len(traj.obs) == 0 or len(traj.masks) == 0:
        return 0.7
    avg_valid = float(np.mean(traj.masks.sum(axis=1)))
    est_deck = avg_valid * 2.5
    if 12 <= est_deck <= 25: q = 1.0
    elif est_deck < 12: q = 0.7
    else: q = max(0.4, 1.0 - (est_deck - 25) * 0.03)
    if len(traj.rewards) > 0 and float(np.mean(traj.rewards)) > 0.3: q += 0.1
    if len(traj.rewards) >= 10:
        h = len(traj.rewards) // 2
        if float(np.mean(traj.rewards[h:])) > float(np.mean(traj.rewards[:h])) + 0.1: q += 0.1
    return round(q, 4)


def score_trajectory(traj: TrajectoryData) -> TrajectoryQuality:
    """Score trajectory DATA QUALITY for tiering (how useful is this data
    for training), NOT game state value (that's compute_potential in
    reward_config.py).
    Composite = 0.5*floor + 0.2*hp + 0.15*deck + 0.15*norm_decisions
    """
    floor_score = min(traj.floor / 55.0, 1.0)
    hp_preservation = float(np.mean(traj.final_floors)) if len(traj.final_floors) > 0 else 0.0
    decisions = len(traj.obs)
    norm_decisions = min(decisions / 200.0, 1.0)
    deck_quality = _estimate_deck_quality(traj)

    composite = (
        0.50 * floor_score
        + 0.20 * hp_preservation
        + 0.15 * deck_quality
        + 0.15 * norm_decisions
    )

    return TrajectoryQuality(
        path=traj.source_path,
        floor=traj.floor,
        composite_score=round(composite, 4),
        floor_score=round(floor_score, 4),
        hp_preservation=round(hp_preservation, 4),
        deck_quality=round(deck_quality, 4),
        decisions_count=decisions,
    )


def score_trajectories(
    dirs: Sequence[Path],
    expected_obs_dim: int = 480,
    expected_action_dim: int = 512,
    top_n: int = 20,
) -> List[TrajectoryQuality]:
    """Score all trajectories and return top N by composite score."""
    files = find_trajectory_files(dirs)
    scores: List[TrajectoryQuality] = []

    for f in files:
        traj = load_trajectory_file(f, expected_obs_dim=expected_obs_dim, expected_action_dim=expected_action_dim)
        if traj is None:
            continue
        scores.append(score_trajectory(traj))

    scores.sort(key=lambda q: q.composite_score, reverse=True)
    return scores[:top_n]


# ---------------------------------------------------------------------------
# Deck Analysis
# ---------------------------------------------------------------------------

# TODO: source from engine card metadata
ATTACK_CARDS = {
    "Eruption", "Tantrum", "FlyingSleeves", "Ragnarok", "Conclude",
    "SashWhip", "CrushJoints", "FollowUp", "FlurryOfBlows",
    "WindmillStrike", "Wallop", "WheelKick", "ReachHeaven",
    "TalkToTheHand", "Weave", "CutThroughFate", "SignatureMove",
    "FearNoEvil", "Brilliance", "Judgment", "LessonLearned",
}

DEFENSE_CARDS = {
    "Protect", "ThirdEye", "Crescendo", "Tranquility",
    "EmptyBody", "SanctityCard", "WaveOfTheHand", "Meditate",
    "Perseverance", "SpiritShield", "MentalFortress",
}

STANCE_CARDS = {
    "Eruption", "Tantrum", "InnerPeace", "Crescendo", "Tranquility",
    "EmptyFist", "EmptyBody", "EmptyMind", "FearNoEvil",
    "FlurryOfBlows", "MentalFortress", "Rushdown",
}

SCALING_CARDS = {
    "Mantra", "Devotion", "Worship", "Prostrate", "Pray",
    "Study", "Establishment", "MasterReality", "Vault",
    "Omniscience", "Scrawl", "Alpha", "BattleHymn",
}


@dataclass
class DeckArchetype:
    name: str
    attack_pct: float
    defense_pct: float
    stance_pct: float
    scaling_pct: float


def classify_deck(cards: List[str]) -> DeckArchetype:
    """Classify a deck into an archetype based on card composition."""
    if not cards:
        return DeckArchetype("empty", 0, 0, 0, 0)

    total = len(cards)
    attack_pct = sum(1 for c in cards if c in ATTACK_CARDS) / total
    defense_pct = sum(1 for c in cards if c in DEFENSE_CARDS) / total
    stance_pct = sum(1 for c in cards if c in STANCE_CARDS) / total
    scaling_pct = sum(1 for c in cards if c in SCALING_CARDS) / total

    if stance_pct >= 0.25:
        name = "stance-cycling"
    elif attack_pct >= 0.40:
        name = "attack-heavy"
    elif defense_pct >= 0.30:
        name = "defense-heavy"
    elif scaling_pct >= 0.20:
        name = "scaling"
    else:
        name = "balanced"

    return DeckArchetype(
        name=name,
        attack_pct=round(attack_pct, 3),
        defense_pct=round(defense_pct, 3),
        stance_pct=round(stance_pct, 3),
        scaling_pct=round(scaling_pct, 3),
    )


@dataclass
class ArchetypeStats:
    archetype: str
    count: int
    avg_floor: float
    max_floor: int
    win_count: int


def analyze_decks(episodes_path: Path) -> Dict[str, ArchetypeStats]:
    """Analyze deck compositions from episodes.jsonl and group by archetype."""
    archetype_data: Dict[str, List[Dict[str, Any]]] = {}

    if not episodes_path.exists():
        logger.warning("Episodes file not found: %s", episodes_path)
        return {}

    with open(episodes_path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                ep = json.loads(line)
                deck = ep.get("deck_final", [])
                if not deck: continue
                arch = classify_deck(deck)
                archetype_data.setdefault(arch.name, []).append({
                    "floor": ep.get("floor", 0),
                    "won": ep.get("won", False),
                })
            except (json.JSONDecodeError, KeyError, TypeError) as e:
                logger.warning("Skipping malformed episode line: %s", e)
                continue

    result: Dict[str, ArchetypeStats] = {}
    for name, episodes in archetype_data.items():
        floors = [e["floor"] for e in episodes]
        wins = sum(1 for e in episodes if e["won"])
        result[name] = ArchetypeStats(
            archetype=name,
            count=len(episodes),
            avg_floor=round(sum(floors) / len(floors), 2) if floors else 0.0,
            max_floor=max(floors) if floors else 0,
            win_count=wins,
        )

    return result


def write_deck_analysis(episodes_path: Path, output_path: Path) -> Dict[str, Any]:
    """Run deck analysis and write results to JSON."""
    stats = analyze_decks(episodes_path)
    analysis = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "episodes_file": str(episodes_path),
        "archetypes": {
            name: {
                "count": s.count,
                "avg_floor": s.avg_floor,
                "max_floor": s.max_floor,
                "win_count": s.win_count,
            }
            for name, s in sorted(stats.items(), key=lambda x: -x[1].avg_floor)
        },
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(analysis, f, indent=2)

    return analysis


# ---------------------------------------------------------------------------
# Checkpoint Pruning
# ---------------------------------------------------------------------------

def prune_checkpoints(
    run_dir: Path,
    keep_latest: int = 5,
    keep_best: bool = True,
    dry_run: bool = False,
) -> List[Path]:
    """Prune old checkpoints, keeping latest N + shutdown + best."""
    if not run_dir.exists():
        return []

    all_ckpts = sorted(run_dir.glob("checkpoint_*.pt"), key=lambda p: p.stat().st_mtime)
    protected = {"shutdown_checkpoint.pt"}
    if keep_best:
        protected.add("best_checkpoint.pt")

    to_keep = set(all_ckpts[-keep_latest:]) if keep_latest > 0 else set()
    for name in protected:
        p = run_dir / name
        if p.exists():
            to_keep.add(p)

    to_delete: List[Path] = []
    for ckpt in all_ckpts:
        if ckpt not in to_keep:
            to_delete.append(ckpt)
            if not dry_run:
                ckpt.unlink()
                logger.info("Pruned checkpoint: %s", ckpt.name)

    return to_delete


# ---------------------------------------------------------------------------
# JSONL Compression
# ---------------------------------------------------------------------------

def compress_old_jsonl(
    search_dir: Path,
    max_age_hours: float = 24.0,
    dry_run: bool = False,
) -> List[Path]:
    """Gzip episodes.jsonl files older than max_age_hours."""
    now = time.time()
    cutoff = now - (max_age_hours * 3600)
    compressed: List[Path] = []

    if not search_dir.exists():
        return compressed

    for jsonl in search_dir.rglob("episodes.jsonl"):
        if jsonl.stat().st_mtime > cutoff:
            continue
        gz_path = jsonl.with_suffix(".jsonl.gz")
        if gz_path.exists():
            continue

        if dry_run:
            compressed.append(gz_path)
            continue

        with open(jsonl, "rb") as f_in:
            with gzip.open(gz_path, "wb") as f_out:
                shutil.copyfileobj(f_in, f_out)
        jsonl.unlink()
        compressed.append(gz_path)
        logger.info("Compressed: %s -> %s", jsonl.name, gz_path.name)

    return compressed


# ---------------------------------------------------------------------------
# Game Replay
# ---------------------------------------------------------------------------

def replay_episode(episodes_path: Path, episode_id: Optional[str] = None, index: int = -1) -> Optional[Dict[str, Any]]:
    """Load a single episode from episodes.jsonl for replay."""
    if not episodes_path.exists():
        return None

    episodes: List[Dict[str, Any]] = []
    with open(episodes_path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                ep = json.loads(line)
                if episode_id and str(ep.get("seed", "")) == str(episode_id):
                    return ep
                episodes.append(ep)
            except (json.JSONDecodeError, KeyError, TypeError) as e:
                logger.warning("Skipping malformed episode in replay: %s", e)
                continue

    if episode_id:
        return None

    if not episodes:
        return None

    try:
        return episodes[index]
    except IndexError:
        return None


def format_replay(episode: Dict[str, Any]) -> str:
    """Format an episode dict into human-readable replay text."""
    lines = []
    lines.append(f"=== Game Replay: Seed {episode.get('seed', '?')} ===")
    lines.append(f"Floor: {episode.get('floor', '?')} | Won: {episode.get('won', False)}")
    lines.append(f"HP: {episode.get('hp', '?')}/{episode.get('max_hp', '?')}")
    lines.append(f"Decisions: {episode.get('decisions', '?')} | Duration: {episode.get('duration_s', 0):.1f}s")
    lines.append(f"Total reward: {episode.get('total_reward', '?')}")

    deck = episode.get("deck_final", [])
    if deck:
        arch = classify_deck(deck)
        lines.append(f"\nDeck ({len(deck)} cards, {arch.name}):")
        card_counts: Dict[str, int] = {}
        for card in deck:
            card_counts[card] = card_counts.get(card, 0) + 1
        for card, count in sorted(card_counts.items()):
            lines.append(f"  {card} x{count}" if count > 1 else f"  {card}")

    relics = episode.get("relics_final", [])
    if relics:
        lines.append(f"\nRelics ({len(relics)}):")
        for r in relics:
            lines.append(f"  {r}")

    combats = episode.get("combats", [])
    if combats:
        lines.append(f"\nCombats ({len(combats)}):")
        for c in combats:
            if isinstance(c, dict):
                enemy = c.get("enemies", c.get("enemy", "?"))
                won = c.get("won", "?")
                hp_lost = c.get("hp_lost", "?")
                lines.append(f"  F{c.get('floor', '?')}: {enemy} - {'Won' if won else 'Lost'} (HP lost: {hp_lost})")
            else:
                lines.append(f"  {c}")

    paths = episode.get("path_choices", [])
    if paths:
        lines.append(f"\nPath choices: {paths}")

    death_enemy = episode.get("death_enemy", "")
    if death_enemy:
        lines.append(f"\nDied to: {death_enemy} (room: {episode.get('death_room', '?')})")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Export Protocol
# ---------------------------------------------------------------------------

class DataExporter(Protocol):
    def export(self, tier: DataTier, files: List[Path], dest_path: Path) -> None: ...


class LocalExporter:
    """Export tiered data as a tar.gz to a local path."""

    def export(self, tier: DataTier, files: List[Path], dest_path: Path) -> None:
        import tarfile

        dest_path.mkdir(parents=True, exist_ok=True)
        archive_name = dest_path / f"{tier.value}_data.tar.gz"

        with tarfile.open(archive_name, "w:gz") as tar:
            for f in files:
                tar.add(f, arcname=f.name)

        logger.info("Exported %d files to %s", len(files), archive_name)


class GDriveExporter:
    """Stub for future Google Drive export integration."""

    def export(self, tier: DataTier, files: List[Path], dest_path: Path) -> None:
        raise NotImplementedError("Google Drive export not yet implemented")
