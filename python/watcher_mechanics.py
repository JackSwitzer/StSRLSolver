"""
Comprehensive Watcher Mechanics for Slay the Spire

Full implementation of all Watcher-specific mechanics including:
- Stances (Neutral, Calm, Wrath, Divinity)
- Mantra accumulation and Divinity entry
- Retain mechanics
- Scry mechanics
- Infinite combo detection
- Card synergies and archetype scoring

Based on wiki research, Baalorlord strategies, and Lifecoach's 52-win streak patterns.
"""

import numpy as np
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Tuple, Set
from enum import Enum, auto
try:
    from .ev_calculator import (
        Stance, PlayerState, MonsterState, CardState, CombatState,
        calculate_player_attack_damage, calculate_total_incoming_damage,
        STANCE_DAMAGE_MULT, STANCE_DAMAGE_RECEIVED_MULT
    )
except ImportError:
    from ev_calculator import (
        Stance, PlayerState, MonsterState, CardState, CombatState,
        calculate_player_attack_damage, calculate_total_incoming_damage,
        STANCE_DAMAGE_MULT, STANCE_DAMAGE_RECEIVED_MULT
    )

# ============ WATCHER CARD DATABASE ============

class CardRarity(Enum):
    STARTER = auto()
    COMMON = auto()
    UNCOMMON = auto()
    RARE = auto()
    SPECIAL = auto()  # Generated cards (Smite, Insight, etc.)

class CardArchetype(Enum):
    STANCE_DANCE = auto()  # Rushdown + stance cycling
    DIVINITY = auto()      # Mantra accumulation
    SCRY = auto()          # Deck filtering
    RETAIN = auto()        # Hand management
    PRESSURE_POINTS = auto()  # Mark stacking
    GENERATED = auto()     # Battle Hymn, Alpha chain
    CALM_DEFENSE = auto()  # Like Water, block focus

@dataclass
class WatcherCard:
    """Complete Watcher card definition."""
    id: str
    name: str
    rarity: CardRarity
    card_type: str  # ATTACK, SKILL, POWER
    cost: int
    cost_upgraded: int
    base_damage: int = 0
    upgraded_damage: int = 0
    base_block: int = 0
    upgraded_block: int = 0

    # Mechanics
    enters_stance: Optional[Stance] = None
    exits_stance: bool = False
    exhausts: bool = False
    retains: bool = False
    innate: bool = False
    ethereal: bool = False

    # Special effects
    mantra_gain: int = 0
    mantra_gain_upgraded: int = 0
    scry_amount: int = 0
    scry_amount_upgraded: int = 0
    draw_cards: int = 0
    draw_cards_upgraded: int = 0

    # Multi-hit
    hits: int = 1
    hits_upgraded: int = 1

    # Synergy tags
    archetypes: List[CardArchetype] = field(default_factory=list)

    # Tier (from Baalorlord/community consensus, 1=S-tier, 5=F-tier)
    tier: int = 3
    tier_notes: str = ""

    # Generates cards
    generates_card: Optional[str] = None

    # Special mechanics
    returns_on_stance_change: bool = False
    damage_scales_with_mantra: bool = False
    cost_decreases_on_retain: bool = False
    damage_increases_on_retain: bool = False
    block_increases_on_retain: bool = False

# Complete Watcher card database
WATCHER_CARDS: Dict[str, WatcherCard] = {}

def _init_watcher_cards():
    """Initialize the complete Watcher card database."""
    cards = [
        # ========== STARTER CARDS ==========
        WatcherCard(
            id="Strike_P", name="Strike", rarity=CardRarity.STARTER,
            card_type="ATTACK", cost=1, cost_upgraded=1,
            base_damage=6, upgraded_damage=9,
            tier=5, tier_notes="Remove ASAP"
        ),
        WatcherCard(
            id="Defend_P", name="Defend", rarity=CardRarity.STARTER,
            card_type="SKILL", cost=1, cost_upgraded=1,
            base_block=5, upgraded_block=8,
            tier=5, tier_notes="Remove after Strikes"
        ),
        WatcherCard(
            id="Eruption", name="Eruption", rarity=CardRarity.STARTER,
            card_type="ATTACK", cost=2, cost_upgraded=1,
            base_damage=9, upgraded_damage=9,
            enters_stance=Stance.WRATH,
            archetypes=[CardArchetype.STANCE_DANCE],
            tier=2, tier_notes="Upgrade priority for cost reduction"
        ),
        WatcherCard(
            id="Vigilance", name="Vigilance", rarity=CardRarity.STARTER,
            card_type="SKILL", cost=2, cost_upgraded=2,
            base_block=8, upgraded_block=12,
            enters_stance=Stance.CALM,
            archetypes=[CardArchetype.STANCE_DANCE, CardArchetype.CALM_DEFENSE],
            tier=3, tier_notes="Decent Calm entry"
        ),

        # ========== COMMON ATTACKS ==========
        WatcherCard(
            id="BowlingBash", name="Bowling Bash", rarity=CardRarity.COMMON,
            card_type="ATTACK", cost=1, cost_upgraded=1,
            base_damage=7, upgraded_damage=10,
            tier=3, tier_notes="Damage per enemy"
        ),
        WatcherCard(
            id="Consecrate", name="Consecrate", rarity=CardRarity.COMMON,
            card_type="ATTACK", cost=0, cost_upgraded=0,
            base_damage=5, upgraded_damage=8,
            tier=3, tier_notes="Free AoE, good for infinite"
        ),
        WatcherCard(
            id="CrushJoints", name="Crush Joints", rarity=CardRarity.COMMON,
            card_type="ATTACK", cost=1, cost_upgraded=1,
            base_damage=8, upgraded_damage=10,
            tier=3, tier_notes="Vuln if previous was Skill"
        ),
        WatcherCard(
            id="EmptyFist", name="Empty Fist", rarity=CardRarity.COMMON,
            card_type="ATTACK", cost=1, cost_upgraded=1,
            base_damage=9, upgraded_damage=14,
            exits_stance=True,
            archetypes=[CardArchetype.STANCE_DANCE],
            tier=2, tier_notes="Safe Wrath exit with damage"
        ),
        WatcherCard(
            id="FlurryOfBlows", name="Flurry of Blows", rarity=CardRarity.COMMON,
            card_type="ATTACK", cost=0, cost_upgraded=0,
            base_damage=4, upgraded_damage=6,
            returns_on_stance_change=True,
            archetypes=[CardArchetype.STANCE_DANCE],
            tier=1, tier_notes="CORE infinite enabler"
        ),
        WatcherCard(
            id="FlyingSleeves", name="Flying Sleeves", rarity=CardRarity.COMMON,
            card_type="ATTACK", cost=1, cost_upgraded=1,
            base_damage=4, upgraded_damage=6,
            hits=2, hits_upgraded=2,
            retains=True,
            archetypes=[CardArchetype.RETAIN],
            tier=3
        ),
        WatcherCard(
            id="JustLucky", name="Just Lucky", rarity=CardRarity.COMMON,
            card_type="ATTACK", cost=0, cost_upgraded=0,
            base_damage=3, upgraded_damage=4,
            base_block=2, upgraded_block=3,
            scry_amount=1, scry_amount_upgraded=2,
            archetypes=[CardArchetype.SCRY],
            tier=3, tier_notes="Free scry"
        ),
        WatcherCard(
            id="CutThroughFate", name="Cut Through Fate", rarity=CardRarity.COMMON,
            card_type="ATTACK", cost=1, cost_upgraded=1,
            base_damage=7, upgraded_damage=9,
            scry_amount=2, scry_amount_upgraded=3,
            draw_cards=1, draw_cards_upgraded=1,
            archetypes=[CardArchetype.SCRY],
            tier=1, tier_notes="S-tier, fits every deck"
        ),

        # ========== COMMON SKILLS ==========
        WatcherCard(
            id="Crescendo", name="Crescendo", rarity=CardRarity.COMMON,
            card_type="SKILL", cost=1, cost_upgraded=0,
            enters_stance=Stance.WRATH,
            exhausts=True, retains=True,
            archetypes=[CardArchetype.STANCE_DANCE],
            tier=2, tier_notes="Cheap Wrath entry"
        ),
        WatcherCard(
            id="EmptyBody", name="Empty Body", rarity=CardRarity.COMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            base_block=7, upgraded_block=10,
            exits_stance=True,
            archetypes=[CardArchetype.STANCE_DANCE],
            tier=2, tier_notes="Safe stance exit with block"
        ),
        WatcherCard(
            id="Evaluate", name="Evaluate", rarity=CardRarity.COMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            base_block=6, upgraded_block=10,
            generates_card="Insight",
            tier=3
        ),
        WatcherCard(
            id="Halt", name="Halt", rarity=CardRarity.COMMON,
            card_type="SKILL", cost=0, cost_upgraded=0,
            base_block=3, upgraded_block=4,
            tier=3, tier_notes="+9/14 in Wrath"
        ),
        WatcherCard(
            id="Protect", name="Protect", rarity=CardRarity.COMMON,
            card_type="SKILL", cost=2, cost_upgraded=2,
            base_block=12, upgraded_block=16,
            retains=True,
            archetypes=[CardArchetype.RETAIN],
            tier=3
        ),
        WatcherCard(
            id="ThirdEye", name="Third Eye", rarity=CardRarity.COMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            base_block=7, upgraded_block=9,
            scry_amount=3, scry_amount_upgraded=5,
            archetypes=[CardArchetype.SCRY],
            tier=2, tier_notes="Good block + scry"
        ),
        WatcherCard(
            id="Tranquility", name="Tranquility", rarity=CardRarity.COMMON,
            card_type="SKILL", cost=1, cost_upgraded=0,
            enters_stance=Stance.CALM,
            exhausts=True, retains=True,
            archetypes=[CardArchetype.STANCE_DANCE, CardArchetype.CALM_DEFENSE],
            tier=3
        ),

        # ========== UNCOMMON ATTACKS ==========
        WatcherCard(
            id="CarveReality", name="Carve Reality", rarity=CardRarity.UNCOMMON,
            card_type="ATTACK", cost=1, cost_upgraded=1,
            base_damage=6, upgraded_damage=10,
            generates_card="Smite",
            archetypes=[CardArchetype.GENERATED],
            tier=3
        ),
        WatcherCard(
            id="FearNoEvil", name="Fear No Evil", rarity=CardRarity.UNCOMMON,
            card_type="ATTACK", cost=1, cost_upgraded=1,
            base_damage=8, upgraded_damage=11,
            tier=2, tier_notes="Enters Calm if enemy attacking"
        ),
        WatcherCard(
            id="ReachHeaven", name="Reach Heaven", rarity=CardRarity.UNCOMMON,
            card_type="ATTACK", cost=2, cost_upgraded=2,
            base_damage=10, upgraded_damage=15,
            generates_card="ThroughViolence",
            archetypes=[CardArchetype.GENERATED, CardArchetype.RETAIN],
            tier=3
        ),
        WatcherCard(
            id="SashWhip", name="Sash Whip", rarity=CardRarity.UNCOMMON,
            card_type="ATTACK", cost=1, cost_upgraded=1,
            base_damage=8, upgraded_damage=10,
            tier=3, tier_notes="Weak if previous was Attack"
        ),
        WatcherCard(
            id="SignatureMove", name="Signature Move", rarity=CardRarity.UNCOMMON,
            card_type="ATTACK", cost=2, cost_upgraded=2,
            base_damage=30, upgraded_damage=40,
            tier=2, tier_notes="Only playable if only Attack in hand"
        ),
        WatcherCard(
            id="Tantrum", name="Tantrum", rarity=CardRarity.UNCOMMON,
            card_type="ATTACK", cost=1, cost_upgraded=1,
            base_damage=3, upgraded_damage=3,
            hits=3, hits_upgraded=4,
            enters_stance=Stance.WRATH,
            archetypes=[CardArchetype.STANCE_DANCE],
            tier=1, tier_notes="CORE stance dance, returns to deck"
        ),
        WatcherCard(
            id="Weave", name="Weave", rarity=CardRarity.UNCOMMON,
            card_type="ATTACK", cost=0, cost_upgraded=0,
            base_damage=4, upgraded_damage=6,
            archetypes=[CardArchetype.SCRY],
            tier=2, tier_notes="Returns from discard on Scry"
        ),
        WatcherCard(
            id="WheelKick", name="Wheel Kick", rarity=CardRarity.UNCOMMON,
            card_type="ATTACK", cost=2, cost_upgraded=2,
            base_damage=15, upgraded_damage=20,
            draw_cards=2, draw_cards_upgraded=2,
            tier=2
        ),
        WatcherCard(
            id="WindmillStrike", name="Windmill Strike", rarity=CardRarity.UNCOMMON,
            card_type="ATTACK", cost=2, cost_upgraded=2,
            base_damage=7, upgraded_damage=10,
            retains=True,
            damage_increases_on_retain=True,
            archetypes=[CardArchetype.RETAIN],
            tier=3, tier_notes="+4/5 damage each turn retained"
        ),

        # ========== UNCOMMON SKILLS ==========
        WatcherCard(
            id="Collect", name="Collect", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=0, cost_upgraded=0,  # X cost
            exhausts=True,
            generates_card="Miracle",
            tier=3, tier_notes="X cost, put Miracle in hand for X turns"
        ),
        WatcherCard(
            id="DeceiveReality", name="Deceive Reality", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            base_block=4, upgraded_block=7,
            generates_card="Safety",
            archetypes=[CardArchetype.GENERATED, CardArchetype.RETAIN],
            tier=3
        ),
        WatcherCard(
            id="EmptyMind", name="Empty Mind", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            draw_cards=2, draw_cards_upgraded=3,
            exits_stance=True,
            archetypes=[CardArchetype.STANCE_DANCE],
            tier=2
        ),
        WatcherCard(
            id="ForeignInfluence", name="Foreign Influence", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=0, cost_upgraded=0,
            exhausts=True,
            tier=4, tier_notes="Choose 1 of 3 random Attacks"
        ),
        WatcherCard(
            id="Indignation", name="Indignation", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            enters_stance=Stance.WRATH,
            archetypes=[CardArchetype.STANCE_DANCE],
            tier=2, tier_notes="If in Wrath, apply Vuln to ALL"
        ),
        WatcherCard(
            id="InnerPeace", name="Inner Peace", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            draw_cards=3, draw_cards_upgraded=4,
            enters_stance=Stance.CALM,
            archetypes=[CardArchetype.STANCE_DANCE, CardArchetype.CALM_DEFENSE],
            tier=1, tier_notes="Draw if in Calm, else enter Calm"
        ),
        WatcherCard(
            id="Meditate", name="Meditate", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            enters_stance=Stance.CALM,
            archetypes=[CardArchetype.STANCE_DANCE, CardArchetype.RETAIN],
            tier=1, tier_notes="Retrieve cards from discard, Retain them"
        ),
        WatcherCard(
            id="Perseverance", name="Perseverance", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            base_block=5, upgraded_block=7,
            retains=True,
            block_increases_on_retain=True,
            archetypes=[CardArchetype.RETAIN],
            tier=3, tier_notes="+2 block each turn retained"
        ),
        WatcherCard(
            id="Pray", name="Pray", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            mantra_gain=3, mantra_gain_upgraded=4,
            generates_card="Insight",
            archetypes=[CardArchetype.DIVINITY],
            tier=3
        ),
        WatcherCard(
            id="Prostrate", name="Prostrate", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=0, cost_upgraded=0,
            base_block=4, upgraded_block=4,
            mantra_gain=2, mantra_gain_upgraded=3,
            archetypes=[CardArchetype.DIVINITY],
            tier=3
        ),
        WatcherCard(
            id="Sanctity", name="Sanctity", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            base_block=6, upgraded_block=9,
            draw_cards=2, draw_cards_upgraded=2,
            tier=3, tier_notes="Draw if previous was Skill"
        ),
        WatcherCard(
            id="SimmeringSFury", name="Simmering Fury", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            tier=2, tier_notes="Next turn: +2 energy, draw 2"
        ),
        WatcherCard(
            id="Swivel", name="Swivel", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=2, cost_upgraded=2,
            base_block=8, upgraded_block=11,
            tier=4, tier_notes="Next Attack deals double"
        ),
        WatcherCard(
            id="TalkToTheHand", name="Talk to the Hand", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            exhausts=True,
            tier=1, tier_notes="Apply Block Closed, gain 2/3 block per Attack"
        ),
        WatcherCard(
            id="Wallop", name="Wallop", rarity=CardRarity.UNCOMMON,
            card_type="ATTACK", cost=2, cost_upgraded=2,
            base_damage=9, upgraded_damage=12,
            tier=3, tier_notes="Gain block = unblocked damage"
        ),
        WatcherCard(
            id="WaveOfTheHand", name="Wave of the Hand", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            archetypes=[CardArchetype.CALM_DEFENSE],
            tier=3, tier_notes="Apply Weak whenever gaining block"
        ),
        WatcherCard(
            id="Worship", name="Worship", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=2, cost_upgraded=2,
            mantra_gain=5, mantra_gain_upgraded=5,
            retains=False,  # Only upgraded version retains
            archetypes=[CardArchetype.DIVINITY],
            tier=3
        ),
        WatcherCard(
            id="Wreath of Flame", name="Wreath of Flame", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            tier=3, tier_notes="Next Attack deals +5/8 damage"
        ),

        # ========== RARE ATTACKS ==========
        WatcherCard(
            id="Brilliance", name="Brilliance", rarity=CardRarity.RARE,
            card_type="ATTACK", cost=1, cost_upgraded=1,
            base_damage=12, upgraded_damage=16,
            damage_scales_with_mantra=True,
            archetypes=[CardArchetype.DIVINITY],
            tier=2, tier_notes="+1 damage per Mantra gained this fight"
        ),
        WatcherCard(
            id="LessonLearned", name="Lesson Learned", rarity=CardRarity.RARE,
            card_type="ATTACK", cost=2, cost_upgraded=2,
            base_damage=10, upgraded_damage=13,
            exhausts=True,
            tier=1, tier_notes="If Fatal, upgrade random card in deck"
        ),
        WatcherCard(
            id="Ragnarok", name="Ragnarok", rarity=CardRarity.RARE,
            card_type="ATTACK", cost=3, cost_upgraded=3,
            base_damage=5, upgraded_damage=6,
            hits=5, hits_upgraded=6,
            tier=2, tier_notes="Hits random enemies"
        ),

        # ========== RARE SKILLS ==========
        WatcherCard(
            id="Alpha", name="Alpha", rarity=CardRarity.RARE,
            card_type="SKILL", cost=1, cost_upgraded=1,
            exhausts=True, innate=False,  # Upgraded is Innate
            generates_card="Beta",
            archetypes=[CardArchetype.GENERATED],
            tier=2, tier_notes="Shuffle Beta into deck"
        ),
        WatcherCard(
            id="Blasphemy", name="Blasphemy", rarity=CardRarity.RARE,
            card_type="SKILL", cost=1, cost_upgraded=1,
            enters_stance=Stance.DIVINITY,
            exhausts=True, retains=False,  # Upgraded retains
            archetypes=[CardArchetype.DIVINITY],
            tier=1, tier_notes="Die at start of next turn - LETHAL OR DEATH"
        ),
        WatcherCard(
            id="ConjureBlade", name="Conjure Blade", rarity=CardRarity.RARE,
            card_type="SKILL", cost=0, cost_upgraded=0,  # X cost
            exhausts=True,
            generates_card="Expunger",
            archetypes=[CardArchetype.GENERATED],
            tier=3
        ),
        WatcherCard(
            id="DeusExMachina", name="Deus Ex Machina", rarity=CardRarity.RARE,
            card_type="SKILL", cost=0, cost_upgraded=0,
            generates_card="Miracle",
            tier=3, tier_notes="Generate 2/3 Miracles when drawn, Exhaust"
        ),
        WatcherCard(
            id="Judgement", name="Judgement", rarity=CardRarity.RARE,
            card_type="SKILL", cost=1, cost_upgraded=1,
            tier=3, tier_notes="If enemy HP <= 30/40, kill it"
        ),
        WatcherCard(
            id="Omniscience", name="Omniscience", rarity=CardRarity.RARE,
            card_type="SKILL", cost=4, cost_upgraded=3,
            exhausts=True,
            tier=1, tier_notes="Play a card from draw pile TWICE for free"
        ),
        WatcherCard(
            id="Scrawl", name="Scrawl", rarity=CardRarity.RARE,
            card_type="SKILL", cost=1, cost_upgraded=0,
            exhausts=True,
            tier=1, tier_notes="Draw until hand full"
        ),
        WatcherCard(
            id="SpiritShield", name="Spirit Shield", rarity=CardRarity.RARE,
            card_type="SKILL", cost=2, cost_upgraded=2,
            archetypes=[CardArchetype.RETAIN],
            tier=2, tier_notes="3/4 block per card in hand"
        ),
        WatcherCard(
            id="Vault", name="Vault", rarity=CardRarity.RARE,
            card_type="SKILL", cost=3, cost_upgraded=2,
            exhausts=True,
            tier=1, tier_notes="Take an extra turn, end current turn"
        ),
        WatcherCard(
            id="Wish", name="Wish", rarity=CardRarity.RARE,
            card_type="SKILL", cost=3, cost_upgraded=3,
            exhausts=True,
            tier=2, tier_notes="Choose: Plated Armor, Strength, or Gold"
        ),

        # ========== UNCOMMON POWERS ==========
        WatcherCard(
            id="BattleHymn", name="Battle Hymn", rarity=CardRarity.UNCOMMON,
            card_type="POWER", cost=1, cost_upgraded=1,
            generates_card="Smite",
            innate=False,  # Upgraded is Innate
            archetypes=[CardArchetype.GENERATED],
            tier=2, tier_notes="Add Smite to hand at turn start"
        ),
        WatcherCard(
            id="Devotion", name="Devotion", rarity=CardRarity.UNCOMMON,
            card_type="POWER", cost=1, cost_upgraded=1,
            mantra_gain=2, mantra_gain_upgraded=3,
            archetypes=[CardArchetype.DIVINITY],
            tier=2, tier_notes="Gain Mantra at start of turn"
        ),
        WatcherCard(
            id="Fasting", name="Fasting", rarity=CardRarity.UNCOMMON,
            card_type="POWER", cost=2, cost_upgraded=2,
            tier=3, tier_notes="+3/4 Str, +3/4 Dex, -1 Energy/turn"
        ),
        WatcherCard(
            id="Foresight", name="Foresight", rarity=CardRarity.UNCOMMON,
            card_type="POWER", cost=1, cost_upgraded=1,
            scry_amount=3, scry_amount_upgraded=4,
            archetypes=[CardArchetype.SCRY],
            tier=2, tier_notes="Scry at start of turn"
        ),
        WatcherCard(
            id="LikeWater", name="Like Water", rarity=CardRarity.UNCOMMON,
            card_type="POWER", cost=1, cost_upgraded=1,
            archetypes=[CardArchetype.CALM_DEFENSE],
            tier=3, tier_notes="5/7 block at turn end while in Calm"
        ),
        WatcherCard(
            id="MentalFortress", name="Mental Fortress", rarity=CardRarity.UNCOMMON,
            card_type="POWER", cost=1, cost_upgraded=1,
            archetypes=[CardArchetype.STANCE_DANCE],
            tier=1, tier_notes="4/6 block on stance change - CORE"
        ),
        WatcherCard(
            id="Nirvana", name="Nirvana", rarity=CardRarity.UNCOMMON,
            card_type="POWER", cost=1, cost_upgraded=1,
            archetypes=[CardArchetype.SCRY],
            tier=3, tier_notes="3/4 block whenever you Scry"
        ),
        WatcherCard(
            id="Rushdown", name="Rushdown", rarity=CardRarity.UNCOMMON,
            card_type="POWER", cost=1, cost_upgraded=0,
            draw_cards=2, draw_cards_upgraded=2,
            archetypes=[CardArchetype.STANCE_DANCE],
            tier=1, tier_notes="CORE INFINITE ENABLER - draw 2 on Wrath entry"
        ),
        WatcherCard(
            id="Study", name="Study", rarity=CardRarity.UNCOMMON,
            card_type="POWER", cost=2, cost_upgraded=1,
            generates_card="Insight",
            archetypes=[CardArchetype.SCRY, CardArchetype.RETAIN],
            tier=3, tier_notes="Shuffle Insight into draw pile at turn end"
        ),

        # ========== RARE POWERS ==========
        WatcherCard(
            id="DevaForm", name="Deva Form", rarity=CardRarity.RARE,
            card_type="POWER", cost=3, cost_upgraded=3,
            ethereal=True,  # Base has Ethereal, upgrade removes it
            tier=1, tier_notes="Gain +1 Energy each turn (stacking)"
        ),
        WatcherCard(
            id="Establishment", name="Establishment", rarity=CardRarity.RARE,
            card_type="POWER", cost=1, cost_upgraded=1,
            innate=False,  # Upgraded is Innate
            archetypes=[CardArchetype.RETAIN],
            tier=2, tier_notes="Retained cards cost 1 less"
        ),
        WatcherCard(
            id="MasterReality", name="Master Reality", rarity=CardRarity.RARE,
            card_type="POWER", cost=1, cost_upgraded=0,
            archetypes=[CardArchetype.GENERATED],
            tier=2, tier_notes="Cards created during combat are Upgraded"
        ),

        # ========== SPECIAL/GENERATED CARDS ==========
        WatcherCard(
            id="Insight", name="Insight", rarity=CardRarity.SPECIAL,
            card_type="SKILL", cost=0, cost_upgraded=0,
            draw_cards=2, draw_cards_upgraded=3,
            retains=True, exhausts=True,
            tier=1
        ),
        WatcherCard(
            id="Miracle", name="Miracle", rarity=CardRarity.SPECIAL,
            card_type="SKILL", cost=0, cost_upgraded=0,
            retains=True, exhausts=True,
            tier=1, tier_notes="Gain 1/2 Energy"
        ),
        WatcherCard(
            id="Smite", name="Smite", rarity=CardRarity.SPECIAL,
            card_type="ATTACK", cost=1, cost_upgraded=1,
            base_damage=12, upgraded_damage=16,
            retains=True, exhausts=True,
            tier=2
        ),
        WatcherCard(
            id="Safety", name="Safety", rarity=CardRarity.SPECIAL,
            card_type="SKILL", cost=1, cost_upgraded=1,
            base_block=12, upgraded_block=16,
            retains=True, exhausts=True,
            tier=2
        ),
        WatcherCard(
            id="ThroughViolence", name="Through Violence", rarity=CardRarity.SPECIAL,
            card_type="ATTACK", cost=0, cost_upgraded=0,
            base_damage=20, upgraded_damage=30,
            retains=True, exhausts=True,
            tier=1
        ),
        WatcherCard(
            id="Expunger", name="Expunger", rarity=CardRarity.SPECIAL,
            card_type="ATTACK", cost=1, cost_upgraded=1,
            base_damage=9, upgraded_damage=15,
            tier=2, tier_notes="Hits X times"
        ),
        WatcherCard(
            id="Beta", name="Beta", rarity=CardRarity.SPECIAL,
            card_type="SKILL", cost=2, cost_upgraded=1,
            generates_card="Omega",
            tier=2
        ),
        WatcherCard(
            id="Omega", name="Omega", rarity=CardRarity.SPECIAL,
            card_type="POWER", cost=3, cost_upgraded=3,
            tier=1, tier_notes="Deal 50/60 damage to ALL at end of turn"
        ),
        WatcherCard(
            id="PressurePoints", name="Pressure Points", rarity=CardRarity.UNCOMMON,
            card_type="SKILL", cost=1, cost_upgraded=1,
            archetypes=[CardArchetype.PRESSURE_POINTS],
            tier=3, tier_notes="Apply 8/11 Mark, deal damage = Mark"
        ),
    ]

    for card in cards:
        WATCHER_CARDS[card.id] = card

_init_watcher_cards()

# ============ WATCHER RELICS ==========

@dataclass
class WatcherRelic:
    """Watcher-specific relic definition."""
    id: str
    name: str
    rarity: str  # STARTER, COMMON, UNCOMMON, RARE, BOSS, SHOP
    effect: str
    tier: int  # 1=S-tier, 5=F-tier

    # Mechanical effects
    starts_in_calm: bool = False
    extra_calm_energy: int = 0
    mantra_per_turn: int = 0
    extra_scry: int = 0
    scry_on_shuffle: int = 0
    block_per_card_in_hand: int = 0

WATCHER_RELICS: Dict[str, WatcherRelic] = {
    "PureWater": WatcherRelic(
        id="PureWater", name="Pure Water", rarity="STARTER",
        effect="Add a Miracle to hand at combat start",
        tier=3
    ),
    "HolyWater": WatcherRelic(
        id="HolyWater", name="Holy Water", rarity="BOSS",
        effect="Add 3 Miracles to hand at combat start",
        tier=1
    ),
    "Damaru": WatcherRelic(
        id="Damaru", name="Damaru", rarity="COMMON",
        effect="+1 Mantra at start of each turn",
        tier=2, mantra_per_turn=1
    ),
    "Duality": WatcherRelic(
        id="Duality", name="Duality", rarity="UNCOMMON",
        effect="Whenever you play an Attack, gain 1 temp Dexterity",
        tier=3
    ),
    "TeardropLocket": WatcherRelic(
        id="TeardropLocket", name="Teardrop Locket", rarity="UNCOMMON",
        effect="Start each combat in Calm",
        tier=2, starts_in_calm=True
    ),
    "CloakClasp": WatcherRelic(
        id="CloakClasp", name="Cloak Clasp", rarity="RARE",
        effect="At turn end, gain 1 block per card in hand",
        tier=3, block_per_card_in_hand=1
    ),
    "GoldenEye": WatcherRelic(
        id="GoldenEye", name="Golden Eye", rarity="RARE",
        effect="Whenever you Scry, Scry 2 additional cards",
        tier=2, extra_scry=2
    ),
    "VioletLotus": WatcherRelic(
        id="VioletLotus", name="Violet Lotus", rarity="BOSS",
        effect="Exiting Calm grants +1 additional Energy",
        tier=1, extra_calm_energy=1
    ),
    "Melange": WatcherRelic(
        id="Melange", name="Melange", rarity="SHOP",
        effect="Whenever you shuffle draw pile, Scry 3",
        tier=3, scry_on_shuffle=3
    ),
}

# ============ STANCE MECHANICS ============

@dataclass
class StanceState:
    """Full stance tracking for a combat."""
    current: Stance = Stance.NEUTRAL
    mantra: int = 0
    total_mantra_gained: int = 0  # For Brilliance damage scaling
    stance_changes_this_turn: int = 0

    # Relics
    has_violet_lotus: bool = False
    has_teardrop_locket: bool = False
    has_damaru: bool = False

def get_energy_from_calm_exit(stance_state: StanceState) -> int:
    """Calculate energy gained from exiting Calm."""
    if stance_state.current != Stance.CALM:
        return 0
    base = 2
    if stance_state.has_violet_lotus:
        base += 1
    return base

def get_energy_from_divinity_entry() -> int:
    """Divinity grants +3 energy on entry."""
    return 3

def process_stance_change(
    from_stance: Stance,
    to_stance: Stance,
    stance_state: StanceState,
    has_rushdown: int = 0,
    has_mental_fortress: int = 0
) -> Dict[str, int]:
    """
    Process a stance change and return effects.

    Returns dict with:
    - energy_gained
    - cards_drawn
    - block_gained
    - flurry_returns (number of Flurry of Blows to return)
    """
    effects = {
        "energy_gained": 0,
        "cards_drawn": 0,
        "block_gained": 0,
        "flurry_returns": 0,
    }

    # Exiting Calm grants energy
    if from_stance == Stance.CALM:
        effects["energy_gained"] += 2
        if stance_state.has_violet_lotus:
            effects["energy_gained"] += 1

    # Entering Divinity grants energy
    if to_stance == Stance.DIVINITY:
        effects["energy_gained"] += 3

    # Rushdown: draw on Wrath entry
    if to_stance == Stance.WRATH and has_rushdown > 0:
        effects["cards_drawn"] += 2 * has_rushdown

    # Mental Fortress: block on any stance change
    if has_mental_fortress > 0 and from_stance != to_stance:
        # Block amount is 4 base, 6 upgraded (we'll assume upgraded here)
        effects["block_gained"] += 6 * has_mental_fortress

    # Flurry of Blows returns on stance change
    if from_stance != to_stance:
        effects["flurry_returns"] = 1  # Per Flurry in discard
        stance_state.stance_changes_this_turn += 1

    stance_state.current = to_stance
    return effects

def add_mantra(amount: int, stance_state: StanceState) -> bool:
    """
    Add mantra and check for Divinity entry.

    Returns True if Divinity was triggered.
    """
    stance_state.mantra += amount
    stance_state.total_mantra_gained += amount

    if stance_state.mantra >= 10:
        stance_state.mantra = 0  # Reset on Divinity entry
        stance_state.current = Stance.DIVINITY
        return True
    return False

def calculate_brilliance_damage(base: int, total_mantra: int) -> int:
    """Calculate Brilliance damage with mantra scaling."""
    return base + total_mantra

# ============ INFINITE COMBO DETECTION ============

def can_go_infinite(deck: List[WatcherCard], relics: List[str]) -> Dict[str, any]:
    """
    Exhaustively check if a deck can go infinite.

    Rushdown Infinite requirements:
    1. Rushdown (power) - draws 2 on Wrath entry
    2. Exactly 1 Wrath entry card (non-exhausting preferred)
    3. Exactly 1 Calm entry card (non-exhausting)
    4. Deck size <= hand size (9-10 cards typical)
       - Base hand: 5
       - Rushdown: +2 on Wrath entry
       - With draw from Calm card like Inner Peace: +3-4 more
    5. Total cycle cost <= energy from Calm exit (2, or 3 with Violet Lotus)

    Key insight: You only need 1 Wrath + 1 Calm entry, not a ratio.
    The simpler the deck, the more consistent.
    """
    result = {
        "possible": False,
        "combo_type": None,
        "required_core": [],
        "all_viable_combos": [],
        "energy_available": 0,
        "deck_size": len(deck),
        "deck_size_ok": False,
        "hand_fill_analysis": {},
        "missing_for_infinite": [],
    }

    # Calculate available energy from Calm exit
    has_violet_lotus = "VioletLotus" in relics
    has_runic_pyramid = "RunicPyramid" in relics
    calm_energy = 3 if has_violet_lotus else 2
    result["energy_available"] = calm_energy

    # Hand size calculation
    base_hand_size = 5
    # Snecko Eye, etc. could modify this

    # Find Rushdown
    rushdowns = [c for c in deck if c.id == "Rushdown"]
    has_rushdown = len(rushdowns) > 0
    rushdown_draws = 2 * len(rushdowns)  # Multiple Rushdowns = more draws

    # Find all Wrath entry options (exhaustively)
    wrath_entries = []
    for c in deck:
        if c.enters_stance == Stance.WRATH:
            effective_cost = min(c.cost, c.cost_upgraded)
            wrath_entries.append({
                "card": c,
                "cost": effective_cost,
                "exhausts": c.exhausts,
                "returns_to_deck": c.id == "Tantrum",  # Tantrum shuffles back
            })

    # Find all Calm entry options (exhaustively)
    calm_entries = []
    for c in deck:
        if c.enters_stance == Stance.CALM:
            effective_cost = min(c.cost, c.cost_upgraded)
            draws_cards = c.draw_cards if hasattr(c, 'draw_cards') else 0
            calm_entries.append({
                "card": c,
                "cost": effective_cost,
                "exhausts": c.exhausts,
                "draws": draws_cards,
                "ends_turn": c.id == "Meditate",  # Meditate ends turn
            })

    # Exhaustively enumerate all viable infinite combos
    viable_combos = []
    for wrath in wrath_entries:
        for calm in calm_entries:
            total_cost = wrath["cost"] + calm["cost"]

            # Skip exhausting cards (can't loop)
            if wrath["exhausts"] and not wrath["returns_to_deck"]:
                continue
            if calm["exhausts"] and calm["card"].id != "Meditate":
                continue

            # Skip Meditate (ends turn, breaks loop)
            if calm["ends_turn"]:
                continue

            # Check energy requirement
            if total_cost <= calm_energy:
                combo = {
                    "wrath_card": wrath["card"].id,
                    "calm_card": calm["card"].id,
                    "total_cost": total_cost,
                    "energy_surplus": calm_energy - total_cost,
                    "calm_draws": calm["draws"],
                    "wrath_returns": wrath["returns_to_deck"],
                }
                viable_combos.append(combo)

    result["all_viable_combos"] = viable_combos

    # Check deck size for hand fill
    # With Rushdown: hand = 5 + 2 (from Rushdown) + calm_draws
    # Deck needs to fit in hand for consistent infinite
    if has_rushdown and viable_combos:
        best_combo = max(viable_combos, key=lambda x: x["calm_draws"])
        effective_hand_size = base_hand_size + rushdown_draws + best_combo["calm_draws"]

        # Runic Pyramid lets you keep cards
        if has_runic_pyramid:
            effective_hand_size = 10  # Pyramid hand cap

        result["hand_fill_analysis"] = {
            "base_hand": base_hand_size,
            "rushdown_draws": rushdown_draws,
            "calm_draws": best_combo["calm_draws"],
            "effective_hand_size": effective_hand_size,
            "deck_fits_in_hand": len(deck) <= effective_hand_size,
        }

        # Optimal deck size = hand size (draw entire deck each cycle)
        result["deck_size_ok"] = len(deck) <= effective_hand_size

        if result["deck_size_ok"]:
            result["possible"] = True
            result["combo_type"] = "Rushdown Infinite"
            result["required_core"] = ["Rushdown", best_combo["wrath_card"], best_combo["calm_card"]]

    # What's missing?
    if not has_rushdown:
        result["missing_for_infinite"].append("Rushdown")
    if not wrath_entries:
        result["missing_for_infinite"].append("Wrath entry (Eruption, Tantrum, Crescendo)")
    if not [c for c in calm_entries if not c["exhausts"] and not c["ends_turn"]]:
        result["missing_for_infinite"].append("Non-exhausting Calm entry (Inner Peace, Vigilance)")
    if len(deck) > 10:
        result["missing_for_infinite"].append(f"Deck too large ({len(deck)} > 10)")

    # Separate analysis: good stance cycling (not true infinite)
    flurry_count = sum(1 for c in deck if c.id == "FlurryOfBlows")
    mental_fortress_count = sum(1 for c in deck if c.id == "MentalFortress")

    result["has_good_cycling"] = False
    if viable_combos and (flurry_count >= 1 or mental_fortress_count >= 1):
        result["has_good_cycling"] = True
        result["cycling_value"] = {
            "flurry_count": flurry_count,
            "mental_fortress_count": mental_fortress_count,
            "damage_per_cycle": flurry_count * 4,  # Base Flurry damage
            "block_per_cycle": mental_fortress_count * 6,  # Upgraded MF
        }

    return result

def estimate_infinite_damage_per_cycle(deck: List[WatcherCard]) -> int:
    """Estimate damage per infinite cycle."""
    damage = 0

    # Flurry of Blows (free, returns)
    flurry = [c for c in deck if c.id == "FlurryOfBlows"]
    for f in flurry:
        damage += f.base_damage  # Plays once per stance change cycle

    # Free attacks played during cycle
    free_attacks = [c for c in deck if c.card_type == "ATTACK" and c.cost == 0]
    for a in free_attacks:
        damage += a.base_damage

    return damage

# ============ ARCHETYPE SCORING ============

def score_deck_archetype(deck: List[WatcherCard]) -> Dict[CardArchetype, float]:
    """
    Score how well a deck fits each archetype.

    Returns normalized scores (0-1) for each archetype.
    """
    scores = {arch: 0.0 for arch in CardArchetype}

    archetype_weights = {
        CardArchetype.STANCE_DANCE: 0,
        CardArchetype.DIVINITY: 0,
        CardArchetype.SCRY: 0,
        CardArchetype.RETAIN: 0,
        CardArchetype.PRESSURE_POINTS: 0,
        CardArchetype.GENERATED: 0,
        CardArchetype.CALM_DEFENSE: 0,
    }

    for card in deck:
        for arch in card.archetypes:
            # Weight by card tier (lower tier = higher weight)
            weight = (6 - card.tier) / 5.0
            archetype_weights[arch] += weight

    # Normalize
    max_weight = max(archetype_weights.values()) if archetype_weights.values() else 1
    if max_weight > 0:
        for arch in archetype_weights:
            scores[arch] = archetype_weights[arch] / max_weight

    return scores

def get_primary_archetype(deck: List[WatcherCard]) -> CardArchetype:
    """Get the primary archetype of a deck."""
    scores = score_deck_archetype(deck)
    return max(scores, key=scores.get)

def score_card_for_deck(card: WatcherCard, deck: List[WatcherCard]) -> float:
    """
    Score how well a card fits the current deck.

    Returns score from 0-10 (higher = better fit).
    """
    deck_archetypes = score_deck_archetype(deck)
    primary_arch = get_primary_archetype(deck)

    base_score = (6 - card.tier)  # Base from tier (1-5)

    # Synergy bonus
    synergy_bonus = 0
    for arch in card.archetypes:
        synergy_bonus += deck_archetypes.get(arch, 0) * 2

    # Check for specific synergies
    # Rushdown + Wrath entry
    if card.id == "Rushdown":
        wrath_entries = sum(1 for c in deck if c.enters_stance == Stance.WRATH)
        synergy_bonus += min(wrath_entries, 3)

    # Mental Fortress + stance changers
    if card.id == "MentalFortress":
        stance_changers = sum(1 for c in deck if c.enters_stance or c.exits_stance)
        synergy_bonus += min(stance_changers, 4)

    # Flurry + stance dance
    if card.id == "FlurryOfBlows":
        stance_changers = sum(1 for c in deck if c.enters_stance or c.exits_stance)
        synergy_bonus += min(stance_changers, 3)

    # Weave + scry
    if card.id == "Weave":
        scry_cards = sum(1 for c in deck if c.scry_amount > 0)
        synergy_bonus += min(scry_cards * 1.5, 4)

    # Brilliance + mantra
    if card.id == "Brilliance":
        mantra_cards = sum(c.mantra_gain for c in deck)
        synergy_bonus += min(mantra_cards * 0.5, 3)

    return min(10, base_score + synergy_bonus)

# ============ EV ADJUSTMENTS FOR WATCHER ============

def calculate_watcher_turn_ev_bonus(
    combat: CombatState,
    stance_state: StanceState,
    has_mental_fortress: int = 0,
    has_rushdown: int = 0,
    flurry_in_discard: int = 0
) -> float:
    """
    Calculate additional EV adjustments specific to Watcher mechanics.

    Returns bonus EV (positive = good).
    """
    bonus = 0.0

    # Value of potential stance cycling
    if stance_state.current == Stance.CALM:
        # Calm exit value
        energy_value = get_energy_from_calm_exit(stance_state)
        bonus += energy_value * 2  # Energy is valuable

        # Potential Wrath damage bonus
        if has_rushdown:
            bonus += 3  # Draw value

    # Mental Fortress cycling value
    if has_mental_fortress > 0:
        potential_changes = count_potential_stance_changes(combat)
        bonus += potential_changes * 6 * has_mental_fortress

    # Flurry of Blows value
    if flurry_in_discard > 0:
        potential_returns = min(flurry_in_discard, count_potential_stance_changes(combat))
        # Each Flurry return is worth its damage (in Wrath = doubled)
        flurry_damage = 4  # Base
        if stance_state.current == Stance.WRATH:
            flurry_damage *= 2
        bonus += potential_returns * flurry_damage

    return bonus

def count_potential_stance_changes(combat: CombatState) -> int:
    """Count potential stance changes from cards in hand."""
    changes = 0
    for card_state in combat.hand:
        card = WATCHER_CARDS.get(card_state.id)
        if card:
            if card.enters_stance and card.enters_stance != combat.player.stance:
                changes += 1
            if card.exits_stance:
                changes += 1
    return changes

# ============ OPTIMAL PLAY HEURISTICS ============

def should_exit_wrath_now(
    combat: CombatState,
    can_lethal: bool,
    net_damage_if_stay: int
) -> bool:
    """
    Determine if we should exit Wrath this turn.

    Lifecoach rule: Exit Wrath if enemies attacking and can't lethal.
    """
    if combat.player.stance != Stance.WRATH:
        return False

    if can_lethal:
        return False  # Stay in Wrath to finish

    # Check incoming damage
    total_incoming = calculate_total_incoming_damage(combat)
    if total_incoming == 0:
        return False  # Safe to stay

    # Exit if taking significant damage
    hp_threshold = combat.player.hp * 0.2  # 20% HP
    if net_damage_if_stay > hp_threshold:
        return True

    return False

def evaluate_blasphemy_play(combat: CombatState) -> Dict[str, any]:
    """
    Evaluate whether Blasphemy is a good play.

    Returns:
    - recommended: bool
    - reason: str
    - damage_potential: int (with Divinity 3x)
    """
    from ev_calculator import calculate_hand_potential_damage

    # Calculate damage with Divinity (3x multiplier)
    # Temporarily set stance to calculate
    original_stance = combat.player.stance
    combat.player.stance = Stance.DIVINITY
    divinity_damage = calculate_hand_potential_damage(combat)
    combat.player.stance = original_stance

    total_enemy_hp = sum(m.hp + m.block for m in combat.monsters if m.hp > 0)

    can_lethal = divinity_damage >= total_enemy_hp

    return {
        "recommended": can_lethal,
        "reason": "Can lethal all enemies" if can_lethal else "Cannot lethal - DEATH",
        "damage_potential": divinity_damage,
        "enemy_hp_total": total_enemy_hp,
        "overkill": divinity_damage - total_enemy_hp if can_lethal else 0,
    }

# ============ STANCE RATIO ANALYSIS ============

def analyze_stance_balance(deck: List[WatcherCard]) -> Dict[str, any]:
    """
    Analyze stance cards in deck for infinite potential.

    For Rushdown Infinite, you need exactly:
    - 1 Wrath entry (non-exhausting)
    - 1 Calm entry (non-exhausting)
    - Rushdown

    More is redundant. Fewer is insufficient.
    """
    # Categorize all cards
    wrath_entries = []
    calm_entries = []
    stance_exits = []

    for c in deck:
        if c.enters_stance == Stance.WRATH:
            wrath_entries.append({
                "name": c.name,
                "id": c.id,
                "cost": c.cost,
                "cost_upgraded": c.cost_upgraded,
                "exhausts": c.exhausts,
                "returns": c.id == "Tantrum",
            })
        if c.enters_stance == Stance.CALM:
            calm_entries.append({
                "name": c.name,
                "id": c.id,
                "cost": c.cost,
                "cost_upgraded": c.cost_upgraded,
                "exhausts": c.exhausts,
                "draws": c.draw_cards,
                "ends_turn": c.id == "Meditate",
            })
        if c.exits_stance:
            stance_exits.append({
                "name": c.name,
                "id": c.id,
            })

    # Filter to usable cards for infinite
    usable_wrath = [w for w in wrath_entries if not w["exhausts"] or w["returns"]]
    usable_calm = [c for c in calm_entries if not c["exhausts"] and not c["ends_turn"]]

    has_rushdown = any(c.id == "Rushdown" for c in deck)

    result = {
        "wrath_entries": wrath_entries,
        "calm_entries": calm_entries,
        "stance_exits": stance_exits,
        "usable_wrath_for_infinite": usable_wrath,
        "usable_calm_for_infinite": usable_calm,
        "has_rushdown": has_rushdown,
        "infinite_ready": False,
        "analysis": [],
    }

    # Minimum for infinite: 1 usable Wrath, 1 usable Calm, Rushdown
    if has_rushdown and usable_wrath and usable_calm:
        result["infinite_ready"] = True
        result["analysis"].append("Infinite core complete: Rushdown + Wrath + Calm")

        # Check for redundancy
        if len(usable_wrath) > 1:
            result["analysis"].append(f"Redundant Wrath entries: {len(usable_wrath)} (1 needed)")
        if len(usable_calm) > 1:
            result["analysis"].append(f"Redundant Calm entries: {len(usable_calm)} (1 needed)")

        # Best combo for cost efficiency
        best_wrath = min(usable_wrath, key=lambda w: min(w["cost"], w["cost_upgraded"]))
        best_calm = min(usable_calm, key=lambda c: min(c["cost"], c["cost_upgraded"]))

        result["best_wrath"] = best_wrath["name"]
        result["best_calm"] = best_calm["name"]
        result["cycle_cost"] = min(best_wrath["cost"], best_wrath["cost_upgraded"]) + \
                              min(best_calm["cost"], best_calm["cost_upgraded"])

        # Check draws from Calm
        if best_calm["draws"] > 0:
            result["analysis"].append(f"Calm ({best_calm['name']}) draws {best_calm['draws']} cards")

    else:
        # What's missing
        if not has_rushdown:
            result["analysis"].append("MISSING: Rushdown (core infinite enabler)")
        if not usable_wrath:
            result["analysis"].append("MISSING: Non-exhausting Wrath entry")
            if wrath_entries:
                result["analysis"].append(f"  (Have {len(wrath_entries)} Wrath cards but they exhaust)")
        if not usable_calm:
            result["analysis"].append("MISSING: Non-exhausting Calm entry that doesn't end turn")
            if calm_entries:
                result["analysis"].append(f"  (Have {len(calm_entries)} Calm cards but issues)")

    # Stance exits are safety but not required for infinite
    if stance_exits:
        result["analysis"].append(f"Have {len(stance_exits)} stance exits (safety for Wrath)")

    return result

# ============ TESTING ============

if __name__ == "__main__":
    print("=== Watcher Mechanics Module ===\n")

    # Test card database
    print(f"Total Watcher cards: {len(WATCHER_CARDS)}")
    print(f"Total Watcher relics: {len(WATCHER_RELICS)}")

    # Show S-tier cards
    s_tier = [c for c in WATCHER_CARDS.values() if c.tier == 1]
    print(f"\nS-Tier Cards ({len(s_tier)}):")
    for card in s_tier:
        print(f"  - {card.name}: {card.tier_notes}")

    # Test stance mechanics
    print("\n=== Stance Mechanics ===")
    stance = StanceState(current=Stance.CALM, has_violet_lotus=True)
    effects = process_stance_change(
        Stance.CALM, Stance.WRATH, stance,
        has_rushdown=1, has_mental_fortress=1
    )
    print(f"Calm -> Wrath with Rushdown+MentalFortress+VioletLotus:")
    print(f"  Energy gained: {effects['energy_gained']}")
    print(f"  Cards drawn: {effects['cards_drawn']}")
    print(f"  Block gained: {effects['block_gained']}")

    # Test infinite detection
    print("\n=== Infinite Detection ===")
    test_deck = [
        WATCHER_CARDS["Rushdown"],
        WATCHER_CARDS["Eruption"],
        WATCHER_CARDS["InnerPeace"],
        WATCHER_CARDS["FlurryOfBlows"],
        WATCHER_CARDS["FlurryOfBlows"],
    ]
    infinite_check = can_go_infinite(test_deck, ["VioletLotus"])
    print(f"Deck: {[c.name for c in test_deck]}")
    print(f"Can go infinite: {infinite_check['possible']}")
    print(f"Combo type: {infinite_check['combo_type']}")

    # Test archetype scoring
    print("\n=== Archetype Scoring ===")
    scores = score_deck_archetype(test_deck)
    primary = get_primary_archetype(test_deck)
    print(f"Primary archetype: {primary.name}")
    for arch, score in sorted(scores.items(), key=lambda x: -x[1]):
        if score > 0:
            print(f"  {arch.name}: {score:.2f}")

    # Test stance balance
    print("\n=== Stance Balance ===")
    balance = analyze_stance_balance(test_deck)
    print(f"Balance score: {balance['balance_score']:.2f}")
    print(f"Calm:Wrath:Exit = {balance['calm_count']}:{balance['wrath_count']}:{balance['exit_count']}")
    print(f"Needs: {', '.join(balance['needs'])}")
