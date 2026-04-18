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
use crate::tests::support::{combat_state_with, end_turn, enemy_no_intent, engine_with_state, make_deck_n};

#[test]
fn dead_cleanup_wave7_runtime_relics_cover_deleted_helper_families() {
    let mut state = combat_state_with(
        make_deck_n("Defend", 10),
        vec![enemy_no_intent("JawWorm", 120, 120)],
        3,
    );
    state.relics = vec![
        "Damaru".to_string(),
        "Inserter".to_string(),
        "HornCleat".to_string(),
        "CaptainsWheel".to_string(),
    ];
    let mut engine = engine_with_state(state);

    assert_eq!(engine.state.mantra, 1);
    assert_eq!(engine.hidden_effect_value("Inserter", EffectOwner::PlayerRelic { slot: 1 }, 0), 1);
    end_turn(&mut engine);
    assert_eq!(engine.state.player.status(crate::status_ids::sid::ORB_SLOTS), 1);
    assert_eq!(engine.state.player.block, 14);
}

#[test]
fn dead_cleanup_wave7_helper_path_deleted_families_are_backed_by_runtime_tests() {
    let mut state = combat_state_with(
        make_deck_n("Defend", 10),
        vec![enemy_no_intent("JawWorm", 120, 120)],
        3,
    );
    state.relics = vec![
        "StoneCalendar".to_string(),
        "Art of War".to_string(),
        "Orichalcum".to_string(),
        "CloakClasp".to_string(),
    ];
    let mut engine = engine_with_state(state);
    let hp_before = engine.state.enemies[0].entity.hp;
    end_turn(&mut engine);
    for _ in 0..5 {
        end_turn(&mut engine);
    }
    end_turn(&mut engine);
    assert!(engine.state.enemies[0].entity.hp < hp_before);
    assert!(engine.state.energy >= 3);
}
