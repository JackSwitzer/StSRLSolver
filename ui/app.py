#!/usr/bin/env python3
"""
Slay the Spire Seed Prediction Visualizer

A Flask-based web UI for visualizing seed predictions with:
- Interactive clickable map nodes
- Neow options display
- Floor-by-floor encounter details
- Act switching
"""

import os
import sys
import json
from flask import Flask, render_template, jsonify, request

# Setup path
_ui_dir = os.path.dirname(os.path.abspath(__file__))
_project_dir = os.path.dirname(_ui_dir)
sys.path.insert(0, _project_dir)

from core.state.rng import Random, seed_to_long
from core.state.game_rng import GameRNGState, RNGStream
from core.generation.map import (
    MapGenerator, MapGeneratorConfig, RoomType,
    get_map_seed_offset, MapRoomNode
)
from core.generation.encounters import (
    generate_exordium_encounters, ENEMY_HP_RANGES
)
from core.generation.rewards import (
    RewardState, generate_card_rewards, generate_gold_reward,
    check_potion_drop, generate_elite_relic_reward, generate_boss_relics,
    generate_shop_inventory
)

app = Flask(__name__, template_folder='templates', static_folder='static')


# =============================================================================
# NEOW OPTIONS GENERATION
# =============================================================================

# Based on decompiled NeowReward.java - exact game logic
NEOW_REWARD_NAMES = {
    # Category 0: Card-focused
    "THREE_CARDS": "Choose a card to obtain",
    "ONE_RANDOM_RARE_CARD": "Obtain a random rare card",
    "REMOVE_CARD": "Remove a card",
    "UPGRADE_CARD": "Upgrade a card",
    "TRANSFORM_CARD": "Transform a card",
    "RANDOM_COLORLESS": "Obtain a random colorless card",
    # Category 1: Resource/HP
    "THREE_SMALL_POTIONS": "Obtain 3 potions",
    "RANDOM_COMMON_RELIC": "Obtain a random common relic",
    "TEN_PERCENT_HP_BONUS": "Gain {hp} Max HP",
    "THREE_ENEMY_KILL": "Enemies in first 3 combats have 1 HP",
    "HUNDRED_GOLD": "Gain 100 Gold",
    # Category 2: Premium (with drawback)
    "RANDOM_COLORLESS_2": "Choose a rare colorless card",
    "REMOVE_TWO": "Remove 2 cards",
    "ONE_RARE_RELIC": "Obtain a random rare relic",
    "THREE_RARE_CARDS": "Choose a rare card",
    "TWO_FIFTY_GOLD": "Gain 250 Gold",
    "TRANSFORM_TWO_CARDS": "Transform 2 cards",
    "TWENTY_PERCENT_HP_BONUS": "Gain {hp2} Max HP",
    # Category 3: Boss swap
    "BOSS_RELIC": "Swap starting relic for a random boss relic",
}

NEOW_DRAWBACK_NAMES = {
    "TEN_PERCENT_HP_LOSS": "Lose {hp} Max HP",
    "NO_GOLD": "Lose all Gold",
    "CURSE": "Obtain a Curse",
    "PERCENT_DAMAGE": "Take damage (HP reduced to 70%)",
}


def generate_neow_options(seed: int, max_hp: int = 72) -> list:
    """
    Generate all 4 Neow options for a seed.

    Based on decompiled NeowEvent.java blessing() method.
    RNG is initialized fresh: rng = new Random(Settings.seed)
    """
    neow_rng = Random(seed)

    # HP bonus values (10% and 20% of max HP)
    hp_bonus = max_hp // 10  # 7 for Watcher (72 HP)
    hp_bonus_2 = hp_bonus * 2  # 14

    options = []

    # ===========================================
    # SLOT 0 (Category 0): Card-focused options
    # ===========================================
    cat0_pool = [
        "THREE_CARDS",
        "ONE_RANDOM_RARE_CARD",
        "REMOVE_CARD",
        "UPGRADE_CARD",
        "TRANSFORM_CARD",
        "RANDOM_COLORLESS",
    ]
    # rng.random(0, size-1) in Java
    idx0 = neow_rng.random(len(cat0_pool) - 1)
    opt0 = cat0_pool[idx0]
    options.append({
        "slot": 1,
        "type": "blessing",
        "option": opt0,
        "name": NEOW_REWARD_NAMES[opt0],
        "drawback": None,
    })

    # ===========================================
    # SLOT 1 (Category 1): Resource options
    # ===========================================
    cat1_pool = [
        "THREE_SMALL_POTIONS",
        "RANDOM_COMMON_RELIC",
        "TEN_PERCENT_HP_BONUS",
        "THREE_ENEMY_KILL",
        "HUNDRED_GOLD",
    ]
    idx1 = neow_rng.random(len(cat1_pool) - 1)
    opt1 = cat1_pool[idx1]
    name1 = NEOW_REWARD_NAMES[opt1].replace("{hp}", str(hp_bonus))
    options.append({
        "slot": 2,
        "type": "bonus",
        "option": opt1,
        "name": name1,
        "drawback": None,
    })

    # ===========================================
    # SLOT 2 (Category 2): Premium with drawback
    # First select drawback, then build conditional pool
    # ===========================================
    drawback_pool = [
        "TEN_PERCENT_HP_LOSS",
        "NO_GOLD",
        "CURSE",
        "PERCENT_DAMAGE",
    ]
    drawback_idx = neow_rng.random(len(drawback_pool) - 1)
    drawback = drawback_pool[drawback_idx]
    drawback_name = NEOW_DRAWBACK_NAMES[drawback].replace("{hp}", str(hp_bonus))

    # Build conditional reward pool based on drawback
    cat2_pool = ["RANDOM_COLORLESS_2"]
    if drawback != "CURSE":
        cat2_pool.append("REMOVE_TWO")
    cat2_pool.append("ONE_RARE_RELIC")
    cat2_pool.append("THREE_RARE_CARDS")
    if drawback != "NO_GOLD":
        cat2_pool.append("TWO_FIFTY_GOLD")
    cat2_pool.append("TRANSFORM_TWO_CARDS")
    if drawback != "TEN_PERCENT_HP_LOSS":
        cat2_pool.append("TWENTY_PERCENT_HP_BONUS")

    idx2 = neow_rng.random(len(cat2_pool) - 1)
    opt2 = cat2_pool[idx2]
    name2 = NEOW_REWARD_NAMES[opt2].replace("{hp2}", str(hp_bonus_2))
    options.append({
        "slot": 3,
        "type": "trade",
        "option": opt2,
        "name": name2,
        "drawback": drawback_name,
        "drawback_id": drawback,
    })

    # ===========================================
    # SLOT 3 (Category 3): Always Boss Relic swap
    # ===========================================
    options.append({
        "slot": 4,
        "type": "boss_swap",
        "option": "BOSS_RELIC",
        "name": NEOW_REWARD_NAMES["BOSS_RELIC"],
        "drawback": "Lose starting relic",
    })

    return options


# =============================================================================
# EVENT ROOM PREDICTION
# =============================================================================

class EventRoomPredictor:
    """
    Predicts what ? rooms will become based on eventRng.

    Based on decompiled EventHelper.java - probabilities are stateful
    and ramp up after each "miss".
    """

    # Base probabilities (out of 100)
    BASE_ELITE = 10
    BASE_MONSTER = 10
    BASE_SHOP = 3
    BASE_TREASURE = 2

    # Event pool for Act 1 (Exordium)
    EXORDIUM_EVENTS = [
        "Big Fish", "The Cleric", "Dead Adventurer", "Golden Idol",
        "Golden Wing", "World of Goop", "Liars Game", "Living Wall",
        "Mushrooms", "Scrap Ooze", "Shining Light", "The Ssssserpent",
        "Wing Statue", "Bonfire Spirits", "Duplicator", "Golden Shrine",
        "Lab", "Match and Keep", "Purifier", "Transmogrifier",
        "Upgrade Shrine", "Wheel of Change", "Wishing Well"
    ]

    # Event pool for Act 2 (The City)
    CITY_EVENTS = [
        "Addict", "Back to Basics", "Beggar", "Colosseum",
        "Cursed Tome", "Drug Dealer", "Forgotten Altar", "Ghosts",
        "Knowing Skull", "Masquerade", "The Mausoleum", "The Nest",
        "The Joust", "Vampires", "Council of Ghosts", "Falling",
        "Fountain of Cleansing", "Mind Bloom", "Mysterious Sphere",
        "Nloth", "Secret Portal", "Sensory Stone", "The Woman in Blue"
    ]

    def __init__(self, seed: int):
        self.event_rng = Random(seed)
        # Probability state (ramps up after each miss)
        self.elite_chance = self.BASE_ELITE
        self.monster_chance = self.BASE_MONSTER
        self.shop_chance = self.BASE_SHOP
        self.treasure_chance = self.BASE_TREASURE

    def predict_room(self, floor: int) -> dict:
        """
        Predict what a ? room on this floor will become.

        Returns dict with 'outcome' and 'details'.
        """
        roll = self.event_rng.random_float()  # 0.0 - 1.0
        roll_percent = int(roll * 100)

        # Build probability slots
        slots = []
        current = 0

        # Elite only available floor 6+
        if floor >= 6:
            slots.append(('ELITE', current, current + self.elite_chance))
            current += self.elite_chance

        slots.append(('MONSTER', current, current + self.monster_chance))
        current += self.monster_chance

        slots.append(('SHOP', current, current + self.shop_chance))
        current += self.shop_chance

        slots.append(('TREASURE', current, current + self.treasure_chance))
        current += self.treasure_chance

        # Rest is EVENT
        slots.append(('EVENT', current, 100))

        # Find outcome
        outcome = 'EVENT'
        for slot_type, start, end in slots:
            if start <= roll_percent < end:
                outcome = slot_type
                break

        # Update probabilities for next roll
        if outcome == 'ELITE':
            self.elite_chance = 0  # Reset
        else:
            self.elite_chance = min(self.elite_chance + self.BASE_ELITE, 50)

        if outcome == 'MONSTER':
            self.monster_chance = self.BASE_MONSTER  # Reset
        else:
            self.monster_chance = min(self.monster_chance + self.BASE_MONSTER, 50)

        if outcome == 'SHOP':
            self.shop_chance = self.BASE_SHOP  # Reset
        else:
            self.shop_chance = min(self.shop_chance + self.BASE_SHOP, 20)

        if outcome == 'TREASURE':
            self.treasure_chance = self.BASE_TREASURE  # Reset
        else:
            self.treasure_chance = min(self.treasure_chance + self.BASE_TREASURE, 15)

        result = {
            'outcome': outcome,
            'roll': roll_percent,
        }

        # Add event name if it's an event
        if outcome == 'EVENT':
            # Second roll to pick specific event
            event_roll = self.event_rng.random(len(self.EXORDIUM_EVENTS) - 1)
            result['event_name'] = self.EXORDIUM_EVENTS[event_roll]

        return result


def predict_event_rooms(seed_str: str, event_nodes: list, act: int = 1) -> dict:
    """
    Predict outcomes for all ? rooms given their floor order.

    Args:
        seed_str: The seed string
        event_nodes: List of (x, y, floor) tuples for ? rooms in visit order
        act: Act number

    Returns:
        Dict mapping (x, y) to prediction result
    """
    seed = seed_to_long(seed_str.upper())
    predictor = EventRoomPredictor(seed)

    predictions = {}
    for x, y, floor in event_nodes:
        pred = predictor.predict_room(floor)
        predictions[(x, y)] = pred

    return predictions


# =============================================================================
# ENEMY DATA
# =============================================================================

ENEMY_FIRST_MOVES = {
    "Cultist": {"move": "Incantation", "intent": "BUFF", "details": "Gains 3 Ritual"},
    "Jaw Worm": {"move": "Chomp", "intent": "ATTACK", "damage": 11},
    "Blue Slaver": {"move": "Stab", "intent": "ATTACK", "damage": 12},
    "Red Slaver": {"move": "Stab", "intent": "ATTACK", "damage": 13},
    "Looter": {"move": "Mug", "intent": "ATTACK", "damage": 10},
    "Gremlin Nob": {"move": "Bellow", "intent": "BUFF", "details": "Gains 2 Enrage"},
    "Lagavulin": {"move": "Sleep", "intent": "SLEEP", "details": "Asleep with 8 Metallicize"},
    "3 Sentries": {"move": "Bolt + Beam", "intent": "ATTACK", "damage": 9, "details": "Adds Dazed"},
    "Small Slimes": {"move": "Tackle/Lick", "intent": "ATTACK", "damage": 5},
    "2 Louse": {"move": "Bite/Grow", "intent": "ATTACK", "damage": 6},
    "Fungi Beast": {"move": "Grow", "intent": "BUFF", "details": "+3 Strength"},
    "2 Fungi Beasts": {"move": "Grow", "intent": "BUFF", "details": "+3 Strength each"},
    "Large Slime": {"move": "Tackle", "intent": "ATTACK", "damage": 16},
    "Lots of Slimes": {"move": "Tackle", "intent": "ATTACK", "damage": 5, "details": "x5"},
    "Gremlin Gang": {"move": "Mixed", "intent": "ATTACK", "details": "3-4 gremlins"},
    "Exordium Thugs": {"move": "Stab + Mug", "intent": "ATTACK", "damage": 22},
    "Exordium Wildlife": {"move": "Mixed", "intent": "ATTACK"},
    "3 Louse": {"move": "Bite/Grow", "intent": "ATTACK", "damage": 6},
}

BOSS_DATA = {
    1: [
        {"name": "Slime Boss", "hp": 140, "a9_hp": 154, "move": "Goop Spray", "details": "Shuffles 3 Slimed. Splits at 50% HP."},
        {"name": "The Guardian", "hp": 240, "a9_hp": 264, "move": "Charging Up", "details": "Defensive Mode → Twin Slam (32 dmg)"},
        {"name": "Hexaghost", "hp": 250, "a9_hp": 275, "move": "Activate", "details": "T2: Divider (6 × HP/12+1 dmg)"},
    ],
    2: [
        {"name": "The Champ", "hp": 420, "a9_hp": 462, "move": "Defensive Stance", "details": "Execute at 50% HP"},
        {"name": "The Collector", "hp": 282, "a9_hp": 310, "move": "Spawn", "details": "Summons 2 Torch Heads"},
        {"name": "Automaton", "hp": 300, "a9_hp": 330, "move": "Spawn Orbs", "details": "Hyper Beam (45 dmg)"},
    ],
    3: [
        {"name": "Awakened One", "hp": 300, "a9_hp": 330, "move": "Slash", "details": "Phase 2: 200 HP"},
        {"name": "Time Eater", "hp": 456, "a9_hp": 502, "move": "Ripple", "details": "Haste after 12 cards"},
        {"name": "Donu and Deca", "hp": 500, "a9_hp": 550, "move": "Buff/Block", "details": "Kill Donu first"},
    ],
    4: [
        {"name": "Corrupt Heart", "hp": 800, "a9_hp": 880, "move": "Debilitate", "details": "300 dmg cap. Beat of Death."},
    ],
}


# =============================================================================
# MAP GENERATION API
# =============================================================================

def generate_map_data(seed_str: str, act: int = 1, ascension: int = 20) -> dict:
    """Generate map data as JSON-serializable dict."""
    seed = seed_to_long(seed_str.upper())

    config = MapGeneratorConfig(ascension_level=ascension)
    map_rng = Random(seed + get_map_seed_offset(act))
    generator = MapGenerator(map_rng, config)
    dungeon_map = generator.generate()

    # First pass: collect all edges and find nodes that are reachable
    all_edges = []
    reachable_nodes = set()  # (x, y) tuples

    for y, row in enumerate(dungeon_map):
        for x, node in enumerate(row):
            for edge in node.edges:
                all_edges.append({
                    "src_x": edge.src_x,
                    "src_y": edge.src_y,
                    "dst_x": edge.dst_x,
                    "dst_y": edge.dst_y,
                    "is_boss": edge.is_boss,
                })
                # Both source and destination are reachable
                reachable_nodes.add((edge.src_x, edge.src_y))
                reachable_nodes.add((edge.dst_x, edge.dst_y))

    # Second pass: only include nodes that are reachable and have room types
    nodes = []
    for y, row in enumerate(dungeon_map):
        for x, node in enumerate(row):
            if (x, y) in reachable_nodes and node.room_type is not None:
                nodes.append({
                    "x": x,
                    "y": y,
                    "type": node.room_type.name if node.room_type else None,
                    "symbol": node.room_type.value if node.room_type else "",
                    "has_edges": node.has_edges(),
                })

    return {
        "nodes": nodes,
        "edges": all_edges,
        "width": 7,
        "height": 15,
    }


def generate_encounters_data(seed_str: str, act: int = 1) -> dict:
    """Generate encounter list for an act."""
    seed = seed_to_long(seed_str.upper())
    monster_rng = Random(seed)

    if act == 1:
        normal, elite = generate_exordium_encounters(monster_rng)
    else:
        # Simplified for now
        normal = ["Unknown"] * 15
        elite = ["Unknown"] * 10

    return {
        "normal": normal,
        "elite": elite,
    }


def generate_floor_rewards(seed_str: str, floor: int, room_type: str,
                           encounter_idx: int, act: int = 1, ascension: int = 20) -> dict:
    """Generate rewards for a specific floor."""
    seed = seed_to_long(seed_str.upper())

    rng_state = GameRNGState(seed_str)
    reward_state = RewardState()
    reward_state.add_relic("PureWater")

    # Simulate up to this floor
    for f in range(1, floor):
        rng_state.enter_floor(f)
        rng_state.apply_combat("monster")  # Simplified

    rng_state.enter_floor(floor)

    result = {
        "floor": floor,
        "room_type": room_type,
    }

    if room_type in ["MONSTER", "ELITE"]:
        # Get encounter
        encounters = generate_encounters_data(seed_str, act)
        if room_type == "ELITE":
            enemy = encounters["elite"][min(encounter_idx, len(encounters["elite"])-1)]
        else:
            enemy = encounters["normal"][min(encounter_idx, len(encounters["normal"])-1)]

        result["enemy"] = enemy

        # Get HP
        hp_rng = Random(seed + floor)
        hp_range = ENEMY_HP_RANGES.get(enemy)
        if hp_range:
            hp = hp_rng.random_int_range(hp_range[0], hp_range[1])
            if ascension >= 7:
                hp = int(hp * 1.07)
            if ascension >= 16:
                hp = int(hp * 1.07)
            result["hp"] = hp

        # Get first move
        if enemy in ENEMY_FIRST_MOVES:
            result["first_move"] = ENEMY_FIRST_MOVES[enemy]

        # Gold
        room_for_gold = "elite" if room_type == "ELITE" else "normal"
        treasure_rng = rng_state.get_rng(RNGStream.TREASURE)
        result["gold"] = generate_gold_reward(treasure_rng, room_for_gold, ascension)

        # Cards
        card_rng = rng_state.get_rng(RNGStream.CARD)
        cards = generate_card_rewards(
            card_rng, reward_state, act=act, player_class="WATCHER",
            ascension=ascension, room_type=room_for_gold
        )
        result["cards"] = [{"name": c.name, "rarity": c.rarity.name, "upgraded": c.upgraded} for c in cards]

        # Potion
        potion_rng = rng_state.get_rng(RNGStream.POTION)
        dropped, potion = check_potion_drop(potion_rng, reward_state, room_for_gold)
        if dropped and potion:
            result["potion"] = {"name": potion.name, "rarity": potion.rarity.name}

        # Elite relic
        if room_type == "ELITE":
            relic_rng = rng_state.get_rng(RNGStream.RELIC)
            relic = generate_elite_relic_reward(relic_rng, reward_state, "WATCHER", act=act)
            if relic:
                result["relic"] = {"name": relic.name, "tier": relic.tier.name}

    elif room_type == "SHOP":
        result["note"] = "Shop - Consumes ~12 cardRng calls"
        try:
            merchant_rng = rng_state.get_rng(RNGStream.MERCHANT)
            shop = generate_shop_inventory(merchant_rng, reward_state, act=act, player_class="WATCHER", ascension=ascension)
            result["shop"] = {
                "cards": [(c.name, p) for c, p in shop.colored_cards],
                "colorless": [(c.name, p) for c, p in shop.colorless_cards],
                "relics": [(r.name, p) for r, p in shop.relics],
                "potions": [(p.name, pr) for p, pr in shop.potions],
                "purge_cost": shop.purge_cost,
            }
        except:
            result["shop"] = {"error": "Could not generate"}

    elif room_type == "REST":
        result["note"] = "Rest site - Heal 30% (22.5% at A5+), Upgrade, Smith, Dig, Toke, Lift"

    elif room_type == "TREASURE":
        result["note"] = "Treasure chest"

    elif room_type == "EVENT":
        result["note"] = "? Room - Event or combat (A14+ more combats)"

    return result


# =============================================================================
# ROUTES
# =============================================================================

@app.route('/')
def index():
    return render_template('index.html')


@app.route('/api/seed/<seed_str>')
def get_seed_data(seed_str):
    """Get complete seed prediction data."""
    act = request.args.get('act', 1, type=int)
    ascension = request.args.get('ascension', 20, type=int)

    seed = seed_to_long(seed_str.upper())

    data = {
        "seed": seed_str.upper(),
        "seed_value": seed,
        "ascension": ascension,
        "act": act,
        "neow_options": generate_neow_options(seed),
        "map": generate_map_data(seed_str, act, ascension),
        "encounters": generate_encounters_data(seed_str, act),
        "boss": BOSS_DATA.get(act, [{}])[0],  # Simplified
    }

    return jsonify(data)


@app.route('/api/floor/<seed_str>/<int:floor>')
def get_floor_data(seed_str, floor):
    """Get detailed data for a specific floor."""
    act = request.args.get('act', 1, type=int)
    room_type = request.args.get('type', 'MONSTER')
    encounter_idx = request.args.get('idx', 0, type=int)
    ascension = request.args.get('ascension', 20, type=int)

    data = generate_floor_rewards(seed_str, floor, room_type, encounter_idx, act, ascension)
    return jsonify(data)


@app.route('/api/path/<seed_str>', methods=['POST'])
def predict_path(seed_str):
    """
    Predict ? room outcomes for a given path.

    POST body: { "path": [{"x": 0, "y": 0, "type": "MONSTER"}, ...] }
    Returns: { "event_predictions": {"0,1": {"outcome": "EVENT", "event_name": "The Cleric"}, ...} }
    """
    act = request.args.get('act', 1, type=int)
    path_data = request.get_json()

    if not path_data or 'path' not in path_data:
        return jsonify({"error": "Missing path data"}), 400

    path = path_data['path']

    # Extract ? rooms from path in order
    event_nodes = []
    for node in path:
        if node.get('type') == 'EVENT':
            event_nodes.append((node['x'], node['y'], node['y'] + 1))  # floor = y + 1

    # Predict outcomes
    predictions = predict_event_rooms(seed_str, event_nodes, act)

    # Convert tuple keys to string for JSON
    result = {}
    for (x, y), pred in predictions.items():
        result[f"{x},{y}"] = pred

    return jsonify({"event_predictions": result})


# =============================================================================
# DECISION TREE & EV CALCULATION ENDPOINTS
# =============================================================================

@app.route('/api/decision-tree/<seed_str>', methods=['POST'])
def get_decision_tree(seed_str):
    """
    Generate a branching decision tree for a given seed.

    POST body: {
        "floor": int,           # Current floor
        "max_depth": int,       # How deep to branch (default 3)
        "prune_threshold": float  # Prune branches with < this probability (default 0.05)
    }

    Returns: Decision tree with branches and EV estimates
    """
    data = request.get_json() or {}
    floor = data.get('floor', 1)
    max_depth = min(data.get('max_depth', 3), 5)  # Cap at 5 for performance
    prune_threshold = data.get('prune_threshold', 0.05)

    seed = seed_to_long(seed_str.upper())
    map_data = generate_map_data(seed_str, act=1)

    # Build decision tree from map
    def build_node(x, y, depth, path_so_far):
        if depth >= max_depth or y >= 15:
            return None

        # Find this node in map
        node_info = None
        for n in map_data['nodes']:
            if n['x'] == x and n['y'] == y:
                node_info = n
                break

        if not node_info:
            return None

        # Find outgoing edges
        children = []
        for edge in map_data['edges']:
            if edge['src_x'] == x and edge['src_y'] == y:
                child = build_node(edge['dst_x'], edge['dst_y'], depth + 1, path_so_far + [(x, y)])
                if child:
                    children.append(child)

        # Calculate simple heuristic EV based on room type
        ev_heuristics = {
            'MONSTER': -0.1,     # Small HP loss expected
            'ELITE': -0.3,       # Higher HP loss but better rewards
            'REST': 0.2,         # HP recovery
            'SHOP': 0.0,         # Neutral
            'EVENT': 0.0,        # Variable
            'TREASURE': 0.1,     # Small bonus
            'BOSS': -0.5,        # High risk
        }
        ev = ev_heuristics.get(node_info['type'], 0.0)

        # Estimate win probability (simplified heuristic)
        base_win = 0.5
        if node_info['type'] == 'ELITE':
            base_win -= 0.1
        elif node_info['type'] == 'REST':
            base_win += 0.05

        return {
            'id': f"{x},{y}",
            'x': x,
            'y': y,
            'type': node_info['type'],
            'symbol': node_info['symbol'],
            'ev': round(ev, 2),
            'winProbability': round(base_win, 2),
            'children': children,
            'isExpanded': depth < 2,  # Auto-expand first 2 levels
            'isPruned': False,
        }

    # Find starting nodes (y=0)
    roots = []
    for edge in map_data['edges']:
        if edge['src_y'] == 0:
            root = build_node(edge['src_x'], edge['src_y'], 0, [])
            if root and root not in roots:
                roots.append(root)

    # Dedupe roots by id
    seen_ids = set()
    unique_roots = []
    for r in roots:
        if r['id'] not in seen_ids:
            seen_ids.add(r['id'])
            unique_roots.append(r)

    return jsonify({
        'seed': seed_str.upper(),
        'floor': floor,
        'tree': unique_roots,
    })


@app.route('/api/path-ev/<seed_str>', methods=['POST'])
def calculate_path_ev(seed_str):
    """
    Calculate expected value for a specific path.

    POST body: {
        "path": [{"x": int, "y": int, "type": str}, ...]
    }

    Returns: EV breakdown by floor
    """
    data = request.get_json() or {}
    path = data.get('path', [])
    ascension = data.get('ascension', 20)

    if not path:
        return jsonify({'error': 'Path required'}), 400

    ev_breakdown = []
    cumulative_ev = 0.0
    estimated_hp = 72  # Watcher starting HP
    estimated_gold = 99  # Starting gold

    for i, node in enumerate(path):
        floor = node.get('y', i) + 1
        room_type = node.get('type', 'MONSTER')

        # Calculate EV impact based on room type
        hp_change = 0
        gold_change = 0
        ev_impact = 0.0

        if room_type == 'MONSTER':
            # Estimate damage taken (varies by floor and ascension)
            hp_change = -8 if ascension >= 10 else -5
            gold_change = 15
            ev_impact = -0.1
        elif room_type == 'ELITE':
            hp_change = -20 if ascension >= 10 else -15
            gold_change = 30
            ev_impact = 0.1  # Better rewards offset HP loss
        elif room_type == 'REST':
            heal_amount = int(estimated_hp * 0.3)
            hp_change = heal_amount
            ev_impact = 0.2
        elif room_type == 'SHOP':
            gold_change = -50  # Assume average purchase
            ev_impact = 0.05
        elif room_type == 'TREASURE':
            gold_change = 25
            ev_impact = 0.05
        elif room_type == 'EVENT':
            # Events are variable
            ev_impact = 0.0

        estimated_hp = max(1, min(estimated_hp + hp_change, 72))
        estimated_gold = max(0, estimated_gold + gold_change)
        cumulative_ev += ev_impact

        ev_breakdown.append({
            'floor': floor,
            'x': node.get('x'),
            'y': node.get('y'),
            'type': room_type,
            'hp_change': hp_change,
            'gold_change': gold_change,
            'ev_impact': round(ev_impact, 2),
            'cumulative_ev': round(cumulative_ev, 2),
            'estimated_hp': estimated_hp,
            'estimated_gold': estimated_gold,
        })

    return jsonify({
        'seed': seed_str.upper(),
        'path_length': len(path),
        'total_ev': round(cumulative_ev, 2),
        'final_hp': estimated_hp,
        'final_gold': estimated_gold,
        'breakdown': ev_breakdown,
    })


@app.route('/api/combat-preview/<seed_str>', methods=['POST'])
def combat_preview(seed_str):
    """
    Preview combat outcomes for a specific encounter.

    POST body: {
        "floor": int,
        "room_type": str,  # "MONSTER" or "ELITE"
        "encounter_idx": int,
        "player_hp": int,
        "deck": [str],     # Card names
        "relics": [str],   # Relic names
    }

    Returns: Combat prediction with expected HP loss, turns, etc.
    """
    data = request.get_json() or {}
    floor = data.get('floor', 1)
    room_type = data.get('room_type', 'MONSTER')
    encounter_idx = data.get('encounter_idx', 0)
    player_hp = data.get('player_hp', 72)
    ascension = data.get('ascension', 20)

    seed = seed_to_long(seed_str.upper())

    # Get encounter info
    encounters = generate_encounters_data(seed_str, act=1)
    if room_type == 'ELITE':
        enemy = encounters['elite'][min(encounter_idx, len(encounters['elite'])-1)]
    else:
        enemy = encounters['normal'][min(encounter_idx, len(encounters['normal'])-1)]

    # Get enemy HP
    hp_rng = Random(seed + floor)
    hp_range = ENEMY_HP_RANGES.get(enemy, (30, 40))
    enemy_hp = hp_rng.random_int_range(hp_range[0], hp_range[1])
    if ascension >= 7:
        enemy_hp = int(enemy_hp * 1.07)
    if ascension >= 16:
        enemy_hp = int(enemy_hp * 1.07)

    # Get first move
    first_move = ENEMY_FIRST_MOVES.get(enemy, {'move': 'Unknown', 'intent': 'UNKNOWN'})

    # Simple combat prediction heuristics
    incoming_damage = first_move.get('damage', 10)
    if ascension >= 2:
        incoming_damage = int(incoming_damage * 1.1)

    # Estimate turns to kill (very simplified)
    avg_damage_per_turn = 12  # Watcher average
    turns_to_kill = max(1, enemy_hp // avg_damage_per_turn)

    # Estimate damage taken
    expected_damage_taken = incoming_damage * (turns_to_kill // 2)

    return jsonify({
        'seed': seed_str.upper(),
        'floor': floor,
        'enemy': enemy,
        'enemy_hp': enemy_hp,
        'first_move': first_move,
        'prediction': {
            'expected_turns': turns_to_kill,
            'expected_damage_taken': expected_damage_taken,
            'kill_probability': 0.95 if room_type == 'MONSTER' else 0.85,
            'hp_after': max(1, player_hp - expected_damage_taken),
        }
    })


@app.route('/api/resource-projection/<seed_str>', methods=['POST'])
def resource_projection(seed_str):
    """
    Project resource changes along a path.

    POST body: {
        "path": [{"x": int, "y": int, "type": str}, ...],
        "starting_hp": int,
        "starting_gold": int,
        "starting_deck_size": int,
    }

    Returns: Resource projection chart data
    """
    data = request.get_json() or {}
    path = data.get('path', [])
    hp = data.get('starting_hp', 72)
    max_hp = data.get('max_hp', 72)
    gold = data.get('starting_gold', 99)
    deck_size = data.get('starting_deck_size', 10)
    ascension = data.get('ascension', 20)

    projection = []
    relics_collected = 0

    for i, node in enumerate(path):
        floor = node.get('y', i) + 1
        room_type = node.get('type', 'MONSTER')

        # Simulate resource changes
        if room_type == 'MONSTER':
            hp = max(1, hp - 8)
            gold += 15
            deck_size += 1  # Card reward
        elif room_type == 'ELITE':
            hp = max(1, hp - 18)
            gold += 30
            deck_size += 1
            relics_collected += 1
        elif room_type == 'REST':
            heal = int(max_hp * 0.3)
            if ascension >= 5:
                heal = int(max_hp * 0.225)
            hp = min(max_hp, hp + heal)
        elif room_type == 'SHOP':
            gold = max(0, gold - 75)  # Assume card + removal
            # deck_size stays same (buy card, remove card)
        elif room_type == 'TREASURE':
            gold += 25
            relics_collected += 0.5  # Maybe a relic
        elif room_type == 'EVENT':
            # Events are variable, small HP loss on average
            hp = max(1, hp - 3)

        projection.append({
            'floor': floor,
            'x': node.get('x'),
            'y': node.get('y'),
            'type': room_type,
            'hp': hp,
            'max_hp': max_hp,
            'gold': gold,
            'deck_size': deck_size,
            'relics': int(relics_collected),
        })

    return jsonify({
        'seed': seed_str.upper(),
        'projection': projection,
        'summary': {
            'final_hp': hp,
            'final_gold': gold,
            'final_deck_size': deck_size,
            'total_relics': int(relics_collected),
            'floors_traversed': len(path),
        }
    })


# =============================================================================
# CORS SUPPORT FOR REACT FRONTEND
# =============================================================================

@app.after_request
def after_request(response):
    """Add CORS headers for React frontend development."""
    response.headers.add('Access-Control-Allow-Origin', '*')
    response.headers.add('Access-Control-Allow-Headers', 'Content-Type,Authorization')
    response.headers.add('Access-Control-Allow-Methods', 'GET,PUT,POST,DELETE,OPTIONS')
    return response


if __name__ == '__main__':
    os.makedirs(os.path.join(_ui_dir, 'templates'), exist_ok=True)
    os.makedirs(os.path.join(_ui_dir, 'static'), exist_ok=True)
    app.run(debug=True, port=5001)
