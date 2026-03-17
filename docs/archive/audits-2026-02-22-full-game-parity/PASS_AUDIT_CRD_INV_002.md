# Pass Audit: `CRD-INV-002`

Date: 2026-02-22

## Repo scope check

This pass was implemented in:
- `/Users/jackswitzer/Desktop/SlayTheSpireRL`

It was **not** implemented in:
- `/Users/jackswitzer/Desktop/StSRLSolver`

`StSRLSolver` is a different repository/history and does not contain `packages/engine/*` from this parity campaign.

## Commit in this pass

- Commit: `e89017e`
- Subject: `CRD-INV-002: close Discipline/Impulse card IDs and Gash alias parity`

Changed files include:
- `packages/engine/content/cards.py`
- `packages/engine/effects/cards.py`
- `packages/engine/effects/defect_cards.py`
- `packages/engine/registry/powers.py`
- `tests/test_cards.py`
- `tests/test_defect_cards.py`
- `tests/test_power_registry_integration.py`
- docs/audit trackers and TODO files

## Existence audit (`Impulse`, `Discipline`, `Gash`)

### Present in code
- `Impulse` card definition and registry entries are present.
- `Discipline` card definition and registry entries are present.
- `Gash` is resolved via alias mapping to `Claw`.

Locations:
- `packages/engine/content/cards.py`
  - `DISCIPLINE` definition
  - `IMPULSE` definition
  - `WATCHER_CARDS["Discipline"]`
  - `DEFECT_CARDS["Impulse"]`
  - `CARD_ID_ALIASES["Gash"] = "Claw"`
- `packages/engine/effects/defect_cards.py`
  - `trigger_orb_start_end`
- `packages/engine/effects/cards.py`
  - `apply_discipline_power`
- `packages/engine/registry/powers.py`
  - `DisciplinePower` hooks at `atEndOfTurn` and `atStartOfTurn`

### Present in tests
- `tests/test_defect_cards.py`
  - `get_card("Gash")`
  - `get_card("Impulse")`
  - Impulse orb-passive behavior test
- `tests/test_cards.py`
  - `get_card("Discipline")` stats/upgrade test
- `tests/test_power_registry_integration.py`
  - `DisciplinePower` registration and behavior test

## Current baseline

Command:
- `uv run pytest tests/ -q`

Result:
- `4669 passed, 5 skipped, 0 failed`

Skipped tests are replay-artifact-gated in `tests/test_parity.py`.

## Card inventory + hook audit

### Inventory status
- Java core card IDs (excluding deprecated/option/temp): `361`
- Python card IDs: `370`
- Raw key overlap: `360`
- Lookup-resolved overlap via `get_card(...)`: `361/361`
- Raw Java-only key: `Gash` (resolved by alias)

### Card effect hook coverage
- Unique effect keys used by cards: `308`
- Registered effect handlers: `312`
- Missing effect handlers for used card keys: `23`

Unresolved effect keys currently used by cards:
- `add_random_attacks_to_draw_cost_0` (`Metamorphosis`)
- `add_random_colorless_each_turn` (`Magnetism`)
- `add_random_colorless_to_hand` (`Jack Of All Trades`)
- `add_random_skills_to_draw_cost_0` (`Chrysalis`)
- `add_x_random_colorless_cost_0` (`Transmutation`)
- `auto_play_top_card_each_turn` (`Mayhem`)
- `deal_damage_to_all_after_3_turns` (`The Bomb`)
- `discover_card` (`Discovery`)
- `draw_2_put_1_on_top_of_draw` (`Thinking Ahead`)
- `draw_if_no_attacks_in_hand` (`Impatience`)
- `every_5_cards_deal_damage_to_all` (`Panache`)
- `exhaust_up_to_x_cards` (`Purity`)
- `gain_no_block_next_2_turns` (`PanicButton`)
- `if_fatal_permanently_increase_damage` (`RitualDagger`)
- `lose_3_hp_gain_strength` (`J.A.X.`)
- `on_debuff_deal_damage` (`Sadistic Nature`)
- `put_attacks_from_draw_into_hand` (`Violence`)
- `put_card_on_bottom_of_draw_cost_0` (`Forethought`)
- `reduce_hand_cost_to_1` (`Enlightenment`)
- `reduce_random_card_cost_to_0` (`Madness`)
- `scry_draw_pile_discard_for_block` (`Unraveling`)
- `search_draw_for_attack` (`Secret Weapon`)
- `search_draw_for_skill` (`Secret Technique`)

## Powers audit

### Inventory status
- Java power classes (base + watcher, excluding `AbstractPower`): `149`
- Python `POWER_DATA` entries: `94`

### Runtime-hook status
- Unique power IDs with registry handlers (`@power_trigger`): `79`
- Hook names with handlers: `24`

Hook counts:
- `atStartOfTurn`: 20
- `atStartOfTurnPostDraw`: 6
- `atEndOfTurnPreEndTurnCards`: 3
- `atEndOfTurn`: 14
- `atEndOfRound`: 4
- `onUseCard`: 11
- `onAfterUseCard`: 3
- `onAfterCardPlayed`: 1
- `onExhaust`: 2
- `onCardDraw`: 3
- `onChangeStance`: 2
- `onGainBlock`: 2
- `onAttack`: 2
- `onAttacked`: 1
- `onAttackedToChangeDamage`: 1
- `wasHPLost`: 2
- `onApplyPower`: 1
- `onEnergyRecharge`: 2
- `onManualDiscard`: 3
- `modifyBlock`: 1
- `atDamageGive`: 3
- `atDamageReceive`: 2
- `onScry`: 1
- `onDeath`: 1

### Pass-specific power closure
- `DisciplinePower` now has registry coverage in both required hooks:
  - `atEndOfTurn`: save current energy if `> 0`
  - `atStartOfTurn`: draw saved amount then reset to sentinel `-1`

## Action-layer audit

## API status
Public action API present and active:
- `GameRunner.get_available_action_dicts()`
- `GameRunner.take_action_dict()`
- `GameRunner.get_observation()`

### Action types emitted by adapter (`_action_to_dict`)
- `path_choice`
- `neow_choice`
- `play_card`
- `use_potion`
- `end_turn`
- `pick_card`
- `skip_card`
- `singing_bowl`
- `claim_gold`
- `claim_potion`
- `skip_potion`
- `claim_relic`
- `claim_emerald_key`
- `skip_emerald_key`
- `proceed_from_rewards`
- `event_choice`
- `buy_card`
- `buy_relic`
- `buy_potion`
- `remove_card`
- `leave_shop`
- `rest`
- `smith`
- `dig`
- `lift`
- `toke`
- `recall`
- `take_relic`
- `sapphire_key`
- `pick_boss_relic`

Selection follow-up actions (pending context):
- `select_cards`
- `select_stance`
- plus synthetic `skip_boss_relic` in boss rewards

### Selection behavior
`take_action_dict` currently supports structured selection interception for:
- event choices requiring card selection
- selection-required potions
- boss relic picks with card selection (`Astrolabe`, `Empty Cage`)
- relic acquisition requiring card selection (`Orrery`, bottled relics, `DollysMirror`)

When required params are missing, it returns:
- `success: false`
- `requires_selection: true`
- `candidate_actions: [...]`

### Action-layer open items from this audit
- `PendingSelectionContext.selection_type` supports card/stance flows; no dedicated `target_select` model yet.
- `observation_schema_version` and `action_schema_version` fields are not present yet.

## Summary

This pass (`CRD-INV-002`) is present and test-locked in the parity repo.
If `Impulse`/`Discipline` are not visible, the issue is repository mismatch (`StSRLSolver` vs `parity-core-loop`), not missing implementation inside this pass.
