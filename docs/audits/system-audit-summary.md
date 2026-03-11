# System Audit Summary — Java vs Python Engine Parity

**Date:** 2026-03-03 (updated continuously)
**Scope:** Complete analysis of all 1317 Java decompiled files vs 55 Python engine files
**Focus:** Watcher character, A20, combat flow, card/power/relic parity
**Tests:** 5769+ passing (up from 5698 at session start)

## Audit Documents Produced

| Document | Status | Summary |
|----------|--------|---------|
| `java-powers-summary.md` | COMPLETE | 149 powers: 83 implemented, 38 missing, 12 partial |
| `java-relics-summary.md` | COMPLETE | 180 relics: 129 implemented, 45 missing, 6 partial |
| `java-actions-summary.md` | COMPLETE | 280 actions: ~95 with Python equivalents |
| `card-parity-report.md` | COMPLETE | 85 Watcher cards: 55 perfect, 2 CRITICAL, 5 HIGH |
| `combat-flow-parity.md` | IN PROGRESS | Turn flow ordering analysis |
| `events-parity-report.md` | IN PROGRESS | 56 event files |
| `monsters-parity-report.md` | IN PROGRESS | 67 monster files |

---

## Critical Bugs Found & Fixed This Session

### Fixed (Committed)
1. **Devotion**: Used `player.statuses["Mantra"]` instead of `state.mantra` + bypassed `_change_stance` → MentalFortress/Rushdown hooks not firing on Divinity entry
2. **Foresight**: Set `pending_scry` but nothing read it → scry never happened
3. **LikeWater/Metallicize/Plated Armor/Study**: Double-trigger (registry handler + inline fallback)
4. **Indignation**: Applied Mantra in Wrath (wrong) → now applies Vulnerable to all enemies (Java parity)
5. **Fasting**: Missing EnergyDown penalty → free +3/+4 Str/Dex with no downside
6. **Spirit Shield**: Counted itself in hand → extra 3-4 block per use
7. **WreathOfFlame**: Used separate status instead of standard Vigor → broke stacking
8. **take_action()**: Redundant `get_available_actions()` call for logging

### New Powers Implemented
- EndTurnDeath/Blasphemy (registry trigger, end-of-turn)
- EnergyDownPower (start-of-turn energy loss from Fasting)
- AngelForm/LiveForever (end-of-turn Plated Armor gain)

### Engine Optimization
- RunState.copy(): 13.3x speedup (84µs → 6.3µs via manual copy)

---

## Combat Flow Ordering: Java vs Python

### Java Turn Flow (from GameActionManager.getNextAction)

**Start of Turn (non-first):**
1. `player.applyStartOfTurnRelics()` — stance.atStartOfTurn() + relic.atTurnStart()
2. `player.applyStartOfTurnPreDrawCards()` — hand cards.atTurnStartPreDraw()
3. `player.applyStartOfTurnCards()` — all cards.atTurnStart()
4. `player.applyStartOfTurnPowers()` — power triggers
5. `player.applyStartOfTurnOrbs()` — orb.onStartOfTurn()
6. `++turn`
7. **Block reset** (Barricade/Blur/Calipers check)
8. `DrawCardAction(gameHandSize)` — draw cards
9. `player.applyStartOfTurnPostDrawRelics()` — post-draw relics
10. `player.applyStartOfTurnPostDrawPowers()` — post-draw powers

**Python Turn Flow (_start_player_turn):**
1. `turn += 1`
2. `energy = max_energy`
3. **Block reset** ← OUT OF ORDER (should be step 7)
4. Divinity auto-exit
5. Reset counters
6. `execute_relic_triggers("atTurnStart")`
7. `_trigger_start_of_turn()` — power triggers
8. Death check
9. `execute_power_triggers("onEnergyRecharge")` — DevaForm etc
10. Draw cards
11. Post-draw triggers

### PARITY ISSUE: Block reset timing
- **Java**: Block resets AFTER start-of-turn triggers (step 7)
- **Python**: Block resets BEFORE start-of-turn triggers (step 3)
- **Impact**: LOW — only matters if a start-of-turn trigger grants block that should be cleared. MentalFortress from Divinity exit is the main case; Python is actually MORE favorable to player.
- **Recommendation**: Fix for correctness, but not blocking for RL training.

### End of Turn Flow
- **Java**: applyEndOfTurnTriggers → discard hand → endTurnAction → monsterStartTurnAction
- **Python**: discard hand → atEndOfTurnPreEndTurnCards powers → atEndOfTurn powers → enemy turns → atEndOfRound
- **Issue**: Python discards BEFORE atEndOfTurnPreEndTurnCards, Java discards AFTER (via DiscardAtEndOfTurnAction queued to action manager)
- **Impact**: MEDIUM — affects cards with `triggerOnEndOfPlayerTurn` that care about hand state

---

## Tests to Write / Port

### CRITICAL (affect combat outcomes)
1. **Block reset ordering test**: Verify block from Divinity exit + MF is handled correctly
2. **Fasting energy penalty test**: Verify EnergyDown reduces energy each turn
3. **Spirit Shield self-exclusion test**: Verify card count excludes itself
4. **Indignation Wrath behavior test**: Verify Vulnerable to all enemies, not Mantra
5. **End-of-turn sequence test**: Verify Metallicize/Plated Armor fire before discard

### HIGH (affect strategy learning)
6. **Vigor stacking test**: WreathOfFlame + Akabeko Vigor stack correctly
7. **Devotion → Divinity → MentalFortress chain test**: Full stance cycling
8. **Foresight + Nirvana + Golden Eye scry chain**: Verify scry amount and block
9. **Blasphemy timing test**: Verify death at correct point in turn flow
10. **Double-trigger regression tests**: Ensure no power fires twice

### MEDIUM (completeness)
11. **Conjure Blade upgraded hit count**: Should be X+1 not X
12. **Simmering Fury split powers**: WrathNextTurn + DrawNextTurn separately
13. **WindmillStrike upgrade bonus**: +5 per retain instead of +4
14. **Event choice outcome tests**: For all 50+ events
15. **Monster AI pattern tests**: For all 67 monsters

### Parity Tests (seed-deterministic)
16. **Full game seed replay**: Play known seed, verify exact floor-by-floor outcomes
17. **Combat seed replay**: Known enemy, known deck, verify exact damage/HP
18. **Card reward seed replay**: Known floor, verify exact card reward options
19. **Map generation seed replay**: Known seed, verify exact map layout

---

## Remaining Implementation Gaps

### Powers (38 missing, sorted by Watcher priority)
**Watcher-specific (7):**
- NoSkillsPower (blocks skill plays, 1 turn)
- VaultPower (wrong implementation — currently "extra turn" not damage)
- CannotChangeStancePower (partial — removal but not stance blocking)
- SimmeringFuryPower (combined status, should be 2 separate powers)
- EstablishmentPower (no-op placeholder, needs per-card cost tracking)
- MarkPower (Pressure Points — partially handled inline)
- OmegaPower (handled inline, not in registry)

**Shared/Enemy (31):**
- Focus, Electro, Draw, DrawReduction, ModeShift, Retain, etc.
- Most are non-Watcher or enemy-only, lower priority

### Relics (45 missing)
**HIGH priority for Watcher:**
- Chemical X (+2 to X cost), Omamori (negate curse), Question Card (+1 card choice)
- Astrolabe, Calling Bell, Pandora's Box (deck modification on pickup)
- Busted Crown, Snecko Eye (card reward/cost modification)

**MEDIUM:**
- 26 relics with `onEquip` hooks (one-time pickup effects)
- Rest site relics (Dream Catcher, Regal Pillow, Girya, Peace Pipe, Shovel)

### Events (estimated 40% coverage)
- ~20 events with correct handlers
- ~30 events with missing or partial handlers
- Priority: Act 1 events (appear most in short runs)

### Monsters (estimated 90% parity)
- All 66 base enemies defined with correct HP/moves
- AI patterns implemented for most
- Known gaps: some ascension-specific behavior, split/summon edge cases

---

## Recommended Next Steps (Priority Order)

1. **Fix block reset ordering** — Move to after start-of-turn triggers for Java parity
2. **Write the 10 CRITICAL/HIGH tests** listed above
3. **Implement Chemical X relic** — Affects X-cost card evaluation (Conjure Blade, Collect, etc.)
4. **Fix VaultPower** — Currently wrong (extra turn vs damage)
5. **Fix end-of-turn ordering** — Discard should happen after end-of-turn powers
6. **Port event handlers** for top 20 most-common events
7. **Add NoSkillsPower** for Fasting/Blasphemy interaction
8. **Run full benchmark** to measure impact of all fixes on agent performance
