"""
Lightweight web GUI server for playing Slay the Spire via the RUST engine.

Run with: bash scripts/play_gui.sh
"""
from __future__ import annotations

import os
import sys
import logging
from typing import Any, Dict, List, Optional

from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles
from fastapi.responses import FileResponse
import uvicorn

logger = logging.getLogger(__name__)

app = FastAPI(title="Slay the Spire Play GUI (Rust Engine)")

# ---------------------------------------------------------------------------
# Game state (single-player, single-session)
# ---------------------------------------------------------------------------

engine = None  # sts_engine.StSEngine instance
action_history: List[int] = []  # action IDs (ints)
current_seed: str = ""
current_ascension: int = 20


def _rust_state_to_frontend(state: dict) -> dict:
    """Reshape Rust engine state dict into the format the frontend expects.

    Rust returns flat keys (hp, gold, floor, combat, deck, relics, potions).
    Frontend expects obs.run.{hp, gold, ...} and obs.phase as lowercase.
    """
    # Phase mapping: Rust uses short names, frontend expects lowercase
    phase_map = {
        "map": "map", "combat": "combat", "card_reward": "reward",
        "shop": "shop", "event": "event", "campfire": "rest",
        "game_over": "game_over", "treasure": "treasure",
    }
    phase = phase_map.get(state.get("phase", ""), state.get("phase", ""))

    # Build obs.run (the frontend reads obs.run.*)
    run = {
        "current_hp": state.get("hp", 0),
        "max_hp": state.get("max_hp", 0),
        "gold": state.get("gold", 0),
        "floor": state.get("floor", 0),
        "act": state.get("act", 1),
        "deck": state.get("deck", []),
        "relics": state.get("relics", []),
        "potions": state.get("potions", []),
        "seed": state.get("seed", ""),
    }

    obs = {"phase": phase, "run": run}

    # Combat state
    if "combat" in state and state["combat"]:
        c = state["combat"]
        enemies = []
        for e in (c.get("enemies") or []):
            enemies.append({
                "id": e.get("name", e.get("id", "?")),
                "name": e.get("name", "?"),
                "hp": e.get("hp", 0),
                "max_hp": e.get("max_hp", 0),
                "block": e.get("block", 0),
                "alive": e.get("alive", True),
                "move_damage": e.get("move_damage", 0),
                "move_hits": e.get("move_hits", 0),
                "move_block": e.get("move_block", 0),
                "statuses": e.get("statuses", {}),
            })
        obs["combat"] = {
            "player": {
                "hp": state.get("hp", 0),
                "max_hp": state.get("max_hp", 0),
                "block": c.get("block", 0),
                "energy": c.get("energy", 0),
                "max_energy": c.get("max_energy", 3),
                "statuses": c.get("player_statuses", {}),
            },
            "hand": c.get("hand", []),
            "enemies": enemies,
            "draw_pile_count": c.get("draw_pile_size", 0),
            "discard_pile_count": c.get("discard_pile_size", 0),
            "exhaust_pile_count": c.get("exhaust_pile_size", 0),
            "stance": c.get("stance", "Neutral"),
            "turn": c.get("turn", 0),
            "card_costs": {},  # Rust doesn't track per-card cost overrides yet
        }

    # Card rewards
    if "card_rewards" in state:
        obs["reward"] = {"card_rewards": [{"cards": [{"id": c} for c in state["card_rewards"]]}]}

    # Shop
    if "shop" in state and state["shop"]:
        obs["shop"] = state["shop"]

    # Event
    if "event_options" in state:
        obs["event"] = {"options": list(range(state["event_options"]))}

    return obs


def _get_state_response() -> Dict[str, Any]:
    """Build the JSON response from the current Rust engine state."""
    if engine is None:
        return {
            "observation": None,
            "actions": [],
            "seed": "",
            "game_over": True,
            "can_undo": False,
            "action_count": 0,
        }

    state = engine.get_state()
    obs = _rust_state_to_frontend(state)

    # Actions: Rust returns ActionInfo objects, convert to dicts
    action_infos = engine.get_legal_actions()
    actions = []
    for ai in action_infos:
        actions.append({
            "id": ai.id,
            "type": ai.action_type,
            "label": ai.description or ai.name,
            "name": ai.name,
            "card_name": ai.card_name,
            "target": ai.target,
            "description": ai.description,
        })

    game_over = state.get("done", False) or not actions

    return {
        "observation": obs,
        "actions": actions,
        "seed": current_seed,
        "game_over": game_over,
        "can_undo": len(action_history) > 0,
        "action_count": len(action_history),
    }


# ---------------------------------------------------------------------------
# API endpoints
# ---------------------------------------------------------------------------

@app.post("/api/new_game")
async def new_game(body: dict) -> Dict[str, Any]:
    global engine, action_history, current_seed, current_ascension
    import sts_engine

    current_seed = body.get("seed", "TEST123")
    current_ascension = int(body.get("ascension", 20))

    engine = sts_engine.StSEngine(current_seed, current_ascension)
    action_history = []
    return _get_state_response()


@app.get("/api/state")
async def get_state() -> Dict[str, Any]:
    return _get_state_response()


@app.post("/api/action")
async def take_action(body: dict) -> Dict[str, Any]:
    global engine, action_history
    if engine is None:
        return _get_state_response()

    action_id = body.get("id")
    if action_id is None:
        return {**_get_state_response(), "action_error": "No action id provided"}

    action_history.append(action_id)
    try:
        engine.step(action_id)
    except Exception as exc:
        action_history.pop()
        return {**_get_state_response(), "action_error": str(exc)}
    return _get_state_response()


@app.get("/api/undo")
async def undo() -> Dict[str, Any]:
    global engine, action_history
    if not action_history:
        return _get_state_response()

    action_history.pop()

    # Replay from scratch
    import sts_engine
    engine = sts_engine.StSEngine(current_seed, current_ascension)
    for aid in action_history:
        engine.step(aid)

    return _get_state_response()


# ---------------------------------------------------------------------------
# Static files & SPA fallback
# ---------------------------------------------------------------------------

_STATIC_DIR = os.path.join(os.path.dirname(__file__), "static")


@app.get("/")
async def index():
    return FileResponse(os.path.join(_STATIC_DIR, "index.html"))


app.mount("/static", StaticFiles(directory=_STATIC_DIR), name="static")


if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8421)
