//! Combat-relevant relic effects for MCTS simulations.
//!
//! Implements ALL 187 relics from Slay the Spire. Relics are grouped by
//! trigger type matching the Java AbstractRelic hooks:
//!
//! - Combat start: atBattleStart / atPreBattle / atBattleStartPreDraw
//! - Turn start: atTurnStart / atTurnStartPostDraw
//! - On card play: onUseCard / onPlayCard
//! - Turn end: onPlayerEndTurn
//! - On HP loss: wasHPLost
//! - On shuffle: onShuffle
//! - On enemy death: onMonsterDeath
//! - Combat end: onVictory
//! - Passive / non-combat: gold, map, shop relics (stub — just track ownership)


use crate::status_ids::sid;
pub mod combat;
pub mod run;

// Re-export all relic functions from sub-modules
pub use combat::apply_combat_start_relics;
pub use combat::apply_turn_start_relics;
pub use combat::apply_lantern_turn_start;
pub use combat::on_card_played;
pub use combat::check_ornamental_fan;
pub use combat::check_pen_nib;
pub use combat::velvet_choker_can_play;
pub use combat::apply_turn_end_relics;

pub use run::on_hp_loss;
pub use run::on_shuffle;
pub use run::on_enemy_death;
pub use run::on_victory;
pub use run::apply_boot;
pub use run::apply_torii;
pub use run::apply_tungsten_rod;
pub use run::champion_belt_on_vulnerable;
pub use run::charons_ashes_on_exhaust;
pub use run::dead_branch_on_exhaust;
pub use run::tough_bandages_on_discard;
pub use run::tingsha_on_discard;
pub use run::toy_ornithopter_on_potion;
pub use run::hand_drill_on_block_break;
pub use run::strike_dummy_bonus;
pub use run::wrist_blade_bonus;
pub use run::snecko_skull_bonus;
pub use run::chemical_x_bonus;
pub use run::gold_plated_cables_active;
pub use run::violet_lotus_calm_exit_bonus;
pub use run::unceasing_top_should_draw;
pub use run::has_runic_pyramid;
pub use run::calipers_block_retention;
pub use run::has_ice_cream;
pub use run::has_sacred_bark;
pub use run::necronomicon_should_trigger;
pub use run::necronomicon_mark_used;
pub use run::necronomicon_reset;

// ==========================================================================
// TESTS
// ==========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{CardRegistry, CardType};
    use crate::state::{CombatState, EnemyCombatState};
    use crate::tests::support::{make_deck, make_deck_n};

    fn make_test_state() -> CombatState {
        let enemy = EnemyCombatState::new("JawWorm", 44, 44);
        CombatState::new(80, 80, vec![enemy], make_deck_n("Strike_P", 5), 3)
    }

    fn make_two_enemy_state() -> CombatState {
        let e1 = EnemyCombatState::new("JawWorm", 44, 44);
        let e2 = EnemyCombatState::new("Cultist", 50, 50);
        CombatState::new(80, 80, vec![e1, e2], make_deck_n("Strike_P", 5), 3)
    }

    // --- Combat start tests ---

    #[test]
    fn test_vajra_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Vajra".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 1);
    }

    #[test]
    fn test_oddly_smooth_stone() {
        let mut state = make_test_state();
        state.relics.push("Oddly Smooth Stone".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.dexterity(), 1);
    }

    #[test]
    fn test_bag_of_marbles_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Bag of Marbles".to_string());
        apply_combat_start_relics(&mut state);
        assert!(state.enemies[0].entity.is_vulnerable());
    }

    #[test]
    fn test_red_mask_combat_start() {
        let mut state = make_two_enemy_state();
        state.relics.push("Red Mask".to_string());
        apply_combat_start_relics(&mut state);
        assert!(state.enemies[0].entity.is_weak());
        assert!(state.enemies[1].entity.is_weak());
    }

    #[test]
    fn test_thread_and_needle_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Thread and Needle".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.status(sid::PLATED_ARMOR), 4);
    }

    #[test]
    fn test_anchor_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Anchor".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.block, 10);
    }

    #[test]
    fn test_akabeko_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Akabeko".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.status(sid::VIGOR), 8);
    }

    #[test]
    fn test_blood_vial_combat_start() {
        let mut state = make_test_state();
        state.player.hp = 70;
        state.relics.push("Blood Vial".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.hp, 72);
    }

    #[test]
    fn test_blood_vial_does_not_exceed_max() {
        let mut state = make_test_state();
        state.player.hp = 79;
        state.relics.push("Blood Vial".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.hp, 80);
    }

    #[test]
    fn test_twisted_funnel() {
        let mut state = make_two_enemy_state();
        state.relics.push("TwistedFunnel".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.enemies[0].entity.status(sid::POISON), 4);
        assert_eq!(state.enemies[1].entity.status(sid::POISON), 4);
    }

    #[test]
    fn test_mutagenic_strength() {
        let mut state = make_test_state();
        state.relics.push("MutagenicStrength".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 3);
        assert_eq!(state.player.status(sid::LOSE_STRENGTH), 3);
    }

    #[test]
    fn test_ninja_scroll() {
        let mut state = make_test_state();
        state.hand.clear();
        state.relics.push("Ninja Scroll".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.hand.len(), 3);
        let reg = CardRegistry::new();
        assert!(state.hand.iter().all(|c| reg.card_name(c.def_id) == "Shiv"));
    }

    #[test]
    fn test_pure_water_adds_miracle() {
        let mut state = make_test_state();
        state.relics.push("PureWater".to_string());
        let hand_before = state.hand.len();
        apply_combat_start_relics(&mut state);
        assert_eq!(state.hand.len(), hand_before + 1);
        let reg = CardRegistry::new();
        assert_eq!(reg.card_name(state.hand.last().unwrap().def_id), "Miracle");
    }

    #[test]
    fn test_mark_of_pain() {
        let mut state = make_test_state();
        state.relics.push("Mark of Pain".to_string());
        let initial_draw_size = state.draw_pile.len();
        apply_combat_start_relics(&mut state);
        assert_eq!(state.draw_pile.len(), initial_draw_size + 2);
        let reg = CardRegistry::new();
        let wound_count = state.draw_pile.iter().filter(|c| reg.card_name(c.def_id) == "Wound").count();
        assert_eq!(wound_count, 2);
    }

    #[test]
    fn test_teardrop_locket() {
        let mut state = make_test_state();
        state.relics.push("TeardropLocket".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.stance, crate::state::Stance::Calm);
    }

    #[test]
    fn test_multiple_relics() {
        let mut state = make_test_state();
        state.relics.push("Vajra".to_string());
        state.relics.push("Bag of Marbles".to_string());
        state.relics.push("Anchor".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 1);
        assert!(state.enemies[0].entity.is_vulnerable());
        assert_eq!(state.player.block, 10);
    }

    // --- Turn start tests ---

    #[test]
    fn test_lantern_turn1_energy() {
        let mut state = make_test_state();
        state.relics.push("Lantern".to_string());
        state.turn = 0;
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.status(sid::LANTERN_READY), 1);
        state.turn = 1;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.energy, 4);
        assert_eq!(state.player.status(sid::LANTERN_READY), 0);
    }

    #[test]
    fn test_lantern_not_turn2() {
        let mut state = make_test_state();
        state.relics.push("Lantern".to_string());
        apply_combat_start_relics(&mut state);
        state.turn = 2;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.energy, 3);
    }

    #[test]
    fn test_happy_flower_every_3_turns() {
        let mut state = make_test_state();
        state.relics.push("Happy Flower".to_string());
        apply_combat_start_relics(&mut state);

        state.turn = 1;
        state.energy = 3;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.energy, 3); // counter=1

        state.turn = 2;
        state.energy = 3;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.energy, 3); // counter=2

        state.turn = 3;
        state.energy = 3;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.energy, 4); // counter=3 -> +1, reset to 0
    }

    #[test]
    fn test_mercury_hourglass() {
        let mut state = make_test_state();
        state.relics.push("Mercury Hourglass".to_string());
        let hp_before = state.enemies[0].entity.hp;
        state.turn = 1;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp_before - 3);
    }

    #[test]
    fn test_incense_burner_every_6_turns() {
        let mut state = make_test_state();
        state.relics.push("Incense Burner".to_string());
        for turn in 1..=6 {
            state.turn = turn;
            apply_turn_start_relics(&mut state);
        }
        assert_eq!(state.player.status(sid::INTANGIBLE), 1);
    }

    #[test]
    fn test_brimstone() {
        let mut state = make_test_state();
        state.relics.push("Brimstone".to_string());
        state.turn = 1;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.player.strength(), 2);
        assert_eq!(state.enemies[0].entity.strength(), 1);
    }

    #[test]
    fn test_horn_cleat_turn2() {
        let mut state = make_test_state();
        state.relics.push("HornCleat".to_string());
        apply_combat_start_relics(&mut state);

        state.turn = 1;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.player.block, 0); // Not yet

        state.turn = 2;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.player.block, 14);
    }

    #[test]
    fn test_captains_wheel_turn3() {
        let mut state = make_test_state();
        state.relics.push("CaptainsWheel".to_string());
        apply_combat_start_relics(&mut state);

        for t in 1..=2 {
            state.turn = t;
            apply_turn_start_relics(&mut state);
        }
        assert_eq!(state.player.block, 0);

        state.turn = 3;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.player.block, 18);
    }

    // --- On card play tests ---

    #[test]
    fn test_ornamental_fan_every_3_attacks() {
        let mut state = make_test_state();
        state.relics.push("Ornamental Fan".to_string());
        apply_combat_start_relics(&mut state);

        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.block, 0);
        on_card_played(&mut state, CardType::Skill); // Skills don't count
        assert_eq!(state.player.block, 0);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.block, 0); // Only 2 attacks so far
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.block, 4); // 3rd attack triggers
    }

    #[test]
    fn test_kunai_every_3_attacks() {
        let mut state = make_test_state();
        state.relics.push("Kunai".to_string());
        apply_combat_start_relics(&mut state);

        on_card_played(&mut state, CardType::Attack);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.dexterity(), 0);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.dexterity(), 1);
    }

    #[test]
    fn test_shuriken_every_3_attacks() {
        let mut state = make_test_state();
        state.relics.push("Shuriken".to_string());
        apply_combat_start_relics(&mut state);

        for _ in 0..3 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.player.strength(), 1);
    }

    #[test]
    fn test_nunchaku_every_10_attacks() {
        let mut state = make_test_state();
        state.relics.push("Nunchaku".to_string());
        let base_energy = state.energy;

        for _ in 0..10 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.energy, base_energy + 1);
    }

    #[test]
    fn test_pen_nib_every_10_attacks() {
        let mut state = make_test_state();
        state.relics.push("Pen Nib".to_string());
        apply_combat_start_relics(&mut state);

        for _ in 0..9 {
            assert!(!check_pen_nib(&mut state));
        }
        assert!(check_pen_nib(&mut state));
    }

    #[test]
    fn test_bird_faced_urn() {
        let mut state = make_test_state();
        state.player.hp = 70;
        state.relics.push("Bird Faced Urn".to_string());
        on_card_played(&mut state, CardType::Power);
        assert_eq!(state.player.hp, 72);
    }

    #[test]
    fn test_velvet_choker_limit() {
        let mut state = make_test_state();
        state.relics.push("Velvet Choker".to_string());
        apply_combat_start_relics(&mut state);

        for _ in 0..6 {
            assert!(velvet_choker_can_play(&state));
            on_card_played(&mut state, CardType::Attack);
        }
        assert!(!velvet_choker_can_play(&state));
    }

    #[test]
    fn test_orange_pellets_all_types() {
        let mut state = make_test_state();
        state.relics.push("OrangePellets".to_string());
        apply_combat_start_relics(&mut state);
        state.player.add_status(sid::WEAKENED, 3);
        state.player.add_status(sid::VULNERABLE, 2);

        on_card_played(&mut state, CardType::Attack);
        on_card_played(&mut state, CardType::Skill);
        assert!(state.player.is_weak()); // Not yet
        on_card_played(&mut state, CardType::Power);
        assert!(!state.player.is_weak()); // Cleared!
        assert!(!state.player.is_vulnerable());
    }

    // --- Turn end tests ---

    #[test]
    fn test_orichalcum_no_block() {
        let mut state = make_test_state();
        state.relics.push("Orichalcum".to_string());
        state.player.block = 0;
        apply_turn_end_relics(&mut state);
        assert_eq!(state.player.block, 6);
    }

    #[test]
    fn test_orichalcum_has_block() {
        let mut state = make_test_state();
        state.relics.push("Orichalcum".to_string());
        state.player.block = 5;
        apply_turn_end_relics(&mut state);
        assert_eq!(state.player.block, 5); // No change
    }

    #[test]
    fn test_cloak_clasp() {
        let mut state = make_test_state();
        state.relics.push("CloakClasp".to_string());
        state.hand = make_deck(&["a", "b", "c"]);
        apply_turn_end_relics(&mut state);
        assert_eq!(state.player.block, 3);
    }

    // --- On HP loss tests ---

    #[test]
    fn test_centennial_puzzle() {
        let mut state = make_test_state();
        state.relics.push("Centennial Puzzle".to_string());
        apply_combat_start_relics(&mut state);

        on_hp_loss(&mut state, 5);
        assert_eq!(state.player.status(sid::CENTENNIAL_PUZZLE_DRAW), 3);

        // Second hit: no more draws
        on_hp_loss(&mut state, 5);
        // CentennialPuzzleReady is already 0
    }

    #[test]
    fn test_self_forming_clay() {
        let mut state = make_test_state();
        state.relics.push("Self Forming Clay".to_string());
        on_hp_loss(&mut state, 5);
        assert_eq!(state.player.status(sid::NEXT_TURN_BLOCK), 3);
    }

    #[test]
    fn test_red_skull_activation() {
        let mut state = make_test_state();
        state.relics.push("Red Skull".to_string());
        state.player.hp = 41; // Above 50%
        on_hp_loss(&mut state, 5);
        assert_eq!(state.player.status(sid::RED_SKULL_ACTIVE), 0);

        state.player.hp = 39; // Below 50%
        on_hp_loss(&mut state, 1);
        assert_eq!(state.player.status(sid::RED_SKULL_ACTIVE), 1);
        assert_eq!(state.player.strength(), 3);
    }

    // --- On shuffle tests ---

    #[test]
    fn test_sundial_every_3_shuffles() {
        let mut state = make_test_state();
        state.relics.push("Sundial".to_string());
        let base_energy = state.energy;

        on_shuffle(&mut state);
        on_shuffle(&mut state);
        assert_eq!(state.energy, base_energy);
        on_shuffle(&mut state);
        assert_eq!(state.energy, base_energy + 2);
    }

    #[test]
    fn test_abacus() {
        let mut state = make_test_state();
        state.relics.push("TheAbacus".to_string());
        on_shuffle(&mut state);
        assert_eq!(state.player.block, 6);
    }

    // --- On enemy death tests ---

    #[test]
    fn test_gremlin_horn() {
        let mut state = make_two_enemy_state();
        state.relics.push("Gremlin Horn".to_string());
        let base_energy = state.energy;
        state.enemies[0].entity.hp = 0; // Kill first
        on_enemy_death(&mut state, 0);
        assert_eq!(state.energy, base_energy + 1);
    }

    #[test]
    fn test_the_specimen() {
        let mut state = make_two_enemy_state();
        state.relics.push("The Specimen".to_string());
        state.enemies[0].entity.add_status(sid::POISON, 5);
        state.enemies[0].entity.hp = 0; // Kill first
        on_enemy_death(&mut state, 0);
        assert_eq!(state.enemies[1].entity.status(sid::POISON), 5);
    }

    // --- Combat end tests ---

    #[test]
    fn test_burning_blood() {
        let mut state = make_test_state();
        state.relics.push("Burning Blood".to_string());
        let heal = on_victory(&mut state);
        assert_eq!(heal, 6);
    }

    #[test]
    fn test_black_blood() {
        let mut state = make_test_state();
        state.relics.push("Black Blood".to_string());
        let heal = on_victory(&mut state);
        assert_eq!(heal, 12);
    }

    // --- Damage modifier tests ---

    #[test]
    fn test_boot_minimum_damage() {
        let mut state = make_test_state();
        state.relics.push("Boot".to_string());
        assert_eq!(apply_boot(&state, 3), 5);
        assert_eq!(apply_boot(&state, 0), 0);
        assert_eq!(apply_boot(&state, 7), 7);
    }

    #[test]
    fn test_charons_ashes() {
        let mut state = make_two_enemy_state();
        state.relics.push("Charon's Ashes".to_string());
        let hp0 = state.enemies[0].entity.hp;
        let hp1 = state.enemies[1].entity.hp;
        charons_ashes_on_exhaust(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp0 - 3);
        assert_eq!(state.enemies[1].entity.hp, hp1 - 3);
    }

    #[test]
    fn test_tough_bandages() {
        let mut state = make_test_state();
        state.relics.push("Tough Bandages".to_string());
        tough_bandages_on_discard(&mut state);
        assert_eq!(state.player.block, 3);
    }

    #[test]
    fn test_violet_lotus_bonus() {
        let mut state = make_test_state();
        assert_eq!(violet_lotus_calm_exit_bonus(&state), 0);
        state.relics.push("Violet Lotus".to_string());
        assert_eq!(violet_lotus_calm_exit_bonus(&state), 1);
    }

    #[test]
    fn test_calipers_block_retention() {
        let mut state = make_test_state();
        state.relics.push("Calipers".to_string());
        assert_eq!(calipers_block_retention(&state, 20), 15);
        assert_eq!(calipers_block_retention(&state, 10), 10);
    }

    #[test]
    fn test_chemical_x_bonus() {
        let mut state = make_test_state();
        assert_eq!(chemical_x_bonus(&state), 0);
        state.relics.push("Chemical X".to_string());
        assert_eq!(chemical_x_bonus(&state), 2);
    }

    #[test]
    fn test_unceasing_top() {
        let mut state = make_test_state();
        state.relics.push("Unceasing Top".to_string());
        state.hand.clear();
        assert!(unceasing_top_should_draw(&state));
        { let reg = crate::cards::CardRegistry::new(); state.hand.push(reg.make_card("Strike")); };
        assert!(!unceasing_top_should_draw(&state));
    }

    #[test]
    fn test_necronomicon() {
        let mut state = make_test_state();
        state.relics.push("Necronomicon".to_string());
        assert!(necronomicon_should_trigger(&state, 2, true));
        assert!(!necronomicon_should_trigger(&state, 1, true));
        assert!(!necronomicon_should_trigger(&state, 2, false));
        necronomicon_mark_used(&mut state);
        assert!(!necronomicon_should_trigger(&state, 2, true));
    }

    #[test]
    fn test_toy_ornithopter() {
        let mut state = make_test_state();
        state.player.hp = 70;
        state.relics.push("Toy Ornithopter".to_string());
        toy_ornithopter_on_potion(&mut state);
        assert_eq!(state.player.hp, 75);
    }

    #[test]
    fn test_hand_drill() {
        let mut state = make_test_state();
        state.relics.push("HandDrill".to_string());
        hand_drill_on_block_break(&mut state, 0);
        assert_eq!(state.enemies[0].entity.status(sid::VULNERABLE), 2);
    }

    #[test]
    fn test_pantograph_boss() {
        let mut state = make_test_state();
        state.enemies[0] = EnemyCombatState::new("Hexaghost", 250, 250);
        state.player.hp = 50;
        state.relics.push("Pantograph".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.hp, 75);
    }

    #[test]
    fn test_letter_opener_hits_all_enemies() {
        let mut state = make_two_enemy_state();
        state.relics.push("Letter Opener".to_string());
        apply_combat_start_relics(&mut state);
        let hp0 = state.enemies[0].entity.hp;
        let hp1 = state.enemies[1].entity.hp;

        on_card_played(&mut state, CardType::Skill);
        on_card_played(&mut state, CardType::Skill);
        assert_eq!(state.enemies[0].entity.hp, hp0);
        assert_eq!(state.enemies[1].entity.hp, hp1);
        on_card_played(&mut state, CardType::Skill);
        assert_eq!(state.enemies[0].entity.hp, hp0 - 5);
        assert_eq!(state.enemies[1].entity.hp, hp1 - 5);
    }

    #[test]
    fn test_duality_yang() {
        let mut state = make_test_state();
        state.relics.push("Yang".to_string());
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.dexterity(), 1);
        assert_eq!(state.player.status(sid::LOSE_DEXTERITY), 1);
    }

    #[test]
    fn test_stone_calendar_turn7() {
        // Use a high-HP enemy so it survives 52 damage
        let enemy = EnemyCombatState::new("Boss", 200, 200);
        let mut state = CombatState::new(80, 80, vec![enemy], make_deck_n("Strike_P", 5), 3);
        state.relics.push("StoneCalendar".to_string());
        apply_combat_start_relics(&mut state);

        for t in 1..=7 {
            state.turn = t;
            apply_turn_start_relics(&mut state);
        }
        let hp_before = state.enemies[0].entity.hp;
        apply_turn_end_relics(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp_before - 52);
    }

    #[test]
    fn test_ink_bottle_every_10_cards() {
        let mut state = make_test_state();
        state.relics.push("InkBottle".to_string());
        for _ in 0..9 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.player.status(sid::INK_BOTTLE_DRAW), 0);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.status(sid::INK_BOTTLE_DRAW), 1);
    }

    #[test]
    fn test_tingsha() {
        let mut state = make_test_state();
        state.relics.push("Tingsha".to_string());
        let hp = state.enemies[0].entity.hp;
        tingsha_on_discard(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp - 3);
    }

    // --- Torii tests ---

    #[test]
    fn test_torii_reduces_small_damage() {
        let mut state = make_test_state();
        state.relics.push("Torii".to_string());
        assert_eq!(apply_torii(&state, 5), 1); // 5 -> 1
        assert_eq!(apply_torii(&state, 3), 1); // 3 -> 1
        assert_eq!(apply_torii(&state, 2), 1); // 2 -> 1
    }

    #[test]
    fn test_torii_no_effect_on_high_damage() {
        let mut state = make_test_state();
        state.relics.push("Torii".to_string());
        assert_eq!(apply_torii(&state, 6), 6);
        assert_eq!(apply_torii(&state, 20), 20);
    }

    #[test]
    fn test_torii_no_effect_on_zero_or_one() {
        let mut state = make_test_state();
        state.relics.push("Torii".to_string());
        assert_eq!(apply_torii(&state, 0), 0);
        assert_eq!(apply_torii(&state, 1), 1);
    }

    // --- Tungsten Rod tests ---

    #[test]
    fn test_tungsten_rod_reduces_by_one() {
        let mut state = make_test_state();
        state.relics.push("TungstenRod".to_string());
        assert_eq!(apply_tungsten_rod(&state, 5), 4);
        assert_eq!(apply_tungsten_rod(&state, 1), 0);
        assert_eq!(apply_tungsten_rod(&state, 0), 0);
    }

    // --- Stone Calendar fires only once ---

    #[test]
    fn test_stone_calendar_fires_once() {
        let enemy = EnemyCombatState::new("Boss", 200, 200);
        let mut state = CombatState::new(80, 80, vec![enemy], make_deck_n("Strike_P", 5), 3);
        state.relics.push("StoneCalendar".to_string());
        apply_combat_start_relics(&mut state);

        for t in 1..=7 {
            state.turn = t;
            apply_turn_start_relics(&mut state);
        }
        let hp_before = state.enemies[0].entity.hp;
        apply_turn_end_relics(&mut state); // Turn 7 end: should fire
        assert_eq!(state.enemies[0].entity.hp, hp_before - 52);

        // Turn 8: should NOT fire again
        state.turn = 8;
        apply_turn_start_relics(&mut state);
        let hp_after_t8 = state.enemies[0].entity.hp;
        apply_turn_end_relics(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp_after_t8); // No additional damage
    }
}
