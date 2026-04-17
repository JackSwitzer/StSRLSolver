#![cfg(test)]

use crate::actions::Action;
use crate::effects::runtime::EffectOwner;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, end_turn, enemy_no_intent, engine_with_state, make_deck, make_deck_n,
    play_on_enemy, play_self,
};
use crate::potions::defs::{potion_runtime_manual_activation_is_authoritative, potion_uses_runtime_manual_activation};

#[test]
fn dead_cleanup_wave1_runtime_authoritative_potions_are_explicit() {
    let ids = [
        "Block Potion",
        "Dexterity Potion",
        "Explosive Potion",
        "Fear Potion",
        "Fire Potion",
        "Poison Potion",
        "Strength Potion",
        "Weak Potion",
    ];

    for id in ids {
        assert!(
            potion_uses_runtime_manual_activation(id),
            "{id} should advertise runtime manual activation"
        );
        assert!(
            potion_runtime_manual_activation_is_authoritative(id),
            "{id} should no longer rely on helper-path-only evidence"
        );
    }
}

#[test]
fn dead_cleanup_wave1_relic_bundle_is_engine_path_authoritative() {
    // Orange Pellets: all three card types must be played in one turn before debuffs clear.
    let mut orange_state = combat_state_with(
        make_deck(&["Strike_R", "Defend_R", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 120, 120)],
        20,
    );
    orange_state.relics.push("OrangePellets".to_string());
    let mut orange_engine = engine_with_state(orange_state);
    orange_engine.state.hand = make_deck(&["Strike_R", "Defend_R", "Inflame"]);
    orange_engine.state.player.set_status(sid::WEAKENED, 2);
    orange_engine.state.player.set_status(sid::VULNERABLE, 2);
    assert!(play_on_enemy(&mut orange_engine, "Strike_R", 0));
    assert!(play_self(&mut orange_engine, "Defend_R"));
    assert_eq!(
        orange_engine.hidden_effect_value("OrangePellets", EffectOwner::PlayerRelic { slot: 0 }, 0),
        1
    );
    assert!(play_self(&mut orange_engine, "Inflame"));
    assert_eq!(orange_engine.state.player.status(sid::WEAKENED), 0);
    assert_eq!(orange_engine.state.player.status(sid::VULNERABLE), 0);

    // Pen Nib: the 10th attack is the only one that should fire the double-damage state.
    let mut nib_state = combat_state_with(
        make_deck_n("Strike_R", 10),
        vec![enemy_no_intent("JawWorm", 160, 160)],
        20,
    );
    nib_state.relics.push("Pen Nib".to_string());
    let mut nib_engine = engine_with_state(nib_state);
    nib_engine.state.hand = make_deck_n("Strike_R", 10);
    for _ in 0..9 {
        assert!(play_on_enemy(&mut nib_engine, "Strike_R", 0));
    }
    assert_eq!(nib_engine.state.player.status(sid::PEN_NIB_COUNTER), 9);
    let hp_before = nib_engine.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut nib_engine, "Strike_R", 0));
    assert_eq!(nib_engine.state.player.status(sid::PEN_NIB_COUNTER), 0);
    assert_eq!(nib_engine.state.enemies[0].entity.hp, hp_before - 12);

    // Velvet Choker: the seventh card remains illegal and the counter resets on end turn.
    let mut choke_state = combat_state_with(
        make_deck_n("Defend_R", 7),
        vec![enemy_no_intent("JawWorm", 120, 120)],
        20,
    );
    choke_state.relics.push("Velvet Choker".to_string());
    let mut choke_engine = engine_with_state(choke_state);
    choke_engine.state.hand = make_deck_n("Defend_R", 7);
    for _ in 0..6 {
        choke_engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });
    }
    assert_eq!(choke_engine.state.cards_played_this_turn, 6);
    let hand_before = choke_engine.state.hand.len();
    choke_engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });
    assert_eq!(choke_engine.state.hand.len(), hand_before);
    assert_eq!(choke_engine.state.cards_played_this_turn, 6);
    end_turn(&mut choke_engine);
    assert_eq!(
        choke_engine.hidden_effect_value("Velvet Choker", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}
