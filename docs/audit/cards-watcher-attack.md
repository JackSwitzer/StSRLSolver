# Watcher Attack Card Audit: Python vs Decompiled Java

Audit date: 2026-02-02

## Summary

17 Watcher attack cards audited. **All base values match Java.** Two documentation-only issues found. One code style concern (hardcoded values in effects instead of reading magic_number).

## Card-by-Card Comparison

| Card | Field | Java | Python | Match |
|------|-------|------|--------|-------|
| **BowlingBash** | baseDamage | 7 | 7 | YES |
| | upgradeDamage | +3 | +3 | YES |
| | cost | 1 | 1 | YES |
| | effect | Hit target once per living enemy | damage_per_enemy | YES |
| **EmptyFist** | baseDamage | 9 | 9 | YES |
| | upgradeDamage | +5 | +5 | YES |
| | cost | 1 | 1 | YES |
| | effect | Exit stance (Neutral) | exit_stance=True | YES |
| **FlurryOfBlows** | baseDamage | 4 | 4 | YES |
| | upgradeDamage | +2 | +2 | YES |
| | cost | 0 | 0 | YES |
| | effect | Play from discard on stance change | on_stance_change_play_from_discard | YES |
| **FlyingSleeves** | baseDamage | 4 | 4 | YES |
| | upgradeDamage | +2 | +2 | YES |
| | cost | 1 | 1 | YES |
| | retain | selfRetain=true | retain=True | YES |
| | effect | Hit twice (2 DamageActions) | damage_twice | YES |
| **FollowUp** | baseDamage | 7 | 7 | YES |
| | upgradeDamage | +4 | +4 | YES |
| | cost | 1 | 1 | YES |
| | effect | +1 energy if last card Attack | if_last_card_attack_gain_energy | YES |
| **JustLucky** | baseDamage | 3 | 3 | YES |
| | upgradeDamage | +1 | +1 | YES |
| | baseBlock | 2 | 2 | YES |
| | upgradeBlock | +1 | +1 | YES |
| | baseMagicNumber (scry) | 1 | 1 | YES |
| | upgradeMagicNumber | +1 | +1 | YES |
| | effect order | Scry, Block, Damage | scry, gain_block (damage implicit) | YES |
| **SashWhip** | baseDamage | 8 | 8 | YES |
| | upgradeDamage | +2 | +2 | YES |
| | baseMagicNumber (weak) | 1 | 1 | YES |
| | upgradeMagicNumber | +1 | +1 | YES |
| | cost | 1 | 1 | YES |
| | effect | Apply weak if last card Attack | if_last_card_attack_weak | YES |
| **CrushJoints** | baseDamage | 8 | 8 | YES |
| | upgradeDamage | +2 | +2 | YES |
| | baseMagicNumber (vuln) | 1 | 1 | YES |
| | upgradeMagicNumber | +1 | +1 | YES |
| | cost | 1 | 1 | YES |
| | effect | Apply vuln if last card Skill | if_last_card_skill_vulnerable | YES |
| **Tantrum** | baseDamage | 3 | 3 | YES |
| | upgradeDamage | none | none | YES |
| | baseMagicNumber (hits) | 3 | 3 | YES |
| | upgradeMagicNumber | +1 | +1 | YES |
| | cost | 1 | 1 | YES |
| | shuffleBack | true | shuffle_back=True | YES |
| | enter stance | Wrath | enter_stance="Wrath" | YES |
| **FearNoEvil** | baseDamage | 8 | 8 | YES |
| | upgradeDamage | +3 | +3 | YES |
| | cost | 1 | 1 | YES |
| | effect | Enter Calm if enemy attacking | if_enemy_attacking_enter_calm | YES |
| **ReachHeaven** | baseDamage | 10 | 10 | YES |
| | upgradeDamage | +5 | +5 | YES |
| | cost | 2 | 2 | YES |
| | effect | Add Through Violence to draw | add_through_violence_to_draw | YES |
| **SignatureMove** | baseDamage | 30 | 30 | YES |
| | upgradeDamage | +10 | +10 | YES |
| | cost | 2 | 2 | YES |
| | effect | Only playable if only Attack in hand | only_attack_in_hand | YES |
| **Wallop** | baseDamage | 9 | 9 | YES |
| | upgradeDamage | +3 | +3 | YES |
| | cost | 2 | 2 | YES |
| | effect | Gain block = unblocked damage | gain_block_equal_unblocked_damage | YES |
| **WheelKick** | baseDamage | 15 | 15 | YES |
| | upgradeDamage | +5 | +5 | YES |
| | cost | 2 | 2 | YES |
| | effect | Draw 2 | draw_2 | YES |
| **WindmillStrike** | baseDamage | 7 | 7 | YES |
| | upgradeDamage | +3 | +3 | YES |
| | baseMagicNumber (retain bonus) | 4 | 4 (via effect name) | YES |
| | upgradeMagicNumber | +1 | +1 | YES |
| | cost | 2 | 2 | YES |
| | retain | selfRetain=true | retain=True | YES |
| | STRIKE tag | yes | not tracked | NOTE |
| **Conclude** | baseDamage | 12 | 12 | YES |
| | upgradeDamage | +4 | +4 | YES |
| | cost | 1 | 1 | YES |
| | target | ALL_ENEMY | ALL_ENEMY | YES |
| | effect | End turn | end_turn | YES |
| **Ragnarok** | baseDamage | 5 | 5 | YES |
| | upgradeDamage | +1 | +1 | YES |
| | baseMagicNumber (hits) | 5 | 5 | YES |
| | upgradeMagicNumber | +1 | +1 | YES |
| | cost | 3 | 3 | YES |
| | target | ALL_ENEMY (random) | ALL_ENEMY | YES |
| | effect | Hit random enemy X times | damage_random_x_times | YES |

## Issues Found

### Documentation-Only

1. **Wallop docstring** in `effects/cards.py` line 41 says "9/14 damage" but correct upgraded value is 9+3=12. Should be "9/12".

2. **Conclude+ separate definition** at line 566-570 of `cards.py` defines `CONCLUDE_PLUS` as a separate Card object with `base_damage=16` and `id="Conclude+"`. Java has no separate Conclude+ card -- it is just Conclude with upgradeDamage(+4). The Python engine presumably handles this for upgraded card lookup. Not a bug but unusual.

### Code Style

3. **SashWhip/CrushJoints effects hardcode amounts** instead of reading `ctx.magic_number`. The `if_last_card_attack_weak` effect uses `amount = 2 if ctx.is_upgraded else 1` rather than `ctx.card.magic_number`. Functionally equivalent but fragile if card values change.

### Missing Tags

4. **WindmillStrike STRIKE tag** is present in Java (`tags.add(CardTags.STRIKE)`) but not tracked in Python. This matters for Perfected Strike interactions (Ironclad only, so low priority for Watcher).

## Files Examined

- Python cards: `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/content/cards.py`
- Python effects: `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/effects/cards.py`
- Java sources: `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/*.java`
