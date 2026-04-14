#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/BagOfPreparation.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/RingOfTheSnake.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/MutagenicStrength.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/Kunai.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/Shuriken.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/LetterOpener.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/BirdFacedUrn.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/Yang.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/EmotionChip.java

use crate::effects::runtime::EffectOwner;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with_state, engine_without_start, make_deck_n,
    play_on_enemy, play_self,
};

fn engine_without_start_with_relics(
    relics: &[&str],
    deck: usize,
) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(
        make_deck_n("Strike_R", deck),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.state.relics = relics.iter().map(|id| (*id).to_string()).collect();
    engine
}

#[test]
fn dead_cleanup_wave9_runtime_combat_start_families_replace_helper_contracts() {
    let mut bag = engine_without_start_with_relics(&["Bag of Preparation"], 8);
    bag.start_combat();
    assert_eq!(bag.state.player.status(sid::BAG_OF_PREP_DRAW), 0);
    assert_eq!(bag.state.hand.len(), 7);

    let mut ring = engine_without_start_with_relics(&["Ring of the Snake"], 8);
    ring.start_combat();
    assert_eq!(ring.state.player.status(sid::BAG_OF_PREP_DRAW), 0);
    assert_eq!(ring.state.hand.len(), 7);

}

#[test]
#[ignore = "blocked on combat-start temporary strength runtime parity; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/MutagenicStrength.java"]
fn dead_cleanup_wave9_mutagenic_strength_remains_queued_until_start_of_combat_runtime_is_authoritative() {}

#[test]
fn dead_cleanup_wave9_runtime_card_play_families_replace_helper_contracts() {
    let mut kunai_state = combat_state_with(
        make_deck_n("Strike_R", 6),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    kunai_state.relics.push("Kunai".to_string());
    let mut kunai = engine_with_state(kunai_state);
    kunai.state.hand = make_deck_n("Strike_R", 3);
    kunai.state.draw_pile.clear();
    assert_eq!(
        kunai.hidden_effect_value("Kunai", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
    assert!(play_on_enemy(&mut kunai, "Strike_R", 0));
    assert!(play_on_enemy(&mut kunai, "Strike_R", 0));
    assert_eq!(kunai.state.player.dexterity(), 0);
    assert!(play_on_enemy(&mut kunai, "Strike_R", 0));
    assert_eq!(kunai.state.player.dexterity(), 1);
    assert_eq!(
        kunai.hidden_effect_value("Kunai", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );

    let mut shuriken_state = combat_state_with(
        make_deck_n("Strike_R", 6),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    shuriken_state.relics.push("Shuriken".to_string());
    let mut shuriken = engine_with_state(shuriken_state);
    shuriken.state.hand = make_deck_n("Strike_R", 3);
    shuriken.state.draw_pile.clear();
    assert_eq!(
        shuriken.hidden_effect_value("Shuriken", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
    assert!(play_on_enemy(&mut shuriken, "Strike_R", 0));
    assert!(play_on_enemy(&mut shuriken, "Strike_R", 0));
    assert_eq!(shuriken.state.player.strength(), 0);
    assert!(play_on_enemy(&mut shuriken, "Strike_R", 0));
    assert_eq!(shuriken.state.player.strength(), 1);
    assert_eq!(
        shuriken.hidden_effect_value("Shuriken", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );

    let mut opener_state = combat_state_with(
        make_deck_n("Defend_R", 4),
        vec![enemy_no_intent("JawWorm", 40, 40), enemy_no_intent("Cultist", 44, 44)],
        3,
    );
    opener_state.relics.push("Letter Opener".to_string());
    let mut opener = engine_with_state(opener_state);
    opener.state.hand = make_deck_n("Defend_R", 3);
    opener.state.draw_pile.clear();
    let hp0 = opener.state.enemies[0].entity.hp;
    let hp1 = opener.state.enemies[1].entity.hp;
    assert_eq!(
        opener.hidden_effect_value("Letter Opener", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
    assert!(play_self(&mut opener, "Defend_R"));
    assert!(play_self(&mut opener, "Defend_R"));
    assert_eq!(opener.state.enemies[0].entity.hp, hp0);
    assert_eq!(opener.state.enemies[1].entity.hp, hp1);
    assert!(play_self(&mut opener, "Defend_R"));
    assert_eq!(opener.state.enemies[0].entity.hp, hp0 - 5);
    assert_eq!(opener.state.enemies[1].entity.hp, hp1 - 5);
    assert_eq!(
        opener.hidden_effect_value("Letter Opener", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );

    let mut urn_state = combat_state_with(
        make_deck_n("Strike_R", 4),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    urn_state.relics.push("Bird Faced Urn".to_string());
    let mut urn = engine_with_state(urn_state);
    urn.state.player.hp = 70;
    urn.state.hand = make_deck_n("Strike_R", 1);
    assert!(play_self(&mut urn, "Strike_R"));
    assert_eq!(urn.state.player.hp, 70);
    urn.state.hand = make_deck_n("Inflame", 1);
    assert!(play_self(&mut urn, "Inflame"));
    assert_eq!(urn.state.player.hp, 72);

    let mut yang_state = combat_state_with(
        make_deck_n("Strike_R", 2),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    yang_state.relics.push("Yang".to_string());
    let mut yang = engine_with_state(yang_state);
    yang.state.hand = make_deck_n("Strike_R", 2);
    assert!(play_on_enemy(&mut yang, "Strike_R", 0));
    assert_eq!(yang.state.player.dexterity(), 1);
    assert_eq!(yang.state.player.status(sid::LOSE_DEXTERITY), 1);
    yang.state.hand = make_deck_n("Defend_R", 1);
    assert!(play_self(&mut yang, "Defend_R"));
    assert_eq!(yang.state.player.dexterity(), 1);
}

#[test]
#[ignore = "blocked on the next-turn orb passive timing primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/EmotionChip.java"]
fn dead_cleanup_wave9_emotion_chip_still_waits_on_timing_primitive() {}
