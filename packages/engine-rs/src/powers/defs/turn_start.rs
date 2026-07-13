//! Turn-start power definitions.
//!
//! Powers that trigger at the start of the player's turn.

use crate::effects::declarative::{
    AmountSource, Effect, GeneratedCardPool, Pile, SimpleEffect, Target,
};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{ChoiceOption, ChoiceReason, CombatEngine};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
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
    trigger: Trigger::TurnStart,
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

fn hook_energy_down(
    engine: &mut CombatEngine,
    owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if owner == EffectOwner::PlayerPower && event.kind == Trigger::TurnStart {
        let amount = engine.state.player.status(sid::ENERGY_DOWN);
        engine.state.energy = (engine.state.energy - amount).max(0);
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
    let double_damage = engine.state.player.status(sid::DOUBLE_DAMAGE);
    if double_damage > 0 {
        engine
            .state
            .player
            .set_status(sid::DOUBLE_DAMAGE, double_damage + phantasmal);
    } else {
        engine.state.player.set_status(sid::DOUBLE_DAMAGE, 1);
    }
    engine
        .state
        .player
        .set_status(sid::PHANTASMAL, phantasmal - 1);
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

static DEMON_FORM_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddStatus(
    Target::Player,
    sid::STRENGTH,
    AmountSource::StatusValue(sid::DEMON_FORM),
))];

static DEMON_FORM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    // DemonFormPower overrides atStartOfTurnPostDraw, not atStartOfTurn.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DemonFormPower.java
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &DEMON_FORM_EFFECTS,
    counter: None,
}];

pub static DEF_DEMON_FORM: EntityDef = EntityDef {
    id: "demon_form",
    name: "Demon Form",
    kind: EntityKind::Power,
    triggers: &DEMON_FORM_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::DEMON_FORM),
};

// ===========================================================================
// Noxious Fumes — poison all living enemies after the normal turn draw.
// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/NoxiousFumesPower.java
// ===========================================================================

static NOXIOUS_FUMES_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddStatus(
    Target::AllEnemies,
    sid::POISON,
    AmountSource::StatusValue(sid::NOXIOUS_FUMES),
))];

static NOXIOUS_FUMES_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &NOXIOUS_FUMES_EFFECTS,
    counter: None,
}];

pub static DEF_NOXIOUS_FUMES: EntityDef = EntityDef {
    id: "noxious_fumes",
    name: "Noxious Fumes",
    kind: EntityKind::Power,
    triggers: &NOXIOUS_FUMES_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::NOXIOUS_FUMES),
};

// ===========================================================================
// Brutality — TurnStartPostDraw: draw 1 card, lose HP equal to stacks
// ===========================================================================

// BrutalityPower.atStartOfTurnPostDraw queues DrawCardAction followed by
// LoseHPAction for its stack amount.
// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BrutalityPower.java
static BRUTALITY_EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::DrawCards(AmountSource::StatusValue(sid::BRUTALITY))),
    Effect::Simple(SimpleEffect::DealDamage(Target::Player, AmountSource::StatusValue(sid::BRUTALITY))),
];

static BRUTALITY_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &BRUTALITY_EFFECTS,
    counter: None,
}];

pub static DEF_BRUTALITY: EntityDef = EntityDef {
    id: "brutality",
    name: "Brutality",
    kind: EntityKind::Power,
    triggers: &BRUTALITY_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::BRUTALITY),
};

// ===========================================================================
// Berserk — TurnStart: gain energy equal to stacks
// ===========================================================================

// Source: powers/BerserkPower.java::atStartOfTurn queues GainEnergyAction for
// the power's stack amount on every turn start.

static BERSERK_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainEnergy(
    AmountSource::StatusValue(sid::BERSERK),
))];

static BERSERK_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &BERSERK_EFFECTS,
    counter: None,
}];

pub static DEF_BERSERK: EntityDef = EntityDef {
    id: "berserk",
    name: "Berserk",
    kind: EntityKind::Power,
    triggers: &BERSERK_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::BERSERK),
};

// ===========================================================================
// Infinite Blades — TurnStart: add Shiv(s) to hand
// ===========================================================================

// Source: powers/InfiniteBladesPower.java::atStartOfTurn creates `amount`
// unupgraded Shivs, and stackPower adds each new card's one stack.

static INFINITE_BLADES_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddCard(
    "Shiv",
    Pile::Hand,
    AmountSource::StatusValue(sid::INFINITE_BLADES),
))];

static INFINITE_BLADES_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &INFINITE_BLADES_EFFECTS,
    counter: None,
}];

pub static DEF_INFINITE_BLADES: EntityDef = EntityDef {
    id: "infinite_blades",
    name: "Infinite Blades",
    kind: EntityKind::Power,
    triggers: &INFINITE_BLADES_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::INFINITE_BLADES),
};

// ===========================================================================
// Battle Hymn — TurnStart: add Smite(s) to hand
// ===========================================================================

static BATTLE_HYMN_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddCard(
    "Smite",
    Pile::Hand,
    AmountSource::StatusValue(sid::BATTLE_HYMN),
))];

static BATTLE_HYMN_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &BATTLE_HYMN_EFFECTS,
    counter: None,
}];

pub static DEF_BATTLE_HYMN: EntityDef = EntityDef {
    id: "battle_hymn",
    name: "Battle Hymn",
    kind: EntityKind::Power,
    triggers: &BATTLE_HYMN_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::BATTLE_HYMN),
};

// ===========================================================================
// Devotion — TurnStartPostDraw: gain mantra equal to stacks
// Java: powers/watcher/DevotionPower.java atStartOfTurnPostDraw().
// ===========================================================================

static DEVOTION_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainMantra(
    AmountSource::StatusValue(sid::DEVOTION),
))];

static DEVOTION_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &DEVOTION_EFFECTS,
    counter: None,
}];

pub static DEF_DEVOTION: EntityDef = EntityDef {
    id: "devotion",
    name: "Devotion",
    kind: EntityKind::Power,
    triggers: &DEVOTION_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::DEVOTION),
};

// ===========================================================================
// Wraith Form — TurnStart: lose 1 Dexterity each turn
// ===========================================================================

static WRAITH_FORM_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddStatus(
    Target::Player,
    sid::DEXTERITY,
    AmountSource::Fixed(-1),
))];

static WRAITH_FORM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &WRAITH_FORM_EFFECTS,
    counter: None,
}];

pub static DEF_WRAITH_FORM: EntityDef = EntityDef {
    id: "wraith_form",
    name: "Wraith Form",
    kind: EntityKind::Power,
    triggers: &WRAITH_FORM_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::WRAITH_FORM),
};

// ===========================================================================
// Deva Form — TurnStart: gain energy (escalating)
// Java DevaPower keeps amount and energyGainAmount separate: recharge grants
// energyGainAmount, then increments it by amount.
// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/DevaPower.java
// ===========================================================================

static DEVA_FORM_EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::GainEnergy(
        AmountSource::StatusValue(sid::DEVA_FORM_ENERGY),
    )),
    // Escalate the hidden energy counter by the stable visible stack amount.
    Effect::Simple(SimpleEffect::AddStatus(
        Target::Player,
        sid::DEVA_FORM_ENERGY,
        AmountSource::StatusValue(sid::DEVA_FORM),
    )),
];

static DEVA_FORM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
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
        if engine.state.hand.len() < 10 {
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
        let Some(card) = crate::effects::interpreter::generate_random_card(
            engine,
            GeneratedCardPool::Colorless,
        ) else {
            continue;
        };
        if engine.state.hand.len() < 10 {
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
        if engine.state.hand.len() < 10 {
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

static DOPPELGANGER_DRAW_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::DrawCards(
    AmountSource::StatusValue(sid::DOPPELGANGER_DRAW),
))];

static DOPPELGANGER_DRAW_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &DOPPELGANGER_DRAW_EFFECTS,
    counter: None,
}];

pub static DEF_DOPPELGANGER_DRAW: EntityDef = EntityDef {
    id: "doppelganger_draw",
    name: "Doppelganger (Draw)",
    kind: EntityKind::Power,
    triggers: &DOPPELGANGER_DRAW_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::DOPPELGANGER_DRAW),
};

// ===========================================================================
// Doppelganger Energy — TurnStart: gain N energy (one-shot, consumed)
// ===========================================================================

static DOPPELGANGER_ENERGY_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainEnergy(
    AmountSource::StatusValue(sid::DOPPELGANGER_ENERGY),
))];

static DOPPELGANGER_ENERGY_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &DOPPELGANGER_ENERGY_EFFECTS,
    counter: None,
}];

pub static DEF_DOPPELGANGER_ENERGY: EntityDef = EntityDef {
    id: "doppelganger_energy",
    name: "Doppelganger (Energy)",
    kind: EntityKind::Power,
    triggers: &DOPPELGANGER_ENERGY_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::DOPPELGANGER_ENERGY),
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
        engine.state.player.set_status(sid::ENTER_DIVINITY, 0);
        engine.change_stance(Stance::Divinity);
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
// Mayhem — TurnStartPostDraw: autoplay the top draw card once per stack
// ===========================================================================

static MAYHEM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
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
// Tools of the Trade — TurnStartPostDraw: draw N, then choose one discard
// Preserve current engine behavior exactly: it draws `N` cards but still
// opens a single-card discard choice rather than `N` discards.
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

    engine.draw_cards(tott);
    if engine.state.hand.is_empty() {
        return;
    }

    let options: Vec<ChoiceOption> = (0..engine.state.hand.len())
        .map(ChoiceOption::HandCard)
        .collect();
    engine.begin_choice(ChoiceReason::DiscardFromHand, options, 1, 1);
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
        assert_eq!(DEF_DEMON_FORM.triggers[0].trigger, Trigger::TurnStartPostDraw);
        assert_eq!(DEF_DEMON_FORM.triggers[0].condition, TriggerCondition::Always);
        assert!(DEF_DEMON_FORM.complex_hook.is_none());
    }

    #[test]
    fn test_brutality_has_two_effects() {
        assert_eq!(DEF_BRUTALITY.triggers[0].effects.len(), 2);
    }

    #[test]
    fn test_all_simple_turn_start_defs_have_correct_trigger() {
        let defs = [
            &DEF_ENERGIZED,
            &DEF_BERSERK, &DEF_INFINITE_BLADES, &DEF_BATTLE_HYMN,
            &DEF_WRAITH_FORM, &DEF_DEVA_FORM,
            &DEF_HELLO_WORLD, &DEF_MAGNETISM,
            &DEF_DOPPELGANGER_DRAW, &DEF_DOPPELGANGER_ENERGY,
        ];
        for def in &defs {
            assert_eq!(def.kind, EntityKind::Power);
            assert!(!def.triggers.is_empty());
            assert_eq!(def.triggers[0].trigger, Trigger::TurnStart);
        }
        // These powers override atStartOfTurnPostDraw, not atStartOfTurn.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BrutalityPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/NoxiousFumesPower.java
        assert_eq!(DEF_BRUTALITY.triggers[0].trigger, Trigger::TurnStartPostDraw);
        assert_eq!(DEF_DEMON_FORM.triggers[0].trigger, Trigger::TurnStartPostDraw);
        assert_eq!(DEF_DEVOTION.triggers[0].trigger, Trigger::TurnStartPostDraw);
        assert_eq!(DEF_NOXIOUS_FUMES.triggers[0].trigger, Trigger::TurnStartPostDraw);
    }

    #[test]
    fn test_complex_turn_start_defs_have_hooks() {
        // CreativeAIPower overrides atStartOfTurn, while the other complex
        // powers here override atStartOfTurnPostDraw.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CreativeAIPower.java
        assert_eq!(DEF_CREATIVE_AI.kind, EntityKind::Power);
        assert!(DEF_CREATIVE_AI.complex_hook.is_some());
        assert_eq!(DEF_CREATIVE_AI.triggers[0].trigger, Trigger::TurnStart);

        for def in [&DEF_ENTER_DIVINITY, &DEF_MAYHEM, &DEF_TOOLS_OF_THE_TRADE] {
            assert_eq!(def.kind, EntityKind::Power);
            assert!(def.complex_hook.is_some());
            assert_eq!(def.triggers.len(), 1);
            assert_eq!(def.triggers[0].trigger, Trigger::TurnStartPostDraw);
        }
    }
}

#[cfg(test)]
#[path = "../../tests/test_power_runtime_turn_start.rs"]
mod test_power_runtime_turn_start;
