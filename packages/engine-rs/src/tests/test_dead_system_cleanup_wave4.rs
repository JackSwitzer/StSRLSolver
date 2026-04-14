#![cfg(test)]

// Java references:
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Boot.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Torii.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/TungstenRod.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/OrnamentalFan.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Nunchaku.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/PenNib.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/OrangePellets.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/VelvetChoker.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Pocketwatch.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/HappyFlower.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/IncenseBurner.java

use crate::effects::runtime::EffectOwner;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, end_turn, enemy, enemy_no_intent, engine_with_state, make_deck,
    make_deck_n, play_on_enemy, play_self,
};

#[test]
fn dead_cleanup_wave4_runtime_relic_bundle_is_authoritative() {
    let mut state = combat_state_with(
        make_deck(&["Strike_R", "Defend_R", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 120, 120)],
        20,
    );
    state.relics = vec![
        "OrangePellets".to_string(),
        "Pen Nib".to_string(),
        "Velvet Choker".to_string(),
        "Nunchaku".to_string(),
        "InkBottle".to_string(),
        "Ornamental Fan".to_string(),
        "Pocketwatch".to_string(),
        "Happy Flower".to_string(),
        "Incense Burner".to_string(),
    ];
    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Strike_R", "Defend_R", "Inflame"]);
    engine.state.player.set_status(sid::WEAKENED, 2);
    engine.state.player.set_status(sid::VULNERABLE, 2);

    assert!(play_on_enemy(&mut engine, "Strike_R", 0));
    assert!(play_self(&mut engine, "Defend_R"));
    assert!(play_self(&mut engine, "Inflame"));
    assert_eq!(engine.state.player.status(sid::WEAKENED), 0);
    assert_eq!(engine.state.player.status(sid::VULNERABLE), 0);
    assert_eq!(
        engine.hidden_effect_value("OrangePellets", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
    assert_eq!(
        engine.hidden_effect_value("Nunchaku", EffectOwner::PlayerRelic { slot: 3 }, 0),
        1
    );
    assert_eq!(
        engine.hidden_effect_value("InkBottle", EffectOwner::PlayerRelic { slot: 4 }, 0),
        3
    );
    assert_eq!(
        engine.hidden_effect_value("Ornamental Fan", EffectOwner::PlayerRelic { slot: 5 }, 0),
        1
    );
    assert_eq!(
        engine.hidden_effect_value("Pocketwatch", EffectOwner::PlayerRelic { slot: 6 }, 0),
        3
    );
    assert_eq!(
        engine.hidden_effect_value("Happy Flower", EffectOwner::PlayerRelic { slot: 7 }, 0),
        1
    );
    assert_eq!(
        engine.hidden_effect_value("Incense Burner", EffectOwner::PlayerRelic { slot: 8 }, 0),
        1
    );
    assert_eq!(engine.state.player.status(sid::PEN_NIB_COUNTER), 1);
    assert_eq!(
        engine.hidden_effect_value("Velvet Choker", EffectOwner::PlayerRelic { slot: 2 }, 0),
        3
    );
}

#[test]
fn dead_cleanup_wave4_damage_modifier_relics_are_covered_on_engine_path() {
    let mut boot_state = combat_state_with(
        make_deck(&["Shiv"]),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    boot_state.relics.push("Boot".to_string());
    let mut boot = engine_with_state(boot_state);
    boot.state.hand = make_deck(&["Shiv"]);
    boot.state.enemies[0].entity.block = 2;
    let hp_before = boot.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut boot, "Shiv", 0));
    assert_eq!(hp_before - boot.state.enemies[0].entity.hp, 5);

    let mut torii_state = combat_state_with(
        make_deck_n("Defend_R", 5),
        vec![enemy("JawWorm", 60, 60, 1, 4, 1)],
        3,
    );
    torii_state.relics.push("Torii".to_string());
    let mut torii = engine_with_state(torii_state);
    let hp_before = torii.state.player.hp;
    end_turn(&mut torii);
    assert_eq!(torii.state.player.hp, hp_before - 1);

    let mut rod_state = combat_state_with(
        make_deck_n("Defend_R", 5),
        vec![enemy("JawWorm", 60, 60, 1, 10, 1)],
        3,
    );
    rod_state.relics.push("Tungsten Rod".to_string());
    let mut rod = engine_with_state(rod_state);
    let hp_before = rod.state.player.hp;
    end_turn(&mut rod);
    assert_eq!(rod.state.player.hp, hp_before - 9);
}
