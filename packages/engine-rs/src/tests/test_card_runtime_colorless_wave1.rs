#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/BandageUp.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Bite.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Blind.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/DarkShackles.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/DeepBreath.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/DramaticEntrance.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/GoodInstincts.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/FlashOfSteel.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/JAX.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Magnetism.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Mayhem.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Panache.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Panacea.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/PanicButton.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/powers/NoBlockPower.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/SadisticNature.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/SwiftStrike.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/TheBomb.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/powers/TheBombPower.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{
    AmountSource as A, Effect as E, SimpleEffect as SE, Target as T,
};
use crate::status_ids::sid;
use crate::tests::support::*;

#[test]
fn the_bomb_keeps_independent_three_turn_countdowns_and_damage() {
    // TheBomb.java installs TheBombPower(3, 40/50). TheBombPower gives every
    // application a unique ID, decrements at player turn end, and at amount 1
    // deals source-less THORNS damage to all enemies after reducing itself.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/TheBomb.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/TheBombPower.java
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 200, 200),
            enemy_no_intent("Cultist", 200, 200),
        ],
        2,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["The Bomb"]);

    assert!(play_self(&mut engine, "The Bomb"));
    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.player.status(sid::THE_BOMB), 40);
    assert_eq!(engine.state.player.status(sid::THE_BOMB_TURNS), 3);

    end_turn(&mut engine);
    assert_eq!(engine.state.player.status(sid::THE_BOMB_TURNS), 2);
    assert_eq!(engine.state.enemies[0].entity.hp, 200);
    assert_eq!(engine.state.enemies[1].entity.hp, 200);

    ensure_in_hand(&mut engine, "The Bomb+");
    assert!(play_self(&mut engine, "The Bomb+"));
    assert_eq!(engine.state.player.status(sid::THE_BOMB), 90);
    assert_eq!(engine.state.player.status(sid::THE_BOMB_TURNS), 2);

    end_turn(&mut engine);
    assert_eq!(engine.state.player.status(sid::THE_BOMB), 90);
    assert_eq!(engine.state.player.status(sid::THE_BOMB_TURNS), 1);
    assert_eq!(engine.state.enemies[0].entity.hp, 200);
    assert_eq!(engine.state.enemies[1].entity.hp, 200);

    end_turn(&mut engine);
    assert_eq!(engine.state.player.status(sid::THE_BOMB), 50);
    assert_eq!(engine.state.player.status(sid::THE_BOMB_TURNS), 1);
    assert_eq!(engine.state.enemies[0].entity.hp, 160);
    assert_eq!(engine.state.enemies[1].entity.hp, 160);

    end_turn(&mut engine);
    assert_eq!(engine.state.player.status(sid::THE_BOMB), 0);
    assert_eq!(engine.state.player.status(sid::THE_BOMB_TURNS), 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 110);
    assert_eq!(engine.state.enemies[1].entity.hp, 110);
}

#[test]
fn thinking_ahead_draws_before_put_on_deck_and_auto_moves_a_singleton() {
    // ThinkingAhead.java queues DrawCardAction(2) before PutOnDeckAction(1,
    // false), and upgrade only removes Exhaust. PutOnDeckAction auto-moves a
    // singleton through getRandomCard, consuming one cardRandomRng tick.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/ThinkingAhead.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/PutOnDeckAction.java
    for (card_id, should_exhaust) in [("Thinking Ahead", true), ("Thinking Ahead+", false)] {
        let mut singleton =
            engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 0);
        force_player_turn(&mut singleton);
        singleton.state.hand = make_deck(&[card_id]);
        singleton.state.draw_pile = make_deck(&["Strike"]);
        let card_random_before = singleton.card_random_rng.counter;

        assert!(play_self(&mut singleton, card_id));

        assert_eq!(singleton.phase, crate::engine::CombatPhase::PlayerTurn);
        assert!(singleton.choice.is_none());
        assert!(singleton.state.hand.is_empty());
        assert_eq!(singleton.state.draw_pile.len(), 1);
        assert_eq!(
            singleton
                .card_registry
                .card_name(singleton.state.draw_pile.last().unwrap().def_id),
            "Strike"
        );
        assert_eq!(singleton.card_random_rng.counter, card_random_before + 1);
        assert_eq!(
            exhaust_prefix_count(&singleton, "Thinking Ahead"),
            should_exhaust as usize
        );
        assert_eq!(
            discard_prefix_count(&singleton, "Thinking Ahead"),
            (!should_exhaust) as usize
        );
    }

    let mut selection =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 0);
    force_player_turn(&mut selection);
    selection.state.hand = make_deck(&["Thinking Ahead"]);
    selection.state.draw_pile = make_deck(&["Strike", "Defend"]);
    let card_random_before = selection.card_random_rng.counter;

    assert!(play_self(&mut selection, "Thinking Ahead"));

    assert_eq!(selection.phase, crate::engine::CombatPhase::AwaitingChoice);
    assert_eq!(selection.state.hand.len(), 2);
    assert_eq!(selection.card_random_rng.counter, card_random_before);
    selection.execute_action(&crate::actions::Action::Choose(0));
    assert_eq!(selection.phase, crate::engine::CombatPhase::PlayerTurn);
    assert_eq!(selection.state.hand.len(), 1);
    assert_eq!(selection.state.draw_pile.len(), 1);
}

#[test]
fn trip_base_targets_one_enemy_while_upgrade_applies_two_to_all() {
    // Trip.java applies magicNumber 2 Vulnerable to its selected enemy. The
    // upgrade changes target to ALL_ENEMY without changing amount or cost.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Trip.java
    let enemies = vec![
        enemy_no_intent("JawWorm", 40, 40),
        enemy_no_intent("Cultist", 35, 35),
    ];
    let mut base = engine_without_start(Vec::new(), enemies.clone(), 0);
    force_player_turn(&mut base);
    base.state.hand = make_deck(&["Trip"]);

    assert!(play_on_enemy(&mut base, "Trip", 1));
    assert_eq!(base.state.energy, 0);
    assert_eq!(base.state.enemies[0].entity.status(sid::VULNERABLE), 0);
    assert_eq!(base.state.enemies[1].entity.status(sid::VULNERABLE), 2);

    let mut upgraded = engine_without_start(Vec::new(), enemies, 0);
    force_player_turn(&mut upgraded);
    upgraded.state.hand = make_deck(&["Trip+"]);
    upgraded.state.enemies[0]
        .entity
        .set_status(sid::VULNERABLE, 1);
    upgraded.state.enemies[1]
        .entity
        .set_status(sid::ARTIFACT, 1);

    assert!(play_self(&mut upgraded, "Trip+"));
    assert_eq!(upgraded.state.energy, 0);
    assert_eq!(upgraded.state.enemies[0].entity.status(sid::VULNERABLE), 3);
    assert_eq!(upgraded.state.enemies[1].entity.status(sid::VULNERABLE), 0);
    assert_eq!(upgraded.state.enemies[1].entity.status(sid::ARTIFACT), 0);
}

#[test]
fn jax_source_loses_fixed_three_hp_then_gains_two_or_three_strength() {
    // JAX.java queues LoseHPAction(p, p, 3) before applying magicNumber
    // Strength (2, upgraded to 3). LoseHPAction uses HP_LOSS damage, so Buffer
    // can prevent the HP loss and its Rupture trigger while block is untouched.
    // Java: cards/colorless/JAX.java and actions/common/LoseHPAction.java.
    for (card_id, card_strength) in [("J.A.X.", 2), ("J.A.X.+", 3)] {
        let mut engine =
            engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 0);
        force_player_turn(&mut engine);
        engine.state.hand = make_deck(&[card_id]);
        engine.state.player.set_status(sid::RUPTURE, 2);
        let hp_before = engine.state.player.hp;

        assert!(play_self(&mut engine, card_id));

        assert_eq!(engine.state.player.hp, hp_before - 3);
        assert_eq!(engine.state.player.status(sid::STRENGTH), card_strength + 2);
        assert_eq!(engine.state.energy, 0);
        assert_eq!(discard_prefix_count(&engine, "J.A.X."), 1);

        let mut buffered =
            engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 0);
        force_player_turn(&mut buffered);
        buffered.state.hand = make_deck(&[card_id]);
        buffered.state.player.block = 9;
        buffered.state.player.set_status(sid::BUFFER, 1);
        buffered.state.player.set_status(sid::RUPTURE, 2);
        let hp_before = buffered.state.player.hp;

        assert!(play_self(&mut buffered, card_id));

        assert_eq!(buffered.state.player.hp, hp_before);
        assert_eq!(buffered.state.player.block, 9);
        assert_eq!(buffered.state.player.status(sid::BUFFER), 0);
        assert_eq!(buffered.state.player.status(sid::STRENGTH), card_strength);
    }
}

#[test]
fn sadistic_nature_plus_deals_unmodified_thorns_damage_for_applied_debuffs() {
    // SadisticNature.java costs zero and upgrades magic 5 -> 7. SadisticPower
    // reacts when its owner applies an enemy debuff not blocked by Artifact and
    // queues DamageInfo.THORNS; Flight therefore neither halves the seven damage
    // nor loses a stack.
    // Java: reference/extracted/methods/card/SadisticNature.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/SadisticPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FlightPower.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("Byrd", 30, 30)], 0);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Sadistic Nature+", "Trip"]);
    engine.state.enemies[0].entity.set_status(sid::FLIGHT, 3);

    assert!(play_self(&mut engine, "Sadistic Nature+"));
    assert_eq!(engine.state.player.status(sid::SADISTIC), 7);
    assert!(play_on_enemy(&mut engine, "Trip", 0));

    assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 2);
    assert_eq!(engine.state.enemies[0].entity.hp, 23);
    assert_eq!(engine.state.enemies[0].entity.status(sid::FLIGHT), 3);
}

#[test]
fn panacea_base_and_upgrade_stack_three_artifact_for_free_and_exhaust() {
    // Panacea.java applies magic 1 ArtifactPower (2 upgraded), costs zero,
    // targets self, and exhausts. Upgrade changes only the magic number.
    let registry = global_registry();
    let base = registry.get("Panacea").expect("Panacea is registered");
    let upgraded = registry.get("Panacea+").expect("Panacea+ is registered");
    assert_eq!(base.cost, 0);
    assert_eq!(base.card_type, CardType::Skill);
    assert_eq!(base.target, CardTarget::SelfTarget);
    assert_eq!(base.base_magic, 1);
    assert_eq!(upgraded.base_magic, 2);
    assert!(base.exhaust && upgraded.exhaust);

    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 0);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Panacea", "Panacea+"]);

    assert!(play_self(&mut engine, "Panacea"));
    assert!(play_self(&mut engine, "Panacea+"));

    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.player.status(sid::ARTIFACT), 3);
    assert_eq!(exhaust_prefix_count(&engine, "Panacea"), 2);
}

#[test]
fn panic_button_blocks_future_block_for_two_rounds_unless_artifact_stops_it() {
    // PanicButton.java queues block 30 before NoBlockPower(2), costs zero, and
    // exhausts; upgrade changes only block to 40. NoBlockPower is a DEBUFF,
    // returns zero from modifyBlockLast, and reduces at each round end.
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 80, 80)], 3);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["PanicButton", "Defend"]);

    assert!(play_self(&mut engine, "PanicButton"));
    assert_eq!(engine.state.player.block, 30);
    assert_eq!(engine.state.player.status(sid::NO_BLOCK), 2);
    assert_eq!(exhaust_prefix_count(&engine, "PanicButton"), 1);
    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.player.block, 30);

    end_turn(&mut engine);
    assert_eq!(engine.state.player.status(sid::NO_BLOCK), 1);
    ensure_in_hand(&mut engine, "Defend");
    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.player.block, 0);

    end_turn(&mut engine);
    assert_eq!(engine.state.player.status(sid::NO_BLOCK), 0);
    ensure_in_hand(&mut engine, "Defend");
    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.player.block, 5);

    let mut artifact =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 80, 80)], 3);
    force_player_turn(&mut artifact);
    artifact.state.player.set_status(sid::ARTIFACT, 1);
    artifact.state.hand = make_deck(&["PanicButton+", "Defend"]);

    assert!(play_self(&mut artifact, "PanicButton+"));
    assert_eq!(artifact.state.player.block, 40);
    assert_eq!(artifact.state.player.status(sid::ARTIFACT), 0);
    assert_eq!(artifact.state.player.status(sid::NO_BLOCK), 0);
    assert!(play_self(&mut artifact, "Defend"));
    assert_eq!(artifact.state.player.block, 45);
}

#[test]
fn colorless_wave1_registry_exports_match_typed_surface() {
    let registry = global_registry();

    let dramatic_entrance = registry
        .get("Dramatic Entrance")
        .expect("Dramatic Entrance should exist");
    assert_eq!(dramatic_entrance.card_type, CardType::Attack);
    assert_eq!(dramatic_entrance.target, CardTarget::AllEnemy);
    assert!(dramatic_entrance.exhaust);
    assert!(dramatic_entrance.has_test_marker("innate"));
    assert_eq!(
        dramatic_entrance.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))]
    );

    let good_instincts = registry
        .get("Good Instincts")
        .expect("Good Instincts should exist");
    assert_eq!(good_instincts.card_type, CardType::Skill);
    assert_eq!(good_instincts.target, CardTarget::SelfTarget);
    assert_eq!(
        good_instincts.effect_data,
        &[E::Simple(SE::GainBlock(A::Block))]
    );

    let finesse = registry.get("Finesse").expect("Finesse should exist");
    assert_eq!(finesse.base_magic, -1);
    assert_eq!(
        finesse.effect_data,
        &[E::Simple(SE::DrawCards(A::Fixed(1)))]
    );

    let flash = registry
        .get("Flash of Steel")
        .expect("Flash of Steel should exist");
    assert_eq!(
        (flash.cost, flash.base_damage, flash.base_magic),
        (0, 3, -1)
    );
    assert_eq!(flash.effect_data, &[E::Simple(SE::DrawCards(A::Fixed(1)))]);

    let swift_strike = registry
        .get("Swift Strike")
        .expect("Swift Strike should exist");
    assert_eq!(
        swift_strike.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let magnetism = registry.get("Magnetism").expect("Magnetism should exist");
    let magnetism_plus = registry.get("Magnetism+").expect("Magnetism+ should exist");
    assert_eq!((magnetism.cost, magnetism.base_magic), (2, 1));
    assert_eq!((magnetism_plus.cost, magnetism_plus.base_magic), (1, 1));
    assert_eq!(
        magnetism.effect_data,
        &[E::Simple(SE::AddStatus(
            T::Player,
            sid::MAGNETISM,
            A::Magic
        ))]
    );

    let mayhem = registry.get("Mayhem").expect("Mayhem should exist");
    let mayhem_plus = registry.get("Mayhem+").expect("Mayhem+ should exist");
    assert_eq!((mayhem.cost, mayhem.base_magic), (2, 1));
    assert_eq!((mayhem_plus.cost, mayhem_plus.base_magic), (1, 1));
    assert_eq!(
        mayhem.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::MAYHEM, A::Magic))]
    );

    let panache = registry.get("Panache").expect("Panache should exist");
    assert_eq!(
        panache.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::PANACHE, A::Magic))]
    );

    let sadistic = registry
        .get("Sadistic Nature")
        .expect("Sadistic Nature should exist");
    assert_eq!(
        sadistic.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::SADISTIC, A::Magic))]
    );
}

#[test]
fn finesse_gains_modified_block_and_draws_exactly_one() {
    // Finesse.java queues GainBlockAction(this.block) and DrawCardAction(1);
    // the upgrade adds two Block and does not change the draw amount.
    // Java: reference/extracted/methods/card/Finesse.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 0);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Finesse", "Finesse+"]);
    engine.state.draw_pile = make_deck(&["Strike", "Defend"]);
    engine.state.player.set_status(sid::DEXTERITY, 2);
    engine.state.player.set_status(sid::FRAIL, 1);

    assert!(play_self(&mut engine, "Finesse"));
    assert_eq!(engine.state.player.block, 3);
    assert_eq!(engine.state.hand.len(), 2);

    assert!(play_self(&mut engine, "Finesse+"));
    assert_eq!(engine.state.player.block, 7);
    assert_eq!(engine.state.hand.len(), 2);
    assert!(engine.state.draw_pile.is_empty());
}

#[test]
fn flash_of_steel_variants_deal_card_damage_then_draw_exactly_one() {
    // FlashOfSteel.java queues DamageAction(this.damage) followed by
    // DrawCardAction(1); upgrading adds three damage and does not add or alter
    // a magic stat.
    // Java: reference/extracted/methods/card/FlashOfSteel.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 60, 60)], 0);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Flash of Steel", "Flash of Steel+"]);
    engine.state.draw_pile = make_deck(&["Strike", "Defend"]);

    assert!(play_on_enemy(&mut engine, "Flash of Steel", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 57);
    assert_eq!(engine.state.hand.len(), 2);

    assert!(play_on_enemy(&mut engine, "Flash of Steel+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 51);
    assert_eq!(engine.state.hand.len(), 2);
    assert!(engine.state.draw_pile.is_empty());
}

#[test]
fn good_instincts_variants_gain_one_modified_block_amount_for_zero_energy() {
    // GoodInstincts.java queues one GainBlockAction using block 6/9 and its
    // upgrade changes only block by +3. Dexterity and Frail therefore apply
    // once to the complete amount.
    // Java: reference/extracted/methods/card/GoodInstincts.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 0);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Good Instincts", "Good Instincts+"]);
    engine.state.player.set_status(sid::DEXTERITY, 2);
    engine.state.player.set_status(sid::FRAIL, 1);

    assert!(play_self(&mut engine, "Good Instincts"));
    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.player.block, 6); // floor((6 + 2) * 0.75)

    assert!(play_self(&mut engine, "Good Instincts+"));
    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.player.block, 14); // + floor((9 + 2) * 0.75)
}

#[test]
fn colorless_wave1_attack_and_block_cards_follow_java_oracle_on_engine_path() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 50, 50),
            enemy_no_intent("Cultist", 40, 40),
        ],
        10,
    );
    force_player_turn(&mut engine);

    ensure_in_hand(&mut engine, "Swift Strike");
    assert!(play_on_enemy(&mut engine, "Swift Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 43);

    ensure_in_hand(&mut engine, "Swift Strike+");
    assert!(play_on_enemy(&mut engine, "Swift Strike+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 33);

    ensure_in_hand(&mut engine, "Good Instincts");
    assert!(play_self(&mut engine, "Good Instincts"));
    assert_eq!(engine.state.player.block, 6);

    ensure_in_hand(&mut engine, "Good Instincts+");
    assert!(play_self(&mut engine, "Good Instincts+"));
    assert_eq!(engine.state.player.block, 15);

    ensure_in_hand(&mut engine, "Dramatic Entrance");
    assert!(play_on_enemy(&mut engine, "Dramatic Entrance", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 25);
    assert_eq!(engine.state.enemies[1].entity.hp, 32);
    assert_eq!(exhaust_prefix_count(&engine, "Dramatic Entrance"), 1);

    ensure_in_hand(&mut engine, "Dramatic Entrance+");
    assert!(play_on_enemy(&mut engine, "Dramatic Entrance+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 13);
    assert_eq!(engine.state.enemies[1].entity.hp, 20);
    assert_eq!(exhaust_prefix_count(&engine, "Dramatic Entrance"), 2);
}

#[test]
fn swift_strike_is_free_and_carries_the_source_strike_tag() {
    // Swift Strike costs zero, deals 7 damage, upgrades by 3, and carries the
    // STRIKE tag. StrikeDummy.atDamageModify adds 3 to each tagged attack.
    // Java: reference/extracted/methods/card/SwiftStrike.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/StrikeDummy.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 30, 30)], 0);
    force_player_turn(&mut engine);
    engine.state.relics.push("StrikeDummy".to_string());
    engine.state.hand = make_deck(&["Swift Strike", "Swift Strike+"]);

    assert!(play_on_enemy(&mut engine, "Swift Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 20);
    assert_eq!(engine.state.energy, 0);

    assert!(play_on_enemy(&mut engine, "Swift Strike+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 7);
    assert_eq!(engine.state.energy, 0);
}

#[test]
fn dramatic_entrance_deals_one_free_aoe_hit_and_exhausts() {
    // DramaticEntrance.java deals multiDamage 8 to all enemies for 0 energy,
    // exhausts, and upgradeDamage(4) is the only numerical upgrade.
    // Java: reference/extracted/methods/card/DramaticEntrance.java
    for (card_id, expected_hp) in [("Dramatic Entrance", 32), ("Dramatic Entrance+", 28)] {
        let mut engine = engine_without_start(
            Vec::new(),
            vec![
                enemy_no_intent("JawWorm", 40, 40),
                enemy_no_intent("Cultist", 40, 40),
            ],
            0,
        );
        force_player_turn(&mut engine);
        engine.state.hand = make_deck(&[card_id]);

        assert!(play_self(&mut engine, card_id));
        assert_eq!(engine.state.enemies[0].entity.hp, expected_hp);
        assert_eq!(engine.state.enemies[1].entity.hp, expected_hp);
        assert_eq!(engine.state.energy, 0);
        assert_eq!(exhaust_prefix_count(&engine, "Dramatic Entrance"), 1);
    }
}

#[test]
fn bandage_up_heals_four_or_six_for_free_then_exhausts() {
    // Source: BandageUp.java queues HealAction for magicNumber 4, costs 0,
    // Exhausts, and upgradeMagicNumber(2) raises the heal to 6.
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.player.hp = 60;

    for (card_id, expected_hp) in [("Bandage Up", 64), ("Bandage Up+", 70)] {
        ensure_in_hand(&mut engine, card_id);
        assert!(play_self(&mut engine, card_id));
        assert_eq!(engine.state.player.hp, expected_hp);
        assert_eq!(engine.state.energy, 3);
    }

    assert_eq!(exhaust_prefix_count(&engine, "Bandage Up"), 2);
}

#[test]
fn bite_variants_heal_after_both_nonlethal_and_final_lethal_damage() {
    // Bite.java lines 41-42 queue 7 damage then HealAction(2), upgrading both
    // by one. Although DamageAction performs post-combat cleanup after a final
    // kill, GameActionManager.java lines 124-130 explicitly preserve HealAction.
    for (card_id, damage, healing) in [("Bite", 7, 2), ("Bite+", 8, 3)] {
        let mut nonlethal =
            engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
        force_player_turn(&mut nonlethal);
        nonlethal.state.player.hp = 50;
        nonlethal.state.hand = make_deck(&[card_id]);
        assert!(play_on_enemy(&mut nonlethal, card_id, 0));
        assert_eq!(nonlethal.state.enemies[0].entity.hp, 40 - damage);
        assert_eq!(nonlethal.state.player.hp, 50 + healing);
        assert_eq!(nonlethal.state.energy, 2);

        let mut lethal = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", damage, damage)],
            3,
        );
        force_player_turn(&mut lethal);
        lethal.state.player.hp = 50;
        lethal.state.hand = make_deck(&[card_id]);
        assert!(play_on_enemy(&mut lethal, card_id, 0));
        assert!(lethal.state.enemies[0].entity.is_dead());
        assert_eq!(lethal.state.player.hp, 50 + healing);
    }
}

#[test]
fn blind_targets_one_enemy_while_blind_plus_targets_every_living_enemy() {
    // Source: Blind.java applies 2 Weak to the selected enemy at base; its
    // upgrade loops every monster with a separate ApplyPowerAction.
    let mut base = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 40, 40),
            enemy_no_intent("Cultist", 40, 40),
        ],
        0,
    );
    force_player_turn(&mut base);
    base.state.hand = make_deck(&["Blind"]);
    assert!(play_on_enemy(&mut base, "Blind", 1));
    assert_eq!(base.state.enemies[0].entity.status(sid::WEAKENED), 0);
    assert_eq!(base.state.enemies[1].entity.status(sid::WEAKENED), 2);

    let mut upgraded = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 40, 40),
            enemy_no_intent("Cultist", 40, 40),
        ],
        0,
    );
    force_player_turn(&mut upgraded);
    upgraded.state.enemies[0]
        .entity
        .set_status(sid::ARTIFACT, 1);
    upgraded.state.hand = make_deck(&["Blind+"]);
    assert!(play_self(&mut upgraded, "Blind+"));
    assert_eq!(upgraded.state.enemies[0].entity.status(sid::ARTIFACT), 0);
    assert_eq!(upgraded.state.enemies[0].entity.status(sid::WEAKENED), 0);
    assert_eq!(upgraded.state.enemies[1].entity.status(sid::WEAKENED), 2);
    assert_eq!(upgraded.state.energy, 0);
}

#[test]
fn dark_shackles_temporarily_removes_strength_unless_artifact_blocks_it() {
    // Source: DarkShackles.java applies StrengthPower(-9), then a matching
    // GainStrengthPower(9) only when Artifact is absent; upgrading adds 6.
    let mut base = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 0);
    force_player_turn(&mut base);
    base.state.enemies[0].entity.set_status(sid::STRENGTH, 5);
    base.state.hand = make_deck(&["Dark Shackles"]);

    assert!(play_on_enemy(&mut base, "Dark Shackles", 0));
    assert_eq!(base.state.enemies[0].entity.status(sid::STRENGTH), -4);
    assert_eq!(
        base.state.enemies[0].entity.status(sid::TEMP_STRENGTH_LOSS),
        9
    );
    assert_eq!(exhaust_prefix_count(&base, "Dark Shackles"), 1);

    end_turn(&mut base);
    assert_eq!(base.state.enemies[0].entity.status(sid::STRENGTH), 5);
    assert_eq!(
        base.state.enemies[0].entity.status(sid::TEMP_STRENGTH_LOSS),
        0
    );

    let mut blocked = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 0);
    force_player_turn(&mut blocked);
    blocked.state.enemies[0].entity.set_status(sid::STRENGTH, 5);
    blocked.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);
    blocked.state.hand = make_deck(&["Dark Shackles+"]);

    assert!(play_on_enemy(&mut blocked, "Dark Shackles+", 0));
    assert_eq!(blocked.state.enemies[0].entity.status(sid::ARTIFACT), 0);
    assert_eq!(blocked.state.enemies[0].entity.status(sid::STRENGTH), 5);
    assert_eq!(
        blocked.state.enemies[0]
            .entity
            .status(sid::TEMP_STRENGTH_LOSS),
        0
    );
    assert_eq!(exhaust_prefix_count(&blocked, "Dark Shackles"), 1);
}

#[test]
fn deep_breath_only_shuffles_a_nonempty_discard_and_consumes_two_shuffle_ticks() {
    // DeepBreath.java skips both shuffle actions when discard is empty, then
    // draws 1. With cards in discard it queues EmptyDeckShuffleAction followed
    // by ShuffleAction, consuming one shuffleRng randomLong each, then draws 2
    // when upgraded.
    let mut empty = engine_without_start(
        make_deck(&["Strike", "Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        0,
    );
    force_player_turn(&mut empty);
    empty.state.hand = make_deck(&["Deep Breath"]);
    empty.clear_event_log();
    let empty_rng_before = empty.shuffle_rng.counter;

    assert!(play_self(&mut empty, "Deep Breath"));
    assert_eq!(empty.shuffle_rng.counter, empty_rng_before);
    assert_eq!(hand_count(&empty, "Defend"), 1);
    assert_eq!(
        empty
            .event_log
            .iter()
            .filter(|record| record.event == crate::effects::trigger::Trigger::OnShuffle)
            .count(),
        0
    );

    let mut shuffled = engine_without_start(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        0,
    );
    force_player_turn(&mut shuffled);
    shuffled.state.hand = make_deck(&["Deep Breath+"]);
    shuffled.state.discard_pile = make_deck(&["Defend", "Bash"]);
    shuffled.clear_event_log();
    let shuffled_rng_before = shuffled.shuffle_rng.counter;

    assert!(play_self(&mut shuffled, "Deep Breath+"));
    assert_eq!(shuffled.shuffle_rng.counter, shuffled_rng_before + 2);
    assert_eq!(shuffled.state.hand.len(), 2);
    assert_eq!(shuffled.state.draw_pile.len(), 1);
    assert_eq!(
        shuffled
            .event_log
            .iter()
            .filter(|record| record.event == crate::effects::trigger::Trigger::OnShuffle)
            .count(),
        1
    );
}

#[test]
fn colorless_wave1_power_cards_install_runtime_owned_statuses() {
    let mut magnetism =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 50, 50)], 10);
    force_player_turn(&mut magnetism);
    ensure_in_hand(&mut magnetism, "Magnetism");
    assert!(play_self(&mut magnetism, "Magnetism"));
    assert_eq!(magnetism.state.player.status(sid::MAGNETISM), 1);

    let mut mayhem = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 50, 50)], 10);
    force_player_turn(&mut mayhem);
    ensure_in_hand(&mut mayhem, "Mayhem+");
    assert!(play_self(&mut mayhem, "Mayhem+"));
    assert_eq!(mayhem.state.player.status(sid::MAYHEM), 1);

    let mut panache =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 50, 50)], 10);
    force_player_turn(&mut panache);
    ensure_in_hand(&mut panache, "Panache+");
    assert!(play_self(&mut panache, "Panache+"));
    assert_eq!(panache.state.player.status(sid::PANACHE), 14);

    let mut sadistic =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 50, 50)], 10);
    force_player_turn(&mut sadistic);
    ensure_in_hand(&mut sadistic, "Sadistic Nature");
    assert!(play_self(&mut sadistic, "Sadistic Nature"));
    assert_eq!(sadistic.state.player.status(sid::SADISTIC), 5);
}
