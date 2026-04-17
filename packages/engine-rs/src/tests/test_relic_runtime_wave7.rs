#![cfg(test)]

// Java references:
// - decompiled/java-src/com/megacrit/cardcrawl/relics/TwistedFunnel.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/SneckoEye.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Sling.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/TeardropLocket.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Pantograph.java

use crate::effects::runtime::EffectOwner;
use crate::state::Stance;
use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, make_deck_n};

fn engine_without_start_with_relics(
    relics: &[&str],
    deck: &[&str],
    enemies: Vec<crate::state::EnemyCombatState>,
    energy: i32,
) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(crate::tests::support::make_deck(deck), enemies, energy);
    engine.state.relics = relics.iter().map(|id| (*id).to_string()).collect();
    engine
}

#[test]
fn twisted_funnel_applies_poison_to_all_enemies_on_runtime_path() {
    let mut engine = engine_without_start_with_relics(
        &["TwistedFunnel"],
        &["Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R"],
        vec![
            enemy_no_intent("JawWorm", 40, 40),
            enemy_no_intent("Cultist", 44, 44),
        ],
        3,
    );

    engine.start_combat();

    assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 4);
    assert_eq!(engine.state.enemies[1].entity.status(sid::POISON), 4);
}

#[test]
fn snecko_eye_confuses_and_draws_two_extra_cards_on_runtime_path() {
    let mut engine = engine_without_start_with_relics(
        &["Snecko Eye"],
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

    engine.start_combat();

    assert_eq!(engine.state.player.status(sid::SNECKO_EYE), 1);
    assert_eq!(engine.state.player.status(sid::CONFUSION), 1);
    assert_eq!(engine.state.player.status(sid::BAG_OF_PREP_DRAW), 2);
}

#[test]
fn sling_grants_strength_only_in_elite_fights_on_runtime_path() {
    let mut elite_engine = engine_without_start_with_relics(
        &["Sling"],
        &["Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R"],
        vec![enemy_no_intent("GremlinNob", 90, 90)],
        3,
    );
    elite_engine.start_combat();
    assert_eq!(elite_engine.state.player.strength(), 2);

    let mut hallway_engine = engine_without_start_with_relics(
        &["Sling"],
        &["Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R"],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    hallway_engine.start_combat();
    assert_eq!(hallway_engine.state.player.strength(), 0);
}

#[test]
fn teardrop_locket_starts_player_in_calm_on_runtime_path() {
    let mut engine = engine_without_start_with_relics(
        &["TeardropLocket"],
        &["Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R"],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    engine.start_combat();

    assert_eq!(engine.state.stance, Stance::Calm);
}

#[test]
fn pantograph_heals_only_at_boss_combat_start_on_runtime_path() {
    let mut boss_engine = engine_without_start_with_relics(
        &["Pantograph"],
        &["Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R"],
        vec![enemy_no_intent("Hexaghost", 250, 250)],
        3,
    );
    boss_engine.state.player.hp = 50;
    boss_engine.start_combat();
    assert_eq!(boss_engine.state.player.hp, 75);

    let mut hallway_engine = engine_without_start_with_relics(
        &["Pantograph"],
        &["Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R"],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    hallway_engine.state.player.hp = 50;
    hallway_engine.start_combat();
    assert_eq!(hallway_engine.state.player.hp, 50);
}

#[test]
fn preserved_insect_non_elite_hallway_fight_does_not_reduce_enemy_hp() {
    let mut engine = engine_without_start_with_relics(
        &["PreservedInsect"],
        &["Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R"],
        vec![
            enemy_no_intent("JawWorm", 20, 20),
            enemy_no_intent("Cultist", 40, 40),
        ],
        3,
    );

    engine.start_combat();

    assert_eq!(engine.state.enemies[0].entity.hp, 20);
    assert_eq!(engine.state.enemies[1].entity.hp, 40);
}

#[test]
fn runtime_hidden_state_relics_do_not_grant_strength_without_stored_progress() {
    let mut du_vu_engine = engine_without_start(
        make_deck_n("Strike_R", 5),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    du_vu_engine.state.relics.push("Du-Vu Doll".to_string());
    du_vu_engine.start_combat();
    assert_eq!(du_vu_engine.state.player.strength(), 0);
    assert_eq!(
        du_vu_engine.hidden_effect_value("Du-Vu Doll", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );

    let mut girya_engine = engine_without_start(
        make_deck_n("Strike_R", 5),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    girya_engine.state.relics.push("Girya".to_string());
    girya_engine.start_combat();
    assert_eq!(girya_engine.state.player.strength(), 0);
    assert_eq!(
        girya_engine.hidden_effect_value("Girya", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}
