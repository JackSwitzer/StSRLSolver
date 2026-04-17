#[cfg(test)]
mod interaction_tests {
    use crate::tests::support::*;
    use crate::status_ids::sid;
    use crate::state::Stance;

    // =========================================================================
    // 1. Strength + Multi-Hit: Pummel
    //    3 Str + Pummel (2 dmg x4) => each hit = 2+3 = 5, total = 20
    // =========================================================================
    #[test]
    fn strength_plus_multi_hit_pummel() {
        let mut engine = engine_with(make_deck_n("Pummel", 5), 50, 0);
        engine.state.player.set_status(sid::STRENGTH, 3);
        ensure_in_hand(&mut engine, "Pummel");
        play_on_enemy(&mut engine, "Pummel", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 30,
            "Pummel with 3 Str: 4 * (2+3) = 20 damage, 50-20 = 30");
    }

    // =========================================================================
    // 1b. Strength + Multi-Hit: Twin Strike
    //     3 Str + Twin Strike (5 dmg x2) => each hit = 5+3 = 8, total = 16
    // =========================================================================
    #[test]
    fn strength_plus_multi_hit_twin_strike() {
        let mut engine = engine_with(make_deck_n("Twin Strike", 5), 50, 0);
        engine.state.player.set_status(sid::STRENGTH, 3);
        ensure_in_hand(&mut engine, "Twin Strike");
        play_on_enemy(&mut engine, "Twin Strike", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 34,
            "Twin Strike with 3 Str: 2 * (5+3) = 16 damage, 50-16 = 34");
    }

    // =========================================================================
    // 2. Vulnerable + Multi-Hit: Twin Strike
    //    Enemy Vulnerable + Twin Strike (5 dmg x2) => each hit = floor(5*1.5) = 7
    //    Total = 14
    // =========================================================================
    #[test]
    fn vulnerable_plus_multi_hit_twin_strike() {
        let mut engine = engine_with(make_deck_n("Twin Strike", 5), 50, 0);
        engine.state.enemies[0].entity.set_status(sid::VULNERABLE, 2);
        ensure_in_hand(&mut engine, "Twin Strike");
        play_on_enemy(&mut engine, "Twin Strike", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 36,
            "Twin Strike + Vuln: 2 * floor(5*1.5) = 2*7 = 14 damage, 50-14 = 36");
    }

    // =========================================================================
    // 2b. Vulnerable + Multi-Hit: Pummel
    //     Enemy Vulnerable + Pummel (2 dmg x4) => each hit = floor(2*1.5) = 3
    //     Total = 12
    // =========================================================================
    #[test]
    fn vulnerable_plus_multi_hit_pummel() {
        let mut engine = engine_with(make_deck_n("Pummel", 5), 50, 0);
        engine.state.enemies[0].entity.set_status(sid::VULNERABLE, 2);
        ensure_in_hand(&mut engine, "Pummel");
        play_on_enemy(&mut engine, "Pummel", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 38,
            "Pummel + Vuln: 4 * floor(2*1.5) = 4*3 = 12 damage, 50-12 = 38");
    }

    // =========================================================================
    // 3. Strength + Vulnerable + Multi-Hit: Twin Strike
    //    3 Str + enemy Vuln + Twin Strike (5 dmg x2)
    //    Each hit = floor((5+3) * 1.5) = floor(12) = 12, total = 24
    // =========================================================================
    #[test]
    fn strength_vulnerable_multi_hit_twin_strike() {
        let mut engine = engine_with(make_deck_n("Twin Strike", 5), 50, 0);
        engine.state.player.set_status(sid::STRENGTH, 3);
        engine.state.enemies[0].entity.set_status(sid::VULNERABLE, 2);
        ensure_in_hand(&mut engine, "Twin Strike");
        play_on_enemy(&mut engine, "Twin Strike", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 26,
            "Twin Strike + 3 Str + Vuln: 2 * floor((5+3)*1.5) = 2*12 = 24, 50-24 = 26");
    }

    // =========================================================================
    // 4. Pen Nib + Wrath (verify existing test's values)
    //    Wrath (2x) + Pen Nib (2x) + Strike_R (6 dmg) = 6*2*2 = 24
    // =========================================================================
    #[test]
    fn pen_nib_in_wrath_strike() {
        let mut engine = engine_with(make_deck_n("Strike_R", 5), 50, 0);
        engine.state.stance = Stance::Wrath;
        engine.state.relics.push("Pen Nib".to_string());
        engine.state.player.set_status(sid::PEN_NIB_COUNTER, 9);
        ensure_in_hand(&mut engine, "Strike_R");
        play_on_enemy(&mut engine, "Strike_R", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 26,
            "Strike in Wrath + Pen Nib: 6 * 2 * 2 = 24 damage, 50-24 = 26");
    }

    // =========================================================================
    // 5. Corruption + Skills cost 0 and exhaust
    //    Play Corruption, then Defend_R (5 block, 1 cost Skill).
    //    Defend costs 0, gives 5 block, goes to exhaust pile.
    // =========================================================================
    #[test]
    fn corruption_skills_free_and_exhaust() {
        let mut engine = engine_with(
            make_deck(&["Corruption", "Defend_R", "Defend_R", "Defend_R", "Defend_R", "Strike_R"]),
            50, 0,
        );
        ensure_in_hand(&mut engine, "Corruption");
        // Corruption costs 3 energy
        play_self(&mut engine, "Corruption");
        assert_eq!(engine.state.player.status(sid::CORRUPTION), 1,
            "Corruption should set CORRUPTION status to 1");
        let energy_after_corruption = engine.state.energy; // 3 - 3 = 0

        // Add a Defend to hand and play it
        ensure_in_hand(&mut engine, "Defend_R");
        play_self(&mut engine, "Defend_R");

        // Defend should cost 0 (Corruption makes skills free)
        assert_eq!(engine.state.energy, energy_after_corruption,
            "Defend should cost 0 energy under Corruption");

        // Player should have 5 block
        assert_eq!(engine.state.player.block, 5,
            "Defend should still give 5 block");

        // Defend should be in exhaust pile, not discard
        assert_eq!(exhaust_prefix_count(&engine, "Defend_R"), 1,
            "Defend should be exhausted under Corruption");
    }

    // =========================================================================
    // 6. Barricade + Block Retention
    //    Play Barricade, gain block, end turn => block stays.
    // =========================================================================
    #[test]
    fn barricade_block_retention() {
        // Use filler deck so only desired cards are played
        let mut engine = engine_with(make_deck_n("Strike_R", 10), 50, 0);
        // Give enough energy and set status directly
        engine.state.energy = 10;
        engine.state.player.set_status(sid::BARRICADE, 1);
        engine.state.player.block = 10;

        // End turn (enemy does 0 damage), then start of next turn
        end_turn(&mut engine);

        // Block should be retained (Barricade prevents block decay)
        assert_eq!(engine.state.player.block, 10,
            "Block should be retained with Barricade active");
    }

    // =========================================================================
    // 6b. Without Barricade, block decays
    // =========================================================================
    #[test]
    fn block_decays_without_barricade() {
        let mut engine = engine_with(make_deck_n("Defend_R", 10), 50, 0);
        ensure_in_hand(&mut engine, "Defend_R");
        play_self(&mut engine, "Defend_R");
        assert_eq!(engine.state.player.block, 5);

        end_turn(&mut engine);

        // Block should decay to 0 without Barricade
        assert_eq!(engine.state.player.block, 0,
            "Block should decay to 0 without Barricade");
    }

    // =========================================================================
    // 7. Echo Form + Attack Replay
    //    Set ECHO_FORM to 1, play Strike (6 dmg), it fires twice = 12 total.
    // =========================================================================
    #[test]
    fn echo_form_attack_replay() {
        let mut engine = engine_with(make_deck_n("Strike_R", 5), 50, 0);
        engine.state.player.set_status(sid::ECHO_FORM, 1);
        let hand_before = engine.state.hand.len();
        ensure_in_hand(&mut engine, "Strike_R");
        play_on_enemy(&mut engine, "Strike_R", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 38,
            "Echo Form + Strike: 6*2 = 12 damage, 50-12 = 38");
        // Only 1 card should be removed from hand
        assert_eq!(engine.state.hand.len(), hand_before - 1,
            "Echo Form should only consume 1 card from hand");
    }

    // =========================================================================
    // 8. Double Tap + Attack
    //    Play Double Tap, then Strike (6 dmg) => Strike fires twice = 12 total.
    // =========================================================================
    #[test]
    fn double_tap_attack_replay() {
        let mut engine = engine_with(
            make_deck(&["Double Tap", "Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R"]),
            50, 0,
        );
        ensure_in_hand(&mut engine, "Double Tap");
        play_self(&mut engine, "Double Tap");
        assert_eq!(engine.state.player.status(sid::DOUBLE_TAP), 1,
            "Double Tap should set status to 1");

        ensure_in_hand(&mut engine, "Strike_R");
        play_on_enemy(&mut engine, "Strike_R", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 38,
            "Double Tap + Strike: 6*2 = 12 damage, 50-12 = 38");

        // Double Tap status should be consumed
        assert_eq!(engine.state.player.status(sid::DOUBLE_TAP), 0,
            "Double Tap status should be consumed after use");
    }

    // =========================================================================
    // 9. Feel No Pain + Exhaust
    //    Set Feel No Pain status to 3 directly. Then play True Grit (7 block + exhaust random).
    //    Total block = 7 (True Grit) + 3 (Feel No Pain) = 10
    // =========================================================================
    #[test]
    fn feel_no_pain_exhaust_trigger() {
        // Use filler deck to avoid accidental card draws
        let mut engine = engine_with(make_deck_n("Strike_R", 10), 50, 0);
        engine.state.energy = 5;

        // Set Feel No Pain directly to avoid the draw-duplication issue
        engine.state.player.set_status(sid::FEEL_NO_PAIN, 3);

        // Make sure there's True Grit in hand and at least one other card to exhaust
        ensure_in_hand(&mut engine, "True Grit");
        ensure_in_hand(&mut engine, "Strike_R");
        play_self(&mut engine, "True Grit");

        // True Grit gives 7 block + Feel No Pain triggers on exhaust = 3 more block
        assert_eq!(engine.state.player.block, 10,
            "True Grit (7) + Feel No Pain (3) = 10 total block");
    }

    // =========================================================================
    // 10. Noxious Fumes + Turn Tick
    //     Set Noxious Fumes status directly. End turn, start next turn.
    //     At player turn start: all enemies gain 2 poison.
    //     During enemy turn: poison ticks (damage + decrement).
    // =========================================================================
    #[test]
    fn noxious_fumes_turn_tick() {
        // Use filler deck to avoid accidental Noxious Fumes draws
        let mut engine = engine_with(make_deck_n("Strike_R", 10), 50, 0);
        engine.state.energy = 5;

        // Set Noxious Fumes status directly
        engine.state.player.set_status(sid::NOXIOUS_FUMES, 2);

        // Give player enough block to survive enemy attack
        engine.state.player.block = 100;

        end_turn(&mut engine);

        // Turn cycle: EndTurn -> enemy turn (poison ticks) -> new player turn (Noxious Fumes fires)
        // Since there was no poison on the enemy before, poison tick during enemy turn does nothing.
        // At start of new player turn, Noxious Fumes applies 2 poison.
        // After full cycle: enemy has 2 poison, HP unchanged.
        assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 2,
            "Enemy should have 2 poison after first turn cycle with Noxious Fumes");
        let hp_after_first = engine.state.enemies[0].entity.hp;

        // Give more block for second cycle
        engine.state.player.block = 100;
        end_turn(&mut engine);

        // Second cycle:
        //   Enemy turn: poison ticks (2 dmg to enemy, poison decrements 2->1)
        //   New player turn: Noxious Fumes adds 2 more poison (1+2=3)
        assert_eq!(engine.state.enemies[0].entity.hp, hp_after_first - 2,
            "Enemy should take 2 poison damage during second turn");
        assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 3,
            "Enemy should have 3 poison (1 remaining + 2 from Noxious Fumes)");
    }

    // =========================================================================
    // 11. Blade Dance + Accuracy
    //     Set Accuracy status directly, then play Blade Dance (add 3 Shivs).
    //     Play Shiv: 4 base + 4 Accuracy = 8 damage.
    // =========================================================================
    #[test]
    fn blade_dance_accuracy_shiv() {
        // Use filler deck to avoid drawing Accuracy/BladeDance naturally
        let mut engine = engine_with(make_deck_n("Strike_G", 10), 50, 0);
        engine.state.energy = 5;

        // Set Accuracy status directly
        engine.state.player.set_status(sid::ACCURACY, 4);

        ensure_in_hand(&mut engine, "Blade Dance");
        play_self(&mut engine, "Blade Dance");

        // Should have 3 Shivs in hand
        let shiv_count = hand_count(&engine, "Shiv");
        assert!(shiv_count >= 1, "Should have at least 1 Shiv in hand after Blade Dance");

        // Play a Shiv on enemy
        play_on_enemy(&mut engine, "Shiv", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 42,
            "Shiv + Accuracy: 4 + 4 = 8 damage, 50-8 = 42");
    }

    // =========================================================================
    // 12. Flurry of Blows -- Return from Discard on Stance Change
    //     Put FlurryOfBlows in discard, change stance => returns to hand.
    //     NOTE: This may fail if the mechanic isn't implemented.
    // =========================================================================
    #[test]
    fn flurry_of_blows_return_on_stance_change() {
        let mut engine = engine_with(
            make_deck(&["Eruption", "FlurryOfBlows", "Strike_P", "Strike_P", "Strike_P",
                        "Defend_P", "Defend_P", "Defend_P", "Defend_P", "Defend_P"]),
            50, 0,
        );
        // Move FlurryOfBlows to discard pile manually
        let flurry_card = engine.card_registry.make_card("FlurryOfBlows");
        engine.state.discard_pile.push(flurry_card);

        let discard_flurry_before = discard_prefix_count(&engine, "FlurryOfBlows");
        assert!(discard_flurry_before >= 1, "FlurryOfBlows should be in discard");

        // Play Eruption to enter Wrath (stance change)
        ensure_in_hand(&mut engine, "Eruption");
        play_on_enemy(&mut engine, "Eruption", 0);

        // FlurryOfBlows should return from discard to hand
        let hand_flurry_after = hand_count(&engine, "FlurryOfBlows");
        assert!(hand_flurry_after >= 1,
            "FlurryOfBlows should return to hand from discard on stance change");
    }

    // =========================================================================
    // 12b. Two FlurryOfBlows in discard: both return
    // =========================================================================
    #[test]
    fn flurry_of_blows_two_return_on_stance_change() {
        let mut engine = engine_with(
            make_deck(&["Eruption", "Strike_P", "Strike_P", "Strike_P", "Strike_P",
                        "Defend_P", "Defend_P", "Defend_P", "Defend_P", "Defend_P"]),
            50, 0,
        );
        // Put 2 FlurryOfBlows in discard
        engine.state.discard_pile.push(engine.card_registry.make_card("FlurryOfBlows"));
        engine.state.discard_pile.push(engine.card_registry.make_card("FlurryOfBlows"));

        ensure_in_hand(&mut engine, "Eruption");
        play_on_enemy(&mut engine, "Eruption", 0);

        let hand_flurry = hand_count(&engine, "FlurryOfBlows");
        assert_eq!(hand_flurry, 2,
            "Both FlurryOfBlows should return to hand from discard");
    }

    // =========================================================================
    // 13. Heavy Blade + Strength Scaling
    //     4 Str + Heavy Blade (14 dmg, 3x str) => 14 + (4*3) = 26
    // =========================================================================
    #[test]
    fn heavy_blade_strength_scaling() {
        let mut engine = engine_with(make_deck_n("Heavy Blade", 5), 50, 0);
        engine.state.player.set_status(sid::STRENGTH, 4);
        ensure_in_hand(&mut engine, "Heavy Blade");
        play_on_enemy(&mut engine, "Heavy Blade", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 24,
            "Heavy Blade + 4 Str: 14 + (4*3) = 26 damage, 50-26 = 24");
    }

    // =========================================================================
    // 13b. Heavy Blade+ (5x str scaling)
    //      4 Str + Heavy Blade+ (14 dmg, 5x str) => 14 + (4*5) = 34
    // =========================================================================
    #[test]
    fn heavy_blade_plus_strength_scaling() {
        let mut engine = engine_with(make_deck_n("Heavy Blade+", 5), 50, 0);
        engine.state.player.set_status(sid::STRENGTH, 4);
        engine.state.energy = 5; // enough energy
        ensure_in_hand(&mut engine, "Heavy Blade+");
        play_on_enemy(&mut engine, "Heavy Blade+", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 16,
            "Heavy Blade+ + 4 Str: 14 + (4*5) = 34 damage, 50-34 = 16");
    }

    // =========================================================================
    // 14. Whirlwind X-Cost AoE
    //     3 energy + Whirlwind (5 dmg AoE x X) => X=3, each enemy takes 3*5=15
    // =========================================================================
    #[test]
    fn whirlwind_x_cost_aoe() {
        let enemies = vec![
            enemy("Louse1", 50, 50, 1, 0, 1),
            enemy("Louse2", 50, 50, 1, 0, 1),
        ];
        let mut engine = engine_with_enemies(make_deck_n("Whirlwind", 5), enemies, 3);
        ensure_in_hand(&mut engine, "Whirlwind");
        play_on_enemy(&mut engine, "Whirlwind", 0);

        // X = 3 (all energy), energy drops to 0
        assert_eq!(engine.state.energy, 0,
            "Whirlwind should consume all energy");
        assert_eq!(engine.state.enemies[0].entity.hp, 35,
            "Whirlwind: each enemy takes 3 * 5 = 15 damage, 50-15 = 35");
        assert_eq!(engine.state.enemies[1].entity.hp, 35,
            "Whirlwind: second enemy also takes 15 damage");
    }

    // =========================================================================
    // 15. Corruption: attacks still cost energy normally
    //     Play Corruption (3 cost), then Strike_R (1 cost attack).
    //     Strike should cost 1 energy and go to discard (not exhaust).
    // =========================================================================
    #[test]
    fn corruption_attack_costs_normally() {
        let mut engine = engine_with(
            make_deck(&["Corruption", "Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R"]),
            50, 0,
        );
        engine.state.energy = 5; // enough for Corruption (3) + Strike (1)
        ensure_in_hand(&mut engine, "Corruption");
        play_self(&mut engine, "Corruption");
        let energy_after_corruption = engine.state.energy; // 5 - 3 = 2

        ensure_in_hand(&mut engine, "Strike_R");
        play_on_enemy(&mut engine, "Strike_R", 0);

        // Strike should cost 1 energy (Corruption only affects Skills)
        assert_eq!(engine.state.energy, energy_after_corruption - 1,
            "Strike should cost 1 energy even with Corruption active");

        // Strike should go to discard, not exhaust
        assert_eq!(discard_prefix_count(&engine, "Strike_R"), 1,
            "Strike should go to discard pile, not exhaust");
        assert_eq!(exhaust_prefix_count(&engine, "Strike_R"), 0,
            "Strike should NOT be exhausted under Corruption");
    }

    // =========================================================================
    // 16. Demon Form: +2 Strength each turn
    //     Play Demon Form (sets DEMON_FORM=2). End turn, start next.
    //     After 1 cycle: 2 Str. After 2 cycles: 4 Str.
    // =========================================================================
    #[test]
    fn demon_form_gains_strength_each_turn() {
        let mut engine = engine_with(make_deck_n("Strike_R", 10), 50, 0);
        engine.state.energy = 5;

        // Set Demon Form status directly (avoids needing 3 energy for the card)
        engine.state.player.set_status(sid::DEMON_FORM, 2);

        assert_eq!(engine.state.player.status(sid::STRENGTH), 0,
            "Should start with 0 Strength");

        // Give block to survive any enemy damage
        engine.state.player.block = 100;
        end_turn(&mut engine);

        // After first turn cycle: Demon Form fires at start of player turn
        assert_eq!(engine.state.player.status(sid::STRENGTH), 2,
            "After 1 turn cycle, Demon Form should grant 2 Strength");

        engine.state.player.block = 100;
        end_turn(&mut engine);

        // After second turn cycle: cumulative 4 Strength
        assert_eq!(engine.state.player.status(sid::STRENGTH), 4,
            "After 2 turn cycles, Demon Form should grant 4 total Strength");
    }

    // =========================================================================
    // 17. After Image: +1 block per card played
    //     Set AFTER_IMAGE=1. Play 3 Strike_R cards.
    //     After Image triggers on each card play, granting 1 block each.
    //     Total block = 3 (from After Image only; Strikes give no block).
    // =========================================================================
    #[test]
    fn after_image_blocks_per_card() {
        let mut engine = engine_with(make_deck_n("Strike_R", 10), 50, 0);
        engine.state.energy = 5;

        // Set After Image status directly
        engine.state.player.set_status(sid::AFTER_IMAGE, 1);

        assert_eq!(engine.state.player.block, 0, "Should start with 0 block");

        // Play 3 Strikes (attacks, no block from card itself)
        ensure_in_hand(&mut engine, "Strike_R");
        play_on_enemy(&mut engine, "Strike_R", 0);
        ensure_in_hand(&mut engine, "Strike_R");
        play_on_enemy(&mut engine, "Strike_R", 0);
        ensure_in_hand(&mut engine, "Strike_R");
        play_on_enemy(&mut engine, "Strike_R", 0);

        // After Image grants 1 block per card played = 3 block total
        assert_eq!(engine.state.player.block, 3,
            "After Image should grant 1 block per card played (3 cards = 3 block)");
    }

    // =========================================================================
    // 18. Burst: replays next Skill
    //     Play Burst, then Backflip (5 block, draw 2).
    //     Burst replays the skill: 5+5=10 block, 2+2=4 draws.
    // =========================================================================
    #[test]
    fn burst_replays_skill() {
        let mut engine = engine_with(
            make_deck(&["Burst", "Backflip", "Strike_R", "Strike_R", "Strike_R",
                        "Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R"]),
            50, 0,
        );
        engine.state.energy = 5;

        ensure_in_hand(&mut engine, "Burst");
        play_self(&mut engine, "Burst");
        assert_eq!(engine.state.player.status(sid::BURST), 1,
            "Burst should set BURST status to 1");

        let _hand_before = engine.state.hand.len();
        ensure_in_hand(&mut engine, "Backflip");
        play_self(&mut engine, "Backflip");

        // Backflip: 5 block + draw 2, replayed by Burst: another 5 block + draw 2
        assert_eq!(engine.state.player.block, 10,
            "Burst + Backflip: 5 + 5 = 10 block");

        // Burst status should be consumed
        assert_eq!(engine.state.player.status(sid::BURST), 0,
            "Burst status should be consumed after replaying a skill");
    }

    // =========================================================================
    // 19. Noxious Fumes: 2 enemies, each gets 2 poison
    //     Set NOXIOUS_FUMES=2 on player. End turn.
    //     At start of new player turn, both enemies get 2 poison.
    // =========================================================================
    #[test]
    fn noxious_fumes_two_enemies() {
        let enemies = vec![
            enemy("Louse1", 50, 50, 1, 0, 1),
            enemy("Louse2", 50, 50, 1, 0, 1),
        ];
        let mut engine = engine_with_enemies(make_deck_n("Strike_R", 10), enemies, 3);
        engine.state.energy = 5;

        // Set Noxious Fumes status directly
        engine.state.player.set_status(sid::NOXIOUS_FUMES, 2);

        engine.state.player.block = 100;
        end_turn(&mut engine);

        // At start of new player turn, Noxious Fumes applies 2 poison to all enemies
        assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 2,
            "First enemy should have 2 poison from Noxious Fumes");
        assert_eq!(engine.state.enemies[1].entity.status(sid::POISON), 2,
            "Second enemy should have 2 poison from Noxious Fumes");
    }

    // =========================================================================
    // 20. Wrath doubles outgoing damage
    //     Enter Wrath stance, play Strike_R (6 base).
    //     Expected: 6 * 2 = 12 damage.
    // =========================================================================
    #[test]
    fn wrath_doubles_damage() {
        let mut engine = engine_with(make_deck_n("Strike_R", 5), 50, 0);
        engine.state.stance = Stance::Wrath;
        ensure_in_hand(&mut engine, "Strike_R");
        play_on_enemy(&mut engine, "Strike_R", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 38,
            "Strike in Wrath: 6 * 2 = 12 damage, 50-12 = 38");
    }

    // =========================================================================
    // 21. Divinity triples outgoing damage
    //     Enter Divinity stance, play Strike_R (6 base).
    //     Expected: 6 * 3 = 18 damage.
    // =========================================================================
    #[test]
    fn divinity_triples_damage() {
        let mut engine = engine_with(make_deck_n("Strike_R", 5), 50, 0);
        engine.state.stance = Stance::Divinity;
        ensure_in_hand(&mut engine, "Strike_R");
        play_on_enemy(&mut engine, "Strike_R", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 32,
            "Strike in Divinity: 6 * 3 = 18 damage, 50-18 = 32");
    }
}
