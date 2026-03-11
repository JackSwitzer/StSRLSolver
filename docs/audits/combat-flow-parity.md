# Combat Flow Parity Audit: Java vs Python

**Date:** 2026-03-03
**Auditor:** Claude Opus 4.6 (1M context)
**Java source:** decompiled CFR 0.152
**Python source:** packages/engine/combat_engine.py

---

## 1. Start of Combat

### Java Sequence (AbstractRoom.update() waitTimer path + AbstractPlayer.preBattlePrep())

When `waitTimer` expires (first combat frame):

| Step | Java Code | File:Line |
|------|-----------|-----------|
| 1 | `player.preBattlePrep()` — clears action manager, inits deck, shuffles draw pile, calls `monsters.usePreBattleAction()`, calls `player.applyPreCombatLogic()` | AbstractPlayer.java:1546-1590 |
| 2 | `actionManager.turnHasEnded = true` | AbstractRoom.java:226 |
| 3 | `addToBottom(GainEnergyAndEnableControlsAction(energyMaster))` — sets energy, triggers `onEnergyRecharge` on relics+powers | AbstractRoom.java:230, GainEnergyAndEnableControlsAction.java:22-38 |
| 4 | `player.applyStartOfCombatPreDrawLogic()` — calls `r.atBattleStartPreDraw()` on all relics | AbstractRoom.java:231, AbstractPlayer.java:1885-1890 |
| 5 | `addToBottom(DrawCardAction(player, gameHandSize))` — queued draw of 5 cards | AbstractRoom.java:232 |
| 6 | `addToBottom(EnableEndTurnButtonAction())` | AbstractRoom.java:233 |
| 7 | `player.applyStartOfCombatLogic()` — calls `r.atBattleStart()` on all relics | AbstractRoom.java:235, AbstractPlayer.java:1874-1883 |
| 8 | `player.applyStartOfTurnRelics()` — calls `stance.atStartOfTurn()` then `r.atTurnStart()` on all relics | AbstractRoom.java:243, AbstractPlayer.java:1892-1902 |
| 9 | `player.applyStartOfTurnPostDrawRelics()` — calls `r.atTurnStartPostDraw()` on all relics | AbstractRoom.java:244, AbstractPlayer.java:1904-1909 |
| 10 | `player.applyStartOfTurnCards()` — calls `c.atTurnStart()` on draw/hand/discard | AbstractRoom.java:245, AbstractPlayer.java:1918-1931 |
| 11 | `player.applyStartOfTurnPowers()` — calls `p.atStartOfTurn()` on all player powers | AbstractRoom.java:246 (inherited method) |
| 12 | `player.applyStartOfTurnOrbs()` — triggers passive orb effects | AbstractRoom.java:247, AbstractPlayer.java:2252-2261 |
| 13 | `actionManager.useNextCombatActions()` — fires any queued next-combat actions | AbstractRoom.java:248 |

**Key observation:** In Java, `atBattleStart` relics fire AFTER the draw action is queued but BEFORE it executes (action queue). The draw action is bottom-queued. `GainEnergyAndEnableControlsAction` also bottom-queued and runs first. So actual execution order when the action queue processes:
1. GainEnergyAndEnableControlsAction (energy + onEnergyRecharge)
2. DrawCardAction (draw 5 cards)
3. EnableEndTurnButtonAction

But `atBattleStart`, `atTurnStart`, `atTurnStartPostDraw`, `atStartOfTurn`, etc. are called SYNCHRONOUSLY (not via action queue) and can `addToBottom` or `addToTop` actions themselves.

### Python Sequence (CombatEngine.start_combat())

| Step | Python Code | Line |
|------|-------------|------|
| 1 | Shuffle draw pile | combat_engine.py:264 |
| 2 | Move Innate cards to top of draw pile | combat_engine.py:267-276 |
| 3 | Roll initial moves for all enemies | combat_engine.py:279-280 |
| 4 | `execute_relic_triggers("atBattleStart", ...)` | combat_engine.py:283 |
| 5 | `execute_relic_triggers("atBattleStartPreDraw", ...)` | combat_engine.py:286 |
| 6 | `_start_player_turn()` (which does energy, block reset, draw, etc.) | combat_engine.py:289 |

### PARITY ISSUES: Start of Combat

| # | Issue | Severity | Description |
|---|-------|----------|-------------|
| **C1** | **atBattleStart vs atBattleStartPreDraw order is REVERSED** | **CRITICAL** | Java: `atBattleStartPreDraw` (step 4) fires BEFORE `atBattleStart` (step 7). Python: `atBattleStart` (step 4) fires BEFORE `atBattleStartPreDraw` (step 5). In Java, `atBattleStartPreDraw` adds cards like Miracle (Pure Water), which should be in the draw pile BEFORE `atBattleStart` triggers like Vajra's Strength. The order matters because some atBattleStart relics can add actions that interact with cards. |
| **C2** | **Energy set before draw in Java, but during _start_player_turn in Python** | **MEDIUM** | Java: `GainEnergyAndEnableControlsAction` is queued first and gives energy, then `DrawCardAction` draws. Also triggers `r.onEnergyRecharge()` and `p.onEnergyRecharge()`. Python: `_start_player_turn()` sets energy directly (line 297), then triggers `onEnergyRecharge` (line 342) AFTER `_trigger_start_of_turn()`, then draws (line 352). The energy recharge hooks fire at different relative positions. |
| **C3** | **Stance atStartOfTurn missing from start of combat path** | **LOW** | Java calls `stance.atStartOfTurn()` inside `applyStartOfTurnRelics()` (line 1893). Python handles Divinity exit in `_start_player_turn()` (line 317-318) but does not call a generic `stance.atStartOfTurn()`. Functionally equivalent for Watcher since Divinity auto-exit is the only stance with `atStartOfTurn`, but technically the hook order differs. |
| **C4** | **applyStartOfTurnCards missing entirely** | **MEDIUM** | Java calls `player.applyStartOfTurnCards()` which iterates draw/hand/discard calling `c.atTurnStart()` on each card (lines 1918-1931). Python has no equivalent. This matters for cards with `atTurnStart` behavior (e.g., resetting per-turn state). |
| **C5** | **applyStartOfTurnPreDrawCards missing on first turn** | **LOW** | Java calls this on subsequent turns (GameActionManager.java:332) but not on the first turn in AbstractRoom. Python doesn't call it at all. Affects cards like those with `atTurnStartPreDraw` behavior. |
| **C6** | **Enemy usePreBattleAction timing** | **LOW** | Java calls `monsters.usePreBattleAction()` during `preBattlePrep()` BEFORE any combat hooks. Python rolls enemy moves (step 3) but doesn't have a separate pre-battle action concept. Most enemy pre-battle actions are handled by enemy AI constructors. |

---

## 2. Start of Player Turn (Subsequent Turns)

### Java Sequence (GameActionManager.getNextAction() "turnHasEnded" branch)

After all monster turns complete and `turnHasEnded` is true:

| Step | Java Code | Line |
|------|-----------|------|
| 1 | `monsters.applyEndOfTurnPowers()` — for each monster: `m.applyEndOfTurnTriggers()`, then player `p.atEndOfRound()`, then monster `p.atEndOfRound()` | GameActionManager.java:320, MonsterGroup.java:285-299 |
| 2 | `player.cardsPlayedThisTurn = 0` | GameActionManager.java:322 |
| 3 | `player.applyStartOfTurnRelics()` — `stance.atStartOfTurn()` then `r.atTurnStart()` | GameActionManager.java:331, AbstractPlayer.java:1892-1902 |
| 4 | `player.applyStartOfTurnPreDrawCards()` — `c.atTurnStartPreDraw()` on hand | GameActionManager.java:332, AbstractPlayer.java:1911-1916 |
| 5 | `player.applyStartOfTurnCards()` — `c.atTurnStart()` on draw/hand/discard | GameActionManager.java:333, AbstractPlayer.java:1918-1931 |
| 6 | `player.applyStartOfTurnPowers()` — `p.atStartOfTurn()` on player powers | GameActionManager.java:334 |
| 7 | `player.applyStartOfTurnOrbs()` | GameActionManager.java:335 |
| 8 | `++turn` | GameActionManager.java:336 |
| 9 | `turnHasEnded = false`, counter resets | GameActionManager.java:337-341 |
| 10 | Block decay: if no Barricade/Blur, `loseBlock()` or `loseBlock(15)` for Calipers | GameActionManager.java:342-348 |
| 11 | `addToBottom(DrawCardAction(gameHandSize))` — queue draw | GameActionManager.java:350 |
| 12 | `player.applyStartOfTurnPostDrawRelics()` — `r.atTurnStartPostDraw()` | GameActionManager.java:351 |
| 13 | `player.applyStartOfTurnPostDrawPowers()` — `p.atStartOfTurnPostDraw()` | GameActionManager.java:352 |
| 14 | `addToBottom(EnableEndTurnButtonAction)` | GameActionManager.java:353 |

**CRITICAL: Block decay happens at step 10, AFTER turn counter increment and AFTER start-of-turn relics/powers. Draw is queued (step 11) but executes after synchronous steps.**

### Python Sequence (CombatEngine._start_player_turn())

| Step | Python Code | Line |
|------|-------------|------|
| 1 | `turn += 1` | combat_engine.py:293 |
| 2 | `phase = PLAYER_TURN` | combat_engine.py:294 |
| 3 | `energy = max_energy` | combat_engine.py:297 |
| 4 | Block decay (Barricade/Blur/Calipers check) | combat_engine.py:300-313 |
| 5 | Divinity auto-exit | combat_engine.py:317-318 |
| 6 | Reset turn counters | combat_engine.py:321-326 |
| 7 | `execute_relic_triggers("atTurnStart", ...)` | combat_engine.py:329 |
| 8 | `_trigger_start_of_turn()` which calls `execute_power_triggers("atStartOfTurn", ...)` | combat_engine.py:332, 367 |
| 9 | Check player death (from poison etc.) | combat_engine.py:335-339 |
| 10 | `execute_power_triggers("onEnergyRecharge", ...)` | combat_engine.py:342 |
| 11 | `_draw_cards(5)` — immediate draw | combat_engine.py:344-352 |
| 12 | `execute_relic_triggers("atTurnStartPostDraw", ...)` | combat_engine.py:355 |
| 13 | `execute_power_triggers("atStartOfTurnPostDraw", ...)` | combat_engine.py:356 |

### PARITY ISSUES: Start of Turn

| # | Issue | Severity | Description |
|---|-------|----------|-------------|
| **T1** | **Block decay timing: Python (step 4) vs Java (step 10)** | **HIGH** | Java decays block AFTER `applyStartOfTurnRelics()`, `applyStartOfTurnPowers()`, and even AFTER incrementing the turn counter. Python decays block FIRST before anything else. This means in Java, start-of-turn power triggers (like Poison damage via `atStartOfTurn`) happen WHILE THE PLAYER STILL HAS LAST TURN'S BLOCK. In Python, block is already gone. **This causes different HP outcomes when poison ticks against block.** |
| **T2** | **Energy recharge timing** | **MEDIUM** | Java: Energy is set via `GainEnergyAndEnableControlsAction` which fires as an action (and triggers `onEnergyRecharge` on relics then powers during action processing). On subsequent turns, there is no explicit GainEnergy action - the energy is handled by the draw action's `endTurnDraw=true` flag. Python: Energy set synchronously at start of `_start_player_turn()` (step 3), `onEnergyRecharge` power triggers at step 10. |
| **T3** | **Turn increment timing** | **LOW** | Java: `++turn` at step 8 (after start-of-turn triggers). Python: `turn += 1` at step 1 (before everything). This means power triggers that check turn number see different values. E.g., a power checking `GameActionManager.turn` during `atStartOfTurn` would see the NEW turn in Python but the OLD turn in Java. |
| **T4** | **applyStartOfTurnCards and applyStartOfTurnPreDrawCards missing** | **MEDIUM** | Java calls `c.atTurnStart()` on all cards in draw/hand/discard and `c.atTurnStartPreDraw()` on hand cards. Python omits both. This affects cards that reset per-turn state (e.g., cost reductions that expire). |
| **T5** | **Divinity exit timing vs Java** | **LOW** | Java: `stance.atStartOfTurn()` is called inside `applyStartOfTurnRelics()` (step 3). Python: Divinity exit at step 5 (after block decay). Since Java block decay is at step 10 (after stance exit), Mental Fortress block from Divinity exit would persist in Java. In Python, block already decayed, so Mental Fortress block from exit is applied fresh (which is correct behavior but for different reasons). |
| **T6** | **Blur handling is incorrect** | **HIGH** | Java: Checks for `hasPower("Blur")` alongside Barricade — if Blur power exists, block is NOT decayed at all. Blur's `atEndOfRound` then decrements itself. Python: Treats Blur as a status counter, decrements it and retains block only for the decremented turn. The Java Blur power retains ALL block (like Barricade) for the turn it's active, and decrements at end of round. Python's implementation may differ in edge cases where multiple stacks of Blur interact with partial decay. |

---

## 3. End of Turn

### Java Sequence

When player clicks "End Turn" (`player.isEndingTurn = true`):

**Phase A: AbstractRoom.endTurn() (synchronous)**

| Step | Java Code | Line |
|------|-----------|------|
| A1 | `player.applyEndOfTurnTriggers()` — calls `p.atEndOfTurn(true)` on all player powers | AbstractRoom.java:384 |
| A2 | `addToBottom(ClearCardQueueAction)` | AbstractRoom.java:385 |
| A3 | `addToBottom(DiscardAtEndOfTurnAction)` — handles retain, ethereal exhaust, discard | AbstractRoom.java:386 |
| A4 | Reset card attributes on draw/discard/hand | AbstractRoom.java:387-398 |
| A5 | Queue anonymous action that adds EndTurnAction + WaitAction + MonsterStartTurnAction | AbstractRoom.java:399-411 |
| A6 | `player.isEndingTurn = false` | AbstractRoom.java:412 |

**Phase B: callEndOfTurnActions() (from cardQueue processing when card==null)**

| Step | Java Code | Line |
|------|-----------|------|
| B1 | `applyEndOfTurnRelics()` — `r.onPlayerEndTurn()` on all relics | GameActionManager.java:359, AbstractRoom.java:528-535 |
| B2 | `applyEndOfTurnPreCardPowers()` — `p.atEndOfTurnPreEndTurnCards(true)` on all player powers (Metallicize, Plated Armor, Like Water) | GameActionManager.java:360, AbstractRoom.java:537-541 |
| B3 | `addToBottom(TriggerEndOfTurnOrbsAction)` — orb end-of-turn passives | GameActionManager.java:361 |
| B4 | For each card in hand: `c.triggerOnEndOfTurnForPlayingCard()` — auto-play end-of-turn cards (e.g., Burn, Regret) | GameActionManager.java:362-364 |
| B5 | `player.stance.onEndOfTurn()` | GameActionManager.java:365 |

**Phase C: DiscardAtEndOfTurnAction.update() (queued action)**

| Step | Java Code | Line |
|------|-----------|------|
| C1 | Separate retained cards (retain/selfRetain) into limbo | DiscardAtEndOfTurnAction.java:27-33 |
| C2 | Queue RestoreRetainedCardsAction (returns retained cards) | DiscardAtEndOfTurnAction.java:34 |
| C3 | If no Runic Pyramid / no Equilibrium: queue DiscardAction for each card | DiscardAtEndOfTurnAction.java:35-39 |
| C4 | Shuffle hand copy, call `c.triggerOnEndOfPlayerTurn()` (Ethereal exhaust) | DiscardAtEndOfTurnAction.java:41-45 |

**Phase D: EndTurnAction (queued from step A5)**

| Step | Java Code | Line |
|------|-----------|------|
| D1 | `actionManager.endTurn()` — sets `turnHasEnded = true`, records player HP | EndTurnAction.java:14, GameActionManager.java:168-172 |

**Phase E: MonsterStartTurnAction (queued from step A5)**

| Step | Java Code | Line |
|------|-----------|------|
| E1 | `monsters.applyPreTurnLogic()` — for each monster: `loseBlock()` (unless Barricade), then `applyStartOfTurnPowers()` (which calls `p.atStartOfTurn()` for monster powers: poison tick, regeneration, etc.) | MonsterStartTurnAction.java:22, MonsterGroup.java:93-101 |

**Phase F: Monster turns (getNextAction() monsterQueue processing)**

| Step | Java Code | Line |
|------|-----------|------|
| F1 | `monsters.queueMonsters()` — add all living monsters to queue | GameActionManager.java:295 |
| F2 | For each monster: `m.takeTurn()` — execute the move | GameActionManager.java:311 |
| F3 | For each monster: `m.applyTurnPowers()` — `p.duringTurn()` | GameActionManager.java:312 |
| F4 | After all monsters done: `monsters.applyEndOfTurnPowers()` — monster `applyEndOfTurnTriggers` + `atEndOfRound` on player + monsters | GameActionManager.java:320, MonsterGroup.java:285-299 |

### Python Sequence (CombatEngine.end_turn())

| Step | Python Code | Line |
|------|-------------|------|
| 1 | `execute_relic_triggers("onPlayerEndTurn", ...)` | combat_engine.py:393 |
| 2 | `_discard_hand()` — handle retain, ethereal exhaust, Runic Pyramid | combat_engine.py:396, 430-458 |
| 3 | `execute_power_triggers("atEndOfTurnPreEndTurnCards", ...)` (Metallicize, etc.) | combat_engine.py:401 |
| 4 | Regen healing (inline) | combat_engine.py:405-408 |
| 5 | `execute_power_triggers("atEndOfTurn", ...)` for player | combat_engine.py:411 |
| 6 | `execute_power_triggers("atEndOfTurn", ...)` for each living enemy | combat_engine.py:412-414 |
| 7 | `_do_enemy_turns()` — enemy block reset, poison tick, Ritual, execute moves | combat_engine.py:417, 460-528 |
| 8 | `execute_power_triggers("atEndOfRound", ...)` for player | combat_engine.py:420 |
| 9 | `execute_power_triggers("atEndOfRound", ...)` for each living enemy | combat_engine.py:421-423 |
| 10 | Check combat end, then `_start_player_turn()` | combat_engine.py:426-428 |

### PARITY ISSUES: End of Turn

| # | Issue | Severity | Description |
|---|-------|----------|-------------|
| **E1** | **applyEndOfTurnTriggers (player atEndOfTurn) fires BEFORE discard in Java, but player atEndOfTurn fires AFTER discard in Python** | **CRITICAL** | Java: `player.applyEndOfTurnTriggers()` (A1) fires first, then `DiscardAtEndOfTurnAction` (C1-C4). Python: `_discard_hand()` (step 2) fires first, then `atEndOfTurn` power triggers (step 5). This means powers like Constricted, Combust, or any power that deals damage `atEndOfTurn` would fire while the player still has cards in hand in Java, but after discard in Python. Also affects powers that check hand size at end of turn. |
| **E2** | **callEndOfTurnActions timing: relics and pre-card powers before discard in Java** | **HIGH** | Java: `applyEndOfTurnRelics` (B1) and `atEndOfTurnPreEndTurnCards` (B2) fire from `callEndOfTurnActions` which runs as part of card queue processing BEFORE `DiscardAtEndOfTurnAction`. Python: `onPlayerEndTurn` relics (step 1) fire before discard, but `atEndOfTurnPreEndTurnCards` (step 3) fires AFTER discard. Metallicize block gained at step B2 in Java happens before discard, but at step 3 in Python it happens after discard. Functionally identical for Metallicize but wrong for powers that interact with cards in hand. |
| **E3** | **triggerOnEndOfTurnForPlayingCard missing** | **HIGH** | Java's `callEndOfTurnActions` (B4) iterates hand and calls `c.triggerOnEndOfTurnForPlayingCard()` on each card BEFORE the discard. This auto-plays cards like Burn (deals damage to player) and Regret (lose HP per cards in hand). Python has no equivalent - these end-of-turn card effects are entirely missing. Burn cards in hand at end of turn should deal damage, Decay should deal damage, Regret should cause HP loss. |
| **E4** | **triggerOnEndOfPlayerTurn (Ethereal exhaust) timing** | **MEDIUM** | Java: `c.triggerOnEndOfPlayerTurn()` is called in `DiscardAtEndOfTurnAction` (C4) on a SHUFFLED copy of the hand AFTER retained cards are moved to limbo. Python: Ethereal exhaust is handled inline in `_discard_hand()` (line 443/455) as part of the same discard logic. The shuffle in Java is cosmetic (for animation), but the fact that it's processed after retain separation means the logic is slightly different. |
| **E5** | **Enemy atEndOfTurn fires twice for enemies in Python** | **MEDIUM** | Python step 6: `execute_power_triggers("atEndOfTurn", ...)` for each enemy. Then step 7: `_do_enemy_turns()` which also handles enemy poison ticks. In Java, enemy `atEndOfTurn` is NOT called separately - instead it's monster `applyEndOfTurnTriggers()` which fires at step F4 AFTER all monster turns complete. Python fires enemy `atEndOfTurn` at step 6 (before enemy turns) AND might fire it again via registry during enemy turns. Need to verify no double-trigger. |
| **E6** | **Monster applyStartOfTurnPowers (poison tick) timing** | **HIGH** | Java: Monster poison ticks happen in `MonsterStartTurnAction` -> `monsters.applyPreTurnLogic()` -> `m.applyStartOfTurnPowers()` which fires BEFORE `m.takeTurn()`. Each monster loses block, then has `atStartOfTurn` called on all powers (including Poison which deals damage and decrements). Python: Poison tick happens inside `_do_enemy_turns()` as inline code (lines 477-490). The timing is the same (before move execution) but the mechanism differs - Java uses the power system, Python is hardcoded. |
| **E7** | **Regen should fire at atEndOfTurn in Java, not as separate step** | **LOW** | Python has inline Regen healing (step 4) between `atEndOfTurnPreEndTurnCards` and `atEndOfTurn`. In Java, Regen is handled by its power's `atEndOfTurn(true)` method (part of step A1). The position matters because Regen healing in Java happens during `applyEndOfTurnTriggers` (before discard), while in Python it happens after discard. |
| **E8** | **applyEndOfTurnTriggers for monsters vs atEndOfRound separation** | **MEDIUM** | Java `MonsterGroup.applyEndOfTurnPowers()` (F4) does three things in order: (1) `m.applyEndOfTurnTriggers()` for each monster, (2) `p.atEndOfRound()` for player, (3) `p.atEndOfRound()` for each monster. Python does: player `atEndOfRound` (step 8), then enemy `atEndOfRound` (step 9). The monster `applyEndOfTurnTriggers` from Java step (1) has no Python equivalent at this point. |
| **E9** | **Enemy debuff decrement inside _do_enemy_turns vs atEndOfRound** | **MEDIUM** | Python decrements enemy Weak/Vuln/Frail inside `_do_enemy_turns()` (lines 507-515) and player debuffs at lines 523-527. Java handles ALL debuff decrements via `atEndOfRound` power hooks. This is a different mechanism — Python is hardcoded while Java is power-driven — but the position is also different. Python does it during the enemy turn phase; Java does it at step F4 after all monster actions complete. |

---

## 4. Card Play Sequence

### Java Sequence

**Phase A: Pre-play (GameActionManager.getNextAction() cardQueue processing)**

| Step | Java Code | Line |
|------|-----------|------|
| A1 | `p.onPlayCard(card, monster)` for each player power | GameActionManager.java:211-213 |
| A2 | `p.onPlayCard(card, monster)` for each monster's powers | GameActionManager.java:214-218 |
| A3 | `r.onPlayCard(card, monster)` for each player relic | GameActionManager.java:219-221 |
| A4 | `player.stance.onPlayCard(card)` | GameActionManager.java:222 |
| A5 | `c.onPlayCard(card, monster)` for each card in hand | GameActionManager.java:226-228 |
| A6 | Pay energy | (inline in cardQueue processing) |
| A7 | `card.use(player, monster)` — execute card's effect | (called via cardQueue) |

**Phase B: UseCardAction constructor (immediate)**

| Step | Java Code | Line |
|------|-----------|------|
| B1 | Set exhaust flag from `card.exhaustOnUseOnce || card.exhaust` | UseCardAction.java:32-34 |
| B2 | `p.onUseCard(card, this)` for each player power | UseCardAction.java:37-40 |
| B3 | `r.onUseCard(card, this)` for each player relic | UseCardAction.java:41-44 |
| B4 | `c.triggerOnCardPlayed(card)` for hand/discard/draw pile cards | UseCardAction.java:45-56 |
| B5 | `p.onUseCard(card, this)` for each monster's powers | UseCardAction.java:57-62 |

**Phase C: UseCardAction.update() (when action processes)**

| Step | Java Code | Line |
|------|-----------|------|
| C1 | `p.onAfterUseCard(card, this)` for each player power | UseCardAction.java:73-76 |
| C2 | `p.onAfterUseCard(card, this)` for each monster's powers | UseCardAction.java:77-82 |
| C3 | Handle card destination: purge / power empower / exhaust (Strange Spoon check) / rebound / shuffle-back / return-to-hand / discard | UseCardAction.java:85-128 |
| C4 | `card.exhaustOnUseOnce = false; card.dontTriggerOnUseCard = false` | UseCardAction.java:128-129 |

### Python Sequence (CombatEngine.play_card())

| Step | Python Code | Line |
|------|-------------|------|
| 1 | Validate hand index | combat_engine.py:1142-1143 |
| 2 | Check can play | combat_engine.py:1148-1149 |
| 3 | Pay energy (X-cost handling) | combat_engine.py:1154-1163 |
| 4 | Remove from hand | combat_engine.py:1166 |
| 5 | Track card play counters | combat_engine.py:1169-1179 |
| 6 | Get target enemy | combat_engine.py:1182-1186 |
| 7 | `execute_relic_triggers("onPlayCard", ...)` | combat_engine.py:1189 |
| 8 | `_apply_card_effects(card, ...)` — execute card effects | combat_engine.py:1192 |
| 9 | Card destination (exhaust/shuffle-back/discard) | combat_engine.py:1195-1207 |
| 10 | `execute_power_triggers("onUseCard", ...)` for player | combat_engine.py:1210 |
| 11 | `execute_power_triggers("onAfterUseCard", ...)` for player and monsters | combat_engine.py:1213-1224 |
| 12 | `execute_power_triggers("onAfterCardPlayed", ...)` for player and monsters | combat_engine.py:1227-1232 |
| 13 | Check Time Eater | combat_engine.py:1241 |
| 14 | Force end turn if needed | combat_engine.py:1244-1245 |
| 15 | `_check_combat_end()` | combat_engine.py:1248 |

### PARITY ISSUES: Card Play

| # | Issue | Severity | Description |
|---|-------|----------|-------------|
| **P1** | **onPlayCard fires BEFORE card.use() in Java, but after card removed from hand in Python** | **HIGH** | Java: `onPlayCard` hooks (A1-A5) fire BEFORE the card is used and while the card is still "in play". Python: Card is removed from hand (step 4) BEFORE `onPlayCard` triggers (step 7) and before effects (step 8). This matters for powers like Rushdown (Watcher) or Corruption which check hand state during `onPlayCard`. |
| **P2** | **Missing power.onPlayCard for monster powers** | **MEDIUM** | Java A2: Each monster's powers get `onPlayCard` called. Python step 7 only triggers relic `onPlayCard`, not monster power `onPlayCard`. Some enemy powers react to card plays (e.g., Angry). |
| **P3** | **Missing stance.onPlayCard** | **MEDIUM** | Java A4: `player.stance.onPlayCard(card)` is called before card execution. Python does not call any stance onPlayCard hook. Wrath stance's `onPlayCard` tracks Wrath-specific card play logic. |
| **P4** | **Missing card.triggerOnCardPlayed for hand/discard/draw** | **LOW** | Java B4: Cards in hand/discard/draw get `triggerOnCardPlayed` called. Python has no equivalent. This affects cards that react to other cards being played (e.g., Searing Blow counting). |
| **P5** | **onUseCard fires after card destination in Java (UseCardAction constructor), but before card destination in Python** | **HIGH** | Java: `onUseCard` (B2-B3) fires in the UseCardAction constructor (immediately after card queue processing), and the card destination (C3) happens in `update()` (next action processing tick). However, the UseCardAction constructor fires AFTER `card.use()`. Python: `onUseCard` (step 10) fires AFTER card destination (step 9). This reversal means powers checking exhaust pile during `onUseCard` would see different states. |
| **P6** | **Card removed from hand before effects in Python, during action in Java** | **HIGH** | Java: Card is in "limbo" during execution (moved to `player.cardInUse`). Python: Card is popped from `self.state.hand` at step 4 before any effects. This means effects that reference hand size, draw from hand, etc., see a hand with one fewer card in Python. |
| **P7** | **Strange Spoon not implemented** | **LOW** | Java C3: If card exhausts and player has Strange Spoon, 50% chance to not exhaust (goes to discard instead). Python has no Strange Spoon check. |
| **P8** | **Missing onAfterCardPlayed hook in Java** | **LOW** | Python step 12 calls `onAfterCardPlayed` which doesn't exist in the Java source. The Python hook `onAfterCardPlayed` appears to be a custom addition. Not harmful but the handlers registered for it need to map to actual Java hooks. |

---

## 5. Damage Application Sequence

### Java: Player Card -> Enemy (DamageInfo.applyPowers + AbstractMonster.damage)

**Phase A: Damage Calculation (AbstractCard.calculateCardDamage)**

| Step | Java Code | Line |
|------|-----------|------|
| A1 | Start with `baseDamage` | AbstractCard.java:2303 |
| A2 | `r.atDamageModify(tmp, card)` for each relic (e.g., Pen Nib) | AbstractCard.java:2304-2308 |
| A3 | `p.atDamageGive(tmp, type, card)` for each player power (Strength, Weak, etc.) | AbstractCard.java:2309-2311 |
| A4 | `stance.atDamageGive(tmp, type, card)` — Wrath 2x, Divinity 3x | AbstractCard.java:2312 |
| A5 | `p.atDamageReceive(tmp, type, card)` for each target power (Vulnerable) | AbstractCard.java:2315-2317 |
| A6 | `p.atDamageFinalGive(tmp, type, card)` for each player power | AbstractCard.java:2318-2320 |
| A7 | `p.atDamageFinalReceive(tmp, type, card)` for each target power (Intangible, Flight) | AbstractCard.java:2321-2323 |
| A8 | `floor(tmp)`, min 0 | AbstractCard.java:2326-2330 |

**Phase B: Damage Application (AbstractMonster.damage)**

| Step | Java Code | Line |
|------|-----------|------|
| B1 | Intangible check: if `hasPower("IntangiblePlayer")` and output > 0, set to 1 | AbstractMonster.java:610-612 |
| B2 | `decrementBlock(info, damageAmount)` — subtract block | AbstractMonster.java:625 |
| B3 | `r.onAttackToChangeDamage(info, damageAmount)` for player relics (e.g., Torii, but Torii uses onAttacked) | AbstractMonster.java:627-629 |
| B4 | `p.onAttackToChangeDamage(info, damageAmount)` for owner powers | AbstractMonster.java:632-634 |
| B5 | `p.onAttackedToChangeDamage(info, damageAmount)` for target powers (Buffer) | AbstractMonster.java:636-638 |
| B6 | `r.onAttack(info, damageAmount, this)` for player relics | AbstractMonster.java:640-642 |
| B7 | `p.wasHPLost(info, damageAmount)` for target powers | AbstractMonster.java:644-646 |
| B8 | `p.onAttack(info, damageAmount, this)` for owner powers | AbstractMonster.java:647-651 |
| B9 | `p.onAttacked(info, damageAmount)` for target powers | AbstractMonster.java:652-654 |
| B10 | `currentHealth -= damageAmount` | AbstractMonster.java:664 |
| B11 | Clamp HP >= 0 | AbstractMonster.java:668-670 |
| B12 | If HP <= 0: `die()`, check `areMonstersBasicallyDead()` | AbstractMonster.java:683-694 |

### Python: Player Card -> Enemy (_calculate_card_damage + _deal_damage_to_enemy)

**Phase A: Damage Calculation (_calculate_card_damage)**

| Step | Python Code | Line |
|------|-------------|------|
| A1 | Start with `base_damage` | combat_engine.py:1910 |
| A2 | Add `strength * multiplier` + `vigor` | combat_engine.py:1914-1915 (via calculate_damage) |
| A3 | Apply Weak (0.75x) | damage.py:127-131 |
| A4 | Apply stance mult (Wrath 2.0, Divinity 3.0) | damage.py:134 |
| A5 | Apply Vulnerable (1.5x) | damage.py:137-141 |
| A6 | (No atDamageFinalGive/Receive here) | - |
| A7 | Intangible cap at 1 | damage.py:147-148 |
| A8 | `int(damage)`, min 0 | damage.py:151 |

**Phase B: Damage Application (_deal_damage_to_enemy)**

| Step | Python Code | Line |
|------|-------------|------|
| B1 | (Vuln already applied in Phase A for card damage) | - |
| B2 | `blocked = min(enemy.block, damage)` | combat_engine.py:1499 |
| B3 | `hp_damage = damage - blocked` | combat_engine.py:1500 |
| B4 | `enemy.block -= blocked` | combat_engine.py:1501 |
| B5 | `enemy.hp -= hp_damage` | combat_engine.py:1502 |
| B6 | Curl Up trigger | combat_engine.py:1507-1511 |
| B7 | Sharp Hide trigger | combat_engine.py:1514-1520 |
| B8 | Clamp HP >= 0 | combat_engine.py:1523-1524 |
| B9 | Guardian mode shift | combat_engine.py:1528 |
| B10 | Split check | combat_engine.py:1531-1532 |
| B11 | If HP <= 0: `_on_enemy_death()` | combat_engine.py:1541-1542 |

### PARITY ISSUES: Damage Application (Player -> Enemy)

| # | Issue | Severity | Description |
|---|-------|----------|-------------|
| **D1** | **Relic atDamageModify missing** | **MEDIUM** | Java A2: `r.atDamageModify(tmp, card)` for each relic. Python has no equivalent. This matters for Pen Nib (handled specially in calculate_damage as a param) but may miss other relic damage modifiers. |
| **D2** | **Power-driven damage calculation vs hardcoded** | **HIGH** | Java uses power hooks (`atDamageGive`, `atDamageReceive`, `atDamageFinalGive`, `atDamageFinalReceive`) for ALL damage modification. Strength, Weak, Vulnerable, Pen Nib, etc. are all powers that implement these hooks. Python hardcodes these as parameters to `calculate_damage()`. This means any power that modifies damage via these hooks but isn't explicitly handled as a parameter will be missed. |
| **D3** | **atDamageFinalGive and atDamageFinalReceive missing from calculation** | **MEDIUM** | Java A6-A7: Final give/receive hooks run after initial damage calculation. Python has no equivalent in `_calculate_card_damage()`. These are used by Intangible (atDamageFinalReceive) and potentially other powers. While Python handles Intangible in calculate_damage(), the hook mechanism is absent. |
| **D4** | **onAttackToChangeDamage ordering** | **MEDIUM** | Java B3-B5: After block subtraction, `onAttackToChangeDamage` hooks run (for relics, then owner powers, then target powers). These can modify the post-block damage amount (e.g., Buffer reduces to 0). Python dispatches `onAttackedToChangeDamage` via power triggers (line 598) but in a different position relative to other hooks. |
| **D5** | **wasHPLost fires before onAttack/onAttacked in Java** | **LOW** | Java: `wasHPLost` (B7) fires BEFORE `onAttack` (B8) and `onAttacked` (B9). Python dispatches these in a different order relative to the actual HP subtraction. |
| **D6** | **HP subtracted before onAttacked in Java, after in Python for enemies** | **MEDIUM** | Java B10: `currentHealth -= damageAmount` THEN checks if HP <= 0 and calls `die()`. Python B5: `enemy.hp -= hp_damage`, then triggers fire. The HP is already reduced when onAttacked fires in both, but Java's `die()` call happens immediately after the HP reduction (B12), before returning to the caller. Python defers death handling to after all hooks. |

### Java: Enemy -> Player (DamageInfo + player.damage)

Python's `_execute_enemy_move()` handles enemy-to-player damage inline (combat_engine.py:544-665).

| # | Issue | Severity | Description |
|---|-------|----------|-------------|
| **D7** | **Enemy damage calc uses float math correctly** | **OK** | Python correctly uses `float(move_damage + strength)`, applies Weak, then stance, then Vuln, then floors. This matches the Java `DamageInfo.applyPowers` flow. |
| **D8** | **Torii before Intangible in Python, but both should be post-block** | **MEDIUM** | Python: Torii check (line 570-571) fires BEFORE Intangible (line 574-575), both on the pre-block `damage` value. Java: Torii is an `onAttacked` hook (fires after block subtraction on the post-block damage), Intangible is `atDamageFinalReceive` (fires before block subtraction on total damage). Python applies both before block, which is wrong — Torii should apply to unblocked damage only. |
| **D9** | **Damage modification hooks fire but don't modify the actual damage value** | **HIGH** | Python fires `atDamageGive`, `atDamageReceive`, `atDamageFinalReceive` via `execute_power_triggers()` (lines 578-595), but the return values are not used to modify the actual damage. The `execute_power_triggers` call at line 598 for `onAttackedToChangeDamage` IS used, but the others are fire-and-forget. In Java, every hook in the chain modifies `tmp` which feeds into the next hook. This means any power that modifies incoming enemy damage (other than those handled by `onAttackedToChangeDamage`) is silently ignored. |

---

## 6. Death Checking

### Java

| When | Mechanism |
|------|-----------|
| After each `damage()` call on monster | `if (currentHealth <= 0) { die(); ... }` inside `AbstractMonster.damage()` (line 683) |
| After each `damage()` call on player | Player death checked by action manager / end of action processing |
| `areMonstersBasicallyDead()` | Checked inside `damage()` after monster dies (line 685) — cleans card queue |
| Player HP 0 during `damage()` | Not immediately fatal — actions continue until action queue check |
| Combat end | Checked when `isBattleOver` is true AND action queue is empty (AbstractRoom.java:267) |

### Python

| When | Mechanism |
|------|-----------|
| After player takes damage in enemy move | `if player.hp <= 0: return` (line 663-665), then check Fairy in Bottle in `_do_enemy_turns` (line 501-505) |
| After each enemy damage call | `if enemy.hp <= 0: _on_enemy_death()` inside `_deal_damage_to_enemy()` (line 1541-1542) |
| After card play | `_check_combat_end()` (line 1248) |
| After enemy turns complete | `_check_combat_end()` at start of new turn (line 426) |

### PARITY ISSUES: Death Checking

| # | Issue | Severity | Description |
|---|-------|----------|-------------|
| **K1** | **Multi-hit attacks vs death checking** | **MEDIUM** | Java: If a monster dies during a multi-hit attack, `die()` is called immediately, and `areMonstersBasicallyDead()` cleans the card queue. Remaining hits on that dead monster would be skipped because the monster is marked as dying. Python: `_apply_card_damage()` (line 1485-1487) breaks out of the hit loop when `enemy.hp <= 0`, which is correct, but the death triggers fire immediately via `_on_enemy_death()`. |
| **K2** | **Player death during multi-hit enemy attacks** | **LOW** | Both Java and Python check player HP after each hit. Java: Player death isn't immediate (action queue continues). Python: Returns immediately from the hit loop (line 663-665). Functionally similar since remaining hits don't matter if player is dead. |
| **K3** | **Combat end check timing** | **LOW** | Java delays combat end until action queue is fully empty (`isBattleOver && actions.isEmpty()`). Python checks immediately after card play and at turn transitions. In practice, this difference shouldn't affect outcomes since Python has no action queue. |

---

## 7. Summary of Critical Issues (Priority-Ordered)

### CRITICAL (Wrong game outcomes)

| ID | Issue | Impact |
|----|-------|--------|
| **E1** | Player `atEndOfTurn` fires BEFORE discard in Java, AFTER in Python | Powers that deal damage/check hand at end of turn see different hand states |
| **C1** | `atBattleStart` / `atBattleStartPreDraw` order reversed | Cards added by Pure Water might not be in draw pile when atBattleStart fires |
| **T1** | Block decay AFTER start-of-turn triggers in Java, BEFORE in Python | Poison damage against block yields different HP |

### HIGH (Likely wrong outcomes in some games)

| ID | Issue | Impact |
|----|-------|--------|
| **E3** | `triggerOnEndOfTurnForPlayingCard` missing (Burn, Regret, Decay) | End-of-turn card auto-play effects entirely missing |
| **T6** | Blur handled as counter not power | Block retention logic may differ with multiple Blur stacks |
| **P1** | `onPlayCard` fires after card removed from hand | Powers checking hand state during onPlayCard see wrong hand |
| **P5** | `onUseCard` vs card destination ordering reversed | Powers checking exhaust pile during onUseCard see wrong state |
| **P6** | Card removed from hand before effects | Hand size wrong during card execution |
| **D2** | Hardcoded damage calc vs power-driven | Any damage-modifying power not explicitly coded is missing |
| **D9** | Enemy damage hooks fire-and-forget (don't modify damage) | Powers modifying incoming damage are silently ignored |
| **E6** | Monster poison tick mechanism (inline vs power-driven) | Works but fragile — won't handle non-standard atStartOfTurn powers |
| **D8** | Torii/Intangible ordering wrong for enemy attacks | Torii applies to pre-block damage instead of post-block |

### MEDIUM (Could cause parity issues in specific scenarios)

| ID | Issue | Impact |
|----|-------|--------|
| **C2** | Energy set timing differs first turn | onEnergyRecharge hooks fire at wrong time relative to other hooks |
| **C4** | `applyStartOfTurnCards` missing | Cards with atTurnStart behavior (cost resets, etc.) don't reset |
| **T2** | Energy recharge mechanism differs subsequent turns | Functional but different hook ordering |
| **T3** | Turn increment timing (start vs middle) | Powers checking turn number see different values |
| **T4** | `applyStartOfTurnCards` and `PreDrawCards` missing | Card per-turn state not reset |
| **E2** | `atEndOfTurnPreEndTurnCards` timing relative to discard | Metallicize block happens at different time relative to hand state |
| **E4** | Ethereal exhaust mechanism differs | Functionally same but different ordering within discard |
| **E5** | Enemy `atEndOfTurn` may double-fire | Registry triggers + inline code could cause duplicate effects |
| **E8** | Monster `applyEndOfTurnTriggers` vs atEndOfRound separation | Monster end-of-turn trigger mechanism differs |
| **E9** | Enemy debuff decrement inline vs power-driven | Works but at different timing |
| **P2** | Missing monster power `onPlayCard` | Angry and similar enemy powers don't react to card plays |
| **P3** | Missing `stance.onPlayCard` | Wrath-specific card play tracking missing |
| **D1** | Missing relic `atDamageModify` | Relic damage modifiers beyond hardcoded ones are missed |
| **D3** | Missing `atDamageFinalGive/Receive` in card damage calc | Powers using final hooks for damage modification missed |
| **D4** | `onAttackToChangeDamage` ordering | Buffer and similar powers may fire at wrong time |
| **D6** | Death handling timing in damage chain | Edge case where hooks after death see different state |

---

## 8. Recommended Fix Order

1. **E1 + E3**: Restructure `end_turn()` to match Java order: player `atEndOfTurn` -> end-of-turn relics -> end-of-turn pre-card powers -> trigger end-of-turn cards (Burn/Regret/Decay) -> THEN discard hand
2. **T1**: Move block decay AFTER start-of-turn relic/power triggers in `_start_player_turn()`
3. **C1**: Swap `atBattleStartPreDraw` to fire before `atBattleStart` in `start_combat()`
4. **D8 + D9**: Fix enemy incoming damage: make power hooks actually modify damage values; fix Torii to apply post-block
5. **P1 + P5 + P6**: Restructure `play_card()`: fire onPlayCard hooks before removing card from hand; fire card effects while card is still "in play"
6. **T6**: Fix Blur to match Java (full block retention while active, decrement at end of round)
7. **C4 + T4**: Add `applyStartOfTurnCards` loop calling atTurnStart on all cards
8. **D2**: Migrate damage calculation to use power hook chain instead of hardcoded parameters (long-term)
