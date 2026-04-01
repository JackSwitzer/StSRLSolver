# Codex Post-Phase-1 Review (2026-04-01)

## Core Issue
Phase 1 installed 68 handlers but many write statuses the engine never reads.
"Many new installs now write statuses that the live engine never reads."

## 15 Findings (12 high, 3 medium)

### Status keys wrong (fix immediately)
- "Tools of the Trade" → Java "Tools Of The Trade"
- "Well-Laid Plans" → Java "Retain Cards"  
- "Hello World" → Java "Hello"
- "Electrodynamics" → Java "Electro"
- "Sadistic Nature" → Java "Sadistic"

### Statuses installed but never consumed by engine
- Corruption: skills should cost 0 + exhaust (engine.rs effective_cost + play_card)
- Establishment: retained cards cost -1 (engine.rs effective_cost)
- Machine Learning: extra draw per turn (engine.rs start_player_turn)
- Bullet Time: hand costs 0 + No Draw (engine.rs effective_cost + draw_cards)
- Doppelganger: next turn draw+energy (engine.rs start_player_turn)
- Wraith Form: -1 Dex per turn (engine.rs start_player_turn)
- Echo Form: replay first card (engine.rs play_card)
- Creative AI: add random Power to hand (engine.rs start_player_turn)

### Temporary effects never expire
- Rage: should clear at end of turn
- Flex (TempStrength): should remove at end of turn
- Piercing Wail (TempStrengthLoss): should restore at end of turn

### Card behavior bugs
- Ragnarok: incorrectly enters Wrath stance (Java doesn't)
- damage_random_x_times: hits all enemies once THEN random loop (should be random only)
- Conjure Blade: Expunger never reads ExpungerHits
- Electrodynamics: wrong amount, should channel Lightning + set Electro
- Heart painful_stabs: should be ongoing power, not immediate Wound

### Still approximations (medium)
- Meditate, Lesson Learned, Foreign Influence, Omniscience, Wish
- PowerId registry has non-Java IDs
- CardDef missing Java tag field (HEALING, STRIKE, etc.)
