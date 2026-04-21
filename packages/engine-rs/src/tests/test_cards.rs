#[cfg(test)]
mod card_registry_tests {
    use crate::cards::*;

    fn reg() -> &'static CardRegistry {
        crate::cards::global_registry()
    }

    // ========== Watcher Basics ==========

    #[test]
    fn strike_base_values() {
        let c = reg().get("Strike").unwrap().clone();
        assert_eq!(c.base_damage, 6);
        assert_eq!(c.cost, 1);
        assert_eq!(c.card_type, CardType::Attack);
        assert_eq!(c.target, CardTarget::Enemy);
        assert!(!c.exhaust);
        assert!(c.enter_stance.is_none());
    }

    #[test]
    fn strike_upgraded_values() {
        let c = reg().get("Strike+").unwrap().clone();
        assert_eq!(c.base_damage, 9);
        assert_eq!(c.cost, 1);
    }

    #[test]
    fn defend_base_values() {
        let c = reg().get("Defend").unwrap().clone();
        assert_eq!(c.base_block, 5);
        assert_eq!(c.cost, 1);
        assert_eq!(c.card_type, CardType::Skill);
        assert_eq!(c.target, CardTarget::SelfTarget);
    }

    #[test]
    fn defend_upgraded_values() {
        let c = reg().get("Defend+").unwrap().clone();
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
        assert!(c.has_test_marker("damage_per_enemy"));
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
        assert!(c.has_test_marker("vuln_if_last_skill"));
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
        assert!(c.has_test_marker("scry"));
        assert!(c.has_test_marker("draw"));
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
        assert_eq!(c.base_block, 10);
    }

    #[test]
    fn flurry_base() {
        let c = reg().get("FlurryOfBlows").unwrap().clone();
        assert_eq!(c.base_damage, 4);
        assert_eq!(c.cost, 0);
    }

    #[test]
    fn flurry_upgraded() {
        let c = reg().get("FlurryOfBlows+").unwrap().clone();
        assert_eq!(c.base_damage, 6);
        assert_eq!(c.cost, 0);
    }

    #[test]
    fn flying_sleeves_base() {
        let c = reg().get("FlyingSleeves").unwrap().clone();
        assert_eq!(c.base_damage, 4);
        assert_eq!(c.base_magic, 2);
        assert!(c.has_test_marker("multi_hit"));
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
        assert!(c.has_test_marker("energy_if_last_attack"));
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
        assert!(c.has_test_marker("extra_block_in_wrath"));
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
        assert!(c.has_test_marker("mantra"));
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
        assert!(c.has_test_marker("multi_hit"));
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
        assert!(c.has_test_marker("scry"));
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
        assert!(c.has_test_marker("if_calm_draw_else_calm"));
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
        assert!(c.has_test_marker("draw"));
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
        assert!(c.has_test_marker("end_turn"));
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
        assert!(c.has_test_marker("apply_block_return"));
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
        assert!(c.has_test_marker("mantra"));
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
        assert!(c.has_test_marker("mantra"));
    }

    #[test]
    fn worship_upgraded_has_retain() {
        let c = reg().get("Worship+").unwrap().clone();
        assert_eq!(c.base_magic, 5);
        assert!(c.has_test_marker("retain"));
    }

    // ========== Power Cards ==========

    #[test]
    fn rushdown_base() {
        let c = reg().get("Adaptation").unwrap().clone();
        assert_eq!(c.card_type, CardType::Power);
        assert_eq!(c.base_magic, 2);
        assert_eq!(c.cost, 1);
        assert!(c.has_test_marker("on_wrath_draw"));
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
        assert!(c.has_test_marker("on_stance_change_block"));
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
        assert_eq!(c.enter_stance, None); // Java Ragnarok does NOT change stance
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
        assert!(c.has_test_marker("gain_energy"));
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
        assert!(c.has_test_marker("retain"));
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
        assert!(c.has_test_marker("unplayable"));
        assert!(c.is_unplayable());
    }

    #[test]
    fn daze_is_unplayable_ethereal() {
        let c = reg().get("Daze").unwrap().clone();
        assert_eq!(c.cost, -2);
        assert!(c.has_test_marker("unplayable"));
        assert!(c.has_test_marker("ethereal"));
    }

    #[test]
    fn burn_is_unplayable() {
        let c = reg().get("Burn").unwrap().clone();
        assert_eq!(c.cost, -2);
        assert!(c.has_test_marker("unplayable"));
    }

    #[test]
    fn ascenders_bane_properties() {
        let c = reg().get("AscendersBane").unwrap().clone();
        assert_eq!(c.card_type, CardType::Curse);
        assert!(c.has_test_marker("unplayable"));
        assert!(c.has_test_marker("ethereal"));
    }

    // ========== Colorless ==========

    #[test]
    fn strike_r_same_as_p() {
        let r = reg();
        let sp = r.get("Strike").unwrap();
        let sr = r.get("Strike").unwrap();
        assert_eq!(sp.base_damage, sr.base_damage);
    }

    // ========== Utility ==========

    #[test]
    fn is_upgraded_check() {
        assert!(CardRegistry::is_upgraded("Strike+"));
        assert!(CardRegistry::is_upgraded("Eruption+"));
        assert!(CardRegistry::is_upgraded("MentalFortress+"));
        assert!(!CardRegistry::is_upgraded("Strike"));
        assert!(!CardRegistry::is_upgraded("Eruption"));
    }

    #[test]
    fn base_id_strips_plus() {
        assert_eq!(CardRegistry::base_id("Strike+"), "Strike");
        assert_eq!(CardRegistry::base_id("Strike"), "Strike");
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
            "Strike", "Strike+", "Defend", "Defend+",
            "Eruption", "Eruption+", "Vigilance", "Vigilance+",
            "BowlingBash", "BowlingBash+", "CrushJoints", "CrushJoints+",
            "CutThroughFate", "CutThroughFate+", "EmptyBody", "EmptyBody+",
            "FlurryOfBlows", "FlurryOfBlows+", "FlyingSleeves", "FlyingSleeves+",
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
            "Strike", "Defend",
        ];
        for id in &expected {
            assert!(r.get(id).is_some(), "Missing card: {}", id);
        }
    }
}

// =============================================================================
// Damage calculation exhaustive tests
// =============================================================================

