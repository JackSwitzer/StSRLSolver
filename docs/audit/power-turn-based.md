# Power Turn-Based Trigger Audit

Audit of Python engine turn-start/end power implementations against decompiled Java source.

## Methodology

Compared Java files at `decompiled/java-src/com/megacrit/cardcrawl/powers/` against:
- `packages/engine/content/powers.py` (POWER_DATA registry + PowerManager)
- `packages/engine/combat_engine.py` (actual turn processing)

## Summary of Findings

| Power | Java Hook | Python POWER_DATA | Python combat_engine | Status |
|-------|-----------|-------------------|---------------------|--------|
| PoisonPower | `atStartOfTurn()` - HP_LOSS then decrement | Documented correctly | Implemented in `_start_player_turn` + `_do_enemy_turns` | OK |
| RegenPower | `atEndOfTurn(isPlayer)` - heal amount, does NOT decrement itself | Not in POWER_DATA | Implemented in `end_turn` but decrements (BUG) | BUG |
| CombustPower | `atEndOfTurn(isPlayer)` - lose hpLoss HP, deal amount THORNS to all enemies | Not in POWER_DATA | Not implemented | MISSING |
| BrutalityPower | `atStartOfTurnPostDraw()` - draw amount, lose amount HP | Not in POWER_DATA | Not implemented | MISSING |
| DarkEmbracePower | `onExhaust(card)` - draw amount | In POWER_DATA correctly | Not verified in combat_engine exhaust path | VERIFY |
| EvolvePower | `onCardDraw(card)` - if STATUS, draw amount (respects No Draw) | In POWER_DATA correctly | Not verified in combat_engine draw path | VERIFY |
| FeelNoPainPower | `onExhaust(card)` - gain amount block | Not in POWER_DATA | Not implemented | MISSING |
| FireBreathingPower | `onCardDraw(card)` - if STATUS or CURSE, deal amount THORNS to all | Not in POWER_DATA | Not implemented | MISSING |
| RupturePower | `wasHPLost(info, damage)` - if self-damage, gain amount Str | In POWER_DATA correctly | Not verified in damage path | VERIFY |
| ThousandCutsPower | `onAfterCardPlayed(card)` - deal amount THORNS to all | Not in POWER_DATA | Not implemented | MISSING |

## Critical Bugs

### 1. RegenPower: Incorrect Decrement Behavior
**Java**: `RegenPower.atEndOfTurn()` heals `amount` HP. It does NOT decrement `amount`. The power stacks and persists.
**Python**: `combat_engine.py:411-419` decrements regen by 1 each turn and removes at 0. This is WRONG -- Regen in the base game does not decrement. (Note: some sources of Regen like the relic "Meat on the Bone" are different; but RegenPower itself from the card does not self-decrement. The RegenAction handles the heal only.)

### 2. CombustPower: Completely Missing
**Java**: `atEndOfTurn(isPlayer)` - owner loses `hpLoss` HP (starts at 1, increments on stack), then deals `amount` THORNS damage to all enemies. Has a custom `stackPower` that increments hpLoss.
**Python**: No implementation anywhere.

### 3. BrutalityPower: Completely Missing
**Java**: `atStartOfTurnPostDraw()` - draw `amount` cards, then lose `amount` HP.
**Python**: No implementation anywhere.

### 4. FeelNoPainPower: Completely Missing
**Java**: `onExhaust(card)` - gain `amount` block.
**Python**: No implementation anywhere.

### 5. FireBreathingPower: Completely Missing
**Java**: `onCardDraw(card)` - if card is STATUS or CURSE type, deal `amount` THORNS to all enemies.
**Python**: No implementation anywhere.

### 6. ThousandCutsPower: Completely Missing
**Java**: `onAfterCardPlayed(card)` - deal `amount` THORNS to all enemies. Note: uses `onAfterCardPlayed` NOT `onUseCard`.
**Python**: No implementation anywhere.

## Hook Ordering Notes (from Java)

The Java turn lifecycle is:
1. `atStartOfTurn()` - Poison ticks here (before draw)
2. Player draws cards
3. `atStartOfTurnPostDraw()` - Brutality, DemonForm, NoxiousFumes, Devotion
4. Player plays cards (card hooks fire here)
5. `atEndOfTurnPreEndTurnCards(isPlayer)` - Metallicize, PlatedArmor block
6. Discard hand
7. `atEndOfTurn(isPlayer)` - Regen heal, Combust damage, Constricted, Omega
8. `atEndOfRound()` - Decrement turn-based debuffs (Weak, Vuln, Frail)

### Python Ordering Issues
- Metallicize/PlatedArmor are in `end_turn()` AFTER discard -- Java fires them BEFORE discard via `atEndOfTurnPreEndTurnCards`. Functionally equivalent for block but ordering matters if interactions exist.
- Regen is processed before enemy turns in Python, which matches Java's `atEndOfTurn(isPlayer=true)`.
- Poison correctly fires at start of turn before draw.

## Rushdown Power Note
Python POWER_DATA says Rushdown triggers "when entering Wrath" -- this is correct per the Java `onChangeStance` hook.
