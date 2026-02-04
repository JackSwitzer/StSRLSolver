# Ultra-Granular Work Units: Relics

## Model-facing actions (no UI)
- [ ] On-acquire relics that require choice should emit explicit action lists. (action: select_cards{pile:deck,card_indices})
- [ ] Relic-driven choices should be resolvable via action parameters (no UI). (action: select_cards{pile:deck,card_indices})

## Action tags
Use explicit signatures on each item (see `granular-actions.md`).

## Watcher-critical energy/reward relics
- [ ] Violet Lotus - gain +1 energy when exiting Calm. (action: none{})
- [ ] Singing Bowl - skipping card reward grants +2 max HP. (action: singing_bowl{})
- [ ] Question Card - +1 card choice per card reward. (action: none{})
- [ ] Prayer Wheel - extra card reward. (action: none{})
- [ ] Busted Crown - reduce card reward choices by 2. (action: none{})

## Potion/reward modifiers
- [ ] White Beast Statue - guaranteed potion reward. (action: none{})
- [ ] Toy Ornithopter - heal on potion use. (action: none{})
- [ ] Sozu - block potion rewards. (action: none{})

## Combat trigger relics
- [ ] Snecko Eye - randomized hand costs each turn. (action: none{})
- [ ] Ice Cream - unused energy carries over. (action: none{})
- [ ] Incense Burner - Intangible every 6 turns (verify counter timing). (action: none{})
- [ ] Pen Nib - double damage every 10th attack. (action: none{})

## On-acquire / transform relics
- [ ] Astrolabe - transform and upgrade 3 cards. (action: select_cards{pile:deck,card_indices})
- [ ] Empty Cage - remove 2 cards. (action: select_cards{pile:deck,card_indices})
- [ ] Pandora's Box - transform all Strikes/Defends. (action: none{})
- [ ] Calling Bell - obtain 3 relics + curse. (action: none{})
- [ ] Tiny House - multi-reward bundle. (action: none{})

## Tests
- [ ] Add tests for Violet Lotus stance exit energy. (action: none{})
- [ ] Add tests for reward modifiers (Question Card, Prayer Wheel, Busted Crown). (action: none{})
- [ ] Add tests for on-acquire relic transformations. (action: none{})

## Failed-tests mapping (2026-02-04, Sacred Bark)
- [ ] Normalize Sacred Bark relic ID (accept `Sacred Bark` and `SacredBark` or unify). (action: none{})
- [ ] Ensure Sacred Bark doubles potion potency across all potion effects (see `granular-potions.md` list). (action: none{})

## Skipped-test mapping (pickup / on-acquire)
- [ ] War Paint upgrades 2 skills (only skills, fewer-than-2 handling, no double-upgrade). (action: none{})
- [ ] Whetstone upgrades 2 attacks (only attacks, fewer-than-2 handling). (action: none{})
- [ ] Astrolabe transforms 3 cards, upgrades transformed cards, and cannot transform basic cards. (action: select_cards{pile:deck,card_indices})
- [ ] Calling Bell grants 3 relics, grants a curse, and respects relic tier rolls. (action: none{})
- [ ] Empty Cage removes 2 cards with explicit choice. (action: select_cards{pile:deck,card_indices})
- [ ] Tiny House grants 50 gold, +5 max HP, potion, card, and upgrades 1 card. (action: none{})
- [ ] Cauldron grants 5 potions and handles full slots. (action: none{})
- [ ] Dolly's Mirror duplicates a card and preserves upgrades. (action: select_cards{pile:deck,card_indices})
- [ ] Lee's Waffle grants 7 max HP and heals to full. (action: none{})
- [ ] Orrery adds 5 cards and allows choice. (action: pick_card{card_reward_index,card_index})
- [ ] Old Coin grants 300 gold and respects Ectoplasm interaction. (action: none{})
- [ ] Pandora's Box transforms strikes/defends, preserves deck size, and excludes starter card protections. (action: none{})

## Skipped-test mapping (rest-site)
- [ ] Dream Catcher triggers on rest (not smith), optional skip, and respects Coffee Dripper. (action: rest{})
- [ ] Regal Pillow adds 15 HP, is affected by Magic Flower, capped at max HP, and does not affect smith. (action: rest{})
- [ ] Girya adds lift option, grants strength per lift, caps at 3 uses, and persists across combats. (action: lift{})
- [ ] Peace Pipe adds toke option, removes a card, allows unlimited uses, and coexists with other options. (action: toke{card_index})
- [ ] Shovel adds dig option, grants relic, is one-time per rest, and replaces rest or smith. (action: dig{})
- [ ] Golden Eye scry on rest only, applies to next combat, Watcher exclusive. (action: rest{})
- [ ] Melange scry on rest and applies to next combat. (action: rest{})
- [ ] Rest-site relic combinations (Coffee Dripper, Fusion Hammer, Mark of Bloom, dual blockers). (action: rest{})

## Skipped-test mapping (bottled relics)
- [ ] Bottled Flame selection (attacks only), innate start-in-hand, no-attacks edge case, save/load preservation. (action: select_cards{pile:deck,card_indices})
- [ ] Bottled Lightning selection (skills only), innate start-in-hand, no-skills edge case. (action: select_cards{pile:deck,card_indices})
- [ ] Bottled Tornado selection (powers only), innate start-in-hand, no-powers edge case. (action: select_cards{pile:deck,card_indices})
- [ ] Multiple bottled relics interactions, combat start ordering, hand size limits. (action: select_cards{pile:deck,card_indices})
- [ ] Bottled relics with upgraded cards, duplicates, removed cards, and transformed cards. (action: select_cards{pile:deck,card_indices})

## Skipped-test mapping (out-of-combat triggers)
- [ ] Maw Bank deactivation on shop purchase. (action: none{})
- [ ] N'loth event relic logic. (action: none{})
- [ ] Chemical X shop price adjustment. (action: none{})
- [ ] Ectoplasm blocked-amount tracking. (action: none{})

## Skipped-test mapping (utilities / coverage)
- [ ] `RunState.get_starter_relic` helper for coverage tests. (action: none{})

## Keep skipped (non-unit-testable or invalid expectation)
- [ ] White Beast Statue potion heal test (heal is Toy Ornithopter behavior). (action: none{})
