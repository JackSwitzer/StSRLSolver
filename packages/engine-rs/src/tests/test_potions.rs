#[cfg(test)]
mod potion_tests {
    use crate::actions::Action;
    use crate::potions::*;
    use crate::status_ids::sid;
    use crate::state::{CombatState, EnemyCombatState};
    use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, make_deck_n};

    fn state() -> CombatState {
        let e = EnemyCombatState::new("Test", 50, 50);
        let mut s = CombatState::new(80, 80, vec![e], make_deck_n("Strike_P", 5), 3);
        s.potions = vec!["".to_string(); 3];
        s
    }

    fn engine() -> crate::engine::CombatEngine {
        let mut engine = engine_with_state(combat_state_with(
            make_deck_n("Strike_P", 5),
            vec![enemy_no_intent("Test", 50, 50)],
            3,
        ));
        engine.state.potions = vec![String::new(); 3];
        engine
    }

    fn use_potion(engine: &mut crate::engine::CombatEngine, potion_idx: usize, target_idx: i32) {
        engine.execute_action(&Action::UsePotion {
            potion_idx,
            target_idx,
        });
    }

    #[test] fn fire_20_dmg() {
        let mut e = engine();
        e.state.potions[0] = "Fire Potion".to_string();
        use_potion(&mut e, 0, 0);
        assert_eq!(e.state.enemies[0].entity.hp, 30);
    }
    #[test] fn fire_through_block() {
        let mut e = engine();
        e.state.potions[0] = "Fire Potion".to_string();
        e.state.enemies[0].entity.block = 8;
        use_potion(&mut e, 0, 0);
        assert_eq!(e.state.enemies[0].entity.hp, 38);
        assert_eq!(e.state.enemies[0].entity.block, 0);
    }
    #[test] fn fire_kills_enemy() {
        let mut e = engine();
        e.state.potions[0] = "Fire Potion".to_string();
        e.state.enemies[0].entity.hp = 15;
        use_potion(&mut e, 0, 0);
        assert_eq!(e.state.enemies[0].entity.hp, 0);
    }
    #[test] fn fire_bad_target() {
        let e = engine();
        assert!(!e.get_legal_actions().contains(&Action::UsePotion { potion_idx: 0, target_idx: 5 }));
    }
    #[test] fn fire_neg_target() {
        let mut e = engine();
        e.state.potions[0] = "Fire Potion".to_string();
        assert!(!e.get_legal_actions().contains(&Action::UsePotion { potion_idx: 0, target_idx: -1 }));
    }
    #[test] fn fire_tracks_damage() {
        let mut e = engine();
        e.state.potions[0] = "Fire Potion".to_string();
        use_potion(&mut e, 0, 0);
        assert_eq!(e.state.total_damage_dealt, 20);
    }

    // ---- Block Potion ----
    #[test] fn block_12() {
        let mut e = engine();
        e.state.potions[0] = "Block Potion".to_string();
        use_potion(&mut e, 0, -1);
        assert_eq!(e.state.player.block, 12);
    }
    #[test] fn block_stacks() {
        let mut e = engine();
        e.state.potions[0] = "Block Potion".to_string();
        e.state.player.block = 5;
        use_potion(&mut e, 0, -1);
        assert_eq!(e.state.player.block, 17);
    }

    // ---- Strength Potion ----
    #[test] fn str_2() {
        let mut e = engine();
        e.state.potions[0] = "Strength Potion".to_string();
        use_potion(&mut e, 0, -1);
        assert_eq!(e.state.player.strength(), 2);
    }
    #[test] fn str_stacks() {
        let mut e = engine();
        e.state.potions[0] = "Strength Potion".to_string();
        e.state.player.set_status(sid::STRENGTH, 3);
        use_potion(&mut e, 0, -1);
        assert_eq!(e.state.player.strength(), 5);
    }

    // ---- Dexterity Potion ----
    #[test] fn dex_2() {
        let mut e = engine();
        e.state.potions[0] = "Dexterity Potion".to_string();
        use_potion(&mut e, 0, -1);
        assert_eq!(e.state.player.dexterity(), 2);
    }

    // ---- Energy Potion ----
    #[test] fn energy_2() {
        let mut e = engine();
        e.state.potions[0] = "Energy Potion".to_string();
        use_potion(&mut e, 0, -1);
        assert_eq!(e.state.energy, 5);
    }

    // ---- Weak Potion ----
    #[test] fn weak_3() {
        let mut e = engine();
        e.state.potions[0] = "Weak Potion".to_string();
        use_potion(&mut e, 0, 0);
        assert_eq!(e.state.enemies[0].entity.status(sid::WEAKENED), 3);
    }
    #[test] fn weak_bad_target() {
        let mut e = engine();
        e.state.potions[0] = "Weak Potion".to_string();
        assert!(!e.get_legal_actions().contains(&Action::UsePotion { potion_idx: 0, target_idx: 5 }));
    }

    // ---- Fear Potion ----
    #[test] fn fear_3() {
        let mut e = engine();
        e.state.potions[0] = "FearPotion".to_string();
        use_potion(&mut e, 0, 0);
        assert_eq!(e.state.enemies[0].entity.status(sid::VULNERABLE), 3);
    }

    // ---- Poison Potion ----
    #[test] fn poison_6() {
        let mut e = engine();
        e.state.potions[0] = "Poison Potion".to_string();
        use_potion(&mut e, 0, 0);
        assert_eq!(e.state.enemies[0].entity.status(sid::POISON), 6);
    }

    // ---- Explosive Potion ----
    #[test] fn explosive_all() {
        let mut e = engine_with_state(combat_state_with(
            make_deck_n("Strike_P", 5),
            vec![enemy_no_intent("Test", 50, 50), enemy_no_intent("T2", 40, 40)],
            3,
        ));
        e.state.potions = vec!["Explosive Potion".to_string(), String::new(), String::new()];
        use_potion(&mut e, 0, -1);
        assert_eq!(e.state.enemies[0].entity.hp, 40);
        assert_eq!(e.state.enemies[1].entity.hp, 30);
    }
    #[test] fn explosive_kills() {
        let mut e = engine();
        e.state.potions[0] = "Explosive Potion".to_string();
        e.state.enemies[0].entity.hp = 5;
        use_potion(&mut e, 0, -1);
        assert_eq!(e.state.enemies[0].entity.hp, 0);
    }

    // ---- Flex / Steroid ----
    #[test] fn flex_temp_str() {
        let mut s = state();
        apply_potion(&mut s, "SteroidPotion", -1);
        assert_eq!(s.player.strength(), 5);
        assert_eq!(s.player.status(sid::LOSE_STRENGTH), 5);
    }

    // ---- Speed Potion ----
    #[test] fn speed_temp_dex() {
        let mut s = state();
        apply_potion(&mut s, "SpeedPotion", -1);
        assert_eq!(s.player.dexterity(), 5);
        assert_eq!(s.player.status(sid::LOSE_DEXTERITY), 5);
    }

    // ---- Ancient Potion ----
    #[test] fn ancient_artifact() {
        let mut s = state();
        apply_potion(&mut s, "Ancient Potion", -1);
        assert_eq!(s.player.status(sid::ARTIFACT), 1);
    }

    // ---- Regen ----
    #[test] fn regen_5() {
        let mut s = state();
        apply_potion(&mut s, "Regen Potion", -1);
        assert_eq!(s.player.status(sid::REGENERATION), 5);
    }

    // ---- Essence of Steel ----
    #[test] fn essence_plated_4() {
        let mut s = state();
        apply_potion(&mut s, "EssenceOfSteel", -1);
        assert_eq!(s.player.status(sid::PLATED_ARMOR), 4);
    }

    // ---- Liquid Bronze ----
    #[test] fn bronze_thorns_3() {
        let mut s = state();
        apply_potion(&mut s, "LiquidBronze", -1);
        assert_eq!(s.player.status(sid::THORNS), 3);
    }

    // ---- Cultist Potion ----
    #[test] fn cultist_ritual_1() {
        let mut s = state();
        apply_potion(&mut s, "CultistPotion", -1);
        assert_eq!(s.player.status(sid::RITUAL), 1);
    }

    // ---- Bottled Miracle ----
    // Bottled Miracle is on the runtime action path, but helper-path coverage
    // stays here until the runtime path is declared authoritative for this potion.
    #[test] fn miracle_2_to_hand() {
        let mut s = state();
        s.hand.clear();
        apply_potion(&mut s, "BottledMiracle", -1);
        assert_eq!(s.hand.len(), 2);
        let reg = crate::cards::global_registry();
        assert_eq!(reg.card_name(s.hand[0].def_id), "Miracle");
    }
    #[test] fn miracle_respects_hand_limit() {
        let mut s = state();
        s.hand = make_deck_n("X", 9);
        apply_potion(&mut s, "BottledMiracle", -1);
        assert_eq!(s.hand.len(), 10);
    }

    // ---- Fairy ----
    #[test] fn fairy_no_manual_use() { assert!(!apply_potion(&mut state(), "FairyPotion", -1)); }
    #[test] fn fairy_check_none() { assert_eq!(check_fairy_revive(&state()), 0); }
    #[test] fn fairy_check_present() {
        let mut s = state();
        s.potions[0] = "FairyPotion".to_string();
        assert_eq!(check_fairy_revive(&s), 24);
    }
    #[test] fn fairy_check_alt_name() {
        let mut s = state();
        s.potions[1] = "Fairy in a Bottle".to_string();
        assert_eq!(check_fairy_revive(&s), 24);
    }
    #[test] fn fairy_consume_slot() {
        let mut s = state();
        s.potions[2] = "FairyPotion".to_string();
        consume_fairy(&mut s);
        assert!(s.potions[2].is_empty());
    }
    #[test] fn fairy_30pct_values() {
        let mut s = state();
        s.potions[0] = "FairyPotion".to_string();
        s.player.max_hp = 100;
        assert_eq!(check_fairy_revive(&s), 30);
    }

    // ---- requires_target ----
    #[test] fn target_fire() { assert!(potion_requires_target("Fire Potion")); }
    #[test] fn target_weak() { assert!(potion_requires_target("Weak Potion")); }
    #[test] fn target_fear() { assert!(potion_requires_target("FearPotion")); }
    #[test] fn target_poison() { assert!(potion_requires_target("Poison Potion")); }
    #[test] fn no_target_block() { assert!(!potion_requires_target("Block Potion")); }
    #[test] fn no_target_str() { assert!(!potion_requires_target("Strength Potion")); }
    #[test] fn no_target_energy() { assert!(!potion_requires_target("Energy Potion")); }
    #[test] fn no_target_dex() { assert!(!potion_requires_target("Dexterity Potion")); }
    #[test] fn no_target_explosive() { assert!(!potion_requires_target("Explosive Potion")); }

    // ---- Sacred Bark doubles potions ----
    #[test] fn bark_doubles_weakness() {
        let mut e = engine();
        e.state.relics.push("SacredBark".to_string());
        e.state.potions[0] = "Weak Potion".to_string();
        use_potion(&mut e, 0, 0);
        assert_eq!(e.state.enemies[0].entity.status(sid::WEAKENED), 6); // 3*2
    }
    #[test] fn bark_doubles_poison() {
        let mut e = engine();
        e.state.relics.push("SacredBark".to_string());
        e.state.potions[0] = "Poison Potion".to_string();
        use_potion(&mut e, 0, 0);
        assert_eq!(e.state.enemies[0].entity.status(sid::POISON), 12); // 6*2
    }
    #[test] fn bark_doubles_regen() {
        let mut s = state();
        s.relics.push("SacredBark".to_string());
        apply_potion(&mut s, "Regen Potion", -1);
        assert_eq!(s.player.status(sid::REGENERATION), 10); // 5*2
    }
    #[test] fn bark_doubles_energy() {
        let mut e = engine();
        e.state.relics.push("SacredBark".to_string());
        e.state.potions[0] = "Energy Potion".to_string();
        use_potion(&mut e, 0, -1);
        assert_eq!(e.state.energy, 7); // 3 base + 2*2
    }
    #[test] fn bark_doubles_explosive() {
        let mut e = engine();
        e.state.relics.push("SacredBark".to_string());
        e.state.potions[0] = "Explosive Potion".to_string();
        use_potion(&mut e, 0, -1);
        assert_eq!(e.state.enemies[0].entity.hp, 30); // 50 - 10*2
    }

    // ---- Unknown potion ----
    #[test] fn unknown_potion_succeeds() {
        assert!(apply_potion(&mut state(), "UnknownPotion", -1));
    }
}

// =============================================================================
// Powers module tests
// =============================================================================
