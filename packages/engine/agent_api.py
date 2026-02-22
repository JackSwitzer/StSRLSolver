"""
Agent API - JSON-serializable action and observation interfaces for RL agents.

This module provides the model-facing API surface for agents to interact with
the game engine. All actions and observations are JSON-serializable dicts.

Key types:
- ActionDict: JSON-serializable action with id, type, label, params, phase
- ActionResult: Result of executing an action
- ObservationDict: Complete observable game state

Usage:
    runner = GameRunner(seed="TEST", ascension=20)

    # Get current observation
    obs = runner.get_observation()

    # Get available actions as dicts
    actions = runner.get_available_action_dicts()

    # Execute action dict
    result = runner.take_action_dict(actions[0])
"""

from __future__ import annotations

from dataclasses import dataclass, asdict
from typing import List, Dict, Any, Optional, Union, TypedDict
from enum import Enum

from .state.run import RunState, CardInstance, RelicInstance, PotionSlot
from .state.combat import CombatState, EnemyCombatState
from .generation.map import MapRoomNode, RoomType


# =============================================================================
# Type Definitions
# =============================================================================

class ActionDict(TypedDict, total=False):
    """JSON-serializable action dict."""
    id: str  # Stable identifier for the action
    type: str  # Action type enum string
    label: str  # Human-readable summary
    params: Dict[str, Any]  # Required parameters
    requires: List[str]  # Optional hints for missing params
    phase: str  # Current phase


class ActionResult(TypedDict, total=False):
    """Result of executing an action."""
    success: bool
    error: Optional[str]
    # Action-specific result fields
    data: Dict[str, Any]


class ObservationDict(TypedDict, total=False):
    """Complete observable game state."""
    phase: str
    run: Dict[str, Any]
    map: Dict[str, Any]
    combat: Optional[Dict[str, Any]]
    event: Optional[Dict[str, Any]]
    reward: Optional[Dict[str, Any]]
    shop: Optional[Dict[str, Any]]
    rest: Optional[Dict[str, Any]]
    treasure: Optional[Dict[str, Any]]


# =============================================================================
# Phase Names (for observation)
# =============================================================================

PHASE_NAMES = {
    "NEOW": "neow",
    "MAP_NAVIGATION": "map",
    "COMBAT": "combat",
    "COMBAT_REWARDS": "reward",
    "EVENT": "event",
    "SHOP": "shop",
    "REST": "rest",
    "TREASURE": "treasure",
    "BOSS_REWARDS": "boss_reward",
    "RUN_COMPLETE": "run_complete",
}


# =============================================================================
# Action ID Generation
# =============================================================================

def generate_action_id(action_type: str, *args) -> str:
    """
    Generate a deterministic action ID from type and parameters.

    IDs are stable for identical state + phase.
    """
    parts = [action_type]
    for arg in args:
        if arg is not None and arg != -1:
            parts.append(str(arg))
    return "_".join(parts)


# =============================================================================
# Action Dict Generators by Phase
# =============================================================================

def generate_path_actions(runner) -> List[ActionDict]:
    """Generate path_choice actions for map navigation."""
    paths = runner.run_state.get_available_paths()
    actions = []

    for i, node in enumerate(paths):
        room_name = node.room_type.name if hasattr(node.room_type, 'name') else str(node.room_type)
        actions.append({
            "id": generate_action_id("path_choice", i),
            "type": "path_choice",
            "label": f"Path to {room_name} at ({node.x}, {node.y})",
            "params": {"node_index": i},
            "phase": "map",
        })

    return actions


def generate_neow_actions(runner) -> List[ActionDict]:
    """Generate neow_choice actions."""
    if runner.neow_blessings is None:
        # Generate blessings if not already generated
        from .handlers.rooms import NeowHandler
        is_first_run = not hasattr(runner.run_state, 'previous_score') or runner.run_state.previous_score == 0
        previous_score = getattr(runner.run_state, 'previous_score', 0)
        runner.neow_blessings = NeowHandler.get_blessing_options(
            runner.neow_rng,
            previous_score=previous_score,
            is_first_run=is_first_run,
        )

    actions = []
    for i, blessing in enumerate(runner.neow_blessings):
        actions.append({
            "id": generate_action_id("neow_choice", i),
            "type": "neow_choice",
            "label": blessing.description,
            "params": {"choice_index": i},
            "phase": "neow",
        })

    return actions


def generate_combat_actions(runner) -> List[ActionDict]:
    """Generate combat actions from CombatEngine."""
    actions = []

    if runner.current_combat is None:
        # Fallback: only end turn available
        actions.append({
            "id": "end_turn",
            "type": "end_turn",
            "label": "End turn",
            "params": {},
            "phase": "combat",
        })
        return actions

    engine_actions = runner.current_combat.get_legal_actions()
    combat_state = runner.current_combat.state

    for action in engine_actions:
        from .state.combat import PlayCard, UsePotion, EndTurn

        if isinstance(action, PlayCard):
            card_id = combat_state.hand[action.card_idx] if action.card_idx < len(combat_state.hand) else "unknown"
            target_name = ""
            if action.target_idx >= 0 and action.target_idx < len(combat_state.enemies):
                target_name = f" -> {combat_state.enemies[action.target_idx].id}"

            params = {"card_index": action.card_idx}
            if action.target_idx >= 0:
                params["target_index"] = action.target_idx

            actions.append({
                "id": generate_action_id("play_card", action.card_idx, action.target_idx),
                "type": "play_card",
                "label": f"{card_id}{target_name}",
                "params": params,
                "phase": "combat",
            })

        elif isinstance(action, UsePotion):
            potion_id = combat_state.potions[action.potion_idx] if action.potion_idx < len(combat_state.potions) else "unknown"
            target_name = ""
            if action.target_idx >= 0 and action.target_idx < len(combat_state.enemies):
                target_name = f" -> {combat_state.enemies[action.target_idx].id}"

            params = {"potion_slot": action.potion_idx}
            if action.target_idx >= 0:
                params["target_index"] = action.target_idx

            actions.append({
                "id": generate_action_id("use_potion", action.potion_idx, action.target_idx),
                "type": "use_potion",
                "label": f"{potion_id}{target_name}",
                "params": params,
                "phase": "combat",
            })

        elif isinstance(action, EndTurn):
            actions.append({
                "id": "end_turn",
                "type": "end_turn",
                "label": "End turn",
                "params": {},
                "phase": "combat",
            })

    return actions


def generate_reward_actions(runner) -> List[ActionDict]:
    """Generate reward actions for combat rewards phase."""
    actions = []
    rewards = runner.current_rewards

    if rewards is None:
        actions.append({
            "id": "proceed_from_rewards",
            "type": "proceed_from_rewards",
            "label": "Proceed",
            "params": {},
            "phase": "reward",
        })
        return actions

    # Gold (auto-claimed but include for completeness)
    if rewards.gold and not rewards.gold.claimed:
        actions.append({
            "id": "claim_gold",
            "type": "claim_gold",
            "label": f"Claim {rewards.gold.amount} gold",
            "params": {},
            "phase": "reward",
        })

    # Potion rewards
    if rewards.potion and not rewards.potion.claimed and not rewards.potion.skipped:
        if runner.run_state.count_empty_potion_slots() > 0:
            actions.append({
                "id": "claim_potion",
                "type": "claim_potion",
                "label": f"Claim {rewards.potion.potion.name}",
                "params": {},
                "phase": "reward",
            })
        actions.append({
            "id": "skip_potion",
            "type": "skip_potion",
            "label": "Skip potion",
            "params": {},
            "phase": "reward",
        })

    # Card rewards
    for i, card_reward in enumerate(rewards.card_rewards):
        if not card_reward.is_resolved:
            # Pick card actions
            for j, card in enumerate(card_reward.cards):
                actions.append({
                    "id": generate_action_id("pick_card", i, j),
                    "type": "pick_card",
                    "label": f"Pick {card.name}",
                    "params": {"card_reward_index": i, "card_index": j},
                    "phase": "reward",
                })

            # Skip card action
            actions.append({
                "id": generate_action_id("skip_card", i),
                "type": "skip_card",
                "label": f"Skip card reward {i}",
                "params": {"card_reward_index": i},
                "phase": "reward",
            })

            # Singing Bowl option
            if runner.run_state.has_relic("Singing Bowl"):
                actions.append({
                    "id": generate_action_id("singing_bowl", i),
                    "type": "singing_bowl",
                    "label": "Singing Bowl (+2 Max HP)",
                    "params": {"card_reward_index": i},
                    "phase": "reward",
                })

    # Relic reward (elite only)
    if rewards.relic and not rewards.relic.claimed:
        actions.append({
            "id": "claim_relic",
            "type": "claim_relic",
            "label": f"Claim {rewards.relic.relic.name}",
            "params": {},
            "phase": "reward",
        })

    # Emerald key (burning elite)
    if rewards.emerald_key and not rewards.emerald_key.claimed:
        actions.append({
            "id": "claim_emerald_key",
            "type": "claim_emerald_key",
            "label": "Claim Emerald Key",
            "params": {},
            "phase": "reward",
        })
        actions.append({
            "id": "skip_emerald_key",
            "type": "skip_emerald_key",
            "label": "Skip Emerald Key",
            "params": {},
            "phase": "reward",
        })

    # Proceed if mandatory rewards resolved
    if _mandatory_rewards_resolved(rewards):
        actions.append({
            "id": "proceed_from_rewards",
            "type": "proceed_from_rewards",
            "label": "Proceed",
            "params": {},
            "phase": "reward",
        })

    return actions


def _mandatory_rewards_resolved(rewards) -> bool:
    """Check if mandatory rewards have been resolved."""
    for card_reward in rewards.card_rewards:
        if not card_reward.is_resolved:
            return False
    if rewards.relic and not rewards.relic.claimed:
        return False
    return True


def generate_event_actions(runner) -> List[ActionDict]:
    """Generate event_choice actions."""
    actions = []

    if runner.current_event_state is None:
        actions.append({
            "id": "event_choice_0",
            "type": "event_choice",
            "label": "Leave",
            "params": {"choice_index": 0},
            "phase": "event",
        })
        return actions

    choices = runner.event_handler.get_available_choices(
        runner.current_event_state,
        runner.run_state
    )

    for choice in choices:
        actions.append({
            "id": generate_action_id("event_choice", choice.index),
            "type": "event_choice",
            "label": choice.text,
            "params": {"choice_index": choice.index},
            "phase": "event",
        })

    return actions


def generate_shop_actions(runner) -> List[ActionDict]:
    """Generate shop actions."""
    actions = []

    # Leave shop is always available
    actions.append({
        "id": "leave_shop",
        "type": "leave_shop",
        "label": "Leave shop",
        "params": {},
        "phase": "shop",
    })

    if runner.current_shop is None:
        return actions

    gold = runner.run_state.gold

    # Colored cards
    for shop_card in runner.current_shop.get_available_colored_cards():
        if shop_card.price <= gold:
            actions.append({
                "id": generate_action_id("buy_card", "colored", shop_card.slot_index),
                "type": "buy_card",
                "label": f"Buy {shop_card.card.name} ({shop_card.price}g)",
                "params": {"item_index": shop_card.slot_index, "card_pool": "colored"},
                "phase": "shop",
            })

    # Colorless cards
    for shop_card in runner.current_shop.get_available_colorless_cards():
        if shop_card.price <= gold:
            actions.append({
                "id": generate_action_id("buy_card", "colorless", shop_card.slot_index),
                "type": "buy_card",
                "label": f"Buy {shop_card.card.name} ({shop_card.price}g)",
                "params": {"item_index": shop_card.slot_index, "card_pool": "colorless"},
                "phase": "shop",
            })

    # Relics
    for shop_relic in runner.current_shop.get_available_relics():
        if shop_relic.price <= gold:
            actions.append({
                "id": generate_action_id("buy_relic", shop_relic.slot_index),
                "type": "buy_relic",
                "label": f"Buy {shop_relic.relic.name} ({shop_relic.price}g)",
                "params": {"item_index": shop_relic.slot_index},
                "phase": "shop",
            })

    # Potions
    if runner.run_state.count_empty_potion_slots() > 0:
        for shop_potion in runner.current_shop.get_available_potions():
            if shop_potion.price <= gold:
                actions.append({
                    "id": generate_action_id("buy_potion", shop_potion.slot_index),
                    "type": "buy_potion",
                    "label": f"Buy {shop_potion.potion.name} ({shop_potion.price}g)",
                    "params": {"item_index": shop_potion.slot_index},
                    "phase": "shop",
                })

    # Card removal
    if runner.current_shop.purge_available and runner.current_shop.purge_cost <= gold:
        removable = runner.run_state.get_removable_cards()
        for card_idx, card in removable:
            actions.append({
                "id": generate_action_id("remove_card", card_idx),
                "type": "remove_card",
                "label": f"Remove {card.id} ({runner.current_shop.purge_cost}g)",
                "params": {"card_index": card_idx},
                "phase": "shop",
            })

    return actions


def generate_rest_actions(runner) -> List[ActionDict]:
    """Generate rest site actions."""
    actions = []

    # Rest (heal)
    if not runner.run_state.has_relic("Coffee Dripper"):
        if runner.run_state.current_hp < runner.run_state.max_hp:
            actions.append({
                "id": "rest",
                "type": "rest",
                "label": "Rest (heal 30%)",
                "params": {},
                "phase": "rest",
            })

    # Smith (upgrade)
    upgradeable = runner.run_state.get_upgradeable_cards()
    for idx, card in upgradeable:
        actions.append({
            "id": generate_action_id("smith", idx),
            "type": "smith",
            "label": f"Smith {card.id}",
            "params": {"card_index": idx},
            "phase": "rest",
        })

    # Dig (Shovel relic)
    if runner.run_state.has_relic("Shovel"):
        actions.append({
            "id": "dig",
            "type": "dig",
            "label": "Dig (Shovel)",
            "params": {},
            "phase": "rest",
        })

    # Lift (Girya relic)
    if runner.run_state.has_relic("Girya"):
        counter = runner.run_state.get_relic_counter("Girya")
        if counter < 3:
            actions.append({
                "id": "lift",
                "type": "lift",
                "label": "Lift (Girya)",
                "params": {},
                "phase": "rest",
            })

    # Toke (Peace Pipe relic)
    if runner.run_state.has_relic("Peace Pipe"):
        removable = runner.run_state.get_removable_cards()
        for idx, card in removable:
            actions.append({
                "id": generate_action_id("toke", idx),
                "type": "toke",
                "label": f"Toke {card.id} (Peace Pipe)",
                "params": {"card_index": idx},
                "phase": "rest",
            })

    # Recall (placeholder for future)
    # Ruby key
    if runner.run_state.act == 3 and not runner.run_state.has_ruby_key:
        actions.append({
            "id": "recall",
            "type": "recall",
            "label": "Recall (Ruby Key)",
            "params": {},
            "phase": "rest",
        })

    return actions


def generate_treasure_actions(runner) -> List[ActionDict]:
    """Generate treasure room actions."""
    actions = []

    actions.append({
        "id": "take_relic",
        "type": "take_relic",
        "label": "Take relic",
        "params": {},
        "phase": "treasure",
    })

    # Sapphire key option
    if runner.run_state.act == 3 and not runner.run_state.has_sapphire_key:
        actions.append({
            "id": "sapphire_key",
            "type": "sapphire_key",
            "label": "Take Sapphire Key (skip relic)",
            "params": {},
            "phase": "treasure",
        })

    return actions


def generate_boss_reward_actions(runner) -> List[ActionDict]:
    """Generate boss relic choice actions."""
    actions = []

    if runner.current_rewards and runner.current_rewards.boss_relics:
        boss_relics = runner.current_rewards.boss_relics
        if not boss_relics.is_resolved:
            for i, relic in enumerate(boss_relics.relics):
                actions.append({
                    "id": generate_action_id("pick_boss_relic", i),
                    "type": "pick_boss_relic",
                    "label": f"Pick {relic.name}",
                    "params": {"relic_index": i},
                    "phase": "boss_reward",
                })

            # Skip option
            actions.append({
                "id": "skip_boss_relic",
                "type": "skip_boss_relic",
                "label": "Skip boss relic",
                "params": {},
                "phase": "boss_reward",
            })
        else:
            actions.append({
                "id": "proceed_from_rewards",
                "type": "proceed_from_rewards",
                "label": "Proceed",
                "params": {},
                "phase": "boss_reward",
            })
    else:
        # Fallback
        for i in range(3):
            actions.append({
                "id": generate_action_id("pick_boss_relic", i),
                "type": "pick_boss_relic",
                "label": f"Pick boss relic {i}",
                "params": {"relic_index": i},
                "phase": "boss_reward",
            })

    return actions


# =============================================================================
# Observation Generators
# =============================================================================

def generate_run_observation(runner) -> Dict[str, Any]:
    """Generate the run section of the observation."""
    rs = runner.run_state

    return {
        "seed": rs.seed_string,
        "ascension": rs.ascension,
        "act": rs.act,
        "floor": rs.floor,
        "gold": rs.gold,
        "current_hp": rs.current_hp,
        "max_hp": rs.max_hp,
        "deck": [
            {
                "id": card.id,
                "upgraded": card.upgraded,
                "misc_value": card.misc_value,
            }
            for card in rs.deck
        ],
        "relics": [
            {
                "id": relic.id,
                "counter": relic.counter,
                "triggered_this_combat": relic.triggered_this_combat,
                "triggered_this_turn": relic.triggered_this_turn,
            }
            for relic in rs.relics
        ],
        "potions": [
            slot.potion_id if not slot.is_empty() else None
            for slot in rs.potion_slots
        ],
        "keys": {
            "ruby": rs.has_ruby_key,
            "emerald": rs.has_emerald_key,
            "sapphire": rs.has_sapphire_key,
        },
        "map_position": {
            "x": rs.map_position.x,
            "y": rs.map_position.y,
        },
    }


def generate_map_observation(runner) -> Dict[str, Any]:
    """Generate the map section of the observation."""
    rs = runner.run_state
    current_map = rs.get_current_map()

    if not current_map:
        return {
            "act": rs.act,
            "nodes": [],
            "edges": [],
            "available_paths": [],
            "visited_nodes": [{"act": v[0], "x": v[1], "y": v[2]} for v in rs.visited_nodes],
        }

    nodes = []
    edges = []

    for y, row in enumerate(current_map):
        for x, node in enumerate(row):
            # Skip nodes that have no room type or no edges (empty slots)
            if node.room_type is None or not node.has_edges():
                continue

            nodes.append({
                "x": node.x,
                "y": node.y,
                "room_type": node.room_type.name,
                "has_emerald_key": getattr(node, 'has_emerald_key', False),
            })

            for edge in node.edges:
                edges.append({
                    "src_x": node.x,
                    "src_y": node.y,
                    "dst_x": edge.dst_x,
                    "dst_y": edge.dst_y,
                    "is_boss": edge.is_boss,
                })

    # Available paths
    available_paths = []
    for i, path_node in enumerate(rs.get_available_paths()):
        available_paths.append({
            "index": i,
            "x": path_node.x,
            "y": path_node.y,
            "room_type": path_node.room_type.name,
        })

    return {
        "act": rs.act,
        "nodes": nodes,
        "edges": edges,
        "available_paths": available_paths,
        "visited_nodes": [{"act": v[0], "x": v[1], "y": v[2]} for v in rs.visited_nodes],
    }


def generate_combat_observation(runner) -> Optional[Dict[str, Any]]:
    """Generate the combat section of the observation."""
    if runner.current_combat is None:
        return None

    state = runner.current_combat.state

    return {
        "player": {
            "hp": state.player.hp,
            "max_hp": state.player.max_hp,
            "block": state.player.block,
            "statuses": dict(state.player.statuses),
        },
        "energy": state.energy,
        "max_energy": state.max_energy,
        "stance": state.stance,
        "mantra": state.mantra,
        "hand": list(state.hand),
        "draw_pile": list(state.draw_pile),
        "discard_pile": list(state.discard_pile),
        "exhaust_pile": list(state.exhaust_pile),
        "enemies": [
            {
                "id": enemy.id,
                "name": enemy.name,
                "hp": enemy.hp,
                "max_hp": enemy.max_hp,
                "block": enemy.block,
                "statuses": dict(enemy.statuses),
                "move_id": enemy.move_id,
                "move_damage": enemy.move_damage,
                "move_hits": enemy.move_hits,
                "move_block": enemy.move_block,
                "move_effects": dict(enemy.move_effects),
            }
            for enemy in state.enemies
        ],
        "turn": state.turn,
        "cards_played_this_turn": state.cards_played_this_turn,
        "attacks_played_this_turn": state.attacks_played_this_turn,
        "skills_played_this_turn": state.skills_played_this_turn,
        "powers_played_this_turn": state.powers_played_this_turn,
        "relic_counters": dict(state.relic_counters),
        "card_costs": dict(state.card_costs),
    }


def generate_event_observation(runner) -> Optional[Dict[str, Any]]:
    """Generate the event section of the observation."""
    if runner.current_event_state is None:
        return None

    event_state = runner.current_event_state
    choices = runner.event_handler.get_available_choices(
        event_state,
        runner.run_state
    )

    return {
        "event_id": event_state.event_id,
        "phase": event_state.phase.name if hasattr(event_state.phase, 'name') else str(event_state.phase),
        "attempt_count": getattr(event_state, 'attempt_count', 0),
        "hp_cost_modifier": getattr(event_state, 'hp_cost_modifier', 1.0),
        "choices": [
            {
                "choice_index": choice.index,
                "label": choice.text,
                "requires_card_selection": getattr(choice, 'requires_card_selection', False),
                "card_selection_type": getattr(choice, 'card_selection_type', None),
                "card_selection_count": getattr(choice, 'card_selection_count', 0),
            }
            for choice in choices
        ],
    }


def generate_reward_observation(runner) -> Optional[Dict[str, Any]]:
    """Generate the reward section of the observation."""
    rewards = runner.current_rewards

    if rewards is None:
        return None

    obs = {}

    if rewards.gold:
        obs["gold"] = {
            "amount": rewards.gold.amount,
            "claimed": rewards.gold.claimed,
        }

    if rewards.potion:
        obs["potion"] = {
            "id": rewards.potion.potion.id,
            "name": rewards.potion.potion.name,
            "claimed": rewards.potion.claimed,
            "skipped": rewards.potion.skipped,
        }

    obs["card_rewards"] = [
        {
            "cards": [
                {
                    "id": card.id,
                    "name": card.name,
                    "upgraded": card.upgraded,
                    "rarity": card.rarity.name if hasattr(card, 'rarity') else "COMMON",
                }
                for card in card_reward.cards
            ],
            "claimed_index": card_reward.claimed_index,
            "skipped": card_reward.skipped,
            "singing_bowl_used": card_reward.singing_bowl_used,
        }
        for card_reward in rewards.card_rewards
    ]

    if rewards.relic:
        obs["relic"] = {
            "id": rewards.relic.relic.id,
            "name": rewards.relic.relic.name,
            "claimed": rewards.relic.claimed,
        }

    if rewards.boss_relics:
        obs["boss_relics"] = {
            "relics": [
                {"id": relic.id, "name": relic.name}
                for relic in rewards.boss_relics.relics
            ],
            "chosen_index": rewards.boss_relics.chosen_index,
        }

    if rewards.emerald_key:
        obs["emerald_key"] = {
            "available": True,
            "claimed": rewards.emerald_key.claimed,
        }

    return obs


def generate_shop_observation(runner) -> Optional[Dict[str, Any]]:
    """Generate the shop section of the observation."""
    if runner.current_shop is None:
        return None

    shop = runner.current_shop

    return {
        "colored_cards": [
            {
                "id": sc.card.id,
                "name": sc.card.name,
                "upgraded": sc.card.upgraded,
                "price": sc.price,
                "purchased": sc.purchased,
                "on_sale": sc.on_sale,
            }
            for sc in shop.colored_cards
        ],
        "colorless_cards": [
            {
                "id": sc.card.id,
                "name": sc.card.name,
                "upgraded": sc.card.upgraded,
                "price": sc.price,
                "purchased": sc.purchased,
            }
            for sc in shop.colorless_cards
        ],
        "relics": [
            {
                "id": sr.relic.id,
                "name": sr.relic.name,
                "price": sr.price,
                "purchased": sr.purchased,
            }
            for sr in shop.relics
        ],
        "potions": [
            {
                "id": sp.potion.id,
                "name": sp.potion.name,
                "price": sp.price,
                "purchased": sp.purchased,
            }
            for sp in shop.potions
        ],
        "purge_cost": shop.purge_cost,
        "purge_available": shop.purge_available,
    }


def generate_rest_observation(runner) -> Optional[Dict[str, Any]]:
    """Generate the rest section of the observation."""
    available = []

    if not runner.run_state.has_relic("Coffee Dripper"):
        if runner.run_state.current_hp < runner.run_state.max_hp:
            available.append("rest")

    if runner.run_state.get_upgradeable_cards():
        available.append("smith")

    if runner.run_state.has_relic("Shovel"):
        available.append("dig")

    if runner.run_state.has_relic("Girya"):
        counter = runner.run_state.get_relic_counter("Girya")
        if counter < 3:
            available.append("lift")

    if runner.run_state.has_relic("Peace Pipe"):
        available.append("toke")

    if runner.run_state.act == 3 and not runner.run_state.has_ruby_key:
        available.append("recall")

    return {
        "available_actions": available,
    }


def generate_treasure_observation(runner) -> Optional[Dict[str, Any]]:
    """Generate the treasure section of the observation."""
    return {
        "chest_type": runner.current_chest_type.value if runner.current_chest_type else "unknown",
        "sapphire_key_available": runner.run_state.act == 3 and not runner.run_state.has_sapphire_key,
    }


# =============================================================================
# GameRunner Extension Methods (to be added to GameRunner)
# =============================================================================

def get_available_action_dicts(runner) -> List[ActionDict]:
    """Return available actions via GameRunner's JSON API."""
    return runner.get_available_action_dicts()


def take_action_dict(runner, action: ActionDict) -> ActionResult:
    """Execute a JSON action dict via GameRunner."""
    try:
        result = runner.take_action_dict(action)
        if isinstance(result, dict):
            return result
        return {"success": bool(result), "data": {}}
    except Exception as exc:
        return {"success": False, "error": str(exc)}


def get_observation(runner) -> ObservationDict:
    """Return the current observation via GameRunner's JSON API."""
    return runner.get_observation()
