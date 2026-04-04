#[cfg(test)]
mod relic_tests {
    use crate::relics::*;
    use crate::status_ids::sid;
    use crate::state::{CombatState, EnemyCombatState};
    use crate::tests::support::{make_deck, make_deck_n};

    fn state() -> CombatState {
        let e = EnemyCombatState::new("Test", 50, 50);
        CombatState::new(80, 80, vec![e], make_deck_n("Strike_P", 5), 3)
    }

    fn state_with(relic: &str) -> CombatState {
        let mut s = state();
        s.relics.push(relic.to_string());
        apply_combat_start_relics(&mut s);
        s
    }

    // ---- Vajra ----

    #[test] fn vajra_str_1() { assert_eq!(state_with("Vajra").player.strength(), 1); }
    #[test] fn vajra_stacks_with_existing() {
        let mut s = state();
        s.player.set_status(sid::STRENGTH, 3);
        s.relics.push("Vajra".to_string());
        apply_combat_start_relics(&mut s);
        assert_eq!(s.player.strength(), 4);
    }

    // ---- Bag of Marbles ----
    #[test] fn marbles_vuln_all() {
        let s = state_with("Bag of Marbles");
        assert!(s.enemies[0].entity.is_vulnerable());
    }
    #[test] fn marbles_vuln_multi_enemy() {
        let mut s = state();
        s.enemies.push(EnemyCombatState::new("Test2", 30, 30));
        s.relics.push("Bag of Marbles".to_string());
        apply_combat_start_relics(&mut s);
        assert!(s.enemies[0].entity.is_vulnerable());
        assert!(s.enemies[1].entity.is_vulnerable());
    }

    // ---- Thread and Needle ----
    #[test] fn thread_needle_plated_4() {
        assert_eq!(state_with("Thread and Needle").player.status(sid::PLATED_ARMOR), 4);
    }

    // ---- Anchor ----
    #[test] fn anchor_10_block() { assert_eq!(state_with("Anchor").player.block, 10); }

    // ---- Akabeko ----
    #[test] fn akabeko_vigor_8() { assert_eq!(state_with("Akabeko").player.status(sid::VIGOR), 8); }

    // ---- Bronze Scales ----
    #[test] fn bronze_scales_thorns_3() {
        assert_eq!(state_with("Bronze Scales").player.status(sid::THORNS), 3);
    }

    // ---- Blood Vial ----
    #[test] fn blood_vial_heal_2() {
        let mut s = state();
        s.player.hp = 70;
        s.relics.push("Blood Vial".to_string());
        apply_combat_start_relics(&mut s);
        assert_eq!(s.player.hp, 72);
    }
    #[test] fn blood_vial_cap_at_max() {
        let mut s = state();
        s.player.hp = 79;
        s.relics.push("Blood Vial".to_string());
        apply_combat_start_relics(&mut s);
        assert_eq!(s.player.hp, 80);
    }
    #[test] fn blood_vial_at_max_stays() {
        let s = state_with("Blood Vial");
        assert_eq!(s.player.hp, 80);
    }

    // ---- Clockwork Souvenir ----
    #[test] fn clockwork_artifact_1() {
        assert_eq!(state_with("ClockworkSouvenir").player.status(sid::ARTIFACT), 1);
    }

    // ---- Fossilized Helix ----
    #[test] fn helix_buffer_1() {
        assert_eq!(state_with("FossilizedHelix").player.status(sid::BUFFER), 1);
    }

    // ---- Data Disk ----
    #[test] fn data_disk_focus_1() {
        assert_eq!(state_with("Data Disk").player.status(sid::FOCUS), 1);
    }

    // ---- Mark of Pain ----
    #[test] fn mark_of_pain_wounds() {
        let s = state_with("Mark of Pain");
        let reg = crate::cards::CardRegistry::new();
        let w = s.draw_pile.iter().filter(|c| reg.card_name(c.def_id) == "Wound").count();
        assert_eq!(w, 2);
    }

    // ---- Lantern ----
    #[test] fn lantern_ready() {
        let s = state_with("Lantern");
        assert_eq!(s.player.status(sid::LANTERN_READY), 1);
    }
    #[test] fn lantern_turn1() {
        let mut s = state_with("Lantern");
        s.turn = 1;
        apply_lantern_turn_start(&mut s);
        assert_eq!(s.energy, 4);
    }
    #[test] fn lantern_turn2_no() {
        let mut s = state_with("Lantern");
        s.turn = 2;
        apply_lantern_turn_start(&mut s);
        assert_eq!(s.energy, 3);
    }
    #[test] fn lantern_consumed_after_use() {
        let mut s = state_with("Lantern");
        s.turn = 1;
        apply_lantern_turn_start(&mut s);
        assert_eq!(s.player.status(sid::LANTERN_READY), 0);
    }

    // ---- Ornamental Fan ----
    #[test] fn fan_no_block_at_1() {
        let mut s = state_with("Ornamental Fan");
        check_ornamental_fan(&mut s);
        assert_eq!(s.player.block, 0);
    }
    #[test] fn fan_no_block_at_2() {
        let mut s = state_with("Ornamental Fan");
        check_ornamental_fan(&mut s);
        check_ornamental_fan(&mut s);
        assert_eq!(s.player.block, 0);
    }
    #[test] fn fan_block_at_3() {
        let mut s = state_with("Ornamental Fan");
        check_ornamental_fan(&mut s);
        check_ornamental_fan(&mut s);
        check_ornamental_fan(&mut s);
        assert_eq!(s.player.block, 4);
    }
    #[test] fn fan_block_at_6() {
        let mut s = state_with("Ornamental Fan");
        for _ in 0..6 { check_ornamental_fan(&mut s); }
        assert_eq!(s.player.block, 8);
    }
    #[test] fn fan_block_at_9() {
        let mut s = state_with("Ornamental Fan");
        for _ in 0..9 { check_ornamental_fan(&mut s); }
        assert_eq!(s.player.block, 12);
    }
    #[test] fn fan_no_relic_no_effect() {
        let mut s = state();
        for _ in 0..3 { check_ornamental_fan(&mut s); }
        assert_eq!(s.player.block, 0);
    }

    // ---- Pen Nib ----
    #[test] fn pen_nib_not_until_10() {
        let mut s = state_with("Pen Nib");
        for _ in 0..9 { assert!(!check_pen_nib(&mut s)); }
    }
    #[test] fn pen_nib_triggers_at_10() {
        let mut s = state_with("Pen Nib");
        for _ in 0..9 { check_pen_nib(&mut s); }
        assert!(check_pen_nib(&mut s));
    }
    #[test] fn pen_nib_resets() {
        let mut s = state_with("Pen Nib");
        for _ in 0..10 { check_pen_nib(&mut s); }
        assert!(!check_pen_nib(&mut s));
    }
    #[test] fn pen_nib_no_relic() {
        let mut s = state();
        for _ in 0..20 { assert!(!check_pen_nib(&mut s)); }
    }

    // ---- Violet Lotus ----
    #[test] fn violet_lotus_bonus() { assert_eq!(violet_lotus_calm_exit_bonus(&state_with("Violet Lotus")), 1); }
    #[test] fn no_violet_lotus_no_bonus() { assert_eq!(violet_lotus_calm_exit_bonus(&state()), 0); }

    // ---- Torii ----
    #[test] fn torii_reduce_5_to_1() {
        let s = state_with("Torii");
        assert_eq!(apply_torii(&s, 5), 1);
    }
    #[test] fn torii_reduce_2_to_1() {
        let s = state_with("Torii");
        assert_eq!(apply_torii(&s, 2), 1);
    }
    #[test] fn torii_no_reduce_6() {
        let s = state_with("Torii");
        assert_eq!(apply_torii(&s, 6), 6);
    }
    #[test] fn torii_no_reduce_1() {
        let s = state_with("Torii");
        assert_eq!(apply_torii(&s, 1), 1);
    }
    #[test] fn torii_no_reduce_0() {
        let s = state_with("Torii");
        assert_eq!(apply_torii(&s, 0), 0);
    }
    #[test] fn torii_no_relic() {
        let s = state();
        assert_eq!(apply_torii(&s, 3), 3);
    }

    // ---- Tungsten Rod ----
    #[test] fn tungsten_reduce_5_to_4() {
        let s = state_with("TungstenRod");
        assert_eq!(apply_tungsten_rod(&s, 5), 4);
    }
    #[test] fn tungsten_reduce_1_to_0() {
        let s = state_with("TungstenRod");
        assert_eq!(apply_tungsten_rod(&s, 1), 0);
    }
    #[test] fn tungsten_no_reduce_0() {
        let s = state_with("TungstenRod");
        assert_eq!(apply_tungsten_rod(&s, 0), 0);
    }
    #[test] fn tungsten_no_relic() {
        let s = state();
        assert_eq!(apply_tungsten_rod(&s, 5), 5);
    }

    // ---- Multiple relics ----
    #[test] fn three_relics_combined() {
        let mut s = state();
        s.relics.push("Vajra".to_string());
        s.relics.push("Anchor".to_string());
        s.relics.push("Bag of Marbles".to_string());
        apply_combat_start_relics(&mut s);
        assert_eq!(s.player.strength(), 1);
        assert_eq!(s.player.block, 10);
        assert!(s.enemies[0].entity.is_vulnerable());
    }

    // ---- Torii + Boot interaction ----
    #[test] fn torii_and_boot_interact() {
        let mut s = state();
        s.relics.push("Torii".to_string());
        s.relics.push("Boot".to_string());
        apply_combat_start_relics(&mut s);
        // Boot turns 3->5, then Torii turns 5->1
        let after_boot = apply_boot(&s, 3); // 3 -> 5
        let after_torii = apply_torii(&s, after_boot); // 5 -> 1
        assert_eq!(after_torii, 1);
    }
}

// =============================================================================
// Potion exhaustive tests
// =============================================================================

