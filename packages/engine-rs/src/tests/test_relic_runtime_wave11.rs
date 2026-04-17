#![cfg(test)]

// Java references:
// - decompiled/java-src/com/megacrit/cardcrawl/relics/HornCleat.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/CaptainsWheel.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/StoneCalendar.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/ArtOfWar.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Orichalcum.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/CloakClasp.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Damaru.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Inserter.java

use crate::effects::runtime::EffectOwner;
use crate::tests::support::{
    combat_state_with, end_turn, enemy, enemy_no_intent, engine_with_state, make_deck, make_deck_n,
};

#[test]
fn relic_wave11_runtime_turn_order_relics_remain_authoritative() {
    let mut state = combat_state_with(
        make_deck_n("Defend_R", 10),
        vec![enemy_no_intent("JawWorm", 120, 120), enemy_no_intent("Cultist", 120, 120)],
        3,
    );
    state.relics = vec![
        "HornCleat".to_string(),
        "CaptainsWheel".to_string(),
        "StoneCalendar".to_string(),
        "Art of War".to_string(),
    ];
    let mut engine = engine_with_state(state);

    assert_eq!(engine.hidden_effect_value("HornCleat", EffectOwner::PlayerRelic { slot: 0 }, 0), 1);
    assert_eq!(
        engine.hidden_effect_value("CaptainsWheel", EffectOwner::PlayerRelic { slot: 1 }, 0),
        1
    );

    end_turn(&mut engine);
    assert_eq!(engine.state.turn, 2);
    assert_eq!(engine.state.player.block, 14);
    assert_eq!(engine.state.energy, 4);
    assert_eq!(engine.hidden_effect_value("HornCleat", EffectOwner::PlayerRelic { slot: 0 }, 0), -1);
    assert_eq!(
        engine.hidden_effect_value("CaptainsWheel", EffectOwner::PlayerRelic { slot: 1 }, 0),
        2
    );

    for _ in 0..5 {
        end_turn(&mut engine);
    }
    let hp0 = engine.state.enemies[0].entity.hp;
    let hp1 = engine.state.enemies[1].entity.hp;
    end_turn(&mut engine);
    assert_eq!(engine.state.enemies[0].entity.hp, hp0 - 52);
    assert_eq!(engine.state.enemies[1].entity.hp, hp1 - 52);
}

#[test]
fn relic_wave11_runtime_end_turn_and_post_draw_paths_match_canonical_runtime() {
    let mut state = combat_state_with(
        make_deck(&["Defend_P", "Defend_P", "Defend_P"]),
        vec![enemy("JawWorm", 60, 60, 1, 5, 1)],
        3,
    );
    state.relics = vec![
        "Orichalcum".to_string(),
        "CloakClasp".to_string(),
        "Damaru".to_string(),
        "Inserter".to_string(),
    ];
    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Defend_P", "Defend_P"]);
    let hp_before = engine.state.player.hp;
    assert_eq!(engine.state.mantra, 1);
    assert_eq!(engine.hidden_effect_value("Inserter", EffectOwner::PlayerRelic { slot: 3 }, 0), 1);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, hp_before);
    assert_eq!(engine.state.player.status(crate::status_ids::sid::ORB_SLOTS), 1);
    assert_eq!(engine.state.player.block, 0);
}
