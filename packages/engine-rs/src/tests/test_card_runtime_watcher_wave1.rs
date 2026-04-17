use crate::cards::global_registry;
use crate::effects::declarative::{Effect as E, SimpleEffect as SE};
use crate::state::Stance;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32, dmg: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy(enemy_id, hp, hp, 1, dmg, 1)], 3);
    force_player_turn(&mut engine);
    engine
}

fn two_enemy_engine(
    a: (&str, i32, i32),
    b: (&str, i32, i32),
) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy(a.0, a.1, a.1, 1, a.2, 1),
            enemy(b.0, b.1, b.1, 1, b.2, 1),
        ],
        3,
    );
    force_player_turn(&mut engine);
    engine
}

#[test]
fn tantrum_and_empty_body_now_declare_stance_changes_in_effect_data() {
    let registry = global_registry();

    let tantrum = registry.get("Tantrum").expect("Tantrum should be registered");
    assert_eq!(tantrum.enter_stance, Some("Wrath"));
    assert_eq!(tantrum.effect_data, &[E::Simple(SE::ChangeStance(Stance::Wrath))]);

    let tantrum_plus = registry.get("Tantrum+").expect("Tantrum+ should be registered");
    assert_eq!(tantrum_plus.enter_stance, Some("Wrath"));
    assert_eq!(tantrum_plus.effect_data, &[E::Simple(SE::ChangeStance(Stance::Wrath))]);

    let empty_body = registry
        .get("EmptyBody")
        .expect("Empty Body should be registered");
    assert_eq!(empty_body.enter_stance, Some("Neutral"));
    assert_eq!(
        empty_body.effect_data,
        &[
            E::Simple(SE::GainBlock(crate::effects::declarative::AmountSource::Block)),
            E::Simple(SE::ChangeStance(Stance::Neutral)),
        ]
    );

    let empty_body_plus = registry
        .get("EmptyBody+")
        .expect("Empty Body+ should be registered");
    assert_eq!(empty_body_plus.enter_stance, Some("Neutral"));
    assert_eq!(
        empty_body_plus.effect_data,
        &[
            E::Simple(SE::GainBlock(crate::effects::declarative::AmountSource::Block)),
            E::Simple(SE::ChangeStance(Stance::Neutral)),
        ]
    );
}

#[test]
fn conclude_and_safety_cover_end_turn_and_retain_paths() {
    let mut conclude_engine = two_enemy_engine(("JawWorm", 50, 0), ("Cultist", 50, 0));
    ensure_in_hand(&mut conclude_engine, "Conclude");
    let turn_before = conclude_engine.state.turn;
    assert!(play_on_enemy(&mut conclude_engine, "Conclude", 0));
    assert_eq!(conclude_engine.state.turn, turn_before + 1);
    assert_eq!(conclude_engine.state.enemies[0].entity.hp, 38);
    assert_eq!(conclude_engine.state.enemies[1].entity.hp, 38);

    let mut safety_engine = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut safety_engine, "Safety");
    let turn_before = safety_engine.state.turn;
    end_turn(&mut safety_engine);
    assert_eq!(safety_engine.state.turn, turn_before + 1);
    assert_eq!(hand_count(&safety_engine, "Safety"), 1);
    assert!(
        !safety_engine
            .state
            .discard_pile
            .iter()
            .any(|card| safety_engine.card_registry.card_name(card.def_id) == "Safety")
    );
}

#[test]
fn watcher_power_cards_install_and_trigger_through_the_existing_runtime() {
    let mut battle_hymn = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut battle_hymn, "BattleHymn");
    assert!(play_self(&mut battle_hymn, "BattleHymn"));
    end_turn(&mut battle_hymn);
    assert_eq!(hand_count(&battle_hymn, "Smite"), 1);

    let mut deva_form = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut deva_form, "DevaForm");
    assert!(play_self(&mut deva_form, "DevaForm"));
    end_turn(&mut deva_form);
    assert_eq!(deva_form.state.energy, 4);

    let mut fasting = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut fasting, "Fasting2");
    assert!(play_self(&mut fasting, "Fasting2"));
    assert_eq!(fasting.state.player.strength(), 3);
    assert_eq!(fasting.state.player.dexterity(), 3);
    assert_eq!(fasting.state.max_energy, 2);
    assert_eq!(fasting.state.energy, 1);

    let mut mental_fortress = one_enemy_engine("JawWorm", 100, 0);
    set_stance(&mut mental_fortress, Stance::Wrath);
    ensure_in_hand(&mut mental_fortress, "MentalFortress");
    ensure_in_hand(&mut mental_fortress, "EmptyBody");
    assert!(play_self(&mut mental_fortress, "MentalFortress"));
    let block_before = mental_fortress.state.player.block;
    assert!(play_self(&mut mental_fortress, "EmptyBody"));
    assert_eq!(mental_fortress.state.stance, Stance::Neutral);
    assert_eq!(mental_fortress.state.player.block, block_before + 11);
}
