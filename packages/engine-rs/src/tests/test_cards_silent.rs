#[cfg(test)]
mod silent_card_java_parity_tests {
    // Java sources referenced:
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/cards/green/*.java

    use crate::actions::Action;
    use crate::effects::declarative::{AmountSource, CardFilter, ChoiceAction, Effect, Pile};
    use crate::status_ids::sid;
    use crate::cards::{CardRegistry, CardTarget, CardType};
    use crate::tests::support::*;

    fn reg() -> &'static CardRegistry {
        crate::cards::global_registry()
    }

    fn assert_card(
        reg: &CardRegistry,
        id: &str,
        cost: i32,
        damage: i32,
        block: i32,
        magic: i32,
        card_type: CardType,
        target: CardTarget,
        exhaust: bool,
        enter_stance: Option<&str>,
        effects: &[&str],
    ) {
        let card = reg.get(id).unwrap_or_else(|| panic!("missing card {id}"));
        assert_eq!(card.cost, cost, "{id} cost");
        assert_eq!(card.base_damage, damage, "{id} damage");
        assert_eq!(card.base_block, block, "{id} block");
        assert_eq!(card.base_magic, magic, "{id} magic");
        assert_eq!(card.card_type, card_type, "{id} type");
        assert_eq!(card.target, target, "{id} target");
        assert_eq!(card.exhaust, exhaust, "{id} exhaust");
        assert_eq!(card.enter_stance, enter_stance, "{id} stance");
        assert_card_markers_eq(card, effects, id);
    }

    macro_rules! card_pair_test {
        (
            $name:ident,
            $base_id:expr, $base_cost:expr, $base_damage:expr, $base_block:expr, $base_magic:expr,
            $base_type:expr, $base_target:expr, $base_exhaust:expr, $base_stance:expr, $base_effects:expr,
            $up_id:expr, $up_cost:expr, $up_damage:expr, $up_block:expr, $up_magic:expr,
            $up_type:expr, $up_target:expr, $up_exhaust:expr, $up_stance:expr, $up_effects:expr $(,)?
        ) => {
            #[test]
            fn $name() {
                let reg = reg();
                assert_card(
                    &reg,
                    $base_id,
                    $base_cost,
                    $base_damage,
                    $base_block,
                    $base_magic,
                    $base_type,
                    $base_target,
                    $base_exhaust,
                    $base_stance,
                    $base_effects,
                );
                assert_card(
                    &reg,
                    $up_id,
                    $up_cost,
                    $up_damage,
                    $up_block,
                    $up_magic,
                    $up_type,
                    $up_target,
                    $up_exhaust,
                    $up_stance,
                    $up_effects,
                );
            }
        };
    }

    // ---------------------------------------------------------------------
    // Exact Java parity for every Silent card in /cards/green
    // ---------------------------------------------------------------------

    card_pair_test!(strike_g,
        "Strike", 1, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &[],
        "Strike+", 1, 9, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &[],
    );
    card_pair_test!(defend_g,
        "Defend", 1, -1, 5, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &[],
        "Defend+", 1, -1, 8, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &[],
    );
    card_pair_test!(neutralize,
        "Neutralize", 0, 3, -1, 1, CardType::Attack, CardTarget::Enemy, false, None, &["weak"],
        "Neutralize+", 0, 4, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["weak"],
    );
    card_pair_test!(survivor,
        "Survivor", 1, -1, 8, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["discard"],
        "Survivor+", 1, -1, 11, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["discard"],
    );
    card_pair_test!(acrobatics,
        "Acrobatics", 1, -1, -1, 3, CardType::Skill, CardTarget::None, false, None, &["draw", "discard"],
        "Acrobatics+", 1, -1, -1, 4, CardType::Skill, CardTarget::None, false, None, &["draw", "discard"],
    );

    #[test]
    fn acrobatics_variants_draw_before_mandatory_manual_discard() {
        // Acrobatics.java queues DrawCardAction(3) then DiscardAction(1,
        // false); the upgrade adds exactly one draw. DiscardAction's selected
        // card is a manual discard, so Tactician grants its one Energy.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Acrobatics.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Tactician.java
        for (card_id, draw_count) in [("Acrobatics", 3), ("Acrobatics+", 4)] {
            let state = combat_state_with(
                Vec::new(),
                vec![enemy_no_intent("JawWorm", 40, 40)],
                3,
            );
            let mut engine = engine_with_state(state);
            engine.state.hand = make_deck(&[card_id, "Tactician"]);
            engine.state.draw_pile = make_deck(&[
                "Strike",
                "Defend",
                "Neutralize",
                "Survivor",
            ]);
            engine.state.discard_pile.clear();
            engine.state.energy = 3;

            assert!(play_self(&mut engine, card_id));
            assert_eq!(engine.phase, crate::engine::CombatPhase::AwaitingChoice);
            assert_eq!(engine.state.hand.len(), 1 + draw_count);
            let choice = engine.choice.as_ref().expect("Acrobatics discard choice");
            assert_eq!(choice.min_picks, 1);
            assert_eq!(choice.max_picks, 1);
            assert_eq!(choice.options.len(), 1 + draw_count);
            let tactician_choice = choice.options.iter().position(|option| {
                matches!(option, crate::engine::ChoiceOption::HandCard(index)
                    if engine.card_registry.card_name(engine.state.hand[*index].def_id) == "Tactician")
            }).expect("Tactician discard option");
            engine.execute_action(&Action::Choose(tactician_choice));

            assert_eq!(engine.phase, crate::engine::CombatPhase::PlayerTurn);
            assert_eq!(engine.state.hand.len(), draw_count);
            assert_eq!(engine.state.energy, 3);
            assert!(engine.state.discard_pile.iter().any(|card| {
                engine.card_registry.card_name(card.def_id) == "Tactician"
            }));
        }
    }
    card_pair_test!(backflip,
        "Backflip", 1, -1, 5, 2, CardType::Skill, CardTarget::SelfTarget, false, None, &["draw"],
        "Backflip+", 1, -1, 8, 2, CardType::Skill, CardTarget::SelfTarget, false, None, &["draw"],
    );
    card_pair_test!(bane,
        "Bane", 1, 7, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["double_if_poisoned"],
        "Bane+", 1, 10, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["double_if_poisoned"],
    );
    card_pair_test!(blade_dance,
        "Blade Dance", 1, -1, -1, 3, CardType::Skill, CardTarget::None, false, None, &["add_shivs"],
        "Blade Dance+", 1, -1, -1, 4, CardType::Skill, CardTarget::None, false, None, &["add_shivs"],
    );
    card_pair_test!(cloak_and_dagger,
        "Cloak and Dagger", 1, -1, 6, 1, CardType::Skill, CardTarget::SelfTarget, false, None, &["add_shivs"],
        "Cloak and Dagger+", 1, -1, 6, 2, CardType::Skill, CardTarget::SelfTarget, false, None, &["add_shivs"],
    );
    card_pair_test!(dagger_spray,
        "Dagger Spray", 1, 4, -1, 2, CardType::Attack, CardTarget::AllEnemy, false, None, &["multi_hit"],
        "Dagger Spray+", 1, 6, -1, 2, CardType::Attack, CardTarget::AllEnemy, false, None, &["multi_hit"],
    );

    #[test]
    fn dagger_spray_plus_resolves_as_two_separate_six_damage_aoe_hits() {
        // DaggerSpray.java queues two distinct DamageAllEnemiesActions; its
        // upgrade adds 2 to each 4-damage hit. Malleable therefore gains 3
        // block after hit one, blocks 3 of hit two, then gains 4 block.
        let mut engine = engine_without_start(
            Vec::new(),
            vec![
                enemy_no_intent("JawWorm", 40, 40),
                enemy_no_intent("SnakePlant", 40, 40),
            ],
            1,
        );
        force_player_turn(&mut engine);
        engine.state.enemies[1].entity.set_status(sid::MALLEABLE, 3);
        engine.state.hand = make_deck(&["Dagger Spray+"]);

        assert!(play_on_enemy(&mut engine, "Dagger Spray+", 0));

        assert_eq!(engine.state.enemies[0].entity.hp, 28);
        assert_eq!(engine.state.enemies[1].entity.hp, 31);
        assert_eq!(engine.state.enemies[1].entity.block, 4);
        assert_eq!(engine.state.enemies[1].entity.status(sid::MALLEABLE), 5);
        assert_eq!(engine.state.energy, 0);
    }

    card_pair_test!(dagger_throw,
        "Dagger Throw", 1, 9, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["draw", "discard"],
        "Dagger Throw+", 1, 12, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["draw", "discard"],
    );
    card_pair_test!(deadly_poison,
        "Deadly Poison", 1, -1, -1, 5, CardType::Skill, CardTarget::Enemy, false, None, &["poison"],
        "Deadly Poison+", 1, -1, -1, 7, CardType::Skill, CardTarget::Enemy, false, None, &["poison"],
    );

    #[test]
    fn deadly_poison_applies_source_amounts_and_is_blocked_by_artifact() {
        // DeadlyPoison.java queues PoisonPower for magicNumber 5; its upgrade
        // adds 2. PoisonPower is a debuff, so Artifact consumes one stack and
        // prevents the application.
        for (card_id, expected_poison) in [("Deadly Poison", 5), ("Deadly Poison+", 7)] {
            let mut engine = engine_without_start(
                Vec::new(),
                vec![enemy_no_intent("JawWorm", 40, 40)],
                1,
            );
            force_player_turn(&mut engine);
            engine.state.hand = make_deck(&[card_id]);

            assert!(play_on_enemy(&mut engine, card_id, 0));
            assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), expected_poison);
            assert_eq!(engine.state.energy, 0);
        }

        let mut blocked = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            1,
        );
        force_player_turn(&mut blocked);
        blocked.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);
        blocked.state.hand = make_deck(&["Deadly Poison+"]);

        assert!(play_on_enemy(&mut blocked, "Deadly Poison+", 0));
        assert_eq!(blocked.state.enemies[0].entity.status(sid::ARTIFACT), 0);
        assert_eq!(blocked.state.enemies[0].entity.status(sid::POISON), 0);
    }
    card_pair_test!(deflect,
        "Deflect", 0, -1, 4, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &[],
        "Deflect+", 0, -1, 7, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &[],
    );

    #[test]
    fn deflect_variants_gain_source_block_for_zero_energy() {
        // Deflect.java queues GainBlockAction(this.block), using base Block 4
        // or 7 after upgradeBlock(3). Two Dexterity therefore makes 6 or 9.
        for (card_id, expected_block) in [("Deflect", 6), ("Deflect+", 9)] {
            let mut engine = engine_without_start(
                Vec::new(),
                vec![enemy_no_intent("JawWorm", 40, 40)],
                0,
            );
            force_player_turn(&mut engine);
            engine.state.player.set_status(sid::DEXTERITY, 2);
            engine.state.hand = make_deck(&[card_id]);

            assert!(play_self(&mut engine, card_id));
            assert_eq!(engine.state.player.block, expected_block);
            assert_eq!(engine.state.energy, 0);
        }
    }
    card_pair_test!(dodge_and_roll,
        "Dodge and Roll", 1, -1, 4, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["next_turn_block"],
        "Dodge and Roll+", 1, -1, 6, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["next_turn_block"],
    );

    #[test]
    fn dodge_and_roll_uses_modified_block_for_now_and_next_turn() {
        // DodgeAndRoll.java passes this.block to both GainBlockAction and
        // NextTurnBlockPower. With two Dexterity, base grants/stores 6 and the
        // upgraded card grants/stores 8 after upgradeBlock(2).
        for (card_id, expected_block) in [("Dodge and Roll", 6), ("Dodge and Roll+", 8)] {
            let mut engine = engine_without_start(
                Vec::new(),
                vec![enemy_no_intent("JawWorm", 40, 40)],
                1,
            );
            force_player_turn(&mut engine);
            engine.state.player.set_status(sid::DEXTERITY, 2);
            engine.state.hand = make_deck(&[card_id]);

            assert!(play_self(&mut engine, card_id));
            assert_eq!(engine.state.player.block, expected_block);
            assert_eq!(engine.state.player.status(sid::NEXT_TURN_BLOCK), expected_block);
            assert_eq!(engine.state.energy, 0);
        }
    }
    card_pair_test!(flying_knee,
        "Flying Knee", 1, 8, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["next_turn_energy"],
        "Flying Knee+", 1, 11, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["next_turn_energy"],
    );
    card_pair_test!(outmaneuver,
        "Outmaneuver", 1, -1, -1, 2, CardType::Skill, CardTarget::None, false, None, &["next_turn_energy"],
        "Outmaneuver+", 1, -1, -1, 3, CardType::Skill, CardTarget::None, false, None, &["next_turn_energy"],
    );
    card_pair_test!(piercing_wail,
        "Piercing Wail", 1, -1, -1, 6, CardType::Skill, CardTarget::AllEnemy, true, None, &["reduce_strength_all_temp"],
        "Piercing Wail+", 1, -1, -1, 8, CardType::Skill, CardTarget::AllEnemy, true, None, &["reduce_strength_all_temp"],
    );
    card_pair_test!(poisoned_stab,
        "Poisoned Stab", 1, 6, -1, 3, CardType::Attack, CardTarget::Enemy, false, None, &["poison"],
        "Poisoned Stab+", 1, 8, -1, 4, CardType::Attack, CardTarget::Enemy, false, None, &["poison"],
    );
    card_pair_test!(prepared,
        "Prepared", 0, -1, -1, 1, CardType::Skill, CardTarget::None, false, None, &["draw", "discard"],
        "Prepared+", 0, -1, -1, 2, CardType::Skill, CardTarget::None, false, None, &["draw", "discard"],
    );
    card_pair_test!(quick_slash,
        "Quick Slash", 1, 8, -1, 1, CardType::Attack, CardTarget::Enemy, false, None, &["draw"],
        "Quick Slash+", 1, 12, -1, 1, CardType::Attack, CardTarget::Enemy, false, None, &["draw"],
    );
    card_pair_test!(slice,
        "Slice", 0, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &[],
        "Slice+", 0, 9, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &[],
    );
    card_pair_test!(sneaky_strike,
        "Sneaky Strike", 2, 12, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["refund_energy_on_discard"],
        "Sneaky Strike+", 2, 16, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["refund_energy_on_discard"],
    );
    card_pair_test!(sucker_punch,
        "Sucker Punch", 1, 7, -1, 1, CardType::Attack, CardTarget::Enemy, false, None, &["weak"],
        "Sucker Punch+", 1, 9, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["weak"],
    );
    card_pair_test!(accuracy,
        "Accuracy", 1, -1, -1, 4, CardType::Power, CardTarget::SelfTarget, false, None, &["accuracy"],
        "Accuracy+", 1, -1, -1, 6, CardType::Power, CardTarget::SelfTarget, false, None, &["accuracy"],
    );

    #[test]
    fn accuracy_cards_stack_and_modify_both_shiv_variants_on_play() {
        // Accuracy.java applies 4 Accuracy for one energy and upgrades magic
        // by 2. AccuracyPower stacks additively; base Shiv damage is 4 and
        // upgraded Shiv damage is 6 before the shared Accuracy amount.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Accuracy.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/AccuracyPower.java
        let state = combat_state_with(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 100, 100)],
            5,
        );
        let mut engine = engine_with_state(state);
        engine.state.hand = make_deck(&["Accuracy", "Shiv", "Accuracy+", "Shiv+"]);
        engine.state.draw_pile.clear();
        engine.state.discard_pile.clear();

        assert!(play_self(&mut engine, "Accuracy"));
        assert_eq!(engine.state.player.status(sid::ACCURACY), 4);
        assert!(play_on_enemy(&mut engine, "Shiv", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 92);

        assert!(play_self(&mut engine, "Accuracy+"));
        assert_eq!(engine.state.player.status(sid::ACCURACY), 10);
        assert!(play_on_enemy(&mut engine, "Shiv+", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 76);
    }
    card_pair_test!(all_out_attack,
        "All Out Attack", 1, 10, -1, -1, CardType::Attack, CardTarget::AllEnemy, false, None, &["discard_random"],
        "All Out Attack+", 1, 14, -1, -1, CardType::Attack, CardTarget::AllEnemy, false, None, &["discard_random"],
    );
    card_pair_test!(backstab,
        "Backstab", 0, 11, -1, -1, CardType::Attack, CardTarget::Enemy, true, None, &["innate"],
        "Backstab+", 0, 15, -1, -1, CardType::Attack, CardTarget::Enemy, true, None, &["innate"],
    );
    card_pair_test!(blur,
        "Blur", 1, -1, 5, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["retain_block"],
        "Blur+", 1, -1, 8, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["retain_block"],
    );
    card_pair_test!(bouncing_flask,
        "Bouncing Flask", 2, -1, -1, 3, CardType::Skill, CardTarget::AllEnemy, false, None, &["poison_random_multi"],
        "Bouncing Flask+", 2, -1, -1, 4, CardType::Skill, CardTarget::AllEnemy, false, None, &["poison_random_multi"],
    );
    card_pair_test!(calculated_gamble,
        "Calculated Gamble", 0, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, &["calculated_gamble"],
        "Calculated Gamble+", 0, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["calculated_gamble"],
    );
    card_pair_test!(caltrops,
        "Caltrops", 1, -1, -1, 3, CardType::Power, CardTarget::SelfTarget, false, None, &["thorns"],
        "Caltrops+", 1, -1, -1, 5, CardType::Power, CardTarget::SelfTarget, false, None, &["thorns"],
    );
    card_pair_test!(catalyst,
        "Catalyst", 1, -1, -1, 2, CardType::Skill, CardTarget::Enemy, true, None, &["catalyst_double"],
        "Catalyst+", 1, -1, -1, 3, CardType::Skill, CardTarget::Enemy, true, None, &["catalyst_triple"],
    );
    card_pair_test!(choke,
        "Choke", 2, 12, -1, 3, CardType::Attack, CardTarget::Enemy, false, None, &["choke"],
        "Choke+", 2, 12, -1, 5, CardType::Attack, CardTarget::Enemy, false, None, &["choke"],
    );
    card_pair_test!(concentrate,
        // Source: Concentrate.java declares CardTarget.SELF, not NONE.
        "Concentrate", 0, -1, -1, 3, CardType::Skill, CardTarget::SelfTarget, false, None, &["discard_gain_energy"],
        "Concentrate+", 0, -1, -1, 2, CardType::Skill, CardTarget::SelfTarget, false, None, &["discard_gain_energy"],
    );
    card_pair_test!(crippling_cloud,
        "Crippling Cloud", 2, -1, -1, 4, CardType::Skill, CardTarget::AllEnemy, true, None, &["poison_all", "weak_all"],
        "Crippling Cloud+", 2, -1, -1, 7, CardType::Skill, CardTarget::AllEnemy, true, None, &["poison_all", "weak_all"],
    );

    #[test]
    fn crippling_cloud_plus_applies_seven_poison_then_two_weak_and_exhausts() {
        // Source: CripplingPoison.java queues 4 Poison then 2 Weak per living
        // enemy; upgradeMagicNumber(3) changes only Poison to 7.
        let mut engine = engine_without_start(
            Vec::new(),
            vec![
                enemy_no_intent("JawWorm", 40, 40),
                enemy_no_intent("Cultist", 35, 35),
            ],
            2,
        );
        force_player_turn(&mut engine);
        engine.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);
        engine.state.hand = make_deck(&["Crippling Cloud+"]);

        assert!(play_self(&mut engine, "Crippling Cloud+"));

        assert_eq!(engine.state.enemies[0].entity.status(sid::ARTIFACT), 0);
        assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 0);
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 2);
        assert_eq!(engine.state.enemies[1].entity.status(sid::POISON), 7);
        assert_eq!(engine.state.enemies[1].entity.status(sid::WEAKENED), 2);
        assert_eq!(engine.state.energy, 0);
        assert_eq!(engine.state.exhaust_pile.len(), 1);
    }

    card_pair_test!(dash,
        "Dash", 2, 10, 10, -1, CardType::Attack, CardTarget::Enemy, false, None, &[],
        "Dash+", 2, 13, 13, -1, CardType::Attack, CardTarget::Enemy, false, None, &[],
    );
    card_pair_test!(distraction,
        "Distraction", 1, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, &["random_skill_to_hand"],
        "Distraction+", 0, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, &["random_skill_to_hand"],
    );
    card_pair_test!(endless_agony,
        "Endless Agony", 0, 4, -1, -1, CardType::Attack, CardTarget::Enemy, true, None, &["copy_on_draw"],
        "Endless Agony+", 0, 6, -1, -1, CardType::Attack, CardTarget::Enemy, true, None, &["copy_on_draw"],
    );
    card_pair_test!(envenom,
        "Envenom", 2, -1, -1, -1, CardType::Power, CardTarget::SelfTarget, false, None, &["envenom"],
        "Envenom+", 1, -1, -1, -1, CardType::Power, CardTarget::SelfTarget, false, None, &["envenom"],
    );

    #[test]
    fn envenom_cards_stack_one_power_each_and_poison_on_positive_attack_damage() {
        // Envenom.java applies a literal one EnvenomPower per copy. The power's
        // onAttack hook applies its stacked amount after positive NORMAL damage.
        // Java: reference/extracted/methods/card/Envenom.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/EnvenomPower.java
        let mut engine = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            4,
        );
        force_player_turn(&mut engine);
        engine.state.hand = make_deck(&["Envenom", "Envenom+", "Strike"]);

        assert!(play_self(&mut engine, "Envenom"));
        assert!(play_self(&mut engine, "Envenom+"));
        assert_eq!(engine.state.player.status(sid::ENVENOM), 2);

        assert!(play_on_enemy(&mut engine, "Strike", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 34);
        assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 2);
        assert_eq!(engine.state.energy, 0);
    }
    card_pair_test!(escape_plan,
        "Escape Plan", 0, -1, 3, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["block_if_skill"],
        "Escape Plan+", 0, -1, 5, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["block_if_skill"],
    );
    card_pair_test!(eviscerate,
        "Eviscerate", 3, 7, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["multi_hit", "cost_reduce_on_discard"],
        "Eviscerate+", 3, 9, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["multi_hit", "cost_reduce_on_discard"],
    );
    card_pair_test!(expertise,
        "Expertise", 1, -1, -1, 6, CardType::Skill, CardTarget::None, false, None, &["draw_to_n"],
        "Expertise+", 1, -1, -1, 7, CardType::Skill, CardTarget::None, false, None, &["draw_to_n"],
    );
    card_pair_test!(finisher,
        "Finisher", 1, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["finisher"],
        "Finisher+", 1, 8, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["finisher"],
    );
    card_pair_test!(flechettes,
        "Flechettes", 1, 4, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["flechettes"],
        "Flechettes+", 1, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["flechettes"],
    );
    card_pair_test!(footwork,
        "Footwork", 1, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, &["gain_dexterity"],
        "Footwork+", 1, -1, -1, 3, CardType::Power, CardTarget::SelfTarget, false, None, &["gain_dexterity"],
    );
    card_pair_test!(heel_hook,
        "Heel Hook", 1, 5, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["if_weak_energy_draw"],
        "Heel Hook+", 1, 8, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["if_weak_energy_draw"],
    );

    #[test]
    fn heel_hook_refunds_and_draws_on_weak_unless_its_damage_ends_combat() {
        // HeelHookAction checks Weak, then queues DamageAction ahead of its
        // GainEnergyAction and DrawCardAction. DamageAction clears both later
        // actions only when that hit makes all monsters basically dead.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/HeelHookAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAction.java
        let mut ordinary = engine_with_enemies(
            make_deck(&["Strike"]),
            vec![enemy_no_intent("JawWorm", 20, 20)],
            1,
        );
        ordinary.state.hand = make_deck(&["Heel Hook"]);
        ordinary.state.draw_pile = make_deck(&["Defend"]);
        assert!(play_on_enemy(&mut ordinary, "Heel Hook", 0));
        assert_eq!(ordinary.state.enemies[0].entity.hp, 15);
        assert_eq!(ordinary.state.energy, 0);
        assert!(ordinary.state.hand.is_empty());

        let mut surviving_fight = engine_with_enemies(
            Vec::new(),
            vec![
                enemy_no_intent("JawWorm", 8, 8),
                enemy_no_intent("Cultist", 20, 20),
            ],
            1,
        );
        surviving_fight.state.hand = make_deck(&["Heel Hook+"]);
        surviving_fight.state.draw_pile = make_deck(&["Defend"]);
        surviving_fight.state.enemies[0].entity.set_status(sid::WEAKENED, 1);
        assert!(play_on_enemy(&mut surviving_fight, "Heel Hook+", 0));
        assert!(surviving_fight.state.enemies[0].entity.is_dead());
        assert_eq!(surviving_fight.state.energy, 1);
        assert_eq!(hand_count(&surviving_fight, "Defend"), 1);

        let mut final_kill = engine_with_enemies(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 8, 8)],
            1,
        );
        final_kill.state.hand = make_deck(&["Heel Hook+"]);
        final_kill.state.draw_pile = make_deck(&["Defend"]);
        final_kill.state.enemies[0].entity.set_status(sid::WEAKENED, 1);
        assert!(play_on_enemy(&mut final_kill, "Heel Hook+", 0));
        assert!(final_kill.state.enemies[0].entity.is_dead());
        assert_eq!(final_kill.state.energy, 0);
        assert_eq!(hand_count(&final_kill, "Defend"), 0);
    }
    card_pair_test!(infinite_blades,
        "Infinite Blades", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["infinite_blades"],
        "Infinite Blades+", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["infinite_blades", "innate"],
    );

    #[test]
    fn infinite_blades_upgrade_is_innate_only_and_stacks_shivs_at_turn_start() {
        // InfiniteBlades.java upgrades only isInnate and applies one power stack.
        // InfiniteBladesPower.java::stackPower adds amounts; atStartOfTurn makes
        // exactly that many base Shivs before the ordinary turn draw.
        let mut engine = engine_with_enemies(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );
        engine.state.hand = make_deck(&["Infinite Blades", "Infinite Blades+"]);
        engine.state.draw_pile = make_deck(&[
            "Defend", "Defend", "Defend", "Defend", "Defend",
        ]);

        assert!(play_self(&mut engine, "Infinite Blades"));
        assert!(play_self(&mut engine, "Infinite Blades+"));
        assert_eq!(engine.state.energy, 1);
        assert_eq!(engine.state.player.status(sid::INFINITE_BLADES), 2);
        assert!(engine.state.discard_pile.is_empty());

        engine.execute_action(&Action::EndTurn);

        assert_eq!(hand_count(&engine, "Shiv"), 2);
        assert_eq!(hand_count(&engine, "Defend"), 5);
        assert_eq!(engine.state.player.status(sid::INFINITE_BLADES), 2);
    }
    card_pair_test!(leg_sweep,
        "Leg Sweep", 2, -1, 11, 2, CardType::Skill, CardTarget::Enemy, false, None, &["weak"],
        "Leg Sweep+", 2, -1, 14, 3, CardType::Skill, CardTarget::Enemy, false, None, &["weak"],
    );

    #[test]
    fn leg_sweep_source_applies_weak_before_gaining_block() {
        // LegSweep.java queues ApplyPowerAction(Weak) before GainBlockAction.
        // With one Artifact and Wave of the Hand, Leg Sweep's own Weak is
        // blocked first; the later Block gain then applies exactly one Weak.
        for (card_id, block) in [("Leg Sweep", 11), ("Leg Sweep+", 14)] {
            let mut engine = engine_with_enemies(
                Vec::new(),
                vec![enemy_no_intent("JawWorm", 40, 40)],
                2,
            );
            engine.state.hand = make_deck(&[card_id]);
            engine.state.player.set_status(sid::WAVE_OF_THE_HAND, 1);
            engine.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);

            assert!(play_on_enemy(&mut engine, card_id, 0));

            assert_eq!(engine.state.enemies[0].entity.status(sid::ARTIFACT), 0);
            assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 1);
            assert_eq!(engine.state.player.block, block);
            assert_eq!(engine.state.energy, 0);
        }
    }
    card_pair_test!(masterful_stab,
        "Masterful Stab", 0, 12, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["cost_increase_on_hp_loss"],
        "Masterful Stab+", 0, 16, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["cost_increase_on_hp_loss"],
    );
    card_pair_test!(noxious_fumes,
        "Noxious Fumes", 1, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, &["noxious_fumes"],
        "Noxious Fumes+", 1, -1, -1, 3, CardType::Power, CardTarget::SelfTarget, false, None, &["noxious_fumes"],
    );
    card_pair_test!(predator,
        "Predator", 2, 15, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["draw_next_turn"],
        "Predator+", 2, 20, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["draw_next_turn"],
    );
    card_pair_test!(reflex,
        "Reflex", -2, -1, -1, 2, CardType::Skill, CardTarget::None, false, None, &["unplayable", "draw_on_discard"],
        "Reflex+", -2, -1, -1, 3, CardType::Skill, CardTarget::None, false, None, &["unplayable", "draw_on_discard"],
    );
    card_pair_test!(riddle_with_holes,
        "Riddle With Holes", 2, 3, -1, 5, CardType::Attack, CardTarget::Enemy, false, None, &["multi_hit"],
        "Riddle With Holes+", 2, 4, -1, 5, CardType::Attack, CardTarget::Enemy, false, None, &["multi_hit"],
    );
    card_pair_test!(setup,
        "Setup", 1, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["setup"],
        "Setup+", 0, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["setup"],
    );
    card_pair_test!(skewer,
        "Skewer", -1, 7, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["x_cost"],
        "Skewer+", -1, 10, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["x_cost"],
    );
    card_pair_test!(tactician,
        "Tactician", -2, -1, -1, 1, CardType::Skill, CardTarget::None, false, None, &["unplayable", "energy_on_discard"],
        "Tactician+", -2, -1, -1, 2, CardType::Skill, CardTarget::None, false, None, &["unplayable", "energy_on_discard"],
    );
    card_pair_test!(terror,
        "Terror", 1, -1, -1, 99, CardType::Skill, CardTarget::Enemy, true, None, &["vulnerable"],
        "Terror+", 0, -1, -1, 99, CardType::Skill, CardTarget::Enemy, true, None, &["vulnerable"],
    );
    card_pair_test!(well_laid_plans,
        "Well-Laid Plans", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["well_laid_plans"],
        "Well-Laid Plans+", 1, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, &["well_laid_plans"],
    );
    card_pair_test!(a_thousand_cuts,
        "A Thousand Cuts", 2, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["thousand_cuts"],
        "A Thousand Cuts+", 2, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, &["thousand_cuts"],
    );

    #[test]
    fn a_thousand_cuts_cards_install_stack_and_trigger_the_java_power() {
        // AThousandCuts.java costs 2 and applies ThousandCutsPower for magic
        // 1, upgraded to 2. ThousandCutsPower stacks additively and its
        // onAfterCardPlayed THORNS matrix hits every enemy after each card.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/AThousandCuts.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ThousandCutsPower.java
        let state = combat_state_with(
            Vec::new(),
            vec![
                enemy_no_intent("JawWorm", 40, 40),
                enemy_no_intent("Cultist", 35, 35),
            ],
            8,
        );
        let mut engine = engine_with_state(state);
        engine.state.hand = make_deck(&[
            "A Thousand Cuts",
            "Defend",
            "A Thousand Cuts+",
            "Defend",
        ]);
        engine.state.draw_pile.clear();
        engine.state.discard_pile.clear();

        assert!(play_self(&mut engine, "A Thousand Cuts"));
        assert_eq!(engine.state.player.status(sid::THOUSAND_CUTS), 1);
        assert_eq!(engine.state.enemies[0].entity.hp, 40);
        assert_eq!(engine.state.enemies[1].entity.hp, 35);

        assert!(play_self(&mut engine, "Defend"));
        assert_eq!(engine.state.enemies[0].entity.hp, 39);
        assert_eq!(engine.state.enemies[1].entity.hp, 34);

        assert!(play_self(&mut engine, "A Thousand Cuts+"));
        assert_eq!(engine.state.player.status(sid::THOUSAND_CUTS), 3);
        assert_eq!(engine.state.enemies[0].entity.hp, 38);
        assert_eq!(engine.state.enemies[1].entity.hp, 33);

        assert!(play_self(&mut engine, "Defend"));
        assert_eq!(engine.state.enemies[0].entity.hp, 35);
        assert_eq!(engine.state.enemies[1].entity.hp, 30);
    }
    card_pair_test!(adrenaline,
        "Adrenaline", 0, -1, -1, 1, CardType::Skill, CardTarget::None, true, None, &["draw"],
        "Adrenaline+", 0, -1, -1, 2, CardType::Skill, CardTarget::None, true, None, &["draw"],
    );

    #[test]
    fn adrenaline_variants_gain_full_energy_when_the_two_card_draw_hits_hand_cap() {
        // Adrenaline.java queues 1 Energy (2 upgraded), then DrawCardAction(2),
        // costs zero, and exhausts. With ten cards before play, removing
        // Adrenaline leaves one hand slot, so only one queued draw fits.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Adrenaline.java
        for (card_id, expected_energy) in [("Adrenaline", 1), ("Adrenaline+", 2)] {
            let state = combat_state_with(
                Vec::new(),
                vec![enemy_no_intent("JawWorm", 40, 40)],
                0,
            );
            let mut engine = engine_with_state(state);
            engine.state.hand = make_deck(&[
                card_id,
                "Strike", "Strike", "Strike",
                "Defend", "Defend", "Defend",
                "Neutralize", "Survivor", "Backflip",
            ]);
            engine.state.draw_pile = make_deck(&["Strike", "Defend", "Neutralize"]);
            engine.state.discard_pile.clear();
            engine.state.energy = 0;

            assert!(play_self(&mut engine, card_id));
            assert_eq!(engine.state.energy, expected_energy);
            assert_eq!(engine.state.hand.len(), 10);
            assert_eq!(engine.state.draw_pile.len(), 2);
            assert!(engine.state.exhaust_pile.iter().any(|card| {
                engine.card_registry.card_name(card.def_id) == card_id
            }));
        }
    }
    card_pair_test!(after_image,
        "After Image", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["after_image"],
        "After Image+", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["after_image", "innate"],
    );

    #[test]
    fn after_image_upgrade_is_innate_and_existing_stacks_fire_before_new_stack() {
        // AfterImage.java keeps cost 1 on upgrade and sets isInnate. The card
        // queues ApplyPowerAction(1), while AfterImagePower.onUseCard fires
        // before the played card's queued power stack resolves: the first copy
        // gives no block, the second gives 1, and the next card gives 2.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/AfterImage.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/AfterImagePower.java
        let opening_state = combat_state_with(
            make_deck(&[
                "Strike", "Strike", "Strike", "Strike", "Strike",
                "Strike", "Strike", "Strike", "Strike", "After Image+",
            ]),
            vec![enemy_no_intent("JawWorm", 60, 60)],
            3,
        );
        let opening = engine_with_state(opening_state);
        assert!(opening.state.hand.iter().any(|card| {
            opening.card_registry.card_name(card.def_id) == "After Image+"
        }));

        let state = combat_state_with(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 60, 60)],
            3,
        );
        let mut engine = engine_with_state(state);
        engine.state.hand = make_deck(&["After Image", "After Image+", "Strike"]);
        engine.state.draw_pile.clear();
        engine.state.discard_pile.clear();
        engine.state.energy = 3;

        assert!(play_self(&mut engine, "After Image"));
        assert_eq!(engine.state.energy, 2);
        assert_eq!(engine.state.player.status(sid::AFTER_IMAGE), 1);
        assert_eq!(engine.state.player.block, 0);

        assert!(play_self(&mut engine, "After Image+"));
        assert_eq!(engine.state.energy, 1);
        assert_eq!(engine.state.player.status(sid::AFTER_IMAGE), 2);
        assert_eq!(engine.state.player.block, 1);

        assert!(play_on_enemy(&mut engine, "Strike", 0));
        assert_eq!(engine.state.energy, 0);
        assert_eq!(engine.state.player.block, 3);
        assert_eq!(engine.state.enemies[0].entity.hp, 54);
    }
    card_pair_test!(alchemize,
        "Alchemize", 1, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, &["alchemize"],
        "Alchemize+", 0, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, &["alchemize"],
    );
    card_pair_test!(bullet_time,
        "Bullet Time", 3, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["bullet_time"],
        "Bullet Time+", 2, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["bullet_time"],
    );
    card_pair_test!(burst,
        "Burst", 1, -1, -1, 1, CardType::Skill, CardTarget::SelfTarget, false, None, &["burst"],
        "Burst+", 1, -1, -1, 2, CardType::Skill, CardTarget::SelfTarget, false, None, &["burst"],
    );

    #[test]
    fn burst_stacks_replays_each_skill_and_expires_with_original_x_value() {
        // Burst.java applies 1 BurstPower (2 upgraded). BurstPower replays one
        // non-purge Skill per charge with the original energyOnUse, consumes a
        // charge per original Skill, and removes all unused charges at turn end.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Burst.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BurstPower.java
        let mut upgraded = engine_with_enemies(
            Vec::new(),
            vec![enemy("Dummy", 80, 80, 1, 0, 1)],
            4,
        );
        upgraded.state.hand = make_deck(&["Burst+", "Defend", "Backflip"]);
        upgraded.state.draw_pile = make_deck(&["Strike", "Strike", "Strike", "Strike"]);
        upgraded.state.energy = 4;

        assert!(play_self(&mut upgraded, "Burst+"));
        assert_eq!(upgraded.state.player.status(sid::BURST), 2);
        assert!(play_self(&mut upgraded, "Defend"));
        assert_eq!(upgraded.state.player.block, 10);
        assert_eq!(upgraded.state.player.status(sid::BURST), 1);
        assert!(play_self(&mut upgraded, "Backflip"));
        assert_eq!(upgraded.state.player.block, 20);
        assert_eq!(upgraded.state.player.status(sid::BURST), 0);
        assert_eq!(upgraded.state.hand.len(), 4);

        let mut expires = engine_with_enemies(
            Vec::new(),
            vec![enemy("Dummy", 80, 80, 1, 0, 1)],
            1,
        );
        expires.state.hand = make_deck(&["Burst"]);
        expires.state.energy = 1;
        assert!(play_self(&mut expires, "Burst"));
        assert_eq!(expires.state.player.status(sid::BURST), 1);
        expires.execute_action(&Action::EndTurn);
        assert_eq!(expires.state.player.status(sid::BURST), 0);

        let mut x_skill = engine_with_enemies(
            Vec::new(),
            vec![enemy("Dummy", 80, 80, 1, 0, 1)],
            4,
        );
        x_skill.state.hand = make_deck(&["Burst", "Malaise"]);
        x_skill.state.energy = 4;
        assert!(play_self(&mut x_skill, "Burst"));
        assert!(play_on_enemy(&mut x_skill, "Malaise", 0));
        assert_eq!(x_skill.state.enemies[0].entity.status(sid::WEAKENED), 6);
        assert_eq!(x_skill.state.enemies[0].entity.status(sid::STRENGTH), -6);
        assert_eq!(x_skill.state.energy, 0);
        assert_eq!(x_skill.state.player.status(sid::BURST), 0);
    }
    card_pair_test!(corpse_explosion,
        "Corpse Explosion", 2, -1, -1, 6, CardType::Skill, CardTarget::Enemy, false, None, &["corpse_explosion"],
        "Corpse Explosion+", 2, -1, -1, 9, CardType::Skill, CardTarget::Enemy, false, None, &["corpse_explosion"],
    );

    #[test]
    fn corpse_explosion_variants_stack_poison_and_thorns_death_damage() {
        // Sources: CorpseExplosion.java applies 6/9 Poison followed by one
        // CorpseExplosionPower. CorpseExplosionPower.java stacks amount and on
        // death deals maxHealth * amount as source-less THORNS damage to all.
        let mut engine = engine_without_start(
            Vec::new(),
            vec![
                enemy_no_intent("JawWorm", 50, 50),
                enemy_no_intent("JawWorm", 120, 120),
            ],
            4,
        );
        force_player_turn(&mut engine);
        engine.state.enemies[1].entity.set_status(sid::MALLEABLE, 3);
        engine.state.hand = make_deck(&["Corpse Explosion", "Corpse Explosion+"]);

        assert!(play_on_enemy(&mut engine, "Corpse Explosion", 0));
        assert!(play_on_enemy(&mut engine, "Corpse Explosion+", 0));
        assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 15);
        assert_eq!(engine.state.enemies[0].entity.status(sid::CORPSE_EXPLOSION), 2);

        engine.state.enemies[0].entity.hp = 1;
        engine.deal_damage_to_enemy(0, 1);

        assert!(engine.state.enemies[0].entity.is_dead());
        assert_eq!(engine.state.enemies[1].entity.hp, 20);
        assert_eq!(engine.state.enemies[1].entity.block, 0);
        assert_eq!(engine.state.enemies[1].entity.status(sid::MALLEABLE), 3);
    }

    #[test]
    fn corpse_explosion_poison_then_power_each_consume_artifact() {
        // CorpseExplosion.java queues two ApplyPowerActions in order. Both
        // PoisonPower and CorpseExplosionPower are DEBUFF powers, so two
        // Artifact charges block both applications.
        let mut engine = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            2,
        );
        force_player_turn(&mut engine);
        engine.state.enemies[0].entity.set_status(sid::ARTIFACT, 2);
        engine.state.hand = make_deck(&["Corpse Explosion+"]);

        assert!(play_on_enemy(&mut engine, "Corpse Explosion+", 0));

        assert_eq!(engine.state.enemies[0].entity.status(sid::ARTIFACT), 0);
        assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 0);
        assert_eq!(engine.state.enemies[0].entity.status(sid::CORPSE_EXPLOSION), 0);
    }

    card_pair_test!(die_die_die,
        "Die Die Die", 1, 13, -1, -1, CardType::Attack, CardTarget::AllEnemy, true, None, &[],
        "Die Die Die+", 1, 17, -1, -1, CardType::Attack, CardTarget::AllEnemy, true, None, &[],
    );
    card_pair_test!(doppelganger,
        "Doppelganger", -1, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, &["x_cost"],
        "Doppelganger+", -1, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, &["x_cost"],
    );
    card_pair_test!(glass_knife,
        "Glass Knife", 1, 8, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["multi_hit", "glass_knife"],
        "Glass Knife+", 1, 12, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["multi_hit", "glass_knife"],
    );
    card_pair_test!(grand_finale,
        "Grand Finale", 0, 50, -1, -1, CardType::Attack, CardTarget::AllEnemy, false, None, &["only_empty_draw"],
        "Grand Finale+", 0, 60, -1, -1, CardType::Attack, CardTarget::AllEnemy, false, None, &["only_empty_draw"],
    );
    card_pair_test!(malaise,
        "Malaise", -1, -1, -1, 0, CardType::Skill, CardTarget::Enemy, true, None, &["x_cost"],
        "Malaise+", -1, -1, -1, 1, CardType::Skill, CardTarget::Enemy, true, None, &["x_cost"],
    );
    #[test]
    fn nightmare_java_parity() {
        let reg = reg();
        let expected = [Effect::ChooseCards {
            source: Pile::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::StoreCardForNextTurnCopies,
            min_picks: AmountSource::Fixed(1),
            max_picks: AmountSource::Fixed(1),
            post_choice_draw: AmountSource::Fixed(0),
        }];
        for id in ["Night Terror", "Night Terror+"] {
            let card = reg.get(id).unwrap_or_else(|| panic!("missing card {id}"));
            assert_eq!(card.cost, if id.ends_with('+') { 2 } else { 3 }, "{id} cost");
            assert_eq!(card.base_damage, -1, "{id} damage");
            assert_eq!(card.base_block, -1, "{id} block");
            assert_eq!(card.base_magic, 3, "{id} magic");
            assert_eq!(card.card_type, CardType::Skill, "{id} type");
            assert_eq!(card.target, CardTarget::None, "{id} target");
            assert!(card.exhaust, "{id} exhaust");
            assert_eq!(card.enter_stance, None, "{id} stance");
            assert_eq!(card.effect_data, &expected, "{id} effect_data");
        }
    }
    // Source: cards/green/PhantasmalKiller.java — SELF target, neither version
    // is Ethereal, and upgradeBaseCost changes only the cost from 1 to 0.
    card_pair_test!(phantasmal_killer,
        "Phantasmal Killer", 1, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["phantasmal_killer"],
        "Phantasmal Killer+", 0, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["phantasmal_killer"],
    );
    card_pair_test!(storm_of_steel,
        "Storm of Steel", 1, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["storm_of_steel"],
        "Storm of Steel+", 1, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["storm_of_steel"],
    );
    card_pair_test!(tools_of_the_trade,
        "Tools of the Trade", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["tools_of_the_trade"],
        "Tools of the Trade+", 0, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["tools_of_the_trade"],
    );
    card_pair_test!(unload,
        "Unload", 1, 14, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["discard_non_attacks"],
        "Unload+", 1, 18, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["discard_non_attacks"],
    );
    card_pair_test!(wraith_form,
        "Wraith Form", 3, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, &[],
        "Wraith Form+", 3, -1, -1, 3, CardType::Power, CardTarget::SelfTarget, false, None, &[],
    );

    // ---------------------------------------------------------------------
    // Breadth-first runtime checks for cards that the Rust engine already
    // wires up, plus a few exact Java-mechanic coverage checks.
    // ---------------------------------------------------------------------

    #[test]
    fn neutralize_applies_weak_and_damage() {
        let mut engine = engine_with(make_deck_n("Neutralize", 6), 40, 0);
        ensure_in_hand(&mut engine, "Neutralize");
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Neutralize", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 3);
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 1);
    }

    #[test]
    fn lethal_neutralize_damage_prevents_its_later_weak_action() {
        // Neutralize.java queues DamageAction before ApplyPowerAction. The
        // latter cancels when its target is dead; a second living enemy keeps
        // combat active so this specifically exercises the target gate.
        let mut engine = engine_without_start(
            Vec::new(),
            vec![
                enemy_no_intent("JawWorm", 3, 3),
                enemy_no_intent("Cultist", 40, 40),
            ],
            3,
        );
        force_player_turn(&mut engine);
        engine.state.hand = make_deck(&["Neutralize"]);

        assert!(play_on_enemy(&mut engine, "Neutralize", 0));
        assert!(engine.state.enemies[0].entity.is_dead());
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 0);
        assert_eq!(engine.state.enemies[1].entity.hp, 40);
        assert!(!engine.state.combat_over);
    }

    #[test]
    fn backflip_blocks_and_draws() {
        let mut engine = engine_with(make_deck_n("Backflip", 8), 40, 0);
        ensure_in_hand(&mut engine, "Backflip");
        let hand_before = engine.state.hand.len();
        assert!(play_self(&mut engine, "Backflip"));
        assert_eq!(engine.state.player.block, 5);
        assert_eq!(engine.state.hand.len(), hand_before + 1);
    }

    #[test]
    fn quick_slash_draws_one() {
        let mut engine = engine_with(make_deck_n("Quick Slash", 8), 40, 0);
        ensure_in_hand(&mut engine, "Quick Slash");
        let hand_before = engine.state.hand.len();
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Quick Slash", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 8);
        assert_eq!(engine.state.hand.len(), hand_before);
    }

    #[test]
    fn slice_deals_exact_damage() {
        let mut engine = engine_with(make_deck_n("Slice", 8), 40, 0);
        ensure_in_hand(&mut engine, "Slice");
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Slice", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 6);
    }

    #[test]
    fn sucker_punch_applies_weak() {
        let mut engine = engine_with(make_deck_n("Sucker Punch", 8), 40, 0);
        ensure_in_hand(&mut engine, "Sucker Punch");
        assert!(play_on_enemy(&mut engine, "Sucker Punch", 0));
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 1);
    }

    #[test]
    fn leg_sweep_blocks_and_weakens() {
        let mut engine = engine_with(make_deck_n("Leg Sweep", 8), 40, 0);
        ensure_in_hand(&mut engine, "Leg Sweep");
        assert!(play_on_enemy(&mut engine, "Leg Sweep", 0));
        assert_eq!(engine.state.player.block, 11);
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 2);
    }

    #[test]
    fn dash_deals_damage_and_block() {
        let mut engine = engine_with(make_deck_n("Dash", 8), 50, 0);
        ensure_in_hand(&mut engine, "Dash");
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Dash", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 10);
        assert_eq!(engine.state.player.block, 10);
    }

    #[test]
    fn escape_plan_draws_and_blocks() {
        let mut engine = engine_with(make_deck_n("Escape Plan", 8), 40, 0);
        ensure_in_hand(&mut engine, "Escape Plan");
        let hand_before = engine.state.hand.len();
        assert!(play_self(&mut engine, "Escape Plan"));
        assert_eq!(engine.state.player.block, 3);
        assert_eq!(engine.state.hand.len(), hand_before);
    }

    #[test]
    fn catalyst_doubles_poison() {
        let mut engine = engine_with(make_deck_n("Catalyst", 8), 40, 0);
        engine.state.enemies[0].entity.set_status(sid::POISON, 5);
        ensure_in_hand(&mut engine, "Catalyst");
        assert!(play_on_enemy(&mut engine, "Catalyst", 0));
        assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 10);
        assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Catalyst"));
    }

    #[test]
    fn catalyst_plus_triples_poison() {
        let mut engine = engine_with(make_deck_n("Catalyst+", 8), 40, 0);
        engine.state.enemies[0].entity.set_status(sid::POISON, 5);
        ensure_in_hand(&mut engine, "Catalyst+");
        assert!(play_on_enemy(&mut engine, "Catalyst+", 0));
        assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 15);
    }

    #[test]
    fn terror_applies_vulnerable() {
        let mut engine = engine_with(make_deck_n("Terror", 8), 40, 0);
        ensure_in_hand(&mut engine, "Terror");
        assert!(play_on_enemy(&mut engine, "Terror", 0));
        assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 99);
        assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Terror"));
    }

    #[test]
    fn skewer_spends_all_energy() {
        let mut engine = engine_with(make_deck_n("Skewer", 8), 100, 0);
        ensure_in_hand(&mut engine, "Skewer");
        engine.state.energy = 3;
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Skewer", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 21);
        assert_eq!(engine.state.energy, 0);
    }

    #[test]
    fn riddle_with_holes_hits_five_times() {
        // RiddleWithHoles.java queues exactly five DamageActions using the
        // card's already Strength-modified damage. Base 3 plus 2 Strength is
        // therefore five hits of 5, and the canonical ID capitalizes "With".
        // Java: reference/extracted/methods/card/RiddleWithHoles.java
        let mut engine = engine_with(make_deck_n("Riddle With Holes", 8), 100, 0);
        engine.state.player.set_status(sid::STRENGTH, 2);
        ensure_in_hand(&mut engine, "Riddle With Holes");
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Riddle With Holes", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 25);
    }

    #[test]
    fn all_out_attack_hits_all_enemies() {
        // AllOutAttack.java uses canonical ID "All Out Attack", deals 10 to
        // every enemy (14 upgraded), then DiscardAction randomly discards one.
        // With more than one card, that action consumes one cardRandomRng tick
        // and triggers manual-discard callbacks; with one card it consumes no RNG.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/AllOutAttack.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DiscardAction.java
        let enemies = vec![
            enemy("A", 50, 50, 1, 0, 1),
            enemy("B", 50, 50, 1, 0, 1),
        ];
        let mut engine = engine_with_enemies(Vec::new(), enemies, 1);
        engine.state.hand = make_deck(&["All Out Attack", "Tactician", "Strike", "Defend"]);
        engine.state.draw_pile.clear();
        engine.state.discard_pile.clear();
        engine.state.energy = 1;
        let mut oracle = engine.card_random_rng.clone();
        let expected_index = oracle.random(2) as usize;
        let expected_discard = ["Tactician", "Strike", "Defend"][expected_index];
        let card_random_before = engine.rng_counters()["cardRandom"];
        let card_before = engine.rng_counters()["card"];

        assert!(play_self(&mut engine, "All Out Attack"));
        assert_eq!(engine.state.enemies[0].entity.hp, 40);
        assert_eq!(engine.state.enemies[1].entity.hp, 40);
        assert_eq!(engine.rng_counters()["cardRandom"], card_random_before + 1);
        assert_eq!(engine.rng_counters()["card"], card_before);
        assert!(engine.state.discard_pile.iter().any(|card| {
            engine.card_registry.card_name(card.def_id) == expected_discard
        }));
        assert_eq!(engine.state.player.status(sid::DISCARDED_THIS_TURN), 1);
        assert_eq!(engine.state.energy, i32::from(expected_discard == "Tactician"));

        let mut upgraded = engine_with_enemies(
            Vec::new(),
            vec![enemy("A", 50, 50, 1, 0, 1), enemy("B", 50, 50, 1, 0, 1)],
            1,
        );
        upgraded.state.hand = make_deck(&["All Out Attack+", "Tactician"]);
        upgraded.state.draw_pile.clear();
        upgraded.state.discard_pile.clear();
        upgraded.state.energy = 1;
        let card_random_before = upgraded.rng_counters()["cardRandom"];
        assert!(play_self(&mut upgraded, "All Out Attack+"));
        assert_eq!(upgraded.state.enemies[0].entity.hp, 36);
        assert_eq!(upgraded.state.enemies[1].entity.hp, 36);
        assert_eq!(upgraded.rng_counters()["cardRandom"], card_random_before);
        assert_eq!(upgraded.state.energy, 1);
    }

    #[test]
    fn die_die_die_hits_all_enemies() {
        let enemies = vec![
            enemy("A", 50, 50, 1, 0, 1),
            enemy("B", 50, 50, 1, 0, 1),
        ];
        let mut engine = engine_with_enemies(make_deck_n("Die Die Die", 8), enemies, 3);
        ensure_in_hand(&mut engine, "Die Die Die");
        assert!(play_self(&mut engine, "Die Die Die"));
        assert_eq!(engine.state.enemies[0].entity.hp, 37);
        assert_eq!(engine.state.enemies[1].entity.hp, 37);
        assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Die Die Die"));
    }

    #[test]
    fn adrenaline_gains_energy_and_draws() {
        let mut engine = engine_with(make_deck_n("Adrenaline", 8), 40, 0);
        ensure_in_hand(&mut engine, "Adrenaline");
        let energy = engine.state.energy;
        let hand_before = engine.state.hand.len();
        assert!(play_self(&mut engine, "Adrenaline"));
        assert_eq!(engine.state.energy, energy + 1);
        assert_eq!(engine.state.hand.len(), hand_before + 1);
        assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Adrenaline"));
    }

    #[test]
    fn bullet_time_zeroes_only_the_current_non_x_hand_and_no_draw_obeys_artifact() {
        // BulletTime.java queues NoDrawPower then ApplyBulletTimeAction. The
        // latter loops the current hand and calls setCostForTurn(-9), which
        // AbstractCard clamps to zero only for cards whose costForTurn is
        // non-negative. X-cost cards stay X, and cards drawn later are not
        // modified. NoDrawPower is a DEBUFF, so Artifact can negate it without
        // preventing the subsequent hand-cost action.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/BulletTime.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ApplyBulletTimeAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/AbstractCard.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/NoDrawPower.java
        let mut engine = engine_with_enemies(
            Vec::new(),
            vec![enemy("Dummy", 50, 50, 1, 0, 1)],
            4,
        );
        engine.state.hand = make_deck(&["Bullet Time", "Strike", "Skewer"]);
        engine.state.energy = 4;

        assert!(play_self(&mut engine, "Bullet Time"));
        assert_eq!(engine.state.player.status(sid::NO_DRAW), 1);
        assert_eq!(engine.state.player.status(sid::BULLET_TIME), 0);
        let strike = engine.state.hand.iter().find(|card| {
            engine.card_registry.card_name(card.def_id) == "Strike"
        }).expect("Strike in hand");
        let skewer = engine.state.hand.iter().find(|card| {
            engine.card_registry.card_name(card.def_id) == "Skewer"
        }).expect("Skewer in hand");
        assert_eq!(strike.cost, 0);
        assert_eq!(skewer.cost, -1);
        assert!(play_on_enemy(&mut engine, "Strike", 0));
        assert!(play_on_enemy(&mut engine, "Skewer", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 37);
        assert_eq!(engine.state.energy, 0);

        let mut artifact = engine_with_enemies(
            Vec::new(),
            vec![enemy("Dummy", 50, 50, 1, 0, 1)],
            2,
        );
        artifact.state.hand = make_deck(&["Bullet Time+", "Strike"]);
        artifact.state.draw_pile = make_deck(&["Bash"]);
        artifact.state.player.set_status(sid::ARTIFACT, 1);
        artifact.state.energy = 2;

        assert!(play_self(&mut artifact, "Bullet Time+"));
        assert_eq!(artifact.state.player.status(sid::ARTIFACT), 0);
        assert_eq!(artifact.state.player.status(sid::NO_DRAW), 0);
        artifact.draw_cards(1);
        let strike_idx = artifact.state.hand.iter().position(|card| {
            artifact.card_registry.card_name(card.def_id) == "Strike"
        }).expect("pre-existing Strike");
        let bash_idx = artifact.state.hand.iter().position(|card| {
            artifact.card_registry.card_name(card.def_id) == "Bash"
        }).expect("later-drawn Bash");
        let legal = artifact.get_legal_actions();
        assert!(legal.iter().any(|action| matches!(
            action,
            Action::PlayCard { card_idx, .. } if *card_idx == strike_idx
        )));
        assert!(!legal.iter().any(|action| matches!(
            action,
            Action::PlayCard { card_idx, .. } if *card_idx == bash_idx
        )));
    }

    #[test]
    fn doppelganger_uses_x_upgrade_and_chemical_x_for_matching_next_turn_bonuses() {
        // DoppelgangerAction.java starts with energyOnUse, adds Chemical X's
        // 2, then adds 1 if upgraded. It applies equal next-turn energy/draw
        // powers only when the resulting effect is positive.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DoppelgangerAction.java
        for (card_id, energy, chemical_x, expected) in [
            ("Doppelganger", 0, false, 0),
            ("Doppelganger+", 0, false, 1),
            ("Doppelganger", 3, false, 3),
            ("Doppelganger+", 3, true, 6),
        ] {
            let mut engine = engine_with(make_deck_n("Strike", 8), 40, 0);
            ensure_in_hand(&mut engine, card_id);
            engine.state.energy = energy;
            if chemical_x {
                engine.state.relics.push("Chemical X".to_string());
            }

            assert!(play_self(&mut engine, card_id));
            assert_eq!(engine.state.player.status(sid::DOPPELGANGER_DRAW), expected);
            assert_eq!(engine.state.player.status(sid::DOPPELGANGER_ENERGY), expected);
            assert_eq!(engine.state.energy, 0);
            assert!(engine.state.exhaust_pile.iter().any(|c| {
                engine.card_registry.card_name(c.def_id) == card_id
            }));
        }
    }

    #[test]
    fn malaise_applies_weak_and_strength_down() {
        let mut engine = engine_with(make_deck_n("Malaise", 8), 40, 0);
        engine.state.enemies[0].entity.set_status(sid::STRENGTH, 4);
        ensure_in_hand(&mut engine, "Malaise");
        engine.state.energy = 3;
        assert!(play_on_enemy(&mut engine, "Malaise", 0));
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 3);
        assert_eq!(engine.state.enemies[0].entity.strength(), 1);
    }

    #[test]
    fn wraith_form_sets_intangible() {
        let mut engine = engine_with(make_deck_n("Wraith Form", 8), 40, 0);
        ensure_in_hand(&mut engine, "Wraith Form");
        assert!(play_self(&mut engine, "Wraith Form"));
        assert_eq!(engine.state.player.status(sid::INTANGIBLE), 2);
        assert_eq!(engine.state.player.status(sid::WRAITH_FORM), 1);
    }

    #[test]
    fn grand_finale_is_blocked_when_draw_pile_is_not_empty() {
        let state = combat_state_with(
            make_deck(&["Strike", "Strike", "Strike", "Strike", "Strike", "Defend"]),
            vec![enemy("A", 60, 60, 1, 0, 1)],
            3,
        );
        let mut engine = engine_with_state(state);
        ensure_in_hand(&mut engine, "Grand Finale");
        let grand_finale_idx = engine.state.hand.iter().position(|card| engine.card_registry.card_name(card.def_id) == "Grand Finale").expect("Grand Finale should be in hand");
        assert!(
            !engine.get_legal_actions().iter().any(|action| matches!(
                action,
                Action::PlayCard { card_idx, .. } if *card_idx == grand_finale_idx
            ))
        );
        assert_eq!(hand_count(&engine, "Grand Finale"), 1);
    }

    #[test]
    fn grand_finale_hits_for_50_when_draw_pile_empty() {
        let state = combat_state_with(Vec::new(), vec![enemy("A", 90, 90, 1, 0, 1)], 3);
        let mut engine = engine_with_state(state);
        ensure_in_hand(&mut engine, "Grand Finale");
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_self(&mut engine, "Grand Finale"));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 50);
    }

    #[test]
    fn backstab_exhausts_on_play() {
        let mut engine = engine_with(make_deck_n("Backstab", 8), 40, 0);
        ensure_in_hand(&mut engine, "Backstab");
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Backstab", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 11);
        assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Backstab"));
    }

}
