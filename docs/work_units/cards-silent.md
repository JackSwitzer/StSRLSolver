# Silent Card Effects Work Units

## Scope summary
- Implement missing Silent (GREEN) card effects in the effect registry/executor.
- Add any needed combat-state hooks for per-turn, on-draw, and on-discard triggers.
- Update card data only if effect lists need corrections, and add tests for each batch.
- Model-facing actions only (no UI); see `docs/work_units/granular-actions.md`.

## Missing card effects (by card)
| Card | Missing effects |
| --- | --- |
| A Thousand Cuts | `deal_damage_per_card_played` |
| Accuracy | `shivs_deal_more_damage` |
| Acrobatics | `draw_x`, `discard_1` |
| After Image | `gain_1_block_per_card_played` |
| Alchemize | `obtain_random_potion` |
| All-Out Attack | `discard_random_1` |
| Bane | `double_damage_if_poisoned` |
| Blade Dance | `add_shivs_to_hand` |
| Blur | `block_not_removed_next_turn` |
| Bouncing Flask | `apply_poison_random_3_times` |
| Bullet Time | `no_draw_this_turn`, `cards_cost_0_this_turn` |
| Burst | `double_next_skills` |
| Calculated Gamble | `discard_hand_draw_same` |
| Caltrops | `gain_thorns` |
| Catalyst | `double_poison` |
| Choke | `apply_choke` |
| Cloak and Dagger | `add_shivs_to_hand` |
| Concentrate | `discard_x` |
| Corpse Explosion | `apply_poison`, `apply_corpse_explosion` |
| Crippling Poison | `apply_poison_all`, `apply_weak_2_all` |
| Dagger Spray | `damage_all_x_times` |
| Dagger Throw | `discard_1` |
| Deadly Poison | `apply_poison` |
| Distraction | `add_random_skill_cost_0` |
| Dodge and Roll | `block_next_turn` |
| Doppelganger | `draw_x_next_turn`, `gain_x_energy_next_turn` |
| Endless Agony | `copy_to_hand_when_drawn` |
| Envenom | `attacks_apply_poison` |
| Escape Plan | `if_skill_drawn_gain_block` |
| Eviscerate | `cost_reduces_per_discard` |
| Expertise | `draw_to_x_cards` |
| Finisher | `damage_per_attack_this_turn` |
| Flechettes | `damage_per_skill_in_hand` |
| Flying Knee | `gain_energy_next_turn_1` |
| Footwork | `gain_dexterity` |
| Glass Knife | `reduce_damage_by_2` |
| Grand Finale | `only_playable_if_draw_pile_empty` |
| Heel Hook | `if_target_weak_gain_energy_draw` |
| Infinite Blades | `add_shiv_each_turn` |
| Malaise | `apply_weak_x`, `apply_strength_down_x` |
| Masterful Stab | `cost_increases_when_damaged` |
| Nightmare | `copy_card_to_hand_next_turn` |
| Noxious Fumes | `apply_poison_all_each_turn` |
| Outmaneuver | `gain_energy_next_turn` |
| Phantasmal Killer | `double_damage_next_turn` |
| Piercing Wail | `reduce_strength_all_enemies` |
| Poisoned Stab | `apply_poison` |
| Predator | `draw_2_next_turn` |
| Prepared | `draw_x`, `discard_x` |
| Reflex | `when_discarded_draw` |
| Setup | `put_card_on_draw_pile_cost_0` |
| Skewer | `damage_x_times_energy` |
| Sneaky Strike | `refund_2_energy_if_discarded_this_turn` |
| Storm of Steel | `discard_hand`, `add_shivs_equal_to_discarded` |
| Survivor | `discard_1` |
| Tactician | `when_discarded_gain_energy` |
| Tools of the Trade | `draw_1_discard_1_each_turn` |
| Unload | `discard_non_attacks` |
| Well-Laid Plans | `retain_cards_each_turn` |
| Wraith Form | `gain_intangible`, `lose_1_dexterity_each_turn` |

## Suggested small task batches (with acceptance criteria)
- **Poison + debuff core**: implement `apply_poison`, `apply_poison_all`, `apply_poison_random_3_times`, `apply_poison_all_each_turn`, `double_poison`, `attacks_apply_poison`, `apply_corpse_explosion`, `apply_choke`, `reduce_strength_all_enemies`, `apply_weak_2_all`, `apply_weak_x`, `apply_strength_down_x`; Acceptance: poison/debuff statuses update correctly on target/all/envenom triggers, and poison tick behavior remains intact.
- **Shiv package**: implement `add_shivs_to_hand`, `add_shiv_each_turn`, `add_shivs_equal_to_discarded`, `shivs_deal_more_damage`; Acceptance: correct Shiv counts added to hand/draw, per-turn Shiv generation works, and Accuracy modifies Shiv damage.
- **Discard + draw manipulation**: implement `discard_1`, `discard_random_1`, `discard_hand`, `discard_hand_draw_same`, `discard_non_attacks`, `discard_x`, `draw_x`, `draw_to_x_cards`, `put_card_on_draw_pile_cost_0`, `add_random_skill_cost_0`; Acceptance: hand/discard sizes match expected outcomes and chosen/random discards follow rules.
- **On-discard/on-draw triggers**: implement `when_discarded_draw`, `when_discarded_gain_energy`, `refund_2_energy_if_discarded_this_turn`, `copy_to_hand_when_drawn`, `if_skill_drawn_gain_block`; Acceptance: triggers fire only on the correct event and do not double-trigger.
- **Next-turn and duration effects**: implement `block_next_turn`, `block_not_removed_next_turn`, `gain_energy_next_turn`, `gain_energy_next_turn_1`, `draw_2_next_turn`, `draw_x_next_turn`, `gain_x_energy_next_turn`, `copy_card_to_hand_next_turn`, `retain_cards_each_turn`, `no_draw_this_turn`, `cards_cost_0_this_turn`, `double_damage_next_turn`, `double_next_skills`, `gain_intangible`, `lose_1_dexterity_each_turn`, `gain_1_block_per_card_played`, `deal_damage_per_card_played`, `gain_dexterity`, `gain_thorns`; Acceptance: effects persist for the correct duration, apply at the correct timing, and clear afterward.
- **Attack scaling + playability**: implement `double_damage_if_poisoned`, `damage_per_attack_this_turn`, `damage_per_skill_in_hand`, `damage_all_x_times`, `damage_x_times_energy`, `reduce_damage_by_2`, `cost_reduces_per_discard`, `cost_increases_when_damaged`, `only_playable_if_draw_pile_empty`, `if_target_weak_gain_energy_draw`; Acceptance: damage/cost calculations match card text, and playability checks block invalid plays.
- **Potion grant**: implement `obtain_random_potion`; Acceptance: adds a potion when slots available, respects potion slot limits, and does nothing when full.

## Files to touch
- `packages/engine/effects/cards.py` (effect handlers)
- `packages/engine/effects/registry.py` (new effect registrations/patterns)
- `packages/engine/effects/executor.py` (special handling and timing hooks)
- `packages/engine/state/combat.py` (turn-based flags, on-draw/on-discard tracking)
- `packages/engine/content/cards.py` (only if effect lists need corrections)
- `tests/test_effects_and_combat.py`, `tests/test_cards.py`, `tests/test_combat.py` (Silent effect coverage)
