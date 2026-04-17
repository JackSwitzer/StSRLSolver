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
    combat_state_with, enemy_no_intent, engine_with_state, engine_without_start, make_deck,
    play_on_enemy, play_self,
};

fn engine_without_start_with_relics(
    relics: &[&str],
    deck: &[&str],
    enemies: Vec<crate::state::EnemyCombatState>,
    energy: i32,
) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(make_deck(deck), enemies, energy);
    engine.state.relics = relics.iter().map(|id| (*id).to_string()).collect();
    engine
}

#[test]
fn opening_turn_draw_relics_follow_runtime_path() {
    let mut bag = engine_without_start_with_relics(
        &["Bag of Preparation"],
        &[
            "Strike_R",
            "Strike_R",
            "Strike_R",
            "Strike_R",
            "Strike_R",
            "Defend_R",
            "Defend_R",
            "Defend_R",
        ],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    bag.start_combat();
    assert_eq!(bag.state.player.status(sid::BAG_OF_PREP_DRAW), 0);
    assert_eq!(bag.state.hand.len(), 7);

    let mut ring = engine_without_start_with_relics(
        &["Ring of the Snake"],
        &[
            "Strike_G",
            "Strike_G",
            "Strike_G",
            "Strike_G",
            "Strike_G",
            "Defend_G",
            "Defend_G",
            "Defend_G",
        ],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    ring.start_combat();
    assert_eq!(ring.state.player.status(sid::BAG_OF_PREP_DRAW), 0);
    assert_eq!(ring.state.hand.len(), 7);

}

#[test]
fn card_play_relics_follow_runtime_path() {
    let mut opener_state = combat_state_with(
        make_deck(&["Defend_R", "Defend_R", "Defend_R"]),
        vec![enemy_no_intent("JawWorm", 40, 40), enemy_no_intent("Cultist", 44, 44)],
        20,
    );
    opener_state.relics.push("Letter Opener".to_string());
    let mut opener = engine_with_state(opener_state);
    opener.state.hand = make_deck(&["Defend_R", "Defend_R", "Defend_R"]);
    opener.state.draw_pile.clear();
    let hp0 = opener.state.enemies[0].entity.hp;
    let hp1 = opener.state.enemies[1].entity.hp;
    assert!(play_self(&mut opener, "Defend_R"));
    assert!(play_self(&mut opener, "Defend_R"));
    assert_eq!(opener.state.enemies[0].entity.hp, hp0);
    assert_eq!(opener.state.enemies[1].entity.hp, hp1);
    assert!(play_self(&mut opener, "Defend_R"));
    assert_eq!(opener.state.enemies[0].entity.hp, hp0 - 5);
    assert_eq!(opener.state.enemies[1].entity.hp, hp1 - 5);

    let mut urn_state = combat_state_with(
        make_deck(&["Inflame", "Defend_R"]),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        5,
    );
    urn_state.relics.push("Bird Faced Urn".to_string());
    let mut urn = engine_with_state(urn_state);
    urn.state.player.hp = 70;
    urn.state.hand = make_deck(&["Inflame", "Defend_R"]);
    assert!(play_self(&mut urn, "Inflame"));
    assert_eq!(urn.state.player.hp, 72);

    let mut mummified_state = combat_state_with(
        make_deck(&["Inflame", "Strike_R", "Defend_R"]),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        5,
    );
    mummified_state.relics.push("Mummified Hand".to_string());
    let mut mummified = engine_with_state(mummified_state);
    mummified.state.hand = make_deck(&["Inflame", "Strike_R", "Defend_R"]);
    assert!(play_self(&mut mummified, "Inflame"));
    assert!(mummified.state.hand.iter().any(|card| card.cost == 0));

    let mut yang_state = combat_state_with(
        make_deck(&["Strike_R", "Defend_R"]),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        5,
    );
    yang_state.relics.push("Yang".to_string());
    let mut yang = engine_with_state(yang_state);
    yang.state.hand = make_deck(&["Strike_R", "Defend_R"]);
    assert!(play_on_enemy(&mut yang, "Strike_R", 0));
    assert_eq!(yang.state.player.dexterity(), 1);
    assert_eq!(yang.state.player.status(sid::LOSE_DEXTERITY), 1);
}
