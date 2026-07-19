#[cfg(test)]
mod boss_java_parity_tests {
    // Java references:
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/exordium/TheGuardian.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/exordium/Hexaghost.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/exordium/SlimeBoss.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/city/BronzeAutomaton.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/city/TheCollector.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/city/Champ.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/beyond/AwakenedOne.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/beyond/Donu.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/beyond/Deca.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/beyond/TimeEater.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/ending/CorruptHeart.java

    use crate::combat_hooks::do_enemy_turns;
    use crate::combat_types::mfx;
    use crate::status_ids::sid;
    use crate::engine::CombatEngine;
    use crate::enemies::*;
    use crate::enemies::move_ids;
    use crate::run::RunEngine;
    use crate::tests::support::*;

    fn boss_engine(id: &str, hp: i32, max_hp: i32) -> CombatEngine {
        engine_without_start(Vec::new(), vec![create_enemy(id, hp, max_hp)], 3)
    }

    // ---------------------------------------------------------------------
    // Act 1 bosses
    // ---------------------------------------------------------------------

    #[test]
    fn guardian_base_hp_and_opening_move() {
        let enemy = create_enemy("TheGuardian", 240, 240);
        assert_eq!(enemy.entity.hp, 240);
        assert_eq!(enemy.entity.max_hp, 240);
        assert_eq!(enemy.move_id, move_ids::GUARD_CHARGING_UP);
        assert_eq!(enemy.move_block(), 9);
        assert_eq!(enemy.entity.status(sid::MODE_SHIFT), 30);
    }

    #[test]
    fn guardian_constructor_defaults_match_java() {
        // Ascension-derived values are patched at the run spawn site.
        let enemy = create_enemy("TheGuardian", 250, 250);
        assert_eq!(enemy.entity.hp, 250);
        assert_eq!(enemy.entity.max_hp, 250);
        assert_eq!(enemy.entity.status(sid::FIERCE_BASH_DMG), 32);
        assert_eq!(enemy.entity.status(sid::ROLL_DMG), 9);
    }

    #[test]
    fn guardian_defensive_mode_uses_java_threshold_and_sharp_hide() {
        let mut enemy = create_enemy("TheGuardian", 240, 240);
        enemy.entity.set_status(sid::MODE_SHIFT, 40);
        enemy.entity.set_status(sid::STR_AMT, 4);

        let shifted = guardian_check_mode_shift(&mut enemy, 40);
        assert!(shifted);
        assert_eq!(enemy.entity.status(sid::SHARP_HIDE), 0);
        assert_eq!(enemy.move_id, move_ids::GUARD_CLOSE_UP);
        assert_eq!(enemy.effect(mfx::SHARP_HIDE), Some(4));
        assert_eq!(enemy.entity.status(sid::MODE_SHIFT), 50);
    }

    #[test]
    fn guardian_switch_back_to_offensive_matches_java() {
        let mut enemy = create_enemy("TheGuardian", 240, 240);
        guardian_switch_to_offensive(&mut enemy);
        assert_eq!(enemy.entity.status(sid::SHARP_HIDE), 0);
        assert_eq!(enemy.move_id, move_ids::GUARD_WHIRLWIND);
        assert_eq!(enemy.move_damage(), 5);
    }

    #[test]
    fn guardian_offensive_cycle_matches_java_base_values() {
        let mut enemy = create_enemy("TheGuardian", 240, 240);

        crate::enemies::act1::advance_guardian_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_FIERCE_BASH);
        assert_eq!(enemy.move_damage(), 32);

        crate::enemies::act1::advance_guardian_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_VENT_STEAM);
        assert_eq!(enemy.effect(mfx::WEAK), Some(2));
        assert_eq!(enemy.effect(mfx::VULNERABLE), Some(2));

        crate::enemies::act1::advance_guardian_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_WHIRLWIND);
        assert_eq!(enemy.move_damage(), 5);
        assert_eq!(enemy.move_hits(), 4);

        crate::enemies::act1::advance_guardian_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_CHARGING_UP);
        assert_eq!(enemy.move_block(), 9);
    }

    #[test]
    fn hexaghost_base_hp_and_activation() {
        let enemy = create_enemy("Hexaghost", 250, 250);
        assert_eq!(enemy.entity.hp, 250);
        assert_eq!(enemy.entity.max_hp, 250);
        assert_eq!(enemy.move_id, move_ids::HEX_ACTIVATE);
    }

    #[test]
    fn hexaghost_a2_hp_matches_java() {
        let enemy = create_enemy("Hexaghost", 264, 264);
        assert_eq!(enemy.entity.hp, 264);
        assert_eq!(enemy.entity.max_hp, 264);
        assert_eq!(enemy.move_id, move_ids::HEX_ACTIVATE);
    }

    #[test]
    fn hexaghost_divider_formula_matches_java() {
        let mut enemy = create_enemy("Hexaghost", 250, 250);
        hexaghost_set_divider(&mut enemy, 80);
        assert_eq!(enemy.move_id, move_ids::HEX_DIVIDER);
        assert_eq!(enemy.move_damage(), 7);
        assert_eq!(enemy.move_hits(), 6);

        hexaghost_set_divider(&mut enemy, 60);
        assert_eq!(enemy.move_damage(), 6);
        assert_eq!(enemy.move_hits(), 6);
    }

    #[test]
    fn hexaghost_base_cycle_matches_java_shape() {
        let mut enemy = create_enemy("Hexaghost", 250, 250);
        let mut rng = crate::seed::StsRandom::new(0);

        crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 80, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEX_DIVIDER);
        assert_eq!(enemy.move_damage(), 7);

        crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 38, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEX_SEAR);
        assert_eq!(enemy.move_damage(), 6);
        assert_eq!(enemy.effect(mfx::BURN), Some(1));

        crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 32, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEX_TACKLE);
        assert_eq!(enemy.move_damage(), 5);
        assert_eq!(enemy.move_hits(), 2);

        crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 22, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEX_SEAR);

        crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 16, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEX_INFLAME);
        assert_eq!(enemy.move_block(), 12);
        assert_eq!(enemy.effect(mfx::STRENGTH), Some(2));

        crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 16, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEX_TACKLE);

        crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 6, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEX_SEAR);

        crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 1, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEX_INFERNO);
        assert_eq!(enemy.move_damage(), 2);
        assert_eq!(enemy.move_hits(), 6);
    }

    #[test]
    fn hexaghost_a4_scaling_matches_java_expectations() {
        let mut enemy = create_enemy("Hexaghost", 264, 264);
        enemy.entity.set_status(sid::FIRE_TACKLE_DMG, 6);
        enemy.entity.set_status(sid::INFERNO_DMG, 3);
        let mut rng = crate::seed::StsRandom::new(0);

        // Activate -> Divider -> Sear(orb=0) -> Tackle(orb=1)
        crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 80, &mut rng);
        crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 38, &mut rng);
        crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 32, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEX_TACKLE);
        assert_eq!(enemy.move_damage(), 6);
        assert_eq!(enemy.move_hits(), 2);

        // Sear(2) -> Inflame(3) -> Tackle(4) -> Sear(5) -> Inferno(6)
        for _ in 0..5 {
            crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 80, &mut rng);
        }
        assert_eq!(enemy.move_id, move_ids::HEX_INFERNO);
        assert_eq!(enemy.move_damage(), 3);
        assert_eq!(enemy.move_hits(), 6);
    }

    #[test]
    fn hexaghost_a19_burn_and_strength_matches_java_expectations() {
        let mut enemy = create_enemy("Hexaghost", 264, 264);
        enemy.entity.set_status(sid::STR_AMT, 3);
        enemy.entity.set_status(sid::SEAR_BURN_COUNT, 2);
        let mut rng = crate::seed::StsRandom::new(0);

        crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 80, &mut rng);
        crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 38, &mut rng);
        assert_eq!(enemy.effect(mfx::BURN), Some(2));

        // Sear(0) -> Tackle(1) -> Sear(2) -> Inflame(3)
        for _ in 0..3 {
            crate::enemies::act1::advance_hexaghost_after_turn(&mut enemy, 80, &mut rng);
        }
        assert_eq!(enemy.move_id, move_ids::HEX_INFLAME);
        assert_eq!(enemy.effect(mfx::STRENGTH), Some(3));
    }

    #[test]
    fn slime_boss_base_hp_and_opening_move() {
        let enemy = create_enemy("SlimeBoss", 140, 140);
        assert_eq!(enemy.entity.hp, 140);
        assert_eq!(enemy.entity.max_hp, 140);
        assert_eq!(enemy.move_id, move_ids::SB_STICKY);
        assert_eq!(enemy.effect(mfx::SLIMED), Some(3));
    }

    #[test]
    fn slime_boss_a2_hp_matches_java() {
        let enemy = create_enemy("SlimeBoss", 150, 150);
        assert_eq!(enemy.entity.hp, 150);
        assert_eq!(enemy.entity.max_hp, 150);
        assert_eq!(enemy.move_id, move_ids::SB_STICKY);
    }

    #[test]
    fn slime_boss_split_hook_matches_java() {
        let mut engine = boss_engine("SlimeBoss", 140, 140);
        engine.deal_damage_to_enemy(0, 70);

        assert_eq!(engine.state.enemies[0].entity.hp, 70);
        assert_eq!(engine.state.enemies[0].move_id, move_ids::SB_SPLIT);
        assert_eq!(engine.state.enemies.len(), 1);

        do_enemy_turns(&mut engine);
        assert_eq!(engine.state.enemies[0].entity.hp, 0);
        assert_eq!(engine.state.enemies.len(), 3);
        assert_eq!(engine.state.enemies[1].id, "SpikeSlime_L");
        assert_eq!(engine.state.enemies[2].id, "AcidSlime_L");
        assert_eq!(engine.state.enemies[1].entity.hp, 70);
        assert_eq!(engine.state.enemies[2].entity.hp, 70);
        assert!(slime_boss_should_split(&create_enemy("SlimeBoss", 70, 140)));
        assert!(!slime_boss_should_split(&create_enemy("SlimeBoss", 71, 140)));
    }

    // ---------------------------------------------------------------------
    // Act 2 bosses
    // ---------------------------------------------------------------------

    #[test]
    fn bronze_automaton_base_hp_and_opening_move() {
        let enemy = create_enemy("BronzeAutomaton", 300, 300);
        assert_eq!(enemy.entity.hp, 300);
        assert_eq!(enemy.move_id, move_ids::BA_SPAWN_ORBS);
    }

    #[test]
    fn bronze_automaton_factory_uses_source_base_stats() {
        // Source: reference/extracted/methods/monster/BronzeAutomaton.java.
        // create_enemy has no ascension input, so RunEngine applies the
        // independent A4/A9/A19 gates at the spawn site.
        let enemy = create_enemy("BronzeAutomaton", 320, 320);
        assert_eq!(enemy.entity.status(sid::FLAIL_DMG), 7);
        assert_eq!(enemy.entity.status(sid::BEAM_DMG), 45);
        assert_eq!(enemy.entity.status(sid::STR_AMT), 3);
        assert_eq!(enemy.entity.status(sid::BLOCK_AMT), 9);
        assert_eq!(enemy.entity.status(sid::ARTIFACT), 3);
    }

    #[test]
    fn bronze_automaton_cycle_matches_java_base_pattern() {
        let mut enemy = create_enemy("BronzeAutomaton", 300, 300);
        roll_initial_move_with_num_and_rng(
            &mut enemy, 0, &mut crate::seed::StsRandom::new(0));

        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::BA_FLAIL);
        assert_eq!(enemy.move_damage(), 7);
        assert_eq!(enemy.move_hits(), 2);

        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::BA_BOOST);
        assert_eq!(enemy.effect(mfx::STRENGTH), Some(3));

        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::BA_FLAIL);

        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::BA_BOOST);

        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::BA_HYPER_BEAM);
        assert_eq!(enemy.move_damage(), 45);

        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::BA_STUNNED);
    }

    #[test]
    fn collector_base_hp_and_spawn() {
        let enemy = create_enemy("TheCollector", 282, 282);
        assert_eq!(enemy.entity.hp, 282);
        assert_eq!(enemy.move_id, move_ids::COLL_SPAWN);
    }

    #[test]
    fn collector_does_not_mega_debuff_immediately_after_spawn_like_java() {
        let mut enemy = create_enemy("TheCollector", 282, 282);
        enemy.entity.set_status(sid::FIRST_MOVE, 0);
        enemy.entity.set_status(sid::TURN_COUNT, 1);
        roll_next_move_with_num(&mut enemy, 70);
        assert_eq!(enemy.move_id, move_ids::COLL_FIREBALL);
    }

    #[test]
    fn collector_stats_spawn_revive_debuff_buff_and_death_match_java() {
        // Source: reference/extracted/methods/monster/TheCollector.java and
        // TorchHead.java. Thresholds are A4/A9/A19, not inferred from HP.
        for (ascension, hp, fireball, strength, block, mega) in [
            (0, 282, 18, 3, 15, 3),
            (4, 282, 21, 4, 15, 3),
            (9, 300, 21, 4, 18, 3),
            (19, 300, 21, 5, 23, 5),
        ] {
            let mut run = RunEngine::new(60, ascension);
            run.debug_enter_specific_combat(&["TheCollector"]);
            let combat = run.get_combat_engine().expect("Collector combat");
            let collector = &combat.state.enemies[0];
            assert_eq!((collector.entity.hp, collector.entity.max_hp), (hp, hp));
            assert_eq!(collector.entity.status(sid::FIREBALL_DMG), fireball);
            assert_eq!(collector.entity.status(sid::STR_AMT), strength);
            assert_eq!(collector.entity.status(sid::BLOCK_AMT), block);
            assert_eq!(collector.entity.status(sid::STARTING_DMG), mega);
            assert_eq!(collector.move_id, move_ids::COLL_SPAWN);
            assert_eq!(combat.ai_rng.counter, 1,
                "initialSpawn still consumes AbstractMonster.rollMove's integer");
        }

        let mut run = RunEngine::new(61, 0);
        run.debug_enter_specific_combat(&["TheCollector"]);
        let combat = run.debug_combat_engine_mut();
        let ai_before = combat.ai_rng.counter;
        do_enemy_turns(combat);
        assert_eq!(combat.state.enemies.len(), 3);
        assert_eq!(combat.ai_rng.counter - ai_before, 3,
            "two Torch Head init rolls, then Collector's RollMove");
        assert_eq!(combat.state.enemies[0].entity.status(sid::FIRST_MOVE), 0);
        assert_eq!(combat.state.enemies[0].entity.status(sid::TURN_COUNT), 1);
        for torch in &combat.state.enemies[1..] {
            assert_eq!(torch.id, "TorchHead");
            assert!((38..=40).contains(&torch.entity.hp));
            assert!(torch.is_minion);
            assert_eq!(torch.move_id, move_ids::TORCH_TACKLE);
            assert_eq!(torch.move_damage(), 7);
        }

        combat.state.enemies[0].set_move(move_ids::COLL_BUFF, 0, 0, 15);
        combat.state.enemies[0].intent = crate::combat_types::Intent::DefendBuff {
            block: 15, effects: 0,
        };
        combat.state.enemies[0].add_effect(mfx::STRENGTH, 3);
        combat.state.enemies[0].add_effect(mfx::STRENGTH_ALL_ALLIES, 3);
        combat.state.enemies[1].move_id = -1;
        combat.state.enemies[2].move_id = -1;
        do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.block, 15);
        assert!(combat.state.enemies.iter().all(|enemy| enemy.entity.strength() == 3));

        assert!(combat.instant_kill_enemy(1));
        combat.state.enemies[0].set_move(move_ids::COLL_REVIVE, 0, 0, 0);
        combat.state.enemies[0].intent = crate::combat_types::Intent::Unknown;
        combat.state.enemies[2].move_id = -1;
        let ai_before = combat.ai_rng.counter;
        do_enemy_turns(combat);
        assert!(combat.state.enemies[1].is_alive());
        assert!((38..=40).contains(&combat.state.enemies[1].entity.hp));
        assert!(combat.state.enemies[1].is_minion);
        assert_eq!(combat.ai_rng.counter - ai_before, 2,
            "revived Torch Head init plus Collector RollMove");

        let mut high = RunEngine::new(62, 19);
        high.debug_enter_specific_combat(&["TheCollector"]);
        let combat = high.debug_combat_engine_mut();
        combat.state.player.set_status(sid::ARTIFACT, 1);
        combat.state.enemies[0].set_move(move_ids::COLL_MEGA_DEBUFF, 0, 0, 0);
        combat.state.enemies[0].intent = crate::combat_types::Intent::Debuff { effects: 0 };
        combat.state.enemies[0].add_effect(mfx::WEAK, 5);
        combat.state.enemies[0].add_effect(mfx::VULNERABLE, 5);
        combat.state.enemies[0].add_effect(mfx::FRAIL, 5);
        do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(sid::ARTIFACT), 0);
        assert_eq!(combat.state.player.status(sid::WEAKENED), 0);
        assert_eq!(combat.state.player.status(sid::VULNERABLE), 5);
        assert_eq!(combat.state.player.status(sid::FRAIL), 5);
        assert_eq!(combat.state.enemies[0].entity.status(sid::USED_MEGA_DEBUFF), 1);

        // Collector.die queues SuicideAction for every surviving minion.
        let mut death = RunEngine::new(63, 0);
        death.debug_enter_specific_combat(&["TheCollector"]);
        let combat = death.debug_combat_engine_mut();
        do_enemy_turns(combat);
        assert!(combat.instant_kill_enemy(0));
        assert!(combat.state.enemies.iter().all(|enemy| !enemy.is_alive()));
    }

    #[test]
    fn champ_base_hp_and_opening_move() {
        let enemy = create_enemy("Champ", 420, 420);
        assert_eq!(enemy.entity.hp, 420);
        assert_eq!(enemy.move_id, move_ids::CHAMP_FACE_SLAP);
        assert_eq!(enemy.move_damage(), 12);
        assert_eq!(enemy.effect(mfx::FRAIL), Some(2));
        assert_eq!(enemy.effect(mfx::VULNERABLE), Some(2));
        assert_eq!(enemy.entity.status(sid::STR_AMT), 2);
        assert_eq!(enemy.entity.status(sid::FORGE_AMT), 5);
        assert_eq!(enemy.entity.status(sid::BLOCK_AMT), 15);
    }

    #[test]
    fn champ_turn_four_uses_java_taunt_branch() {
        // Source: reference/extracted/methods/monster/Champ.java (`getMove`).
        let mut enemy = create_enemy("Champ", 420, 420);

        roll_initial_move_with_num_and_rng(
            &mut enemy, 99, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::CHAMP_HEAVY_SLASH);

        roll_next_move_with_num(&mut enemy, 99);
        assert_eq!(enemy.move_id, move_ids::CHAMP_FACE_SLAP);

        roll_next_move_with_num(&mut enemy, 99);
        assert_eq!(enemy.move_id, move_ids::CHAMP_HEAVY_SLASH);

        roll_next_move_with_num(&mut enemy, 99);
        assert_eq!(enemy.move_id, move_ids::CHAMP_TAUNT);
        assert_eq!(enemy.entity.status(sid::NUM_TURNS), 0);
    }

    #[test]
    fn champ_a4_and_a19_scaling_matches_java_expectations() {
        // Source: reference/extracted/methods/monster/Champ.java (constructor).
        for (ascension, hp, slash, slap, strength, forge, block) in [
            (0, 420, 16, 12, 2, 5, 15),
            (4, 420, 18, 14, 3, 5, 15),
            (9, 440, 18, 14, 3, 6, 18),
            (19, 440, 18, 14, 4, 7, 20),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.debug_enter_specific_combat(&["TheChamp"]);
            let enemy = &run.get_combat_engine().unwrap().state.enemies[0];
            assert_eq!(enemy.entity.hp, hp, "A{ascension}");
            assert_eq!(enemy.entity.status(sid::SLASH_DMG), slash, "A{ascension}");
            assert_eq!(enemy.entity.status(sid::SLAP_DMG), slap, "A{ascension}");
            assert_eq!(enemy.entity.status(sid::STR_AMT), strength, "A{ascension}");
            assert_eq!(enemy.entity.status(sid::FORGE_AMT), forge, "A{ascension}");
            assert_eq!(enemy.entity.status(sid::BLOCK_AMT), block, "A{ascension}");
        }
    }

    // ---------------------------------------------------------------------
    // Act 3 bosses
    // ---------------------------------------------------------------------

    #[test]
    fn awakened_one_base_hp_and_p1_setup() {
        let enemy = create_enemy("AwakenedOne", 300, 300);
        assert_eq!(enemy.entity.hp, 300);
        assert_eq!(enemy.entity.max_hp, 300);
        assert_eq!(enemy.move_id, move_ids::AO_SLASH);
        assert_eq!(enemy.move_damage(), 20);
        assert_eq!(enemy.entity.status(sid::CURIOSITY), 1);
        assert_eq!(enemy.entity.status(sid::PHASE), 1);
        assert_eq!(enemy.entity.status(sid::REGENERATION), 10);
    }

    #[test]
    fn awakened_one_run_spawn_uses_independent_a4_a9_and_a19_thresholds() {
        // AwakenedOne's constructor changes HP at A9; usePreBattleAction adds
        // 2 Strength at A4 and changes Curiosity/Regenerate only at A19.
        // MonsterHelper constructs both Cultists before Awakened One.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/MonsterHelper.java
        // Java: reference/extracted/methods/monster/AwakenedOne.java
        for (ascension, hp, strength, curiosity, regen) in [
            (0, 300, 0, 1, 10),
            (4, 300, 2, 1, 10),
            (9, 320, 2, 1, 10),
            (19, 320, 2, 2, 15),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.debug_enter_specific_combat(&["AwakenedOne"]);
            let combat = run.get_combat_engine().expect("Awakened One combat");
            assert_eq!(
                combat.state.enemies.iter().map(|enemy| enemy.id.as_str()).collect::<Vec<_>>(),
                ["Cultist", "Cultist", "AwakenedOne"],
                "A{ascension} must preserve MonsterHelper's construction order"
            );
            let enemy = &combat.state.enemies[2];
            assert_eq!(enemy.entity.hp, hp, "A{ascension}");
            assert_eq!(enemy.entity.max_hp, hp, "A{ascension}");
            assert_eq!(enemy.entity.status(sid::STRENGTH), strength, "A{ascension}");
            assert_eq!(enemy.entity.status(sid::CURIOSITY), curiosity, "A{ascension}");
            assert_eq!(enemy.entity.status(sid::REGENERATION), regen, "A{ascension}");
            assert_eq!(combat.rng_counters()["ai"], 3, "three opening rolls at A{ascension}");
            assert_eq!(enemy.move_id, move_ids::AO_SLASH);
        }
    }

    #[test]
    fn awakened_one_phase_two_rebirth_matches_java() {
        let mut engine = boss_engine("AwakenedOne", 300, 300);
        engine.state.enemies[0].entity.set_status(sid::VULNERABLE, 2);
        engine.state.enemies[0].entity.set_status(sid::STRENGTH, -3);
        engine.state.enemies[0].entity.set_status(sid::TEMP_STRENGTH_LOSS, 3);
        engine.deal_damage_to_enemy(0, 300);

        assert_eq!(engine.state.enemies[0].entity.status(sid::REBIRTH_PENDING), 1);
        assert_eq!(engine.state.enemies[0].entity.hp, 0);
        assert_eq!(engine.state.enemies[0].move_id, move_ids::AO_REBIRTH);
        assert_eq!(engine.state.enemies[0].entity.status(sid::CURIOSITY), 0);
        assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 0);
        assert_eq!(engine.state.enemies[0].entity.status(sid::STRENGTH), 0);
        assert_eq!(engine.state.enemies[0].entity.status(sid::TEMP_STRENGTH_LOSS), 0);
        let ai_before = engine.rng_counters()["ai"];

        do_enemy_turns(&mut engine);

        assert_eq!(engine.state.enemies[0].entity.status(sid::PHASE), 2);
        assert_eq!(engine.state.enemies[0].entity.hp, 300);
        assert_eq!(engine.state.enemies[0].move_id, move_ids::AO_DARK_ECHO);
        assert_eq!(engine.state.enemies[0].move_damage(), 40);
        assert!(engine.state.enemies[0].move_history.is_empty());
        assert_eq!(engine.rng_counters()["ai"], ai_before + 1);
    }

    #[test]
    fn awakened_one_sludge_randomly_inserts_void_then_regenerates_after_acting() {
        // Sludge queues 18 damage before a random-spot Void. After every
        // monster has acted, RegenerateMonsterPower heals its fixed amount.
        // Java: reference/extracted/methods/monster/AwakenedOne.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/RegenerateMonsterPower.java
        let mut engine = boss_engine("AwakenedOne", 250, 300);
        engine.state.draw_pile = make_deck(&["Strike", "Defend", "Bash"]);
        engine.state.enemies[0].entity.set_status(sid::PHASE, 2);
        engine.state.enemies[0].set_move(move_ids::AO_SLUDGE, 18, 1, 0);
        engine.state.enemies[0].add_effect(mfx::VOID, 1);
        let mut oracle = engine.card_random_rng.clone();
        let expected_idx = oracle.random_int_range(0, 2) as usize;

        do_enemy_turns(&mut engine);

        assert_eq!(engine.state.player.hp, 62);
        assert_eq!(engine.state.enemies[0].entity.hp, 260);
        assert_eq!(engine.card_random_rng.counter, oracle.counter);
        assert_eq!(
            engine.card_registry.card_name(engine.state.draw_pile[expected_idx].def_id),
            "Void"
        );
        assert_eq!(
            engine.card_registry.card_name(engine.state.draw_pile.last().unwrap().def_id),
            "Bash"
        );
    }

    #[test]
    fn donu_base_hp_and_cycle_matches_java() {
        let mut enemy = create_enemy("Donu", 250, 250);
        assert_eq!(enemy.entity.hp, 250);
        assert_eq!(enemy.entity.status(sid::ARTIFACT), 2);
        assert_eq!(enemy.move_id, move_ids::DONU_CIRCLE);
        assert_eq!(enemy.effect(mfx::STRENGTH), Some(3));

        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::DONU_BEAM);
        assert_eq!(enemy.move_damage(), 10);
        assert_eq!(enemy.move_hits(), 2);

        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::DONU_CIRCLE);
    }

    #[test]
    fn donu_constructor_defaults_do_not_infer_ascension_from_hp() {
        // Source: reference/extracted/methods/monster/Donu.java. create_enemy
        // has no ascension input; run.rs tests the independent thresholds.
        let enemy = create_enemy("Donu", 265, 265);
        assert_eq!(enemy.entity.hp, 265);
        assert_eq!(enemy.entity.max_hp, 265);
        assert_eq!(enemy.entity.status(sid::ARTIFACT), 2);
        assert_eq!(enemy.entity.status(sid::BEAM_DMG), 10);
    }

    #[test]
    fn deca_base_hp_and_cycle_matches_java() {
        let mut enemy = create_enemy("Deca", 250, 250);
        assert_eq!(enemy.entity.hp, 250);
        assert_eq!(enemy.move_id, move_ids::DECA_BEAM);
        assert_eq!(enemy.move_damage(), 10);
        assert_eq!(enemy.effect(mfx::DAZE), Some(2));
        assert_eq!(enemy.entity.status(sid::ARTIFACT), 2);

        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::DECA_SQUARE);
        assert_eq!(enemy.move_block(), 16);
    }

    #[test]
    fn deca_constructor_defaults_do_not_infer_ascension_from_hp() {
        // Source: reference/extracted/methods/monster/Deca.java. create_enemy
        // has no ascension input; run.rs tests the independent thresholds.
        let enemy = create_enemy("Deca", 265, 265);
        assert_eq!(enemy.entity.hp, 265);
        assert_eq!(enemy.entity.max_hp, 265);
        assert_eq!(enemy.entity.status(sid::ARTIFACT), 2);
        assert_eq!(enemy.move_damage(), 10);
    }

    #[test]
    fn donu_and_deca_run_group_preserves_java_deca_then_donu_order() {
        // MonsterHelper.java constructs `new Deca()` before `new Donu()`.
        // This order controls both target indices and opening aiRng draws.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/MonsterHelper.java
        let mut run = RunEngine::new(42, 0);
        run.debug_enter_specific_combat(&["DonuAndDeca"]);
        let combat = run.get_combat_engine().expect("Donu and Deca combat");

        assert_eq!(
            combat.state.enemies.iter().map(|enemy| enemy.id.as_str()).collect::<Vec<_>>(),
            ["Deca", "Donu"]
        );
        assert_eq!(combat.state.enemies[0].move_id, move_ids::DECA_BEAM);
        assert_eq!(combat.state.enemies[1].move_id, move_ids::DONU_CIRCLE);
        assert_eq!(combat.rng_counters()["ai"], 2);
    }

    #[test]
    fn time_eater_base_hp_and_opening_move() {
        let enemy = create_enemy("TimeEater", 456, 456);
        assert_eq!(enemy.entity.hp, 456);
        assert_eq!(enemy.entity.max_hp, 456);
        assert_eq!(enemy.move_id, move_ids::TE_REVERBERATE);
        assert_eq!(enemy.move_damage(), 7);
        assert_eq!(enemy.move_hits(), 3);
    }

    #[test]
    fn time_eater_a9_and_a4_scaling_matches_java_expectations() {
        // create_enemy has no ascension input; HP must not imply A4 damage.
        let enemy = create_enemy("TimeEater", 480, 480);
        assert_eq!(enemy.entity.hp, 480);
        assert_eq!(enemy.entity.max_hp, 480);
        assert_eq!(enemy.move_damage(), 7);
        assert_eq!(enemy.move_hits(), 3);
    }

    #[test]
    fn time_eater_ai_stats_haste_and_a19_payloads_match_java() {
        // Source: reference/extracted/methods/monster/TimeEater.java.
        for (ascension, hp, reverb, head, high_ai) in [
            (0, 456, 7, 26, 0),
            (4, 456, 8, 32, 0),
            (9, 480, 8, 32, 0),
            (19, 480, 8, 32, 1),
        ] {
            let mut run = RunEngine::new(70, ascension);
            run.debug_enter_specific_combat(&["TimeEater"]);
            let combat = run.get_combat_engine().expect("Time Eater combat");
            let enemy = &combat.state.enemies[0];
            assert_eq!((enemy.entity.hp, enemy.entity.max_hp), (hp, hp));
            assert_eq!(enemy.entity.status(sid::REVERB_DMG), reverb);
            assert_eq!(enemy.entity.status(sid::HEAD_SLAM_DMG), head);
            assert_eq!(enemy.entity.status(sid::HIGH_ASCENSION_AI), high_ai);
            assert!(matches!(enemy.move_id,
                move_ids::TE_REVERBERATE | move_ids::TE_HEAD_SLAM | move_ids::TE_RIPPLE));
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut enemy = create_enemy("TimeEater", 456, 456);
        enemy.entity.hp = 228;
        roll_initial_move_with_num_and_rng(
            &mut enemy, 0, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::TE_REVERBERATE,
            "exactly half HP does not trigger Haste");
        enemy.entity.hp = 227;
        enemy.entity.set_status(sid::USED_HASTE, 0);
        roll_initial_move_with_num_and_rng(
            &mut enemy, 99, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::TE_HASTE);
        assert_eq!(enemy.effect(mfx::REMOVE_DEBUFFS), Some(1));
        assert_eq!(enemy.effect(mfx::HEAL_TO_HALF), Some(1));
        assert_eq!(enemy.entity.status(sid::USED_HASTE), 1);

        // Haste does not force a follow-up: normal num branches resume.
        enemy.move_id = move_ids::TE_HASTE;
        enemy.move_history.clear();
        roll_next_move_with_num(&mut enemy, 0);
        assert_eq!(enemy.move_id, move_ids::TE_REVERBERATE);

        // Two Reverberates recursively reroll 50..99 and consume one ai tick.
        enemy.move_id = move_ids::TE_REVERBERATE;
        enemy.move_history = vec![move_ids::TE_REVERBERATE];
        let mut rng = crate::seed::StsRandom::new(0);
        roll_next_move_with_num_and_rng(&mut enemy, 0, &mut rng);
        assert!(matches!(enemy.move_id, move_ids::TE_HEAD_SLAM | move_ids::TE_RIPPLE));
        assert_eq!(rng.counter, 1);

        // Repeated Head Slam consumes randomBoolean(0.66), selecting either
        // Reverberate or Ripple; repeated Ripple recursively random(74).
        enemy.move_id = move_ids::TE_HEAD_SLAM;
        enemy.move_history.clear();
        let mut rng = crate::seed::StsRandom::new(1);
        roll_next_move_with_num_and_rng(&mut enemy, 45, &mut rng);
        assert!(matches!(enemy.move_id, move_ids::TE_REVERBERATE | move_ids::TE_RIPPLE));
        assert_eq!(rng.counter, 1);
        enemy.move_id = move_ids::TE_RIPPLE;
        enemy.move_history.clear();
        let mut rng = crate::seed::StsRandom::new(2);
        roll_next_move_with_num_and_rng(&mut enemy, 80, &mut rng);
        assert!(matches!(enemy.move_id, move_ids::TE_REVERBERATE | move_ids::TE_HEAD_SLAM));
        assert_eq!(rng.counter, 1);

        let mut high = RunEngine::new(71, 19);
        high.debug_enter_specific_combat(&["TimeEater"]);
        let combat = high.debug_combat_engine_mut();
        combat.state.enemies[0].entity.hp = 200;
        combat.state.enemies[0].entity.set_status(sid::VULNERABLE, 2);
        combat.state.enemies[0].entity.set_status(sid::POISON, 5);
        combat.state.enemies[0].set_move(move_ids::TE_HASTE, 0, 0, 0);
        combat.state.enemies[0].add_effect(mfx::REMOVE_DEBUFFS, 1);
        combat.state.enemies[0].add_effect(mfx::HEAL_TO_HALF, 1);
        do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.hp, 240);
        assert_eq!(combat.state.enemies[0].entity.block, 32);
        assert_eq!(combat.state.enemies[0].entity.status(sid::VULNERABLE), 0);
        assert_eq!(combat.state.enemies[0].entity.status(sid::POISON), 0);

        // Ripple applies Vulnerable, then Weak, then A19 Frail. Artifact blocks
        // only the first. Head Slam's Draw Reduction is also Artifact-aware,
        // while its two Slimed cards still enter discard.
        let mut ripple = RunEngine::new(72, 19);
        ripple.debug_enter_specific_combat(&["TimeEater"]);
        let combat = ripple.debug_combat_engine_mut();
        combat.state.player.set_status(sid::ARTIFACT, 1);
        combat.state.enemies[0].set_move(move_ids::TE_RIPPLE, 0, 0, 20);
        combat.state.enemies[0].add_effect(mfx::VULNERABLE, 1);
        combat.state.enemies[0].add_effect(mfx::WEAK, 1);
        combat.state.enemies[0].add_effect(mfx::FRAIL, 1);
        do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(sid::VULNERABLE), 0);
        assert_eq!(combat.state.player.status(sid::WEAKENED), 1);
        assert_eq!(combat.state.player.status(sid::FRAIL), 1);

        let mut slam = RunEngine::new(73, 19);
        slam.debug_enter_specific_combat(&["TimeEater"]);
        let combat = slam.debug_combat_engine_mut();
        combat.state.player.set_status(sid::ARTIFACT, 1);
        combat.state.enemies[0].set_move(move_ids::TE_HEAD_SLAM, 32, 1, 0);
        combat.state.enemies[0].intent = crate::combat_types::Intent::AttackDebuff {
            damage: 32, hits: 1, effects: 0,
        };
        combat.state.enemies[0].add_effect(mfx::DRAW_REDUCTION, 1);
        combat.state.enemies[0].add_effect(mfx::SLIMED, 2);
        do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(sid::ARTIFACT), 0);
        assert_eq!(combat.state.player.status(sid::DRAW_REDUCTION), 0);
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Slimed").count(), 2);

        let mut lifetime = RunEngine::new(74, 0);
        lifetime.debug_enter_specific_combat(&["TimeEater"]);
        let combat = lifetime.debug_combat_engine_mut();
        combat.state.enemies[0].set_move(move_ids::TE_HEAD_SLAM, 26, 1, 0);
        combat.state.enemies[0].intent = crate::combat_types::Intent::AttackDebuff {
            damage: 26, hits: 1, effects: 0,
        };
        combat.state.enemies[0].add_effect(mfx::DRAW_REDUCTION, 1);
        do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(sid::DRAW_REDUCTION), 1);
        assert_eq!(combat.state.player.status(sid::DRAW_REDUCTION_JUST_APPLIED), 1);
        crate::powers::decrement_debuffs(&mut combat.state.player);
        assert_eq!(combat.state.player.status(sid::DRAW_REDUCTION_JUST_APPLIED), 0);
        assert_eq!(combat.state.player.status(sid::DRAW_REDUCTION), 1);
        combat.state.enemies[0].set_move(move_ids::TE_REVERBERATE, 7, 3, 0);
        do_enemy_turns(combat);
        crate::powers::decrement_debuffs(&mut combat.state.player);
        assert_eq!(combat.state.player.status(sid::DRAW_REDUCTION), 0);
    }

    // ---------------------------------------------------------------------
    // Act 4 boss
    // ---------------------------------------------------------------------

    #[test]
    fn corrupt_heart_base_hp_and_debilitate_matches_java() {
        let enemy = create_enemy("CorruptHeart", 750, 750);
        assert_eq!(enemy.entity.hp, 750);
        assert_eq!(enemy.entity.max_hp, 750);
        assert_eq!(enemy.move_id, move_ids::HEART_DEBILITATE);
        assert_eq!(enemy.effect(mfx::VULNERABLE), Some(2));
        assert_eq!(enemy.effect(mfx::WEAK), Some(2));
        assert_eq!(enemy.effect(mfx::FRAIL), Some(2));
        assert_eq!(enemy.entity.status(sid::INVINCIBLE), 300);
        assert_eq!(enemy.entity.status(sid::BEAT_OF_DEATH), 1);
        assert_eq!(enemy.entity.status(sid::BLOOD_HIT_COUNT), 12);
        assert_eq!(enemy.entity.status(sid::ECHO_DMG), 40);
    }

    #[test]
    fn corrupt_heart_constructor_defaults_do_not_infer_ascension_from_hp() {
        // Source: reference/extracted/methods/monster/CorruptHeart.java.
        // create_enemy has no ascension input; the run spawn site applies the
        // independent A4/A9/A19 thresholds tested in run.rs.
        let enemy = create_enemy("CorruptHeart", 800, 800);
        assert_eq!(enemy.entity.hp, 800);
        assert_eq!(enemy.entity.max_hp, 800);
        assert_eq!(enemy.entity.status(sid::INVINCIBLE), 300);
        assert_eq!(enemy.entity.status(sid::BEAT_OF_DEATH), 1);
        assert_eq!(enemy.entity.status(sid::BLOOD_HIT_COUNT), 12);
        assert_eq!(enemy.entity.status(sid::ECHO_DMG), 40);
    }

    #[test]
    fn corrupt_heart_buff_cycle_matches_java() {
        let mut enemy = create_enemy("CorruptHeart", 750, 750);
        let mut rng = crate::seed::StsRandom::new(0);

        roll_next_move(&mut enemy, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEART_DEBILITATE);

        roll_next_move(&mut enemy, &mut rng);
        let first_attack = enemy.move_id;
        roll_next_move(&mut enemy, &mut rng);
        assert_ne!(enemy.move_id, first_attack);

        roll_next_move(&mut enemy, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEART_BUFF);
        assert_eq!(enemy.entity.status(sid::BUFF_COUNT), 0,
            "CorruptHeart.takeTurn, not getMove, increments buffCount");
    }
}
