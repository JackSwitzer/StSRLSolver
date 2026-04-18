"""
RL observation encoding utilities for the Slay the Spire engine.

Provides functions to convert JSON observation dicts (from GameRunner.get_observation)
into flat numpy feature vectors suitable for RL training, and the inverse.

Feature layout (observation_to_array):
    [0]   current_hp (float, normalized 0..1 by max_hp)
    [1]   max_hp (float, raw)
    [2]   gold (float, raw)
    [3]   floor (float, raw)
    [4]   act (float, raw)
    [5]   ascension (float, raw)
    [6]   has_ruby_key (0/1)
    [7]   has_emerald_key (0/1)
    [8]   has_sapphire_key (0/1)
    [9..9+N_CARDS-1]  deck composition (count of each card id)
    [9+N_CARDS..9+N_CARDS+N_RELICS-1]  relic presence (0/1 per relic id)
    [9+N_CARDS+N_RELICS..9+N_CARDS+N_RELICS+N_POTIONS-1]  potion slot encoding
    [combat block]  combat features (energy, block, stance, hand, enemies...)

The exact sizes depend on the card/relic/potion catalogs passed at construction
time.  Default catalogs are derived from the engine content modules.

Usage:
    from packages.engine.rl_observations import ObservationEncoder

    encoder = ObservationEncoder()
    obs = runner.get_observation()
    arr = encoder.observation_to_array(obs)
    obs2 = encoder.array_to_observation(arr)
"""

from __future__ import annotations

from typing import Dict, Any, List, Optional, Tuple
import numpy as np


# ---------------------------------------------------------------------------
# Stance encoding
# ---------------------------------------------------------------------------

STANCE_IDS: Tuple[str, ...] = ("Neutral", "Wrath", "Calm", "Divinity")


def _stance_to_index(stance: str) -> int:
    """Map stance string to one-hot index."""
    try:
        return STANCE_IDS.index(stance)
    except ValueError:
        return 0  # fallback to Neutral


# ---------------------------------------------------------------------------
# Observation encoder
# ---------------------------------------------------------------------------

class ObservationEncoder:
    """Converts observation dicts to/from flat numpy arrays.

    The encoder maintains a fixed feature layout determined at construction
    time by the card, relic, and potion catalogs.  Unknown IDs encountered
    at encode time are silently ignored (they don't contribute features).

    Args:
        card_catalog: Ordered list of all known card IDs (base, not upgraded).
            Upgraded cards are tracked as separate entries with ``+`` suffix.
            If ``None``, a default catalog is built from the engine.
        relic_catalog: Ordered list of all known relic IDs.
        potion_catalog: Ordered list of all known potion IDs.
        max_enemies: Maximum number of enemies to encode (default 5).
        max_hand_size: Maximum hand size to encode (default 12).
        max_potion_slots: Maximum potion slots (default 5).
    """

    # Feature block names for documentation/debugging
    BLOCK_NAMES = (
        "run_scalars",
        "keys",
        "deck_composition",
        "relic_presence",
        "potion_slots",
        "combat_scalars",
        "combat_stance",
        "combat_enemies",
    )

    def __init__(
        self,
        card_catalog: Optional[List[str]] = None,
        relic_catalog: Optional[List[str]] = None,
        potion_catalog: Optional[List[str]] = None,
        max_enemies: int = 5,
        max_hand_size: int = 12,
        max_potion_slots: int = 5,
    ) -> None:
        self.card_catalog = card_catalog or _default_card_catalog()
        self.relic_catalog = relic_catalog or _default_relic_catalog()
        self.potion_catalog = potion_catalog or _default_potion_catalog()
        self.max_enemies = max_enemies
        self.max_hand_size = max_hand_size
        self.max_potion_slots = max_potion_slots

        # Build lookup dicts
        self._card_to_idx = {c: i for i, c in enumerate(self.card_catalog)}
        self._relic_to_idx = {r: i for i, r in enumerate(self.relic_catalog)}
        self._potion_to_idx = {p: i for i, p in enumerate(self.potion_catalog)}

        # Compute block sizes and offsets
        self.n_run_scalars = 6  # hp_ratio, max_hp, gold, floor, act, ascension
        self.n_keys = 3  # ruby, emerald, sapphire
        self.n_deck = len(self.card_catalog)
        self.n_relics = len(self.relic_catalog)
        self.n_potions = self.max_potion_slots * (len(self.potion_catalog) + 1)  # one-hot + empty
        # Combat: energy, block, turn, hand_size, draw_count, discard_count, exhaust_count
        self.n_combat_scalars = 7
        self.n_combat_stance = len(STANCE_IDS)
        # Per enemy: hp_ratio, max_hp, block, move_damage, move_hits, is_alive
        self.n_per_enemy = 6
        self.n_combat_enemies = self.max_enemies * self.n_per_enemy

        # Total
        self._size = (
            self.n_run_scalars
            + self.n_keys
            + self.n_deck
            + self.n_relics
            + self.n_potions
            + self.n_combat_scalars
            + self.n_combat_stance
            + self.n_combat_enemies
        )

        # Offsets
        self._off_keys = self.n_run_scalars
        self._off_deck = self._off_keys + self.n_keys
        self._off_relics = self._off_deck + self.n_deck
        self._off_potions = self._off_relics + self.n_relics
        self._off_combat_scalars = self._off_potions + self.n_potions
        self._off_combat_stance = self._off_combat_scalars + self.n_combat_scalars
        self._off_combat_enemies = self._off_combat_stance + self.n_combat_stance

    @property
    def size(self) -> int:
        """Total feature vector length."""
        return self._size

    def observation_to_array(self, obs: Dict[str, Any]) -> np.ndarray:
        """Convert an observation dict to a flat float32 feature vector.

        Args:
            obs: Observation dict from ``GameRunner.get_observation()``.

        Returns:
            1-D numpy array of shape ``(self.size,)`` with dtype float32.
        """
        arr = np.zeros(self._size, dtype=np.float32)

        run = obs.get("run", {})
        max_hp = run.get("max_hp", 1)
        current_hp = run.get("current_hp", 0)

        # Run scalars
        arr[0] = current_hp / max(max_hp, 1)  # hp ratio
        arr[1] = float(max_hp)
        arr[2] = float(run.get("gold", 0))
        arr[3] = float(run.get("floor", 0))
        arr[4] = float(run.get("act", 1))
        arr[5] = float(run.get("ascension", 0))

        # Keys
        keys = run.get("keys", {})
        arr[self._off_keys] = float(keys.get("ruby", False))
        arr[self._off_keys + 1] = float(keys.get("emerald", False))
        arr[self._off_keys + 2] = float(keys.get("sapphire", False))

        # Deck composition (card counts)
        for card in run.get("deck", []):
            card_id = card.get("id", "")
            if card.get("upgraded", False):
                card_id = card_id + "+"
            idx = self._card_to_idx.get(card_id)
            if idx is not None:
                arr[self._off_deck + idx] += 1.0

        # Relic presence
        for relic in run.get("relics", []):
            idx = self._relic_to_idx.get(relic.get("id", ""))
            if idx is not None:
                arr[self._off_relics + idx] = 1.0

        # Potion slots (one-hot per slot)
        potion_dim = len(self.potion_catalog) + 1  # +1 for empty
        for slot_i, potion_id in enumerate(run.get("potions", [])):
            if slot_i >= self.max_potion_slots:
                break
            base = self._off_potions + slot_i * potion_dim
            if potion_id is None or potion_id == "" or potion_id == "Potion Slot":
                arr[base] = 1.0  # empty flag
            else:
                pidx = self._potion_to_idx.get(potion_id)
                if pidx is not None:
                    arr[base + 1 + pidx] = 1.0
                else:
                    arr[base] = 1.0  # unknown potion treated as empty

        # Combat features
        combat = obs.get("combat")
        if combat is not None:
            player = combat.get("player", {})
            arr[self._off_combat_scalars] = float(combat.get("energy", 0))
            arr[self._off_combat_scalars + 1] = float(player.get("block", 0))
            arr[self._off_combat_scalars + 2] = float(combat.get("turn", 0))
            hand = combat.get("hand", [])
            arr[self._off_combat_scalars + 3] = float(len(hand))
            # Pile counts -- use count fields if present, else len of list
            draw_pile = combat.get("draw_pile", [])
            arr[self._off_combat_scalars + 4] = float(
                combat.get("draw_pile_count", len(draw_pile))
            )
            discard_pile = combat.get("discard_pile", [])
            arr[self._off_combat_scalars + 5] = float(
                combat.get("discard_pile_count", len(discard_pile))
            )
            exhaust_pile = combat.get("exhaust_pile", [])
            arr[self._off_combat_scalars + 6] = float(
                combat.get("exhaust_pile_count", len(exhaust_pile))
            )

            # Stance (one-hot)
            stance_idx = _stance_to_index(combat.get("stance", "Neutral"))
            arr[self._off_combat_stance + stance_idx] = 1.0

            # Enemies
            for ei, enemy in enumerate(combat.get("enemies", [])):
                if ei >= self.max_enemies:
                    break
                base = self._off_combat_enemies + ei * self.n_per_enemy
                emax_hp = enemy.get("max_hp", 1)
                arr[base] = enemy.get("hp", 0) / max(emax_hp, 1)
                arr[base + 1] = float(emax_hp)
                arr[base + 2] = float(enemy.get("block", 0))
                arr[base + 3] = float(enemy.get("move_damage", 0))
                arr[base + 4] = float(enemy.get("move_hits", 0))
                arr[base + 5] = 1.0 if enemy.get("hp", 0) > 0 else 0.0

        return arr

    def array_to_observation(self, arr: np.ndarray) -> Dict[str, Any]:
        """Reconstruct a (partial) observation dict from a feature vector.

        This is a lossy inverse of ``observation_to_array``.  Not all fields
        can be recovered (e.g., individual card instances vs counts, exact
        potion slot assignment, enemy IDs).  The returned dict uses the same
        top-level structure as the engine observations.

        Args:
            arr: Feature vector of shape ``(self.size,)``.

        Returns:
            Partial observation dict.
        """
        assert arr.shape == (self._size,), f"Expected shape ({self._size},), got {arr.shape}"

        hp_ratio = float(arr[0])
        max_hp = float(arr[1])
        current_hp = int(round(hp_ratio * max_hp))

        run: Dict[str, Any] = {
            "current_hp": current_hp,
            "max_hp": int(max_hp),
            "gold": int(arr[2]),
            "floor": int(arr[3]),
            "act": int(arr[4]),
            "ascension": int(arr[5]),
            "keys": {
                "ruby": bool(arr[self._off_keys] > 0.5),
                "emerald": bool(arr[self._off_keys + 1] > 0.5),
                "sapphire": bool(arr[self._off_keys + 2] > 0.5),
            },
        }

        # Deck
        deck = []
        for i, card_id in enumerate(self.card_catalog):
            count = int(round(arr[self._off_deck + i]))
            upgraded = card_id.endswith("+")
            base_id = card_id.rstrip("+") if upgraded else card_id
            for _ in range(count):
                deck.append({"id": base_id, "upgraded": upgraded, "misc_value": 0})
        run["deck"] = deck

        # Relics
        relics = []
        for i, relic_id in enumerate(self.relic_catalog):
            if arr[self._off_relics + i] > 0.5:
                relics.append({"id": relic_id, "counter": 0})
        run["relics"] = relics

        # Potions
        potions = []
        potion_dim = len(self.potion_catalog) + 1
        for slot_i in range(self.max_potion_slots):
            base = self._off_potions + slot_i * potion_dim
            if arr[base] > 0.5:
                potions.append(None)
            else:
                found = None
                for pidx, pid in enumerate(self.potion_catalog):
                    if arr[base + 1 + pidx] > 0.5:
                        found = pid
                        break
                potions.append(found)
        run["potions"] = potions

        # Combat
        combat: Optional[Dict[str, Any]] = None
        energy = float(arr[self._off_combat_scalars])
        if energy > 0 or arr[self._off_combat_scalars + 2] > 0:
            # Looks like we were in combat
            stance_idx = int(np.argmax(arr[self._off_combat_stance:self._off_combat_stance + len(STANCE_IDS)]))
            stance = STANCE_IDS[stance_idx] if arr[self._off_combat_stance + stance_idx] > 0.5 else "Neutral"

            enemies = []
            for ei in range(self.max_enemies):
                ebase = self._off_combat_enemies + ei * self.n_per_enemy
                if arr[ebase + 5] > 0.5:  # is_alive
                    emax_hp = float(arr[ebase + 1])
                    enemies.append({
                        "hp": int(round(arr[ebase] * max(emax_hp, 1))),
                        "max_hp": int(emax_hp),
                        "block": int(arr[ebase + 2]),
                        "move_damage": int(arr[ebase + 3]),
                        "move_hits": int(arr[ebase + 4]),
                    })

            combat = {
                "energy": int(energy),
                "player": {
                    "block": int(arr[self._off_combat_scalars + 1]),
                },
                "turn": int(arr[self._off_combat_scalars + 2]),
                "hand_size": int(arr[self._off_combat_scalars + 3]),
                "draw_pile_count": int(arr[self._off_combat_scalars + 4]),
                "discard_pile_count": int(arr[self._off_combat_scalars + 5]),
                "exhaust_pile_count": int(arr[self._off_combat_scalars + 6]),
                "stance": stance,
                "enemies": enemies,
            }

        return {
            "run": run,
            "combat": combat,
        }


# ---------------------------------------------------------------------------
# Convenience free functions
# ---------------------------------------------------------------------------

# Module-level singleton encoder, lazily initialised.
_default_encoder: Optional[ObservationEncoder] = None


def _get_default_encoder() -> ObservationEncoder:
    global _default_encoder
    if _default_encoder is None:
        _default_encoder = ObservationEncoder()
    return _default_encoder


def observation_to_array(obs: Dict[str, Any]) -> np.ndarray:
    """Convert observation dict to flat feature vector for RL.

    Uses a default ``ObservationEncoder`` built from the engine's card, relic
    and potion catalogs.  For custom catalogs, instantiate ``ObservationEncoder``
    directly.
    """
    return _get_default_encoder().observation_to_array(obs)


def array_to_observation(arr: np.ndarray) -> Dict[str, Any]:
    """Convert flat feature vector back to a (partial) observation dict.

    This is a lossy inverse -- see ``ObservationEncoder.array_to_observation``.
    """
    return _get_default_encoder().array_to_observation(arr)


# ---------------------------------------------------------------------------
# Default catalog builders
# ---------------------------------------------------------------------------

def _default_card_catalog() -> List[str]:
    """Build default card catalog from engine content."""
    try:
        from .content.cards import ALL_CARDS
        catalog: List[str] = []
        for card_id in sorted(ALL_CARDS.keys()):
            catalog.append(card_id)
            catalog.append(card_id + "+")
        return catalog
    except ImportError:
        # Minimal fallback
        return []


def _default_relic_catalog() -> List[str]:
    """Build default relic catalog from engine content."""
    try:
        from .content.relics import ALL_RELICS
        # ALL_RELICS is a dict keyed by relic id
        if isinstance(ALL_RELICS, dict):
            return sorted(ALL_RELICS.keys())
        return sorted(r.id if hasattr(r, "id") else str(r) for r in ALL_RELICS)
    except ImportError:
        return []


def _default_potion_catalog() -> List[str]:
    """Build default potion catalog from engine content."""
    try:
        from .content.potions import ALL_POTIONS
        # ALL_POTIONS is a dict keyed by potion id
        if isinstance(ALL_POTIONS, dict):
            return sorted(ALL_POTIONS.keys())
        # Fallback if it's a list of Potion objects
        return sorted(p.id if hasattr(p, "id") else str(p) for p in ALL_POTIONS)
    except ImportError:
        return []
