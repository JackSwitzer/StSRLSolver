#![cfg(test)]

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{
    AmountSource as A, Condition as Cond, Effect as E, SimpleEffect as SE, Target as T,
};
use crate::effects::types::CardRuntimeTraits;
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::{
    enemy_no_intent, engine_with, engine_without_start, exhaust_prefix_count, force_player_turn,
    hand_count, make_deck, make_deck_n, play_on_enemy, play_self,
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
    assert_eq!(
        boot.runtime_traits(),
        CardRuntimeTraits { innate: true, ..CardRuntimeTraits::default() }
    );
    assert_eq!(boot.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let defend = reg.get("Defend").expect("Defend");
    assert_eq!(defend.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);
    assert!(defend.complex_hook.is_none());

    let buffer = reg.get("Buffer").expect("Buffer");
    assert_eq!(
        buffer.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::BUFFER, A::Magic))]
    );
    assert!(buffer.complex_hook.is_none());

    let chaos = reg.get("Chaos").expect("Chaos");
    assert_eq!(
        chaos.effect_data,
        &[E::Simple(SE::ChannelRandomOrb(A::Magic))]
    );
    assert!(chaos.complex_hook.is_none());

    let ftl = reg.get("FTL").expect("FTL");
    assert_eq!(
        ftl.effect_data,
        &[
            E::Conditional(
                Cond::CardsPlayedThisTurnLessThan(4),
                &[E::Simple(SE::DrawCards(A::Fixed(1)))],
                &[],
            ),
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ]
    );
    assert!(ftl.complex_hook.is_none());

    let claw = reg.get("Gash").expect("Gash");
    assert_eq!(
        claw.effect_data,
        &[E::Simple(SE::IncreaseAllClawDamage(A::Magic))]
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
    assert_eq!(capacitor.declared_effect_count, 1);
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
    defend.state.hand = make_deck(&["Defend+"]);
    assert!(play_self(&mut defend, "Defend+"));
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
fn chill_channels_per_living_enemy_and_upgrade_is_innate_only() {
    // Chill.java counts monsters for which isDeadOrEscaped is false, channels
    // count * magicNumber Frost, and Exhausts. upgrade() changes only isInnate.
    let registry = global_registry();
    assert!(!registry.get("Chill").expect("Chill").runtime_traits().innate);
    assert!(registry.get("Chill+").expect("Chill+").runtime_traits().innate);

    let mut deck = make_deck_n("Defend", 9);
    deck.push(registry.make_card("Chill+"));
    let opening = engine_with(deck, 40, 0);
    assert_eq!(hand_count(&opening, "Chill+"), 1);

    let mut enemies = vec![
        enemy_no_intent("JawWorm", 40, 40),
        enemy_no_intent("Cultist", 40, 40),
        enemy_no_intent("Dead", 0, 40),
    ];
    enemies[2].entity.hp = 0;
    let mut count = engine_without_start(Vec::new(), enemies, 0);
    force_player_turn(&mut count);
    count.init_defect_orbs(3);
    count.state.hand = make_deck(&["Chill"]);
    assert!(play_self(&mut count, "Chill"));
    assert_eq!(count.state.orb_slots.occupied_count(), 2);
    assert!(count.state.orb_slots.slots[0..2]
        .iter()
        .all(|orb| orb.orb_type == OrbType::Frost));
    assert_eq!(exhaust_prefix_count(&count, "Chill"), 1);
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
    ftl_draws.state.draw_pile = make_deck(&["Strike", "Defend", "Zap", "Dualcast"]);
    assert!(play_on_enemy(&mut ftl_draws, "FTL+", 0));
    assert_eq!(ftl_draws.state.hand.len(), 1);

    let mut ftl_gated = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut ftl_gated);
    ftl_gated.state.cards_played_this_turn = 3;
    ftl_gated.state.hand = make_deck(&["FTL"]);
    ftl_gated.state.draw_pile = make_deck(&["Strike", "Defend"]);
    assert!(play_on_enemy(&mut ftl_gated, "FTL", 0));
    assert_eq!(ftl_gated.state.hand.len(), 0);

    let mut claw = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    force_player_turn(&mut claw);
    // GashAction increases the played instance and Claws in hand/draw/discard
    // after damage. Exhausted and subsequently created Claws are untouched.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Claw.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/GashAction.java
    claw.state.hand = make_deck(&["Gash", "Gash+"]);
    claw.state.draw_pile = make_deck(&["Gash"]);
    claw.state.discard_pile = make_deck(&["Gash+"]);
    claw.state.exhaust_pile = make_deck(&["Gash"]);

    assert!(play_on_enemy(&mut claw, "Gash", 0));
    assert_eq!(claw.state.enemies[0].entity.hp, 57);
    assert_eq!(claw.state.hand[0].misc, 7);
    assert_eq!(claw.state.draw_pile[0].misc, 5);
    assert_eq!(claw.state.exhaust_pile[0].misc, -1);
    assert!(claw.state.discard_pile.iter().any(|card| {
        claw.card_registry.card_def_by_id(card.def_id).id == "Gash+" && card.misc == 7
    }));
    assert!(claw.state.discard_pile.iter().any(|card| {
        claw.card_registry.card_def_by_id(card.def_id).id == "Gash" && card.misc == 5
    }));

    claw.state.hand.push(claw.card_registry.make_card("Gash"));
    let hp_before = claw.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut claw, "Gash", 0));
    assert_eq!(claw.state.enemies[0].entity.hp, hp_before - 3);
}
