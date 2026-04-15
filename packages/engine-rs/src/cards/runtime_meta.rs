use crate::effects::types::{
    CanPlayRule, CardRuntimeTraits, CardRuntimeTrigger, CostModifierRule, DamageModifierRule,
    EndTurnHandRule, OnDiscardRule, OnDrawRule, OnExhaustRule, OnRetainRule, PostPlayRule,
    WhileInHandRule,
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

pub fn runtime_traits_for_card(legacy_effects: &[&'static str]) -> CardRuntimeTraits {
    CardRuntimeTraits {
        innate: legacy_effects.contains(&"innate"),
        retain: legacy_effects.contains(&"retain"),
        ethereal: legacy_effects.contains(&"ethereal"),
        unplayable: legacy_effects.contains(&"unplayable"),
        limit_cards_per_turn: legacy_effects.contains(&"limit_cards_per_turn"),
        unremovable: false,
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
        "Claw" | "Claw+" => CLAW_TRIGGERS,
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

fn runtime_trait_tag_names(traits: CardRuntimeTraits) -> [Option<&'static str>; 5] {
    [
        traits.innate.then_some("innate"),
        traits.retain.then_some("retain"),
        traits.ethereal.then_some("ethereal"),
        traits.unplayable.then_some("unplayable"),
        traits.limit_cards_per_turn.then_some("limit_cards_per_turn"),
    ]
}

fn runtime_trigger_tag_names(trigger: CardRuntimeTrigger) -> &'static [&'static str] {
    match trigger {
        CardRuntimeTrigger::CanPlay(CanPlayRule::OnlyAttackInHand) => &["only_attack_in_hand"],
        CardRuntimeTrigger::CanPlay(CanPlayRule::OnlyAttacksInHand) => &["only_attacks_in_hand"],
        CardRuntimeTrigger::CanPlay(CanPlayRule::OnlyEmptyDraw) => &["only_empty_draw"],
        CardRuntimeTrigger::ModifyCost(CostModifierRule::ReduceOnHpLoss) => {
            &["cost_reduce_on_hp_loss"]
        }
        CardRuntimeTrigger::ModifyCost(CostModifierRule::ReducePerPower) => {
            &["reduce_cost_per_power"]
        }
        CardRuntimeTrigger::ModifyCost(CostModifierRule::ReduceOnDiscard) => {
            &["cost_reduce_on_discard"]
        }
        CardRuntimeTrigger::ModifyCost(CostModifierRule::IncreaseOnHpLoss) => {
            &["cost_increase_on_hp_loss"]
        }
        CardRuntimeTrigger::ModifyDamage(DamageModifierRule::HeavyBlade) => &["heavy_blade"],
        CardRuntimeTrigger::ModifyDamage(DamageModifierRule::DamageEqualsBlock) => {
            &["damage_equals_block"]
        }
        CardRuntimeTrigger::ModifyDamage(DamageModifierRule::DamagePlusMantra) => {
            &["damage_plus_mantra"]
        }
        CardRuntimeTrigger::ModifyDamage(DamageModifierRule::PerfectedStrike) => {
            &["perfected_strike"]
        }
        CardRuntimeTrigger::ModifyDamage(DamageModifierRule::Rampage) => &["rampage"],
        CardRuntimeTrigger::ModifyDamage(DamageModifierRule::GlassKnife) => &["glass_knife"],
        CardRuntimeTrigger::ModifyDamage(DamageModifierRule::RitualDagger) => {
            &["ritual_dagger"]
        }
        CardRuntimeTrigger::ModifyDamage(DamageModifierRule::SearingBlow) => {
            &["searing_blow"]
        }
        CardRuntimeTrigger::ModifyDamage(DamageModifierRule::DamageRandomXTimes) => {
            &["damage_random_x_times"]
        }
        CardRuntimeTrigger::ModifyDamage(DamageModifierRule::WindmillStrike) => {
            &["grow_damage_on_retain"]
        }
        CardRuntimeTrigger::ModifyDamage(DamageModifierRule::ClawScaling) => &["claw_scaling"],
        CardRuntimeTrigger::ModifyDamage(DamageModifierRule::DamagePerLightning) => {
            &["damage_per_lightning_channeled"]
        }
        CardRuntimeTrigger::ModifyDamage(DamageModifierRule::DamageFromDrawPile) => {
            &["damage_from_draw_pile"]
        }
        CardRuntimeTrigger::OnDraw(OnDrawRule::LoseEnergy) => &["lose_energy_on_draw"],
        CardRuntimeTrigger::OnDraw(OnDrawRule::CopySelf) => &["copy_on_draw"],
        CardRuntimeTrigger::OnDraw(OnDrawRule::DeusExMachina) => &["deus_ex_machina"],
        CardRuntimeTrigger::OnDiscard(OnDiscardRule::DrawCards) => &["draw_on_discard"],
        CardRuntimeTrigger::OnDiscard(OnDiscardRule::GainEnergy) => &["energy_on_discard"],
        CardRuntimeTrigger::OnRetain(OnRetainRule::ReduceCost) => &["reduce_cost_on_retain"],
        CardRuntimeTrigger::OnRetain(OnRetainRule::GrowBlock) => &["grow_block_on_retain"],
        CardRuntimeTrigger::OnRetain(OnRetainRule::GrowDamage) => &["grow_damage_on_retain"],
        CardRuntimeTrigger::OnExhaust(OnExhaustRule::GainEnergy) => &["energy_on_exhaust"],
        CardRuntimeTrigger::PostPlay(PostPlayRule::ShuffleIntoDraw) => &["shuffle_self_into_draw"],
        CardRuntimeTrigger::PostPlay(PostPlayRule::EndTurn) => &["end_turn"],
        CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Damage) => &["end_turn_damage"],
        CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Regret) => &["end_turn_regret"],
        CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Weak) => &["end_turn_weak"],
        CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Frail) => &["end_turn_frail"],
        CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::AddCopy) => &["add_copy_end_turn"],
        CardRuntimeTrigger::WhileInHand(WhileInHandRule::PainOnOtherCardPlayed) => {
            &["damage_on_draw"]
        }
    }
}

fn dedupe(tags: &mut Vec<&'static str>) {
    let mut seen = Vec::new();
    tags.retain(|tag| {
        if seen.contains(tag) {
            false
        } else {
            seen.push(*tag);
            true
        }
    });
}

pub fn compat_effect_tags_for_card(id: &str, legacy_effects: &[&'static str]) -> Vec<&'static str> {
    let mut tags = legacy_effect_tags_for_card(id, legacy_effects);
    let traits = runtime_traits_for_card(legacy_effects);
    for tag in runtime_trait_tag_names(traits).into_iter().flatten() {
        tags.push(tag);
    }
    for trigger in runtime_triggers_for_card(id) {
        tags.extend_from_slice(runtime_trigger_tag_names(*trigger));
    }
    dedupe(&mut tags);
    tags
}

pub fn legacy_effect_tags_for_card(id: &str, legacy_effects: &[&'static str]) -> Vec<&'static str> {
    let traits = runtime_traits_for_card(legacy_effects);
    let mut filtered = Vec::with_capacity(legacy_effects.len());
    for tag in legacy_effects {
        let stripped_trait = matches!(
            *tag,
            "innate" | "retain" | "ethereal" | "unplayable" | "limit_cards_per_turn"
        ) && runtime_trait_tag_names(traits).into_iter().flatten().any(|candidate| candidate == *tag);
        let stripped_trigger = runtime_triggers_for_card(id)
            .iter()
            .flat_map(|trigger| runtime_trigger_tag_names(*trigger).iter().copied())
            .any(|candidate| candidate == *tag);
        if !stripped_trait && !stripped_trigger {
            filtered.push(*tag);
        }
    }
    filtered
}
