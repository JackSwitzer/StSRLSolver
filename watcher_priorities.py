"""
Watcher Priority System for Slay the Spire AI

Based on high-level Watcher strategy:
- Stance cycling is key (Wrath for damage, Calm for energy)
- Rushdown + stance dancing = scaling engine
- Small decks are better (avoid bloat)
- Scry helps set up perfect hands
"""
import math


class WatcherPriority:
    """Priority system optimized for Watcher gameplay."""

    # Cards to pick during rewards (lower index = higher priority)
    CARD_PRIORITY_LIST = [
        # === TOP TIER: Win conditions and scaling ===
        "Rushdown",           # Core engine - draw on wrath entry
        "Tantrum",            # Wrath entry + multi-hit
        "Ragnarok",           # Massive damage in wrath
        "MentalFortress",     # Block on stance change
        "TalkToTheHand",      # Block generation vs single target
        "Apotheosis",         # Upgrade all

        # === HIGH TIER: Strong utility ===
        "Wallop",             # Big damage + block
        "Brilliance",         # Scales with mantra
        "LessonLearned",      # Kill reward
        "Vault",              # Extra turn setup
        "Omniscience",        # Double play
        "EmptyMind",          # Calm + draw
        "Scrawl",             # Big draw

        # === SOLID TIER: Good cards ===
        "InnerPeace",         # Calm + draw
        "CutThroughFate",     # Scry + draw + damage
        "WheelKick",          # Damage + draw
        "Conclude",           # AOE finisher
        "SashWhip",           # Retain + weak
        "FlyingSleeves",      # Retain attack
        "WindmillStrike",     # Retain + big damage
        "Worship",            # Mantra gain
        "Devotion",           # Passive mantra
        "WaveOfTheHand",      # Weak application
        "Prostrate",          # Mantra + block
        "Protect",            # Retain block
        "Halt",               # Block + wrath bonus
        "ThirdEye",           # Block + scry
        "Perseverance",       # Retain + scaling block
        "Evaluate",           # Insight generation
        "FearNoEvil",         # Wrath entry on attack
        "Crescendo",          # Free wrath entry
        "FlurryOfBlows",      # Free attack on stance change
        "Tranquility",        # Free calm entry
        "EmptyFist",          # Exit wrath + damage
        "EmptyBody",          # Exit wrath + block
        "Swivel",             # Block + next attack buff
        "SandsOfTime",        # Big retain attack
        "ReachHeaven",        # Creates Through Violence
        "JustLucky",          # Scry + block + damage
        "Consecrate",         # AOE damage
        "FollowUp",           # Conditional energy
        "Indignation",        # Wrath/vulnerable
        "WreathOfFlame",      # Next attack buff
        "Collect",            # Miracle generation
        "Pray",               # Mantra + insight

        # === OKAY TIER: Situational ===
        "BowlingBash",        # Damage based on enemies
        "Vigilance",          # Basic calm + block
        "Eruption",           # Basic wrath + damage
        "CarveReality",       # Creates Smite
        "Sanctity",           # Block + draw if debuffed
        "BattleHymn",         # Smite generation
        "MasterReality",      # Retain upgrade
        "DeceiveReality",     # Block + safety
        "ForeignInfluence",   # Card generation
        "SpiritShield",       # Block based on hand
        "Meditate",           # Retain + calm
        "Nirvana",            # Block on scry
        "LikeWater",          # Block if calm
        "Study",              # Insight generation
        "Alpha",              # Slow but powerful
        "ConjureBlade",       # Expensive but strong

        # === SKIP LINE ===
        "Skip",

        # === BELOW SKIP: Generally avoid ===
        "Fasting",            # Risky
        "Wish",               # Expensive
        "Establishment",      # Niche
        "PressurePoints",     # Different build
        "SignatureMove",      # Hard to enable
        "Blasphemy",          # Very risky
        "Judgment",           # Only for kills

        # === CURSES AND STATUS ===
        "Dazed",
        "Void",
        "AscendersBane",
        "CrownOfThorns",
        "Wound",
        "Slimed",
        "Burn",
        "Clumsy",
        "Parasite",
        "Injury",
        "Shame",
        "Doubt",
        "Decay",
        "Writhe",
        "Regret",
        "Pain",
        "Necronomicurse",
        "Normality",
        "Pride",

        # === STARTER CARDS (want to remove) ===
        "Strike_P",
        "Defend_P",
    ]

    # Cards to play during combat (lower index = higher priority)
    PLAY_PRIORITY_LIST = [
        # === SETUP FIRST ===
        "Apotheosis",         # Upgrade all immediately
        "Omniscience",        # Double next card
        "Scrawl",             # Draw when hand empty
        "Vault",              # Extra turn
        "Devotion",           # Passive mantra
        "MentalFortress",     # Passive block
        "LikeWater",          # Passive block
        "Rushdown",           # Draw engine
        "TalkToTheHand",      # Block engine
        "Establishment",      # Cost reduction
        "MasterReality",      # Retain improvement
        "BattleHymn",         # Smite generation
        "Nirvana",            # Scry block
        "Alpha",              # Start chain

        # === ENERGY/DRAW ===
        "Collect",            # Miracle generation
        "EmptyMind",          # Calm + draw
        "InnerPeace",         # Calm + draw
        "Meditate",           # Calm + retain
        "Pray",               # Mantra + insight
        "Prostrate",          # Mantra + block
        "Worship",            # Mantra
        "Evaluate",           # Insight
        "Study",              # Insight

        # === ENTER CALM (for energy) ===
        "Tranquility",        # Free calm
        "Vigilance",          # Calm + block
        "FearNoEvil",         # Conditional wrath

        # === ATTACKS (play in wrath for 2x damage) ===
        "Ragnarok",           # Big multi-hit
        "Brilliance",         # Mantra scaling
        "LessonLearned",      # Kill reward
        "Wallop",             # Damage + block
        "Conclude",           # AOE
        "SandsOfTime",        # Big single target
        "WindmillStrike",     # Big retain
        "ReachHeaven",        # Creates Through Violence
        "WheelKick",          # Damage + draw
        "CutThroughFate",     # Scry + draw + damage
        "Tantrum",            # Multi-hit + wrath
        "FlyingSleeves",      # Retain
        "SashWhip",           # Retain + weak
        "BowlingBash",        # Multi-enemy scaling
        "Consecrate",         # AOE
        "CarveReality",       # Creates Smite
        "EmptyFist",          # Exit wrath
        "FlurryOfBlows",      # Free on stance change
        "ConjureBlade",       # Big expensive
        "JustLucky",          # Scry + damage
        "FollowUp",           # Energy
        "SignatureMove",      # Conditional big damage

        # === ENTER WRATH (before attacks) ===
        "Crescendo",          # Free wrath
        "Eruption",           # Wrath + damage
        "Indignation",        # Wrath + vulnerable
        "WreathOfFlame",      # Attack buff

        # === DEFENSE ===
        "DeceiveReality",     # Block + safety
        "SpiritShield",       # Hand-based block
        "Protect",            # Retain block
        "Perseverance",       # Scaling retain
        "EmptyBody",          # Exit wrath + block
        "Halt",               # Wrath bonus
        "WaveOfTheHand",      # Weak
        "Sanctity",           # Conditional
        "Swivel",             # Block + buff
        "ThirdEye",           # Scry + block
        "Defend_P",           # Basic

        # === LOW PRIORITY ===
        "Strike_P",           # Basic attack
        "ForeignInfluence",   # Random
        "Fasting",            # Risky
        "Wish",               # Expensive
        "PressurePoints",     # Different build
        "Blasphemy",          # Very risky
        "Judgment",           # Finisher only

        # === STATUS/CURSES (don't play) ===
        "Dazed",
        "Void",
        "AscendersBane",
        "Wound",
        "Slimed",
        "Burn",
        "Clumsy",
        "Parasite",
        "Injury",
        "Shame",
        "Doubt",
        "Decay",
        "Writhe",
        "Regret",
        "Pain",
        "Necronomicurse",
        "Normality",
        "Pride",
    ]

    AOE_CARDS = [
        "Ragnarok",
        "Conclude",
        "Consecrate",
        "BowlingBash",
        "WaveOfTheHand",
        "Indignation",
    ]

    DEFENSIVE_CARDS = [
        "Vigilance",
        "Protect",
        "Halt",
        "Perseverance",
        "EmptyBody",
        "SpiritShield",
        "DeceiveReality",
        "ThirdEye",
        "Swivel",
        "Sanctity",
        "WaveOfTheHand",
        "Defend_P",
        "LikeWater",
        "Nirvana",
        "MentalFortress",
        "TalkToTheHand",
    ]

    # Max copies to pick up
    MAX_COPIES = {
        "Rushdown": 2,
        "Tantrum": 2,
        "Ragnarok": 1,
        "MentalFortress": 1,
        "TalkToTheHand": 2,
        "Apotheosis": 1,
        "Wallop": 2,
        "Brilliance": 1,
        "LessonLearned": 1,
        "Vault": 1,
        "Omniscience": 1,
        "EmptyMind": 2,
        "Scrawl": 1,
        "InnerPeace": 2,
        "CutThroughFate": 2,
        "WheelKick": 2,
        "Conclude": 1,
        "FlyingSleeves": 2,
        "WindmillStrike": 1,
        "Worship": 2,
        "Devotion": 1,
        "Crescendo": 2,
        "Tranquility": 1,
        "FlurryOfBlows": 2,
        "Vigilance": 1,
        "Eruption": 1,
    }

    BOSS_RELIC_PRIORITY_LIST = [
        "Violet Lotus",       # Watcher specific - energy on calm exit
        "Sozu",               # No potions but strong
        "Snecko Eye",         # Great with high-cost cards
        "Runic Dome",         # No intent but strong
        "Philosopher's Stone",
        "Cursed Key",
        "Fusion Hammer",
        "Velvet Choker",
        "Ectoplasm",
        "Busted Crown",
        "Empty Cage",
        "Coffee Dripper",
        "Runic Pyramid",
        "Astrolabe",
        "Pandora's Box",
        "Tiny House",
        "Black Star",
        "Holy Water",         # Watcher specific
        "Calling Bell",
    ]

    # Map priorities by act
    MAP_NODE_PRIORITIES_1 = {'R': 1000, 'E': 50, '$': 100, '?': 100, 'M': 1, 'T': 0}
    MAP_NODE_PRIORITIES_2 = {'R': 1000, 'E': 100, '$': 50, '?': 50, 'M': 1, 'T': 0}
    MAP_NODE_PRIORITIES_3 = {'R': 1000, 'E': 10, '$': 100, '?': 100, 'M': 1, 'T': 0}

    GOOD_CARD_ACTIONS = [
        "PutOnDeckAction",
        "ArmamentsAction",
        "DualWieldAction",
        "NightmareAction",
        "RetainCardsAction",
        "SetupAction",
        "MeditateAction",
    ]

    BAD_CARD_ACTIONS = [
        "DiscardAction",
        "ExhaustAction",
        "PutOnBottomOfDeckAction",
        "RecycleAction",
        "ForethoughtAction",
        "GamblingChipAction",
    ]

    def __init__(self):
        self.CARD_PRIORITIES = {self.CARD_PRIORITY_LIST[i]: i for i in range(len(self.CARD_PRIORITY_LIST))}
        self.PLAY_PRIORITIES = {self.PLAY_PRIORITY_LIST[i]: i for i in range(len(self.PLAY_PRIORITY_LIST))}
        self.BOSS_RELIC_PRIORITIES = {self.BOSS_RELIC_PRIORITY_LIST[i]: i for i in range(len(self.BOSS_RELIC_PRIORITY_LIST))}
        self.MAP_NODE_PRIORITIES = {
            1: self.MAP_NODE_PRIORITIES_1,
            2: self.MAP_NODE_PRIORITIES_2,
            3: self.MAP_NODE_PRIORITIES_3,
            4: self.MAP_NODE_PRIORITIES_3,
        }

    def get_best_card(self, card_list):
        return min(card_list, key=lambda x: self.CARD_PRIORITIES.get(x.card_id, math.inf) - 0.5 * x.upgrades)

    def get_worst_card(self, card_list):
        return max(card_list, key=lambda x: self.CARD_PRIORITIES.get(x.card_id, math.inf) - 0.5 * x.upgrades)

    def get_sorted_cards(self, card_list, reverse=False):
        return sorted(card_list, key=lambda x: self.CARD_PRIORITIES.get(x.card_id, math.inf) - 0.5 * x.upgrades, reverse=reverse)

    def get_sorted_cards_to_play(self, card_list, reverse=False):
        return sorted(card_list, key=lambda x: self.PLAY_PRIORITIES.get(x.card_id, math.inf) - 0.5 * x.upgrades, reverse=reverse)

    def get_best_card_to_play(self, card_list):
        return min(card_list, key=lambda x: self.PLAY_PRIORITIES.get(x.card_id, math.inf) - 0.5 * x.upgrades)

    def get_worst_card_to_play(self, card_list):
        return max(card_list, key=lambda x: self.PLAY_PRIORITIES.get(x.card_id, math.inf) - 0.5 * x.upgrades)

    def should_skip(self, card):
        skip_priority = self.CARD_PRIORITIES.get("Skip", math.inf)
        card_priority = self.CARD_PRIORITIES.get(card.card_id, math.inf)
        return card_priority > skip_priority

    def needs_more_copies(self, card, num_copies):
        return self.MAX_COPIES.get(card.card_id, 0) > num_copies

    def get_best_boss_relic(self, relic_list):
        return min(relic_list, key=lambda x: self.BOSS_RELIC_PRIORITIES.get(x.relic_id, math.inf))

    def is_card_aoe(self, card):
        return card.card_id in self.AOE_CARDS

    def is_card_defensive(self, card):
        return card.card_id in self.DEFENSIVE_CARDS

    def get_cards_for_action(self, action, cards, max_cards):
        if action in self.GOOD_CARD_ACTIONS:
            sorted_cards = self.get_sorted_cards(cards, reverse=False)
        else:
            sorted_cards = self.get_sorted_cards(cards, reverse=True)
        num_cards = min(max_cards, len(cards))
        return sorted_cards[:num_cards]
