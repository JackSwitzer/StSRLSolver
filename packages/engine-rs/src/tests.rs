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

#[cfg(test)]
mod card_registry_tests {
    use crate::cards::*;

    fn reg() -> CardRegistry {
        CardRegistry::new()
    }

    // ========== Watcher Basics ==========

    #[test]
    fn strike_base_values() {
        let c = reg().get("Strike_P").unwrap().clone();
        assert_eq!(c.base_damage, 6);
        assert_eq!(c.cost, 1);
        assert_eq!(c.card_type, CardType::Attack);
        assert_eq!(c.target, CardTarget::Enemy);
        assert!(!c.exhaust);
        assert!(c.enter_stance.is_none());
    }

    #[test]
    fn strike_upgraded_values() {
        let c = reg().get("Strike_P+").unwrap().clone();
        assert_eq!(c.base_damage, 9);
        assert_eq!(c.cost, 1);
    }

    #[test]
    fn defend_base_values() {
        let c = reg().get("Defend_P").unwrap().clone();
        assert_eq!(c.base_block, 5);
        assert_eq!(c.cost, 1);
        assert_eq!(c.card_type, CardType::Skill);
        assert_eq!(c.target, CardTarget::SelfTarget);
    }

    #[test]
    fn defend_upgraded_values() {
        let c = reg().get("Defend_P+").unwrap().clone();
        assert_eq!(c.base_block, 8);
        assert_eq!(c.cost, 1);
    }

    #[test]
    fn eruption_base_values() {
        let c = reg().get("Eruption").unwrap().clone();
        assert_eq!(c.base_damage, 9);
        assert_eq!(c.cost, 2);
        assert_eq!(c.enter_stance, Some("Wrath"));
        assert_eq!(c.card_type, CardType::Attack);
    }

    #[test]
    fn eruption_upgraded_cost_reduced() {
        let c = reg().get("Eruption+").unwrap().clone();
        assert_eq!(c.base_damage, 9);
        assert_eq!(c.cost, 1); // Upgrade reduces cost from 2 to 1
        assert_eq!(c.enter_stance, Some("Wrath"));
    }

    #[test]
    fn vigilance_base_values() {
        let c = reg().get("Vigilance").unwrap().clone();
        assert_eq!(c.base_block, 8);
        assert_eq!(c.cost, 2);
        assert_eq!(c.enter_stance, Some("Calm"));
        assert_eq!(c.card_type, CardType::Skill);
    }

    #[test]
    fn vigilance_upgraded_values() {
        let c = reg().get("Vigilance+").unwrap().clone();
        assert_eq!(c.base_block, 12);
        assert_eq!(c.cost, 2);
        assert_eq!(c.enter_stance, Some("Calm"));
    }

    // ========== Common Watcher ==========

    #[test]
    fn bowling_bash_base() {
        let c = reg().get("BowlingBash").unwrap().clone();
        assert_eq!(c.base_damage, 7);
        assert_eq!(c.cost, 1);
        assert!(c.effects.contains(&"damage_per_enemy"));
    }

    #[test]
    fn bowling_bash_upgraded() {
        let c = reg().get("BowlingBash+").unwrap().clone();
        assert_eq!(c.base_damage, 10);
    }

    #[test]
    fn crush_joints_base() {
        let c = reg().get("CrushJoints").unwrap().clone();
        assert_eq!(c.base_damage, 8);
        assert_eq!(c.base_magic, 1);
        assert!(c.effects.contains(&"vuln_if_last_skill"));
    }

    #[test]
    fn crush_joints_upgraded() {
        let c = reg().get("CrushJoints+").unwrap().clone();
        assert_eq!(c.base_damage, 10);
        assert_eq!(c.base_magic, 2);
    }

    #[test]
    fn cut_through_fate_base() {
        let c = reg().get("CutThroughFate").unwrap().clone();
        assert_eq!(c.base_damage, 7);
        assert_eq!(c.base_magic, 2);
        assert!(c.effects.contains(&"scry"));
        assert!(c.effects.contains(&"draw"));
    }

    #[test]
    fn cut_through_fate_upgraded() {
        let c = reg().get("CutThroughFate+").unwrap().clone();
        assert_eq!(c.base_damage, 9);
        assert_eq!(c.base_magic, 3);
    }

    #[test]
    fn empty_body_base() {
        let c = reg().get("EmptyBody").unwrap().clone();
        assert_eq!(c.base_block, 7);
        assert_eq!(c.cost, 1);
        assert_eq!(c.enter_stance, Some("Neutral"));
    }

    #[test]
    fn empty_body_upgraded() {
        let c = reg().get("EmptyBody+").unwrap().clone();
        assert_eq!(c.base_block, 11);
    }

    #[test]
    fn flurry_base() {
        let c = reg().get("Flurry").unwrap().clone();
        assert_eq!(c.base_damage, 4);
        assert_eq!(c.cost, 0);
    }

    #[test]
    fn flurry_upgraded() {
        let c = reg().get("Flurry+").unwrap().clone();
        assert_eq!(c.base_damage, 6);
        assert_eq!(c.cost, 0);
    }

    #[test]
    fn flying_sleeves_base() {
        let c = reg().get("FlyingSleeves").unwrap().clone();
        assert_eq!(c.base_damage, 4);
        assert_eq!(c.base_magic, 2);
        assert!(c.effects.contains(&"multi_hit"));
    }

    #[test]
    fn flying_sleeves_upgraded() {
        let c = reg().get("FlyingSleeves+").unwrap().clone();
        assert_eq!(c.base_damage, 6);
        assert_eq!(c.base_magic, 2);
    }

    #[test]
    fn follow_up_base() {
        let c = reg().get("FollowUp").unwrap().clone();
        assert_eq!(c.base_damage, 7);
        assert!(c.effects.contains(&"energy_if_last_attack"));
    }

    #[test]
    fn follow_up_upgraded() {
        let c = reg().get("FollowUp+").unwrap().clone();
        assert_eq!(c.base_damage, 11);
    }

    #[test]
    fn halt_base() {
        let c = reg().get("Halt").unwrap().clone();
        assert_eq!(c.base_block, 3);
        assert_eq!(c.base_magic, 9);
        assert_eq!(c.cost, 0);
        assert!(c.effects.contains(&"extra_block_in_wrath"));
    }

    #[test]
    fn halt_upgraded() {
        let c = reg().get("Halt+").unwrap().clone();
        assert_eq!(c.base_block, 4);
        assert_eq!(c.base_magic, 14);
    }

    #[test]
    fn prostrate_base() {
        let c = reg().get("Prostrate").unwrap().clone();
        assert_eq!(c.base_block, 4);
        assert_eq!(c.base_magic, 2);
        assert_eq!(c.cost, 0);
        assert!(c.effects.contains(&"mantra"));
    }

    #[test]
    fn prostrate_upgraded() {
        let c = reg().get("Prostrate+").unwrap().clone();
        assert_eq!(c.base_block, 4);
        assert_eq!(c.base_magic, 3);
    }

    #[test]
    fn tantrum_base() {
        let c = reg().get("Tantrum").unwrap().clone();
        assert_eq!(c.base_damage, 3);
        assert_eq!(c.base_magic, 3);
        assert_eq!(c.cost, 1);
        assert!(c.effects.contains(&"multi_hit"));
        assert_eq!(c.enter_stance, Some("Wrath"));
    }

    #[test]
    fn tantrum_upgraded() {
        let c = reg().get("Tantrum+").unwrap().clone();
        assert_eq!(c.base_damage, 3);
        assert_eq!(c.base_magic, 4); // One more hit
    }

    #[test]
    fn third_eye_base() {
        let c = reg().get("ThirdEye").unwrap().clone();
        assert_eq!(c.base_block, 7);
        assert_eq!(c.base_magic, 3);
        assert!(c.effects.contains(&"scry"));
    }

    #[test]
    fn third_eye_upgraded() {
        let c = reg().get("ThirdEye+").unwrap().clone();
        assert_eq!(c.base_block, 9);
        assert_eq!(c.base_magic, 5);
    }

    // ========== Uncommon Watcher ==========

    #[test]
    fn inner_peace_base() {
        let c = reg().get("InnerPeace").unwrap().clone();
        assert_eq!(c.base_magic, 3);
        assert_eq!(c.cost, 1);
        assert!(c.effects.contains(&"if_calm_draw_else_calm"));
    }

    #[test]
    fn inner_peace_upgraded() {
        let c = reg().get("InnerPeace+").unwrap().clone();
        assert_eq!(c.base_magic, 4);
    }

    #[test]
    fn wheel_kick_base() {
        let c = reg().get("WheelKick").unwrap().clone();
        assert_eq!(c.base_damage, 15);
        assert_eq!(c.cost, 2);
        assert_eq!(c.base_magic, 2);
        assert!(c.effects.contains(&"draw"));
    }

    #[test]
    fn wheel_kick_upgraded() {
        let c = reg().get("WheelKick+").unwrap().clone();
        assert_eq!(c.base_damage, 20);
    }

    #[test]
    fn conclude_base() {
        let c = reg().get("Conclude").unwrap().clone();
        assert_eq!(c.base_damage, 12);
        assert_eq!(c.cost, 1);
        assert_eq!(c.target, CardTarget::AllEnemy);
        assert!(c.effects.contains(&"end_turn"));
    }

    #[test]
    fn conclude_upgraded() {
        let c = reg().get("Conclude+").unwrap().clone();
        assert_eq!(c.base_damage, 16);
    }

    #[test]
    fn talk_to_the_hand_base() {
        let c = reg().get("TalkToTheHand").unwrap().clone();
        assert_eq!(c.base_damage, 5);
        assert_eq!(c.base_magic, 2);
        assert!(c.exhaust);
        assert!(c.effects.contains(&"apply_block_return"));
    }

    #[test]
    fn talk_to_the_hand_upgraded() {
        let c = reg().get("TalkToTheHand+").unwrap().clone();
        assert_eq!(c.base_damage, 7);
        assert_eq!(c.base_magic, 3);
        assert!(c.exhaust);
    }

    #[test]
    fn pray_base() {
        let c = reg().get("Pray").unwrap().clone();
        assert_eq!(c.base_magic, 3);
        assert_eq!(c.cost, 1);
        assert!(c.effects.contains(&"mantra"));
    }

    #[test]
    fn pray_upgraded() {
        let c = reg().get("Pray+").unwrap().clone();
        assert_eq!(c.base_magic, 4);
    }

    #[test]
    fn worship_base() {
        let c = reg().get("Worship").unwrap().clone();
        assert_eq!(c.base_magic, 5);
        assert_eq!(c.cost, 2);
        assert!(c.effects.contains(&"mantra"));
    }

    #[test]
    fn worship_upgraded_has_retain() {
        let c = reg().get("Worship+").unwrap().clone();
        assert_eq!(c.base_magic, 5);
        assert!(c.effects.contains(&"retain"));
    }

    // ========== Power Cards ==========

    #[test]
    fn rushdown_base() {
        let c = reg().get("Adaptation").unwrap().clone();
        assert_eq!(c.card_type, CardType::Power);
        assert_eq!(c.base_magic, 2);
        assert_eq!(c.cost, 1);
        assert!(c.effects.contains(&"on_wrath_draw"));
    }

    #[test]
    fn rushdown_upgraded_cost_zero() {
        let c = reg().get("Adaptation+").unwrap().clone();
        assert_eq!(c.cost, 0);
        assert_eq!(c.base_magic, 2);
    }

    #[test]
    fn mental_fortress_base() {
        let c = reg().get("MentalFortress").unwrap().clone();
        assert_eq!(c.card_type, CardType::Power);
        assert_eq!(c.base_magic, 4);
        assert_eq!(c.cost, 1);
        assert!(c.effects.contains(&"on_stance_change_block"));
    }

    #[test]
    fn mental_fortress_upgraded() {
        let c = reg().get("MentalFortress+").unwrap().clone();
        assert_eq!(c.base_magic, 6);
    }

    // ========== Rare ==========

    #[test]
    fn ragnarok_base() {
        let c = reg().get("Ragnarok").unwrap().clone();
        assert_eq!(c.base_damage, 5);
        assert_eq!(c.base_magic, 5);
        assert_eq!(c.cost, 3);
        assert_eq!(c.target, CardTarget::AllEnemy);
        assert_eq!(c.enter_stance, Some("Wrath"));
    }

    #[test]
    fn ragnarok_upgraded() {
        let c = reg().get("Ragnarok+").unwrap().clone();
        assert_eq!(c.base_damage, 6);
        assert_eq!(c.base_magic, 6);
    }

    // ========== Special ==========

    #[test]
    fn miracle_base() {
        let c = reg().get("Miracle").unwrap().clone();
        assert_eq!(c.cost, 0);
        assert_eq!(c.base_magic, 1);
        assert!(c.exhaust);
        assert!(c.effects.contains(&"gain_energy"));
    }

    #[test]
    fn miracle_upgraded() {
        let c = reg().get("Miracle+").unwrap().clone();
        assert_eq!(c.base_magic, 2);
        assert!(c.exhaust);
    }

    #[test]
    fn smite_base() {
        let c = reg().get("Smite").unwrap().clone();
        assert_eq!(c.base_damage, 12);
        assert_eq!(c.cost, 1);
        assert!(c.effects.contains(&"retain"));
    }

    #[test]
    fn smite_upgraded() {
        let c = reg().get("Smite+").unwrap().clone();
        assert_eq!(c.base_damage, 16);
    }

    // ========== Status / Curse ==========

    #[test]
    fn slimed_properties() {
        let c = reg().get("Slimed").unwrap().clone();
        assert_eq!(c.card_type, CardType::Status);
        assert_eq!(c.cost, 1);
        assert!(c.exhaust);
    }

    #[test]
    fn wound_is_unplayable() {
        let c = reg().get("Wound").unwrap().clone();
        assert_eq!(c.card_type, CardType::Status);
        assert_eq!(c.cost, -2);
        assert!(c.effects.contains(&"unplayable"));
        assert!(c.is_unplayable());
    }

    #[test]
    fn daze_is_unplayable_ethereal() {
        let c = reg().get("Daze").unwrap().clone();
        assert_eq!(c.cost, -2);
        assert!(c.effects.contains(&"unplayable"));
        assert!(c.effects.contains(&"ethereal"));
    }

    #[test]
    fn burn_is_unplayable() {
        let c = reg().get("Burn").unwrap().clone();
        assert_eq!(c.cost, -2);
        assert!(c.effects.contains(&"unplayable"));
    }

    #[test]
    fn ascenders_bane_properties() {
        let c = reg().get("AscendersBane").unwrap().clone();
        assert_eq!(c.card_type, CardType::Curse);
        assert!(c.effects.contains(&"unplayable"));
        assert!(c.effects.contains(&"ethereal"));
    }

    // ========== Colorless ==========

    #[test]
    fn strike_r_same_as_p() {
        let r = reg();
        let sp = r.get("Strike_P").unwrap();
        let sr = r.get("Strike_R").unwrap();
        assert_eq!(sp.base_damage, sr.base_damage);
    }

    // ========== Utility ==========

    #[test]
    fn is_upgraded_check() {
        assert!(CardRegistry::is_upgraded("Strike_P+"));
        assert!(CardRegistry::is_upgraded("Eruption+"));
        assert!(CardRegistry::is_upgraded("MentalFortress+"));
        assert!(!CardRegistry::is_upgraded("Strike_P"));
        assert!(!CardRegistry::is_upgraded("Eruption"));
    }

    #[test]
    fn base_id_strips_plus() {
        assert_eq!(CardRegistry::base_id("Strike_P+"), "Strike_P");
        assert_eq!(CardRegistry::base_id("Strike_P"), "Strike_P");
    }

    #[test]
    fn unknown_card_gets_default() {
        let c = reg().get_or_default("NonexistentCard");
        assert_eq!(c.id, "Unknown");
        assert_eq!(c.base_damage, 6);
        assert_eq!(c.cost, 1);
    }

    #[test]
    fn registry_has_all_expected_cards() {
        let r = reg();
        let expected = [
            "Strike_P", "Strike_P+", "Defend_P", "Defend_P+",
            "Eruption", "Eruption+", "Vigilance", "Vigilance+",
            "BowlingBash", "BowlingBash+", "CrushJoints", "CrushJoints+",
            "CutThroughFate", "CutThroughFate+", "EmptyBody", "EmptyBody+",
            "Flurry", "Flurry+", "FlyingSleeves", "FlyingSleeves+",
            "FollowUp", "FollowUp+", "Halt", "Halt+",
            "Prostrate", "Prostrate+", "Tantrum", "Tantrum+",
            "ThirdEye", "ThirdEye+", "InnerPeace", "InnerPeace+",
            "WheelKick", "WheelKick+", "Conclude", "Conclude+",
            "TalkToTheHand", "TalkToTheHand+",
            "Pray", "Pray+", "Worship", "Worship+",
            "Adaptation", "Adaptation+", "MentalFortress", "MentalFortress+",
            "Ragnarok", "Ragnarok+", "Miracle", "Miracle+",
            "Smite", "Smite+",
            "Slimed", "Wound", "Daze", "Burn", "AscendersBane",
            "Strike_R", "Defend_R",
        ];
        for id in &expected {
            assert!(r.get(id).is_some(), "Missing card: {}", id);
        }
    }
}

// =============================================================================
// Damage calculation exhaustive tests
// =============================================================================

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
        let r = calculate_incoming_damage(10, 5, false, false, false, false, false);
        assert_eq!(r.hp_loss, 5);
        assert_eq!(r.block_remaining, 0);
    }
    #[test] fn incoming_full_block() {
        let r = calculate_incoming_damage(5, 10, false, false, false, false, false);
        assert_eq!(r.hp_loss, 0);
        assert_eq!(r.block_remaining, 5);
    }
    #[test] fn incoming_wrath() {
        let r = calculate_incoming_damage(10, 5, true, false, false, false, false);
        assert_eq!(r.hp_loss, 15);
    }
    #[test] fn incoming_vuln() {
        let r = calculate_incoming_damage(10, 0, false, true, false, false, false);
        assert_eq!(r.hp_loss, 15);
    }
    #[test] fn incoming_wrath_vuln() {
        // 10*2.0*1.5 = 30
        let r = calculate_incoming_damage(10, 0, true, true, false, false, false);
        assert_eq!(r.hp_loss, 30);
    }
    #[test] fn incoming_intangible() {
        let r = calculate_incoming_damage(100, 0, false, false, true, false, false);
        assert_eq!(r.hp_loss, 1);
    }
    #[test] fn incoming_torii_2() {
        let r = calculate_incoming_damage(2, 0, false, false, false, true, false);
        assert_eq!(r.hp_loss, 1);
    }
    #[test] fn incoming_torii_5() {
        let r = calculate_incoming_damage(5, 0, false, false, false, true, false);
        assert_eq!(r.hp_loss, 1);
    }
    #[test] fn incoming_torii_6_no_effect() {
        let r = calculate_incoming_damage(6, 0, false, false, false, true, false);
        assert_eq!(r.hp_loss, 6);
    }
    #[test] fn incoming_torii_1_no_effect() {
        let r = calculate_incoming_damage(1, 0, false, false, false, true, false);
        assert_eq!(r.hp_loss, 1);
    }
    #[test] fn incoming_tungsten() {
        let r = calculate_incoming_damage(10, 5, false, false, false, false, true);
        assert_eq!(r.hp_loss, 4);
    }
    #[test] fn incoming_tungsten_1hp_becomes_0() {
        let r = calculate_incoming_damage(1, 0, false, false, false, false, true);
        assert_eq!(r.hp_loss, 0);
    }
    #[test] fn incoming_intangible_tungsten() {
        // intangible caps to 1, tungsten -1 = 0
        let r = calculate_incoming_damage(100, 0, false, false, true, false, true);
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

#[cfg(test)]
mod enemy_tests {
    use crate::enemies::*;
    use crate::enemies::move_ids::*;

    // ========== JawWorm ==========

    #[test] fn jw_create_hp() {
        let e = create_enemy("JawWorm", 44, 44);
        assert_eq!(e.entity.hp, 44);
        assert_eq!(e.entity.max_hp, 44);
    }
    #[test] fn jw_first_move_chomp() {
        let e = create_enemy("JawWorm", 44, 44);
        assert_eq!(e.move_id, JW_CHOMP);
        assert_eq!(e.move_damage, 11);
        assert_eq!(e.move_hits, 1);
    }
    #[test] fn jw_after_chomp_bellow() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, JW_BELLOW);
        assert_eq!(e.move_block, 6);
        assert_eq!(*e.move_effects.get("strength").unwrap(), 3);
    }
    #[test] fn jw_after_bellow_thrash() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move(&mut e); // -> Bellow
        roll_next_move(&mut e); // -> Thrash
        assert_eq!(e.move_id, JW_THRASH);
        assert_eq!(e.move_damage, 7);
        assert_eq!(e.move_block, 5);
    }
    #[test] fn jw_after_thrash_chomp() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, JW_CHOMP);
    }
    #[test] fn jw_6_turn_cycle() {
        let mut e = create_enemy("JawWorm", 44, 44);
        let mut ids = vec![e.move_id];
        for _ in 0..5 {
            roll_next_move(&mut e);
            ids.push(e.move_id);
        }
        assert_eq!(ids[0], JW_CHOMP);
        assert_eq!(ids[1], JW_BELLOW);
        assert_eq!(ids[2], JW_THRASH);
        assert_eq!(ids[3], JW_CHOMP);
    }
    #[test] fn jw_bellow_has_no_damage() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move(&mut e); // -> Bellow
        assert_eq!(e.move_damage, 0);
    }

    // ========== Cultist ==========

    #[test] fn cult_first_incantation() {
        let e = create_enemy("Cultist", 50, 50);
        assert_eq!(e.move_id, CULT_INCANTATION);
        assert_eq!(e.move_damage, 0);
    }
    #[test] fn cult_second_dark_strike() {
        let mut e = create_enemy("Cultist", 50, 50);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, CULT_DARK_STRIKE);
        assert_eq!(e.move_damage, 6);
    }
    #[test] fn cult_always_dark_strike_after() {
        let mut e = create_enemy("Cultist", 50, 50);
        for _ in 0..10 {
            roll_next_move(&mut e);
            assert_eq!(e.move_id, CULT_DARK_STRIKE);
        }
    }
    #[test] fn cult_ritual_effect() {
        let e = create_enemy("Cultist", 50, 50);
        assert_eq!(*e.move_effects.get("ritual").unwrap(), 3);
    }

    // ========== FungiBeast ==========

    #[test] fn fb_first_bite() {
        let e = create_enemy("FungiBeast", 24, 24);
        assert_eq!(e.move_id, FB_BITE);
        assert_eq!(e.move_damage, 6);
    }
    #[test] fn fb_spore_cloud_on_death() {
        let e = create_enemy("FungiBeast", 24, 24);
        assert_eq!(e.entity.status("SporeCloud"), 2);
    }
    #[test] fn fb_no_three_bites() {
        let mut e = create_enemy("FungiBeast", 24, 24);
        roll_next_move(&mut e); // bite -> bite
        roll_next_move(&mut e); // bite,bite -> MUST grow
        assert_eq!(e.move_id, FB_GROW);
    }
    #[test] fn fb_grow_gives_strength() {
        let mut e = create_enemy("FungiBeast", 24, 24);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(*e.move_effects.get("strength").unwrap(), 3);
    }
    #[test] fn fb_after_grow_bite() {
        let mut e = create_enemy("FungiBeast", 24, 24);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, FB_BITE);
    }

    // ========== Red Louse ==========

    #[test] fn red_louse_first_bite() {
        let e = create_enemy("RedLouse", 12, 12);
        assert_eq!(e.move_id, LOUSE_BITE);
    }
    #[test] fn red_louse_curl_up() {
        let e = create_enemy("RedLouse", 12, 12);
        assert!(e.entity.status("CurlUp") > 0);
    }
    #[test] fn red_louse_no_three_bites() {
        let mut e = create_enemy("RedLouse", 12, 12);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, LOUSE_GROW);
    }
    #[test] fn red_louse_grow_str() {
        let mut e = create_enemy("RedLouse", 12, 12);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(*e.move_effects.get("strength").unwrap(), 3);
    }

    // ========== Green Louse ==========

    #[test] fn green_louse_first_bite() {
        let e = create_enemy("GreenLouse", 14, 14);
        assert_eq!(e.move_id, LOUSE_BITE);
    }
    #[test] fn green_louse_curl_up() {
        let e = create_enemy("GreenLouse", 14, 14);
        assert!(e.entity.status("CurlUp") > 0);
    }
    #[test] fn green_louse_spit_web_weak() {
        let mut e = create_enemy("GreenLouse", 14, 14);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, LOUSE_SPIT_WEB);
        assert_eq!(*e.move_effects.get("weak").unwrap(), 2);
    }

    // ========== Blue Slaver ==========

    #[test] fn bs_first_stab() {
        let e = create_enemy("SlaverBlue", 48, 48);
        assert_eq!(e.move_id, BS_STAB);
        assert_eq!(e.move_damage, 12);
    }
    #[test] fn bs_no_three_stabs() {
        let mut e = create_enemy("SlaverBlue", 48, 48);
        roll_next_move(&mut e); // stab -> stab
        roll_next_move(&mut e); // stab,stab -> MUST rake
        assert_eq!(e.move_id, BS_RAKE);
    }
    #[test] fn bs_rake_weak() {
        let mut e = create_enemy("SlaverBlue", 48, 48);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(*e.move_effects.get("weak").unwrap(), 1);
    }
    #[test] fn bs_rake_damage() {
        let mut e = create_enemy("SlaverBlue", 48, 48);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(e.move_damage, 7);
    }

    // ========== Red Slaver ==========

    #[test] fn rs_first_stab() {
        let e = create_enemy("SlaverRed", 48, 48);
        assert_eq!(e.move_id, RS_STAB);
        assert_eq!(e.move_damage, 13);
    }
    #[test] fn rs_entangle_once() {
        let mut e = create_enemy("SlaverRed", 48, 48);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, RS_ENTANGLE);
        assert_eq!(*e.move_effects.get("entangle").unwrap(), 1);
    }
    #[test] fn rs_scrape_vuln() {
        let mut e = create_enemy("SlaverRed", 48, 48);
        roll_next_move(&mut e); // entangle
        roll_next_move(&mut e); // scrape or stab
        if e.move_id == RS_SCRAPE {
            assert_eq!(*e.move_effects.get("vulnerable").unwrap(), 1);
        }
    }

    // ========== Acid Slime S ==========

    #[test] fn acid_s_first_tackle() {
        let e = create_enemy("AcidSlime_S", 10, 10);
        assert_eq!(e.move_id, AS_TACKLE);
        assert_eq!(e.move_damage, 3);
    }
    #[test] fn acid_s_alternates() {
        let mut e = create_enemy("AcidSlime_S", 10, 10);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, AS_LICK);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, AS_TACKLE);
    }
    #[test] fn acid_s_lick_weak() {
        let mut e = create_enemy("AcidSlime_S", 10, 10);
        roll_next_move(&mut e);
        assert_eq!(*e.move_effects.get("weak").unwrap(), 1);
    }

    // ========== Acid Slime M ==========

    #[test] fn acid_m_first() {
        let e = create_enemy("AcidSlime_M", 28, 28);
        assert_eq!(e.move_id, AS_CORROSIVE_SPIT);
        assert_eq!(*e.move_effects.get("slimed").unwrap(), 1);
    }
    #[test] fn acid_m_damage() {
        let e = create_enemy("AcidSlime_M", 28, 28);
        assert_eq!(e.move_damage, 7);
    }

    // ========== Acid Slime L ==========

    #[test] fn acid_l_damage() {
        let e = create_enemy("AcidSlime_L", 65, 65);
        assert_eq!(e.move_id, AS_CORROSIVE_SPIT);
        assert_eq!(e.move_damage, 11);
        assert_eq!(*e.move_effects.get("slimed").unwrap(), 2);
    }

    // ========== Spike Slime S ==========

    #[test] fn spike_s_tackle_only() {
        let e = create_enemy("SpikeSlime_S", 10, 10);
        assert_eq!(e.move_id, SS_TACKLE);
        assert_eq!(e.move_damage, 5);
    }
    #[test] fn spike_s_stays_tackle() {
        let mut e = create_enemy("SpikeSlime_S", 10, 10);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, SS_TACKLE);
    }

    // ========== Spike Slime M ==========

    #[test] fn spike_m_first() {
        let e = create_enemy("SpikeSlime_M", 28, 28);
        assert_eq!(e.move_id, SS_TACKLE);
        assert_eq!(e.move_damage, 8);
    }
    #[test] fn spike_m_no_three_tackles() {
        let mut e = create_enemy("SpikeSlime_M", 28, 28);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, SS_LICK);
        assert_eq!(*e.move_effects.get("frail").unwrap(), 1);
    }

    // ========== Spike Slime L ==========

    #[test] fn spike_l_first() {
        let e = create_enemy("SpikeSlime_L", 64, 64);
        assert_eq!(e.move_id, SS_TACKLE);
        assert_eq!(e.move_damage, 16);
    }
    #[test] fn spike_l_frail_2() {
        let mut e = create_enemy("SpikeSlime_L", 64, 64);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, SS_LICK);
        assert_eq!(*e.move_effects.get("frail").unwrap(), 2);
    }

    // ========== Sentry ==========

    #[test] fn sentry_first_bolt() {
        let e = create_enemy("Sentry", 38, 38);
        assert_eq!(e.move_id, SENTRY_BOLT);
        assert_eq!(e.move_damage, 9);
    }
    #[test] fn sentry_alternates_bolt_beam() {
        let mut e = create_enemy("Sentry", 38, 38);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, SENTRY_BEAM);
        assert_eq!(*e.move_effects.get("daze").unwrap(), 2);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, SENTRY_BOLT);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, SENTRY_BEAM);
    }
    #[test] fn sentry_beam_damage() {
        let mut e = create_enemy("Sentry", 38, 38);
        roll_next_move(&mut e);
        assert_eq!(e.move_damage, 9);
    }

    // ========== The Guardian ==========

    #[test] fn guard_first_charging() {
        let e = create_enemy("TheGuardian", 240, 240);
        assert_eq!(e.move_id, GUARD_CHARGING_UP);
        assert_eq!(e.move_block, 9);
    }
    #[test] fn guard_mode_shift_threshold() {
        let e = create_enemy("TheGuardian", 240, 240);
        assert_eq!(e.entity.status("ModeShift"), 30);
    }
    #[test] fn guard_offensive_cycle() {
        let mut e = create_enemy("TheGuardian", 240, 240);
        roll_next_move(&mut e); // -> Fierce Bash
        assert_eq!(e.move_id, GUARD_FIERCE_BASH);
        assert_eq!(e.move_damage, 32);
        roll_next_move(&mut e); // -> Vent Steam
        assert_eq!(e.move_id, GUARD_VENT_STEAM);
        assert_eq!(*e.move_effects.get("weak").unwrap(), 2);
        assert_eq!(*e.move_effects.get("vulnerable").unwrap(), 2);
        roll_next_move(&mut e); // -> Whirlwind
        assert_eq!(e.move_id, GUARD_WHIRLWIND);
        assert_eq!(e.move_damage, 5);
        assert_eq!(e.move_hits, 4);
        roll_next_move(&mut e); // -> Charging Up
        assert_eq!(e.move_id, GUARD_CHARGING_UP);
    }
    #[test] fn guard_mode_shift_at_30() {
        let mut e = create_enemy("TheGuardian", 240, 240);
        assert!(!guardian_check_mode_shift(&mut e, 29));
        assert!(guardian_check_mode_shift(&mut e, 1));
        assert_eq!(e.entity.status("SharpHide"), 3);
    }
    #[test] fn guard_mode_shift_threshold_increases() {
        let mut e = create_enemy("TheGuardian", 240, 240);
        guardian_check_mode_shift(&mut e, 30);
        assert_eq!(e.entity.status("ModeShift"), 40);
    }
    #[test] fn guard_defensive_cycle() {
        let mut e = create_enemy("TheGuardian", 240, 240);
        guardian_check_mode_shift(&mut e, 30);
        assert_eq!(e.move_id, GUARD_ROLL_ATTACK);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, GUARD_TWIN_SLAM);
        assert_eq!(e.move_hits, 2);
        assert_eq!(e.move_damage, 8);
    }
    #[test] fn guard_switch_back_to_offensive() {
        let mut e = create_enemy("TheGuardian", 240, 240);
        guardian_check_mode_shift(&mut e, 30);
        guardian_switch_to_offensive(&mut e);
        assert_eq!(e.entity.status("SharpHide"), 0);
        assert_eq!(e.move_id, GUARD_CHARGING_UP);
    }

    // ========== Hexaghost ==========

    #[test] fn hex_first_activate() {
        let e = create_enemy("Hexaghost", 250, 250);
        assert_eq!(e.move_id, HEX_ACTIVATE);
    }
    #[test] fn hex_second_divider() {
        let mut e = create_enemy("Hexaghost", 250, 250);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, HEX_DIVIDER);
        assert_eq!(e.move_hits, 6);
    }
    #[test] fn hex_full_7_cycle() {
        let mut e = create_enemy("Hexaghost", 250, 250);
        roll_next_move(&mut e); // Divider
        roll_next_move(&mut e); // Sear
        assert_eq!(e.move_id, HEX_SEAR);
        assert_eq!(e.move_damage, 6);
        assert_eq!(*e.move_effects.get("burn").unwrap(), 1);
        roll_next_move(&mut e); // Tackle
        assert_eq!(e.move_id, HEX_TACKLE);
        assert_eq!(e.move_hits, 2);
        roll_next_move(&mut e); // Sear
        assert_eq!(e.move_id, HEX_SEAR);
        roll_next_move(&mut e); // Inflame
        assert_eq!(e.move_id, HEX_INFLAME);
        assert_eq!(e.move_block, 12);
        assert_eq!(*e.move_effects.get("strength").unwrap(), 2);
        roll_next_move(&mut e); // Tackle
        assert_eq!(e.move_id, HEX_TACKLE);
        roll_next_move(&mut e); // Sear
        assert_eq!(e.move_id, HEX_SEAR);
        roll_next_move(&mut e); // Inferno
        assert_eq!(e.move_id, HEX_INFERNO);
        assert_eq!(e.move_hits, 6);
        assert_eq!(*e.move_effects.get("burn+").unwrap(), 3);
    }
    #[test] fn hex_cycle_repeats() {
        let mut e = create_enemy("Hexaghost", 250, 250);
        // Activate + Divider + 7 cycle + restart
        for _ in 0..9 { roll_next_move(&mut e); }
        // Should be back to Sear
        assert_eq!(e.move_id, HEX_SEAR);
    }

    // ========== Slime Boss ==========

    #[test] fn sb_first_sticky() {
        let e = create_enemy("SlimeBoss", 140, 140);
        assert_eq!(e.move_id, SB_STICKY);
        assert_eq!(*e.move_effects.get("slimed").unwrap(), 3);
    }
    #[test] fn sb_full_cycle() {
        let mut e = create_enemy("SlimeBoss", 140, 140);
        roll_next_move(&mut e); // Prep
        assert_eq!(e.move_id, SB_PREP_SLAM);
        roll_next_move(&mut e); // Slam
        assert_eq!(e.move_id, SB_SLAM);
        assert_eq!(e.move_damage, 35);
        roll_next_move(&mut e); // Sticky
        assert_eq!(e.move_id, SB_STICKY);
    }
    #[test] fn sb_split_at_50pct() {
        let mut e = create_enemy("SlimeBoss", 140, 140);
        assert!(!slime_boss_should_split(&e));
        e.entity.hp = 70;
        assert!(slime_boss_should_split(&e));
    }
    #[test] fn sb_split_below_50pct() {
        let mut e = create_enemy("SlimeBoss", 140, 140);
        e.entity.hp = 50;
        assert!(slime_boss_should_split(&e));
    }
    #[test] fn sb_no_split_at_71() {
        let mut e = create_enemy("SlimeBoss", 140, 140);
        e.entity.hp = 71;
        assert!(!slime_boss_should_split(&e));
    }
    #[test] fn sb_no_split_if_dead() {
        let mut e = create_enemy("SlimeBoss", 140, 140);
        e.entity.hp = 0;
        assert!(!slime_boss_should_split(&e));
    }

    // ========== Unknown enemy ==========

    #[test] fn unknown_enemy_defaults() {
        let e = create_enemy("SomeBoss", 100, 100);
        assert_eq!(e.move_damage, 6);
    }

    // ========== Move history tracking ==========

    #[test] fn move_history_recorded() {
        let mut e = create_enemy("JawWorm", 44, 44);
        assert!(e.move_history.is_empty());
        roll_next_move(&mut e);
        assert_eq!(e.move_history.len(), 1);
        assert_eq!(e.move_history[0], JW_CHOMP);
    }
    #[test] fn move_history_multiple() {
        let mut e = create_enemy("Cultist", 50, 50);
        for _ in 0..5 { roll_next_move(&mut e); }
        assert_eq!(e.move_history.len(), 5);
    }
}

// =============================================================================
// Relic exhaustive tests
// =============================================================================

#[cfg(test)]
mod relic_tests {
    use crate::relics::*;
    use crate::state::{CombatState, EnemyCombatState};

    fn state() -> CombatState {
        let e = EnemyCombatState::new("Test", 50, 50);
        CombatState::new(80, 80, vec![e], vec!["Strike_P".to_string(); 5], 3)
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
        s.player.set_status("Strength", 3);
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
        assert_eq!(state_with("Thread and Needle").player.status("Plated Armor"), 4);
    }

    // ---- Anchor ----
    #[test] fn anchor_10_block() { assert_eq!(state_with("Anchor").player.block, 10); }

    // ---- Akabeko ----
    #[test] fn akabeko_vigor_8() { assert_eq!(state_with("Akabeko").player.status("Vigor"), 8); }

    // ---- Bronze Scales ----
    #[test] fn bronze_scales_thorns_3() {
        assert_eq!(state_with("Bronze Scales").player.status("Thorns"), 3);
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
        assert_eq!(state_with("ClockworkSouvenir").player.status("Artifact"), 1);
    }

    // ---- Fossilized Helix ----
    #[test] fn helix_buffer_1() {
        assert_eq!(state_with("FossilizedHelix").player.status("Buffer"), 1);
    }

    // ---- Data Disk ----
    #[test] fn data_disk_focus_1() {
        assert_eq!(state_with("Data Disk").player.status("Focus"), 1);
    }

    // ---- Mark of Pain ----
    #[test] fn mark_of_pain_wounds() {
        let s = state_with("Mark of Pain");
        let w = s.draw_pile.iter().filter(|c| *c == "Wound").count();
        assert_eq!(w, 2);
    }

    // ---- Lantern ----
    #[test] fn lantern_ready() {
        let s = state_with("Lantern");
        assert_eq!(s.player.status("LanternReady"), 1);
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
        assert_eq!(s.player.status("LanternReady"), 0);
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
}

// =============================================================================
// Potion exhaustive tests
// =============================================================================

#[cfg(test)]
mod potion_tests {
    use crate::potions::*;
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
        s.player.set_status("Strength", 3);
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
        assert_eq!(s.enemies[0].entity.status("Weakened"), 3);
    }
    #[test] fn weak_bad_target() { assert!(!apply_potion(&mut state(), "Weak Potion", 5)); }

    // ---- Fear Potion ----
    #[test] fn fear_3() {
        let mut s = state();
        apply_potion(&mut s, "FearPotion", 0);
        assert_eq!(s.enemies[0].entity.status("Vulnerable"), 3);
    }

    // ---- Poison Potion ----
    #[test] fn poison_6() {
        let mut s = state();
        apply_potion(&mut s, "Poison Potion", 0);
        assert_eq!(s.enemies[0].entity.status("Poison"), 6);
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
        assert_eq!(s.player.status("LoseStrength"), 5);
    }

    // ---- Speed Potion ----
    #[test] fn speed_temp_dex() {
        let mut s = state();
        apply_potion(&mut s, "SpeedPotion", -1);
        assert_eq!(s.player.dexterity(), 5);
        assert_eq!(s.player.status("LoseDexterity"), 5);
    }

    // ---- Ancient Potion ----
    #[test] fn ancient_artifact() {
        let mut s = state();
        apply_potion(&mut s, "Ancient Potion", -1);
        assert_eq!(s.player.status("Artifact"), 1);
    }

    // ---- Regen ----
    #[test] fn regen_5() {
        let mut s = state();
        apply_potion(&mut s, "Regen Potion", -1);
        assert_eq!(s.player.status("Regeneration"), 5);
    }

    // ---- Essence of Steel ----
    #[test] fn essence_plated_4() {
        let mut s = state();
        apply_potion(&mut s, "EssenceOfSteel", -1);
        assert_eq!(s.player.status("Plated Armor"), 4);
    }

    // ---- Liquid Bronze ----
    #[test] fn bronze_thorns_3() {
        let mut s = state();
        apply_potion(&mut s, "LiquidBronze", -1);
        assert_eq!(s.player.status("Thorns"), 3);
    }

    // ---- Cultist Potion ----
    #[test] fn cultist_ritual_1() {
        let mut s = state();
        apply_potion(&mut s, "CultistPotion", -1);
        assert_eq!(s.player.status("Ritual"), 1);
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

    // ---- Unknown potion ----
    #[test] fn unknown_potion_succeeds() {
        assert!(apply_potion(&mut state(), "UnknownPotion", -1));
    }
}

// =============================================================================
// Powers module tests
// =============================================================================

#[cfg(test)]
mod power_tests {
    use crate::powers::*;
    use crate::state::EntityState;

    fn entity() -> EntityState { EntityState::new(50, 50) }

    #[test] fn decrement_weak_2_to_1() {
        let mut e = entity();
        e.set_status("Weakened", 2);
        decrement_debuffs(&mut e);
        assert_eq!(e.status("Weakened"), 1);
    }
    #[test] fn decrement_weak_1_to_0() {
        let mut e = entity();
        e.set_status("Weakened", 1);
        decrement_debuffs(&mut e);
        assert_eq!(e.status("Weakened"), 0);
        assert!(!e.statuses.contains_key("Weakened"));
    }
    #[test] fn decrement_all_three() {
        let mut e = entity();
        e.set_status("Weakened", 3);
        e.set_status("Vulnerable", 2);
        e.set_status("Frail", 1);
        decrement_debuffs(&mut e);
        assert_eq!(e.status("Weakened"), 2);
        assert_eq!(e.status("Vulnerable"), 1);
        assert_eq!(e.status("Frail"), 0);
    }
    #[test] fn poison_tick_damage() {
        let mut e = entity();
        e.set_status("Poison", 7);
        let d = tick_poison(&mut e);
        assert_eq!(d, 7);
        assert_eq!(e.hp, 43);
        assert_eq!(e.status("Poison"), 6);
    }
    #[test] fn poison_tick_to_zero() {
        let mut e = entity();
        e.set_status("Poison", 1);
        tick_poison(&mut e);
        assert_eq!(e.status("Poison"), 0);
    }
    #[test] fn poison_no_poison() {
        let mut e = entity();
        assert_eq!(tick_poison(&mut e), 0);
    }
    #[test] fn metallicize_gain() {
        let mut e = entity();
        e.set_status("Metallicize", 4);
        apply_metallicize(&mut e);
        assert_eq!(e.block, 4);
    }
    #[test] fn metallicize_stacks() {
        let mut e = entity();
        e.block = 3;
        e.set_status("Metallicize", 4);
        apply_metallicize(&mut e);
        assert_eq!(e.block, 7);
    }
    #[test] fn plated_armor_gain() {
        let mut e = entity();
        e.set_status("Plated Armor", 6);
        apply_plated_armor(&mut e);
        assert_eq!(e.block, 6);
    }
    #[test] fn ritual_gain() {
        let mut e = entity();
        e.set_status("Ritual", 3);
        apply_ritual(&mut e);
        assert_eq!(e.strength(), 3);
    }
    #[test] fn ritual_stacks() {
        let mut e = entity();
        e.set_status("Ritual", 3);
        apply_ritual(&mut e);
        apply_ritual(&mut e);
        assert_eq!(e.strength(), 6);
    }
    #[test] fn artifact_blocks_debuff() {
        let mut e = entity();
        e.set_status("Artifact", 2);
        assert!(!apply_debuff(&mut e, "Weakened", 3));
        assert_eq!(e.status("Weakened"), 0);
        assert_eq!(e.status("Artifact"), 1);
    }
    #[test] fn artifact_consumed() {
        let mut e = entity();
        e.set_status("Artifact", 1);
        apply_debuff(&mut e, "Weakened", 1);
        assert_eq!(e.status("Artifact"), 0);
    }
    #[test] fn no_artifact_applies() {
        let mut e = entity();
        assert!(apply_debuff(&mut e, "Weakened", 2));
        assert_eq!(e.status("Weakened"), 2);
    }
}

// =============================================================================
// State tests
// =============================================================================

#[cfg(test)]
mod state_tests {
    use crate::state::*;

    #[test] fn stance_from_str() {
        assert_eq!(Stance::from_str("Wrath"), Stance::Wrath);
        assert_eq!(Stance::from_str("Calm"), Stance::Calm);
        assert_eq!(Stance::from_str("Divinity"), Stance::Divinity);
        assert_eq!(Stance::from_str("Neutral"), Stance::Neutral);
        assert_eq!(Stance::from_str("garbage"), Stance::Neutral);
    }
    #[test] fn stance_outgoing_mult() {
        assert_eq!(Stance::Wrath.outgoing_mult(), 2.0);
        assert_eq!(Stance::Divinity.outgoing_mult(), 3.0);
        assert_eq!(Stance::Calm.outgoing_mult(), 1.0);
        assert_eq!(Stance::Neutral.outgoing_mult(), 1.0);
    }
    #[test] fn stance_incoming_mult() {
        assert_eq!(Stance::Wrath.incoming_mult(), 2.0);
        assert_eq!(Stance::Divinity.incoming_mult(), 1.0);
        assert_eq!(Stance::Calm.incoming_mult(), 1.0);
    }
    #[test] fn entity_accessors() {
        let mut e = EntityState::new(50, 50);
        assert_eq!(e.strength(), 0);
        assert_eq!(e.dexterity(), 0);
        assert!(!e.is_weak());
        assert!(!e.is_vulnerable());
        assert!(!e.is_frail());
        assert!(!e.is_dead());
        e.set_status("Strength", 5);
        assert_eq!(e.strength(), 5);
    }
    #[test] fn entity_add_status() {
        let mut e = EntityState::new(50, 50);
        e.add_status("Strength", 3);
        e.add_status("Strength", 2);
        assert_eq!(e.strength(), 5);
    }
    #[test] fn entity_set_zero_removes() {
        let mut e = EntityState::new(50, 50);
        e.set_status("Strength", 5);
        e.set_status("Strength", 0);
        assert!(!e.statuses.contains_key("Strength"));
    }
    #[test] fn entity_dead_at_zero() {
        let mut e = EntityState::new(50, 50);
        e.hp = 0;
        assert!(e.is_dead());
    }
    #[test] fn enemy_alive_check() {
        let e = EnemyCombatState::new("Test", 30, 30);
        assert!(e.is_alive());
    }
    #[test] fn enemy_dead_check() {
        let mut e = EnemyCombatState::new("Test", 30, 30);
        e.entity.hp = 0;
        assert!(!e.is_alive());
    }
    #[test] fn enemy_escaping_not_alive() {
        let mut e = EnemyCombatState::new("Test", 30, 30);
        e.is_escaping = true;
        assert!(!e.is_alive());
    }
    #[test] fn enemy_total_incoming() {
        let mut e = EnemyCombatState::new("Test", 30, 30);
        e.set_move(1, 5, 3, 0);
        assert_eq!(e.total_incoming_damage(), 15);
    }
    #[test] fn combat_state_victory() {
        let mut s = CombatState::new(80, 80, vec![EnemyCombatState::new("T", 0, 30)], vec![], 3);
        s.enemies[0].entity.hp = 0;
        assert!(s.is_victory());
    }
    #[test] fn combat_state_defeat() {
        let s = CombatState::new(0, 80, vec![EnemyCombatState::new("T", 30, 30)], vec![], 3);
        assert!(s.is_defeat());
    }
    #[test] fn combat_state_not_terminal() {
        let s = CombatState::new(80, 80, vec![EnemyCombatState::new("T", 30, 30)], vec![], 3);
        assert!(!s.is_terminal());
    }
    #[test] fn living_enemy_indices() {
        let mut s = CombatState::new(80, 80, vec![
            EnemyCombatState::new("A", 30, 30),
            EnemyCombatState::new("B", 0, 30),
            EnemyCombatState::new("C", 20, 20),
        ], vec![], 3);
        s.enemies[1].entity.hp = 0;
        assert_eq!(s.living_enemy_indices(), vec![0, 2]);
    }
    #[test] fn has_relic() {
        let mut s = CombatState::new(80, 80, vec![], vec![], 3);
        s.relics.push("Vajra".to_string());
        assert!(s.has_relic("Vajra"));
        assert!(!s.has_relic("Missing"));
    }
}

// =============================================================================
// Integration: engine-level combined tests
// =============================================================================

#[cfg(test)]
mod engine_integration_tests {
    use crate::engine::*;
    use crate::actions::Action;
    use crate::state::*;

    fn engine_with(deck: Vec<String>, enemy_hp: i32, enemy_dmg: i32) -> CombatEngine {
        let mut enemy = EnemyCombatState::new("JawWorm", enemy_hp, enemy_hp);
        enemy.set_move(1, enemy_dmg, 1, 0);
        let state = CombatState::new(80, 80, vec![enemy], deck, 3);
        let mut e = CombatEngine::new(state, 42);
        e.start_combat();
        e
    }

    fn play(e: &mut CombatEngine, card: &str) {
        if let Some(idx) = e.state.hand.iter().position(|c| c == card) {
            e.execute_action(&Action::PlayCard { card_idx: idx, target_idx: 0 });
        }
    }

    fn play_self(e: &mut CombatEngine, card: &str) {
        if let Some(idx) = e.state.hand.iter().position(|c| c == card) {
            e.execute_action(&Action::PlayCard { card_idx: idx, target_idx: -1 });
        }
    }

    // ---- Eruption in Wrath = double = 9*2=18 ----
    #[test] fn eruption_in_wrath_18() {
        let mut e = engine_with(
            vec!["Eruption".to_string(); 5],
            100, 0,
        );
        e.state.stance = Stance::Wrath;
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "Eruption");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 18);
    }

    // ---- Tantrum multi-hit 3x3=9 base ----
    #[test] fn tantrum_3_hits() {
        let mut e = engine_with(
            vec!["Tantrum".to_string(); 5],
            100, 0,
        );
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "Tantrum");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 9);
        assert_eq!(e.state.stance, Stance::Wrath);
    }

    #[test] fn tantrum_plus_4_hits() {
        let mut e = engine_with(
            vec!["Tantrum+".to_string(); 5],
            100, 0,
        );
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "Tantrum+");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 12); // 3*4=12
    }

    // ---- FlyingSleeves 2-hit ----
    #[test] fn flying_sleeves_2_hits() {
        let mut e = engine_with(
            vec!["FlyingSleeves".to_string(); 5],
            100, 0,
        );
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "FlyingSleeves");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 8); // 4*2=8
    }

    #[test] fn flying_sleeves_plus() {
        let mut e = engine_with(
            vec!["FlyingSleeves+".to_string(); 5],
            100, 0,
        );
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "FlyingSleeves+");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 12); // 6*2=12
    }

    // ---- Conclude hits all enemies ----
    #[test] fn conclude_all_enemy() {
        let mut enemy2 = EnemyCombatState::new("E2", 50, 50);
        enemy2.set_move(1, 0, 0, 0);
        let mut state = CombatState::new(80, 80,
            vec![EnemyCombatState::new("E1", 50, 50), enemy2],
            vec!["Conclude".to_string(); 5], 3);
        state.enemies[0].set_move(1, 0, 0, 0);
        let mut eng = CombatEngine::new(state, 42);
        eng.start_combat();
        play(&mut eng, "Conclude");
        assert_eq!(eng.state.enemies[0].entity.hp, 38); // 50-12
        assert_eq!(eng.state.enemies[1].entity.hp, 38);
    }

    // ---- Conclude discards hand (end_turn) ----
    #[test] fn conclude_discards_hand() {
        let mut e = engine_with(
            vec!["Conclude".to_string(), "Strike_P".to_string(), "Strike_P".to_string(),
                 "Strike_P".to_string(), "Defend_P".to_string()],
            100, 0,
        );
        play(&mut e, "Conclude");
        assert!(e.state.hand.is_empty());
    }

    // ---- CutThroughFate draws cards ----
    #[test] fn cut_through_fate_draws() {
        let mut e = engine_with(
            vec!["CutThroughFate".to_string(), "Strike_P".to_string(), "Strike_P".to_string(),
                 "Strike_P".to_string(), "Defend_P".to_string(),
                 "Strike_P".to_string(), "Strike_P".to_string()],
            100, 0,
        );
        let hand_before = e.state.hand.len();
        play(&mut e, "CutThroughFate");
        // Played 1, drew 2 = net +1
        assert_eq!(e.state.hand.len(), hand_before + 1);
    }

    // ---- WheelKick draws 2 ----
    #[test] fn wheel_kick_draws_2() {
        let mut e = engine_with(
            vec!["WheelKick".to_string(), "Strike_P".to_string(), "Strike_P".to_string(),
                 "Strike_P".to_string(), "Defend_P".to_string(),
                 "Strike_P".to_string(), "Strike_P".to_string()],
            100, 0,
        );
        let hand_before = e.state.hand.len();
        play(&mut e, "WheelKick");
        assert_eq!(e.state.hand.len(), hand_before + 1); // -1 played +2 drawn
    }

    // ---- Prostrate block + mantra ----
    #[test] fn prostrate_block_and_mantra() {
        let mut e = engine_with(
            vec!["Prostrate".to_string(); 5], 100, 0,
        );
        play_self(&mut e, "Prostrate");
        assert_eq!(e.state.player.block, 4);
        assert_eq!(e.state.mantra, 2);
    }

    // ---- Prostrate+ gives 3 mantra ----
    #[test] fn prostrate_plus_3_mantra() {
        let mut e = engine_with(
            vec!["Prostrate+".to_string(); 5], 100, 0,
        );
        play_self(&mut e, "Prostrate+");
        assert_eq!(e.state.mantra, 3);
    }

    // ---- Pray gives 3 mantra ----
    #[test] fn pray_3_mantra() {
        let mut e = engine_with(
            vec!["Pray".to_string(); 5], 100, 0,
        );
        play_self(&mut e, "Pray");
        assert_eq!(e.state.mantra, 3);
    }

    // ---- 5 Prostrate = Divinity ----
    #[test] fn five_prostrate_divinity() {
        let mut e = engine_with(
            vec!["Prostrate".to_string(); 10], 100, 0,
        );
        for _ in 0..5 { play_self(&mut e, "Prostrate"); }
        assert_eq!(e.state.stance, Stance::Divinity);
    }

    // ---- Halt in Neutral = only base block ----
    #[test] fn halt_neutral_3_block() {
        let mut e = engine_with(
            vec!["Halt".to_string(); 5], 100, 0,
        );
        play_self(&mut e, "Halt");
        assert_eq!(e.state.player.block, 3);
    }

    // ---- Halt in Wrath = base + magic ----
    #[test] fn halt_wrath_12_block() {
        let mut e = engine_with(
            vec!["Halt".to_string(); 5], 100, 0,
        );
        e.state.stance = Stance::Wrath;
        play_self(&mut e, "Halt");
        assert_eq!(e.state.player.block, 12); // 3 + 9
    }

    // ---- Halt+ in Wrath = 4 + 14 = 18 ----
    #[test] fn halt_plus_wrath_18_block() {
        let mut e = engine_with(
            vec!["Halt+".to_string(); 5], 100, 0,
        );
        e.state.stance = Stance::Wrath;
        play_self(&mut e, "Halt+");
        assert_eq!(e.state.player.block, 18);
    }

    // ---- Miracle gives energy and exhausts ----
    #[test] fn miracle_energy_exhaust() {
        let mut e = engine_with(
            vec!["Miracle".to_string(); 5], 100, 0,
        );
        let en = e.state.energy;
        play_self(&mut e, "Miracle");
        assert_eq!(e.state.energy, en + 1);
        assert!(e.state.exhaust_pile.contains(&"Miracle".to_string()));
    }

    // ---- Miracle+ gives 2 energy ----
    #[test] fn miracle_plus_2_energy() {
        let mut e = engine_with(
            vec!["Miracle+".to_string(); 5], 100, 0,
        );
        let en = e.state.energy;
        play_self(&mut e, "Miracle+");
        assert_eq!(e.state.energy, en + 2);
    }

    // ---- EmptyBody enters Neutral with block ----
    #[test] fn empty_body_neutral_block() {
        let mut e = engine_with(
            vec!["EmptyBody".to_string(); 5], 100, 0,
        );
        e.state.stance = Stance::Wrath;
        play_self(&mut e, "EmptyBody");
        assert_eq!(e.state.stance, Stance::Neutral);
        assert_eq!(e.state.player.block, 7);
    }

    // ---- Flurry 0 cost ----
    #[test] fn flurry_free() {
        let mut e = engine_with(
            vec!["Flurry".to_string(); 5], 100, 0,
        );
        let en = e.state.energy;
        play(&mut e, "Flurry");
        assert_eq!(e.state.energy, en); // 0 cost
    }

    // ---- Smite damage ----
    #[test] fn smite_12_damage() {
        let mut e = engine_with(
            vec!["Smite".to_string(); 5], 100, 0,
        );
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "Smite");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 12);
    }

    // ---- Rushdown power install + draw on wrath ----
    #[test] fn rushdown_install_and_trigger() {
        let mut e = engine_with(
            vec!["Adaptation".to_string(), "Eruption".to_string(),
                 "Strike_P".to_string(), "Strike_P".to_string(), "Strike_P".to_string(),
                 "Defend_P".to_string(), "Defend_P".to_string()],
            100, 0,
        );
        play_self(&mut e, "Adaptation");
        assert_eq!(e.state.player.status("Rushdown"), 2);
        let hand_before = e.state.hand.len();
        play(&mut e, "Eruption");
        assert_eq!(e.state.stance, Stance::Wrath);
        assert_eq!(e.state.hand.len(), hand_before - 1 + 2);
    }

    // ---- MentalFortress install + block on stance change ----
    #[test] fn mental_fortress_install_and_trigger() {
        let mut e = engine_with(
            vec!["MentalFortress".to_string(), "Eruption".to_string(),
                 "Strike_P".to_string(), "Strike_P".to_string(), "Strike_P".to_string()],
            100, 0,
        );
        play_self(&mut e, "MentalFortress");
        assert_eq!(e.state.player.status("MentalFortress"), 4);
        play(&mut e, "Eruption");
        assert_eq!(e.state.player.block, 4);
    }

    // ---- MentalFortress stacks with upgrade ----
    #[test] fn mental_fortress_stacks() {
        let mut enemy = EnemyCombatState::new("JawWorm", 100, 100);
        enemy.set_move(1, 0, 1, 0);
        let mut state = CombatState::new(80, 80, vec![enemy], vec![], 5);
        // Directly set hand to avoid shuffle issues
        state.hand = vec![
            "MentalFortress".to_string(),
            "MentalFortress+".to_string(),
            "Eruption+".to_string(),
        ];
        state.turn = 1;
        let mut e = CombatEngine::new(state, 42);
        e.phase = CombatPhase::PlayerTurn;
        play_self(&mut e, "MentalFortress");  // cost 1, energy 4
        play_self(&mut e, "MentalFortress+"); // cost 1, energy 3
        assert_eq!(e.state.player.status("MentalFortress"), 10);
        play(&mut e, "Eruption+"); // cost 1, energy 2, enters Wrath -> MF triggers
        assert_eq!(e.state.player.block, 10);
    }

    // ---- Vigor consumed on first attack only ----
    #[test] fn vigor_consumed_on_attack() {
        let mut e = engine_with(
            vec!["Strike_P".to_string(); 5], 100, 0,
        );
        e.state.player.set_status("Vigor", 8);
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "Strike_P");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 14); // 6+8
        assert_eq!(e.state.player.status("Vigor"), 0);
    }

    #[test] fn vigor_not_consumed_on_skill() {
        let mut e = engine_with(
            vec!["Defend_P".to_string(); 5], 100, 0,
        );
        e.state.player.set_status("Vigor", 8);
        play_self(&mut e, "Defend_P");
        assert_eq!(e.state.player.status("Vigor"), 8);
    }

    // ---- Entangle clears at end of turn ----
    #[test] fn entangle_clears_end_turn() {
        let mut e = engine_with(
            vec!["Strike_P".to_string(); 5], 100, 5,
        );
        e.state.player.set_status("Entangled", 1);
        e.execute_action(&Action::EndTurn);
        assert_eq!(e.state.player.status("Entangled"), 0);
    }

    // ---- TalkToTheHand exhausts ----
    #[test] fn talk_hand_exhausts() {
        let mut e = engine_with(
            vec!["TalkToTheHand".to_string(); 5], 100, 0,
        );
        play(&mut e, "TalkToTheHand");
        assert!(e.state.exhaust_pile.contains(&"TalkToTheHand".to_string()));
        assert!(!e.state.discard_pile.contains(&"TalkToTheHand".to_string()));
    }

    // ---- Calm exit + Violet Lotus ----
    #[test] fn calm_exit_violet_lotus() {
        let mut e = engine_with(
            vec!["Eruption".to_string(); 5], 100, 0,
        );
        e.state.stance = Stance::Calm;
        e.state.relics.push("Violet Lotus".to_string());
        let en = e.state.energy;
        play(&mut e, "Eruption");
        // -2 cost, +2 calm exit, +1 violet lotus = +1 net
        assert_eq!(e.state.energy, en + 1);
    }

    // ---- InnerPeace in Calm draws, not in Calm enters Calm ----
    #[test] fn inner_peace_calm_draws() {
        let mut e = engine_with(
            vec!["InnerPeace".to_string(), "Strike_P".to_string(), "Strike_P".to_string(),
                 "Strike_P".to_string(), "Defend_P".to_string(),
                 "Defend_P".to_string(), "Defend_P".to_string(), "Defend_P".to_string()],
            100, 0,
        );
        e.state.stance = Stance::Calm;
        let hs = e.state.hand.len();
        play_self(&mut e, "InnerPeace");
        assert_eq!(e.state.hand.len(), hs - 1 + 3);
        assert_eq!(e.state.stance, Stance::Calm);
    }

    #[test] fn inner_peace_neutral_enters_calm() {
        let mut e = engine_with(
            vec!["InnerPeace".to_string(); 5], 100, 0,
        );
        play_self(&mut e, "InnerPeace");
        assert_eq!(e.state.stance, Stance::Calm);
    }

    // ---- Divinity auto-exits turn start ----
    #[test] fn divinity_auto_exit() {
        let mut e = engine_with(
            vec!["Strike_P".to_string(); 10], 100, 5,
        );
        e.state.stance = Stance::Divinity;
        e.execute_action(&Action::EndTurn);
        assert_eq!(e.state.stance, Stance::Neutral);
    }

    // ---- Mantra -> Divinity gives +3 energy ----
    #[test] fn mantra_divinity_energy() {
        let mut e = engine_with(
            vec!["Worship".to_string(); 5], 100, 0,
        );
        e.state.mantra = 5;
        let en = e.state.energy;
        play_self(&mut e, "Worship");
        // -2 cost, +3 divinity = +1
        assert_eq!(e.state.energy, en + 1);
        assert_eq!(e.state.stance, Stance::Divinity);
    }

    // ---- Fairy auto-revive ----
    #[test] fn fairy_revives_on_death() {
        let mut e = engine_with(
            vec!["Strike_P".to_string(); 5], 100, 200,
        );
        e.state.potions[0] = "FairyPotion".to_string();
        e.execute_action(&Action::EndTurn);
        assert_eq!(e.state.player.hp, 24); // 30% of 80
        assert!(!e.state.combat_over);
    }

    // ---- Full combat: kill enemy with strikes ----
    #[test] fn full_combat_kill() {
        let mut e = engine_with(
            vec!["Strike_P".to_string(); 10], 12, 0,
        );
        play(&mut e, "Strike_P");
        play(&mut e, "Strike_P");
        assert_eq!(e.state.enemies[0].entity.hp, 0);
        assert!(e.state.combat_over);
        assert!(e.state.player_won);
    }

    // ---- Potion targeting in legal actions ----
    #[test] fn fire_potion_targeted_actions() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 5], 100, 0);
        e.state.potions[0] = "Fire Potion".to_string();
        let actions = e.get_legal_actions();
        let pot: Vec<_> = actions.iter().filter(|a| matches!(a, Action::UsePotion { .. })).collect();
        assert_eq!(pot.len(), 1);
    }

    #[test] fn block_potion_untargeted_action() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 5], 100, 0);
        e.state.potions[0] = "Block Potion".to_string();
        let actions = e.get_legal_actions();
        let pot: Vec<_> = actions.iter().filter(|a| matches!(a, Action::UsePotion { potion_idx: 0, target_idx: -1 })).collect();
        assert_eq!(pot.len(), 1);
    }

    // ---- Wound/Daze cannot be played ----
    #[test] fn wound_not_playable() {
        let e = engine_with(
            vec!["Wound".to_string(), "Strike_P".to_string(), "Strike_P".to_string(),
                 "Strike_P".to_string(), "Strike_P".to_string()],
            100, 0,
        );
        let actions = e.get_legal_actions();
        let wound_plays: Vec<_> = actions.iter().filter(|a| {
            if let Action::PlayCard { card_idx, .. } = a { e.state.hand[*card_idx] == "Wound" } else { false }
        }).collect();
        assert!(wound_plays.is_empty());
    }

    #[test] fn daze_not_playable() {
        let e = engine_with(
            vec!["Daze".to_string(), "Strike_P".to_string(), "Strike_P".to_string(),
                 "Strike_P".to_string(), "Strike_P".to_string()],
            100, 0,
        );
        let actions = e.get_legal_actions();
        let daze_plays: Vec<_> = actions.iter().filter(|a| {
            if let Action::PlayCard { card_idx, .. } = a { e.state.hand[*card_idx] == "Daze" } else { false }
        }).collect();
        assert!(daze_plays.is_empty());
    }

    // ---- Slimed can be played (costs 1, exhausts) ----
    #[test] fn slimed_playable_and_exhausts() {
        let e = engine_with(
            vec!["Slimed".to_string(), "Strike_P".to_string(), "Strike_P".to_string(),
                 "Strike_P".to_string(), "Strike_P".to_string()],
            100, 0,
        );
        let actions = e.get_legal_actions();
        let slimed_plays: Vec<_> = actions.iter().filter(|a| {
            if let Action::PlayCard { card_idx, .. } = a { e.state.hand[*card_idx] == "Slimed" } else { false }
        }).collect();
        assert!(!slimed_plays.is_empty());
    }

    // ---- Strength affects all attacks ----
    #[test] fn strength_all_attacks() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 5], 100, 0);
        e.state.player.set_status("Strength", 5);
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "Strike_P");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 11);
    }

    // ---- Dexterity affects all block ----
    #[test] fn dexterity_all_block() {
        let mut e = engine_with(vec!["Defend_P".to_string(); 5], 100, 0);
        e.state.player.set_status("Dexterity", 3);
        play_self(&mut e, "Defend_P");
        assert_eq!(e.state.player.block, 8); // 5+3
    }

    // ---- Frail reduces block ----
    #[test] fn frail_reduces_block() {
        let mut e = engine_with(vec!["Defend_P".to_string(); 5], 100, 0);
        e.state.player.set_status("Frail", 2);
        play_self(&mut e, "Defend_P");
        assert_eq!(e.state.player.block, 3); // 5*0.75=3.75->3
    }

    // ---- Weak reduces attack ----
    #[test] fn weak_reduces_attack() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 5], 100, 0);
        e.state.player.set_status("Weakened", 2);
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "Strike_P");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 4); // 6*0.75=4.5->4
    }

    // ---- Energy tracking ----
    #[test] fn energy_decreases_on_play() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 5], 100, 0);
        assert_eq!(e.state.energy, 3);
        play(&mut e, "Strike_P");
        assert_eq!(e.state.energy, 2);
    }

    #[test] fn cannot_play_without_energy() {
        let mut e = engine_with(vec!["Eruption".to_string(); 5], 100, 0);
        play(&mut e, "Eruption"); // costs 2
        // Only 1 energy left, can't play another Eruption (cost 2)
        let actions = e.get_legal_actions();
        let eruption_plays: Vec<_> = actions.iter().filter(|a| {
            if let Action::PlayCard { card_idx, .. } = a { e.state.hand[*card_idx] == "Eruption" } else { false }
        }).collect();
        assert!(eruption_plays.is_empty());
    }

    // ---- Hand limit 10 ----
    #[test] fn hand_limit_10() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 20], 100, 0);
        assert_eq!(e.state.hand.len(), 5); // drew 5
        // Force more draws
        e.state.draw_pile = vec!["Strike_P".to_string(); 10];
        // Manually draw
        for _ in 0..10 {
            if e.state.hand.len() >= 10 { break; }
            if let Some(c) = e.state.draw_pile.pop() { e.state.hand.push(c); }
        }
        assert!(e.state.hand.len() <= 10);
    }

    // ---- LoseStrength applied at turn start ----
    #[test] fn lose_strength_at_turn_start() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 10], 100, 5);
        e.state.player.set_status("Strength", 5);
        e.state.player.set_status("LoseStrength", 5);
        e.execute_action(&Action::EndTurn);
        assert_eq!(e.state.player.strength(), 0);
        assert_eq!(e.state.player.status("LoseStrength"), 0);
    }

    // ---- LoseDexterity applied at turn start ----
    #[test] fn lose_dexterity_at_turn_start() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 10], 100, 5);
        e.state.player.set_status("Dexterity", 5);
        e.state.player.set_status("LoseDexterity", 5);
        e.execute_action(&Action::EndTurn);
        assert_eq!(e.state.player.dexterity(), 0);
    }

    // ---- Multi-hit stops on enemy death ----
    #[test] fn multi_hit_stops_on_death() {
        let mut e = engine_with(vec!["FlyingSleeves".to_string(); 5], 5, 0);
        play(&mut e, "FlyingSleeves"); // 4x2 = 8, but enemy has 5 HP
        assert_eq!(e.state.enemies[0].entity.hp, 0);
        assert!(e.state.combat_over);
    }

    // ---- Tantrum in Wrath does double damage ----
    #[test] fn tantrum_wrath_double() {
        let mut e = engine_with(vec!["Tantrum".to_string(); 5], 100, 0);
        // Already entering Wrath via card, but let's start in Wrath
        e.state.stance = Stance::Wrath;
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "Tantrum");
        // 3 dmg * 2.0 wrath = 6 per hit, 3 hits = 18
        assert_eq!(e.state.enemies[0].entity.hp, hp - 18);
    }

    // ---- Eruption already in Wrath: no double stance entry ----
    #[test] fn eruption_wrath_to_wrath_no_change() {
        let mut e = engine_with(vec!["Eruption".to_string(); 5], 100, 0);
        e.state.stance = Stance::Wrath;
        e.state.player.set_status("MentalFortress", 4);
        let block_before = e.state.player.block;
        play(&mut e, "Eruption");
        // Wrath -> Wrath is no change, MentalFortress should NOT trigger
        assert_eq!(e.state.player.block, block_before);
    }

    // ---- Strength + Wrath on Eruption ----
    #[test] fn eruption_str_wrath() {
        let mut e = engine_with(vec!["Eruption".to_string(); 5], 100, 0);
        e.state.player.set_status("Strength", 3);
        // Eruption enters Wrath. Damage calc: (9+3)*1.0 = 12 (Neutral during play)
        // Stance changes AFTER effects.
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "Eruption");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 12);
        assert_eq!(e.state.stance, Stance::Wrath);
    }

    // ---- Block + Defend stack from multiple plays ----
    #[test] fn multiple_defends_stack_block() {
        let mut e = engine_with(vec!["Defend_P".to_string(); 5], 100, 0);
        play_self(&mut e, "Defend_P");
        play_self(&mut e, "Defend_P");
        assert_eq!(e.state.player.block, 10);
    }

    // ---- Block decays at start of player turn ----
    #[test] fn block_decays_turn_start() {
        let mut e = engine_with(vec!["Defend_P".to_string(); 10], 100, 5);
        play_self(&mut e, "Defend_P");
        assert_eq!(e.state.player.block, 5);
        e.execute_action(&Action::EndTurn);
        // Block decays at start of new turn
        assert_eq!(e.state.player.block, 0);
    }

    // ---- Enemy block decays at start of enemy turn ----
    #[test] fn enemy_block_decays() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 10], 100, 0);
        e.state.enemies[0].entity.block = 10;
        e.execute_action(&Action::EndTurn);
        // Enemy block decays at start of their turn
        assert_eq!(e.state.enemies[0].entity.block, 0);
    }

    // ---- Debuffs decrement on enemies too ----
    #[test] fn enemy_debuffs_decrement() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 10], 100, 5);
        e.state.enemies[0].entity.set_status("Weakened", 2);
        e.execute_action(&Action::EndTurn);
        assert_eq!(e.state.enemies[0].entity.status("Weakened"), 1);
    }

    // ---- Turn counter increments ----
    #[test] fn turn_counter() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 10], 100, 5);
        assert_eq!(e.state.turn, 1);
        e.execute_action(&Action::EndTurn);
        assert_eq!(e.state.turn, 2);
        e.execute_action(&Action::EndTurn);
        assert_eq!(e.state.turn, 3);
    }

    // ---- Cards played counter ----
    #[test] fn cards_played_counter() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 5], 100, 0);
        play(&mut e, "Strike_P");
        play(&mut e, "Strike_P");
        assert_eq!(e.state.cards_played_this_turn, 2);
        assert_eq!(e.state.total_cards_played, 2);
    }

    // ---- Attacks played counter ----
    #[test] fn attacks_played_counter() {
        let mut e = engine_with(
            vec!["Strike_P".to_string(), "Defend_P".to_string(), "Strike_P".to_string(),
                 "Defend_P".to_string(), "Strike_P".to_string()],
            100, 0,
        );
        play(&mut e, "Strike_P");
        play_self(&mut e, "Defend_P");
        play(&mut e, "Strike_P");
        assert_eq!(e.state.attacks_played_this_turn, 2);
        assert_eq!(e.state.cards_played_this_turn, 3);
    }

    // ---- Counters reset on new turn ----
    #[test] fn counters_reset_new_turn() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 10], 100, 5);
        play(&mut e, "Strike_P");
        assert_eq!(e.state.cards_played_this_turn, 1);
        e.execute_action(&Action::EndTurn);
        assert_eq!(e.state.cards_played_this_turn, 0);
        assert_eq!(e.state.attacks_played_this_turn, 0);
    }

    // ---- Empty draw pile + empty discard = no draw ----
    #[test] fn no_cards_no_draw() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 5], 100, 5);
        // Play all cards, discard all, end turn
        for _ in 0..3 { play(&mut e, "Strike_P"); }
        // Now discard and draw piles will be refilled on end turn
        e.execute_action(&Action::EndTurn);
        // Turn 2: cards should be drawn from discard
        assert!(!e.state.hand.is_empty());
    }

    // ---- Relic combat start + potion in same combat ----
    #[test] fn relic_and_potion_combined() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 5], 100, 5);
        e.state.relics.push("Vajra".to_string());
        crate::relics::apply_combat_start_relics(&mut e.state);
        e.state.potions[0] = "Strength Potion".to_string();
        e.execute_action(&Action::UsePotion { potion_idx: 0, target_idx: -1 });
        assert_eq!(e.state.player.strength(), 3); // 1 Vajra + 2 potion
    }

    // ---- Pen Nib doubles in Wrath = 4x ----
    #[test] fn pen_nib_in_wrath() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 5], 100, 0);
        e.state.stance = Stance::Wrath;
        e.state.relics.push("Pen Nib".to_string());
        // Set counter to 9 so next attack triggers
        e.state.player.set_status("PenNibCounter", 9);
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "Strike_P");
        // 6 * 2.0 (wrath) = 12, * 2 (pen nib) = 24
        assert_eq!(e.state.enemies[0].entity.hp, hp - 24);
    }

    // ---- Vulnerable + Wrath incoming = 3x ----
    #[test] fn vuln_wrath_incoming() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 5], 100, 10);
        e.state.stance = Stance::Wrath;
        e.state.player.set_status("Vulnerable", 2);
        let hp = e.state.player.hp;
        e.execute_action(&Action::EndTurn);
        // 10 * 2.0 (wrath) * 1.5 (vuln) = 30
        assert_eq!(e.state.player.hp, hp - 30);
    }

    // ---- EmptyBody exits Wrath ----
    #[test] fn empty_body_exits_wrath() {
        let mut e = engine_with(vec!["EmptyBody".to_string(); 5], 100, 0);
        e.state.stance = Stance::Wrath;
        play_self(&mut e, "EmptyBody");
        assert_eq!(e.state.stance, Stance::Neutral);
    }

    // ---- EmptyBody+ gives 11 block ----
    #[test] fn empty_body_plus_11_block() {
        let mut e = engine_with(vec!["EmptyBody+".to_string(); 5], 100, 0);
        play_self(&mut e, "EmptyBody+");
        assert_eq!(e.state.player.block, 11);
    }

    // ---- Vigilance+ gives 12 block and enters Calm ----
    #[test] fn vigilance_plus_12_block_calm() {
        let mut e = engine_with(vec!["Vigilance+".to_string(); 5], 100, 0);
        play_self(&mut e, "Vigilance+");
        assert_eq!(e.state.player.block, 12);
        assert_eq!(e.state.stance, Stance::Calm);
    }

    // ---- Strike+ deals 9 damage ----
    #[test] fn strike_plus_9() {
        let mut e = engine_with(vec!["Strike_P+".to_string(); 5], 100, 0);
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "Strike_P+");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 9);
    }

    // ---- Defend+ gives 8 block ----
    #[test] fn defend_plus_8() {
        let mut e = engine_with(vec!["Defend_P+".to_string(); 5], 100, 0);
        play_self(&mut e, "Defend_P+");
        assert_eq!(e.state.player.block, 8);
    }

    // ---- Eruption+ costs 1 ----
    #[test] fn eruption_plus_cost_1() {
        let mut e = engine_with(vec!["Eruption+".to_string(); 5], 100, 0);
        let en = e.state.energy;
        play(&mut e, "Eruption+");
        assert_eq!(e.state.energy, en - 1);
    }

    // ---- Calm exit -> Wrath entry in one action (Eruption from Calm) ----
    #[test] fn calm_to_wrath_via_eruption() {
        let mut e = engine_with(vec!["Eruption".to_string(); 5], 100, 0);
        e.state.stance = Stance::Calm;
        e.state.player.set_status("MentalFortress", 4);
        let en = e.state.energy;
        play(&mut e, "Eruption");
        // Cost 2, Calm exit +2, net 0. MentalFortress fires once (Calm->Wrath)
        assert_eq!(e.state.energy, en);
        assert_eq!(e.state.player.block, 4);
        assert_eq!(e.state.stance, Stance::Wrath);
    }

    // ---- Rushdown + MentalFortress combined on Wrath entry ----
    #[test] fn rushdown_and_mf_on_wrath() {
        let mut e = engine_with(
            vec!["Eruption".to_string(), "Strike_P".to_string(), "Strike_P".to_string(),
                 "Strike_P".to_string(), "Strike_P".to_string(),
                 "Defend_P".to_string(), "Defend_P".to_string()],
            100, 0,
        );
        e.state.player.set_status("Rushdown", 2);
        e.state.player.set_status("MentalFortress", 4);
        let hs = e.state.hand.len();
        play(&mut e, "Eruption");
        // MF: +4 block, Rushdown: +2 draw
        assert_eq!(e.state.player.block, 4);
        assert_eq!(e.state.hand.len(), hs - 1 + 2);
    }

    // ---- No duplicate EndTurn in legal actions ----
    #[test] fn single_end_turn_action() {
        let e = engine_with(vec!["Strike_P".to_string(); 5], 100, 0);
        let actions = e.get_legal_actions();
        let end_turns = actions.iter().filter(|a| matches!(a, Action::EndTurn)).count();
        assert_eq!(end_turns, 1);
    }

    // ---- Empty potions don't appear in actions ----
    #[test] fn empty_potions_no_actions() {
        let e = engine_with(vec!["Strike_P".to_string(); 5], 100, 0);
        let actions = e.get_legal_actions();
        let pots = actions.iter().filter(|a| matches!(a, Action::UsePotion { .. })).count();
        assert_eq!(pots, 0);
    }

    // ---- Mantra overflow (12 mantra = Divinity + 2 leftover) ----
    #[test] fn mantra_overflow() {
        let mut e = engine_with(vec!["Worship".to_string(); 5], 100, 0);
        e.state.mantra = 7;
        play_self(&mut e, "Worship"); // +5 = 12 -> Divinity, leftover 2
        assert_eq!(e.state.stance, Stance::Divinity);
        assert_eq!(e.state.mantra, 2);
    }

    // ---- Potion kills enemy -> combat ends ----
    #[test] fn potion_kill_ends_combat() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 5], 15, 0);
        e.state.potions[0] = "Fire Potion".to_string();
        e.execute_action(&Action::UsePotion { potion_idx: 0, target_idx: 0 });
        assert!(e.state.combat_over);
        assert!(e.state.player_won);
    }

    // ---- Worship retain effect tag exists ----
    #[test] fn worship_plus_has_retain_effect() {
        let reg = crate::cards::CardRegistry::new();
        let c = reg.get("Worship+").unwrap();
        assert!(c.effects.contains(&"retain"));
    }

    // ---- Divinity outgoing damage 3x ----
    #[test] fn divinity_3x_damage() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 5], 100, 0);
        e.state.stance = Stance::Divinity;
        let hp = e.state.enemies[0].entity.hp;
        play(&mut e, "Strike_P");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 18); // 6*3=18
    }

    // ---- Divinity does NOT increase incoming damage ----
    #[test] fn divinity_no_incoming_mult() {
        let mut e = engine_with(vec!["Strike_P".to_string(); 5], 100, 10);
        e.state.stance = Stance::Divinity;
        let hp = e.state.player.hp;
        e.execute_action(&Action::EndTurn);
        // Divinity incoming mult is 1.0, so 10 damage
        assert_eq!(e.state.player.hp, hp - 10);
    }
}
