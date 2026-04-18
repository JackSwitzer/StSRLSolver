"""
RL action masking utilities for the Slay the Spire engine.

Provides a canonical action space with fixed-size binary mask support,
enabling RL agents to work with a stable, deterministic action interface.

Design decisions:
- The mask maps action IDs (strings) to canonical indices, not raw parameter
  combinations.  This keeps the mask compact and avoids combinatorial explosion
  from variable-length selection actions.
- Two-step selection actions (select_cards, select_stance) are handled by
  regenerating the mask after the engine enters its pending_selection state.
  Each call to actions_to_mask always reflects the *current* legal surface.
- The ActionSpace is built lazily from observed action dicts -- it does not
  hardcode parameter ranges.  A pre-seeded catalog of known action types is
  provided for convenience, but the space grows if new IDs appear.

Usage:
    from packages.engine.rl_masks import ActionSpace

    space = ActionSpace()

    actions = runner.get_available_action_dicts()
    mask = space.actions_to_mask(actions)         # np.ndarray, dtype bool
    idx  = space.action_to_index(actions[0])      # int
    act  = space.index_to_action(idx, actions)    # ActionDict or None
"""

from __future__ import annotations

from typing import Dict, List, Optional, Any, Tuple

import numpy as np


# ---------------------------------------------------------------------------
# Canonical action type catalog.
# This is the *known* universe of action types.  IDs are deterministic strings
# produced by GameRunner._make_action_id (type|key=val|...).
# ---------------------------------------------------------------------------

ACTION_TYPES: Tuple[str, ...] = (
    # Map / navigation
    "path_choice",
    "neow_choice",
    # Combat
    "play_card",
    "use_potion",
    "end_turn",
    # Rewards
    "pick_card",
    "skip_card",
    "singing_bowl",
    "claim_gold",
    "claim_potion",
    "skip_potion",
    "claim_relic",
    "claim_emerald_key",
    "skip_emerald_key",
    "proceed_from_rewards",
    # Boss rewards
    "pick_boss_relic",
    "skip_boss_relic",
    # Events
    "event_choice",
    # Shop
    "buy_card",
    "buy_relic",
    "buy_potion",
    "remove_card",
    "leave_shop",
    # Rest site
    "rest",
    "smith",
    "dig",
    "lift",
    "toke",
    "recall",
    # Treasure
    "take_relic",
    "sapphire_key",
    # Two-step selection
    "select_cards",
    "select_stance",
)


class ActionSpace:
    """Canonical action space with fixed-size mask support.

    The space maintains a bidirectional mapping between action ID strings
    (produced by the engine) and integer indices suitable for RL mask vectors.

    The space is *open*: if an unseen action ID appears it is assigned a new
    index.  The ``size`` property always reflects the current maximum index.
    """

    def __init__(self) -> None:
        self._id_to_index: Dict[str, int] = {}
        self._index_to_id: Dict[int, str] = {}
        self._next_index: int = 0

    # -- properties ----------------------------------------------------------

    @property
    def size(self) -> int:
        """Current number of registered action IDs."""
        return self._next_index

    @property
    def action_types(self) -> Tuple[str, ...]:
        """Known action type names (informational)."""
        return ACTION_TYPES

    # -- registration --------------------------------------------------------

    def register(self, action_id: str) -> int:
        """Register an action ID and return its index.

        If the ID is already registered the existing index is returned.
        """
        if action_id in self._id_to_index:
            return self._id_to_index[action_id]
        idx = self._next_index
        self._id_to_index[action_id] = idx
        self._index_to_id[idx] = action_id
        self._next_index += 1
        return idx

    def register_actions(self, action_dicts: List[Dict[str, Any]]) -> None:
        """Bulk-register all IDs found in a list of action dicts."""
        for action in action_dicts:
            aid = action.get("id")
            if aid is not None:
                self.register(aid)

    # -- mask operations -----------------------------------------------------

    def actions_to_mask(self, action_dicts: List[Dict[str, Any]]) -> np.ndarray:
        """Convert available action dicts to a binary mask vector.

        Returns a boolean numpy array of length ``self.size``.  Indices
        corresponding to legal actions are ``True``.

        Any previously unseen action IDs are auto-registered so the mask
        always covers the full known space.
        """
        self.register_actions(action_dicts)
        mask = np.zeros(self.size, dtype=np.bool_)
        for action in action_dicts:
            aid = action.get("id")
            if aid is not None and aid in self._id_to_index:
                mask[self._id_to_index[aid]] = True
        return mask

    def mask_to_actions(
        self,
        mask: np.ndarray,
        action_dicts: List[Dict[str, Any]],
    ) -> List[Dict[str, Any]]:
        """Filter action dicts by a binary mask.

        Returns only those action dicts whose IDs map to ``True`` entries
        in *mask*.
        """
        id_lookup = {a.get("id"): a for a in action_dicts}
        result: List[Dict[str, Any]] = []
        for idx in np.flatnonzero(mask):
            aid = self._index_to_id.get(int(idx))
            if aid is not None and aid in id_lookup:
                result.append(id_lookup[aid])
        return result

    # -- index <-> action mapping --------------------------------------------

    def action_to_index(self, action_dict: Dict[str, Any]) -> int:
        """Map an action dict to its canonical index.

        The action ID is auto-registered if not yet known.

        Raises ``KeyError`` if the action dict has no ``id`` field.
        """
        aid = action_dict.get("id")
        if aid is None:
            raise KeyError("Action dict has no 'id' field")
        return self.register(aid)

    def index_to_action(
        self,
        index: int,
        action_dicts: List[Dict[str, Any]],
    ) -> Optional[Dict[str, Any]]:
        """Map a mask index back to the corresponding action dict.

        Returns ``None`` if the index does not correspond to any action in
        *action_dicts*.
        """
        aid = self._index_to_id.get(index)
        if aid is None:
            return None
        for action in action_dicts:
            if action.get("id") == aid:
                return action
        return None

    def index_to_action_id(self, index: int) -> Optional[str]:
        """Map a mask index to its action ID string, or ``None``."""
        return self._index_to_id.get(index)

    # -- utilities -----------------------------------------------------------

    def get_index(self, action_id: str) -> Optional[int]:
        """Return the index for an action ID, or ``None`` if unregistered."""
        return self._id_to_index.get(action_id)

    def __contains__(self, action_id: str) -> bool:
        return action_id in self._id_to_index

    def __repr__(self) -> str:
        return f"ActionSpace(size={self.size})"
