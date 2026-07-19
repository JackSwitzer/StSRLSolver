#![cfg(test)]

// Java oracle sources for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/status/Slimed.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Decay.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Regret.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Shame.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Pain.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/status/VoidCard.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Parasite.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Necronomicurse.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java

use crate::actions::Action;
use crate::cards::{global_registry, CardTarget};
use crate::effects::types::{CardRuntimeTrigger, EndTurnHandRule, OnDrawRule, WhileInHandRule};
use crate::status_ids::sid;
use crate::tests::support::*;

#[test]
fn support_wave1_registry_keeps_shared_support_cards_typed_runtime_metadata() {
    let registry = global_registry();

    for card_id in [
        "Slimed", "Wound", "Dazed", "Burn", "Burn+", "Void", "Decay", "Regret", "Doubt",
        "Shame", "AscendersBane", "Clumsy", "CurseOfTheBell", "Injury", "Necronomicurse",
        "Normality", "Pain", "Parasite", "Pride", "Writhe",
    ] {
        let card = registry.get(card_id).unwrap_or_else(|| panic!("{card_id} should exist"));
        assert!(card.complex_hook.is_none(), "{card_id} should stay non-bespoke");
    }

    let burn = registry.get("Burn").unwrap();
    assert!(burn.runtime_traits().unplayable);
    assert!(burn
        .runtime_triggers()
        .iter()
        .any(|trigger| matches!(trigger, CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Damage))));

    let regret = registry.get("Regret").unwrap();
    assert!(regret.runtime_traits().unplayable);
    assert!(regret
        .runtime_triggers()
        .iter()
        .any(|trigger| matches!(trigger, CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Regret))));

    let doubt = registry.get("Doubt").unwrap();
    assert!(doubt.runtime_traits().unplayable);
    assert!(doubt
        .runtime_triggers()
        .iter()
        .any(|trigger| matches!(trigger, CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Weak))));

    let shame = registry.get("Shame").unwrap();
    assert!(shame.runtime_traits().unplayable);
    assert!(shame
        .runtime_triggers()
        .iter()
        .any(|trigger| matches!(trigger, CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Frail))));

    let pain = registry.get("Pain").unwrap();
    assert!(pain.runtime_traits().unplayable);
    assert!(pain
        .runtime_triggers()
        .iter()
        .any(|trigger| matches!(trigger, CardRuntimeTrigger::WhileInHand(WhileInHandRule::PainOnOtherCardPlayed))));

    let void = registry.get("Void").unwrap();
    assert!(void.runtime_traits().unplayable);
    assert!(void.runtime_traits().ethereal);
    assert!(void
        .runtime_triggers()
        .iter()
        .any(|trigger| matches!(trigger, CardRuntimeTrigger::OnDraw(OnDrawRule::LoseEnergy))));

    assert!(registry.get("Parasite").unwrap().runtime_traits().unplayable);
    assert!(registry.get("Dazed").unwrap().runtime_traits().ethereal);
    assert!(registry.get("Wound").unwrap().runtime_traits().unplayable);
}

#[test]
fn slimed_is_a_one_cost_self_targeted_no_op_that_exhausts() {
    // Slimed.java constructs a SELF-targeting Status at cost 1, sets exhaust,
    // and leaves both use() and upgrade() empty.
    let registry = global_registry();
    let slimed = registry.get("Slimed").expect("Slimed");
    assert_eq!(slimed.target, CardTarget::SelfTarget);
    assert_eq!(slimed.cost, 1);
    assert!(slimed.exhaust);
    assert!(registry.get("Slimed+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        1,
    );
    force_player_turn(&mut engine);
    engine.state.energy = 1;
    engine.state.hand = make_deck(&["Slimed"]);

    assert!(play_self(&mut engine, "Slimed"));

    assert_eq!(engine.state.energy, 0);
    assert_eq!(exhaust_prefix_count(&engine, "Slimed"), 1);
    assert_eq!(discard_prefix_count(&engine, "Slimed"), 0);
}

#[test]
fn wound_is_unplayable_unupgradable_non_ethereal_and_discards_normally() {
    // Wound.java constructs a NONE-targeted cost -2 Status and leaves use()
    // and upgrade() empty. It sets neither Ethereal nor Exhaust, so an
    // ordinary copy cannot be played and moves from hand to discard at end of
    // turn.
    // Java: reference/extracted/methods/card/Wound.java
    let registry = global_registry();
    let wound = registry.get("Wound").expect("Wound");
    assert_eq!(wound.target, CardTarget::None);
    assert_eq!(wound.cost, -2);
    assert!(wound.runtime_traits().unplayable);
    assert!(!wound.runtime_traits().ethereal);
    assert!(!wound.exhaust);
    assert!(registry.get("Wound+").is_none());

    let mut engine = engine_without_start(
        make_deck_n("Strike", 10),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Wound"]);
    assert!(!engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { card_idx: 0, .. })
    }));

    end_turn(&mut engine);

    assert_eq!(discard_prefix_count(&engine, "Wound"), 1);
    assert_eq!(exhaust_prefix_count(&engine, "Wound"), 0);
}

#[test]
fn curse_of_the_bell_is_unplayable_unupgradable_non_ethereal_and_unremovable() {
    // CurseOfTheBell.java constructs a cost -2 Curse with empty use/upgrade
    // and no Ethereal flag. CardGroup.getPurgeableCards explicitly excludes
    // CurseOfTheBell from removal pools.
    let registry = global_registry();
    let card = registry.get("CurseOfTheBell").expect("Curse of the Bell");
    assert_eq!(card.cost, -2);
    assert!(card.runtime_traits().unplayable);
    assert!(card.runtime_traits().unremovable);
    assert!(!card.runtime_traits().ethereal);
    assert!(registry.get("CurseOfTheBell+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["CurseOfTheBell"]);
    assert!(!engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { card_idx: 0, .. })
    }));

    end_turn(&mut engine);

    assert_eq!(hand_count(&engine, "CurseOfTheBell"), 1);
    assert_eq!(exhaust_prefix_count(&engine, "CurseOfTheBell"), 0);
}

#[test]
fn necronomicurse_is_unplayable_unremovable_and_returns_after_exhaust() {
    let registry = global_registry();
    let card = registry.get("Necronomicurse").expect("Necronomicurse");
    assert_eq!(card.cost, -2);
    assert!(card.runtime_traits().unplayable);
    assert!(card.runtime_traits().unremovable);
    assert!(!card.runtime_traits().ethereal);
    assert!(registry.get("Necronomicurse+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.relics.push("Blue Candle".to_string());
    engine.state.player.set_status(sid::DARK_EMBRACE, 1);
    engine.rebuild_effect_runtime();
    engine.state.hand = make_deck(&[
        "Necronomicurse", "Defend", "Defend", "Defend", "Defend",
        "Defend", "Defend", "Defend", "Defend", "Defend",
    ]);
    engine.state.draw_pile = make_deck(&["Strike"]);

    assert!(play_self(&mut engine, "Necronomicurse"));

    // CardGroup.moveToExhaustPile calls Dark Embrace before the card's own
    // trigger. The draw fills the tenth hand slot, so MakeTempCardInHandAction
    // spills the new curse into discard while the original remains exhausted.
    assert_eq!(engine.state.player.hp, 79);
    assert_eq!(hand_count(&engine, "Necronomicurse"), 0);
    assert_eq!(hand_count(&engine, "Strike"), 1);
    assert_eq!(discard_prefix_count(&engine, "Necronomicurse"), 1);
    assert_eq!(exhaust_prefix_count(&engine, "Necronomicurse"), 1);
}

#[test]
fn injury_is_unplayable_unupgradable_non_ethereal_and_inert() {
    // Injury.java constructs a cost -2 Curse with target NONE, empty use(),
    // empty upgrade(), and no Ethereal flag. It therefore has no legal play or
    // end-turn effect and is discarded normally with the rest of the hand.
    let registry = global_registry();
    let injury = registry.get("Injury").expect("Injury is registered");
    assert_eq!(injury.cost, -2);
    assert!(injury.runtime_traits().unplayable);
    assert!(!injury.runtime_traits().ethereal);
    assert!(registry.get("Injury+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Injury"]);
    engine.state.draw_pile = make_deck(&[
        "Defend", "Defend", "Defend", "Defend", "Defend",
    ]);
    let hp_before = engine.state.player.hp;

    assert!(!engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { card_idx: 0, .. })
    }));
    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, hp_before);
    assert_eq!(discard_prefix_count(&engine, "Injury"), 1);
    assert_eq!(exhaust_prefix_count(&engine, "Injury"), 0);
}

#[test]
fn parasite_is_unplayable_unupgradable_non_ethereal_and_inert_in_combat() {
    // Parasite.java constructs a cost -2 Curse with target NONE, empty use and
    // upgrade methods, and no Ethereal flag. Its only effect is the separate
    // onRemoveFromMasterDeck max-HP hook.
    let registry = global_registry();
    let parasite = registry.get("Parasite").expect("Parasite is registered");
    assert_eq!(parasite.cost, -2);
    assert!(parasite.runtime_traits().unplayable);
    assert!(!parasite.runtime_traits().ethereal);
    assert!(registry.get("Parasite+").is_none());

    let mut engine = engine_without_start(
        make_deck(&["Defend", "Defend", "Defend", "Defend", "Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Parasite"]);
    assert!(!engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { .. })
    }));

    let hp_before = engine.state.player.hp;
    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, hp_before);
    assert_eq!(discard_prefix_count(&engine, "Parasite"), 1);
    assert_eq!(exhaust_prefix_count(&engine, "Parasite"), 0);
}

#[test]
fn normality_blocks_every_card_from_the_fourth_play_only_while_in_hand() {
    // Normality.java::canPlay checks cardsPlayedThisTurn.size() >= 3. Because
    // AbstractCard.hasEnoughEnergy polls canPlay on every card in hand, one
    // Normality blocks every candidate (including itself with Blue Candle).
    let registry = global_registry();
    let normality = registry.get("Normality").expect("Normality is registered");
    assert_eq!(normality.cost, -2);
    assert!(normality.runtime_traits().unplayable);
    assert!(!normality.runtime_traits().ethereal);
    assert!(registry.get("Normality+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        99,
    );
    force_player_turn(&mut engine);
    engine.state.relics.push("Blue Candle".to_string());
    engine.state.hand = make_deck(&["Normality", "Strike", "Defend"]);

    engine.state.cards_played_this_turn = 2;
    assert!(engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { card_idx: 1, .. })
    }));

    engine.state.cards_played_this_turn = 3;
    assert!(!engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { .. })
    }));

    engine.state.hand.remove(0);
    assert!(engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { card_idx: 0, .. })
    }));
}

#[test]
fn support_wave1_end_turn_curse_and_status_hooks_fire_on_the_runtime_path() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Burn", "Regret", "Doubt", "Shame", "Pride"]);

    let hp_before = engine.state.player.hp;
    end_turn(&mut engine);

    assert_eq!(hp_before - engine.state.player.hp, 7);
    let pride_count = engine
        .state
        .hand
        .iter()
        .chain(engine.state.draw_pile.iter())
        .chain(engine.state.discard_pile.iter())
        .filter(|card| engine.card_registry.card_name(card.def_id).starts_with("Pride"))
        .count();
    assert_eq!(
        pride_count, 2,
        "Pride's end-turn copy may be drawn immediately but must still exist"
    );
}

#[test]
fn rng_collections_001_ordinary_actions_resolve_before_shuffled_card_queue() {
    // DiscardAtEndOfTurnAction clones the non-retained hand, calls the
    // no-argument Collections.shuffle, then triggers that shuffled snapshot.
    // JDK 8 Collections uses its separate static java.util.Random; an internal
    // state of 0x5DEECE66D (the post-constructor state for Java seed 0) shuffles
    // [Pride, Shame, Regret, Burn] to [Burn, Pride, Shame, Regret] and ends at
    // 0x3D93CB7AB84E after exactly three nextInt calls.
    // Sources: actions/common/DiscardAtEndOfTurnAction.java:21-45 and JDK 8
    // java.util.Collections.shuffle/java.util.Random.
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.player.hp = 2;
    engine.state.hand = make_deck(&["Pride", "Shame", "Regret", "Burn"]);
    engine.restore_java_collections_rng_state(0x5DEECE66D);
    assert_eq!(engine.java_collections_rng_state(), 0x5DEECE66D);
    let dungeon_rngs_before = engine.rng_counters();

    end_turn(&mut engine);

    assert!(engine.state.combat_over, "shuffled Burn should be the first card-queue item");
    assert_eq!(engine.state.player.hp, 0);
    assert_eq!(engine.state.player.status(sid::FRAIL), 0);
    let pride_count = engine
        .state
        .hand
        .iter()
        .chain(engine.state.draw_pile.iter())
        .chain(engine.state.discard_pile.iter())
        .filter(|card| engine.card_registry.card_name(card.def_id) == "Pride")
        .count();
    assert_eq!(
        pride_count, 2,
        "Pride's ordinary action resolves before every card-queue item"
    );
    assert_eq!(engine.java_collections_rng_state(), 0x3D93CB7AB84E);
    assert_eq!(engine.rng_counters(), dungeon_rngs_before);
}

#[test]
fn rng_collections_001_run_carries_state_across_combat_boundaries() {
    // The no-argument Collections.shuffle stream is process-global in Java.
    // RunEngine must absorb the post-combat LCG state and inject that same
    // state into the next combat instead of rebuilding Java seed zero.
    let mut run = crate::run::RunEngine::new(42, 0);
    run.debug_enter_specific_combat(&["JawWorm"]);

    let state_after_first_shuffle = {
        let combat = run.debug_combat_engine_mut();
        combat.state.hand = make_deck(&["Strike", "Defend", "Eruption", "Burn"]);
        combat.restore_java_collections_rng_state(0x5DEECE66D);
        assert!(!crate::status_effects::process_end_turn_hand_cards(combat));
        combat.java_collections_rng_state()
    };
    assert_eq!(state_after_first_shuffle, 0x3D93CB7AB84E);

    run.debug_force_current_combat_outcome(true);
    run.debug_resolve_current_combat_outcome();
    run.debug_enter_specific_combat(&["Cultist"]);

    assert_eq!(
        run.get_combat_engine()
            .expect("second combat")
            .java_collections_rng_state(),
        state_after_first_shuffle
    );
}

#[test]
fn shame_applies_one_frail_that_survives_its_application_round() {
    // Shame.java leaves the inherited magic number at -1, then queues itself
    // at end of turn and constructs FrailPower(player, 1, true). FrailPower's
    // true justApplied flag skips the immediately following atEndOfRound
    // decrement, so the player starts the next turn with exactly one Frail.
    let registry = global_registry();
    let shame = registry.get("Shame").expect("Shame");
    assert_eq!(shame.cost, -2);
    assert_eq!(shame.base_magic, -1);
    assert!(registry.get("Shame+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Shame"]);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.status(sid::FRAIL), 1);
    assert_eq!(engine.state.player.status(sid::FRAIL_JUST_APPLIED), 0);
}

#[test]
fn regret_uses_the_hand_size_after_explicitly_retained_cards_leave_hand() {
    // DiscardAtEndOfTurnAction moves retained cards to limbo before calling
    // triggerOnEndOfPlayerTurn. Regret then snapshots player.hand.size(), so
    // the retained Strike neither counts nor prevents HP_LOSS bypassing Block.
    // Sources: actions/common/DiscardAtEndOfTurnAction.java,
    // cards/curses/Regret.java, and actions/common/LoseHPAction.java.
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    let registry = global_registry();
    let mut retained_strike = registry.make_card("Strike");
    retained_strike.set_retained(true);
    engine.state.hand = vec![
        registry.make_card("Regret"),
        retained_strike,
        registry.make_card("Defend"),
    ];
    engine.state.player.block = 20;
    let hp_before = engine.state.player.hp;

    assert!(!crate::status_effects::process_end_turn_hand_cards(&mut engine));

    assert_eq!(engine.state.player.hp, hp_before - 2);
    assert_eq!(engine.state.player.block, 20);
}

#[test]
fn pride_is_playable_or_copies_itself_to_the_top_without_rng() {
    // Pride.java constructs a one-cost SELF Curse with Exhaust and Innate,
    // defines no upgrade, and has an empty use(). Its end-turn hand trigger
    // queues one stat-equivalent copy with randomSpot=false.
    let registry = global_registry();
    let pride = registry.get("Pride").expect("Pride");
    assert_eq!(pride.cost, 1);
    assert_eq!(pride.target, CardTarget::SelfTarget);
    assert!(pride.exhaust);
    assert!(pride.runtime_traits().innate);
    assert!(!pride.runtime_traits().unplayable);
    assert!(registry.get("Pride+").is_none());

    let mut played = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut played);
    played.state.hand = make_deck(&["Pride"]);
    assert!(play_self(&mut played, "Pride"));
    assert_eq!(played.state.energy, 2);
    assert_eq!(exhaust_prefix_count(&played, "Pride"), 1);
    assert_eq!(draw_prefix_count(&played, "Pride"), 0);

    let mut held = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut held);
    held.state.draw_pile = make_deck(&["Strike", "Defend"]);
    let mut pride_instance = registry.make_card("Pride");
    pride_instance.misc = 17;
    held.state.hand = vec![pride_instance];
    let card_random_before = held.rng_counters()["cardRandom"];

    assert!(!crate::status_effects::process_end_turn_hand_cards(&mut held));

    assert_eq!(held.state.draw_pile.last(), Some(&pride_instance));
    assert_eq!(held.rng_counters()["cardRandom"], card_random_before);
}

#[test]
fn writhe_is_an_unplayable_unupgradable_innate_non_ethereal_curse() {
    // Writhe.java constructs a NONE-targeted cost -2 Curse, sets only
    // isInnate, and leaves use() and upgrade() empty. Combat setup therefore
    // moves it into the opening draw while it remains unplayable and persists
    // normally if held.
    // Java: reference/extracted/methods/card/Writhe.java
    let registry = global_registry();
    let writhe = registry.get("Writhe").expect("Writhe");
    assert_eq!(writhe.target, CardTarget::None);
    assert_eq!(writhe.cost, -2);
    assert!(writhe.runtime_traits().innate);
    assert!(writhe.runtime_traits().unplayable);
    assert!(!writhe.runtime_traits().ethereal);
    assert!(!writhe.exhaust);
    assert!(registry.get("Writhe+").is_none());

    let mut deck = make_deck_n("Defend", 9);
    deck.push(registry.make_card("Writhe"));
    let mut engine = engine_without_start(
        deck,
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.start_combat();

    let writhe_idx = engine
        .state
        .hand
        .iter()
        .position(|card| engine.card_registry.card_name(card.def_id) == "Writhe")
        .expect("Innate Writhe should be in the opening hand");
    assert!(!engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { card_idx, .. } if *card_idx == writhe_idx)
    }));
}

#[test]
fn decay_has_no_magic_stat_and_deals_two_thorns_damage_at_end_of_turn() {
    // Decay.java has cost -2, empty upgrade(), and no magic-number assignment.
    // Its end-turn queue auto-plays the card, whose use() deals a literal 2
    // THORNS damage; one Block therefore leaves one HP damage.
    let registry = global_registry();
    let decay = registry.get("Decay").expect("Decay");
    assert_eq!(decay.cost, -2);
    assert_eq!(decay.base_magic, -1);
    assert!(decay.runtime_traits().unplayable);
    assert!(registry.get("Decay+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.player.block = 1;
    engine.state.hand = make_deck(&["Decay"]);
    let hp_before = engine.state.player.hp;

    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, hp_before - 1);
}

#[test]
fn doubt_has_no_magic_stat_and_applies_one_weak_at_end_of_turn() {
    // Doubt.java assigns no magic number. Its end-turn queue auto-plays the
    // card with dontTriggerOnUseCard set, and use() applies a literal 1 Weak.
    // Java: reference/extracted/methods/card/Doubt.java
    let registry = global_registry();
    let doubt = registry.get("Doubt").expect("Doubt");
    assert_eq!(doubt.cost, -2);
    assert_eq!(doubt.base_magic, -1);
    assert!(doubt.runtime_traits().unplayable);
    assert!(registry.get("Doubt+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Doubt"]);

    assert!(!crate::status_effects::process_end_turn_hand_cards(&mut engine));
    assert_eq!(engine.state.player.status(crate::status_ids::sid::WEAKENED), 1);
}

#[test]
fn burn_variants_use_thorns_block_then_buffer_order_at_end_of_turn() {
    // Burn.use queues self-targeted DamageAction with DamageType.THORNS for
    // magicNumber 2 (4 upgraded). AbstractPlayer.damage applies Intangible,
    // then block, then Buffer; Buffer consumes only if positive damage remains.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BufferPower.java
    let mut ordinary = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("Dummy", 40, 40)],
        3,
    );
    force_player_turn(&mut ordinary);
    ordinary.state.hand = make_deck(&["Burn", "Burn+"]);
    let hp_before = ordinary.state.player.hp;
    end_turn(&mut ordinary);
    assert_eq!(ordinary.state.player.hp, hp_before - 6);
    assert_eq!(ordinary.state.player.status(crate::status_ids::sid::HP_LOSS_THIS_COMBAT), 2);

    let mut mitigated = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("Dummy", 40, 40)],
        3,
    );
    force_player_turn(&mut mitigated);
    mitigated.state.hand = make_deck(&["Burn", "Burn+"]);
    mitigated.state.player.block = 3;
    mitigated.state.player.set_status(crate::status_ids::sid::BUFFER, 1);
    let hp_before = mitigated.state.player.hp;
    end_turn(&mut mitigated);
    assert_eq!(mitigated.state.player.hp, hp_before);
    assert_eq!(mitigated.state.player.block, 0);
    assert_eq!(mitigated.state.player.status(crate::status_ids::sid::BUFFER), 0);
    assert_eq!(mitigated.state.player.status(crate::status_ids::sid::HP_LOSS_THIS_COMBAT), 0);
}

#[test]
fn support_wave1_pain_triggers_when_any_other_card_is_played() {
    // Pain.java is cost -2, has no upgrade, and queues LoseHPAction(1) from
    // triggerOnOtherCardPlayed. AbstractPlayer.useCard queues card effects
    // first, then Pain adds its loss to the top so it resolves before them.
    let registry = global_registry();
    let pain = registry.get("Pain").expect("Pain is registered");
    assert_eq!(pain.cost, -2);
    assert!(pain.runtime_traits().unplayable);
    assert!(!pain.runtime_traits().ethereal);
    assert!(registry.get("Pain+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Pain", "Strike"]);

    let hp_before = engine.state.player.hp;
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(hp_before - engine.state.player.hp, 1);
    assert_eq!(hand_count(&engine, "Pain"), 1);

    // The Pain action resolves before Buffer's ApplyPowerAction, so the newly
    // installed Buffer survives and cannot prevent this HP loss.
    let mut before_buffer = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut before_buffer);
    before_buffer.state.hand = make_deck(&["Pain", "Buffer"]);
    let hp_before = before_buffer.state.player.hp;

    assert!(play_self(&mut before_buffer, "Buffer"));
    assert_eq!(before_buffer.state.player.hp, hp_before - 1);
    assert_eq!(before_buffer.state.player.status(sid::BUFFER), 1);

    // CardGroup excludes the played card itself but leaves the other Pain to
    // trigger; Blue Candle then contributes its own separate LoseHPAction.
    let mut blue_candle = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        0,
    );
    force_player_turn(&mut blue_candle);
    blue_candle.state.relics.push("Blue Candle".to_string());
    blue_candle.state.hand = make_deck(&["Pain", "Pain"]);
    let hp_before = blue_candle.state.player.hp;

    assert!(play_self(&mut blue_candle, "Pain"));
    assert_eq!(blue_candle.state.player.hp, hp_before - 2);
    assert_eq!(hand_count(&blue_candle, "Pain"), 1);
    assert_eq!(exhaust_prefix_count(&blue_candle, "Pain"), 1);
}

#[test]
fn support_wave1_void_loses_energy_when_drawn() {
    // VoidCard.java is cost -2, has no upgrade, is Ethereal, and queues
    // LoseEnergyAction(1) when drawn. EnergyManager.use cannot take energy
    // below zero; unplayed Void exhausts even through Runic Pyramid.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/VoidCard.java
    let mut engine = engine_without_start(
        make_deck(&["Void"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    let energy_before = engine.state.energy;
    engine.draw_cards(1);

    assert_eq!(engine.state.energy, energy_before - 1);
    assert_eq!(hand_count(&engine, "Void"), 1);
    assert!(global_registry().get("Void+").is_none());

    force_player_turn(&mut engine);
    engine.state.relics.push("Runic Pyramid".to_string());
    let void_idx = engine
        .state
        .hand
        .iter()
        .position(|card| engine.card_registry.card_name(card.def_id) == "Void")
        .expect("drawn Void should be in hand");
    assert!(!engine.get_legal_actions().iter().any(|action| matches!(
        action,
        Action::PlayCard { card_idx, .. } if *card_idx == void_idx
    )));
    end_turn(&mut engine);
    assert_eq!(hand_count(&engine, "Void"), 0);
    assert_eq!(exhaust_prefix_count(&engine, "Void"), 1);

    let mut zero_energy = engine_without_start(
        make_deck(&["Void"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        0,
    );
    zero_energy.draw_cards(1);
    assert_eq!(zero_energy.state.energy, 0);
}

#[test]
fn ascenders_bane_is_unplayable_unupgradable_and_exhausts_as_ethereal() {
    // Source: cards/curses/AscendersBane.java sets cost to -2, sets
    // isEthereal, and implements neither use() nor upgrade().
    let registry = global_registry();
    let bane = registry.get("AscendersBane").expect("Ascender's Bane is registered");
    assert_eq!(bane.cost, -2);
    assert!(bane.runtime_traits().unplayable);
    assert!(bane.runtime_traits().ethereal);
    assert!(registry.get("AscendersBane+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.relics.push("Runic Pyramid".to_string());
    engine.state.hand = make_deck(&["AscendersBane", "Strike"]);

    let bane_idx = engine
        .state
        .hand
        .iter()
        .position(|card| engine.card_registry.card_name(card.def_id) == "AscendersBane")
        .unwrap();
    assert!(!engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { card_idx, .. } if *card_idx == bane_idx)
    }));

    end_turn(&mut engine);

    assert_eq!(exhaust_prefix_count(&engine, "AscendersBane"), 1);
    assert_eq!(hand_count(&engine, "Strike"), 1);
}

#[test]
fn clumsy_is_unplayable_unupgradable_and_exhausts_as_ethereal() {
    // Source: cards/curses/Clumsy.java sets cost -2 and isEthereal, queues
    // ExhaustSpecificCardAction at end of turn, and leaves upgrade() empty.
    let registry = global_registry();
    let clumsy = registry.get("Clumsy").expect("Clumsy is registered");
    assert_eq!(clumsy.cost, -2);
    assert!(clumsy.runtime_traits().unplayable);
    assert!(clumsy.runtime_traits().ethereal);
    assert!(registry.get("Clumsy+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.relics.push("Runic Pyramid".to_string());
    engine.state.hand = make_deck(&["Clumsy", "Strike"]);

    let clumsy_idx = engine
        .state
        .hand
        .iter()
        .position(|card| engine.card_registry.card_name(card.def_id) == "Clumsy")
        .unwrap();
    assert!(!engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { card_idx, .. } if *card_idx == clumsy_idx)
    }));

    end_turn(&mut engine);

    assert_eq!(exhaust_prefix_count(&engine, "Clumsy"), 1);
    assert_eq!(hand_count(&engine, "Strike"), 1);
}
