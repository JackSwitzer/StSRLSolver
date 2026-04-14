#[cfg(test)]
mod relic_java_parity_tests {
    // Java references:
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Vajra.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/OddlySmoothStone.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/DataDisk.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Akabeko.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BagOfMarbles.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/RedMask.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/ThreadAndNeedle.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BronzeScales.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Anchor.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BloodVial.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/ClockworkSouvenir.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/FossilizedHelix.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/MarkOfPain.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/MutagenicStrength.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/PureWater.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/HolyWater.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/NinjaScroll.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BagOfPreparation.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/RingOfTheSnake.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/PhilosopherStone.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/PenNib.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/OrnamentalFan.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Kunai.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Shuriken.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/LetterOpener.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Nunchaku.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/InkBottle.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/VelvetChoker.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Pocketwatch.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/ArtOfWar.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BirdFacedUrn.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/MummifiedHand.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/OrangePellets.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/SneckoEye.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/HappyFlower.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/IncenseBurner.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/MercuryHourglass.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Brimstone.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Damaru.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Inserter.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/HornCleat.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/CaptainsWheel.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/StoneCalendar.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Orichalcum.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/CloakClasp.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/CharonsAshes.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/ToughBandages.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Tingsha.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/ToyOrnithopter.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/HandDrill.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/StrikeDummy.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/WristBlade.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/SneckoSkull.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Boot.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Torii.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/TungstenRod.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/ChemicalX.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/GoldenCables.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/RunicPyramid.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/IceCream.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/SacredBark.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Necronomicon.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/CentennialPuzzle.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/SelfFormingClay.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/RunicCube.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/RedSkull.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/EmotionChip.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/GremlinHorn.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/TheSpecimen.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BurningBlood.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BlackBlood.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/MeatOnTheBone.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/FaceOfCleric.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/GremlinMask.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Pantograph.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Sling.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/PreservedInsect.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/NeowsLament.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/DuVuDoll.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Girya.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/TeardropLocket.java

    use crate::status_ids::sid;
    use crate::relics::*;
    use crate::state::{CombatState, EnemyCombatState};
    use crate::tests::support::make_deck_n;

    fn base_state() -> CombatState {
        let enemy = EnemyCombatState::new("JawWorm", 50, 50);
        CombatState::new(80, 80, vec![enemy], make_deck_n("Strike_P", 5), 3)
    }

    fn two_enemy_state() -> CombatState {
        let e1 = EnemyCombatState::new("JawWorm", 40, 40);
        let e2 = EnemyCombatState::new("Cultist", 50, 50);
        CombatState::new(80, 80, vec![e1, e2], make_deck_n("Strike_P", 5), 3)
    }

    #[test]
    fn dead_branch_returns_true_when_owned() {
        let mut state = base_state();
        state.relics.push("Dead Branch".to_string());
        assert!(dead_branch_on_exhaust(&state));
        state.relics.clear();
        assert!(!dead_branch_on_exhaust(&state));
    }

    #[test]
    fn charons_ashes_deals_three_to_all_enemies_on_exhaust() {
        let mut state = two_enemy_state();
        state.relics.push("Charon's Ashes".to_string());
        let hp0 = state.enemies[0].entity.hp;
        let hp1 = state.enemies[1].entity.hp;
        charons_ashes_on_exhaust(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp0 - 3);
        assert_eq!(state.enemies[1].entity.hp, hp1 - 3);
    }

    #[test]
    fn tough_bandages_give_three_block_on_discard() {
        let mut state = base_state();
        state.relics.push("Tough Bandages".to_string());
        tough_bandages_on_discard(&mut state);
        assert_eq!(state.player.block, 3);
    }

    #[test]
    fn tingsha_deals_three_to_first_alive_enemy_on_discard() {
        let mut state = base_state();
        state.relics.push("Tingsha".to_string());
        let hp = state.enemies[0].entity.hp;
        tingsha_on_discard(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp - 3);
    }

    #[test]
    fn toy_ornithopter_heals_five_on_potion() {
        let mut state = base_state();
        state.relics.push("Toy Ornithopter".to_string());
        state.player.hp = 70;
        toy_ornithopter_on_potion(&mut state);
        assert_eq!(state.player.hp, 75);
    }

    #[test]
    fn hand_drill_applies_two_vulnerable_when_block_break() {
        let mut state = base_state();
        state.relics.push("HandDrill".to_string());
        hand_drill_on_block_break(&mut state, 0);
        assert_eq!(state.enemies[0].entity.status(sid::VULNERABLE), 2);
    }

    #[test]
    fn strike_dummy_bonus_is_three() {
        let mut state = base_state();
        assert_eq!(strike_dummy_bonus(&state), 0);
        state.relics.push("StrikeDummy".to_string());
        assert_eq!(strike_dummy_bonus(&state), 3);
    }

    #[test]
    fn wrist_blade_bonus_is_four() {
        let mut state = base_state();
        assert_eq!(wrist_blade_bonus(&state), 0);
        state.relics.push("WristBlade".to_string());
        assert_eq!(wrist_blade_bonus(&state), 4);
    }

    #[test]
    fn snecko_skull_bonus_is_one() {
        let mut state = base_state();
        assert_eq!(snecko_skull_bonus(&state), 0);
        state.relics.push("SneckoSkull".to_string());
        assert_eq!(snecko_skull_bonus(&state), 1);
    }

    #[test]
    fn chemical_x_bonus_is_two() {
        let mut state = base_state();
        assert_eq!(chemical_x_bonus(&state), 0);
        state.relics.push("Chemical X".to_string());
        assert_eq!(chemical_x_bonus(&state), 2);
    }

    #[test]
    fn gold_plated_cables_only_active_at_full_hp() {
        let mut state = base_state();
        state.relics.push("Cables".to_string());
        assert!(gold_plated_cables_active(&state));
        state.player.hp = 70;
        assert!(!gold_plated_cables_active(&state));
    }

    #[test]
    fn runic_pyramid_presence_check() {
        let mut state = base_state();
        state.relics.push("Runic Pyramid".to_string());
        assert!(has_runic_pyramid(&state));
    }

    #[test]
    fn ice_cream_presence_check() {
        let mut state = base_state();
        state.relics.push("Ice Cream".to_string());
        assert!(has_ice_cream(&state));
    }

    #[test]
    fn sacred_bark_presence_check() {
        let mut state = base_state();
        state.relics.push("SacredBark".to_string());
        assert!(has_sacred_bark(&state));
    }

    #[test]
    fn calipers_retains_up_to_fifteen_block() {
        let mut state = base_state();
        state.relics.push("Calipers".to_string());
        assert_eq!(calipers_block_retention(&state, 20), 15);
        assert_eq!(calipers_block_retention(&state, 10), 10);
    }

    #[test]
    fn unceasing_top_draws_when_hand_empty() {
        let mut state = base_state();
        state.relics.push("Unceasing Top".to_string());
        state.hand.clear();
        { let reg = crate::cards::global_registry(); state.draw_pile.push(reg.make_card("Strike_P")); };
        assert!(unceasing_top_should_draw(&state));
        { let reg = crate::cards::global_registry(); state.hand.push(reg.make_card("Defend_P")); };
        assert!(!unceasing_top_should_draw(&state));
    }

    #[test]
    fn necronomicon_triggers_once_for_first_two_cost_attack() {
        let mut state = base_state();
        state.relics.push("Necronomicon".to_string());
        assert!(necronomicon_should_trigger(&state, 2, true));
        assert!(!necronomicon_should_trigger(&state, 1, true));
        assert!(!necronomicon_should_trigger(&state, 2, false));
        necronomicon_mark_used(&mut state);
        assert!(!necronomicon_should_trigger(&state, 2, true));
    }

    #[test]
    fn necronomicon_reset_clears_flag() {
        let mut state = base_state();
        state.relics.push("Necronomicon".to_string());
        state.player.set_status(sid::NECRONOMICON_USED, 1);
        necronomicon_reset(&mut state);
        assert_eq!(state.player.status(sid::NECRONOMICON_USED), 0);
    }

    macro_rules! extra_case {
        ($name:ident, $body:block) => {
            #[test]
            fn $name() $body
        };
    }

    extra_case!(ice_cream_compact_name, {
        let mut state = base_state();
        state.relics.push("IceCream".to_string());
        assert!(has_ice_cream(&state));
    });

    extra_case!(runic_pyramid_compact_name, {
        let mut state = base_state();
        state.relics.push("RunicPyramid".to_string());
        assert!(has_runic_pyramid(&state));
    });

    extra_case!(sacred_bark_negative, {
        let state = base_state();
        assert!(!has_sacred_bark(&state));
    });

    extra_case!(runic_pyramid_negative, {
        let state = base_state();
        assert!(!has_runic_pyramid(&state));
    });

    extra_case!(ice_cream_negative, {
        let state = base_state();
        assert!(!has_ice_cream(&state));
    });

    extra_case!(calipers_no_relic_returns_zero, {
        let state = base_state();
        assert_eq!(calipers_block_retention(&state, 20), 0);
    });

    extra_case!(calipers_zero_block_returns_zero, {
        let mut state = base_state();
        state.relics.push("Calipers".to_string());
        assert_eq!(calipers_block_retention(&state, 0), 0);
    });

    extra_case!(gold_plated_cables_no_relic, {
        let state = base_state();
        assert!(!gold_plated_cables_active(&state));
    });

    extra_case!(gold_plated_cables_not_full, {
        let mut state = base_state();
        state.relics.push("Cables".to_string());
        state.player.hp = 70;
        assert!(!gold_plated_cables_active(&state));
    });

    extra_case!(unceasing_top_no_relic, {
        let state = base_state();
        assert!(!unceasing_top_should_draw(&state));
    });

    extra_case!(unceasing_top_nonempty_hand, {
        let mut state = base_state();
        state.relics.push("Unceasing Top".to_string());
        { let reg = crate::cards::global_registry(); state.hand.push(reg.make_card("Defend_P")); };
        { let reg = crate::cards::global_registry(); state.draw_pile.push(reg.make_card("Strike_P")); };
        assert!(!unceasing_top_should_draw(&state));
    });

    extra_case!(chemical_x_no_relic, {
        let state = base_state();
        assert_eq!(chemical_x_bonus(&state), 0);
    });

    extra_case!(strike_dummy_no_relic, {
        let state = base_state();
        assert_eq!(strike_dummy_bonus(&state), 0);
    });

    extra_case!(wrist_blade_no_relic, {
        let state = base_state();
        assert_eq!(wrist_blade_bonus(&state), 0);
    });

    extra_case!(snecko_skull_no_relic, {
        let state = base_state();
        assert_eq!(snecko_skull_bonus(&state), 0);
    });

    extra_case!(necronomicon_no_relic_false, {
        let state = base_state();
        assert!(!necronomicon_should_trigger(&state, 2, true));
    });

    extra_case!(necronomicon_non_attack_false, {
        let mut state = base_state();
        state.relics.push("Necronomicon".to_string());
        assert!(!necronomicon_should_trigger(&state, 2, false));
    });

    extra_case!(necronomicon_one_cost_attack_false, {
        let mut state = base_state();
        state.relics.push("Necronomicon".to_string());
        assert!(!necronomicon_should_trigger(&state, 1, true));
    });

    // --- Turn end tests ---

    extra_case!(charons_ashes_no_relic_no_damage, {
        let mut state = base_state();
        let hp = state.enemies[0].entity.hp;
        charons_ashes_on_exhaust(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp);
    });

    extra_case!(tough_bandages_no_relic_no_block, {
        let mut state = base_state();
        tough_bandages_on_discard(&mut state);
        assert_eq!(state.player.block, 0);
    });

    extra_case!(tingsha_no_relic_no_damage, {
        let mut state = base_state();
        let hp = state.enemies[0].entity.hp;
        tingsha_on_discard(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp);
    });

    extra_case!(toy_ornithopter_no_relic_no_heal, {
        let mut state = base_state();
        state.player.hp = 70;
        toy_ornithopter_on_potion(&mut state);
        assert_eq!(state.player.hp, 70);
    });

    extra_case!(hand_drill_no_relic_no_vuln, {
        let mut state = base_state();
        hand_drill_on_block_break(&mut state, 0);
        assert_eq!(state.enemies[0].entity.status(sid::VULNERABLE), 0);
    });

}
