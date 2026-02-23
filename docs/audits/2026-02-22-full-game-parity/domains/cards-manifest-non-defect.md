# Cards Manifest (Non-Defect)

Last updated: 2026-02-22

Scope: Java card classes in `red`, `green`, `purple`, `colorless`, `curses`, `status` (excluding `deprecated`, `optionCards`, `tempCards`, and Defect `blue` cards for this phase).

Status semantics:
- `exact`: behavior audited and locked against Java (none claimed yet in this manifest)
- `approximate`: card exists in Python and effect handlers resolve, but behavior parity still needs explicit lock
- `missing`: Java card ID has no Python inventory row

## Summary

| bucket | java cards | approximate | missing | rows with unresolved effect handlers |
|---|---:|---:|---:|---:|
| `red` | 75 | 75 | 0 | 0 |
| `green` | 75 | 75 | 0 | 0 |
| `purple` | 77 | 77 | 0 | 1 |
| `colorless` | 39 | 39 | 0 | 22 |
| `curses` | 14 | 14 | 0 | 0 |
| `status` | 5 | 5 | 0 | 0 |
| **total** | **285** | **285** | **0** | **23** |

## Manifest Rows

### `red`

| java_id | java_class | java_path | status | python_effect_keys | unresolved_effect_keys | test_ref_count | notes |
|---|---|---|---|---|---|---:|---|
| `Anger` | `Anger` | `red/Anger.java` | `approximate` | add_copy_to_discard | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Armaments` | `Armaments` | `red/Armaments.java` | `approximate` | upgrade_card_in_hand | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Barricade` | `Barricade` | `red/Barricade.java` | `approximate` | block_not_lost | none | 31 | effect handlers resolved; behavior parity audit still required |
| `Bash` | `Bash` | `red/Bash.java` | `approximate` | apply_vulnerable | none | 20 | effect handlers resolved; behavior parity audit still required |
| `Battle Trance` | `BattleTrance` | `red/BattleTrance.java` | `approximate` | draw_then_no_draw | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Berserk` | `Berserk` | `red/Berserk.java` | `approximate` | gain_vulnerable_gain_energy_per_turn | none | 6 | effect handlers resolved; behavior parity audit still required |
| `Blood for Blood` | `BloodForBlood` | `red/BloodForBlood.java` | `approximate` | cost_reduces_when_damaged | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Bloodletting` | `Bloodletting` | `red/Bloodletting.java` | `approximate` | lose_hp_gain_energy | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Bludgeon` | `Bludgeon` | `red/Bludgeon.java` | `approximate` | n/a | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Body Slam` | `BodySlam` | `red/BodySlam.java` | `approximate` | damage_equals_block | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Brutality` | `Brutality` | `red/Brutality.java` | `approximate` | start_turn_lose_hp_draw | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Burning Pact` | `BurningPact` | `red/BurningPact.java` | `approximate` | exhaust_to_draw | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Carnage` | `Carnage` | `red/Carnage.java` | `approximate` | n/a | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Clash` | `Clash` | `red/Clash.java` | `approximate` | only_attacks_in_hand | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Cleave` | `Cleave` | `red/Cleave.java` | `approximate` | n/a | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Clothesline` | `Clothesline` | `red/Clothesline.java` | `approximate` | apply_weak | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Combust` | `Combust` | `red/Combust.java` | `approximate` | end_turn_damage_all_lose_hp | none | 6 | effect handlers resolved; behavior parity audit still required |
| `Corruption` | `Corruption` | `red/Corruption.java` | `approximate` | skills_cost_0_exhaust | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Dark Embrace` | `DarkEmbrace` | `red/DarkEmbrace.java` | `approximate` | draw_on_exhaust | none | 11 | effect handlers resolved; behavior parity audit still required |
| `Defend_R` | `Defend_Red` | `red/Defend_Red.java` | `approximate` | n/a | none | 16 | effect handlers resolved; behavior parity audit still required |
| `Demon Form` | `DemonForm` | `red/DemonForm.java` | `approximate` | gain_strength_each_turn | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Disarm` | `Disarm` | `red/Disarm.java` | `approximate` | reduce_enemy_strength | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Double Tap` | `DoubleTap` | `red/DoubleTap.java` | `approximate` | play_attacks_twice | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Dropkick` | `Dropkick` | `red/Dropkick.java` | `approximate` | if_vulnerable_draw_and_energy | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Dual Wield` | `DualWield` | `red/DualWield.java` | `approximate` | copy_attack_or_power | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Entrench` | `Entrench` | `red/Entrench.java` | `approximate` | double_block | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Evolve` | `Evolve` | `red/Evolve.java` | `approximate` | draw_on_status | none | 11 | effect handlers resolved; behavior parity audit still required |
| `Exhume` | `Exhume` | `red/Exhume.java` | `approximate` | return_exhausted_card_to_hand | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Feed` | `Feed` | `red/Feed.java` | `approximate` | if_fatal_gain_max_hp | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Feel No Pain` | `FeelNoPain` | `red/FeelNoPain.java` | `approximate` | block_on_exhaust | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Fiend Fire` | `FiendFire` | `red/FiendFire.java` | `approximate` | exhaust_hand_damage_per_card | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Fire Breathing` | `FireBreathing` | `red/FireBreathing.java` | `approximate` | damage_on_status_curse | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Flame Barrier` | `FlameBarrier` | `red/FlameBarrier.java` | `approximate` | when_attacked_deal_damage | none | 15 | effect handlers resolved; behavior parity audit still required |
| `Flex` | `Flex` | `red/Flex.java` | `approximate` | gain_temp_strength | none | 9 | effect handlers resolved; behavior parity audit still required |
| `Ghostly Armor` | `GhostlyArmor` | `red/GhostlyArmor.java` | `approximate` | n/a | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Havoc` | `Havoc` | `red/Havoc.java` | `approximate` | play_top_card | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Headbutt` | `Headbutt` | `red/Headbutt.java` | `approximate` | put_card_from_discard_on_draw | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Heavy Blade` | `HeavyBlade` | `red/HeavyBlade.java` | `approximate` | strength_multiplier | none | 9 | effect handlers resolved; behavior parity audit still required |
| `Hemokinesis` | `Hemokinesis` | `red/Hemokinesis.java` | `approximate` | lose_hp | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Immolate` | `Immolate` | `red/Immolate.java` | `approximate` | add_burn_to_discard | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Impervious` | `Impervious` | `red/Impervious.java` | `approximate` | n/a | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Infernal Blade` | `InfernalBlade` | `red/InfernalBlade.java` | `approximate` | add_random_attack_cost_0 | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Inflame` | `Inflame` | `red/Inflame.java` | `approximate` | gain_strength | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Intimidate` | `Intimidate` | `red/Intimidate.java` | `approximate` | apply_weak_all | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Iron Wave` | `IronWave` | `red/IronWave.java` | `approximate` | n/a | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Juggernaut` | `Juggernaut` | `red/Juggernaut.java` | `approximate` | damage_random_on_block | none | 7 | effect handlers resolved; behavior parity audit still required |
| `Limit Break` | `LimitBreak` | `red/LimitBreak.java` | `approximate` | double_strength | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Metallicize` | `Metallicize` | `red/Metallicize.java` | `approximate` | end_turn_gain_block | none | 34 | effect handlers resolved; behavior parity audit still required |
| `Offering` | `Offering` | `red/Offering.java` | `approximate` | lose_hp_gain_energy_draw | none | 7 | effect handlers resolved; behavior parity audit still required |
| `Perfected Strike` | `PerfectedStrike` | `red/PerfectedStrike.java` | `approximate` | damage_per_strike | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Pommel Strike` | `PommelStrike` | `red/PommelStrike.java` | `approximate` | draw_cards | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Power Through` | `PowerThrough` | `red/PowerThrough.java` | `approximate` | add_wounds_to_hand | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Pummel` | `Pummel` | `red/Pummel.java` | `approximate` | damage_x_times | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Rage` | `Rage` | `red/Rage.java` | `approximate` | gain_block_per_attack | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Rampage` | `Rampage` | `red/Rampage.java` | `approximate` | increase_damage_on_use | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Reaper` | `Reaper` | `red/Reaper.java` | `approximate` | damage_all_heal_unblocked | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Reckless Charge` | `RecklessCharge` | `red/RecklessCharge.java` | `approximate` | shuffle_dazed_into_draw | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Rupture` | `Rupture` | `red/Rupture.java` | `approximate` | gain_strength_on_hp_loss | none | 8 | effect handlers resolved; behavior parity audit still required |
| `Searing Blow` | `SearingBlow` | `red/SearingBlow.java` | `approximate` | can_upgrade_unlimited | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Second Wind` | `SecondWind` | `red/SecondWind.java` | `approximate` | exhaust_non_attacks_gain_block | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Seeing Red` | `SeeingRed` | `red/SeeingRed.java` | `approximate` | gain_2_energy | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Sentinel` | `Sentinel` | `red/Sentinel.java` | `approximate` | gain_energy_on_exhaust_2_3 | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Sever Soul` | `SeverSoul` | `red/SeverSoul.java` | `approximate` | exhaust_all_non_attacks | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Shockwave` | `Shockwave` | `red/Shockwave.java` | `approximate` | apply_weak_and_vulnerable_all | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Shrug It Off` | `ShrugItOff` | `red/ShrugItOff.java` | `approximate` | draw_1 | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Spot Weakness` | `SpotWeakness` | `red/SpotWeakness.java` | `approximate` | gain_strength_if_enemy_attacking | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Strike_R` | `Strike_Red` | `red/Strike_Red.java` | `approximate` | n/a | none | 28 | effect handlers resolved; behavior parity audit still required |
| `Sword Boomerang` | `SwordBoomerang` | `red/SwordBoomerang.java` | `approximate` | random_enemy_x_times | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Thunderclap` | `ThunderClap` | `red/ThunderClap.java` | `approximate` | apply_vulnerable_1_all | none | 2 | effect handlers resolved; behavior parity audit still required |
| `True Grit` | `TrueGrit` | `red/TrueGrit.java` | `approximate` | exhaust_random_card | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Twin Strike` | `TwinStrike` | `red/TwinStrike.java` | `approximate` | damage_x_times | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Uppercut` | `Uppercut` | `red/Uppercut.java` | `approximate` | apply_weak_and_vulnerable | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Warcry` | `Warcry` | `red/Warcry.java` | `approximate` | draw_then_put_on_draw | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Whirlwind` | `Whirlwind` | `red/Whirlwind.java` | `approximate` | damage_all_x_times | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Wild Strike` | `WildStrike` | `red/WildStrike.java` | `approximate` | shuffle_wound_into_draw | none | 2 | effect handlers resolved; behavior parity audit still required |

### `green`

| java_id | java_class | java_path | status | python_effect_keys | unresolved_effect_keys | test_ref_count | notes |
|---|---|---|---|---|---|---:|---|
| `A Thousand Cuts` | `AThousandCuts` | `green/AThousandCuts.java` | `approximate` | deal_damage_per_card_played | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Accuracy` | `Accuracy` | `green/Accuracy.java` | `approximate` | shivs_deal_more_damage | none | 9 | effect handlers resolved; behavior parity audit still required |
| `Acrobatics` | `Acrobatics` | `green/Acrobatics.java` | `approximate` | draw_x, discard_1 | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Adrenaline` | `Adrenaline` | `green/Adrenaline.java` | `approximate` | gain_energy, draw_2 | none | 4 | effect handlers resolved; behavior parity audit still required |
| `After Image` | `AfterImage` | `green/AfterImage.java` | `approximate` | gain_1_block_per_card_played | none | 8 | effect handlers resolved; behavior parity audit still required |
| `All Out Attack` | `AllOutAttack` | `green/AllOutAttack.java` | `approximate` | discard_random_1 | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Backflip` | `Backflip` | `green/Backflip.java` | `approximate` | draw_2 | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Backstab` | `Backstab` | `green/Backstab.java` | `approximate` | n/a | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Bane` | `Bane` | `green/Bane.java` | `approximate` | double_damage_if_poisoned | none | 21 | effect handlers resolved; behavior parity audit still required |
| `Blade Dance` | `BladeDance` | `green/BladeDance.java` | `approximate` | add_shivs_to_hand | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Blur` | `Blur` | `green/Blur.java` | `approximate` | block_not_removed_next_turn | none | 16 | effect handlers resolved; behavior parity audit still required |
| `Bouncing Flask` | `BouncingFlask` | `green/BouncingFlask.java` | `approximate` | apply_poison_random_3_times | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Bullet Time` | `BulletTime` | `green/BulletTime.java` | `approximate` | no_draw_this_turn, cards_cost_0_this_turn | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Burst` | `Burst` | `green/Burst.java` | `approximate` | double_next_skills | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Calculated Gamble` | `CalculatedGamble` | `green/CalculatedGamble.java` | `approximate` | discard_hand_draw_same | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Caltrops` | `Caltrops` | `green/Caltrops.java` | `approximate` | gain_thorns | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Catalyst` | `Catalyst` | `green/Catalyst.java` | `approximate` | double_poison | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Choke` | `Choke` | `green/Choke.java` | `approximate` | apply_choke | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Cloak And Dagger` | `CloakAndDagger` | `green/CloakAndDagger.java` | `approximate` | add_shivs_to_hand | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Concentrate` | `Concentrate` | `green/Concentrate.java` | `approximate` | discard_x, gain_energy_2 | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Corpse Explosion` | `CorpseExplosion` | `green/CorpseExplosion.java` | `approximate` | apply_poison, apply_corpse_explosion | none | 6 | effect handlers resolved; behavior parity audit still required |
| `Crippling Poison` | `CripplingPoison` | `green/CripplingPoison.java` | `approximate` | apply_poison_all, apply_weak_2_all | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Dagger Spray` | `DaggerSpray` | `green/DaggerSpray.java` | `approximate` | damage_all_x_times | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Dagger Throw` | `DaggerThrow` | `green/DaggerThrow.java` | `approximate` | draw_1, discard_1 | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Dash` | `Dash` | `green/Dash.java` | `approximate` | n/a | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Deadly Poison` | `DeadlyPoison` | `green/DeadlyPoison.java` | `approximate` | apply_poison | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Defend_G` | `Defend_Green` | `green/Defend_Green.java` | `approximate` | n/a | none | 3 | effect handlers resolved; behavior parity audit still required |
| `Deflect` | `Deflect` | `green/Deflect.java` | `approximate` | n/a | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Die Die Die` | `DieDieDie` | `green/DieDieDie.java` | `approximate` | n/a | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Distraction` | `Distraction` | `green/Distraction.java` | `approximate` | add_random_skill_cost_0 | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Dodge and Roll` | `DodgeAndRoll` | `green/DodgeAndRoll.java` | `approximate` | block_next_turn | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Doppelganger` | `Doppelganger` | `green/Doppelganger.java` | `approximate` | draw_x_next_turn, gain_x_energy_next_turn | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Endless Agony` | `EndlessAgony` | `green/EndlessAgony.java` | `approximate` | copy_to_hand_when_drawn | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Envenom` | `Envenom` | `green/Envenom.java` | `approximate` | attacks_apply_poison | none | 8 | effect handlers resolved; behavior parity audit still required |
| `Escape Plan` | `EscapePlan` | `green/EscapePlan.java` | `approximate` | draw_1, if_skill_drawn_gain_block | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Eviscerate` | `Eviscerate` | `green/Eviscerate.java` | `approximate` | cost_reduces_per_discard, damage_x_times | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Expertise` | `Expertise` | `green/Expertise.java` | `approximate` | draw_to_x_cards | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Finisher` | `Finisher` | `green/Finisher.java` | `approximate` | damage_per_attack_this_turn | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Flechettes` | `Flechettes` | `green/Flechettes.java` | `approximate` | damage_per_skill_in_hand | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Flying Knee` | `FlyingKnee` | `green/FlyingKnee.java` | `approximate` | gain_energy_next_turn_1 | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Footwork` | `Footwork` | `green/Footwork.java` | `approximate` | gain_dexterity | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Glass Knife` | `GlassKnife` | `green/GlassKnife.java` | `approximate` | damage_x_times, reduce_damage_by_2 | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Grand Finale` | `GrandFinale` | `green/GrandFinale.java` | `approximate` | only_playable_if_draw_pile_empty | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Heel Hook` | `HeelHook` | `green/HeelHook.java` | `approximate` | if_target_weak_gain_energy_draw | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Infinite Blades` | `InfiniteBlades` | `green/InfiniteBlades.java` | `approximate` | add_shiv_each_turn | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Leg Sweep` | `LegSweep` | `green/LegSweep.java` | `approximate` | apply_weak | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Malaise` | `Malaise` | `green/Malaise.java` | `approximate` | apply_weak_x, apply_strength_down_x | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Masterful Stab` | `MasterfulStab` | `green/MasterfulStab.java` | `approximate` | cost_increases_when_damaged | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Neutralize` | `Neutralize` | `green/Neutralize.java` | `approximate` | apply_weak | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Night Terror` | `Nightmare` | `green/Nightmare.java` | `approximate` | copy_card_to_hand_next_turn | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Noxious Fumes` | `NoxiousFumes` | `green/NoxiousFumes.java` | `approximate` | apply_poison_all_each_turn | none | 6 | effect handlers resolved; behavior parity audit still required |
| `Outmaneuver` | `Outmaneuver` | `green/Outmaneuver.java` | `approximate` | gain_energy_next_turn | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Phantasmal Killer` | `PhantasmalKiller` | `green/PhantasmalKiller.java` | `approximate` | double_damage_next_turn | none | 4 | effect handlers resolved; behavior parity audit still required |
| `PiercingWail` | `PiercingWail` | `green/PiercingWail.java` | `approximate` | reduce_strength_all_enemies | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Poisoned Stab` | `PoisonedStab` | `green/PoisonedStab.java` | `approximate` | apply_poison | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Predator` | `Predator` | `green/Predator.java` | `approximate` | draw_2_next_turn | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Prepared` | `Prepared` | `green/Prepared.java` | `approximate` | draw_x, discard_x | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Quick Slash` | `QuickSlash` | `green/QuickSlash.java` | `approximate` | draw_1 | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Reflex` | `Reflex` | `green/Reflex.java` | `approximate` | unplayable, when_discarded_draw | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Riddle With Holes` | `RiddleWithHoles` | `green/RiddleWithHoles.java` | `approximate` | damage_x_times | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Setup` | `Setup` | `green/Setup.java` | `approximate` | put_card_on_draw_pile_cost_0 | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Skewer` | `Skewer` | `green/Skewer.java` | `approximate` | damage_x_times_energy | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Slice` | `Slice` | `green/Slice.java` | `approximate` | n/a | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Storm of Steel` | `StormOfSteel` | `green/StormOfSteel.java` | `approximate` | discard_hand, add_shivs_equal_to_discarded | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Strike_G` | `Strike_Green` | `green/Strike_Green.java` | `approximate` | n/a | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Sucker Punch` | `SuckerPunch` | `green/SuckerPunch.java` | `approximate` | apply_weak | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Survivor` | `Survivor` | `green/Survivor.java` | `approximate` | discard_1 | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Tactician` | `Tactician` | `green/Tactician.java` | `approximate` | unplayable, when_discarded_gain_energy | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Terror` | `Terror` | `green/Terror.java` | `approximate` | apply_vulnerable | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Tools of the Trade` | `ToolsOfTheTrade` | `green/ToolsOfTheTrade.java` | `approximate` | draw_1_discard_1_each_turn | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Underhanded Strike` | `SneakyStrike` | `green/SneakyStrike.java` | `approximate` | refund_2_energy_if_discarded_this_turn | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Unload` | `Unload` | `green/Unload.java` | `approximate` | discard_non_attacks | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Venomology` | `Alchemize` | `green/Alchemize.java` | `approximate` | obtain_random_potion | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Well Laid Plans` | `WellLaidPlans` | `green/WellLaidPlans.java` | `approximate` | retain_cards_each_turn | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Wraith Form v2` | `WraithForm` | `green/WraithForm.java` | `approximate` | gain_intangible, lose_1_dexterity_each_turn | none | 3 | effect handlers resolved; behavior parity audit still required |

### `purple`

| java_id | java_class | java_path | status | python_effect_keys | unresolved_effect_keys | test_ref_count | notes |
|---|---|---|---|---|---|---:|---|
| `Discipline` | `Discipline` | `purple/Discipline.java` | `approximate` | apply_discipline_power | none | 3 | card + deprecated `DisciplinePower` hooks implemented (`atEndOfTurn` save energy, `atStartOfTurn` draw/reset) |
| `Adaptation` | `Rushdown` | `purple/Rushdown.java` | `approximate` | on_wrath_draw | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Alpha` | `Alpha` | `purple/Alpha.java` | `approximate` | shuffle_beta_into_draw | none | 14 | effect handlers resolved; behavior parity audit still required |
| `BattleHymn` | `BattleHymn` | `purple/BattleHymn.java` | `approximate` | add_smite_each_turn | none | 7 | effect handlers resolved; behavior parity audit still required |
| `Blasphemy` | `Blasphemy` | `purple/Blasphemy.java` | `approximate` | enter_divinity, die_next_turn | none | 19 | effect handlers resolved; behavior parity audit still required |
| `BowlingBash` | `BowlingBash` | `purple/BowlingBash.java` | `approximate` | damage_per_enemy | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Brilliance` | `Brilliance` | `purple/Brilliance.java` | `approximate` | damage_plus_mantra_gained | none | 3 | effect handlers resolved; behavior parity audit still required |
| `CarveReality` | `CarveReality` | `purple/CarveReality.java` | `approximate` | add_smite_to_hand | none | 2 | effect handlers resolved; behavior parity audit still required |
| `ClearTheMind` | `Tranquility` | `purple/Tranquility.java` | `approximate` | n/a | none | 9 | effect handlers resolved; behavior parity audit still required |
| `Collect` | `Collect` | `purple/Collect.java` | `approximate` | put_x_miracles_on_draw | none | 7 | effect handlers resolved; behavior parity audit still required |
| `Conclude` | `Conclude` | `purple/Conclude.java` | `approximate` | end_turn | none | 10 | effect handlers resolved; behavior parity audit still required |
| `ConjureBlade` | `ConjureBlade` | `purple/ConjureBlade.java` | `approximate` | add_expunger_to_hand | none | 3 | effect handlers resolved; behavior parity audit still required |
| `Consecrate` | `Consecrate` | `purple/Consecrate.java` | `approximate` | n/a | none | 9 | effect handlers resolved; behavior parity audit still required |
| `Crescendo` | `Crescendo` | `purple/Crescendo.java` | `approximate` | n/a | none | 12 | effect handlers resolved; behavior parity audit still required |
| `CrushJoints` | `CrushJoints` | `purple/CrushJoints.java` | `approximate` | if_last_card_skill_vulnerable | none | 3 | effect handlers resolved; behavior parity audit still required |
| `CutThroughFate` | `CutThroughFate` | `purple/CutThroughFate.java` | `approximate` | scry, draw_1 | none | 3 | effect handlers resolved; behavior parity audit still required |
| `DeceiveReality` | `DeceiveReality` | `purple/DeceiveReality.java` | `approximate` | add_safety_to_hand | none | 3 | effect handlers resolved; behavior parity audit still required |
| `Defend_P` | `Defend_Watcher` | `purple/Defend_Watcher.java` | `approximate` | n/a | none | 76 | effect handlers resolved; behavior parity audit still required |
| `DeusExMachina` | `DeusExMachina` | `purple/DeusExMachina.java` | `approximate` | on_draw_add_miracles_and_exhaust | none | 0 | effect handlers resolved; behavior parity audit still required |
| `DevaForm` | `DevaForm` | `purple/DevaForm.java` | `approximate` | gain_energy_each_turn_stacking | none | 13 | effect handlers resolved; behavior parity audit still required |
| `Devotion` | `Devotion` | `purple/Devotion.java` | `approximate` | gain_mantra_each_turn | none | 8 | effect handlers resolved; behavior parity audit still required |
| `EmptyBody` | `EmptyBody` | `purple/EmptyBody.java` | `approximate` | n/a | none | 8 | effect handlers resolved; behavior parity audit still required |
| `EmptyFist` | `EmptyFist` | `purple/EmptyFist.java` | `approximate` | n/a | none | 7 | effect handlers resolved; behavior parity audit still required |
| `EmptyMind` | `EmptyMind` | `purple/EmptyMind.java` | `approximate` | draw_cards | none | 7 | effect handlers resolved; behavior parity audit still required |
| `Eruption` | `Eruption` | `purple/Eruption.java` | `approximate` | n/a | none | 57 | effect handlers resolved; behavior parity audit still required |
| `Establishment` | `Establishment` | `purple/Establishment.java` | `approximate` | retained_cards_cost_less | none | 9 | effect handlers resolved; behavior parity audit still required |
| `Evaluate` | `Evaluate` | `purple/Evaluate.java` | `approximate` | add_insight_to_draw | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Fasting2` | `Fasting` | `purple/Fasting.java` | `approximate` | gain_strength_and_dex_lose_focus | none | 2 | effect handlers resolved; behavior parity audit still required |
| `FearNoEvil` | `FearNoEvil` | `purple/FearNoEvil.java` | `approximate` | if_enemy_attacking_enter_calm | none | 3 | effect handlers resolved; behavior parity audit still required |
| `FlurryOfBlows` | `FlurryOfBlows` | `purple/FlurryOfBlows.java` | `approximate` | on_stance_change_play_from_discard | none | 24 | effect handlers resolved; behavior parity audit still required |
| `FlyingSleeves` | `FlyingSleeves` | `purple/FlyingSleeves.java` | `approximate` | damage_twice | none | 5 | effect handlers resolved; behavior parity audit still required |
| `FollowUp` | `FollowUp` | `purple/FollowUp.java` | `approximate` | if_last_card_attack_gain_energy | none | 3 | effect handlers resolved; behavior parity audit still required |
| `ForeignInfluence` | `ForeignInfluence` | `purple/ForeignInfluence.java` | `approximate` | choose_attack_from_any_class | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Halt` | `Halt` | `purple/Halt.java` | `approximate` | if_in_wrath_extra_block_6 | none | 9 | effect handlers resolved; behavior parity audit still required |
| `Indignation` | `Indignation` | `purple/Indignation.java` | `approximate` | if_wrath_gain_mantra_else_wrath | none | 5 | effect handlers resolved; behavior parity audit still required |
| `InnerPeace` | `InnerPeace` | `purple/InnerPeace.java` | `approximate` | if_calm_draw_else_calm | none | 3 | effect handlers resolved; behavior parity audit still required |
| `Judgement` | `Judgement` | `purple/Judgement.java` | `approximate` | if_enemy_hp_below_kill | none | 7 | effect handlers resolved; behavior parity audit still required |
| `JustLucky` | `JustLucky` | `purple/JustLucky.java` | `approximate` | scry, gain_block | none | 3 | effect handlers resolved; behavior parity audit still required |
| `LessonLearned` | `LessonLearned` | `purple/LessonLearned.java` | `approximate` | if_fatal_upgrade_random_card | none | 3 | effect handlers resolved; behavior parity audit still required |
| `LikeWater` | `LikeWater` | `purple/LikeWater.java` | `approximate` | if_calm_end_turn_gain_block | none | 9 | effect handlers resolved; behavior parity audit still required |
| `MasterReality` | `MasterReality` | `purple/MasterReality.java` | `approximate` | created_cards_upgraded | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Meditate` | `Meditate` | `purple/Meditate.java` | `approximate` | put_cards_from_discard_to_hand, enter_calm, end_turn | none | 10 | effect handlers resolved; behavior parity audit still required |
| `MentalFortress` | `MentalFortress` | `purple/MentalFortress.java` | `approximate` | on_stance_change_gain_block | none | 19 | effect handlers resolved; behavior parity audit still required |
| `Nirvana` | `Nirvana` | `purple/Nirvana.java` | `approximate` | on_scry_gain_block | none | 31 | effect handlers resolved; behavior parity audit still required |
| `Omniscience` | `Omniscience` | `purple/Omniscience.java` | `approximate` | play_card_from_draw_twice | none | 7 | effect handlers resolved; behavior parity audit still required |
| `PathToVictory` | `PressurePoints` | `purple/PressurePoints.java` | `approximate` | apply_mark, trigger_all_marks | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Perseverance` | `Perseverance` | `purple/Perseverance.java` | `approximate` | gains_block_when_retained | none | 6 | effect handlers resolved; behavior parity audit still required |
| `Pray` | `Pray` | `purple/Pray.java` | `approximate` | gain_mantra_add_insight | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Prostrate` | `Prostrate` | `purple/Prostrate.java` | `approximate` | gain_mantra | none | 6 | effect handlers resolved; behavior parity audit still required |
| `Protect` | `Protect` | `purple/Protect.java` | `approximate` | n/a | none | 8 | effect handlers resolved; behavior parity audit still required |
| `Ragnarok` | `Ragnarok` | `purple/Ragnarok.java` | `approximate` | damage_random_x_times | none | 9 | effect handlers resolved; behavior parity audit still required |
| `ReachHeaven` | `ReachHeaven` | `purple/ReachHeaven.java` | `approximate` | add_through_violence_to_draw | none | 3 | effect handlers resolved; behavior parity audit still required |
| `Sanctity` | `Sanctity` | `purple/Sanctity.java` | `approximate` | if_last_skill_draw_2 | none | 8 | effect handlers resolved; behavior parity audit still required |
| `SandsOfTime` | `SandsOfTime` | `purple/SandsOfTime.java` | `approximate` | cost_reduces_each_turn | none | 3 | effect handlers resolved; behavior parity audit still required |
| `SashWhip` | `SashWhip` | `purple/SashWhip.java` | `approximate` | if_last_card_attack_weak | none | 3 | effect handlers resolved; behavior parity audit still required |
| `Scrawl` | `Scrawl` | `purple/Scrawl.java` | `approximate` | draw_until_hand_full | none | 11 | effect handlers resolved; behavior parity audit still required |
| `SignatureMove` | `SignatureMove` | `purple/SignatureMove.java` | `approximate` | only_attack_in_hand | none | 7 | effect handlers resolved; behavior parity audit still required |
| `SpiritShield` | `SpiritShield` | `purple/SpiritShield.java` | `approximate` | gain_block_per_card_in_hand | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Strike_P` | `Strike_Purple` | `purple/Strike_Purple.java` | `approximate` | n/a | none | 174 | effect handlers resolved; behavior parity audit still required |
| `Study` | `Study` | `purple/Study.java` | `approximate` | add_insight_end_turn | none | 11 | effect handlers resolved; behavior parity audit still required |
| `Swivel` | `Swivel` | `purple/Swivel.java` | `approximate` | free_attack_next_turn | none | 1 | effect handlers resolved; behavior parity audit still required |
| `TalkToTheHand` | `TalkToTheHand` | `purple/TalkToTheHand.java` | `approximate` | apply_block_return | none | 4 | effect handlers resolved; behavior parity audit still required |
| `Tantrum` | `Tantrum` | `purple/Tantrum.java` | `approximate` | damage_x_times | none | 47 | effect handlers resolved; behavior parity audit still required |
| `ThirdEye` | `ThirdEye` | `purple/ThirdEye.java` | `approximate` | scry | none | 3 | effect handlers resolved; behavior parity audit still required |
| `Unraveling` | `Unraveling` | `purple/Unraveling.java` | `approximate` | scry_draw_pile_discard_for_block | scry_draw_pile_discard_for_block | 0 | unresolved effect handlers present |
| `Vault` | `Vault` | `purple/Vault.java` | `approximate` | take_extra_turn | none | 9 | effect handlers resolved; behavior parity audit still required |
| `Vengeance` | `SimmeringFury` | `purple/SimmeringFury.java` | `approximate` | wrath_next_turn_draw_next_turn | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Vigilance` | `Vigilance` | `purple/Vigilance.java` | `approximate` | n/a | none | 73 | effect handlers resolved; behavior parity audit still required |
| `Wallop` | `Wallop` | `purple/Wallop.java` | `approximate` | gain_block_equal_unblocked_damage | none | 6 | effect handlers resolved; behavior parity audit still required |
| `WaveOfTheHand` | `WaveOfTheHand` | `purple/WaveOfTheHand.java` | `approximate` | block_gain_applies_weak | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Weave` | `Weave` | `purple/Weave.java` | `approximate` | on_scry_play_from_discard | none | 12 | effect handlers resolved; behavior parity audit still required |
| `WheelKick` | `WheelKick` | `purple/WheelKick.java` | `approximate` | draw_2 | none | 5 | effect handlers resolved; behavior parity audit still required |
| `WindmillStrike` | `WindmillStrike` | `purple/WindmillStrike.java` | `approximate` | gain_damage_when_retained_4 | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Wireheading` | `Foresight` | `purple/Foresight.java` | `approximate` | scry_each_turn | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Wish` | `Wish` | `purple/Wish.java` | `approximate` | choose_plated_armor_or_strength_or_gold | none | 10 | effect handlers resolved; behavior parity audit still required |
| `Worship` | `Worship` | `purple/Worship.java` | `approximate` | gain_mantra | none | 8 | effect handlers resolved; behavior parity audit still required |
| `WreathOfFlame` | `WreathOfFlame` | `purple/WreathOfFlame.java` | `approximate` | next_attack_plus_damage | none | 4 | effect handlers resolved; behavior parity audit still required |

### `colorless`

| java_id | java_class | java_path | status | python_effect_keys | unresolved_effect_keys | test_ref_count | notes |
|---|---|---|---|---|---|---:|---|
| `Apotheosis` | `Apotheosis` | `colorless/Apotheosis.java` | `approximate` | upgrade_all_cards_in_combat | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Bandage Up` | `BandageUp` | `colorless/BandageUp.java` | `approximate` | heal_magic_number | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Bite` | `Bite` | `colorless/Bite.java` | `approximate` | heal_magic_number | none | 6 | effect handlers resolved; behavior parity audit still required |
| `Blind` | `Blind` | `colorless/Blind.java` | `approximate` | apply_weak | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Chrysalis` | `Chrysalis` | `colorless/Chrysalis.java` | `approximate` | add_random_skills_to_draw_cost_0 | add_random_skills_to_draw_cost_0 | 0 | unresolved effect handlers present |
| `Dark Shackles` | `DarkShackles` | `colorless/DarkShackles.java` | `approximate` | apply_temp_strength_down | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Deep Breath` | `DeepBreath` | `colorless/DeepBreath.java` | `approximate` | shuffle_discard_into_draw, draw_cards | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Discovery` | `Discovery` | `colorless/Discovery.java` | `approximate` | discover_card | discover_card | 12 | unresolved effect handlers present |
| `Dramatic Entrance` | `DramaticEntrance` | `colorless/DramaticEntrance.java` | `approximate` | n/a | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Enlightenment` | `Enlightenment` | `colorless/Enlightenment.java` | `approximate` | reduce_hand_cost_to_1 | reduce_hand_cost_to_1 | 0 | unresolved effect handlers present |
| `Finesse` | `Finesse` | `colorless/Finesse.java` | `approximate` | draw_1 | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Flash of Steel` | `FlashOfSteel` | `colorless/FlashOfSteel.java` | `approximate` | draw_1 | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Forethought` | `Forethought` | `colorless/Forethought.java` | `approximate` | put_card_on_bottom_of_draw_cost_0 | put_card_on_bottom_of_draw_cost_0 | 0 | unresolved effect handlers present |
| `Ghostly` | `Apparition` | `colorless/Apparition.java` | `approximate` | gain_intangible_1 | none | 2 | effect handlers resolved; behavior parity audit still required |
| `Good Instincts` | `GoodInstincts` | `colorless/GoodInstincts.java` | `approximate` | n/a | none | 0 | effect handlers resolved; behavior parity audit still required |
| `HandOfGreed` | `HandOfGreed` | `colorless/HandOfGreed.java` | `approximate` | if_fatal_gain_gold | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Impatience` | `Impatience` | `colorless/Impatience.java` | `approximate` | draw_if_no_attacks_in_hand | draw_if_no_attacks_in_hand | 0 | unresolved effect handlers present |
| `J.A.X.` | `JAX` | `colorless/JAX.java` | `approximate` | lose_3_hp_gain_strength | lose_3_hp_gain_strength | 2 | unresolved effect handlers present |
| `Jack Of All Trades` | `JackOfAllTrades` | `colorless/JackOfAllTrades.java` | `approximate` | add_random_colorless_to_hand | add_random_colorless_to_hand | 0 | unresolved effect handlers present |
| `Madness` | `Madness` | `colorless/Madness.java` | `approximate` | reduce_random_card_cost_to_0 | reduce_random_card_cost_to_0 | 2 | unresolved effect handlers present |
| `Magnetism` | `Magnetism` | `colorless/Magnetism.java` | `approximate` | add_random_colorless_each_turn | add_random_colorless_each_turn | 0 | unresolved effect handlers present |
| `Master of Strategy` | `MasterOfStrategy` | `colorless/MasterOfStrategy.java` | `approximate` | draw_cards | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Mayhem` | `Mayhem` | `colorless/Mayhem.java` | `approximate` | auto_play_top_card_each_turn | auto_play_top_card_each_turn | 0 | unresolved effect handlers present |
| `Metamorphosis` | `Metamorphosis` | `colorless/Metamorphosis.java` | `approximate` | add_random_attacks_to_draw_cost_0 | add_random_attacks_to_draw_cost_0 | 0 | unresolved effect handlers present |
| `Mind Blast` | `MindBlast` | `colorless/MindBlast.java` | `approximate` | damage_equals_draw_pile_size | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Panacea` | `Panacea` | `colorless/Panacea.java` | `approximate` | gain_artifact | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Panache` | `Panache` | `colorless/Panache.java` | `approximate` | every_5_cards_deal_damage_to_all | every_5_cards_deal_damage_to_all | 0 | unresolved effect handlers present |
| `PanicButton` | `PanicButton` | `colorless/PanicButton.java` | `approximate` | gain_no_block_next_2_turns | gain_no_block_next_2_turns | 0 | unresolved effect handlers present |
| `Purity` | `Purity` | `colorless/Purity.java` | `approximate` | exhaust_up_to_x_cards | exhaust_up_to_x_cards | 0 | unresolved effect handlers present |
| `RitualDagger` | `RitualDagger` | `colorless/RitualDagger.java` | `approximate` | if_fatal_permanently_increase_damage | if_fatal_permanently_increase_damage | 0 | unresolved effect handlers present |
| `Sadistic Nature` | `SadisticNature` | `colorless/SadisticNature.java` | `approximate` | on_debuff_deal_damage | on_debuff_deal_damage | 0 | unresolved effect handlers present |
| `Secret Technique` | `SecretTechnique` | `colorless/SecretTechnique.java` | `approximate` | search_draw_for_skill | search_draw_for_skill | 0 | unresolved effect handlers present |
| `Secret Weapon` | `SecretWeapon` | `colorless/SecretWeapon.java` | `approximate` | search_draw_for_attack | search_draw_for_attack | 0 | unresolved effect handlers present |
| `Swift Strike` | `SwiftStrike` | `colorless/SwiftStrike.java` | `approximate` | n/a | none | 0 | effect handlers resolved; behavior parity audit still required |
| `The Bomb` | `TheBomb` | `colorless/TheBomb.java` | `approximate` | deal_damage_to_all_after_3_turns | deal_damage_to_all_after_3_turns | 0 | unresolved effect handlers present |
| `Thinking Ahead` | `ThinkingAhead` | `colorless/ThinkingAhead.java` | `approximate` | draw_2_put_1_on_top_of_draw | draw_2_put_1_on_top_of_draw | 0 | unresolved effect handlers present |
| `Transmutation` | `Transmutation` | `colorless/Transmutation.java` | `approximate` | add_x_random_colorless_cost_0 | add_x_random_colorless_cost_0 | 0 | unresolved effect handlers present |
| `Trip` | `Trip` | `colorless/Trip.java` | `approximate` | apply_vulnerable | none | 0 | effect handlers resolved; behavior parity audit still required |
| `Violence` | `Violence` | `colorless/Violence.java` | `approximate` | put_attacks_from_draw_into_hand | put_attacks_from_draw_into_hand | 4 | unresolved effect handlers present |

### `curses`

| java_id | java_class | java_path | status | python_effect_keys | unresolved_effect_keys | test_ref_count | notes |
|---|---|---|---|---|---|---:|---|
| `AscendersBane` | `AscendersBane` | `curses/AscendersBane.java` | `approximate` | unplayable, cannot_be_removed | none | 13 | effect handlers resolved; behavior parity audit still required |
| `Clumsy` | `Clumsy` | `curses/Clumsy.java` | `approximate` | unplayable | none | 10 | effect handlers resolved; behavior parity audit still required |
| `CurseOfTheBell` | `CurseOfTheBell` | `curses/CurseOfTheBell.java` | `approximate` | unplayable, cannot_be_removed | none | 5 | effect handlers resolved; behavior parity audit still required |
| `Decay` | `Decay` | `curses/Decay.java` | `approximate` | unplayable, end_of_turn_take_2_damage | none | 29 | effect handlers resolved; behavior parity audit still required |
| `Doubt` | `Doubt` | `curses/Doubt.java` | `approximate` | unplayable, end_of_turn_gain_weak_1 | none | 34 | effect handlers resolved; behavior parity audit still required |
| `Injury` | `Injury` | `curses/Injury.java` | `approximate` | unplayable | none | 21 | effect handlers resolved; behavior parity audit still required |
| `Necronomicurse` | `Necronomicurse` | `curses/Necronomicurse.java` | `approximate` | unplayable, returns_when_exhausted_or_removed | none | 21 | effect handlers resolved; behavior parity audit still required |
| `Normality` | `Normality` | `curses/Normality.java` | `approximate` | unplayable, limit_3_cards_per_turn | none | 18 | effect handlers resolved; behavior parity audit still required |
| `Pain` | `Pain` | `curses/Pain.java` | `approximate` | unplayable, lose_1_hp_when_other_card_played | none | 19 | effect handlers resolved; behavior parity audit still required |
| `Parasite` | `Parasite` | `curses/Parasite.java` | `approximate` | unplayable, lose_3_max_hp_when_removed | none | 18 | effect handlers resolved; behavior parity audit still required |
| `Pride` | `Pride` | `curses/Pride.java` | `approximate` | end_of_turn_add_copy_to_draw | none | 13 | effect handlers resolved; behavior parity audit still required |
| `Regret` | `Regret` | `curses/Regret.java` | `approximate` | unplayable, end_of_turn_lose_hp_equal_to_hand_size | none | 40 | effect handlers resolved; behavior parity audit still required |
| `Shame` | `Shame` | `curses/Shame.java` | `approximate` | unplayable, end_of_turn_gain_frail_1 | none | 21 | effect handlers resolved; behavior parity audit still required |
| `Writhe` | `Writhe` | `curses/Writhe.java` | `approximate` | unplayable | none | 11 | effect handlers resolved; behavior parity audit still required |

### `status`

| java_id | java_class | java_path | status | python_effect_keys | unresolved_effect_keys | test_ref_count | notes |
|---|---|---|---|---|---|---:|---|
| `Burn` | `Burn` | `status/Burn.java` | `approximate` | unplayable, end_of_turn_take_damage | none | 33 | effect handlers resolved; behavior parity audit still required |
| `Dazed` | `Dazed` | `status/Dazed.java` | `approximate` | unplayable | none | 13 | effect handlers resolved; behavior parity audit still required |
| `Slimed` | `Slimed` | `status/Slimed.java` | `approximate` | n/a | none | 23 | effect handlers resolved; behavior parity audit still required |
| `Void` | `VoidCard` | `status/VoidCard.java` | `approximate` | unplayable, lose_1_energy_when_drawn | none | 22 | effect handlers resolved; behavior parity audit still required |
| `Wound` | `Wound` | `status/Wound.java` | `approximate` | unplayable | none | 21 | effect handlers resolved; behavior parity audit still required |
