"""
Lightweight web GUI server for playing Slay the Spire via the engine.

Run with: uv run uvicorn packages.play.server:app --host 0.0.0.0 --port 8421
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

# Ensure project root is on the path so engine imports work.
_PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
if _PROJECT_ROOT not in sys.path:
    sys.path.insert(0, _PROJECT_ROOT)

logger = logging.getLogger(__name__)

app = FastAPI(title="Slay the Spire Play GUI")

# ---------------------------------------------------------------------------
# Game state (single-player, single-session)
# ---------------------------------------------------------------------------

runner = None
action_history: List[Dict[str, Any]] = []
current_seed: str = ""
current_ascension: int = 20
current_character: str = "Watcher"


def _lookup_card_cost(card_id: str) -> int:
    """Look up a card's energy cost from the card registry."""
    try:
        from packages.engine.content.cards import get_card
        base_id = card_id.rstrip("+")
        upgraded = card_id.endswith("+")
        card = get_card(base_id, upgraded=upgraded)
        return card.cost
    except Exception:
        return -1


def _enrich_actions(actions: List[Dict[str, Any]], obs: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Add human-readable labels to actions where the engine gives generic ones."""
    phase = obs.get("phase", "")

    # Neow: replace "Neow choice N" with blessing description
    if phase == "neow" and runner and runner.neow_blessings:
        for a in actions:
            if a.get("type") == "neow_choice":
                idx = a.get("params", {}).get("choice_index", -1)
                if 0 <= idx < len(runner.neow_blessings):
                    b = runner.neow_blessings[idx]
                    desc = b.description
                    if b.drawback_description:
                        desc += " (cost: " + b.drawback_description + ")"
                    a["label"] = desc

    # Combat: replace "Play card N" with actual card name + cost
    if phase == "combat" and obs.get("combat"):
        hand = obs["combat"].get("hand", [])
        card_costs = obs["combat"].get("card_costs", {})
        for a in actions:
            if a.get("type") == "play_card":
                params = a.get("params", {})
                ci = params.get("card_index", -1)
                if 0 <= ci < len(hand):
                    card_id = hand[ci]
                    cost = card_costs.get(card_id)
                    if cost is None:
                        cost = _lookup_card_cost(card_id)
                    ti = params.get("target_index")
                    enemies = obs["combat"].get("enemies", [])
                    target_str = ""
                    if ti is not None and 0 <= ti < len(enemies):
                        target_str = " -> " + enemies[ti].get("id", "?")
                    a["label"] = card_id + " [" + str(cost) + "]" + target_str
            elif a.get("type") == "use_potion":
                params = a.get("params", {})
                slot = params.get("potion_slot", -1)
                potions = (obs.get("run") or {}).get("potions", [])
                if 0 <= slot < len(potions) and potions[slot]:
                    a["label"] = "Use " + potions[slot]

    # Map: replace "Path to node N" with room type
    if phase == "map" and obs.get("map"):
        paths = obs["map"].get("available_paths", [])
        for a in actions:
            if a.get("type") == "path_choice":
                ni = a.get("params", {}).get("node_index", -1)
                if 0 <= ni < len(paths):
                    p = paths[ni]
                    room = p.get("room_type", "?")
                    a["label"] = room + " (x:" + str(p.get("x", "?")) + ")"

    # Rewards: enrich card pick labels
    if phase in ("reward", "boss_reward") and obs.get("reward"):
        card_rewards = obs["reward"].get("card_rewards", [])
        for a in actions:
            if a.get("type") == "pick_card":
                params = a.get("params", {})
                ri = params.get("card_reward_index", 0)
                ci = params.get("card_index", 0)
                if 0 <= ri < len(card_rewards):
                    cards = card_rewards[ri].get("cards", [])
                    if 0 <= ci < len(cards):
                        c = cards[ci]
                        name = c["id"] + ("+" if c.get("upgraded") else "")
                        a["label"] = "Pick " + name

    # Rest: enrich smith labels with card name
    if phase == "rest":
        deck = (obs.get("run") or {}).get("deck", [])
        for a in actions:
            if a.get("type") == "smith":
                ci = a.get("params", {}).get("card_index", -1)
                if 0 <= ci < len(deck):
                    c = deck[ci]
                    a["label"] = "Upgrade " + c["id"]

    return actions


def _get_state_response() -> Dict[str, Any]:
    """Build the JSON response from the current runner state."""
    if runner is None:
        return {
            "error": "No game started",
            "observation": None,
            "actions": [],
            "seed": "",
            "game_over": True,
            "can_undo": False,
            "action_count": 0,
        }

    obs = runner.get_observation(profile="human")
    actions = runner.get_available_action_dicts()
    actions = _enrich_actions(actions, obs)
    game_over = runner.game_over or not actions

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
    global runner, action_history, current_seed, current_ascension, current_character
    from packages.engine.game import GameRunner

    current_seed = body.get("seed", "TEST123")
    current_ascension = int(body.get("ascension", 20))
    current_character = body.get("character", "Watcher")

    runner = GameRunner(
        seed=current_seed,
        ascension=current_ascension,
        character=current_character,
        skip_neow=False,
        verbose=False,
    )
    action_history = []
    return _get_state_response()


@app.get("/api/state")
async def get_state() -> Dict[str, Any]:
    return _get_state_response()


@app.post("/api/action")
async def take_action(action_dict: dict) -> Dict[str, Any]:
    global runner, action_history
    if runner is None:
        return _get_state_response()

    action_history.append(action_dict)
    try:
        runner.take_action_dict(action_dict)
    except Exception as exc:
        # Roll back the failed action from history
        action_history.pop()
        return {
            **_get_state_response(),
            "action_error": str(exc),
        }
    return _get_state_response()


@app.get("/api/undo")
async def undo() -> Dict[str, Any]:
    global runner, action_history
    if not action_history:
        return _get_state_response()

    action_history.pop()

    # Replay from scratch with the same seed
    from packages.engine.game import GameRunner

    runner = GameRunner(
        seed=current_seed,
        ascension=current_ascension,
        character=current_character,
        skip_neow=False,
        verbose=False,
    )
    for a in action_history:
        runner.take_action_dict(a)

    return _get_state_response()


# ---------------------------------------------------------------------------
# Static files & SPA fallback
# ---------------------------------------------------------------------------

_STATIC_DIR = os.path.join(os.path.dirname(__file__), "static")


@app.get("/")
async def index():
    return FileResponse(os.path.join(_STATIC_DIR, "index.html"))


app.mount("/static", StaticFiles(directory=_STATIC_DIR), name="static")


# ---------------------------------------------------------------------------
# Entrypoint
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8421)
