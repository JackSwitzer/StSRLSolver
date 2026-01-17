# Ascension Modifiers (A1-A20)

Complete documentation of all Ascension modifiers extracted from decompiled Slay the Spire source code.

## Summary Table

| Level | Category | Effect |
|-------|----------|--------|
| A1 | Map | Elite room frequency increased by 1.6x |
| A2 | Monsters | Normal enemies deal more damage |
| A3 | Elites | Elites deal more damage |
| A4 | Bosses | Bosses deal more damage |
| A5 | Healing | Heal only 75% between acts (instead of full) |
| A6 | Starting | Start at 90% max HP |
| A7 | Monsters | Normal enemies have more HP |
| A8 | Elites | Elites have more HP |
| A9 | Bosses | Bosses have more HP |
| A10 | Starting | Start with Ascender's Bane (unplayable, unremovable curse) |
| A11 | Potions | One fewer potion slot |
| A12 | Cards | Upgraded card rewards reduced by 50% |
| A13 | Rewards | Boss gold rewards reduced by 25% |
| A14 | Starting | Max HP reduced at start |
| A15 | Events | Events have worse outcomes |
| A16 | Shop | Shop prices increased by 10% |
| A17 | Monsters | Normal enemies have improved AI/abilities |
| A18 | Elites | Elites have improved AI/abilities |
| A19 | Bosses | Bosses have improved AI/abilities |
| A20 | Double Boss | Two bosses in Act 3 (if you've beaten the Heart) |

---

## Ascension 1: More Elites

**Effect:** Elite room frequency multiplied by 1.6x

**Source:** `AbstractDungeon.java:551`
```java
} else if (ascensionLevel >= 1) {
    eliteCount = Math.round((float)availableRoomCount * eliteRoomChance * 1.6f);
}
```

---

## Ascension 2: Monsters Deal More Damage

**Effect:** All normal enemies deal increased damage.

### Exordium Monsters

| Monster | Normal Damage | A2+ Damage |
|---------|---------------|------------|
| Cultist | Ritual 3 | Ritual 4 |
| Spike Slime (L) | 16 | 18 |
| Spike Slime (M) | 8 | 10 |
| Spike Slime (S) | 5 | 6 |
| Acid Slime (L) | Tackle 11, Lick 16 | Tackle 12, Lick 18 |
| Acid Slime (M) | Tackle 7, Lick 10 | Tackle 8, Lick 12 |
| Acid Slime (S) | 3 | 4 |
| Jaw Worm | Bellow Str 3/Block 5, Thrash 6/4 | Bellow Str 4/Block 6, Thrash 7/5 |
| Louse (Normal/Defensive) | 5-7 | 6-8 |
| Looter | Swipe 10, Lunge 12 | Swipe 11, Lunge 14 |
| Fungi Beast | Str 3, Bite 6 | Str 4, Bite 6 |
| Slaver Blue | Stab 12, Rake 7 | Stab 13, Rake 8 |
| Slaver Red | Stab 13, Scrape 8 | Stab 14, Scrape 9 |
| Fat Gremlin | 4 | 5 |
| Gremlin Warrior | 4 | 5 |
| Gremlin Wizard | 25 | 30 |
| Gremlin Thief | 9 | 10 |
| Gremlin Sneaky (Tsundere) | 6 | 8 |

### City Monsters

| Monster | Normal Damage | A2+ Damage |
|---------|---------------|------------|
| Mugger | Swipe 10, Big Swipe 16 | Swipe 11, Big Swipe 18 |
| Healer | Magic 8, Str 3, Heal 16 | Magic 9, Str 3, Heal 16 |
| Spheric Guardian | 10 | 11 |
| Chosen | Zap 18, Debilitate 10 | Zap 21, Debilitate 12 |
| Byrd | Peck 1x5 | Peck 1x6 |
| Snecko | Bite 15, Tail 8 | Bite 18, Tail 10 |
| Shelled Parasite | Double Strike 6, Fell 18 | Double Strike 7, Fell 21 |
| Snake Plant | Rain Blows 7 | Rain Blows 8 |
| Centurion | Slash 12, Fury 6 | Slash 14, Fury 7 |
| Bandit Pointy | 5 | 6 |
| Bandit Bear | Maul 18, Lunge 9 | Maul 20, Lunge 10 |
| Bandit Leader | Slash 15, Agonize 10 | Slash 17, Agonize 12 |

### Beyond Monsters

| Monster | Normal Damage | A2+ Damage |
|---------|---------------|------------|
| Spire Growth | Tackle 16, Smash 22 | Tackle 18, Smash 25 |
| Transient | Starting Death Dmg 30 | Starting Death Dmg 40 |
| Repulsor | 11 | 13 |
| Exploder | 9 | 11 |
| Maw | Slam 25, Nom 4 | Slam 30, Nom 5 |
| Darkling | Chomp 8, Nip 7-11 | Chomp 9, Nip 9-13 |
| Orb Walker | Claw 15, Laser 10 | Claw 16, Laser 11 |
| Spiker | 7, Thorns 3 | 9, Thorns 4 |
| Writhing Mass | Strong 34, Multi 8 | Strong 38, Multi 9 |

---

## Ascension 3: Elites Deal More Damage

**Effect:** All elite enemies deal increased damage.

### Exordium Elites

| Elite | Normal | A3+ |
|-------|--------|-----|
| Gremlin Nob | Bash 6, Rush 14 | Bash 8, Rush 16 |
| Lagavulin | 18 | 20 |
| Sentry | Beam 9 | Beam 10 |

### City Elites

| Elite | Normal | A3+ |
|-------|--------|-----|
| Gremlin Leader | Str 3, Block 6 | Str 4, Block 6 |
| Book of Stabbing | Stab 6, Single 21 | Stab 7, Single 24 |
| Taskmaster | Wounds 1 | Wounds 2 |

### Beyond Elites

| Elite | Normal | A3+ |
|-------|--------|-----|
| Giant Head | Death Dmg 30 | Death Dmg 40 |
| Nemesis | Fire 6 | Fire 7 |
| Reptomancer | Dmg 13, Big 30 | Dmg 16, Big 34 |

### Ending Elites (Spire Shields/Spears)

| Elite | Normal | A3+ |
|-------|--------|-----|
| Spire Shield | Bash 12, Smash 34 | Bash 14, Smash 38 |
| Spire Spear | Skewer 3x | Skewer 4x |

---

## Ascension 4: Bosses Deal More Damage

**Effect:** All bosses deal increased damage.

### Exordium Bosses

| Boss | Normal | A4+ |
|------|--------|-----|
| Slime Boss | Tackle 8, Slam 35 | Tackle 10, Slam 38 |
| The Guardian | Fierce Bash 32, Roll 9 | Fierce Bash 36, Roll 10 |
| Hexaghost | Str 2, Sear 1 burn | Str 2, Sear 1 burn (same until A19) |

### City Bosses

| Boss | Normal | A4+ |
|------|--------|-----|
| Champ | Slash 16, Execute 10, Forge 4 | Slash 18, Execute 10, Forge 5 |
| Bronze Automaton | Flail 7, Hyper Beam 45 | Flail 8, Hyper Beam 50 |
| The Collector | Rake 18, Str 3 | Rake 21, Str 4 |

### Beyond Bosses

| Boss | Normal | A4+ |
|------|--------|-----|
| Time Eater | Reverb 7, Head Slam 26 | Reverb 8, Head Slam 32 |
| Awakened One | Starts with Str 0 | Starts with Str 2 |
| Donu | Beam 10 | Beam 12 |
| Deca | Beam 10 | Beam 12 |

### Corrupt Heart

| Attack | Normal | A4+ |
|--------|--------|-----|
| Blood Shots | 2x15 | 2x15 (same) |
| Main Attack | 40 | 45 |

---

## Ascension 5: Reduced Healing Between Acts

**Effect:** Heal 75% of missing HP between acts instead of 100%.

**Source:** `AbstractDungeon.java:2562`
```java
if (ascensionLevel >= 5) {
    player.heal(MathUtils.round((float)(AbstractDungeon.player.maxHealth - AbstractDungeon.player.currentHealth) * 0.75f), false);
} else {
    player.heal(player.maxHealth - player.currentHealth, false);
}
```

---

## Ascension 6: Start at 90% HP

**Effect:** Start the game at 90% of max HP instead of full.

**Source:** `AbstractDungeon.java:2574`
```java
if (ascensionLevel >= 6) {
    AbstractDungeon.player.currentHealth = MathUtils.round((float)AbstractDungeon.player.maxHealth * 0.9f);
}
```

---

## Ascension 7: Monsters Have More HP

**Effect:** All normal enemies have increased HP.

### Exordium Monsters

| Monster | Normal HP | A7+ HP |
|---------|-----------|--------|
| Cultist | 48-54 | 50-56 |
| Spike Slime (L) | 64-70 | 67-73 |
| Spike Slime (M) | 28-32 | 29-34 |
| Spike Slime (S) | 10-14 | 11-15 |
| Acid Slime (L) | 65-69 | 68-72 |
| Acid Slime (M) | 28-32 | 29-34 |
| Acid Slime (S) | 8-12 | 9-13 |
| Jaw Worm | 40-44 | 42-46 |
| Louse (Normal) | 10-15 | 11-16 |
| Louse (Defensive) | 11-17 | 12-18 |
| Looter | 44-48 | 46-50 |
| Fungi Beast | 22-28 | 24-28 |
| Slaver Blue | 46-50 | 48-52 |
| Slaver Red | 46-50 | 48-52 |
| Fat Gremlin | 13-17 | 14-18 |
| Gremlin Warrior | 20-24 | 21-25 |
| Gremlin Wizard | 21-25 | 22-26 |
| Gremlin Thief | 10-14 | 11-15 |

### City Monsters

| Monster | Normal HP | A7+ HP |
|---------|-----------|--------|
| Mugger | 48-52 | 50-54 |
| Healer | 48-56 | 50-58 |
| Byrd | 25-31 | 26-33 |
| Snecko | 114-120 | 120-125 |
| Shelled Parasite | 68-72 | 70-75 |
| Snake Plant | 75-79 | 78-82 |
| Centurion | 76-80 | 78-83 |
| Chosen | 95-99 | 98-103 |
| Bandit Bear | 38-42 | 40-44 |
| Bandit Leader | 35-39 | 37-41 |
| Bandit Pointy | 30 | 34 |

### Beyond Monsters

| Monster | Normal HP | A7+ HP |
|---------|-----------|--------|
| Spire Growth | 170 | 190 |
| Writhing Mass | 160 | 175 |
| Spiker | 42-56 | 44-60 |
| Darkling | 48-56 | 50-59 |
| Orb Walker | 90-96 | 92-102 |
| Exploder | 30 | 30-35 |
| Repulsor | 29-35 | 31-38 |

### Louse Curl Up Power

| Level | Curl Up Block |
|-------|---------------|
| Normal | 3-7 |
| A7+ | 4-8 |
| A17+ | 9-12 |

---

## Ascension 8: Elites Have More HP

**Effect:** All elite enemies have increased HP.

### Exordium Elites

| Elite | Normal HP | A8+ HP |
|-------|-----------|--------|
| Gremlin Nob | 82-86 | 85-90 |
| Lagavulin | 109-111 | 112-115 |
| Sentry | 38-42 | 39-45 |

### City Elites

| Elite | Normal HP | A8+ HP |
|-------|-----------|--------|
| Gremlin Leader | 140-148 | 145-155 |
| Book of Stabbing | 160-164 | 168-172 |
| Taskmaster | 54-60 | 57-64 |

### Beyond Elites

| Elite | Normal HP | A8+ HP |
|-------|-----------|--------|
| Giant Head | 500 | 520 |
| Nemesis | 185 | 200 |
| Reptomancer | 180-190 | 190-200 |

### Ending Elites

| Elite | Normal HP | A8+ HP |
|-------|-----------|--------|
| Spire Shield | 110 | 125 |
| Spire Spear | 160 | 180 |

---

## Ascension 9: Bosses Have More HP

**Effect:** All bosses have increased HP.

### Exordium Bosses

| Boss | Normal HP | A9+ HP |
|------|-----------|--------|
| Slime Boss | 140 | 150 |
| The Guardian | 240 | 250 |
| Hexaghost | 250 | 264 |

### City Bosses

| Boss | Normal HP | A9+ HP |
|------|-----------|--------|
| Champ | 420 | 440 |
| Bronze Automaton | 300, Block 9 | 320, Block 12 |
| Bronze Orb | 52-58 | 54-60 |
| Torch Head | 38-40 | 40-45 |
| The Collector | 280, Block 15 | 300, Block 18 |

### Beyond Bosses

| Boss | Normal HP | A9+ HP |
|------|-----------|--------|
| Time Eater | 456 | 480 |
| Awakened One | 300 (each phase) | 320 (each phase) |
| Donu | 250 | 265 |
| Deca | 250 | 265 |

### Corrupt Heart

| Level | HP |
|-------|-----|
| Normal | 750 |
| A9+ | 800 |

---

## Ascension 10: Ascender's Bane

**Effect:** Start with Ascender's Bane in your deck.

**Ascender's Bane Properties:**
- Unplayable curse
- Ethereal (exhausts if in hand at end of turn)
- Cannot be removed from deck
- Takes up a card slot every shuffle

**Source:** `AbstractDungeon.java:2577`
```java
if (ascensionLevel >= 10) {
    AbstractDungeon.player.masterDeck.addToTop(new AscendersBane());
    UnlockTracker.markCardAsSeen("AscendersBane");
}
```

---

## Ascension 11: One Fewer Potion Slot

**Effect:** Start with 2 potion slots instead of 3.

**Source:** `AbstractPlayer.java:193`
```java
if (AbstractDungeon.ascensionLevel >= 11) {
    --this.potionSlots;
}
```

---

## Ascension 12: Reduced Upgraded Card Rewards

**Effect:** Chance of finding upgraded cards in rewards reduced by 50%.

| Dungeon | Normal | A12+ |
|---------|--------|------|
| Act 1 (Exordium) | N/A (uses different system) | N/A |
| Act 2 (City) | 25% | 12.5% |
| Act 3 (Beyond) | 50% | 25% |
| Act 4 (Ending) | 50% | 25% |

**Source:** `TheCity.java:80`
```java
cardUpgradedChance = AbstractDungeon.ascensionLevel >= 12 ? 0.125f : 0.25f;
```

---

## Ascension 13: Reduced Boss Gold Rewards

**Effect:** Boss gold rewards reduced by 25%.

**Source:** `AbstractRoom.java:282`
```java
int tmp = 100 + AbstractDungeon.miscRng.random(-5, 5);
if (AbstractDungeon.ascensionLevel >= 13) {
    this.addGoldToRewards(MathUtils.round((float)tmp * 0.75f));
} else {
    this.addGoldToRewards(tmp);
}
```

**Also affects Mind Bloom event:**
- Normal boss fight reward: 50 gold
- A13+ boss fight reward: 25 gold

---

## Ascension 14: Reduced Starting Max HP

**Effect:** Max HP reduced at the start of the run.

The reduction amount is character-specific (retrieved via `player.getAscensionMaxHPLoss()`).

| Character | HP Loss |
|-----------|---------|
| Ironclad | 5 |
| Silent | 4 |
| Defect | 4 |
| Watcher | 4 |

**Source:** `AbstractDungeon.java:2571`
```java
if (ascensionLevel >= 14) {
    player.decreaseMaxHealth(player.getAscensionMaxHPLoss());
}
```

---

## Ascension 15: Worse Event Outcomes

**Effect:** Events have worse outcomes - higher costs, less rewards, more damage.

### Exordium Events

| Event | Normal | A15+ |
|-------|--------|------|
| Cleric Purify Cost | 50 gold | 75 gold |
| Goop Puddle Gold Loss | 20-50 | 35-75 |
| Shining Light Damage | 20% max HP | 30% max HP |
| Scrap Ooze Damage | 3 per dig | 5 per dig |
| Golden Idol Trap | 25% HP damage | 35% HP damage |
| Golden Idol Curse | 8% max HP loss | 10% max HP loss |
| Dead Adventurer Fight Chance | 25% | 35% |
| Sssserpent Gold Reward | 175 | 150 |

### City Events

| Event | Normal | A15+ |
|-------|--------|------|
| Ghosts (Apparitions) | 5 copies | 3 copies |
| Forgotten Altar HP Loss | 25% max HP | 35% max HP |
| The Library Heal | 33% max HP | 20% max HP |
| Cursed Tome Final Damage | 10 | 15 |
| The Mausoleum Curse Chance | 50% | 100% |
| The Nest Gold Gain | 99 | 50 |

### Beyond Events

| Event | Normal | A15+ |
|-------|--------|------|
| Moai Head HP Loss | 12.5% max HP | 18% max HP |
| Winding Halls HP Loss/Heal | 12.5% / 25% | 18% / 20% |

### Shrine Events

| Event | Normal | A15+ |
|-------|--------|------|
| Designer Adjust Cost | 40 gold | 50 gold |
| Designer Cleanup Cost | 60 gold | 75 gold |
| Gremlin Wheel Damage | 10% max HP | 15% max HP |
| Gold Shrine Reward | 100 gold | 50 gold |
| Woman in Blue (all potions) | Free | 5% max HP damage |
| Face Trader Gold Reward | 75 gold | 50 gold |
| Gremlin Match Game | 2 rare, 2 uncommon, 1 common | 1 rare, 2 uncommon, 2 common |

### Other A15+ Effects

- **Note For Yourself** event is disabled at A15+

**Source:** `AbstractDungeon.java:1345`
```java
if (ascensionLevel >= 15) {
    logger.info("Note For Yourself is disabled beyond Ascension 15+");
    return false;
}
```

---

## Ascension 16: Shop Price Increase

**Effect:** All shop prices increased by 10%.

**Source:** `ShopScreen.java:212`
```java
if (AbstractDungeon.ascensionLevel >= 16) {
    this.applyDiscount(1.1f, false);
}
```

This affects:
- Card prices
- Relic prices
- Potion prices
- Card removal cost

---

## Ascension 17: Monster AI Improvements

**Effect:** Normal enemies have improved AI and often enhanced abilities.

### Exordium Monsters

| Monster | Change |
|---------|--------|
| Cultist | First turn Ritual gives +1 extra strength (5 instead of 4) |
| Spike Slime (L) | Frail increased to 3 turns (from 2), uses Flame Tackle more often |
| Spike Slime (M) | Uses Flame Tackle more often |
| Acid Slime (S) | Can use Weak attack more frequently |
| Acid Slime (L/M) | Uses Corrosive Spit more often |
| Jaw Worm | Bellow: Str 5, Block 9 (from 4/6) |
| Louse (Normal) | Curl Up 9-12 (from 4-8), Grow gives +4 Str (from +3) |
| Louse (Defensive) | Curl Up 9-12 (from 4-8) |
| Looter | Steals 20 gold (from 15) |
| Fat Gremlin | Also applies Frail 1 with attack |
| Gremlin Wizard | Charges Ultimate faster |
| Gremlin Warrior | Angry power gives +2 Str (from +1) |
| Gremlin Sneaky | HP 13-17, Block 11 (from 7-8) |
| Fungi Beast | Grow gives +4 Str (from +3) |
| Slaver Blue | Rake applies +1 extra Weak |
| Slaver Red | Scrape applies +1 extra Vulnerable, attacks more often |

### City Monsters

| Monster | Change |
|---------|--------|
| Mugger | Escape gains +6 block (17 instead of 11) |
| Healer | Magic damage 9, Str 4, Heal 20; heals more aggressively |
| Spheric Guardian | Harden gives 35 block (from 25) |
| Shelled Parasite | Starts with Stunned + attack instead of just Stunned |
| Byrd | Flight stacks 4 (from 3) |
| Snecko | Tail Whip also applies Weak 2 |
| Snake Plant | Uses Chomp more often (65% instead of 50%) |
| Bandit Bear | Smash reduces CON by -4 (from -2) |
| Chosen | Always starts with Hex on first turn |
| Bandit Leader | Weak 3 turns (from 2), can attack twice in a row |
| Centurion | Block amount increased |

### Beyond Monsters

| Monster | Change |
|---------|--------|
| Spire Growth | Constrict deals +2 damage, prioritizes applying Constricted |
| Transient | Fading power triggers at 6 turns (from 5) |
| Spiker | Starting Thorns +3 (total 7 at A17) |
| Orb Walker | Strength Up is 5 per turn (from 3) |
| Maw | Strength Up +2 (5 total), Terrify duration +2 (5 total) |
| Darkling | Can use Defend+Buff more often |

---

## Ascension 18: Elite AI Improvements

**Effect:** Elite enemies have improved AI and enhanced abilities.

### Exordium Elites

| Elite | Change |
|-------|--------|
| Gremlin Nob | Anger power gives +3 Str (from +2), can Skull Bash twice in a row |
| Lagavulin | Siphon Soul debuff is -2 Str/Dex (from -1) |
| Sentry | Daze cards increased to 3 (from 2) |

### City Elites

| Elite | Change |
|-------|--------|
| Gremlin Leader | Str +5, Block 10 (from 4/6 or 5/8) |
| Book of Stabbing | Stab count increases faster (every turn vs every other turn) |
| Taskmaster | Scouring Whip adds 3 Wounds (from 2) |

### Beyond Elites

| Elite | Change |
|-------|--------|
| Giant Head | Count starts at 4 (from 5) - attacks faster |
| Nemesis | Scorch adds 5 Burns (from 3) |
| Reptomancer | Spawns 2 daggers at a time (from 1) |

### Ending Elites

| Elite | Change |
|-------|--------|
| Spire Shield | Starts with 2 Artifact (from 1), Fortify gives 99 block (from 30) |
| Spire Spear | Starts with 2 Artifact (from 1), Skewer adds 2 Burns (from 1) |

---

## Ascension 19: Boss AI Improvements

**Effect:** Bosses have improved AI and enhanced abilities.

### Exordium Bosses

| Boss | Change |
|------|--------|
| Slime Boss | Slam applies 5 Slimed (from 3) |
| The Guardian | Threshold 40 (from 35), Sharp Hide +1 thorns |
| Hexaghost | Str 3, Sear adds 2 Burns (from 2/1) |

### City Bosses

| Boss | Change |
|------|--------|
| Champ | More aggressive forging, enhanced stats |
| Bronze Automaton | Uses Boost (Str+Block) after Hyper Beam instead of just Stun |
| The Collector | Mega Debuff 5 (from 4), Block +5 on defend, Rake 21/Str 5 |

### Beyond Bosses

| Boss | Change |
|------|--------|
| Time Eater | (No specific A19 changes found beyond A9 HP) |
| Awakened One | Gains Regenerate 15 HP/turn, Curiosity gives +2 Str (from +1) |
| Donu | Starts with 3 Artifact (from 2) |
| Deca | Starts with 3 Artifact (from 2), uses Defend+Buff instead of just Defend |

### Corrupt Heart

| Aspect | Change |
|--------|--------|
| Invincible | 200 damage cap (from 300) |
| Beat of Death | Deals 2 damage (from 1) per card played |

---

## Ascension 20: Double Boss

**Effect:** Fight two bosses in Act 3 (only applicable after defeating the Heart once).

**Source:** `AbstractMonster.java:1045`
```java
if (!(AbstractDungeon.ascensionLevel >= 20 && AbstractDungeon.bossList.size() == 2 || Settings.isEndless))
```

At A20, after defeating the first Act 3 boss, you must fight a second boss immediately.

---

## Cumulative Effects at A20

When playing at Ascension 20, ALL of the above modifiers are active simultaneously:

### Starting State
- Max HP reduced (character specific)
- Start at 90% of reduced max HP
- Start with Ascender's Bane curse
- 2 potion slots (instead of 3)

### During Run
- 1.6x elite frequency
- All enemies deal maximum damage (A2/A3/A4 combined)
- All enemies have maximum HP (A7/A8/A9 combined)
- All enemies have best AI (A17/A18/A19 combined)
- Heal only 75% between acts
- 50% fewer upgraded card rewards
- 25% less boss gold
- All events have worst outcomes
- Shop prices +10%

### Final Challenge
- Two bosses in Act 3
- Heart at full difficulty

---

## Code Reference Quick Guide

Key files for ascension checks:
- `AbstractDungeon.java` - Core game logic, healing, map generation
- `AbstractPlayer.java` - Player initialization, potion slots
- `AbstractRoom.java` - Rewards, gold
- `ShopScreen.java` - Pricing
- Individual monster files in `monsters/exordium/`, `monsters/city/`, `monsters/beyond/`, `monsters/ending/`
- Event files in `events/exordium/`, `events/city/`, `events/beyond/`, `events/shrines/`

All checks follow the pattern:
```java
if (AbstractDungeon.ascensionLevel >= X) {
    // Apply modifier
}
```

Where X is the ascension level at which the modifier activates.
