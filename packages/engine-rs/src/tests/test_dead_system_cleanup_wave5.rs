#![cfg(test)]

// Java references:
// - decompiled/java-src/com/megacrit/cardcrawl/relics/BagOfPreparation.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/RingOfTheSnake.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/LetterOpener.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/BirdFacedUrn.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/MummifiedHand.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Duality.java

use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_without_start, engine_with_state, make_deck,
    play_on_enemy, play_self,
};

#[test]
fn dead_cleanup_wave5_runtime_opening_and_turn_relics_are_authoritative() {
    let mut engine = engine_without_start(
        make_deck(&[
            "Strike_R",
            "Strike_R",
            "Strike_R",
            "Strike_R",
            "Strike_R",
            "Defend_R",
            "Defend_R",
            "Defend_R",
            "Bash",
            "Inflame",
            "Pommel Strike",
            "Twin Strike",
        ]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.state.relics.extend([
        "Bag of Preparation".to_string(),
        "Ring of the Snake".to_string(),
    ]);
    engine.start_combat();

    assert_eq!(engine.state.player.status(sid::BAG_OF_PREP_DRAW), 0);
    assert_eq!(engine.state.hand.len(), 9);
}

#[test]
fn dead_cleanup_wave5_runtime_cardplay_and_endturn_relics_replace_helper_assertions() {
    let mut state = combat_state_with(
        make_deck(&["Defend_R", "Defend_R", "Defend_R", "Inflame", "Strike_R", "Defend_R", "Bash"]),
        vec![enemy_no_intent("JawWorm", 120, 120), enemy_no_intent("Cultist", 44, 44)],
        20,
    );
    state.relics.extend([
        "Letter Opener".to_string(),
        "Bird Faced Urn".to_string(),
        "Mummified Hand".to_string(),
        "Yang".to_string(),
    ]);
    let mut engine = engine_with_state(state);
    engine.state.player.hp = 70;
    engine.state.hand = make_deck(&["Defend_R", "Defend_R", "Defend_R", "Inflame", "Strike_R", "Defend_R", "Bash"]);
    let hp0 = engine.state.enemies[0].entity.hp;
    let hp1 = engine.state.enemies[1].entity.hp;

    assert!(play_self(&mut engine, "Defend_R"));
    assert!(play_self(&mut engine, "Defend_R"));
    assert!(play_self(&mut engine, "Defend_R"));
    assert_eq!(engine.state.enemies[0].entity.hp, hp0 - 5);
    assert_eq!(engine.state.enemies[1].entity.hp, hp1 - 5);
    assert!(play_self(&mut engine, "Inflame"));
    assert_eq!(engine.state.player.hp, 72);
    assert!(engine.state.hand.iter().any(|card| card.cost == 0));
    assert!(play_on_enemy(&mut engine, "Strike_R", 0));
    assert_eq!(engine.state.player.status(sid::LOSE_DEXTERITY), 1);
}
