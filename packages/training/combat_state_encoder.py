"""
Combat state encoder for GPU-accelerated MCTS evaluation.

Encodes a CombatEngine state into a fixed-size float32 observation vector
compatible with the strategic model's input_dim=260. This reuses the same
value head that already estimates P(win) from run-level observations.

The encoding is intentionally lightweight -- it doesn't need to capture every
nuance of combat state. MCTS only needs a rough value estimate to guide its
search. The simulation itself handles exact mechanics.

Layout (260 dims, zero-padded):
    [0..7]    Player scalars (8)
    [8..11]   Stance one-hot (4)
    [12..41]  Enemy features (5 enemies x 6 dims = 30)
    [42..53]  Hand summary (12)
    [54..69]  Power features (16)
    [70..79]  Combat context (10)
    [80..259] Zero padding to match input_dim=260
"""

from __future__ import annotations

from typing import Any, Dict, Optional, Tuple

import numpy as np

# Target observation size (matches StrategicNet input_dim default)
COMBAT_OBS_DIM = 260

# Number of enemies to encode
MAX_ENEMIES = 5

# Features per enemy: hp_ratio, max_hp_norm, block_norm, move_damage_norm, move_hits_norm, alive
FEATURES_PER_ENEMY = 6

# Important powers to track (index -> power name)
TRACKED_POWERS = [
    "Strength", "Dexterity", "Vulnerable", "Weakened",
    "Frail", "Ritual", "MentalFortress", "Rushdown",
]

# Stance encoding
STANCE_IDS = ("Neutral", "Wrath", "Calm", "Divinity")


def encode_combat_state(engine: Any, input_dim: int = COMBAT_OBS_DIM) -> np.ndarray:
    """Encode a CombatEngine state into a flat observation vector.

    Args:
        engine: CombatEngine instance with a .state attribute (CombatState).
        input_dim: Target vector size (zero-padded if encoding is shorter).

    Returns:
        float32 array of shape (input_dim,).
    """
    obs = np.zeros(input_dim, dtype=np.float32)
    state = engine.state
    player = state.player

    # --- Player scalars [0..7] ---
    obs[0] = player.hp / max(player.max_hp, 1)          # HP ratio
    obs[1] = float(player.max_hp) / 100.0               # Normalized max HP
    obs[2] = float(player.block) / 50.0                 # Normalized block
    obs[3] = float(state.energy) / 5.0                  # Normalized energy
    obs[4] = float(state.max_energy) / 5.0              # Max energy
    obs[5] = float(len(state.hand)) / 12.0              # Hand size ratio
    obs[6] = float(state.turn) / 10.0                   # Turn number
    obs[7] = float(state.mantra) / 10.0                 # Mantra progress

    # --- Stance one-hot [8..11] ---
    stance = state.stance
    for i, s in enumerate(STANCE_IDS):
        if stance == s:
            obs[8 + i] = 1.0
            break

    # --- Enemy features [12..41] ---
    offset = 12
    live_enemies = [e for e in state.enemies if e.hp > 0 and not getattr(e, "is_escaping", False)]
    for i, enemy in enumerate(live_enemies[:MAX_ENEMIES]):
        base = offset + i * FEATURES_PER_ENEMY
        obs[base] = enemy.hp / max(enemy.max_hp, 1)     # HP ratio
        obs[base + 1] = float(enemy.max_hp) / 200.0     # Normalized max HP
        obs[base + 2] = float(enemy.block) / 50.0       # Normalized block
        obs[base + 3] = float(enemy.move_damage) / 30.0 # Normalized damage
        obs[base + 4] = float(enemy.move_hits) / 5.0    # Normalized hits
        obs[base + 5] = 1.0                              # Alive flag

    # --- Hand summary [42..53] ---
    offset = 42
    hand = state.hand
    n_attacks = 0
    n_skills = 0
    n_powers = 0
    total_cost = 0
    for card_id in hand:
        # Rough card type classification from card ID conventions
        base_id = card_id.rstrip("+")
        if _is_attack(base_id):
            n_attacks += 1
        elif _is_power(base_id):
            n_powers += 1
        else:
            n_skills += 1
        cost = state.card_costs.get(card_id, _default_card_cost(base_id))
        total_cost += cost

    obs[offset] = float(n_attacks) / 6.0
    obs[offset + 1] = float(n_skills) / 6.0
    obs[offset + 2] = float(n_powers) / 3.0
    obs[offset + 3] = float(total_cost) / 15.0
    obs[offset + 4] = float(len(state.draw_pile)) / 30.0
    obs[offset + 5] = float(len(state.discard_pile)) / 30.0
    obs[offset + 6] = float(len(state.exhaust_pile)) / 15.0
    obs[offset + 7] = float(state.cards_played_this_turn) / 10.0
    obs[offset + 8] = float(state.attacks_played_this_turn) / 5.0
    obs[offset + 9] = float(len(hand)) / 12.0
    obs[offset + 10] = float(state.energy) / float(max(total_cost, 1))  # Energy adequacy
    # Number of playable cards (cost <= energy)
    playable = sum(
        1 for c in hand
        if state.card_costs.get(c, _default_card_cost(c.rstrip("+"))) <= state.energy
    )
    obs[offset + 11] = float(playable) / max(len(hand), 1)

    # --- Power features [54..69] ---
    offset = 54
    for i, power_name in enumerate(TRACKED_POWERS):
        # Player powers
        obs[offset + i] = float(player.statuses.get(power_name, 0)) / 10.0

    # Enemy aggregate powers (most dangerous enemy)
    offset_enemy_powers = offset + len(TRACKED_POWERS)
    for i, power_name in enumerate(TRACKED_POWERS[:MAX_ENEMIES]):
        total = sum(e.statuses.get(power_name, 0) for e in live_enemies)
        if offset_enemy_powers + i < 70:
            obs[offset_enemy_powers + i] = float(total) / 10.0

    # --- Combat context [70..79] ---
    offset = 70
    # Total enemy HP ratio
    total_enemy_hp = sum(e.hp for e in live_enemies)
    total_enemy_max = sum(max(e.max_hp, 1) for e in state.enemies) if state.enemies else 1
    obs[offset] = total_enemy_hp / max(total_enemy_max, 1)
    obs[offset + 1] = float(len(live_enemies)) / 5.0

    # Incoming damage estimate
    incoming = 0
    for e in live_enemies:
        if e.move_damage > 0:
            incoming += e.move_damage * e.move_hits
    obs[offset + 2] = float(incoming) / 50.0

    # Danger: incoming vs (block + hp)
    effective_hp = player.hp + player.block
    obs[offset + 3] = float(incoming) / max(effective_hp, 1)

    # Wrath modifier awareness
    if stance == "Wrath":
        obs[offset + 4] = 1.0  # In wrath (deals and takes 2x)
    obs[offset + 5] = float(state.total_damage_dealt) / 200.0
    obs[offset + 6] = float(state.total_damage_taken) / 100.0

    # Combat type
    combat_type = state.combat_type
    obs[offset + 7] = 1.0 if combat_type == "elite" else 0.0
    obs[offset + 8] = 1.0 if combat_type == "boss" else 0.0
    obs[offset + 9] = float(state.total_cards_played) / 50.0

    return obs


# ---------------------------------------------------------------------------
# Card type heuristics (fast, no lookups needed)
# ---------------------------------------------------------------------------

# Known power cards (base IDs without upgrade suffix)
_KNOWN_POWERS = frozenset({
    "Blasphemy", "DevaForm", "Devotion", "Establishment",
    "Foresight", "LikeWater", "MasterReality", "MentalFortress",
    "Rushdown", "Study", "WaveOfTheHand", "Wireheading",
    "BattleHymn", "Worship",
    # Generic
    "Inflame", "Demon Form", "Metallicize", "Corruption",
    "Defragment", "Creative AI", "Echo Form",
    "Footwork", "Noxious Fumes", "Accuracy", "A Thousand Cuts",
})

# Known attack cards (very common ones)
_KNOWN_ATTACKS = frozenset({
    "Strike", "Strike_P", "Eruption", "Tantrum", "Ragnarok",
    "FlurryOfBlows", "CrushJoints", "Conclude", "FlyingSleeves",
    "ReachHeaven", "SashWhip", "SignatureMove", "TalkToTheHand",
    "Wallop", "Weave", "WheelKick", "WindmillStrike",
    "BowlingBash", "EmptyFist", "FollowUp", "JustLucky",
    "SandsOfTime", "SimmeringFury",
    # Generic
    "Bash", "Anger", "Bludgeon", "Carnage", "Cleave",
    "Headbutt", "HeavyBlade", "Pummel",
})


def _is_attack(base_id: str) -> bool:
    """Heuristic: is this card an attack?"""
    return base_id in _KNOWN_ATTACKS or "Strike" in base_id


def _is_power(base_id: str) -> bool:
    """Heuristic: is this card a power?"""
    return base_id in _KNOWN_POWERS


def _default_card_cost(base_id: str) -> int:
    """Default card cost heuristic (most cards cost 1)."""
    # 0-cost cards
    if base_id in ("Miracle", "Insight", "Smite", "Safety", "Weave",
                    "FlurryOfBlows", "CrushJoints", "Defend_P", "Strike_P"):
        return 0
    # 2-cost cards
    if base_id in ("WheelKick", "Ragnarok", "Conclude", "Wallop",
                    "Worship", "DevaForm", "Blasphemy", "Vault",
                    "Omniscience", "SpiritShield", "Scrawl"):
        return 2
    # 3-cost cards
    if base_id in ("DevaForm", "Brilliance"):
        return 3
    return 1


def make_mcts_policy_fn(client: Any, input_dim: int = COMBAT_OBS_DIM):
    """Create a policy_fn for GumbelMCTS that routes evaluation through InferenceClient.

    The returned function:
    1. Encodes the CombatEngine state into a flat observation
    2. Sends it to the InferenceServer via the combat route
    3. Returns (uniform_priors, value_estimate)

    Priors are uniform because we don't have a combat-specific policy head.
    The value estimate comes from the strategic model's value head, which
    already knows how to estimate P(win) from game state features.

    Args:
        client: InferenceClient instance (from get_client()).
        input_dim: Observation dimension (must match strategic model).

    Returns:
        Callable compatible with GumbelMCTS policy_fn signature:
            CombatEngine -> (Dict[action, float], float)
    """

    def policy_fn(engine: Any) -> Tuple[Dict[Any, float], float]:
        # Encode combat state
        obs = encode_combat_state(engine, input_dim=input_dim)

        # Get legal actions for uniform priors
        legal_actions = engine.get_legal_actions()
        if not legal_actions:
            return {}, 0.0

        uniform_prior = 1.0 / len(legal_actions)
        priors = {a: uniform_prior for a in legal_actions}

        # Send to InferenceServer via combat route
        legal_indices = np.arange(len(legal_actions), dtype=np.int32)
        result = client.infer_combat(obs, legal_indices)

        if result is None or not result.get("ok", False):
            # Fallback: heuristic value estimate
            value = _heuristic_combat_value(engine)
            return priors, value

        value = float(result.get("value", 0.0))
        # Map value from [-1, 1] (tanh output) to [0, 1] for MCTS
        value = (value + 1.0) / 2.0
        return priors, value

    return policy_fn


def _heuristic_combat_value(engine: Any) -> float:
    """Fallback heuristic value when InferenceServer is unavailable."""
    if engine.is_combat_over():
        if engine.is_victory():
            hp = engine.state.player.hp
            max_hp = max(engine.state.player.max_hp, 1)
            return 0.7 + 0.3 * (hp / max_hp)
        return 0.0

    state = engine.state
    player = state.player
    hp_ratio = player.hp / max(player.max_hp, 1)

    live_enemies = [e for e in state.enemies if e.hp > 0]
    total_enemy_hp = sum(e.hp for e in live_enemies)
    total_enemy_max = sum(max(1, e.max_hp) for e in state.enemies)

    if total_enemy_hp <= 0:
        return 0.85 + 0.15 * hp_ratio

    enemy_progress = 1.0 - (total_enemy_hp / max(total_enemy_max, 1))
    return max(0.0, min(1.0, 0.35 * hp_ratio + 0.30 * enemy_progress + 0.15))
