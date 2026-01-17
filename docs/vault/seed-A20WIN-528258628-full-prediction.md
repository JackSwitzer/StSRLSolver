# Full Game Prediction: Seed A20WIN (528258628)

**Ascension:** 20
**Character:** Watcher
**Neow Option:** HUNDRED_GOLD

---


## Ascension 20 Modifiers

| Ascension | Effect |
|-----------|--------|
| A1 | +1 elite per act |
| A2 | +1 enemy damage (weak monsters) |
| A3 | -1 potion slot |
| A4 | +1 enemy damage (elites) |
| A5 | Heal only 75% at rest |
| A6 | Start with Ascender's Bane curse |
| A7 | +7% enemy HP |
| A8 | +1 enemy damage (bosses) |
| A9 | +10% boss HP |
| A10 | Start with 10% less gold |
| A11 | Start at 90% HP |
| A12 | -50% card upgrade chance |
| A13 | Reduced gold drops |
| A14 | More ? room combats |
| A15 | +1 elite per act (total +2) |
| A16 | +7% more enemy HP (total +14%) |
| A17 | No healing between acts |
| A18 | -1 potion slot (total -2) |
| A19 | +1 enemy damage (all) |
| A20 | Double boss fight at end |


---

# Act 1: Exordium

## Map
```
        /     /   \        
14     R     R     R       
       | \ / | \ /         
13     M  ?  M  E          
       |/ |    \| \        
12     M  ?     M  M       
       | \|     |    \     
11     M  R     M     E    
         \  \ / |       \  
10        ?  M  ?        M 
          |/ | \|      /   
 9        E  M  E     E    
        / |/    | \     \  
 8     T  T     T  T     T 
       |/       |  |   /   
 7     M        M  M  M    
       |        |  |  |    
 6     E        ?  M  $    
       |        |    \|    
 5     R        R     E    
       | \        \ /   \  
 4     M  M        M     ? 
       |    \      |   /   
 3     M     M     M  ?    
       |   /       | \|    
 2     $  M        ?  $    
       | \|        |/ |    
 1     M  ?        M  ?    
       |/        /      \  
 0     M        M        M 
```

**Room Counts:**
- ELITE: 7
- EVENT: 10
- MONSTER: 31
- REST: 10
- SHOP: 3
- TREASURE: 7

## Encounters (Pre-Generated)

### Normal Encounters
| # | Encounter | Type |
|---|-----------|------|
| 1 | Jaw Worm | WEAK |
| 2 | Small Slimes | WEAK |
| 3 | Cultist | WEAK |
| 4 | Blue Slaver | STRONG |
| 5 | 3 Louse | STRONG |
| 6 | Looter | STRONG |
| 7 | Large Slime | STRONG |
| 8 | 2 Fungi Beasts | STRONG |
| 9 | Looter | STRONG |
| 10 | Exordium Thugs | STRONG |
| 11 | Gremlin Gang | STRONG |
| 12 | Lots of Slimes | STRONG |
| 13 | Exordium Thugs | STRONG |
| 14 | 3 Louse | STRONG |
| 15 | Red Slaver | STRONG |

### Elite Encounters
| # | Encounter |
|---|-----------|
| 1 | 3 Sentries |
| 2 | Gremlin Nob |
| 3 | Lagavulin |
| 4 | Gremlin Nob |
| 5 | Lagavulin |
| 6 | Gremlin Nob |
| 7 | Lagavulin |
| 8 | 3 Sentries |
| 9 | Gremlin Nob |
| 10 | Lagavulin |

## Floor-by-Floor Details

### Floor 1 - MONSTER
**Enemy:** Jaw Worm
- HP: 46
- **First Move:** Chomp [ATTACK] (11 dmg)
- *11 dmg (12 at A2+)*
**Gold:** 15
**Cards:** Protect (C), Follow-Up (C), Third Eye (C)

### Floor 2 - MONSTER
**Enemy:** Small Slimes
**Gold:** 15
**Cards:** Indignation (U), Fear No Evil (U), Evaluate (C)

### Floor 3 - SHOP
**Shop Contents:**
- Cards: Third Eye (51g), Sash Whip (49g), Simmering Fury (77g), Fasting (76g), Omniscience (151g)
- Colorless: Flash of Steel (94g), Purity (86g)
- Relics: Sundial (257g), Self Forming Clay (237g), Hand Drill (147g)
- Potions: Power Potion (48g), Energy Potion (50g), Essence of Steel (71g)
- Card Remove: 75g
*Shop - generates cards (12+ cardRng calls), relics, potions*

### Floor 4 - MONSTER
**Enemy:** Cultist
- HP: 57
- **First Move:** Incantation [BUFF]
- *Gains 3 Ritual (+3 Str/turn)*
**Gold:** 15
**Cards:** Crescendo (C), Crush Joints (C), Tranquility (C)

### Floor 5 - MONSTER
**Enemy:** Blue Slaver
- HP: 56
- **First Move:** Stab [ATTACK] (12 dmg)
- *12 dmg (13 at A2+)*
**Gold:** 15
**Cards:** Bowling Bash (C), Pressure Points (C), Meditate (U)
**Potion:** Swift Potion (COMMON)

### Floor 6 - REST
*Rest site - Rest (heal 30% or 22.5% at A5+), Upgrade, Smith, Dig, Toke, Lift*

### Floor 7 - ELITE
**Enemy:** 3 Sentries
- **Pattern:** Bolt x2 + Beam
- *9 dmg each. Beam adds Dazed. Alternates.*
**Gold:** 30
**Cards:** Omniscience (R), Flurry of Blows (C), Alpha (R)
**Potion:** Fruit Juice (RARE)
**Relic:** Peace Pipe (RARE)

### Floor 8 - MONSTER
**Enemy:** 3 Louse
**Gold:** 15
**Cards:** Deceive Reality (U), Empty Body (C), Halt (C)
**Potion:** Liquid Bronze (UNCOMMON)

### Floor 9 - TREASURE
*Treasure chest - uses treasureRng only*

### Floor 10 - ELITE
**Enemy:** Gremlin Nob
- HP: 98
- **First Move:** Bellow [BUFF]
- *Gains 2 Enrage (2 Str per Skill you play)*
**Gold:** 30
**Cards:** Crush Joints (C), Crescendo (C), Consecrate (C)
**Potion:** Stance Potion (UNCOMMON)
**Relic:** Bottled Lightning (UNCOMMON)

### Floor 11 - EVENT
*Event room - uses eventRng/miscRng (most don't affect cardRng)*

### Floor 12 - MONSTER
**Enemy:** Looter
- HP: 51
- **First Move:** Mug [ATTACK] (10 dmg)
- *Steals 15 gold*
**Gold:** 15
**Cards:** Crush Joints (C), Crescendo (C), Consecrate (C)

### Floor 13 - MONSTER
**Enemy:** Large Slime
**Gold:** 15
**Cards:** Sash Whip (C), Tranquility (C), Reach Heaven (U)

### Floor 14 - MONSTER
**Enemy:** 2 Fungi Beasts
**Gold:** 15
**Cards:** Brilliance (R), Sanctity (U), Conclude (U)

### Floor 15 - REST
*Rest site - Rest (heal 30% or 22.5% at A5+), Upgrade, Smith, Dig, Toke, Lift*

## Boss
### Slime Boss (154 HP)
- **First Move:** Goop Spray [DEBUFF]
- Shuffles 3 Slimed into deck. Splits at 50% HP.

**Boss Relic Choices:**
1. Astrolabe
2. Black Star
3. Tiny House

---

# Act 2: The City

## Map
```
        /     / |       \  
14     R     R  R        R 
         \   | \|      /   
13        M  M  ?     $    
            \  \| \   |    
12           E  ?  M  ?    
           /  / | \|    \  
11        R  M  M  ?     M 
        /  /    |/ |     | 
10     M  E     E  M     E 
       |/     /      \ /   
 9     M     M        R    
         \   | \    / |    
 8        T  T  T  T  T    
        / |    \|/  /      
 7     M  M     E  ?       
         \  \ / |    \     
 6        ?  M  $     E    
            \|/   \     \  
 5           E     R     R 
           / | \   |     | 
 4        M  M  M  ?     M 
        /  / |  |/       | 
 3     M  M  M  ?        M 
       |    \  \|      /   
 2     M     ?  M     ?    
         \ /  / | \ /      
 1        ?  M  M  $       
        /    |  |/         
 0     M     M  M          
```

**Room Counts:**
- ELITE: 7
- EVENT: 11
- MONSTER: 32
- REST: 11
- SHOP: 3
- TREASURE: 7

## Encounters (Pre-Generated)

### Normal Encounters
| # | Encounter | Type |
|---|-----------|------|
| 1 | Shell Parasite | WEAK |
| 2 | 2 Thieves | WEAK |
| 3 | 3 Cultists | STRONG |
| 4 | Snecko | STRONG |
| 5 | Centurion and Healer | STRONG |
| 6 | Snake Plant | STRONG |
| 7 | 3 Cultists | STRONG |
| 8 | Shelled Parasite and Fungi | STRONG |
| 9 | 3 Darklings | STRONG |
| 10 | 3 Cultists | STRONG |
| 11 | Centurion and Healer | STRONG |
| 12 | Chosen and Byrds | STRONG |
| 13 | Cultist and Chosen | STRONG |
| 14 | Snake Plant | STRONG |

### Elite Encounters
| # | Encounter |
|---|-----------|
| 1 | Gremlin Leader |
| 2 | Slavers |
| 3 | Gremlin Leader |
| 4 | Slavers |
| 5 | Book of Stabbing |
| 6 | Gremlin Leader |
| 7 | Slavers |
| 8 | Gremlin Leader |
| 9 | Slavers |
| 10 | Gremlin Leader |

## Floor-by-Floor Details

### Floor 1 - MONSTER
**Enemy:** Shell Parasite
- HP: 80
- **First Move:** Double Strike [ATTACK] (7 dmg)
- *7x2 dmg. Applies Frail with Suck.*
**Gold:** 15
**Cards:** Protect (C), Flying Sleeves (C), Prostrate+ (C)

### Floor 2 - EVENT
*Event room - uses eventRng/miscRng (most don't affect cardRng)*

### Floor 3 - MONSTER
**Enemy:** 2 Thieves
**Gold:** 15
**Cards:** Flying Sleeves (C), Empty Body (C), Rushdown (U)

### Floor 4 - MONSTER
**Enemy:** 3 Cultists
**Gold:** 15
**Cards:** Meditate (U), Talk to the Hand (U), Conclude (U)
**Potion:** Dexterity Potion (COMMON)

### Floor 5 - MONSTER
**Enemy:** Snecko
- HP: 133
- **First Move:** Perplexing Glare [DEBUFF]
- *Applies 2 Confused (random card costs 0-3)*
**Gold:** 15
**Cards:** Windmill Strike (U), Third Eye (C), Fasting (U)
**Potion:** Essence of Steel (UNCOMMON)

### Floor 6 - ELITE
**Enemy:** Gremlin Leader
- HP: 165
- **First Move:** Encourage [BUFF]
- *Summons gremlins. All gain 3 Str and 6 Block.*
**Gold:** 30
**Cards:** Empty Body (C), Like Water (U), Sash Whip (C)
**Relic:** Shuriken (UNCOMMON)

### Floor 7 - EVENT
*Event room - uses eventRng/miscRng (most don't affect cardRng)*

### Floor 8 - MONSTER
**Enemy:** Centurion and Healer
**Gold:** 15
**Cards:** Consecrate (C), Evaluate (C), Swivel (U)
**Potion:** Gambler's Brew (UNCOMMON)

### Floor 9 - TREASURE
*Treasure chest - uses treasureRng only*

### Floor 10 - MONSTER
**Enemy:** Snake Plant
- HP: 85
- **First Move:** Chomp [ATTACK] (7 dmg)
- *7x3 dmg. Applies 2 Weak with Enfeebling Spores.*
**Gold:** 15
**Cards:** Establishment (R), Wreath of Flame (U), Wheel Kick (U)
**Potion:** Power Potion (COMMON)

### Floor 11 - MONSTER
**Enemy:** 3 Cultists
**Gold:** 15
**Cards:** Pressure Points (C), Carve Reality+ (U), Sash Whip (C)
**Potion:** Liquid Memories (UNCOMMON)

### Floor 12 - REST
*Rest site - Rest (heal 30% or 22.5% at A5+), Upgrade, Smith, Dig, Toke, Lift*

### Floor 13 - ELITE
**Enemy:** Slavers
- **Pattern:** Mixed attacks
- *Blue: Stab 12. Red: Stab 13. Taskmaster: Whip 7.*
**Gold:** 30
**Cards:** Talk to the Hand (U), Empty Fist (C), Bowling Bash (C)
**Relic:** Snecko Skull (COMMON)

### Floor 14 - MONSTER
**Enemy:** Shelled Parasite and Fungi
**Gold:** 15
**Cards:** Wave of the Hand (U), Sash Whip (C), Pressure Points+ (C)
**Potion:** Block Potion (COMMON)

### Floor 15 - REST
*Rest site - Rest (heal 30% or 22.5% at A5+), Upgrade, Smith, Dig, Toke, Lift*

## Boss
### The Champ (462 HP)
- **First Move:** Defensive Stance [DEFEND]
- Gains 15-18 Block. At 50% HP executes everyone.

**Boss Relic Choices:**
1. Mark of Pain
2. Sacred Bark
3. Hovering Kite

---

# Act 3: The Beyond

## Map
```
        /     / |    \     
14     R     R  R     R    
       | \   |/       | \  
13     ?  M  $        M  M 
         \|/   \      |  | 
12        ?     ?     R  R 
        / | \     \ /    | 
11     ?  M  ?     ?     M 
       |/  /     /   \   | 
10     M  M     E     M  M 
         \  \ /       |  | 
 9        E  R        R  M 
        / |/   \      |/   
 8     T  T     T     T    
       |/   \     \ / |    
 7     M     M     M  R    
       | \   |   /   \  \  
 6     M  M  E  M     M  E 
       |    \  \  \     \| 
 5     E     E  R  E     R 
       |     |/  /     / | 
 4     M     ?  M     ?  M 
         \ / |    \ /    | 
 3        M  ?     M     ? 
        / |/       | \ /   
 2     M  ?        M  ?    
       |/ |        |/ |    
 1     ?  $        M  M    
       |  |      /    |    
 0     M  M     M     M    
```

**Room Counts:**
- ELITE: 7
- EVENT: 13
- MONSTER: 33
- REST: 14
- SHOP: 2
- TREASURE: 7

## Encounters (Pre-Generated)

### Normal Encounters
| # | Encounter | Type |
|---|-----------|------|
| 1 | Orb Walker | WEAK |
| 2 | 3 Shapes | WEAK |
| 3 | 4 Shapes | STRONG |
| 4 | Maw | STRONG |
| 5 | Writhing Mass | STRONG |
| 6 | Jaw Worm Horde | STRONG |
| 7 | Maw | STRONG |
| 8 | Writhing Mass | STRONG |
| 9 | 4 Shapes | STRONG |
| 10 | Spire Growth | STRONG |
| 11 | Writhing Mass | STRONG |
| 12 | Transient | STRONG |
| 13 | Spire Growth | STRONG |
| 14 | Jaw Worm Horde | STRONG |

### Elite Encounters
| # | Encounter |
|---|-----------|
| 1 | Giant Head |
| 2 | Nemesis |
| 3 | Giant Head |
| 4 | Nemesis |
| 5 | Reptomancer |
| 6 | Giant Head |
| 7 | Nemesis |
| 8 | Giant Head |
| 9 | Nemesis |
| 10 | Giant Head |

## Floor-by-Floor Details

### Floor 1 - MONSTER
**Enemy:** Orb Walker
- HP: 108
- **First Move:** Claw [ATTACK] (15 dmg)
- *Burns a card in hand.*
**Gold:** 15
**Cards:** Windmill Strike+ (U), Pressure Points+ (C), Tranquility+ (C)
**Potion:** Liquid Bronze (UNCOMMON)

### Floor 2 - EVENT
*Event room - uses eventRng/miscRng (most don't affect cardRng)*

### Floor 3 - MONSTER
**Enemy:** 3 Shapes
**Gold:** 15
**Cards:** Crush Joints (C), Talk to the Hand (U), Follow-Up (C)
**Potion:** Bottled Miracle (COMMON)

### Floor 4 - MONSTER
**Enemy:** 4 Shapes
**Gold:** 15
**Cards:** Fasting (U), Empty Mind (U), Carve Reality (U)

### Floor 5 - MONSTER
**Enemy:** Maw
- HP: 343
- **First Move:** Slam [ATTACK] (25 dmg)
- *25 dmg. Roar weakens. HUNGRY doubles damage when low HP.*
**Gold:** 15
**Cards:** Protect (C), Empty Fist (C), Third Eye (C)

### Floor 6 - ELITE
**Enemy:** Giant Head
- HP: 583
- **First Move:** Glare [DEBUFF]
- *Applies 1 Weak. Slow debuff. 35-40 dmg attacks.*
**Gold:** 30
**Cards:** Scrawl (R), Worship (U), Sanctity (U)
**Potion:** Fire Potion (COMMON)
**Relic:** Pear (UNCOMMON)

### Floor 7 - MONSTER
**Enemy:** Writhing Mass
- HP: 182
- **First Move:** Implant [UNKNOWN]
- *Copies a card from your deck. Random intent each turn. Malleable (gains Block when hit).*
**Gold:** 15
**Cards:** Bowling Bash (C), Halt (C), Just Lucky (C)
**Potion:** Smoke Bomb (RARE)

### Floor 8 - MONSTER
**Enemy:** Jaw Worm Horde
**Gold:** 15
**Cards:** Talk to the Hand+ (U), Just Lucky+ (C), Nirvana (U)

### Floor 9 - TREASURE
*Treasure chest - uses treasureRng only*

### Floor 10 - ELITE
**Enemy:** Nemesis
- HP: 225
- **First Move:** Debuff [DEBUFF]
- *Burns cards. Goes intangible (1 dmg max). 45 dmg Scythe attack.*
**Gold:** 30
**Cards:** Evaluate (C), Wish (R), Halt+ (C)
**Potion:** Explosive Potion (COMMON)
**Relic:** Ink Bottle (UNCOMMON)

### Floor 11 - MONSTER
**Enemy:** Maw
- HP: 343
- **First Move:** Slam [ATTACK] (25 dmg)
- *25 dmg. Roar weakens. HUNGRY doubles damage when low HP.*
**Gold:** 15
**Cards:** Tranquility (C), Follow-Up (C), Consecrate (C)

### Floor 12 - EVENT
*Event room - uses eventRng/miscRng (most don't affect cardRng)*

### Floor 13 - EVENT
*Event room - uses eventRng/miscRng (most don't affect cardRng)*

### Floor 14 - EVENT
*Event room - uses eventRng/miscRng (most don't affect cardRng)*

### Floor 15 - REST
*Rest site - Rest (heal 30% or 22.5% at A5+), Upgrade, Smith, Dig, Toke, Lift*

## Boss
### Awakened One (330 HP)
- **First Move:** Slash [ATTACK]
- 10 damage. Gains Curiosity (2 Str when you play Power). Phase 2: Reborn with 200 HP.

**Boss Relic Choices:**
1. Runic Pyramid
2. Inserter
3. Busted Crown

---

# Act 4: The Ending

## Map
```
Rest -> Shop -> Elite (Shield & Spear) -> Boss (Heart)
```

**Room Counts:**
- BOSS: 1
- ELITE: 1
- REST: 1
- SHOP: 1

## Encounters (Pre-Generated)

### Elite Encounters
| # | Encounter |
|---|-----------|
| 1 | Shield and Spear |

## Floor-by-Floor Details

### Floor 1 - REST
*Rest site - Rest (heal 30% or 22.5% at A5+), Upgrade, Smith, Dig, Toke, Lift*

### Floor 2 - SHOP
**Shop Contents:**
- Cards: Flurry of Blows (51g), Crush Joints (51g), Weave (75g), Fear No Evil (71g), Spirit Shield (144g)
- Colorless: Purity (91g), Hand of Greed (172g)
- Relics: Ninja Scroll (239g), Bottled Tornado (253g), Chemical X (147g)
- Potions: Weak Potion (48g), Liquid Bronze (77g), Attack Potion (48g)
- Card Remove: 75g
*Shop - generates cards (12+ cardRng calls), relics, potions*

### Floor 3 - ELITE
**Enemy:** Shield and Spear
- **Pattern:** Shield defends, Spear attacks
- *Shield: gains 30 Block. Spear: Strong Stab 30 dmg.*
**Gold:** 30
**Cards:** Worship (U), Sanctity (U), Bowling Bash (C)
**Relic:** Regal Pillow (COMMON)

### Floor 4 - BOSS
**Enemy:** Boss
**Cards:** Like Water+ (U), Just Lucky (C), Empty Mind (U)

## Boss
### Corrupt Heart (880 HP)
- **First Move:** Debilitate [DEBUFF]
- Vulnerable 2, Weak 2, Frail 2. Gains 2 Invincible (max 300 dmg/turn). Beat of Death damages you per card.

---

## Verification Checklist

For each act, verify:
- [ ] Floor 1-3 encounters match
- [ ] Enemy HP values match
- [ ] Card rewards match
- [ ] Gold amounts match
- [ ] Elite relic drops match
- [ ] Boss matches
- [ ] Boss relic choices match

**Known Limitations:**
- Shop visits shift cardRng for subsequent floors
- Event choices may consume various RNG streams
- Multi-enemy HP requires fresh RNG calls per enemy
- Path through map affects actual room types encountered