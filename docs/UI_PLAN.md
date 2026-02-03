# Python Game UI Asset Plan

## Current Asset Audit

### Summary

| Category | Count | Format | Status |
|----------|-------|--------|--------|
| Relics | 232 | PNG 128x128 | Ready to use |
| Potions | 27 (layered) | PNG | Requires compositing |
| Enemies | 40 | PNG (Spine sheets) | Need frame extraction |
| UI Elements | 20+ | PNG | Ready to use |
| Intent Icons | 39 | PNG | Ready to use |
| Card UI | 4 atlases | PNG | Need sprite extraction |
| Powers | 1 atlas | PNG + atlas | Need sprite extraction |

### Detailed Asset Inventory

#### Relics (`assets/relics/`)
- **Count**: 232 PNG files
- **Dimensions**: 128x128 pixels
- **Status**: Direct use - each PNG is a complete relic icon
- **Frames available**: Common, Uncommon, Rare, Boss (`assets/ui/relicFrame*.png`)

#### Potions (`assets/potions/`)
- **Count**: 27 files (5 sizes x 5 layers + 2 misc)
- **Sizes**: h=huge, l=large, m=medium, s=small, t=tiny
- **Layers**: glass, liquid, outline, spots, hybrid
- **Status**: Requires composite rendering (layer glass + liquid + outline + spots)
- **Workaround**: Use `potion_*_hybrid.png` for single-file rendering

#### Enemies (`assets/enemies/`)
- **Count**: 40 sprite sheets
- **Format**: Spine animation sprite sheets
- **Status**: Need frame extraction for static display
- **Files by Act**:
  - Act 1 Monsters: cultist, jawWorm, jawWormAlt, louseGreen, louseRed, slimeS/M/L, slimeAltS/M/L, angryGremlin, fatGremlin, femaleGremlin, thiefGremlin, wizardGremlin, blueSlaver, redSlaver, looter, sentry
  - Act 1 Elites: nobGremlin, Lagavulin
  - Act 1 Bosses: guardian, slime
  - Act 2 Monsters: automaton, bookOfStabbing, looterAlt, slaverMaster, torchHead
  - Act 2 Bosses: collector
  - Act 3 Monsters: exploder, maw, repulser, spiker, head, transient
  - Act 3 Bosses: AwakenedOne, TimeEater, Donu, Nemesis

#### UI Elements (`assets/ui/`)
- **Resources**: gold.png, panelHeart.png, bar.png, block.png
- **Energy Orb**: 11 layers for animated orb
- **Relic Frames**: Common, Uncommon, Rare, Boss frames

#### Intent Icons (`assets/ui/intent/`)
- **Attack**: 7 tiers (attack_intent_1.png through attack_intent_7.png)
- **Buff**: buff1.png, buff1L.png
- **Debuff**: debuff1.png, debuff1L.png, debuff2.png, debuff2L.png
- **Defend**: defend.png, defendL.png, defendBuff.png, defendBuffL.png
- **Special**: attackBuff.png, attackDebuff.png, attackDefend.png, escape.png, magic.png, sleep.png, special.png, stun.png, unknown.png
- **Status**: Ready for direct use

#### Cards (`assets/cards/`)
- **UI Atlases**: cardui.png through cardui4.png
- **Locked Cards**: locked_attack.png, locked_skill.png, locked_power.png (+ large variants)
- **Tiny Cards**: portrait frames, banner, cardBack, descBox
- **Status**: Atlas files need sprite extraction (no .atlas file available)

#### Powers (`assets/powers/`)
- **Files**: powers.png (atlas image), powers.atlas (coordinate data)
- **Status**: Extractable using powers.atlas coordinates

---

## Phase 1 - Minimal Viable UI

### Goal
Get a functional combat visualization working with minimal new asset creation.

### Strategy
1. Use existing assets where possible
2. Create simple SVG sprites for enemies (not Spine-dependent)
3. Text-based card rendering
4. Simple HP bars and UI

### Required New Assets

#### 1. Player Sprite - Watcher
- **Format**: SVG, 64x64
- **Style**: Simple pixel art silhouette
- **Color palette**: Purple/violet (Watcher's color)
- **Poses**: Standing (single static pose for MVP)

#### 2. Enemy Sprites (Priority Tier System)

**Tier 1 - Act 1 Core (implement first)**
| Enemy | Suggested Style |
|-------|-----------------|
| Jaw Worm | Green worm, simple curves |
| Cultist | Hooded figure, simple robe |
| Acid Slime (S/M/L) | Green blob with face |
| Spike Slime (S/M/L) | Purple blob with spikes |
| Louse (Red/Green) | Round bug with legs |

**Tier 2 - Act 1 Complete**
| Enemy | Suggested Style |
|-------|-----------------|
| Fungus Beast | Mushroom creature |
| Gremlins (all 5 types) | Small goblin variants |
| Blue Slaver | Humanoid with whip |
| Red Slaver | Humanoid with whip (red) |
| Looter | Bandit figure |

**Tier 3 - Act 1 Elites**
| Enemy | Suggested Style |
|-------|-----------------|
| Gremlin Nob | Large muscular gremlin |
| Lagavulin | Armored sleeping giant |
| Sentries | Three floating orbs |

**Tier 4 - Act 1 Bosses**
| Enemy | Suggested Style |
|-------|-----------------|
| Slime Boss | Massive green slime |
| The Guardian | Mechanical construct |
| Hexaghost | Six-headed ghost |

### Card Rendering (Text-Based)
```
+------------------+
| [2] Card Name    |  <- Cost in brackets, name
|                  |
| Attack           |  <- Card type
|                  |
| Deal 8 damage.   |  <- Effect text
+------------------+
```

- **Frame colors**:
  - Attack: Red border
  - Skill: Green border
  - Power: Blue border
  - Status/Curse: Gray border
- **Upgrade indicator**: "+" suffix on name, gold border

### UI Components (Use Existing)

| Component | Asset |
|-----------|-------|
| HP Icon | `assets/ui/panelHeart.png` |
| Block Icon | `assets/ui/block.png` |
| Gold Icon | `assets/ui/gold.png` |
| Intent Icons | `assets/ui/intent/*.png` |
| Relic Icons | `assets/relics/*.png` |
| Relic Frames | `assets/ui/relicFrame*.png` |

### Combat Layout
```
+------------------------------------------+
|  HP: 70/80  Block: 5  Energy: 3/3        |
+------------------------------------------+
|                                          |
|  [Enemy Area]                            |
|    Jaw Worm (42/44)                      |
|    Intent: Attack 11                     |
|                                          |
+------------------------------------------+
|  [Player Area]                           |
|    Watcher                               |
|    Stance: Calm                          |
+------------------------------------------+
|  [Hand]                                  |
|  [Card1] [Card2] [Card3] [Card4] [Card5] |
+------------------------------------------+
|  [Relics] [Potions]                      |
+------------------------------------------+
```

---

## Phase 2 - Enhanced UI

### Map Visualization
- Node-based graph display
- Node types: Monster, Elite, Rest, Shop, Event, Treasure, Boss
- Path connections with branching
- Current position indicator

### Combat Enhancements
- Animated HP bars (smooth decrease)
- Damage numbers (floating text)
- Status effect icons
- Turn indicator
- Draw/Discard pile counts

### Deck Viewer
- Grid view of all cards
- Filter by type (Attack/Skill/Power)
- Search functionality
- Upgrade status display

### Side-by-Side Java Comparison
- Split view: Python state | Java game state
- Diff highlighting for mismatches
- Useful for validation during development

---

## Pixel Art Style Guide

### Dimensions
- **Sprites**: 64x64 pixels (scalable)
- **Small icons**: 32x32 pixels
- **Large bosses**: 128x128 pixels

### Color Palette (Slay the Spire Aesthetic)

**Primary Colors**
| Color | Hex | Use |
|-------|-----|-----|
| Blood Red | #8B0000 | Attacks, Ironclad |
| Deep Gold | #DAA520 | Highlights, coins |
| Shadow Black | #1A1A2E | Backgrounds, outlines |
| Bone White | #F5F5DC | Text, highlights |

**Character Colors**
| Character | Primary | Secondary |
|-----------|---------|-----------|
| Watcher | #7B68EE (Purple) | #DDA0DD (Lavender) |
| Ironclad | #8B0000 (Red) | #CD5C5C (Indian Red) |
| Silent | #228B22 (Green) | #98FB98 (Pale Green) |
| Defect | #4169E1 (Blue) | #87CEEB (Sky Blue) |

**Enemy Colors**
| Enemy Type | Primary | Secondary |
|------------|---------|-----------|
| Slimes | #32CD32 (Lime) | #006400 (Dark Green) |
| Cultists | #8B4513 (Brown) | #800080 (Purple) |
| Gremlins | #808000 (Olive) | #556B2F (Dark Olive) |
| Elites | #FFD700 (Gold outline) | varies |
| Bosses | #FF4500 (Orange Red outline) | varies |

### SVG Guidelines
- Use `viewBox="0 0 64 64"` for standard sprites
- Keep stroke widths consistent (2px for outlines)
- Use simple shapes (circles, rectangles, paths)
- Avoid gradients for pixel art feel
- Include `fill` and `stroke` for all elements

### Example SVG Template
```svg
<svg viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
  <!-- Background/Base -->
  <rect x="16" y="16" width="32" height="40" fill="#32CD32" stroke="#006400" stroke-width="2"/>
  <!-- Face/Details -->
  <circle cx="24" cy="28" r="4" fill="#1A1A2E"/>
  <circle cx="40" cy="28" r="4" fill="#1A1A2E"/>
</svg>
```

---

## Complete Enemy List (from packages/engine/content/enemies.py)

### Act 1 - The Exordium

**Normal Enemies**
1. JawWorm
2. Cultist
3. AcidSlimeM (Medium)
4. AcidSlimeL (Large)
5. AcidSlimeS (Small)
6. SpikeSlimeM (Medium)
7. SpikeSlimeL (Large)
8. SpikeSlimeS (Small)
9. Louse (base class)
10. LouseNormal (Red/Green variants)
11. LouseDefensive
12. FungiBeast

**Gremlins (appear together)**
13. GremlinFat
14. GremlinThief
15. GremlinTsundere
16. GremlinWarrior
17. GremlinWizard

**Slavers**
18. SlaverBlue
19. SlaverRed

**Other**
20. Looter

**Elites**
21. GremlinNob
22. Lagavulin
23. Sentries

**Bosses**
24. SlimeBoss
25. TheGuardian
26. Hexaghost

### Act 2 - The City

**Normal Enemies**
27. Chosen
28. Byrd
29. Centurion
30. Healer
31. Snecko
32. SnakePlant
33. Mugger
34. Taskmaster
35. ShelledParasite
36. SphericGuardian

**Bandits**
37. BanditBear
38. BanditLeader
39. BanditPointy

**Elites**
40. GremlinLeader
41. BookOfStabbing
42. Taskmaster

**Bosses**
43. Champ
44. TheCollector
45. BronzeAutomaton

### Act 3 - The Beyond

**Normal Enemies**
46. Maw
47. Darkling
48. OrbWalker
49. Spiker
50. Repulsor
51. WrithingMass
52. Transient
53. Exploder
54. SpireGrowth
55. SnakeDagger

**Elites**
56. GiantHead
57. Nemesis
58. Reptomancer

**Bosses**
59. AwakenedOne
60. TimeEater
61. Donu
62. Deca

### Act 4 - The Spire

**Elites**
63. SpireShield
64. SpireSpear

**Boss**
65. CorruptHeart

### Minions/Summons
66. TorchHead (Bronze Automaton summon)
67. BronzeOrb (Bronze Automaton summon)

---

## Asset Priority Matrix

| Priority | Asset Type | Count | Effort | Impact |
|----------|-----------|-------|--------|--------|
| P0 | Intent icons | 0 (have) | None | High |
| P0 | Relic icons | 0 (have) | None | High |
| P0 | HP/Block/Gold UI | 0 (have) | None | High |
| P1 | Watcher sprite | 1 | Low | High |
| P1 | Act 1 basic enemies | 10 | Medium | High |
| P2 | Act 1 elites | 3 | Medium | Medium |
| P2 | Act 1 bosses | 3 | High | Medium |
| P3 | Act 2 enemies | 15 | High | Low |
| P4 | Act 3+ enemies | 20+ | High | Low |

---

## Implementation Checklist

### Phase 1 Checklist
- [ ] Create Watcher SVG sprite (64x64)
- [ ] Create 10 basic Act 1 enemy SVGs
- [ ] Implement text-based card renderer
- [ ] Create combat layout with existing UI assets
- [ ] Wire up intent icons to enemy state
- [ ] Display relics with frames
- [ ] Basic HP/Block display

### Phase 2 Checklist
- [ ] Map visualization component
- [ ] Deck viewer modal
- [ ] Status effect icons
- [ ] Damage number animations
- [ ] Java state comparison view
- [ ] Act 1 elite sprites
- [ ] Act 1 boss sprites

### Future Considerations
- Extract frames from Spine sprite sheets
- Power icon extraction from atlas
- Card art extraction from atlases
- Animation support for sprites
- Sound effects integration
