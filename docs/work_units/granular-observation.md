# Observation Schema (Model-Facing, No UI)

## Scope summary
- Define the observation payload for each phase.
- Ensure JSON‑serializable, stable fields, and deterministic ordering.

## Observation API
- `GameRunner.get_observation() -> ObservationDict`
- Must be deterministic and JSON‑serializable.
- Must include `map` visibility for the current act (see `granular-map-visibility.md`).

## Top-level shape
```json
{
  "phase": "combat",
  "run": { ... },
  "map": { ... },
  "combat": { ... },
  "event": null,
  "reward": null,
  "shop": null,
  "rest": null,
  "treasure": null
}
```

## Run (always present)
- `seed`, `ascension`, `act`, `floor`
- `gold`, `current_hp`, `max_hp`
- `deck`: list of `{id, upgraded, misc_value}`
- `relics`: list of `{id, counter, triggered_this_combat?, triggered_this_turn?}`
- `potions`: list of potion ids or `null`
- `keys`: `{ruby, emerald, sapphire}`
- `map_position`: `{x, y}`
- `rng_counters` (optional debug flag; not required for agent policy)

## Map (always present)
- `act`: current act number
- `nodes`: list of `{x, y, room_type, has_emerald_key}`
- `edges`: list of `{src_x, src_y, dst_x, dst_y, is_boss}`
- `available_paths`: ordered list of `{x, y, room_type}` matching `path_choice{node_index}`
- `visited_nodes`: list of `{act, x, y}`

## Combat (combat phase only)
- `player`: `{hp, max_hp, block, statuses}`
- `energy`, `max_energy`, `stance`, `mantra`
- `hand`, `draw_pile`, `discard_pile`, `exhaust_pile` (card id strings)
- `enemies`: list of `{id, hp, max_hp, block, statuses, move_id, move_damage, move_hits, move_block, move_effects}`
- `turn`, `cards_played_this_turn`, `attacks_played_this_turn`, `skills_played_this_turn`, `powers_played_this_turn`
- `relic_counters`, `card_costs` (cost overrides)

## Event (event phase only)
- `event_id`, `phase`, `attempt_count`, `hp_cost_modifier`
- `choices`: list of `{choice_index, label, requires_card_selection, card_selection_type, card_selection_count}`
- `pending_rewards` (if any)

## Reward (reward phases)
- `gold`: `{amount, claimed}`
- `potion`: `{id, claimed, skipped}`
- `card_rewards`: list of `{cards:[{id, upgraded, rarity}], claimed_index, skipped, singing_bowl_used}`
- `relic`: `{id, claimed}`
- `boss_relics`: list of `{id}` + `chosen_index` if any
- `emerald_key`: `{available, claimed}`

## Shop (shop phase)
- `colored_cards`: list of `{id, upgraded, price, purchased}`
- `colorless_cards`: list of `{id, upgraded, price, purchased}`
- `relics`: list of `{id, price, purchased}`
- `potions`: list of `{id, price, purchased}`
- `purge_cost`, `purge_available`

## Rest (rest phase)
- `available_actions`: list of action ids (e.g., `rest`, `smith`, `dig`, `lift`, `toke`, `recall`)

## Acceptance criteria
- Observation is JSON‑serializable and deterministic for identical state.
- Field presence is stable by phase (null for irrelevant sections).
- Round‑trip serialization does not lose information needed for action selection.
- `available_paths` ordering is stable and matches the map action indices.
