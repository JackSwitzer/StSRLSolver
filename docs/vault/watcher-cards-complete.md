# Watcher Card Data - Complete Reference

Extracted from decompiled source: `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/`

## Table of Contents
- [Basic Cards](#basic-cards)
- [Common Cards](#common-cards)
- [Uncommon Cards](#uncommon-cards)
- [Rare Cards](#rare-cards)
- [Generated/Temp Cards](#generatedtemp-cards)
- [Stance Interaction Reference](#stance-interaction-reference)
- [Mantra Interaction Reference](#mantra-interaction-reference)
- [Scry Interaction Reference](#scry-interaction-reference)
- [Retain Interaction Reference](#retain-interaction-reference)

---

## Basic Cards

### Strike (Strike_P)
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Basic | Basic |
| Target | Enemy | Enemy |
| Damage | 6 | 9 |

**Effect:** Deal damage to target enemy.
**Tags:** STRIKE, STARTER_STRIKE

---

### Defend (Defend_P)
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Basic | Basic |
| Target | Self | Self |
| Block | 5 | 8 |

**Effect:** Gain block.
**Tags:** STARTER_DEFEND

---

### Eruption
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 2 | 1 |
| Rarity | Basic | Basic |
| Target | Enemy | Enemy |
| Damage | 9 | 9 |

**Effect:** Deal damage. Enter Wrath stance.
**Stance:** Enters Wrath

---

### Vigilance
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 2 | 2 |
| Rarity | Basic | Basic |
| Target | Self | Self |
| Block | 8 | 12 |

**Effect:** Gain block. Enter Calm stance.
**Stance:** Enters Calm

---

## Common Cards

### Bowling Bash
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Common | Common |
| Target | Enemy | Enemy |
| Damage | 7 | 10 |

**Effect:** Deal damage to target enemy for each enemy in combat (hits same target multiple times based on enemy count).

---

### Consecrate
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 0 | 0 |
| Rarity | Common | Common |
| Target | All Enemies | All Enemies |
| Damage | 5 | 8 |

**Effect:** Deal damage to ALL enemies.
**Multi-damage:** Yes

---

### Crescendo
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 0 |
| Rarity | Common | Common |
| Target | Self | Self |

**Effect:** Enter Wrath stance.
**Keywords:** Exhaust, Retain
**Stance:** Enters Wrath

---

### Crush Joints
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Common | Common |
| Target | Enemy | Enemy |
| Damage | 8 | 10 |
| Magic Number | 1 | 2 |

**Effect:** Deal damage. If the last card played was a Skill, apply Vulnerable.
**Glow:** Gold when last card was a Skill

---

### Cut Through Fate
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Common | Common |
| Target | Enemy | Enemy |
| Damage | 7 | 9 |
| Magic Number (Scry) | 2 | 3 |

**Effect:** Deal damage. Scry X. Draw 1 card.
**Scry:** Yes (2/3)

---

### Empty Body
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Common | Common |
| Target | Self | Self |
| Block | 7 | 10 |

**Effect:** Gain block. Exit your current Stance.
**Tags:** EMPTY
**Stance:** Exits to Neutral

---

### Empty Fist
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Common | Common |
| Target | Enemy | Enemy |
| Damage | 9 | 14 |

**Effect:** Deal damage. Exit your current Stance.
**Tags:** EMPTY
**Stance:** Exits to Neutral

---

### Evaluate
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Common | Common |
| Target | Self | Self |
| Block | 6 | 10 |

**Effect:** Gain block. Shuffle an Insight into your draw pile.
**Creates:** Insight (in draw pile)

---

### Flurry of Blows
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 0 | 0 |
| Rarity | Common | Common |
| Target | Enemy | Enemy |
| Damage | 4 | 6 |

**Effect:** Deal damage. Whenever you change Stances, return this from discard to hand.
**Stance Trigger:** On ANY stance change, returns from discard pile to hand

---

### Flying Sleeves
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Common | Common |
| Target | Enemy | Enemy |
| Damage | 4x2 | 6x2 |

**Effect:** Retain. Deal damage twice.
**Keywords:** Retain

---

### Follow-Up
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Common | Common |
| Target | Enemy | Enemy |
| Damage | 7 | 11 |

**Effect:** Deal damage. If the last card played was an Attack, gain 1 Energy.
**Glow:** Gold when last card was an Attack

---

### Halt
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 0 | 0 |
| Rarity | Common | Common |
| Target | Self | Self |
| Block | 3 | 4 |
| Magic Number | 9 | 14 |

**Effect:** Gain block. If in Wrath, gain additional block (total = magicNumber when in Wrath).
**Stance:** Enhanced in Wrath (block = 3+6=9 base, 4+10=14 upgraded)

---

### Just Lucky
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 0 | 0 |
| Rarity | Common | Common |
| Target | Enemy | Enemy |
| Damage | 3 | 4 |
| Block | 2 | 3 |
| Magic Number (Scry) | 1 | 2 |

**Effect:** Scry X. Gain block. Deal damage.
**Scry:** Yes (1/2)

---

### Pressure Points (PathToVictory)
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Common | Common |
| Target | Enemy | Enemy |
| Magic Number (Mark) | 8 | 11 |

**Effect:** Apply Mark. Activate all Marks (deal damage equal to Mark to all enemies with Mark).
**Note:** Internal ID is "PathToVictory"

---

### Prostrate
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 0 | 0 |
| Rarity | Common | Common |
| Target | Self | Self |
| Block | 4 | 4 |
| Magic Number (Mantra) | 2 | 3 |

**Effect:** Gain Mantra. Gain block.
**Mantra:** Yes (2/3)

---

### Protect
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 2 | 2 |
| Rarity | Common | Common |
| Target | Self | Self |
| Block | 12 | 16 |

**Effect:** Retain. Gain block.
**Keywords:** Retain

---

### Sash Whip
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Common | Common |
| Target | Enemy | Enemy |
| Damage | 8 | 10 |
| Magic Number (Weak) | 1 | 2 |

**Effect:** Deal damage. If last card played was an Attack, apply Weak.
**Glow:** Gold when last card was an Attack

---

### Third Eye
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Common | Common |
| Target | Self | Self |
| Block | 7 | 9 |
| Magic Number (Scry) | 3 | 5 |

**Effect:** Gain block. Scry X.
**Scry:** Yes (3/5)

---

### Tranquility (ClearTheMind)
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 0 |
| Rarity | Common | Common |
| Target | Self | Self |

**Effect:** Retain. Enter Calm.
**Keywords:** Exhaust, Retain
**Stance:** Enters Calm
**Note:** Internal ID is "ClearTheMind"

---

## Uncommon Cards

### Battle Hymn
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Magic Number | 1 | 1 |

**Effect:** At the start of each turn, add a Smite to your hand.
**Upgrade:** Innate
**Creates:** Smite (in hand at turn start)

---

### Carve Reality
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Enemy | Enemy |
| Damage | 6 | 10 |

**Effect:** Deal damage. Add a Smite to your hand.
**Creates:** Smite (in hand)

---

### Collect
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | X | X |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |

**Effect:** Put X Miracles into your hand at the start of your next turn.
**Keywords:** Exhaust
**Upgrade:** Miracles are upgraded
**Creates:** Miracle+ (via CollectAction)

---

### Conclude
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | All Enemies | All Enemies |
| Damage | 12 | 16 |

**Effect:** Deal damage to ALL enemies. End your turn.
**Multi-damage:** Yes
**Special:** Ends turn immediately

---

### Deceive Reality
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Block | 4 | 7 |

**Effect:** Gain block. Add a Safety to your hand.
**Creates:** Safety (in hand)

---

### Empty Mind
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Magic Number (Draw) | 2 | 3 |

**Effect:** Draw X cards. Exit your current Stance.
**Tags:** EMPTY
**Stance:** Exits to Neutral

---

### Fasting (Fasting2)
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 2 | 2 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Magic Number | 3 | 4 |

**Effect:** Gain X Strength. Gain X Dexterity. Lose 1 Energy each turn.
**Note:** Internal ID is "Fasting2"

---

### Fear No Evil
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Enemy | Enemy |
| Damage | 8 | 11 |

**Effect:** Deal damage. If the enemy intends to Attack, enter Calm.
**Stance:** Conditionally enters Calm (if enemy attacking)

---

### Foreign Influence
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 0 | 0 |
| Rarity | Uncommon | Uncommon |
| Target | None | None |

**Effect:** Choose 1 of 3 Attack cards from other classes to add to your hand.
**Keywords:** Exhaust
**Upgrade:** Cards are upgraded

---

### Foresight (Wireheading)
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | None | None |
| Magic Number (Scry) | 3 | 4 |

**Effect:** At the start of your turn, Scry X.
**Scry:** Yes (start of turn)
**Note:** Internal ID is "Wireheading"

---

### Indignation
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | None | None |
| Magic Number (Vulnerable) | 3 | 5 |

**Effect:** If in Wrath, apply X Vulnerable to ALL enemies. Otherwise, enter Wrath.
**Stance:** Enters Wrath OR applies Vulnerable if already in Wrath

---

### Inner Peace
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Magic Number (Draw) | 3 | 4 |

**Effect:** If in Calm, draw X cards. Otherwise, enter Calm.
**Stance:** Enters Calm OR draws cards if already in Calm

---

### Like Water
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | None | None |
| Magic Number (Block) | 5 | 7 |

**Effect:** At the end of your turn, if you are in Calm, gain X Block.
**Stance:** Triggers in Calm

---

### Meditate
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | None | None |
| Magic Number (Cards) | 1 | 2 |

**Effect:** Put X cards from your discard pile into your hand and Retain them. Enter Calm. End your turn.
**Stance:** Enters Calm
**Special:** Ends turn immediately

---

### Mental Fortress
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Magic Number (Block) | 4 | 6 |

**Effect:** Whenever you change Stances, gain X Block.
**Stance Trigger:** On ANY stance change

---

### Nirvana
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Magic Number (Block) | 3 | 4 |

**Effect:** Whenever you Scry, gain X Block.
**Scry Trigger:** On scry

---

### Perseverance
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Block | 5 | 7 |
| Magic Number | 2 | 3 |

**Effect:** Retain. Gain block. When Retained, increase this card's Block by X.
**Keywords:** Retain
**Retain Effect:** +2/+3 block per retain

---

### Pray
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Magic Number (Mantra) | 3 | 4 |

**Effect:** Gain X Mantra. Shuffle an Insight into your draw pile.
**Mantra:** Yes (3/4)
**Creates:** Insight (in draw pile)

---

### Reach Heaven
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 2 | 2 |
| Rarity | Uncommon | Uncommon |
| Target | Enemy | Enemy |
| Damage | 10 | 15 |

**Effect:** Deal damage. Shuffle a Through Violence into your draw pile.
**Creates:** Through Violence (in draw pile)

---

### Rushdown (Adaptation)
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 1 | 0 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Magic Number (Draw) | 2 | 2 |

**Effect:** Whenever you enter Wrath, draw X cards.
**Stance Trigger:** On entering Wrath
**Note:** Internal ID is "Adaptation"

---

### Sanctity
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Block | 6 | 9 |
| Magic Number (Draw) | 2 | 2 |

**Effect:** Gain block. If the previous card played this turn was a Skill, draw X cards.
**Glow:** Gold when last card was a Skill

---

### Sands of Time
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 4 | 4 |
| Rarity | Uncommon | Uncommon |
| Target | Enemy | Enemy |
| Damage | 20 | 26 |

**Effect:** Retain. Deal damage. When Retained, reduce this card's cost by 1.
**Keywords:** Retain
**Retain Effect:** Cost reduces by 1 each retain

---

### Signature Move
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 2 | 2 |
| Rarity | Uncommon | Uncommon |
| Target | Enemy | Enemy |
| Damage | 30 | 40 |

**Effect:** Can only be played if this is the only Attack in your hand. Deal damage.
**Condition:** Only Attack in hand
**Glow:** Gold when condition met

---

### Simmering Fury (Vengeance)
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | None | None |
| Magic Number (Draw) | 2 | 3 |

**Effect:** At the start of your next turn, enter Wrath and draw X cards.
**Stance:** Enters Wrath (next turn)
**Note:** Internal ID is "Vengeance"

---

### Study
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 2 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Magic Number | 1 | 1 |

**Effect:** At the end of your turn, shuffle an Insight into your draw pile.
**Creates:** Insight (in draw pile at turn end)

---

### Swivel
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 2 | 2 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Block | 8 | 11 |

**Effect:** Gain block. Your next Attack this turn costs 0.
**Creates Power:** FreeAttackPower

---

### Talk to the Hand
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Enemy | Enemy |
| Damage | 5 | 7 |
| Magic Number (Block Return) | 2 | 3 |

**Effect:** Deal damage. Whenever you attack this enemy, gain X Block.
**Keywords:** Exhaust
**Creates Power:** BlockReturnPower on enemy

---

### Tantrum
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Enemy | Enemy |
| Damage | 3 | 3 |
| Magic Number (Hits) | 3 | 4 |

**Effect:** Deal damage X times. Enter Wrath. Shuffle this card into your draw pile.
**Stance:** Enters Wrath
**Special:** Shuffles back into draw pile

---

### Wallop
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 2 | 2 |
| Rarity | Uncommon | Uncommon |
| Target | Enemy | Enemy |
| Damage | 9 | 12 |

**Effect:** Deal damage. Gain Block equal to unblocked damage dealt.

---

### Wave of the Hand
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Magic Number (Weak) | 1 | 2 |

**Effect:** Whenever you gain Block this turn, apply X Weak to ALL enemies.
**Creates Power:** WaveOfTheHandPower (this turn)

---

### Weave
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 0 | 0 |
| Rarity | Uncommon | Uncommon |
| Target | Enemy | Enemy |
| Damage | 4 | 6 |

**Effect:** Deal damage. Whenever you Scry, return this from discard pile to your hand.
**Scry Trigger:** Returns from discard to hand on scry

---

### Wheel Kick
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 2 | 2 |
| Rarity | Uncommon | Uncommon |
| Target | Enemy | Enemy |
| Damage | 15 | 20 |
| Magic Number (Draw) | 2 | 2 |

**Effect:** Deal damage. Draw X cards.

---

### Windmill Strike
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 2 | 2 |
| Rarity | Uncommon | Uncommon |
| Target | Enemy | Enemy |
| Damage | 7 | 10 |
| Magic Number | 4 | 5 |

**Effect:** Retain. Deal damage. When Retained, increase this card's damage by X.
**Keywords:** Retain
**Tags:** STRIKE
**Retain Effect:** +4/+5 damage per retain

---

### Worship
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 2 | 2 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Magic Number (Mantra) | 5 | 5 |

**Effect:** Gain X Mantra.
**Upgrade:** Retain
**Mantra:** Yes (5)

---

### Wreath of Flame
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Uncommon | Uncommon |
| Target | Self | Self |
| Magic Number (Vigor) | 5 | 8 |

**Effect:** Gain X Vigor. (Your next Attack deals X additional damage.)

---

## Rare Cards

### Alpha
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Rare | Rare |
| Target | None | None |

**Effect:** Shuffle a Beta into your draw pile.
**Keywords:** Exhaust
**Upgrade:** Innate
**Creates:** Beta (in draw pile) -> Omega

---

### Blasphemy
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Rare | Rare |
| Target | Self | Self |

**Effect:** Enter Divinity. Die at the start of your next turn.
**Keywords:** Exhaust
**Upgrade:** Retain
**Stance:** Enters Divinity

---

### Brilliance
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Rare | Rare |
| Target | Enemy | Enemy |
| Damage | 12 | 16 |

**Effect:** Deal damage. Deals additional damage equal to Mantra gained this combat.
**Mantra Scaling:** +1 damage per Mantra gained this combat

---

### Conjure Blade
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | X | X |
| Rarity | Rare | Rare |
| Target | Self | Self |

**Effect:** Shuffle an Expunger with X+1 hits into your draw pile.
**Keywords:** Exhaust
**Upgrade:** X+2 hits instead of X+1
**Creates:** Expunger (in draw pile)

---

### Deus Ex Machina
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | Unplayable | Unplayable |
| Rarity | Rare | Rare |
| Target | Self | Self |
| Magic Number (Miracles) | 2 | 3 |

**Effect:** Unplayable. When drawn, add X Miracles to your hand. Exhaust.
**Keywords:** Exhaust
**Trigger:** When drawn
**Creates:** Miracle (in hand)

---

### Deva Form
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 3 | 3 |
| Rarity | Rare | Rare |
| Target | Self | Self |
| Magic Number | 1 | 1 |

**Effect:** Ethereal. At the start of each turn, gain Energy equal to the number of times this was played.
**Keywords:** Ethereal (base only)
**Upgrade:** Removes Ethereal

---

### Devotion
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 1 | 1 |
| Rarity | Rare | Rare |
| Target | None | None |
| Magic Number (Mantra) | 2 | 3 |

**Effect:** At the start of your turn, gain X Mantra.
**Mantra:** Yes (2/3 per turn)

---

### Discipline (DEPRECATED)
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 2 | 1 |
| Rarity | Rare | Rare |
| Target | Self | Self |

**Effect:** Uses deprecated power (DEPRECATEDDisciplinePower).
**Note:** This card uses deprecated functionality

---

### Establishment
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 1 | 1 |
| Rarity | Rare | Rare |
| Target | Self | Self |
| Magic Number | 1 | 1 |

**Effect:** Whenever a card is Retained, reduce its cost by 1.
**Upgrade:** Innate

---

### Judgement
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Rare | Rare |
| Target | Enemy | Enemy |
| Magic Number (HP Threshold) | 30 | 40 |

**Effect:** If the enemy has X or less HP, set their HP to 0.
**Note:** Instakill threshold

---

### Lesson Learned
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 2 | 2 |
| Rarity | Rare | Rare |
| Target | Enemy | Enemy |
| Damage | 10 | 13 |

**Effect:** Deal damage. If this kills the enemy, upgrade a random card in your deck.
**Keywords:** Exhaust
**Tags:** HEALING

---

### Master Reality
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 1 | 0 |
| Rarity | Rare | Rare |
| Target | Self | Self |

**Effect:** Whenever a card is created during combat, Upgrade it.

---

### Omniscience
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 4 | 3 |
| Rarity | Rare | Rare |
| Target | None | None |
| Magic Number | 2 | 2 |

**Effect:** Choose a card in your draw pile. Play the chosen card X times and Exhaust it.
**Keywords:** Exhaust

---

### Ragnarok
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 3 | 3 |
| Rarity | Rare | Rare |
| Target | All Enemies | All Enemies |
| Damage | 5 | 6 |
| Magic Number (Hits) | 5 | 6 |

**Effect:** Deal damage to a random enemy X times.

---

### Scrawl
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 0 |
| Rarity | Rare | Rare |
| Target | None | None |

**Effect:** Draw cards until you have 10 cards in your hand.
**Keywords:** Exhaust

---

### Spirit Shield
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 2 | 2 |
| Rarity | Rare | Rare |
| Target | Self | Self |
| Magic Number | 3 | 4 |

**Effect:** Gain X Block for each card in your hand (excluding this card).

---

### Unraveling
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 2 | 1 |
| Rarity | Rare | Rare |
| Target | None | None |

**Effect:** Removes ALL player debuffs (via UnravelingAction).
**Keywords:** Exhaust

---

### Vault
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 3 | 2 |
| Rarity | Rare | Rare |
| Target | All | All |

**Effect:** Take an extra turn. End your turn.
**Keywords:** Exhaust
**Special:** Skips enemy turn (SkipEnemiesTurnAction)

---

### Wish
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 3 | 3 |
| Rarity | Rare | Rare |
| Target | None | None |

**Effect:** Choose one: Gain 3/4 Strength, Gain 25/30 Gold, Gain 6/8 Plated Armor.
**Keywords:** Exhaust
**Tags:** HEALING
**Options:**
- Become Almighty: +3/+4 Strength
- Fame and Fortune: +25/+30 Gold
- Live Forever: +6/+8 Plated Armor

---

## Generated/Temp Cards

### Beta
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 2 | 1 |
| Rarity | Special | Special |
| Color | Colorless | Colorless |

**Effect:** Shuffle an Omega into your draw pile.
**Keywords:** Exhaust
**Created by:** Alpha

---

### Omega
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Power | Power |
| Cost | 3 | 3 |
| Rarity | Special | Special |
| Color | Colorless | Colorless |
| Magic Number | 50 | 60 |

**Effect:** At the end of your turn, deal X damage to ALL enemies.
**Created by:** Beta

---

### Smite
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Special | Special |
| Color | Colorless | Colorless |
| Damage | 12 | 16 |

**Effect:** Retain. Exhaust. Deal damage.
**Keywords:** Retain, Exhaust
**Created by:** Battle Hymn, Carve Reality

---

### Miracle
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 0 | 0 |
| Rarity | Special | Special |
| Color | Colorless | Colorless |

**Effect:** Retain. Exhaust. Gain 1/2 Energy.
**Keywords:** Retain, Exhaust
**Created by:** Collect, Deus Ex Machina

---

### Insight
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 0 | 0 |
| Rarity | Special | Special |
| Color | Colorless | Colorless |
| Magic Number (Draw) | 2 | 3 |

**Effect:** Retain. Exhaust. Draw X cards.
**Keywords:** Retain, Exhaust
**Created by:** Evaluate, Pray, Study

---

### Safety
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Skill | Skill |
| Cost | 1 | 1 |
| Rarity | Special | Special |
| Color | Colorless | Colorless |
| Block | 12 | 16 |

**Effect:** Retain. Exhaust. Gain block.
**Keywords:** Retain, Exhaust
**Created by:** Deceive Reality

---

### Expunger
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 1 | 1 |
| Rarity | Special | Special |
| Color | Colorless | Colorless |
| Damage | 9 | 15 |

**Effect:** Deal damage X times (X set by Conjure Blade).
**Created by:** Conjure Blade

---

### Through Violence
| Property | Base | Upgraded |
|----------|------|----------|
| Type | Attack | Attack |
| Cost | 0 | 0 |
| Rarity | Special | Special |
| Color | Colorless | Colorless |
| Damage | 20 | 30 |

**Effect:** Retain. Exhaust. Deal damage.
**Keywords:** Retain, Exhaust
**Created by:** Reach Heaven

---

## Stance Interaction Reference

### Cards that Enter Wrath
| Card | Condition |
|------|-----------|
| Eruption | Always |
| Crescendo | Always |
| Tantrum | Always |
| Indignation | If NOT in Wrath |
| Simmering Fury | Next turn |

### Cards that Enter Calm
| Card | Condition |
|------|-----------|
| Vigilance | Always |
| Tranquility | Always |
| Meditate | Always (ends turn) |
| Fear No Evil | If enemy intends to Attack |
| Inner Peace | If NOT in Calm |

### Cards that Enter Divinity
| Card | Condition |
|------|-----------|
| Blasphemy | Always (die next turn) |

### Cards that Exit Stance (Neutral)
| Card | Effect |
|------|--------|
| Empty Body | Gain block, exit stance |
| Empty Fist | Deal damage, exit stance |
| Empty Mind | Draw cards, exit stance |

### Cards that Trigger on Stance Change
| Card | Effect |
|------|--------|
| Flurry of Blows | Returns from discard to hand |
| Mental Fortress | Gain block |
| Rushdown | Draw 2 cards (Wrath only) |

### Stance-Enhanced Cards
| Card | Enhancement |
|------|-------------|
| Halt | +6/+10 Block when in Wrath |
| Indignation | Apply Vulnerable when in Wrath |
| Inner Peace | Draw 3/4 cards when in Calm |
| Like Water | Gain 5/7 Block at end of turn in Calm |

---

## Mantra Interaction Reference

### Cards that Gain Mantra
| Card | Amount (Base/Upgraded) |
|------|------------------------|
| Prostrate | 2/3 |
| Pray | 3/4 |
| Worship | 5/5 |
| Devotion (Power) | 2/3 per turn |

### Cards that Scale with Mantra
| Card | Effect |
|------|--------|
| Brilliance | +1 damage per Mantra gained this combat |

**Note:** At 10 Mantra, player enters Divinity stance and Mantra resets to 0.

---

## Scry Interaction Reference

### Cards that Scry
| Card | Amount (Base/Upgraded) |
|------|------------------------|
| Cut Through Fate | 2/3 |
| Just Lucky | 1/2 |
| Third Eye | 3/5 |
| Foresight (Power) | 3/4 per turn |

### Cards that Trigger on Scry
| Card | Effect |
|------|--------|
| Weave | Returns from discard to hand |
| Nirvana (Power) | Gain 3/4 Block |

---

## Retain Interaction Reference

### Cards with Retain
| Card | Notes |
|------|-------|
| Crescendo | - |
| Flying Sleeves | - |
| Tranquility | - |
| Protect | - |
| Perseverance | +2/+3 Block per retain |
| Sands of Time | -1 Cost per retain |
| Windmill Strike | +4/+5 Damage per retain |
| Worship (Upgraded) | - |
| Blasphemy (Upgraded) | - |
| Smite (generated) | - |
| Miracle (generated) | - |
| Insight (generated) | - |
| Safety (generated) | - |
| Through Violence (generated) | - |

### Cards that Affect Retain
| Card | Effect |
|------|--------|
| Establishment (Power) | Retained cards cost 1 less |
| Meditate | Put cards from discard into hand and Retain them |

---

## Card Count Summary

| Rarity | Count |
|--------|-------|
| Basic | 4 |
| Common | 20 |
| Uncommon | 32 |
| Rare | 21 |
| **Total** | **77** |

## Keywords Summary

| Keyword | Card Count |
|---------|------------|
| Exhaust | 16 |
| Retain | 12 (base) + 2 (upgraded) |
| Ethereal | 1 (Deva Form base) |
| Innate | 4 (upgraded only) |
| Unplayable | 1 (Deus Ex Machina) |

## Tags Summary

| Tag | Cards |
|-----|-------|
| STRIKE | Strike, Windmill Strike |
| STARTER_STRIKE | Strike |
| STARTER_DEFEND | Defend |
| EMPTY | Empty Body, Empty Fist, Empty Mind |
| HEALING | Lesson Learned, Wish |
