"""
State encoders for the two-model RL architecture.

RunStateEncoder: encodes run-level state for the strategic model (~250 dims, cached per floor).
CombatStateEncoder: encodes combat state for the combat net (~280 dims, per MCTS call).

Cards are encoded by EFFECTS (not identity) so the model generalizes across cards.
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional, Tuple

import numpy as np


# ---------------------------------------------------------------------------
# Card effect vector (18 dims per card)
# ---------------------------------------------------------------------------

# Powers whose application we track in the 3-dim compressed embedding.
# Ordered by frequency/impact for Watcher.
TOP_POWERS = [
    "Strength", "Dexterity", "Vulnerable", "Weakened",
    "Mantra", "Ritual", "MentalFortress", "Rushdown",
]

_STANCE_MAP = {"Wrath": 0, "Calm": 1, "Divinity": 2, "exit": 3}


def _card_effect_vector(card_data) -> np.ndarray:
    """Encode a card's static data into an 18-dim effect vector.

    Layout:
        [0]  energy_cost (normalized, -1 for X)
        [1]  base_damage (normalized)
        [2]  base_block (normalized)
        [3]  cards_drawn (from magic_number if draw effect)
        [4]  cards_discarded (from effects)
        [5]  aoe (1 if ALL_ENEMY target)
        [6]  exhaust
        [7]  ethereal
        [8]  type: attack
        [9]  type: skill
        [10] type: power
        [11] stance: wrath
        [12] stance: calm
        [13] stance: divinity
        [14] stance: exit
        [15] power_embed_0  (compressed from top powers)
        [16] power_embed_1
        [17] power_amount (magic_number when applying a power)
    """
    from packages.engine.content.cards import CardType, CardTarget

    v = np.zeros(18, dtype=np.float32)

    cost = card_data.cost
    v[0] = cost / 4.0 if cost >= 0 else -1.0

    dmg = card_data.base_damage
    v[1] = max(dmg, 0) / 40.0

    blk = card_data.base_block
    v[2] = max(blk, 0) / 30.0

    # Draw: heuristic — if effects list contains "draw" or magic is used for draw
    effects_lower = [e.lower() for e in card_data.effects]
    for e in effects_lower:
        if "draw" in e:
            v[3] = max(card_data.base_magic, 0) / 5.0
            break
    for e in effects_lower:
        if "discard" in e:
            v[4] = max(card_data.base_magic, 0) / 5.0
            break

    if card_data.target in (CardTarget.ALL_ENEMY, CardTarget.ALL):
        v[5] = 1.0
    v[6] = float(card_data.exhaust)
    v[7] = float(card_data.ethereal)

    # Type one-hot
    if card_data.card_type == CardType.ATTACK:
        v[8] = 1.0
    elif card_data.card_type == CardType.SKILL:
        v[9] = 1.0
    elif card_data.card_type == CardType.POWER:
        v[10] = 1.0

    # Stance
    if card_data.enter_stance:
        idx = _STANCE_MAP.get(card_data.enter_stance)
        if idx is not None and idx < 4:
            v[11 + idx] = 1.0
    if card_data.exit_stance:
        v[14] = 1.0

    # Power embedding — simple hash of effect names to 3 dims
    for i, pw in enumerate(TOP_POWERS[:3]):
        pw_lower = pw.lower()
        for e in effects_lower:
            if pw_lower in e:
                v[15 + i] = 1.0
                break

    # Power amount
    if card_data.card_type == CardType.POWER and card_data.base_magic > 0:
        v[17] = card_data.base_magic / 10.0

    return v


# Cache card effect vectors (they're static)
_CARD_EFFECT_CACHE: Dict[str, np.ndarray] = {}


def _get_card_effect(card_id: str) -> np.ndarray:
    """Get cached 18-dim effect vector for a card ID."""
    if card_id not in _CARD_EFFECT_CACHE:
        from packages.engine.content.cards import ALL_CARDS
        base_id = card_id.rstrip("+")
        card_data = ALL_CARDS.get(base_id)
        if card_data is None:
            _CARD_EFFECT_CACHE[card_id] = np.zeros(18, dtype=np.float32)
        else:
            _CARD_EFFECT_CACHE[card_id] = _card_effect_vector(card_data)
    return _CARD_EFFECT_CACHE[card_id]


# ---------------------------------------------------------------------------
# Relic catalog for one-hot encoding
# ---------------------------------------------------------------------------

_RELIC_CATALOG: Optional[List[str]] = None


def _get_relic_catalog() -> List[str]:
    global _RELIC_CATALOG
    if _RELIC_CATALOG is None:
        from packages.engine.content.relics import ALL_RELICS
        _RELIC_CATALOG = sorted(ALL_RELICS.keys())
    return _RELIC_CATALOG


_RELIC_TO_IDX: Optional[Dict[str, int]] = None


def _get_relic_index(relic_id: str) -> int:
    """Get index for relic one-hot. Returns -1 if unknown."""
    global _RELIC_TO_IDX
    if _RELIC_TO_IDX is None:
        catalog = _get_relic_catalog()
        _RELIC_TO_IDX = {r: i for i, r in enumerate(catalog)}
    return _RELIC_TO_IDX.get(relic_id, -1)


# ---------------------------------------------------------------------------
# RunStateEncoder
# ---------------------------------------------------------------------------

# Dimension breakdown:
#   6  HP/resources
#   3  keys
#  16  deck functional aggregate
# 181  relic binary flags
#  20  potion slots (5 slots x 4 functional dims)
#  21  map lookahead (3 rows x 7 room-type features)
#   4  progress features
#   3  recent combat HP losses
#   6  decision phase type (one-hot)
# ---
# 260  total (dims 248-259 encode boss/elite info in previously zero-padded space)

_CARD_EFFECT_DIM = 18
_MAX_POTION_SLOTS = 5
_POTION_FUNC_DIM = 4  # damage, heal, block, utility
_N_RELICS = 181
_MAP_ROWS = 3
_MAP_COLS = 7
_PHASE_DIM = 6  # path, card_pick, rest, shop, event, other

# Boss ID mapping for compact encoding (10 bosses → single normalized scalar)
_BOSS_ID_MAP = {
    "The Guardian": 0, "Hexaghost": 1, "Slime Boss": 2,      # Act 1
    "Automaton": 3, "Collector": 4, "Champ": 5,               # Act 2
    "Awakened One": 6, "Time Eater": 7, "Donu and Deca": 8,   # Act 3
    "Corrupt Heart": 9,                                        # Act 4
}

# Phase type mapping for the 6-dim one-hot
PHASE_TYPE_MAP = {
    "path": 0,
    "card_pick": 1,
    "rest": 2,
    "shop": 3,
    "event": 4,
    "other": 5,
}


class RunStateEncoder:
    """Encode run state for the strategic model.

    Produces a fixed-size vector from run_state that captures:
    - Resource levels (HP, gold, floor, act)
    - Deck composition (functional aggregate, not card identity)
    - Relic presence (one-hot over catalog)
    - Potion functional summary
    - Map lookahead
    - Progress features
    - Decision phase type (one-hot: path/card/rest/shop/event/other)
    """

    _BASE_DIM = 6 + 3 + 16 + _N_RELICS + (_MAX_POTION_SLOTS * _POTION_FUNC_DIM) + (_MAP_ROWS * _MAP_COLS) + 4 + 3
    # = 6 + 3 + 16 + 181 + 20 + 21 + 4 + 3 = 254
    RUN_DIM = _BASE_DIM + _PHASE_DIM  # 254 + 6 = 260

    def __init__(self):
        self._relic_catalog = _get_relic_catalog()
        self._n_relics = len(self._relic_catalog)

    def encode(
        self,
        run_state,
        phase_type: str = "other",
        boss_name: str = "",
        room_type: str = "",
    ) -> np.ndarray:
        """Encode run state into a fixed-size float32 vector.

        Args:
            run_state: The run state object.
            phase_type: Decision type — one of "path", "card_pick", "rest",
                        "shop", "event", "other". Used for 6-dim one-hot.
            boss_name: Current act's boss name (e.g. "Hexaghost"). Used for
                       boss ID encoding in progress dims.
            room_type: Current room type string (e.g. "boss", "elite").
                       Encodes is_boss/is_elite flags.
        """
        features = np.zeros(self.RUN_DIM, dtype=np.float32)
        off = 0

        # --- HP/resources (6 dims) ---
        max_hp = max(getattr(run_state, "max_hp", 1), 1)
        features[off] = run_state.current_hp / max_hp
        features[off + 1] = max_hp / 100.0
        features[off + 2] = getattr(run_state, "gold", 0) / 500.0
        features[off + 3] = getattr(run_state, "floor", 0) / 55.0
        features[off + 4] = getattr(run_state, "act", 1) / 3.0
        features[off + 5] = getattr(run_state, "ascension", 0) / 20.0
        off += 6

        # --- Keys (3 dims) ---
        features[off] = float(getattr(run_state, "has_ruby_key", False))
        features[off + 1] = float(getattr(run_state, "has_emerald_key", False))
        features[off + 2] = float(getattr(run_state, "has_sapphire_key", False))
        off += 3

        # --- Deck functional aggregate (16 dims) ---
        # Average effect vector across deck + deck size + type distribution
        deck = getattr(run_state, "deck", [])
        n_deck = len(deck)
        if n_deck > 0:
            # Accumulate effect vectors
            effect_sum = np.zeros(18, dtype=np.float32)
            n_attacks = 0
            n_skills = 0
            n_powers = 0
            n_upgraded = 0
            for card in deck:
                card_id = card.id if hasattr(card, "id") else str(card)
                ev = _get_card_effect(card_id)
                effect_sum += ev
                # Count types from the effect vector (indices 8,9,10)
                n_attacks += ev[8]
                n_skills += ev[9]
                n_powers += ev[10]
                if hasattr(card, "upgraded") and card.upgraded:
                    n_upgraded += 1

            # Average of first 8 effect dims (cost, damage, block, draw, discard, aoe, exhaust, ethereal)
            avg_effects = effect_sum[:8] / n_deck
            features[off:off + 8] = avg_effects

            # Deck composition (4 dims)
            features[off + 8] = n_deck / 40.0
            features[off + 9] = n_attacks / max(n_deck, 1)
            features[off + 10] = n_skills / max(n_deck, 1)
            features[off + 11] = n_powers / max(n_deck, 1)

            # Upgrade ratio + stance card density (4 dims)
            features[off + 12] = n_upgraded / max(n_deck, 1)
            # Stance cards: sum of stance dims (11-14) across deck
            features[off + 13] = effect_sum[11] / max(n_deck, 1)  # wrath density
            features[off + 14] = effect_sum[12] / max(n_deck, 1)  # calm density
            features[off + 15] = (effect_sum[13] + effect_sum[14]) / max(n_deck, 1)  # divinity/exit
        off += 16

        # --- Relic binary flags (181 dims) ---
        relics = getattr(run_state, "relics", [])
        for relic in relics:
            rid = relic.id if hasattr(relic, "id") else str(relic)
            idx = _get_relic_index(rid)
            if 0 <= idx < self._n_relics:
                features[off + idx] = 1.0
        off += self._n_relics

        # --- Potion slots (5 x 4 = 20 dims) ---
        potion_slots = getattr(run_state, "potion_slots", [])
        for i in range(min(len(potion_slots), _MAX_POTION_SLOTS)):
            slot = potion_slots[i]
            base = off + i * _POTION_FUNC_DIM
            if slot is None or (isinstance(slot, list) and (not slot or slot == ["empty"])):
                continue
            if isinstance(slot, str) and slot == "[empty]":
                continue
            # Functional encoding: presence + rough categorization
            features[base] = 1.0  # has potion
            potion_id = slot.id if hasattr(slot, "id") else str(slot)
            pid_lower = potion_id.lower()
            if any(w in pid_lower for w in ("fire", "explosive", "attack", "poison", "fear")):
                features[base + 1] = 1.0  # damage potion
            if any(w in pid_lower for w in ("fairy", "fruit", "blood", "regen")):
                features[base + 2] = 1.0  # heal potion
            if any(w in pid_lower for w in ("block", "ghost", "ancient")):
                features[base + 3] = 1.0  # defensive potion
        off += _MAX_POTION_SLOTS * _POTION_FUNC_DIM

        # --- Map lookahead (3 rows x 7 features = 21 dims) ---
        # Encode next 3 reachable floor types as room-type distribution
        # Room types: monster, elite, rest, shop, event, treasure, boss
        _ROOM_TYPES = {"monster": 0, "elite": 1, "rest": 2, "shop": 3, "event": 4, "treasure": 5, "boss": 6}
        act_maps = getattr(run_state, "act_maps", None)
        map_pos = getattr(run_state, "map_position", None)
        if act_maps and map_pos is not None:
            current_act = getattr(run_state, "act", 1)
            current_map = act_maps.get(current_act) if isinstance(act_maps, dict) else None
            if current_map and hasattr(current_map, "nodes"):
                # Simplified: just encode the next few floors' room type distribution
                current_floor = getattr(run_state, "floor", 0)
                for row_i in range(_MAP_ROWS):
                    target_floor = current_floor + row_i + 1
                    base = off + row_i * _MAP_COLS
                    # Find nodes at this floor
                    if hasattr(current_map, "get_nodes_at_floor"):
                        nodes = current_map.get_nodes_at_floor(target_floor)
                    else:
                        nodes = [n for n in current_map.nodes if getattr(n, "floor", -1) == target_floor]
                    for node in nodes:
                        room_type = getattr(node, "room_type", "monster")
                        if isinstance(room_type, str):
                            rt_lower = room_type.lower()
                            for rt_name, rt_idx in _ROOM_TYPES.items():
                                if rt_name in rt_lower:
                                    features[base + rt_idx] += 1.0
                                    break
                    # Normalize per row
                    row_sum = features[base:base + _MAP_COLS].sum()
                    if row_sum > 0:
                        features[base:base + _MAP_COLS] /= row_sum
        off += _MAP_ROWS * _MAP_COLS

        # --- Progress features + boss context (4 dims) ---
        features[off] = getattr(run_state, "combats_won", 0) / 20.0
        features[off + 1] = getattr(run_state, "elites_killed", 0) / 5.0
        features[off + 2] = getattr(run_state, "bosses_killed", 0) / 3.0
        # Boss identity for current act (replaces perfect_floors — low signal)
        boss_id = _BOSS_ID_MAP.get(boss_name, -1)
        features[off + 3] = (boss_id + 1) / 11.0 if boss_id >= 0 else 0.0
        off += 4

        # --- HP deficit + floor type flags (3 dims) ---
        features[off] = 1.0 - (run_state.current_hp / max_hp)  # HP deficit
        # is_boss / is_elite flags (replace redundant binary HP thresholds)
        rt_lower = room_type.lower() if room_type else ""
        features[off + 1] = 1.0 if "boss" in rt_lower else 0.0
        features[off + 2] = 1.0 if rt_lower in ("elite", "e") else 0.0
        off += 3

        # --- Decision phase type (6 dims one-hot) ---
        phase_idx = PHASE_TYPE_MAP.get(phase_type, PHASE_TYPE_MAP["other"])
        features[off + phase_idx] = 1.0
        off += _PHASE_DIM

        return features


# ---------------------------------------------------------------------------
# CombatStateEncoder
# ---------------------------------------------------------------------------

# Dimension breakdown:
#   9   energy, block, turn, stance, mantra
#   1   mantra
#  40   active powers (20 x 2)
# 180   hand cards (10 x 18 effect dims)
#  60   enemies (5 x 12)
#   6   draw pile summary
#   2   discard summary
# ---
# 298   total (approximately 280)

_MAX_HAND = 10
_MAX_ENEMIES = 5
_ENEMY_DIM = 12
_MAX_POWERS = 20
_POWER_DIM = 2  # id_hash, amount

# Power name → stable index (for one-hot-ish encoding)
POWER_IDS = [
    "Strength", "Dexterity", "Vulnerable", "Weakened", "Frail",
    "MentalFortress", "Rushdown", "Vigor", "Mantra",
    "Plated Armor", "Metallicize", "Thorns", "Ritual",
    "Retain", "Artifact", "Intangible", "Barricade",
    "Rage", "Anger", "Regeneration",
]
_POWER_TO_IDX = {p: i for i, p in enumerate(POWER_IDS)}


class CombatStateEncoder:
    """Encode combat state for the combat net.

    Takes a CombatEngine and produces a fixed-size float32 vector.
    """

    COMBAT_DIM = 9 + 1 + (_MAX_POWERS * _POWER_DIM) + (_MAX_HAND * _CARD_EFFECT_DIM) + (_MAX_ENEMIES * _ENEMY_DIM) + 6 + 2
    # = 9 + 1 + 40 + 180 + 60 + 6 + 2 = 298

    def encode(self, engine) -> np.ndarray:
        """Encode a CombatEngine state into a fixed-size float32 vector."""
        features = np.zeros(self.COMBAT_DIM, dtype=np.float32)
        off = 0
        state = engine.state
        player = state.player

        # --- Energy/block/turn/stance (9 dims) ---
        features[off] = float(state.energy) / 4.0
        features[off + 1] = float(player.block) / 50.0
        features[off + 2] = float(getattr(state, "turn", 1)) / 20.0
        features[off + 3] = float(len(state.hand)) / 10.0
        features[off + 4] = float(len(state.draw_pile)) / 30.0
        features[off + 5] = float(len(state.discard_pile)) / 30.0
        features[off + 6] = float(len(getattr(state, "exhaust_pile", []))) / 20.0
        # Stance one-hot (4 values packed into 2 dims for efficiency)
        stance = getattr(state, "stance", "Neutral")
        if stance == "Wrath":
            features[off + 7] = 1.0
        elif stance == "Calm":
            features[off + 7] = -1.0
        elif stance == "Divinity":
            features[off + 8] = 1.0
        # Neutral = (0, 0)
        off += 9

        # --- Mantra (1 dim) ---
        mantra = 0
        statuses = getattr(player, "statuses", {})
        if isinstance(statuses, dict):
            mantra = statuses.get("Mantra", 0)
        features[off] = float(mantra) / 10.0
        off += 1

        # --- Active powers top 20 x 2 (40 dims) ---
        if isinstance(statuses, dict):
            for power_name, amount in statuses.items():
                idx = _POWER_TO_IDX.get(power_name, -1)
                if 0 <= idx < _MAX_POWERS:
                    base = off + idx * _POWER_DIM
                    features[base] = 1.0  # present
                    features[base + 1] = float(amount) / 10.0  # amount
        off += _MAX_POWERS * _POWER_DIM

        # --- Hand cards: 10 x 18 effect dims (180 dims) ---
        hand = list(state.hand)
        for i in range(min(len(hand), _MAX_HAND)):
            card = hand[i]
            card_id = card.id if hasattr(card, "id") else str(card)
            ev = _get_card_effect(card_id)
            base = off + i * _CARD_EFFECT_DIM
            features[base:base + _CARD_EFFECT_DIM] = ev
            # Adjust cost for current turn cost
            if hasattr(card, "cost_for_turn") and card.cost_for_turn is not None:
                features[base] = card.cost_for_turn / 4.0
        off += _MAX_HAND * _CARD_EFFECT_DIM

        # --- Enemy features: 5 x 12 (60 dims) ---
        enemies = state.enemies
        for i in range(min(len(enemies), _MAX_ENEMIES)):
            enemy = enemies[i]
            base = off + i * _ENEMY_DIM
            emax = max(getattr(enemy, "max_hp", 1), 1)
            features[base] = enemy.hp / emax  # hp ratio
            features[base + 1] = float(emax) / 300.0  # max hp normalized
            features[base + 2] = float(enemy.block) / 50.0
            features[base + 3] = float(getattr(enemy, "move_damage", 0) or 0) / 40.0
            features[base + 4] = float(getattr(enemy, "move_hits", 0) or 0) / 5.0
            features[base + 5] = 1.0 if enemy.hp > 0 else 0.0  # alive

            # Enemy statuses (6 dims)
            e_statuses = getattr(enemy, "statuses", {})
            if isinstance(e_statuses, dict):
                features[base + 6] = float(e_statuses.get("Vulnerable", 0)) / 5.0
                features[base + 7] = float(e_statuses.get("Weakened", 0)) / 5.0
                features[base + 8] = float(e_statuses.get("Strength", 0)) / 10.0
                features[base + 9] = float(e_statuses.get("Ritual", 0)) / 5.0
                features[base + 10] = float(e_statuses.get("Artifact", 0)) / 3.0
                features[base + 11] = float(e_statuses.get("Intangible", 0)) / 3.0
        off += _MAX_ENEMIES * _ENEMY_DIM

        # --- Draw pile summary (6 dims) ---
        draw = list(state.draw_pile)
        if draw:
            n_draw = len(draw)
            draw_atk = sum(1 for c in draw if _get_card_effect(c.id if hasattr(c, "id") else str(c))[8] > 0)
            draw_skl = sum(1 for c in draw if _get_card_effect(c.id if hasattr(c, "id") else str(c))[9] > 0)
            features[off] = n_draw / 30.0
            features[off + 1] = draw_atk / max(n_draw, 1)
            features[off + 2] = draw_skl / max(n_draw, 1)
            # Average damage/block in draw pile
            draw_dmg = sum(_get_card_effect(c.id if hasattr(c, "id") else str(c))[1] for c in draw)
            draw_blk = sum(_get_card_effect(c.id if hasattr(c, "id") else str(c))[2] for c in draw)
            features[off + 3] = draw_dmg / max(n_draw, 1)
            features[off + 4] = draw_blk / max(n_draw, 1)
            # Stance cards in draw
            draw_stance = sum(1 for c in draw if _get_card_effect(c.id if hasattr(c, "id") else str(c))[11:15].any())
            features[off + 5] = draw_stance / max(n_draw, 1)
        off += 6

        # --- Discard summary (2 dims) ---
        discard = list(state.discard_pile)
        features[off] = len(discard) / 30.0
        if discard:
            disc_dmg = sum(_get_card_effect(c.id if hasattr(c, "id") else str(c))[1] for c in discard)
            features[off + 1] = disc_dmg / max(len(discard), 1)
        off += 2

        return features
