# Parity Test Seeds

Seeds specifically chosen to test edge cases in our RNG implementation.

## Quick Test Seeds

| Seed | Character | Test Case | What to Verify |
|------|-----------|-----------|----------------|
| `1234567890` | WATCHER | Basic sanity | Card rewards floors 1-3 |
| `AAAAAAAAAA1` | WATCHER | Simple alphanumeric | Monster list, boss |
| `111111111111111` | WATCHER | Repeated digits | All RNG streams |

## Edge Case Seeds

### Card Reward Edge Cases

| Seed | Test Case | Expected Behavior |
|------|-----------|-------------------|
| `3RARECARDS` | Early rare pity | Should see rare card by floor 3-4 |
| `SHOPFLOOR2` | Early shop | Shop on floor 2, verify shop cardRng consumption (~12 calls) |
| `COLORLESS1` | Colorless pity | Should trigger colorless card around floor 10-15 |

### Neow Edge Cases

| Seed | Neow Choice | cardRng Impact |
|------|-------------|----------------|
| `NEOWGOLD` | HUNDRED_GOLD | 0 cardRng calls |
| `NEOWRELIC` | RANDOM_COMMON_RELIC | 0 cardRng calls |
| `NEOWCOLOR` | RANDOM_COLORLESS | 3+ cardRng calls |
| `NEOWCURSE` | CURSE drawback | 1 cardRng call |
| `BOSSSWAP1` | Boss swap (Calling Bell) | ~3 cardRng calls |

### Act Transition Edge Cases

| Seed | Test Case | What to Check |
|------|-----------|---------------|
| `ACT1END` | Act 1 boss | cardRng counter snapping (1-249 → 250) |
| `ACT2START` | Act 2 first floor | Counter should be at 250+ |
| `ACT3SNAP` | Act 3 transition | Counter snapping (501-749 → 750) |

### Encounter Edge Cases

| Seed | Test Case | Exclusion Logic |
|------|-----------|-----------------|
| `SHAPES123` | 3 Shapes → 4 Shapes | 4 Shapes excluded after 3 Shapes |
| `DARKLING3` | 3 Darklings | Self-exclusion in Act 3 |
| `ORBWALK` | Orb Walker | Orb Walker exclusion |

### Event Edge Cases

| Seed | Test Case | What to Check |
|------|-----------|---------------|
| `LIBRARY1` | The Library event | ~20 cardRng calls |
| `WHEELSPIN` | Wheel of Change | Event RNG vs Card RNG |
| `SHRINE01` | Shrine event | Shared event pool |

## Verified Seeds (from VOD extraction)

These seeds are from verified Merl runs with known outcomes:

| Seed | Run | Notes |
|------|-----|-------|
| `227QYN385T72G` | Merl Run 7 | Win, good for full run test |
| `47XSIW6LZ7YVW` | Merl Run 8 | Win, good for full run test |
| `39L652HYDJ137` | Merl Run 9 | Win, good for full run test |

## How to Test

### Manual Test (in-game)
```bash
# Start STS with seed
./scripts/dev/launch_sts.sh

# In game: Custom Run → Set Seed → Enter seed
# Play through and compare predictions
```

### Automated Test
```bash
# Run parity checker on specific seed
uv run python scripts/dev/test_parity.py --seed "1234567890" --character WATCHER

# Run all edge case tests
uv run python scripts/dev/test_parity.py --all-edge-cases
```

### Save File Test
```bash
# Start a run with known seed, make Neow choice
# Then read save and compare:
./scripts/dev/save.sh

# Compare monster_list, card_seed_count, etc. with predictions
```

## Parity Checklist

For each test seed, verify:

- [ ] Monster list matches (first 5-10 encounters)
- [ ] Elite list matches (first 3-4 elites)
- [ ] Boss matches
- [ ] Card rewards match (test 3-5 floors)
- [ ] card_seed_count matches after each floor
- [ ] Relic pools match (common/uncommon/rare order)
- [ ] Shop inventory matches (if shop visited)
- [ ] Event outcomes match (if event visited)

## Known Issues to Watch

1. **Neow cardRng consumption** - Different Neow choices consume different amounts
2. **Shop visits** - Each shop consumes ~12 cardRng calls
3. **The Library event** - Consumes ~20 cardRng calls
4. **Boss relic swap** - Calling Bell can consume cardRng
5. **Act transitions** - Counter snapping must be exact
