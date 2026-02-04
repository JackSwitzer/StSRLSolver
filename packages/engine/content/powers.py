"""
Powers System - Comprehensive implementation of all Slay the Spire buffs/debuffs.

Extracted from decompiled source at:
/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/powers/

=== POWER EXECUTION HOOKS (from AbstractPower.java) ===

Damage calculation hooks (in order):
1. atDamageGive(damage, type) - Attacker powers (Strength, Vigor, Weak, DoubleDamage, PenNib)
2. atDamageFinalGive(damage, type) - Final attacker mods (rare)
3. atDamageReceive(damage, type) - Defender powers (Vulnerable, Slow, Flight)
4. atDamageFinalReceive(damage, type) - Defender final (Intangible)
5. onAttackedToChangeDamage(info, damage) - Can prevent damage (Buffer, Invincible)

Block hooks:
1. modifyBlock(block) - Powers that add/mult block (Dexterity, Frail)
2. modifyBlockLast(block) - Final block mods (NoBlock sets to 0)

Turn hooks:
- atStartOfTurn() - Before draw (Poison, Foresight)
- atStartOfTurnPostDraw() - After draw (DemonForm, Devotion, NoxiousFumes)
- atEndOfTurn(isPlayer) - End of owner's turn (Intangible decrement, Constricted damage)
- atEndOfTurnPreEndTurnCards(isPlayer) - Before discarding (Metallicize, PlatedArmor block)
- atEndOfRound() - After all turns (duration debuff decrement: Weak, Vuln, Frail)
- duringTurn() - During turn (Fading countdown)

Card hooks:
- onCardDraw(card) - When drawn (Evolve, Corruption cost change)
- onUseCard(card, action) - When played (Vigor consumed, AfterImage block)
- onAfterUseCard(card, action) - After resolve (BeatOfDeath damage, TimeWarp count)
- onExhaust(card) - When exhausted (DarkEmbrace draw)

Combat hooks:
- onAttack(info, damage, target) - After dealing damage (Envenom poison)
- onAttacked(info, damage) - When hit, return modified damage (Thorns counter)
- wasHPLost(info, damage) - After losing HP (PlatedArmor decrement, Rupture str)
- onGainedBlock(block) - After gaining block (Juggernaut damage, WaveOfTheHand weak)
- onChangeStance(old, new) - Stance change (MentalFortress, Rushdown)
- onScry() - When scrying (Nirvana block)
- onApplyPower(power, target, source) - Any power applied (Sadistic damage)
- onSpecificTrigger() - Special trigger (Artifact debuff block)
- triggerMarks(card) - Pressure Points (Mark damage)
- onEnergyRecharge() - Start of turn energy (Energized, DevaForm)
- onRemove() - When power removed
- onDeath() - Owner dies
- onVictory() - Combat won (Repair heal)
- onHeal(amount) - Modify healing
- onLoseHp(damage) - Modify HP loss
- canPlayCard(card) - Restrict card play (Entangled blocks attacks)

=== POWER TYPE INFO ===

PowerType.BUFF - Beneficial effects
PowerType.DEBUFF - Harmful effects (can be blocked by Artifact)

isTurnBased - Duration powers that decrement (displayed in yellow)
canGoNegative - Str/Dex can go negative
priority - Execution order (lower = earlier, default 5)
"""

from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Optional, Dict, List, Any, Callable
import math


class PowerType(Enum):
    """Power classification matching AbstractPower.PowerType."""
    BUFF = "BUFF"
    DEBUFF = "DEBUFF"


class DamageType(Enum):
    """Damage types matching DamageInfo.DamageType."""
    NORMAL = "NORMAL"    # Standard attack damage - affected by Str, Weak, Vuln, stances
    THORNS = "THORNS"    # Retaliation damage - not affected by most modifiers
    HP_LOSS = "HP_LOSS"  # Direct HP loss - ignores block, not affected by modifiers


# =============================================================================
# CORE POWER DATA STRUCTURE
# =============================================================================

@dataclass
class Power:
    """
    Represents a power (buff/debuff) in combat.
    Based on AbstractPower.java from decompiled source.

    Attributes:
        id: Exact string ID from game (e.g., "Weakened", "Strength")
        name: Display name
        power_type: BUFF or DEBUFF
        amount: Stack count (can be negative for Str/Dex)
        is_turn_based: If True, displayed yellow and decrements
        can_go_negative: If True, can have negative amount
        stacks: If True, amount increases when reapplied
        priority: Execution order (lower = earlier)
        just_applied: For monster debuffs, skip first decrement
        max_amount: Cap for stacking (usually 999)
        min_amount: Floor for negative (usually -999)
    """
    id: str
    name: str
    power_type: PowerType
    amount: int = 1
    is_turn_based: bool = False
    can_go_negative: bool = False
    stacks: bool = True
    priority: int = 5
    just_applied: bool = False
    max_amount: int = 999
    min_amount: int = -999

    def stack(self, add_amount: int) -> None:
        """Add to power amount, respecting caps."""
        if not self.stacks:
            return
        self.amount += add_amount
        self.amount = max(self.min_amount, min(self.max_amount, self.amount))

    def reduce(self, reduce_amount: int) -> bool:
        """Reduce power amount. Returns True if should be removed."""
        self.amount -= reduce_amount
        return self.amount <= 0 and not self.can_go_negative

    def should_remove(self) -> bool:
        """Check if power should be removed (0 stacks and can't go negative)."""
        # Special case: Dexterity/Strength are removed at exactly 0 in Java
        # even though can_go_negative=True
        if self.id in ("Strength", "Dexterity") and self.amount == 0:
            return True
        return self.amount <= 0 and not self.can_go_negative


# =============================================================================
# POWER REGISTRY - All power data from decompiled source
# =============================================================================

# Complete power definitions with exact IDs and mechanics
POWER_DATA = {
    # =========== COMMON DEBUFFS ===========
    "Weakened": {
        "name": "Weak",
        "type": PowerType.DEBUFF,
        "is_turn_based": True,
        "priority": 99,
        "mechanics": {
            "at_damage_give": "damage * 0.75 (0.6 with Paper Crane vs player)",
            "at_end_of_round": "decrement by 1",
        },
        "notes": "Applied to owner. Reduces NORMAL damage dealt by 25%."
    },
    "Vulnerable": {
        "name": "Vulnerable",
        "type": PowerType.DEBUFF,
        "is_turn_based": True,
        "mechanics": {
            "at_damage_receive": "damage * 1.5 (1.25 Odd Mushroom, 1.75 Paper Frog)",
            "at_end_of_round": "decrement by 1",
        },
        "notes": "Applied to owner. Take 50% more NORMAL damage."
    },
    "Frail": {
        "name": "Frail",
        "type": PowerType.DEBUFF,
        "is_turn_based": True,
        "priority": 10,
        "mechanics": {
            "modify_block": "block * 0.75",
            "at_end_of_round": "decrement by 1",
        },
        "notes": "Block from cards reduced by 25%."
    },
    "Poison": {
        "name": "Poison",
        "type": PowerType.DEBUFF,
        "is_turn_based": True,
        "max_amount": 9999,
        "mechanics": {
            "at_start_of_turn": "deal amount HP_LOSS, then decrement",
        },
        "notes": "Deals damage and decrements at START of turn (not end of round)."
    },
    "Constricted": {
        "name": "Constricted",
        "type": PowerType.DEBUFF,
        "priority": 105,
        "mechanics": {
            "at_end_of_turn": "deal amount THORNS damage",
        },
        "notes": "Applied by Bronze Automaton's Constrict."
    },
    "Choked": {
        "name": "Choke",
        "type": PowerType.DEBUFF,
        "mechanics": {
            "on_use_card": "lose amount HP",
            "at_start_of_turn": "remove this power",
        },
        "notes": "Lose HP each card played. Removed at start of turn."
    },
    "Slow": {
        "name": "Slow",
        "type": PowerType.DEBUFF,
        "mechanics": {
            "at_damage_receive": "damage * (1 + amount * 0.1)",
            "on_after_use_card": "increment amount",
            "at_end_of_round": "reset amount to 0",
        },
        "notes": "Time Eater. Each card adds 10% damage taken. Resets each round."
    },
    "Lockon": {
        "name": "Lock-On",
        "type": PowerType.DEBUFF,
        "is_turn_based": True,
        "uses_just_applied": True,  # Added for Java parity - skips first decrement when monster-applied
        "mechanics": {
            "orb_damage": "damage * 1.5",
            "at_end_of_round": "decrement by 1",
        },
        "notes": "Defect. Orbs deal 50% more damage to this enemy."
    },
    "NoBlockPower": {
        "name": "No Block",
        "type": PowerType.DEBUFF,
        "is_turn_based": True,
        "mechanics": {
            "modify_block_last": "return 0",
            "at_end_of_round": "decrement by 1",
        },
        "notes": "Cannot gain block."
    },
    "Entangled": {
        "name": "Entangle",
        "type": PowerType.DEBUFF,
        "is_turn_based": True,
        "stacks": False,
        "mechanics": {
            "can_play_card": "False if card.type == ATTACK",
            "at_end_of_turn": "remove if isPlayer",
        },
        "notes": "Cannot play attacks this turn."
    },
    "No Draw": {
        "name": "No Draw",
        "type": PowerType.DEBUFF,
        "stacks": False,
        "mechanics": {
            "at_end_of_turn": "remove if isPlayer",
        },
        "notes": "Cannot draw cards this turn."
    },
    "Draw Reduction": {
        "name": "Draw Down",
        "type": PowerType.DEBUFF,
        "is_turn_based": True,
        "always_just_applied": True,  # Java: private boolean justApplied = true;
        "mechanics": {
            "on_initial_application": "gameHandSize -= 1",
            "on_remove": "gameHandSize += 1",
            "at_end_of_round": "decrement by 1",
        },
        "notes": "Draw 1 fewer card. Directly modifies gameHandSize. Always skips first decrement."
    },

    # =========== COMMON BUFFS ===========
    "Strength": {
        "name": "Strength",
        "type": PowerType.BUFF,  # Changes to DEBUFF if negative
        "can_go_negative": True,
        "mechanics": {
            "at_damage_give": "damage + amount (NORMAL only)",
            "on_stack_to_zero": "remove power",
        },
        "notes": "Can go negative (Strength Down). Adds flat damage to attacks."
    },
    "Dexterity": {
        "name": "Dexterity",
        "type": PowerType.BUFF,
        "can_go_negative": True,
        "mechanics": {
            "modify_block": "max(0, block + amount)",
            "on_stack_to_zero": "remove power",
        },
        "notes": "Can go negative. Adds flat block to cards."
    },
    "Focus": {
        "name": "Focus",
        "type": PowerType.BUFF,
        "can_go_negative": True,
        "mechanics": {
            "orb_passive": "add amount to passive orb value",
            "orb_evoke": "add amount to evoke value",
        },
        "notes": "Defect. Affects orb passive/evoke amounts."
    },
    "Artifact": {
        "name": "Artifact",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_specific_trigger": "when debuff blocked, decrement by 1",
        },
        "notes": "Blocks debuff application. Consumed on use."
    },
    "Intangible": {
        "name": "Intangible",
        "type": PowerType.BUFF,
        "is_turn_based": True,
        "priority": 75,
        "uses_just_applied": True,  # Java: justApplied = true when monster applies to self
        "mechanics": {
            "at_damage_receive_final": "if damage > 1, return 1",
            "at_end_of_turn": "decrement by 1",
        },
        "notes": "Monster version. All damage reduced to 1. Decrements at end of turn."
    },
    "IntangiblePlayer": {
        "name": "Intangible",
        "type": PowerType.BUFF,
        "is_turn_based": True,
        "priority": 75,
        "mechanics": {
            "at_damage_receive_final": "if damage > 1, return 1",
            "at_end_of_round": "decrement by 1",
        },
        "notes": "Player version. All damage reduced to 1. Decrements at end of round."
    },
    "Plated Armor": {
        "name": "Plated Armor",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_end_of_turn_pre_cards": "gain amount block",
            "was_hp_lost": "if NORMAL attack, decrement by 1",
            "on_remove": "trigger ARMOR_BREAK state for monsters",
        },
        "notes": "Gain block at end of turn. Loses 1 when hit by attack."
    },
    "Thorns": {
        "name": "Thorns",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_attacked": "if not THORNS/HP_LOSS, deal amount THORNS to attacker",
        },
        "notes": "Counter-attack when hit."
    },
    "Metallicize": {
        "name": "Metallicize",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_end_of_turn_pre_cards": "gain amount block",
        },
        "notes": "Gain block at end of turn. Does not lose stacks."
    },
    "Barricade": {
        "name": "Barricade",
        "type": PowerType.BUFF,
        "stacks": False,
        "mechanics": {
            "passive": "block not removed at start of turn",
        },
        "notes": "Ironclad power. Block persists."
    },
    "Buffer": {
        "name": "Buffer",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_attacked_to_change_damage": "if damage > 0, decrement and return 0",
        },
        "notes": "Prevent next HP loss entirely."
    },
    "Double Damage": {
        "name": "Double Damage",
        "type": PowerType.BUFF,
        "is_turn_based": True,
        "priority": 6,
        "uses_just_applied": True,  # Java: justApplied = isSourceMonster (skips first decrement when monster-applied)
        "mechanics": {
            "at_damage_give": "damage * 2.0 (NORMAL only)",
            "at_end_of_round": "decrement by 1",
        },
        "notes": "Attacks deal double damage."
    },
    "Pen Nib": {
        "name": "Pen Nib",
        "type": PowerType.BUFF,
        "priority": 6,
        "mechanics": {
            "at_damage_give": "damage * 2.0 (NORMAL only)",
            "on_use_card": "if ATTACK, remove power",
        },
        "notes": "From relic. Next attack deals double."
    },
    "Demon Form": {
        "name": "Demon Form",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_start_of_turn_post_draw": "apply amount Strength",
        },
        "notes": "Ironclad. Gain Strength each turn."
    },
    "Draw": {
        "name": "Draw Up",
        "type": PowerType.BUFF,
        "can_go_negative": True,
        "mechanics": {
            "on_initial_application": "gameHandSize += amount",
            "on_remove": "gameHandSize -= amount",
        },
        "notes": "Permanent draw change while active."
    },
    "Energized": {
        "name": "Energized",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_energy_recharge": "gain amount energy, then remove",
        },
        "notes": "Next turn gain energy. One-time effect."
    },
    "Blur": {
        "name": "Blur",
        "type": PowerType.BUFF,
        "is_turn_based": True,
        "mechanics": {
            "passive": "block not removed at start of turn",
            "at_end_of_round": "decrement by 1",
        },
        "notes": "Temporary Barricade effect."
    },
    "After Image": {
        "name": "After Image",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_use_card": "gain amount block",
        },
        "notes": "Silent. Gain block per card played."
    },
    "Repair": {
        "name": "Self Repair",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_victory": "heal amount HP",
        },
        "notes": "Defect. Heal after combat."
    },

    # =========== WATCHER POWERS ===========
    "Vigor": {
        "name": "Vigor",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_damage_give": "damage + amount (NORMAL only)",
            "on_use_card": "if ATTACK, remove power",
        },
        "notes": "Next attack deals bonus damage. Consumed on attack."
    },
    "Mantra": {
        "name": "Mantra",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_stack": "if amount >= 10, enter Divinity, subtract 10",
            "on_stack_to_zero": "remove power",
        },
        "notes": "At 10 Mantra, enter Divinity stance."
    },
    "PathToVictoryPower": {
        "name": "Mark",
        "type": PowerType.DEBUFF,
        "mechanics": {
            "trigger_marks": "if card is PressurePoints, deal amount HP_LOSS",
        },
        "notes": "Pressure Points debuff. Triggered by playing Pressure Points."
    },
    "BlockReturnPower": {
        "name": "Block Return",
        "type": PowerType.DEBUFF,
        "mechanics": {
            "on_attacked": "if NORMAL attack, player gains amount block",
        },
        "notes": "Talk to the Hand. Player gains block when attacking this enemy."
    },
    "Controlled": {
        "name": "Mental Fortress",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_change_stance": "if old != new, gain amount block",
        },
        "notes": "Gain block on stance change."
    },
    "Adaptation": {
        "name": "Rushdown",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_change_stance": "if entering Wrath and old != new, draw amount",
        },
        "notes": "Draw cards when entering Wrath."
    },
    "EstablishmentPower": {
        "name": "Establishment",
        "type": PowerType.BUFF,
        "priority": 25,
        "mechanics": {
            "at_end_of_turn": "reduce cost of retained cards by amount",
        },
        "notes": "Retained cards cost less."
    },
    "LikeWaterPower": {
        "name": "Like Water",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_end_of_turn_pre_cards": "if in Calm, gain amount block",
        },
        "notes": "Gain block at end of turn if in Calm."
    },
    "DevotionPower": {
        "name": "Devotion",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_start_of_turn_post_draw": "gain amount Mantra (or Divinity if >= 10)",
        },
        "notes": "Gain Mantra each turn."
    },
    "Nirvana": {
        "name": "Nirvana",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_scry": "gain amount block",
        },
        "notes": "Gain block when scrying."
    },
    "WaveOfTheHandPower": {
        "name": "Wave of the Hand",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_gained_block": "if block > 0, apply amount Weak to all enemies",
            "at_end_of_round": "remove power",
        },
        "notes": "Single turn. Apply Weak when gaining block."
    },
    "DevaForm": {
        "name": "Deva Form",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_energy_recharge": "gain energyGainAmount energy, then increase by amount",
        },
        "notes": "Starts at 1 energy/turn, increases each turn."
    },
    "OmegaPower": {
        "name": "Omega",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_end_of_turn": "if isPlayer, deal amount THORNS to all enemies",
        },
        "notes": "Lesion. Deal damage to all at end of turn."
    },
    "BattleHymn": {
        "name": "Battle Hymn",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_start_of_turn": "add amount Smite(s) to hand",
        },
        "notes": "Add Smite cards each turn."
    },
    "FreeAttackPower": {
        "name": "Free Attack",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_use_card": "if ATTACK, decrement; remove at 0",
        },
        "notes": "Swivel. Next X attacks cost 0."
    },
    "CannotChangeStancePower": {
        "name": "Cannot Change Stance",
        "type": PowerType.DEBUFF,
        "stacks": False,
        "mechanics": {
            "at_end_of_turn": "if isPlayer, remove",
        },
        "notes": "Cannot change stance this turn."
    },
    "WrathNextTurnPower": {
        "name": "Wrath Next Turn",
        "type": PowerType.BUFF,
        "stacks": False,
        "mechanics": {
            "at_start_of_turn": "enter Wrath, then remove",
        },
        "notes": "Tantrum effect."
    },
    "MasterRealityPower": {
        "name": "Master Reality",
        "type": PowerType.BUFF,
        "stacks": False,
        "mechanics": {
            "passive": "created cards are upgraded",
        },
        "notes": "Cards created during combat are upgraded."
    },
    "Study": {
        "name": "Study",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_end_of_turn": "shuffle amount Insight into draw pile",
        },
        "notes": "Add Insight cards to draw pile."
    },
    "WireheadingPower": {
        "name": "Foresight",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_start_of_turn": "Scry amount",
        },
        "notes": "Scry at start of turn."
    },

    # =========== BOSS/ENEMY POWERS ===========
    "BeatOfDeath": {
        "name": "Beat of Death",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_after_use_card": "deal amount THORNS to player",
        },
        "notes": "The Heart. Player takes damage per card."
    },
    "Curiosity": {
        "name": "Curiosity",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_use_card": "if POWER card, gain amount Strength",
        },
        "notes": "Awakened One. Gains Strength when player plays Powers."
    },
    "Mode Shift": {
        "name": "Mode Shift",
        "type": PowerType.BUFF,
        "mechanics": {
            "passive": "tracks damage taken; triggers form change at threshold",
        },
        "notes": "The Guardian. Changes form after taking X damage."
    },
    "Split": {
        "name": "Split",
        "type": PowerType.BUFF,
        "stacks": False,
        "mechanics": {
            "passive": "at <= 50% HP, split into smaller versions",
        },
        "notes": "Slimes. Split when low HP."
    },
    "Time Warp": {
        "name": "Time Warp",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_after_use_card": "increment counter; at 12, end turn + all enemies gain 2 Str",
        },
        "notes": "Time Eater. After 12 cards, forces turn end."
    },
    "Invincible": {
        "name": "Invincible",
        "type": PowerType.BUFF,
        "priority": 99,
        "mechanics": {
            "on_attacked_to_change_damage": "cap damage at amount, reduce amount",
            "at_start_of_turn": "reset amount to max",
        },
        "notes": "The Champ, Heart. Damage cap per turn."
    },
    "Angry": {
        "name": "Angry",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_attacked": "if damage > 0 from attack, gain amount Strength",
        },
        "notes": "Gremlin Nob. Gains Strength when hit."
    },
    "GrowthPower": {
        "name": "Ritual",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_end_of_round": "skip first, then gain amount Strength",
        },
        "notes": "Cultist. Gains Strength each round after first."
    },
    "Fading": {
        "name": "Fading",
        "type": PowerType.BUFF,
        "mechanics": {
            "during_turn": "decrement; at 1, suicide",
        },
        "notes": "Summoned creatures. Dies after X turns."
    },
    "Life Link": {
        "name": "Life Link",
        "type": PowerType.BUFF,
        "stacks": False,
        "mechanics": {
            "passive": "boss-specific effect while alive",
        },
        "notes": "Darklings, etc. Special death/revival mechanics."
    },
    "Thievery": {
        "name": "Thievery",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_attack": "steal gold when dealing damage",
        },
        "notes": "Thieves. Steal gold on hit."
    },

    # =========== IRONCLAD TURN-BASED POWERS ===========
    "Regeneration": {
        "name": "Regen",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_end_of_turn": "heal amount HP",
        },
        "notes": "Java RegenPower: heals at end of turn. Does NOT decrement."
    },
    "Combust": {
        "name": "Combust",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_end_of_turn": "lose hpLoss HP, deal amount THORNS to all enemies",
        },
        "notes": "Ironclad. Self-damage + AoE thorns at end of turn."
    },
    "Brutality": {
        "name": "Brutality",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_start_of_turn_post_draw": "draw amount cards, lose amount HP",
        },
        "notes": "Ironclad. Draw cards and lose HP at start of turn after draw."
    },
    "Feel No Pain": {
        "name": "Feel No Pain",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_exhaust": "gain amount block",
        },
        "notes": "Ironclad. Gain block whenever a card is exhausted."
    },
    "Fire Breathing": {
        "name": "Fire Breathing",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_card_draw": "if STATUS or CURSE card, deal amount THORNS to all enemies",
        },
        "notes": "Ironclad. Deal damage to all enemies when drawing Status/Curse."
    },
    "Thousand Cuts": {
        "name": "Thousand Cuts",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_after_card_played": "deal amount THORNS to all enemies",
        },
        "notes": "Silent. Deal damage to all enemies after playing any card."
    },

    # =========== IRONCLAD POWERS ===========
    "Corruption": {
        "name": "Corruption",
        "type": PowerType.BUFF,
        "stacks": False,
        "mechanics": {
            "on_card_draw": "if SKILL, set cost to 0",
            "on_use_card": "if SKILL, exhaust it",
        },
        "notes": "Skills cost 0 but exhaust."
    },
    "Flame Barrier": {
        "name": "Flame Barrier",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_attacked": "if NORMAL attack, deal amount THORNS to attacker",
            "at_start_of_turn": "remove power",
        },
        "notes": "Single turn Thorns. Removed at start of turn."
    },
    "Juggernaut": {
        "name": "Juggernaut",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_gained_block": "if block > 0, deal amount THORNS to random enemy",
        },
        "notes": "Deal damage when gaining block."
    },
    "Rupture": {
        "name": "Rupture",
        "type": PowerType.BUFF,
        "mechanics": {
            "was_hp_lost": "if damage from self (card), gain amount Strength",
        },
        "notes": "Gain Strength from self-damage cards."
    },
    "Flex": {
        "name": "Lose Strength",
        "type": PowerType.DEBUFF,
        "mechanics": {
            "at_end_of_turn": "apply -amount Strength, then remove",
        },
        "notes": "Flex downside. Lose Strength at end of turn."
    },
    "LoseDexterity": {
        "name": "Lose Dexterity",
        "type": PowerType.DEBUFF,
        "mechanics": {
            "at_end_of_turn": "apply -amount Dexterity, then remove",
        },
        "notes": "Duality downside. Lose Dexterity at end of turn."
    },
    "Evolve": {
        "name": "Evolve",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_card_draw": "if STATUS card, draw amount cards",
        },
        "notes": "Draw when drawing Status cards."
    },
    "Dark Embrace": {
        "name": "Dark Embrace",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_exhaust": "draw amount cards",
        },
        "notes": "Draw when exhausting cards."
    },

    # =========== SILENT POWERS ===========
    "Envenom": {
        "name": "Envenom",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_attack": "if damage > 0 NORMAL, apply amount Poison to target",
        },
        "notes": "Apply Poison on unblocked damage."
    },
    "Noxious Fumes": {
        "name": "Noxious Fumes",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_start_of_turn_post_draw": "apply amount Poison to all enemies",
        },
        "notes": "Apply Poison to all each turn."
    },
    "Wraith Form v2": {
        "name": "Wraith Form",
        "type": PowerType.DEBUFF,
        "can_go_negative": True,
        "mechanics": {
            "at_end_of_turn": "apply amount Dexterity (negative)",
        },
        "notes": "Lose Dexterity each turn. Amount is negative."
    },
    "Sadistic": {
        "name": "Sadistic Nature",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_apply_power": "if DEBUFF to enemy (not Shackled), deal amount THORNS",
        },
        "notes": "Deal damage when applying debuffs."
    },
    "Accuracy": {
        "name": "Accuracy",
        "type": PowerType.BUFF,
        "mechanics": {
            "passive": "Shivs deal +amount damage",
        },
        "notes": "Increases Shiv damage."
    },
    "Infinite Blades": {
        "name": "Infinite Blades",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_start_of_turn": "add amount Shiv(s) to hand",
        },
        "notes": "Add Shivs each turn."
    },

    # =========== DEFECT POWERS ===========
    "Bias": {
        "name": "Bias",
        "type": PowerType.DEBUFF,
        "mechanics": {
            "at_start_of_turn": "apply -amount Focus",
        },
        "notes": "Lose Focus each turn."
    },
    "Creative AI": {
        "name": "Creative AI",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_start_of_turn": "add random Power card to hand",
        },
        "notes": "Generate random Power cards."
    },
    "Heatsink": {
        "name": "Heatsink",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_use_card": "if POWER card, draw amount cards",
        },
        "notes": "Draw when playing Powers."
    },
    "Storm": {
        "name": "Storm",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_use_card": "if POWER card, channel Lightning",
        },
        "notes": "Channel Lightning when playing Powers."
    },
    "Static Discharge": {
        "name": "Static Discharge",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_attacked": "if took damage, channel amount Lightning",
        },
        "notes": "Channel Lightning when hit."
    },
    "Electro": {
        "name": "Electro",
        "type": PowerType.BUFF,
        "stacks": False,
        "mechanics": {
            "passive": "Lightning hits ALL enemies",
        },
        "notes": "Lightning orbs target all enemies."
    },

    # =========== COLORLESS/SPECIAL POWERS ===========
    "Panache": {
        "name": "Panache",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_use_card": "decrement counter; at 0, deal damage to all, reset to 5",
            "at_start_of_turn": "reset counter to 5",
        },
        "notes": "Every 5 cards, deal damage. Damage stacks, counter resets."
    },
    "Double Tap": {
        "name": "Double Tap",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_use_card": "if ATTACK, play again, decrement; remove at 0",
        },
        "notes": "Next Attack(s) played twice."
    },
    "Burst": {
        "name": "Burst",
        "type": PowerType.BUFF,
        "mechanics": {
            "on_use_card": "if SKILL, play again, decrement; remove at 0",
        },
        "notes": "Next Skill(s) played twice."
    },
    "Echo Form": {
        "name": "Echo Form",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_start_of_turn": "first card each turn plays twice",
        },
        "notes": "Defect. First card each turn played twice."
    },
    "Retain Cards": {
        "name": "Retain Cards",
        "type": PowerType.BUFF,
        "mechanics": {
            "at_end_of_turn": "retain amount cards from hand",
        },
        "notes": "Well-Laid Plans effect."
    },
    "Equilibrium": {
        "name": "Equilibrium",
        "type": PowerType.BUFF,
        "mechanics": {
            "passive": "retain entire hand at end of turn",
            "at_end_of_round": "decrement; remove at 0",
        },
        "notes": "Retain all cards (duration-based)."
    },
}


# =============================================================================
# MULTIPLIER CONSTANTS (from decompiled source)
# =============================================================================

# Weak - reduces NORMAL damage dealt
WEAK_MULTIPLIER = 0.75  # Standard
WEAK_MULTIPLIER_PAPER_CRANE = 0.60  # With Paper Crane relic (player vs weakened enemy)

# Vulnerable - increases NORMAL damage received
VULNERABLE_MULTIPLIER = 1.50  # Standard
VULNERABLE_MULTIPLIER_ODD_MUSHROOM = 1.25  # Player with Odd Mushroom
VULNERABLE_MULTIPLIER_PAPER_FROG = 1.75  # Enemy vs player with Paper Frog

# Frail - reduces block from cards
FRAIL_MULTIPLIER = 0.75

# Flight - enemy takes less damage
FLIGHT_MULTIPLIER = 0.50

# Lock-On - orbs deal more damage
LOCKON_MULTIPLIER = 1.50


# =============================================================================
# POWER FACTORY FUNCTIONS
# =============================================================================

def create_power(
    power_id: str,
    amount: int = 1,
    is_source_monster: bool = False
) -> Power:
    """
    Create a power instance by ID.

    Args:
        power_id: Exact ID from POWER_DATA (e.g., "Strength", "Weakened")
        amount: Stack amount
        is_source_monster: For debuffs, if True, skip first decrement (just_applied)

    Returns:
        Power instance
    """
    data = POWER_DATA.get(power_id)
    if not data:
        # Unknown power - create generic
        return Power(
            id=power_id,
            name=power_id,
            power_type=PowerType.BUFF,
            amount=amount,
        )

    # Determine just_applied flag:
    # - Some powers (Draw Reduction, Intangible) always start with justApplied=True
    # - Some powers (Lock-On, Double Damage) use justApplied when monster-applied (uses_just_applied)
    # - Standard debuffs (Weak, Vuln, Frail) use justApplied when monster-applied
    if data.get("always_just_applied", False):
        just_applied = True
    elif is_source_monster and data.get("uses_just_applied", False):
        # Powers that explicitly opt-in to justApplied behavior when monster-applied
        just_applied = True
    elif is_source_monster and data.get("type") == PowerType.DEBUFF:
        # Standard debuff behavior: skip first decrement when applied by monster
        just_applied = True
    else:
        just_applied = False

    return Power(
        id=power_id,
        name=data.get("name", power_id),
        power_type=data.get("type", PowerType.BUFF),
        amount=amount,
        is_turn_based=data.get("is_turn_based", False),
        can_go_negative=data.get("can_go_negative", False),
        stacks=data.get("stacks", True),
        priority=data.get("priority", 5),
        just_applied=just_applied,
        max_amount=data.get("max_amount", 999),
        min_amount=data.get("min_amount", -999),
    )


# Convenience factory functions for common powers
def create_strength(amount: int) -> Power:
    return create_power("Strength", amount)


def create_dexterity(amount: int) -> Power:
    return create_power("Dexterity", amount)


def create_weak(amount: int, is_source_monster: bool = False) -> Power:
    return create_power("Weakened", amount, is_source_monster)


def create_vulnerable(amount: int, is_source_monster: bool = False) -> Power:
    return create_power("Vulnerable", amount, is_source_monster)


def create_frail(amount: int, is_source_monster: bool = False) -> Power:
    return create_power("Frail", amount, is_source_monster)


def create_poison(amount: int) -> Power:
    return create_power("Poison", amount)


def create_artifact(amount: int) -> Power:
    return create_power("Artifact", amount)


def create_intangible(amount: int, is_monster: bool = False) -> Power:
    return create_power("Intangible", amount, is_monster)


def create_vigor(amount: int) -> Power:
    return create_power("Vigor", amount)


def create_mantra(amount: int) -> Power:
    return create_power("Mantra", amount)


# =============================================================================
# POWER MANAGER
# =============================================================================

@dataclass
class PowerManager:
    """
    Manages all powers on a creature (player or enemy).
    Handles stacking, removal, and damage/block calculation.
    """
    powers: Dict[str, Power] = field(default_factory=dict)

    # Relic flags for modifier calculations
    has_paper_crane: bool = False  # Enemy weak does 40% less
    has_odd_mushroom: bool = False  # Player vuln only 25% more
    has_paper_frog: bool = False  # Enemy vuln 75% more

    def add_power(self, power: Power) -> bool:
        """
        Add or stack a power.

        Returns:
            True if power was added/stacked successfully,
            False if blocked by Artifact
        """
        # Check Artifact for debuffs
        if power.power_type == PowerType.DEBUFF and self.has_power("Artifact"):
            artifact = self.powers["Artifact"]
            if artifact.amount > 0:
                artifact.reduce(1)
                if artifact.should_remove():
                    del self.powers["Artifact"]
                return False  # Debuff blocked

        if power.id in self.powers:
            self.powers[power.id].stack(power.amount)
            # Remove if at 0 and can't go negative
            if self.powers[power.id].should_remove():
                del self.powers[power.id]
        else:
            self.powers[power.id] = power
        return True

    def remove_power(self, power_id: str) -> Optional[Power]:
        """Remove a power by ID."""
        return self.powers.pop(power_id, None)

    def reduce_power(self, power_id: str, amount: int) -> bool:
        """Reduce a power's amount. Returns True if removed."""
        if power_id not in self.powers:
            return False
        if self.powers[power_id].reduce(amount):
            del self.powers[power_id]
            return True
        return False

    def get_power(self, power_id: str) -> Optional[Power]:
        """Get a power by ID."""
        return self.powers.get(power_id)

    def has_power(self, power_id: str) -> bool:
        """Check if creature has a power."""
        return power_id in self.powers

    def get_amount(self, power_id: str) -> int:
        """Get power amount, or 0 if not present."""
        power = self.powers.get(power_id)
        return power.amount if power else 0

    # === Convenience accessors ===

    def get_strength(self) -> int:
        return self.get_amount("Strength")

    def get_dexterity(self) -> int:
        return self.get_amount("Dexterity")

    def get_focus(self) -> int:
        return self.get_amount("Focus")

    def is_weak(self) -> bool:
        return self.has_power("Weakened")

    def is_vulnerable(self) -> bool:
        return self.has_power("Vulnerable")

    def is_frail(self) -> bool:
        return self.has_power("Frail")

    def is_intangible(self) -> bool:
        return self.has_power("Intangible")

    def has_barricade(self) -> bool:
        return self.has_power("Barricade") or self.has_power("Blur")

    def has_artifact(self) -> bool:
        return self.get_amount("Artifact") > 0

    # === Damage Calculation ===

    def calculate_damage_dealt(
        self,
        base_damage: int,
        target_is_player: bool = False,
    ) -> float:
        """
        Calculate outgoing NORMAL damage after attacker powers.
        Applies: Strength, Vigor, Weak, Double Damage, Pen Nib

        Args:
            base_damage: Card's base damage
            target_is_player: True if attacking the player (for Paper Crane)

        Returns:
            Modified damage as float (not yet floored)
        """
        damage = float(base_damage)

        # Add Strength
        damage += self.get_strength()

        # Add Vigor (consumed separately)
        damage += self.get_amount("Vigor")

        # Pen Nib - 2x (consumed separately)
        if self.has_power("Pen Nib"):
            damage *= 2.0

        # Double Damage - 2x
        if self.has_power("Double Damage"):
            damage *= 2.0

        # Weak - 25% less (40% with Paper Crane if enemy is weak)
        if self.is_weak():
            if target_is_player and self.has_paper_crane:
                damage *= WEAK_MULTIPLIER_PAPER_CRANE
            else:
                damage *= WEAK_MULTIPLIER

        return damage

    def calculate_damage_received(
        self,
        incoming_damage: float,
        is_player: bool = True,
    ) -> int:
        """
        Calculate NORMAL damage after defender powers.
        Applies: Vulnerable, Flight, Intangible

        Args:
            incoming_damage: Damage after attacker mods and stance
            is_player: True if defender is player

        Returns:
            Final damage as int (floored)
        """
        damage = incoming_damage

        # Vulnerable - 50% more
        if self.is_vulnerable():
            if is_player and self.has_odd_mushroom:
                damage *= VULNERABLE_MULTIPLIER_ODD_MUSHROOM
            elif not is_player and self.has_paper_frog:
                damage *= VULNERABLE_MULTIPLIER_PAPER_FROG
            else:
                damage *= VULNERABLE_MULTIPLIER

        # Flight (enemies) - 50% less
        if self.has_power("Flight"):
            damage *= FLIGHT_MULTIPLIER

        # Intangible - cap at 1 (applied last)
        if self.is_intangible() and damage > 1:
            damage = 1

        return max(0, int(damage))

    def calculate_block(self, base_block: int) -> int:
        """
        Calculate block after powers.
        Applies: Dexterity (additive), Frail (multiplicative)

        Returns:
            Final block as int (floored, min 0)
        """
        block = float(base_block)

        # Add Dexterity
        block += self.get_dexterity()

        # Frail - 25% less
        if self.is_frail():
            block *= FRAIL_MULTIPLIER

        # No Block power - set to 0
        if self.has_power("NoBlockPower"):
            block = 0

        return max(0, int(block))

    # === Turn Processing ===

    def at_end_of_round(self) -> List[str]:
        """
        Process end-of-round effects.
        Decrements turn-based powers (Weak, Vuln, Frail, etc.)

        Returns:
            List of removed power IDs
        """
        removed = []
        to_remove = []

        for power_id, power in self.powers.items():
            if power.is_turn_based:
                if power.just_applied:
                    power.just_applied = False
                else:
                    power.amount -= 1
                    if power.amount <= 0:
                        to_remove.append(power_id)

        for power_id in to_remove:
            del self.powers[power_id]
            removed.append(power_id)

        return removed

    def at_start_of_turn(self) -> Dict[str, Any]:
        """
        Process start-of-turn effects.

        Returns:
            Dict of effects to apply (poison damage, power removals, etc.)
        """
        effects = {
            "poison_damage": 0,
            "removed_powers": [],
            "stance_change": None,
        }

        # Poison damage
        if self.has_power("Poison"):
            poison = self.powers["Poison"]
            effects["poison_damage"] = poison.amount
            # Poison decrements after damage

        # Remove Flame Barrier
        if self.has_power("Flame Barrier"):
            del self.powers["Flame Barrier"]
            effects["removed_powers"].append("Flame Barrier")

        # Wrath Next Turn
        if self.has_power("WrathNextTurnPower"):
            effects["stance_change"] = "Wrath"
            del self.powers["WrathNextTurnPower"]
            effects["removed_powers"].append("WrathNextTurnPower")

        return effects


# =============================================================================
# EXPORTS
# =============================================================================

__all__ = [
    # Enums
    "PowerType",
    "DamageType",
    # Core classes
    "Power",
    "PowerManager",
    # Data
    "POWER_DATA",
    # Constants
    "WEAK_MULTIPLIER",
    "WEAK_MULTIPLIER_PAPER_CRANE",
    "VULNERABLE_MULTIPLIER",
    "VULNERABLE_MULTIPLIER_ODD_MUSHROOM",
    "VULNERABLE_MULTIPLIER_PAPER_FROG",
    "FRAIL_MULTIPLIER",
    "FLIGHT_MULTIPLIER",
    "LOCKON_MULTIPLIER",
    # Factory functions
    "create_power",
    "create_strength",
    "create_dexterity",
    "create_weak",
    "create_vulnerable",
    "create_frail",
    "create_poison",
    "create_artifact",
    "create_intangible",
    "create_vigor",
    "create_mantra",
]


# =============================================================================
# TESTING
# =============================================================================

if __name__ == "__main__":
    print("=== Power System Tests ===\n")

    # Test PowerManager damage calculation
    pm = PowerManager()

    # Test Strength
    pm.add_power(create_strength(3))
    damage = pm.calculate_damage_dealt(6)
    print(f"6 base + 3 Str = {damage}")
    assert damage == 9.0

    # Test Weak
    pm.add_power(create_weak(2))
    damage = pm.calculate_damage_dealt(10)
    print(f"10 base + 3 Str - Weak = {damage} (should be (10+3)*0.75 = 9.75)")
    assert damage == 9.75

    # Test Vulnerable on defender
    defender = PowerManager()
    defender.add_power(create_vulnerable(1))
    final = defender.calculate_damage_received(10.0)
    print(f"10 damage vs Vulnerable = {final} (should be 15)")
    assert final == 15

    # Test Intangible
    defender.add_power(create_intangible(1))
    final = defender.calculate_damage_received(100.0)
    print(f"100 damage vs Intangible = {final} (should be 1)")
    assert final == 1

    # Test block with Dexterity
    pm2 = PowerManager()
    pm2.add_power(create_dexterity(2))
    block = pm2.calculate_block(5)
    print(f"5 block + 2 Dex = {block}")
    assert block == 7

    # Test Frail
    pm2.add_power(create_frail(1))
    block = pm2.calculate_block(8)
    print(f"8 block + 2 Dex - Frail = {block} (should be (8+2)*0.75 = 7)")
    assert block == 7  # (8+2) * 0.75 = 7.5 -> 7

    # Test Artifact blocking debuff
    pm3 = PowerManager()
    pm3.add_power(create_artifact(2))
    result = pm3.add_power(create_weak(1))
    print(f"Weak blocked by Artifact: {not result}")
    assert result == False
    assert pm3.get_amount("Artifact") == 1
    assert not pm3.is_weak()

    # Test Vigor
    pm4 = PowerManager()
    pm4.add_power(create_vigor(5))
    damage = pm4.calculate_damage_dealt(6)
    print(f"6 base + 5 Vigor = {damage}")
    assert damage == 11.0

    print("\n=== All power tests passed! ===")
    print(f"\nTotal powers in registry: {len(POWER_DATA)}")
