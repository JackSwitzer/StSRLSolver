"""Parser for Spirelogs run history files (from Baalorlord, Spireblight, etc.)."""

import json
from pathlib import Path
from typing import Optional
from dataclasses import dataclass, field


@dataclass
class CardChoice:
    """A card reward choice."""
    floor: int
    picked: str
    not_picked: list[str]
    outcome: str = "unknown"

    def to_training_format(self, character: str = "WATCHER", ascension: int = 20) -> dict:
        """Convert to training data format."""
        options = [self.picked] + self.not_picked
        return {
            "type": "card_reward",
            "floor": self.floor,
            "character": character,
            "ascension": ascension,
            "options": options,
            "chosen": self.picked,
            "chosen_idx": 0,
            "outcome": self.outcome,
        }


@dataclass
class CampfireChoice:
    """A campfire/rest site choice."""
    floor: int
    action: str  # SMITH, REST, LIFT, DIG, etc.
    data: Optional[str]  # Card upgraded, if SMITH

    def to_training_format(self, character: str = "WATCHER", ascension: int = 20) -> dict:
        return {
            "type": "rest_site",
            "floor": self.floor,
            "character": character,
            "ascension": ascension,
            "action": self.action,
            "card_upgraded": self.data if self.action == "SMITH" else None,
        }


@dataclass
class NeowChoice:
    """Neow bonus selection."""
    bonus: str
    cost: str
    skipped: list[str]

    def to_training_format(self, character: str = "WATCHER", ascension: int = 20) -> dict:
        return {
            "type": "neow",
            "floor": 0,
            "character": character,
            "ascension": ascension,
            "chosen": self.bonus,
            "cost": self.cost,
            "skipped_options": self.skipped,
        }


@dataclass
class RunHistory:
    """Parsed run history from Spirelogs format."""
    play_id: str
    seed: str
    character: str
    ascension: int
    floor_reached: int
    victory: bool
    score: int
    playtime: int  # seconds

    card_choices: list[CardChoice] = field(default_factory=list)
    campfire_choices: list[CampfireChoice] = field(default_factory=list)
    neow: Optional[NeowChoice] = None

    master_deck: list[str] = field(default_factory=list)
    relics: list[str] = field(default_factory=list)
    items_purged: list[str] = field(default_factory=list)

    @classmethod
    def from_file(cls, path: Path) -> "RunHistory":
        """Parse a run history JSON file."""
        with open(path) as f:
            data = json.load(f)
        return cls.from_dict(data)

    @classmethod
    def from_dict(cls, data: dict) -> "RunHistory":
        """Parse from dictionary."""
        # Determine character from filename or data
        character = data.get("character_chosen", "WATCHER")

        # Victory is indicated by reaching floor 51+ or specific victory key
        floor_reached = data.get("floor_reached", 0)
        victory = data.get("victory", floor_reached >= 51)

        run = cls(
            play_id=data.get("play_id", ""),
            seed=data.get("seed_played", ""),
            character=character,
            ascension=data.get("ascension_level", 20) if data.get("is_ascension_mode") else 0,
            floor_reached=floor_reached,
            victory=victory,
            score=data.get("score", 0),
            playtime=data.get("playtime", 0),
            master_deck=data.get("master_deck", []),
            relics=data.get("relics", []),
            items_purged=data.get("items_purged", []),
        )

        # Parse card choices
        for choice in data.get("card_choices", []):
            run.card_choices.append(CardChoice(
                floor=choice.get("floor", 0),
                picked=choice.get("picked", "SKIP"),
                not_picked=choice.get("not_picked", []),
                outcome="win" if run.victory else "loss",
            ))

        # Parse campfire choices
        for choice in data.get("campfire_choices", []):
            run.campfire_choices.append(CampfireChoice(
                floor=choice.get("floor", 0),
                action=choice.get("key", "REST"),
                data=choice.get("data"),
            ))

        # Parse Neow choice
        if data.get("neow_bonus"):
            run.neow = NeowChoice(
                bonus=data["neow_bonus"],
                cost=data.get("neow_cost", "NONE"),
                skipped=data.get("neow_bonuses_skipped_log", []),
            )

        return run

    def to_training_data(self) -> list[dict]:
        """Convert to list of training examples."""
        examples = []

        if self.neow:
            examples.append(self.neow.to_training_format(self.character, self.ascension))

        for choice in self.card_choices:
            examples.append(choice.to_training_format(self.character, self.ascension))

        for choice in self.campfire_choices:
            examples.append(choice.to_training_format(self.character, self.ascension))

        return examples


class SpirelogsParser:
    """Parser for directories of Spirelogs run files."""

    def __init__(self, data_dir: Path):
        self.data_dir = Path(data_dir)

    def parse_all_runs(self, filter_character: Optional[str] = "WATCHER") -> list[RunHistory]:
        """Parse all run files in directory.

        Args:
            filter_character: Only include runs with this character (None for all)

        Returns:
            List of parsed runs
        """
        runs = []

        for path in self.data_dir.glob("*.json"):
            try:
                run = RunHistory.from_file(path)
                if filter_character is None or filter_character.upper() in run.character.upper():
                    runs.append(run)
            except Exception as e:
                print(f"Error parsing {path}: {e}")
                continue

        return runs

    def aggregate_training_data(
        self,
        runs: list[RunHistory],
        wins_only: bool = False,
    ) -> dict:
        """Aggregate training data from multiple runs.

        Args:
            runs: List of parsed runs
            wins_only: Only include data from winning runs

        Returns:
            Aggregated training data by type
        """
        if wins_only:
            runs = [r for r in runs if r.victory]

        data = {
            "card_rewards": [],
            "rest_decisions": [],
            "neow_choices": [],
            "metadata": {
                "total_runs": len(runs),
                "wins": sum(1 for r in runs if r.victory),
                "losses": sum(1 for r in runs if not r.victory),
            },
        }

        for run in runs:
            examples = run.to_training_data()
            for ex in examples:
                if ex["type"] == "card_reward":
                    data["card_rewards"].append(ex)
                elif ex["type"] == "rest_site":
                    data["rest_decisions"].append(ex)
                elif ex["type"] == "neow":
                    data["neow_choices"].append(ex)

        return data

    def compute_card_stats(self, runs: list[RunHistory]) -> dict:
        """Compute pick rates and stats for each card.

        Returns:
            Dict mapping card_id -> {picked, offered, skipped_for, pick_rate}
        """
        stats = {}

        for run in runs:
            for choice in run.card_choices:
                # Track picked card
                picked = choice.picked
                if picked not in stats:
                    stats[picked] = {"picked": 0, "offered": 0, "skipped_for": []}
                stats[picked]["picked"] += 1
                stats[picked]["offered"] += 1

                # Track not-picked cards
                for card in choice.not_picked:
                    if card not in stats:
                        stats[card] = {"picked": 0, "offered": 0, "skipped_for": []}
                    stats[card]["offered"] += 1
                    stats[card]["skipped_for"].append(picked)

        # Compute pick rates
        for card, data in stats.items():
            data["pick_rate"] = data["picked"] / data["offered"] if data["offered"] > 0 else 0

        return stats


def load_baalorlord_data(data_dir: Path = Path("data/baalorlord/raw")) -> dict:
    """Load and aggregate Baalorlord's Watcher data.

    Returns:
        Aggregated training data dict
    """
    parser = SpirelogsParser(data_dir)
    runs = parser.parse_all_runs(filter_character="WATCHER")

    print(f"Loaded {len(runs)} Watcher runs")
    print(f"  Wins: {sum(1 for r in runs if r.victory)}")
    print(f"  Losses: {sum(1 for r in runs if not r.victory)}")

    return parser.aggregate_training_data(runs)
