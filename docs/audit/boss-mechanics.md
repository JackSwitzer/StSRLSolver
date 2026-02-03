# Boss Mechanics Audit: Python Engine vs Decompiled Java

Audit date: 2026-02-02
Auditor: Claude Opus 4.5

## Summary

| Boss | Status | Bugs Found |
|------|--------|------------|
| Champ | PASS | None |
| TheCollector | PASS | None |
| BronzeAutomaton | PASS | None |
| AwakenedOne | **FAIL** | Rebirth clears ALL powers instead of only debuffs+Curiosity+Unawakened+Shackled |
| TimeEater | PASS | None |
| CorruptHeart | PASS (conditional) | Buff strength depends on executor handling clear_negative_strength |
| SpireShield | **FAIL** | Smash block below A18 uses base damage instead of damage output |
| SpireSpear | PASS | None |

## Detailed Findings

### Champ (Act 2 Boss)
**File**: `packages/engine/content/enemies.py:3937`
**Java**: `decompiled/java-src/.../monsters/city/Champ.java`

- HP: 420 / A9: 440 -- MATCH
- Damage values across all ascension tiers -- MATCH
- Move IDs (1-7) -- MATCH
- Phase transition at <50% HP (`currentHealth < maxHealth / 2`) -- MATCH
- Anger gives `strAmt * 3` Strength -- MATCH
- Execute uses `lastMove` AND `lastMoveBefore` to prevent 3x repeat -- MATCH
- Taunt every 4th turn (numTurns==4), resets counter, only in Phase 1 -- MATCH
- Defensive Stance: A19 threshold 30, else 15; max 2 uses -- MATCH
- Gloat: 30% threshold, blocked after Gloat OR Defensive Stance -- MATCH
- Face Slap: 55% threshold, no repeat -- MATCH
- Heavy Slash fallback with Face Slap if repeated -- MATCH

### TheCollector (Act 2 Boss)
**File**: `packages/engine/content/enemies.py:4139`
**Java**: `decompiled/java-src/.../monsters/city/TheCollector.java`

- HP: 282 / A9: 300 -- MATCH
- Damage/Strength values across ascension tiers -- MATCH
- A19 megaDebuffAmt = 5 (not 3) -- MATCH
- Spawn on first turn -- MATCH
- Mega Debuff after turnsTaken >= 3 -- MATCH (both increment after move execution)
- Revive: 25% chance when minion dead, no repeat -- MATCH
- Fireball: 70% chance, no 2x repeat (`lastTwoMoves`) -- MATCH
- Buff fallback: no repeat, else Fireball -- MATCH
- A19 Buff block is `blockAmt + 5` (18 + 5 = 23) -- MATCH

### BronzeAutomaton (Act 2 Boss)
**File**: `packages/engine/content/enemies.py:4291`
**Java**: `decompiled/java-src/.../monsters/city/BronzeAutomaton.java`

- HP: 300 / A9: 320 -- MATCH
- Damage values: A4+ flail=8/beam=50, else 7/45 -- MATCH
- Block: A9+ 12, else 9; Strength: A4+ 4, else 3 -- MATCH
- Pre-battle: 3 Artifact -- MATCH
- First turn: Spawn Orbs -- MATCH
- Hyper Beam at numTurns==4, resets to 0 -- MATCH
- After Hyper Beam: A19+ Boost, else Stunned -- MATCH
- After Stunned/Boost/Spawn: Flail -- MATCH
- Default: Boost -- MATCH
- numTurns only increments in final else branch -- MATCH
- Expected pattern: Spawn, Flail, Boost, Flail, Boost, HyperBeam, ... -- MATCH

### AwakenedOne (Act 3 Boss)
**File**: `packages/engine/content/enemies.py:4435`
**Java**: `decompiled/java-src/.../monsters/beyond/AwakenedOne.java`

- HP: 300 / A9: 320 (both phases same) -- MATCH
- Damage values (all fixed, not ascension-dependent) -- MATCH
- Pre-battle: Regenerate 10 (A19: 15), Curiosity 1 (A19: 2), Unawakened, A4+ Str 2 -- MATCH
- Phase 1 AI: first turn Slash, 25% Soul Strike (no repeat), 75% Slash (no 2x repeat) -- MATCH
- Phase 2 AI: first turn Dark Echo, 50/50 Sludge/Tackle (no 2x repeat each) -- MATCH

**BUG: Rebirth power handling**
- Java (line 291-296): On rebirth, iterates powers and removes ONLY those where `p.type == DEBUFF` OR `p.ID.equals("Curiosity")` OR `p.ID.equals("Unawakened")` OR `p.ID.equals("Shackled")`. Keeps all other buffs (Strength from A4+, Regenerate).
- Python (line 4555): `self.state.powers = {}` -- clears ALL powers including Strength and Regenerate.
- **Impact**: At A4+, the Awakened One should retain its +2 Strength through rebirth. At A19, it should retain Regenerate 15. Python incorrectly strips these.
- **Fix**: Replace `self.state.powers = {}` with selective removal of debuffs + Curiosity + Unawakened + Shackled.

### TimeEater (Act 3 Boss)
**File**: `packages/engine/content/enemies.py:4609`
**Java**: `decompiled/java-src/.../monsters/beyond/TimeEater.java`

- HP: 456 / A9: 480 -- MATCH
- Damage: A4+ reverb=8/slam=32, else 7/26 -- MATCH
- Pre-battle: TimeWarp power -- MATCH
- Haste: triggers at `currentHealth < maxHealth / 2`, heals to half HP -- MATCH
- Haste A19: also gains block equal to headSlamDmg (32) -- MATCH
- Haste removes debuffs AND Shackled -- MATCH (Java line 138-139)
- AI recursion: Reverberate fallback uses aiRng.random(50,99), Ripple fallback uses aiRng.random(74) [0-74] -- MATCH
- Head Slam fallback: 66% Reverberate, else Ripple -- MATCH

### CorruptHeart (Act 4 Boss)
**File**: `packages/engine/content/enemies.py:5066`
**Java**: `decompiled/java-src/.../monsters/ending/CorruptHeart.java`

- HP: 750 / A9: 800 -- MATCH
- Damage: A4+ echo=45/blood=2x15, else 40/2x12 -- MATCH
- Pre-battle: Invincible 300 (A19: 200), Beat of Death 1 (A19: 2) -- MATCH
- First turn: Debilitate (2 Vuln, 2 Weak, 2 Frail + 5 status cards) -- MATCH
- 3-turn cycle: attack/attack/buff -- MATCH
- Cycle 0: 50% Blood Shots / Echo -- MATCH
- Cycle 1: prefer Echo, fallback Blood Shots -- MATCH
- Cycle 2: Buff with cycling effects -- MATCH
- Buff Strength: `additionalAmount + 2` where additionalAmount = abs(negative Strength) -- MATCH semantically
- Buff cycle 0: +2 Artifact -- MATCH
- Buff cycle 1: +1 Beat of Death -- MATCH
- Buff cycle 2: Painful Stabs -- MATCH
- Buff cycle 3: +10 Strength (additional) -- MATCH (Python uses 12 = 2 base + 10)
- Buff cycle 4+: +50 Strength (additional) -- MATCH (Python uses 52 = 2 base + 50)

**Note**: The Python combines base Strength (2) with the bonus in a single value. The Java applies them as two separate StrengthPower applications. The executor must handle `clear_negative_strength` BEFORE applying the combined Strength amount for correct behavior.

### SpireShield (Act 4 Elite)
**File**: `packages/engine/content/enemies.py:4906`
**Java**: `decompiled/java-src/.../monsters/ending/SpireShield.java`

- HP: 110 / A8: 125 -- MATCH
- Damage: A3+ bash=14/smash=38, else 12/34 -- MATCH
- Pre-battle: Surrounded on player, Artifact 1 (A18: 2) -- MATCH
- 3-turn cycle move pattern -- MATCH
- Fortify: 30 block to ALL monsters -- MATCH
- Bash: -1 Strength OR -1 Focus (50% if player has orbs) -- MATCH

**BUG: Smash block amount below A18**
- Java (line 100): Below A18, Smash grants block equal to `this.damage.get(1).output` -- this is the CALCULATED damage output (affected by Strength, Weak, Vulnerable, etc.), not the base damage.
- Python (line 4980): Below A18, uses `dmg["smash"]` which is the base damage value (34 or 38).
- **Impact**: If SpireShield has gained Strength (e.g., from SpireSpear's Piercer buff), the Smash block should scale with it. Python block stays at base.
- **Fix**: Track actual damage output for Smash and use that for block below A18.

### SpireSpear (Act 4 Elite)
**File**: `packages/engine/content/enemies.py:4989`
**Java**: `decompiled/java-src/.../monsters/ending/SpireSpear.java`

- HP: 160 / A8: 180 -- MATCH
- Damage: A3+ strike=6/skewer=10x4, else 5/10x3 -- MATCH
- Pre-battle: Artifact 1 (A18: 2) -- MATCH
- 3-turn cycle pattern -- MATCH
- Burn Strike: 2 Burns to discard (A18: to draw pile) -- MATCH
- Piercer: +2 Strength to ALL monsters -- MATCH

## Bugs to Fix (Priority Order)

1. **AwakenedOne rebirth power clearing** (HIGH) -- Incorrectly clears Strength and Regenerate buffs on rebirth. Should only remove debuffs, Curiosity, Unawakened, and Shackled.

2. **SpireShield Smash block scaling** (MEDIUM) -- Below A18, Smash block should use calculated damage output (affected by Strength), not base damage value.
