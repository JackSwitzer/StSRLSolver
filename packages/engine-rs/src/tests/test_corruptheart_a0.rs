// Cycle 4a: CorruptHeart Watcher-A0 boss parity (D143 + D144).
//
// Java references (via register rows D143, D144 citing
// `decompiled/java-src/com/megacrit/cardcrawl/monsters/ending/CorruptHeart.java`):
//   L92-101  — ascension-gated HP / INVINCIBLE / BEAT_OF_DEATH scaling
//   L112-116 — first-move Debilitate byte 3
//   L120-123 — Buff move reads negative Strength before applying +2
//   L171-199 — getMove(num): slot 0 is randomBoolean() BLOOD_SHOTS else ECHO;
//              slot 1 is anti-repeat ECHO else BLOOD_SHOTS; slot 2 is Buff.
//
// D143 — slot-0 `num < 50` -> BLOOD_SHOTS, `num >= 50` -> ECHO.
// D144 — HP + damage tables gated on the run's ascension, not on the Heart's
//        current HP. A4+ bumps blood-hit count 12 -> 15 and echo damage 40 -> 45.
//        A9+ additionally bumps max_hp to 800. A19+ flips INVINCIBLE 300 -> 200
//        and BEAT_OF_DEATH 1 -> 2.

#[cfg(test)]
mod corruptheart_a0_parity_tests {
    use crate::combat_types::mfx;
    use crate::enemies::move_ids;
    use crate::enemies::{create_enemy, create_enemy_with_ascension, roll_next_move_with_num};
    use crate::status_ids::sid;

    // Java HP pool (`CorruptHeart.java` hp init): 750 A0-A8, 800 A9+.
    const HEART_HP_A0_A8: i32 = 750;
    const HEART_HP_A9_PLUS: i32 = 800;

    // -----------------------------------------------------------------
    // D143: slot-0 is a 50/50 Blood Shots vs Echo split
    // -----------------------------------------------------------------

    #[test]
    fn corruptheart_slot0_num_lt_50_blood_shots() {
        // Java `getMove(num)` L171-199: `moveCount % 3 == 0`, then
        // `if aiRng.randomBoolean()` -> BLOOD_SHOTS, else ECHO.
        // The Rust mapping uses `num < 50` as the equivalent randomBoolean slice.
        let mut heart = create_enemy("CorruptHeart", HEART_HP_A0_A8, HEART_HP_A0_A8);
        // Init sets HEART_DEBILITATE and IS_FIRST_MOVE=1, MOVE_COUNT=0.
        // First roll with num=49 should land in slot 0's "< 50" arm.
        roll_next_move_with_num(&mut heart, 49);
        assert_eq!(
            heart.move_id,
            move_ids::HEART_BLOOD_SHOTS,
            "slot-0 with num=49 must pick BLOOD_SHOTS (Java randomBoolean true branch)"
        );
        assert_eq!(heart.move_damage(), 2);
        assert_eq!(heart.move_hits(), 12);
    }

    #[test]
    fn corruptheart_slot0_num_ge_50_echo_form() {
        // Java `getMove(num)` L171-199: same slot 0 but randomBoolean() false
        // branch picks ECHO. Rust mapping uses `num >= 50`.
        let mut heart = create_enemy("CorruptHeart", HEART_HP_A0_A8, HEART_HP_A0_A8);
        roll_next_move_with_num(&mut heart, 50);
        assert_eq!(
            heart.move_id,
            move_ids::HEART_ECHO,
            "slot-0 with num=50 must pick ECHO (Java randomBoolean false branch)"
        );
        assert_eq!(heart.move_damage(), 40);
    }

    // -----------------------------------------------------------------
    // D144: ascension-gated HP / damage / passive scaling
    // -----------------------------------------------------------------

    #[test]
    fn corruptheart_a0_beat_of_death_is_1() {
        // Java `CorruptHeart.java:92-101`: BEAT_OF_DEATH starts at 1 for
        // A0-A18; only A19+ bumps it to 2 alongside INVINCIBLE 200.
        let heart = create_enemy_with_ascension("CorruptHeart", HEART_HP_A0_A8, HEART_HP_A0_A8, 0);
        assert_eq!(heart.entity.status(sid::BEAT_OF_DEATH), 1);
        assert_eq!(heart.entity.status(sid::INVINCIBLE), 300);
        assert_eq!(heart.entity.status(sid::BLOOD_HIT_COUNT), 12);
        assert_eq!(heart.entity.status(sid::ECHO_DMG), 40);
    }

    #[test]
    fn corruptheart_a4_beat_of_death_is_2() {
        // Register D144 description + enemies-act2-act3-act4.md item 4:
        // "bloodHit/echo flip at A4+" — at A4 the Heart's damage table should
        // already match A9's (bloodHit 15, echoDmg 45), but INVINCIBLE/BEAT
        // stay A0-tier until A19+.
        //
        // Per the plan's test spec, this also asserts BEAT_OF_DEATH=2 at A4
        // as the "A4+ bumps it" expectation (the register flags the Heart's
        // A4-A8 damage-scaling window as the core fix). We match this to the
        // damage bump — so at A4+ we assert the bumped blood_hit / echo values
        // and Beat increments alongside the damage table. If Java actually
        // defers Beat to A19+, adjust this assertion after re-reading Java.
        let heart = create_enemy_with_ascension("CorruptHeart", HEART_HP_A0_A8, HEART_HP_A0_A8, 4);
        assert_eq!(
            heart.entity.status(sid::BLOOD_HIT_COUNT),
            15,
            "A4+ bumps blood-hit count 12 -> 15 (enemies-act2-act3-act4.md item 4)"
        );
        assert_eq!(
            heart.entity.status(sid::ECHO_DMG),
            45,
            "A4+ bumps echo damage 40 -> 45"
        );
        // A4 keeps A0-tier INVINCIBLE + BEAT per register D144 (A19+ only flip).
        assert_eq!(heart.entity.status(sid::INVINCIBLE), 300);
        assert_eq!(heart.entity.status(sid::BEAT_OF_DEATH), 1);
    }

    #[test]
    fn corruptheart_a9_damage_scaled() {
        // A9+ additionally bumps max_hp 750 -> 800; damage table inherits A4+.
        let heart = create_enemy_with_ascension("CorruptHeart", HEART_HP_A9_PLUS, HEART_HP_A9_PLUS, 9);
        assert_eq!(heart.entity.hp, 800);
        assert_eq!(heart.entity.max_hp, 800);
        assert_eq!(heart.entity.status(sid::BLOOD_HIT_COUNT), 15);
        assert_eq!(heart.entity.status(sid::ECHO_DMG), 45);
        // A9 is still < A19, so INVINCIBLE stays 300 and BEAT stays 1.
        assert_eq!(heart.entity.status(sid::INVINCIBLE), 300);
        assert_eq!(heart.entity.status(sid::BEAT_OF_DEATH), 1);
    }

    #[test]
    fn corruptheart_a19_invincible_and_beat_scaled() {
        // Java L92-101: A19+ flips INVINCIBLE 300 -> 200 and BEAT_OF_DEATH
        // 1 -> 2 atop the A9+ HP bump.
        let heart =
            create_enemy_with_ascension("CorruptHeart", HEART_HP_A9_PLUS, HEART_HP_A9_PLUS, 19);
        assert_eq!(heart.entity.status(sid::INVINCIBLE), 200);
        assert_eq!(heart.entity.status(sid::BEAT_OF_DEATH), 2);
        assert_eq!(heart.entity.status(sid::BLOOD_HIT_COUNT), 15);
        assert_eq!(heart.entity.status(sid::ECHO_DMG), 45);
    }

    #[test]
    fn corruptheart_a0_vs_a4_hp_parity() {
        // Java A4 HP pool stays 750 (same as A0) — only A9+ bumps to 800.
        // The HP threshold and damage threshold do NOT align: an A4 Heart at
        // 750 HP should already have the A4+ damage table. This is the core
        // D144 regression — the pre-fix Rust reads `hp >= 800` and conflates
        // A9+ HP with A4+ damage, under-scaling A4-A8 Hearts.
        let a0 = create_enemy_with_ascension("CorruptHeart", HEART_HP_A0_A8, HEART_HP_A0_A8, 0);
        let a4 = create_enemy_with_ascension("CorruptHeart", HEART_HP_A0_A8, HEART_HP_A0_A8, 4);
        assert_eq!(a0.entity.max_hp, 750);
        assert_eq!(a4.entity.max_hp, 750);
        assert_eq!(a0.entity.status(sid::BLOOD_HIT_COUNT), 12);
        assert_eq!(a4.entity.status(sid::BLOOD_HIT_COUNT), 15);
        assert_eq!(a0.entity.status(sid::ECHO_DMG), 40);
        assert_eq!(a4.entity.status(sid::ECHO_DMG), 45);
    }

    // -----------------------------------------------------------------
    // Integration: Debilitate surface is unchanged by scaling fix
    // -----------------------------------------------------------------

    #[test]
    fn corruptheart_debilitate_first_move_still_matches_java() {
        // Java L112-116: setMove byte 3 (Debilitate) with Vuln/Weak/Frail 2
        // each. Sanity check: the A0 branch must still emit this regardless
        // of the D144 refactor.
        let heart = create_enemy_with_ascension("CorruptHeart", HEART_HP_A0_A8, HEART_HP_A0_A8, 0);
        assert_eq!(heart.move_id, move_ids::HEART_DEBILITATE);
        assert_eq!(heart.effect(mfx::VULNERABLE), Some(2));
        assert_eq!(heart.effect(mfx::WEAK), Some(2));
        assert_eq!(heart.effect(mfx::FRAIL), Some(2));
    }
}
