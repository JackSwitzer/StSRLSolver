# Game Loop Audit: Python Engine vs Decompiled Java

## Overview

Audit of `packages/engine/game.py` (GameRunner), `packages/engine/combat_engine.py` (CombatEngine), `packages/engine/calc/combat_sim.py` (CombatSimulator), and `packages/engine/handlers/rooms.py` against decompiled Java sources in `decompiled/java-src/com/megacrit/cardcrawl/`.

## Phase Transitions

### Java (AbstractRoom.java)
- `RoomPhase.COMBAT` -> player actions -> `endTurn()` -> enemy turn -> check `isBattleOver`
- `isBattleOver` + action queue empty -> `RoomPhase.COMPLETE`
- In COMPLETE: combat reward screen opens, then proceed button shows
- Boss room COMPLETE: no rewards screen if TheBeyond/TheEnding (goes to act transition)

### Python (game.py GameRunner)
- `GamePhase.COMBAT` -> player actions via CombatEngine -> `_end_combat(victory)` -> `GamePhase.COMBAT_REWARDS`
- `COMBAT_REWARDS` -> player picks rewards -> proceed -> `GamePhase.MAP_NAVIGATION`
- Boss: `COMBAT_REWARDS` -> proceed -> `GamePhase.BOSS_REWARDS` -> pick boss relic -> advance act -> `MAP_NAVIGATION`

**Findings:**
1. CORRECT: Combat -> Rewards -> Map flow matches Java lifecycle
2. CORRECT: Boss has intermediate COMBAT_REWARDS before BOSS_REWARDS
3. MINOR ISSUE: Python skips the "COMPLETE" phase that Java uses for animation timing. Not relevant for simulation.

## Combat Flow

### Java (AbstractRoom.java lines 212-351)
1. `onPlayerEntry()` sets `waitTimer = 0.1f`
2. During COMBAT phase with waitTimer > 0: processes action queue, then on timer expiry:
   - `GainEnergyAndEnableControlsAction(energyMaster)`
   - `applyStartOfCombatPreDrawLogic()`
   - `DrawCardAction(gameHandSize)` (default 5)
   - `EnableEndTurnButtonAction()`
   - `applyStartOfCombatLogic()`, `applyStartOfTurnRelics()`, `applyStartOfTurnPostDrawRelics()`, `applyStartOfTurnCards()`, `applyStartOfTurnPowers()`, `applyStartOfTurnOrbs()`
3. `endTurn()`:
   - `applyEndOfTurnTriggers()`
   - `ClearCardQueueAction`
   - `DiscardAtEndOfTurnAction`
   - Reset card attributes
   - `EndTurnAction` -> `WaitAction(1.2f)` -> `MonsterStartTurnAction`
4. `endBattle()`:
   - Triggers "Meat on the Bone" relic
   - `player.onVictory()`
   - Gold rewards by room type:
     - Boss: 100 +/- 5 (A13+: * 0.75)
     - Elite: treasureRng.random(25, 35)
     - Monster: treasureRng.random(10, 20)
   - `dropReward()` (relic for elites)
   - `addPotionToRewards()` (40% base + blizzard modifier)
   - Card rewards via `RewardItem` constructor

### Python (combat_engine.py + game.py)
1. `_enter_combat()`: creates CombatEngine, calls `start_combat()`
2. Combat actions: `PlayCard`, `UsePotion`, `EndTurn` dispatched via CombatEngine
3. `_end_combat()`: generates rewards via RewardHandler, triggers post-combat relics

**Findings:**
1. CORRECT: Turn start order (energy -> draw -> relics/powers) matches Java
2. CORRECT: End turn order (discard -> enemy turn -> start next turn) matches
3. BUG: `_end_combat` triggers Meat on the Bone, Blood Vial, Burning Blood, Black Blood correctly
4. GOLD RANGE DISCREPANCY: Python uses `RewardHandler.generate_combat_rewards()` which delegates to `generate_gold_reward()` -- need to verify exact ranges match Java (Monster: 10-20, Elite: 25-35, Boss: 95-105 or A13+ 71-79)
5. MISSING: Java's `endBattle()` calls `player.onVictory()` which triggers relic effects. Python handles some but may miss others.

## Rest Site Options

### Java (CampfireUI.java lines 78-103)
Initialization order:
1. `RestOption(true)` -- always added, usable check via relics
2. `SmithOption(canUpgrade && !Midas)` -- upgrade
3. Relic-added options via `r.addCampfireOption(buttons)`:
   - Shovel -> DigOption
   - Girya -> LiftOption
   - Peace Pipe -> TokeOption
4. Relic `canUseCampfireOption` check (e.g., Coffee Dripper disables Rest, Fusion Hammer disables Smith)
5. RecallOption if `isFinalActAvailable && !hasRubyKey`

### Python (rooms.py RestHandler.get_options + game.py _get_rest_actions)
- Rest: available unless Coffee Dripper, and HP < max
- Smith: available unless Fusion Hammer, and has upgradeable cards
- Dig: requires Shovel
- Lift: requires Girya, counter < 3
- Toke: requires Peace Pipe (in RestHandler but NOT in GameRunner._get_rest_actions)
- Recall/Ruby Key: Act 3, key not obtained

**Findings:**
1. BUG (MINOR): `_get_rest_actions` does NOT include "toke" (Peace Pipe card removal). The RestHandler has the logic, but the GameRunner does not wire it up.
2. BUG (MINOR): `_get_rest_actions` does not check Coffee Dripper. It always offers "rest". The RestHandler.get_options does check correctly, but GameRunner uses its own logic.
3. BUG: Java checks `isFinalActAvailable` for Recall, Python checks `act == 3`. These are different -- Java's flag depends on whether keys are being tracked. In practice equivalent for standard runs.
4. CORRECT: Girya counter max 3 matches Java.
5. MISSING: `Eternal Feather` on-enter healing. GameRunner._enter_rest does not call `RestHandler.on_enter_rest_site()`.
6. MISSING: `Dream Catcher` card reward after resting. RestHandler supports it but GameRunner doesn't trigger it.

## Shop Flow

### Java (ShopRoom.java)
- `onPlayerEntry()`: creates Merchant, sets proceed button
- Phase starts at `COMPLETE` (not COMBAT) -- shop is non-combat
- Card rarity: `baseRareCardChance = 9`, `baseUncommonCardChance = 37` (higher rare chance than normal rooms)
- Purge: via grid select screen, removes from masterDeck

### Python (game.py + shop_handler.py)
- `_enter_shop()`: creates shop via ShopHandler, phase = SHOP
- Meal Ticket healing on shop entry (correct)
- Shop inventory generated via ShopHandler.create_shop
- Card removal tracks purge_count (correct)

**Findings:**
1. CORRECT: Shop generation and purchase flow works
2. CORRECT: Meal Ticket trigger on entry
3. NOTE: Java shop has `shopRarityBonus = 6` and different card rarity distribution. Python may not replicate this exactly.

## Treasure Rooms

### Java (TreasureRoom.java)
- `onPlayerEntry()`: `AbstractDungeon.getRandomChest()` generates chest
- Phase starts at COMPLETE (non-combat)
- Chest opening handled by AbstractChest subclasses

### Python (game.py + rooms.py TreasureHandler)
- `_enter_treasure()`: sets phase to TREASURE
- Chest type rolled via TreasureHandler.determine_chest_type
- Relic tier based on chest type
- Cursed Key, Matryoshka, Sapphire Key all handled

**Findings:**
1. CORRECT: Chest type probabilities match (Small < 50, Medium < 83, Large otherwise)
2. CORRECT: Relic tier by chest type matches Java
3. CORRECT: Sapphire Key logic (Act 3, skip relic)

## Potion Drop Mechanics

### Java (AbstractRoom.java lines 582-609)
- Base chance: 40% for Monster/Elite/Event rooms
- `blizzardPotionMod` adjusts: +10 on miss, -10 on drop
- `White Beast Statue`: 100% chance
- Cap: if rewards.size() >= 4, chance = 0
- Uses `potionRng.random(0, 99)`

### Python (rooms.py RewardHandler.generate_combat_rewards)
- Delegates to `check_potion_drop()` in rewards.py
- Tracks blizzard modifier via run_state

**Finding:** Need to verify the 4-reward cap is implemented in Python.

## Combat Simulator (combat_sim.py)

### Coverage Status
- ~9% coverage, essentially untested
- Provides immutable-state combat simulation for tree search
- Independent from CombatEngine (which is the mutable real-time combat system)

### Key Issues Found
1. `_draw_cards` uses a deterministic but non-standard shuffle (based on rng_state tuple + turn number modulo), which differs from Java's Fisher-Yates with RNG stream
2. `_roll_enemy_move` uses simplified alternating attack/buff pattern instead of real enemy AI
3. `simulate_enemy_turn` applies strength to base_damage correctly
4. Stance mechanics (Wrath/Calm/Divinity) energy and damage multipliers correct
5. `_change_stance` triggers Mental Fortress, Rushdown, Flurry of Blows -- all correct

## Summary of Issues

| # | Severity | Component | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | GameRunner._get_rest_actions | Missing Toke (Peace Pipe) option |
| 2 | MEDIUM | GameRunner._get_rest_actions | Missing Coffee Dripper check for rest |
| 3 | LOW | GameRunner._enter_rest | Missing Eternal Feather on-enter healing |
| 4 | LOW | GameRunner._handle_rest_action | Missing Dream Catcher card reward after rest |
| 5 | LOW | GameRunner._enter_rest | Should call RestHandler.on_enter_rest_site() |
| 6 | LOW | CombatSimulator._draw_cards | Non-standard shuffle on reshuffle |
| 7 | LOW | CombatSimulator._roll_enemy_move | Placeholder enemy AI, not real |
| 8 | INFO | GameRunner | Boss gold: verify A13+ 0.75x multiplier applied correctly |
| 9 | INFO | Potion drop | Verify 4-reward cap in Python |
