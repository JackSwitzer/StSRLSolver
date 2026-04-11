//! Comprehensive test suite for the Rust combat engine.
//!
//! Organized by module:
//! 1. Card registry — every card base/upgraded data values
//! 2. Damage calculation — rounding, multiplier combos, edge cases
//! 3. Block calculation — dexterity, frail, edge cases
//! 4. Incoming damage — block absorption, stance mult, relics
//! 5. Card play effects — every card effect in the engine
//! 6. Stance mechanics — transitions, energy, power triggers
//! 7. Enemy AI — every enemy pattern, move sequences, special mechanics
//! 8. Relic effects — combat start, per-card, per-turn
//! 9. Potion effects — every potion, targeting, auto-revive
//! 10. Integration — multi-turn combats, combined effects


pub(crate) mod support;
mod test_cards;
mod test_cards_defect;
mod test_cards_ironclad;
mod test_cards_silent;
mod test_cards_watcher;
mod test_damage;
mod test_enemy_ai;
mod test_enemies;
mod test_events_parity;
mod test_bosses;
mod test_integration;
mod test_potions;
mod test_powers;
mod test_relics;
mod test_relics_parity;
mod test_run_parity;
mod test_state;
mod test_interactions;
