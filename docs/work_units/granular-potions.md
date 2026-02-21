# Ultra-Granular Work Units: Potions

## Model-facing actions (no UI)
- [x] Use `use_potion{potion_slot,...}` actions with required params. (action: use_potion{potion_slot})
- [x] If params are omitted, return explicit candidate action list. (action: use_potion{potion_slot} + select_cards{pile:offer,card_indices})

## Action tags
Use explicit signatures on each item (see `granular-actions.md`).

## Checklist
- [x] Discovery potions: implement choose-1-of-3 with cardRng (Attack/Skill/Power/Colorless). (action: use_potion{potion_slot} + select_cards{pile:offer,card_indices})
- [x] Discovery potions: Sacred Bark adds 2 copies of the chosen card (not 2 random cards), cost 0 for turn. (action: use_potion{potion_slot} + select_cards{pile:offer,card_indices})
- [x] Discovery potions: ensure unique options and deterministic RNG advancement. (action: use_potion{potion_slot} + select_cards{pile:offer,card_indices})
- [x] Distilled Chaos: play top N cards (3/6) for free, not draw. (action: use_potion{potion_slot})
- [x] Distilled Chaos: handle targeting (random valid enemy) and on-play triggers. (action: use_potion{potion_slot})
- [x] Liquid Memories: select card(s) from discard, move to hand, cost 0 this turn. (action: use_potion{potion_slot} + select_cards{pile:discard,card_indices})
- [x] Liquid Memories: handle empty discard, full hand, Sacred Bark (2 cards), and always use `ctx.potency` (no special-casing). (action: use_potion{potion_slot} + select_cards{pile:discard,card_indices})
- [x] Entropic Brew: fill empty slots using potionRng and class pool parity. (action: use_potion{potion_slot})
- [x] Entropic Brew: enforce Sozu, use actual empty slot count (not potency), handle out-of-combat use. (action: use_potion{potion_slot})
- [ ] Fairy Potion: verify auto-trigger conditions, heal percent, consumption rules. (action: none{}; auto-trigger)
- [x] Gambler's Brew: choose discard set, draw same count. (action: use_potion{potion_slot} + select_cards{pile:hand,card_indices})
- [x] Elixir: choose cards to exhaust (not all by default). (action: use_potion{potion_slot} + select_cards{pile:hand,card_indices})
- [x] Stance Potion: choose stance (Calm/Wrath) and handle stance change triggers (no auto-toggle). (action: use_potion{potion_slot} + select_stance{stance})
- [x] Snecko Oil: randomize hand costs via cardRandomRng (cost for turn only, deterministic). (action: use_potion{potion_slot})
- [x] Smoke Bomb: disallow on bosses/BackAttack; apply escape and suppress rewards. (action: use_potion{potion_slot})
- [x] Potion targeting: use `PotionTargetType` for action generation and targeting. (action: use_potion{potion_slot,target_index})
- [x] Relic hooks: ensure onUsePotion relic triggers fire in all potion paths. (action: use_potion{potion_slot})
- [x] Tests: add focused tests for each potion behavior above + RNG parity. (action: none{})

## Model-traversable actions (no UI)
- [x] Potions that require selection should expose explicit actions if parameters aren’t provided. (action: use_potion{potion_slot} + select_cards{pile:offer,card_indices})
- [x] Liquid Memories: support `use_potion{potion_slot,card_indices}` with 1 card (2 with Sacred Bark). If not provided, return candidate actions for discard selection. (action: use_potion{potion_slot,card_indices})
- [x] Discovery potions: return 3 card choices as actions; selection uses `select_cards{pile:offer,card_indices}`. (action: select_cards{pile:offer,card_indices})

## Failed-tests mapping (2026-02-04, Sacred Bark)
- [x] Block Potion doubles block (12 → 24). (action: use_potion{potion_slot})
- [x] Strength Potion doubles Strength (2 → 4). (action: use_potion{potion_slot})
- [x] Fire Potion doubles damage (20 → 40). (action: use_potion{potion_slot})
- [x] Energy Potion doubles energy gain (2 → 4). (action: use_potion{potion_slot})
- [x] Swift Potion doubles draws (3 → 6). (action: use_potion{potion_slot})
- [x] Fairy Potion revive at 60% max HP with Sacred Bark. (action: none{}; auto-trigger)
- [x] Regen Potion doubles Regeneration (5 → 10). (action: use_potion{potion_slot})
- [x] Ancient Potion doubles Artifact (1 → 2). (action: use_potion{potion_slot})
- [x] Essence of Steel doubles Plated Armor (4 → 8). (action: use_potion{potion_slot})
- [x] Fruit Juice doubles max HP gain (5 → 10) and heals to new max. (action: use_potion{potion_slot})
- [x] Duplication Potion doubles stacks (1 → 2). (action: use_potion{potion_slot})
