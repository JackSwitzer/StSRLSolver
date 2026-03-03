# Ultra-Granular Work Units: Cards (Defect)

## Phase note
- Defect closure is intentionally sequenced after `ORB-001` and `POW-*` stabilization.
- Do not mark Defect rows `exact` until orb timing/runtime parity is locked.

## Inventory closure slice (`CRD-INV-002`)
- [x] Add missing Java-ID card `Impulse` (Defect) with behavior mapping and tests.
- [x] Add Java-ID alias coverage for `Gash` -> `Claw` and lock with inventory tests/docs.

### `CRD-INV-002` closure notes
- Java refs: `cards/blue/Impulse.java`, `actions/defect/ImpulseAction.java`, `cards/blue/Claw.java` (`ID = "Gash"`).
- Python implementation: `content/cards.py` adds `Impulse`; `effects/defect_cards.py` adds `trigger_orb_start_end` (includes `Cables` rightmost-orb extra trigger behavior).
- Tests: `tests/test_defect_cards.py` validates `get_card("Gash")`, `get_card("Impulse")`, and Impulse orb-passive execution.

## Behavior verification closure (`CRD-DE-001`)
- [x] All 68 card effect handlers verified against Java source.
- [x] 279 behavioral tests in `tests/test_defect_card_verification.py`.
- [x] Card data values (cost, damage, block, magic) verified for all 68 cards.
- [x] Upgrade values (upgrade_damage, upgrade_block, upgrade_magic, upgrade_cost) verified.
- [x] Upgrade flag changes (exhaust, ethereal) verified: Hologram+, Rainbow+, Echo Form+, Impulse+.

### `CRD-DE-001` fixes applied
- **Hologram+**: Added `upgrade_exhaust=False` (Java: `this.exhaust = false` in upgrade).
- **Rainbow+**: Added `upgrade_exhaust=False` (Java: `this.exhaust = false` in upgrade).
- **Echo Form+**: Added `upgrade_ethereal=False` (Java: `this.isEthereal = false` in upgrade).
- **Darkness+**: Added `darkness_trigger_dark_orbs` effect (Java: DarkImpulseAction triggers passive on all Dark orbs).
- **Tempest+**: Fixed `channel_x_lightning` to channel X+1 (Java: TempestAction `++effect` when upgraded).
- **Multi-Cast+**: Fixed `evoke_first_orb_x_times` to evoke X+1 (Java: MulticastAction `++effect` when upgraded).

## Model-facing actions (no UI)
- [ ] All card effects that require choices/targets must emit explicit action options. (action: play_card{card_index,target_index})

## Action tags
Use explicit signatures on each item (see examples in `granular-actions.md`).

## Checklist (verified effects)
- [x] Aggregate - gain_energy_per_x_cards_in_draw (action: play_card{card_index})
- [x] All For One - return_all_0_cost_from_discard (action: play_card{card_index})
- [x] Amplify - next_power_plays_twice (action: play_card{card_index})
- [x] Auto Shields - only_if_no_block (action: play_card{card_index})
- [x] Ball Lightning - channel_lightning (action: play_card{card_index,target_index})
- [x] Barrage - damage_per_orb (action: play_card{card_index,target_index})
- [x] Biased Cognition - gain_focus_lose_focus_each_turn (action: play_card{card_index})
- [x] Blizzard - damage_per_frost_channeled (action: play_card{card_index})
- [x] Buffer - prevent_next_hp_loss (action: play_card{card_index})
- [x] Capacitor - increase_orb_slots (action: play_card{card_index})
- [x] Chaos - channel_random_orb (action: play_card{card_index})
- [x] Chill - channel_frost_per_enemy (action: play_card{card_index})
- [x] Claw - increase_all_claw_damage (action: play_card{card_index,target_index})
- [x] Cold Snap - channel_frost (action: play_card{card_index,target_index})
- [x] Compile Driver - draw_per_unique_orb (action: play_card{card_index,target_index})
- [x] Charge Battery - gain_1_energy_next_turn (action: play_card{card_index})
- [x] Consume - gain_focus_lose_orb_slot (action: play_card{card_index})
- [x] Coolheaded - channel_frost (action: play_card{card_index})
- [x] Creative AI - add_random_power_each_turn (action: play_card{card_index})
- [x] Darkness - channel_dark, darkness_trigger_dark_orbs (action: play_card{card_index})
- [x] Defragment - gain_focus (action: play_card{card_index})
- [x] Doom and Gloom - channel_dark (action: play_card{card_index})
- [x] Double Energy - double_energy (action: play_card{card_index})
- [x] Dualcast - evoke_orb_twice (action: play_card{card_index})
- [x] Echo Form - play_first_card_twice (action: play_card{card_index})
- [x] Electrodynamics - lightning_hits_all, channel_lightning_magic (action: play_card{card_index})
- [x] FTL - if_played_less_than_x_draw (action: play_card{card_index,target_index})
- [x] Fission - remove_orbs_gain_energy_and_draw (action: play_card{card_index})
- [x] Force Field - cost_reduces_per_power_played (action: play_card{card_index})
- [x] Fusion - channel_plasma (action: play_card{card_index})
- [x] Genetic Algorithm - block_increases_permanently (action: play_card{card_index})
- [x] Glacier - channel_2_frost (action: play_card{card_index})
- [x] Go for the Eyes - if_attacking_apply_weak (action: play_card{card_index,target_index})
- [x] Heatsinks - draw_on_power_play (action: play_card{card_index})
- [x] Hello World - add_common_card_each_turn (action: play_card{card_index})
- [x] Hologram - return_card_from_discard (action: play_card{card_index} + select_cards{pile:discard,card_indices})
- [x] Hyperbeam - lose_focus (action: play_card{card_index})
- [x] Lockon - apply_lockon (action: play_card{card_index,target_index})
- [x] Loop - trigger_orb_passive_extra (action: play_card{card_index})
- [x] Machine Learning - draw_extra_each_turn (action: play_card{card_index})
- [x] Melter - remove_enemy_block (action: play_card{card_index,target_index})
- [x] Meteor Strike - channel_3_plasma (action: play_card{card_index,target_index})
- [x] Multi-Cast - evoke_first_orb_x_times (action: play_card{card_index,energy_spent})
- [x] Rainbow - channel_lightning_frost_dark (action: play_card{card_index})
- [x] Reboot - shuffle_hand_and_discard_draw (action: play_card{card_index})
- [x] Rebound - next_card_on_top_of_draw (action: play_card{card_index,target_index})
- [x] Recycle - exhaust_card_gain_energy (action: play_card{card_index} + select_cards{pile:hand,card_indices})
- [x] Recursion - evoke_then_channel_same_orb (action: play_card{card_index})
- [x] Reinforced Body - block_x_times (action: play_card{card_index,energy_spent})
- [x] Reprogram - lose_focus_gain_strength_dex (action: play_card{card_index})
- [x] Rip and Tear - damage_random_enemy_twice (action: play_card{card_index} + target_index:auto)
- [x] Scrape - draw_discard_non_zero_cost (action: play_card{card_index,target_index})
- [x] Seek - search_draw_pile (action: play_card{card_index} + select_cards{pile:draw,card_indices})
- [x] Self Repair - heal_at_end_of_combat (action: play_card{card_index})
- [x] Stack - block_equals_discard_size (action: play_card{card_index})
- [x] Static Discharge - channel_lightning_on_damage (action: play_card{card_index})
- [x] Steam Barrier - lose_1_block_permanently (action: play_card{card_index})
- [x] Overclock - draw_cards, add_burn_to_discard (action: play_card{card_index})
- [x] Storm - channel_lightning_on_power_play (action: play_card{card_index})
- [x] Streamline - reduce_cost_permanently (action: play_card{card_index,target_index})
- [x] Sunder - if_fatal_gain_3_energy (action: play_card{card_index,target_index})
- [x] Tempest - channel_x_lightning (action: play_card{card_index,energy_spent})
- [x] Thunder Strike - damage_per_lightning_channeled (action: play_card{card_index,target_index})
- [x] Turbo - gain_energy_magic, add_void_to_discard (action: play_card{card_index})
- [x] Equilibrium - retain_hand (action: play_card{card_index})
- [x] White Noise - add_random_power_to_hand_cost_0 (action: play_card{card_index})
- [x] Zap - channel_lightning (action: play_card{card_index})
