# Watcher Skill & Power Card Audit

Audit of Python engine card definitions vs decompiled Java source.

## Summary

- **Cards audited**: 27 Skills + 11 Powers = 38 cards
- **Bugs found**: 11
- **Correct**: 27
- **Tests**: 64 passed, 11 xfail (documenting bugs)

## Bugs Found

### BUG-1: Halt Wrath bonus incorrect when upgraded
- **File**: `packages/engine/effects/cards.py:429`
- **Issue**: Upgraded Halt in Wrath gives 4+9=13 block. Java gives 14 (baseBlock 4 + wrath bonus 10).
- **Java**: `baseMagicNumber = baseBlock + 6 + timesUpgraded*4`. Upgraded: 4+6+4=14.
- **Python**: `extra = 9 if ctx.is_upgraded else 6` -- should be `10 if upgraded else 6`.
- **Fix**: Change Wrath extra from 9 to 10 when upgraded.

### BUG-2: Crescendo comment wrong about exhaust on upgrade
- **File**: `packages/engine/content/cards.py:289`
- **Issue**: Comment says "Upgraded: not exhaust" but Java upgrade only reduces cost to 0, exhaust stays true.
- **Java**: `upgradeBaseCost(0)` only. No exhaust change.
- **Impact**: If code respects the comment and sets upgrade_exhaust=False, this is a gameplay bug. Currently the Card definition has no `upgrade_exhaust` field set, so data is correct -- only comment is wrong.

### BUG-3: Worship has retain=True at base (should be upgrade only)
- **File**: `packages/engine/content/cards.py:498`
- **Issue**: Python has `retain=True` at base. Java only sets `selfRetain = true` on upgrade.
- **Java**: Base has no retain. `upgrade()` sets `this.selfRetain = true`.
- **Fix**: Change to `retain=False, upgrade_retain=True`.

### BUG-4: Sanctity draw amount increases on upgrade (should not)
- **File**: `packages/engine/effects/cards.py:421`
- **Issue**: Effect gives draw 3 when upgraded, draw 2 when not. Java never upgrades magicNumber.
- **Java**: `baseMagicNumber=2`, upgrade only calls `upgradeBlock(3)`. Draw is always 2.
- **Fix**: Remove the `3 if ctx.is_upgraded else` logic, always draw 2.

### BUG-5: DevaForm missing upgrade_ethereal=False
- **File**: `packages/engine/content/cards.py:664`
- **Issue**: DevaForm is Ethereal at base. Java upgrade removes Ethereal. Python has no `upgrade_ethereal=False`.
- **Java**: `this.isEthereal = false` on upgrade.
- **Fix**: Add `upgrade_ethereal=False` to DEVA_FORM card definition.

### BUG-6: Establishment missing upgrade_innate=True
- **File**: `packages/engine/content/cards.py:519`
- **Issue**: Java Establishment upgrade sets `isInnate = true`. Python has no `upgrade_innate=True`.
- **Fix**: Add `upgrade_innate=True` to ESTABLISHMENT card definition.

### BUG-7: BattleHymn missing upgrade_innate=True
- **File**: `packages/engine/content/cards.py:513`
- **Issue**: Java BattleHymn upgrade sets `isInnate = true`. Python has no `upgrade_innate=True`.
- **Fix**: Add `upgrade_innate=True` to BATTLE_HYMN card definition.

### BUG-8: Card.copy() drops upgrade flag fields
- **File**: `packages/engine/content/cards.py:164-177`
- **Issue**: `Card.copy()` does not copy `upgrade_retain`, `upgrade_innate`, `upgrade_exhaust`, `upgrade_ethereal` fields. All are lost, defaulting to None.
- **Impact**: `get_card()` calls `copy()`, so all upgrade flags are lost. Alpha's `upgrade_innate=True` and Blasphemy's `upgrade_retain=True` are defined on the constants but never reach consumers.
- **Fix**: Add these fields to the `copy()` method.

### BUG-9: DevaForm missing base_magic=1
- **File**: `packages/engine/content/cards.py:664`
- **Issue**: Java has `baseMagicNumber=1`. Python has no base_magic (defaults -1).
- **Fix**: Add `base_magic=1` to DEVA_FORM.

### BUG-10: Sanctity missing base_magic=2
- **File**: `packages/engine/content/cards.py:472`
- **Issue**: Java has `baseMagicNumber=2`. Python has no base_magic, effect hardcodes draw amount.
- **Fix**: Add `base_magic=2` and use it in the effect instead of hardcoded values.

### BUG-11: ForeignInfluence has spurious upgrade_magic=1
- **File**: `packages/engine/content/cards.py:625`
- **Issue**: Java ForeignInfluence has no baseMagicNumber. Upgrade only changes description. Python has `upgrade_magic=1` which has no Java basis.
- **Fix**: Remove `upgrade_magic=1`.

## Additional Notes

### Devotion docstring mismatch
- **File**: `packages/engine/effects/cards.py:95`
- **Issue**: Docstring says "3/4 Mantra" but Java base is 2, upgraded 3 (matches Python data, docstring is wrong).

### ConjureBlade upgrade semantics
- **Java**: Upgraded adds +1 to energyOnUse (effectively X+1 hits on Expunger).
- **Python**: No explicit upgrade behavior beyond what's in the effect. Verify that executor handles this.

### Wish values
- **Java**: Uses separate option cards (BecomeAlmighty, FameAndFortune, LiveForever) with their own base/upgraded values. The card itself stores baseDamage=3 (Strength), baseMagicNumber=25 (Gold?), baseBlock=6 (Plated Armor). Upgrades: +1/+5/+2.
- **Python**: Only stores `upgrade_magic=1` and defers to effect. Actual option values need separate verification.

## Verified Correct Cards

| Card | ID | Type | Values Match | Upgrade Match |
|------|----|------|:---:|:---:|
| Evaluate | Evaluate | Skill | Y | Y (block +4) |
| Protect | Protect | Skill | Y | Y (block +4) |
| Third Eye | ThirdEye | Skill | Y | Y (block +2, magic +2) |
| Vigilance | Vigilance | Skill | Y | Y (block +4) |
| Empty Body | EmptyBody | Skill | Y | Y (block +3) |
| Empty Mind | EmptyMind | Skill | Y | Y (magic +1) |
| Tranquility | ClearTheMind | Skill | Y | Y (cost 0) |
| Crescendo | Crescendo | Skill | Y | Y (cost 0) |
| Inner Peace | InnerPeace | Skill | Y | Y (magic +1) |
| Perseverance | Perseverance | Skill | Y | Y (block +2, magic +1) |
| Swivel | Swivel | Skill | Y | Y (block +3) |
| Meditate | Meditate | Skill | Y | Y (magic +1) |
| Collect | Collect | Skill | Y | Y (description only) |
| Deceive Reality | DeceiveReality | Skill | Y | Y (block +3) |
| Judgement | Judgement | Skill | Y | Y (magic +10) |
| Alpha | Alpha | Skill | Y | Y (innate) |
| Blasphemy | Blasphemy | Skill | Y | Y (retain) |
| Omniscience | Omniscience | Skill | Y | Y (cost 3) |
| Scrawl | Scrawl | Skill | Y | Y (cost 0) |
| Spirit Shield | SpiritShield | Skill | Y | Y (magic +1) |
| Vault | Vault | Skill | Y | Y (cost 2) |
| Foreign Influence | ForeignInfluence | Skill | Y | Y (desc only) |
| Wave of the Hand | WaveOfTheHand | Skill | Y | Y (magic +1) |
| Prostrate | Prostrate | Skill | Y | Y (magic +1) |
| Rushdown | Adaptation | Power | Y | Y (cost 0) |
| Mental Fortress | MentalFortress | Power | Y | Y (magic +2) |
| Talk to the Hand | TalkToTheHand | Attack | Y | Y (dmg +2, magic +1) |
| Like Water | LikeWater | Power | Y | Y (magic +2) |
| Devotion | Devotion | Power | Y | Y (magic +1) |
| Foresight | Wireheading | Power | Y | Y (magic +1) |
| Study | Study | Power | Y | Y (cost 1) |
