#![cfg(test)]

// Java references:
// - decompiled/java-src/com/megacrit/cardcrawl/relics/OddlySmoothStone.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/RedMask.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/PhilosopherStone.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/PureWater.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/CentennialPuzzle.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/SelfformingClay.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/RunicCube.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/RedSkull.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Sundial.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Abacus.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Melange.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/GremlinHorn.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/TheSpecimen.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/BurningBlood.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/BlackBlood.java
use crate::effects::runtime::EffectOwner;
use crate::engine::{ChoiceReason, CombatPhase};
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, end_turn, enemy_no_intent, engine_with_state, engine_without_start,
    force_player_turn, make_deck, make_deck_n,
};

#[test]
fn relic_wave12_runtime_combat_start_buffs_and_debuffs_match_canonical_runtime() {
    // OddlySmoothStone.java::atBattleStart applies exactly one Dexterity.
    let mut state = combat_state_with(
        Vec::new(),
        vec![
            enemy_no_intent("Cultist", 24, 24),
            enemy_no_intent("JawWorm", 40, 40),
        ],
        3,
    );
    state.relics = vec![
        "Oddly Smooth Stone".to_string(),
        "Red Mask".to_string(),
        "Philosopher's Stone".to_string(),
    ];

    let engine = engine_with_state(state);

    assert_eq!(engine.state.player.dexterity(), 1);
    assert!(engine
        .state
        .enemies
        .iter()
        .all(|enemy| enemy.entity.status(sid::WEAKENED) == 1));
    assert!(engine
        .state
        .enemies
        .iter()
        .all(|enemy| enemy.entity.strength() == 1));
}

#[test]
fn philosophers_stone_precedes_spawned_minion_power_at_equal_priority() {
    // SpawnMonsterAction calls onSpawnMonster first, where Philosopher's Stone
    // directly appends Strength, then queues ApplyPowerAction(MinionPower).
    // Both powers have priority 5, so Java's stable sort preserves this order.
    // Java: actions/common/SpawnMonsterAction.java:45-68 and
    // relics/PhilosopherStone.java:49-52.
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("GremlinLeader", 140, 140)],
        3,
    );
    engine.state.relics.push("Philosopher's Stone".to_string());
    engine.add_spawned_minion(enemy_no_intent("GremlinFat", 14, 14));

    let spawned = engine.state.enemies.last().unwrap();
    assert_eq!(spawned.entity.strength(), 1);
    assert!(spawned.is_minion());
    assert_eq!(
        spawned
            .entity
            .ordered_status_ids()
            .into_iter()
            .filter(|status| matches!(*status, sid::STRENGTH | sid::MINION_POWER))
            .collect::<Vec<_>>(),
        [sid::STRENGTH, sid::MINION_POWER]
    );
}

#[test]
fn red_skull_uses_direct_add_only_at_battle_start() {
    // RedSkull.atBattleStart calls player.addPower directly; onBloodied uses
    // ApplyPowerAction and therefore restores sorted priority order.
    // Java: RedSkull.java:38-41, 54-57.
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.player.hp = state.player.max_hp / 2;
    state.player.set_status_direct(sid::TOOLS_OF_THE_TRADE, 1);
    state.relics.push("Red Skull".to_string());

    let mut engine = engine_with_state(state);
    let relevant_order = |engine: &crate::engine::CombatEngine| {
        engine
            .state
            .player
            .ordered_status_ids()
            .into_iter()
            .filter(|status| matches!(*status, sid::TOOLS_OF_THE_TRADE | sid::STRENGTH))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        relevant_order(&engine),
        [sid::TOOLS_OF_THE_TRADE, sid::STRENGTH]
    );

    engine.heal_player(2);
    assert_eq!(engine.state.player.strength(), 0);
    engine.player_lose_hp(3);
    assert_eq!(engine.state.player.strength(), 3);
    assert_eq!(
        relevant_order(&engine),
        [sid::STRENGTH, sid::TOOLS_OF_THE_TRADE]
    );
}

#[test]
fn relic_wave12_runtime_combat_start_temp_cards_match_canonical_runtime() {
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics = vec!["PureWater".to_string()];

    let engine = engine_with_state(state);

    let names: Vec<_> = engine
        .state
        .hand
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id))
        .collect();
    assert_eq!(names.iter().filter(|name| **name == "Miracle").count(), 1);
    assert_eq!(engine.state.hand.len(), 1);
}

#[test]
fn relic_wave12_runtime_hp_loss_families_match_canonical_runtime() {
    let mut state = combat_state_with(
        make_deck(&["Strike"; 10]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.relics = vec![
        "Centennial Puzzle".to_string(),
        "Self Forming Clay".to_string(),
        "Runic Cube".to_string(),
        "Red Skull".to_string(),
    ];
    state.player.hp = 50;

    let mut engine = engine_with_state(state);
    engine.state.hand.clear();
    engine.state.draw_pile = make_deck(&["Strike", "Defend", "Bash", "Strike", "Defend", "Strike"]);

    engine.player_lose_hp(11);

    assert_eq!(engine.state.hand.len(), 4);
    assert_eq!(engine.state.player.status(sid::NEXT_TURN_BLOCK), 3);
    assert_eq!(engine.state.player.strength(), 3);
    assert_eq!(
        engine.hidden_effect_value("Centennial Puzzle", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
    assert_eq!(
        engine.hidden_effect_value("Red Skull", EffectOwner::PlayerRelic { slot: 3 }, 0),
        1
    );

    engine.player_lose_hp(4);

    assert_eq!(engine.state.hand.len(), 5);

    // RedSkull.java::onNotBloodied removes its three Strength when healing
    // crosses above half HP, regardless of the relic's inventory slot.
    engine.heal_player(6);
    assert_eq!(engine.state.player.hp, 41);
    assert_eq!(engine.state.player.strength(), 0);
    assert_eq!(
        engine.hidden_effect_value("Red Skull", EffectOwner::PlayerRelic { slot: 3 }, 0),
        0
    );
}

#[test]
fn runic_cube_draws_one_card_per_positive_hp_loss_event() {
    // RunicCube.java::wasHPLost checks only that combat is active and the
    // reported damageAmount is positive, then queues exactly one DrawCardAction.
    // The amount lost does not change the draw count.
    let mut engine = engine_without_start(
        make_deck(&["Strike", "Defend", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.state.relics = vec!["Runic Cube".to_string()];
    engine.rebuild_effect_runtime();
    engine.state.hand.clear();
    engine.state.draw_pile = make_deck(&["Strike", "Defend", "Bash"]);

    engine.player_lose_hp(1);
    assert_eq!(engine.state.hand.len(), 1);
    engine.player_lose_hp(17);
    assert_eq!(engine.state.hand.len(), 2);
    engine.player_lose_hp(0);
    assert_eq!(engine.state.hand.len(), 2);
}

#[test]
fn self_forming_clay_stacks_three_block_per_hp_loss_for_the_next_turn() {
    // SelfFormingClay.java::wasHPLost applies three NextTurnBlockPower for
    // every positive loss event. NextTurnBlockPower.java grants the stacked
    // amount at turn start and then removes itself.
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.relics = vec!["Self Forming Clay".to_string()];
    engine.rebuild_effect_runtime();
    engine.state.player.block = 20;

    engine.player_lose_hp(1);
    engine.player_lose_hp(12);
    assert_eq!(engine.state.player.status(sid::NEXT_TURN_BLOCK), 6);

    engine.state.skip_enemy_turn = true;
    end_turn(&mut engine);
    assert_eq!(engine.state.player.block, 6);
    assert_eq!(engine.state.player.status(sid::NEXT_TURN_BLOCK), 0);
}

#[test]
fn relic_wave12_runtime_shuffle_and_enemy_death_families_match_canonical_runtime() {
    let mut shuffle = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    shuffle.state.relics = vec!["Sundial".to_string(), "TheAbacus".to_string()];
    shuffle.rebuild_effect_runtime();

    for expected_shuffle in [1, 2, 0] {
        shuffle.state.hand.clear();
        shuffle.state.draw_pile.clear();
        shuffle.state.discard_pile = make_deck(&["Strike"]);
        shuffle.draw_cards(1);
        assert_eq!(
            shuffle.hidden_effect_value("Sundial", EffectOwner::PlayerRelic { slot: 0 }, 0),
            expected_shuffle
        );
    }
    assert_eq!(shuffle.state.energy, 5);
    assert_eq!(shuffle.state.player.block, 18);

    let mut state = combat_state_with(
        make_deck_n("Strike", 3),
        vec![
            enemy_no_intent("JawWorm", 10, 10),
            enemy_no_intent("Cultist", 30, 30),
        ],
        3,
    );
    state.relics = vec!["Gremlin Horn".to_string(), "The Specimen".to_string()];
    let mut engine = engine_with_state(state);
    engine.state.hand.clear();
    engine.state.draw_pile = make_deck(&["Strike"]);
    engine.state.enemies[0].entity.set_status(sid::POISON, 5);
    engine.state.enemies[0].entity.hp = 0;

    engine.finalize_enemy_death(0);

    assert_eq!(engine.state.energy, 4);
    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.state.enemies[1].entity.status(sid::POISON), 5);
}

#[test]
fn the_abacus_grants_exactly_six_block_on_one_shuffle() {
    // Source: reference/extracted/methods/relic/Abacus.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    engine.state.relics = vec!["TheAbacus".to_string()];
    engine.rebuild_effect_runtime();
    engine.state.draw_pile.clear();
    engine.state.discard_pile = make_deck(&["Strike"]);

    engine.draw_cards(1);

    assert_eq!(engine.state.player.block, 6);
}

#[test]
fn melange_scries_after_the_shuffle_draw_and_composes_with_golden_eye() {
    // Melange.java queues ScryAction(3) from onShuffle. EmptyDeckShuffleAction
    // completes between the split draw actions, while Melange's addToBot Scry
    // runs only after the requested cards have been drawn.
    let shuffled_cards = [
        "Strike",
        "Defend",
        "Bash",
        "Eruption",
        "Vigilance",
        "Scrawl",
        "EmptyBody",
    ];

    let mut baseline =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    baseline.state.discard_pile = make_deck(&shuffled_cards);
    baseline.draw_cards(2);
    let baseline_hand: Vec<_> = baseline
        .state
        .hand
        .iter()
        .map(|card| baseline.card_registry.card_name(card.def_id))
        .collect();

    let mut melange = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    melange.state.relics.push("Melange".to_string());
    melange.rebuild_effect_runtime();
    melange.state.discard_pile = make_deck(&shuffled_cards);
    melange.draw_cards(2);
    let melange_hand: Vec<_> = melange
        .state
        .hand
        .iter()
        .map(|card| melange.card_registry.card_name(card.def_id))
        .collect();
    assert_eq!(melange_hand, baseline_hand);
    assert_eq!(melange.phase, CombatPhase::AwaitingChoice);
    let choice = melange.choice.as_ref().expect("Melange Scry choice");
    assert_eq!(choice.reason, ChoiceReason::Scry);
    assert_eq!(choice.options.len(), 3);

    let mut golden = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    golden.state.relics = vec!["Melange".to_string(), "GoldenEye".to_string()];
    golden.rebuild_effect_runtime();
    golden.state.discard_pile = make_deck(&shuffled_cards);
    golden.draw_cards(2);
    let choice = golden
        .choice
        .as_ref()
        .expect("GoldenEye Melange Scry choice");
    assert_eq!(choice.options.len(), 5);
    assert_eq!(choice.max_picks, 5);
}

#[test]
fn relic_wave12_runtime_victory_families_match_canonical_runtime() {
    let mut burn_state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    burn_state.relics = vec!["Burning Blood".to_string()];
    burn_state.player.hp = 60;
    let mut burn_engine = engine_with_state(burn_state);
    burn_engine.state.enemies[0].entity.hp = 0;

    burn_engine.finalize_enemy_death(0);
    burn_engine.check_combat_end();

    assert!(burn_engine.state.combat_over);
    assert!(burn_engine.state.player_won);
    assert_eq!(burn_engine.state.player.hp, 66);

    // BurningBlood.java guards its six-point heal with currentHealth > 0;
    // simultaneous zero player/enemy HP must not turn the relic into a revive.
    let mut burn_zero_state =
        combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    burn_zero_state.relics = vec!["Burning Blood".to_string()];
    burn_zero_state.player.hp = 0;
    burn_zero_state.enemies[0].entity.hp = 0;
    let mut burn_zero_engine = engine_with_state(burn_zero_state);
    burn_zero_engine.check_combat_end();
    assert_eq!(burn_zero_engine.state.player.hp, 0);

    let mut black_state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    black_state.relics = vec!["Black Blood".to_string()];
    black_state.player.hp = 60;
    let mut black_engine = engine_with_state(black_state);
    black_engine.state.enemies[0].entity.hp = 0;

    black_engine.finalize_enemy_death(0);
    black_engine.check_combat_end();

    assert!(black_engine.state.combat_over);
    assert!(black_engine.state.player_won);
    assert_eq!(black_engine.state.player.hp, 72);

    // BlackBlood.java guards its twelve-point heal with currentHealth > 0;
    // simultaneous zero player/enemy HP must not turn the relic into a revive.
    let mut zero_state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    zero_state.relics = vec!["Black Blood".to_string()];
    zero_state.player.hp = 0;
    zero_state.enemies[0].entity.hp = 0;
    let mut zero_engine = engine_with_state(zero_state);
    zero_engine.check_combat_end();
    assert_eq!(zero_engine.state.player.hp, 0);
}
