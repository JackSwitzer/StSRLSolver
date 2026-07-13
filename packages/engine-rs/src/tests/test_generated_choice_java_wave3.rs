#![cfg(test)]

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Chrysalis.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Metamorphosis.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Transmutation.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/TransmutationAction.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/AttackPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/SkillPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/PowerPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/ColorlessPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DiscoveryAction.java

use crate::actions::Action;
use crate::cards::CardType;
use crate::engine::{ChoiceOption, ChoiceReason, CombatPhase};
use crate::status_ids::sid;
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, make_deck, play_self};

const COLORLESS_CHOICES: &[&str] = &[
    "Apotheosis", "Bandage Up", "Bite", "Blind", "Chrysalis", "Dark Shackles", "Deep Breath",
    "Defend", "Discovery", "Dramatic Entrance", "Enlightenment", "Finesse", "Flash of Steel",
    "Forethought", "Ghostly", "Good Instincts", "HandOfGreed", "Impatience", "J.A.X.",
    "Jack Of All Trades", "Madness", "Magnetism", "Master of Strategy", "Mayhem",
    "Metamorphosis", "Mind Blast", "Panacea", "Panache", "PanicButton", "Purity",
    "RitualDagger", "Sadistic Nature", "Secret Technique", "Secret Weapon", "Strike",
    "Swift Strike", "The Bomb", "Thinking Ahead", "Transmutation", "Trip", "Violence",
];

const COLORLESS_POTION_CHOICES: &[&str] = &[
    "Apotheosis",
    "Blind",
    "Chrysalis",
    "Dark Shackles",
    "Deep Breath",
    "Discovery",
    "Dramatic Entrance",
    "Enlightenment",
    "Finesse",
    "Flash of Steel",
    "Forethought",
    "Good Instincts",
    "HandOfGreed",
    "Impatience",
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
    "Sadistic Nature",
    "Secret Technique",
    "Secret Weapon",
    "Swift Strike",
    "The Bomb",
    "Thinking Ahead",
    "Transmutation",
    "Trip",
    "Violence",
];

const WATCHER_SKILL_POOL_IN_JAVA_ORDER: &[&str] = &[
    "Prostrate", "Evaluate", "PathToVictory", "EmptyBody", "ClearTheMind", "Crescendo",
    "ThirdEye", "Protect", "Halt", "Pray", "EmptyMind", "Worship", "Swivel",
    "Perseverance", "Meditate", "WaveOfTheHand", "DeceiveReality", "InnerPeace", "Collect",
    "WreathOfFlame", "ForeignInfluence", "Indignation", "Sanctity", "Vengeance", "Judgement",
    "ConjureBlade", "Blasphemy", "Scrawl", "Vault", "Alpha", "Omniscience", "SpiritShield",
    "DeusExMachina",
];

#[test]
fn watcher_skill_generation_pool_matches_java_source_pool_order() {
    // CardLibrary's HashMap iteration builds rarity pools; initializeCardPools
    // then reverses each rarity through addToBottom. Chrysalis concatenates
    // common, uncommon, and rare Skills and excludes HEALING-tagged Wish.
    // Java: helpers/CardLibrary.java and dungeons/AbstractDungeon.java.
    let engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    assert_eq!(
        super::generated_card_pool(&engine, super::GeneratedCardPool::Skill),
        WATCHER_SKILL_POOL_IN_JAVA_ORDER
    );
}

const WATCHER_ATTACK_CHOICES: &[&str] = &[
    "BowlingBash",
    "Brilliance",
    "CarveReality",
    "Conclude",
    "Consecrate",
    "CrushJoints",
    "CutThroughFate",
    "EmptyFist",
    "FearNoEvil",
    "FlurryOfBlows",
    "FlyingSleeves",
    "FollowUp",
    "JustLucky",
    "Ragnarok",
    "ReachHeaven",
    "SandsOfTime",
    "SashWhip",
    "SignatureMove",
    "TalkToTheHand",
    "Tantrum",
    "Wallop",
    "Weave",
    "WheelKick",
    "WindmillStrike",
];

fn use_potion(engine: &mut crate::engine::CombatEngine, potion_idx: usize, target_idx: i32) {
    engine.execute_action(&Action::UsePotion {
        potion_idx,
        target_idx,
    });
}

#[test]
fn chrysalis_generates_zero_cost_skills_into_draw_pile() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Chrysalis"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Chrysalis"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Chrysalis"));
    assert_eq!(engine.state.draw_pile.len(), 3);
    for card in &engine.state.draw_pile {
        let def = engine.card_registry.card_def_by_id(card.def_id);
        assert_eq!(def.card_type, CardType::Skill);
        assert!(card.cost <= 0, "generated Chrysalis cards should be free this turn");
    }
}

#[test]
fn metamorphosis_generates_zero_cost_attacks_into_draw_pile() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Metamorphosis"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Metamorphosis"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Metamorphosis"));
    assert_eq!(engine.state.draw_pile.len(), 3);
    for card in &engine.state.draw_pile {
        let def = engine.card_registry.card_def_by_id(card.def_id);
        assert_eq!(def.card_type, CardType::Attack);
        assert!(card.cost <= 0, "generated Metamorphosis cards should be free this turn");
    }
}

#[test]
fn transmutation_generates_x_zero_cost_colorless_cards_to_hand() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Transmutation"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Transmutation"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.state.energy = 3;

    assert!(play_self(&mut engine, "Transmutation"));
    assert_eq!(engine.state.hand.len(), 3);
    for card in &engine.state.hand {
        let name = engine.card_registry.card_name(card.def_id);
        assert!(COLORLESS_CHOICES.contains(&name));
        assert_eq!(card.cost, 0);
    }
}

#[test]
fn transmutation_plus_upgrades_generated_cards() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Transmutation+"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        2,
    ));
    engine.state.hand = make_deck(&["Transmutation+"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.state.energy = 2;

    assert!(play_self(&mut engine, "Transmutation+"));
    assert_eq!(engine.state.hand.len(), 2);
    assert!(engine.state.hand.iter().all(|card| card.is_upgraded()));
    assert!(engine.state.hand.iter().all(|card| card.cost == 0));
}

#[test]
fn discovery_potions_open_java_style_choice_and_track_copy_count() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
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
        engine.state.hand.clear();
        engine.state.potions = vec![String::new(); 3];
        engine.state.potions[0] = potion.to_string();

        use_potion(&mut engine, 0, -1);

        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        let choice = engine.choice.as_ref().expect("potion should open a discovery choice");
        assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
        assert_eq!(choice.aux_count, 1, "{potion} should default to one generated copy");
        assert_eq!(choice.options.len(), 3);
        for option in &choice.options {
            let ChoiceOption::GeneratedCard(card) = option else {
                panic!("{potion} should offer generated cards");
            };
            if let Some(card_type) = expected_type {
                assert_eq!(engine.card_registry.card_def_by_id(card.def_id).card_type, card_type);
            }
            if expect_colorless {
                let name = engine.card_registry.card_name(card.def_id);
                assert!(COLORLESS_CHOICES.contains(&name));
            }
            assert_eq!(card.cost, 0);
        }

        engine.execute_action(&Action::Choose(0));
        assert_eq!(engine.state.hand.len(), 1);
        assert!(engine.state.potions[0].is_empty());
    }
}

#[test]
fn attack_potion_uses_watcher_pool_and_card_random_rng_tick_for_tick() {
    // Source-derived (verify potion/AttackPotion): DiscoveryAction requests
    // three unique ATTACK cards from AbstractDungeon's Watcher source pools.
    // returnTrulyRandomCardInCombat(type) consumes cardRandomRng once per
    // attempt and excludes BASIC, SPECIAL, and HEALING cards.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.potions[0] = "AttackPotion".to_string();
    let card_random_before = engine.card_random_rng.counter;
    let general_before = engine.rng.counter;
    let attack_pool = super::generated_card_pool(&engine, super::GeneratedCardPool::Attack);
    let mut card_random_oracle = engine.card_random_rng.clone();
    let mut oracle_seen = std::collections::HashSet::new();
    while oracle_seen.len() < 3 {
        let idx = card_random_oracle.random((attack_pool.len() - 1) as i32) as usize;
        oracle_seen.insert(attack_pool[idx]);
    }

    use_potion(&mut engine, 0, -1);

    let choice = engine.choice.as_ref().expect("Attack Potion choice");
    let names: Vec<_> = choice
        .options
        .iter()
        .map(|option| match option {
            ChoiceOption::GeneratedCard(card) => engine.card_registry.card_name(card.def_id),
            _ => panic!("Attack Potion must generate card choices"),
        })
        .collect();
    assert_eq!(names.len(), 3);
    assert!(names.iter().all(|name| WATCHER_ATTACK_CHOICES.contains(name)));
    assert_eq!(card_random_oracle.counter, card_random_before + 4);
    assert_eq!(engine.card_random_rng.counter, card_random_oracle.counter);
    assert_eq!(engine.rng.counter, general_before);
}

#[test]
fn power_potion_uses_watcher_power_pool_rng_and_bark_copy_count() {
    // Source-derived (verify potion/PowerPotion): DiscoveryAction requests
    // three unique POWER cards, consuming cardRandomRng once per attempt.
    // Potency is constant one; Sacred Bark doubles the selected copies, both
    // cost zero this turn, and Master Reality upgrades only those copies.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/PowerPotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DiscoveryAction.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&[
        "Defend", "Defend", "Defend", "Defend", "Defend", "Defend", "Defend", "Defend",
        "Defend",
    ]);
    engine.state.relics.push("SacredBark".to_string());
    engine.state.player.set_status(sid::MASTER_REALITY, 1);
    engine.state.potions[0] = "PowerPotion".to_string();
    let pool = super::generated_card_pool(&engine, super::GeneratedCardPool::Power);
    let mut oracle = engine.card_random_rng.clone();
    let mut seen = std::collections::HashSet::new();
    while seen.len() < 3 {
        let idx = oracle.random((pool.len() - 1) as i32) as usize;
        seen.insert(pool[idx]);
    }

    use_potion(&mut engine, 0, -1);

    let choice = engine.choice.as_ref().expect("Power Potion choice");
    assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
    assert_eq!(choice.options.len(), 3);
    assert_eq!(choice.aux_count, 2);
    assert_eq!(engine.card_random_rng.counter, oracle.counter);
    for option in &choice.options {
        let ChoiceOption::GeneratedCard(card) = option else {
            panic!("Power Potion should offer generated cards");
        };
        assert_eq!(
            engine.card_registry.card_def_by_id(card.def_id).card_type,
            CardType::Power
        );
        assert!(!card.is_upgraded());
        assert_eq!(card.cost, 0);
    }

    engine.execute_action(&Action::Choose(0));
    assert_eq!(engine.state.hand.len(), 10);
    assert_eq!(engine.state.discard_pile.len(), 1);
    let hand_copy = engine.state.hand.last().expect("first selected copy");
    let discard_copy = engine.state.discard_pile.last().expect("second selected copy");
    assert!(hand_copy.is_upgraded());
    assert!(discard_copy.is_upgraded());
    assert_eq!(hand_copy.cost, 0);
    assert_eq!(discard_copy.cost, 0);
}

#[test]
fn skill_potion_uses_watcher_skill_pool_rng_and_bark_copy_count() {
    // Source-derived (verify potion/SkillPotion): DiscoveryAction requests
    // three unique SKILL cards, consuming cardRandomRng once per attempt.
    // Potency is constant one; Sacred Bark doubles the selected zero-cost
    // copies and Master Reality upgrades only those copies.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/SkillPotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DiscoveryAction.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&[
        "Defend", "Defend", "Defend", "Defend", "Defend", "Defend", "Defend", "Defend",
        "Defend",
    ]);
    engine.state.relics.push("SacredBark".to_string());
    engine.state.player.set_status(sid::MASTER_REALITY, 1);
    engine.state.potions[0] = "SkillPotion".to_string();
    let pool = super::generated_card_pool(&engine, super::GeneratedCardPool::Skill);
    let mut oracle = engine.card_random_rng.clone();
    let mut seen = std::collections::HashSet::new();
    while seen.len() < 3 {
        let idx = oracle.random((pool.len() - 1) as i32) as usize;
        seen.insert(pool[idx]);
    }

    use_potion(&mut engine, 0, -1);

    let choice = engine.choice.as_ref().expect("Skill Potion choice");
    assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
    assert_eq!(choice.options.len(), 3);
    assert_eq!(choice.aux_count, 2);
    assert_eq!(engine.card_random_rng.counter, oracle.counter);
    for option in &choice.options {
        let ChoiceOption::GeneratedCard(card) = option else {
            panic!("Skill Potion should offer generated cards");
        };
        assert_eq!(
            engine.card_registry.card_def_by_id(card.def_id).card_type,
            CardType::Skill
        );
        assert!(!card.is_upgraded());
        assert_eq!(card.cost, 0);
    }

    engine.execute_action(&Action::Choose(0));
    assert_eq!(engine.state.hand.len(), 10);
    assert_eq!(engine.state.discard_pile.len(), 1);
    let hand_copy = engine.state.hand.last().expect("first selected copy");
    let discard_copy = engine.state.discard_pile.last().expect("second selected copy");
    assert!(hand_copy.is_upgraded());
    assert!(discard_copy.is_upgraded());
    assert_eq!(hand_copy.cost, 0);
    assert_eq!(discard_copy.cost, 0);
}

#[test]
fn colorless_potion_uses_normal_pool_base_previews_and_exact_card_rng() {
    // Source-derived (verify potion/ColorlessPotion): DiscoveryAction(true,
    // potency) chooses three unique cards from srcColorlessCardPool, excluding
    // HEALING. Master Reality upgrades only the selected copies; Sacred Bark
    // doubles the one-copy potency.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.relics.push("SacredBark".to_string());
    engine.state.player.set_status(sid::MASTER_REALITY, 1);
    engine.state.potions[0] = "ColorlessPotion".to_string();
    let pool = super::generated_card_pool(&engine, super::GeneratedCardPool::Colorless);
    let mut card_random_oracle = engine.card_random_rng.clone();
    let mut oracle_seen = std::collections::HashSet::new();
    while oracle_seen.len() < 3 {
        let idx = card_random_oracle.random((pool.len() - 1) as i32) as usize;
        oracle_seen.insert(pool[idx]);
    }
    let general_before = engine.rng.counter;

    use_potion(&mut engine, 0, -1);

    let choice = engine.choice.as_ref().expect("Colorless Potion choice");
    assert_eq!(choice.aux_count, 2);
    let names: Vec<_> = choice
        .options
        .iter()
        .map(|option| match option {
            ChoiceOption::GeneratedCard(card) => {
                assert!(!card.is_upgraded());
                engine.card_registry.card_name(card.def_id)
            }
            _ => panic!("Colorless Potion must generate card choices"),
        })
        .collect();
    assert!(names.iter().all(|name| COLORLESS_POTION_CHOICES.contains(name)));
    assert_eq!(engine.card_random_rng.counter, card_random_oracle.counter);
    assert_eq!(engine.rng.counter, general_before);

    engine.execute_action(&Action::Choose(0));
    assert_eq!(engine.state.hand.len(), 2);
    assert!(engine.state.hand.iter().all(|card| card.is_upgraded()));
    assert!(engine.state.hand.iter().all(|card| card.cost == 0));
}

#[test]
fn master_reality_upgrades_resolved_generated_discovery_card() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.player.set_status(sid::MASTER_REALITY, 1);
    engine.state.potions[0] = "AttackPotion".to_string();

    use_potion(&mut engine, 0, -1);

    let choice = engine.choice.as_ref().expect("Attack Potion should open a choice");
    assert_eq!(choice.reason, ChoiceReason::DiscoverCard);

    engine.execute_action(&Action::Choose(0));

    assert_eq!(engine.state.hand.len(), 1);
    assert!(
        engine.state.hand[0].is_upgraded(),
        "Master Reality should upgrade the resolved generated card copy"
    );
}

#[test]
fn sacred_bark_discovery_choice_needs_copy_count_resolution() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "SkillPotion".to_string();

    use_potion(&mut engine, 0, -1);

    let choice = engine.choice.as_ref().expect("Skill Potion should open a choice");
    assert_eq!(choice.aux_count, 2);
    engine.execute_action(&Action::Choose(0));
    assert_eq!(engine.state.hand.len(), 2);
    assert!(engine.state.hand.iter().all(|card| card.cost == 0));
}
