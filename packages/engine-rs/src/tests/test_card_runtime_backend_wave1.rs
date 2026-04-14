#![cfg(test)]

use crate::card_effects::execute_card_effects;
use crate::cards::{CardDef, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};
use crate::gameplay::EffectOp;
use crate::state::Stance;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn};

static TRIPLE_HIT_EFFECTS: [E; 1] = [E::ExtraHits(A::Fixed(3))];
static X_COST_HIT_EFFECTS: [E; 1] = [E::ExtraHits(A::XCost)];
static WRATH_BLOCK_EFFECTS: [E; 1] = [E::Simple(SE::ChangeStance(Stance::Wrath))];

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
        effects: &[],
        effect_data,
        complex_hook: None,
    }
}

#[test]
fn test_card_runtime_backend_wave1_registry_exports_include_declarative_card_hints() {
    let registry = crate::gameplay::global_registry();

    let riddle = registry.card("Riddle with Holes").expect("Riddle with Holes export");
    let riddle_schema = riddle.card_schema().expect("card schema");
    assert_eq!(riddle_schema.declared_effect_count, 1);
    assert!(riddle_schema.declared_extra_hits);
    assert!(!riddle_schema.declared_stance_change);
    assert!(!riddle_schema.uses_x_cost);

    let tantrum = registry.card("Tantrum").expect("Tantrum export");
    let tantrum_schema = tantrum.card_schema().expect("card schema");
    assert!(tantrum_schema.declared_stance_change);

    let doppelganger = registry.card("Doppelganger").expect("Doppelganger export");
    let doppelganger_schema = doppelganger.card_schema().expect("card schema");
    assert!(doppelganger_schema.uses_x_cost);

    let program = riddle.program();
    assert!(program.steps.iter().any(|step| matches!(
        step,
        EffectOp::PlayCard {
            declared_effect_count: 1,
            declared_extra_hits: true,
            declared_stance_change: false,
            uses_x_cost: false,
            ..
        }
    )));
}

#[test]
fn test_card_runtime_backend_wave1_effect_data_drives_extra_hits_without_string_tags() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);

    let card = backend_card(
        "BackendTripleHit",
        CardType::Attack,
        CardTarget::Enemy,
        1,
        5,
        -1,
        &TRIPLE_HIT_EFFECTS,
    );

    let card_inst = engine.card_registry.make_card("Strike_R");
    execute_card_effects(&mut engine, &card, card_inst, 0);

    assert_eq!(engine.state.enemies[0].entity.hp, 25);
}

#[test]
fn test_card_runtime_backend_wave1_effect_data_drives_x_cost_hits_without_string_tags() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.energy = 3;

    let card = backend_card(
        "BackendXCoster",
        CardType::Attack,
        CardTarget::Enemy,
        -1,
        4,
        -1,
        &X_COST_HIT_EFFECTS,
    );

    let card_inst = engine.card_registry.make_card("Strike_R");
    execute_card_effects(&mut engine, &card, card_inst, 0);

    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 28);
}

#[test]
fn test_card_runtime_backend_wave1_effect_data_handles_stance_change_alongside_base_block() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);

    let card = backend_card(
        "BackendWrathBlock",
        CardType::Skill,
        CardTarget::SelfTarget,
        1,
        -1,
        7,
        &WRATH_BLOCK_EFFECTS,
    );

    let card_inst = engine.card_registry.make_card("Defend_R");
    execute_card_effects(&mut engine, &card, card_inst, -1);

    assert_eq!(engine.state.player.block, 7);
    assert_eq!(engine.state.stance, Stance::Wrath);
}
