# Ultra-Granular Work Units: Relics

## Model-facing actions (no UI)
- [ ] On-acquire relics that require choice should emit explicit action lists. (action: select_cards{pile:deck,card_indices})
- [ ] Relic-driven choices should be resolvable via action parameters (no UI). (action: select_cards{pile:deck,card_indices})

## Action tags
Use explicit signatures on each item (see `granular-actions.md`).

## Watcher-critical energy/reward relics
- [ ] Violet Lotus - gain +1 energy when exiting Calm. (action: none{})
- [x] Singing Bowl - skipping card reward grants +2 max HP. (action: singing_bowl{})
- [x] Question Card - +1 card choice per card reward. (action: none{})
- [x] Prayer Wheel - extra card reward. (action: none{})
- [x] Busted Crown - reduce card reward choices by 2. (action: none{})

## Potion/reward modifiers
- [ ] White Beast Statue - guaranteed potion reward. (action: none{})
- [ ] Toy Ornithopter - heal on potion use. (action: none{})
- [x] Sozu - block potion rewards. (action: none{})

## Combat trigger relics
- [ ] Snecko Eye - randomized hand costs each turn. (action: none{})
- [ ] Ice Cream - unused energy carries over. (action: none{})
- [ ] Incense Burner - Intangible every 6 turns (verify counter timing). (action: none{})
- [ ] Pen Nib - double damage every 10th attack. (action: none{})
- [x] Preserved Insect - elite combat start applies 25% HP reduction to elite enemies. (action: none{})
- [x] Combat context includes stable `combat_type` so elite/boss-conditional relic hooks (`Sling`, `Pantograph`, `Slaver's Collar`) do not crash. (action: none{})

## Chest counters / on-open triggers
- [x] Tiny Chest - increment counter on room entry, trigger chest at 4, reset timing. (action: none{})
- [x] Matryoshka - on chest open, grant extra relic while counter > 0. (action: none{})
- [x] Cursed Key - on chest open, add a curse when a chest relic is actually taken. (action: none{})
- [x] N'loth's Hungry Face - remove one relic reward from the next non-boss chest, then disable. (action: none{})

## On-acquire / transform relics
- [x] Astrolabe boss-relic pick path uses explicit `select_cards` action flow (no hidden auto-pick in action API). (action: pick_boss_relic{relic_index} + select_cards{pile:deck,card_indices})
- [x] Empty Cage boss-relic pick path uses explicit `select_cards` action flow (no hidden auto-pick in action API). (action: pick_boss_relic{relic_index} + select_cards{pile:deck,card_indices})
- [x] Astrolabe - transform and upgrade 3 cards. (action: select_cards{pile:deck,card_indices})
- [x] Empty Cage - remove 2 cards. (action: select_cards{pile:deck,card_indices})
- [x] Pandora's Box - transform all Strikes/Defends. (action: none{})
- [x] Calling Bell - obtain 3 relics + curse. (action: none{})
- [x] Tiny House - multi-reward bundle. (action: none{})
- [x] Toolbox - register as SHOP relic and expose Java-consistent ID/effect metadata. (action: none{})
- [x] Relic alias resolver maps Java/class-name forms to canonical content IDs (`Abacus/Courier/Waffle/Wing Boots/...`). (action: none{})

## On-obtain card modifiers (Egg relic family)
- [x] Frozen Egg 2 - cards obtained as Powers are automatically upgraded. (action: none{})
- [x] Molten Egg 2 - cards obtained as Attacks are automatically upgraded. (action: none{})
- [x] Toxic Egg 2 - cards obtained as Skills are automatically upgraded. (action: none{})
- [x] Centralized egg upgrade policy in `RunState.add_card` so reward/shop/event paths inherit behavior. (action: none{})

## Tests
- [ ] Add tests for Violet Lotus stance exit energy. (action: none{})
- [ ] Add tests for reward modifiers (Question Card, Prayer Wheel, Busted Crown). (action: none{})
- [ ] Add tests for on-acquire relic transformations. (action: none{})
- [x] Add integration tests for Egg relic upgrades across generated cards and acquisition paths. (action: none{})
- [x] Verify Egg relics upgrade matching card types even for off-class generated cards (e.g., Toxic Egg + non-Watcher skills). (action: none{})

## Failed-tests mapping (2026-02-04, Sacred Bark)
- [x] Normalize Sacred Bark relic ID (accept `Sacred Bark` and `SacredBark` or unify). (action: none{})
- [ ] Ensure Sacred Bark doubles potion potency across all potion effects (see `granular-potions.md` list). (action: none{})

## Skipped-test mapping (pickup / on-acquire)
- [x] War Paint upgrades 2 skills (only skills, fewer-than-2 handling, no double-upgrade, use miscRng). (action: none{})
- [x] Whetstone upgrades 2 attacks (only attacks, fewer-than-2 handling, use miscRng). (action: none{})
- [x] Astrolabe transforms 3 cards, upgrades transformed cards, and cannot transform basic cards. (action: select_cards{pile:deck,card_indices})
- [x] Calling Bell grants 3 relics, grants a curse, and respects relic tier rolls. (action: none{})
- [x] Empty Cage removes 2 cards with explicit choice. (action: select_cards{pile:deck,card_indices})
- [x] Tiny House grants 50 gold, +5 max HP, potion, card, and upgrades 1 card. (action: none{})
- [x] Cauldron grants 5 potions and handles full slots. (action: none{})
- [x] Dolly's Mirror duplicates a card and preserves upgrades. (action: select_cards{pile:deck,card_indices})
- [x] Lee's Waffle grants 7 max HP and heals to full. (action: none{})
- [x] Orrery adds 5 cards and allows choice. (action: pick_card{card_reward_index,card_index})
- [x] Old Coin grants 300 gold and respects Ectoplasm interaction. (action: none{})
- [x] Pandora's Box transforms strikes/defends, preserves deck size, and excludes starter card protections. (action: none{})

## Skipped-test mapping (rest-site)
- [x] Dream Catcher triggers on rest (not smith), optional skip, and respects Coffee Dripper. (action: rest{})
- [x] Regal Pillow adds 15 HP, is NOT affected by Magic Flower, caps at max HP, and does not affect smith. (action: rest{})
- [x] Girya adds lift option, grants strength per lift, caps at 3 uses, and persists across combats. (action: lift{})
- [x] Peace Pipe adds toke option, removes a card, allows unlimited uses, and coexists with other options. (action: toke{card_index})
- [x] Shovel adds dig option and grants a relic when used. (action: dig{})
- [x] Rest-site relic combinations (Coffee Dripper, Fusion Hammer, Mark of Bloom, dual blockers). (action: rest{})

## Skipped-test mapping (bottled relics)
- [x] Bottled Flame selection (attacks only), innate start-in-hand, no-attacks edge case, save/load preservation. (action: select_cards{pile:deck,card_indices})
- [x] Bottled Lightning selection (skills only), innate start-in-hand, no-skills edge case. (action: select_cards{pile:deck,card_indices})
- [x] Bottled Tornado selection (powers only), innate start-in-hand, no-powers edge case. (action: select_cards{pile:deck,card_indices})
- [x] Multiple bottled relics interactions, combat start ordering, hand size limits. (action: select_cards{pile:deck,card_indices})
- [x] Bottled relics with upgraded cards, duplicates, removed cards, and transformed cards. (action: select_cards{pile:deck,card_indices})

## Skipped-test mapping (out-of-combat triggers)
- [x] Maw Bank gains 12 on room entry and deactivates after spending gold. (action: none{})
- [x] N'loth's Hungry Face chest-removal behavior. (action: none{})
- [x] Ectoplasm blocks gold gain and adjusts energy on equip/unequip; track blocked amount. (action: none{})
- [x] Fallback post-combat relic logic does not duplicate combat-engine `onVictory` effects; Meat on the Bone uses Java `<= 50%` threshold; Blood Vial is not applied post-combat. (action: none{})

## Skipped-test mapping (utilities / coverage)
- [x] `RunState.get_starter_relic` helper for coverage tests. (action: none{})

## Keep skipped (non-unit-testable or invalid expectation)
- [ ] White Beast Statue potion heal test (heal is Toy Ornithopter behavior). (action: none{})
