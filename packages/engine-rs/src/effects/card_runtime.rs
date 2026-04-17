use crate::cards::CardDef;
use crate::combat_types::CardInstance;
use crate::engine::CombatEngine;
use crate::effects::types::{
    CanPlayRule, CardPlayContext, CardRuntimeTrigger, CostModifierRule, DamageModifier,
    DamageModifierRule, OnDiscardEffect, OnDiscardRule, OnDrawRule, OnExhaustRule,
    OnRetainRule, PostPlayDestination, PostPlayRule,
};
use crate::status_ids::sid;

pub fn allows_play(engine: &CombatEngine, card: &CardDef, card_inst: CardInstance) -> bool {
    for trigger in card.runtime_triggers() {
        if let CardRuntimeTrigger::CanPlay(rule) = trigger {
            match rule {
                CanPlayRule::OnlyAttackInHand => {
                    let has_other_attack = engine.state.hand.iter().any(|candidate| {
                        let def = engine.card_registry.card_def_by_id(candidate.def_id);
                        def.card_type == crate::cards::CardType::Attack
                            && candidate.def_id != card_inst.def_id
                    });
                    if has_other_attack {
                        return false;
                    }
                }
                CanPlayRule::OnlyAttacksInHand => {
                    let has_non_attack = engine.state.hand.iter().any(|candidate| {
                        let def = engine.card_registry.card_def_by_id(candidate.def_id);
                        def.card_type != crate::cards::CardType::Attack
                    });
                    if has_non_attack {
                        return false;
                    }
                }
                CanPlayRule::OnlyEmptyDraw => {
                    if !engine.state.draw_pile.is_empty() {
                        return false;
                    }
                }
            }
        }
    }
    true
}

pub fn apply_cost_modifiers(engine: &CombatEngine, card: &CardDef, base_cost: i32) -> i32 {
    let mut cost = base_cost;
    for trigger in card.runtime_triggers() {
        if let CardRuntimeTrigger::ModifyCost(rule) = trigger {
            match rule {
                CostModifierRule::ReduceOnHpLoss => {
                    let hp_lost = engine.state.player.status(sid::HP_LOSS_THIS_COMBAT);
                    cost = (cost - hp_lost).max(0);
                }
                CostModifierRule::ReducePerPower => {
                    let power_count =
                        crate::powers::registry::active_player_power_count(&engine.state.player);
                    cost = (cost - power_count).max(0);
                }
                CostModifierRule::ReduceOnDiscard => {
                    let discarded = engine.state.player.status(sid::DISCARDED_THIS_TURN);
                    cost = (cost - discarded).max(0);
                }
                CostModifierRule::IncreaseOnHpLoss => {
                    cost += engine.state.total_damage_taken;
                }
            }
        }
    }
    cost
}

pub fn resolve_damage_modifiers(
    engine: &CombatEngine,
    card: &CardDef,
    card_inst: CardInstance,
) -> DamageModifier {
    use crate::effects::hooks_damage;

    let mut out = DamageModifier::default();
    for trigger in card.runtime_triggers() {
        if let CardRuntimeTrigger::ModifyDamage(rule) = trigger {
            let modifier = match rule {
                DamageModifierRule::HeavyBlade => hooks_damage::hook_heavy_blade(engine, card, card_inst),
                DamageModifierRule::DamageEqualsBlock => {
                    hooks_damage::hook_damage_equals_block(engine, card, card_inst)
                }
                DamageModifierRule::DamagePlusMantra => {
                    hooks_damage::hook_damage_plus_mantra(engine, card, card_inst)
                }
                DamageModifierRule::PerfectedStrike => {
                    hooks_damage::hook_perfected_strike(engine, card, card_inst)
                }
                DamageModifierRule::Rampage => hooks_damage::hook_rampage(engine, card, card_inst),
                DamageModifierRule::GlassKnife => {
                    hooks_damage::hook_glass_knife(engine, card, card_inst)
                }
                DamageModifierRule::RitualDagger => {
                    hooks_damage::hook_ritual_dagger(engine, card, card_inst)
                }
                DamageModifierRule::SearingBlow => {
                    hooks_damage::hook_searing_blow(engine, card, card_inst)
                }
                DamageModifierRule::DamageRandomXTimes => {
                    hooks_damage::hook_damage_random_x_times(engine, card, card_inst)
                }
                DamageModifierRule::WindmillStrike => {
                    hooks_damage::hook_windmill_strike_damage(engine, card, card_inst)
                }
                DamageModifierRule::ClawScaling => hooks_damage::hook_claw_damage(engine, card, card_inst),
                DamageModifierRule::DamagePerLightning | DamageModifierRule::DamageFromDrawPile => {
                    hooks_damage::hook_skip_generic_damage(engine, card, card_inst)
                }
            };
            out.merge(modifier);
        }
    }
    out
}

pub fn apply_on_draw(engine: &mut CombatEngine, card_inst: CardInstance) {
    let card = engine.card_registry.card_def_by_id(card_inst.def_id);
    for trigger in card.runtime_triggers() {
        if let CardRuntimeTrigger::OnDraw(rule) = trigger {
            match rule {
                OnDrawRule::LoseEnergy => crate::effects::hooks_draw::hook_lose_energy_on_draw(engine, card_inst),
                OnDrawRule::CopySelf => crate::effects::hooks_draw::hook_copy_on_draw(engine, card_inst),
                OnDrawRule::DeusExMachina => {
                    crate::effects::hooks_draw::hook_deus_ex_machina_on_draw(engine, card_inst)
                }
            }
        }
    }
}

pub fn apply_on_discard(engine: &mut CombatEngine, card_inst: CardInstance) -> OnDiscardEffect {
    let card = engine.card_registry.card_def_by_id(card_inst.def_id);
    let mut out = OnDiscardEffect::default();
    for trigger in card.runtime_triggers() {
        if let CardRuntimeTrigger::OnDiscard(rule) = trigger {
            let effect = match rule {
                OnDiscardRule::DrawCards => crate::effects::hooks_discard::hook_draw_on_discard(engine, card_inst),
                OnDiscardRule::GainEnergy => crate::effects::hooks_discard::hook_energy_on_discard(engine, card_inst),
            };
            out.merge(effect);
        }
    }
    out
}

pub fn apply_on_exhaust(engine: &mut CombatEngine, card: &CardDef, card_inst: CardInstance) {
    for trigger in card.runtime_triggers() {
        if let CardRuntimeTrigger::OnExhaust(rule) = trigger {
            let ctx = CardPlayContext {
                card,
                card_inst,
                target_idx: -1,
                x_value: 0,
                pen_nib_active: false,
                vigor: 0,
                total_unblocked_damage: 0,
                enemy_killed: false,
                hand_size_at_play: engine.state.hand.len(),
                last_bulk_count: 0,
            };
            match rule {
                OnExhaustRule::GainEnergy => crate::effects::hooks_simple::hook_energy_on_exhaust(engine, &ctx),
            }
        }
    }
}

pub fn apply_on_retain(card_inst: &mut CardInstance, card: &CardDef) -> (i32, i32) {
    let mut perseverance_bonus = 0;
    let mut windmill_bonus = 0;
    for trigger in card.runtime_triggers() {
        if let CardRuntimeTrigger::OnRetain(rule) = trigger {
            match rule {
                OnRetainRule::ReduceCost => {
                    card_inst.cost = (card_inst.cost - 1).max(0);
                }
                OnRetainRule::GrowBlock => {
                    perseverance_bonus += card.base_magic;
                }
                OnRetainRule::GrowDamage => {
                    windmill_bonus += card.base_magic;
                }
            }
        }
    }
    (perseverance_bonus, windmill_bonus)
}

pub fn post_play_destination(card: &CardDef) -> PostPlayDestination {
    for trigger in card.runtime_triggers() {
        if let CardRuntimeTrigger::PostPlay(rule) = trigger {
            return match rule {
                PostPlayRule::ShuffleIntoDraw => PostPlayDestination::ShuffleIntoDraw,
                PostPlayRule::EndTurn => PostPlayDestination::EndTurn,
            };
        }
    }
    PostPlayDestination::Normal
}
