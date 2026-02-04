# Action Spec (Model-Facing, No UI)

Goal: every decision point should be traversable by a non-player agent via explicit, JSON-serializable actions.

## Action object (minimum fields)
- `id`: stable identifier for the action (string).
- `type`: action type enum (e.g., `play_card`, `use_potion`, `event_choice`).
- `label`: human-readable summary for debugging.
- `params`: required parameters (indices/ids/targets).
- `requires`: optional hints (e.g., `target`, `card_ids`, `potion_slot`).
- `phase`: current phase (combat/event/reward/shop/rest/map).

## Parameter schema (canonical)
- `card_index`: index into **hand**.
- `card_indices`: list of indices into a **pile**.
- `pile`: `"hand" | "discard" | "draw" | "exhaust" | "deck" | "offer"`.
- `target_index`: index into **living enemies** list.
- `potion_slot`: index into potion slots.
- `potion_id`: optional string alias; resolve to a slot if unique.
- `choice_index`: index into event choice list.
- `card_reward_index`: index into card reward list.
- `relic_index`: index into boss relic choices.
- `potion_reward_index`: index into potion rewards (if multiple).
- `relic_reward_index`: index into relic rewards (if multiple).
- `item_index`: index into shop items.
- `card_pool`: `"colored" | "colorless"` for shop card purchases.
- `energy_spent`: integer for X‑cost.
- `stance`: `"Calm" | "Wrath" | "Neutral" | "Divinity"`.
- `node_index`: index into `map.available_paths`.
- `parent_action_id`: optional for follow‑up selection actions.
- `min_cards` / `max_cards`: selection bounds for `select_cards`.

## Action types and required params
Automatic (no action required):
- `none{}`: indicates a behavior that triggers without user action.

Combat:
- `play_card`: `{card_index, target_index? , energy_spent?}`
- `end_turn`: `{}`

Potions:
- `use_potion`: `{potion_slot, target_index? , card_indices? , pile? }`
- `select_stance`: `{stance, parent_action_id?}`
- `select_cards`: `{pile, card_indices, min_cards?, max_cards?, parent_action_id?}`

Map:
- `path_choice`: `{node_index}`

Events:
- `event_choice`: `{choice_index, card_index? }`
- `select_cards`: `{pile:"deck", card_indices, min_cards?, max_cards?, parent_action_id?}`

Neow:
- `neow_choice`: `{choice_index}`

Rewards:
- `pick_card`: `{card_reward_index, card_index}`
- `skip_card`: `{card_reward_index}`
- `singing_bowl`: `{card_reward_index}`
- `claim_gold`: `{}`
- `claim_potion`: `{potion_reward_index?}`
- `skip_potion`: `{potion_reward_index?}`
- `claim_relic`: `{relic_reward_index?}`
- `pick_boss_relic`: `{relic_index}`
- `skip_boss_relic`: `{}`
- `claim_emerald_key`: `{}`
- `skip_emerald_key`: `{}`
- `proceed_from_rewards`: `{}`

Shop:
- `buy_card`: `{item_index, card_pool?}` (or `buy_colored_card` / `buy_colorless_card`)
- `buy_relic`: `{item_index}`
- `buy_potion`: `{item_index}`
- `remove_card`: `{card_index}`
- `leave_shop`: `{}`

Rest:
- `rest`: `{}`
- `smith`: `{card_index}`
- `dig`: `{}`
- `lift`: `{}`
- `toke`: `{card_index}`
- `recall`: `{}`

Treasure:
- `take_relic`: `{}`
- `sapphire_key`: `{}`

## Engine behavior
- `get_available_action_dicts()` returns JSON action dicts valid for the current phase.
- `take_action_dict(action)` executes a JSON action dict.
- `get_available_actions()` / `take_action()` remain supported for dataclass actions.
- If parameters are missing, return a structured action list instead of erroring.
- All actions must be deterministic and serialize cleanly (no object refs).

## Combat actions
- `play_card`: `{card_index, target_index?}` (target only when required).
- `use_potion`: `{potion_slot, target_index?, card_ids?}`.
- `end_turn`: no params.

## Event actions
- `event_choice`: `{choice_index, card_index?}` (card_index only when selection required).
- Multi-phase events must emit new choices after each phase change.

## Reward actions
- `claim_gold`, `pick_card`, `skip_card`, `singing_bowl`, `claim_potion`, `skip_potion`,
  `claim_relic`, `claim_emerald_key`, `skip_emerald_key`, `pick_boss_relic`, `skip_boss_relic`,
  `proceed_from_rewards`.

## Shop actions
- `buy_card`, `buy_relic`, `buy_potion`, `remove_card`, `leave_shop` with indices/ids.
- `buy_colored_card` / `buy_colorless_card` are accepted aliases for shop card pools.

## Rest-site actions
- `rest`, `smith`, `dig`, `lift`, `toke`, `recall` with card indices when applicable.

## Selection-based potions (model-friendly)
- If caller provides `card_ids`, execute directly (e.g., Liquid Memories with Sacred Bark).
- If not provided, return available actions listing valid selections.

## Boss relics (clone policy)
- Present `pick_boss_relic` actions + an explicit `skip_boss_relic` action.
- Proceed allowed after choose or skip.
- `skip_boss_relic` should advance act without granting a relic.

## Examples (JSON actions)
Combat:
```json
{
  "id": "play_card_2",
  "type": "play_card",
  "label": "Strike -> Cultist",
  "params": {"card_index": 2, "target_index": 0},
  "phase": "combat"
}
```

Potion with explicit selection:
```json
{
  "id": "use_potion_1",
  "type": "use_potion",
  "label": "Liquid Memories (pick 1)",
  "params": {"potion_slot": 1, "card_ids": ["Defend"]},
  "phase": "combat"
}
```

Potion with missing params (engine should return choices):
```json
{
  "id": "use_potion_1",
  "type": "use_potion",
  "label": "Liquid Memories (needs card_ids)",
  "params": {"potion_slot": 1},
  "requires": ["card_ids"],
  "phase": "combat"
}
```

Event choice:
```json
{
  "id": "event_choice_0",
  "type": "event_choice",
  "label": "Falling: Choose Skill",
  "params": {"choice_index": 0},
  "phase": "event"
}
```

Reward choice:
```json
{
  "id": "pick_boss_relic_2",
  "type": "pick_boss_relic",
  "label": "Pick boss relic #2",
  "params": {"relic_index": 2},
  "phase": "reward"
}
```

Boss relic skip:
```json
{
  "id": "skip_boss_relic",
  "type": "skip_boss_relic",
  "label": "Skip boss relic",
  "params": {},
  "phase": "reward"
}
```

Shop action:
```json
{
  "id": "buy_relic_0",
  "type": "buy_relic",
  "label": "Buy relic #0",
  "params": {"item_index": 0},
  "phase": "shop"
}
```

Rest action:
```json
{
  "id": "smith_7",
  "type": "smith",
  "label": "Smith card #7",
  "params": {"card_index": 7},
  "phase": "rest"
}
```

Map path choice:
```json
{
  "id": "path_choice_0",
  "type": "path_choice",
  "label": "Path to node #0",
  "params": {"node_index": 0},
  "phase": "map"
}
```
