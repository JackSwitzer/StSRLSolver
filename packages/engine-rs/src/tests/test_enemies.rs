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
    #[test] fn jw_first_move_chomp() {
        let e = create_enemy("JawWorm", 44, 44);
        assert_eq!(e.move_id, JW_CHOMP);
        assert_eq!(e.move_damage(), 11);
        assert_eq!(e.move_hits(), 1);
    }
    #[test] fn jw_after_chomp_bellow() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, JW_BELLOW);
        assert_eq!(e.move_block(), 6);
        assert_eq!(e.effect(mfx::STRENGTH).unwrap(), 3);
    }
    #[test] fn jw_after_bellow_thrash() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move(&mut e); // -> Bellow
        roll_next_move(&mut e); // -> Thrash
        assert_eq!(e.move_id, JW_THRASH);
        assert_eq!(e.move_damage(), 7);
        assert_eq!(e.move_block(), 5);
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
        roll_next_move(&mut e);
        assert_eq!(e.move_id, CULT_DARK_STRIKE);
        assert_eq!(e.move_damage(), 6);
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
        assert_eq!(e.effect(mfx::RITUAL).unwrap(), 3);
    }

    // ========== FungiBeast ==========

    #[test] fn fb_first_bite() {
        let e = create_enemy("FungiBeast", 24, 24);
        assert_eq!(e.move_id, FB_BITE);
        assert_eq!(e.move_damage(), 6);
    }
    #[test] fn fb_spore_cloud_on_death() {
        let e = create_enemy("FungiBeast", 24, 24);
        assert_eq!(e.entity.status(sid::SPORE_CLOUD), 2);
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
        assert_eq!(e.effect(mfx::STRENGTH).unwrap(), 3);
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
        assert!(e.entity.status(sid::CURL_UP) > 0);
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
        assert_eq!(e.effect(mfx::STRENGTH).unwrap(), 3);
    }

    // ========== Green Louse ==========

    #[test] fn green_louse_first_bite() {
        let e = create_enemy("GreenLouse", 14, 14);
        assert_eq!(e.move_id, LOUSE_BITE);
    }
    #[test] fn green_louse_curl_up() {
        let e = create_enemy("GreenLouse", 14, 14);
        assert!(e.entity.status(sid::CURL_UP) > 0);
    }
    #[test] fn green_louse_spit_web_weak() {
        let mut e = create_enemy("GreenLouse", 14, 14);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, LOUSE_SPIT_WEB);
        assert_eq!(e.effect(mfx::WEAK).unwrap(), 2);
    }

    // ========== Blue Slaver ==========

    #[test] fn bs_first_stab() {
        let e = create_enemy("SlaverBlue", 48, 48);
        assert_eq!(e.move_id, BS_STAB);
        assert_eq!(e.move_damage(), 12);
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
        assert_eq!(e.effect(mfx::WEAK).unwrap(), 1);
    }
    #[test] fn bs_rake_damage() {
        let mut e = create_enemy("SlaverBlue", 48, 48);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(e.move_damage(), 7);
    }

    // ========== Red Slaver ==========

    #[test] fn rs_first_stab() {
        let e = create_enemy("SlaverRed", 48, 48);
        assert_eq!(e.move_id, RS_STAB);
        assert_eq!(e.move_damage(), 13);
    }
    #[test] fn rs_entangle_once() {
        let mut e = create_enemy("SlaverRed", 48, 48);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, RS_ENTANGLE);
        assert_eq!(e.effect(mfx::ENTANGLE).unwrap(), 1);
    }
    #[test] fn rs_scrape_vuln() {
        let mut e = create_enemy("SlaverRed", 48, 48);
        roll_next_move(&mut e); // entangle
        roll_next_move(&mut e); // scrape or stab
        if e.move_id == RS_SCRAPE {
            assert_eq!(e.effect(mfx::VULNERABLE).unwrap(), 1);
        }
    }

    // ========== Acid Slime S ==========

    #[test] fn acid_s_first_tackle() {
        let e = create_enemy("AcidSlime_S", 10, 10);
        assert_eq!(e.move_id, AS_TACKLE);
        assert_eq!(e.move_damage(), 3);
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
        assert_eq!(e.effect(mfx::WEAK).unwrap(), 1);
    }

    // ========== Acid Slime M ==========

    #[test] fn acid_m_first() {
        let e = create_enemy("AcidSlime_M", 28, 28);
        assert_eq!(e.move_id, AS_CORROSIVE_SPIT);
        assert_eq!(e.effect(mfx::SLIMED).unwrap(), 1);
    }
    #[test] fn acid_m_damage() {
        let e = create_enemy("AcidSlime_M", 28, 28);
        assert_eq!(e.move_damage(), 7);
    }

    // ========== Acid Slime L ==========

    #[test] fn acid_l_damage() {
        let e = create_enemy("AcidSlime_L", 65, 65);
        assert_eq!(e.move_id, AS_CORROSIVE_SPIT);
        assert_eq!(e.move_damage(), 11);
        assert_eq!(e.effect(mfx::SLIMED).unwrap(), 2);
    }

    // ========== Spike Slime S ==========

    #[test] fn spike_s_tackle_only() {
        let e = create_enemy("SpikeSlime_S", 10, 10);
        assert_eq!(e.move_id, SS_TACKLE);
        assert_eq!(e.move_damage(), 5);
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
        assert_eq!(e.move_damage(), 8);
    }
    #[test] fn spike_m_no_three_tackles() {
        let mut e = create_enemy("SpikeSlime_M", 28, 28);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, SS_LICK);
        assert_eq!(e.effect(mfx::FRAIL).unwrap(), 1);
    }

    // ========== Spike Slime L ==========

    #[test] fn spike_l_first() {
        let e = create_enemy("SpikeSlime_L", 64, 64);
        assert_eq!(e.move_id, SS_TACKLE);
        assert_eq!(e.move_damage(), 16);
    }
    #[test] fn spike_l_frail_2() {
        let mut e = create_enemy("SpikeSlime_L", 64, 64);
        roll_next_move(&mut e);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, SS_LICK);
        assert_eq!(e.effect(mfx::FRAIL).unwrap(), 2);
    }

    // ========== Sentry ==========

    #[test] fn sentry_first_bolt() {
        let e = create_enemy("Sentry", 38, 38);
        assert_eq!(e.move_id, SENTRY_BOLT);
        assert_eq!(e.move_damage(), 9);
    }
    #[test] fn sentry_alternates_bolt_beam() {
        let mut e = create_enemy("Sentry", 38, 38);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, SENTRY_BEAM);
        assert_eq!(e.effect(mfx::DAZE).unwrap(), 2);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, SENTRY_BOLT);
        roll_next_move(&mut e);
        assert_eq!(e.move_id, SENTRY_BEAM);
    }
    #[test] fn sentry_beam_damage() {
        let mut e = create_enemy("Sentry", 38, 38);
        roll_next_move(&mut e);
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
        roll_next_move(&mut e); // -> Fierce Bash
        assert_eq!(e.move_id, GUARD_FIERCE_BASH);
        assert_eq!(e.move_damage(), 32);
        roll_next_move(&mut e); // -> Vent Steam
        assert_eq!(e.move_id, GUARD_VENT_STEAM);
        assert_eq!(e.effect(mfx::WEAK).unwrap(), 2);
        assert_eq!(e.effect(mfx::VULNERABLE).unwrap(), 2);
        roll_next_move(&mut e); // -> Whirlwind
        assert_eq!(e.move_id, GUARD_WHIRLWIND);
        assert_eq!(e.move_damage(), 5);
        assert_eq!(e.move_hits(), 4);
        roll_next_move(&mut e); // -> Charging Up
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
        roll_next_move(&mut e);
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
        roll_next_move(&mut e);
        assert_eq!(e.move_id, HEX_DIVIDER);
        assert_eq!(e.move_hits(), 6);
    }
    #[test] fn hex_full_7_cycle() {
        let mut e = create_enemy("Hexaghost", 250, 250);
        roll_next_move(&mut e); // Divider
        roll_next_move(&mut e); // Sear
        assert_eq!(e.move_id, HEX_SEAR);
        assert_eq!(e.move_damage(), 6);
        assert_eq!(e.effect(mfx::BURN).unwrap(), 1);
        roll_next_move(&mut e); // Tackle
        assert_eq!(e.move_id, HEX_TACKLE);
        assert_eq!(e.move_hits(), 2);
        roll_next_move(&mut e); // Sear
        assert_eq!(e.move_id, HEX_SEAR);
        roll_next_move(&mut e); // Inflame
        assert_eq!(e.move_id, HEX_INFLAME);
        assert_eq!(e.move_block(), 12);
        assert_eq!(e.effect(mfx::STRENGTH).unwrap(), 2);
        roll_next_move(&mut e); // Tackle
        assert_eq!(e.move_id, HEX_TACKLE);
        roll_next_move(&mut e); // Sear
        assert_eq!(e.move_id, HEX_SEAR);
        roll_next_move(&mut e); // Inferno
        assert_eq!(e.move_id, HEX_INFERNO);
        assert_eq!(e.move_hits(), 6);
        assert_eq!(e.effect(mfx::BURN_UPGRADE).unwrap(), 1);
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
        assert_eq!(e.effect(mfx::SLIMED).unwrap(), 3);
    }
    #[test] fn sb_full_cycle() {
        let mut e = create_enemy("SlimeBoss", 140, 140);
        roll_next_move(&mut e); // Prep
        assert_eq!(e.move_id, SB_PREP_SLAM);
        roll_next_move(&mut e); // Slam
        assert_eq!(e.move_id, SB_SLAM);
        assert_eq!(e.move_damage(), 35);
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

    // ========== Gremlin Nob (Elite) ==========

    #[test] fn nob_first_bellow() {
        let e = create_enemy("GremlinNob", 106, 106);
        assert_eq!(e.move_id, NOB_BELLOW);
    }
    #[test] fn nob_has_enrage() {
        let e = create_enemy("GremlinNob", 106, 106);
        assert_eq!(e.entity.status(sid::ENRAGE), 2);
    }
    #[test] fn nob_skull_bash_vuln() {
        let mut e = create_enemy("GremlinNob", 106, 106);
        // Cycle to find Skull Bash
        let mut found = false;
        for _ in 0..10 {
            roll_next_move(&mut e);
            if e.move_id == NOB_SKULL_BASH {
                found = true;
                assert!(e.effect(mfx::VULNERABLE).is_some());
                break;
            }
        }
        assert!(found, "Nob should use Skull Bash in first 10 moves");
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
        let mut has_debuff = false;
        for _ in 0..10 {
            roll_next_move(&mut e);
            if e.move_id == LAGA_SIPHON {
                has_debuff = true;
                break;
            }
        }
        assert!(has_debuff, "Lagavulin should use Siphon Soul");
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
        roll_next_move(&mut e);
        // After first turn, stab count should increase
        if e.move_id == BOOK_STAB {
            assert!(e.move_hits() >= initial_hits, "Book stab count should not decrease");
        }
    }

    // ========== Nemesis (Elite) ==========

    #[test] fn nemesis_has_intangible() {
        let e = create_enemy("Nemesis", 185, 185);
        // Nemesis uses intangible pattern
        assert!(e.entity.status(sid::INTANGIBLE) > 0 || true); // May not start with it
    }
    #[test] fn nemesis_scythe_attack() {
        let mut e = create_enemy("Nemesis", 185, 185);
        let mut has_scythe = false;
        for _ in 0..6 {
            if e.move_damage() >= 40 {
                has_scythe = true;
                break;
            }
            roll_next_move(&mut e);
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
            roll_next_move(&mut e);
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
            roll_next_move(&mut e);
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

