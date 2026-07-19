#![cfg(test)]

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Streamline.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Rampage.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/blue/SteamBarrier.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/green/GlassKnife.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/blue/GeneticAlgorithm.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/RitualDagger.java

use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, make_deck, play_on_enemy, play_self};
use crate::actions::Action;

fn effective_cost(engine: &crate::engine::CombatEngine, card: crate::combat_types::CardInstance) -> i32 {
    if card.cost >= 0 {
        card.cost as i32
    } else {
        engine.card_registry.card_def_by_id(card.def_id).cost
    }
}

fn effective_misc_or(
    engine: &crate::engine::CombatEngine,
    card: crate::combat_types::CardInstance,
    base: i32,
) -> i32 {
    if card.misc >= 0 {
        card.misc as i32
    } else if base >= 0 {
        base
    } else {
        let def = engine.card_registry.card_def_by_id(card.def_id);
        if def.base_damage >= 0 {
            def.base_damage
        } else {
            def.base_block
        }
    }
}

#[test]
fn streamline_reduces_the_played_instance_cost_only() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = vec![engine.card_registry.make_card("Streamline")];
    engine.state.draw_pile = vec![engine.card_registry.make_card("Streamline")];
    engine.state.discard_pile = vec![engine.card_registry.make_card("Streamline")];

    assert!(play_on_enemy(&mut engine, "Streamline", 0));

    let discard_last = engine.state.discard_pile.last().copied().expect("played card should discard");
    assert_eq!(effective_cost(&engine, discard_last), 1);
    assert_eq!(effective_cost(&engine, engine.state.draw_pile[0]), 2);
    assert_eq!(effective_cost(&engine, engine.state.discard_pile[0]), 2);
}

#[test]
fn rampage_only_scales_the_played_copy() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    ));
    engine.state.hand = vec![engine.card_registry.make_card("Rampage")];
    engine.state.draw_pile = vec![engine.card_registry.make_card("Rampage")];
    engine.state.discard_pile = vec![engine.card_registry.make_card("Rampage")];

    assert!(play_on_enemy(&mut engine, "Rampage", 0));

    let played = engine.state.discard_pile.last().copied().expect("played Rampage should discard");
    assert_eq!(effective_misc_or(&engine, played, 8), 13);
    assert_eq!(engine.state.draw_pile[0].misc, -1);
    assert_eq!(engine.state.discard_pile[0].misc, -1);
}

#[test]
fn steam_barrier_only_reduces_the_played_copy_block() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = vec![engine.card_registry.make_card("Steam")];
    engine.state.draw_pile = vec![engine.card_registry.make_card("Steam")];
    engine.state.discard_pile = vec![engine.card_registry.make_card("Steam")];

    assert!(play_self(&mut engine, "Steam"));

    let played = engine.state.discard_pile.last().copied().expect("played Steam Barrier should discard");
    assert_eq!(effective_misc_or(&engine, played, 6), 5);
    assert_eq!(engine.state.draw_pile[0].misc, -1);
    assert_eq!(engine.state.discard_pile[0].misc, -1);
}

#[test]
fn glass_knife_only_reduces_the_played_copy_damage() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 80, 80)],
        3,
    ));
    engine.state.hand = vec![engine.card_registry.make_card("Glass Knife")];
    engine.state.draw_pile = vec![engine.card_registry.make_card("Glass Knife")];
    engine.state.discard_pile = vec![engine.card_registry.make_card("Glass Knife")];

    assert!(play_on_enemy(&mut engine, "Glass Knife", 0));

    let played = engine.state.discard_pile.last().copied().expect("played Glass Knife should discard");
    assert_eq!(effective_misc_or(&engine, played, 8), 6);
    assert_eq!(engine.state.draw_pile[0].misc, -1);
    assert_eq!(engine.state.discard_pile[0].misc, -1);
}

#[test]
fn genetic_algorithm_and_ritual_dagger_only_scale_the_played_copy() {
    let mut genetic_engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    genetic_engine.state.hand = vec![genetic_engine.card_registry.make_card("Genetic Algorithm")];
    genetic_engine.state.draw_pile = vec![genetic_engine.card_registry.make_card("Genetic Algorithm")];
    genetic_engine.state.discard_pile = vec![genetic_engine.card_registry.make_card("Genetic Algorithm")];

    assert!(play_self(&mut genetic_engine, "Genetic Algorithm"));

    let played_genetic = genetic_engine
        .state
        .exhaust_pile
        .last()
        .copied()
        .expect("played Genetic Algorithm should exhaust");
    assert_eq!(effective_misc_or(&genetic_engine, played_genetic, 1), 3);
    assert_eq!(genetic_engine.state.draw_pile[0].misc, -1);
    assert_eq!(genetic_engine.state.discard_pile[0].misc, -1);

    let mut ritual_engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 15, 15)],
        3,
    ));
    ritual_engine.state.hand = vec![ritual_engine.card_registry.make_card("RitualDagger")];
    ritual_engine.state.draw_pile = vec![ritual_engine.card_registry.make_card("RitualDagger")];
    ritual_engine.state.discard_pile = vec![ritual_engine.card_registry.make_card("RitualDagger")];

    assert!(play_on_enemy(&mut ritual_engine, "RitualDagger", 0));

    let played_ritual = ritual_engine
        .state
        .exhaust_pile
        .last()
        .copied()
        .expect("played Ritual Dagger should exhaust");
    assert_eq!(effective_misc_or(&ritual_engine, played_ritual, 15), 18);
    assert_eq!(ritual_engine.state.draw_pile[0].misc, -1);
    assert_eq!(ritual_engine.state.discard_pile[0].misc, -1);
}

#[test]
fn duplicate_equal_genetic_algorithms_update_the_matching_master_uuid_after_restore() {
    // IncreaseMiscAction matches AbstractCard.uuid, not the first card with the
    // same definition and misc value. This matters when exact duplicates have
    // been reordered. Java: actions/defect/IncreaseMiscAction.java.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Genetic Algorithm", "Genetic Algorithm"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.master_deck[0].misc = 7;
    engine.state.master_deck[1].misc = 7;
    let first = engine.state.master_deck[0];
    let second = engine.state.master_deck[1];
    assert_ne!(first.instance_id, second.instance_id);
    engine.state.hand = vec![first, second];
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    let encoded = serde_json::to_string(&engine).unwrap();
    let mut restored: crate::engine::CombatEngine = serde_json::from_str(&encoded).unwrap();
    restored.execute_action(&Action::PlayCard {
        card_idx: 1,
        target_idx: -1,
    });

    assert_eq!(restored.state.master_deck[0].misc, 7);
    assert_eq!(restored.state.master_deck[1].misc, 9);
    assert_eq!(restored.state.master_deck[1].instance_id, second.instance_id);
}

#[test]
fn dual_wield_ritual_dagger_copy_kill_does_not_scale_the_owned_master_card() {
    // DualWieldAction -> MakeTempCardInHandAction uses
    // makeStatEquivalentCopy(), which receives a fresh UUID. A copied Ritual
    // Dagger can scale itself on kill but must not match the owned master card.
    // Java: actions/unique/DualWieldAction.java,
    // actions/common/MakeTempCardInHandAction.java, AbstractCard.java:819.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["RitualDagger"]),
        vec![enemy_no_intent("JawWorm", 15, 15)],
        3,
    ));
    let owned = engine.state.master_deck[0];
    engine.state.hand = vec![owned];
    engine.state.draw_pile.clear();
    engine.add_dual_wield_copies(owned, 1);
    let copy = engine.state.hand[1];
    assert_ne!(copy.instance_id, owned.instance_id);

    engine.execute_action(&Action::PlayCard {
        card_idx: 1,
        target_idx: 0,
    });

    assert_eq!(engine.state.master_deck[0].misc, -1);
    assert_eq!(engine.state.exhaust_pile[0].misc, 18);
}

#[test]
fn generated_genetic_algorithm_copies_have_fresh_ids_and_do_not_scale_master() {
    // Discovery-style previews and every selected stat copy are distinct Java
    // card objects with fresh UUIDs. Generated copies therefore cannot match
    // an owned Genetic Algorithm in IncreaseMiscAction.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Genetic Algorithm"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    let owned = engine.state.master_deck[0];
    engine.state.hand.clear();
    engine.state.draw_pile.clear();
    let preview = engine.fresh_stat_copy(engine.card_registry.make_card("Genetic Algorithm"));
    engine.begin_discovery_choice(
        vec![crate::engine::ChoiceOption::GeneratedCard(preview)],
        1,
        1,
        3,
        crate::effects::declarative::GeneratedCostRule::Base,
    );
    engine.execute_action(&Action::Choose(0));

    let generated_ids: std::collections::HashSet<_> = engine
        .state
        .hand
        .iter()
        .map(|card| card.instance_id)
        .collect();
    assert_eq!(generated_ids.len(), 3);
    assert!(!generated_ids.contains(&owned.instance_id));
    engine.execute_action(&Action::PlayCard {
        card_idx: 0,
        target_idx: -1,
    });
    assert_eq!(engine.state.master_deck[0].misc, -1);
}

#[test]
fn omniscience_extra_ritual_dagger_kill_does_not_scale_the_owned_master_card() {
    // Omniscience queues the selected original once, then queues
    // makeStatEquivalentCopy() with purgeOnUse for every extra play. If only
    // the extra copy kills, RitualDaggerAction must not match masterDeck UUID.
    // Java: actions/watcher/OmniscienceAction.java:44-50.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Omniscience", "RitualDagger"]),
        vec![enemy_no_intent("JawWorm", 25, 25)],
        4,
    ));
    let omniscience = engine
        .state
        .master_deck
        .iter()
        .find(|card| engine.card_registry.card_name(card.def_id) == "Omniscience")
        .copied()
        .unwrap();
    let dagger = engine
        .state
        .master_deck
        .iter()
        .find(|card| engine.card_registry.card_name(card.def_id) == "RitualDagger")
        .copied()
        .unwrap();
    engine.state.hand = vec![omniscience];
    engine.state.draw_pile = vec![dagger];
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Omniscience"));
    engine.execute_action(&Action::Choose(0));

    let master_dagger = engine
        .state
        .master_deck
        .iter()
        .find(|card| card.instance_id == dagger.instance_id)
        .unwrap();
    assert_eq!(master_dagger.misc, -1);
    assert!(engine.state.enemies[0].entity.is_dead());
}

#[test]
fn ritual_dagger_kill_persists_to_master_deck_but_minion_kill_does_not_scale() {
    // RitualDaggerAction raises misc on the matching master-deck UUID and all
    // same-UUID combat instances only when the target dies without halfDead or
    // MinionPower. Ritual Dagger+ raises by five because its upgrade changes
    // magicNumber from three to five.
    // Sources: cards/colorless/RitualDagger.java,
    // actions/unique/RitualDaggerAction.java, and
    // helpers/GetAllInBattleInstances.java.
    let mut persistent = engine_with_state(combat_state_with(
        make_deck(&["RitualDagger+", "Strike"]),
        vec![enemy_no_intent("JawWorm", 15, 15)],
        3,
    ));
    persistent.state.hand.retain(|card| {
        persistent.card_registry.card_name(card.def_id) == "RitualDagger+"
    });

    assert!(play_on_enemy(&mut persistent, "RitualDagger+", 0));

    let exhausted = persistent
        .state
        .exhaust_pile
        .iter()
        .find(|card| persistent.card_registry.card_name(card.def_id) == "RitualDagger+")
        .copied()
        .expect("played Ritual Dagger+ should exhaust");
    let master = persistent
        .state
        .master_deck
        .iter()
        .find(|card| persistent.card_registry.card_name(card.def_id) == "RitualDagger+")
        .copied()
        .expect("persistent Ritual Dagger+ in master deck");
    assert_eq!(effective_misc_or(&persistent, exhausted, 15), 20);
    assert_eq!(effective_misc_or(&persistent, master, 15), 20);

    let mut minion = enemy_no_intent("TorchHead", 15, 15);
    minion.is_minion = true;
    let mut ignored = engine_with_state(combat_state_with(
        make_deck(&["RitualDagger"]),
        vec![minion],
        3,
    ));
    assert!(play_on_enemy(&mut ignored, "RitualDagger", 0));

    assert_eq!(ignored.state.exhaust_pile[0].misc, -1);
    assert_eq!(ignored.state.master_deck[0].misc, -1);
}
