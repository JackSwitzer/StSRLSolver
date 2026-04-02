#[cfg(test)]
mod potion_tests {
    use crate::potions::*;
    use crate::status_ids::sid;
    use crate::state::{CombatState, EnemyCombatState};

    fn state() -> CombatState {
        let e = EnemyCombatState::new("Test", 50, 50);
        let mut s = CombatState::new(80, 80, vec![e], vec!["Strike_P".to_string(); 5], 3);
        s.potions = vec!["".to_string(); 3];
        s
    }

    // ---- Fire Potion ----
    #[test] fn fire_20_dmg() {
        let mut s = state();
        apply_potion(&mut s, "Fire Potion", 0);
        assert_eq!(s.enemies[0].entity.hp, 30);
    }
    #[test] fn fire_through_block() {
        let mut s = state();
        s.enemies[0].entity.block = 8;
        apply_potion(&mut s, "Fire Potion", 0);
        assert_eq!(s.enemies[0].entity.hp, 38);
        assert_eq!(s.enemies[0].entity.block, 0);
    }
    #[test] fn fire_kills_enemy() {
        let mut s = state();
        s.enemies[0].entity.hp = 15;
        apply_potion(&mut s, "Fire Potion", 0);
        assert_eq!(s.enemies[0].entity.hp, 0);
    }
    #[test] fn fire_bad_target() { assert!(!apply_potion(&mut state(), "Fire Potion", 5)); }
    #[test] fn fire_neg_target() { assert!(!apply_potion(&mut state(), "Fire Potion", -1)); }
    #[test] fn fire_tracks_damage() {
        let mut s = state();
        apply_potion(&mut s, "Fire Potion", 0);
        assert_eq!(s.total_damage_dealt, 20);
    }

    // ---- Block Potion ----
    #[test] fn block_12() { let mut s = state(); apply_potion(&mut s, "Block Potion", -1); assert_eq!(s.player.block, 12); }
    #[test] fn block_stacks() {
        let mut s = state();
        s.player.block = 5;
        apply_potion(&mut s, "Block Potion", -1);
        assert_eq!(s.player.block, 17);
    }

    // ---- Strength Potion ----
    #[test] fn str_2() { let mut s = state(); apply_potion(&mut s, "Strength Potion", -1); assert_eq!(s.player.strength(), 2); }
    #[test] fn str_stacks() {
        let mut s = state();
        s.player.set_status(sid::STRENGTH, 3);
        apply_potion(&mut s, "Strength Potion", -1);
        assert_eq!(s.player.strength(), 5);
    }

    // ---- Dexterity Potion ----
    #[test] fn dex_2() { let mut s = state(); apply_potion(&mut s, "Dexterity Potion", -1); assert_eq!(s.player.dexterity(), 2); }

    // ---- Energy Potion ----
    #[test] fn energy_2() { let mut s = state(); apply_potion(&mut s, "Energy Potion", -1); assert_eq!(s.energy, 5); }

    // ---- Weak Potion ----
    #[test] fn weak_3() {
        let mut s = state();
        apply_potion(&mut s, "Weak Potion", 0);
        assert_eq!(s.enemies[0].entity.status(sid::WEAKENED), 3);
    }
    #[test] fn weak_bad_target() { assert!(!apply_potion(&mut state(), "Weak Potion", 5)); }

    // ---- Fear Potion ----
    #[test] fn fear_3() {
        let mut s = state();
        apply_potion(&mut s, "FearPotion", 0);
        assert_eq!(s.enemies[0].entity.status(sid::VULNERABLE), 3);
    }

    // ---- Poison Potion ----
    #[test] fn poison_6() {
        let mut s = state();
        apply_potion(&mut s, "Poison Potion", 0);
        assert_eq!(s.enemies[0].entity.status(sid::POISON), 6);
    }

    // ---- Explosive Potion ----
    #[test] fn explosive_all() {
        let mut s = state();
        s.enemies.push(EnemyCombatState::new("T2", 40, 40));
        apply_potion(&mut s, "Explosive Potion", -1);
        assert_eq!(s.enemies[0].entity.hp, 40);
        assert_eq!(s.enemies[1].entity.hp, 30);
    }
    #[test] fn explosive_kills() {
        let mut s = state();
        s.enemies[0].entity.hp = 5;
        apply_potion(&mut s, "Explosive Potion", -1);
        assert_eq!(s.enemies[0].entity.hp, 0);
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
    #[test] fn miracle_2_to_hand() {
        let mut s = state();
        s.hand.clear();
        apply_potion(&mut s, "BottledMiracle", -1);
        assert_eq!(s.hand.len(), 2);
        assert_eq!(s.hand[0], "Miracle");
    }
    #[test] fn miracle_respects_hand_limit() {
        let mut s = state();
        s.hand = vec!["X".to_string(); 9];
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
        let mut s = state();
        s.relics.push("SacredBark".to_string());
        apply_potion(&mut s, "Weak Potion", 0);
        assert_eq!(s.enemies[0].entity.status(sid::WEAKENED), 6); // 3*2
    }
    #[test] fn bark_doubles_poison() {
        let mut s = state();
        s.relics.push("SacredBark".to_string());
        apply_potion(&mut s, "Poison Potion", 0);
        assert_eq!(s.enemies[0].entity.status(sid::POISON), 12); // 6*2
    }
    #[test] fn bark_doubles_regen() {
        let mut s = state();
        s.relics.push("SacredBark".to_string());
        apply_potion(&mut s, "Regen Potion", -1);
        assert_eq!(s.player.status(sid::REGENERATION), 10); // 5*2
    }
    #[test] fn bark_doubles_energy() {
        let mut s = state();
        s.relics.push("SacredBark".to_string());
        apply_potion(&mut s, "Energy Potion", -1);
        assert_eq!(s.energy, 7); // 3 base + 2*2
    }
    #[test] fn bark_doubles_explosive() {
        let mut s = state();
        s.relics.push("SacredBark".to_string());
        apply_potion(&mut s, "Explosive Potion", -1);
        assert_eq!(s.enemies[0].entity.hp, 30); // 50 - 10*2
    }

    // ---- Unknown potion ----
    #[test] fn unknown_potion_succeeds() {
        assert!(apply_potion(&mut state(), "UnknownPotion", -1));
    }
}

// =============================================================================
// Powers module tests
// =============================================================================

