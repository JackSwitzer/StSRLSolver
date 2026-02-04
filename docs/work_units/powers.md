# Power Trigger Work Units

## Scope summary
- Close gaps between `POWER_DATA` hook definitions and registry/engine triggers.
- Add missing hook dispatch points (ex: `onAfterUseCard`, `modifyBlockLast`,
  `canPlayCard`, `onVictory`, `duringTurn`) where power logic depends on them.
- Implement missing power handlers (player, enemy, colorless) with correct
  timing and stack rules.
- Extend targeted tests for each new trigger (no full-suite runs).
- Model-facing actions only (no UI); see `docs/work_units/granular-actions.md`.

## Missing power triggers (grouped)
### System / Shared
- Slow (atDamageReceive, onAfterUseCard, atEndOfRound reset)
- Lock-On (orb damage modifier, atEndOfRound decrement)
- NoBlockPower (modifyBlockLast)
- Entangled (canPlayCard + remove at end of turn)
- No Draw (skip draw + remove at end of turn)
- Draw Reduction (onInitialApplication/onRemove + atEndOfRound decrement)
- Draw (onInitialApplication/onRemove)
- Artifact (onSpecificTrigger consume when debuff blocked)
- IntangiblePlayer (atEndOfRound decrement)
- Double Damage (atDamageGive + atEndOfRound decrement)
- Pen Nib (atDamageGive + onUseCard remove)
- Blur (atEndOfRound decrement)
- Barricade (block retention)
- Repair (onVictory heal)
- Thorns (onAttacked hook missing; currently wired to onAttack)

### Ironclad
- Corruption (onCardDraw: Skills cost 0, onUseCard: Skills exhaust)

### Silent
- Accuracy (passive Shiv baseDamage boost)
- Thousand Cuts (onAfterUseCard hook missing; currently onUseCard)

### Defect
- Bias (atStartOfTurn lose Focus timing)
- Storm (onUseCard: Power -> channel Lightning)
- Static Discharge (onAttacked: channel Lightning)
- Electro (passive: Lightning hits all)
- Focus / Lock-On (orb modifiers; see System)

### Watcher
- Mantra (onStack -> enter Divinity at 10)
- BlockReturnPower (onAttacked: gain block when attacking marked enemy)
- EstablishmentPower (atEndOfTurn: reduce retained card costs)
- WaveOfTheHandPower (atEndOfRound remove)
- FreeAttackPower (onUseCard: decrement on ATTACK)
- CannotChangeStancePower (canPlayCard/stance change lock + remove at end of turn)

### Boss/Enemy
- Beat of Death (onAfterUseCard damage)
- Curiosity (onUseCard: Power -> gain Strength)
- Time Warp (onAfterUseCard counter + end turn + gain Strength)
- Invincible (onAttackedToChangeDamage + atStartOfTurn reset)
- Angry (onAttacked: gain Strength)
- GrowthPower/Ritual (atEndOfRound, skip first)
- Fading (duringTurn decrement/suicide)
- Thievery (onAttack: steal gold)
- Mode Shift / Split / Life Link (passive systems)

### Colorless/Special
- Panache (atStartOfTurn reset counter)
- Double Tap (onUseCard: ATTACK replay + decrement)
- Burst (onUseCard: SKILL replay + decrement)
- Echo Form (atStartOfTurn: first card doubles)
- Retain Cards (atEndOfTurn: select retain count)
- Equilibrium (passive retain hand + atEndOfRound decrement)

## Suggested task batches + acceptance criteria
1. **Core hook dispatch + shared debuffs**
   - Add dispatch for `onAfterUseCard`, `modifyBlockLast`, `canPlayCard`,
     `onVictory`, `duringTurn` in combat flows.
   - Implement System/Shared list items that do not depend on orbs.
   - Acceptance: registry handlers added; new tests in
     `tests/test_power_registry_integration.py` or
     `tests/test_power_edge_cases.py` cover Slow, Entangled, No Draw,
     Draw Reduction, Double Damage, Pen Nib, Blur, Repair.

2. **Ironclad/Silent trigger fixes**
   - Add Corruption handlers.
   - Add Accuracy passive + move Thousand Cuts to `onAfterUseCard`.
   - Acceptance: hook names match `POWER_DATA`; targeted tests verify behavior.

3. **Watcher triggers**
   - Mantra on-stack divinity, BlockReturnPower, FreeAttackPower,
     CannotChangeStance removal, WaveOfTheHand removal, Establishment end-turn.
   - Acceptance: stance/divinity transitions and block return verified in tests.

4. **Defect/orb-dependent powers**
   - Focus, Lock-On, Storm, Static Discharge, Electro, Bias timing.
   - Acceptance: orb damage path uses modifiers; unit tests isolate each power.

5. **Boss/Enemy powers**
   - Beat of Death, Time Warp, Invincible, Angry, GrowthPower, Fading, Thievery,
     plus passive hooks for Mode Shift/Split/Life Link.
   - Acceptance: enemy turn loops call the new hooks; tests simulate boss turns.

## Files to touch
- `packages/engine/registry/powers.py`
- `packages/engine/registry/__init__.py`
- `packages/engine/combat_engine.py`
- `packages/engine/handlers/combat.py`
- `packages/engine/content/powers.py`
- `packages/engine/calc/damage.py`
- `packages/engine/effects/cards.py`
- `packages/engine/effects/registry.py`
- `tests/test_power_registry_integration.py`
- `tests/test_power_edge_cases.py`
