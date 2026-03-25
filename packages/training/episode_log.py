"""Episode logging: writes per-game results to episodes.jsonl."""

from __future__ import annotations

import json
from datetime import datetime
from pathlib import Path
from typing import Any, Dict

import numpy as np


class _NumpyEncoder(json.JSONEncoder):
    def default(self, obj):
        if isinstance(obj, (np.integer,)):
            return int(obj)
        if isinstance(obj, (np.floating,)):
            return float(obj)
        if isinstance(obj, np.ndarray):
            return obj.tolist()
        return super().default(obj)


def log_episode(episodes_path: Path, result: Dict[str, Any], config_name: str = "") -> None:
    """Append one episode to episodes.jsonl."""
    transitions = result.get("transitions", [])
    total_reward = sum(t.get("reward", 0) for t in transitions)
    total_pbrs = sum(t.get("pbrs", 0) for t in transitions)
    total_event = sum(t.get("event_reward", 0) for t in transitions)

    entry = {
        "timestamp": datetime.now().isoformat(),
        "config_name": config_name,
        "seed": result["seed"],
        "floor": result["floor"],
        "won": result["won"],
        "hp": result["hp"],
        "max_hp": result.get("max_hp", 0),
        "decisions": result["decisions"],
        "duration_s": result["duration_s"],
        "num_transitions": len(transitions),
        "total_reward": round(total_reward, 4),
        "pbrs_reward": round(total_pbrs, 4),
        "event_reward": round(total_event, 4),
        "deck_final": result.get("deck_final", []),
        "relics_final": result.get("relics_final", []),
        "death_enemy": result.get("death_enemy", ""),
        "death_room": result.get("room_type", ""),
        "combats": result.get("combats", []),
        "events": result.get("events", []),
        "path_choices": result.get("path_choices", []),
        "construction_failure": result.get("construction_failure", False),
    }
    with open(episodes_path, "a") as f:
        f.write(json.dumps(entry, cls=_NumpyEncoder) + "\n")
