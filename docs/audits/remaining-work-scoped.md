# Remaining Work -- Verified Gap List (2026-03-11)

**Methodology:** Every item from final-gap-checklist.md and java-*-summary.md was cross-referenced against the CURRENT Python source code. Items are classified as:
- **MISSING**: Confirmed absent from Python engine, verified as real Java gameplay mechanic
- **GHOST**: Listed as missing in old audit docs but already implemented in current code
- **IRRELEVANT**: Test/deprecated/UI-only/non-gameplay item

**Current counts (verified):**
- `@power_trigger` decorators in `powers.py`: 164
- `@relic_trigger` decorators in `relics.py`: 165
- Event handlers in `EVENT_HANDLERS` dict: 43
- Unique power IDs registered: 121

---

## GHOST Items (Listed as Missing, Actually Implemented)

These were marked MISSING in previous audits but are NOW present in the codebase. No action needed.

### Powers (20 GHOSTs)
| Old ID | Power | Where Implemented | Line |
|--------|-------|-------------------|------|
| PW-01 | AngerPower (Nob) | `powers.py` `@power_trigger("onPlayCard", power="Anger")` | L1730 |
| PW-02 | EnergyDownPower | `powers.py` `@power_trigger("atStartOfTurn", power="EnergyDownPower")` | L1701 |
| PW-03 | EndTurnDeathPower | `powers.py` `@power_trigger("atEndOfTurn", power="EndTurnDeath")` | L1686 |
| PW-04 | ConfusionPower | `powers.py` `@power_trigger("onCardDraw", power="Confusion")` | L1870 |
| PW-05 | CurlUpPower | `combat_engine.py` inline at L1676-1684 | L1676 |
| PW-06 | ArtifactPower | `combat_engine.py` inline `_apply_status` at L2367-2375 | L2367 |
| PW-10 | LiveForeverPower | `powers.py` `@power_trigger("atEndOfTurn", power="AngelForm")` | L1714 |
| PW-11 | GainStrengthPower | `powers.py` `@power_trigger("atEndOfTurn", power="Shackled")` | L1761 |
| PW-12 | HexPower | `powers.py` `@power_trigger("onUseCard", power="Hex")` | L1746 |
| PW-13 | SporeCloudPower | `combat_engine.py` inline `_handle_enemy_death` at L1822-1829 | L1822 |
| PW-14 | SharpHidePower | `combat_engine.py` inline at L1686-1690 | L1686 |
| PW-15 | ReactivePower | `powers.py` `@power_trigger("onAttacked", power="Compulsive")` | L1938 |
| PW-16 | RegenerateMonster | `powers.py` `@power_trigger("atEndOfTurn", power="Regenerate")` | L1839 |
| PW-08 | CollectPower | `powers.py` `@power_trigger("onEnergyRecharge", power="Collect")` | L1883 |
| PW-17 | AmplifyPower | `powers.py` `@power_trigger("onUseCard", power="Amplify")` | L2013 |
| PW-25 | GenericStrengthUp | `powers.py` `@power_trigger("atEndOfRound", power="Generic Strength Up Power")` | L1778 |
| PW-24 | ForcefieldPower | `powers.py` `@power_trigger("atDamageFinalReceive", power="Nullify Attack")` | L1787 |
| PW-28 | NoSkillsPower | `powers.py` `@power_trigger("atEndOfTurn", power="NoSkills")` | L1863 |
| PW-29 | PainfulStabsPower | `powers.py` `@power_trigger("onAttacked", power="Painful Stabs")` | L1852 |
| PW-30 | ReboundPower | `powers.py` `@power_trigger("onAfterUseCard", power="Rebound")` | L2045 |

### Combat Flow (6 GHOSTs)
| Old ID | Issue | Status |
|--------|-------|--------|
| CF-01 | Block decay before start-of-turn triggers | FIXED: Block reset at L345 (after L325 atTurnStart triggers) |
| CF-02 | Player atEndOfTurn fires after discard | FIXED: `atEndOfTurn` at L444 fires BEFORE `_discard_hand()` at L455 |
| CF-03 | End-of-turn card auto-play (Burn/Regret/Decay) | FIXED: `_trigger_end_of_turn_cards()` at L449/L483 |
| CF-04 | atBattleStartPreDraw after atBattleStart | FIXED: PreDraw at L284 fires before Start at L285 |
| CF-05 | onPlayCard fires after card removed from hand | FIXED: onPlayCard at L1316 fires BEFORE hand.pop at L1326 |
| CF-10 | Monster power onPlayCard missing | FIXED: L1317-1320 loops enemy powers for onPlayCard |

### Cards (7 GHOSTs)
| Old ID | Card | Status |
|--------|------|--------|
| CD-01 | Fasting missing EnergyDown | FIXED: `cards.py:1160` applies EnergyDownPower(1) |
| CD-02 | Spirit Shield includes self in hand count | FIXED: `cards.py:562` uses `len(ctx.hand) - 1` |
| CD-03 | Conjure Blade upgraded X+1 | FIXED: `cards.py:889-890` adds 1 when upgraded |
| CD-05 | WreathOfFlame uses custom status | FIXED: `executor.py:453` applies "Vigor" |
| CD-06 | Simmering Fury combined power | FIXED: `executor.py:454` applies WrathNextTurn + DrawCardNextTurn |
| CD-07 | WindmillStrike upgrade +4/turn | FIXED: `cards.py:419` base_magic=4, upgrade_magic=1 |
| CD-15 | Indignation doc bug | N/A (was already noted as doc-only) |

### Monsters (5 GHOSTs)
| Old ID | Monster | Status |
|--------|---------|--------|
| MN-01 | AcidSlime_S A17 pattern | FIXED: `enemies.py:894-904` alternates LICK/TACKLE |
| MN-02 | Looter/Mugger flee pattern | FIXED: `enemies.py:2580-2612` full MUG->MUG->SMOKE/LUNGE->ESCAPE |
| MN-03 | BronzeOrb first move Stasis | FIXED: `enemies.py:5385-5390` always Stasis on first move |
| MN-04 | Hexaghost burn_upgraded | FIXED: `enemies.py:1855` burn_upgraded flag tracked |
| MN-07 | Green Louse HP range | FIXED: `enemies.py:1143-1147` (11-17 / 12-18 A7+) |

### Events (7 GHOSTs)
| Old ID | Event | Status |
|--------|-------|--------|
| EV-01 | Back to Basics removes ALL | FIXED: `event_handler.py:1748-1764` removes 1 card |
| EV-03 | Forgotten Altar wrong mechanics | FIXED: `event_handler.py:1821-1836` +5 maxHP, percent damage |
| EV-04 | Mausoleum relic/curse inverted | FIXED: `event_handler.py:2099-2120` always relic, conditional curse |
| EV-05 | Nest options wrong | FIXED: `event_handler.py:1867-1884` steal=gold, join=6dmg+RitualDagger |
| EV-06 | Sensory Stone wrong structure | FIXED: `event_handler.py:2567-2593` 3 options with HP costs |
| EV-07 | Designer wrong options | FIXED: `event_handler.py:3324-3463` randomized via miscRng |
| EV-10 | Augmenter two errors | FIXED: `event_handler.py:3640-3678` option 0 no card removal, option 2 MutagenicStrength |

### Relics (17 GHOSTs)
| Old ID | Relic | Status |
|--------|-------|--------|
| RL-01 | Chemical X | FIXED: `combat_engine.py:1281-1283` inline check |
| RL-02 | Blue Candle (onUseCard) | FIXED: `relics.py:912-919` onPlayCard trigger |
| RL-03 | Medical Kit (onUseCard) | FIXED: `relics.py:924-930` onPlayCard trigger |
| RL-04 | Neow's Lament | FIXED: `relics.py:1654-1672` atBattleStart trigger |
| RL-05 | Omamori | FIXED: `relics.py:1542-1568` onEquip + onObtainCard triggers |
| RL-06 | Astrolabe | FIXED: `run.py:661-690` onEquip in _on_relic_obtained |
| RL-07 | Calling Bell | FIXED: `run.py:712-728` generates 3 tier relics + curse |
| RL-08 | Empty Cage | FIXED: `run.py:692-710` removes 2 cards |
| RL-09 | Pandora's Box | FIXED: `run.py:817-824` transforms strikes/defends |
| RL-10 | Tiny House | FIXED: `run.py:730-757` all 5 bonuses applied |
| RL-11 | Prayer Wheel | FIXED: `run.py:409` adds +1 card count |
| RL-12 | Question Card | FIXED: `run.py:411` adds +1 card count |
| RL-13 | Old Coin | FIXED: `relics.py:1516-1520` +300 gold onEquip |
| RL-14 | Busted Crown (passive) | FIXED: `run.py:413-414` -2 card choices |
| RL-15 | Singing Bowl | FIXED: `run.py:419` skip-for-HP option |
| PW-P2 | Plated Armor wasHPLost | FIXED: `powers.py:894` + L1990 (two triggers for wasHPLost) |
| RL-16 | Cursed Key (chest) | FIXED: `handlers/rooms.py:920-923` adds curse on chest open |

---

## ACTUALLY MISSING Items (Verified against current code)

### BLOCKS_RL (7 items -- MUST fix before training)

**M-01: Boss relic energy bonus not wired to combat**
- Priority: **HIGH** (every boss relic that gives +1 energy is broken at run level)
- Status: Relic data has `energy_bonus=1` field, but `game.py:_enter_combat()` always passes `energy=3` to `create_combat_from_enemies()` without computing the bonus from equipped boss relics.
- Affects: Busted Crown, Coffee Dripper, Cursed Key, Ectoplasm, Fusion Hammer, Mark of Pain, Philosopher's Stone, Runic Dome, Snecko Eye, Sozu, Velvet Choker (all +1 energy boss relics)
- Fix location: `packages/engine/game.py:3281` -- compute energy from 3 + sum of relic energy_bonus
- Also: `packages/engine/state/run.py` should track base_energy as a field

**M-02: Vault card "take_extra_turn" effect not consumed**
- Priority: **HIGH** (Watcher rare skill, core game mechanic)
- Status: `effects/cards.py:608-611` sets `ctx.extra_data["extra_turn"] = True` but `combat_engine.py` never reads this flag. Vault does nothing after playing.
- Java behavior: `SkipEnemiesTurnAction` skips all enemy turns, then `PressEndTurnButtonAction` ends player turn.
- Fix location: `packages/engine/combat_engine.py:end_turn()` -- check extra_turn flag, skip `_do_enemy_turns()` if set
- Note: VaultPower in Java is actually a DIFFERENT mechanic (deals damage at end of round) -- the Vault card doesn't use VaultPower.

**M-03: DrawPower passive draw modification missing**
- Priority: **MEDIUM** (used by Offering, Draw power, some enemies)
- Status: `combat_engine.py:373-374` reads `player.statuses.get("Draw", 0)` but there is no power trigger to SET or maintain this. DrawPower in Java modifies `gameHandSize` permanently on application and removal. Python only reads it statically.
- Fix: The inline read at L373 works IF the status is set correctly. Verify that cards/powers applying "Draw" set the value correctly.

**M-04: Vault power (PW-09) -- skip enemy turns**
- Priority: **HIGH** (same as M-02, this is the implementation need)
- Note: This is not the same as VaultPower from Java. The Vault card needs SkipEnemiesTurnAction behavior.
- Merged with M-02.

**M-05: CF-06 -- Enemy damage power hooks ignore return values**
- Priority: **MEDIUM** (affects monsters that modify their own damage)
- Status: Player damage hooks (atDamageGive, atDamageReceive, atDamageFinalReceive) ARE chained for player attacks. Need to verify enemy-to-player damage path chains return values.
- Fix location: `combat_engine.py` enemy damage path -- verify power hook return values are captured

**M-06: CF-09 -- Torii applies to pre-block damage**
- Priority: **MEDIUM** (Torii is a Rare relic, affects block efficiency)
- Status: `relics.py:1128` registers `onAttackedToChangeDamage` for Torii. Need to verify this fires AFTER block subtraction in the damage pipeline, not before.
- Fix location: Verify in `combat_engine.py` damage pipeline ordering

**M-07: CF-11 -- applyStartOfTurnCards missing**
- Priority: **LOW** (mostly affects cost reductions that expire)
- Status: `combat_engine.py:329-330` clears `card_costs` dict, which is a partial implementation. Java iterates ALL cards calling `c.atTurnStart()`.
- Fix: Only matters for cards with per-turn cost modifications. Current implementation may be sufficient.

### AFFECTS_RL (12 items -- should fix before serious training)

**(These are additional GHOSTs found during verification:)**

- **CF-07 Blur**: GHOST -- `combat_engine.py:347-351` retains all block when Blur active, `powers.py:1200` decrements at atEndOfRound. Correct.
- **CF-08 onUseCard timing**: GHOST -- `combat_engine.py:1333-1334` fires onUseCard BEFORE card destination at L1340-1351. Correct.
- **CF-12 stance.onPlayCard**: GHOST/IRRELEVANT -- No Watcher stance implements meaningful onPlayCard behavior in Java.
- **CF-13 Enemy atEndOfTurn**: GHOST -- `combat_engine.py:445-447` fires atEndOfTurn for enemies, `_do_enemy_turns()` handles actions. These are different phases, no double-trigger.
- **CF-14 Enemy debuff decrement**: GHOST -- `combat_engine.py:472-476` fires atEndOfRound for all enemies, handles via registry.

**M-13: EV-08 -- Wing Statue (GoldenWing) missing conditional gold option**
- Priority: **LOW** (Wing Statue is Act 1, conditional option is rare)
- Status: Python has 2 options (purify + leave). Java has 3: purify (7 damage + remove card), conditional gold (50-80g if player has Attack with 10+ damage), leave.
- File: `packages/engine/handlers/event_handler.py:1553`
- Java source: `decompiled/.../events/exordium/GoldenWing.java`

**M-14: EV-09 -- Pleading Vagrant (Addict) option ordering wrong**
- Priority: **MEDIUM** (option 1 and 2 are swapped vs Java)
- Status: Java has: option 0 = pay 85g for relic, option 1 = steal relic + Shame curse (free), option 2 = leave. Python has: option 0 = pay (CORRECT), option 1 = refuse with Shame only (WRONG -- should be steal relic + Shame), option 2 = rob relic + Shame (WRONG -- should be leave).
- File: `packages/engine/handlers/event_handler.py:1912-1928`
- Java source: `decompiled/.../events/city/Addict.java`

**M-15: EV-15 -- N'loth wrong relic selection**
- Priority: **LOW** (single event, Act 3 only)
- Status: `event_handler.py:3466` handler exists. Need to verify it presents 2 relics for selection vs auto-trading.
- File: `packages/engine/handlers/event_handler.py:3466`

**M-16: EV-16 -- Relic tier system**
- Priority: N/A -- **GHOST**
- Status: `_get_random_relic(run_state, misc_rng, "weighted")` correctly uses `_get_weighted_relic_tier()` (L330-339) matching Java's `returnRandomRelicTier()` with 50/33/17 Common/Uncommon/Rare split. ALL 9 event callers use `"weighted"`. No fix needed.

**M-17: EV-17 -- Transform card rarity-preserving system (partial)**
- Priority: **MEDIUM** (affects Transmogrifier, DrugDealer, Designer -- NOT LivingWall)
- Status: LivingWall is FIXED (event_handler.py:1437-1446 preserves rarity). However, Transmogrifier (L2965), Augmenter/DrugDealer (L3654), and Designer cleanup (L3416) all still use `"common"` instead of preserving rarity.
- Fix location: `packages/engine/handlers/event_handler.py` -- apply same rarity-preservation logic from Living Wall to these 3 handlers

**M-18: Ring of the Serpent boss relic missing**
- Priority: **LOW** (Silent-only, Watcher project)
- Status: Not in relics.py triggers. Data exists in `generation/relics.py:237`. Needs `onEquip` to set draw bonus.
- Fix location: `packages/engine/registry/relics.py` (add draw bonus trigger)

**M-19: Runic Dome boss relic energy not wired**
- Priority: **LOW** (UI-only effect "can't see intent", energy handled by M-01)
- Status: Runic Dome's energy bonus is covered by M-01 fix. The "can't see intent" effect is UI-only and irrelevant for RL.

### LOW Priority (16 items -- fix as needed)

**Powers still missing (confirmed not in powers.py):**

| ID | Power | Java Class | Watcher? | Notes |
|----|-------|------------|----------|-------|
| L-01 | ExplosivePower | `ExplosivePower.java` | No | Exploder enemy (currently hardcoded inline in combat_engine.py) |
| L-02 | DrawPower | `DrawPower.java` | No | Modifies hand size (Offering). Partially handled inline at combat_engine.py:373 |
| L-03 | DrawReductionPower | `DrawReductionPower.java` | No | atEndOfRound decrement IS registered at powers.py:1966. Draw reduction at combat start IS read at combat_engine.py:374-375. **Possible GHOST.** |
| L-04 | HelloPower | `HelloPower.java` | No | Defect (Hello World card) |
| L-05 | NightmarePower | `NightmarePower.java` | No | Silent (Nightmare card) |
| L-06 | ConservePower | N/A | No | Defect variant of Energized |
| L-07 | EnergizedBluePower | N/A | No | Defect variant |
| L-08 | StasisPower | `StasisPower.java` | No | Bronze Orb (return card on death) |
| L-09 | TimeMazePower | `TimeWarpPower.java` | No | Time Eater (hardcoded via Time Warp at powers.py:651) |
| L-10 | WinterPower | `WinterPower.java` | No | Defect (Winter orb buff) |
| L-11 | RechargingCorePower | N/A | No | Defect (Bronze Automaton) |
| L-12 | SkillBurnPower | N/A | No | Enemy (Skill exhaust) |
| L-13 | StrikeUpPower | N/A | No | Whetstone display-only |

**Relics still missing (confirmed not in relics.py, relics_passive.py, or run.py):**

| ID | Relic | Priority | Notes |
|----|-------|----------|-------|
| L-14 | Eternal Feather | GHOST | Rest site heal IS implemented in game.py:3500-3503. |
| L-15 | White Beast Statue | GHOST | Handled in potion generation (generation/potions.py:293). |
| L-16 | Cauldron | GHOST | IS implemented in run.py:759-768. |
| L-17 | Dolly's Mirror | GHOST | IS implemented in run.py:770-780. |
| L-18 | Orrery | GHOST | IS implemented in run.py:782-815. |
| L-19 | Circlet / Red Circlet | GHOST | Circlet IS the duplicate relic fallback at run.py:542. No game effect. |
| L-20 | Spirit Poop | IRRELEVANT | No game effect. |
| L-21 | N'loth's Mask | LOW | `onChestOpenAfter` trigger missing. Event-specific, rare. |
| L-22 | Frozen Eye | IRRELEVANT | UI-only (draw pile visible). In relics_passive.py:61. |
| L-23 | Prismatic Shard | LOW | Passive flag exists in relics_passive.py:64. Card reward system needs to check this flag. |
| L-24 | Discerning Monocle | LOW | Passive flag exists in relics_passive.py:65. Not the same as Membership Card -- Java effect is actually shop item rarity boost, NOT discount. |

**Monster gaps:**

| ID | Monster | Priority | Notes |
|----|---------|----------|-------|
| L-25 | MN-05: WrithingMass Reactive | LOW | Compulsive power IS registered. Re-roll on damage is handled via the power trigger. **Possible GHOST.** |
| L-26 | MN-06: SpireGrowth Constricted | LOW | Need combat engine to pass player constricted status. Edge case. |

---

## Summary

| Category | Old Count | GHOSTs Found | Actually Missing | Priority |
|----------|-----------|--------------|------------------|----------|
| Combat Flow | 19 | 11 | 2 real gaps | M-01 (energy), M-02 (Vault) critical |
| Powers | 50 | 20 | ~13 non-Watcher | DrawPower passive only real Watcher gap |
| Cards | 15 | 7 | 1 (Vault effect) | M-02 is the card fix |
| Monsters | 10 | 5 | 1-2 edge cases | Mostly fixed |
| Events | 23 | 9 | 4 real issues | Addict ordering, transform rarity, Wing Statue |
| Relics | 31 | 22 | 1 critical (energy) | M-01 is the fix; 6 GHOSTs in LOW |
| **Totals** | **148** | **74** | **~12 real gaps** | 2 critical, 3 medium, rest low |

### Critical Path (2 items, blocks training accuracy)

1. **M-01: Boss relic energy bonus** -- Every boss relic that gives +1 energy is broken. ~11 relics affected. Simple fix: compute energy from relics in `_enter_combat()`.
   - File: `packages/engine/game.py:3281`
   - Also: `packages/engine/state/run.py` (add base_energy tracking)

2. **M-02: Vault card does nothing** -- The "take_extra_turn" flag is set but never consumed. Vault is a key Watcher card.
   - File: `packages/engine/combat_engine.py:end_turn()` (check flag, skip enemy turns)
   - File: `packages/engine/effects/cards.py:608-611` (already sets the flag)

### Medium Priority (4 items)

3. **M-14: Addict event option ordering** -- Options 1 and 2 are swapped vs Java. Agent gets wrong EV for event choices.
4. **M-17: Transform rarity preservation** -- Transmogrifier, Augmenter, Designer transforms always give common cards. Should preserve rarity of removed card.
5. **M-05: Enemy damage power hook return values** -- Verify enemy-to-player damage chains power hooks correctly.
6. **M-03: DrawPower passive** -- Verify draw modification works end-to-end.

### Low Priority (rest)
Everything else is non-Watcher, edge cases, or cosmetic.
