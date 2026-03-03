"""
Auto-generate CARD_EFFECTS dict from engine card definitions.

Reads ALL_CARDS from packages/engine/content/cards.py and produces a
dict compatible with LineSimulator.simulate_line(). Covers all 300+ cards
(all characters + colorless + curses + statuses), always in sync with engine.

~80% auto-extracted from Card fields. ~15 cards need manual overrides
for special mechanics that can't be inferred from the dataclass alone.
"""

from __future__ import annotations

from typing import Any, Dict, Optional


def generate_card_effects() -> Dict[str, Dict[str, Any]]:
    """Generate CARD_EFFECTS dict from engine card definitions.

    Returns dict mapping card_id (and card_id+) to effect dicts usable
    by LineSimulator.simulate_line().
    """
    from packages.engine.content.cards import (
        ALL_CARDS,
        CardTarget,
        CardType,
    )

    effects: Dict[str, Dict[str, Any]] = {}

    for card_id, card in ALL_CARDS.items():
        # Base version
        entry = _card_to_effects(card, upgraded=False)
        if entry:
            effects[card_id] = entry

        # Upgraded version
        entry_up = _card_to_effects(card, upgraded=True)
        if entry_up:
            effects[card_id + "+"] = entry_up

    # Apply manual overrides for special mechanics
    _apply_overrides(effects)

    return effects


def _card_to_effects(card: Any, upgraded: bool = False) -> Dict[str, Any]:
    """Convert a Card dataclass to a LineSimulator effect dict."""
    from packages.engine.content.cards import CardTarget, CardType

    entry: Dict[str, Any] = {}

    # Cost
    if upgraded and card.upgrade_cost is not None:
        entry["cost"] = card.upgrade_cost
    else:
        entry["cost"] = card.cost

    # Unplayable cards (cost=-2 or has "unplayable" effect)
    if card.cost == -2 or "unplayable" in card.effects:
        entry["cost"] = 99  # LineSimulator skips cards with cost > energy
        return entry

    # X-cost cards
    if card.cost == -1:
        entry["cost"] = 0  # X-cost: playable with any energy

    # Damage
    base_dmg = card.base_damage
    if base_dmg >= 0:
        dmg = base_dmg + (card.upgrade_damage if upgraded else 0)
        entry["damage"] = dmg

    # Block
    base_blk = card.base_block
    if base_blk >= 0:
        blk = base_blk + (card.upgrade_block if upgraded else 0)
        entry["block"] = blk

    # AoE (from target type)
    if card.target == CardTarget.ALL_ENEMY:
        entry["aoe"] = True

    # Stance
    if card.enter_stance:
        entry["enters"] = card.enter_stance
    if card.exit_stance:
        entry["exits_stance"] = True

    # Parse effects list for LineSimulator-relevant mechanics
    magic = card.base_magic
    if upgraded:
        magic = card.base_magic + card.upgrade_magic if card.base_magic >= 0 else -1

    _parse_effects(entry, card.effects, magic, card)

    return entry


# Effect strings that map to multi-hit (base_magic = hit count)
_MULTI_HIT_EFFECTS = frozenset({
    "damage_x_times", "hits_x_times", "damage_all_x_times",
    "damage_random_x_times", "random_enemy_x_times",
})

# Effect strings that map to draw
_DRAW_EFFECTS = {
    "draw_1": 1,
    "draw_2": 2,
}

# Effect strings that mean end turn
_END_TURN_EFFECTS = frozenset({
    "end_turn",
})

# Effect strings for scry
_SCRY_EFFECTS = frozenset({
    "scry",
})

# Effect strings for mantra
_MANTRA_EFFECTS = frozenset({
    "gain_mantra",
})


def _parse_effects(
    entry: Dict[str, Any],
    effects: list,
    magic: int,
    card: Any,
) -> None:
    """Parse effect strings and enrich the entry dict."""
    for eff in effects:
        # Multi-hit: base_magic = number of hits
        if eff in _MULTI_HIT_EFFECTS:
            if magic > 0:
                entry["hits"] = magic

        # Draw
        elif eff in _DRAW_EFFECTS:
            entry["draw"] = _DRAW_EFFECTS[eff]
        elif eff == "draw_cards" or eff == "draw_x":
            if magic > 0:
                entry["draw"] = magic

        # End turn
        elif eff in _END_TURN_EFFECTS:
            entry["ends_turn"] = True

        # Scry
        elif eff in _SCRY_EFFECTS:
            if magic > 0:
                entry["scry"] = magic

        # Mantra
        elif eff in _MANTRA_EFFECTS:
            if magic > 0:
                entry["mantra"] = magic

        # Draw until hand full
        elif eff == "draw_until_hand_full":
            entry["draw_to_full"] = True

        # Die next turn (Blasphemy)
        elif eff == "die_next_turn":
            entry["die_next_turn"] = True

        # Take extra turn (Vault)
        elif eff == "take_extra_turn":
            entry["extra_turn"] = True

        # Apply weak
        elif eff == "apply_weak":
            if magic > 0:
                entry["weak"] = magic
        elif eff == "apply_weak_all" or eff == "apply_weak_2_all":
            if magic > 0:
                entry["weak_all"] = magic

        # Apply vulnerable
        elif eff == "apply_vulnerable":
            if magic > 0:
                entry["vulnerable"] = magic
        elif eff == "apply_vulnerable_1_all":
            entry["vulnerable_all"] = magic if magic > 0 else 1

        # Conditional stance entry
        elif eff == "if_enemy_attacking_enter_calm":
            entry["enters"] = "Calm"
            entry["conditional"] = "enemy_attacking"
        elif eff == "if_calm_draw_else_calm":
            entry["enters"] = "Calm"

        # Wrath-conditional block bonus
        elif eff.startswith("if_in_wrath_extra_block_"):
            try:
                bonus = int(eff.split("_")[-1])
                entry["wrath_bonus"] = bonus
            except ValueError:
                pass

        # Block equal to unblocked damage (Wallop)
        elif eff == "gain_block_equal_unblocked_damage":
            entry["block_equal_damage"] = True

        # Block per card in hand (Spirit Shield)
        elif eff == "gain_block_per_card_in_hand":
            if magic > 0:
                entry["block_per_card"] = magic

        # Energy gain
        elif eff == "gain_1_energy":
            entry["gain_energy"] = 1
        elif eff == "gain_2_energy" or eff == "gain_energy_2":
            entry["gain_energy"] = 2
        elif eff == "gain_energy_magic":
            if magic > 0:
                entry["gain_energy"] = magic

        # If last card was attack, gain energy (FollowUp)
        elif eff == "if_last_card_attack_gain_energy":
            entry["energy_if_last_attack"] = 1

        # On stance change (FlurryOfBlows)
        elif eff == "on_stance_change_play_from_discard":
            entry["on_stance_change"] = True

        # Retain flag
        # (already on Card dataclass, but some effects grant it contextually)

        # Execute (Judgement) — handled in manual overrides since threshold
        # is stored in base_magic but means "kill if HP <= X"


def _apply_overrides(effects: Dict[str, Dict[str, Any]]) -> None:
    """Apply manual overrides for cards with special mechanics."""

    # Judgement: kill enemy if HP <= threshold (base_magic=30, upgrade=+10)
    if "Judgement" in effects:
        effects["Judgement"]["execute"] = 30
        effects["Judgement"]["damage"] = 0  # No direct damage
    if "Judgement+" in effects:
        effects["Judgement+"]["execute"] = 40
        effects["Judgement+"]["damage"] = 0

    # BowlingBash: damage per enemy (NOT aoe, targets single enemy)
    # The sim treats it as single-target with bonus damage per enemy alive
    if "BowlingBash" in effects:
        effects["BowlingBash"].pop("aoe", None)
        effects["BowlingBash"]["damage_per_enemy"] = True
    if "BowlingBash+" in effects:
        effects["BowlingBash+"].pop("aoe", None)
        effects["BowlingBash+"]["damage_per_enemy"] = True

    # Blasphemy: enters Divinity (not just die_next_turn)
    if "Blasphemy" in effects:
        effects["Blasphemy"]["enters"] = "Divinity"
    if "Blasphemy+" in effects:
        effects["Blasphemy+"]["enters"] = "Divinity"

    # Smite/ThroughViolence: retain flag
    for cid in ("Smite", "Smite+", "ThroughViolence", "ThroughViolence+"):
        if cid in effects:
            effects[cid]["retain"] = True

    # Perseverance: block scales when retained
    for cid in ("Perseverance", "Perseverance+"):
        if cid in effects:
            effects[cid]["retain"] = True
            effects[cid]["scales"] = True

    # Protect: retain
    for cid in ("Protect", "Protect+"):
        if cid in effects:
            effects[cid]["retain"] = True

    # Crescendo/Tranquility: retain
    for cid in ("Crescendo", "Crescendo+", "ClearTheMind", "ClearTheMind+"):
        if cid in effects:
            effects[cid]["retain"] = True

    # Windmill Strike: retain + damage_up when retained
    for cid in ("WindmillStrike", "WindmillStrike+"):
        if cid in effects:
            effects[cid]["retain"] = True
            effects[cid]["damage_up"] = 4

    # Sands of Time: retain + cost reduces
    for cid in ("SandsOfTime", "SandsOfTime+"):
        if cid in effects:
            effects[cid]["retain"] = True
            effects[cid]["cost_down"] = 1

    # ReachHeaven: adds ThroughViolence to hand
    for cid in ("ReachHeaven", "ReachHeaven+"):
        if cid in effects:
            effects[cid]["add_to_hand"] = "ThroughViolence"

    # Evaluate: adds Insight to draw pile
    for cid in ("Evaluate", "Evaluate+"):
        if cid in effects:
            effects[cid]["add_insight"] = True

    # Meditate: ends turn, enters Calm, retrieves cards
    if "Meditate" in effects:
        effects["Meditate"]["ends_turn"] = True
        effects["Meditate"]["enters"] = "Calm"
        effects["Meditate"]["retrieve"] = 1
    if "Meditate+" in effects:
        effects["Meditate+"]["ends_turn"] = True
        effects["Meditate+"]["enters"] = "Calm"
        effects["Meditate+"]["retrieve"] = 2

    # TalkToTheHand: applies block-on-attack debuff
    for cid in ("TalkToTheHand", "TalkToTheHand+"):
        if cid in effects:
            effects[cid]["block_on_attack"] = effects[cid].get("block_on_attack", 2)

    # SignatureMove: only playable if only attack in hand
    for cid in ("SignatureMove", "SignatureMove+"):
        if cid in effects:
            effects[cid]["only_attack"] = True

    # LessonLearned: upgrade on kill
    for cid in ("LessonLearned", "LessonLearned+"):
        if cid in effects:
            effects[cid]["upgrade_on_kill"] = True

    # Weave: plays from discard on scry
    for cid in ("Weave", "Weave+"):
        if cid in effects:
            effects[cid]["on_scry"] = True

    # Brilliance: damage + mantra gained this combat
    for cid in ("Brilliance", "Brilliance+"):
        if cid in effects:
            effects[cid]["per_mantra"] = True

    # Wish: choose buff
    for cid in ("Wish", "Wish+"):
        if cid in effects:
            effects[cid]["choose_buff"] = True

    # Vault: extra turn
    if "Vault" in effects:
        effects["Vault"]["extra_turn"] = True
    if "Vault+" in effects:
        effects["Vault+"]["extra_turn"] = True

    # Scrawl: draw to full hand
    for cid in ("Scrawl", "Scrawl+"):
        if cid in effects:
            effects[cid]["draw_to_full"] = True


# ──────────────────────────────────────────────────────────────────
# Module-level generation: runs once at import time
# ──────────────────────────────────────────────────────────────────

_GENERATED: Optional[Dict[str, Dict[str, Any]]] = None


def get_card_effects() -> Dict[str, Dict[str, Any]]:
    """Get or lazily generate the CARD_EFFECTS dict."""
    global _GENERATED
    if _GENERATED is None:
        _GENERATED = generate_card_effects()
    return _GENERATED
