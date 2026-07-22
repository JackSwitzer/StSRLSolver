#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/MeatOnTheBone.java

use crate::actions::Action;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with_state, make_deck, play_self,
};

#[test]
fn relic_wave14_meat_on_the_bone_heals_on_victory_at_half_or_below() {
    // MeatOnTheBone.java::onTrigger uses an inclusive maxHealth / 2.0 check
    // and heals exactly 12 while currentHealth remains positive.
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    state.relics.push("Meat on the Bone".to_string());
    state.player.hp = 40;
    let mut engine = engine_with_state(state);
    engine.state.enemies[0].entity.hp = 0;

    engine.finalize_enemy_death(0);
    engine.check_combat_end();

    assert!(engine.state.combat_over);
    assert!(engine.state.player_won);
    assert_eq!(engine.state.player.hp, 52);
}

#[test]
fn relic_wave14_meat_on_the_bone_does_not_heal_above_half() {
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    state.relics.push("Meat on the Bone".to_string());
    state.player.hp = 60;
    let mut engine = engine_with_state(state);
    engine.state.enemies[0].entity.hp = 0;

    engine.finalize_enemy_death(0);
    engine.check_combat_end();

    assert!(engine.state.combat_over);
    assert!(engine.state.player_won);
    assert_eq!(engine.state.player.hp, 60);
}

#[test]
fn meat_on_the_bone_precedes_owned_on_victory_relic_order() {
    // AbstractRoom.endBattle calls MeatOnTheBone.onTrigger before
    // AbstractPlayer.onVictory iterates owned relics. Burning Blood therefore
    // cannot heal the player above half before Meat checks its threshold.
    // Java: AbstractRoom.java::endBattle and AbstractPlayer.java::onVictory.
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    state
        .relics
        .extend(["Burning Blood".to_string(), "Meat on the Bone".to_string()]);
    state.player.hp = 38;
    let mut engine = engine_with_state(state);
    engine.state.enemies[0].entity.hp = 0;

    engine.finalize_enemy_death(0);
    engine.check_combat_end();

    assert!(engine.state.player_won);
    assert_eq!(
        engine.state.player.hp, 56,
        "Meat 12 must precede Burning Blood 6"
    );
}

#[test]
fn medical_kit_makes_status_cards_playable_for_free_and_exhausts_them() {
    // MedicalKit.java::onUseCard marks STATUS cards and their UseCardAction as
    // exhausting. The relic description supplies the otherwise-unplayable permission.
    let mut state = combat_state_with(
        make_deck(&["Wound"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.relics.push("Medical Kit".to_string());
    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Wound"]);
    engine.state.draw_pile.clear();

    assert!(engine.get_legal_actions().iter().any(|action| {
        matches!(
            action,
            Action::PlayCard {
                card_idx: 0,
                target_idx: -1
            }
        )
    }));
    assert!(play_self(&mut engine, "Wound"));
    assert!(engine.state.hand.is_empty());
    assert_eq!(engine.state.exhaust_pile.len(), 1);
    assert_eq!(
        engine
            .card_registry
            .card_name(engine.state.exhaust_pile[0].def_id),
        "Wound"
    );
}
