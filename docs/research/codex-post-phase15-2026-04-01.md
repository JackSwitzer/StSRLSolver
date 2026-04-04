# Codex Post-Phase-1.5 Review (2026-04-01)

## 5 Findings (3 high, 2 medium)

### 1. HIGH: 11 powers still installed but never consumed
- Establishment, RetainCards, Loop, Electrodynamics, Magnetism, Mayhem (from install_power)
- DoppelgangerDraw, DoppelgangerEnergy, WraithForm, EchoForm, CreativeAI (from card_effects)
- Helper consumers exist in powers/buffs.rs but engine.rs never calls them

### 2. HIGH: Trigger ordering mismatches
- Devotion fires pre-draw (should be post-draw)
- Metallicize/PlatedArmor/LikeWater fire before status card damage (should be after in Java?)
- WraithForm not consumed at all (should lose Dex each turn)

### 3. MEDIUM: Raw status strings remain in peripheral files
- state.rs, potions.rs, relics/run.rs, obs.rs, enemies/*.rs still use raw strings

### 4. MEDIUM: 131 card effect tags still unhandled
- High-signal: next_turn_energy/block/draw, double_tap, burst, echo_form, creative_ai
- rampage, finisher, flechettes, genetic_algorithm, dual_wield, recycle, discovery, madness, etc.

### 5. HIGH: Damage pipeline missing generic power hooks
- deal_damage_to_enemy: misses Invincible cap, ModeShift, Angry, Curiosity, Buffer
- deal_damage_to_player: misses Thorns, Flame Barrier, Static Discharge
- Hardcoded double_damage=false, flight=false
- Offering HP loss bypasses Rupture hook
