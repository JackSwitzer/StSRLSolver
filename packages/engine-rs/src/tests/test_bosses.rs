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
    use crate::engine::CombatEngine;
    use crate::enemies::*;
    use crate::enemies::move_ids;
    use crate::tests::support::*;

    fn roll_times(enemy: &mut crate::state::EnemyCombatState, turns: usize) {
        for _ in 0..turns {
            roll_next_move(enemy);
        }
    }

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
        assert_eq!(enemy.move_block, 9);
        assert_eq!(enemy.entity.status("ModeShift"), 30);
    }

    #[test]
    fn guardian_a2_hp_and_bash_damage_matches_java() {
        let mut enemy = create_enemy("TheGuardian", 250, 250);
        assert_eq!(enemy.entity.hp, 250);
        assert_eq!(enemy.entity.max_hp, 250);
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_damage, 36);
    }

    #[test]
    fn guardian_defensive_mode_uses_java_threshold_and_sharp_hide() {
        let mut enemy = create_enemy("TheGuardian", 240, 240);
        enemy.entity.set_status("ModeShift", 40);

        let shifted = guardian_check_mode_shift(&mut enemy, 40);
        assert!(shifted);
        assert_eq!(enemy.entity.status("SharpHide"), 4);
        assert_eq!(enemy.entity.status("ModeShift"), 50);
    }

    #[test]
    fn guardian_switch_back_to_offensive_matches_java() {
        let mut enemy = create_enemy("TheGuardian", 240, 240);
        guardian_switch_to_offensive(&mut enemy);
        assert_eq!(enemy.entity.status("SharpHide"), 0);
        assert_eq!(enemy.move_id, move_ids::GUARD_CHARGING_UP);
        assert_eq!(enemy.move_block, 9);
    }

    #[test]
    fn guardian_offensive_cycle_matches_java_base_values() {
        let mut enemy = create_enemy("TheGuardian", 240, 240);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_FIERCE_BASH);
        assert_eq!(enemy.move_damage, 32);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_VENT_STEAM);
        assert_eq!(enemy.move_effects.get("weak"), Some(&2));
        assert_eq!(enemy.move_effects.get("vulnerable"), Some(&2));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_WHIRLWIND);
        assert_eq!(enemy.move_damage, 5);
        assert_eq!(enemy.move_hits, 4);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_CHARGING_UP);
        assert_eq!(enemy.move_block, 9);
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
        assert_eq!(enemy.move_damage, 7);
        assert_eq!(enemy.move_hits, 6);

        hexaghost_set_divider(&mut enemy, 60);
        assert_eq!(enemy.move_damage, 6);
        assert_eq!(enemy.move_hits, 6);
    }

    #[test]
    fn hexaghost_base_cycle_matches_java_shape() {
        let mut enemy = create_enemy("Hexaghost", 250, 250);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_DIVIDER);
        assert_eq!(enemy.move_damage, 7);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_SEAR);
        assert_eq!(enemy.move_damage, 6);
        assert_eq!(enemy.move_effects.get("burn"), Some(&1));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_TACKLE);
        assert_eq!(enemy.move_damage, 5);
        assert_eq!(enemy.move_hits, 2);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_SEAR);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_INFLAME);
        assert_eq!(enemy.move_block, 12);
        assert_eq!(enemy.move_effects.get("strength"), Some(&2));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_TACKLE);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_SEAR);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_INFERNO);
        assert_eq!(enemy.move_damage, 2);
        assert_eq!(enemy.move_hits, 6);
    }

    #[test]
    fn hexaghost_a4_scaling_matches_java_expectations() {
        let mut enemy = create_enemy("Hexaghost", 264, 264);

        // Activate -> Divider -> Sear(orb=0) -> Tackle(orb=1)
        roll_next_move(&mut enemy);
        roll_next_move(&mut enemy);
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_TACKLE);
        assert_eq!(enemy.move_damage, 6);
        assert_eq!(enemy.move_hits, 2);

        // Sear(2) -> Inflame(3) -> Tackle(4) -> Sear(5) -> Inferno(6)
        roll_times(&mut enemy, 5);
        assert_eq!(enemy.move_id, move_ids::HEX_INFERNO);
        assert_eq!(enemy.move_damage, 3);
        assert_eq!(enemy.move_hits, 6);
    }

    #[test]
    fn hexaghost_a19_burn_and_strength_matches_java_expectations() {
        let mut enemy = create_enemy("Hexaghost", 264, 264);

        roll_next_move(&mut enemy);
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_effects.get("burn"), Some(&2));

        // Sear(0) -> Tackle(1) -> Sear(2) -> Inflame(3)
        roll_times(&mut enemy, 3);
        assert_eq!(enemy.move_id, move_ids::HEX_INFLAME);
        assert_eq!(enemy.move_effects.get("strength"), Some(&3));
    }

    #[test]
    fn slime_boss_base_hp_and_opening_move() {
        let enemy = create_enemy("SlimeBoss", 140, 140);
        assert_eq!(enemy.entity.hp, 140);
        assert_eq!(enemy.entity.max_hp, 140);
        assert_eq!(enemy.move_id, move_ids::SB_STICKY);
        assert_eq!(enemy.move_effects.get("slimed"), Some(&3));
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

        assert_eq!(engine.state.enemies[0].entity.hp, 0);
        assert_eq!(engine.state.enemies.len(), 3);
        assert_eq!(engine.state.enemies[1].id, "AcidSlime_L");
        assert_eq!(engine.state.enemies[2].id, "SpikeSlime_L");
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
    fn bronze_automaton_a2_scaling_matches_java_expectations() {
        let mut enemy = create_enemy("BronzeAutomaton", 320, 320);
        assert_eq!(enemy.entity.hp, 320);
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_damage, 8);
        assert_eq!(enemy.entity.status("Artifact"), 3);
    }

    #[test]
    fn bronze_automaton_cycle_matches_java_base_pattern() {
        let mut enemy = create_enemy("BronzeAutomaton", 300, 300);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::BA_FLAIL);
        assert_eq!(enemy.move_damage, 7);
        assert_eq!(enemy.move_hits, 2);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::BA_BOOST);
        assert_eq!(enemy.move_effects.get("strength"), Some(&3));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::BA_FLAIL);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::BA_HYPER_BEAM);
        assert_eq!(enemy.move_damage, 45);
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

        roll_next_move(&mut enemy);
        assert_ne!(enemy.move_id, move_ids::COLL_MEGA_DEBUFF);
    }

    #[test]
    fn collector_a2_scaling_matches_java_expectations() {
        let mut enemy = create_enemy("TheCollector", 300, 300);
        roll_next_move(&mut enemy);
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::COLL_FIREBALL);
        assert_eq!(enemy.move_damage, 21);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::COLL_BUFF);
        assert_eq!(enemy.move_block, 18);
        assert_eq!(enemy.move_effects.get("strength"), Some(&4));
    }

    #[test]
    fn champ_base_hp_and_opening_move() {
        let enemy = create_enemy("Champ", 420, 420);
        assert_eq!(enemy.entity.hp, 420);
        assert_eq!(enemy.move_id, move_ids::CHAMP_FACE_SLAP);
        assert_eq!(enemy.move_damage, 12);
        assert_eq!(enemy.move_effects.get("frail"), Some(&2));
        assert_eq!(enemy.move_effects.get("vulnerable"), Some(&2));
        assert_eq!(enemy.entity.status("StrAmt"), 2);
        assert_eq!(enemy.entity.status("ForgeAmt"), 5);
        assert_eq!(enemy.entity.status("BlockAmt"), 15);
    }

    #[test]
    fn champ_turn_four_uses_java_taunt_branch() {
        let mut enemy = create_enemy("Champ", 420, 420);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CHAMP_HEAVY_SLASH);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CHAMP_GLOAT);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CHAMP_FACE_SLAP);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CHAMP_TAUNT);
        assert_eq!(enemy.entity.status("NumTurns"), 0);
    }

    #[test]
    fn champ_a4_and_a19_scaling_matches_java_expectations() {
        let enemy = create_enemy("Champ", 440, 440);
        assert_eq!(enemy.entity.hp, 440);
        assert_eq!(enemy.move_damage, 14);
        assert_eq!(enemy.entity.status("StrAmt"), 4);
        assert_eq!(enemy.entity.status("ForgeAmt"), 7);
        assert_eq!(enemy.entity.status("BlockAmt"), 20);
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
        assert_eq!(enemy.move_damage, 20);
        assert_eq!(enemy.entity.status("Curiosity"), 1);
        assert_eq!(enemy.entity.status("Phase"), 1);
        assert_eq!(enemy.entity.status("Regenerate"), 10);
    }

    #[test]
    fn awakened_one_a9_and_a4_scaling_matches_java_expectations() {
        let enemy = create_enemy("AwakenedOne", 320, 320);
        assert_eq!(enemy.entity.hp, 320);
        assert_eq!(enemy.entity.max_hp, 320);
        assert_eq!(enemy.entity.status("Curiosity"), 2);
        assert_eq!(enemy.entity.status("Regenerate"), 15);
        assert_eq!(enemy.entity.status("Strength"), 2);
    }

    #[test]
    fn awakened_one_phase_two_rebirth_matches_java() {
        let mut engine = boss_engine("AwakenedOne", 300, 300);
        engine.deal_damage_to_enemy(0, 300);

        assert_eq!(engine.state.enemies[0].entity.status("RebirthPending"), 1);
        assert_eq!(engine.state.enemies[0].entity.hp, 0);

        do_enemy_turns(&mut engine);

        assert_eq!(engine.state.enemies[0].entity.status("Phase"), 2);
        assert_eq!(engine.state.enemies[0].entity.hp, 300);
        assert_eq!(engine.state.enemies[0].move_id, move_ids::AO_DARK_ECHO);
        assert_eq!(engine.state.enemies[0].move_damage, 40);
        assert!(engine.state.enemies[0].move_history.is_empty());
    }

    #[test]
    fn donu_base_hp_and_cycle_matches_java() {
        let mut enemy = create_enemy("Donu", 250, 250);
        assert_eq!(enemy.entity.hp, 250);
        assert_eq!(enemy.entity.status("Artifact"), 2);
        assert_eq!(enemy.move_id, move_ids::DONU_CIRCLE);
        assert_eq!(enemy.move_effects.get("strength"), Some(&3));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::DONU_BEAM);
        assert_eq!(enemy.move_damage, 10);
        assert_eq!(enemy.move_hits, 2);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::DONU_CIRCLE);
    }

    #[test]
    fn donu_a2_and_a19_scaling_matches_java_expectations() {
        let mut enemy = create_enemy("Donu", 265, 265);
        assert_eq!(enemy.entity.hp, 265);
        assert_eq!(enemy.entity.max_hp, 265);
        assert_eq!(enemy.entity.status("Artifact"), 3);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::DONU_BEAM);
        assert_eq!(enemy.move_damage, 12);
    }

    #[test]
    fn deca_base_hp_and_cycle_matches_java() {
        let mut enemy = create_enemy("Deca", 250, 250);
        assert_eq!(enemy.entity.hp, 250);
        assert_eq!(enemy.move_id, move_ids::DECA_BEAM);
        assert_eq!(enemy.move_damage, 10);
        assert_eq!(enemy.move_effects.get("daze"), Some(&2));
        assert_eq!(enemy.entity.status("Artifact"), 2);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::DECA_SQUARE);
        assert_eq!(enemy.move_block, 16);
    }

    #[test]
    fn deca_a2_and_a19_scaling_matches_java_expectations() {
        let enemy = create_enemy("Deca", 265, 265);
        assert_eq!(enemy.entity.hp, 265);
        assert_eq!(enemy.entity.max_hp, 265);
        assert_eq!(enemy.entity.status("Artifact"), 3);
        assert_eq!(enemy.move_damage, 12);
    }

    #[test]
    fn time_eater_base_hp_and_opening_move() {
        let enemy = create_enemy("TimeEater", 456, 456);
        assert_eq!(enemy.entity.hp, 456);
        assert_eq!(enemy.entity.max_hp, 456);
        assert_eq!(enemy.move_id, move_ids::TE_REVERBERATE);
        assert_eq!(enemy.move_damage, 7);
        assert_eq!(enemy.move_hits, 3);
    }

    #[test]
    fn time_eater_a9_and_a4_scaling_matches_java_expectations() {
        let enemy = create_enemy("TimeEater", 480, 480);
        assert_eq!(enemy.entity.hp, 480);
        assert_eq!(enemy.entity.max_hp, 480);
        assert_eq!(enemy.move_damage, 8);
        assert_eq!(enemy.move_hits, 3);
    }

    #[test]
    fn time_eater_haste_and_head_slam_cycle_matches_java() {
        let mut enemy = create_enemy("TimeEater", 456, 456);
        enemy.entity.hp = 200;
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::TE_HASTE);
        assert_eq!(enemy.move_effects.get("remove_debuffs"), Some(&1));
        assert_eq!(enemy.move_effects.get("heal_to_half"), Some(&1));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::TE_HEAD_SLAM);
        assert_eq!(enemy.move_damage, 26);
        assert_eq!(enemy.move_effects.get("draw_reduction"), Some(&1));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::TE_RIPPLE);
        assert_eq!(enemy.move_block, 20);
        assert_eq!(enemy.move_effects.get("vulnerable"), Some(&1));
        assert_eq!(enemy.move_effects.get("weak"), Some(&1));
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
        assert_eq!(enemy.move_effects.get("vulnerable"), Some(&2));
        assert_eq!(enemy.move_effects.get("weak"), Some(&2));
        assert_eq!(enemy.move_effects.get("frail"), Some(&2));
        assert_eq!(enemy.entity.status("Invincible"), 300);
        assert_eq!(enemy.entity.status("BeatOfDeath"), 1);
        assert_eq!(enemy.entity.status("BloodHitCount"), 12);
        assert_eq!(enemy.entity.status("EchoDmg"), 40);
    }

    #[test]
    fn corrupt_heart_a9_and_a19_scaling_matches_java_expectations() {
        let enemy = create_enemy("CorruptHeart", 800, 800);
        assert_eq!(enemy.entity.hp, 800);
        assert_eq!(enemy.entity.max_hp, 800);
        assert_eq!(enemy.entity.status("Invincible"), 200);
        assert_eq!(enemy.entity.status("BeatOfDeath"), 2);
        assert_eq!(enemy.entity.status("BloodHitCount"), 15);
        assert_eq!(enemy.entity.status("EchoDmg"), 45);
    }

    #[test]
    fn corrupt_heart_buff_cycle_matches_java() {
        let mut enemy = create_enemy("CorruptHeart", 750, 750);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEART_BLOOD_SHOTS);
        assert_eq!(enemy.move_damage, 2);
        assert_eq!(enemy.move_hits, 12);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEART_ECHO);
        assert_eq!(enemy.move_damage, 40);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEART_BUFF);
        assert_eq!(enemy.move_effects.get("strength"), Some(&2));
        assert_eq!(enemy.move_effects.get("artifact"), Some(&2));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEART_BLOOD_SHOTS);
    }
}
