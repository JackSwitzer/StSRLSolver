"""
Slay the Spire Relic Definitions - Extracted from decompiled relic classes.

Relic structure matches AbstractRelic fields:
- relicId: Unique identifier string
- tier: STARTER, COMMON, UNCOMMON, RARE, BOSS, SHOP, SPECIAL
- counter: Tracks uses/charges (-1 = no counter, -2 = special state)

Character restrictions determined by canSpawn() method:
- Starter relics are character-locked
- Some boss relics require the starter relic (e.g., Black Blood requires Burning Blood)
- Some relics check AbstractDungeon.player.chosenClass

Hook methods (called at specific game events):
- onEquip, onUnequip: When relic is obtained/removed
- atPreBattle, atBattleStart, atBattleStartPreDraw: Combat start
- atTurnStart, atTurnStartPostDraw: Turn start
- onPlayerEndTurn: Turn end
- onUseCard, onPlayCard: When cards are played
- onExhaust: When cards are exhausted
- onManualDiscard: When cards are discarded manually
- onVictory: Combat victory
- atDamageModify: Modify outgoing damage
- onAttacked, onAttackedToChangeDamage: When receiving damage
- onLoseHp, onLoseHpLast, wasHPLost: HP loss events
- onPlayerGainBlock: Block gain events
- onPlayerHeal: Heal events
- onGainGold, onLoseGold, onSpendGold: Gold events
- onShuffle: When draw pile shuffles
- onEnterRoom, justEnteredRoom: Room transitions
- onMonsterDeath: When enemies die
- onChangeStance: When stance changes (Watcher)
- onEvokeOrb: When orbs are evoked (Defect)
- onObtainCard: When adding cards to deck
- changeNumberOfCardsInReward: Modify card reward count
- addCampfireOption, canUseCampfireOption: Rest site modifications

Counter mechanics:
- Some relics track progress with counter field
- counter = -1 means no display
- counter = -2 often means "ready to trigger"
"""

from dataclasses import dataclass, field
from typing import List, Optional, Dict, Any, Set
from enum import Enum


class RelicTier(Enum):
    """Relic tiers matching AbstractRelic.RelicTier."""
    DEPRECATED = "DEPRECATED"
    STARTER = "STARTER"
    COMMON = "COMMON"
    UNCOMMON = "UNCOMMON"
    RARE = "RARE"
    SPECIAL = "SPECIAL"
    BOSS = "BOSS"
    SHOP = "SHOP"


class PlayerClass(Enum):
    """Player classes for relic restrictions."""
    IRONCLAD = "RED"
    SILENT = "GREEN"
    DEFECT = "BLUE"
    WATCHER = "PURPLE"
    ALL = "ALL"  # No class restriction


@dataclass
class RelicEffect:
    """An effect that a relic provides."""
    hook: str  # Which hook triggers this effect
    effect_type: str  # Type of effect
    value: int = 0
    condition: Optional[str] = None  # Condition for triggering
    extra: Dict[str, Any] = field(default_factory=dict)


@dataclass
class Relic:
    """A relic definition."""
    id: str
    name: str
    tier: RelicTier
    player_class: PlayerClass = PlayerClass.ALL

    # Counter mechanics
    counter_type: Optional[str] = None  # "combat", "permanent", "uses", None
    counter_max: Optional[int] = None  # Max value before triggering
    counter_start: int = -1  # Starting counter value

    # Passive effects (always active)
    energy_bonus: int = 0  # Permanent energy increase
    max_hp_bonus: int = 0  # Permanent max HP increase
    potion_slots: int = 0  # Additional potion slots
    card_draw_bonus: int = 0  # Additional cards drawn per turn
    hand_size_bonus: int = 0  # Additional hand size
    orb_slots: int = 0  # Additional orb slots (Defect)

    # Combat modifiers
    block_loss_reduction: int = 0  # Reduces block lost at turn start
    damage_bonus_flat: int = 0  # Flat damage bonus

    # Effects by hook
    effects: List[str] = field(default_factory=list)  # List of effect descriptions

    # Spawn conditions
    requires_relic: Optional[str] = None  # Must have this relic to spawn
    act_restriction: Optional[int] = None  # Only spawns in this act or earlier

    # Special flags
    prevents_healing: bool = False
    prevents_gold_gain: bool = False
    prevents_potions: bool = False
    prevents_resting: bool = False
    prevents_smithing: bool = False
    hides_intent: bool = False  # Runic Dome

    def copy(self) -> 'Relic':
        """Create a copy of this relic."""
        return Relic(
            id=self.id, name=self.name, tier=self.tier,
            player_class=self.player_class, counter_type=self.counter_type,
            counter_max=self.counter_max, counter_start=self.counter_start,
            energy_bonus=self.energy_bonus, max_hp_bonus=self.max_hp_bonus,
            potion_slots=self.potion_slots, card_draw_bonus=self.card_draw_bonus,
            hand_size_bonus=self.hand_size_bonus, orb_slots=self.orb_slots,
            block_loss_reduction=self.block_loss_reduction,
            damage_bonus_flat=self.damage_bonus_flat,
            effects=self.effects.copy(),
            requires_relic=self.requires_relic,
            act_restriction=self.act_restriction,
            prevents_healing=self.prevents_healing,
            prevents_gold_gain=self.prevents_gold_gain,
            prevents_potions=self.prevents_potions,
            prevents_resting=self.prevents_resting,
            prevents_smithing=self.prevents_smithing,
            hides_intent=self.hides_intent,
        )


# ============================================================================
# STARTER RELICS (4 total - one per class)
# ============================================================================

BURNING_BLOOD = Relic(
    id="Burning Blood", name="Burning Blood", tier=RelicTier.STARTER,
    player_class=PlayerClass.IRONCLAD,
    effects=["onVictory: Heal 6 HP"],
)

RING_OF_THE_SNAKE = Relic(
    id="Ring of the Snake", name="Ring of the Snake", tier=RelicTier.STARTER,
    player_class=PlayerClass.SILENT,
    effects=["atBattleStart: Draw 2 additional cards"],
)

CRACKED_CORE = Relic(
    id="Cracked Core", name="Cracked Core", tier=RelicTier.STARTER,
    player_class=PlayerClass.DEFECT,
    effects=["atPreBattle: Channel 1 Lightning orb"],
)

PURE_WATER = Relic(
    id="PureWater", name="Pure Water", tier=RelicTier.STARTER,
    player_class=PlayerClass.WATCHER,
    effects=["atBattleStartPreDraw: Add 1 Miracle to hand"],
)

# ============================================================================
# COMMON RELICS (28 total)
# ============================================================================

AKABEKO = Relic(
    id="Akabeko", name="Akabeko", tier=RelicTier.COMMON,
    effects=["atBattleStart: Gain 8 Vigor (first attack deals +8 damage)"],
)

ANCHOR = Relic(
    id="Anchor", name="Anchor", tier=RelicTier.COMMON,
    effects=["atBattleStart: Gain 10 Block"],
)

ANCIENT_TEA_SET = Relic(
    id="Ancient Tea Set", name="Ancient Tea Set", tier=RelicTier.COMMON,
    counter_type="special", counter_start=-1,
    effects=["onEnterRestRoom: Set counter to -2",
             "atTurnStart (first turn, if counter=-2): Gain 2 Energy"],
)

ART_OF_WAR = Relic(
    id="Art of War", name="Art of War", tier=RelicTier.COMMON,
    effects=["atTurnStart: If no attacks played last turn, gain 1 Energy"],
)

BAG_OF_MARBLES = Relic(
    id="Bag of Marbles", name="Bag of Marbles", tier=RelicTier.COMMON,
    effects=["atBattleStart: Apply 1 Vulnerable to ALL enemies"],
)

BAG_OF_PREPARATION = Relic(
    id="Bag of Preparation", name="Bag of Preparation", tier=RelicTier.COMMON,
    effects=["atBattleStart: Draw 2 additional cards"],
)

BLOOD_VIAL = Relic(
    id="Blood Vial", name="Blood Vial", tier=RelicTier.COMMON,
    effects=["atBattleStart: Heal 2 HP"],
)

BRONZE_SCALES = Relic(
    id="Bronze Scales", name="Bronze Scales", tier=RelicTier.COMMON,
    effects=["atBattleStart: Gain 3 Thorns"],
)

CENTENNIAL_PUZZLE = Relic(
    id="Centennial Puzzle", name="Centennial Puzzle", tier=RelicTier.COMMON,
    effects=["wasHPLost (first time per combat): Draw 3 cards"],
)

CERAMIC_FISH = Relic(
    id="CeramicFish", name="Ceramic Fish", tier=RelicTier.COMMON,
    effects=["onObtainCard: Gain 9 Gold"],
)

DAMARU = Relic(
    id="Damaru", name="Damaru", tier=RelicTier.COMMON,
    player_class=PlayerClass.WATCHER,
    effects=["atTurnStart: Gain 1 Mantra"],
)

DATA_DISK = Relic(
    id="DataDisk", name="Data Disk", tier=RelicTier.COMMON,
    player_class=PlayerClass.DEFECT,
    effects=["atBattleStart: Gain 1 Focus"],
)

DREAM_CATCHER = Relic(
    id="Dream Catcher", name="Dream Catcher", tier=RelicTier.COMMON,
    effects=["onRest: Add a card to your deck"],
)

HAPPY_FLOWER = Relic(
    id="Happy Flower", name="Happy Flower", tier=RelicTier.COMMON,
    counter_type="combat", counter_max=3, counter_start=0,
    effects=["atTurnStart: Counter +1. At 3, gain 1 Energy and reset"],
)

JUZU_BRACELET = Relic(
    id="Juzu Bracelet", name="Juzu Bracelet", tier=RelicTier.COMMON,
    effects=["Prevents ? room encounters from being combats"],
)

LANTERN = Relic(
    id="Lantern", name="Lantern", tier=RelicTier.COMMON,
    effects=["atBattleStart (first turn): Gain 1 Energy"],
)

MAW_BANK = Relic(
    id="MawBank", name="Maw Bank", tier=RelicTier.COMMON,
    counter_type="permanent", counter_start=0,
    effects=["onEnterRoom (not shop): Gain 12 Gold",
             "onSpendGold: Lose this relic's effect"],
)

MEAL_TICKET = Relic(
    id="MealTicket", name="Meal Ticket", tier=RelicTier.COMMON,
    effects=["onEnterRoom (shop): Heal 15 HP"],
)

NUNCHAKU = Relic(
    id="Nunchaku", name="Nunchaku", tier=RelicTier.COMMON,
    counter_type="permanent", counter_max=10, counter_start=0,
    effects=["onUseCard (attack): Counter +1. At 10, gain 1 Energy and reset"],
)

ODDLY_SMOOTH_STONE = Relic(
    id="Oddly Smooth Stone", name="Oddly Smooth Stone", tier=RelicTier.COMMON,
    effects=["atBattleStart: Gain 1 Dexterity"],
)

OMAMORI = Relic(
    id="Omamori", name="Omamori", tier=RelicTier.COMMON,
    counter_type="uses", counter_start=2,
    effects=["Negates next 2 Curses added to deck"],
)

ORICHALCUM = Relic(
    id="Orichalcum", name="Orichalcum", tier=RelicTier.COMMON,
    effects=["onPlayerEndTurn: If no Block, gain 6 Block"],
)

PEN_NIB = Relic(
    id="Pen Nib", name="Pen Nib", tier=RelicTier.COMMON,
    counter_type="permanent", counter_max=10, counter_start=0,
    effects=["onUseCard (attack): Counter +1. At 9, next attack deals double damage"],
)

POTION_BELT = Relic(
    id="Potion Belt", name="Potion Belt", tier=RelicTier.COMMON,
    potion_slots=2,
    effects=["onEquip: Gain 2 potion slots"],
)

PRESERVED_INSECT = Relic(
    id="PreservedInsect", name="Preserved Insect", tier=RelicTier.COMMON,
    effects=["Elites have 25% less HP"],
)

REGAL_PILLOW = Relic(
    id="Regal Pillow", name="Regal Pillow", tier=RelicTier.COMMON,
    effects=["onRest: Heal 15 additional HP"],
)

SMILING_MASK = Relic(
    id="Smiling Mask", name="Smiling Mask", tier=RelicTier.COMMON,
    effects=["Shop card removal (purge) always costs 50 Gold"],
)

SNECKO_SKULL = Relic(
    id="Snake Skull", name="Snecko Skull", tier=RelicTier.COMMON,
    player_class=PlayerClass.SILENT,
    effects=["Whenever you apply Poison, apply 1 additional Poison"],
)

STRAWBERRY = Relic(
    id="Strawberry", name="Strawberry", tier=RelicTier.COMMON,
    max_hp_bonus=7,
    effects=["onEquip: Gain 7 Max HP"],
)

THE_BOOT = Relic(
    id="Boot", name="The Boot", tier=RelicTier.COMMON,
    effects=["onAttackToChangeDamage: If attack would deal <5 damage, deal 5 instead"],
)

TINY_CHEST = Relic(
    id="Tiny Chest", name="Tiny Chest", tier=RelicTier.COMMON,
    counter_type="permanent", counter_max=4, counter_start=0,
    effects=["Every 4th ? room contains a treasure chest"],
)

TOY_ORNITHOPTER = Relic(
    id="Toy Ornithopter", name="Toy Ornithopter", tier=RelicTier.COMMON,
    effects=["Whenever you use a potion, heal 5 HP"],
)

VAJRA = Relic(
    id="Vajra", name="Vajra", tier=RelicTier.COMMON,
    effects=["atBattleStart: Gain 1 Strength"],
)

WAR_PAINT = Relic(
    id="War Paint", name="War Paint", tier=RelicTier.COMMON,
    effects=["onEquip: Upgrade 2 random Skills"],
)

WHETSTONE = Relic(
    id="Whetstone", name="Whetstone", tier=RelicTier.COMMON,
    effects=["onEquip: Upgrade 2 random Attacks"],
)

RED_SKULL = Relic(
    id="Red Skull", name="Red Skull", tier=RelicTier.COMMON,
    player_class=PlayerClass.IRONCLAD,
    effects=["onBloodied (HP <= 50%): Gain 3 Strength. Lose when healed above 50%"],
)

# ============================================================================
# UNCOMMON RELICS (35 total)
# ============================================================================

BLUE_CANDLE = Relic(
    id="Blue Candle", name="Blue Candle", tier=RelicTier.UNCOMMON,
    effects=["Curse cards can be played. Playing a Curse exhausts it and deals 1 HP loss"],
)

BOTTLED_FLAME = Relic(
    id="Bottled Flame", name="Bottled Flame", tier=RelicTier.UNCOMMON,
    effects=["onEquip: Choose an Attack. It starts combat in hand (Innate)"],
)

BOTTLED_LIGHTNING = Relic(
    id="Bottled Lightning", name="Bottled Lightning", tier=RelicTier.UNCOMMON,
    effects=["onEquip: Choose a Skill. It starts combat in hand (Innate)"],
)

BOTTLED_TORNADO = Relic(
    id="Bottled Tornado", name="Bottled Tornado", tier=RelicTier.UNCOMMON,
    effects=["onEquip: Choose a Power. It starts combat in hand (Innate)"],
)

DARKSTONE_PERIAPT = Relic(
    id="Darkstone Periapt", name="Darkstone Periapt", tier=RelicTier.UNCOMMON,
    effects=["Whenever you obtain a Curse, gain 6 Max HP"],
)

ETERNAL_FEATHER = Relic(
    id="Eternal Feather", name="Eternal Feather", tier=RelicTier.UNCOMMON,
    effects=["Whenever you enter a Rest Site, heal 3 HP per 5 cards in deck"],
)

FROZEN_EGG = Relic(
    id="Frozen Egg 2", name="Frozen Egg", tier=RelicTier.UNCOMMON,
    effects=["Whenever you add a Power to your deck, it is Upgraded"],
)

GOLD_PLATED_CABLES = Relic(
    id="Cables", name="Gold-Plated Cables", tier=RelicTier.UNCOMMON,
    player_class=PlayerClass.DEFECT,
    effects=["Your rightmost Orb triggers its passive an additional time"],
)

GREMLIN_HORN = Relic(
    id="Gremlin Horn", name="Gremlin Horn", tier=RelicTier.UNCOMMON,
    effects=["onMonsterDeath: Gain 1 Energy and draw 1 card"],
)

HORN_CLEAT = Relic(
    id="HornCleat", name="Horn Cleat", tier=RelicTier.UNCOMMON,
    effects=["atTurnStart (turn 2): Gain 14 Block"],
)

INK_BOTTLE = Relic(
    id="InkBottle", name="Ink Bottle", tier=RelicTier.UNCOMMON,
    counter_type="permanent", counter_max=10, counter_start=0,
    effects=["onUseCard: Counter +1. At 10, draw 1 card and reset"],
)

KUNAI = Relic(
    id="Kunai", name="Kunai", tier=RelicTier.UNCOMMON,
    counter_type="combat", counter_max=3, counter_start=0,
    effects=["onUseCard (attack): Counter +1. At 3, gain 1 Dexterity and reset"],
)

LETTER_OPENER = Relic(
    id="Letter Opener", name="Letter Opener", tier=RelicTier.UNCOMMON,
    counter_type="combat", counter_max=3, counter_start=0,
    effects=["onUseCard (skill): Counter +1. At 3, deal 5 damage to ALL enemies and reset"],
)

MATRYOSHKA = Relic(
    id="Matryoshka", name="Matryoshka", tier=RelicTier.UNCOMMON,
    counter_type="uses", counter_start=2,
    effects=["Next 2 chests contain 2 relics"],
)

MEAT_ON_THE_BONE = Relic(
    id="Meat on the Bone", name="Meat on the Bone", tier=RelicTier.UNCOMMON,
    effects=["onVictory: If HP <= 50%, heal 12 HP"],
)

MERCURY_HOURGLASS = Relic(
    id="Mercury Hourglass", name="Mercury Hourglass", tier=RelicTier.UNCOMMON,
    effects=["atTurnStart: Deal 3 damage to ALL enemies"],
)

MOLTEN_EGG = Relic(
    id="Molten Egg 2", name="Molten Egg", tier=RelicTier.UNCOMMON,
    effects=["Whenever you add an Attack to your deck, it is Upgraded"],
)

MUMMIFIED_HAND = Relic(
    id="Mummified Hand", name="Mummified Hand", tier=RelicTier.UNCOMMON,
    effects=["onUseCard (power): A random card in hand costs 0 this turn"],
)

NINJA_SCROLL = Relic(
    id="Ninja Scroll", name="Ninja Scroll", tier=RelicTier.UNCOMMON,
    player_class=PlayerClass.SILENT,
    effects=["atBattleStartPreDraw: Add 3 Shivs to hand"],
)

ORNAMENTAL_FAN = Relic(
    id="Ornamental Fan", name="Ornamental Fan", tier=RelicTier.UNCOMMON,
    counter_type="combat", counter_max=3, counter_start=0,
    effects=["onUseCard (attack): Counter +1. At 3, gain 4 Block and reset"],
)

PANTOGRAPH = Relic(
    id="Pantograph", name="Pantograph", tier=RelicTier.UNCOMMON,
    effects=["atBattleStart (boss): Heal 25 HP"],
)

PAPER_CRANE = Relic(
    id="Paper Crane", name="Paper Crane", tier=RelicTier.UNCOMMON,
    player_class=PlayerClass.SILENT,
    effects=["Enemies with Weak deal 40% less damage instead of 25%"],
)

PAPER_FROG = Relic(
    id="Paper Frog", name="Paper Frog", tier=RelicTier.UNCOMMON,
    player_class=PlayerClass.IRONCLAD,
    effects=["Enemies with Vulnerable take 75% more damage instead of 50%"],
)

PEAR = Relic(
    id="Pear", name="Pear", tier=RelicTier.UNCOMMON,
    max_hp_bonus=10,
    effects=["onEquip: Gain 10 Max HP"],
)

QUESTION_CARD = Relic(
    id="Question Card", name="Question Card", tier=RelicTier.UNCOMMON,
    effects=["Card rewards contain 1 additional card. You may skip card rewards"],
)

SELF_FORMING_CLAY = Relic(
    id="Self Forming Clay", name="Self Forming Clay", tier=RelicTier.UNCOMMON,
    player_class=PlayerClass.IRONCLAD,
    effects=["wasHPLost: Gain 3 Block next turn"],
)

SHURIKEN = Relic(
    id="Shuriken", name="Shuriken", tier=RelicTier.UNCOMMON,
    counter_type="combat", counter_max=3, counter_start=0,
    effects=["onUseCard (attack): Counter +1. At 3, gain 1 Strength and reset"],
)

SINGING_BOWL = Relic(
    id="Singing Bowl", name="Singing Bowl", tier=RelicTier.UNCOMMON,
    effects=["When adding a card, may gain 2 Max HP instead"],
)

STRIKE_DUMMY = Relic(
    id="StrikeDummy", name="Strike Dummy", tier=RelicTier.UNCOMMON,
    effects=["Cards containing 'Strike' deal 3 additional damage"],
)

SUNDIAL = Relic(
    id="Sundial", name="Sundial", tier=RelicTier.UNCOMMON,
    counter_type="permanent", counter_max=3, counter_start=0,
    effects=["onShuffle: Counter +1. At 3, gain 2 Energy and reset"],
)

SYMBIOTIC_VIRUS = Relic(
    id="Symbiotic Virus", name="Symbiotic Virus", tier=RelicTier.UNCOMMON,
    player_class=PlayerClass.DEFECT,
    effects=["atPreBattle: Channel 1 Dark orb"],
)

TEARDROP_LOCKET = Relic(
    id="TeardropLocket", name="Teardrop Locket", tier=RelicTier.UNCOMMON,
    player_class=PlayerClass.WATCHER,
    effects=["atBattleStart: Enter Calm"],
)

THE_COURIER = Relic(
    id="The Courier", name="The Courier", tier=RelicTier.UNCOMMON,
    effects=["Shop always has card removal. 20% discount on everything"],
)

TOXIC_EGG = Relic(
    id="Toxic Egg 2", name="Toxic Egg", tier=RelicTier.UNCOMMON,
    effects=["Whenever you add a Skill to your deck, it is Upgraded"],
)

WHITE_BEAST_STATUE = Relic(
    id="White Beast Statue", name="White Beast Statue", tier=RelicTier.UNCOMMON,
    effects=["Potions always drop from combat rewards"],
)

DUALITY = Relic(
    id="Yang", name="Duality", tier=RelicTier.UNCOMMON,
    player_class=PlayerClass.WATCHER,
    effects=["onUseCard (attack): Gain 1 Dexterity this turn"],
)

DISCERNING_MONOCLE = Relic(
    id="Discerning Monocle", name="Discerning Monocle", tier=RelicTier.UNCOMMON,
    effects=["Increases chance of Rare cards in rewards"],
)

# ============================================================================
# RARE RELICS (29 total)
# ============================================================================

BIRD_FACED_URN = Relic(
    id="Bird Faced Urn", name="Bird-Faced Urn", tier=RelicTier.RARE,
    effects=["onUseCard (power): Heal 2 HP"],
)

CALIPERS = Relic(
    id="Calipers", name="Calipers", tier=RelicTier.RARE,
    block_loss_reduction=15,
    effects=["At turn start, lose 15 Block instead of all Block"],
)

CAPTAINS_WHEEL = Relic(
    id="CaptainsWheel", name="Captain's Wheel", tier=RelicTier.RARE,
    counter_type="combat", counter_max=3, counter_start=0,
    effects=["atTurnStart (turn 3): Gain 18 Block (once per combat)"],
)

CHARONS_ASHES = Relic(
    id="Charon's Ashes", name="Charon's Ashes", tier=RelicTier.RARE,
    player_class=PlayerClass.IRONCLAD,
    effects=["onExhaust: Deal 3 damage to ALL enemies"],
)

CHAMPIONS_BELT = Relic(
    id="Champion Belt", name="Champion Belt", tier=RelicTier.RARE,
    player_class=PlayerClass.IRONCLAD,
    effects=["Whenever you apply Vulnerable, also apply 1 Weak"],
)

CLOAK_CLASP = Relic(
    id="CloakClasp", name="Cloak Clasp", tier=RelicTier.RARE,
    player_class=PlayerClass.WATCHER,
    effects=["onPlayerEndTurn: Gain 1 Block per card in hand"],
)

DEAD_BRANCH = Relic(
    id="Dead Branch", name="Dead Branch", tier=RelicTier.RARE,
    effects=["onExhaust: Add a random card to hand"],
)

DU_VU_DOLL = Relic(
    id="Du-Vu Doll", name="Du-Vu Doll", tier=RelicTier.RARE,
    effects=["atBattleStart: Gain 1 Strength per Curse in deck"],
)

EMOTION_CHIP = Relic(
    id="Emotion Chip", name="Emotion Chip", tier=RelicTier.RARE,
    player_class=PlayerClass.DEFECT,
    effects=["wasHPLost (once per combat): Trigger passive of all Orbs"],
)

FOSSILIZED_HELIX = Relic(
    id="FossilizedHelix", name="Fossilized Helix", tier=RelicTier.RARE,
    effects=["onAttacked (first time per combat): Prevent all damage"],
)

GAMBLING_CHIP = Relic(
    id="Gambling Chip", name="Gambling Chip", tier=RelicTier.RARE,
    effects=["atBattleStart: Discard any cards, draw that many"],
)

GINGER = Relic(
    id="Ginger", name="Ginger", tier=RelicTier.RARE,
    effects=["You can no longer become Weakened"],
)

GIRYA = Relic(
    id="Girya", name="Girya", tier=RelicTier.RARE,
    counter_type="uses", counter_start=0,
    effects=["Can Lift at rest sites (max 3 times). Each Lift gives 1 permanent Strength"],
)

GOLDEN_EYE = Relic(
    id="GoldenEye", name="Golden Eye", tier=RelicTier.RARE,
    player_class=PlayerClass.WATCHER,
    effects=["onScry: Scry 2 additional cards"],
)

ICE_CREAM = Relic(
    id="Ice Cream", name="Ice Cream", tier=RelicTier.RARE,
    effects=["Energy is conserved between turns"],
)

INCENSE_BURNER = Relic(
    id="Incense Burner", name="Incense Burner", tier=RelicTier.RARE,
    counter_type="permanent", counter_max=6, counter_start=0,
    effects=["atTurnStart: Counter +1. At 6, gain 1 Intangible and reset"],
)

LIZARD_TAIL = Relic(
    id="Lizard Tail", name="Lizard Tail", tier=RelicTier.RARE,
    effects=["When you would die, heal to 50% HP (once per run)"],
)

MAGIC_FLOWER = Relic(
    id="Magic Flower", name="Magic Flower", tier=RelicTier.RARE,
    player_class=PlayerClass.IRONCLAD,
    effects=["Healing is 50% more effective"],
)

MANGO = Relic(
    id="Mango", name="Mango", tier=RelicTier.RARE,
    max_hp_bonus=14,
    effects=["onEquip: Gain 14 Max HP"],
)

OLD_COIN = Relic(
    id="Old Coin", name="Old Coin", tier=RelicTier.RARE,
    effects=["onEquip: Gain 300 Gold"],
)

PEACE_PIPE = Relic(
    id="Peace Pipe", name="Peace Pipe", tier=RelicTier.RARE,
    effects=["Can Toke at rest sites to remove a card"],
)

POCKETWATCH = Relic(
    id="Pocketwatch", name="Pocketwatch", tier=RelicTier.RARE,
    effects=["onPlayerEndTurn: If played 3 or fewer cards, draw 3 next turn"],
)

PRAYER_WHEEL = Relic(
    id="Prayer Wheel", name="Prayer Wheel", tier=RelicTier.RARE,
    effects=["Normal combats have 2 card rewards"],
)

SHOVEL = Relic(
    id="Shovel", name="Shovel", tier=RelicTier.RARE,
    effects=["Can Dig at rest sites for a relic"],
)

STONE_CALENDAR = Relic(
    id="StoneCalendar", name="Stone Calendar", tier=RelicTier.RARE,
    counter_type="combat", counter_start=0,
    effects=["atTurnStart: Counter +1. At turn 7+, deal 52 damage at end of turn"],
)

THE_SPECIMEN = Relic(
    id="The Specimen", name="The Specimen", tier=RelicTier.RARE,
    player_class=PlayerClass.SILENT,
    effects=["onMonsterDeath: Transfer enemy's Poison to random enemy"],
)

THREAD_AND_NEEDLE = Relic(
    id="Thread and Needle", name="Thread and Needle", tier=RelicTier.RARE,
    effects=["atBattleStart: Gain 4 Plated Armor"],
)

TINGSHA = Relic(
    id="Tingsha", name="Tingsha", tier=RelicTier.RARE,
    player_class=PlayerClass.SILENT,
    effects=["onManualDiscard: Deal 3 damage to random enemy per card"],
)

TORII = Relic(
    id="Torii", name="Torii", tier=RelicTier.RARE,
    effects=["onAttacked: If damage 2-5, reduce to 1"],
)

TOUGH_BANDAGES = Relic(
    id="Tough Bandages", name="Tough Bandages", tier=RelicTier.RARE,
    player_class=PlayerClass.SILENT,
    effects=["onManualDiscard: Gain 3 Block per card"],
)

TUNGSTEN_ROD = Relic(
    id="TungstenRod", name="Tungsten Rod", tier=RelicTier.RARE,
    effects=["onLoseHpLast: Reduce HP loss by 1"],
)

TURNIP = Relic(
    id="Turnip", name="Turnip", tier=RelicTier.RARE,
    effects=["You can no longer become Frail"],
)

UNCEASING_TOP = Relic(
    id="Unceasing Top", name="Unceasing Top", tier=RelicTier.RARE,
    effects=["When hand is empty, draw 1 card"],
)

WING_BOOTS = Relic(
    id="WingedGreaves", name="Wing Boots", tier=RelicTier.RARE,
    counter_type="uses", counter_start=3,
    effects=["Can fly over map nodes (3 times)"],
)

# ============================================================================
# BOSS RELICS (32 total)
# ============================================================================

# Boss relics that upgrade starter relics
BLACK_BLOOD = Relic(
    id="Black Blood", name="Black Blood", tier=RelicTier.BOSS,
    player_class=PlayerClass.IRONCLAD,
    requires_relic="Burning Blood",
    effects=["onVictory: Heal 12 HP (replaces Burning Blood)"],
)

RING_OF_THE_SERPENT = Relic(
    id="Ring of the Serpent", name="Ring of the Serpent", tier=RelicTier.BOSS,
    player_class=PlayerClass.SILENT,
    requires_relic="Ring of the Snake",
    hand_size_bonus=1,
    effects=["onEquip: Draw 1 additional card each turn (replaces Ring of the Snake)"],
)

FROZEN_CORE = Relic(
    id="FrozenCore", name="Frozen Core", tier=RelicTier.BOSS,
    player_class=PlayerClass.DEFECT,
    requires_relic="Cracked Core",
    effects=["onPlayerEndTurn: If empty orb slot, Channel 1 Frost (replaces Cracked Core)"],
)

HOLY_WATER = Relic(
    id="HolyWater", name="Holy Water", tier=RelicTier.BOSS,
    player_class=PlayerClass.WATCHER,
    requires_relic="PureWater",
    effects=["atBattleStartPreDraw: Add 3 Miracles to hand (replaces Pure Water)"],
)

# Energy boss relics with downsides
ASTROLABE = Relic(
    id="Astrolabe", name="Astrolabe", tier=RelicTier.BOSS,
    effects=["onEquip: Transform and Upgrade 3 cards"],
)

BLACK_STAR = Relic(
    id="Black Star", name="Black Star", tier=RelicTier.BOSS,
    effects=["Elites drop 2 relics instead of 1"],
)

BUSTED_CROWN = Relic(
    id="Busted Crown", name="Busted Crown", tier=RelicTier.BOSS,
    energy_bonus=1,
    effects=["onEquip: +1 Energy. Card rewards contain 2 fewer choices"],
)

CALLING_BELL = Relic(
    id="Calling Bell", name="Calling Bell", tier=RelicTier.BOSS,
    effects=["onEquip: Obtain 1 Curse, 1 Common, 1 Uncommon, 1 Rare relic"],
)

COFFEE_DRIPPER = Relic(
    id="Coffee Dripper", name="Coffee Dripper", tier=RelicTier.BOSS,
    energy_bonus=1,
    prevents_resting=True,
    effects=["onEquip: +1 Energy. Cannot Rest at rest sites"],
)

CURSED_KEY = Relic(
    id="Cursed Key", name="Cursed Key", tier=RelicTier.BOSS,
    energy_bonus=1,
    effects=["onEquip: +1 Energy. onChestOpen: Obtain a random Curse"],
)

ECTOPLASM = Relic(
    id="Ectoplasm", name="Ectoplasm", tier=RelicTier.BOSS,
    energy_bonus=1,
    prevents_gold_gain=True,
    act_restriction=1,  # Only in Act 1
    effects=["onEquip: +1 Energy. Cannot gain Gold"],
)

EMPTY_CAGE = Relic(
    id="Empty Cage", name="Empty Cage", tier=RelicTier.BOSS,
    effects=["onEquip: Remove 2 cards from deck"],
)

FUSION_HAMMER = Relic(
    id="Fusion Hammer", name="Fusion Hammer", tier=RelicTier.BOSS,
    energy_bonus=1,
    prevents_smithing=True,
    effects=["onEquip: +1 Energy. Cannot Smith at rest sites"],
)

HOVERING_KITE = Relic(
    id="HoveringKite", name="Hovering Kite", tier=RelicTier.BOSS,
    player_class=PlayerClass.SILENT,
    effects=["onManualDiscard (first each turn): Gain 1 Energy"],
)

INSERTER = Relic(
    id="Inserter", name="Inserter", tier=RelicTier.BOSS,
    player_class=PlayerClass.DEFECT,
    counter_type="permanent", counter_max=2, counter_start=0,
    effects=["atTurnStart: Counter +1. At 2, gain 1 Orb slot and reset"],
)

MARK_OF_PAIN = Relic(
    id="Mark of Pain", name="Mark of Pain", tier=RelicTier.BOSS,
    player_class=PlayerClass.IRONCLAD,
    energy_bonus=1,
    effects=["onEquip: +1 Energy. atBattleStart: Shuffle 2 Wounds into draw pile"],
)

NUCLEAR_BATTERY = Relic(
    id="Nuclear Battery", name="Nuclear Battery", tier=RelicTier.BOSS,
    player_class=PlayerClass.DEFECT,
    effects=["atPreBattle: Channel 1 Plasma orb"],
)

PANDORAS_BOX = Relic(
    id="Pandora's Box", name="Pandora's Box", tier=RelicTier.BOSS,
    effects=["onEquip: Transform all Strikes and Defends"],
)

PHILOSOPHERS_STONE = Relic(
    id="Philosopher's Stone", name="Philosopher's Stone", tier=RelicTier.BOSS,
    energy_bonus=1,
    effects=["onEquip: +1 Energy. atBattleStart: ALL enemies gain 1 Strength"],
)

RUNIC_CUBE = Relic(
    id="Runic Cube", name="Runic Cube", tier=RelicTier.BOSS,
    player_class=PlayerClass.IRONCLAD,
    effects=["wasHPLost: Draw 1 card"],
)

RUNIC_DOME = Relic(
    id="Runic Dome", name="Runic Dome", tier=RelicTier.BOSS,
    energy_bonus=1,
    hides_intent=True,
    effects=["onEquip: +1 Energy. Cannot see enemy intents"],
)

RUNIC_PYRAMID = Relic(
    id="Runic Pyramid", name="Runic Pyramid", tier=RelicTier.BOSS,
    effects=["Hand is not discarded at end of turn"],
)

SACRED_BARK = Relic(
    id="SacredBark", name="Sacred Bark", tier=RelicTier.BOSS,
    effects=["Potion effects are doubled"],
)

SLAVERS_COLLAR = Relic(
    id="SlaversCollar", name="Slaver's Collar", tier=RelicTier.BOSS,
    effects=["In Elite and Boss combats: +1 Energy"],
)

SNECKO_EYE = Relic(
    id="Snecko Eye", name="Snecko Eye", tier=RelicTier.BOSS,
    hand_size_bonus=2,
    effects=["onEquip: +2 card draw. atPreBattle: Apply Confused (randomize costs)"],
)

SOZU = Relic(
    id="Sozu", name="Sozu", tier=RelicTier.BOSS,
    energy_bonus=1,
    prevents_potions=True,
    effects=["onEquip: +1 Energy. Cannot obtain potions"],
)

TINY_HOUSE = Relic(
    id="Tiny House", name="Tiny House", tier=RelicTier.BOSS,
    effects=["onEquip: Gain 50 Gold, 5 Max HP, 1 potion, 1 card, Upgrade 1 card"],
)

VELVET_CHOKER = Relic(
    id="Velvet Choker", name="Velvet Choker", tier=RelicTier.BOSS,
    energy_bonus=1,
    counter_type="combat", counter_max=6, counter_start=0,
    effects=["onEquip: +1 Energy. Can only play 6 cards per turn"],
)

VIOLET_LOTUS = Relic(
    id="VioletLotus", name="Violet Lotus", tier=RelicTier.BOSS,
    player_class=PlayerClass.WATCHER,
    effects=["onChangeStance (exit Calm): Gain 1 additional Energy"],
)

WRIST_BLADE = Relic(
    id="WristBlade", name="Wrist Blade", tier=RelicTier.BOSS,
    player_class=PlayerClass.SILENT,
    effects=["atDamageModify: 0-cost Attacks deal 4 additional damage"],
)

RUNIC_CAPACITOR = Relic(
    id="Runic Capacitor", name="Runic Capacitor", tier=RelicTier.SHOP,  # Actually SHOP, but often considered with boss-level power
    player_class=PlayerClass.DEFECT,
    orb_slots=3,
    effects=["onEquip: Gain 3 Orb slots"],
)

# ============================================================================
# SHOP RELICS (15 total)
# ============================================================================

THE_ABACUS = Relic(
    id="TheAbacus", name="The Abacus", tier=RelicTier.SHOP,
    effects=["onShuffle: Gain 6 Block"],
)

BRIMSTONE = Relic(
    id="Brimstone", name="Brimstone", tier=RelicTier.SHOP,
    player_class=PlayerClass.IRONCLAD,
    effects=["atTurnStart: Gain 2 Strength. ALL enemies gain 1 Strength"],
)

CAULDRON = Relic(
    id="Cauldron", name="Cauldron", tier=RelicTier.SHOP,
    effects=["onEquip: Obtain 5 random potions"],
)

CHEMICAL_X = Relic(
    id="Chemical X", name="Chemical X", tier=RelicTier.SHOP,
    effects=["X-cost cards receive +2 to X"],
)

CLOCKWORK_SOUVENIR = Relic(
    id="ClockworkSouvenir", name="Clockwork Souvenir", tier=RelicTier.SHOP,
    effects=["atBattleStart: Gain 1 Artifact"],
)

DOLLYS_MIRROR = Relic(
    id="DollysMirror", name="Dolly's Mirror", tier=RelicTier.SHOP,
    effects=["onEquip: Duplicate a card in your deck"],
)

FROZEN_EYE = Relic(
    id="Frozen Eye", name="Frozen Eye", tier=RelicTier.SHOP,
    effects=["You can view your Draw Pile at any time (sorted)"],
)

HAND_DRILL = Relic(
    id="HandDrill", name="Hand Drill", tier=RelicTier.SHOP,
    effects=["onBlockBroken: Apply 2 Vulnerable"],
)

LEES_WAFFLE = Relic(
    id="Lee's Waffle", name="Lee's Waffle", tier=RelicTier.SHOP,
    max_hp_bonus=7,
    effects=["onEquip: Gain 7 Max HP, heal to full"],
)

MEDICAL_KIT = Relic(
    id="Medical Kit", name="Medical Kit", tier=RelicTier.SHOP,
    effects=["Status cards can be played. Playing a Status exhausts it"],
)

MELANGE = Relic(
    id="Melange", name="Melange", tier=RelicTier.SHOP,
    player_class=PlayerClass.WATCHER,
    effects=["onEnterRestRoom: Scry 3"],
)

MEMBERSHIP_CARD = Relic(
    id="Membership Card", name="Membership Card", tier=RelicTier.SHOP,
    effects=["50% discount at shops"],
)

ORANGE_PELLETS = Relic(
    id="OrangePellets", name="Orange Pellets", tier=RelicTier.SHOP,
    effects=["If Attack, Skill, Power played in same turn: Remove all debuffs"],
)

ORRERY = Relic(
    id="Orrery", name="Orrery", tier=RelicTier.SHOP,
    effects=["onEquip: Choose and add 5 cards to deck"],
)

PRISMATIC_SHARD = Relic(
    id="PrismaticShard", name="Prismatic Shard", tier=RelicTier.SHOP,
    effects=["Card rewards can contain cards from any class"],
)

SLING = Relic(
    id="Sling", name="Sling of Courage", tier=RelicTier.SHOP,
    effects=["atBattleStart (Elite): Gain 2 Strength"],
)

STRANGE_SPOON = Relic(
    id="Strange Spoon", name="Strange Spoon", tier=RelicTier.SHOP,
    effects=["50% chance exhausted cards go to discard instead"],
)

TWISTED_FUNNEL = Relic(
    id="TwistedFunnel", name="Twisted Funnel", tier=RelicTier.SHOP,
    player_class=PlayerClass.SILENT,
    effects=["atBattleStart: Apply 4 Poison to ALL enemies"],
)

# ============================================================================
# SPECIAL/EVENT RELICS (20 total)
# ============================================================================

BLOODY_IDOL = Relic(
    id="Bloody Idol", name="Bloody Idol", tier=RelicTier.SPECIAL,
    effects=["onGainGold: Heal 5 HP"],
)

CIRCLET = Relic(
    id="Circlet", name="Circlet", tier=RelicTier.SPECIAL,
    counter_type="permanent", counter_start=1,
    effects=["Obtained when no other relics available. Stacks."],
)

CULTIST_MASK = Relic(
    id="CultistMask", name="Cultist Headpiece", tier=RelicTier.SPECIAL,
    effects=["atBattleStart: Gain 1 Ritual (gain 1 Strength each turn)"],
)

ENCHIRIDION = Relic(
    id="Enchiridion", name="Enchiridion", tier=RelicTier.SPECIAL,
    effects=["atBattleStart: Add random Power to hand, it costs 0 this turn"],
)

FACE_OF_CLERIC = Relic(
    id="FaceOfCleric", name="Face of Cleric", tier=RelicTier.SPECIAL,
    max_hp_bonus=1,
    effects=["onVictory (any combat): Gain 1 Max HP"],
)

GOLDEN_IDOL = Relic(
    id="Golden Idol", name="Golden Idol", tier=RelicTier.SPECIAL,
    effects=["Gain 25% more Gold"],
)

GREMLIN_MASK = Relic(
    id="GremlinMask", name="Gremlin Visage", tier=RelicTier.SPECIAL,
    effects=["atBattleStart: Gain 1 Weak"],
)

MARK_OF_THE_BLOOM = Relic(
    id="Mark of the Bloom", name="Mark of the Bloom", tier=RelicTier.SPECIAL,
    prevents_healing=True,
    effects=["You can no longer heal"],
)

MUTAGENIC_STRENGTH = Relic(
    id="MutagenicStrength", name="Mutagenic Strength", tier=RelicTier.SPECIAL,
    effects=["atBattleStart: Gain 3 Strength. At end of turn, lose 3 Strength"],
)

NECRONOMICON = Relic(
    id="Necronomicon", name="Necronomicon", tier=RelicTier.SPECIAL,
    effects=["onEquip: Obtain Necronomicurse. First 2+ cost Attack each turn plays twice"],
)

NEOWS_LAMENT = Relic(
    id="NeowsBlessing", name="Neow's Lament", tier=RelicTier.SPECIAL,
    counter_type="uses", counter_start=3,
    effects=["First 3 combats: Enemies have 1 HP"],
)

NILRYS_CODEX = Relic(
    id="Nilry's Codex", name="Nilry's Codex", tier=RelicTier.SPECIAL,
    effects=["onPlayerEndTurn: Choose 1 of 3 cards to add to hand next turn"],
)

NLOTHS_GIFT = Relic(
    id="Nloth's Gift", name="N'loth's Gift", tier=RelicTier.SPECIAL,
    effects=["Triple chance for Rare cards in card rewards"],
)

NLOTHS_MASK = Relic(
    id="NlothsMask", name="N'loth's Hungry Face", tier=RelicTier.SPECIAL,
    effects=["The first chest is empty. Future non-boss chests have better rewards"],
)

ODD_MUSHROOM = Relic(
    id="Odd Mushroom", name="Odd Mushroom", tier=RelicTier.SPECIAL,
    effects=["Vulnerable only increases damage by 25% instead of 50%"],
)

RED_CIRCLET = Relic(
    id="Red Circlet", name="Red Circlet", tier=RelicTier.SPECIAL,
    effects=["Obtained in Endless mode when all relics obtained"],
)

RED_MASK = Relic(
    id="Red Mask", name="Red Mask", tier=RelicTier.SPECIAL,
    effects=["atBattleStart: Apply 1 Weak to ALL enemies"],
)

SPIRIT_POOP = Relic(
    id="Spirit Poop", name="Spirit Poop", tier=RelicTier.SPECIAL,
    effects=["It's the poop of a spirit"],
)

SSSERPENT_HEAD = Relic(
    id="SsserpentHead", name="Ssserpent Head", tier=RelicTier.SPECIAL,
    effects=["Whenever you enter a ? room, gain 50 Gold"],
)

WARPED_TONGS = Relic(
    id="WarpedTongs", name="Warped Tongs", tier=RelicTier.SPECIAL,
    effects=["atTurnStart: Upgrade a random card in hand for this combat"],
)


# ============================================================================
# RELIC REGISTRY
# ============================================================================

# All relics by ID
ALL_RELICS: Dict[str, Relic] = {
    # Starter
    "Burning Blood": BURNING_BLOOD,
    "Ring of the Snake": RING_OF_THE_SNAKE,
    "Cracked Core": CRACKED_CORE,
    "PureWater": PURE_WATER,
    # Common
    "Akabeko": AKABEKO,
    "Anchor": ANCHOR,
    "Ancient Tea Set": ANCIENT_TEA_SET,
    "Art of War": ART_OF_WAR,
    "Bag of Marbles": BAG_OF_MARBLES,
    "Bag of Preparation": BAG_OF_PREPARATION,
    "Blood Vial": BLOOD_VIAL,
    "Bronze Scales": BRONZE_SCALES,
    "Centennial Puzzle": CENTENNIAL_PUZZLE,
    "CeramicFish": CERAMIC_FISH,
    "Damaru": DAMARU,
    "DataDisk": DATA_DISK,
    "Dream Catcher": DREAM_CATCHER,
    "Happy Flower": HAPPY_FLOWER,
    "Juzu Bracelet": JUZU_BRACELET,
    "Lantern": LANTERN,
    "MawBank": MAW_BANK,
    "MealTicket": MEAL_TICKET,
    "Nunchaku": NUNCHAKU,
    "Oddly Smooth Stone": ODDLY_SMOOTH_STONE,
    "Omamori": OMAMORI,
    "Orichalcum": ORICHALCUM,
    "Pen Nib": PEN_NIB,
    "Potion Belt": POTION_BELT,
    "PreservedInsect": PRESERVED_INSECT,
    "Regal Pillow": REGAL_PILLOW,
    "Smiling Mask": SMILING_MASK,
    "Snake Skull": SNECKO_SKULL,
    "Strawberry": STRAWBERRY,
    "Boot": THE_BOOT,
    "Tiny Chest": TINY_CHEST,
    "Toy Ornithopter": TOY_ORNITHOPTER,
    "Vajra": VAJRA,
    "War Paint": WAR_PAINT,
    "Whetstone": WHETSTONE,
    "Red Skull": RED_SKULL,
    # Uncommon
    "Blue Candle": BLUE_CANDLE,
    "Bottled Flame": BOTTLED_FLAME,
    "Bottled Lightning": BOTTLED_LIGHTNING,
    "Bottled Tornado": BOTTLED_TORNADO,
    "Darkstone Periapt": DARKSTONE_PERIAPT,
    "Eternal Feather": ETERNAL_FEATHER,
    "Frozen Egg 2": FROZEN_EGG,
    "Cables": GOLD_PLATED_CABLES,
    "Gremlin Horn": GREMLIN_HORN,
    "HornCleat": HORN_CLEAT,
    "InkBottle": INK_BOTTLE,
    "Kunai": KUNAI,
    "Letter Opener": LETTER_OPENER,
    "Matryoshka": MATRYOSHKA,
    "Meat on the Bone": MEAT_ON_THE_BONE,
    "Mercury Hourglass": MERCURY_HOURGLASS,
    "Molten Egg 2": MOLTEN_EGG,
    "Mummified Hand": MUMMIFIED_HAND,
    "Ninja Scroll": NINJA_SCROLL,
    "Ornamental Fan": ORNAMENTAL_FAN,
    "Pantograph": PANTOGRAPH,
    "Paper Crane": PAPER_CRANE,
    "Paper Frog": PAPER_FROG,
    "Pear": PEAR,
    "Question Card": QUESTION_CARD,
    "Self Forming Clay": SELF_FORMING_CLAY,
    "Shuriken": SHURIKEN,
    "Singing Bowl": SINGING_BOWL,
    "StrikeDummy": STRIKE_DUMMY,
    "Sundial": SUNDIAL,
    "Symbiotic Virus": SYMBIOTIC_VIRUS,
    "TeardropLocket": TEARDROP_LOCKET,
    "The Courier": THE_COURIER,
    "Toxic Egg 2": TOXIC_EGG,
    "White Beast Statue": WHITE_BEAST_STATUE,
    "Yang": DUALITY,
    "Discerning Monocle": DISCERNING_MONOCLE,
    # Rare
    "Bird Faced Urn": BIRD_FACED_URN,
    "Calipers": CALIPERS,
    "CaptainsWheel": CAPTAINS_WHEEL,
    "Charon's Ashes": CHARONS_ASHES,
    "Champion Belt": CHAMPIONS_BELT,
    "CloakClasp": CLOAK_CLASP,
    "Dead Branch": DEAD_BRANCH,
    "Du-Vu Doll": DU_VU_DOLL,
    "Emotion Chip": EMOTION_CHIP,
    "FossilizedHelix": FOSSILIZED_HELIX,
    "Gambling Chip": GAMBLING_CHIP,
    "Ginger": GINGER,
    "Girya": GIRYA,
    "GoldenEye": GOLDEN_EYE,
    "Ice Cream": ICE_CREAM,
    "Incense Burner": INCENSE_BURNER,
    "Lizard Tail": LIZARD_TAIL,
    "Magic Flower": MAGIC_FLOWER,
    "Mango": MANGO,
    "Old Coin": OLD_COIN,
    "Peace Pipe": PEACE_PIPE,
    "Pocketwatch": POCKETWATCH,
    "Prayer Wheel": PRAYER_WHEEL,
    "Shovel": SHOVEL,
    "StoneCalendar": STONE_CALENDAR,
    "The Specimen": THE_SPECIMEN,
    "Thread and Needle": THREAD_AND_NEEDLE,
    "Tingsha": TINGSHA,
    "Torii": TORII,
    "Tough Bandages": TOUGH_BANDAGES,
    "TungstenRod": TUNGSTEN_ROD,
    "Turnip": TURNIP,
    "Unceasing Top": UNCEASING_TOP,
    "WingedGreaves": WING_BOOTS,
    # Boss
    "Black Blood": BLACK_BLOOD,
    "Ring of the Serpent": RING_OF_THE_SERPENT,
    "FrozenCore": FROZEN_CORE,
    "HolyWater": HOLY_WATER,
    "Astrolabe": ASTROLABE,
    "Black Star": BLACK_STAR,
    "Busted Crown": BUSTED_CROWN,
    "Calling Bell": CALLING_BELL,
    "Coffee Dripper": COFFEE_DRIPPER,
    "Cursed Key": CURSED_KEY,
    "Ectoplasm": ECTOPLASM,
    "Empty Cage": EMPTY_CAGE,
    "Fusion Hammer": FUSION_HAMMER,
    "HoveringKite": HOVERING_KITE,
    "Inserter": INSERTER,
    "Mark of Pain": MARK_OF_PAIN,
    "Nuclear Battery": NUCLEAR_BATTERY,
    "Pandora's Box": PANDORAS_BOX,
    "Philosopher's Stone": PHILOSOPHERS_STONE,
    "Runic Cube": RUNIC_CUBE,
    "Runic Dome": RUNIC_DOME,
    "Runic Pyramid": RUNIC_PYRAMID,
    "SacredBark": SACRED_BARK,
    "SlaversCollar": SLAVERS_COLLAR,
    "Snecko Eye": SNECKO_EYE,
    "Sozu": SOZU,
    "Tiny House": TINY_HOUSE,
    "Velvet Choker": VELVET_CHOKER,
    "VioletLotus": VIOLET_LOTUS,
    "WristBlade": WRIST_BLADE,
    # Shop
    "TheAbacus": THE_ABACUS,
    "Brimstone": BRIMSTONE,
    "Cauldron": CAULDRON,
    "Chemical X": CHEMICAL_X,
    "ClockworkSouvenir": CLOCKWORK_SOUVENIR,
    "DollysMirror": DOLLYS_MIRROR,
    "Frozen Eye": FROZEN_EYE,
    "HandDrill": HAND_DRILL,
    "Lee's Waffle": LEES_WAFFLE,
    "Medical Kit": MEDICAL_KIT,
    "Melange": MELANGE,
    "Membership Card": MEMBERSHIP_CARD,
    "OrangePellets": ORANGE_PELLETS,
    "Orrery": ORRERY,
    "PrismaticShard": PRISMATIC_SHARD,
    "Sling": SLING,
    "Strange Spoon": STRANGE_SPOON,
    "TwistedFunnel": TWISTED_FUNNEL,
    "Runic Capacitor": RUNIC_CAPACITOR,
    # Special
    "Bloody Idol": BLOODY_IDOL,
    "Circlet": CIRCLET,
    "CultistMask": CULTIST_MASK,
    "Enchiridion": ENCHIRIDION,
    "FaceOfCleric": FACE_OF_CLERIC,
    "Golden Idol": GOLDEN_IDOL,
    "GremlinMask": GREMLIN_MASK,
    "Mark of the Bloom": MARK_OF_THE_BLOOM,
    "MutagenicStrength": MUTAGENIC_STRENGTH,
    "Necronomicon": NECRONOMICON,
    "NeowsBlessing": NEOWS_LAMENT,
    "Nilry's Codex": NILRYS_CODEX,
    "Nloth's Gift": NLOTHS_GIFT,
    "NlothsMask": NLOTHS_MASK,
    "Odd Mushroom": ODD_MUSHROOM,
    "Red Circlet": RED_CIRCLET,
    "Red Mask": RED_MASK,
    "Spirit Poop": SPIRIT_POOP,
    "SsserpentHead": SSSERPENT_HEAD,
    "WarpedTongs": WARPED_TONGS,
}

# Relics grouped by tier
STARTER_RELICS: List[str] = [k for k, v in ALL_RELICS.items() if v.tier == RelicTier.STARTER]
COMMON_RELICS: List[str] = [k for k, v in ALL_RELICS.items() if v.tier == RelicTier.COMMON]
UNCOMMON_RELICS: List[str] = [k for k, v in ALL_RELICS.items() if v.tier == RelicTier.UNCOMMON]
RARE_RELICS: List[str] = [k for k, v in ALL_RELICS.items() if v.tier == RelicTier.RARE]
BOSS_RELICS: List[str] = [k for k, v in ALL_RELICS.items() if v.tier == RelicTier.BOSS]
SHOP_RELICS: List[str] = [k for k, v in ALL_RELICS.items() if v.tier == RelicTier.SHOP]
SPECIAL_RELICS: List[str] = [k for k, v in ALL_RELICS.items() if v.tier == RelicTier.SPECIAL]


def get_relic(relic_id: str) -> Relic:
    """Get a copy of a relic by ID."""
    if relic_id not in ALL_RELICS:
        raise ValueError(f"Unknown relic: {relic_id}")
    return ALL_RELICS[relic_id].copy()


def get_relics_by_tier(tier: RelicTier) -> List[Relic]:
    """Get all relics of a specific tier."""
    return [v.copy() for v in ALL_RELICS.values() if v.tier == tier]


def get_relics_for_class(player_class: PlayerClass) -> Dict[str, Relic]:
    """Get all relics available to a specific class."""
    result = {}
    for relic_id, relic in ALL_RELICS.items():
        if relic.player_class == PlayerClass.ALL or relic.player_class == player_class:
            result[relic_id] = relic.copy()
    return result


def get_starter_relic(player_class: PlayerClass) -> Relic:
    """Get the starter relic for a specific class."""
    for relic in ALL_RELICS.values():
        if relic.tier == RelicTier.STARTER and relic.player_class == player_class:
            return relic.copy()
    raise ValueError(f"No starter relic for class: {player_class}")


# ============================================================================
# STATISTICS
# ============================================================================

if __name__ == "__main__":
    print("=== Slay the Spire Relic Statistics ===\n")

    print(f"Total relics: {len(ALL_RELICS)}")
    print(f"  Starter: {len(STARTER_RELICS)}")
    print(f"  Common: {len(COMMON_RELICS)}")
    print(f"  Uncommon: {len(UNCOMMON_RELICS)}")
    print(f"  Rare: {len(RARE_RELICS)}")
    print(f"  Boss: {len(BOSS_RELICS)}")
    print(f"  Shop: {len(SHOP_RELICS)}")
    print(f"  Special: {len(SPECIAL_RELICS)}")

    print("\n=== Starter Relics ===")
    for relic_id in STARTER_RELICS:
        relic = ALL_RELICS[relic_id]
        print(f"  {relic.name} ({relic.player_class.name})")

    print("\n=== Boss Relics with Energy ===")
    energy_boss_relics = [r for r in ALL_RELICS.values() if r.tier == RelicTier.BOSS and r.energy_bonus > 0]
    for relic in energy_boss_relics:
        print(f"  {relic.name}: +{relic.energy_bonus} Energy")

    print("\n=== Boss Relics that Upgrade Starters ===")
    upgrade_relics = [r for r in ALL_RELICS.values() if r.requires_relic is not None]
    for relic in upgrade_relics:
        print(f"  {relic.name} (requires {relic.requires_relic})")

    print("\n=== Relics with Counters ===")
    counter_relics = [r for r in ALL_RELICS.values() if r.counter_type is not None]
    for relic in counter_relics:
        counter_info = f"max={relic.counter_max}" if relic.counter_max else f"start={relic.counter_start}"
        print(f"  {relic.name}: {relic.counter_type} ({counter_info})")
