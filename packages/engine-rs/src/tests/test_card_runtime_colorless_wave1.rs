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
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Magnetism.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Mayhem.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Panache.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/SadisticNature.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/SwiftStrike.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::*;

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
    assert_eq!(good_instincts.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let finesse = registry.get("Finesse").expect("Finesse should exist");
    assert_eq!(finesse.base_magic, -1);
    assert_eq!(finesse.effect_data, &[E::Simple(SE::DrawCards(A::Fixed(1)))]);

    let flash = registry
        .get("Flash of Steel")
        .expect("Flash of Steel should exist");
    assert_eq!((flash.cost, flash.base_damage, flash.base_magic), (0, 3, -1));
    assert_eq!(flash.effect_data, &[E::Simple(SE::DrawCards(A::Fixed(1)))]);

    let swift_strike = registry
        .get("Swift Strike")
        .expect("Swift Strike should exist");
    assert_eq!(swift_strike.effect_data, &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]);

    let magnetism = registry.get("Magnetism").expect("Magnetism should exist");
    assert_eq!(
        magnetism.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::MAGNETISM, A::Magic))]
    );

    let mayhem = registry.get("Mayhem").expect("Mayhem should exist");
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
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        0,
    );
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
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        0,
    );
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
fn colorless_wave1_attack_and_block_cards_follow_java_oracle_on_engine_path() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50), enemy_no_intent("Cultist", 40, 40)],
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
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
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
fn bite_variants_heal_after_nonlethal_damage_but_not_after_final_lethal_damage() {
    // Source: Bite.java queues 7 damage then HealAction(2), upgrading both by
    // one. DamageAction.java clears queued post-combat actions after a final kill.
    for (card_id, damage, healing) in [("Bite", 7, 2), ("Bite+", 8, 3)] {
        let mut nonlethal = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );
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
        assert_eq!(lethal.state.player.hp, 50);
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
    upgraded.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);
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
    let mut base = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        0,
    );
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

    let mut blocked = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        0,
    );
    force_player_turn(&mut blocked);
    blocked.state.enemies[0].entity.set_status(sid::STRENGTH, 5);
    blocked.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);
    blocked.state.hand = make_deck(&["Dark Shackles+"]);

    assert!(play_on_enemy(&mut blocked, "Dark Shackles+", 0));
    assert_eq!(blocked.state.enemies[0].entity.status(sid::ARTIFACT), 0);
    assert_eq!(blocked.state.enemies[0].entity.status(sid::STRENGTH), 5);
    assert_eq!(
        blocked.state.enemies[0].entity.status(sid::TEMP_STRENGTH_LOSS),
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
    let empty_rng_before = empty.rng.counter;

    assert!(play_self(&mut empty, "Deep Breath"));
    assert_eq!(empty.rng.counter, empty_rng_before);
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
    let shuffled_rng_before = shuffled.rng.counter;

    assert!(play_self(&mut shuffled, "Deep Breath+"));
    assert_eq!(shuffled.rng.counter, shuffled_rng_before + 2);
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
    let mut magnetism = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        10,
    );
    force_player_turn(&mut magnetism);
    ensure_in_hand(&mut magnetism, "Magnetism");
    assert!(play_self(&mut magnetism, "Magnetism"));
    assert_eq!(magnetism.state.player.status(sid::MAGNETISM), 1);

    let mut mayhem = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        10,
    );
    force_player_turn(&mut mayhem);
    ensure_in_hand(&mut mayhem, "Mayhem+");
    assert!(play_self(&mut mayhem, "Mayhem+"));
    assert_eq!(mayhem.state.player.status(sid::MAYHEM), 1);

    let mut panache = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        10,
    );
    force_player_turn(&mut panache);
    ensure_in_hand(&mut panache, "Panache+");
    assert!(play_self(&mut panache, "Panache+"));
    assert_eq!(panache.state.player.status(sid::PANACHE), 14);

    let mut sadistic = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        10,
    );
    force_player_turn(&mut sadistic);
    ensure_in_hand(&mut sadistic, "Sadistic Nature");
    assert!(play_self(&mut sadistic, "Sadistic Nature"));
    assert_eq!(sadistic.state.player.status(sid::SADISTIC), 5);
}
