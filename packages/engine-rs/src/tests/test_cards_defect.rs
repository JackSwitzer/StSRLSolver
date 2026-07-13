#[cfg(test)]
mod defect_card_java_parity_tests {
    // Java references:
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/cards/blue/*.java

    use crate::cards::{CardRegistry, CardType};
    use crate::combat_types::Intent;
    use crate::status_ids::sid;
    use crate::engine::{CombatEngine, CombatPhase};
    use crate::actions::Action;
    use crate::orbs::OrbType;
    use crate::powers::{process_end_of_round, process_end_of_turn, process_start_of_turn};
    use crate::state::EnemyCombatState;
    use crate::tests::support::*;

    macro_rules! defect_test {
        ($name:ident, $body:block) => {
            #[test]
            fn $name() $body
        };
    }

    #[derive(Clone, Copy)]
    struct StatCase {
        id: &'static str,
        cost: i32,
        damage: i32,
        block: i32,
        magic: i32,
        card_type: CardType,
        exhaust: bool,
    }

    fn reg() -> &'static CardRegistry {
        crate::cards::global_registry()
    }

    fn assert_stats(case: StatCase) {
        let registry = reg();
        let card = registry.get(case.id).unwrap();
        assert_eq!(card.cost, case.cost, "{} cost", case.id);
        assert_eq!(card.base_damage, case.damage, "{} damage", case.id);
        assert_eq!(card.base_block, case.block, "{} block", case.id);
        assert_eq!(card.base_magic, case.magic, "{} magic", case.id);
        assert_eq!(card.card_type, case.card_type, "{} type", case.id);
        assert_eq!(card.exhaust, case.exhaust, "{} exhaust", case.id);
    }

    fn filled_engine(card_ids: &[&str], enemy_hp: i32, enemy_dmg: i32) -> CombatEngine {
        let mut engine = engine_with(make_deck(card_ids), enemy_hp, enemy_dmg);
        force_player_turn(&mut engine);
        engine
    }

    fn bare_engine(card_ids: &[&str], enemies: Vec<EnemyCombatState>) -> CombatEngine {
        let mut engine = engine_without_start(make_deck(card_ids), enemies, 3);
        force_player_turn(&mut engine);
        engine
    }

    fn set_orbs(engine: &mut CombatEngine, orbs: &[OrbType]) {
        engine.init_defect_orbs(orbs.len().max(1));
        for orb in orbs {
            engine.channel_orb(*orb);
        }
    }

    fn enemy(id: &str, hp: i32, dmg: i32) -> EnemyCombatState {
        crate::tests::support::enemy(id, hp, hp, 1, dmg, 1)
    }

    // ------------------------------------------------------------------
    // Registry snapshots
    // ------------------------------------------------------------------

    defect_test!(registry_starter_and_basics, {
        let cases = [
            StatCase { id: "Strike", cost: 1, damage: 6, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Strike+", cost: 1, damage: 9, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Defend", cost: 1, damage: -1, block: 5, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Defend+", cost: 1, damage: -1, block: 8, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Zap", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Zap+", cost: 0, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Dualcast", cost: 1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Dualcast+", cost: 0, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    defect_test!(registry_orb_common, {
        let cases = [
            StatCase { id: "Ball Lightning", cost: 1, damage: 7, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Ball Lightning+", cost: 1, damage: 10, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Barrage", cost: 1, damage: 4, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Barrage+", cost: 1, damage: 6, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Beam Cell", cost: 0, damage: 3, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Beam Cell+", cost: 0, damage: 4, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Cold Snap", cost: 1, damage: 6, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Cold Snap+", cost: 1, damage: 9, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Compile Driver", cost: 1, damage: 7, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Compile Driver+", cost: 1, damage: 10, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Conserve Battery", cost: 1, damage: -1, block: 7, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Conserve Battery+", cost: 1, damage: -1, block: 10, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Coolheaded", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Coolheaded+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Go for the Eyes", cost: 0, damage: 3, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Go for the Eyes+", cost: 0, damage: 4, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Hologram", cost: 1, damage: -1, block: 3, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Hologram+", cost: 1, damage: -1, block: 5, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Leap", cost: 1, damage: -1, block: 9, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Leap+", cost: 1, damage: -1, block: 12, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Rebound", cost: 1, damage: 9, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Rebound+", cost: 1, damage: 12, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    defect_test!(registry_orb_utility, {
        let cases = [
            StatCase { id: "Stack", cost: 1, damage: -1, block: 0, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Stack+", cost: 1, damage: -1, block: 3, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Steam", cost: 0, damage: -1, block: 6, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Steam+", cost: 0, damage: -1, block: 8, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Streamline", cost: 2, damage: 15, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Streamline+", cost: 2, damage: 20, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Sweeping Beam", cost: 1, damage: 6, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Sweeping Beam+", cost: 1, damage: 9, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Turbo", cost: 0, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Turbo+", cost: 0, damage: -1, block: -1, magic: 3, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Gash", cost: 0, damage: 3, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Gash+", cost: 0, damage: 5, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    defect_test!(stack_snapshots_discard_size_and_modifies_the_combined_upgraded_block_once, {
        // Stack.applyPowers replaces baseBlock with the current discard-pile
        // size, adds 3 when upgraded, and only then runs the ordinary block
        // modifier pipeline. The played Stack reaches discard after use().
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Stack.java
        let mut base = bare_engine(&[], vec![enemy_no_intent("JawWorm", 40, 40)]);
        base.state.hand = make_deck(&["Stack"]);
        base.state.discard_pile = make_deck(&["Strike", "Defend"]);
        assert!(play_self(&mut base, "Stack"));
        assert_eq!(base.state.player.block, 2);
        assert_eq!(base.state.discard_pile.len(), 3);

        let mut upgraded = bare_engine(&[], vec![enemy_no_intent("JawWorm", 40, 40)]);
        upgraded.state.hand = make_deck(&["Stack+"]);
        upgraded.state.discard_pile = make_deck(&["Strike", "Defend", "Zap", "Dualcast", "Leap"]);
        upgraded.state.player.add_status(sid::DEXTERITY, 2);
        upgraded.state.player.add_status(sid::FRAIL, 1);
        assert!(play_self(&mut upgraded, "Stack+"));
        assert_eq!(upgraded.state.player.block, 7); // floor((5 + 3 + 2) * 0.75)
        assert_eq!(upgraded.state.discard_pile.len(), 6);
    });

    defect_test!(sweeping_beam_damages_all_enemies_before_its_single_draw, {
        // SweepingBeam.use queues 6 all-enemy damage before DrawCardAction(1);
        // the upgrade raises only damage to 9. DamageAllEnemiesAction clears
        // the queued draw when every monster dies.
        // Java: reference/extracted/methods/card/SweepingBeam.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAllEnemiesAction.java
        let mut surviving = bare_engine(
            &[],
            vec![enemy("JawWorm", 20, 0), enemy("Cultist", 30, 0)],
        );
        surviving.state.hand = make_deck(&["Sweeping Beam"]);
        surviving.state.draw_pile = make_deck(&["Strike"]);
        assert!(play_self(&mut surviving, "Sweeping Beam"));
        assert_eq!(surviving.state.enemies[0].entity.hp, 14);
        assert_eq!(surviving.state.enemies[1].entity.hp, 24);
        assert_eq!(hand_count(&surviving, "Strike"), 1);

        let mut lethal = bare_engine(
            &[],
            vec![enemy("JawWorm", 9, 0), enemy("Cultist", 9, 0)],
        );
        lethal.state.hand = make_deck(&["Sweeping Beam+"]);
        lethal.state.draw_pile = make_deck(&["Strike"]);
        assert!(play_self(&mut lethal, "Sweeping Beam+"));
        assert!(lethal.state.enemies.iter().all(|enemy| enemy.entity.hp == 0));
        assert_eq!(lethal.state.draw_pile.len(), 1);
        assert!(lethal.state.hand.is_empty());
    });

    defect_test!(hologram_matches_block_exhaust_and_discard_retrieval_edges, {
        // Hologram.java grants 3 Block, exhausts, then runs the mandatory
        // BetterDiscardPileToHandAction(1). That action is a no-op on an empty
        // pile and auto-moves a singleton. Hologram+ grants 5 and does not exhaust.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Hologram.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/BetterDiscardPileToHandAction.java
        let mut empty = bare_engine(
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
        );
        empty.state.hand = make_deck(&["Hologram"]);
        assert!(play_self(&mut empty, "Hologram"));
        assert_eq!(empty.phase, CombatPhase::PlayerTurn);
        assert_eq!(empty.state.player.block, 3);
        assert_eq!(exhaust_prefix_count(&empty, "Hologram"), 1);

        let mut singleton = bare_engine(
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
        );
        singleton.state.hand = make_deck(&["Hologram+"]);
        singleton.state.discard_pile = make_deck(&["Strike"]);
        assert!(play_self(&mut singleton, "Hologram+"));
        assert_eq!(singleton.phase, CombatPhase::PlayerTurn);
        assert_eq!(singleton.state.player.block, 5);
        assert_eq!(hand_count(&singleton, "Strike"), 1);
        assert_eq!(discard_prefix_count(&singleton, "Hologram"), 1);

        let mut multiple = bare_engine(
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
        );
        multiple.state.hand = make_deck(&["Hologram+"]);
        multiple.state.discard_pile = make_deck(&["Strike", "Defend"]);
        assert!(play_self(&mut multiple, "Hologram+"));
        assert_eq!(multiple.phase, CombatPhase::AwaitingChoice);
        assert_eq!(multiple.state.player.block, 5);
        assert_eq!(multiple.choice.as_ref().expect("discard choice").options.len(), 2);
        multiple.execute_action(&Action::Choose(0));
        assert_eq!(multiple.phase, CombatPhase::PlayerTurn);
        assert_eq!(multiple.state.hand.len(), 1);
        assert_eq!(multiple.state.discard_pile.len(), 2); // unchosen card + Hologram+
    });

    defect_test!(registry_uncommon_and_rare_core, {
        let cases = [
            StatCase { id: "Aggregate", cost: 1, damage: -1, block: -1, magic: 4, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Aggregate+", cost: 1, damage: -1, block: -1, magic: 3, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Auto Shields", cost: 1, damage: -1, block: 11, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Auto Shields+", cost: 1, damage: -1, block: 15, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Blizzard", cost: 1, damage: 0, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Blizzard+", cost: 1, damage: 0, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "BootSequence", cost: 0, damage: -1, block: 10, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "BootSequence+", cost: 0, damage: -1, block: 13, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Capacitor", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Capacitor+", cost: 1, damage: -1, block: -1, magic: 3, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Chaos", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Chaos+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Chill", cost: 0, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Chill+", cost: 0, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Consume", cost: 2, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Consume+", cost: 2, damage: -1, block: -1, magic: 3, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Darkness", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Darkness+", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Defragment", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Defragment+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Doom and Gloom", cost: 2, damage: 10, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Doom and Gloom+", cost: 2, damage: 14, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Double Energy", cost: 1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Double Energy+", cost: 0, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Undo", cost: 2, damage: -1, block: 13, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Undo+", cost: 2, damage: -1, block: 16, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Force Field", cost: 4, damage: -1, block: 12, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Force Field+", cost: 4, damage: -1, block: 16, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "FTL", cost: 0, damage: 5, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "FTL+", cost: 0, damage: 6, block: -1, magic: 4, card_type: CardType::Attack, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    defect_test!(aggregate_variants_gain_truncated_energy_from_current_draw_pile, {
        // Aggregate.java costs 1 and passes magic 4 to AggregateEnergyAction;
        // the upgrade reduces that divisor to 3. The action samples the
        // current draw pile and uses integer division with no minimum gain.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Aggregate.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/AggregateEnergyAction.java
        for (card_id, divisor, draw_counts) in [
            ("Aggregate", 4, [3_usize, 4, 7, 8]),
            ("Aggregate+", 3, [2_usize, 3, 5, 6]),
        ] {
            for draw_count in draw_counts {
                let mut engine = bare_engine(&[], vec![enemy("JawWorm", 40, 0)]);
                engine.state.hand = make_deck(&[card_id]);
                engine.state.draw_pile = make_deck_n("Strike", draw_count);
                engine.state.discard_pile.clear();
                engine.state.energy = 1;

                assert!(play_self(&mut engine, card_id));
                assert_eq!(
                    engine.state.energy,
                    draw_count as i32 / divisor,
                    "{card_id} with {draw_count} cards in draw pile",
                );
            }
        }
    });

    defect_test!(registry_rare_power_and_orb_finishers, {
        let cases = [
            StatCase { id: "Fusion", cost: 2, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Fusion+", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Genetic Algorithm", cost: 1, damage: -1, block: 1, magic: 2, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Genetic Algorithm+", cost: 1, damage: -1, block: 1, magic: 3, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Glacier", cost: 2, damage: -1, block: 7, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Glacier+", cost: 2, damage: -1, block: 10, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Heatsinks", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Heatsinks+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Hello World", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Hello World+", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Lockon", cost: 1, damage: 8, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Lockon+", cost: 1, damage: 11, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Loop", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Loop+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Melter", cost: 1, damage: 10, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Melter+", cost: 1, damage: 14, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Steam Power", cost: 0, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Steam Power+", cost: 0, damage: -1, block: -1, magic: 3, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Recycle", cost: 1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Recycle+", cost: 0, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Redo", cost: 1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Redo+", cost: 0, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    defect_test!(steam_power_draws_before_adding_one_base_burn_to_discard, {
        // Overclock.java (ID "Steam Power") queues DrawCardAction for
        // magicNumber 2 (3 upgraded), then MakeTempCardInDiscardAction for one
        // unupgraded Burn. UseCardAction moves Overclock itself afterward.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Overclock.java
        let mut base = bare_engine(&[], vec![enemy_no_intent("JawWorm", 40, 40)]);
        base.state.hand = make_deck(&["Steam Power"]);
        base.state.draw_pile = make_deck(&["Strike", "Defend", "Zap"]);
        let energy = base.state.energy;
        assert!(play_self(&mut base, "Steam Power"));
        assert_eq!(base.state.energy, energy);
        assert_eq!(base.state.hand.len(), 2);
        assert_eq!(base.state.draw_pile.len(), 1);
        assert_eq!(base.state.discard_pile.len(), 2);
        assert_eq!(base.card_registry.card_name(base.state.discard_pile[0].def_id), "Burn");
        assert_eq!(base.card_registry.card_name(base.state.discard_pile[1].def_id), "Steam Power");

        let mut upgraded = bare_engine(&[], vec![enemy_no_intent("JawWorm", 40, 40)]);
        upgraded.state.hand = make_deck(&["Steam Power+"]);
        upgraded.state.draw_pile = make_deck(&["Strike", "Defend", "Zap"]);
        assert!(play_self(&mut upgraded, "Steam Power+"));
        assert_eq!(upgraded.state.hand.len(), 3);
        assert_eq!(discard_prefix_count(&upgraded, "Burn"), 1);
    });

    defect_test!(registry_rare_powers_and_finishers, {
        let cases = [
            StatCase { id: "Reinforced Body", cost: -1, damage: -1, block: 7, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Reinforced Body+", cost: -1, damage: -1, block: 9, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Reprogram", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Reprogram+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Rip and Tear", cost: 1, damage: 7, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Rip and Tear+", cost: 1, damage: 9, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Scrape", cost: 1, damage: 7, block: -1, magic: 4, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Scrape+", cost: 1, damage: 10, block: -1, magic: 5, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Self Repair", cost: 1, damage: -1, block: -1, magic: 7, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Self Repair+", cost: 1, damage: -1, block: -1, magic: 10, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Skim", cost: 1, damage: -1, block: -1, magic: 3, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Skim+", cost: 1, damage: -1, block: -1, magic: 4, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Static Discharge", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Static Discharge+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Storm", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Storm+", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Sunder", cost: 3, damage: 24, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Sunder+", cost: 3, damage: 32, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Tempest", cost: -1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Tempest+", cost: -1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "White Noise", cost: 1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "White Noise+", cost: 0, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "All For One", cost: 2, damage: 10, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "All For One+", cost: 2, damage: 14, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    defect_test!(skim_plus_spends_one_energy_draws_four_and_discards_normally, {
        // Skim.java queues one DrawCardAction for magicNumber=3; its only
        // upgrade raises that number to 4. The Skill neither exhausts nor
        // changes target behavior.
        let mut engine = bare_engine(&[], vec![enemy("JawWorm", 40, 0)]);
        engine.state.energy = 1;
        engine.state.hand = make_deck(&["Skim+"]);
        engine.state.draw_pile = make_deck(&[
            "Strike", "Defend", "Bash", "Inflame", "Zap",
        ]);
        engine.state.discard_pile.clear();

        assert!(play_self(&mut engine, "Skim+"));

        assert_eq!(engine.state.energy, 0);
        assert_eq!(engine.state.hand.len(), 4);
        assert_eq!(engine.state.draw_pile.len(), 1);
        assert_eq!(discard_prefix_count(&engine, "Skim"), 1);
        assert_eq!(exhaust_prefix_count(&engine, "Skim"), 0);
    });

    defect_test!(lock_on_source_applies_after_its_hit_and_amplifies_orb_thorns, {
        // LockOn.java deals damage before applying its DEBUFF power. Lock-On+
        // deals 11 and applies 3. AbstractOrb.applyLockOn multiplies Lightning
        // and Dark damage by 1.5; their DamageInfo type is THORNS, so Flight and
        // Malleable do not react. LockOnPower loses one stack at end of round.
        // Java: cards/blue/LockOn.java, powers/LockOnPower.java, and
        // orbs/AbstractOrb.java.
        let mut engine = bare_engine(&[], vec![enemy("JawWorm", 80, 0)]);
        engine.state.hand = make_deck(&["Lockon+"]);
        assert!(play_on_enemy(&mut engine, "Lockon+", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 69);
        assert_eq!(engine.state.enemies[0].entity.status(sid::LOCK_ON), 3);

        engine.state.enemies[0].entity.set_status(sid::FLIGHT, 2);
        engine.state.enemies[0].entity.set_status(sid::MALLEABLE, 1);
        engine.init_defect_orbs(1);
        engine.channel_orb(OrbType::Lightning);
        let mut oracle = engine.card_random_rng.clone();
        oracle.random(0);
        engine.evoke_front_orb();
        assert_eq!(engine.state.enemies[0].entity.hp, 57); // floor(8 * 1.5)
        assert_eq!(engine.card_random_rng.counter, oracle.counter);
        assert_eq!(engine.state.enemies[0].entity.status(sid::FLIGHT), 2);
        assert_eq!(engine.state.enemies[0].entity.status(sid::MALLEABLE), 1);

        let card_random_before_dark = engine.card_random_rng.counter;
        engine.channel_orb(OrbType::Dark);
        engine.evoke_front_orb();
        assert_eq!(engine.state.enemies[0].entity.hp, 48); // floor(6 * 1.5)
        assert_eq!(engine.card_random_rng.counter, card_random_before_dark);
        process_end_of_round(&mut engine.state.enemies[0].entity);
        assert_eq!(engine.state.enemies[0].entity.status(sid::LOCK_ON), 2);

        let mut artifact = bare_engine(&[], vec![enemy("JawWorm", 80, 0)]);
        artifact.state.hand = make_deck(&["Lockon"]);
        artifact.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);
        assert!(play_on_enemy(&mut artifact, "Lockon", 0));
        assert_eq!(artifact.state.enemies[0].entity.hp, 72);
        assert_eq!(artifact.state.enemies[0].entity.status(sid::ARTIFACT), 0);
        assert_eq!(artifact.state.enemies[0].entity.status(sid::LOCK_ON), 0);
        artifact.init_defect_orbs(1);
        artifact.channel_orb(OrbType::Lightning);
        artifact.evoke_front_orb();
        assert_eq!(artifact.state.enemies[0].entity.hp, 64);
    });

    defect_test!(registry_final_rare_cards, {
        let cases = [
            StatCase { id: "Amplify", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Amplify+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Biased Cognition", cost: 1, damage: -1, block: -1, magic: 4, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Biased Cognition+", cost: 1, damage: -1, block: -1, magic: 5, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Buffer", cost: 2, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Buffer+", cost: 2, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Core Surge", cost: 1, damage: 11, block: -1, magic: 1, card_type: CardType::Attack, exhaust: true },
            StatCase { id: "Core Surge+", cost: 1, damage: 15, block: -1, magic: 1, card_type: CardType::Attack, exhaust: true },
            StatCase { id: "Creative AI", cost: 3, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Creative AI+", cost: 2, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Echo Form", cost: 3, damage: -1, block: -1, magic: -1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Echo Form+", cost: 3, damage: -1, block: -1, magic: -1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Electrodynamics", cost: 2, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Electrodynamics+", cost: 2, damage: -1, block: -1, magic: 3, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Fission", cost: 0, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Fission+", cost: 0, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Hyperbeam", cost: 2, damage: 26, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Hyperbeam+", cost: 2, damage: 34, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Impulse", cost: 1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Impulse+", cost: 1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Machine Learning", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Machine Learning+", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Meteor Strike", cost: 5, damage: 24, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Meteor Strike+", cost: 5, damage: 30, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Multi-Cast", cost: -1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Multi-Cast+", cost: -1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Rainbow", cost: 2, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Rainbow+", cost: 2, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Reboot", cost: 0, damage: -1, block: -1, magic: 4, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Reboot+", cost: 0, damage: -1, block: -1, magic: 6, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Seek", cost: 0, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Seek+", cost: 0, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Thunder Strike", cost: 3, damage: 7, block: -1, magic: 0, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Thunder Strike+", cost: 3, damage: 9, block: -1, magic: 0, card_type: CardType::Attack, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    // ------------------------------------------------------------------
    // Runtime parity cases for implemented mechanics
    // ------------------------------------------------------------------

    defect_test!(zap_channels_lightning, {
        let mut e = filled_engine(&["Zap"], 40, 0);
        e.init_defect_orbs(1);
        ensure_in_hand(&mut e, "Zap");
        play_self(&mut e, "Zap");
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
        assert_eq!(e.state.energy, 2);
    });

    defect_test!(zap_plus_is_zero_cost_and_channels, {
        let mut e = filled_engine(&["Zap+"], 40, 0);
        e.init_defect_orbs(1);
        ensure_in_hand(&mut e, "Zap+");
        let before = e.state.energy;
        play_self(&mut e, "Zap+");
        assert_eq!(e.state.energy, before);
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
    });

    defect_test!(dualcast_evokes_the_same_front_orb_twice, {
        // Dualcast.java queues EvokeWithoutRemovingOrbAction(1), then
        // EvokeOrbAction(1). A front Lightning therefore deals 8 twice and is
        // removed, while the Frost behind it is not evoked.
        // Java: reference/extracted/methods/card/Dualcast.java
        for (card_id, starting_energy) in [("Dualcast", 1), ("Dualcast+", 0)] {
            let mut e = bare_engine(&[card_id], vec![enemy("JawWorm", 40, 0)]);
            e.init_defect_orbs(2);
            e.channel_orb(OrbType::Lightning);
            e.channel_orb(OrbType::Frost);
            e.state.energy = starting_energy;
            ensure_in_hand(&mut e, card_id);

            assert!(play_self(&mut e, card_id));
            assert_eq!(e.state.enemies[0].entity.hp, 24);
            assert_eq!(e.state.orb_slots.occupied_count(), 1);
            assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Frost);
            assert_eq!(e.state.player.block, 0);
            assert_eq!(e.state.energy, 0);
        }
    });

    defect_test!(ball_lightning_channels_and_hits, {
        let mut e = filled_engine(&["Ball Lightning"], 40, 0);
        e.init_defect_orbs(1);
        ensure_in_hand(&mut e, "Ball Lightning");
        let hp = e.state.enemies[0].entity.hp;
        play_on_enemy(&mut e, "Ball Lightning", 0);
        assert_eq!(e.state.enemies[0].entity.hp, hp - 7);
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
    });

    defect_test!(barrage_scales_with_orbs, {
        let mut e = filled_engine(&["Barrage"], 60, 0);
        set_orbs(&mut e, &[OrbType::Lightning, OrbType::Frost, OrbType::Dark]);
        ensure_in_hand(&mut e, "Barrage");
        let hp = e.state.enemies[0].entity.hp;
        play_on_enemy(&mut e, "Barrage", 0);
        assert_eq!(e.state.enemies[0].entity.hp, hp - 12);
    });

    defect_test!(beam_cell_applies_vulnerable, {
        let mut e = filled_engine(&["Beam Cell"], 40, 0);
        ensure_in_hand(&mut e, "Beam Cell");
        play_on_enemy(&mut e, "Beam Cell", 0);
        assert_eq!(e.state.enemies[0].entity.status(sid::VULNERABLE), 1);
    });

    defect_test!(cold_snap_channels_frost, {
        // Source: ColdSnap.java sets damage 6 and magic 1, queues damage before
        // ChannelAction(Frost), and upgrades only damage by 3.
        let mut e = filled_engine(&["Cold Snap"], 40, 0);
        e.init_defect_orbs(1);
        ensure_in_hand(&mut e, "Cold Snap");
        let hp = e.state.enemies[0].entity.hp;
        play_on_enemy(&mut e, "Cold Snap", 0);
        assert_eq!(e.state.enemies[0].entity.hp, hp - 6);
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Frost);

        let mut upgraded = filled_engine(&["Cold Snap+"], 40, 0);
        upgraded.init_defect_orbs(2);
        ensure_in_hand(&mut upgraded, "Cold Snap+");
        let hp = upgraded.state.enemies[0].entity.hp;
        play_on_enemy(&mut upgraded, "Cold Snap+", 0);
        assert_eq!(upgraded.state.enemies[0].entity.hp, hp - 9);
        assert_eq!(
            upgraded
                .state
                .orb_slots
                .slots
                .iter()
                .filter(|slot| slot.orb_type == OrbType::Frost)
                .count(),
            1
        );
    });

    defect_test!(compile_driver_draws_per_unique_orb, {
        let mut e = bare_engine(&["Compile Driver"], vec![enemy("JawWorm", 50, 0)]);
        e.state.draw_pile = make_deck(&["Strike", "Defend", "Zap", "Cold Snap"]);
        set_orbs(&mut e, &[OrbType::Lightning, OrbType::Frost, OrbType::Dark]);
        ensure_in_hand(&mut e, "Compile Driver");
        let hand_before = e.state.hand.len();
        play_on_enemy(&mut e, "Compile Driver", 0);
        assert_eq!(e.state.hand.len(), hand_before + 2);
    });

    defect_test!(conserve_battery_plus_blocks_and_grants_one_energy_next_turn, {
        // Source: ConserveBattery.java queues GainBlockAction(10 upgraded)
        // before EnergizedBluePower(1). EnergizedBluePower.onEnergyRecharge
        // grants its amount and removes itself.
        let mut e = bare_engine(&["Conserve Battery+"], vec![enemy("JawWorm", 50, 0)]);
        ensure_in_hand(&mut e, "Conserve Battery+");

        assert!(play_self(&mut e, "Conserve Battery+"));
        assert_eq!(e.state.player.block, 10);
        assert_eq!(e.state.player.status(sid::ENERGIZED), 1);
        assert_eq!(e.state.energy, 2);

        end_turn(&mut e);

        assert_eq!(e.state.energy, 4);
        assert_eq!(e.state.player.status(sid::ENERGIZED), 0);

        let mut capped = bare_engine(&["Conserve Battery"], vec![enemy("JawWorm", 50, 0)]);
        capped.state.player.set_status(sid::ENERGIZED, 999);
        ensure_in_hand(&mut capped, "Conserve Battery");
        assert!(play_self(&mut capped, "Conserve Battery"));
        assert_eq!(capped.state.player.status(sid::ENERGIZED), 999);
    });

    defect_test!(consume_reduces_orb_slots_and_gains_focus, {
        let mut e = bare_engine(&["Consume"], vec![enemy("JawWorm", 50, 0)]);
        e.init_defect_orbs(3);
        ensure_in_hand(&mut e, "Consume");
        play_self(&mut e, "Consume");
        assert_eq!(e.state.orb_slots.get_slot_count(), 2);
        assert_eq!(e.state.player.focus(), 2);
    });

    defect_test!(consume_plus_removes_the_last_orb_without_evoking_it, {
        // Source: Consume.java queues FocusPower(3 upgraded) before
        // DecreaseMaxOrbAction(1). AbstractPlayer.decreaseMaxOrbSlots removes
        // the final orb directly and does not invoke onEvoke.
        let mut e = bare_engine(&["Consume+"], vec![enemy("JawWorm", 50, 0)]);
        e.init_defect_orbs(2);
        e.channel_orb(OrbType::Lightning);
        e.channel_orb(OrbType::Frost);
        ensure_in_hand(&mut e, "Consume+");

        assert!(play_self(&mut e, "Consume+"));

        assert_eq!(e.state.player.focus(), 3);
        assert_eq!(e.state.orb_slots.get_slot_count(), 1);
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
        assert_eq!(e.state.player.block, 0);
    });

    defect_test!(darkness_plus_accumulates_on_entry, {
        let mut e = bare_engine(&["Darkness+"], vec![enemy("JawWorm", 50, 0)]);
        e.init_defect_orbs(1);
        ensure_in_hand(&mut e, "Darkness+");
        play_self(&mut e, "Darkness+");
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Dark);
        assert_eq!(e.state.orb_slots.slots[0].evoke_amount, 12);
    });

    defect_test!(doom_and_gloom_damages_every_enemy_then_channels_one_dark, {
        // DoomAndGloom.java queues DamageAllEnemiesAction for 10 damage, then
        // ChannelAction(new Dark()). upgradeDamage(4) is its only upgrade.
        // Java: reference/extracted/methods/card/DoomAndGloom.java
        for (card_id, expected_hp) in [("Doom and Gloom", 30), ("Doom and Gloom+", 26)] {
            let mut e = bare_engine(
                &[card_id],
                vec![enemy("JawWorm", 40, 0), enemy("Cultist", 40, 0)],
            );
            e.init_defect_orbs(3);
            e.state.energy = 2;
            ensure_in_hand(&mut e, card_id);

            assert!(play_self(&mut e, card_id));

            assert_eq!(e.state.enemies[0].entity.hp, expected_hp);
            assert_eq!(e.state.enemies[1].entity.hp, expected_hp);
            assert_eq!(e.state.orb_slots.occupied_count(), 1);
            assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Dark);
            assert_eq!(e.state.energy, 0);
        }
    });

    defect_test!(defragment_gains_focus, {
        // Defragment.java applies FocusPower(magicNumber) at 1 base, while
        // upgradeMagicNumber(1) makes 2 and leaves the one-energy cost intact.
        let mut e = bare_engine(&[], vec![enemy("JawWorm", 40, 0)]);
        e.state.hand = make_deck(&["Defragment", "Defragment+"]);
        e.state.energy = 2;

        assert!(play_self(&mut e, "Defragment"));
        assert_eq!(e.state.player.focus(), 1);
        assert_eq!(e.state.energy, 1);

        assert!(play_self(&mut e, "Defragment+"));
        assert_eq!(e.state.player.focus(), 3);
        assert_eq!(e.state.energy, 0);
    });

    defect_test!(double_energy_doubles_energy_remaining_after_its_cost, {
        // DoubleEnergy.java costs 1 (0 upgraded) and exhausts. Its action gains
        // EnergyPanel.totalCount, which is the energy left after card payment.
        // Java: reference/extracted/methods/card/DoubleEnergy.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/DoubleEnergyAction.java
        for (card_id, starting_energy, expected_energy) in [
            ("Double Energy", 4, 6),
            ("Double Energy+", 4, 8),
        ] {
            let mut e = filled_engine(&[card_id], 40, 0);
            ensure_in_hand(&mut e, card_id);
            e.state.energy = starting_energy;

            assert!(play_self(&mut e, card_id));
            assert_eq!(e.state.energy, expected_energy);
            assert!(e.state.exhaust_pile.iter().any(|c| {
                e.card_registry.card_name(c.def_id) == card_id
            }));
        }
    });

    defect_test!(fusion_channels_exactly_one_plasma_and_upgrade_only_lowers_cost, {
        // Fusion.java sets magicNumber to 1 and enqueues one ChannelAction per
        // magicNumber. Its upgrade changes only the base cost from 2 to 1.
        // Java: reference/extracted/methods/card/Fusion.java
        for (card_id, expected_energy) in [("Fusion", 1), ("Fusion+", 2)] {
            let mut e = bare_engine(&[], vec![enemy("JawWorm", 40, 0)]);
            e.init_defect_orbs(3);
            e.state.hand = make_deck(&[card_id]);
            e.state.energy = 3;

            assert!(play_self(&mut e, card_id));
            assert_eq!(e.state.energy, expected_energy);
            assert_eq!(e.state.orb_slots.occupied_count(), 1);
            assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Plasma);
        }
    });

    defect_test!(fission_removes_orbs_and_refunds_energy, {
        let mut e = bare_engine(&["Fission"], vec![enemy("JawWorm", 40, 0)]);
        set_orbs(&mut e, &[OrbType::Lightning, OrbType::Frost]);
        ensure_in_hand(&mut e, "Fission");
        let before_energy = e.state.energy;
        play_self(&mut e, "Fission");
        assert_eq!(e.state.energy, before_energy + 2);
        assert_eq!(e.state.orb_slots.occupied_count(), 0);
    });

    defect_test!(fission_plus_evokes_all_orbs, {
        let mut e = bare_engine(&["Fission+"], vec![enemy("JawWorm", 80, 0)]);
        set_orbs(&mut e, &[OrbType::Lightning, OrbType::Frost, OrbType::Dark]);
        ensure_in_hand(&mut e, "Fission+");
        let before_hp = e.state.enemies[0].entity.hp;
        let before_block = e.state.player.block;
        play_self(&mut e, "Fission+");
        assert_eq!(e.state.enemies[0].entity.hp, before_hp - 14); // Lightning evoke 8 + Dark evoke 6
        assert_eq!(e.state.player.block, before_block + 5); // Frost evoke 5
        assert_eq!(e.state.energy, 6);
    });

    defect_test!(glacier_gains_exact_block_and_channels_exactly_two_frost, {
        // Glacier.java queues GainBlockAction(block), then loops magicNumber=2
        // ChannelAction(Frost). upgrade() only adds 3 block.
        // Java: reference/extracted/methods/card/Glacier.java
        for (card_id, expected_block) in [("Glacier", 7), ("Glacier+", 10)] {
            let mut e = bare_engine(&[], vec![enemy("JawWorm", 40, 0)]);
            e.init_defect_orbs(3);
            e.state.hand = make_deck(&[card_id]);
            e.state.energy = 3;

            assert!(play_self(&mut e, card_id));
            assert_eq!(e.state.energy, 1);
            assert_eq!(e.state.player.block, expected_block);
            assert_eq!(e.state.orb_slots.occupied_count(), 2);
            assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Frost);
            assert_eq!(e.state.orb_slots.slots[1].orb_type, OrbType::Frost);
            assert_eq!(e.state.orb_slots.slots[2].orb_type, OrbType::Empty);
        }
    });

    defect_test!(go_for_the_eyes_weakens_exactly_the_four_attacking_intent_variants, {
        // ForTheEyesAction checks getIntentBaseDmg() >= 0. Java assigns a
        // non-negative base damage to ATTACK, ATTACK_DEFEND, ATTACK_BUFF, and
        // ATTACK_DEBUFF, while non-attacking intents keep -1.
        // Java: reference/extracted/methods/card/GoForTheEyes.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ForTheEyesAction.java
        let attacking = [
            Intent::Attack { damage: 6, hits: 1, effects: 0 },
            Intent::AttackBlock { damage: 6, hits: 1, block: 5, effects: 0 },
            Intent::AttackBuff { damage: 6, hits: 1, effects: 0 },
            Intent::AttackDebuff { damage: 6, hits: 1, effects: 0 },
        ];
        for intent in attacking {
            let mut e = bare_engine(&[], vec![enemy("JawWorm", 40, 0)]);
            e.state.enemies[0].intent = intent;
            e.state.hand = make_deck(&["Go for the Eyes+"]);

            assert!(play_on_enemy(&mut e, "Go for the Eyes+", 0));
            assert_eq!(e.state.enemies[0].entity.hp, 36);
            assert_eq!(e.state.enemies[0].entity.status(sid::WEAKENED), 2);
        }

        let mut non_attacking = bare_engine(&[], vec![enemy("JawWorm", 40, 0)]);
        non_attacking.state.enemies[0].intent = Intent::Buff { effects: 0 };
        non_attacking.state.hand = make_deck(&["Go for the Eyes"]);
        assert!(play_on_enemy(&mut non_attacking, "Go for the Eyes", 0));
        assert_eq!(non_attacking.state.enemies[0].entity.hp, 37);
        assert_eq!(non_attacking.state.enemies[0].entity.status(sid::WEAKENED), 0);

        let mut artifact = bare_engine(&[], vec![enemy("JawWorm", 40, 0)]);
        artifact.state.enemies[0].intent = Intent::Attack { damage: 6, hits: 1, effects: 0 };
        artifact.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);
        artifact.state.hand = make_deck(&["Go for the Eyes"]);
        assert!(play_on_enemy(&mut artifact, "Go for the Eyes", 0));
        assert_eq!(artifact.state.enemies[0].entity.status(sid::ARTIFACT), 0);
        assert_eq!(artifact.state.enemies[0].entity.status(sid::WEAKENED), 0);
    });

    defect_test!(hyperbeam_aoe_and_negative_focus_match_java_action_order, {
        // Hyperbeam.java deals 26 AoE (34 upgraded), then applies
        // FocusPower(-3). FocusPower.java marks a negative instance DEBUFF, so
        // Artifact blocks it. DamageAllEnemiesAction clears the queued Focus
        // loss when the AoE ends combat.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Hyperbeam.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FocusPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAllEnemiesAction.java
        let mut base = bare_engine(
            &[],
            vec![
                enemy_no_intent("JawWorm", 50, 50),
                enemy_no_intent("Cultist", 50, 50),
            ],
        );
        base.state.hand = make_deck(&["Hyperbeam"]);
        base.state.player.set_status(sid::FOCUS, 4);
        assert!(play_self(&mut base, "Hyperbeam"));
        assert_eq!(base.state.enemies[0].entity.hp, 24);
        assert_eq!(base.state.enemies[1].entity.hp, 24);
        assert_eq!(base.state.player.focus(), 1);

        let mut artifact = bare_engine(
            &[],
            vec![
                enemy_no_intent("JawWorm", 50, 50),
                enemy_no_intent("Cultist", 50, 50),
            ],
        );
        artifact.state.hand = make_deck(&["Hyperbeam+"]);
        artifact.state.player.set_status(sid::FOCUS, 4);
        artifact.state.player.set_status(sid::ARTIFACT, 1);
        assert!(play_self(&mut artifact, "Hyperbeam+"));
        assert_eq!(artifact.state.enemies[0].entity.hp, 16);
        assert_eq!(artifact.state.enemies[1].entity.hp, 16);
        assert_eq!(artifact.state.player.focus(), 4);
        assert_eq!(artifact.state.player.status(sid::ARTIFACT), 0);

        let mut lethal = bare_engine(
            &[],
            vec![enemy_no_intent("JawWorm", 26, 26)],
        );
        lethal.state.hand = make_deck(&["Hyperbeam"]);
        lethal.state.player.set_status(sid::FOCUS, 4);
        assert!(play_self(&mut lethal, "Hyperbeam"));
        assert!(lethal.state.is_victory());
        assert_eq!(lethal.state.player.focus(), 4);
    });

    defect_test!(impulse_triggers_every_orb_and_cables_with_card_random_ticks, {
        // ImpulseAction invokes start then end callbacks for each orb, and
        // Cables repeats both for the front orb. Lightning chooses through
        // cardRandomRng even with one target. Upgrade removes only Exhaust.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Impulse.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ImpulseAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
        let mut engine = bare_engine(
            &[],
            vec![enemy_no_intent("JawWorm", 100, 100)],
        );
        engine.state.hand = make_deck(&["Impulse"]);
        engine.state.relics.push("Cables".to_string());
        engine.init_defect_orbs(4);
        engine.channel_orb(OrbType::Lightning);
        engine.channel_orb(OrbType::Frost);
        engine.channel_orb(OrbType::Dark);
        engine.channel_orb(OrbType::Plasma);
        let card_random_before = engine.card_random_rng.counter;

        assert!(play_self(&mut engine, "Impulse"));

        assert_eq!(engine.state.energy, 3); // pay 1, then Plasma start grants 1
        assert_eq!(engine.state.enemies[0].entity.hp, 94); // Lightning + Cables
        assert_eq!(engine.state.player.block, 2);
        assert_eq!(engine.state.orb_slots.slots[2].evoke_amount, 12);
        assert_eq!(engine.card_random_rng.counter, card_random_before + 2);
        assert_eq!(exhaust_prefix_count(&engine, "Impulse"), 1);

        let mut upgraded = bare_engine(
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
        );
        upgraded.state.hand = make_deck(&["Impulse+"]);
        assert!(play_self(&mut upgraded, "Impulse+"));
        assert_eq!(exhaust_prefix_count(&upgraded, "Impulse"), 0);
        assert_eq!(discard_prefix_count(&upgraded, "Impulse"), 1);
    });

    defect_test!(meteo_strike_channels_plasma, {
        let mut e = filled_engine(&["Meteor Strike"], 60, 0);
        e.init_defect_orbs(3);
        e.state.energy = 5;
        ensure_in_hand(&mut e, "Meteor Strike");
        play_on_enemy(&mut e, "Meteor Strike", 0);
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Plasma);
    });

    defect_test!(meteor_strike_source_damage_precedes_three_plasma_channels, {
        // MeteorStrike.java queues its DamageAction before three ChannelActions.
        // A nonlethal Meteor Strike+ channels all three, while lethal damage
        // clears those queued channels. The constructor also adds STRIKE.
        let registry = crate::cards::global_registry();
        assert!(registry.is_strike(registry.card_id("Meteor Strike")));
        assert!(registry.is_strike(registry.card_id("Meteor Strike+")));

        let mut nonlethal = bare_engine(&[], vec![enemy_no_intent("JawWorm", 80, 80)]);
        nonlethal.init_defect_orbs(3);
        nonlethal.state.energy = 5;
        nonlethal.state.hand = make_deck(&["Meteor Strike+"]);
        assert!(play_on_enemy(&mut nonlethal, "Meteor Strike+", 0));
        assert_eq!(nonlethal.state.enemies[0].entity.hp, 50);
        assert_eq!(nonlethal.state.orb_slots.occupied_count(), 3);
        assert!(nonlethal.state.orb_slots.slots.iter().all(|orb| {
            orb.orb_type == OrbType::Plasma
        }));
        assert_eq!(nonlethal.state.energy, 0);

        let mut lethal = bare_engine(&[], vec![enemy_no_intent("JawWorm", 20, 20)]);
        lethal.init_defect_orbs(3);
        lethal.state.energy = 5;
        lethal.state.hand = make_deck(&["Meteor Strike"]);
        assert!(play_on_enemy(&mut lethal, "Meteor Strike", 0));
        assert!(lethal.state.enemies[0].entity.is_dead());
        assert_eq!(lethal.state.orb_slots.occupied_count(), 0);
    });

    defect_test!(multi_cast_evokes_front_orb_x_times, {
        let mut e = bare_engine(&["Multi-Cast"], vec![enemy("JawWorm", 80, 0)]);
        set_orbs(&mut e, &[OrbType::Lightning, OrbType::Frost, OrbType::Dark]);
        ensure_in_hand(&mut e, "Multi-Cast");
        let hp = e.state.enemies[0].entity.hp;
        play_self(&mut e, "Multi-Cast");
        // MulticastAction repeats the same front Lightning X=3 times and
        // removes only that orb on the final evoke.
        assert_eq!(e.state.enemies[0].entity.hp, hp - 24);
        assert_eq!(e.state.player.block, 0);
        assert_eq!(e.state.orb_slots.occupied_count(), 2);
        assert_eq!(e.state.orb_slots.front_orb_type(), OrbType::Frost);
    });

    defect_test!(reinforced_body_spends_x_for_block, {
        let mut e = bare_engine(&["Reinforced Body"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Reinforced Body");
        e.state.energy = 3;
        play_self(&mut e, "Reinforced Body");
        assert_eq!(e.state.player.block, 21);
        assert_eq!(e.state.energy, 0);
    });

    defect_test!(reprogram_loses_focus_and_gains_stats, {
        let mut e = bare_engine(&["Reprogram"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Reprogram");
        e.state.player.set_status(sid::FOCUS, 3);
        play_self(&mut e, "Reprogram");
        assert_eq!(e.state.player.focus(), 2);
        assert_eq!(e.state.player.strength(), 1);
        assert_eq!(e.state.player.dexterity(), 1);
    });

    defect_test!(reprogram_plus_applies_artifact_blocked_focus_before_buffs, {
        // Reprogram.java queues FocusPower(-magic), StrengthPower(magic), then
        // DexterityPower(magic). FocusPower classifies a negative amount as a
        // DEBUFF, so Artifact consumes only the Focus loss; the upgraded +2
        // Strength and Dexterity still resolve afterward.
        // Sources: cards/blue/Reprogram.java, powers/FocusPower.java, and
        // actions/common/ApplyPowerAction.java.
        let mut e = bare_engine(&["Reprogram+"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Reprogram+");
        e.state.player.set_status(sid::FOCUS, 3);
        e.state.player.set_status(sid::ARTIFACT, 1);

        assert!(play_self(&mut e, "Reprogram+"));

        assert_eq!(e.state.player.focus(), 3);
        assert_eq!(e.state.player.status(sid::ARTIFACT), 0);
        assert_eq!(e.state.player.strength(), 2);
        assert_eq!(e.state.player.dexterity(), 2);
    });

    defect_test!(rip_and_tear_hits_twice_against_single_enemy, {
        let mut e = filled_engine(&["Rip and Tear"], 40, 0);
        ensure_in_hand(&mut e, "Rip and Tear");
        let hp = e.state.enemies[0].entity.hp;
        play_on_enemy(&mut e, "Rip and Tear", 0);
        assert_eq!(e.state.enemies[0].entity.hp, hp - 14);
    });

    defect_test!(storm_stacks_channel_once_each_and_upgrade_is_innate_only, {
        // Storm.java applies one StormPower stack in both versions and makes
        // only the upgrade Innate. StormPower.onUseCard fires before the new
        // Power's use(), so Storm does not trigger itself: the second Storm is
        // seen by one existing stack, then Defragment is seen by both stacks.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Storm.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/StormPower.java
        let base = reg().get("Storm").expect("Storm");
        let upgraded = reg().get("Storm+").expect("Storm+");
        assert!(!base.runtime_traits().innate);
        assert!(upgraded.runtime_traits().innate);
        assert_eq!((base.base_magic, upgraded.base_magic), (1, 1));

        let mut e = bare_engine(&[], vec![enemy("JawWorm", 40, 0)]);
        e.init_defect_orbs(3);
        e.state.hand = make_deck(&["Storm", "Storm+", "Defragment"]);
        assert!(play_self(&mut e, "Storm"));
        assert_eq!(e.state.orb_slots.occupied_count(), 0);
        assert!(play_self(&mut e, "Storm+"));
        assert_eq!(e.state.orb_slots.occupied_count(), 1);
        assert_eq!(e.state.player.status(sid::STORM), 2);
        assert!(play_self(&mut e, "Defragment"));
        assert_eq!(e.state.orb_slots.occupied_count(), 3);
        assert!(e.state.orb_slots.slots.iter().all(|orb| orb.orb_type == OrbType::Lightning));
    });

    defect_test!(sunder_refunds_only_when_its_kill_does_not_end_combat, {
        // SunderAction deals 24 damage (32 upgraded), queues 3 energy on a
        // kill, then clears that queued non-combat action when all monsters
        // are basically dead.
        // Java: reference/extracted/methods/card/Sunder.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/SunderAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
        let mut mid_fight = bare_engine(
            &[],
            vec![enemy("JawWorm", 24, 0), enemy("Cultist", 40, 0)],
        );
        mid_fight.state.hand = make_deck(&["Sunder"]);
        mid_fight.state.energy = 3;
        assert!(play_on_enemy(&mut mid_fight, "Sunder", 0));
        assert_eq!(mid_fight.state.enemies[0].entity.hp, 0);
        assert_eq!(mid_fight.state.enemies[1].entity.hp, 40);
        assert_eq!(mid_fight.state.energy, 3);

        let mut nonlethal = filled_engine(&["Sunder"], 25, 0);
        nonlethal.state.energy = 3;
        assert!(play_on_enemy(&mut nonlethal, "Sunder", 0));
        assert_eq!(nonlethal.state.enemies[0].entity.hp, 1);
        assert_eq!(nonlethal.state.energy, 0);

        let mut final_kill = filled_engine(&["Sunder+"], 32, 0);
        final_kill.state.energy = 3;
        assert!(play_on_enemy(&mut final_kill, "Sunder+", 0));
        assert_eq!(final_kill.state.enemies[0].entity.hp, 0);
        assert_eq!(final_kill.state.energy, 0);
    });

    defect_test!(tempest_channels_x_lightning, {
        let mut e = bare_engine(&["Tempest"], vec![enemy("JawWorm", 40, 0)]);
        e.init_defect_orbs(3);
        ensure_in_hand(&mut e, "Tempest");
        e.state.energy = 3;
        play_self(&mut e, "Tempest");
        assert_eq!(e.state.orb_slots.occupied_count(), 3);
        assert!(e.state.orb_slots.slots.iter().all(|orb| orb.orb_type == OrbType::Lightning));
    });

    defect_test!(thunder_strike_scales_with_lightning_channel_count, {
        let mut e = filled_engine(&["Thunder Strike"], 40, 0);
        ensure_in_hand(&mut e, "Thunder Strike");
        e.state.player.set_status(sid::LIGHTNING_CHANNELED, 4);
        play_on_enemy(&mut e, "Thunder Strike", 0);
        assert_eq!(e.state.enemies[0].entity.hp, 40 - 28); // 4 hits of 7 base damage = 28
    });

    defect_test!(white_noise_adds_a_power_card, {
        let mut e = bare_engine(&["White Noise"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "White Noise");
        let hand_before = e.state.hand.len();
        play_self(&mut e, "White Noise");
        assert_eq!(e.state.hand.len(), hand_before);
        assert!(e.state.hand.iter().any(|c| e.card_registry.card_name(c.def_id).starts_with("Defragment")));
    });

    defect_test!(all_for_one_returns_zero_cost_cards_from_discard, {
        // AllForOne.java deals 10 (14 upgraded), then AllCostToHandAction scans
        // discard in order for current cost 0 OR freeToPlayOnce. Its queued
        // DiscardToHandActions move the earliest matches until hand size ten.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/AllForOne.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/AllCostToHandAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/DiscardToHandAction.java
        let mut e = bare_engine(&[], vec![enemy("JawWorm", 50, 0)]);
        e.state.hand = make_deck(&[
            "All For One",
            "Defend", "Defend", "Defend", "Defend",
            "Defend", "Defend", "Defend", "Defend",
        ]);
        e.state.energy = 2;
        let mut temporary_zero = e.card_registry.make_card("Strike");
        temporary_zero.cost = 0;
        let free_defend = e.card_registry.make_card("Defend").set_free(true);
        let mut raised_zero = e.card_registry.make_card("Zap+");
        raised_zero.cost = 1;
        let turbo = e.card_registry.make_card("Turbo");
        e.state.discard_pile = vec![temporary_zero, free_defend, raised_zero, turbo];

        assert!(play_on_enemy(&mut e, "All For One", 0));
        assert_eq!(e.state.enemies[0].entity.hp, 40);
        assert_eq!(e.state.hand.len(), 10);
        let returned = e.state.hand.iter().rev().take(2)
            .map(|card| e.card_registry.card_name(card.def_id))
            .collect::<Vec<_>>();
        assert_eq!(returned, vec!["Defend", "Strike"]);
        assert!(e.state.hand.last().expect("free Defend").is_free());
        assert!(e.state.discard_pile.iter().any(|card| {
            e.card_registry.card_name(card.def_id) == "Zap+" && card.cost == 1
        }));
        assert!(e.state.discard_pile.iter().any(|card| {
            e.card_registry.card_name(card.def_id) == "Turbo"
        }));

        let mut upgraded = bare_engine(&[], vec![enemy("JawWorm", 50, 0)]);
        upgraded.state.hand = make_deck(&["All For One+"]);
        upgraded.state.energy = 2;
        assert!(play_on_enemy(&mut upgraded, "All For One+", 0));
        assert_eq!(upgraded.state.enemies[0].entity.hp, 36);
    });

    defect_test!(amplify_variants_replay_power_cards_consume_charges_and_expire, {
        // Amplify.java applies one charge, upgraded to two. AmplifyPower plays
        // each next non-purge Power card one additional time, consumes one
        // charge per original card, and removes unused charges at turn end.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Amplify.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/AmplifyPower.java
        let mut base = bare_engine(&[], vec![enemy("JawWorm", 60, 0)]);
        base.state.hand = make_deck(&["Amplify", "Defragment", "Capacitor"]);
        base.state.energy = 3;
        assert!(play_self(&mut base, "Amplify"));
        assert_eq!(base.state.player.status(sid::AMPLIFY), 1);
        assert!(play_self(&mut base, "Defragment"));
        assert_eq!(base.state.player.focus(), 2);
        assert_eq!(base.state.player.status(sid::AMPLIFY), 0);
        assert!(play_self(&mut base, "Capacitor"));
        assert_eq!(base.state.orb_slots.get_slot_count(), 2);

        let mut upgraded = bare_engine(&[], vec![enemy("JawWorm", 60, 0)]);
        upgraded.state.hand = make_deck(&["Amplify+", "Defragment", "Capacitor"]);
        upgraded.state.energy = 3;
        assert!(play_self(&mut upgraded, "Amplify+"));
        assert_eq!(upgraded.state.player.status(sid::AMPLIFY), 2);
        assert!(play_self(&mut upgraded, "Defragment"));
        assert_eq!(upgraded.state.player.focus(), 2);
        assert_eq!(upgraded.state.player.status(sid::AMPLIFY), 1);
        assert!(play_self(&mut upgraded, "Capacitor"));
        assert_eq!(upgraded.state.orb_slots.get_slot_count(), 4);
        assert_eq!(upgraded.state.player.status(sid::AMPLIFY), 0);

        let mut expires = bare_engine(&[], vec![enemy("JawWorm", 60, 0)]);
        expires.state.hand = make_deck(&["Amplify"]);
        expires.state.energy = 1;
        assert!(play_self(&mut expires, "Amplify"));
        assert_eq!(expires.state.player.status(sid::AMPLIFY), 1);
        expires.execute_action(&Action::EndTurn);
        assert_eq!(expires.state.player.status(sid::AMPLIFY), 0);

        let mut purge = bare_engine(&[], vec![enemy("JawWorm", 60, 0)]);
        purge.state.player.set_status(sid::AMPLIFY, 1);
        let mut purge_power = purge.card_registry.make_card("Defragment");
        purge_power.flags |= crate::combat_types::CardInstance::FLAG_PURGE;
        purge.state.hand = vec![purge_power];
        purge.state.energy = 1;
        assert!(play_self(&mut purge, "Defragment"));
        assert_eq!(purge.state.player.focus(), 1);
        assert_eq!(purge.state.player.status(sid::AMPLIFY), 1);
    });

    defect_test!(biased_cognition_gives_focus, {
        let mut e = bare_engine(&["Biased Cognition"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Biased Cognition");
        play_self(&mut e, "Biased Cognition");
        assert_eq!(e.state.player.focus(), 4);
    });

    defect_test!(biased_cognition_bias_and_focus_loss_obey_artifact, {
        // Sources: BiasedCognition.java applies Focus before BiasPower;
        // BiasPower.java is a DEBUFF and applies negative FocusPower each turn.
        let mut blocked_on_play = bare_engine(&[], vec![enemy("JawWorm", 40, 0)]);
        blocked_on_play.state.player.set_status(sid::ARTIFACT, 1);
        blocked_on_play.state.hand = make_deck(&["Biased Cognition+"]);
        assert!(play_self(&mut blocked_on_play, "Biased Cognition+"));
        assert_eq!(blocked_on_play.state.player.focus(), 5);
        assert_eq!(blocked_on_play.state.player.status(sid::ARTIFACT), 0);
        assert_eq!(blocked_on_play.state.player.status(sid::BIASED_COG_FOCUS_LOSS), 0);
        end_turn(&mut blocked_on_play);
        assert_eq!(blocked_on_play.state.player.focus(), 5);

        let mut blocked_tick = bare_engine(&[], vec![enemy("JawWorm", 40, 0)]);
        blocked_tick.state.hand = make_deck(&["Biased Cognition"]);
        assert!(play_self(&mut blocked_tick, "Biased Cognition"));
        assert_eq!(blocked_tick.state.player.focus(), 4);
        assert_eq!(blocked_tick.state.player.status(sid::BIASED_COG_FOCUS_LOSS), 1);
        blocked_tick.state.player.set_status(sid::ARTIFACT, 1);
        end_turn(&mut blocked_tick);
        assert_eq!(blocked_tick.state.player.focus(), 4);
        assert_eq!(blocked_tick.state.player.status(sid::ARTIFACT), 0);
        end_turn(&mut blocked_tick);
        assert_eq!(blocked_tick.state.player.focus(), 3);
    });

    defect_test!(buffer_installs_buffer, {
        let mut e = bare_engine(&["Buffer"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Buffer");
        play_self(&mut e, "Buffer");
        assert_eq!(e.state.player.status(sid::BUFFER), 1);
    });

    defect_test!(core_surge_grants_artifact, {
        let mut e = filled_engine(&["Core Surge"], 40, 0);
        ensure_in_hand(&mut e, "Core Surge");
        play_on_enemy(&mut e, "Core Surge", 0);
        assert_eq!(e.state.player.status(sid::ARTIFACT), 1);
    });

    defect_test!(core_surge_variants_damage_then_grant_one_artifact_and_exhaust, {
        // Source: CoreSurge.java queues DamageAction for 11 before applying 1
        // Artifact; upgradeDamage(4) changes only damage, and the card Exhausts.
        for (card_id, expected_hp) in [("Core Surge", 29), ("Core Surge+", 25)] {
            let mut e = filled_engine(&[card_id], 40, 0);
            ensure_in_hand(&mut e, card_id);

            assert!(play_on_enemy(&mut e, card_id, 0));

            assert_eq!(e.state.enemies[0].entity.hp, expected_hp);
            assert_eq!(e.state.player.status(sid::ARTIFACT), 1);
            assert_eq!(e.state.energy, 2);
            assert!(e.state.exhaust_pile.iter().any(|card| {
                e.card_registry.card_name(card.def_id) == card_id
            }));
        }
    });

    defect_test!(creative_ai_installs_power, {
        let mut e = bare_engine(&["Creative AI"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Creative AI");
        play_self(&mut e, "Creative AI");
        assert_eq!(e.state.player.status(sid::CREATIVE_AI), 1);
    });

    defect_test!(creative_ai_variants_stack_and_roll_one_defect_power_per_stack, {
        // CreativeAI.java applies one stack at cost 3 (2 upgraded).
        // CreativeAIPower.java rolls one non-healing source-pool Power per
        // stack with cardRandomRng at the start of the next turn.
        const POOL: &[&str] = &[
            "Defragment", "Capacitor", "Heatsinks", "Static Discharge", "Loop", "Hello World", "Storm",
            "Biased Cognition", "Machine Learning", "Electrodynamics", "Buffer", "Echo Form", "Creative AI",
        ];
        let mut e = bare_engine(&[], vec![enemy("JawWorm", 40, 0)]);
        e.state.hand = make_deck(&["Creative AI", "Creative AI+"]);
        e.state.energy = 5;
        assert!(play_self(&mut e, "Creative AI"));
        assert!(play_self(&mut e, "Creative AI+"));
        assert_eq!(e.state.player.status(sid::CREATIVE_AI), 2);
        assert_eq!(e.state.energy, 0);
        e.state.draw_pile.clear();
        e.state.discard_pile.clear();
        let mut oracle = e.card_random_rng.clone();
        let expected: Vec<&str> = (0..2)
            .map(|_| POOL[oracle.random((POOL.len() - 1) as i32) as usize])
            .collect();

        end_turn(&mut e);

        let generated: Vec<&str> = e.state.hand.iter()
            .map(|card| e.card_registry.card_name(card.def_id))
            .collect();
        assert_eq!(generated, expected);
        assert_eq!(e.card_random_rng.counter, oracle.counter);
    });

    defect_test!(echo_form_installs_one_echo_without_replaying_itself, {
        // EchoForm.java queues ApplyPowerAction for exactly one EchoPower. The
        // newly installed power was absent from this card's onUseCard window,
        // so the first Echo Form must not immediately copy itself.
        // Java: reference/extracted/methods/card/EchoForm.java
        for card_id in ["Echo Form", "Echo Form+"] {
            let mut e = bare_engine(&[card_id], vec![enemy("JawWorm", 40, 0)]);
            e.state.energy = 3;
            ensure_in_hand(&mut e, card_id);

            assert!(play_self(&mut e, card_id));
            assert_eq!(e.state.player.status(sid::ECHO_FORM), 1);
            assert_eq!(e.state.energy, 0);
        }

        // With one EchoPower already installed, that prior stack does see the
        // new Echo Form and replays it: existing 1 + original 1 + copy 1.
        let mut stacked = bare_engine(&["Echo Form+"], vec![enemy("JawWorm", 40, 0)]);
        stacked.state.player.set_status(sid::ECHO_FORM, 1);
        stacked.state.energy = 3;
        ensure_in_hand(&mut stacked, "Echo Form+");
        assert!(play_self(&mut stacked, "Echo Form+"));
        assert_eq!(stacked.state.player.status(sid::ECHO_FORM), 3);
    });

    defect_test!(electrodynamics_installs_electro_before_channeling_lightning, {
        // Electrodynamics.java queues ElectroPower before its 2/3
        // ChannelActions. With one full Lightning slot, every channel evokes
        // the prior Lightning for 8 damage to every enemy under ElectroPower.
        // Java: reference/extracted/methods/card/Electrodynamics.java
        for (card_id, expected_hp) in [("Electrodynamics", 24), ("Electrodynamics+", 16)] {
            let mut e = bare_engine(
                &[card_id],
                vec![enemy("JawWorm", 40, 0), enemy("Cultist", 40, 0)],
            );
            e.init_defect_orbs(1);
            e.channel_orb(OrbType::Lightning);
            e.state.energy = 2;
            ensure_in_hand(&mut e, card_id);

            assert!(play_self(&mut e, card_id));
            assert_eq!(e.state.player.status(sid::ELECTRODYNAMICS), 1);
            assert_eq!(e.state.enemies[0].entity.hp, expected_hp);
            assert_eq!(e.state.enemies[1].entity.hp, expected_hp);
            assert_eq!(e.state.orb_slots.occupied_count(), 1);
            assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
            assert_eq!(e.state.energy, 0);
        }
    });

    defect_test!(machine_learning_will_add_draw_status, {
        let mut e = bare_engine(&["Machine Learning"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Machine Learning");
        play_self(&mut e, "Machine Learning");
        assert_eq!(e.state.player.status(sid::DRAW), 1);
    });

    defect_test!(rainbow_channels_all_orbs, {
        let mut e = bare_engine(&["Rainbow"], vec![enemy("JawWorm", 40, 0)]);
        e.init_defect_orbs(3);
        ensure_in_hand(&mut e, "Rainbow");
        play_self(&mut e, "Rainbow");
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
        assert_eq!(e.state.orb_slots.slots[1].orb_type, OrbType::Frost);
        assert_eq!(e.state.orb_slots.slots[2].orb_type, OrbType::Dark);
    });

    defect_test!(reboot_draws_a_fresh_hand, {
        let mut e = bare_engine(&["Reboot"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Reboot");
        e.state.draw_pile = make_deck(&["Strike", "Defend", "Zap", "Cold Snap"]);
        play_self(&mut e, "Reboot");
        assert_eq!(e.state.hand.len(), 4); // Reboot draws base_magic=4 cards
    });

    defect_test!(seek_is_a_tutor_effect, {
        let mut e = bare_engine(&["Seek"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Seek");
        e.state.draw_pile = make_deck(&["Zap", "Turbo", "Defragment"]);
        play_self(&mut e, "Seek");
        // Seek now presents a PickFromDrawPile choice
        assert_eq!(e.phase, CombatPhase::AwaitingChoice);
        e.execute_action(&Action::Choose(0)); // pick first card
        assert!(!e.state.hand.is_empty());
    });

    defect_test!(process_start_of_turn_handles_power_helpers, {
        let mut entity = crate::state::EntityState::new(50, 50);
        entity.set_status(sid::ENERGIZED, 2);
        entity.set_status(sid::DRAW_CARD, 1);
        entity.set_status(sid::NEXT_TURN_BLOCK, 4);
        entity.set_status(sid::BATTLE_HYMN, 3);
        entity.set_status(sid::DEVOTION, 2);
        entity.set_status(sid::DRAW, 1);
        let result = process_start_of_turn(&mut entity);
        assert_eq!(result.extra_energy, 2);
        assert_eq!(result.draw_card_next_turn, 1);
        assert_eq!(result.block_from_next_turn, 4);
        assert_eq!(result.battle_hymn_smites, 3);
        assert_eq!(result.devotion_mantra, 2);
        assert_eq!(result.extra_draw, 1);
    });

    defect_test!(process_end_of_turn_handles_power_helpers, {
        let mut entity = crate::state::EntityState::new(50, 50);
        entity.set_status(sid::METALLICIZE, 4);
        entity.set_status(sid::PLATED_ARMOR, 3);
        entity.set_status(sid::OMEGA, 5);
        entity.set_status(sid::LIKE_WATER, 2);
        let result = process_end_of_turn(&mut entity, true);
        assert_eq!(result.metallicize_block, 4);
        assert_eq!(result.plated_armor_block, 3);
        assert_eq!(result.omega_damage, 5);
        assert_eq!(result.like_water_block, 2);
    });

    defect_test!(process_end_of_round_clears_debuffs, {
        let mut entity = crate::state::EntityState::new(50, 50);
        entity.set_status(sid::WEAKENED, 2);
        entity.set_status(sid::VULNERABLE, 1);
        entity.set_status(sid::FRAIL, 1);
        entity.set_status(sid::BLUR, 1);
        entity.set_status(sid::LOCK_ON, 2);
        process_end_of_round(&mut entity);
        assert_eq!(entity.status(sid::WEAKENED), 1);
        assert_eq!(entity.status(sid::VULNERABLE), 0);
        assert_eq!(entity.status(sid::FRAIL), 0);
        assert_eq!(entity.status(sid::BLUR), 0);
        assert_eq!(entity.status(sid::LOCK_ON), 1);
    });

    defect_test!(orb_passives_and_evokes_match_java_basics, {
        let mut slots = crate::orbs::OrbSlots::new(3);
        slots.channel(OrbType::Lightning, 0);
        slots.channel(OrbType::Frost, 0);
        slots.channel(OrbType::Dark, 0);
        let passives = slots.trigger_end_of_turn_passives(0);
        assert_eq!(passives.len(), 3);
        let evoke = slots.evoke_front(0);
        match evoke {
            crate::orbs::EvokeEffect::LightningDamage(8) => {}
            other => panic!("unexpected evoke effect: {:?}", other),
        }
    });

}
