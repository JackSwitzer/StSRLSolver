#![cfg(test)]

// Java oracle references:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Streamline.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Chaos.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Fission.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Reboot.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Barrage.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/LiquidMemories.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/DistilledChaosPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/CrackedCore.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/FrozenCore.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/EmotionChip.java

use crate::actions::Action;
use crate::combat_hooks;
use crate::combat_types::CardInstance;
use crate::orbs::OrbType;
use crate::tests::support::{
    combat_state_with, enemy, enemy_no_intent, engine_with_state, engine_without_start,
    force_player_turn, make_deck, play_on_enemy, play_self,
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

fn effective_cost(engine: &crate::engine::CombatEngine, card: CardInstance) -> i32 {
    if card.cost >= 0 {
        card.cost as i32
    } else {
        engine.card_registry.card_def_by_id(card.def_id).cost
    }
}

#[test]
fn orb_wave1_streamline_reduces_only_the_played_instance_cost() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = vec![engine.card_registry.make_card("Streamline")];
    engine.state.draw_pile = vec![engine.card_registry.make_card("Streamline")];
    engine.state.discard_pile = vec![engine.card_registry.make_card("Streamline")];

    assert!(play_on_enemy(&mut engine, "Streamline", 0));

    let played = engine
        .state
        .discard_pile
        .last()
        .copied()
        .expect("played Streamline should land at the end of discard");
    assert_eq!(effective_cost(&engine, played), 1);
    assert_eq!(effective_cost(&engine, engine.state.draw_pile[0]), 2);
    assert_eq!(effective_cost(&engine, engine.state.discard_pile[0]), 2);
}

#[test]
fn orb_wave1_chaos_and_barrage_follow_java_orb_count_behavior() {
    let mut chaos = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    force_player_turn(&mut chaos);
    chaos.init_defect_orbs(3);
    chaos.state.hand = make_deck(&["Chaos+"]);
    assert!(play_self(&mut chaos, "Chaos+"));
    assert_eq!(chaos.state.orb_slots.occupied_count(), 2);

    let mut barrage = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    force_player_turn(&mut barrage);
    barrage.init_defect_orbs(3);
    barrage.channel_orb(OrbType::Lightning);
    barrage.channel_orb(OrbType::Frost);
    barrage.channel_orb(OrbType::Dark);
    barrage.state.hand = make_deck(&["Barrage"]);
    assert!(play_on_enemy(&mut barrage, "Barrage", 0));
    assert_eq!(barrage.state.enemies[0].entity.hp, 48);
}

#[test]
fn orb_wave1_fission_variants_match_remove_vs_evoke_behavior() {
    let mut fission = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 80, 80)],
        3,
    );
    force_player_turn(&mut fission);
    fission.init_defect_orbs(3);
    fission.channel_orb(OrbType::Lightning);
    fission.channel_orb(OrbType::Frost);
    fission.channel_orb(OrbType::Dark);
    fission.state.hand = make_deck(&["Fission"]);
    fission.state.draw_pile = make_deck(&["Strike_B", "Defend_B", "Zap", "Dualcast"]);
    assert!(play_self(&mut fission, "Fission"));
    assert_eq!(fission.state.orb_slots.occupied_count(), 0);
    assert_eq!(fission.state.energy, 6);
    assert_eq!(fission.state.hand.len(), 3);

    let mut fission_plus = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 80, 80)],
        3,
    );
    force_player_turn(&mut fission_plus);
    fission_plus.init_defect_orbs(3);
    fission_plus.channel_orb(OrbType::Lightning);
    fission_plus.channel_orb(OrbType::Frost);
    fission_plus.channel_orb(OrbType::Dark);
    fission_plus.state.hand = make_deck(&["Fission+"]);
    fission_plus.state.draw_pile = make_deck(&["Strike_B", "Defend_B", "Zap", "Dualcast"]);
    let hp_before = fission_plus.state.enemies[0].entity.hp;
    let block_before = fission_plus.state.player.block;
    assert!(play_self(&mut fission_plus, "Fission+"));
    assert_eq!(fission_plus.state.orb_slots.occupied_count(), 0);
    assert_eq!(fission_plus.state.energy, 6);
    assert_eq!(fission_plus.state.hand.len(), 3);
    assert_eq!(fission_plus.state.enemies[0].entity.hp, hp_before - 14);
    assert_eq!(fission_plus.state.player.block, block_before + 5);
}

#[test]
fn orb_wave1_distilled_chaos_plays_top_cards_in_draw_order_for_free() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_B", "Defend_B", "Zap"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.init_defect_orbs(3);
    engine.state.hand.clear();
    engine.state.draw_pile = make_deck(&["Strike_B", "Defend_B", "Zap"]);
    engine.state.potions[0] = "DistilledChaos".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(engine.state.draw_pile.len(), 0);
    assert!(engine.state.hand.is_empty());
    assert_eq!(engine.state.player.block, 5);
    assert_eq!(engine.state.enemies[0].entity.hp, 34);
    assert_eq!(engine.state.orb_slots.occupied_count(), 1);
}

#[test]
fn orb_wave1_liquid_memories_returns_top_discard_cards_at_cost_zero_in_order() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_B"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.discard_pile = make_deck(&["Strike_B", "Defend_B", "Zap"]);
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "LiquidMemories".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(hand_names(&engine), vec!["Zap", "Defend_B"]);
    assert_eq!(engine.state.hand[0].cost, 0);
    assert_eq!(engine.state.hand[1].cost, 0);
    assert_eq!(engine.state.discard_pile.len(), 1);
}

#[test]
fn orb_wave1_cracked_core_and_frozen_core_follow_current_runtime_path() {
    let mut cracked = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    cracked.init_defect_orbs(3);
    cracked.state.relics.push("Cracked Core".to_string());
    cracked.start_combat();
    assert!(cracked.state.orb_slots.slots.iter().any(|orb| orb.orb_type == OrbType::Lightning));

    let mut frozen = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    frozen.init_defect_orbs(3);
    frozen.state.relics.push("FrozenCore".to_string());
    frozen.start_combat();
    frozen.channel_orb(OrbType::Lightning);
    let occupied_before = frozen.state.orb_slots.occupied_count();
    frozen.execute_action(&Action::EndTurn);
    assert_eq!(frozen.state.orb_slots.occupied_count(), occupied_before + 1);
    assert!(frozen.state.orb_slots.slots.iter().any(|orb| orb.orb_type == OrbType::Frost));
}

#[test]
#[ignore = "blocked on engine.rs: Emotion Chip currently fires passive immediately in player_lose_hp instead of pulsing for next turn start"]
fn orb_wave1_emotion_chip_should_wait_until_next_turn_start_like_java() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy("JawWorm", 40, 40, 1, 5, 1)],
        3,
    );
    engine.init_defect_orbs(1);
    engine.state.relics.push("Emotion Chip".to_string());
    engine.state.enemies[0].first_turn = false;
    engine.channel_orb(OrbType::Lightning);
    engine.start_combat();

    let hp_before = engine.state.enemies[0].entity.hp;
    combat_hooks::do_enemy_turns(&mut engine);

    assert_eq!(engine.state.enemies[0].entity.hp, hp_before, "Java Emotion Chip should not trigger the passive until next turn start");
}

#[test]
fn orb_wave1_reboot_should_shuffle_hand_and_discard_before_drawing() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Reboot", "Strike_B", "Defend_B"]);
    engine.state.discard_pile = make_deck(&["Zap", "Dualcast", "Cold Snap"]);
    assert!(play_self(&mut engine, "Reboot"));

    assert_eq!(engine.state.hand.len(), 4);
    assert_eq!(engine.state.exhaust_pile.len(), 1);
    assert_eq!(
        engine.card_registry.card_name(engine.state.exhaust_pile[0].def_id),
        "Reboot"
    );
    assert_eq!(engine.state.discard_pile.len(), 0);
}

#[test]
#[ignore = "blocked on missing discard-choice primitive: Java Liquid Memories lets the player choose any discard card, while current runtime returns top discard cards deterministically"]
fn orb_wave1_liquid_memories_should_support_java_choice_selection() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_B"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.discard_pile = make_deck(&["Strike_B", "Defend_B", "Zap"]);
    engine.state.potions[0] = "LiquidMemories".to_string();
    use_potion(&mut engine, 0, -1);
}
