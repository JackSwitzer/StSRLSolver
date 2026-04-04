# Codex Post-Phase-2 Review (2026-04-01)

## 5 HIGH findings

### 1. Turn ordering mismatches
- WraithForm: Rust start-turn, Java end-turn
- Loop: Rust end-turn, Java start-turn
- CreativeAI/Magnetism: Rust post-draw, Java start-turn

### 2. 10 remaining dead statuses
- Mayhem, RetainCards (installed but no-oped)
- DrawCard, NextTurnBlock, Energized (set by card_effects, never consumed at turn start)
- DoubleTap, Burst (set by card_effects, never consumed in play_card)
- Curiosity, Invincible (helpers exist, never called)

### 3. Double Tap/Burst/Echo Form replay incomplete
- DoubleTap/Burst: statuses set but play_card never checks them
- EchoForm: only re-runs effects, Java duplicates full card from onUseCard

### 4. Enemy damage pipeline gaps
- Invincible cap not checked in deal_damage_to_enemy
- Flight per-turn reset missing
- Angry too broad (fires on poison/thorns, Java only on player attacks)

### 5. Reactive power cards never install
- Static Discharge, Flame Barrier: card tags exist but install_power has no match arm
- Curiosity: helper exists but never called from play_card
