#![cfg(test)]

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::gameplay::GameplayProgramSource;
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::{
    enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy, play_self,
};

fn assert_gameplay_card_export(
    id: &str,
    card_type: CardType,
    target: CardTarget,
    cost: i32,
    exhausts: bool,
    upgraded_from: Option<&str>,
) -> crate::gameplay::CardSchema {
    let def = crate::gameplay::global_registry()
        .card(id)
        .unwrap_or_else(|| panic!("missing gameplay card export for {id}"));
    assert_eq!(def.program_source(), GameplayProgramSource::AdaptedLegacy, "{id} source");

    let schema = def.card_schema().expect("card schema");
    assert_eq!(schema.card_type, Some(card_type), "{id} type");
    assert_eq!(schema.target, Some(target), "{id} target");
    assert_eq!(schema.cost, Some(cost), "{id} cost");
    assert_eq!(schema.exhausts, exhausts, "{id} exhaust");
    assert_eq!(schema.upgraded_from.as_deref(), upgraded_from, "{id} upgraded_from");
    schema.clone()
}

#[test]
fn test_card_runtime_defect_wave4_registry_exports_cover_runtime_progress() {
    let reg = global_registry();

    let boot = reg.get("BootSequence").expect("BootSequence");
    assert!(boot.effects.contains(&"innate"));
    assert!(boot.effect_data.is_empty());

    let defend = reg.get("Defend_B").expect("Defend_B");
    assert!(defend.effect_data.is_empty());
    assert!(defend.complex_hook.is_none());

    let buffer = reg.get("Buffer").expect("Buffer");
    assert_eq!(
        buffer.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::BUFFER, A::Magic))]
    );
    assert!(buffer.complex_hook.is_none());

    let chaos = reg.get("Chaos").expect("Chaos");
    assert!(chaos.complex_hook.is_some(), "Chaos still needs a random-orb hook");

    let ftl = reg.get("FTL").expect("FTL");
    assert!(ftl.complex_hook.is_some(), "FTL still needs a cards-played hook");

    let claw = reg.get("Gash").expect("Gash");
    assert_eq!(
        claw.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::CLAW_BONUS, A::Magic))]
    );
    assert!(claw.complex_hook.is_none());

    let capacitor = assert_gameplay_card_export(
        "Capacitor+",
        CardType::Power,
        CardTarget::SelfTarget,
        1,
        false,
        Some("Capacitor"),
    );
    assert_eq!(capacitor.declared_effect_count, 0);
}

#[test]
fn test_card_runtime_defect_wave4_boot_sequence_defend_and_buffer_follow_engine_path() {
    let mut boot = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut boot);
    boot.state.hand = make_deck(&["BootSequence+"]);
    assert!(play_self(&mut boot, "BootSequence+"));
    assert_eq!(boot.state.player.block, 13);
    assert!(boot
        .state
        .exhaust_pile
        .iter()
        .any(|card| boot.card_registry.card_name(card.def_id) == "BootSequence+"));

    let mut defend = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut defend);
    defend.state.hand = make_deck(&["Defend_B+"]);
    assert!(play_self(&mut defend, "Defend_B+"));
    assert_eq!(defend.state.player.block, 8);

    let mut buffer = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut buffer);
    buffer.state.hand = make_deck(&["Buffer+"]);
    assert!(play_self(&mut buffer, "Buffer+"));
    assert_eq!(buffer.state.player.status(sid::BUFFER), 2);
}

#[test]
fn test_card_runtime_defect_wave4_capacitor_and_chaos_change_orb_state_on_engine_path() {
    let mut capacitor = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut capacitor);
    capacitor.init_defect_orbs(1);
    capacitor.state.hand = make_deck(&["Capacitor+"]);
    assert_eq!(capacitor.state.orb_slots.max_slots, 1);
    assert!(play_self(&mut capacitor, "Capacitor+"));
    assert_eq!(capacitor.state.orb_slots.max_slots, 4);

    let mut chaos = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut chaos);
    chaos.init_defect_orbs(2);
    chaos.state.hand = make_deck(&["Chaos+"]);
    assert!(play_self(&mut chaos, "Chaos+"));
    assert_eq!(chaos.state.orb_slots.occupied_count(), 2);
    for orb in &chaos.state.orb_slots.slots[0..2] {
        assert_ne!(orb.orb_type, OrbType::Empty);
    }
}

#[test]
fn test_card_runtime_defect_wave4_ftl_draw_gate_and_claw_scaling_follow_engine_rules() {
    let mut ftl_draws = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut ftl_draws);
    ftl_draws.state.hand = make_deck(&["FTL+"]);
    ftl_draws.state.draw_pile = make_deck(&["Strike_B", "Defend_B", "Zap", "Dualcast"]);
    assert!(play_on_enemy(&mut ftl_draws, "FTL+", 0));
    assert_eq!(ftl_draws.state.hand.len(), 4);

    let mut ftl_gated = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut ftl_gated);
    ftl_gated.state.cards_played_this_turn = 3;
    ftl_gated.state.hand = make_deck(&["FTL"]);
    ftl_gated.state.draw_pile = make_deck(&["Strike_B", "Defend_B"]);
    assert!(play_on_enemy(&mut ftl_gated, "FTL", 0));
    assert_eq!(ftl_gated.state.hand.len(), 0);

    let mut claw = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    force_player_turn(&mut claw);
    claw.state.hand = make_deck(&["Gash", "Gash"]);
    assert!(play_on_enemy(&mut claw, "Gash", 0));
    assert_eq!(claw.state.player.status(sid::CLAW_BONUS), 2);
    let hp_before = claw.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut claw, "Gash", 0));
    assert_eq!(claw.state.enemies[0].entity.hp, hp_before - 5);
    assert_eq!(claw.state.player.status(sid::CLAW_BONUS), 4);
}
