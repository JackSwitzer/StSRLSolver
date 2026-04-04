# Full Engine Audit — 2026-04-02

3 Opus 4.6 agents reviewed the entire engine. Consolidated findings below.

## CRITICAL — Wrong combat behavior

| # | Issue | Root Cause | Fix |
|---|-------|-----------|-----|
| C1 | Time Eater 12-card mechanic broken | TIME_WARP_ACTIVE never set in create_enemy | Add `set_status(sid::TIME_WARP_ACTIVE, 1)` |
| C2 | Transient never auto-dies | FADING never set in create_enemy | Add `set_status(sid::FADING, 5/6)` |
| C3 | Transient takes normal HP damage | SHIFTING never consumed in damage pipeline | Check in deal_damage_to_enemy: convert HP loss to block gain |
| C4 | Nemesis takes full damage every turn | Intangible cycling not implemented | Add Intangible grant in enemy turn/move logic |
| C5 | No minion spawning | Collector/Automaton/Reptomancer/GremlinLeader spawn moves do nothing | Implement spawn_minion in execute_enemy_move |
| C6 | Potion damage bypasses pipeline | potions.rs has local damage code skipping Slow/Flight/Invincible/boss hooks | Route through deal_damage_to_enemy |
| C7 | Beat of Death can kill without fairy | Inline damage lacks fairy revive check | Use centralized hp_loss function |
| C8 | Curiosity (Awakened One) not triggered | Logic in dead on_player_card_played, never called from play_card | Add Curiosity check inline in play_card Power branch |

## HIGH — Missing features that affect strategy

| # | Issue | Root Cause | Fix |
|---|-------|-----------|-----|
| H1 | Curl-Up never triggers | Set on Louse but never consumed in damage pipeline | Add on_first_hit check in deal_damage_to_enemy |
| H2 | Sharp Hide never fires | Set on enemies but never deals retaliatory damage | Add on_attacked check |
| H3 | Malleable never fires | Set but never consumed | Add escalating block on hit |
| H4 | Reactive (WrithingMass) dead | writhing_mass_reactive_reroll never called | Call from on_enemy_damaged |
| H5 | Juggernaut installed but never triggers | No centralized gain_block hook | Centralize block gain, add Juggernaut dispatch |
| H6 | Wave of Hand only on card block | Block from relics/potions/orbs doesn't trigger | Same: centralize gain_block |
| H7 | Blasphemy skips fairy revive | Inline death, no fairy check | Use centralized player_die function |
| H8 | Evolve not triggered on Status draw | Set but never read in draw_cards | Add on_draw check |
| H9 | Fire Breathing not triggered on Status/Curse draw | Set but never read | Add on_draw check |
| H10 | on_hp_loss relics never fire | Self-Forming Clay, Centennial Puzzle, Runic Cube, Red Skull, Emotion Chip | Add on_hp_loss hook call in enemy attack path |
| H11 | on_enemy_death relics never fire | Gremlin Horn, The Specimen | Add on_enemy_death hook |
| H12 | on_shuffle relics never fire | Sundial, Abacus | Add on_shuffle hook in draw_cards |
| H13 | Charon's Ashes not on exhaust | Exists but not called from trigger_on_exhaust | Wire relic call |
| H14 | Unceasing Top never triggers | Exists but never called after card play | Check hand empty after play_card |
| H15 | on_victory relics never fire | Burning Blood, Black Blood, Meat on Bone | Add on_victory hook |
| H16 | Sentry stagger missing | All 3 start on Bolt, should alternate | Fix create_enemy |
| H17 | EchoForm ignores stack count | Only replays first card regardless of stacks | Check EchoForm amount |
| H18 | Necronomicon never fires | Helper exists but play_card never calls it | Wire in play_card after 2+ cost Attack |
| H19 | Forcefield (Automaton) dead | Never consumed | Add on_power_play check |
| H20 | SkillBurn (Book of Stabbing) dead | Never consumed | Add on_skill_play damage |
| H21 | Elites in ACT3_STRONG pool | GiantHead/Nemesis/Reptomancer double-counted | Remove from strong pool |
| H22 | ~15 encounter types missing from pools | Gremlin Gang, Looter, Snecko, multi-slime, etc. | Add to pool arrays |

## ARCHITECTURE — Needs refactoring

| # | Issue | Impact |
|---|-------|--------|
| A1 | Death-check-fairy pattern duplicated 7 times | Create `fn player_lose_hp(&mut self, amount) -> bool` |
| A2 | Block gain scattered across 27+ sites | Create `fn player_gain_block(&mut self, amount)` with Juggernaut/WaveOfHand hooks |
| A3 | Dead enemy turn code in engine.rs (130 lines) | Delete do_enemy_turns + execute_enemy_move from engine.rs |
| A4 | 65 dead functions in buffs.rs | Delete entire file or gut it |
| A5 | 13 dead functions in enemy_powers.rs | Clean up |
| A6 | 10 dead functions in debuffs.rs | Clean up |
| A7 | PowerId/PowerDef trigger flags never used for dispatch | Remove or make completeness test use them |
| A8 | deal_damage_to_player exists but never called | Either use it everywhere or delete it |
| A9 | Run is Act 1 only | No act transitions, no boss relics, no Neow |

## MISSING TRIGGER HOOKS (the systematic gap)

The hook dispatch system covers turn-start, turn-end, card-played, exhaust, stance-change. But these trigger types have NO dispatch:

| Trigger | Java Method | Powers/Relics That Need It |
|---------|------------|---------------------------|
| **on_attacked** (enemy) | onAttacked | Curl-Up, Malleable, Sharp Hide, Shifting, Angry |
| **on_hp_loss** (player) | wasHPLost | Rupture, Self-Forming Clay, Centennial Puzzle, Runic Cube, Red Skull, Emotion Chip |
| **on_block_gained** (player) | onGainedBlock | Wave of Hand, Juggernaut |
| **on_card_draw** | onCardDraw | Evolve, Fire Breathing |
| **on_enemy_death** | onDeath (enemy) | SporeCloud, Gremlin Horn, The Specimen |
| **on_shuffle** | onShuffle | Sundial, Abacus |
| **on_victory** | onVictory | Burning Blood, Black Blood, Meat on Bone |

## DEAD CODE — Safe to delete (~400 lines)

- buffs.rs: all 65 pub fns (entire file is dead)
- debuffs.rs: 10 dead fns
- enemy_powers.rs: 13 dead fns
- engine.rs: do_enemy_turns + execute_enemy_move (130 lines)
- combat_hooks.rs: on_player_card_played (dead, but has Curiosity logic that needs rescuing)
- 7 write-only status IDs, 4 completely unused status IDs
