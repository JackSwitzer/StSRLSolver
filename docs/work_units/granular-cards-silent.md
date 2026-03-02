# Ultra-Granular Work Units: Cards (Silent)

## Current parity source
- Non-Defect manifest row source: `docs/audits/2026-02-22-full-game-parity/domains/cards-manifest-non-defect.md` (`green` section)
- Phase target: close Silent rows from `approximate` to Java-audited `exact`

## Model-facing actions (no UI)
- [ ] All card effects that require choices/targets must emit explicit action options. (action: play_card{card_index,target_index})

## Action tags
Use explicit signatures on each item (see examples in `granular-actions.md`).

## Checklist (missing effects)

### Verified CRD-SI-001 (2026-03-02)

All 61 Silent cards verified against Java decompiled source. Effect handlers exist,
card data matches Java values (cost, damage, block, magic_number, exhaust, upgrade
deltas). Tests in `tests/test_silent_card_verification.py` (130 tests).

**Bugs fixed during verification:**
- Calculated Gamble: added `upgrade_exhaust=False` (Java: `this.exhaust = false` on upgrade)
- Adrenaline: changed `"gain_energy"` to `"gain_energy_magic"` (parameterized effect TypeError)
- Distraction: fixed skill pool to use all colors (Java: `returnTrulyRandomCardInCombat`)
- `apply_weak`/`apply_vulnerable`/`apply_strength`: made `amount` parameter optional with magic_number fallback
- Card.upgrade(): now applies upgrade_exhaust/upgrade_retain/upgrade_innate/upgrade_ethereal

**Passive/tracking effects (handled by combat engine, not effect executor):**
- Eviscerate: `cost_reduces_per_discard` -- Java `didDiscard()` reduces cost per discard
- Masterful Stab: `cost_increases_when_damaged` -- Java `tookDamage()` increases cost
- Endless Agony: `copy_to_hand_when_drawn` -- Java draw-time trigger
- Reflex: `when_discarded_draw` -- handled in `_handle_manual_discard`
- Tactician: `when_discarded_gain_energy` -- handled in `_handle_manual_discard`
- Grand Finale: `only_playable_if_draw_pile_empty` -- Java `canUse()` check

- [x] A Thousand Cuts - deal_damage_per_card_played (action: play_card{card_index})
- [x] Accuracy - shivs_deal_more_damage (action: play_card{card_index})
- [x] Acrobatics - draw_x, discard_1 (action: play_card{card_index} + select_cards{pile:hand,card_indices})
- [x] After Image - gain_1_block_per_card_played (action: play_card{card_index})
- [x] All Out Attack - discard_random_1 (action: play_card{card_index} + target_index:auto)
- [x] Bane - double_damage_if_poisoned (action: play_card{card_index,target_index})
- [x] Blade Dance - add_shivs_to_hand (action: play_card{card_index})
- [x] Blur - block_not_removed_next_turn (action: play_card{card_index})
- [x] Bouncing Flask - apply_poison_random_3_times (action: play_card{card_index,target_index})
- [x] Bullet Time - no_draw_this_turn, cards_cost_0_this_turn (action: play_card{card_index})
- [x] Burst - double_next_skills (action: play_card{card_index})
- [x] Calculated Gamble - discard_hand_draw_same (action: play_card{card_index})
- [x] Caltrops - gain_thorns (action: play_card{card_index})
- [x] Catalyst - double_poison (action: play_card{card_index,target_index})
- [x] Choke - apply_choke (action: play_card{card_index,target_index})
- [x] Cloak And Dagger - add_shivs_to_hand (action: play_card{card_index})
- [x] Concentrate - discard_x (action: play_card{card_index} + select_cards{pile:hand,card_indices})
- [x] Corpse Explosion - apply_poison, apply_corpse_explosion (action: play_card{card_index,target_index})
- [x] Crippling Poison - apply_poison_all, apply_weak_2_all (action: play_card{card_index})
- [x] Dagger Spray - damage_all_x_times (action: play_card{card_index})
- [x] Dagger Throw - discard_1 (action: play_card{card_index,target_index} + select_cards{pile:hand,card_indices})
- [x] Deadly Poison - apply_poison (action: play_card{card_index,target_index})
- [x] Distraction - add_random_skill_cost_0 (action: play_card{card_index})
- [x] Dodge and Roll - block_next_turn (action: play_card{card_index})
- [x] Doppelganger - draw_x_next_turn, gain_x_energy_next_turn (action: play_card{card_index,energy_spent})
- [x] Endless Agony - copy_to_hand_when_drawn (action: play_card{card_index})
- [x] Envenom - attacks_apply_poison (action: play_card{card_index})
- [x] Escape Plan - if_skill_drawn_gain_block (action: play_card{card_index})
- [x] Eviscerate - cost_reduces_per_discard (action: play_card{card_index,target_index})
- [x] Expertise - draw_to_x_cards (action: play_card{card_index})
- [x] Finisher - damage_per_attack_this_turn (action: play_card{card_index,target_index})
- [x] Flechettes - damage_per_skill_in_hand (action: play_card{card_index,target_index})
- [x] Flying Knee - gain_energy_next_turn_1 (action: play_card{card_index,target_index})
- [x] Footwork - gain_dexterity (action: play_card{card_index})
- [x] Glass Knife - reduce_damage_by_2 (action: play_card{card_index,target_index})
- [x] Grand Finale - only_playable_if_draw_pile_empty (action: play_card{card_index})
- [x] Heel Hook - if_target_weak_gain_energy_draw (action: play_card{card_index,target_index})
- [x] Infinite Blades - add_shiv_each_turn (action: play_card{card_index})
- [x] Malaise - apply_weak_x, apply_strength_down_x (action: play_card{card_index,target_index,energy_spent})
- [x] Masterful Stab - cost_increases_when_damaged (action: play_card{card_index,target_index})
- [x] Nightmare - copy_card_to_hand_next_turn (action: play_card{card_index} + select_cards{pile:hand,card_indices})
- [x] Noxious Fumes - apply_poison_all_each_turn (action: play_card{card_index})
- [x] Outmaneuver - gain_energy_next_turn (action: play_card{card_index})
- [x] Phantasmal Killer - double_damage_next_turn (action: play_card{card_index})
- [x] PiercingWail - reduce_strength_all_enemies (action: play_card{card_index})
- [x] Poisoned Stab - apply_poison (action: play_card{card_index,target_index})
- [x] Predator - draw_2_next_turn (action: play_card{card_index,target_index})
- [x] Prepared - draw_x, discard_x (action: play_card{card_index} + select_cards{pile:hand,card_indices})
- [x] Reflex - when_discarded_draw (action: play_card{card_index})
- [x] Setup - put_card_on_draw_pile_cost_0 (action: play_card{card_index} + select_cards{pile:hand,card_indices})
- [x] Skewer - damage_x_times_energy (action: play_card{card_index,target_index,energy_spent})
- [x] Storm of Steel - discard_hand, add_shivs_equal_to_discarded (action: play_card{card_index})
- [x] Survivor - discard_1 (action: play_card{card_index} + select_cards{pile:hand,card_indices})
- [x] Tactician - when_discarded_gain_energy (action: play_card{card_index})
- [x] Tools of the Trade - draw_1_discard_1_each_turn (action: play_card{card_index})
- [x] Sneaky Strike - refund_2_energy_if_discarded_this_turn (action: play_card{card_index,target_index})
- [x] Unload - discard_non_attacks (action: play_card{card_index,target_index})
- [x] Alchemize - obtain_random_potion (action: play_card{card_index})
- [x] Well Laid Plans - retain_cards_each_turn (action: play_card{card_index} + select_cards{pile:hand,card_indices} at end of turn)
- [x] Wraith Form - gain_intangible, lose_1_dexterity_each_turn (action: play_card{card_index})
