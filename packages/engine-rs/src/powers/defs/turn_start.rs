//! Turn-start power definitions.
//!
//! Powers that trigger at the start of the player's turn.

use crate::effects::declarative::{AmountSource, Effect, GeneratedCardPool, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{ChoiceOption, ChoiceReason, CombatEngine, TurnStartQueuedAction};
use crate::state::Stance;
use crate::status_ids::sid;

// ===========================================================================
// Energized — OnEnergyRecharge: gain stored energy, then remove the power
// ===========================================================================

// EnergizedPower and EnergizedBluePower gain their amount during
// onEnergyRecharge and queue removal of the shared "Energized" power.
// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/EnergizedPower.java
// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/EnergizedBluePower.java

static ENERGIZED_EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::GainEnergy(AmountSource::StatusValue(
        sid::ENERGIZED,
    ))),
    Effect::Simple(SimpleEffect::SetStatus(
        Target::Player,
        sid::ENERGIZED,
        AmountSource::Fixed(0),
    )),
];

static ENERGIZED_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::EnergyRecharge,
    condition: TriggerCondition::Always,
    effects: &ENERGIZED_EFFECTS,
    counter: None,
}];

pub static DEF_ENERGIZED: EntityDef = EntityDef {
    id: "energized",
    name: "Energized",
    kind: EntityKind::Power,
    triggers: &ENERGIZED_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::ENERGIZED),
};

static ENERGIZED_BLUE_EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::GainEnergy(AmountSource::StatusValue(
        sid::ENERGIZED_BLUE,
    ))),
    Effect::Simple(SimpleEffect::SetStatus(
        Target::Player,
        sid::ENERGIZED_BLUE,
        AmountSource::Fixed(0),
    )),
];

static ENERGIZED_BLUE_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::EnergyRecharge,
    condition: TriggerCondition::Always,
    effects: &ENERGIZED_BLUE_EFFECTS,
    counter: None,
}];

pub static DEF_ENERGIZED_BLUE: EntityDef = EntityDef {
    id: "energized_blue",
    name: "Energized Blue",
    kind: EntityKind::Power,
    triggers: &ENERGIZED_BLUE_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::ENERGIZED_BLUE),
};

fn hook_energy_down(
    engine: &mut CombatEngine,
    owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if owner == EffectOwner::PlayerPower && event.kind == Trigger::TurnStart {
        let amount = engine.state.player.status(sid::ENERGY_DOWN);
        if engine.is_collecting_turn_start_actions() {
            engine.queue_turn_start_action_bottom(TurnStartQueuedAction::LoseEnergy(amount));
        } else {
            engine.state.energy = (engine.state.energy - amount).max(0);
        }
    }
}

static ENERGY_DOWN_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF_ENERGY_DOWN: EntityDef = EntityDef {
    id: "energy_down",
    name: "Energy Down",
    kind: EntityKind::Power,
    triggers: &ENERGY_DOWN_TRIGGERS,
    complex_hook: Some(hook_energy_down),
    status_guard: Some(sid::ENERGY_DOWN),
};

// ===========================================================================
// Phantasmal — grant one turn of Double Damage, then consume one schedule stack
// ===========================================================================

fn hook_phantasmal(
    engine: &mut CombatEngine,
    owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if owner != EffectOwner::PlayerPower || event.kind != Trigger::TurnStart {
        return;
    }

    let phantasmal = engine.state.player.status(sid::PHANTASMAL);
    if phantasmal <= 0 {
        return;
    }

    // Java: powers/PhantasmalPower.java constructs DoubleDamagePower(amount=1)
    // but passes the full Phantasmal amount as ApplyPowerAction.stackAmount.
    // Thus an absent power starts at one; an existing power gains all pending
    // stacks. Phantasmal itself always loses exactly one stack per turn.
    if engine.is_collecting_turn_start_actions() {
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::ApplyPhantasmal(phantasmal));
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::ReducePlayerPower(
            sid::PHANTASMAL,
            1,
        ));
    } else {
        let double_damage = engine.state.player.status(sid::DOUBLE_DAMAGE);
        engine.state.player.set_status(
            sid::DOUBLE_DAMAGE,
            if double_damage > 0 {
                double_damage + phantasmal
            } else {
                1
            },
        );
        engine
            .state
            .player
            .set_status(sid::PHANTASMAL, phantasmal - 1);
    }
}

static PHANTASMAL_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF_PHANTASMAL: EntityDef = EntityDef {
    id: "phantasmal",
    name: "Phantasmal",
    kind: EntityKind::Power,
    triggers: &PHANTASMAL_TRIGGERS,
    complex_hook: Some(hook_phantasmal),
    status_guard: Some(sid::PHANTASMAL),
};

// ===========================================================================
// Demon Form — post-draw turn start: gain Strength equal to stacks
// ===========================================================================

static DEMON_FORM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    // DemonFormPower overrides atStartOfTurnPostDraw, not atStartOfTurn.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DemonFormPower.java
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_demon_form(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let amount = engine.state.player.status(sid::DEMON_FORM);
    if engine.is_collecting_turn_start_actions() {
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::AddPlayerStatus(
            sid::STRENGTH,
            amount,
        ));
    } else {
        engine.state.player.add_status(sid::STRENGTH, amount);
    }
}

pub static DEF_DEMON_FORM: EntityDef = EntityDef {
    id: "demon_form",
    name: "Demon Form",
    kind: EntityKind::Power,
    triggers: &DEMON_FORM_TRIGGERS,
    complex_hook: Some(hook_demon_form),
    status_guard: Some(sid::DEMON_FORM),
};

// ===========================================================================
// Noxious Fumes — poison all living enemies after the normal turn draw.
// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/NoxiousFumesPower.java
// ===========================================================================

static NOXIOUS_FUMES_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_noxious_fumes(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let amount = engine.state.player.status(sid::NOXIOUS_FUMES);
    for enemy_idx in engine.state.living_enemy_indices() {
        if engine.is_collecting_turn_start_actions() {
            engine.queue_turn_start_action_bottom(TurnStartQueuedAction::AddEnemyStatus(
                enemy_idx,
                sid::POISON,
                amount,
            ));
        } else if let Some(enemy) = engine.state.enemies.get_mut(enemy_idx) {
            enemy.entity.add_status(sid::POISON, amount);
        }
    }
}

pub static DEF_NOXIOUS_FUMES: EntityDef = EntityDef {
    id: "noxious_fumes",
    name: "Noxious Fumes",
    kind: EntityKind::Power,
    triggers: &NOXIOUS_FUMES_TRIGGERS,
    complex_hook: Some(hook_noxious_fumes),
    status_guard: Some(sid::NOXIOUS_FUMES),
};

// ===========================================================================
// Brutality — TurnStartPostDraw: draw 1 card, lose HP equal to stacks
// ===========================================================================

// BrutalityPower.atStartOfTurnPostDraw queues DrawCardAction followed by
// LoseHPAction for its stack amount.
// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BrutalityPower.java
static BRUTALITY_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_brutality(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let amount = engine.state.player.status(sid::BRUTALITY);
    if engine.is_collecting_turn_start_actions() {
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::DrawCards(amount));
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::PlayerLoseHp(amount));
    } else {
        engine.draw_cards(amount);
        engine.player_lose_hp_from_damage(amount);
    }
}

pub static DEF_BRUTALITY: EntityDef = EntityDef {
    id: "brutality",
    name: "Brutality",
    kind: EntityKind::Power,
    triggers: &BRUTALITY_TRIGGERS,
    complex_hook: Some(hook_brutality),
    status_guard: Some(sid::BRUTALITY),
};

// ===========================================================================
// Berserk — TurnStart: gain energy equal to stacks
// ===========================================================================

// Source: powers/BerserkPower.java::atStartOfTurn queues GainEnergyAction for
// the power's stack amount on every turn start.

static BERSERK_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_berserk(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let amount = engine.state.player.status(sid::BERSERK);
    if engine.is_collecting_turn_start_actions() {
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::GainEnergy(amount));
    } else {
        engine.state.energy += amount;
    }
}

pub static DEF_BERSERK: EntityDef = EntityDef {
    id: "berserk",
    name: "Berserk",
    kind: EntityKind::Power,
    triggers: &BERSERK_TRIGGERS,
    complex_hook: Some(hook_berserk),
    status_guard: Some(sid::BERSERK),
};

// ===========================================================================
// Infinite Blades — TurnStart: add Shiv(s) to hand
// ===========================================================================

// Source: powers/InfiniteBladesPower.java::atStartOfTurn creates `amount`
// unupgraded Shivs, and stackPower adds each new card's one stack.

static INFINITE_BLADES_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_infinite_blades(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let amount = engine.state.player.status(sid::INFINITE_BLADES);
    if engine.is_collecting_turn_start_actions() {
        for _ in 0..amount {
            let card = engine.temp_card("Shiv");
            engine.queue_turn_start_action_bottom(TurnStartQueuedAction::AddCardToHand(card));
        }
    } else {
        engine.add_temp_cards_to_hand("Shiv", amount);
    }
}

pub static DEF_INFINITE_BLADES: EntityDef = EntityDef {
    id: "infinite_blades",
    name: "Infinite Blades",
    kind: EntityKind::Power,
    triggers: &INFINITE_BLADES_TRIGGERS,
    complex_hook: Some(hook_infinite_blades),
    status_guard: Some(sid::INFINITE_BLADES),
};

// ===========================================================================
// Battle Hymn — TurnStart: add Smite(s) to hand
// ===========================================================================

static BATTLE_HYMN_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_battle_hymn(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let amount = engine.state.player.status(sid::BATTLE_HYMN);
    if engine.is_collecting_turn_start_actions() {
        for _ in 0..amount {
            let card = engine.temp_card("Smite");
            engine.queue_turn_start_action_bottom(TurnStartQueuedAction::AddCardToHand(card));
        }
    } else {
        engine.add_temp_cards_to_hand("Smite", amount);
    }
}

pub static DEF_BATTLE_HYMN: EntityDef = EntityDef {
    id: "battle_hymn",
    name: "Battle Hymn",
    kind: EntityKind::Power,
    triggers: &BATTLE_HYMN_TRIGGERS,
    complex_hook: Some(hook_battle_hymn),
    status_guard: Some(sid::BATTLE_HYMN),
};

// ===========================================================================
// Devotion — TurnStartPostDraw: gain mantra equal to stacks, except Java
// enters Divinity directly when no MantraPower exists and amount >= 10.
// Java: powers/watcher/DevotionPower.java atStartOfTurnPostDraw().
// ===========================================================================

static DEVOTION_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_devotion(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let amount = engine.state.player.status(sid::DEVOTION);
    if engine.is_collecting_turn_start_actions() {
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::GainDevotion(amount));
    } else if engine.state.mantra == 0 && amount >= 10 {
        engine.change_stance(Stance::Divinity);
    } else {
        engine.gain_mantra(amount);
    }
}

pub static DEF_DEVOTION: EntityDef = EntityDef {
    id: "devotion",
    name: "Devotion",
    kind: EntityKind::Power,
    triggers: &DEVOTION_TRIGGERS,
    complex_hook: Some(hook_devotion),
    status_guard: Some(sid::DEVOTION),
};

// ===========================================================================
// Deva Form — TurnStart: gain energy (escalating)
// Java DevaPower keeps amount and energyGainAmount separate: recharge grants
// energyGainAmount, then increments it by amount.
// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/DevaPower.java
// ===========================================================================

static DEVA_FORM_EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::GainEnergy(AmountSource::StatusValue(
        sid::DEVA_FORM_ENERGY,
    ))),
    // Escalate the hidden energy counter by the stable visible stack amount.
    Effect::Simple(SimpleEffect::AddStatus(
        Target::Player,
        sid::DEVA_FORM_ENERGY,
        AmountSource::StatusValue(sid::DEVA_FORM),
    )),
];

static DEVA_FORM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::EnergyRecharge,
    condition: TriggerCondition::Always,
    effects: &DEVA_FORM_EFFECTS,
    counter: None,
}];

pub static DEF_DEVA_FORM: EntityDef = EntityDef {
    id: "deva_form",
    name: "Deva Form",
    kind: EntityKind::Power,
    triggers: &DEVA_FORM_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::DEVA_FORM),
};

// ===========================================================================
// Hello World — atStartOfTurn: add one random common card per stack
// ===========================================================================

static HELLO_WORLD_EFFECTS: [Effect; 0] = [];

static HELLO_WORLD_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &HELLO_WORLD_EFFECTS,
    counter: None,
}];

fn hook_hello_world(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    // HelloPower.atStartOfTurn calls getCard(COMMON, cardRandomRng) once per
    // stack and queues a MakeTempCardInHandAction for each base copy. The temp
    // action applies Master Reality and spills cards past the hand cap.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/HelloPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
    let hello_world = engine.state.player.status(sid::HELLO_WORLD);
    for _ in 0..hello_world {
        let Some(card) = crate::effects::interpreter::generate_random_card(
            engine,
            GeneratedCardPool::DefectCommon,
        ) else {
            continue;
        };
        if engine.is_collecting_turn_start_actions() {
            engine.queue_turn_start_action_bottom(TurnStartQueuedAction::AddCardToHand(card));
        } else if engine.state.hand.len() < 10 {
            engine.state.hand.push(card);
        } else {
            engine.state.discard_pile.push(card);
        }
    }
}

pub static DEF_HELLO_WORLD: EntityDef = EntityDef {
    id: "hello_world",
    name: "Hello World",
    kind: EntityKind::Power,
    triggers: &HELLO_WORLD_TRIGGERS,
    complex_hook: Some(hook_hello_world),
    status_guard: Some(sid::HELLO_WORLD),
};

// ===========================================================================
// Magnetism — TurnStart: add one random Colorless card per stack
// ===========================================================================

static MAGNETISM_EFFECTS: [Effect; 0] = [];

static MAGNETISM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &MAGNETISM_EFFECTS,
    counter: None,
}];

fn hook_magnetism(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    // MagnetismPower.atStartOfTurn calls returnTrulyRandomColorlessCardInCombat
    // once per stack, then queues one MakeTempCardInHandAction per base copy.
    // That selection consumes cardRandomRng and hand overflow goes to discard.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/MagnetismPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
    let magnetism = engine.state.player.status(sid::MAGNETISM);
    for _ in 0..magnetism {
        let Some(card) =
            crate::effects::interpreter::generate_random_card(engine, GeneratedCardPool::Colorless)
        else {
            continue;
        };
        if engine.is_collecting_turn_start_actions() {
            engine.queue_turn_start_action_bottom(TurnStartQueuedAction::AddCardToHand(card));
        } else if engine.state.hand.len() < 10 {
            engine.state.hand.push(card);
        } else {
            engine.state.discard_pile.push(card);
        }
    }
}

pub static DEF_MAGNETISM: EntityDef = EntityDef {
    id: "magnetism",
    name: "Magnetism",
    kind: EntityKind::Power,
    triggers: &MAGNETISM_TRIGGERS,
    complex_hook: Some(hook_magnetism),
    status_guard: Some(sid::MAGNETISM),
};

// ===========================================================================
// Creative AI — atStartOfTurn: add random Defect Power cards before normal draw
// ===========================================================================

static EMPTY_EFFECTS: [Effect; 0] = [];

static CREATIVE_AI_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_creative_ai(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    // CreativeAIPower.atStartOfTurn selects one non-healing source-pool Power
    // per stack through cardRandomRng, then queues one MakeTempCardInHandAction
    // per selection. Those actions resolve before normal draw and spill past
    // the ten-card hand limit into discard.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CreativeAIPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
    let creative_ai = engine.state.player.status(sid::CREATIVE_AI);
    for _ in 0..creative_ai {
        let Some(card) = crate::effects::interpreter::generate_random_card(
            engine,
            GeneratedCardPool::DefectPower,
        ) else {
            continue;
        };
        if engine.is_collecting_turn_start_actions() {
            engine.queue_turn_start_action_bottom(TurnStartQueuedAction::AddCardToHand(card));
        } else if engine.state.hand.len() < 10 {
            engine.state.hand.push(card);
        } else {
            engine.state.discard_pile.push(card);
        }
    }
}

pub static DEF_CREATIVE_AI: EntityDef = EntityDef {
    id: "creative_ai",
    name: "Creative AI",
    kind: EntityKind::Power,
    triggers: &CREATIVE_AI_TRIGGERS,
    complex_hook: Some(hook_creative_ai),
    status_guard: Some(sid::CREATIVE_AI),
};

// ===========================================================================
// Doppelganger Draw — TurnStart: draw N cards (one-shot, consumed)
// ===========================================================================

static DOPPELGANGER_DRAW_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_doppelganger_draw(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let amount = engine.state.player.status(sid::DOPPELGANGER_DRAW);
    if engine.is_collecting_turn_start_actions() {
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::DrawCards(amount));
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::RemovePlayerPower(
            sid::DOPPELGANGER_DRAW,
        ));
    } else {
        engine.draw_cards(amount);
        engine.state.player.set_status(sid::DOPPELGANGER_DRAW, 0);
    }
}

pub static DEF_DOPPELGANGER_DRAW: EntityDef = EntityDef {
    id: "doppelganger_draw",
    name: "Doppelganger (Draw)",
    kind: EntityKind::Power,
    triggers: &DOPPELGANGER_DRAW_TRIGGERS,
    complex_hook: Some(hook_doppelganger_draw),
    status_guard: Some(sid::DOPPELGANGER_DRAW),
};

// ===========================================================================
// Doppelganger Energy — TurnStart: gain N energy (one-shot, consumed)
// ===========================================================================

static DOPPELGANGER_ENERGY_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_doppelganger_energy(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let amount = engine.state.player.status(sid::DOPPELGANGER_ENERGY);
    if engine.is_collecting_turn_start_actions() {
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::GainEnergy(amount));
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::RemovePlayerPower(
            sid::DOPPELGANGER_ENERGY,
        ));
    } else {
        engine.state.energy += amount;
        engine.state.player.set_status(sid::DOPPELGANGER_ENERGY, 0);
    }
}

pub static DEF_DOPPELGANGER_ENERGY: EntityDef = EntityDef {
    id: "doppelganger_energy",
    name: "Doppelganger (Energy)",
    kind: EntityKind::Power,
    triggers: &DOPPELGANGER_ENERGY_TRIGGERS,
    complex_hook: Some(hook_doppelganger_energy),
    status_guard: Some(sid::DOPPELGANGER_ENERGY),
};

// ===========================================================================
// Draw Card Next Turn — priority-20 post-draw callback.
// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DrawCardNextTurnPower.java
// ===========================================================================

static DRAW_CARD_NEXT_TURN_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_draw_card_next_turn(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let amount = engine.state.player.status(sid::DRAW_CARD);
    if engine.is_collecting_turn_start_actions() {
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::DrawCards(amount));
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::RemovePlayerPower(
            sid::DRAW_CARD,
        ));
    } else {
        engine.draw_cards(amount);
        engine.state.player.set_status(sid::DRAW_CARD, 0);
    }
}

pub static DEF_DRAW_CARD_NEXT_TURN: EntityDef = EntityDef {
    id: "draw_card_next_turn",
    name: "Draw Card Next Turn",
    kind: EntityKind::Power,
    triggers: &DRAW_CARD_NEXT_TURN_TRIGGERS,
    complex_hook: Some(hook_draw_card_next_turn),
    status_guard: Some(sid::DRAW_CARD),
};

// ===========================================================================
// Foresight — atStartOfTurn queues EmptyDeckShuffleAction to top when needed,
// then ScryAction to bottom before the ordinary turn draw is appended.
// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/ForesightPower.java
// ===========================================================================

static FORESIGHT_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_foresight(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let amount = engine.state.player.status(sid::FORESIGHT);
    if engine.is_collecting_turn_start_actions() {
        if engine.state.draw_pile.is_empty() {
            engine.queue_turn_start_action_top(TurnStartQueuedAction::ShuffleDrawPile);
        }
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::Scry(amount));
    } else {
        engine.do_scry(amount);
    }
}

pub static DEF_FORESIGHT: EntityDef = EntityDef {
    id: "foresight",
    name: "Foresight",
    kind: EntityKind::Power,
    triggers: &FORESIGHT_TRIGGERS,
    complex_hook: Some(hook_foresight),
    status_guard: Some(sid::FORESIGHT),
};

// ===========================================================================
// Enter Divinity — TurnStartPostDraw: enter Divinity stance (one-shot flag)
// ===========================================================================

static ENTER_DIVINITY_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_enter_divinity(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    if engine.state.player.status(sid::ENTER_DIVINITY) > 0 {
        if engine.is_collecting_turn_start_actions() {
            engine.queue_turn_start_action_bottom(TurnStartQueuedAction::EnterDivinity);
        } else {
            engine.state.player.set_status(sid::ENTER_DIVINITY, 0);
            engine.change_stance(Stance::Divinity);
        }
    }
}

pub static DEF_ENTER_DIVINITY: EntityDef = EntityDef {
    id: "enter_divinity",
    name: "Enter Divinity",
    kind: EntityKind::Power,
    triggers: &ENTER_DIVINITY_TRIGGERS,
    complex_hook: Some(hook_enter_divinity),
    status_guard: Some(sid::ENTER_DIVINITY),
};

// ===========================================================================
// Mayhem — atStartOfTurn queues wrappers before the ordinary draw.
// ===========================================================================

static MAYHEM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_mayhem(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let mayhem = engine.state.player.status(sid::MAYHEM);
    if engine.is_collecting_turn_start_actions() {
        for _ in 0..mayhem {
            engine.queue_turn_start_action_bottom(TurnStartQueuedAction::MayhemWrapper);
        }
        return;
    }
    // MayhemPower queues all wrapper actions before any PlayTopCardAction runs,
    // so every stack selects its target first. Each selection consumes
    // cardRandomRng even with one living monster. Unlike Havoc, exhausts=false.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/MayhemPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/PlayTopCardAction.java
    let targets: Vec<i32> = (0..mayhem)
        .map(|_| engine.random_living_enemy().map_or(-1, |idx| idx as i32))
        .collect();
    for target_idx in targets {
        if engine.state.combat_over {
            break;
        }
        engine.play_top_card_of_draw_at_target(target_idx, false);
    }
}

pub static DEF_MAYHEM: EntityDef = EntityDef {
    id: "mayhem",
    name: "Mayhem",
    kind: EntityKind::Power,
    triggers: &MAYHEM_TRIGGERS,
    complex_hook: Some(hook_mayhem),
    status_guard: Some(sid::MAYHEM),
};

// ===========================================================================
// Tools of the Trade — TurnStartPostDraw: draw N, then discard N
// ===========================================================================

static TOOLS_OF_THE_TRADE_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_tools_of_the_trade(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let tott = engine.state.player.status(sid::TOOLS_OF_THE_TRADE);
    if tott <= 0 {
        return;
    }

    if engine.is_collecting_turn_start_actions() {
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::DrawCards(tott));
        engine
            .queue_turn_start_action_bottom(TurnStartQueuedAction::DiscardFromHand(tott as usize));
        return;
    }

    engine.draw_cards(tott);
    if engine.state.hand.is_empty() {
        return;
    }

    let discard_count = (tott as usize).min(engine.state.hand.len());
    if engine.state.hand.len() <= discard_count {
        // DiscardAction auto-discards the whole hand from the top when its
        // size is at most amount and fires manual-discard hooks for every card.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DiscardAction.java
        while let Some(card) = engine.state.hand.pop() {
            engine.state.discard_pile.push(card);
            engine.on_card_discarded(card);
        }
        return;
    }

    let options: Vec<ChoiceOption> = (0..engine.state.hand.len())
        .map(ChoiceOption::HandCard)
        .collect();
    engine.begin_choice(
        ChoiceReason::DiscardFromHand,
        options,
        discard_count,
        discard_count,
    );
}

pub static DEF_TOOLS_OF_THE_TRADE: EntityDef = EntityDef {
    id: "tools_of_the_trade",
    name: "Tools of the Trade",
    kind: EntityKind::Power,
    triggers: &TOOLS_OF_THE_TRADE_TRIGGERS,
    complex_hook: Some(hook_tools_of_the_trade),
    status_guard: Some(sid::TOOLS_OF_THE_TRADE),
};

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demon_form_def() {
        assert_eq!(DEF_DEMON_FORM.triggers.len(), 1);
        assert_eq!(
            DEF_DEMON_FORM.triggers[0].trigger,
            Trigger::TurnStartPostDraw
        );
        assert_eq!(
            DEF_DEMON_FORM.triggers[0].condition,
            TriggerCondition::Always
        );
        assert!(DEF_DEMON_FORM.complex_hook.is_some());
    }

    #[test]
    fn test_brutality_uses_a_queue_aware_hook() {
        assert!(DEF_BRUTALITY.triggers[0].effects.is_empty());
        assert!(DEF_BRUTALITY.complex_hook.is_some());
    }

    #[test]
    fn test_all_simple_turn_start_defs_have_correct_trigger() {
        let defs = [
            &DEF_BERSERK,
            &DEF_INFINITE_BLADES,
            &DEF_BATTLE_HYMN,
            &DEF_HELLO_WORLD,
            &DEF_MAGNETISM,
            &DEF_DOPPELGANGER_DRAW,
            &DEF_DOPPELGANGER_ENERGY,
        ];
        for def in &defs {
            assert_eq!(def.kind, EntityKind::Power);
            assert!(!def.triggers.is_empty());
            assert_eq!(def.triggers[0].trigger, Trigger::TurnStart);
        }
        // These callbacks belong to PlayerTurnEffect/GainEnergyAndEnableControls,
        // not AbstractCreature.applyStartOfTurnPowers.
        // Java: EnergizedPower.java and watcher/DevaPower.java.
        assert_eq!(DEF_ENERGIZED.triggers[0].trigger, Trigger::EnergyRecharge);
        assert_eq!(DEF_DEVA_FORM.triggers[0].trigger, Trigger::EnergyRecharge);
        // These powers override atStartOfTurnPostDraw, not atStartOfTurn.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BrutalityPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/NoxiousFumesPower.java
        assert_eq!(
            DEF_BRUTALITY.triggers[0].trigger,
            Trigger::TurnStartPostDraw
        );
        assert_eq!(
            DEF_DEMON_FORM.triggers[0].trigger,
            Trigger::TurnStartPostDraw
        );
        assert_eq!(DEF_DEVOTION.triggers[0].trigger, Trigger::TurnStartPostDraw);
        assert_eq!(
            DEF_NOXIOUS_FUMES.triggers[0].trigger,
            Trigger::TurnStartPostDraw
        );
    }

    #[test]
    fn test_complex_turn_start_defs_have_hooks() {
        // CreativeAIPower overrides atStartOfTurn, while the other complex
        // powers here override atStartOfTurnPostDraw.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CreativeAIPower.java
        assert_eq!(DEF_CREATIVE_AI.kind, EntityKind::Power);
        assert!(DEF_CREATIVE_AI.complex_hook.is_some());
        assert_eq!(DEF_CREATIVE_AI.triggers[0].trigger, Trigger::TurnStart);

        for def in [&DEF_ENTER_DIVINITY, &DEF_TOOLS_OF_THE_TRADE] {
            assert_eq!(def.kind, EntityKind::Power);
            assert!(def.complex_hook.is_some());
            assert_eq!(def.triggers.len(), 1);
            assert_eq!(def.triggers[0].trigger, Trigger::TurnStartPostDraw);
        }
        // MayhemPower overrides atStartOfTurn and queues MayhemAction before
        // the ordinary turn draw.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/MayhemPower.java.
        assert!(DEF_MAYHEM.complex_hook.is_some());
        assert_eq!(DEF_MAYHEM.triggers[0].trigger, Trigger::TurnStart);
    }
}

#[cfg(test)]
#[path = "../../tests/test_power_runtime_turn_start.rs"]
mod test_power_runtime_turn_start;
