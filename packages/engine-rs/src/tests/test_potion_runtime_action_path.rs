use crate::actions::Action;
use crate::cards::CardType;
use crate::engine::{ChoiceOption, ChoiceReason, CombatPhase};
use crate::state::Stance;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with_state, make_deck, TEST_SEED,
};

fn use_potion(engine: &mut crate::engine::CombatEngine, potion_idx: usize, target_idx: i32) {
    engine.execute_action(&Action::UsePotion {
        potion_idx,
        target_idx,
    });
}

fn hand_names(engine: &crate::engine::CombatEngine) -> Vec<&str> {
    engine
        .state
        .hand
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id))
        .collect()
}

const COLORLESS_CHOICES: &[&str] = &[
    "Apotheosis",
    "Bandage Up",
    "Bite",
    "Blind",
    "Chrysalis",
    "Dark Shackles",
    "Deep Breath",
    "Defend",
    "Discovery",
    "Dramatic Entrance",
    "Enlightenment",
    "Finesse",
    "Flash of Steel",
    "Forethought",
    "Ghostly",
    "Good Instincts",
    "HandOfGreed",
    "Impatience",
    "J.A.X.",
    "Jack Of All Trades",
    "Madness",
    "Magnetism",
    "Master of Strategy",
    "Mayhem",
    "Metamorphosis",
    "Mind Blast",
    "Panacea",
    "Panache",
    "PanicButton",
    "Purity",
    "RitualDagger",
    "Sadistic Nature",
    "Secret Technique",
    "Secret Weapon",
    "Strike",
    "Swift Strike",
    "The Bomb",
    "Thinking Ahead",
    "Transmutation",
    "Trip",
    "Violence",
];

#[test]
fn generation_potions_use_engine_action_path_and_consume_slot() {
    // Java oracle:
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/AttackPotion.java
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/SkillPotion.java
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/PowerPotion.java
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/ColorlessPotion.java
    // - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DiscoveryAction.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Strike", "Defend", "Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();

    let cases = [
        ("AttackPotion", Some(CardType::Attack), false),
        ("SkillPotion", Some(CardType::Skill), false),
        ("PowerPotion", Some(CardType::Power), false),
        ("ColorlessPotion", None, true),
    ];

    for (potion, expected_type, expect_colorless) in cases {
        let slot = 0;
        engine.state.potions = vec![String::new(); 3];
        engine.state.hand.clear();
        engine.state.potions[slot] = potion.to_string();

        let actions = engine.get_legal_actions();
        assert!(actions.iter().any(|action| {
            matches!(
                action,
                Action::UsePotion {
                    potion_idx,
                    target_idx: -1
                } if *potion_idx == slot
            )
        }));

        use_potion(&mut engine, slot, -1);

        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        let choice = engine
            .choice
            .as_ref()
            .expect("generation potion should open a discover choice");
        assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
        assert_eq!(choice.aux_count, 1, "{potion} should default to one chosen copy");
        assert_eq!(choice.options.len(), 3, "{potion} should offer three choices");
        for option in &choice.options {
            let generated = match option {
                ChoiceOption::GeneratedCard(card) => *card,
                other => panic!("{potion} should offer generated-card options, got {other:?}"),
            };
            if let Some(card_type) = expected_type {
                assert_eq!(
                    engine.card_registry.card_def_by_id(generated.def_id).card_type,
                    card_type,
                    "{potion} should offer a {card_type:?} card via the action path"
                );
            }
            if expect_colorless {
                let generated_name = engine.card_registry.card_name(generated.def_id);
                assert!(
                    COLORLESS_CHOICES.contains(&generated_name),
                    "{potion} should offer a colorless card via the action path, got {generated_name}"
                );
            }
            assert_eq!(
                generated.cost, 0,
                "{potion} generated choices should be zero-cost this turn"
            );
        }

        engine.execute_action(&Action::Choose(0));

        assert_eq!(engine.state.hand.len(), 1, "{potion} should resolve one chosen card");
        assert!(
            engine.state.potions[slot].is_empty(),
            "{potion} should consume its slot"
        );
    }
}

#[test]
fn generation_potions_emit_runtime_manual_activation_records() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let cases = [
        ("AttackPotion", "AttackPotion"),
        ("SkillPotion", "SkillPotion"),
        ("PowerPotion", "PowerPotion"),
        ("ColorlessPotion", "ColorlessPotion"),
    ];

    for (potion_id, def_id) in cases {
        engine.state.hand.clear();
        engine.state.potions = vec![String::new(); 3];
        engine.state.potions[0] = potion_id.to_string();
        engine.clear_event_log();

        use_potion(&mut engine, 0, -1);

        assert!(engine.event_log.iter().any(|record| {
            record.event == crate::effects::trigger::Trigger::ManualActivation
                && record.def_id == Some(def_id)
        }));
        assert!(engine.event_log.iter().any(|record| {
            record.event == crate::effects::trigger::Trigger::OnPotionUsed
                && record.potion_slot == 0
        }));
    }
}

#[test]
fn distilled_chaos_moves_top_draw_cards_via_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Zap"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.init_defect_orbs(3);
    engine.state.hand.clear();
    engine.state.draw_pile = make_deck(&["Strike", "Defend", "Zap"]);
    engine.state.potions[0] = "DistilledChaos".to_string();

    use_potion(&mut engine, 0, -1);

    assert!(engine.state.hand.is_empty());
    assert_eq!(engine.state.draw_pile.len(), 0);
    assert_eq!(engine.state.player.block, 5);
    assert_eq!(engine.state.enemies[0].entity.hp, 34);
    assert_eq!(engine.state.orb_slots.occupied_count(), 1);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn distilled_chaos_preselects_random_targets_and_retries_after_shuffle() {
    // Source-derived (verify potion/DistilledChaosPotion): use() queues three
    // PlayTopCardActions with targets selected through cardRandomRng, while
    // PlayTopCardAction shuffles discard and retries when draw is empty.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/DistilledChaosPotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/MonsterGroup.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/PlayTopCardAction.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Strike", "Strike"]),
        vec![
            enemy_no_intent("JawWorm", 40, 40),
            enemy_no_intent("Cultist", 40, 40),
        ],
        3,
    ));
    engine.state.hand.clear();
    engine.state.draw_pile.clear();
    engine.state.discard_pile = make_deck(&["Strike", "Strike", "Strike"]);
    engine.state.potions[0] = "DistilledChaos".to_string();

    let counter_before = engine.card_random_rng.counter;
    let mut oracle = engine.card_random_rng.clone();
    let mut expected_hits = [0; 2];
    for _ in 0..3 {
        expected_hits[oracle.random_range(0, 1) as usize] += 1;
    }

    use_potion(&mut engine, 0, -1);

    assert_eq!(engine.card_random_rng.counter, counter_before + 3);
    assert_eq!(engine.card_random_rng.counter, oracle.counter);
    assert_eq!(engine.state.enemies[0].entity.hp, 40 - expected_hits[0] * 6);
    assert_eq!(engine.state.enemies[1].entity.hp, 40 - expected_hits[1] * 6);
    assert!(engine.state.draw_pile.is_empty());
    assert_eq!(engine.state.discard_pile.len(), 3);
    assert_eq!(engine.state.energy, 3, "autoplayed cards are free");
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn liquid_memories_returns_discard_cards_via_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.discard_pile = make_deck(&["Strike", "Bash", "Shrug It Off"]);
    engine.state.potions[0] = "LiquidMemories".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("Liquid Memories should open a discard choice");
    assert_eq!(choice.reason, ChoiceReason::ReturnFromDiscard);
    assert_eq!(choice.options.len(), 3);
    engine.execute_action(&Action::Choose(1));

    assert_eq!(hand_names(&engine), vec!["Bash"]);
    assert_eq!(engine.state.hand[0].cost, 0);
    assert_eq!(engine.state.discard_pile.len(), 2);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn liquid_memories_auto_return_preserves_discard_order_and_hand_limit() {
    // Source-derived (verify potion/LiquidMemories): Sacred Bark doubles the
    // constant-one potency. When the discard has no more than that many cards,
    // BetterDiscardPileToHandAction iterates CardGroup.group in stored order;
    // each moved card costs zero this turn and the ten-card hand cap leaves
    // later cards in the discard pile.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/LiquidMemories.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/BetterDiscardPileToHandAction.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&[
        "Defend", "Defend", "Defend", "Defend", "Defend", "Defend", "Defend", "Defend",
        "Defend",
    ]);
    engine.state.discard_pile = make_deck(&["Strike", "Bash"]);
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "LiquidMemories".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 10);
    assert_eq!(hand_names(&engine).last().copied(), Some("Strike"));
    assert_eq!(engine.state.hand.last().expect("returned card").cost, 0);
    assert_eq!(engine.state.discard_pile.len(), 1);
    assert_eq!(
        engine
            .card_registry
            .card_name(engine.state.discard_pile[0].def_id),
        "Bash"
    );
}

#[test]
fn entropic_brew_rolls_for_every_slot_and_refills_its_consumed_slot() {
    // Source-derived (verify potion/EntropicBrew): combat use rolls one limited
    // random potion per potion slot before queued ObtainPotionActions resolve.
    // The UI destroys the brew first, so its own slot is available to refill.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/EntropicBrew.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/ui/panels/PotionPopUp.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Strike", "Defend", "Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.potions = vec![String::new(); 3];
    engine.state.potions[0] = "EntropicBrew".to_string();
    engine.state.potions[2] = "Fire Potion".to_string();
    let potion_rng_before = engine.potion_rng.counter;

    use_potion(&mut engine, 0, -1);

    assert!(engine.state.potions.iter().all(|potion| !potion.is_empty()));
    assert_eq!(engine.state.potions[2], "Fire Potion");
    assert!(engine.state.potions[..2].iter().all(|potion| {
        crate::potions::defs::entropic_brew::is_watcher_limited_potion(potion)
    }));
    assert!(engine.potion_rng.counter >= potion_rng_before + 9);
}

#[test]
fn entropic_brew_rolls_but_sozu_blocks_every_obtain_action() {
    // ObtainPotionAction checks Sozu only when each already-rolled action
    // resolves, so the brew is consumed and no replacement is obtained.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ObtainPotionAction.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.relics.push("Sozu".to_string());
    engine.state.potions = vec!["EntropicBrew".to_string(), String::new(), String::new()];
    let potion_rng_before = engine.potion_rng.counter;

    use_potion(&mut engine, 0, -1);

    assert!(engine.state.potions.iter().all(|potion| potion.is_empty()));
    assert!(engine.potion_rng.counter >= potion_rng_before + 9);
}

#[test]
fn elixir_uses_runtime_action_path_and_exhausts_hand() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Strike", "Defend", "Bash"]);
    engine.state.exhaust_pile.clear();
    engine.state.potions[0] = "Elixir".to_string();
    engine.clear_event_log();

    use_potion(&mut engine, 0, -1);

    assert!(engine.state.hand.is_empty());
    assert_eq!(engine.state.exhaust_pile.len(), 3);
    assert!(engine.state.potions[0].is_empty());
    assert!(engine.event_log.iter().any(|record| {
        record.event == crate::effects::trigger::Trigger::ManualActivation
            && record.def_id == Some("Elixir")
    }));
}

#[test]
fn blessing_of_the_forge_upgrades_hand_via_runtime_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Strike", "Defend"]);
    engine.state.potions[0] = "BlessingOfTheForge".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(hand_names(&engine), vec!["Strike+", "Defend+"]);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn blessing_of_the_forge_upgrades_each_eligible_hand_card_once() {
    // Source-derived (verify potion/BlessingOfTheForge): ArmamentsAction(true)
    // iterates the current hand and calls upgrade only when canUpgrade is true.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.hand = make_deck(&["Strike", "Defend+", "Dazed", "Miracle"]);
    engine.state.potions[0] = "BlessingOfTheForge".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(hand_names(&engine), vec!["Strike+", "Defend+", "Dazed", "Miracle+"]);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn bottled_miracle_and_cunning_potion_use_runtime_hooks() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.potions[0] = "BottledMiracle".to_string();
    engine.state.potions[1] = "CunningPotion".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(hand_names(&engine), vec!["Miracle", "Miracle"]);

    use_potion(&mut engine, 1, -1);
    assert_eq!(
        hand_names(&engine),
        vec!["Miracle", "Miracle", "Shiv", "Shiv", "Shiv"]
    );
}

#[test]
fn bottled_miracle_and_cunning_potion_respect_sacred_bark_and_hand_limit_via_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.hand = make_deck(&[
        "Strike",
        "Defend",
        "Bash",
        "Zap",
        "Dualcast",
        "Inflame",
        "Shrug It Off",
        "Defend",
    ]);
    engine.state.potions[0] = "BottledMiracle".to_string();
    engine.state.potions[1] = "CunningPotion".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.hand.len(), 10);
    assert_eq!(
        hand_names(&engine)[8..],
        ["Miracle", "Miracle"]
    );

    use_potion(&mut engine, 1, -1);
    assert_eq!(engine.state.hand.len(), 10);
    let shiv_count = hand_names(&engine)
        .into_iter()
        .filter(|name| *name == "Shiv")
        .count();
    assert_eq!(shiv_count, 0, "full hand should block extra Shiv generation");
}

#[test]
fn gamblers_brew_selects_any_subset_then_discards_and_redraws_that_count() {
    // Source-derived (verify potion/GamblersBrew): use queues a
    // GamblingChipAction with notchip=true. That action opens an any-number
    // hand selection, manually discards only those cards, and draws exactly
    // the number selected.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/GamblersBrew.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/GamblingChipAction.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame", "Zap", "Dualcast"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Strike", "Defend", "Bash"]);
    engine.state.draw_pile = make_deck(&["Shrug It Off", "Inflame", "Zap", "Dualcast"]);
    engine.state.discard_pile.clear();
    engine.state.potions[0] = "GamblersBrew".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("Gambler's Brew choice");
    assert_eq!(choice.reason, ChoiceReason::DiscardFromHand);
    assert_eq!(choice.min_picks, 0);
    assert_eq!(choice.max_picks, 3);
    assert!(engine.state.potions[0].is_empty());

    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::Choose(2));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(hand_names(&engine), vec!["Defend", "Dualcast", "Zap"]);
    assert_eq!(engine.state.discard_pile.len(), 2);
    assert_eq!(engine.state.player.status(sid::GAMBLING_CHIP_ACTIVE), 0);
}

#[test]
fn gamblers_brew_allows_zero_discards_and_consumes_with_an_empty_hand() {
    // GamblingChipAction's `anyNumber=true` permits confirming zero cards;
    // GamblersBrew.use queues no action at all when the hand is empty.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/GamblersBrew.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/GamblingChipAction.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Strike", "Defend"]);
    engine.state.draw_pile = make_deck(&["Bash"]);
    engine.state.potions[0] = "GamblersBrew".to_string();

    use_potion(&mut engine, 0, -1);
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(hand_names(&engine), vec!["Strike", "Defend"]);
    assert_eq!(engine.state.draw_pile.len(), 1);
    assert!(engine.state.discard_pile.is_empty());

    engine.state.hand.clear();
    engine.state.potions[0] = "GamblersBrew".to_string();
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert!(engine.choice.is_none());
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn snecko_oil_draws_then_randomizes_hand_costs_without_confusion() {
    // Source-derived (verify potion/SneckoOil): getPotency is always five,
    // Sacred Bark doubles the draw, and RandomizeHandCostAction then rolls
    // cardRandomRng.random(3) once for each non-negative printed-cost card in
    // the whole hand. It changes combat cost rather than applying Confusion.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/SneckoOil.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/RandomizeHandCostAction.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Strike", "Whirlwind"]);
    engine.state.draw_pile = make_deck(&[
        "Defend", "Bash", "Inflame", "Zap", "Dualcast", "Shrug It Off", "Strike", "Defend",
    ]);
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "SneckoOil".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(engine.state.hand.len(), 10);
    assert!(engine.state.draw_pile.is_empty());
    assert_eq!(engine.state.player.status(sid::CONFUSION), 0);
    let mut oracle = crate::seed::StsRandom::new(TEST_SEED);
    for card in &engine.state.hand {
        let printed_cost = engine.card_registry.card_def_by_id(card.def_id).cost;
        if printed_cost < 0 {
            assert_eq!(engine.card_registry.card_name(card.def_id), "Whirlwind");
            continue;
        }
        let expected = oracle.random(3);
        let actual = if card.base_cost >= 0 {
            card.base_cost as i32
        } else {
            printed_cost
        };
        assert_eq!(actual, expected);
    }
    assert_eq!(engine.card_random_rng.counter, 9);
    assert!(engine.state.potions[0].is_empty());
    assert!(engine.event_log.iter().any(|record| {
        record.event == crate::effects::trigger::Trigger::ManualActivation
            && record.def_id == Some("SneckoOil")
    }));
}

#[test]
fn temporary_effect_potions_apply_statuses_through_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let cases: [(&str, crate::ids::StatusId, i32, Option<crate::ids::StatusId>); 4] = [
        ("DuplicationPotion", sid::DUPLICATION, 1, None),
        ("GhostInAJar", sid::INTANGIBLE, 1, None),
        ("SteroidPotion", sid::STRENGTH, 5, Some(sid::LOSE_STRENGTH)),
        ("SpeedPotion", sid::DEXTERITY, 5, Some(sid::LOSE_DEXTERITY)),
    ];

    for (potion_id, primary_status, primary_amount, trailing_status) in cases {
        engine.state.player.set_status(sid::DUPLICATION, 0);
        engine.state.player.set_status(sid::INTANGIBLE, 0);
        engine.state.player.set_status(sid::STRENGTH, 0);
        engine.state.player.set_status(sid::DEXTERITY, 0);
        engine.state.player.set_status(sid::LOSE_STRENGTH, 0);
        engine.state.player.set_status(sid::LOSE_DEXTERITY, 0);
        engine.state.potions = vec![String::new(); 3];
        engine.state.potions[0] = potion_id.to_string();
        engine.clear_event_log();

        use_potion(&mut engine, 0, -1);

        assert_eq!(engine.state.player.status(primary_status), primary_amount, "{potion_id} primary status");
        if let Some(trailing_status) = trailing_status {
            assert_eq!(engine.state.player.status(trailing_status), primary_amount, "{potion_id} trailing status");
        }
        assert!(engine.state.potions[0].is_empty(), "{potion_id} should consume its slot");
        assert!(engine.event_log.iter().any(|record| {
            record.event == crate::effects::trigger::Trigger::ManualActivation
        }));
    }
}

#[test]
fn speed_potion_keeps_five_potency_and_artifact_can_block_only_dex_loss() {
    // Source-derived (verify potion/SpeedPotion): getPotency always returns
    // five. Java applies DexterityPower before debuff-typed LoseDexterityPower,
    // so Sacred Bark doubles both while Artifact consumes itself to block only
    // the delayed loss.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/SpeedPotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/LoseDexterityPower.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Defend"]);
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "SpeedPotion".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.player.status(sid::DEXTERITY), 10);
    assert_eq!(engine.state.player.status(sid::LOSE_DEXTERITY), 10);
    assert!(crate::tests::support::play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.player.block, 15);
    engine.execute_action(&Action::EndTurn);
    assert_eq!(engine.state.player.status(sid::DEXTERITY), 0);
    assert_eq!(engine.state.player.status(sid::LOSE_DEXTERITY), 0);

    let mut artifact = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    artifact.state.player.set_status(sid::ARTIFACT, 1);
    artifact.state.potions[0] = "SpeedPotion".to_string();
    use_potion(&mut artifact, 0, -1);
    assert_eq!(artifact.state.player.status(sid::DEXTERITY), 5);
    assert_eq!(artifact.state.player.status(sid::LOSE_DEXTERITY), 0);
    assert_eq!(artifact.state.player.status(sid::ARTIFACT), 0);
    artifact.execute_action(&Action::EndTurn);
    assert_eq!(artifact.state.player.status(sid::DEXTERITY), 5);
}

#[test]
fn steroid_potion_keeps_five_potency_and_artifact_can_block_only_strength_loss() {
    // Source-derived (verify potion/SteroidPotion): getPotency always returns
    // five. Java applies StrengthPower before debuff-typed LoseStrengthPower,
    // so Sacred Bark doubles both while Artifact blocks only the delayed loss.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/SteroidPotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/LoseStrengthPower.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Strike"]);
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "SteroidPotion".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.player.status(sid::STRENGTH), 10);
    assert_eq!(engine.state.player.status(sid::LOSE_STRENGTH), 10);
    engine.execute_action(&Action::PlayCard {
        card_idx: 0,
        target_idx: 0,
    });
    assert_eq!(engine.state.enemies[0].entity.hp, 24);
    engine.execute_action(&Action::EndTurn);
    assert_eq!(engine.state.player.status(sid::STRENGTH), 0);
    assert_eq!(engine.state.player.status(sid::LOSE_STRENGTH), 0);

    let mut artifact = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    artifact.state.player.set_status(sid::ARTIFACT, 1);
    artifact.state.potions[0] = "SteroidPotion".to_string();
    use_potion(&mut artifact, 0, -1);
    assert_eq!(artifact.state.player.status(sid::STRENGTH), 5);
    assert_eq!(artifact.state.player.status(sid::LOSE_STRENGTH), 0);
    assert_eq!(artifact.state.player.status(sid::ARTIFACT), 0);
    artifact.execute_action(&Action::EndTurn);
    assert_eq!(artifact.state.player.status(sid::STRENGTH), 5);
}

#[test]
fn fairy_potion_is_passive_and_revives_through_java_healing_rules() {
    // Source-derived (verify potion/FairyPotion): canUse is always false. On
    // lethal damage, potency is 30% at every ascension, Sacred Bark doubles it,
    // the amount is clamped to at least one, and player.heal applies Magic
    // Flower before the potion slot is destroyed.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FairyPotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
    let mut lethal_enemy = enemy_no_intent("JawWorm", 40, 40);
    lethal_enemy.set_move(1, 200, 1, 0);
    let mut state = combat_state_with(make_deck(&["Strike"]), vec![lethal_enemy], 3);
    state.relics.push("SacredBark".to_string());
    state.relics.push("Magic Flower".to_string());
    state.potions[0] = "FairyPotion".to_string();
    let mut engine = engine_with_state(state);

    assert!(!engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::UsePotion { potion_idx: 0, .. })
    }));

    engine.execute_action(&Action::EndTurn);

    // 80 * 60% Sacred Bark = 48; Magic Flower raises that heal to 72.
    assert_eq!(engine.state.player.hp, 72);
    assert!(!engine.state.combat_over);
    assert!(engine.state.potions[0].is_empty());

    let mut tiny_enemy = enemy_no_intent("JawWorm", 40, 40);
    tiny_enemy.set_move(1, 2, 1, 0);
    let mut tiny_state = combat_state_with(make_deck(&["Strike"]), vec![tiny_enemy], 3);
    tiny_state.player.hp = 1;
    tiny_state.player.max_hp = 1;
    tiny_state.potions[0] = "FairyPotion".to_string();
    let mut tiny = engine_with_state(tiny_state);
    tiny.execute_action(&Action::EndTurn);
    assert_eq!(tiny.state.player.hp, 1, "Fairy heal is clamped to one");
}

#[test]
fn stance_potion_opens_choose_one_and_sets_stance_via_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Strike", "Defend", "Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.stance = Stance::Calm;
    engine.state.energy = 0;
    engine.state.potions[0] = "StancePotion".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.phase, crate::engine::CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("Stance Potion should open a choice");
    let labels: Vec<&str> = choice
        .options
        .iter()
        .filter_map(|opt| match opt {
            crate::engine::ChoiceOption::Named(label) => Some(*label),
            _ => None,
        })
        .collect();
    assert_eq!(labels, vec!["Wrath", "Calm"]);
    engine.execute_action(&Action::Choose(0));
    assert_eq!(engine.state.stance, Stance::Wrath);
    assert_eq!(engine.state.energy, 2);
    assert!(engine.state.potions[0].is_empty());

    engine.state.potions[0] = "StancePotion".to_string();
    use_potion(&mut engine, 0, -1);
    engine.execute_action(&Action::Choose(1));
    assert_eq!(engine.state.stance, Stance::Calm);
}

#[test]
fn ambrosia_essence_of_darkness_and_capacity_use_runtime_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.init_defect_orbs(2);
    engine.state.stance = Stance::Neutral;

    engine.state.potions[0] = "Ambrosia".to_string();
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.stance, Stance::Divinity);

    engine.state.potions[0] = "EssenceOfDarkness".to_string();
    use_potion(&mut engine, 0, -1);
    assert!(engine
        .state
        .orb_slots
        .slots
        .iter()
        .all(|orb| orb.orb_type == crate::orbs::OrbType::Dark));

    engine.state.potions[0] = "PotionOfCapacity".to_string();
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.player.status(sid::ORB_SLOTS), 2);
}

#[test]
fn smoke_bomb_uses_runtime_action_path_and_consumes_its_slot() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.potions[0] = "Smoke Bomb".to_string();
    engine.clear_event_log();

    use_potion(&mut engine, 0, -1);

    assert!(engine.state.combat_over);
    assert!(engine.state.potions[0].is_empty());
    assert!(engine.event_log.iter().any(|record| {
        record.event == crate::effects::trigger::Trigger::ManualActivation
            && record.def_id == Some("SmokeBomb")
    }));
}
