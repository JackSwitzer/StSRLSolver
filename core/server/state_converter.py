"""
State converter: JSON from Java → CombatState for Python simulation.

Handles the conversion of TurnStateCapture JSON format to CombatState objects
for use in the simulation engine.
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional, Tuple

from ..state.combat import (
    CombatState,
    EntityState,
    EnemyCombatState,
    create_player,
    create_enemy,
)
from .protocol import SearchRequest


def json_to_combat_state(
    data: Dict[str, Any],
    card_registry: Optional[Dict[str, dict]] = None,
) -> CombatState:
    """
    Convert JSON from Java TurnStateCapture to CombatState.

    Args:
        data: JSON dict from Java (either raw capture or SearchRequest fields)
        card_registry: Optional card registry for cost/target info

    Returns:
        CombatState object ready for simulation
    """
    # Handle both raw JSON and SearchRequest structure
    player_data = data.get("player", {})
    card_piles = data.get("card_piles", data)  # Fallback to top-level
    enemies_data = data.get("enemies", data.get("monsters", []))
    potions_data = data.get("potions", [])
    relics_data = data.get("relics", [])

    # Build player state
    player = _build_player_state(player_data)

    # Build enemy states
    enemies = [_build_enemy_state(e) for e in enemies_data]

    # Extract card piles
    hand = _extract_card_ids(card_piles.get("hand", []))
    draw_pile = _extract_card_ids(card_piles.get("draw_pile", []))
    discard_pile = _extract_card_ids(card_piles.get("discard_pile", []))
    exhaust_pile = _extract_card_ids(card_piles.get("exhaust_pile", []))

    # Extract potions (non-empty slots)
    potions = _extract_potion_ids(potions_data)

    # Extract relics
    relics = _extract_relic_ids(relics_data)

    # Build card costs cache
    card_costs = _build_card_costs(card_piles.get("hand", []))

    # Get energy
    energy = player_data.get("energy", 3)
    max_energy = player_data.get("max_energy", 3)

    # Get stance
    stance = player_data.get("stance_id", player_data.get("stance", "Neutral"))
    if stance == "com.megacrit.cardcrawl.stances.NeutralStance":
        stance = "Neutral"
    elif stance == "com.megacrit.cardcrawl.stances.WrathStance":
        stance = "Wrath"
    elif stance == "com.megacrit.cardcrawl.stances.CalmStance":
        stance = "Calm"
    elif stance == "com.megacrit.cardcrawl.stances.DivinityStance":
        stance = "Divinity"

    return CombatState(
        player=player,
        energy=energy,
        max_energy=max_energy,
        stance=stance,
        hand=hand,
        draw_pile=draw_pile,
        discard_pile=discard_pile,
        exhaust_pile=exhaust_pile,
        enemies=enemies,
        potions=potions,
        relics=relics,
        card_costs=card_costs,
    )


def request_to_combat_state(
    request: SearchRequest,
    card_registry: Optional[Dict[str, dict]] = None,
) -> CombatState:
    """
    Convert a SearchRequest to CombatState.

    Args:
        request: SearchRequest from Java
        card_registry: Optional card registry for cost/target info

    Returns:
        CombatState object ready for simulation
    """
    return json_to_combat_state(
        {
            "player": request.player,
            "card_piles": request.card_piles,
            "enemies": request.enemies,
            "potions": request.potions,
            "relics": request.relics,
        },
        card_registry=card_registry,
    )


def _build_player_state(data: Dict[str, Any]) -> EntityState:
    """Build EntityState from player JSON."""
    hp = data.get("current_hp", data.get("hp", 70))
    max_hp = data.get("max_hp", 70)
    block = data.get("block", 0)

    # Build statuses dict from powers
    statuses = {}
    powers = data.get("powers", [])
    for power in powers:
        power_id = power.get("id", "")
        amount = power.get("amount", 1)
        if power_id:
            statuses[power_id] = amount

    # Also check direct status fields
    if data.get("strength", 0) != 0:
        statuses["Strength"] = data.get("strength", 0)
    if data.get("dexterity", 0) != 0:
        statuses["Dexterity"] = data.get("dexterity", 0)
    if data.get("vulnerable", 0) > 0:
        statuses["Vulnerable"] = data.get("vulnerable", 0)
    if data.get("weak", 0) > 0:
        statuses["Weak"] = data.get("weak", 0)
    if data.get("frail", 0) > 0:
        statuses["Frail"] = data.get("frail", 0)
    if data.get("intangible", 0) > 0:
        statuses["Intangible"] = data.get("intangible", 0)

    return EntityState(
        hp=hp,
        max_hp=max_hp,
        block=block,
        statuses=statuses,
    )


def _build_enemy_state(data: Dict[str, Any]) -> EnemyCombatState:
    """Build EnemyCombatState from enemy JSON."""
    enemy_id = data.get("id", data.get("name", "Unknown"))
    hp = data.get("current_hp", data.get("hp", 40))
    max_hp = data.get("max_hp", 40)
    block = data.get("block", 0)

    # Build statuses from powers
    statuses = {}
    powers = data.get("powers", [])
    for power in powers:
        power_id = power.get("id", "")
        amount = power.get("amount", 1)
        if power_id:
            statuses[power_id] = amount

    # Also check direct status fields
    if data.get("strength", 0) != 0:
        statuses["Strength"] = data.get("strength", 0)
    if data.get("vulnerable", 0) > 0:
        statuses["Vulnerable"] = data.get("vulnerable", 0)
    if data.get("weak", 0) > 0:
        statuses["Weak"] = data.get("weak", 0)

    # Extract move info
    move_id = data.get("move_id", -1)
    if move_id == -1:
        # Try to infer from intent
        intent = data.get("intent", "UNKNOWN")
        if intent == "ATTACK":
            move_id = 1
        elif intent == "ATTACK_DEFEND":
            move_id = 2
        elif intent == "DEFEND":
            move_id = 3
        elif intent == "BUFF":
            move_id = 4

    # Get damage info
    move_damage = data.get("intent_calculated_damage", data.get("intent_base_damage", 0))
    if move_damage == -1:
        move_damage = 0

    move_hits = data.get("intent_multi", 1)
    if move_hits == -1:
        move_hits = 1

    move_block = data.get("move_block", 0)

    # Move effects (strength gain, etc.)
    move_effects = {}
    if data.get("intent", "") in ["BUFF", "ATTACK_BUFF", "DEFEND_BUFF"]:
        # Will need to look up specific enemy moves
        pass

    return EnemyCombatState(
        hp=hp,
        max_hp=max_hp,
        block=block,
        statuses=statuses,
        id=enemy_id,
        move_id=move_id,
        move_damage=move_damage,
        move_hits=move_hits,
        move_block=move_block,
        move_effects=move_effects,
    )


def _extract_card_ids(cards: List[Any]) -> List[str]:
    """Extract card IDs from card list."""
    result = []
    for card in cards:
        if isinstance(card, str):
            result.append(card)
        elif isinstance(card, dict):
            card_id = card.get("id", card.get("name", "Unknown"))
            # Handle upgrades
            upgraded = card.get("upgraded", False) or card.get("timesUpgraded", 0) > 0
            if upgraded and not card_id.endswith("+"):
                card_id = card_id + "+"
            result.append(card_id)
    return result


def _extract_potion_ids(potions: List[Any]) -> List[str]:
    """Extract potion IDs from potion list."""
    result = []
    for potion in potions:
        if isinstance(potion, str):
            result.append(potion if potion != "Potion Slot" else "")
        elif isinstance(potion, dict):
            potion_id = potion.get("id", "")
            # Skip placeholder slots
            if potion.get("is_placeholder", False) or potion_id == "Potion Slot":
                result.append("")
            else:
                result.append(potion_id)
    return result


def _extract_relic_ids(relics: List[Any]) -> List[str]:
    """Extract relic IDs from relic list."""
    result = []
    for relic in relics:
        if isinstance(relic, str):
            result.append(relic)
        elif isinstance(relic, dict):
            relic_id = relic.get("id", relic.get("name", "Unknown"))
            result.append(relic_id)
    return result


def _build_card_costs(hand_cards: List[Any]) -> Dict[str, int]:
    """Build card cost cache from hand cards."""
    costs = {}
    for card in hand_cards:
        if isinstance(card, dict):
            card_id = card.get("id", card.get("name", ""))
            # Handle upgrades
            upgraded = card.get("upgraded", False) or card.get("timesUpgraded", 0) > 0
            if upgraded and not card_id.endswith("+"):
                card_id = card_id + "+"
            # Use cost_for_turn if available (accounts for modifiers)
            cost = card.get("cost_for_turn", card.get("cost", 1))
            if cost == -1:  # X cost cards
                cost = 0  # Will use all energy
            costs[card_id] = cost
    return costs


# =============================================================================
# RNG State Handling
# =============================================================================


def extract_rng_state(data: Dict[str, Any]) -> Dict[str, int]:
    """
    Extract RNG counters from Java state.

    Returns dict mapping RNG stream name to counter value.
    """
    rng_state = data.get("rng_state", {})
    counters = rng_state.get("rng_counters", {})

    # Normalize counter names
    return {
        "card_rng": counters.get("cardRng", 0),
        "monster_rng": counters.get("monsterRng", 0),
        "event_rng": counters.get("eventRng", 0),
        "relic_rng": counters.get("relicRng", 0),
        "treasure_rng": counters.get("treasureRng", 0),
        "potion_rng": counters.get("potionRng", 0),
        "merchant_rng": counters.get("merchantRng", 0),
        "monster_hp_rng": counters.get("monsterHpRng", 0),
        "ai_rng": counters.get("aiRng", 0),
        "shuffle_rng": counters.get("shuffleRng", 0),
        "card_random_rng": counters.get("cardRandomRng", 0),
        "misc_rng": counters.get("miscRng", 0),
    }


def extract_seed(data: Dict[str, Any]) -> Tuple[int, str]:
    """
    Extract seed from Java state.

    Returns (seed_long, seed_string) tuple.
    """
    seed_data = data.get("seed", data.get("run_info", {}))

    seed_long = seed_data.get("seed_long", seed_data.get("seed", 0))
    seed_string = seed_data.get("seed_string", str(seed_long))

    return (seed_long, seed_string)


# =============================================================================
# State Comparison (for verification)
# =============================================================================


def compare_states(
    predicted: CombatState,
    actual: Dict[str, Any],
) -> List[Dict[str, Any]]:
    """
    Compare predicted CombatState with actual Java state.

    Returns list of mismatches with diagnosis information.
    """
    mismatches = []
    actual_state = json_to_combat_state(actual)

    # Player HP
    if predicted.player.hp != actual_state.player.hp:
        mismatches.append({
            "field": "player.hp",
            "predicted": predicted.player.hp,
            "actual": actual_state.player.hp,
            "diff": actual_state.player.hp - predicted.player.hp,
            "diagnosis": "HP mismatch - check damage calculation",
        })

    # Player block
    if predicted.player.block != actual_state.player.block:
        mismatches.append({
            "field": "player.block",
            "predicted": predicted.player.block,
            "actual": actual_state.player.block,
            "diagnosis": "Block mismatch - check block calculation or Frail",
        })

    # Energy
    if predicted.energy != actual_state.energy:
        mismatches.append({
            "field": "energy",
            "predicted": predicted.energy,
            "actual": actual_state.energy,
            "diagnosis": "Energy mismatch - check card costs or stance exit",
        })

    # Stance
    if predicted.stance != actual_state.stance:
        mismatches.append({
            "field": "stance",
            "predicted": predicted.stance,
            "actual": actual_state.stance,
            "diagnosis": "Stance mismatch - check stance change triggers",
        })

    # Enemy HP
    for i, (pred_enemy, actual_enemy) in enumerate(
        zip(predicted.enemies, actual_state.enemies)
    ):
        if pred_enemy.hp != actual_enemy.hp:
            mismatches.append({
                "field": f"enemies[{i}].hp",
                "enemy_id": pred_enemy.id,
                "predicted": pred_enemy.hp,
                "actual": actual_enemy.hp,
                "diff": actual_enemy.hp - pred_enemy.hp,
                "diagnosis": "Enemy HP mismatch - check damage with Strength/Weak/Vulnerable",
            })

        if pred_enemy.block != actual_enemy.block:
            mismatches.append({
                "field": f"enemies[{i}].block",
                "enemy_id": pred_enemy.id,
                "predicted": pred_enemy.block,
                "actual": actual_enemy.block,
                "diagnosis": "Enemy block mismatch - check enemy turn order",
            })

    # Hand size
    if len(predicted.hand) != len(actual_state.hand):
        mismatches.append({
            "field": "hand.length",
            "predicted": len(predicted.hand),
            "actual": len(actual_state.hand),
            "diagnosis": "Hand size mismatch - check draw/discard effects",
        })

    return mismatches


def diagnose_mismatch(mismatch: Dict[str, Any]) -> str:
    """
    Provide detailed diagnosis for a mismatch.

    Returns human-readable diagnosis string.
    """
    field = mismatch.get("field", "")
    diff = mismatch.get("diff", 0)

    if "hp" in field and "enemy" in field:
        if diff > 0:
            return (
                f"Enemy took {-diff} LESS damage than predicted. "
                "Check: Strength application, Weak calculation, multi-hit cards"
            )
        else:
            return (
                f"Enemy took {-diff} MORE damage than predicted. "
                "Check: Vulnerable, Pen Nib, Wrath stance"
            )

    if field == "player.hp":
        if diff > 0:
            return (
                f"Player took {-diff} LESS damage than predicted. "
                "Check: Block calculation, Weak on enemies, Intangible"
            )
        else:
            return (
                f"Player took {-diff} MORE damage than predicted. "
                "Check: Vulnerable, Wrath stance damage taken"
            )

    if field == "energy":
        return (
            f"Energy off by {diff}. "
            "Check: Card costs, Calm exit (+2), Divinity (+3), relics"
        )

    if field == "stance":
        return (
            f"Stance mismatch. "
            "Check: Flurry of Blows/Rushdown triggers, stance card effects"
        )

    return mismatch.get("diagnosis", "Unknown cause")
