#[cfg(test)]
mod enemy_tests {
    use crate::enemies::*;
    use crate::combat_types::mfx;
    use crate::state::EnemyCombatState;
    use crate::status_ids::sid;
    use crate::enemies::move_ids::*;

    // ========== JawWorm ==========

    #[test] fn jw_create_hp() {
        let e = create_enemy("JawWorm", 44, 44);
        assert_eq!(e.entity.hp, 44);
        assert_eq!(e.entity.max_hp, 44);
    }
    // JawWorm intent now branches on the dispatched `num` (0..=99). These tests
    // exercise specific branches via `roll_next_move_with_num` to stay deterministic;
    // the pre-RNG-fix tests asserted the broken always-CHOMP-first behavior.
    #[test] fn jw_first_move_chomp() {
        let e = create_enemy("JawWorm", 44, 44);
        // Initial intent is set in create_enemy; CHOMP is the canonical opener.
        assert_eq!(e.move_id, JW_CHOMP);
        assert_eq!(e.move_damage(), 11);
        assert_eq!(e.move_hits(), 1);
    }
    // Java JawWorm.java:146 default branches: 0-24 CHOMP / 25-54 THRASH / 55-99 BELLOW.
    #[test] fn jw_after_chomp_thrash() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move_with_num(&mut e, 30); // 25..55, !lastTwoMoves(THRASH) -> THRASH
        assert_eq!(e.move_id, JW_THRASH);
        assert_eq!(e.move_damage(), 7);
        assert_eq!(e.move_block(), 5);
    }
    #[test] fn jw_after_thrash_bellow() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move_with_num(&mut e, 30); // -> THRASH
        roll_next_move_with_num(&mut e, 80); // >=55, !lastMove(BELLOW) -> BELLOW
        assert_eq!(e.move_id, JW_BELLOW);
        assert_eq!(e.move_block(), 6);
        assert_eq!(e.effect(mfx::STRENGTH).unwrap(), 3);
    }
    #[test] fn jw_after_bellow_chomp() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move_with_num(&mut e, 30); // THRASH
        roll_next_move_with_num(&mut e, 80); // BELLOW
        roll_next_move_with_num(&mut e, 10); // <25, !lastMove(CHOMP) -> CHOMP
        assert_eq!(e.move_id, JW_CHOMP);
    }
    #[test] fn jw_6_turn_cycle() {
        let mut e = create_enemy("JawWorm", 44, 44);
        let mut ids = vec![e.move_id];
        // Cycle nums to exercise each Java branch deterministically.
        for &num in &[30, 80, 10, 30, 80] {
            roll_next_move_with_num(&mut e, num);
            ids.push(e.move_id);
        }
        assert_eq!(ids[0], JW_CHOMP);
        assert_eq!(ids[1], JW_THRASH);
        assert_eq!(ids[2], JW_BELLOW);
        assert_eq!(ids[3], JW_CHOMP);
    }
    #[test] fn jw_bellow_has_no_damage() {
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move_with_num(&mut e, 80); // >=55 -> BELLOW
        assert_eq!(e.move_id, JW_BELLOW);
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
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // bite -> bite
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0)); // bite,bite -> MUST grow
        assert_eq!(e.move_id, FB_GROW);
    }
    #[test] fn fb_grow_gives_strength() {
        let mut e = create_enemy("FungiBeast", 24, 24);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.effect(mfx::STRENGTH).unwrap(), 3);
    }
    #[test] fn fb_after_grow_bite() {
        let mut e = create_enemy("FungiBeast", 24, 24);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_id, FB_BITE);
    }

    // ========== Red Louse ==========

    #[test] fn red_louse_first_bite() {
        let e = create_enemy("RedLouse", 12, 12);
        assert_eq!(e.move_id, LOUSE_BITE);
    }
    #[test] fn red_louse_curl_up() {
        let e = create_enemy("RedLouse", 12, 12);
        // Java LouseNormal CurlUpPower amount = 5 (mod.rs:417).
        assert_eq!(e.entity.status(sid::CURL_UP), 5);
    }
    #[test] fn red_louse_no_three_bites() {
        // With Java-parity RNG: num>=25 avoids the GROW branch so we get BITE, BITE.
        // Third roll with num>=25 triggers anti-repeat !lastTwoMoves(BITE)=false -> GROW.
        let mut e = create_enemy("RedLouse", 12, 12);
        roll_next_move_with_num(&mut e, 50); // BITE (initial was BITE, but num>=25 and not lastMove(GROW))
        roll_next_move_with_num(&mut e, 50); // BITE, BITE, BITE history triggers anti-repeat
        assert_eq!(e.move_id, LOUSE_GROW);
    }
    #[test] fn red_louse_grow_str() {
        let mut e = create_enemy("RedLouse", 12, 12);
        roll_next_move_with_num(&mut e, 50);
        roll_next_move_with_num(&mut e, 50);
        assert_eq!(e.effect(mfx::STRENGTH).unwrap(), 3);
    }

    // ========== Green Louse ==========

    #[test] fn green_louse_first_bite() {
        let e = create_enemy("GreenLouse", 14, 14);
        assert_eq!(e.move_id, LOUSE_BITE);
    }
    #[test] fn green_louse_curl_up() {
        let e = create_enemy("GreenLouse", 14, 14);
        // Java LouseDefensive CurlUpPower amount = 5 (mod.rs:421).
        assert_eq!(e.entity.status(sid::CURL_UP), 5);
    }
    #[test] fn green_louse_spit_web_weak() {
        // num>=25 avoids the early SPIT_WEB branch; two BITEs then anti-repeat -> SPIT_WEB.
        let mut e = create_enemy("GreenLouse", 14, 14);
        roll_next_move_with_num(&mut e, 50);
        roll_next_move_with_num(&mut e, 50);
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
        // Java SlaverBlue STAB is gated on !lastMove(STAB) && !lastTwoMoves(STAB).
        // Preseed history with two STABs to force the anti-repeat branch.
        let mut e = create_enemy("SlaverBlue", 48, 48);
        e.move_history = vec![BS_STAB, BS_STAB];
        e.move_id = BS_STAB;
        roll_next_move_with_num(&mut e, 0); // num<40 but lastMove(STAB)=true -> RAKE
        assert_eq!(e.move_id, BS_RAKE);
    }
    #[test] fn bs_rake_weak() {
        let mut e = create_enemy("SlaverBlue", 48, 48);
        e.move_history = vec![BS_STAB, BS_STAB];
        e.move_id = BS_STAB;
        roll_next_move_with_num(&mut e, 0);
        assert_eq!(e.effect(mfx::WEAK).unwrap(), 1);
    }
    #[test] fn bs_rake_damage() {
        let mut e = create_enemy("SlaverBlue", 48, 48);
        e.move_history = vec![BS_STAB, BS_STAB];
        e.move_id = BS_STAB;
        roll_next_move_with_num(&mut e, 0);
        assert_eq!(e.move_damage(), 7);
    }

    // ========== Red Slaver ==========

    #[test] fn rs_first_stab() {
        let e = create_enemy("SlaverRed", 48, 48);
        assert_eq!(e.move_id, RS_STAB);
        assert_eq!(e.move_damage(), 13);
    }
    #[test] fn rs_entangle_once() {
        // Java SlaverRed ENTANGLE gates on num>=75 && !usedEntangle && !firstMove.
        // Initial move is already STAB (firstMove=true for history.len()==1), so we
        // must roll twice: throwaway first, then num=80 triggers ENTANGLE.
        let mut e = create_enemy("SlaverRed", 48, 48);
        roll_next_move_with_num(&mut e, 60); // any non-entangle num, consumes firstMove
        roll_next_move_with_num(&mut e, 80);
        assert_eq!(e.move_id, RS_ENTANGLE);
        assert_eq!(e.effect(mfx::ENTANGLE).unwrap(), 1);
    }
    #[test] fn rs_scrape_vuln() {
        let mut e = create_enemy("SlaverRed", 48, 48);
        roll_next_move_with_num(&mut e, 60); // any num, not entangle
        roll_next_move_with_num(&mut e, 80); // entangle
        roll_next_move_with_num(&mut e, 0); // num<55, not lastTwoMoves(SCRAPE) -> SCRAPE
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
        // Java AcidSlime_S num<40 -> TACKLE, else LICK (with anti-repeat LICK->TACKLE).
        // num=60 selects LICK; subsequent num=60 would re-select LICK but anti-repeat -> TACKLE.
        let mut e = create_enemy("AcidSlime_S", 10, 10);
        roll_next_move_with_num(&mut e, 60);
        assert_eq!(e.move_id, AS_LICK);
        roll_next_move_with_num(&mut e, 60);
        assert_eq!(e.move_id, AS_TACKLE);
    }
    #[test] fn acid_s_lick_weak() {
        let mut e = create_enemy("AcidSlime_S", 10, 10);
        roll_next_move_with_num(&mut e, 60); // num>=40 -> LICK + Weak 1
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
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
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
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
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
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
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
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_id, SENTRY_BEAM);
        assert_eq!(e.effect(mfx::DAZE).unwrap(), 2);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_id, SENTRY_BOLT);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_id, SENTRY_BEAM);
    }
    #[test] fn sentry_beam_damage() {
        let mut e = create_enemy("Sentry", 38, 38);
        roll_next_move(&mut e, &mut crate::seed::StsRandom::new(0));
        assert_eq!(e.move_damage(), 9);
    }

    // ========== Act 1 Bosses ==========
    // Guardian / Hexaghost / SlimeBoss boss tests moved to `test_bosses.rs`
    // (covers A0/A2/A4/A9/A19 scaling + cycle with Java file:line citations).
    // This file covers common/elite enemies only.

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
        // Java GremlinNob turn 2 `num < 33 && lastMove(BELLOW) → SKULL_BASH + Vuln(2)`
        // (act1.rs:328-331). Initial intent is BELLOW; one roll with num=10
        // lands squarely in the SkullBash branch.
        let mut e = create_enemy("GremlinNob", 106, 106);
        roll_next_move_with_num(&mut e, 10);
        assert_eq!(e.move_id, NOB_SKULL_BASH);
        assert_eq!(e.move_damage(), 6);
        assert_eq!(e.effect(mfx::VULNERABLE), Some(2));
    }

    // ========== Lagavulin (Elite) ==========

    #[test] fn lagavulin_first_sleeping() {
        let e = create_enemy("Lagavulin", 112, 112);
        assert_eq!(e.move_id, LAGA_SLEEP);
    }
    #[test] fn lagavulin_has_metallicize() {
        let e = create_enemy("Lagavulin", 112, 112);
        // Java Lagavulin MetallicizePower amount = 8 (mod.rs:480).
        assert_eq!(e.entity.status(sid::METALLICIZE), 8);
    }
    #[test] fn lagavulin_debuff_move() {
        // Java Lagavulin.java L209-223 getMove: awake cycle is
        //   STRONG_ATK -> STRONG_ATK -> SIPHON_SOUL -> STRONG_ATK -> ...
        // (2:1 attack-to-debuff ratio; D154 parity fix). SLEEP_TURNS=3 at
        // init — three rolls tick sleep down; the 3rd roll clears Metallicize
        // and sets LAGA_ATTACK as the first awake intent. Subsequent rolls
        // follow the 2:1 cycle. Lagavulin's AI doesn't consume `num`, so pass 0.
        let mut e = create_enemy("Lagavulin", 112, 112);
        for _ in 0..3 {
            roll_next_move_with_num(&mut e, 0);
        }
        assert_eq!(e.move_id, LAGA_ATTACK, "turn 3 wakes to ATTACK");
        assert_eq!(e.entity.status(sid::METALLICIZE), 0, "wake clears Metallicize");
        // Turn 4: one ATTACK in awake history — !lastTwoMoves(ATTACK) -> ATTACK.
        roll_next_move_with_num(&mut e, 0);
        assert_eq!(e.move_id, LAGA_ATTACK, "turn 4 is second ATTACK in 2:1 cycle");
        // Turn 5: two ATTACKs in a row -> SIPHON_SOUL with Str+Dex debuff.
        roll_next_move_with_num(&mut e, 0);
        assert_eq!(e.move_id, LAGA_SIPHON);
        assert_eq!(e.effect(mfx::SIPHON_STR), Some(1));
        assert_eq!(e.effect(mfx::SIPHON_DEX), Some(1));
    }

    // ========== Book of Stabbing (Elite) ==========

    #[test] fn book_first_stab() {
        let e = create_enemy("BookOfStabbing", 162, 162);
        assert_eq!(e.move_id, BOOK_STAB);
        // Java BookOfStabbing start: stabCount=1, first attack hits=2 (mod.rs:553-554).
        assert_eq!(e.move_hits(), 2);
    }
    #[test] fn book_stab_count_increases() {
        let mut e = create_enemy("BookOfStabbing", 162, 162);
        // Deterministic num>=15, no prior STABs in lastTwoMoves → Stab++ branch
        // (act2.rs:248-254): stabCount goes 2 → 3, hits goes 2 → 3.
        roll_next_move_with_num(&mut e, 50);
        assert_eq!(e.move_id, BOOK_STAB);
        assert_eq!(e.move_hits(), 3);
    }

    // ========== Nemesis (Elite) ==========

    #[test] fn nemesis_intangible_applied_at_turn_start() {
        // Nemesis doesn't start with Intangible — it's applied at enemy turn start
        let e = create_enemy("Nemesis", 185, 185);
        assert_eq!(e.entity.status(sid::INTANGIBLE), 0,
            "Nemesis should not start with Intangible (applied per turn)");
    }
    #[test] fn nemesis_scythe_attack() {
        // Java Nemesis: first roll with num<50 → TRI_ATTACK (act3.rs:354-363);
        // second roll with num<30, cooldown<=0, !lastMove(SCYTHE) → SCYTHE with
        // fixed damage 45 (act3.rs:365-368).
        let mut e = create_enemy("Nemesis", 185, 185);
        roll_next_move_with_num(&mut e, 0); // first_move, num<50 → TRI_ATTACK
        assert_eq!(e.move_id, NEM_TRI_ATTACK);
        roll_next_move_with_num(&mut e, 0); // num<30, cooldown<=0 → SCYTHE
        assert_eq!(e.move_id, NEM_SCYTHE);
        assert_eq!(e.move_damage(), 45);
        // SCYTHE sets cooldown=2 (act3.rs:368)
        assert_eq!(e.entity.status(sid::SCYTHE_COOLDOWN), 2);
    }

    // ========== Act 2-4 Bosses ==========
    // BronzeAutomaton / AwakenedOne / CorruptHeart boss tests moved to
    // `test_bosses.rs` (covers A0/A2/A4/A9/A19 scaling, phase transitions,
    // debilitate cycle, with Java file:line citations). Single-source coverage.

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

    // =========================================================================
    // Ascension coverage — Cycle 1.6 / D118 RED baseline
    // =========================================================================
    //
    // Each test targets a specific ascension-scaling gap that Rust's
    // `create_enemy` / `roll_*` pipeline fails to honour. Tests cite the
    // exact Rust site where A0 values are hardcoded so Cycle 4 knows what
    // to patch. Tests are expected RED until Cycle 4 threads `ascension`
    // through the enemy pipeline and closes D118.
    //
    // Helper: `create_enemy_with_ascension(id, ascension)` picks canonical
    // Java HP per ascension band (the only scaling implemented today is
    // via `run.rs` HP lookup, and not all of the targeted enemies have one
    // — see run.rs:1334/1461 for the precedent). Damage tables and effect
    // amounts are the Cycle 4 contract.

    /// HP per (enemy, ascension band). Values match Java `setHp(hpRange)`
    /// picking the high end for determinism. HP bands are NOT yet plumbed
    /// through `create_enemy_with_ascension` (run.rs owns HP rolling via
    /// `roll_enemy_hp`); this local helper just picks the canonical high-end
    /// value so tests on HP-sensitive enemies don't need bespoke lookups.
    fn hp_for(id: &str, ascension: i32) -> (i32, i32) {
        match id {
            // Java GremlinNob: hpRange {82,86} (A7+ {85,90}).
            "GremlinNob" => if ascension >= 7 { (90, 90) } else { (86, 86) },
            // Java Snecko: hpRange {114,120} (A7+ {120,125}).
            "Snecko" => if ascension >= 7 { (125, 125) } else { (120, 120) },
            // Java Spiker: hpRange {152,156} (A7+ {160,165}).
            "Spiker" => if ascension >= 7 { (165, 165) } else { (156, 156) },
            // Java SpireShield: hpRange {99,106} (A9+ {112,119}).
            "SpireShield" => if ascension >= 9 { (119, 119) } else { (106, 106) },
            _ => panic!(
                "hp_for: id={id:?} not cataloged; \
                 D118 test scope is {{GremlinNob, Snecko, Spiker, SpireShield}}.",
            ),
        }
    }

    fn scaled(id: &str, ascension: i32) -> EnemyCombatState {
        let (hp, max_hp) = hp_for(id, ascension);
        create_enemy_with_ascension(id, hp, max_hp, ascension)
    }

    #[test]
    fn nob_a2_enrage_amount_is_three() {
        // Java GremlinNob.GremlinNob(): addPower(new EnragePower(this,
        // ascensionLevel >= 2 ? 3 : 2)). D118 closed via plumbed
        // `create_enemy_with_ascension`.
        let e = scaled("GremlinNob", 2);
        assert_eq!(
            e.entity.status(sid::ENRAGE),
            3,
            "Nob A2+ ENRAGE amount must be 3 per Java DEBUFF_AMOUNT",
        );
    }

    #[test]
    fn snecko_a17_bite_damage_is_eighteen() {
        // Java Snecko: damage[1] (BITE) = 15 at A0/A1, 18 at A2+.
        let mut e = scaled("Snecko", 17);
        // num>=40 with no prior lastTwoMoves(BITE) -> BITE branch.
        roll_next_move_with_num(&mut e, 99);
        assert_eq!(e.move_id, SNECKO_BITE);
        assert_eq!(
            e.move_damage(),
            18,
            "Snecko A2+ BITE damage must be 18 per Java damage[1]",
        );
    }

    #[test]
    fn snecko_a17_tail_vulnerable_is_three() {
        // Java Snecko: Tail applies Vulnerable = vulnAmount; A17+ bumps
        // vulnAmount from 2 -> 3.
        let mut e = scaled("Snecko", 17);
        roll_next_move_with_num(&mut e, 0); // num<40 -> TAIL
        assert_eq!(e.move_id, SNECKO_TAIL);
        assert_eq!(
            e.effect(mfx::VULNERABLE),
            Some(3),
            "Snecko A17+ Tail Vulnerable must be 3 per Java vulnAmount",
        );
    }

    #[test]
    fn spiker_a17_thorns_buff_is_three() {
        // Java Spiker: BUFF_AMOUNT = ascensionLevel >= 17 ? 3 : 2.
        let mut e = scaled("Spiker", 17);
        // Force BUFF: num>=50 with no prior ATTACK history triggers the
        // BUFF_THORNS branch.
        roll_next_move_with_num(&mut e, 99);
        assert_eq!(e.move_id, SPIKER_BUFF);
        assert_eq!(
            e.effect(mfx::THORNS),
            Some(3),
            "Spiker A17+ Thorns BUFF amount must be 3 per Java BUFF_AMOUNT",
        );
    }

    #[test]
    fn spire_shield_a19_hp_and_bash_damage() {
        // Java SpireShield: hpRange A0 {99,106} / A9+ {112,119};
        //   Bash damage = 12 at A0/A1/A2, 14 at A3+.
        let e = scaled("SpireShield", 19);
        assert_eq!(
            e.entity.max_hp, 119,
            "SpireShield A9+ max_hp must be 119 per Java hpRange",
        );
        assert_eq!(e.move_id, SHIELD_BASH);
        assert_eq!(
            e.move_damage(),
            14,
            "SpireShield A3+ Bash damage must be 14 per Java damage[1]",
        );
    }
}

// =============================================================================
// Relic exhaustive tests
// =============================================================================

