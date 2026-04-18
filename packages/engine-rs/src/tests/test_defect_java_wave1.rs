#![cfg(test)]

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Streamline.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/LiquidMemories.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/DistilledChaosPotion.java

use crate::actions::Action;
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, make_deck};

fn use_potion(engine: &mut crate::engine::CombatEngine, potion_idx: usize, target_idx: i32) {
    engine.execute_action(&Action::UsePotion {
        potion_idx,
        target_idx,
    });
}

fn streamline_costs(engine: &crate::engine::CombatEngine) -> Vec<i32> {
    engine
        .state
        .hand
        .iter()
        .chain(engine.state.draw_pile.iter())
        .chain(engine.state.discard_pile.iter())
        .filter(|card| {
            let name = engine.card_registry.card_name(card.def_id);
            name == "Streamline" || name == "Streamline+"
        })
        .map(|card| {
            if card.cost >= 0 {
                card.cost as i32
            } else {
                engine.card_registry.card_def_by_id(card.def_id).cost
            }
        })
        .collect()
}

#[test]
fn streamline_reduces_only_one_copy_instead_of_broadcasting_to_all_copies() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = vec![engine.card_registry.make_card("Streamline")];
    engine.state.draw_pile = vec![engine.card_registry.make_card("Streamline")];
    engine.state.discard_pile = vec![engine.card_registry.make_card("Streamline")];

    crate::tests::support::play_card(&mut engine, "Streamline", 0);

    let costs = streamline_costs(&engine);
    assert_eq!(costs.iter().filter(|&&cost| cost == 1).count(), 1);
    assert_eq!(costs.iter().filter(|&&cost| cost == 2).count(), 2);
    assert_eq!(
        engine
            .state
            .discard_pile
            .last()
            .copied()
            .map(|card| if card.cost >= 0 { card.cost as i32 } else { engine.card_registry.card_def_by_id(card.def_id).cost }),
        Some(1),
        "Java ReduceCostAction targets the played Streamline instance by UUID"
    );
}

#[test]
fn liquid_memories_returns_discard_cards_with_zero_cost() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.discard_pile = make_deck(&["Strike", "Bash"]);
    engine.state.potions[0] = "LiquidMemories".to_string();

    use_potion(&mut engine, 0, -1);
    engine.execute_action(&Action::Choose(1));

    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.card_registry.card_name(engine.state.hand[0].def_id), "Bash");
    assert_eq!(engine.state.hand[0].cost, 0);
    assert_eq!(engine.state.discard_pile.len(), 1);
}

#[test]
fn distilled_chaos_now_plays_top_draw_cards_through_the_runtime_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.draw_pile = make_deck(&["Strike", "Defend", "Bash", "Shrug It Off"]);
    engine.state.potions[0] = "DistilledChaos".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.card_registry.card_name(engine.state.hand[0].def_id), "Bash");
    assert_eq!(engine.state.draw_pile.len(), 0);
    assert_eq!(engine.state.player.block, 13);
    assert_eq!(engine.state.enemies[0].entity.hp, 34);
}
