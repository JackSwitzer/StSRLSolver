use crate::effects::declarative::{
    AmountSource, BulkAction, ChoiceAction, Effect, Pile, SimpleEffect,
};
use crate::effects::types::{
    CanPlayRule, CardBlockHint, CardEvokeHint, CardMetadata, CardPlayHints, CardRuntimeTraits,
    CardRuntimeTrigger, CostModifierRule, DamageModifierRule, EndTurnHandRule, OnDiscardRule,
    OnDrawRule, OnExhaustRule, OnRetainRule, PostPlayRule, WhileInHandRule,
};

const EMPTY_TRIGGERS: &[CardRuntimeTrigger] = &[];
const CLASH_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::CanPlay(CanPlayRule::OnlyAttacksInHand)];
const SIGNATURE_MOVE_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::CanPlay(CanPlayRule::OnlyAttackInHand)];
const GRAND_FINALE_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::CanPlay(CanPlayRule::OnlyEmptyDraw)];
const ENDLESS_AGONY_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::OnDraw(OnDrawRule::CopySelf)];
const VOID_TRIGGERS: &[CardRuntimeTrigger] = &[
    CardRuntimeTrigger::OnDraw(OnDrawRule::LoseEnergy),
    CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Damage),
];
const DEUS_EX_MACHINA_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::OnDraw(OnDrawRule::DeusExMachina)];
const REFLEX_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::OnDiscard(OnDiscardRule::DrawCards)];
const TACTICIAN_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::OnDiscard(OnDiscardRule::GainEnergy)];
const SANDS_OF_TIME_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::OnRetain(OnRetainRule::ReduceCost)];
const PERSEVERANCE_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::OnRetain(OnRetainRule::GrowBlock)];
const WINDMILL_STRIKE_TRIGGERS: &[CardRuntimeTrigger] = &[
    CardRuntimeTrigger::OnRetain(OnRetainRule::GrowDamage),
    CardRuntimeTrigger::ModifyDamage(DamageModifierRule::WindmillStrike),
];
const SENTINEL_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::OnExhaust(OnExhaustRule::GainEnergy)];
const BLOOD_FOR_BLOOD_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::ModifyCost(CostModifierRule::ReduceOnHpLoss)];
const FORCE_FIELD_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::ModifyCost(CostModifierRule::ReducePerPower)];
const EVISCERATE_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::ModifyCost(CostModifierRule::ReduceOnDiscard)];
const MASTERFUL_STAB_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::ModifyCost(CostModifierRule::IncreaseOnHpLoss)];
const TANTRUM_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::PostPlay(PostPlayRule::ShuffleIntoDraw)];
const CONCLUDE_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::PostPlay(PostPlayRule::EndTurn)];
const VAULT_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::PostPlay(PostPlayRule::EndTurn)];
const MEDITATE_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::PostPlay(PostPlayRule::EndTurn)];
const HEAVY_BLADE_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::ModifyDamage(DamageModifierRule::HeavyBlade)];
const PERFECTED_STRIKE_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::ModifyDamage(DamageModifierRule::PerfectedStrike)];
const RAMPAGE_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::ModifyDamage(DamageModifierRule::Rampage)];
const GLASS_KNIFE_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::ModifyDamage(DamageModifierRule::GlassKnife)];
const RITUAL_DAGGER_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::ModifyDamage(DamageModifierRule::RitualDagger)];
const SEARING_BLOW_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::ModifyDamage(DamageModifierRule::SearingBlow)];
const CLAW_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::ModifyDamage(DamageModifierRule::ClawScaling)];
const MIND_BLAST_TRIGGERS: &[CardRuntimeTrigger] = &[
    CardRuntimeTrigger::ModifyDamage(DamageModifierRule::DamageFromDrawPile),
];
const BRILLIANCE_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::ModifyDamage(DamageModifierRule::DamagePlusMantra)];
const BURN_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Damage)];
const DECAY_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Damage)];
const REGRET_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Regret)];
const DOUBT_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Weak)];
const SHAME_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Frail)];
const PRIDE_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::AddCopy)];
const PAIN_TRIGGERS: &[CardRuntimeTrigger] =
    &[CardRuntimeTrigger::WhileInHand(WhileInHandRule::PainOnOtherCardPlayed)];

pub fn runtime_traits_for_card(id: &str, cost: i32) -> CardRuntimeTraits {
    CardRuntimeTraits {
        innate: matches!(
            id,
            "Dramatic Entrance"
                | "Dramatic Entrance+"
                | "Mind Blast"
                | "Mind Blast+"
                | "BootSequence"
                | "BootSequence+"
                | "Chill+"
                | "Hello World+"
                | "Machine Learning+"
                | "Storm+"
                | "Brutality+"
                | "Backstab"
                | "Backstab+"
                | "Infinite Blades+"
                | "Alpha+"
                | "BattleHymn+"
                | "Establishment+"
                | "Pride"
                | "Writhe"
        ),
        retain: matches!(
            id,
            "Crescendo"
                | "Crescendo+"
                | "FlyingSleeves"
                | "FlyingSleeves+"
                | "HolyWater"
                | "HolyWater+"
                | "Insight"
                | "Insight+"
                | "Perseverance"
                | "Perseverance+"
                | "Protect"
                | "Protect+"
                | "Safety"
                | "Safety+"
                | "SandsOfTime"
                | "SandsOfTime+"
                | "Smite"
                | "Smite+"
                | "ThroughViolence"
                | "ThroughViolence+"
                | "ClearTheMind"
                | "ClearTheMind+"
                | "WindmillStrike"
                | "WindmillStrike+"
                | "Worship+"
                | "Blasphemy+"
        ),
        ethereal: matches!(
            id,
            "Daze"
                | "Void"
                | "AscendersBane"
                | "Clumsy"
                | "Ghostly"
                | "Carnage"
                | "Carnage+"
                | "Ghostly Armor"
                | "Ghostly Armor+"
                | "Phantasmal Killer"
                | "DevaForm"
                | "Echo Form"
        ),
        unplayable: cost == -2,
        limit_cards_per_turn: matches!(id, "Normality"),
        unremovable: matches!(id, "AscendersBane" | "CurseOfTheBell" | "Necronomicurse"),
    }
}

pub fn runtime_triggers_for_card(id: &str) -> &'static [CardRuntimeTrigger] {
    match id {
        "Clash" | "Clash+" => CLASH_TRIGGERS,
        "SignatureMove" | "SignatureMove+" => SIGNATURE_MOVE_TRIGGERS,
        "GrandFinale" | "Grand Finale" | "GrandFinale+" | "Grand Finale+" => GRAND_FINALE_TRIGGERS,
        "Endless Agony" | "Endless Agony+" => ENDLESS_AGONY_TRIGGERS,
        "Void" => VOID_TRIGGERS,
        "DeusExMachina" | "Deus Ex Machina" | "DeusExMachina+" | "Deus Ex Machina+" => {
            DEUS_EX_MACHINA_TRIGGERS
        }
        "Reflex" | "Reflex+" => REFLEX_TRIGGERS,
        "Tactician" | "Tactician+" => TACTICIAN_TRIGGERS,
        "SandsOfTime" | "Sands of Time" | "SandsOfTime+" | "Sands of Time+" => {
            SANDS_OF_TIME_TRIGGERS
        }
        "Perseverance" | "Perseverance+" => PERSEVERANCE_TRIGGERS,
        "WindmillStrike" | "Windmill Strike" | "WindmillStrike+" | "Windmill Strike+" => {
            WINDMILL_STRIKE_TRIGGERS
        }
        "Sentinel" | "Sentinel+" => SENTINEL_TRIGGERS,
        "Blood for Blood" | "Blood for Blood+" => BLOOD_FOR_BLOOD_TRIGGERS,
        "Force Field" | "Force Field+" => FORCE_FIELD_TRIGGERS,
        "Eviscerate" | "Eviscerate+" => EVISCERATE_TRIGGERS,
        "Masterful Stab" | "Masterful Stab+" => MASTERFUL_STAB_TRIGGERS,
        "Tantrum" | "Tantrum+" => TANTRUM_TRIGGERS,
        "Conclude" | "Conclude+" => CONCLUDE_TRIGGERS,
        "Vault" | "Vault+" => VAULT_TRIGGERS,
        "Meditate" | "Meditate+" => MEDITATE_TRIGGERS,
        "Heavy Blade" | "Heavy Blade+" => HEAVY_BLADE_TRIGGERS,
        "Perfected Strike" | "Perfected Strike+" => PERFECTED_STRIKE_TRIGGERS,
        "Rampage" | "Rampage+" => RAMPAGE_TRIGGERS,
        "Glass Knife" | "Glass Knife+" => GLASS_KNIFE_TRIGGERS,
        "RitualDagger" | "Ritual Dagger" | "RitualDagger+" | "Ritual Dagger+" => {
            RITUAL_DAGGER_TRIGGERS
        }
        "Searing Blow" => SEARING_BLOW_TRIGGERS,
        "Claw" | "Claw+" | "Gash" | "Gash+" => CLAW_TRIGGERS,
        "Mind Blast" | "Mind Blast+" => MIND_BLAST_TRIGGERS,
        "Brilliance" | "Brilliance+" => BRILLIANCE_TRIGGERS,
        "Burn" | "Burn+" => BURN_TRIGGERS,
        "Decay" => DECAY_TRIGGERS,
        "Regret" => REGRET_TRIGGERS,
        "Doubt" => DOUBT_TRIGGERS,
        "Shame" => SHAME_TRIGGERS,
        "Pride" => PRIDE_TRIGGERS,
        "Pain" => PAIN_TRIGGERS,
        _ => EMPTY_TRIGGERS,
    }
}

pub fn play_hints_for_card(id: &str, cost: i32, effects: &[Effect]) -> CardPlayHints {
    CardPlayHints {
        draws_cards: effect_draws_cards(effects),
        discards_cards: effect_discards_cards(effects),
        x_cost: cost == -1,
        // A small number of cards express repeated hits through typed card-owned
        // modifiers instead of declarative ExtraHits.
        multi_hit: effect_has_extra_hits(effects)
            || matches!(id, "Tantrum" | "Tantrum+" | "Glass Knife" | "Glass Knife+"),
        block_hint: match id {
            "ReinforcedBody" | "ReinforcedBody+" | "Reinforced Body" | "Reinforced Body+" => {
                Some(CardBlockHint::XTimes)
            }
            "Escape Plan" | "Escape Plan+" => Some(CardBlockHint::IfSkill),
            "Auto Shields" | "Auto Shields+" => Some(CardBlockHint::IfNoBlock),
            "Second Wind" | "Second Wind+" => Some(CardBlockHint::BulkCountTimesBaseBlock),
            "Genetic Algorithm" | "Genetic Algorithm+" | "SteamBarrier" | "SteamBarrier+" => {
                Some(CardBlockHint::UsesCardMisc)
            }
            _ => None,
        },
        evoke_hint: derive_evoke_hint(effects),
        channel_evoked_orb: matches!(id, "Redo" | "Redo+"),
    }
}

pub fn metadata_for_card(id: &str, cost: i32, effects: &[Effect]) -> CardMetadata {
    CardMetadata {
        runtime_traits: runtime_traits_for_card(id, cost),
        runtime_triggers: runtime_triggers_for_card(id).to_vec().into_boxed_slice(),
        play_hints: play_hints_for_card(id, cost, effects),
    }
}

fn effect_draws_cards(effects: &[Effect]) -> bool {
    effects.iter().any(|effect| match effect {
        Effect::Simple(
            SimpleEffect::DrawCards(_)
            | SimpleEffect::DrawCardsThenDiscardDrawnNonZeroCost(_)
            | SimpleEffect::DrawToHandSize(_)
            | SimpleEffect::DrawRandomCardsFromPileToHand(_, _, _),
        ) => true,
        Effect::ChooseCards {
            post_choice_draw, ..
        } => !matches!(post_choice_draw, AmountSource::Fixed(0)),
        Effect::Conditional(_, then_effects, else_effects) => {
            effect_draws_cards(then_effects) || effect_draws_cards(else_effects)
        }
        _ => false,
    })
}

fn effect_discards_cards(effects: &[Effect]) -> bool {
    effects.iter().any(|effect| match effect {
        Effect::Simple(SimpleEffect::DiscardRandomCardsFromPile(_, _)) => true,
        Effect::ChooseCards { source, action, .. } => {
            matches!(source, Pile::Hand)
                && matches!(action, ChoiceAction::Discard | ChoiceAction::DiscardForEffect)
        }
        Effect::ForEachInPile { action, .. } => matches!(action, BulkAction::Discard),
        Effect::Conditional(_, then_effects, else_effects) => {
            effect_discards_cards(then_effects) || effect_discards_cards(else_effects)
        }
        _ => false,
    })
}

fn effect_has_extra_hits(effects: &[Effect]) -> bool {
    effects.iter().any(|effect| match effect {
        Effect::ExtraHits(_) => true,
        Effect::Conditional(_, then_effects, else_effects) => {
            effect_has_extra_hits(then_effects) || effect_has_extra_hits(else_effects)
        }
        _ => false,
    })
}

fn derive_evoke_hint(effects: &[Effect]) -> Option<CardEvokeHint> {
    let mut fixed_count: u8 = 0;
    let mut x_hint = None;
    collect_evoke_hint(effects, &mut fixed_count, &mut x_hint);
    x_hint.or_else(|| (fixed_count > 0).then_some(CardEvokeHint::Fixed(fixed_count)))
}

fn collect_evoke_hint(
    effects: &[Effect],
    fixed_count: &mut u8,
    x_hint: &mut Option<CardEvokeHint>,
) {
    for effect in effects {
        match effect {
            Effect::Simple(SimpleEffect::EvokeOrb(amount)) => match amount {
                AmountSource::Fixed(count) if *count > 0 => {
                    *fixed_count = fixed_count.saturating_add(*count as u8);
                }
                AmountSource::XCost => *x_hint = Some(CardEvokeHint::XCost),
                AmountSource::XCostPlus(count) if *count > 0 => {
                    *x_hint = Some(CardEvokeHint::XCostPlus(*count as u8));
                }
                _ => {}
            },
            Effect::Conditional(_, then_effects, else_effects) => {
                collect_evoke_hint(then_effects, fixed_count, x_hint);
                collect_evoke_hint(else_effects, fixed_count, x_hint);
            }
            _ => {}
        }
    }
}
