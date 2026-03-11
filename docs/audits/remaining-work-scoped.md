# Remaining Work — Scoped Units for Subagents

**Date:** 2026-03-11
**Baseline:** 5825 tests passing, 155 power triggers, 169 relic triggers

## Unit 1: Fix 5 Event Handlers (AFFECTS_RL)
**Effort:** ~30 min, 1 agent
**File:** `packages/engine/handlers/event_handler.py`
**Java ref:** `decompiled/java-src/com/megacrit/cardcrawl/events/`

1. **Augmenter/DrugDealer**: Option 0 should NOT remove a card (just give J.A.X.). Option 2 should always give MutagenicStrength (not random Str/Dex).
2. **Joust**: Add 50g bet cost to both options. Fix murderer reward: 100g not 50g.
3. **Library Sleep**: Change heal from full HP to `ceil(max_hp * 0.33)` (or `ceil(max_hp * 0.20)` on A15+).
4. **Tomb of Red Mask**: If has Red Mask: option 0 = gain 222 gold. If not: option 0 = pay all gold, get Red Mask.
5. **Face Trader**: Option 0 = take damage (maxHP/10) + gain gold (75/50 A15+). Option 1 = trade for random face relic (CultistMask/FaceOfCleric/GremlinMask/NlothsMask/SsserpentHead), no gold cost.

Read Java source for each. Add 1 test per event.

## Unit 2: Blue Candle + Medical Kit Relics (AFFECTS_RL)
**Effort:** ~15 min, 1 agent (haiku-capable)
**Files:** `packages/engine/registry/relics.py`, `packages/engine/combat_engine.py`

1. **Blue Candle**: Add `onUseCard` relic trigger — if card is Curse type, exhaust it and deal 1 HP to player. Also need to allow Curse cards to be played (check `_can_play_card`).
2. **Medical Kit**: Add `onUseCard` relic trigger — if card is Status type, exhaust it. Also need to allow Status cards to be played.

Read Java: `BlueCandle.java`, `MedicalKit.java`

## Unit 3: Boss Relic onEquip Effects (AFFECTS_RL)
**Effort:** ~45 min, 1 agent
**Files:** `packages/engine/game.py` (relic acquisition), `packages/engine/state/run.py`
**Java ref:** `decompiled/java-src/com/megacrit/cardcrawl/relics/`

For each boss relic, implement the onEquip effect in the relic acquisition flow:
1. **Astrolabe**: Transform 3 cards + upgrade results (auto-select for RL: pick 3 worst cards)
2. **Calling Bell**: Generate 1 common + 1 uncommon + 1 rare relic + add random curse
3. **Empty Cage**: Remove 2 cards (auto-select: remove worst 2)
4. **Pandora's Box**: Transform all Strike_P and Defend_P cards
5. **Tiny House**: +50 gold, +5 max HP, random potion, random card, upgrade random card

For RL, auto-selection is fine (no interactive card picking needed).

## Unit 4: PlatedArmor wasHPLost (BLOCKS_RL edge case)
**Effort:** ~5 min, direct fix
**File:** `packages/engine/registry/powers.py`

Add `@power_trigger("wasHPLost", power="Plated Armor")` that decrements Plated Armor by 1 when player loses HP (matching Java `PlatedArmorPower.wasHPLost`).

## Unit 5: Verify & Update Audit Summary
**Effort:** ~10 min, haiku-capable
**File:** `docs/audits/final-gap-checklist.md`

Mark all completed items with [x]. Update summary counts. Add "Session Complete" section.

## Priority Order
1. Unit 4 (direct, 5 min)
2. Unit 1 (events, biggest remaining impact)
3. Unit 2 (Blue Candle/Medical Kit)
4. Unit 3 (boss relics)
5. Unit 5 (audit cleanup)
