#!/usr/bin/env python3
"""
Slay the Spire Live Dashboard Server

A real-time web dashboard for tracking game state with predictions.
Uses Server-Sent Events (SSE) for live updates.

Usage:
    uv run python web/server.py

Then open http://localhost:8080
"""

import asyncio
import json
import os
import sys
import time
from pathlib import Path
from typing import Optional, Dict, Any, List

# Add core to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from fastapi import FastAPI, Request
from fastapi.responses import HTMLResponse, StreamingResponse, JSONResponse
from fastapi.staticfiles import StaticFiles
import uvicorn

# Import core modules
from core.comparison.full_rng_tracker import (
    read_save_file, predict_boss_relics, predict_card_reward,
    SAVE_PATH, CLASS_STARTER_RELICS, ROOM_SYMBOLS
)
from core.state.rng import long_to_seed, seed_to_long, Random
from core.state.game_rng import GameRNGState, RNGStream
from core.generation.map import (
    MapGenerator, MapGeneratorConfig, RoomType,
    get_map_seed_offset, MapRoomNode
)
from core.content.events import ALL_EVENTS, get_events_for_act, Act
from core.generation.encounters import predict_act_encounters, predict_all_acts, predict_all_bosses_extended
from core.generation.shop import predict_shop_inventory, format_shop_inventory
from core.generation.treasure import predict_chest, predict_full_chest, ChestType
from core.generation.potions import predict_potion_drop, predict_potion_from_seed


app = FastAPI(title="Slay the Spire Live Dashboard")

# Serve static files
STATIC_DIR = Path(__file__).parent / "static"
if STATIC_DIR.exists():
    app.mount("/static", StaticFiles(directory=str(STATIC_DIR)), name="static")


# ============================================================================
# GAME STATE PROCESSING
# ============================================================================

def get_current_map(seed_long: int, act: int, ascension: int = 20) -> List[List[Dict]]:
    """Generate the current act's map."""
    from core.state.rng import Random

    # Map seed offset
    offset = get_map_seed_offset(act)
    map_seed = seed_long + offset

    config = MapGeneratorConfig(ascension_level=ascension)
    map_rng = Random(map_seed)
    generator = MapGenerator(map_rng, config)

    dungeon = generator.generate()

    # Convert to JSON-serializable format
    map_data = []
    for row_idx, row in enumerate(dungeon):
        row_data = []
        for node in row:
            node_data = {
                "x": node.x,
                "y": node.y,
                "room_type": node.room_type.value if node.room_type else None,
                "room_name": node.room_type.name if node.room_type else None,
                "has_edges": node.has_edges(),
                "edges": [
                    {"dst_x": e.dst_x, "dst_y": e.dst_y, "is_boss": e.is_boss}
                    for e in node.edges
                ],
                "has_emerald_key": node.has_emerald_key,
            }
            row_data.append(node_data)
        map_data.append(row_data)

    return map_data


def find_node_at(map_data: List[List[Dict]], x: int, y: int) -> Optional[Dict]:
    """Find node at specific (x, y) coordinates."""
    if y < 0 or y >= len(map_data):
        return None
    for node in map_data[y]:
        if node["x"] == x:
            return node
    return None


def find_accessible_nodes(map_data: List[List[Dict]], current_x: int, current_y: int) -> List[Dict]:
    """Find all nodes accessible from current position."""
    accessible = []

    if current_y < 0 or (current_x < 0 and current_y == 0):
        # At floor 0 (before first move), can access all nodes in row 0 that have edges
        for node in map_data[0]:
            if node["has_edges"]:
                accessible.append(node)
    elif current_y >= len(map_data):
        # Beyond the map (at boss or done with act)
        # Create a synthetic boss node
        accessible.append({
            "x": 3,
            "y": current_y,
            "room_type": "B",
            "room_name": "BOSS",
            "has_edges": False,
            "edges": [],
            "has_emerald_key": False,
        })
    elif current_x >= 0 and current_y < len(map_data):
        # On the map - find edges from current node by coordinate lookup
        current_node = find_node_at(map_data, current_x, current_y)
        if current_node:
            for edge in current_node["edges"]:
                if edge["is_boss"]:
                    # Boss edge - create synthetic boss node
                    accessible.append({
                        "x": edge["dst_x"],
                        "y": edge["dst_y"],
                        "room_type": "B",
                        "room_name": "BOSS",
                        "has_edges": False,
                        "edges": [],
                        "has_emerald_key": False,
                    })
                else:
                    # Find target node by coordinates
                    target = find_node_at(map_data, edge["dst_x"], edge["dst_y"])
                    if target:
                        accessible.append(target)

    return accessible


def build_path_tree(
    map_data: List[List[Dict]],
    current_x: int,
    current_y: int,
    depth: int = 3,
) -> Dict[str, List[Dict]]:
    """
    Build a tree of possible paths for the next N floors.

    Returns a dict like:
    {
        "floor_1": [list of accessible nodes from current position],
        "floor_2": [list of all nodes reachable from floor_1 nodes],
        "floor_3": [list of all nodes reachable from floor_2 nodes],
    }

    Each node includes a "parents" field showing which floor_N-1 nodes lead to it.
    """
    path_tree = {}

    # Get floor 1 nodes (immediately accessible)
    floor_1_nodes = find_accessible_nodes(map_data, current_x, current_y)

    # Add parent info to floor 1 nodes (they all come from current position)
    for node in floor_1_nodes:
        node = dict(node)  # Copy to avoid mutating original
        node["parents"] = [{"x": current_x, "y": current_y}]

    path_tree["floor_1"] = [dict(n) for n in floor_1_nodes]

    # Build subsequent floors
    prev_floor_nodes = floor_1_nodes
    for floor_num in range(2, depth + 1):
        current_floor_nodes = {}  # key: (x, y), value: node with parents

        for prev_node in prev_floor_nodes:
            # Find nodes accessible from this previous floor node
            next_nodes = find_accessible_nodes(map_data, prev_node["x"], prev_node["y"])

            for next_node in next_nodes:
                key = (next_node["x"], next_node["y"])
                if key not in current_floor_nodes:
                    # First time seeing this node - copy it and add parents list
                    node_copy = dict(next_node)
                    node_copy["parents"] = [{"x": prev_node["x"], "y": prev_node["y"]}]
                    current_floor_nodes[key] = node_copy
                else:
                    # Already seen - add this parent to the list
                    parent = {"x": prev_node["x"], "y": prev_node["y"]}
                    if parent not in current_floor_nodes[key]["parents"]:
                        current_floor_nodes[key]["parents"].append(parent)

        floor_nodes = list(current_floor_nodes.values())
        path_tree[f"floor_{floor_num}"] = floor_nodes
        prev_floor_nodes = floor_nodes

    return path_tree


def enrich_path_tree_with_predictions(
    path_tree: Dict[str, List[Dict]],
    seed_str: str,
    card_counter: int,
    card_blizzard: int,
    act: int,
    owned_relics: List[str],
    seed_long: int,
    boss_relics_taken: int,
    potion_counter: int,
    blizzard_mod: int,
    merchant_counter: int,
    treasure_counter: int,
    player_class: str,
    monsters: List[str],
    elites: List[str],
    events: List[str],
    monsters_used: int,
    elites_used: int,
    events_used: int,
) -> Dict[str, List[Dict]]:
    """
    Enrich path tree nodes with predictions (card rewards, enemy names, etc).

    Note: This uses simplified prediction that doesn't track counter changes
    across hypothetical paths. For floor 1, predictions are accurate. For
    floors 2-3, they show what WOULD happen if you went directly there.
    """
    enriched_tree = {}

    # Track counters for each floor
    # This is a simplification - we predict as if taking the "fastest" path
    floor_monster_offset = 0
    floor_elite_offset = 0
    floor_event_offset = 0

    for floor_key in ["floor_1", "floor_2", "floor_3"]:
        if floor_key not in path_tree:
            continue

        enriched_nodes = []
        floor_num = int(floor_key.split("_")[1])

        for node in path_tree[floor_key]:
            enriched = dict(node)
            room_type = node.get("room_type")

            # Add enemy/event name based on room type and how many we've "used"
            # This is approximate for floors 2-3 since we don't know the exact path
            if room_type == "M":
                monster_idx = monsters_used + floor_monster_offset
                if monster_idx < len(monsters):
                    enriched["enemy"] = monsters[monster_idx]
            elif room_type == "E":
                elite_idx = elites_used + floor_elite_offset
                if elite_idx < len(elites):
                    enriched["enemy"] = elites[elite_idx]
            elif room_type == "?":
                event_idx = events_used + floor_event_offset
                if event_idx < len(events):
                    enriched["event"] = events[event_idx]

            # For floor 1, add full predictions (these are accurate)
            if floor_num == 1:
                relics_from_chests = {"COMMON": 0, "UNCOMMON": 0, "RARE": 0}
                pred = predict_node_rewards(
                    seed_str, card_counter, card_blizzard, act, node,
                    owned_relics, seed_long, boss_relics_taken,
                    potion_counter=potion_counter,
                    blizzard_mod=blizzard_mod,
                    merchant_counter=merchant_counter,
                    treasure_counter=treasure_counter,
                    relics_from_chests=relics_from_chests,
                    player_class=player_class,
                )

                # Extract key prediction info
                if pred.get("card_reward") and not pred["card_reward"].get("error"):
                    enriched["cards"] = pred["card_reward"]["cards"]
                if pred.get("potion_drop") and not pred["potion_drop"].get("error"):
                    if pred["potion_drop"]["will_drop"]:
                        enriched["potion"] = pred["potion_drop"]["potion_name"]
                if pred.get("shop_inventory") and not pred["shop_inventory"].get("error"):
                    shop = pred["shop_inventory"]
                    enriched["shop_cards"] = [c["card_name"] for c in shop.get("colored_cards", [])]
                    enriched["shop_relics"] = [r["relic_name"] for r in shop.get("relics", [])]
                if pred.get("treasure") and not pred["treasure"].get("error"):
                    enriched["relic"] = pred["treasure"]["relic_name"]
                if pred.get("boss_relics"):
                    enriched["boss_relics"] = pred["boss_relics"]

            enriched_nodes.append(enriched)

        enriched_tree[floor_key] = enriched_nodes

        # Update offsets for next floor based on room types in this floor
        # This is a simplification - counts one of each type per floor
        floor_has_monster = any(n.get("room_type") == "M" for n in enriched_nodes)
        floor_has_elite = any(n.get("room_type") == "E" for n in enriched_nodes)
        floor_has_event = any(n.get("room_type") == "?" for n in enriched_nodes)

        if floor_has_monster:
            floor_monster_offset += 1
        if floor_has_elite:
            floor_elite_offset += 1
        if floor_has_event:
            floor_event_offset += 1

    return enriched_tree


def predict_node_rewards(
    seed_str: str,
    card_counter: int,
    card_blizzard: int,
    act: int,
    node: Dict,
    owned_relics: List[str],
    seed_long: int,
    boss_relics_taken: int,
    potion_counter: int = 0,
    blizzard_mod: int = 0,
    merchant_counter: int = 0,
    treasure_counter: int = 0,
    relics_from_chests: Optional[Dict[str, int]] = None,
    player_class: str = "WATCHER",
) -> Dict[str, Any]:
    """Predict rewards for a specific node."""
    predictions = {
        "node": node,
        "card_reward": None,
        "boss_relics": None,
        "event_info": None,
        "potion_drop": None,
        "shop_inventory": None,
        "treasure": None,
    }

    room_type = node.get("room_type")

    if room_type in ["M", "E"]:
        # Combat node - predict card reward
        rt = "elite" if room_type == "E" else "normal"
        try:
            cards, new_counter = predict_card_reward(
                seed_str, card_counter, act, rt,
                card_blizzard=card_blizzard,
                relics=owned_relics,
            )
            predictions["card_reward"] = {
                "cards": cards,
                "counter_before": card_counter,
                "counter_after": new_counter,
            }
        except Exception as e:
            predictions["card_reward"] = {"error": str(e)}

        # Predict potion drop for combat nodes
        try:
            has_white_beast = "White Beast Statue" in owned_relics
            has_sozu = "Sozu" in owned_relics

            potion_pred = predict_potion_from_seed(
                seed=seed_str,
                potion_counter=potion_counter,
                blizzard_mod=blizzard_mod,
                player_class=player_class,
                room_type="elite" if room_type == "E" else "monster",
                has_white_beast_statue=has_white_beast,
                has_sozu=has_sozu,
            )
            predictions["potion_drop"] = {
                "will_drop": potion_pred.will_drop,
                "potion_id": potion_pred.potion_id,
                "potion_name": potion_pred.potion.name if potion_pred.potion else None,
                "rarity": potion_pred.rarity.name if potion_pred.rarity else None,
                "drop_chance": potion_pred.drop_chance,
                "drop_roll": potion_pred.drop_roll,
                "new_blizzard_mod": potion_pred.new_blizzard_mod,
                "new_potion_counter": potion_pred.new_potion_counter,
            }
        except Exception as e:
            predictions["potion_drop"] = {"error": str(e)}

    elif room_type == "$":
        # Shop node - predict full shop inventory
        try:
            has_membership = "Membership Card" in owned_relics
            has_courier = "The Courier" in owned_relics

            shop_result = predict_shop_inventory(
                seed=seed_str,
                card_counter=card_counter,
                merchant_counter=merchant_counter,
                potion_counter=potion_counter,
                act=act,
                player_class=player_class,
                owned_relics=set(owned_relics),
                purge_count=0,  # TODO: track purge count from save
                has_membership_card=has_membership,
                has_the_courier=has_courier,
            )
            inv = shop_result.inventory

            predictions["shop_inventory"] = {
                "colored_cards": [
                    {
                        "card_id": sc.card.id,
                        "card_name": sc.card.name,
                        "rarity": sc.card.rarity.name,
                        "price": sc.price,
                        "on_sale": sc.on_sale,
                    }
                    for sc in inv.colored_cards
                ],
                "colorless_cards": [
                    {
                        "card_id": sc.card.id,
                        "card_name": sc.card.name,
                        "rarity": sc.card.rarity.name,
                        "price": sc.price,
                    }
                    for sc in inv.colorless_cards
                ],
                "relics": [
                    {
                        "relic_id": sr.relic.id,
                        "relic_name": sr.relic.name,
                        "tier": sr.relic.tier.name,
                        "price": sr.price,
                    }
                    for sr in inv.relics
                ],
                "potions": [
                    {
                        "potion_id": sp.potion.id,
                        "potion_name": sp.potion.name,
                        "rarity": sp.potion.rarity.name,
                        "price": sp.price,
                    }
                    for sp in inv.potions
                ],
                "purge_cost": inv.purge_cost,
                "sale_card_index": inv.sale_card_index,
                "final_card_counter": shop_result.final_card_counter,
                "final_merchant_counter": shop_result.final_merchant_counter,
                "final_potion_counter": shop_result.final_potion_counter,
            }
        except Exception as e:
            predictions["shop_inventory"] = {"error": str(e)}

    elif room_type == "T":
        # Treasure node - predict chest contents
        try:
            has_nloths_face = "N'loth's Hungry Face" in owned_relics
            # TODO: properly track whether N'loth's first empty chest has triggered
            nloths_triggered = False

            chest_pred = predict_full_chest(
                seed=seed_long,
                treasure_counter=treasure_counter,
                player_class=player_class,
                relics_obtained=relics_from_chests,
                has_nloths_face=has_nloths_face,
                nloths_face_triggered=nloths_triggered,
            )
            predictions["treasure"] = {
                "chest_type": chest_pred.chest_type.value,
                "relic_tier": chest_pred.relic_tier,
                "relic_name": chest_pred.relic_name,
                "has_gold": chest_pred.has_gold,
                "gold_amount": chest_pred.gold_amount,
            }
        except Exception as e:
            predictions["treasure"] = {"error": str(e)}

    elif room_type == "B":
        # Boss node - predict boss relics and card reward
        try:
            has_starter = any(
                r in owned_relics
                for r in CLASS_STARTER_RELICS.values()
            )

            boss_relics = predict_boss_relics(
                seed_long,
                player_class=player_class,
                has_starter_relic=has_starter,
                relics_already_taken=boss_relics_taken,
                already_owned_relics=owned_relics,
            )
            predictions["boss_relics"] = boss_relics
        except Exception as e:
            predictions["boss_relics"] = {"error": str(e)}

        # Boss card reward (uses "elite" room type for boss fights)
        try:
            cards, new_counter = predict_card_reward(
                seed_str, card_counter, act, "elite",
                card_blizzard=card_blizzard,
                relics=owned_relics,
            )
            predictions["card_reward"] = {
                "cards": cards,
                "counter_before": card_counter,
                "counter_after": new_counter,
            }
        except Exception as e:
            predictions["card_reward"] = {"error": str(e)}

    elif room_type == "?":
        # Event node - show possible events
        act_enum = {1: Act.ACT_1, 2: Act.ACT_2, 3: Act.ACT_3}.get(act, Act.ACT_1)
        events = get_events_for_act(act_enum)

        # List event names and brief info
        event_list = []
        for event_id, event in events.items():
            choices = [
                {"index": c.index, "description": c.description}
                for c in event.choices[:3]  # Limit to first 3 choices
            ]
            event_list.append({
                "id": event_id,
                "name": event.name,
                "choices": choices,
            })

        predictions["event_info"] = {
            "possible_events": event_list[:10],  # Limit display
            "total_count": len(events),
        }

    return predictions


def calculate_rng_accuracy(
    save_data: Dict,
    seed_str: str,
    player_class: str,
) -> Dict[str, Any]:
    """
    Calculate RNG prediction accuracy by comparing predictions with actual results.

    Compares:
    - Card rewards: predicted cards vs what was actually offered
    - Boss relics: predicted relics vs what was actually offered
    - Potions: predicted drops vs what was actually obtained (limited tracking)

    Returns accuracy statistics and mismatches for debugging.
    """
    accuracy = {
        "cards": {"correct": 0, "total": 0, "ratio": 0.0},
        "relics": {"correct": 0, "total": 0, "ratio": 0.0},
        "potions": {"correct": 0, "total": 0, "ratio": 0.0},
        "overall": {"correct": 0, "total": 0, "ratio": 0.0},
        "mismatches": [],
    }

    # Get actual card choices from save
    card_choices = save_data.get("metric_card_choices", [])
    path_taken = save_data.get("metric_path_per_floor", [])

    # Get actual boss relics from save
    boss_relics_actual = save_data.get("boss_relics", [])

    # Get seed info
    seed_long = save_data.get("seed", 0)

    # Track card RNG state - we need to simulate the progression
    # Start from the beginning and predict each card reward
    card_counter = 0
    card_blizzard = 5  # Starting value

    # Get relics obtained during run (affects some predictions)
    relics_obtained_list = save_data.get("relics_obtained", [])
    owned_relics = []  # Track relics as we progress
    starter_relics = ["Burning Blood", "Ring of the Snake", "Cracked Core", "PureWater"]
    for sr in starter_relics:
        if sr in save_data.get("relics", []):
            owned_relics.append(sr)
            break

    # Process each floor's card choice
    for choice in card_choices:
        floor_num = choice.get("floor", 0)
        actual_picked = choice.get("picked")
        actual_not_picked = choice.get("not_picked", [])

        # All cards that were actually offered
        actual_offered = []
        if actual_picked and actual_picked != "SKIP":
            actual_offered.append(actual_picked)
        actual_offered.extend(actual_not_picked)

        if not actual_offered:
            continue

        # Determine room type for this floor
        if floor_num - 1 < len(path_taken):
            room_type = path_taken[floor_num - 1]
        else:
            room_type = "M"  # Default to monster

        # Determine act for this floor
        if floor_num <= 17:
            act = 1
        elif floor_num <= 34:
            act = 2
        elif floor_num <= 51:
            act = 3
        else:
            act = 3

        # Map room symbol to reward type
        reward_type = "normal"
        if room_type == "E":
            reward_type = "elite"
        elif room_type == "B":
            reward_type = "elite"  # Boss uses elite reward type

        # Only predict for combat rooms (M, E, B)
        if room_type not in ["M", "E", "B"]:
            continue

        # Try to predict what cards should have been offered
        try:
            predicted_cards, new_counter = predict_card_reward(
                seed_str,
                card_counter,
                act,
                reward_type,
                card_blizzard=card_blizzard,
                relics=owned_relics,
            )
            card_counter = new_counter

            # Compare predicted vs actual
            # Normalize card names (remove upgrade markers for comparison)
            def normalize_card(name):
                if not name:
                    return ""
                return name.replace("+", "").strip()

            predicted_set = set(normalize_card(c) for c in predicted_cards)
            actual_set = set(normalize_card(c) for c in actual_offered)

            # Count matching cards
            matching = predicted_set & actual_set

            # For accuracy: count how many of our predictions were correct
            if predicted_set:
                accuracy["cards"]["total"] += len(predicted_set)
                accuracy["cards"]["correct"] += len(matching)

            # Track mismatches
            if predicted_set != actual_set:
                missing = actual_set - predicted_set
                extra = predicted_set - actual_set
                if missing or extra:
                    accuracy["mismatches"].append({
                        "floor": floor_num,
                        "type": "card",
                        "predicted": list(predicted_set),
                        "actual": list(actual_set),
                        "missing": list(missing),  # In actual but not predicted
                        "extra": list(extra),  # In predicted but not actual
                    })

        except Exception as e:
            # Prediction failed - count as mismatch
            accuracy["mismatches"].append({
                "floor": floor_num,
                "type": "card",
                "error": str(e),
                "actual": actual_offered,
            })

        # Update owned relics (rough approximation)
        for rel in relics_obtained_list:
            if rel.get("floor", 0) <= floor_num and rel.get("key") not in owned_relics:
                owned_relics.append(rel.get("key", ""))

    # Process boss relic predictions
    for boss_rel in boss_relics_actual:
        floor_num = boss_rel.get("floor", 0)
        actual_picked = boss_rel.get("picked")
        actual_not_picked = boss_rel.get("not_picked", [])

        all_offered = []
        if actual_picked:
            all_offered.append(actual_picked)
        all_offered.extend(actual_not_picked)

        if not all_offered:
            continue

        # Determine which act this boss was for
        if floor_num <= 17:
            boss_act = 1
            relics_taken_before = 0
        elif floor_num <= 34:
            boss_act = 2
            relics_taken_before = 1
        elif floor_num <= 51:
            boss_act = 3
            relics_taken_before = 2
        else:
            continue

        # Try to predict boss relics
        try:
            has_starter = any(r in owned_relics for r in starter_relics)
            predicted_relics = predict_boss_relics(
                seed_long,
                player_class=player_class,
                has_starter_relic=has_starter,
                relics_already_taken=relics_taken_before,
                already_owned_relics=owned_relics,
            )

            predicted_set = set(predicted_relics) if predicted_relics else set()
            actual_set = set(all_offered)

            matching = predicted_set & actual_set

            if predicted_set:
                accuracy["relics"]["total"] += len(predicted_set)
                accuracy["relics"]["correct"] += len(matching)

            if predicted_set != actual_set:
                accuracy["mismatches"].append({
                    "floor": floor_num,
                    "type": "boss_relic",
                    "predicted": list(predicted_set),
                    "actual": list(actual_set),
                })

        except Exception as e:
            accuracy["mismatches"].append({
                "floor": floor_num,
                "type": "boss_relic",
                "error": str(e),
                "actual": all_offered,
            })

    # Calculate ratios
    for category in ["cards", "relics", "potions"]:
        if accuracy[category]["total"] > 0:
            accuracy[category]["ratio"] = round(
                accuracy[category]["correct"] / accuracy[category]["total"], 3
            )

    # Calculate overall
    total_correct = sum(accuracy[cat]["correct"] for cat in ["cards", "relics", "potions"])
    total_total = sum(accuracy[cat]["total"] for cat in ["cards", "relics", "potions"])

    accuracy["overall"]["correct"] = total_correct
    accuracy["overall"]["total"] = total_total
    if total_total > 0:
        accuracy["overall"]["ratio"] = round(total_correct / total_total, 3)

    # Limit mismatches to last 10 for display
    accuracy["mismatches"] = accuracy["mismatches"][-10:]

    return accuracy


def predict_neow_options(seed_long: int, player_class: str = "WATCHER") -> List[Dict[str, Any]]:
    """
    Predict Neow's blessing options for a seed.

    Neow offers 4 options:
    - Slot 1: Category 0 (card-focused bonuses)
    - Slot 2: Category 1 (small free bonuses)
    - Slot 3: Category 2 (strong bonus with drawback)
    - Slot 4: Boss relic swap (always available)

    Returns list of dicts with option details.
    """
    from core.state.rng import Random
    from core.generation.relics import predict_neow_boss_swap, predict_all_relic_pools

    neow_rng = Random(seed_long)

    # Category pools (exact order from NeowReward.java)
    cat0 = ["THREE_CARDS", "ONE_RANDOM_RARE_CARD", "REMOVE_CARD",
            "UPGRADE_CARD", "TRANSFORM_CARD", "RANDOM_COLORLESS"]
    cat1 = ["THREE_SMALL_POTIONS", "RANDOM_COMMON_RELIC", "TEN_PERCENT_HP_BONUS",
            "THREE_ENEMY_KILL", "HUNDRED_GOLD"]
    drawbacks = ["TEN_PERCENT_HP_LOSS", "NO_GOLD", "CURSE", "PERCENT_DAMAGE"]

    # Human-readable names and descriptions
    OPTION_NAMES = {
        # Category 0
        "THREE_CARDS": "Choose a Card",
        "ONE_RANDOM_RARE_CARD": "Random Rare Card",
        "REMOVE_CARD": "Remove a Card",
        "UPGRADE_CARD": "Upgrade a Card",
        "TRANSFORM_CARD": "Transform a Card",
        "RANDOM_COLORLESS": "Random Colorless",
        # Category 1
        "THREE_SMALL_POTIONS": "3 Potions",
        "RANDOM_COMMON_RELIC": "Random Common Relic",
        "TEN_PERCENT_HP_BONUS": "+10% Max HP",
        "THREE_ENEMY_KILL": "Enemies in first 3 combats have 1 HP",
        "HUNDRED_GOLD": "+100 Gold",
        # Category 2
        "RANDOM_COLORLESS_2": "2 Random Colorless Cards",
        "REMOVE_TWO": "Remove 2 Cards",
        "ONE_RARE_RELIC": "Random Rare Relic",
        "THREE_RARE_CARDS": "Choose a Rare Card",
        "TWO_FIFTY_GOLD": "+250 Gold",
        "TRANSFORM_TWO_CARDS": "Transform 2 Cards",
        "TWENTY_PERCENT_HP_BONUS": "+20% Max HP",
        # Special
        "BOSS_RELIC": "Boss Relic Swap",
    }

    DRAWBACK_NAMES = {
        "TEN_PERCENT_HP_LOSS": "Lose 10% Max HP",
        "NO_GOLD": "Lose all Gold",
        "CURSE": "Obtain a Curse",
        "PERCENT_DAMAGE": "Take damage",
    }

    options = []

    # Slot 1 (Category 0)
    roll0 = neow_rng.random(len(cat0) - 1)
    option0 = cat0[roll0]
    options.append({
        "slot": 1,
        "option": option0,
        "name": OPTION_NAMES.get(option0, option0),
        "drawback": None,
        "category": "blessing",
    })

    # Slot 2 (Category 1)
    roll1 = neow_rng.random(len(cat1) - 1)
    option1 = cat1[roll1]

    # Try to get relic name for RANDOM_COMMON_RELIC
    relic_detail = None
    if option1 == "RANDOM_COMMON_RELIC":
        try:
            pools = predict_all_relic_pools(seed_long, player_class)
            if pools.common:
                relic_detail = pools.common[0]
        except Exception:
            pass

    name1 = OPTION_NAMES.get(option1, option1)
    if relic_detail:
        name1 = f"{name1} ({relic_detail})"

    options.append({
        "slot": 2,
        "option": option1,
        "name": name1,
        "drawback": None,
        "category": "blessing",
    })

    # Slot 3 - Drawback first, then conditional pool
    drawback_roll = neow_rng.random(len(drawbacks) - 1)
    drawback = drawbacks[drawback_roll]

    # Build conditional pool based on drawback
    cat2 = ["RANDOM_COLORLESS_2"]
    if drawback != "CURSE":
        cat2.append("REMOVE_TWO")
    cat2.append("ONE_RARE_RELIC")
    cat2.append("THREE_RARE_CARDS")
    if drawback != "NO_GOLD":
        cat2.append("TWO_FIFTY_GOLD")
    cat2.append("TRANSFORM_TWO_CARDS")
    if drawback != "TEN_PERCENT_HP_LOSS":
        cat2.append("TWENTY_PERCENT_HP_BONUS")

    roll2 = neow_rng.random(len(cat2) - 1)
    option2 = cat2[roll2]

    # Try to get relic name for ONE_RARE_RELIC
    relic_detail2 = None
    if option2 == "ONE_RARE_RELIC":
        try:
            pools = predict_all_relic_pools(seed_long, player_class)
            if pools.rare:
                relic_detail2 = pools.rare[0]
        except Exception:
            pass

    name2 = OPTION_NAMES.get(option2, option2)
    if relic_detail2:
        name2 = f"{name2} ({relic_detail2})"

    options.append({
        "slot": 3,
        "option": option2,
        "name": name2,
        "drawback": DRAWBACK_NAMES.get(drawback, drawback),
        "drawback_id": drawback,
        "category": "drawback",
    })

    # Slot 4 (Boss Relic swap - always available)
    try:
        boss_relic = predict_neow_boss_swap(seed_long, player_class)
        boss_name = f"Trade starter relic for {boss_relic}"
    except Exception:
        boss_name = "Trade starter relic for random Boss Relic"

    options.append({
        "slot": 4,
        "option": "BOSS_RELIC",
        "name": boss_name,
        "drawback": "Lose starter relic",
        "category": "boss_swap",
    })

    return options


def process_save_data(save_data: Dict) -> Dict[str, Any]:
    """Process save data into dashboard-friendly format."""
    seed_long = save_data.get("seed", 0)
    seed_str = long_to_seed(seed_long)
    floor = save_data.get("floor_num", 0)
    act = save_data.get("act_num", 1)
    ascension = save_data.get("ascension_level", 0)

    # RNG counters from save
    card_counter = save_data.get("card_seed_count", 0)
    card_blizzard = save_data.get("card_random_seed_randomizer", 5)
    relic_counter = save_data.get("relic_seed_count", 0)
    potion_counter = save_data.get("potion_seed_count", 0)
    monster_counter = save_data.get("monster_seed_count", 0)
    event_counter = save_data.get("event_seed_count", 0)
    merchant_counter = save_data.get("merchant_seed_count", 0)
    treasure_counter = save_data.get("treasure_seed_count", 0)

    # Blizzard modifier for potion drops (affects drop chance)
    # Default to 0 if not present in save
    blizzard_mod = save_data.get("potion_chance", 0)

    # Player class detection
    player_class = save_data.get("class", "WATCHER").upper()
    if player_class == "THE_SILENT":
        player_class = "SILENT"

    # Player state
    current_hp = save_data.get("current_health", 0)
    max_hp = save_data.get("max_health", 0)
    gold = save_data.get("gold", 0)

    # Deck
    cards = save_data.get("cards", [])
    deck = []
    for card in cards:
        name = card.get("id", "")
        upgrades = card.get("upgrades", 0)
        deck.append({
            "id": name,
            "upgrades": upgrades,
            "display": f"{name}{'+'*upgrades}" if upgrades else name,
        })

    # Relics
    relics = save_data.get("relics", [])

    # Potions
    potions = save_data.get("potions", [])

    # Current room - check more specific names first
    current_room = save_data.get("current_room", "")
    room_symbol = "?"
    # Sort by length descending to match more specific names first
    sorted_rooms = sorted(ROOM_SYMBOLS.items(), key=lambda x: len(x[0]), reverse=True)
    for java_name, symbol in sorted_rooms:
        if java_name in current_room:
            room_symbol = symbol
            break

    # Path taken
    path_taken = [p for p in save_data.get("metric_path_per_floor", []) if p]

    # Current position on map
    # The game stores room_x and room_y for current position
    # path_x and path_y arrays track the complete history of x,y positions
    path_x = save_data.get("room_x", 0)
    path_y = save_data.get("room_y", -1)  # -1 if at Neow/floor 0
    path_x_history = save_data.get("path_x", [])
    path_y_history = save_data.get("path_y", [])

    # Generate map
    try:
        map_data = get_current_map(seed_long, act, ascension)
    except Exception as e:
        map_data = []

    # Find accessible nodes and predictions
    accessible_nodes = []
    node_predictions = []

    # Track relics obtained from chests by tier (for treasure prediction)
    # This is an approximation - ideally we'd track this more precisely
    relics_from_chests: Dict[str, int] = {"COMMON": 0, "UNCOMMON": 0, "RARE": 0}

    if map_data:
        accessible_nodes = find_accessible_nodes(map_data, path_x, path_y)

        # Boss relics already taken
        boss_relics_picked = save_data.get("metric_boss_relics", [])
        boss_relics_taken = len(boss_relics_picked)

        for node in accessible_nodes:
            pred = predict_node_rewards(
                seed_str, card_counter, card_blizzard, act, node,
                relics, seed_long, boss_relics_taken,
                potion_counter=potion_counter,
                blizzard_mod=blizzard_mod,
                merchant_counter=merchant_counter,
                treasure_counter=treasure_counter,
                relics_from_chests=relics_from_chests,
                player_class=player_class,
            )
            node_predictions.append(pred)

    # Boss relic prediction for current act
    boss_relics = None
    boss_relics_per_act = {}
    if act <= 3:
        try:
            has_starter = any(r in relics for r in CLASS_STARTER_RELICS.values())
            boss_relics_picked = save_data.get("metric_boss_relics", [])
            boss_relics_taken = len(boss_relics_picked)

            boss_relics = predict_boss_relics(
                seed_long,
                player_class=player_class,
                has_starter_relic=has_starter,
                relics_already_taken=boss_relics_taken,
                already_owned_relics=relics,
            )

            # Predict boss relics for all 3 acts
            # Note: this is an approximation - actual relics depend on what player picks
            # We show what WOULD be offered if player hasn't picked any yet
            for act_num in range(1, 4):
                act_relics_taken = max(0, act_num - 1)  # Acts already completed
                boss_relics_per_act[act_num] = predict_boss_relics(
                    seed_long,
                    player_class=player_class,
                    has_starter_relic=has_starter,
                    relics_already_taken=act_relics_taken,
                    already_owned_relics=[],  # Show original pool for future acts
                )
        except Exception:
            pass

    # Card choice history (full history, not limited)
    card_choices = save_data.get("metric_card_choices", [])

    # Floor-by-floor history data for timeline
    floor_history = []
    hp_per_floor = save_data.get("current_hp_per_floor", [])
    max_hp_per_floor = save_data.get("max_hp_per_floor", [])
    gold_per_floor = save_data.get("gold_per_floor", [])
    damage_taken_list = save_data.get("damage_taken", [])
    event_choices_list = save_data.get("event_choices", [])
    campfire_choices = save_data.get("campfire_choices", [])
    # Note: boss_relics in autosave is the POOL of boss relics (list of strings)
    # Boss relic CHOICES would be in a different format (list of dicts with floor/picked/not_picked)
    # Check if we have the choices format vs the pool format
    boss_relics_raw = save_data.get("boss_relics", [])
    if boss_relics_raw and isinstance(boss_relics_raw[0], dict):
        boss_relics_list = boss_relics_raw  # Choices format
    else:
        boss_relics_list = []  # Pool format - no choices to iterate
    potions_obtained = save_data.get("potions_obtained", [])
    relics_obtained = save_data.get("relics_obtained", [])
    items_purchased = save_data.get("items_purchased", [])
    items_purged = save_data.get("items_purged", [])

    # Get encounter lists for inferring what was encountered each floor
    monster_list = save_data.get("monster_list", [])
    elite_list = save_data.get("elite_monster_list", [])
    event_list = save_data.get("event_list", [])

    # Counters for tracking list consumption
    monsters_seen = 0
    elites_seen = 0
    events_seen = 0

    # Build floor-by-floor history
    for i, room_type in enumerate(path_taken):
        floor_num = i + 1
        floor_data = {
            "floor": floor_num,
            "room_type": room_type,
            "act": 1 if floor_num <= 17 else (2 if floor_num <= 34 else (3 if floor_num <= 51 else 4)),
        }

        # Infer encounter name from lists based on room type
        if room_type == 'M':
            if monsters_seen < len(monster_list):
                floor_data["encounter"] = monster_list[monsters_seen]
            monsters_seen += 1
        elif room_type == 'E':
            if elites_seen < len(elite_list):
                floor_data["encounter"] = elite_list[elites_seen]
            elites_seen += 1
        elif room_type == '?':
            if events_seen < len(event_list):
                floor_data["encounter"] = event_list[events_seen]
            events_seen += 1
        elif room_type == 'B':
            floor_data["encounter"] = save_data.get("boss", "Boss")

        # Map coordinates for this floor
        if i < len(path_x_history):
            floor_data["x"] = path_x_history[i]
        if i < len(path_y_history):
            floor_data["y"] = path_y_history[i]

        # HP data (index is floor-1 since it's 0-indexed)
        if i < len(hp_per_floor):
            floor_data["hp"] = hp_per_floor[i]
        if i < len(max_hp_per_floor):
            floor_data["max_hp"] = max_hp_per_floor[i]

        # Gold at this floor
        if i < len(gold_per_floor):
            floor_data["gold"] = gold_per_floor[i]

        # Find card choice for this floor
        for choice in card_choices:
            if choice.get("floor") == floor_num:
                floor_data["card_choice"] = {
                    "picked": choice.get("picked"),
                    "not_picked": choice.get("not_picked", []),
                }
                break

        # Find damage taken for this floor
        for dmg in damage_taken_list:
            if dmg.get("floor") == floor_num:
                floor_data["damage_taken"] = dmg.get("damage", 0)
                floor_data["enemies"] = dmg.get("enemies", [])
                floor_data["turns"] = dmg.get("turns", 0)
                break

        # Find event choice for this floor
        for event in event_choices_list:
            if event.get("floor") == floor_num:
                floor_data["event"] = {
                    "name": event.get("event_name"),
                    "choice": event.get("player_choice"),
                    "damage": event.get("damage_taken", 0),
                    "gold_change": event.get("gold_change", 0),
                    "cards": event.get("cards_obtained", []),
                    "relics": event.get("relics_obtained", []),
                }
                break

        # Find campfire choice for this floor
        for camp in campfire_choices:
            if camp.get("floor") == floor_num:
                floor_data["campfire"] = {
                    "key": camp.get("key"),
                    "data": camp.get("data"),
                }
                break

        # Find boss relic for this floor
        for boss_rel in boss_relics_list:
            if boss_rel.get("floor") == floor_num:
                floor_data["boss_relic"] = {
                    "picked": boss_rel.get("picked"),
                    "not_picked": boss_rel.get("not_picked", []),
                }
                break

        floor_history.append(floor_data)

    # Boss card reward prediction (uses "elite" room type for boss fights)
    boss_card_reward = None
    try:
        boss_cards, boss_new_counter = predict_card_reward(
            seed_str, card_counter, act, "elite",
            card_blizzard=card_blizzard,
            relics=relics,
        )
        boss_card_reward = {
            "cards": boss_cards,
            "counter_before": card_counter,
            "counter_after": boss_new_counter,
        }
    except Exception as e:
        boss_card_reward = {"error": str(e)}

    # Predict bosses for all acts
    predicted_bosses = {}
    try:
        all_bosses = predict_all_bosses_extended(seed_long, ascension=ascension)
        # Convert to a simple dict with act -> list of boss names
        for act_num, boss_list in all_bosses.items():
            predicted_bosses[act_num] = boss_list
    except Exception as e:
        # Fallback: use current boss from save
        predicted_bosses = {
            1: ["Unknown"],
            2: ["Unknown"],
            3: ["Unknown"],
        }
        if act <= 3:
            current_boss = save_data.get("boss", "Unknown Boss")
            predicted_bosses[act] = [current_boss]

    # Predict future acts encounters (Act 2 and Act 3 if not there yet)
    future_acts = {}
    try:
        all_act_predictions = predict_all_acts(seed_str)
        for act_key, act_data in all_act_predictions.items():
            act_num = int(act_key.replace("act", ""))
            if act_num > act:  # Only include future acts
                future_acts[act_num] = {
                    "monsters": act_data["monsters"][:5],  # First 5 encounters
                    "elites": act_data["elites"][:5],  # First 5 elite encounters
                    "boss": act_data["boss"],
                }
    except Exception as e:
        future_acts = {"error": str(e)}

    # Generate shop prediction if a shop node is accessible
    shop_prediction = None
    for node in accessible_nodes:
        if node.get("room_type") == "$":
            # Already computed in node_predictions
            for pred in node_predictions:
                if pred["node"].get("room_type") == "$" and pred.get("shop_inventory"):
                    shop_prediction = pred["shop_inventory"]
                    break
            break

    # Generate treasure prediction if a treasure node is accessible
    treasure_prediction = None
    for node in accessible_nodes:
        if node.get("room_type") == "T":
            # Already computed in node_predictions
            for pred in node_predictions:
                if pred["node"].get("room_type") == "T" and pred.get("treasure"):
                    treasure_prediction = pred["treasure"]
                    break
            break

    # Build path tree for next 3 floors
    path_tree = {}
    if map_data:
        try:
            # Get monster/elite/event lists
            monsters = save_data.get("monster_list", [])
            elites = save_data.get("elite_monster_list", [])
            events = save_data.get("event_list", [])

            # Count how many of each we've used
            monsters_used = sum(1 for p in path_taken if p == 'M')
            elites_used = sum(1 for p in path_taken if p == 'E')
            events_used = sum(1 for p in path_taken if p == '?')

            # Build the raw path tree
            raw_tree = build_path_tree(map_data, path_x, path_y, depth=3)

            # Enrich with predictions and enemy names
            boss_relics_picked = save_data.get("metric_boss_relics", [])
            boss_relics_taken_count = len(boss_relics_picked)

            path_tree = enrich_path_tree_with_predictions(
                raw_tree,
                seed_str=seed_str,
                card_counter=card_counter,
                card_blizzard=card_blizzard,
                act=act,
                owned_relics=relics,
                seed_long=seed_long,
                boss_relics_taken=boss_relics_taken_count,
                potion_counter=potion_counter,
                blizzard_mod=blizzard_mod,
                merchant_counter=merchant_counter,
                treasure_counter=treasure_counter,
                player_class=player_class,
                monsters=monsters,
                elites=elites,
                events=events,
                monsters_used=monsters_used,
                elites_used=elites_used,
                events_used=events_used,
            )
        except Exception as e:
            path_tree = {"error": str(e)}

    # Calculate RNG prediction accuracy
    rng_accuracy = None
    try:
        rng_accuracy = calculate_rng_accuracy(save_data, seed_str, player_class)
    except Exception as e:
        rng_accuracy = {"error": str(e), "overall": {"correct": 0, "total": 0, "ratio": 0.0}}

    # Predict Neow options if at floor 0
    neow_options = []
    if floor == 0:
        try:
            neow_options = predict_neow_options(seed_long, player_class)
        except Exception as e:
            neow_options = [{"error": str(e)}]

    return {
        "seed": {
            "string": seed_str,
            "long": seed_long,
        },
        "run": {
            "floor": floor,
            "act": act,
            "ascension": ascension,
            "room_type": room_symbol,
            "current_room": current_room,
        },
        "player": {
            "hp": current_hp,
            "max_hp": max_hp,
            "gold": gold,
        },
        "rng": {
            "card_counter": card_counter,
            "card_blizzard": card_blizzard,
            "relic_counter": relic_counter,
            "potion_counter": potion_counter,
            "monster_counter": monster_counter,
            "event_counter": event_counter,
            "merchant_counter": merchant_counter,
            "treasure_counter": treasure_counter,
            "blizzard_mod": blizzard_mod,
        },
        "player_class": player_class,
        "deck": deck,
        "deck_count": len(deck),
        "relics": relics,
        "potions": potions,
        "path_taken": path_taken,
        "current_position": {
            "x": path_x,
            "y": path_y,
        },
        "map": map_data,
        "accessible_nodes": accessible_nodes,
        "node_predictions": node_predictions,
        "boss_relics": boss_relics,
        "boss_relics_per_act": boss_relics_per_act,
        "boss_card_reward": boss_card_reward,
        "card_choices": card_choices[-5:],  # Last 5 for display
        "floor_history": floor_history,  # Full floor-by-floor history
        "monster_list": save_data.get("monster_list", []),
        "elite_monster_list": save_data.get("elite_monster_list", []),
        "event_list": save_data.get("event_list", []),
        "boss": save_data.get("boss", "Unknown Boss"),
        "predicted_bosses": predicted_bosses,
        "future_acts": future_acts,
        "shop_prediction": shop_prediction,
        "treasure_prediction": treasure_prediction,
        "path_tree": path_tree,
        "rng_accuracy": rng_accuracy,
        "neow_options": neow_options,
        "timestamp": time.time(),
    }


# ============================================================================
# API ENDPOINTS
# ============================================================================

@app.get("/", response_class=HTMLResponse)
async def index():
    """Serve the main dashboard page."""
    html_path = Path(__file__).parent / "index.html"
    if html_path.exists():
        return html_path.read_text()
    return "<h1>Dashboard not found</h1>"


@app.get("/api/state")
async def get_state():
    """Get current game state as JSON."""
    try:
        save_data = read_save_file()
        state = process_save_data(save_data)
        return JSONResponse(state)
    except FileNotFoundError:
        return JSONResponse({
            "error": "No save file found",
            "path": SAVE_PATH,
        }, status_code=404)
    except Exception as e:
        return JSONResponse({
            "error": str(e),
        }, status_code=500)


@app.get("/api/stream")
async def stream_state(request: Request):
    """Server-Sent Events stream for live updates."""
    async def event_generator():
        last_mtime = 0
        last_state = None
        first_message = True

        while True:
            if await request.is_disconnected():
                break

            try:
                mtime = os.path.getmtime(SAVE_PATH)

                # Send on first connection OR when file changes
                if first_message or mtime != last_mtime:
                    first_message = False
                    last_mtime = mtime
                    save_data = read_save_file()
                    state = process_save_data(save_data)

                    # Only send if state changed (skip duplicate on first load)
                    state_json = json.dumps(state)
                    if state_json != last_state:
                        last_state = state_json
                        yield f"data: {state_json}\n\n"

            except FileNotFoundError:
                if first_message or last_state != '{"error": "No save file"}':
                    last_state = '{"error": "No save file"}'
                    yield f"data: {last_state}\n\n"
                first_message = False
            except Exception as e:
                error_json = json.dumps({'error': str(e)})
                if first_message or last_state != error_json:
                    last_state = error_json
                    yield f"data: {error_json}\n\n"
                first_message = False

            await asyncio.sleep(2)  # Poll every 2 seconds

    return StreamingResponse(
        event_generator(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
        }
    )


@app.get("/api/predict/cards")
async def predict_cards(
    seed: str,
    counter: int = 0,
    blizzard: int = 5,
    act: int = 1,
    room_type: str = "normal",
):
    """Predict card rewards for given RNG state."""
    try:
        cards, new_counter = predict_card_reward(
            seed, counter, act, room_type,
            card_blizzard=blizzard
        )
        return JSONResponse({
            "cards": cards,
            "counter_before": counter,
            "counter_after": new_counter,
        })
    except Exception as e:
        return JSONResponse({"error": str(e)}, status_code=500)


@app.get("/api/predict/boss-relics")
async def predict_boss(
    seed: int,
    relics_taken: int = 0,
    owned: str = "",
):
    """Predict boss relic offerings."""
    try:
        owned_relics = [r.strip() for r in owned.split(",") if r.strip()]
        has_starter = any(r in owned_relics for r in CLASS_STARTER_RELICS.values())

        boss_relics = predict_boss_relics(
            seed,
            player_class="WATCHER",
            has_starter_relic=has_starter,
            relics_already_taken=relics_taken,
            already_owned_relics=owned_relics,
        )
        return JSONResponse({"boss_relics": boss_relics})
    except Exception as e:
        return JSONResponse({"error": str(e)}, status_code=500)


@app.post("/api/predict/path")
async def predict_path(request: Request):
    """Predict cumulative rewards for a planned path through the map.

    Request body:
    {
        "path": [{"x": 0, "y": 0}, {"x": 1, "y": 1}, ...]
    }

    Returns cumulative predictions for all nodes in the path.
    """
    try:
        body = await request.json()
        path_nodes = body.get("path", [])

        if not path_nodes:
            return JSONResponse({"error": "No path provided"}, status_code=400)

        # Read current game state
        save_data = read_save_file()
        seed_long = save_data.get("seed", 0)
        seed_str = long_to_seed(seed_long)
        act = save_data.get("act_num", 1)
        card_counter = save_data.get("card_seed_count", 0)
        card_blizzard = save_data.get("card_random_seed_randomizer", 5)
        relics = save_data.get("relics", [])
        path_taken = save_data.get("metric_path_per_floor", [])

        # Track counters for sequential room types
        monsters_used = sum(1 for p in path_taken if p == 'M')
        elites_used = sum(1 for p in path_taken if p == 'E')
        events_used = sum(1 for p in path_taken if p == '?')

        # Generate map to get room types
        try:
            map_data = get_current_map(seed_long, act, save_data.get("ascension_level", 0))
        except Exception:
            map_data = []

        # Collect predictions for each node in path
        all_cards = []
        all_relics = []
        all_potions = []
        current_counter = card_counter

        for path_node in path_nodes:
            x = path_node.get("x")
            y = path_node.get("y")

            # Find node in map
            node = find_node_at(map_data, x, y)
            if not node:
                continue

            room_type = node.get("room_type")

            # Predict based on room type
            if room_type in ["M", "E"]:
                # Combat - predict card reward
                rt = "elite" if room_type == "E" else "normal"
                try:
                    cards, new_counter = predict_card_reward(
                        seed_str, current_counter, act, rt,
                        card_blizzard=card_blizzard,
                        relics=relics,
                    )
                    all_cards.extend(cards)
                    current_counter = new_counter
                except Exception:
                    pass

                # Note: potion prediction would go here
                all_potions.append(f"~40% from {room_type}")

                # Elite relic chance
                if room_type == "E":
                    all_relics.append("~25% from Elite")

            elif room_type == "T":
                # Treasure - relic
                all_relics.append("Chest relic")

            elif room_type == "$":
                # Shop - might buy relics
                all_relics.append("Shop relics available")

        return JSONResponse({
            "cards": all_cards,
            "relics": all_relics,
            "potions": list(set(all_potions)),  # Dedupe
            "card_counter_final": current_counter,
        })

    except FileNotFoundError:
        return JSONResponse({
            "error": "No save file found",
            "cards": [],
            "relics": [],
            "potions": [],
        }, status_code=404)
    except Exception as e:
        return JSONResponse({"error": str(e)}, status_code=500)


# ============================================================================
# MAIN
# ============================================================================

if __name__ == "__main__":
    print(f"""
    ========================================
    Slay the Spire Live Dashboard
    ========================================

    Dashboard URL: http://localhost:8080

    Watching save file:
    {SAVE_PATH}

    Press Ctrl+C to stop.
    ========================================
    """)

    uvicorn.run(
        app,
        host="0.0.0.0",
        port=8080,
        log_level="info",
    )
