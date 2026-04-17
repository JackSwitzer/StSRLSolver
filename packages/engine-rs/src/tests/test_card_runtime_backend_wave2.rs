#![cfg(test)]

use crate::card_effects::execute_card_effects;
use crate::cards::{CardDef, CardTarget, CardType};
use crate::effects::declarative::{
    AmountSource as A, ChoiceAction, Effect as E, Pile as P, SimpleEffect as SE, Target as T,
};
use crate::engine::{ChoiceReason, CombatPhase};
use crate::gameplay::{ChoiceCountHint, EffectOp, OrbCountHint};
use crate::orbs::OrbType;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck};

static ALL_ENEMY_DAMAGE_EFFECTS: [E; 1] = [E::Simple(SE::DealDamage(T::AllEnemies, A::Fixed(4)))];
static DISCARD_TWO_EFFECTS: [E; 1] = [E::ChooseCards {
    source: P::Hand,
    filter: crate::effects::declarative::CardFilter::All,
    action: ChoiceAction::Discard,
    min_picks: A::Fixed(2),
    max_picks: A::Fixed(2),
    post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
}];
static EXHAUST_TWO_EFFECTS: [E; 1] = [E::ChooseCards {
    source: P::Hand,
    filter: crate::effects::declarative::CardFilter::All,
    action: ChoiceAction::Exhaust,
    min_picks: A::Fixed(2),
    max_picks: A::Fixed(2),
    post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
}];
static SCRY_THREE_EFFECTS: [E; 1] = [E::Simple(SE::Scry(A::Fixed(3)))];
static CHANNEL_TWO_LIGHTNING_EFFECTS: [E; 1] = [E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Fixed(2)))];
static EVOKE_TWO_EFFECTS: [E; 1] = [E::Simple(SE::EvokeOrb(A::Fixed(2)))];
static X_COST_ENERGY_EFFECTS: [E; 1] = [E::Simple(SE::GainEnergy(A::XCost))];

fn backend_card(
    id: &'static str,
    card_type: CardType,
    target: CardTarget,
    cost: i32,
    base_damage: i32,
    base_block: i32,
    effect_data: &'static [E],
) -> CardDef {
    CardDef {
        id,
        name: id,
        card_type,
        target,
        cost,
        base_damage,
        base_block,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
        metadata: crate::effects::types::CardMetadata::default(),
        effect_data,
        complex_hook: None,
    }
}

#[test]
fn test_card_runtime_backend_wave2_registry_exports_include_extended_declarative_hints() {
    let registry = crate::gameplay::global_registry();

    let consecrate = registry.card("Consecrate").expect("Consecrate export");
    let consecrate_schema = consecrate.card_schema().expect("card schema");
    assert_eq!(consecrate_schema.declared_all_enemy_damage, Some(A::Damage));

    let third_eye = registry.card("ThirdEye").expect("ThirdEye export");
    let third_eye_schema = third_eye.card_schema().expect("card schema");
    assert_eq!(third_eye_schema.declared_scry_count, Some(A::Magic));

    let recycle = registry.card("Recycle").expect("Recycle export");
    let recycle_schema = recycle.card_schema().expect("card schema");
    assert_eq!(
        recycle_schema.declared_exhaust_from_hand,
        Some(ChoiceCountHint {
            min: A::Fixed(1),
            max: A::Fixed(1),
        })
    );

    let doppelganger = registry.card("Doppelganger").expect("Doppelganger export");
    let doppelganger_schema = doppelganger.card_schema().expect("card schema");
    assert_eq!(doppelganger_schema.declared_x_cost_amounts, vec![A::XCost, A::XCost]);

    let multi_cast = registry.card("Multi-Cast").expect("Multi-Cast export");
    let multi_cast_schema = multi_cast.card_schema().expect("card schema");
    assert_eq!(multi_cast_schema.declared_evoke_count, Some(A::XCost));

    let rainbow = registry.card("Rainbow").expect("Rainbow export");
    let rainbow_schema = rainbow.card_schema().expect("card schema");
    assert_eq!(
        rainbow_schema.declared_channel_orbs,
        vec![
            OrbCountHint { orb_type: OrbType::Lightning, count: A::Fixed(1) },
            OrbCountHint { orb_type: OrbType::Frost, count: A::Fixed(1) },
            OrbCountHint { orb_type: OrbType::Dark, count: A::Fixed(1) },
        ]
    );

    let program = rainbow.program();
    assert!(program.steps.iter().any(|step| matches!(
        step,
        EffectOp::PlayCard {
            declared_channel_orbs,
            ..
        } if declared_channel_orbs.len() == 3
    )));
}

#[test]
fn test_card_runtime_backend_wave2_carddef_helpers_extract_shared_declarative_counts() {
    let discard = backend_card(
        "BackendDiscardTwo",
        CardType::Skill,
        CardTarget::None,
        1,
        -1,
        -1,
        &DISCARD_TWO_EFFECTS,
    );
    assert_eq!(discard.declared_discard_from_hand_count(), Some((A::Fixed(2), A::Fixed(2))));

    let exhaust = backend_card(
        "BackendExhaustTwo",
        CardType::Skill,
        CardTarget::None,
        1,
        -1,
        -1,
        &EXHAUST_TWO_EFFECTS,
    );
    assert_eq!(exhaust.declared_exhaust_from_hand_count(), Some((A::Fixed(2), A::Fixed(2))));

    let aoe = backend_card(
        "BackendAoe",
        CardType::Skill,
        CardTarget::None,
        1,
        -1,
        -1,
        &ALL_ENEMY_DAMAGE_EFFECTS,
    );
    assert_eq!(aoe.declared_all_enemy_damage(), Some(A::Fixed(4)));

    let scry = backend_card(
        "BackendScry",
        CardType::Skill,
        CardTarget::SelfTarget,
        1,
        -1,
        -1,
        &SCRY_THREE_EFFECTS,
    );
    assert_eq!(scry.declared_scry_count(), Some(A::Fixed(3)));

    let channel = backend_card(
        "BackendChannel",
        CardType::Skill,
        CardTarget::None,
        1,
        -1,
        -1,
        &CHANNEL_TWO_LIGHTNING_EFFECTS,
    );
    assert_eq!(channel.declared_channel_orbs(), vec![(OrbType::Lightning, A::Fixed(2))]);

    let evoke = backend_card(
        "BackendEvoke",
        CardType::Skill,
        CardTarget::None,
        1,
        -1,
        -1,
        &EVOKE_TWO_EFFECTS,
    );
    assert_eq!(evoke.declared_evoke_count(), Some(A::Fixed(2)));

    let x_cost = backend_card(
        "BackendXCost",
        CardType::Skill,
        CardTarget::None,
        -1,
        -1,
        -1,
        &X_COST_ENERGY_EFFECTS,
    );
    assert_eq!(x_cost.declared_x_cost_amounts(), vec![A::XCost]);
    assert!(x_cost.uses_declared_x_cost());
}

#[test]
fn test_card_runtime_backend_wave2_effect_data_deals_all_enemy_damage_without_tags() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 30, 30),
            enemy_no_intent("Cultist", 25, 25),
        ],
        3,
    );
    force_player_turn(&mut engine);

    let card = backend_card(
        "BackendAllEnemyDamage",
        CardType::Skill,
        CardTarget::None,
        1,
        -1,
        -1,
        &ALL_ENEMY_DAMAGE_EFFECTS,
    );
    let card_inst = engine.card_registry.make_card("Defend_R");
    execute_card_effects(&mut engine, &card, card_inst, -1);

    assert_eq!(engine.state.enemies[0].entity.hp, 26);
    assert_eq!(engine.state.enemies[1].entity.hp, 21);
}

#[test]
fn test_card_runtime_backend_wave2_effect_data_generates_discard_and_exhaust_choices_without_tags() {
    let mut discard_engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 30, 30)], 3);
    force_player_turn(&mut discard_engine);
    discard_engine.state.hand = make_deck(&["Strike_R", "Defend_R", "Bash"]);
    let discard_card = backend_card(
        "BackendDiscardChoice",
        CardType::Skill,
        CardTarget::None,
        1,
        -1,
        -1,
        &DISCARD_TWO_EFFECTS,
    );
    let discard_inst = discard_engine.card_registry.make_card("Defend_R");
    execute_card_effects(&mut discard_engine, &discard_card, discard_inst, -1);
    assert_eq!(discard_engine.phase, CombatPhase::AwaitingChoice);
    let discard_choice = discard_engine.choice.as_ref().expect("discard choice");
    assert_eq!(discard_choice.reason, ChoiceReason::DiscardFromHand);
    assert_eq!(discard_choice.min_picks, 2);
    assert_eq!(discard_choice.max_picks, 2);
    assert_eq!(discard_choice.options.len(), 3);

    let mut exhaust_engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 30, 30)], 3);
    force_player_turn(&mut exhaust_engine);
    exhaust_engine.state.hand = make_deck(&["Strike_R", "Defend_R", "Bash"]);
    let exhaust_card = backend_card(
        "BackendExhaustChoice",
        CardType::Skill,
        CardTarget::None,
        1,
        -1,
        -1,
        &EXHAUST_TWO_EFFECTS,
    );
    let exhaust_inst = exhaust_engine.card_registry.make_card("Defend_R");
    execute_card_effects(&mut exhaust_engine, &exhaust_card, exhaust_inst, -1);
    assert_eq!(exhaust_engine.phase, CombatPhase::AwaitingChoice);
    let exhaust_choice = exhaust_engine.choice.as_ref().expect("exhaust choice");
    assert_eq!(exhaust_choice.reason, ChoiceReason::ExhaustFromHand);
    assert_eq!(exhaust_choice.min_picks, 2);
    assert_eq!(exhaust_choice.max_picks, 2);
    assert_eq!(exhaust_choice.options.len(), 3);
}

#[test]
fn test_card_runtime_backend_wave2_effect_data_scry_orb_and_x_cost_primitives_work_without_tags() {
    let mut scry_engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 30, 30)], 3);
    force_player_turn(&mut scry_engine);
    scry_engine.state.draw_pile = make_deck(&["Strike_R", "Defend_R", "Bash"]);
    let scry_card = backend_card(
        "BackendScryChoice",
        CardType::Skill,
        CardTarget::SelfTarget,
        1,
        -1,
        -1,
        &SCRY_THREE_EFFECTS,
    );
    let scry_inst = scry_engine.card_registry.make_card("Defend_R");
    execute_card_effects(&mut scry_engine, &scry_card, scry_inst, -1);
    assert_eq!(scry_engine.phase, CombatPhase::AwaitingChoice);
    let scry_choice = scry_engine.choice.as_ref().expect("scry choice");
    assert_eq!(scry_choice.reason, ChoiceReason::Scry);
    assert_eq!(scry_choice.max_picks, 3);
    assert_eq!(scry_choice.options.len(), 3);

    let mut orb_engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut orb_engine);
    orb_engine.init_defect_orbs(3);
    let channel_card = backend_card(
        "BackendChannelLightning",
        CardType::Skill,
        CardTarget::None,
        1,
        -1,
        -1,
        &CHANNEL_TWO_LIGHTNING_EFFECTS,
    );
    let channel_inst = orb_engine.card_registry.make_card("Defend_R");
    execute_card_effects(&mut orb_engine, &channel_card, channel_inst, -1);
    assert_eq!(orb_engine.state.orb_slots.occupied_count(), 2);
    assert_eq!(orb_engine.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
    assert_eq!(orb_engine.state.orb_slots.slots[1].orb_type, OrbType::Lightning);

    let evoke_card = backend_card(
        "BackendEvokeLightning",
        CardType::Skill,
        CardTarget::None,
        1,
        -1,
        -1,
        &EVOKE_TWO_EFFECTS,
    );
    let evoke_inst = orb_engine.card_registry.make_card("Defend_R");
    execute_card_effects(&mut orb_engine, &evoke_card, evoke_inst, -1);
    assert_eq!(orb_engine.state.orb_slots.occupied_count(), 0);
    assert_eq!(orb_engine.state.enemies[0].entity.hp, 24);

    let mut x_cost_engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 30, 30)], 4);
    force_player_turn(&mut x_cost_engine);
    x_cost_engine.state.energy = 4;
    let x_cost_card = backend_card(
        "BackendXCostEnergy",
        CardType::Skill,
        CardTarget::None,
        -1,
        -1,
        -1,
        &X_COST_ENERGY_EFFECTS,
    );
    let x_cost_inst = x_cost_engine.card_registry.make_card("Defend_R");
    execute_card_effects(&mut x_cost_engine, &x_cost_card, x_cost_inst, -1);
    assert_eq!(x_cost_engine.state.energy, 4);
}
