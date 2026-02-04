# cards-defect work units

## Scope summary
- Implement missing Defect card effects from `packages/engine/content/cards.py`.
- Add orb system prerequisites (slots, channel/evoke, passives, Focus/Lock-On) needed by those cards.
- Keep scope to engine logic only (no UI, balance tuning, or non-Defect features).
- Model-facing actions only (no UI); see `docs/work_units/granular-actions.md`.

## Missing Defect card effects (by card)
Note: base-only cards are already handled (Strike_B, Defend_B, Leap, Boot Sequence, Beam Cell, Core Surge, Skim, Sweeping Beam).

Orb/Focus/Slot effects:
- Zap — `channel_lightning`
- Dualcast — `evoke_orb_twice`
- Ball Lightning — `channel_lightning`
- Cold Snap — `channel_frost`
- Coolheaded — `channel_frost`
- Chaos — `channel_random_orb`
- Chill — `channel_frost_per_enemy`
- Darkness — `channel_dark`
- Doom and Gloom — `channel_dark`
- Fusion — `channel_plasma`
- Glacier — `channel_2_frost`
- Tempest — `channel_x_lightning`
- Rainbow — `channel_lightning_frost_dark`
- Meteor Strike — `channel_3_plasma`
- Recursion — `evoke_then_channel_same_orb`
- Fission — `remove_orbs_gain_energy_and_draw`
- Multi-Cast — `evoke_first_orb_x_times`
- Barrage — `damage_per_orb`
- Compile Driver — `draw_per_unique_orb`
- Blizzard — `damage_per_frost_channeled`
- Thunder Strike — `damage_per_lightning_channeled`
- Lock-On — `apply_lockon` (orb damage multiplier)
- Capacitor — `increase_orb_slots`
- Defragment — `gain_focus`
- Consume — `gain_focus_lose_orb_slot`
- Biased Cognition — `gain_focus_lose_focus_each_turn`
- Hyperbeam — `lose_focus`
- Reprogram — `lose_focus_gain_strength_dex`
- Loop — `trigger_orb_passive_extra`
- Storm — `channel_lightning_on_power_play`
- Static Discharge — `channel_lightning_on_damage`
- Electrodynamics — `lightning_hits_all`, `channel_lightning`

Non-orb effects:
- Aggregate — `gain_energy_per_x_cards_in_draw`
- All For One — `return_all_0_cost_from_discard`
- Amplify — `next_power_plays_twice`
- Auto-Shields — `only_if_no_block`
- Buffer — `prevent_next_hp_loss`
- Claw — `increase_all_claw_damage`
- Charge Battery — `gain_1_energy_next_turn`
- Creative AI — `add_random_power_each_turn`
- Double Energy — `double_energy`
- Echo Form — `play_first_card_twice`
- FTL — `if_played_less_than_x_draw`
- Force Field — `cost_reduces_per_power_played`
- Genetic Algorithm — `block_increases_permanently`
- Go for the Eyes — `if_attacking_apply_weak`
- Heatsinks — `draw_on_power_play`
- Hello World — `add_common_card_each_turn`
- Hologram — `return_card_from_discard`
- Machine Learning — `draw_extra_each_turn`
- Melter — `remove_enemy_block`
- Reboot — `shuffle_hand_and_discard_draw`
- Rebound — `next_card_on_top_of_draw`
- Recycle — `exhaust_card_gain_energy`
- Reinforced Body — `block_x_times`
- Rip and Tear — `damage_random_enemy_twice`
- Scrape — `draw_discard_non_zero_cost`
- Seek — `search_draw_pile`
- Self Repair — `heal_at_end_of_combat`
- Stack — `block_equals_discard_size`
- Steam Barrier — `lose_1_block_permanently`
- Overclock — `draw`, `add_burn_to_discard`
- Streamline — `reduce_cost_permanently`
- Sunder — `if_fatal_gain_3_energy`
- Turbo — `add_void_to_discard`
- Equilibrium — `retain_hand`
- White Noise — `add_random_power_to_hand_cost_0`

## Orb-system prerequisites
- Add ordered orb state to `CombatState` (orbs list, max slots, per-orb stored values).
- Add `channel_orb`/`evoke_orb` helpers on effect + relic + power contexts; emit `onChannelOrb`/`onEvokeOrb` hooks.
- Implement per-turn orb passives (Lightning, Frost, Dark, Plasma) and evoke behavior.
- Apply Focus and Lock-On modifiers in orb damage/block calculations.
- Wire orb triggers for relics/potions (Data Disk, Nuclear Battery, Symbiotic Virus, Inserter, Frozen Core, Emotion Chip, Potion of Capacity, Essence of Darkness, Focus Potion).

## Suggested small task batches (with acceptance criteria)
- Orb state + helpers. Acceptance: orbs/slots persist in `CombatState.copy()`, channeling into full slots evokes leftmost, hooks fire for channel/evoke.
- Orb types + passives. Acceptance: Lightning/Frost/Dark/Plasma passive/evoke work with Focus and correct per-turn timing.
- Orb modifiers. Acceptance: Lock-On multiplies orb damage to target, Focus adjusts orb values without affecting non-orb damage.
- Channel/evoke cards. Acceptance: all channel/evoke cards above update orb state correctly, including X-cost and multi-evoke.
- Orb-scaling attacks. Acceptance: Barrage/Compile Driver/Blizzard/Thunder Strike scale with current orb state.
- Orb slot + orb-trigger powers/relics. Acceptance: Capacitor/Consume/Loop/Storm/Static Discharge/Electrodynamics plus orb relics/potions trigger correctly.
- Non-orb card effects. Acceptance: remaining non-orb effects execute and update state (energy, draw, discard, costs, block, powers).

## Files to touch
- `packages/engine/state/combat.py`
- `packages/engine/effects/cards.py` (or a new Defect effects module)
- `packages/engine/handlers/combat.py`
- `packages/engine/registry/powers.py`
- `packages/engine/registry/relics.py`
- `packages/engine/registry/potions.py`
- `packages/engine/content/cards.py`
- `packages/engine/content/powers.py`
- `packages/engine/calc/damage.py`
