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
fn relic_wave10_delayed_turn_block_relics_follow_runtime_path() {
    let mut horn_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    horn_state.relics.push("HornCleat".to_string());
    let mut horn = engine_with_state(horn_state);
    assert_eq!(horn.hidden_effect_value("HornCleat", EffectOwner::PlayerRelic { slot: 0 }, 0), 1);
    end_turn(&mut horn);
    assert_eq!(horn.state.turn, 2);
    assert_eq!(horn.state.player.block, 14);
    assert_eq!(horn.hidden_effect_value("HornCleat", EffectOwner::PlayerRelic { slot: 0 }, 0), -1);

    let mut wheel_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    wheel_state.relics.push("CaptainsWheel".to_string());
    let mut wheel = engine_with_state(wheel_state);
    assert_eq!(wheel.hidden_effect_value("CaptainsWheel", EffectOwner::PlayerRelic { slot: 0 }, 0), 1);
    end_turn(&mut wheel);
    assert_eq!(wheel.hidden_effect_value("CaptainsWheel", EffectOwner::PlayerRelic { slot: 0 }, 0), 2);
    end_turn(&mut wheel);
    assert_eq!(wheel.state.turn, 3);
    assert_eq!(wheel.state.player.block, 18);
    assert_eq!(
        wheel.hidden_effect_value("CaptainsWheel", EffectOwner::PlayerRelic { slot: 0 }, 0),
        -1
    );
}

#[test]
fn relic_wave10_turn_end_runtime_relics_match_java_timing() {
    let mut calendar_state = combat_state_with(
        make_deck_n("Defend", 10),
        vec![enemy_no_intent("JawWorm", 120, 120), enemy_no_intent("Cultist", 120, 120)],
        3,
    );
    calendar_state.relics.push("StoneCalendar".to_string());
    let mut calendar = engine_with_state(calendar_state);
    for _ in 0..6 {
        end_turn(&mut calendar);
    }
    assert_eq!(calendar.hidden_effect_value("StoneCalendar", EffectOwner::PlayerRelic { slot: 0 }, 0), 7);
    let hp0 = calendar.state.enemies[0].entity.hp;
    let hp1 = calendar.state.enemies[1].entity.hp;
    end_turn(&mut calendar);
    assert_eq!(calendar.state.enemies[0].entity.hp, hp0 - 52);
    assert_eq!(calendar.state.enemies[1].entity.hp, hp1 - 52);

    let mut orichalcum_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy("JawWorm", 60, 60, 1, 5, 1)],
        3,
    );
    orichalcum_state.relics.push("Orichalcum".to_string());
    let mut orichalcum = engine_with_state(orichalcum_state);
    let hp_before = orichalcum.state.player.hp;
    end_turn(&mut orichalcum);
    assert_eq!(orichalcum.state.player.hp, hp_before);

    let mut clasp_state = combat_state_with(
        make_deck(&["Defend", "Defend", "Defend"]),
        vec![enemy("JawWorm", 60, 60, 1, 3, 1)],
        3,
    );
    clasp_state.relics.push("CloakClasp".to_string());
    let mut clasp = engine_with_state(clasp_state);
    clasp.state.hand = make_deck(&["Defend", "Defend"]);
    clasp.state.draw_pile.clear();
    let hp_before = clasp.state.player.hp;
    end_turn(&mut clasp);
    assert_eq!(clasp.state.player.hp, hp_before - 1);
}

#[test]
fn relic_wave10_turn_progression_relics_follow_runtime_path() {
    let mut art_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    art_state.relics.push("Art of War".to_string());
    let mut art = engine_with_state(art_state);
    end_turn(&mut art);
    assert_eq!(art.state.turn, 2);
    assert_eq!(art.state.energy, 4);
    assert_eq!(art.hidden_effect_value("Art of War", EffectOwner::PlayerRelic { slot: 0 }, 0), 1);

    let mut damaru_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    damaru_state.relics.push("Damaru".to_string());
    let mut damaru = engine_with_state(damaru_state);
    assert_eq!(damaru.state.mantra, 1);
    damaru.state.mantra = 9;
    end_turn(&mut damaru);
    assert_eq!(damaru.state.stance, crate::state::Stance::Divinity);
    assert_eq!(damaru.state.mantra, 0);

    let mut inserter_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    inserter_state.relics.push("Inserter".to_string());
    let mut inserter = engine_with_state(inserter_state);
    assert_eq!(inserter.state.player.status(crate::status_ids::sid::ORB_SLOTS), 0);
    assert_eq!(inserter.hidden_effect_value("Inserter", EffectOwner::PlayerRelic { slot: 0 }, 0), 1);
    end_turn(&mut inserter);
    assert_eq!(inserter.state.player.status(crate::status_ids::sid::ORB_SLOTS), 1);
    assert_eq!(inserter.hidden_effect_value("Inserter", EffectOwner::PlayerRelic { slot: 0 }, 0), 0);
}
