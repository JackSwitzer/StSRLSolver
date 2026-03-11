# Java Actions Audit Summary

**Generated:** 2026-03-03
**Source:** `decompiled/java-src/com/megacrit/cardcrawl/actions/`
**Python effects:** `packages/engine/effects/cards.py`, `packages/engine/effects/executor.py`

## Summary Counts

| Metric | Count |
|--------|-------|
| **Total Java action files** | 280 |
| **Non-animation action files** | 270 |
| **Used by Watcher cards** | ~65 |
| **Python equivalent exists** | ~95 |
| **Missing Python equivalent** | ~175 |
| **VFX/Animation only (no logic)** | ~25 |
| **Deprecated** | 7 |

### Breakdown by Directory

| Directory | Files | Description |
|-----------|-------|-------------|
| `common/` | 61 | Core shared actions (damage, block, draw, etc.) |
| `unique/` | 84 | Card-specific actions (mostly Ironclad/Silent) |
| `watcher/` | 42 | Watcher-specific actions |
| `defect/` | 45 | Defect-specific actions (orbs, etc.) |
| `utility/` | 25 | Framework actions (ScryAction, UseCardAction, etc.) |
| `animations/` | 10 | Pure VFX/animation (no game logic) |
| `deprecated/` | 7 | DEPRECATED actions (unused) |
| `root/` | 6 | AbstractGameAction, GameActionManager, IntentFlashAction |

---

## Watcher Actions (42 files) -- HIGHEST PRIORITY

All 42 files in `actions/watcher/`. These are used by Watcher cards and are essential for engine parity.

### Fully Implemented in Python

| Java Action | What It Does | Used By | Python Equivalent |
|-------------|-------------|---------|-------------------|
| **ChangeStanceAction** | Change player stance (Wrath/Calm/Divinity/Neutral). Triggers onChangeStance for powers/relics, onExitStance, onEnterStance, FlurryOfBlows from discard. | Eruption, Vigilance, Crescendo, Tantrum, FearNoEvil, InnerPeace, Indignation, Meditate, Blasphemy | `ctx.change_stance()` in cards.py, `trigger_on_stance_change()` |
| **WallopAction** | Deal damage, gain Block equal to unblocked damage dealt (lastDamageTaken). | Wallop | `gain_block_equal_unblocked_damage` effect |
| **FearNoEvilAction** | Deal damage; if enemy intent is ATTACK/ATTACK_BUFF/ATTACK_DEBUFF/ATTACK_DEFEND, enter Calm. | FearNoEvil | `if_enemy_attacking_enter_calm` effect |
| **FollowUpAction** | If second-to-last card played this combat was Attack type, gain 1 energy. | FollowUp | `if_last_card_attack_gain_energy` effect |
| **CrushJointsAction** | If second-to-last card played was Skill type, apply Vulnerable. | CrushJoints | `if_last_card_skill_vulnerable_1` effect |
| **HeadStompAction** | If second-to-last card played was Attack type, apply Weak. (Note: applies Weak, not Vulnerable as name suggests) | SashWhip | `if_last_card_attack_weak_1` effect |
| **SanctityAction** | If second-to-last card played was Skill type, draw cards. | Sanctity | `if_last_skill_draw_2` effect |
| **InnerPeaceAction** | If in Calm stance, draw N cards; else enter Calm. | InnerPeace | `if_calm_draw_else_calm` effect |
| **IndignationAction** | If in Wrath, apply Vulnerable to ALL enemies; else enter Wrath. | Indignation | `if_wrath_gain_mantra_else_wrath` effect (NOTE: Python applies Mantra not Vuln -- **PARITY BUG**) |
| **HaltAction** | Gain block; if in Wrath, gain additional block. | Halt | `if_in_wrath_extra_block_6` effect |
| **JudgementAction** | If target HP <= cutoff, InstantKill. | Judgement | `if_enemy_hp_below_kill` effect |
| **SpiritShieldAction** | Gain block equal to hand size * blockPerCard. | SpiritShield | `gain_block_per_card_in_hand` effect |
| **TriggerMarksAction** | Trigger all Mark powers on all monsters (deals Mark damage). | PressurePoints (PathToVictory) | `trigger_all_marks` effect |
| **MeditateAction** | Player chooses N cards from discard pile, move to hand with Retain, end turn. | Meditate | `put_cards_from_discard_to_hand` effect + `end_turn` |
| **PressEndTurnButtonAction** | Calls `callEndTurnEarlySequence()` to end the player's turn. | Conclude, Meditate | `end_turn` effect |
| **SkipEnemiesTurnAction** | Sets `skipMonsterTurn = true` -- enemies skip their next turn. | Vault | `take_extra_turn` effect (conceptual equivalent) |
| **ChooseOneAction** | Opens card reward screen with choice cards (e.g., Wish options). | Wish | `choose_plated_armor_or_strength_or_gold` effect |
| **LessonLearnedAction** | Deal damage; if fatal, upgrade random card in master deck (uses miscRng). | LessonLearned | `if_fatal_upgrade_random_card` effect |
| **CollectAction** | X-cost: Apply CollectPower with X stacks (+2 if Chemical X, +1 if upgraded). Drains all energy. | Collect | `put_x_miracles_on_draw` effect |
| **ConjureBladeAction** | X-cost: Create Expunger card with X hits in draw pile (top, shuffled). Chemical X +2. | ConjureBlade | `add_expunger_to_hand` effect |

### Partially Implemented or With Differences

| Java Action | What It Does | Used By | Python Status |
|-------------|-------------|---------|---------------|
| **OmniscienceAction** | Choose a card from draw pile, play it 1 time + playAmt-1 copies. Card gets exhaust=true, copies get purgeOnUse=true. | Omniscience | `play_card_from_draw_twice` -- marker only, actual double-play not fully simulated |
| **ForeignInfluenceAction** | Discovery: Generate 3 random Attack cards from ANY color class (55% common, 30% uncommon, 15% rare). If upgraded, chosen card costs 0 this turn. | ForeignInfluence | `choose_attack_from_any_class` -- simplified (picks from hardcoded list) |
| **UnravelingAction** | Queue all cards in hand for free play. Targeted cards go to random monster. | Unraveling (deprecated card, but action exists) | `play_all_hand_free` -- marker flag only |
| **BrillianceAction** | X-cost: Heal X * amount. Chemical X +2. | Brilliance (X-cost variant, possibly deprecated) | Brilliance damage effect uses `damage_plus_mantra_gained` instead |
| **DivinePunishmentAction** | X-cost: Add X copies of a card to hand. Chemical X +2. | DivinePunishment (deprecated card) | Not implemented |
| **EmptyBodyAction** | If in Neutral stance, change to Neutral + draw 1+additional; else just draw 1. | EmptyBody (deprecated variant) | `exit_stance` + base block on card handles this |
| **ContemplateAction** | Upgrade card in hand. If upgraded version: player chooses; base: random card upgraded. | Contemplate (deprecated card) | Not implemented |

### VFX-Only or Not Needed for Simulation

| Java Action | What It Does | Used By | Python Status |
|-------------|-------------|---------|---------------|
| **ExpungeVFXAction** | Pure VFX: animated slash effect on monster. | Expunger card | Not needed (VFX) |
| **SwipeAction** | Damage with VFX timing (used internally by DamageAction pattern). | Consecrate | Not needed (handled by base damage) |
| **FlickerAction** | Deal damage; if kill, trigger FlickerReturnToHandAction. | Flicker (deprecated card) | Not implemented (deprecated) |
| **FlickerReturnToHandAction** | If card is in discard and hand < 10, set returnToHand=true. | Flicker (deprecated card) | Not implemented (deprecated) |

### Conditional/Stance Check Actions

| Java Action | What It Does | Used By | Python Status |
|-------------|-------------|---------|---------------|
| **StanceCheckAction** | If player is in specified stance, execute buffered action. | Various powers/cards | Inline stance checks in Python |
| **NotStanceCheckAction** | If player is NOT in specified stance, execute buffered action. | Various powers/cards | Inline stance checks in Python |
| **VengeanceAction** | If player HP decreased since last turn, enter Wrath. | Vengeance (deprecated power?) | Not implemented |
| **EmotionalTurmoilAction** | If Calm -> enter Wrath; if Wrath -> enter Calm (toggle). | EmotionalTurmoil (deprecated) | Not implemented |
| **PerfectedFormAction** | Stub/incomplete -- checks if not in Divinity but does nothing. | PerfectedForm (deprecated) | Not implemented (empty action) |

### Deprecated / Dead Code

| Java Action | What It Does | Python Status |
|-------------|-------------|---------------|
| **CrescentKickAction** | If card hadVigor, draw 1 + gain 1 energy. References DEPRECATEDCrescentKick. | Not needed (deprecated) |
| **ClarityAction** | Look at top N cards of draw; choose which go to hand, rest exhaust. | Not needed (deprecated) |
| **MasterRealityAction** | Count retained cards in hand, deal lightning damage per retained card. (Not the MasterReality power!) | Not needed (deprecated) |
| **PathVictoryAction** | Draw top card and set its cost to 0. | Not needed (deprecated card; Python uses PressurePoints instead) |
| **RetreatingHandAction** | If second-to-last card was Attack, set card.returnToHand=true. | Not needed (deprecated) |
| **TranscendenceAction** | Upgrade all retained cards in hand. | Not needed (deprecated) |

---

## Common Actions (61 files) -- CORE FRAMEWORK

These are the fundamental building blocks used by ALL characters.

### Core Damage/Block/HP Actions

| Java Action | What It Does | Used By | Python Equivalent |
|-------------|-------------|---------|-------------------|
| **DamageAction** | Deal damage to single target with attack effect, gold steal variant. | Strike, most single-target attacks | `ctx.deal_damage_to_enemy()` |
| **DamageAllEnemiesAction** | Deal damage to ALL enemies using damage matrix. | Consecrate, Whirlwind, Cleave | `ctx.deal_damage_to_enemy()` loop over living enemies |
| **DamageRandomEnemyAction** | Deal damage to random enemy. | Ragnarok | `ctx.deal_damage_to_random_enemy()` |
| **AttackDamageRandomEnemyAction** | Attack-type damage to random enemy (with VFX). | Ragnarok, some relics | `ctx.deal_damage_to_random_enemy()` |
| **PummelDamageAction** | Fast repeated damage hits. | Pummel | Multi-hit via `damage_x_times` effect |
| **GainBlockAction** | Target gains block amount. | Defend, all block cards | `ctx.gain_block()` |
| **RemoveAllBlockAction** | Remove all block from target. | Neutralize, some enemies | Not explicitly implemented |
| **HealAction** | Heal target for amount. | Reaper, Bite, relics | `ctx.heal_player()` |
| **LoseHPAction** | Target loses HP (bypasses block). | Offering, Brutality, enemy abilities | `ctx.state.player.hp -= amount` |
| **LosePercentHPAction** | Lose percentage of max HP. | Some events/enemies | Not implemented |
| **InstantKillAction** | Instantly kill target creature. | Judgement | `target.hp = 0` |

### Card Draw / Manipulation

| Java Action | What It Does | Used By | Python Equivalent |
|-------------|-------------|---------|-------------------|
| **DrawCardAction** | Draw N cards (handles shuffle if draw pile empty). Has follow-up action support. | Acrobatics, Backflip, many cards | `ctx.draw_cards()` |
| **FastDrawCardAction** | Fast variant of DrawCardAction. | Some fast-draw effects | `ctx.draw_cards()` |
| **DiscardAction** | Player chooses N cards to discard (or random). | Gambler's Chip, Calculated Gamble | Partial via `ctx.cards_discarded` |
| **DiscardSpecificCardAction** | Discard a specific card from hand/group. | Snecko, powers | Not explicitly implemented |
| **DiscardAtEndOfTurnAction** | End-of-turn discard (handles Retain, Equilibrium). | Turn end system | Handled in combat engine |
| **ExhaustAction** | Player chooses N cards to exhaust (or random, anyNumber). | True Grit, Burning Pact | Partial |
| **ExhaustSpecificCardAction** | Exhaust a specific card from a card group. | Various powers | Not explicitly implemented |
| **PutOnDeckAction** | Put N cards from hand on top of draw pile. | Headbutt, Warcry | Not implemented |
| **PutOnBottomOfDeckAction** | Put specific card on bottom of deck. | Forethought | Not implemented |
| **PlayTopCardAction** | Play top card of draw pile. | Havoc, Mayhem | Not implemented |

### Card Creation (Temp Cards)

| Java Action | What It Does | Used By | Python Equivalent |
|-------------|-------------|---------|-------------------|
| **MakeTempCardInHandAction** | Add temp card to hand (respects MasterReality upgrade, hand limit 10). | Carve Reality, Smite generation | `ctx.add_card_to_hand()` |
| **MakeTempCardInDrawPileAction** | Add temp card to draw pile (random/top/bottom, respects MasterReality). | Alpha/Beta chains, Conjure Blade | `ctx.add_card_to_draw_pile()` |
| **MakeTempCardInDiscardAction** | Add temp card to discard pile (respects MasterReality). | Immolate (Burn), various | Not explicitly implemented |
| **MakeTempCardInDiscardAndDeckAction** | Add temp card to BOTH discard and deck. | Specific enemy/event effects | Not implemented |
| **MakeTempCardAtBottomOfDeckAction** | Add temp card to bottom of draw pile. | Forethought variant | Not implemented |
| **TransformCardInHandAction** | Transform a card in hand into random card. | Transmutation | Not implemented |

### Power Management

| Java Action | What It Does | Used By | Python Equivalent |
|-------------|-------------|---------|-------------------|
| **ApplyPowerAction** | Apply/stack a power on target. Handles Artifact negation, Snake Skull, Corruption auto-cost. | All power/debuff cards | `ctx.apply_status_to_player/target()` |
| **ApplyPowerToRandomEnemyAction** | Apply power to random enemy. | Some relics | Not implemented |
| **ApplyPoisonOnRandomMonsterAction** | Apply Poison to random monster. | Envenom | Not implemented |
| **RemoveSpecificPowerAction** | Remove a specific power entirely. | Orange Pellets, some effects | `ctx.remove_status_from_player()` |
| **ReducePowerAction** | Reduce a power's stack by amount. | Various internal effects | Not implemented |

### Energy Management

| Java Action | What It Does | Used By | Python Equivalent |
|-------------|-------------|---------|-------------------|
| **GainEnergyAction** | Gain energy and trigger onGainEnergy for cards. | Miracle, Calm exit, relics | `ctx.gain_energy()` |
| **GainEnergyAndEnableControlsAction** | Gain energy + re-enable controls (start of turn). | Turn start system | Handled in combat engine |

### Cost Modification

| Java Action | What It Does | Used By | Python Equivalent |
|-------------|-------------|---------|-------------------|
| **ReduceCostAction** | Permanently reduce card cost for combat (by UUID lookup). | Enlightenment, Establishment | Not implemented (cost reduction tracked differently) |
| **ReduceCostForTurnAction** | Reduce card cost for current turn only. | Snecko Eye, Madness | Not implemented |
| **ModifyDamageAction** | Permanently modify card's baseDamage. | Ritual Dagger, Genetic Algorithm | Not implemented |
| **ModifyBlockAction** | Permanently modify card's baseBlock. | Genetic Algorithm | Not implemented |

### Card Search / Specific Retrieval

| Java Action | What It Does | Used By | Python Equivalent |
|-------------|-------------|---------|-------------------|
| **BetterDrawPileToHandAction** | Search draw pile for a card matching type, move to hand. | Headbutt variant | Not implemented |
| **BetterDiscardPileToHandAction** | Search discard pile for a card, move to hand. | Exhume | Not implemented |

### Miscellaneous Common Actions

| Java Action | What It Does | Used By | Python Equivalent |
|-------------|-------------|---------|-------------------|
| **UpgradeRandomCardAction** | Upgrade a random card in hand. | Armaments (base) | Not implemented |
| **UpgradeSpecificCardAction** | Upgrade a specific card. | Armaments (upgraded) | Not implemented |
| **ObtainPotionAction** | Add potion to potion slots. | White Beast potion drops | Not implemented |
| **GainGoldAction** | Gain gold. | Hand of Greed, events | Not implemented |
| **ShuffleAction** | Shuffle draw pile. | Blue Candle exhaust trigger | Not implemented |
| **EmptyDeckShuffleAction** | Shuffle discard pile into draw pile. | Internal draw system | Handled in combat engine draw logic |
| **RollMoveAction** | Enemy rolls next move/intent. | All enemies | Handled in enemy AI system |
| **SetMoveAction** | Set enemy move explicitly. | Enemy AI | Handled in enemy AI system |
| **ChangeStateAction** | Change enemy animation state. | Enemy animations | Not needed (VFX) |
| **ShowMoveNameAction** | Display enemy move name. | Enemy intents | Not needed (VFX) |
| **MonsterStartTurnAction** | Trigger monster start-of-turn. | Turn system | Handled in combat engine |
| **EndTurnAction** | End turn processing. | Turn system | Handled in combat engine |
| **EnableEndTurnButtonAction** | UI control. | Turn system | Not needed (UI) |
| **SpawnMonsterAction** | Spawn a new monster in combat. | Gremlin Leader, some bosses | Handled in enemy spawning system |
| **ReviveMonsterAction** | Revive a dead monster. | Donu/Deca, some bosses | Not implemented |
| **SuicideAction** | Monster kills itself (Exploder, etc.). | Exploder, Transient | Not implemented |
| **EscapeAction** | Monster escapes combat. | Snecko, some events | Not implemented |
| **RelicAboveCreatureAction** | Show relic icon above creature (VFX). | All relics | Not needed (VFX) |
| **SetDontTriggerAction** | Set card.dontTriggerOnUseCard flag. | Internal | Not needed |
| **DarkOrbEvokeAction** | Evoke Dark orb (Defect). | Dark Orb | Not applicable (Defect) |

---

## Unique Actions (84 files) -- CARD-SPECIFIC

### Used by Colorless Cards (Available to Watcher)

| Java Action | What It Does | Used By | Python Equivalent |
|-------------|-------------|---------|-------------------|
| **ApotheosisAction** | Upgrade ALL cards in hand, draw, discard, and exhaust piles. | Apotheosis | `upgrade_all_cards_in_combat` (noop marker) |
| **DiscoveryAction** | Discovery mechanic: show 3 random cards, add chosen to hand at 0 cost. Can filter by type or colorless. | Discovery, Jack of All Trades | Not implemented |
| **DualWieldAction** | Choose Attack or Power card from hand, create N copies in hand. | Dual Wield | Not implemented |
| **NightmareAction** | Choose card from hand, apply NightmarePower (adds 3 copies next turn). | Nightmare | Not implemented |
| **ExhumeAction** | Choose card from exhaust pile (except Exhume), return to hand. If upgraded version chosen, upgrade it. | Exhume | Not implemented |
| **ForethoughtAction** | Choose card from hand, put on bottom of draw pile at 0 cost. | Forethought | Not implemented |
| **MadnessAction** | Reduce random card in hand to 0 cost for combat. | Madness | Not implemented |
| **AddCardToDeckAction** | Add a card directly to master deck. | Feed, Lesson Learned | Not implemented |
| **IncreaseMaxHpAction** | Increase player max HP. | Feed | Not implemented |
| **MindBlastAction** | Deal damage equal to draw pile size (with multi-hit if specified). | Mind Blast | `damage_equals_draw_pile_size` effect |
| **SetupAction** | Choose card from hand, set cost to 0 for next turn (put on top of draw). | Setup | Not implemented |
| **RetainCardsAction** | At end of turn, choose N cards to retain. | Well-Laid Plans, Runic Pyramid | Not implemented |
| **RestoreRetainedCardsAction** | Return retained cards to hand at start of turn. | Retain system | Handled in combat engine |
| **RandomizeHandCostAction** | Set all cards in hand to random costs (0-3). | Snecko Eye, Snecko card | Not implemented |
| **EnlightenmentAction** | Reduce all cards in hand to 1 cost (for turn or permanently). | Enlightenment | Not implemented |
| **GamblingChipAction** | Discard any number of cards then draw that many. | Gambling Chip relic | Not implemented |
| **RemoveDebuffsAction** | Remove all debuffs from player. | Orange Pellets, Panacea | Not implemented |
| **RemoveAllPowersAction** | Remove ALL powers from target. | Clockwork/specific enemies | Not implemented |
| **RandomCardFromDiscardPileToHandAction** | Move random card from discard to hand. | Gambling relic variant | Not implemented |
| **DoubleYourBlockAction** | Double player's current block. | Barricade combo, Body Slam setup | Not implemented |

### Ironclad-Specific Unique Actions

| Java Action | What It Does | Used By |
|-------------|-------------|---------|
| **ArmamentsAction** | Upgrade 1 card in hand (base: choose 1, upgraded: all). | Armaments |
| **WhirlwindAction** | X-cost: deal damage X times to all enemies. Chemical X +2. | Whirlwind |
| **LimitBreakAction** | Double player's Strength. | Limit Break |
| **FeedAction** | Deal damage; if kill, gain max HP permanently. | Feed |
| **FiendFireAction** | Exhaust all cards in hand, deal damage per card exhausted. | Fiend Fire |
| **RitualDaggerAction** | Deal damage; if kill, permanently increase damage. | Ritual Dagger |
| **DropkickAction** | If enemy Vulnerable, gain 1 energy + draw 1. | Dropkick |
| **SpotWeaknessAction** | If enemy attacking, gain Strength. | Spot Weakness |
| **SwordBoomerangAction** | Deal damage N times to random enemies. | Sword Boomerang |
| **ImmolateAction** | Deal damage to all + add Burn to discard. | Immolate |
| **UnloadAction** | Exhaust all non-Attack cards, deal damage per exhausted. | Unload |
| **BurnIncreaseAction** | Upgrade Burn damage (status). | Burn (status card) |
| **CorpseExplosionAction** | When enemy dies, deal damage to all enemies = its max HP. | Corpse Explosion |
| **ExhaustAllNonAttackAction** | Exhaust all non-Attack cards in hand. | Warcry variant |
| **BlockPerNonAttackAction** | Gain block per non-Attack card in hand. | Backpack |

### Silent-Specific Unique Actions

| Java Action | What It Does | Used By |
|-------------|-------------|---------|
| **CalculatedGambleAction** | Discard entire hand, draw same number. | Calculated Gamble |
| **EscapePlanAction** | Draw 1; if drawn card is Skill, gain block. | Escape Plan |
| **HeelHookAction** | If enemy Weak, gain 1 energy + draw 1. | Heel Hook |
| **SkewerAction** | X-cost: deal damage X times. | Skewer |
| **BladeFuryAction** | X-cost deal damage X times to all enemies. | Blade Fury (deprecated?) |
| **BouncingFlaskAction** | Apply Poison N times to random enemies. | Bouncing Flask |
| **FlechetteAction** | Deal damage per Skill in hand. | Flechette |
| **ExpertiseAction** | Draw until hand has N cards. | Expertise |
| **DoublePoisonAction** | Double a monster's Poison. | Catalyst |
| **TriplePoisonAction** | Triple a monster's Poison. | Catalyst+ |
| **PoisonLoseHpAction** | Enemy loses HP equal to Poison (Poison tick). | Poison power |
| **DiscardPileToTopOfDeckAction** | Choose card from discard, put on top of draw. | Headbutt |
| **MalaiseAction** | X-cost: apply X Weak + Strength down to enemy. Lose all energy. | Malaise |
| **DoppelgangerAction** | X-cost: draw X + gain X energy next turn. | Doppelganger |
| **GainEnergyIfDiscardAction** | Gain energy if player discarded this turn. | Tactician/Reflex |
| **ApplyBulletTimeAction** | Set all cards in hand to 0 cost, apply NoDraw. | Bullet Time |
| **VampireDamageAction** | Deal damage, heal for unblocked damage. | Bite |
| **VampireDamageAllEnemiesAction** | Deal damage to all, heal for total unblocked. | Reaper |
| **GreedAction** | Deal damage; if fatal, gain gold. | Hand of Greed |
| **DamagePerAttackPlayedAction** | Deal damage multiplied by attacks played this turn. | Finisher |

### Enemy/NPC-Specific Unique Actions

| Java Action | What It Does | Used By |
|-------------|-------------|---------|
| **GainBlockRandomMonsterAction** | Random monster gains block. | Gremlin Leader |
| **SpawnDaggerAction** | Spawn a Dagger enemy. | Bear (enemy) |
| **SummonGremlinAction** | Summon a random Gremlin. | Gremlin Leader |
| **CrowReviveAction** | Revive Byrd enemies. | The Collector |
| **CanLoseAction** | Set "can lose" flag (re-enable death). | Spire Heart |
| **CannotLoseAction** | Set "cannot lose" flag (prevent death). | Spire Heart |
| **ApplyStasisAction** | Stasis orb captures a card. | Bronze Automaton |
| **StepThroughTimeAction** | Time Eater mechanic. | Time Eater |
| **ChannelDestructionAction** | Channel destruction pattern. | Awakened One |

### Miscellaneous Unique Actions

| Java Action | What It Does | Used By |
|-------------|-------------|---------|
| **LoseEnergyAction** | Lose N energy. | Snecko Eye, some enemies |
| **EstablishmentPowerAction** | Reduce cost of retained card by 1 (Establishment trigger). | Establishment power |
| **CodexAction** | Discover from class-specific pool (Enchiridion relic). | Enchiridion relic |
| **InspirationAction** | Add random class-specific cards at 0 cost. | Foreign Influence variant |
| **RegenAction** | Heal from Regen power at end of turn. | Regen power |
| **DeckToHandAction** | Move specific card type from draw to hand. | Internal |
| **SkillFromDeckToHandAction** | Move Skill card from draw to hand. | Holy Water variant |
| **AttackFromDeckToHandAction** | Move Attack card from draw to hand. | Warcry variant |
| **UndoAction** | Restore saved game state. | Appears unused |
| **BendAction** | Force enemy to change target. | Appears unused |
| **TransmuteAction** | Transform X cards (old version). | Transmutation |
| **Transmutev2Action** | Transform X cards (new version). | Transmutation |
| **TransmutationAction** | X-cost transform variant. | Transmutation |
| **PatientMissileAction** | Deal damage N times (delayed missiles). | Enemy attacks |
| **TripleYourBlockAction** | Triple current block. | Specific enemies |
| **FullHealthAdditionalDamageAction** | Deal extra damage if at full HP. | Specific enemies |
| **TumbleAction** | Damage with knockback VFX. | Enemy abilities |
| **BaneAction** | If enemy Poisoned, deal damage again. | Bane |
| **RipAndTearAction** | Deal damage twice to random enemies. | Rip and Tear |
| **MulticastAction** | Evoke orb X times (Defect). | Multicast |

---

## Defect Actions (45 files) -- LOW PRIORITY (Not Watcher)

All Defect-specific, primarily orb mechanics. Not needed for Watcher RL.

| Java Action | What It Does | Used By |
|-------------|-------------|---------|
| **ChannelAction** | Channel an orb. | All orb-channeling cards |
| **EvokeOrbAction** | Evoke frontmost orb. | Dualcast, Recycle |
| **EvokeAllOrbsAction** | Evoke all orbs. | Tempest |
| **EvokeWithoutRemovingOrbAction** | Trigger evoke effect without removing. | Some powers |
| **RemoveNextOrbAction** | Remove next orb without evoke. | Internal |
| **RemoveAllOrbsAction** | Remove all orbs. | Fission |
| **IncreaseMaxOrbAction** | Increase max orb slots. | Capacitor |
| **DecreaseMaxOrbAction** | Decrease max orb slots. | Negative capacitor effects |
| **AnimateOrbAction** | Pure VFX for orb. | Orb animations |
| **LightningOrbEvokeAction** | Lightning orb evoke: deal damage to random enemy. | Lightning orb |
| **LightningOrbPassiveAction** | Lightning orb passive: deal damage to random enemy. | Lightning orb |
| **DarkImpulseAction** | Add Dark orb passive channeling. | Dark orb |
| **TriggerEndOfTurnOrbsAction** | Trigger all orb end-of-turn effects. | Turn system |
| **ThunderStrikeAction** | Deal damage per Lightning channeled this combat. | Thunder Strike |
| **NewThunderStrikeAction** | Updated version. | Thunder Strike |
| **BarrageAction** | Deal damage per orb. | Barrage |
| **FTLAction** | If < N cards played this turn, draw 1. | FTL |
| **SunderAction** | Deal damage; if kill, gain energy. | Sunder |
| **CompileDriverAction** | Deal damage; draw 1 per unique orb type. | Compile Driver |
| **ScrapeAction** | Draw N cards. | Scrape |
| **ScrapeFollowUpAction** | Discard non-drawn cards from Scrape. | Scrape |
| **SeekAction** | Choose N cards from draw pile, move to hand. | Seek |
| **FissionAction** | Remove all orbs, gain energy + draw per orb. | Fission |
| **OldFissionAction** | Old version of Fission. | Deprecated |
| **RecycleAction** | Exhaust card, gain energy equal to cost. | Recycle |
| **CacheAction** | Add random colorless card to hand. | White Noise |
| **DoubleEnergyAction** | Double current energy. | Double Energy |
| **AggregateEnergyAction** | Gain energy = draw pile size / N. | Aggregate |
| **AllCostToHandAction** | Move all cards of specific cost from draw to hand. | All for One |
| **ReinforcedBodyAction** | Gain block per orb slot. | Reinforced Body |
| **EnergyBlockAction** | Gain block equal to energy. | Equilibrium |
| **GashAction** | Deal damage to self (enemy). | Enemy moves |
| **ForTheEyesAction** | Enemy move. | Automaton |
| **FluxAction** | Status card effect. | Flux status |
| **IceWallAction** | Gain focus block variant. | Ice Wall |
| **ShuffleAllAction** | Shuffle everything. | Chaos |
| **ReprieveAction** | Gain block somehow. | Internal |
| **ImpulseAction** | Trigger orb passives. | Impulse |
| **BlasterAction** | Damage per orb. | Blaster |
| **RedoAction** | Re-channel last orb. | Redo |
| **EssenceOfDarknessAction** | Channel Dark orb per orb slot. | Essence of Darkness |
| **IncreaseMiscAction** | Increase card's misc value (Genetic Algorithm). | Genetic Algorithm |
| **DamageAllButOneEnemyAction** | Damage all enemies except target. | Hyperbeam side effect |
| **DiscardPileToHandAction** | Move cards from discard to hand (Defect variant). | Hologram |
| **NewRipAndTearAction** | Updated Rip and Tear. | Defect variant |

---

## Utility Actions (25 files) -- FRAMEWORK

| Java Action | What It Does | Used By | Python Equivalent |
|-------------|-------------|---------|-------------------|
| **ScryAction** | Scry N cards: look at top N of draw pile, choose which to discard. Adds +2 if Golden Eye relic. Triggers onScry for powers and triggerOnScry for discard pile cards. | CutThroughFate, ThirdEye, JustLucky, Foresight | `ctx.scry()` |
| **UseCardAction** | Main card-use pipeline: handle exhaust, returnToHand, rebound, trigger power.onUseCard, relic.onUseCard. | Every card play | Handled in combat engine `play_card()` |
| **NewQueueCardAction** | Queue a card for play (immediate or autoplay). Handles targeting, energy, freeToPlay. | Omniscience, Havoc | Not explicitly implemented |
| **QueueCardAction** | Deprecated card queue action. | Old code | Not needed |
| **ExhaustAllEtherealAction** | Exhaust all Ethereal cards in hand at end of turn. | Turn system | Handled in combat engine |
| **ExhaustToHandAction** | Move card from exhaust pile to hand. | Dead Branch trigger | Not implemented |
| **DiscardToHandAction** | Move card from discard to hand. | Meditate, some relics | `ctx.move_card_from_discard_to_hand()` |
| **DrawPileToHandAction** | Move card from draw pile to hand. | Seek, Headbutt | Not implemented |
| **LoseBlockAction** | Lose specific amount of block. | Start of turn block decay | Handled in combat engine |
| **ReApplyPowersAction** | Recalculate all power effects (refresh damage/block numbers). | After power changes | Not needed (synchronous) |
| **HandCheckAction** | Check hand size and refresh layout. | After draw/discard | Not needed (UI) |
| **ConditionalDrawAction** | Draw only if condition met (e.g., < N cards in hand). | Some powers | Not implemented |
| **ChooseOneColorless** | Choose from colorless card options. | Jack of All Trades | Not implemented |
| **ResetFlagsAction** | Reset various combat flags. | Turn system | Handled in combat engine |
| **UnlimboAction** | Return card from limbo to hand. | Card manipulation | Not implemented |
| **UnhoverCardAction** | UI: unhover card. | UI | Not needed |
| **UpdateCardDescriptionAction** | Refresh card text. | After upgrades | Not needed |
| **WaitAction** | Wait for duration (animation timing). | Everywhere | Not needed |
| **SFXAction** | Play sound effect. | Everywhere | Not needed |
| **ShakeScreenAction** | Shake screen. | Big hits | Not needed |
| **ShowCardAction** | Show card briefly. | Card effects | Not needed |
| **ShowCardAndPoofAction** | Show card then poof. | Exhaust VFX | Not needed |
| **TextAboveCreatureAction** | Show text above creature. | Buff/debuff text | Not needed |
| **TextCenteredAction** | Show centered text. | Turn labels | Not needed |
| **HideHealthBarAction** | Hide health bar. | Some bosses | Not needed |

---

## Animation Actions (10 files) -- NOT NEEDED

Pure VFX/animation, no game logic.

| Java Action | What It Does |
|-------------|-------------|
| **VFXAction** | Play visual effect |
| **AnimateFastAttackAction** | Fast attack animation |
| **AnimateSlowAttackAction** | Slow attack animation |
| **AnimateHopAction** | Hop animation |
| **AnimateJumpAction** | Jump animation |
| **AnimateShakeAction** | Shake animation |
| **FastShakeAction** | Fast shake animation |
| **SetAnimationAction** | Set creature animation state |
| **ShoutAction** | Enemy shout/speech bubble |
| **TalkAction** | NPC talk/speech bubble |

---

## Deprecated Actions (7 files) -- NOT NEEDED

| Java Action | What It Does |
|-------------|-------------|
| **DEPRECATEDBlockSelectedAmountAction** | Old block mechanic |
| **DEPRECATEDExperiencedAction** | Old XP mechanic |
| **DEPRECATEDBrillianceAction** | Old Brilliance (pre-rework) |
| **DEPRECATEDRandomStanceAction** | Random stance entry |
| **DEPRECATEDDamagePerCardAction** | Damage per card in hand |
| **DEPRECATEDEruptionAction** | Old Eruption |
| **DEPRECATEDArmorSelectedAmountAction** | Old armor mechanic |

---

## Root-Level Actions (6 files)

| Java Action | What It Does | Python Equivalent |
|-------------|-------------|-------------------|
| **AbstractGameAction** | Base class for all actions (ActionType enum, timing, etc.) | No direct equivalent (Python is synchronous) |
| **GameActionManager** | Manages action queue, processes actions, tracks turn state. | CombatEngine handles this |
| **IntentFlashAction** | Flash enemy intent icon. | Not needed (VFX) |

---

## Parity Issues Found

### CRITICAL: IndignationAction Mismatch

**Java:** If in Wrath, apply **Vulnerable** to ALL enemies; else enter Wrath.
**Python:** If in Wrath, gain **Mantra**; else enter Wrath.

The Python `if_wrath_gain_mantra_else_wrath` effect applies Mantra instead of Vulnerable. This is a parity bug.

### HIGH: Conditional Card Check Semantics

Java checks `cardsPlayedThisCombat.get(size - 2)` (second-to-last card played THIS COMBAT), while the Python effects use `get_last_card_type()` which may only track the immediately previous card this turn. Needs verification that the Python implementation matches the Java "combat-wide" history.

### MEDIUM: X-Cost Actions Missing Chemical X

Java X-cost actions (CollectAction, ConjureBladeAction, BrillianceAction, DivinePunishmentAction) all check for Chemical X relic (+2 to X). The Python equivalents do not handle Chemical X.

### MEDIUM: MasterReality Integration

Java `MakeTempCardInHandAction`, `MakeTempCardInDrawPileAction`, and `MakeTempCardInDiscardAction` all check `hasPower("MasterRealityPower")` and auto-upgrade non-curse/non-status cards. Python card generation effects only check MasterReality in some places (Study, BattleHymn) but not consistently.

### LOW: ScryAction Golden Eye Relic

Java `ScryAction` adds +2 to scry amount if player has Golden Eye relic. Python `ctx.scry()` may not handle this.

---

## Priority Implementation Gaps for Watcher RL

### Must Have (blocks correct gameplay)

1. **Fix IndignationAction parity** -- Vulnerable to all enemies, not Mantra
2. **PutOnDeckAction** -- Used by Headbutt (colorless), important for deck manipulation
3. **ExhumeAction** -- Retrieve from exhaust pile (colorless rare)
4. **RetainCardsAction** -- Well-Laid Plans relic/card
5. **EnlightenmentAction** -- Reduce all hand costs to 1
6. **DiscoveryAction** -- Discovery mechanic
7. **RemoveAllBlockAction** -- Used by some enemy abilities

### Should Have (improves simulation accuracy)

8. **ReduceCostAction/ReduceCostForTurnAction** -- Cost modification
9. **DualWieldAction** -- Duplicate Attack/Power in hand
10. **NightmareAction** -- Copy card 3x next turn
11. **ForethoughtAction** -- Put card on bottom at 0 cost
12. **ApotheosisAction** -- Full implementation (upgrade all cards)
13. **MadnessAction** -- Random card to 0 cost
14. **Chemical X relic** in X-cost actions
15. **GoldenEye relic** in ScryAction

### Nice to Have (edge cases)

16. **SetupAction** -- Card to top of draw at 0 cost
17. **RandomizeHandCostAction** -- Snecko effects
18. **TransformCardInHandAction** -- Transmutation
19. **DoubleYourBlockAction** -- Block doubling
20. **MakeTempCardInDiscardAction** -- Generate cards to discard
