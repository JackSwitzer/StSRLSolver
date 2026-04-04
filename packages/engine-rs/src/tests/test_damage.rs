#[cfg(test)]
mod damage_tests {
    use crate::damage::*;

    // ---- Basic outgoing ----

    #[test] fn basic_6() { assert_eq!(calculate_damage(6, 0, false, 1.0, false, false), 6); }
    #[test] fn basic_0() { assert_eq!(calculate_damage(0, 0, false, 1.0, false, false), 0); }
    #[test] fn basic_1() { assert_eq!(calculate_damage(1, 0, false, 1.0, false, false), 1); }
    #[test] fn basic_100() { assert_eq!(calculate_damage(100, 0, false, 1.0, false, false), 100); }

    // ---- Strength ----

    #[test] fn str_positive() { assert_eq!(calculate_damage(6, 3, false, 1.0, false, false), 9); }
    #[test] fn str_large() { assert_eq!(calculate_damage(6, 10, false, 1.0, false, false), 16); }
    #[test] fn str_negative() { assert_eq!(calculate_damage(6, -2, false, 1.0, false, false), 4); }
    #[test] fn str_neg_floor_zero() { assert_eq!(calculate_damage(5, -10, false, 1.0, false, false), 0); }
    #[test] fn str_neg_exact_zero() { assert_eq!(calculate_damage(5, -5, false, 1.0, false, false), 0); }

    // ---- Weak ----

    #[test] fn weak_10() { assert_eq!(calculate_damage(10, 0, true, 1.0, false, false), 7); }
    #[test] fn weak_8() { assert_eq!(calculate_damage(8, 0, true, 1.0, false, false), 6); }
    #[test] fn weak_11() { assert_eq!(calculate_damage(11, 0, true, 1.0, false, false), 8); }
    #[test] fn weak_13() { assert_eq!(calculate_damage(13, 0, true, 1.0, false, false), 9); }
    #[test] fn weak_14() { assert_eq!(calculate_damage(14, 0, true, 1.0, false, false), 10); }
    #[test] fn weak_15() { assert_eq!(calculate_damage(15, 0, true, 1.0, false, false), 11); }
    #[test] fn weak_1() { assert_eq!(calculate_damage(1, 0, true, 1.0, false, false), 0); }
    #[test] fn weak_0_stays_0() { assert_eq!(calculate_damage(0, 0, true, 1.0, false, false), 0); }

    // ---- Vulnerable ----

    #[test] fn vuln_10() { assert_eq!(calculate_damage(10, 0, false, 1.0, true, false), 15); }
    #[test] fn vuln_7() { assert_eq!(calculate_damage(7, 0, false, 1.0, true, false), 10); }
    #[test] fn vuln_11() { assert_eq!(calculate_damage(11, 0, false, 1.0, true, false), 16); }
    #[test] fn vuln_1() { assert_eq!(calculate_damage(1, 0, false, 1.0, true, false), 1); }

    // ---- Stances ----

    #[test] fn wrath_6() { assert_eq!(calculate_damage(6, 0, false, WRATH_MULT, false, false), 12); }
    #[test] fn wrath_9() { assert_eq!(calculate_damage(9, 0, false, WRATH_MULT, false, false), 18); }
    #[test] fn divinity_6() { assert_eq!(calculate_damage(6, 0, false, DIVINITY_MULT, false, false), 18); }
    #[test] fn divinity_10() { assert_eq!(calculate_damage(10, 0, false, DIVINITY_MULT, false, false), 30); }

    // ---- Intangible ----

    #[test] fn intangible_100() { assert_eq!(calculate_damage(100, 0, false, 1.0, false, true), 1); }
    #[test] fn intangible_1_stays() { assert_eq!(calculate_damage(1, 0, false, 1.0, false, true), 1); }
    #[test] fn intangible_0_stays() { assert_eq!(calculate_damage(0, 0, false, 1.0, false, true), 0); }

    // ---- Compound rounding ----

    #[test] fn str_before_weak() {
        // (6+4)*0.75 = 7.5 -> 7
        assert_eq!(calculate_damage(6, 4, true, 1.0, false, false), 7);
    }

    #[test] fn str_weak_vuln() {
        // (6+2)*0.75*1.5 = 9.0 -> 9
        assert_eq!(calculate_damage(6, 2, true, 1.0, true, false), 9);
    }

    #[test] fn weak_wrath() {
        // 6*0.75*2.0 = 9.0
        assert_eq!(calculate_damage(6, 0, true, WRATH_MULT, false, false), 9);
    }

    #[test] fn weak_wrath_vuln() {
        // 7*0.75*2.0*1.5 = 15.75 -> 15
        assert_eq!(calculate_damage(7, 0, true, WRATH_MULT, true, false), 15);
    }

    #[test] fn str_wrath_vuln() {
        // (6+3)*2.0*1.5 = 27
        assert_eq!(calculate_damage(6, 3, false, WRATH_MULT, true, false), 27);
    }

    #[test] fn divinity_vuln() {
        // 10*3.0*1.5 = 45
        assert_eq!(calculate_damage(10, 0, false, DIVINITY_MULT, true, false), 45);
    }

    #[test] fn str_divinity_weak_vuln() {
        // (10+5)*0.75*3.0*1.5 = 50.625 -> 50
        assert_eq!(calculate_damage(10, 5, true, DIVINITY_MULT, true, false), 50);
    }

    #[test] fn wrath_intangible() {
        // 50*2.0=100 -> intangible cap 1
        assert_eq!(calculate_damage(50, 0, false, WRATH_MULT, false, true), 1);
    }

    // ---- Full calculate_damage_full ----

    #[test] fn full_pen_nib() {
        assert_eq!(calculate_damage_full(6, 0, 0, false, false, true, false, 1.0, false, false, false, false), 12);
    }

    #[test] fn full_vigor() {
        assert_eq!(calculate_damage_full(6, 0, 5, false, false, false, false, 1.0, false, false, false, false), 11);
    }

    #[test] fn full_str_vigor() {
        assert_eq!(calculate_damage_full(6, 3, 5, false, false, false, false, 1.0, false, false, false, false), 14);
    }

    #[test] fn full_flight() {
        assert_eq!(calculate_damage_full(10, 0, 0, false, false, false, false, 1.0, false, false, true, false), 5);
    }

    #[test] fn full_paper_frog_vuln() {
        // 10*1.75 = 17.5 -> 17
        assert_eq!(calculate_damage_full(10, 0, 0, false, false, false, false, 1.0, true, true, false, false), 17);
    }

    #[test] fn full_paper_crane_weak() {
        // 10*0.60 = 6
        assert_eq!(calculate_damage_full(10, 0, 0, true, true, false, false, 1.0, false, false, false, false), 6);
    }

    #[test] fn full_double_damage() {
        assert_eq!(calculate_damage_full(10, 0, 0, false, false, false, true, 1.0, false, false, false, false), 20);
    }

    #[test] fn full_pen_nib_wrath_vuln() {
        // 6*2(pen)*2.0(wrath)*1.5(vuln) = 36
        assert_eq!(calculate_damage_full(6, 0, 0, false, false, true, false, WRATH_MULT, true, false, false, false), 36);
    }

    // ---- Block ----

    #[test] fn block_basic() { assert_eq!(calculate_block(5, 0, false), 5); }
    #[test] fn block_dex() { assert_eq!(calculate_block(5, 2, false), 7); }
    #[test] fn block_frail() { assert_eq!(calculate_block(8, 0, true), 6); }
    #[test] fn block_dex_frail() { assert_eq!(calculate_block(5, 2, true), 5); }
    #[test] fn block_neg_dex() { assert_eq!(calculate_block(5, -2, false), 3); }
    #[test] fn block_neg_dex_floor() { assert_eq!(calculate_block(5, -10, false), 0); }
    #[test] fn block_frail_round_7() { assert_eq!(calculate_block(7, 0, true), 5); }
    #[test] fn block_frail_round_11() { assert_eq!(calculate_block(11, 0, true), 8); }
    #[test] fn block_zero() { assert_eq!(calculate_block(0, 0, false), 0); }
    #[test] fn block_all_negative() { assert_eq!(calculate_block(3, -5, true), 0); }

    // ---- Incoming damage ----

    #[test] fn incoming_basic() {
        let r = calculate_incoming_damage(10, 5, false, false, false, false, false, false);
        assert_eq!(r.hp_loss, 5);
        assert_eq!(r.block_remaining, 0);
    }
    #[test] fn incoming_full_block() {
        let r = calculate_incoming_damage(5, 10, false, false, false, false, false, false);
        assert_eq!(r.hp_loss, 0);
        assert_eq!(r.block_remaining, 5);
    }
    #[test] fn incoming_wrath() {
        let r = calculate_incoming_damage(10, 5, true, false, false, false, false, false);
        assert_eq!(r.hp_loss, 15);
    }
    #[test] fn incoming_vuln() {
        let r = calculate_incoming_damage(10, 0, false, true, false, false, false, false);
        assert_eq!(r.hp_loss, 15);
    }
    #[test] fn incoming_wrath_vuln() {
        // 10*2.0*1.5 = 30
        let r = calculate_incoming_damage(10, 0, true, true, false, false, false, false);
        assert_eq!(r.hp_loss, 30);
    }
    #[test] fn incoming_intangible() {
        let r = calculate_incoming_damage(100, 0, false, false, true, false, false, false);
        assert_eq!(r.hp_loss, 1);
    }
    #[test] fn incoming_torii_2() {
        let r = calculate_incoming_damage(2, 0, false, false, false, true, false, false);
        assert_eq!(r.hp_loss, 1);
    }
    #[test] fn incoming_torii_5() {
        let r = calculate_incoming_damage(5, 0, false, false, false, true, false, false);
        assert_eq!(r.hp_loss, 1);
    }
    #[test] fn incoming_torii_6_no_effect() {
        let r = calculate_incoming_damage(6, 0, false, false, false, true, false, false);
        assert_eq!(r.hp_loss, 6);
    }
    #[test] fn incoming_torii_1_no_effect() {
        let r = calculate_incoming_damage(1, 0, false, false, false, true, false, false);
        assert_eq!(r.hp_loss, 1);
    }
    #[test] fn incoming_tungsten() {
        let r = calculate_incoming_damage(10, 5, false, false, false, false, true, false);
        assert_eq!(r.hp_loss, 4);
    }
    #[test] fn incoming_tungsten_1hp_becomes_0() {
        let r = calculate_incoming_damage(1, 0, false, false, false, false, true, false);
        assert_eq!(r.hp_loss, 0);
    }
    #[test] fn incoming_intangible_tungsten() {
        // intangible caps to 1, tungsten -1 = 0
        let r = calculate_incoming_damage(100, 0, false, false, true, false, true, false);
        assert_eq!(r.hp_loss, 0);
    }

    // ---- HP loss ----

    #[test] fn hp_loss_basic() { assert_eq!(apply_hp_loss(5, false, false), 5); }
    #[test] fn hp_loss_intangible() { assert_eq!(apply_hp_loss(10, true, false), 1); }
    #[test] fn hp_loss_tungsten() { assert_eq!(apply_hp_loss(5, false, true), 4); }
    #[test] fn hp_loss_both() { assert_eq!(apply_hp_loss(10, true, true), 0); }
    #[test] fn hp_loss_intangible_1() { assert_eq!(apply_hp_loss(1, true, false), 1); }
}

// =============================================================================
// Enemy AI exhaustive tests
// =============================================================================

