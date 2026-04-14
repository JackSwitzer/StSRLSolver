use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::state::Stance;
use crate::status_ids::sid;
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
fn conclude_and_consecrate_export_declarative_all_enemy_damage() {
    let registry = global_registry();

    let conclude = registry.get("Conclude").expect("Conclude should be registered");
    assert_eq!(conclude.declared_all_enemy_damage(), Some(A::Damage));
    assert!(conclude.complex_hook.is_none());

    let conclude_plus = registry.get("Conclude+").expect("Conclude+ should be registered");
    assert_eq!(conclude_plus.declared_all_enemy_damage(), Some(A::Damage));
    assert!(conclude_plus.complex_hook.is_none());

    let consecrate = registry.get("Consecrate").expect("Consecrate should be registered");
    assert_eq!(consecrate.declared_all_enemy_damage(), Some(A::Damage));
    assert!(consecrate.complex_hook.is_none());

    let consecrate_plus = registry
        .get("Consecrate+")
        .expect("Consecrate+ should be registered");
    assert_eq!(consecrate_plus.declared_all_enemy_damage(), Some(A::Damage));
    assert!(consecrate_plus.complex_hook.is_none());
}

#[test]
fn conclude_hits_all_enemies_and_ends_the_turn() {
    let mut engine = two_enemy_engine(("JawWorm", 50, 0), ("Cultist", 50, 0));
    ensure_in_hand(&mut engine, "Conclude");
    let turn_before = engine.state.turn;
    assert!(play_on_enemy(&mut engine, "Conclude", 0));
    assert_eq!(engine.state.turn, turn_before + 1);
    assert_eq!(engine.state.enemies[0].entity.hp, 38);
    assert_eq!(engine.state.enemies[1].entity.hp, 38);
}

#[test]
fn consecrate_deals_all_enemy_damage_without_triggering_extra_hooks() {
    let mut engine = two_enemy_engine(("JawWorm", 50, 0), ("Cultist", 50, 0));
    ensure_in_hand(&mut engine, "Consecrate");
    assert!(play_on_enemy(&mut engine, "Consecrate", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 45);
    assert_eq!(engine.state.enemies[1].entity.hp, 45);
}

#[test]
fn crescendo_is_declared_as_a_stance_change_and_can_return_flurry() {
    let registry = global_registry();
    let crescendo = registry.get("Crescendo").expect("Crescendo should be registered");
    assert_eq!(crescendo.enter_stance, Some("Wrath"));
    assert_eq!(crescendo.effect_data, &[]);

    let mut engine = one_enemy_engine("JawWorm", 50, 0);
    engine.state.discard_pile.push(engine.card_registry.make_card("FlurryOfBlows"));
    ensure_in_hand(&mut engine, "Crescendo");
    assert!(play_self(&mut engine, "Crescendo"));
    assert_eq!(engine.state.stance, Stance::Wrath);
    assert_eq!(hand_count(&engine, "FlurryOfBlows"), 1);
    assert_eq!(discard_prefix_count(&engine, "FlurryOfBlows"), 0);
}

#[test]
fn collect_uses_x_cost_setup_and_adds_miracles_next_turn() {
    let registry = global_registry();
    let collect = registry.get("Collect").expect("Collect should be registered");
    assert_eq!(
        collect.effect_data,
        &[E::Simple(SE::SetStatus(T::SelfEntity, sid::COLLECT_MIRACLES, A::XCost))]
    );
    assert!(collect.complex_hook.is_some());

    let collect_plus = registry.get("Collect+").expect("Collect+ should be registered");
    assert_eq!(
        collect_plus.effect_data,
        &[E::Simple(SE::SetStatus(T::SelfEntity, sid::COLLECT_MIRACLES, A::XCost))]
    );
    assert!(collect_plus.complex_hook.is_some());

    let mut engine = one_enemy_engine("JawWorm", 50, 0);
    engine.state.energy = 3;
    ensure_in_hand(&mut engine, "Collect");
    assert!(play_self(&mut engine, "Collect"));
    assert_eq!(engine.state.player.status(sid::COLLECT_MIRACLES), 3);
    let miracles_before = hand_count(&engine, "Miracle");
    end_turn(&mut engine);
    assert!(hand_count(&engine, "Miracle") >= miracles_before + 3);

    let mut upgraded = one_enemy_engine("JawWorm", 50, 0);
    upgraded.state.energy = 3;
    ensure_in_hand(&mut upgraded, "Collect+");
    assert!(play_self(&mut upgraded, "Collect+"));
    assert_eq!(upgraded.state.player.status(sid::COLLECT_MIRACLES), 4);
}

#[test]
fn conjure_blade_is_x_cost_setup_for_expunger() {
    let registry = global_registry();
    let conjure_blade = registry.get("ConjureBlade").expect("ConjureBlade should be registered");
    assert_eq!(
        conjure_blade.effect_data,
        &[E::Simple(SE::AddCard(
            "Expunger",
            crate::effects::declarative::Pile::Hand,
            A::Fixed(1),
        ))]
    );
    assert!(conjure_blade.complex_hook.is_some());

    let mut engine = one_enemy_engine("JawWorm", 100, 0);
    engine.state.energy = 3;
    ensure_in_hand(&mut engine, "ConjureBlade");
    assert!(play_self(&mut engine, "ConjureBlade"));
    let expunger = engine
        .state
        .hand
        .iter()
        .find(|card| engine.card_registry.card_name(card.def_id) == "Expunger")
        .expect("generated Expunger");
    assert_eq!(expunger.misc, 3);

    engine.state.energy = 1;
    let expunger_hp_before = engine.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut engine, "Expunger", 0));
    assert_eq!(expunger_hp_before - engine.state.enemies[0].entity.hp, 27);

    let mut upgraded = one_enemy_engine("JawWorm", 100, 0);
    upgraded.state.energy = 3;
    ensure_in_hand(&mut upgraded, "ConjureBlade+");
    assert!(play_self(&mut upgraded, "ConjureBlade+"));
    let expunger = upgraded
        .state
        .hand
        .iter()
        .find(|card| upgraded.card_registry.card_name(card.def_id) == "Expunger")
        .expect("generated Expunger");
    assert_eq!(expunger.misc, 4);
}

#[test]
fn deus_ex_machina_still_draws_miracles_on_draw() {
    let engine = engine_with(make_deck(&["DeusExMachina"]), 50, 0);
    assert_eq!(hand_count(&engine, "Miracle"), 2);
    assert_eq!(hand_count(&engine, "DeusExMachina"), 0);
    assert_eq!(exhaust_prefix_count(&engine, "DeusExMachina"), 1);
}
