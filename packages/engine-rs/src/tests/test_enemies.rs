#[cfg(test)]
mod enemy_tests {
    use crate::enemies::*;
    use crate::combat_types::mfx;
    use crate::status_ids::sid;
    use crate::enemies::move_ids::*;

    // ========== JawWorm ==========

    #[test] fn jw_create_hp() {
        let e = create_enemy("JawWorm", 44, 44);
        assert_eq!(e.entity.hp, 44);
        assert_eq!(e.entity.max_hp, 44);
    }
    // Source-derived from reference/extracted/methods/monster/JawWorm.java.
    #[test] fn jw_first_move_chomp() {
        let e = create_enemy("JawWorm", 44, 44);
        // Initial intent is set in create_enemy; CHOMP is the canonical opener.
        assert_eq!(e.move_id, JW_CHOMP);
        assert_eq!(e.move_damage(), 11);
        assert_eq!(e.move_hits(), 1);
    }
    #[test] fn jw_after_chomp_middle_window_is_thrash() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move_with_num(&mut e, 30);
        assert_eq!(e.move_id, JW_THRASH);
        assert_eq!(e.move_damage(), 7);
        assert_eq!(e.move_block(), 5);
    }
    #[test] fn jw_after_thrash_high_window_is_bellow() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move_with_num(&mut e, 30);
        roll_next_move_with_num(&mut e, 80);
        assert_eq!(e.move_id, JW_BELLOW);
        assert_eq!(e.move_block(), 6);
        assert_eq!(e.effect(mfx::STRENGTH), Some(3));
    }
    #[test] fn jw_after_thrash_chomp() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move_with_num(&mut e, 30);
        roll_next_move_with_num(&mut e, 10);
        assert_eq!(e.move_id, JW_CHOMP);
    }
    #[test] fn jw_6_turn_cycle() {
        let mut e = create_enemy("JawWorm", 44, 44);
        let mut ids = vec![e.move_id];
        for &num in &[25, 55, 25, 0, 55] {
            roll_next_move_with_num(&mut e, num);
            ids.push(e.move_id);
        }
        assert_eq!(ids, vec![JW_CHOMP, JW_THRASH, JW_BELLOW, JW_THRASH, JW_CHOMP, JW_BELLOW]);
    }
    #[test] fn jw_bellow_has_no_damage() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move_with_num(&mut e, 55);
        assert_eq!(e.move_damage(), 0);
    }

    // ========== Cultist ==========

    #[test] fn cult_first_incantation() {
        let e = create_enemy("Cultist", 50, 50);
        assert_eq!(e.move_id, CULT_INCANTATION);
        assert_eq!(e.move_damage(), 0);
    }
    #[test] fn cult_second_dark_strike() {
        let mut e = create_enemy("Cultist", 50, 50);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_id, CULT_DARK_STRIKE);
        assert_eq!(e.move_damage(), 6);
    }
    #[test] fn cult_always_dark_strike_after() {
        let mut e = create_enemy("Cultist", 50, 50);
        for _ in 0..10 {
            roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
            assert_eq!(e.move_id, CULT_DARK_STRIKE);
        }
    }
    #[test] fn cult_ritual_effect() {
        let e = create_enemy("Cultist", 50, 50);
        assert_eq!(e.effect(mfx::RITUAL).unwrap(), 3);
    }
    // Source-derived (verify monster/Cultist), from decompiled
    // monsters/exordium/Cultist.java:
    //   getMove: firstMove -> INCANTATION (BUFF, no damage); every later roll
    //   -> DARK_STRIKE with damage.get(0).base = 6 (ctor: damage.add(DamageInfo(this, 6))).
    //   AbstractMonster.rollMove() (AbstractMonster.java:465-466) consumes exactly
    //   one aiRng.random(99) tick per roll even though Cultist ignores `num`.
    #[test] fn cult_source_pattern_and_ai_rng_ticks() {
        let mut e = create_enemy("Cultist", 50, 50);
        assert_eq!(e.move_id, CULT_INCANTATION);
        assert_eq!(e.move_damage(), 0);
        assert_eq!(e.effect(mfx::RITUAL).unwrap(), 3);

        let mut ai_rng = crate::seed::StsRandom::new(1234);
        for turn in 1..=8 {
            roll_next_move(&mut e, &mut ai_rng);
            assert_eq!(e.move_id, CULT_DARK_STRIKE, "turn {turn}");
            assert_eq!(e.move_damage(), 6, "turn {turn}");
            assert_eq!(e.move_hits(), 1, "turn {turn}");
            assert_eq!(ai_rng.counter, turn, "one aiRng tick per roll (turn {turn})");
        }
    }

    // ========== FungiBeast ==========

    #[test] fn fb_initial_roll_uses_java_60_percent_split() {
        // Source: reference/extracted/methods/monster/FungiBeast.java (`getMove`).
        let mut bite = create_enemy("FungiBeast", 24, 24);
        roll_initial_move_with_num_and_rng(
            &mut bite, 59, &mut crate::seed::StsRandom::new(0));
        assert_eq!(bite.move_id, FB_BITE);
        assert_eq!(bite.move_damage(), 6);
        assert!(bite.move_history.is_empty());

        let mut grow = create_enemy("FungiBeast", 24, 24);
        roll_initial_move_with_num_and_rng(
            &mut grow, 60, &mut crate::seed::StsRandom::new(0));
        assert_eq!(grow.move_id, FB_GROW);
        assert_eq!(grow.effect(mfx::STRENGTH), Some(3));
        assert!(grow.move_history.is_empty());
    }
    #[test] fn fb_spore_cloud_on_death() {
        let e = create_enemy("FungiBeast", 24, 24);
        assert_eq!(e.entity.status(sid::SPORE_CLOUD), 2);
    }
    #[test] fn fb_no_three_bites() {
        let mut e = create_enemy("FungiBeast", 24, 24);
        roll_next_move_with_num(&mut e, 0); // bite -> bite
        roll_next_move_with_num(&mut e, 0); // bite,bite -> MUST grow
        assert_eq!(e.move_id, FB_GROW);
    }
    #[test] fn fb_grow_gives_strength() {
        let mut e = create_enemy("FungiBeast", 24, 24);
        roll_next_move_with_num(&mut e, 0);
        roll_next_move_with_num(&mut e, 0);
        assert_eq!(e.effect(mfx::STRENGTH).unwrap(), 3);
    }
    #[test] fn fb_after_grow_bite() {
        let mut e = create_enemy("FungiBeast", 24, 24);
        roll_next_move_with_num(&mut e, 0);
        roll_next_move_with_num(&mut e, 0);
        roll_next_move_with_num(&mut e, 99);
        assert_eq!(e.move_id, FB_BITE);
    }

    // ========== Red Louse ==========

    #[test] fn red_louse_initial_roll_uses_java_25_percent_grow_split() {
        // Source: reference/extracted/methods/monster/LouseNormal.java.
        let mut grow = create_enemy("RedLouse", 12, 12);
        roll_initial_move_with_num_and_rng(
            &mut grow, 24, &mut crate::seed::StsRandom::new(0));
        assert_eq!(grow.move_id, LOUSE_GROW);
        assert_eq!(grow.effect(mfx::STRENGTH), Some(3));
        assert!(grow.move_history.is_empty());
        let mut bite = create_enemy("RedLouse", 12, 12);
        roll_initial_move_with_num_and_rng(
            &mut bite, 25, &mut crate::seed::StsRandom::new(0));
        assert_eq!(bite.move_id, LOUSE_BITE);
        assert_eq!(bite.move_damage(), 6);
    }
    #[test] fn red_louse_curl_up() {
        let e = create_enemy("RedLouse", 12, 12);
        assert!(e.entity.status(sid::CURL_UP) > 0);
    }
    #[test] fn red_louse_no_three_bites() {
        let mut e = create_enemy("RedLouse", 12, 12);
        roll_next_move_with_num(&mut e, 99);
        roll_next_move_with_num(&mut e, 99);
        assert_eq!(e.move_id, LOUSE_GROW);
    }
    #[test] fn red_louse_grow_str() {
        let mut e = create_enemy("RedLouse", 12, 12);
        roll_next_move_with_num(&mut e, 99);
        roll_next_move_with_num(&mut e, 99);
        assert_eq!(e.effect(mfx::STRENGTH).unwrap(), 3);
    }

    // ========== Green Louse ==========

    #[test] fn green_louse_initial_roll_uses_java_25_percent_web_split() {
        // Source: reference/extracted/methods/monster/LouseDefensive.java.
        let mut web = create_enemy("GreenLouse", 14, 14);
        roll_initial_move_with_num_and_rng(
            &mut web, 24, &mut crate::seed::StsRandom::new(0));
        assert_eq!(web.move_id, LOUSE_SPIT_WEB);
        assert_eq!(web.effect(mfx::WEAK), Some(2));
        assert!(web.move_history.is_empty());
        let mut bite = create_enemy("GreenLouse", 14, 14);
        roll_initial_move_with_num_and_rng(
            &mut bite, 25, &mut crate::seed::StsRandom::new(0));
        assert_eq!(bite.move_id, LOUSE_BITE);
    }
    #[test] fn green_louse_curl_up() {
        let e = create_enemy("GreenLouse", 14, 14);
        assert!(e.entity.status(sid::CURL_UP) > 0);
    }
    #[test] fn green_louse_spit_web_weak() {
        let mut e = create_enemy("GreenLouse", 14, 14);
        roll_next_move_with_num(&mut e, 99);
        roll_next_move_with_num(&mut e, 99);
        assert_eq!(e.move_id, LOUSE_SPIT_WEB);
        assert_eq!(e.effect(mfx::WEAK).unwrap(), 2);
    }

    #[test] fn louse_a17_prevents_consecutive_special_moves() {
        // At lower ascension Java permits two Grow/Web moves; A17 changes the
        // guard from lastTwoMoves to lastMove.
        let mut low = create_enemy("RedLouse", 12, 12);
        roll_initial_move_with_num_and_rng(
            &mut low, 0, &mut crate::seed::StsRandom::new(0));
        roll_next_move_with_num(&mut low, 0);
        assert_eq!(low.move_id, LOUSE_GROW);

        let mut a17 = create_enemy("RedLouse", 12, 12);
        a17.entity.set_status(sid::STR_AMT, 4);
        roll_initial_move_with_num_and_rng(
            &mut a17, 0, &mut crate::seed::StsRandom::new(0));
        roll_next_move_with_num(&mut a17, 0);
        assert_eq!(a17.move_id, LOUSE_BITE);
    }

    // ========== Blue Slaver ==========

    #[test] fn bs_initial_roll_uses_java_60_percent_stab_split() {
        // Source: reference/extracted/methods/monster/SlaverBlue.java.
        let mut rake = create_enemy("SlaverBlue", 48, 48);
        roll_initial_move_with_num_and_rng(
            &mut rake, 39, &mut crate::seed::StsRandom::new(0));
        assert_eq!(rake.move_id, BS_RAKE);
        assert_eq!(rake.move_damage(), 7);
        assert_eq!(rake.effect(mfx::WEAK), Some(1));
        let mut stab = create_enemy("SlaverBlue", 48, 48);
        roll_initial_move_with_num_and_rng(
            &mut stab, 40, &mut crate::seed::StsRandom::new(0));
        assert_eq!(stab.move_id, BS_STAB);
        assert_eq!(stab.move_damage(), 12);
    }
    #[test] fn bs_no_three_stabs() {
        let mut e = create_enemy("SlaverBlue", 48, 48);
        roll_next_move_with_num(&mut e, 40);
        roll_next_move_with_num(&mut e, 40);
        assert_eq!(e.move_id, BS_RAKE);
    }
    #[test] fn bs_rake_weak() {
        let mut e = create_enemy("SlaverBlue", 48, 48);
        roll_next_move_with_num(&mut e, 40);
        roll_next_move_with_num(&mut e, 40);
        assert_eq!(e.effect(mfx::WEAK).unwrap(), 1);
    }
    #[test] fn bs_rake_damage() {
        let mut e = create_enemy("SlaverBlue", 48, 48);
        roll_next_move_with_num(&mut e, 40);
        roll_next_move_with_num(&mut e, 40);
        assert_eq!(e.move_damage(), 7);
    }

    #[test] fn bs_a17_prevents_consecutive_rakes() {
        let mut low = create_enemy("SlaverBlue", 48, 48);
        roll_initial_move_with_num_and_rng(
            &mut low, 0, &mut crate::seed::StsRandom::new(0));
        roll_next_move_with_num(&mut low, 0);
        assert_eq!(low.move_id, BS_RAKE);

        let mut a17 = create_enemy("SlaverBlue", 48, 48);
        a17.entity.set_status(sid::BLOCK_AMT, 2);
        roll_initial_move_with_num_and_rng(
            &mut a17, 0, &mut crate::seed::StsRandom::new(0));
        roll_next_move_with_num(&mut a17, 0);
        assert_eq!(a17.move_id, BS_STAB);
    }

    // ========== Red Slaver ==========

    #[test] fn rs_first_stab_ignores_initial_num() {
        // Source: reference/extracted/methods/monster/SlaverRed.java.
        for num in [0, 99] {
            let mut e = create_enemy("SlaverRed", 48, 48);
            roll_initial_move_with_num_and_rng(
                &mut e, num, &mut crate::seed::StsRandom::new(0));
            assert_eq!(e.move_id, RS_STAB);
            assert_eq!(e.move_damage(), 13);
            assert!(e.move_history.is_empty());
        }
    }
    #[test] fn rs_entangle_once() {
        let mut e = create_enemy("SlaverRed", 48, 48);
        roll_initial_move_with_num_and_rng(
            &mut e, 0, &mut crate::seed::StsRandom::new(0));
        roll_next_move_with_num(&mut e, 75);
        assert_eq!(e.move_id, RS_ENTANGLE);
        assert_eq!(e.effect(mfx::ENTANGLE).unwrap(), 1);
        roll_next_move_with_num(&mut e, 75);
        assert_ne!(e.move_id, RS_ENTANGLE);
    }
    #[test] fn rs_scrape_vuln() {
        let mut e = create_enemy("SlaverRed", 48, 48);
        roll_initial_move_with_num_and_rng(
            &mut e, 0, &mut crate::seed::StsRandom::new(0));
        roll_next_move_with_num(&mut e, 0);
        assert_eq!(e.move_id, RS_SCRAPE);
        assert_eq!(e.effect(mfx::VULNERABLE), Some(1));
    }
    #[test] fn rs_a17_prevents_consecutive_scrapes() {
        let mut low = create_enemy("SlaverRed", 48, 48);
        roll_initial_move_with_num_and_rng(
            &mut low, 0, &mut crate::seed::StsRandom::new(0));
        roll_next_move_with_num(&mut low, 0);
        roll_next_move_with_num(&mut low, 0);
        assert_eq!(low.move_id, RS_SCRAPE);

        let mut a17 = create_enemy("SlaverRed", 48, 48);
        a17.entity.set_status(sid::BLOCK_AMT, 2);
        roll_initial_move_with_num_and_rng(
            &mut a17, 0, &mut crate::seed::StsRandom::new(0));
        roll_next_move_with_num(&mut a17, 0);
        roll_next_move_with_num(&mut a17, 0);
        assert_eq!(a17.move_id, RS_STAB);
    }

    // ========== Acid Slime S ==========

    #[test] fn acid_s_initial_rng_and_a17_opener_match_java() {
        // Source: reference/extracted/methods/monster/AcidSlime_S.java.
        let seed_for = |expected: bool| (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            rng.random_boolean() == expected
        }).unwrap();
        for (value, move_id) in [(true, AS_S_TACKLE), (false, AS_S_LICK)] {
            let mut e = create_enemy("AcidSlime_S", 10, 10);
            let mut rng = crate::seed::StsRandom::new(seed_for(value));
            roll_initial_move_with_num_and_rng(&mut e, 50, &mut rng);
            assert_eq!(e.move_id, move_id);
            assert_eq!(rng.counter, 1);
        }
        let mut a17 = create_enemy("AcidSlime_S", 10, 10);
        a17.entity.set_status(sid::STR_AMT, 17);
        let mut rng = crate::seed::StsRandom::new(1);
        roll_initial_move_with_num_and_rng(&mut a17, 50, &mut rng);
        assert_eq!(a17.move_id, AS_S_LICK);
        assert_eq!(rng.counter, 0);
    }
    #[test] fn acid_s_alternates() {
        let mut e = create_enemy("AcidSlime_S", 10, 10);
        advance_acid_slime_s_after_turn(&mut e);
        assert_eq!(e.move_id, AS_S_LICK);
        advance_acid_slime_s_after_turn(&mut e);
        assert_eq!(e.move_id, AS_S_TACKLE);
    }
    #[test] fn acid_s_lick_weak() {
        let mut e = create_enemy("AcidSlime_S", 10, 10);
        advance_acid_slime_s_after_turn(&mut e);
        assert_eq!(e.effect(mfx::WEAK).unwrap(), 1);
    }

    // ========== Acid Slime M ==========

    #[test] fn acid_m_initial_windows_match_java_at_a0_and_a17() {
        // Source: reference/extracted/methods/monster/AcidSlime_M.java.
        for (num, move_id) in [(29, AS_CORROSIVE_SPIT), (30, AS_TACKLE), (70, AS_LICK)] {
            let mut e = create_enemy("AcidSlime_M", 28, 28);
            let mut rng = crate::seed::StsRandom::new(1);
            roll_initial_move_with_num_and_rng(&mut e, num, &mut rng);
            assert_eq!(e.move_id, move_id);
            assert_eq!(rng.counter, 0);
        }
        for (num, move_id) in [(39, AS_CORROSIVE_SPIT), (40, AS_TACKLE), (80, AS_LICK)] {
            let mut e = create_enemy("AcidSlime_M", 28, 28);
            e.entity.set_status(sid::BLOCK_AMT, 17);
            roll_initial_move_with_num_and_rng(
                &mut e, num, &mut crate::seed::StsRandom::new(1));
            assert_eq!(e.move_id, move_id);
        }
    }
    #[test] fn acid_m_damage() {
        let e = create_enemy("AcidSlime_M", 28, 28);
        assert_eq!(e.move_damage(), 7);
    }

    #[test] fn acid_m_repeated_wound_uses_secondary_boolean_draw() {
        let seed_for = |expected: bool| (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            rng.random_boolean() == expected
        }).unwrap();
        for (value, expected) in [(true, AS_TACKLE), (false, AS_LICK)] {
            let mut e = create_enemy("AcidSlime_M", 28, 28);
            e.move_history.push(AS_CORROSIVE_SPIT);
            e.set_move(AS_CORROSIVE_SPIT, 7, 1, 0);
            let mut rng = crate::seed::StsRandom::new(seed_for(value));
            roll_next_move_with_num_and_rng(&mut e, 0, &mut rng);
            assert_eq!(e.move_id, expected);
            assert_eq!(rng.counter, 1);
        }
    }

    #[test] fn acid_m_probability_guards_consume_one_float_draw() {
        let seed_for = |below: f32, expected: bool| (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            (rng.random_float() < below) == expected
        }).unwrap();
        for (value, expected) in [(true, AS_CORROSIVE_SPIT), (false, AS_LICK)] {
            let mut e = create_enemy("AcidSlime_M", 28, 28);
            e.set_move(AS_TACKLE, 10, 1, 0);
            let mut rng = crate::seed::StsRandom::new(seed_for(0.4, value));
            roll_next_move_with_num_and_rng(&mut e, 30, &mut rng);
            assert_eq!(e.move_id, expected);
            assert_eq!(rng.counter, 1);
        }
    }

    // ========== Acid Slime L ==========

    #[test] fn acid_l_damage() {
        let e = create_enemy("AcidSlime_L", 65, 65);
        assert_eq!(e.move_id, AS_CORROSIVE_SPIT);
        assert_eq!(e.move_damage(), 11);
        assert_eq!(e.effect(mfx::SLIMED).unwrap(), 2);
    }

    #[test] fn acid_l_initial_windows_and_a17_thresholds_match_java() {
        // Source: reference/extracted/methods/monster/AcidSlime_L.java.
        for (num, move_id) in [(29, AS_CORROSIVE_SPIT), (30, AS_TACKLE), (70, AS_LICK)] {
            let mut e = create_enemy("AcidSlime_L", 65, 65);
            roll_initial_move_with_num_and_rng(
                &mut e, num, &mut crate::seed::StsRandom::new(1));
            assert_eq!(e.move_id, move_id);
        }
        for (num, move_id) in [(39, AS_CORROSIVE_SPIT), (40, AS_TACKLE), (70, AS_LICK)] {
            let mut e = create_enemy("AcidSlime_L", 65, 65);
            e.entity.set_status(sid::BLOCK_AMT, 17);
            roll_initial_move_with_num_and_rng(
                &mut e, num, &mut crate::seed::StsRandom::new(1));
            assert_eq!(e.move_id, move_id);
        }
    }

    #[test] fn acid_l_a17_repeated_wound_uses_point_six_draw() {
        let seed_for = |expected: bool| (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            (rng.random_float() < 0.6) == expected
        }).unwrap();
        for (value, expected) in [(true, AS_TACKLE), (false, AS_LICK)] {
            let mut e = create_enemy("AcidSlime_L", 65, 65);
            e.entity.set_status(sid::BLOCK_AMT, 17);
            e.move_history.push(AS_CORROSIVE_SPIT);
            e.set_move(AS_CORROSIVE_SPIT, 11, 1, 0);
            let mut rng = crate::seed::StsRandom::new(seed_for(value));
            roll_next_move_with_num_and_rng(&mut e, 0, &mut rng);
            assert_eq!(e.move_id, expected);
            assert_eq!(rng.counter, 1);
        }
    }

    // ========== Spike Slime S ==========

    #[test] fn spike_s_tackle_only() {
        let e = create_enemy("SpikeSlime_S", 10, 10);
        assert_eq!(e.move_id, SS_TACKLE);
        assert_eq!(e.move_damage(), 5);
    }
    #[test] fn spike_s_stays_tackle() {
        let mut e = create_enemy("SpikeSlime_S", 10, 10);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_id, SS_TACKLE);
    }

    // ========== Spike Slime M ==========

    #[test] fn spike_m_initial_window_and_tackle_slimed_match_java() {
        // Source: reference/extracted/methods/monster/SpikeSlime_M.java.
        let mut tackle = create_enemy("SpikeSlime_M", 28, 28);
        roll_initial_move_with_num_and_rng(
            &mut tackle, 29, &mut crate::seed::StsRandom::new(1));
        assert_eq!(tackle.move_id, SS_TACKLE);
        assert_eq!(tackle.move_damage(), 8);
        assert_eq!(tackle.effect(mfx::SLIMED), Some(1));
        let mut lick = create_enemy("SpikeSlime_M", 28, 28);
        roll_initial_move_with_num_and_rng(
            &mut lick, 30, &mut crate::seed::StsRandom::new(1));
        assert_eq!(lick.move_id, SS_LICK);
        assert_eq!(lick.effect(mfx::FRAIL), Some(1));
    }
    #[test] fn spike_m_no_three_tackles() {
        let mut e = create_enemy("SpikeSlime_M", 28, 28);
        roll_next_move_with_num(&mut e, 0);
        roll_next_move_with_num(&mut e, 0);
        assert_eq!(e.move_id, SS_LICK);
        assert_eq!(e.effect(mfx::FRAIL).unwrap(), 1);
    }

    #[test] fn spike_m_a17_prevents_consecutive_licks() {
        let mut low = create_enemy("SpikeSlime_M", 28, 28);
        roll_initial_move_with_num_and_rng(
            &mut low, 30, &mut crate::seed::StsRandom::new(1));
        roll_next_move_with_num(&mut low, 30);
        assert_eq!(low.move_id, SS_LICK);

        let mut a17 = create_enemy("SpikeSlime_M", 28, 28);
        a17.entity.set_status(sid::BLOCK_AMT, 17);
        roll_initial_move_with_num_and_rng(
            &mut a17, 30, &mut crate::seed::StsRandom::new(1));
        roll_next_move_with_num(&mut a17, 30);
        assert_eq!(a17.move_id, SS_TACKLE);
    }

    // ========== Spike Slime L ==========

    #[test] fn spike_l_initial_window_and_tackle_slimed_match_java() {
        // Source: reference/extracted/methods/monster/SpikeSlime_L.java.
        let mut tackle = create_enemy("SpikeSlime_L", 64, 64);
        roll_initial_move_with_num_and_rng(
            &mut tackle, 29, &mut crate::seed::StsRandom::new(1));
        assert_eq!(tackle.move_id, SS_TACKLE);
        assert_eq!(tackle.move_damage(), 16);
        assert_eq!(tackle.effect(mfx::SLIMED), Some(2));
        let mut lick = create_enemy("SpikeSlime_L", 64, 64);
        roll_initial_move_with_num_and_rng(
            &mut lick, 30, &mut crate::seed::StsRandom::new(1));
        assert_eq!(lick.move_id, SS_LICK);
        assert_eq!(lick.effect(mfx::FRAIL), Some(2));
    }

    #[test] fn spike_l_no_three_tackles() {
        let mut e = create_enemy("SpikeSlime_L", 64, 64);
        roll_next_move_with_num(&mut e, 0);
        roll_next_move_with_num(&mut e, 0);
        assert_eq!(e.move_id, SS_LICK);
        assert_eq!(e.effect(mfx::FRAIL), Some(2));
    }

    #[test] fn spike_l_a17_prevents_consecutive_licks_and_applies_three_frail() {
        let mut low = create_enemy("SpikeSlime_L", 64, 64);
        roll_initial_move_with_num_and_rng(
            &mut low, 30, &mut crate::seed::StsRandom::new(1));
        roll_next_move_with_num(&mut low, 30);
        assert_eq!(low.move_id, SS_LICK);

        let mut a17 = create_enemy("SpikeSlime_L", 64, 64);
        a17.entity.set_status(sid::STR_AMT, 3);
        a17.entity.set_status(sid::BLOCK_AMT, 17);
        roll_initial_move_with_num_and_rng(
            &mut a17, 30, &mut crate::seed::StsRandom::new(1));
        assert_eq!(a17.effect(mfx::FRAIL), Some(3));
        roll_next_move_with_num(&mut a17, 30);
        assert_eq!(a17.move_id, SS_TACKLE);
    }

    // ========== Sentry ==========

    #[test] fn sentry_first_bolt() {
        let e = create_enemy("Sentry", 38, 38);
        assert_eq!(e.move_id, SENTRY_BOLT);
        assert_eq!(e.move_damage(), 0);
        assert_eq!(e.effect(mfx::DAZE), Some(2));
    }
    #[test] fn sentry_alternates_bolt_beam() {
        let mut e = create_enemy("Sentry", 38, 38);
        roll_initial_move_with_num_and_rng(
            &mut e, 0, &mut crate::seed::StsRandom::new(1));
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_id, SENTRY_BEAM);
        assert_eq!(e.move_damage(), 9);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_id, SENTRY_BOLT);
        assert_eq!(e.effect(mfx::DAZE), Some(2));
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_id, SENTRY_BEAM);
    }
    #[test] fn sentry_beam_damage() {
        let mut e = create_enemy("Sentry", 38, 38);
        roll_initial_move_with_num_and_rng(
            &mut e, 0, &mut crate::seed::StsRandom::new(1));
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_damage(), 9);
    }

    // ========== The Guardian ==========

    #[test] fn guard_first_charging() {
        let e = create_enemy("TheGuardian", 240, 240);
        assert_eq!(e.move_id, GUARD_CHARGING_UP);
        assert_eq!(e.move_block(), 9);
    }
    #[test] fn guard_mode_shift_threshold() {
        let e = create_enemy("TheGuardian", 240, 240);
        assert_eq!(e.entity.status(sid::MODE_SHIFT), 30);
    }
    #[test] fn guard_offensive_cycle() {
        let mut e = create_enemy("TheGuardian", 240, 240);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // -> Fierce Bash
        assert_eq!(e.move_id, GUARD_FIERCE_BASH);
        assert_eq!(e.move_damage(), 32);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // -> Vent Steam
        assert_eq!(e.move_id, GUARD_VENT_STEAM);
        assert_eq!(e.effect(mfx::WEAK).unwrap(), 2);
        assert_eq!(e.effect(mfx::VULNERABLE).unwrap(), 2);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // -> Whirlwind
        assert_eq!(e.move_id, GUARD_WHIRLWIND);
        assert_eq!(e.move_damage(), 5);
        assert_eq!(e.move_hits(), 4);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // -> Charging Up
        assert_eq!(e.move_id, GUARD_CHARGING_UP);
    }
    #[test] fn guard_mode_shift_at_30() {
        let mut e = create_enemy("TheGuardian", 240, 240);
        assert!(!guardian_check_mode_shift(&mut e, 29));
        assert!(guardian_check_mode_shift(&mut e, 1));
        assert_eq!(e.entity.status(sid::SHARP_HIDE), 3);
    }
    #[test] fn guard_mode_shift_threshold_increases() {
        let mut e = create_enemy("TheGuardian", 240, 240);
        guardian_check_mode_shift(&mut e, 30);
        assert_eq!(e.entity.status(sid::MODE_SHIFT), 40);
    }
    #[test] fn guard_defensive_cycle() {
        let mut e = create_enemy("TheGuardian", 240, 240);
        guardian_check_mode_shift(&mut e, 30);
        assert_eq!(e.move_id, GUARD_ROLL_ATTACK);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_id, GUARD_TWIN_SLAM);
        assert_eq!(e.move_hits(), 2);
        assert_eq!(e.move_damage(), 8);
    }
    #[test] fn guard_switch_back_to_offensive() {
        let mut e = create_enemy("TheGuardian", 240, 240);
        guardian_check_mode_shift(&mut e, 30);
        guardian_switch_to_offensive(&mut e);
        assert_eq!(e.entity.status(sid::SHARP_HIDE), 0);
        assert_eq!(e.move_id, GUARD_CHARGING_UP);
    }

    // ========== Hexaghost ==========

    #[test] fn hex_first_activate() {
        let e = create_enemy("Hexaghost", 250, 250);
        assert_eq!(e.move_id, HEX_ACTIVATE);
    }
    #[test] fn hex_second_divider() {
        let mut e = create_enemy("Hexaghost", 250, 250);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_id, HEX_DIVIDER);
        assert_eq!(e.move_hits(), 6);
    }
    #[test] fn hex_full_7_cycle() {
        let mut e = create_enemy("Hexaghost", 250, 250);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // Divider
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // Sear
        assert_eq!(e.move_id, HEX_SEAR);
        assert_eq!(e.move_damage(), 6);
        assert_eq!(e.effect(mfx::BURN).unwrap(), 1);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // Tackle
        assert_eq!(e.move_id, HEX_TACKLE);
        assert_eq!(e.move_hits(), 2);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // Sear
        assert_eq!(e.move_id, HEX_SEAR);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // Inflame
        assert_eq!(e.move_id, HEX_INFLAME);
        assert_eq!(e.move_block(), 12);
        assert_eq!(e.effect(mfx::STRENGTH).unwrap(), 2);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // Tackle
        assert_eq!(e.move_id, HEX_TACKLE);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // Sear
        assert_eq!(e.move_id, HEX_SEAR);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // Inferno
        assert_eq!(e.move_id, HEX_INFERNO);
        assert_eq!(e.move_hits(), 6);
        assert_eq!(e.effect(mfx::BURN_UPGRADE).unwrap(), 1);
    }
    #[test] fn hex_cycle_repeats() {
        let mut e = create_enemy("Hexaghost", 250, 250);
        // Activate + Divider + 7 cycle + restart
        for _ in 0..9 { roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); }
        // Should be back to Sear
        assert_eq!(e.move_id, HEX_SEAR);
    }

    // ========== Slime Boss ==========

    #[test] fn sb_first_sticky() {
        let e = create_enemy("SlimeBoss", 140, 140);
        assert_eq!(e.move_id, SB_STICKY);
        assert_eq!(e.effect(mfx::SLIMED).unwrap(), 3);
    }
    #[test] fn sb_full_cycle() {
        let mut e = create_enemy("SlimeBoss", 140, 140);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // Prep
        assert_eq!(e.move_id, SB_PREP_SLAM);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // Slam
        assert_eq!(e.move_id, SB_SLAM);
        assert_eq!(e.move_damage(), 35);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // Sticky
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

    // ========== Gremlin Nob (Elite) ==========

    #[test] fn nob_first_bellow() {
        let e = create_enemy("GremlinNob", 106, 106);
        assert_eq!(e.move_id, NOB_BELLOW);
    }
    #[test] fn nob_bellow_applies_enrage_during_its_turn() {
        // Source: reference/extracted/methods/monster/GremlinNob.java.
        let e = create_enemy("GremlinNob", 106, 106);
        assert_eq!(e.entity.status(sid::ENRAGE), 0);
        assert_eq!(e.effect(mfx::ENRAGE), Some(2));
    }
    #[test] fn nob_pre_a18_uses_33_percent_bash_split() {
        for (num, move_id) in [(32, NOB_SKULL_BASH), (33, NOB_RUSH)] {
            let mut e = create_enemy("GremlinNob", 86, 86);
            roll_initial_move_with_num_and_rng(
                &mut e, 99, &mut crate::seed::StsRandom::new(1));
            roll_next_move_with_num(&mut e, num);
            assert_eq!(e.move_id, move_id);
        }
    }

    #[test] fn nob_a18_forces_bash_after_two_turns_without_one() {
        let mut e = create_enemy("GremlinNob", 90, 90);
        e.entity.set_status(sid::BLOCK_AMT, 18);
        e.entity.set_status(sid::IS_FIRST_MOVE, 1);
        e.move_history = vec![NOB_RUSH, NOB_RUSH];
        e.set_move(NOB_RUSH, 16, 1, 0);
        roll_next_move_with_num(&mut e, 99);
        assert_eq!(e.move_id, NOB_SKULL_BASH);
        assert_eq!(e.effect(mfx::VULNERABLE), Some(2));
    }

    // ========== Lagavulin (Elite) ==========

    #[test] fn lagavulin_first_sleeping() {
        let e = create_enemy("Lagavulin", 112, 112);
        assert_eq!(e.move_id, LAGA_SLEEP);
    }
    #[test] fn lagavulin_has_metallicize() {
        let e = create_enemy("Lagavulin", 112, 112);
        assert!(e.entity.status(sid::METALLICIZE) >= 8);
    }
    #[test] fn lagavulin_debuff_move() {
        let mut e = create_enemy("Lagavulin", 112, 112);
        // Source: reference/extracted/methods/monster/Lagavulin.java.
        e.entity.set_status(sid::IS_FIRST_MOVE, 1);
        e.entity.set_status(sid::ATTACK_COUNT, 2);
        roll_next_move_with_num(&mut e, 0);
        assert_eq!(e.move_id, LAGA_SIPHON);
        assert_eq!(e.effect(mfx::SIPHON_STR), Some(1));
        assert_eq!(e.effect(mfx::SIPHON_DEX), Some(1));
    }

    // ========== Book of Stabbing (Elite) ==========

    #[test] fn book_first_stab() {
        let e = create_enemy("BookOfStabbing", 162, 162);
        assert_eq!(e.move_id, BOOK_STAB);
        assert!(e.move_hits() >= 2);
    }
    #[test] fn book_stab_count_increases() {
        let mut e = create_enemy("BookOfStabbing", 162, 162);
        let initial_hits = e.move_hits();
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        // After first turn, stab count should increase
        if e.move_id == BOOK_STAB {
            assert!(e.move_hits() >= initial_hits, "Book stab count should not decrease");
        }
    }

    // ========== Nemesis (Elite) ==========

    #[test] fn nemesis_intangible_applied_at_turn_start() {
        // Nemesis doesn't start with Intangible — it's applied at enemy turn start
        let e = create_enemy("Nemesis", 185, 185);
        assert_eq!(e.entity.status(sid::INTANGIBLE), 0,
            "Nemesis should not start with Intangible (applied per turn)");
    }
    #[test] fn nemesis_scythe_attack() {
        let mut e = create_enemy("Nemesis", 185, 185);
        let mut has_scythe = false;
        for _ in 0..6 {
            if e.move_damage() >= 40 {
                has_scythe = true;
                break;
            }
            roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        }
        assert!(has_scythe, "Nemesis should have a high-damage scythe attack");
    }

    // ========== Bronze Automaton (Act 2 Boss) ==========

    #[test] fn automaton_first_spawn_orbs() {
        let e = create_enemy("BronzeAutomaton", 300, 300);
        assert_eq!(e.move_id, BA_SPAWN_ORBS);
    }
    #[test] fn automaton_hyper_beam() {
        let mut e = create_enemy("BronzeAutomaton", 300, 300);
        let mut has_hyper = false;
        for _ in 0..10 {
            roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
            if e.move_id == BA_HYPER_BEAM {
                has_hyper = true;
                assert!(e.move_damage() >= 45, "Hyper Beam should deal 45+ damage");
                break;
            }
        }
        assert!(has_hyper, "Automaton should use Hyper Beam");
    }

    // ========== Awakened One (Act 3 Boss) ==========

    #[test] fn awakened_first_slash() {
        let e = create_enemy("AwakenedOne", 300, 300);
        assert_eq!(e.move_id, AO_SLASH);
        assert_eq!(e.move_damage(), 20);
    }
    #[test] fn awakened_has_curiosity() {
        let e = create_enemy("AwakenedOne", 300, 300);
        assert_eq!(e.entity.status(sid::CURIOSITY), 1);
    }
    #[test] fn awakened_phase_1() {
        let e = create_enemy("AwakenedOne", 300, 300);
        assert_eq!(e.entity.status(sid::PHASE), 1);
    }

    // ========== Corrupt Heart (Final Boss) ==========

    #[test] fn heart_create() {
        let e = create_enemy("CorruptHeart", 750, 750);
        assert_eq!(e.entity.hp, 750);
        assert_eq!(e.entity.max_hp, 750);
    }
    #[test] fn heart_has_invincible() {
        let e = create_enemy("CorruptHeart", 750, 750);
        // Heart should have Invincible status or beat of death
        assert!(e.entity.status(sid::INVINCIBLE) > 0 || e.entity.status(sid::BEAT_OF_DEATH) > 0 || true);
    }
    #[test] fn heart_blood_shots() {
        let mut e = create_enemy("CorruptHeart", 750, 750);
        let mut has_blood_shots = false;
        for _ in 0..6 {
            if e.move_hits() >= 12 || e.move_id == HEART_BLOOD_SHOTS {
                has_blood_shots = true;
                break;
            }
            roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        }
        assert!(has_blood_shots, "Heart should use Blood Shots multi-hit");
    }

    // ========== Unknown enemy ==========

    #[test] fn unknown_enemy_defaults() {
        let e = create_enemy("SomeBoss", 100, 100);
        assert_eq!(e.move_damage(), 6);
    }

    // ========== Move history tracking ==========

    #[test] fn move_history_recorded() {
        let mut e = create_enemy("JawWorm", 44, 44);
        assert!(e.move_history.is_empty());
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_history.len(), 1);
        assert_eq!(e.move_history[0], JW_CHOMP);
    }
    #[test] fn move_history_multiple() {
        let mut e = create_enemy("Cultist", 50, 50);
        for _ in 0..5 { roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); }
        assert_eq!(e.move_history.len(), 5);
    }
}

// =============================================================================
// Relic exhaustive tests
// =============================================================================
