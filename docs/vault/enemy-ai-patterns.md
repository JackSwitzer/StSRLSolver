# Enemy AI Patterns - Slay the Spire

Complete AI pattern documentation extracted from decompiled source code. This document contains the exact move selection logic, HP values, damage numbers, and ascension scaling for all enemies.

## Table of Contents
- [Act 1 - Exordium](#act-1---exordium)
  - [Basic Enemies](#exordium-basic-enemies)
  - [Elites](#exordium-elites)
  - [Bosses](#exordium-bosses)
- [Act 2 - The City](#act-2---the-city)
  - [Elites](#city-elites)
  - [Bosses](#city-bosses)
- [Act 3 - Beyond](#act-3---beyond)
  - [Elites](#beyond-elites)
  - [Bosses](#beyond-bosses)
- [Ending](#ending)
- [AI Pattern Categories](#ai-pattern-categories)

---

## Act 1 - Exordium

### Exordium Basic Enemies

#### Jaw Worm
**ID:** `JawWorm`
**Type:** Normal

| Stat | Base | A7+ |
|------|------|-----|
| HP | 40-44 | 42-46 |

**Moves:**
| Move | Byte | Intent | Base Damage | A2+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Chomp | 1 | ATTACK | 11 | 12 | - |
| Bellow | 2 | DEFEND_BUFF | - | - | +3 STR, 6 Block (A17+: +5 STR, 9 Block) |
| Thrash | 3 | ATTACK_DEFEND | 7 | 7 | +5 Block |

**AI Pattern (getMove):**
```
First turn: Always CHOMP
Subsequent turns (num = 0-99):
  if num < 25:
    if lastMove(CHOMP):
      56.25% -> BELLOW
      43.75% -> THRASH
    else:
      -> CHOMP
  else if num < 55:
    if lastTwoMoves(THRASH):
      35.7% -> CHOMP
      64.3% -> BELLOW
    else:
      -> THRASH
  else: (num >= 55)
    if lastMove(BELLOW):
      41.6% -> CHOMP
      58.4% -> THRASH
    else:
      -> BELLOW
```

**AI Type:** Weighted Random with Anti-Repeat

---

#### Cultist
**ID:** `Cultist`
**Type:** Normal

| Stat | Base | A7+ |
|------|------|-----|
| HP | 48-54 | 50-56 |

**Moves:**
| Move | Byte | Intent | Damage | Effect |
|------|------|--------|--------|--------|
| Dark Strike | 1 | ATTACK | 6 | - |
| Incantation | 3 | BUFF | - | Ritual: +3 STR/turn (A2+: 4, A17+: 5) |

**AI Pattern:**
```
First turn: Always INCANTATION
All subsequent turns: Always DARK_STRIKE
```

**AI Type:** Fixed Pattern

---

#### Louse (Red/Normal)
**ID:** `FuzzyLouseNormal`
**Type:** Normal

| Stat | Base | A7+ |
|------|------|-----|
| HP | 10-15 | 11-16 |
| Bite Damage | 5-7 | 6-8 (A2+) |
| Curl Up Block | 3-7 | 4-8 (A7+), 9-12 (A17+) |

**Moves:**
| Move | Byte | Intent | Damage | Effect |
|------|------|--------|--------|--------|
| Bite | 3 | ATTACK | 5-7 (A2+: 6-8) | - |
| Strengthen | 4 | BUFF | - | +3 STR (A17+: +4 STR) |

**AI Pattern:**
```
A17+:
  if num < 25:
    if lastMove(STRENGTHEN): BITE
    else: STRENGTHEN
  else:
    if lastTwoMoves(BITE): STRENGTHEN
    else: BITE

Below A17:
  if num < 25:
    if lastTwoMoves(STRENGTHEN): BITE
    else: STRENGTHEN
  else:
    if lastTwoMoves(BITE): STRENGTHEN
    else: BITE
```

**AI Type:** Weighted Random with Anti-Repeat (Ascension-dependent)

---

#### Acid Slime (Large)
**ID:** `AcidSlime_L`
**Type:** Normal (can split)

| Stat | Base | A7+ |
|------|------|-----|
| HP | 65-69 | 68-72 |

**Moves:**
| Move | Byte | Intent | Base Damage | A2+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Corrosive Spit | 1 | ATTACK_DEBUFF | 11 | 12 | +2 Slimed |
| Tackle | 2 | ATTACK | 16 | 18 | - |
| Lick | 4 | DEBUFF | - | - | Weak 2 |
| Split | 3 | UNKNOWN | - | - | Suicide, spawn 2x AcidSlime_M |

**Special Behavior:**
- **Split Trigger:** When HP <= 50% of max, immediately changes move to SPLIT (interrupted)
- Split spawns 2 Medium Acid Slimes with HP = Large Slime's current HP

**AI Pattern (A17+):**
```
if num < 40:
  if lastTwoMoves(CORROSIVE_SPIT):
    60% -> TACKLE
    40% -> LICK
  else: CORROSIVE_SPIT
else if num < 70:
  if lastTwoMoves(TACKLE):
    60% -> CORROSIVE_SPIT
    40% -> LICK
  else: TACKLE
else:
  if lastMove(LICK):
    40% -> CORROSIVE_SPIT
    60% -> TACKLE
  else: LICK
```

**AI Pattern (Below A17):**
```
if num < 30:
  if lastTwoMoves(CORROSIVE_SPIT):
    50% -> TACKLE
    50% -> LICK
  else: CORROSIVE_SPIT
else if num < 70:
  if lastMove(TACKLE):
    40% -> CORROSIVE_SPIT
    60% -> LICK
  else: TACKLE
else:
  if lastTwoMoves(LICK):
    40% -> CORROSIVE_SPIT
    60% -> TACKLE
  else: LICK
```

**AI Type:** Weighted Random + Phase Trigger (Split)

---

#### Fungi Beast
**ID:** `FungiBeast`
**Type:** Normal

| Stat | Base | A7+ |
|------|------|-----|
| HP | 22-28 | 24-28 |

**Pre-Battle:** Applies Spore Cloud 2 (deals 2 Vulnerable on death)

**Moves:**
| Move | Byte | Intent | Damage | Effect |
|------|------|--------|--------|--------|
| Bite | 1 | ATTACK | 6 | - |
| Grow | 2 | BUFF | - | +3 STR (A2+: +4 STR, A17+: +5 STR) |

**AI Pattern:**
```
if num < 60:
  if lastTwoMoves(BITE): GROW
  else: BITE
else:
  if lastMove(GROW): BITE
  else: GROW
```

**AI Type:** Weighted Random with Anti-Repeat

---

### Exordium Elites

#### Gremlin Nob
**ID:** `GremlinNob`
**Type:** Elite

| Stat | Base | A8+ |
|------|------|-----|
| HP | 82-86 | 85-90 |

**Moves:**
| Move | Byte | Intent | Base Damage | A3+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Rush | 1 | ATTACK | 14 | 16 | - |
| Skull Bash | 2 | ATTACK_DEBUFF | 6 | 8 | Vulnerable 2 |
| Bellow | 3 | BUFF | - | - | Anger Power: +2 STR per card played (A18+: +3) |

**AI Pattern:**
```
First turn: Always BELLOW

A18+:
  if !lastMove(SKULL_BASH) && !lastMoveBefore(SKULL_BASH):
    SKULL_BASH
  else if lastTwoMoves(RUSH):
    SKULL_BASH
  else:
    RUSH

Below A18:
  if num < 33:
    SKULL_BASH (with anti-repeat)
  else if lastTwoMoves(RUSH):
    SKULL_BASH
  else:
    RUSH
```

**AI Type:** Fixed First + Conditional Pattern (Deterministic at A18+)

---

#### Lagavulin
**ID:** `Lagavulin`
**Type:** Elite

| Stat | Base | A8+ |
|------|------|-----|
| HP | 109-111 | 112-115 |

**Pre-Battle (if asleep):** 8 Block + Metallicize 8

**Moves:**
| Move | Byte | Intent | Base Damage | A3+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Siphon Soul | 1 | STRONG_DEBUFF | - | - | -1 STR & DEX (A18+: -2) |
| Attack | 3 | ATTACK | 18 | 20 | - |
| Sleep | 5 | SLEEP | - | - | Idle, counts turns |
| Open | 4 | STUN | - | - | Stunned after waking |
| Open Natural | 6 | - | - | - | Wakes after 3 idles |

**Sleep Mechanic:**
- Starts asleep with Metallicize 8
- Wakes after taking damage OR after 3 sleep turns
- On wake: Removes Metallicize, displays "OPEN" animation

**AI Pattern:**
```
if asleep:
  if idleCount >= 3: wake up (OPEN_NATURAL)
  else: SLEEP (increment idleCount)

if awake (isOut):
  if debuffTurnCount < 2:
    if lastTwoMoves(ATTACK): SIPHON_SOUL
    else: ATTACK
  else:
    SIPHON_SOUL
```

**AI Type:** Phase-Based (Sleep -> Awake)

---

#### Sentries
**ID:** `Sentry`
**Type:** Elite (appears in group of 3)

| Stat | Base | A8+ |
|------|------|-----|
| HP | 38-42 | 39-45 |

**Pre-Battle:** Artifact 1

**Moves:**
| Move | Byte | Intent | Base Damage | A3+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Bolt | 3 | DEBUFF | - | - | +2 Dazed (A18+: +3 Dazed) |
| Beam | 4 | ATTACK | 9 | 10 | - |

**AI Pattern:**
```
First turn: Based on position in monster list
  if (index % 2 == 0): BOLT
  else: BEAM

All subsequent turns: Strictly alternate BOLT/BEAM
  if lastMove(BEAM): BOLT
  else: BEAM
```

**AI Type:** Fixed Alternating Pattern

---

### Exordium Bosses

#### Slime Boss
**ID:** `SlimeBoss`
**Type:** Boss

| Stat | Base | A9+ |
|------|------|-----|
| HP | 140 | 150 |

**Pre-Battle:** Split Power (splits at 50% HP)

**Moves:**
| Move | Byte | Intent | Base Damage | A4+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Goop Spray | 4 | STRONG_DEBUFF | - | - | +3 Slimed (A19+: +5 Slimed) |
| Preparing | 2 | UNKNOWN | - | - | Shout, screen shake |
| Slam | 1 | ATTACK | 35 | 38 | - |
| Split | 3 | UNKNOWN | - | - | Spawn SpikeSlime_L + AcidSlime_L |

**AI Pattern:**
```
First turn: Always GOOP_SPRAY

Fixed cycle after first turn:
  GOOP_SPRAY -> PREPARING -> SLAM -> repeat

Split at 50% HP (interrupts pattern)
```

**AI Type:** Fixed Cycle + Phase Trigger (Split)

---

#### The Guardian
**ID:** `TheGuardian`
**Type:** Boss

| Stat | Base | A9+ | A19+ |
|------|------|-----|------|
| HP | 240 | 250 | 250 |
| Mode Shift Threshold | 30 | 35 | 40 |

**Pre-Battle:** Mode Shift Power (threshold damage triggers defensive mode)

**Moves:**
| Move | Byte | Intent | Base Damage | A4+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Charge Up | 6 | DEFEND | - | - | +9 Block |
| Fierce Bash | 2 | ATTACK | 32 | 36 | - |
| Vent Steam | 7 | STRONG_DEBUFF | - | - | Weak 2, Vulnerable 2 |
| Whirlwind | 5 | ATTACK | 5x4 | 5x4 | - |
| Close Up | 1 | BUFF | - | - | Sharp Hide 3 (A19+: 4) |
| Roll Attack | 3 | ATTACK | 9 | 10 | - |
| Twin Slam | 4 | ATTACK_BUFF | 8x2 | 8x2 | Removes Sharp Hide |

**Offensive Mode Cycle:**
```
CHARGE_UP -> FIERCE_BASH -> VENT_STEAM -> WHIRLWIND -> repeat
```

**Defensive Mode Cycle:**
```
CLOSE_UP -> ROLL_ATTACK -> TWIN_SLAM -> (back to Offensive)
```

**Mode Shift Mechanic:**
- Takes damage tracking
- When damage >= threshold: Switch to Defensive Mode, +20 Block
- Threshold increases by 10 each cycle
- Defensive mode: Gains Sharp Hide (thorns)

**AI Type:** Phase-Based (Mode Shifting)

---

#### Hexaghost
**ID:** `Hexaghost`
**Type:** Boss

| Stat | Base | A9+ |
|------|------|-----|
| HP | 250 | 264 |

**Moves:**
| Move | Byte | Intent | Base Damage | A4+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Activate | 5 | UNKNOWN | - | - | Divider damage = playerHP/12 + 1 |
| Divider | 1 | ATTACK | (HP/12+1)x6 | - | 6 hits |
| Tackle | 2 | ATTACK | 5x2 | 6x2 | - |
| Sear | 4 | ATTACK_DEBUFF | 6 | 6 | +1 Burn (A19+: +2 Burn) |
| Inflame | 3 | DEFEND_BUFF | - | - | +12 Block, +2 STR (A19+: +3 STR) |
| Inferno | 6 | ATTACK_DEBUFF | 2x6 | 3x6 | Upgrades all Burns |

**Orb Mechanic:**
- 6 orbs track progress through cycle
- Each move activates one orb
- When all 6 active: INFERNO
- After INFERNO: Deactivate all orbs, restart cycle

**Fixed Cycle:**
```
ACTIVATE -> DIVIDER -> SEAR -> TACKLE -> SEAR -> INFLAME -> TACKLE -> SEAR -> INFERNO -> repeat from SEAR
```

**AI Type:** Fixed Cycle with Phase Mechanic

---

## Act 2 - The City

### City Elites

#### Gremlin Leader
**ID:** `GremlinLeader`
**Type:** Elite

| Stat | Base | A8+ |
|------|------|-----|
| HP | 140-148 | 145-155 |

**Moves:**
| Move | Byte | Intent | Damage | Effect |
|------|------|--------|--------|--------|
| Rally | 2 | UNKNOWN | - | Summon 2 Gremlins |
| Encourage | 3 | DEFEND_BUFF | - | All allies: +3 STR (A3+: +4, A18+: +5), +6 Block (A18+: +10) |
| Stab | 4 | ATTACK | 6x3 | - |

**AI Pattern (depends on alive gremlins):**
```
if numAliveGremlins == 0:
  if num < 75:
    if !lastMove(RALLY): RALLY
    else: STAB
  else:
    if !lastMove(STAB): STAB
    else: RALLY

else if numAliveGremlins < 2:
  if num < 50:
    if !lastMove(RALLY): RALLY
    else: recursive call with num 50-99
  else if num < 80:
    if !lastMove(ENCOURAGE): ENCOURAGE
    else: STAB
  else:
    if !lastMove(STAB): STAB
    else: recursive call with num 0-80

else (gremlins >= 2):
  if num < 66:
    if !lastMove(ENCOURAGE): ENCOURAGE
    else: STAB
  else:
    if !lastMove(STAB): STAB
    else: ENCOURAGE
```

**AI Type:** State-Dependent (minion count)

---

#### Book of Stabbing
**ID:** `BookOfStabbing`
**Type:** Elite

| Stat | Base | A8+ |
|------|------|-----|
| HP | 160-164 | 168-172 |

**Pre-Battle:** Painful Stabs Power (adds Wound on multi-hit turns)

**Moves:**
| Move | Byte | Intent | Base Damage | A3+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Multi-Stab | 1 | ATTACK | 6xN | 7xN | N increases each use |
| Single Stab | 2 | ATTACK | 21 | 24 | - |

**Stab Count Mechanic:**
- Starts at 1
- Each Multi-Stab: stabCount++
- Each Single Stab at A18+: stabCount++

**AI Pattern:**
```
if num < 15:
  if lastMove(SINGLE_STAB):
    stabCount++
    MULTI_STAB(stabCount)
  else:
    SINGLE_STAB (A18+: stabCount++)
else:
  if lastTwoMoves(MULTI_STAB):
    SINGLE_STAB (A18+: stabCount++)
  else:
    stabCount++
    MULTI_STAB(stabCount)
```

**AI Type:** Escalating Pattern

---

### City Bosses

#### The Champ
**ID:** `Champ`
**Type:** Boss

| Stat | Base | A9+ |
|------|------|-----|
| HP | 420 | 440 |

**Moves:**
| Move | Byte | Intent | Base Damage | A4+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Heavy Slash | 1 | ATTACK | 16 | 18 | - |
| Defensive Stance | 2 | DEFEND_BUFF | - | - | +15 Block (A9+: +18, A19+: +20), Metallicize +5 (A9+: +6, A19+: +7) |
| Execute | 3 | ATTACK | 10x2 | 10x2 | - |
| Face Slap | 4 | ATTACK_DEBUFF | 12 | 14 | Frail 2, Vulnerable 2 |
| Gloat | 5 | BUFF | - | - | +2 STR (A4+: +3, A19+: +4) |
| Taunt | 6 | DEBUFF | - | - | Weak 2, Vulnerable 2 |
| Anger | 7 | BUFF | - | - | Remove debuffs, +6 STR (A4+: +9, A19+: +12) |

**Phase Transition:**
- At HP < 50%: Uses ANGER (limit break), then gains access to EXECUTE

**AI Pattern:**
```
Phase 1 (HP >= 50%):
  Every 4th turn: TAUNT
  if num <= 15 && forgeTimes < 2 && !lastMove(DEFENSIVE_STANCE):
    DEFENSIVE_STANCE (A19+: num <= 30)
  else if num <= 30 && !lastMove(GLOAT) && !lastMove(DEFENSIVE_STANCE):
    GLOAT
  else if num <= 55 && !lastMove(FACE_SLAP):
    FACE_SLAP
  else:
    if !lastMove(HEAVY_SLASH): HEAVY_SLASH
    else: FACE_SLAP

Phase 2 (HP < 50%):
  if !lastMove(EXECUTE) && !lastMoveBefore(EXECUTE):
    EXECUTE
  else: Phase 1 logic
```

**AI Type:** Phase-Based + Weighted Random

---

#### Bronze Automaton
**ID:** `BronzeAutomaton`
**Type:** Boss

| Stat | Base | A9+ |
|------|------|-----|
| HP | 300 | 320 |

**Pre-Battle:** Artifact 3

**Moves:**
| Move | Byte | Intent | Base Damage | A4+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Spawn Orbs | 4 | UNKNOWN | - | - | Spawn 2 Bronze Orbs |
| Flail | 1 | ATTACK | 7x2 | 8x2 | - |
| Boost | 5 | DEFEND_BUFF | - | - | +9 Block (A9+: +12), +3 STR (A4+: +4) |
| Hyper Beam | 2 | ATTACK | 45 | 50 | - |
| Stunned | 3 | STUN | - | - | (Only below A19) |

**AI Pattern:**
```
First turn: SPAWN_ORBS
Every 5th turn: HYPER_BEAM (resets counter)

After HYPER_BEAM:
  A19+: BOOST
  Below A19: STUNNED

After STUNNED/BOOST/SPAWN_ORBS:
  FLAIL

Otherwise:
  BOOST
```

**AI Type:** Fixed Cycle with Spawns

---

#### The Collector
**ID:** `TheCollector`
**Type:** Boss

| Stat | Base | A9+ |
|------|------|-----|
| HP | 282 | 300 |

**Moves:**
| Move | Byte | Intent | Base Damage | A4+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Spawn | 1 | UNKNOWN | - | - | Spawn 2 Torch Heads |
| Fireball | 2 | ATTACK | 18 | 21 | - |
| Buff | 3 | DEFEND_BUFF | - | - | +15 Block (A9+: +18, A19+: +23), all allies +3 STR (A4+: +4, A19+: +5) |
| Mega Debuff | 4 | STRONG_DEBUFF | - | - | Weak 3, Vulnerable 3, Frail 3 (A19+: all 5) |
| Revive | 5 | UNKNOWN | - | - | Revive dead minions |

**AI Pattern:**
```
First turn: SPAWN

After turn 3 (if not used): MEGA_DEBUFF (once per fight)

if num <= 25 && minionDead && !lastMove(REVIVE):
  REVIVE
else if num <= 70 && !lastTwoMoves(FIREBALL):
  FIREBALL
else:
  if !lastMove(BUFF): BUFF
  else: FIREBALL
```

**AI Type:** Spawn-Dependent + Weighted Random

---

## Act 3 - Beyond

### Beyond Elites

#### Giant Head
**ID:** `GiantHead`
**Type:** Elite

| Stat | Base | A8+ |
|------|------|-----|
| HP | 500 | 520 |

**Pre-Battle:** Slow Power (A18+: count starts at 4 instead of 5)

**Moves:**
| Move | Byte | Intent | Base Damage | A3+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Glare | 1 | DEBUFF | - | - | Weak 1 |
| It Is Time | 2 | ATTACK | 30+ | 40+ | Damage increases by 5 each turn after countdown |
| Count | 3 | ATTACK | 13 | 13 | - |

**Countdown Mechanic:**
- Count starts at 5 (A18+: 4)
- Each turn: count--
- When count <= 1: IT_IS_TIME
- IT_IS_TIME damage = 30 + (1-count)*5

**AI Pattern:**
```
if count <= 1:
  count-- (min -6)
  IT_IS_TIME

else:
  count--
  if num < 50:
    if !lastTwoMoves(GLARE): GLARE
    else: COUNT
  else:
    if !lastTwoMoves(COUNT): COUNT
    else: GLARE
```

**AI Type:** Countdown Timer + Weighted Random

---

#### Nemesis
**ID:** `Nemesis`
**Type:** Elite

| Stat | Base | A8+ |
|------|------|-----|
| HP | 185 | 200 |

**Special:** Gains Intangible at end of every turn

**Moves:**
| Move | Byte | Intent | Base Damage | A3+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Scythe | 3 | ATTACK | 45 | 45 | 2-turn cooldown |
| Attack | 2 | ATTACK | 6x3 | 7x3 | - |
| Debuff | 4 | DEBUFF | - | - | +3 Burns (A18+: +5 Burns) |

**AI Pattern:**
```
First turn:
  if num < 50: ATTACK
  else: DEBUFF

Subsequent (scytheCooldown decrements each turn):
  if num < 30:
    if !lastMove(SCYTHE) && scytheCooldown <= 0:
      SCYTHE (cooldown = 2)
    else: 50% ATTACK / 50% DEBUFF (with anti-repeat)
  else if num < 65:
    if !lastTwoMoves(ATTACK): ATTACK
    else: 50% SCYTHE / 50% DEBUFF
  else:
    if !lastMove(DEBUFF): DEBUFF
    else: 50% SCYTHE / 50% ATTACK
```

**AI Type:** Cooldown-Based + Weighted Random

---

### Beyond Bosses

#### Awakened One
**ID:** `AwakenedOne`
**Type:** Boss

| Stat | Base | A9+ |
|------|------|-----|
| HP (Phase 1) | 300 | 320 |
| HP (Phase 2) | 300 | 320 |

**Pre-Battle:**
- Regenerate 10 (A19+: 15)
- Curiosity: +1 STR when you play a Power (A19+: +2)
- Unawakened Power (prevents death, triggers phase 2)
- A4+: +2 STR

**Moves (Phase 1):**
| Move | Byte | Intent | Damage | Effect |
|------|------|--------|--------|--------|
| Slash | 1 | ATTACK | 20 | - |
| Soul Strike | 2 | ATTACK | 6x4 | - |
| Rebirth | 3 | UNKNOWN | - | Full heal, phase transition |

**Moves (Phase 2):**
| Move | Byte | Intent | Damage | Effect |
|------|------|--------|--------|--------|
| Dark Echo | 5 | ATTACK | 40 | - |
| Sludge | 6 | ATTACK_DEBUFF | 18 | +1 Void |
| Tackle | 8 | ATTACK | 10x3 | - |

**Phase 1 AI:**
```
First turn: SLASH

if num < 25:
  if !lastMove(SOUL_STRIKE): SOUL_STRIKE
  else: SLASH
else:
  if !lastTwoMoves(SLASH): SLASH
  else: SOUL_STRIKE
```

**Phase 2 AI:**
```
First turn: DARK_ECHO

if num < 50:
  if !lastTwoMoves(SLUDGE): SLUDGE
  else: TACKLE
else:
  if !lastTwoMoves(TACKLE): TACKLE
  else: SLUDGE
```

**AI Type:** Two-Phase Boss

---

#### Time Eater
**ID:** `TimeEater`
**Type:** Boss

| Stat | Base | A9+ |
|------|------|-----|
| HP | 456 | 480 |

**Pre-Battle:** Time Warp Power (after 12 cards played: ends turn, +2 STR)

**Moves:**
| Move | Byte | Intent | Base Damage | A4+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Reverberate | 2 | ATTACK | 7x3 | 8x3 | - |
| Ripple | 3 | DEFEND_DEBUFF | - | - | +20 Block, Vulnerable 1, Weak 1 (A19+: +Frail 1) |
| Head Slam | 4 | ATTACK_DEBUFF | 26 | 32 | Draw -1 (A19+: +2 Slimed) |
| Haste | 5 | BUFF | - | - | Remove debuffs, heal to 50% (A19+: +32 Block) |

**Haste Trigger:** When HP < 50%, uses HASTE once

**AI Pattern:**
```
if HP < 50% && !usedHaste:
  HASTE

if num < 45:
  if !lastTwoMoves(REVERBERATE): REVERBERATE
  else: recursive call with num 50-99
else if num < 80:
  if !lastMove(HEAD_SLAM): HEAD_SLAM
  else: 66% REVERBERATE / 34% RIPPLE
else:
  if !lastMove(RIPPLE): RIPPLE
  else: recursive call with num 0-74
```

**AI Type:** Phase Trigger + Weighted Random

---

#### Donu and Deca
**IDs:** `Donu`, `Deca`
**Type:** Boss (paired)

| Stat | Base | A9+ |
|------|------|-----|
| Donu HP | 250 | 265 |
| Deca HP | 250 | 265 |

**Pre-Battle (Both):** Artifact 2 (A19+: Artifact 3)

**Donu Moves:**
| Move | Byte | Intent | Base Damage | A4+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Beam | 0 | ATTACK | 10x2 | 12x2 | - |
| Circle | 2 | BUFF | - | - | All allies +3 STR |

**Deca Moves:**
| Move | Byte | Intent | Base Damage | A4+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Beam | 0 | ATTACK_DEBUFF | 10x2 | 12x2 | +2 Dazed |
| Square | 2 | DEFEND | - | - | All allies +16 Block (A19+: +Plated Armor 3) |

**AI Pattern:**
```
Both strictly alternate ATTACK / BUFF (or DEFEND)
Donu starts with BUFF
Deca starts with ATTACK
```

**AI Type:** Fixed Alternating (Coordinated)

---

## Ending

#### Corrupt Heart
**ID:** `CorruptHeart`
**Type:** Boss

| Stat | Base | A9+ |
|------|------|-----|
| HP | 750 | 800 |

**Pre-Battle:**
- Invincible 300 (A19+: 200)
- Beat of Death 1: Deal 1 damage per card played (A19+: 2)

**Moves:**
| Move | Byte | Intent | Base Damage | A4+ Damage | Effect |
|------|------|--------|-------------|------------|--------|
| Debilitate | 3 | STRONG_DEBUFF | - | - | Vulnerable 2, Weak 2, Frail 2, +1 each status (Dazed/Slimed/Wound/Burn/Void) |
| Buff | 4 | BUFF | - | - | +2 STR (removes negative), escalating buffs |
| Blood Shots | 1 | ATTACK | 2x12 | 2x15 | - |
| Echo | 2 | ATTACK | 40 | 45 | - |

**Buff Escalation (moveCount 4):**
| Use | Effect |
|-----|--------|
| 1st | +2 Artifact |
| 2nd | +1 Beat of Death |
| 3rd | Painful Stabs |
| 4th | +10 STR |
| 5th+ | +50 STR |

**AI Pattern:**
```
First turn: DEBILITATE

3-turn cycle (moveCount % 3):
  0: 50% BLOOD_SHOTS / 50% ECHO
  1: if !lastMove(ECHO): ECHO else: BLOOD_SHOTS
  2: BUFF

moveCount++
```

**AI Type:** Fixed Cycle + Escalating Buffs

---

#### Spire Shield
**ID:** `SpireShield`
**Type:** Elite (Ending)

| Stat | Base | A8+ |
|------|------|-----|
| HP | 110 | 125 |

**Pre-Battle:** Artifact 1 (A18+: Artifact 2)

**Moves:**
| Move | Byte | Intent | Damage | Effect |
|------|------|--------|--------|--------|
| Bash | 1 | ATTACK | 12 (A3+: 14) | - |
| Fortify | 2 | DEFEND | - | +30 Block |
| Smash | 3 | ATTACK_DEFEND | 34 (A3+: 38) | +99 Block |

**AI Pattern:**
```
First turn: BASH

if num < 50:
  if !lastMove(BASH) && !lastMoveBefore(BASH): BASH
  else if !lastMove(FORTIFY): FORTIFY
  else: SMASH
else:
  if !lastMove(SMASH): SMASH
  else if !lastMove(FORTIFY): FORTIFY
  else: BASH
```

**AI Type:** 3-Move Cycle with Randomness

---

#### Spire Spear
**ID:** `SpireSpear`
**Type:** Elite (Ending)

| Stat | Base | A8+ |
|------|------|-----|
| HP | 160 | 180 |

**Pre-Battle:** Artifact 1 (A18+: Artifact 2)

**Moves:**
| Move | Byte | Intent | Damage | Effect |
|------|------|--------|--------|--------|
| Burn Strike | 1 | ATTACK_DEBUFF | 5x2 (A3+: 6x2) | +2 Burns |
| Piercer | 3 | ATTACK | 10x3 (A3+: 12x3) | - |
| Skewer | 2 | ATTACK_DEBUFF | 8x4 (A3+: 10x4) | +1 Burn per hit |

**AI Pattern:**
```
First turn: BURN_STRIKE

if num < 30:
  if !lastMove(BURN_STRIKE) && !lastMoveBefore(BURN_STRIKE): BURN_STRIKE
  else if !lastMove(PIERCER): PIERCER
  else: SKEWER
else if num < 70:
  if !lastTwoMoves(PIERCER): PIERCER
  else: SKEWER
else:
  if !lastMove(SKEWER): SKEWER
  else if !lastMove(PIERCER): PIERCER
  else: BURN_STRIKE
```

**AI Type:** 3-Move Cycle with Randomness

---

## AI Pattern Categories

### Fixed Pattern
Enemies that follow deterministic move sequences:
- **Cultist**: Incantation -> Attack forever
- **Sentries**: Strictly alternating Bolt/Beam
- **Donu/Deca**: Alternating Attack/Buff

### Fixed Cycle
Enemies that follow a repeating cycle but may have phase triggers:
- **Slime Boss**: Goop -> Prep -> Slam (splits at 50%)
- **Hexaghost**: Set sequence through 6 orbs
- **Bronze Automaton**: Spawn -> Flail/Boost -> Hyper Beam

### Weighted Random
Enemies that use probability rolls with anti-repeat rules:
- **Jaw Worm**: Complex probability tree
- **Fungi Beast**: 60/40 split with anti-repeat
- **Louse**: 25/75 split with anti-repeat

### Phase-Based
Enemies that change behavior based on HP or other triggers:
- **Guardian**: Offensive/Defensive mode shift
- **Awakened One**: Two distinct phases
- **Champ**: Phase 2 at 50% HP (Execute access)
- **Slimes**: Split at 50% HP

### State-Dependent
Enemies whose behavior depends on game state:
- **Gremlin Leader**: Changes based on minion count
- **The Collector**: Revives dead minions

### Escalating
Enemies that get stronger over time:
- **Book of Stabbing**: Increasing stab count
- **Giant Head**: Countdown to massive damage
- **Corrupt Heart**: Escalating buff powers

### Cooldown-Based
Enemies with moves on cooldown timers:
- **Nemesis**: Scythe has 2-turn cooldown

---

## Key Implementation Notes

1. **Anti-Repeat Functions:**
   - `lastMove(byte)`: Returns true if previous move matches
   - `lastTwoMoves(byte)`: Returns true if last two moves both match
   - `lastMoveBefore(byte)`: Returns true if move before last matches

2. **Random Roll:**
   - `getMove(int num)` receives a random 0-99 value
   - Used for weighted probability selection

3. **Ascension Thresholds:**
   - A2: Basic enemy damage increase
   - A3: Elite damage increase
   - A4: Boss damage increase
   - A7: Basic enemy HP increase
   - A8: Elite HP increase
   - A9: Boss HP increase
   - A17-19: Special ability changes

4. **Split Mechanic:**
   - Triggered when `currentHealth <= maxHealth / 2`
   - Immediately interrupts current move
   - Spawns two smaller versions with current HP

5. **Intent Types:**
   - `ATTACK`: Damage
   - `ATTACK_BUFF`: Damage + self buff
   - `ATTACK_DEBUFF`: Damage + player debuff
   - `ATTACK_DEFEND`: Damage + block
   - `BUFF`: Self buff
   - `DEBUFF`: Player debuff
   - `STRONG_DEBUFF`: Major player debuff
   - `DEFEND`: Block
   - `DEFEND_BUFF`: Block + buff
   - `DEFEND_DEBUFF`: Block + debuff
   - `SLEEP`: Idle/asleep
   - `STUN`: Stunned
   - `UNKNOWN`: Hidden intent

---

## Implementation Notes

Our implementation is in `packages/engine/content/enemies.py`. Key bug fixes from Java decompilation:

### HP Range Corrections

Several enemies had off-by-one errors in online documentation. Our implementation uses the correct values from Java decompiled source:

| Enemy | Incorrect Value | Correct Value | Notes |
|-------|-----------------|---------------|-------|
| Red/Green Louse (A7+) | (11, 17) | (11, 16) | Java: `A_2_HP_MAX = 16` |
| Lagavulin (A8+) | (112, 116) | (112, 115) | Java: `A_2_HP_MAX = 115` |

These are documented inline in the code with "Fixed:" comments.

### AI Pattern Implementation

Each enemy class implements `get_move(roll: int)` matching the exact decision tree from the game. The `roll` parameter is `aiRng.random(99)` which gives `[0, 99]` inclusive.

Key state tracking methods used:
- `state.last_move(move_id)`: Was last move this ID?
- `state.last_two_moves(move_id)`: Were last two moves both this ID?
- `state.last_move_before(move_id)`: Was the move before last this ID?

### Ascension Thresholds Reference

For quick reference, the standard ascension thresholds that affect enemies:

| Ascension | Effect |
|-----------|--------|
| A2 | Basic enemy damage increase |
| A3 | Elite damage increase |
| A4 | Boss damage increase |
| A7 | Basic enemy HP increase |
| A8 | Elite HP increase |
| A9 | Boss HP increase |
| A17+ | Special ability changes (varies by enemy) |
| A18+ | Elite special ability changes |
| A19+ | Boss special ability changes |
