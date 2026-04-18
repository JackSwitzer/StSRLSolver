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

use crate::actions::Action;
use crate::effects::runtime::EffectOwner;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, end_turn, enemy, enemy_no_intent, engine_with_state, make_deck,
    make_deck_n, play_on_enemy, play_self,
};

#[test]
fn relic_wave8_boot_torii_and_tungsten_rod_follow_engine_path_damage_rules() {
    let mut boot_state = combat_state_with(
        make_deck(&["Shiv"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    boot_state.relics.push("Boot".to_string());
    let mut boot = engine_with_state(boot_state);
    boot.state.hand = make_deck(&["Shiv"]);
    boot.state.draw_pile.clear();
    boot.state.enemies[0].entity.block = 2;
    let boot_hp_before = boot.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut boot, "Shiv", 0));
    assert_eq!(boot.state.enemies[0].entity.block, 0);
    assert_eq!(boot_hp_before - boot.state.enemies[0].entity.hp, 5);

    let mut torii_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy("JawWorm", 100, 100, 1, 4, 1)],
        3,
    );
    torii_state.relics.push("Torii".to_string());
    let mut torii = engine_with_state(torii_state);
    let torii_hp_before = torii.state.player.hp;
    end_turn(&mut torii);
    assert_eq!(torii.state.player.hp, torii_hp_before - 1);

    let mut rod_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy("JawWorm", 100, 100, 1, 10, 1)],
        3,
    );
    rod_state.relics.push("Tungsten Rod".to_string());
    let mut rod = engine_with_state(rod_state);
    let rod_hp_before = rod.state.player.hp;
    end_turn(&mut rod);
    assert_eq!(rod.state.player.hp, rod_hp_before - 9);
}

#[test]
fn relic_wave8_counter_relics_track_hidden_runtime_state_without_helper_path() {
    let mut fan_state = combat_state_with(
        make_deck_n("Strike", 12),
        vec![enemy_no_intent("JawWorm", 120, 120)],
        20,
    );
    fan_state.relics.push("Ornamental Fan".to_string());
    let mut fan = engine_with_state(fan_state);
    fan.state.hand = make_deck_n("Strike", 3);
    fan.state.draw_pile.clear();
    fan.state.discard_pile.clear();

    assert!(play_on_enemy(&mut fan, "Strike", 0));
    assert!(play_on_enemy(&mut fan, "Strike", 0));
    assert_eq!(fan.state.player.block, 0);
    assert!(play_on_enemy(&mut fan, "Strike", 0));
    assert_eq!(fan.state.player.block, 4);
    assert_eq!(
        fan.hidden_effect_value("Ornamental Fan", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );

    let mut nunchaku_state = combat_state_with(
        make_deck_n("Strike", 10),
        vec![enemy_no_intent("JawWorm", 160, 160)],
        20,
    );
    nunchaku_state.relics.push("Nunchaku".to_string());
    let mut nunchaku = engine_with_state(nunchaku_state);
    nunchaku.state.hand = make_deck_n("Strike", 10);

    for _ in 0..9 {
        assert!(play_on_enemy(&mut nunchaku, "Strike", 0));
    }
    assert_eq!(
        nunchaku.hidden_effect_value("Nunchaku", EffectOwner::PlayerRelic { slot: 0 }, 0),
        9
    );
    let energy_before = nunchaku.state.energy;
    assert!(play_on_enemy(&mut nunchaku, "Strike", 0));
    assert_eq!(nunchaku.state.energy, energy_before);
    assert_eq!(
        nunchaku.hidden_effect_value("Nunchaku", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn relic_wave8_pen_nib_orange_pellets_and_velvet_choker_use_runtime_path() {
    let mut pellets_state = combat_state_with(
        make_deck(&["Strike", "Defend", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 120, 120)],
        20,
    );
    pellets_state.relics.push("OrangePellets".to_string());
    let mut pellets = engine_with_state(pellets_state);
    pellets.state.hand = make_deck(&["Strike", "Defend", "Inflame"]);
    pellets.state.player.set_status(sid::WEAKENED, 2);
    pellets.state.player.set_status(sid::VULNERABLE, 2);
    assert!(play_on_enemy(&mut pellets, "Strike", 0));
    assert!(play_self(&mut pellets, "Defend"));
    assert!(play_self(&mut pellets, "Inflame"));
    assert_eq!(pellets.state.player.status(sid::WEAKENED), 0);
    assert_eq!(pellets.state.player.status(sid::VULNERABLE), 0);

    let mut nib_state = combat_state_with(
        make_deck_n("Strike", 10),
        vec![enemy_no_intent("JawWorm", 160, 160)],
        20,
    );
    nib_state.relics.push("Pen Nib".to_string());
    let mut nib = engine_with_state(nib_state);
    nib.state.hand = make_deck_n("Strike", 10);
    for _ in 0..9 {
        assert!(play_on_enemy(&mut nib, "Strike", 0));
    }
    assert_eq!(nib.state.player.status(sid::PEN_NIB_COUNTER), 9);
    let hp_before = nib.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut nib, "Strike", 0));
    assert_eq!(nib.state.player.status(sid::PEN_NIB_COUNTER), 0);
    assert_eq!(nib.state.enemies[0].entity.hp, hp_before - 12);

    let mut choker_state = combat_state_with(
        make_deck_n("Defend", 7),
        vec![enemy_no_intent("JawWorm", 120, 120)],
        20,
    );
    choker_state.relics.push("Velvet Choker".to_string());
    let mut choker = engine_with_state(choker_state);
    choker.state.hand = make_deck_n("Defend", 7);
    for _ in 0..6 {
        choker.execute_action(&Action::PlayCard {
            card_idx: 0,
            target_idx: -1,
        });
    }
    let hand_before = choker.state.hand.len();
    choker.execute_action(&Action::PlayCard {
        card_idx: 0,
        target_idx: -1,
    });
    assert_eq!(choker.state.hand.len(), hand_before);
    assert_eq!(choker.state.cards_played_this_turn, 6);
}
