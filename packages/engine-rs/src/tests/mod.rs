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
mod test_card_runtime_backend_wave1;
mod test_card_runtime_backend_wave2;
mod test_card_runtime_attack_context_wave1;
mod test_card_runtime_defect_wave1;
mod test_card_runtime_defect_wave2;
mod test_card_runtime_defect_wave8;
mod test_card_runtime_defect_wave9;
mod test_card_runtime_defect_wave11;
mod test_card_runtime_defect_wave12;
mod test_card_runtime_defect_wave13;
mod test_card_runtime_defect_wave14;
mod test_card_runtime_defect_wave15;
mod test_card_runtime_defect_wave16;
mod test_card_runtime_defect_wave17;
mod test_card_runtime_instance_mutation_wave1;
mod test_card_runtime_xcount_wave1;
mod test_card_runtime_xcount_wave2;
mod test_card_runtime_xcount_wave3;
mod test_card_runtime_colorless_wave1;
mod test_card_runtime_colorless_wave2;
mod test_card_runtime_colorless_wave3;
mod test_card_runtime_colorless_wave4;
mod test_card_runtime_colorless_wave5;
mod test_card_runtime_colorless_wave6;
mod test_card_runtime_colorless_wave7;
mod test_card_runtime_ironclad_wave1;
mod test_card_runtime_ironclad_wave2;
mod test_card_runtime_ironclad_wave8;
mod test_card_runtime_ironclad_wave9;
mod test_card_runtime_ironclad_wave10;
mod test_card_runtime_ironclad_wave11;
mod test_card_runtime_ironclad_wave12;
mod test_card_runtime_ironclad_wave13;
mod test_card_runtime_ironclad_wave14;
mod test_card_runtime_silent_wave1;
mod test_card_runtime_silent_wave2;
mod test_card_runtime_silent_wave8;
mod test_card_runtime_silent_wave9;
mod test_card_runtime_silent_wave10;
mod test_card_runtime_silent_wave11;
mod test_card_runtime_silent_wave12;
mod test_card_runtime_silent_wave13;
mod test_card_runtime_silent_wave14;
mod test_card_runtime_nonplay_triggers_wave1;
mod test_card_runtime_support_wave1;
mod test_zone_batch_java_wave1;
mod test_zone_batch_java_wave3;
mod test_dead_system_cleanup_wave1;
mod test_dead_system_cleanup_wave2;
mod test_dead_system_cleanup_wave2_power;
mod test_dead_system_cleanup_wave3;
mod test_dead_system_cleanup_wave4;
mod test_card_runtime_silent_wave_java1;
mod test_card_play_timing_java_wave1;
mod test_card_runtime_watcher_wave1;
mod test_card_runtime_watcher_wave2;
mod test_card_runtime_watcher_wave8;
mod test_card_runtime_watcher_wave10;
mod test_card_runtime_watcher_wave11;
mod test_card_runtime_watcher_wave12;
mod test_card_runtime_watcher_wave13;
mod test_card_runtime_watcher_wave14;
mod test_card_runtime_watcher_wave15;
mod test_card_runtime_watcher_wave16;
mod test_card_runtime_watcher_wave17;
mod test_card_runtime_watcher_wave18;
mod test_card_runtime_watcher_wave19;
mod test_card_runtime_watcher_wave20;
mod test_card_runtime_watcher_wave21;
mod test_card_runtime_watcher_wave22;
mod test_card_runtime_watcher_wave23;
mod test_card_runtime_watcher_wave24;
mod test_card_runtime_watcher_wave25;
mod test_card_runtime_post_damage_wave1;
mod test_card_runtime_temp_wave1;
mod test_cards_watcher;
mod test_generated_choice_java_wave1;
mod test_generated_choice_java_wave2;
mod test_card_runtime_generated_choice_wave4;
mod test_card_runtime_generated_choice_wave5;
mod test_card_runtime_generated_choice_wave6;
mod test_card_legality_wave1;
mod test_damage;
mod test_enemy_ai;
mod test_enemies;
mod test_events_parity;
mod test_event_runtime_wave2;
mod test_event_runtime_wave3;
mod test_event_runtime_wave7;
mod test_entity_runtime;
mod test_bosses;
mod test_integration;
mod test_potions;
mod test_potion_runtime_wave2;
mod test_potion_runtime_wave3;
mod test_potion_runtime_wave7;
mod test_powers;
mod test_power_runtime_metadata_wave1;
mod test_relic_runtime_wave3;
mod test_relic_runtime_wave5;
mod test_relic_runtime_wave6;
mod test_relic_runtime_wave7;
mod test_relic_runtime_wave8;
mod test_relic_runtime_wave9;
mod test_relic_runtime_wave10;
mod test_relic_runtime_wave11;
mod test_relic_runtime_wave12;
mod test_relic_runtime_wave13;
mod test_relic_runtime_wave14;
mod test_relic_runtime_wave15;
mod test_relic_runtime_wave16;
mod test_relic_runtime_wave17;
mod test_relic_runtime_wave18;
mod test_relic_runtime_java_green1;
mod test_dead_system_cleanup_wave5;
mod test_dead_system_cleanup_wave6;
mod test_dead_system_cleanup_wave7;
mod test_dead_system_cleanup_wave8;
mod test_dead_system_cleanup_wave9;
mod test_dead_system_cleanup_wave10;
mod test_dead_system_cleanup_wave11;
mod test_dead_system_cleanup_wave13;
mod test_dead_system_cleanup_wave14;
mod test_dead_system_cleanup_wave15;
mod test_dead_system_cleanup_wave16;
mod test_dead_system_cleanup_wave17;
mod test_reward_relic_runtime_wave2;
mod test_reward_relic_runtime_wave3;
mod test_reward_runtime;
mod test_rl_contract;
mod test_run_parity;
mod test_search_harness;
mod test_state;
mod test_interactions;
mod test_runtime_inline_cutover_wave1;
mod test_runtime_inline_cutover_wave2;
