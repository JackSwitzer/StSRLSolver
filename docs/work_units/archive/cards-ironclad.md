# ARCHIVED (use granular work units)

This legacy work unit is archived. Use `docs/work_units/granular-cards-ironclad.md`.

# Ironclad Card Effects Work Units

## Scope summary
- Implement missing Ironclad card effects in the effects registry/executor and wire any needed combat or power hooks.
- Keep card data effect names resolvable (align names or add handlers), and add focused tests for each mechanic.
- Model-facing actions only (no UI); see `docs/work_units/granular-actions.md`.

## Missing Ironclad effects (by mechanic)
### Pile manipulation & card creation
- Anger — `add_copy_to_discard`
- Headbutt — `put_card_from_discard_on_draw`
- Warcry — `draw_then_put_on_draw`
- Havoc — `play_top_card`
- Infernal Blade — `add_random_attack_cost_0`
- Wild Strike — `shuffle_wound_into_draw`
- Reckless Charge — `shuffle_dazed_into_draw`
- Power Through — `add_wounds_to_hand`
- Immolate — `add_burn_to_discard`
- Exhume — `return_exhausted_card_to_hand`

### Exhaust / upgrade / copy
- Armaments — `upgrade_card_in_hand`
- True Grit — `exhaust_random_card`
- Burning Pact — `exhaust_to_draw`
- Sever Soul — `exhaust_all_non_attacks`
- Fiend Fire — `exhaust_hand_damage_per_card`
- Second Wind — `exhaust_non_attacks_gain_block`
- Dual Wield — `copy_attack_or_power`
- Searing Blow — `can_upgrade_unlimited`
- Corruption — `skills_cost_0_exhaust`

### Damage scaling / multi-hit / on-kill
- Body Slam — `damage_equals_block`
- Heavy Blade — `strength_multiplier`
- Perfected Strike — `damage_per_strike`
- Rampage — `increase_damage_on_use`
- Sword Boomerang — `random_enemy_x_times`
- Whirlwind — `damage_all_x_times`
- Reaper — `damage_all_heal_unblocked`
- Feed — `if_fatal_gain_max_hp`

### Conditional rules / cost / energy / draw
- Clash — `only_attacks_in_hand`
- Dropkick — `if_vulnerable_draw_and_energy`
- Blood for Blood — `cost_reduces_when_damaged`
- Battle Trance — `draw_then_no_draw`
- Seeing Red — `gain_2_energy`
- Sentinel — `gain_energy_on_exhaust_2_3`
- Berserk — `gain_vulnerable_gain_energy_per_turn`

### HP-loss & self-cost effects
- Hemokinesis — `lose_hp`
- Bloodletting — `lose_hp_gain_energy`
- Offering — `lose_hp_gain_energy_draw`
- Combust — `end_turn_damage_all_lose_hp`
- Brutality — `start_turn_lose_hp_draw`

### Debuffs & Strength
- Flex — `gain_temp_strength`
- Disarm — `reduce_enemy_strength`
- Thunderclap — `apply_vulnerable_1_all`
- Uppercut — `apply_weak_and_vulnerable`
- Shockwave — `apply_weak_and_vulnerable_all`
- Intimidate — `apply_weak_all`
- Spot Weakness — `gain_strength_if_enemy_attacking`
- Inflame — `gain_strength`
- Demon Form — `gain_strength_each_turn`
- Limit Break — `double_strength`
- Rupture — `gain_strength_on_hp_loss`

### Block & block-based powers
- Entrench — `double_block`
- Barricade — `block_not_lost`
- Metallicize — `end_turn_gain_block`
- Flame Barrier — `when_attacked_deal_damage`
- Rage — `gain_block_per_attack`
- Juggernaut — `damage_random_on_block`

## Suggested task batches (small units)
### Batch 1: Pile manipulation & creation
Cards: Anger, Headbutt, Warcry, Havoc, Infernal Blade, Wild Strike, Reckless Charge, Power Through, Immolate, Exhume
Acceptance:
- Each effect resolves without "Unknown effect" in the executor.
- Card piles reflect expected adds/moves (hand/draw/discard/exhaust) for base + upgraded versions.
- Add tests that assert pile changes and random selection bounds.

### Batch 2: Exhaust / upgrade / copy
Cards: Armaments, True Grit, Burning Pact, Sever Soul, Fiend Fire, Second Wind, Dual Wield, Searing Blow, Corruption
Acceptance:
- Exhaust-driven effects move cards into `exhaust_pile` with correct counts.
- Armaments/Upgrade logic respects upgrade rules (Searing Blow multi-upgrade).
- Corruption enforces skill cost 0 + exhaust behavior consistently.

### Batch 3: Damage scaling / multi-hit / on-kill
Cards: Body Slam, Heavy Blade, Perfected Strike, Rampage, Sword Boomerang, Whirlwind, Reaper, Feed
Acceptance:
- Damage formulas match card text and account for Strength/Vulnerable where appropriate.
- X-cost effects consume `energy_spent` and scale hit counts correctly.
- On-kill effects (Feed) update max HP only on fatal hit.

### Batch 4: Conditional rules / cost / energy / draw
Cards: Clash, Dropkick, Blood for Blood, Battle Trance, Seeing Red, Sentinel, Berserk
Acceptance:
- Play restrictions are enforced (Clash) and error reason surfaces in `can_play_card`.
- Cost reductions persist correctly through combat (Blood for Blood).
- Energy/draw flags update state and prevent extra draw when applicable (Battle Trance).

### Batch 5: HP-loss & self-cost effects
Cards: Hemokinesis, Bloodletting, Offering, Combust, Brutality
Acceptance:
- HP loss is treated as HP_LOSS (ignores block) and triggers Rupture where applicable.
- Start/end of turn triggers align with existing power hooks (Combust, Brutality).
- Tests cover lethal edge cases (HP cannot go below 0).

### Batch 6: Debuffs & Strength
Cards: Flex, Disarm, Thunderclap, Uppercut, Shockwave, Intimidate, Spot Weakness, Inflame, Demon Form, Limit Break, Rupture
Acceptance:
- Debuffs apply to correct target set (single vs all) with proper durations.
- Temp Strength decays at end of turn; Strength doubling respects current Strength only.
- Power status names match `registry/powers.py` triggers.

### Batch 7: Block & block-based powers
Cards: Entrench, Barricade, Metallicize, Flame Barrier, Rage, Juggernaut
Acceptance:
- Block math is correct (double block, retain block across turns, on-block triggers).
- Flame Barrier reflects damage on hit without blocking interaction regressions.
- Tests confirm block retention in end-of-turn cleanup.

## Files to touch
- Effects registry/executor: `packages/engine/effects/cards.py`, `packages/engine/effects/executor.py`, `packages/engine/effects/__init__.py`
- Combat/power hooks: `packages/engine/handlers/combat.py`, `packages/engine/registry/powers.py`, `packages/engine/state/combat.py`
- Card data (if aligning effect names): `packages/engine/content/cards.py`, `packages/engine/content/powers.py`
- Tests: add `tests/test_ironclad_card_effects.py` (patterned after `tests/test_watcher_card_effects.py`) and extend existing card data tests if needed
