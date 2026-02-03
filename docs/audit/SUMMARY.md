# Audit Summary: Final Status

Audit conducted 2026-02-02. All critical/high bugs fixed in Rounds 1-2.

## Final Test Results

```
3498 passed, 5 skipped, 0 xfailed, 0 failures
Coverage: 67.78%
```

## Bugs Fixed (48 total)

### Round 1: Critical Bugs (7 fixed)
| Bug | File | Fix |
|-----|------|-----|
| Vuln rounding | combat_engine.py | Single-floor chain, pass vuln=True to calculate_damage() |
| Torii before block | calc/damage.py | Restructured to apply Torii post-block |
| Beggar event | events.py | Corrected to: pay 75g + remove card, or leave |
| Nest event | events.py | Split into 2 options, added 6 HP cost |
| AwakenedOne rebirth | enemies.py | Selective power clearing (keep Str/Regen) |
| Buffer timing | combat_engine.py | Check Buffer before block subtraction |
| Plated Armor | combat_engine.py | Only decrement on NORMAL enemy damage |

### Round 1: High Bugs (41 fixed)
- Event rounding (Big Fish, Golden Idol, Shining Light, Forgotten Altar) - per-event rounding modes
- Relic ID mismatches (23 camelCase -> spaced IDs)
- Counter-per-turn relics reset at turn start
- Blur decrement logic implemented
- Lantern/Horn Cleat turn counter fixed
- Blood Vial moved to combat start
- Card.copy() preserves upgrade flags
- 9 Watcher card data fixes (Halt, Worship, DevaForm, etc.)
- Divinity exit timing (end-of-turn -> start-of-next-turn)
- 6 turn-based powers added to registry
- Regen no longer decrements

### Round 2: Remaining Fixes
| Category | Items Fixed |
|----------|-------------|
| Last 4 xfails | Dex/Str at 0, IntangiblePlayer, Duality, StoneCalendar |
| Damage relics | Self-Forming Clay, Boot, Champion's Belt, Red Skull, Fossilized Helix, Runic Cube, Orichalcum |
| Potions | Sacred Bark support, Fairy, Regen, Ancient, Essence of Steel, Fruit Juice |
| Rest site | Peace Pipe, Coffee Dripper, Eternal Feather, Dream Catcher |

## Remaining Work (Low Priority)

### Partially Implemented Potions
- Duplication Potion (status set, needs card play trigger)
- Smoke Bomb (needs escape logic)
- Distilled Chaos (needs auto-play)
- Liquid Memories (needs card selection)
- Entropic Brew (needs potion generation)

### Skipped Tests (5)
- 3 random-seed dependent game loop tests
- 1 RunState.get_starter_relic not implemented
- 1 MapGenerator API difference

## Files Most Changed

1. `packages/engine/combat_engine.py` - damage pipeline, potion effects, relic triggers
2. `packages/engine/handlers/combat.py` - relic ID fixes, counter resets, new relic hooks
3. `packages/engine/content/events.py` - event mechanics, per-event rounding
4. `packages/engine/content/enemies.py` - boss mechanics
5. `packages/engine/content/cards.py` - card data, copy() method
6. `packages/engine/content/powers.py` - 6 new powers, should_remove fix
7. `packages/engine/game.py` - rest site relics
8. `packages/engine/content/stances.py` - Divinity timing

## Verification Commands

```bash
uv run pytest tests/ -q                           # All tests pass
uv run pytest tests/ --cov=packages/engine -q     # Coverage 67.78%
uv run pytest tests/test_parity.py -q             # Parity tests pass
uv run pytest tests/test_rng_audit.py -q          # RNG audit clean
```
