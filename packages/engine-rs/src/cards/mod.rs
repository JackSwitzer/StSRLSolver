//! Canonical card registry and typed card metadata for the Rust gameplay runtime.
//!
//! Card definitions carry:
//! - primary declarative play bodies in `effect_data`
//! - typed secondary/runtime behavior in `metadata`
//! - optional irreducible on-play hooks in `complex_hook`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::combat_types::CardInstance;
use crate::effects::declarative::{
    AmountSource, ChoiceAction, Effect, Pile, SimpleEffect, Target,
};
use crate::effects::types::{
    CardBlockHint, CardEvokeHint, CardMetadata, CardPlayHints, CardRuntimeTraits,
    CardRuntimeTrigger, ComplexCardHook,
};
use crate::ids::StatusId;
use crate::orbs::OrbType;
use crate::state::Stance;
#[cfg(test)]
use crate::effects::declarative::CardFilter;
#[cfg(test)]
use crate::effects::types::{
    CanPlayRule, CostModifierRule, DamageModifierRule, EndTurnHandRule, OnDiscardRule,
    OnDrawRule, OnRetainRule, PostPlayRule, WhileInHandRule,
};

mod prelude;
mod watcher;
mod ironclad;
mod silent;
pub(crate) mod defect;
mod colorless;
mod curses;
mod runtime_meta;
mod status;
mod temp;

/// Insert a card spec into the staging registry map.
pub(crate) fn insert(map: &mut HashMap<&'static str, CardSpec>, card: CardSpec) {
    map.insert(card.id, card);
}


// ---------------------------------------------------------------------------
// Card types (match Python enums)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardType {
    Attack,
    Skill,
    Power,
    Status,
    Curse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardTarget {
    /// Single enemy (requires target selection)
    Enemy,
    /// All enemies (no target needed)
    AllEnemy,
    /// Self only
    SelfTarget,
    /// No target
    None,
}

// ---------------------------------------------------------------------------
// Card definition — static data, no mutation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct CardDef {
    pub id: &'static str,
    pub name: &'static str,
    pub card_type: CardType,
    pub target: CardTarget,
    pub cost: i32,
    pub base_damage: i32,
    pub base_block: i32,
    pub base_magic: i32,
    /// Does this card exhaust when played?
    pub exhaust: bool,
    /// Does this card change stance?
    pub enter_stance: Option<&'static str>,
    /// Canonical typed runtime metadata for card-owned secondary behavior.
    pub metadata: CardMetadata,
    /// Declarative effect data for the primary on-play body.
    /// Empty slice = the card has no primary declarative play body.
    #[serde(skip)]
    pub effect_data: &'static [crate::effects::declarative::Effect],
    /// Complex on-play hook for irreducible effects (Pressure Points, Judgement, etc.).
    /// None = no complex hook.
    #[serde(skip)]
    pub complex_hook: Option<ComplexCardHook>,
}

#[derive(Debug, Clone)]
pub struct CardSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub card_type: CardType,
    pub target: CardTarget,
    pub cost: i32,
    pub base_damage: i32,
    pub base_block: i32,
    pub base_magic: i32,
    pub exhaust: bool,
    pub enter_stance: Option<&'static str>,
    pub effect_data: &'static [crate::effects::declarative::Effect],
    pub complex_hook: Option<ComplexCardHook>,
}

impl From<CardSpec> for CardDef {
    fn from(value: CardSpec) -> Self {
        let enter_stance = value
            .enter_stance
            .or_else(|| declared_stance_name(find_unconditional_stance_change(value.effect_data)));
        Self {
            id: value.id,
            name: value.name,
            card_type: value.card_type,
            target: value.target,
            cost: value.cost,
            base_damage: value.base_damage,
            base_block: value.base_block,
            base_magic: value.base_magic,
            exhaust: value.exhaust,
            enter_stance,
            metadata: runtime_meta::metadata_for_card(value.id, value.cost, value.effect_data),
            effect_data: value.effect_data,
            complex_hook: value.complex_hook,
        }
    }
}

impl CardDef {
    pub fn runtime_traits(&self) -> CardRuntimeTraits {
        self.metadata.runtime_traits
    }

    pub fn runtime_triggers(&self) -> &[CardRuntimeTrigger] {
        &self.metadata.runtime_triggers
    }

    pub fn play_hints(&self) -> &CardPlayHints {
        &self.metadata.play_hints
    }

    pub fn has_block_hint(&self, hint: CardBlockHint) -> bool {
        self.play_hints().block_hint == Some(hint)
    }

    pub fn evoke_hint(&self) -> Option<CardEvokeHint> {
        self.play_hints().evoke_hint
    }

    /// Is this card an unplayable status/curse?
    pub fn is_unplayable(&self) -> bool {
        self.cost == -2 || self.runtime_traits().unplayable
    }

    pub fn is_runtime_only(&self) -> bool {
        self.effect_data.is_empty()
            && self.complex_hook.is_none()
            && (self.runtime_traits() != CardRuntimeTraits::default()
                || !self.runtime_triggers().is_empty())
    }

    pub fn declared_effect_count(&self) -> usize {
        self.effect_data.len()
    }

    pub fn declared_extra_hits(&self) -> Option<AmountSource> {
        find_declared_extra_hits(self.effect_data)
    }

    pub fn declared_stance_change(&self) -> Option<Stance> {
        find_declared_stance_change(self.effect_data)
    }

    pub fn declared_all_enemy_damage(&self) -> Option<AmountSource> {
        if self.target == CardTarget::AllEnemy && self.base_damage >= 0 {
            Some(AmountSource::Damage)
        } else {
            find_declared_all_enemy_damage(self.effect_data)
        }
    }

    pub fn declared_primary_attack_target(&self) -> Option<Target> {
        find_declared_primary_attack_target(self.effect_data)
    }

    pub fn declared_primary_block(&self) -> bool {
        has_declared_primary_block(self.effect_data)
    }

    pub fn uses_typed_primary_preamble(&self) -> bool {
        self.declared_primary_attack_target().is_some() || self.declared_primary_block()
    }

    pub fn declared_discard_from_hand_count(&self) -> Option<(AmountSource, AmountSource)> {
        find_declared_choice_count(self.effect_data, Pile::Hand, &[ChoiceAction::Discard])
    }

    pub fn declared_exhaust_from_hand_count(&self) -> Option<(AmountSource, AmountSource)> {
        find_declared_choice_count(
            self.effect_data,
            Pile::Hand,
            &[ChoiceAction::Exhaust, ChoiceAction::ExhaustAndGainEnergy],
        )
    }

    pub fn declared_scry_count(&self) -> Option<AmountSource> {
        find_declared_scry_count(self.effect_data)
    }

    pub fn declared_draw_count(&self) -> Option<AmountSource> {
        find_declared_draw_count(self.effect_data)
    }

    pub fn declared_channel_orbs(&self) -> Vec<(OrbType, AmountSource)> {
        let mut hints = Vec::new();
        collect_declared_channel_orbs(self.effect_data, &mut hints);
        hints
    }

    pub fn declared_evoke_count(&self) -> Option<AmountSource> {
        find_declared_evoke_count(self.effect_data)
    }

    pub fn uses_declared_x_cost(&self) -> bool {
        effect_slice_uses_x_cost(self.effect_data)
    }

    pub fn declared_x_cost_amounts(&self) -> Vec<AmountSource> {
        let mut amounts = Vec::new();
        collect_declared_x_cost_amounts(self.effect_data, &mut amounts);
        amounts
    }

    pub fn uses_x_cost(&self) -> bool {
        self.uses_declared_x_cost() || self.play_hints().x_cost
    }

    pub fn uses_multi_hit_hint(&self) -> bool {
        self.play_hints().multi_hit
    }

    pub fn draws_cards_hint(&self) -> bool {
        self.play_hints().draws_cards
    }

    pub fn discards_cards_hint(&self) -> bool {
        self.play_hints().discards_cards
    }

    pub fn declared_player_statuses(&self) -> Vec<StatusId> {
        let mut statuses = Vec::new();
        collect_declared_player_statuses(self.effect_data, &mut statuses);
        statuses
    }

    pub fn power_embedding_flags(&self) -> (bool, bool) {
        let statuses = self.declared_player_statuses();
        let strength_like = statuses.iter().any(|status| {
            matches!(
                *status,
                crate::status_ids::sid::STRENGTH
                    | crate::status_ids::sid::RUSHDOWN
                    | crate::status_ids::sid::HEATSINK
                    | crate::status_ids::sid::STORM
                    | crate::status_ids::sid::STATIC_DISCHARGE
                    | crate::status_ids::sid::ELECTRODYNAMICS
            )
        });
        let dexterity_like = statuses.iter().any(|status| {
            matches!(
                *status,
                crate::status_ids::sid::DEXTERITY
                    | crate::status_ids::sid::MENTAL_FORTRESS
                    | crate::status_ids::sid::LIKE_WATER
                    | crate::status_ids::sid::NIRVANA
                    | crate::status_ids::sid::WAVE_OF_THE_HAND
            )
        });
        (strength_like, dexterity_like)
    }

    #[cfg(test)]
    pub fn has_test_marker(&self, marker: &str) -> bool {
        self.test_markers().iter().any(|candidate| *candidate == marker)
    }

    #[cfg(test)]
    pub fn test_markers(&self) -> Vec<&'static str> {
        let mut markers = Vec::new();
        let traits = self.runtime_traits();
        if traits.innate {
            add_test_marker(&mut markers, "innate");
        }
        if traits.retain {
            add_test_marker(&mut markers, "retain");
        }
        if traits.ethereal {
            add_test_marker(&mut markers, "ethereal");
        }
        if traits.unplayable {
            add_test_marker(&mut markers, "unplayable");
        }
        if traits.limit_cards_per_turn {
            add_test_marker(&mut markers, "limit_cards_per_turn");
        }
        if traits.unremovable {
            add_test_marker(&mut markers, "unremovable");
        }

        for trigger in self.runtime_triggers() {
            match trigger {
                CardRuntimeTrigger::CanPlay(CanPlayRule::OnlyAttackInHand) => {
                    add_test_marker(&mut markers, "only_attack_in_hand");
                }
                CardRuntimeTrigger::CanPlay(CanPlayRule::OnlyAttacksInHand) => {
                    add_test_marker(&mut markers, "only_attacks_in_hand");
                }
                CardRuntimeTrigger::CanPlay(CanPlayRule::OnlyEmptyDraw) => {
                    add_test_marker(&mut markers, "only_empty_draw");
                }
                CardRuntimeTrigger::ModifyCost(CostModifierRule::ReduceOnHpLoss) => {
                    add_test_marker(&mut markers, "cost_reduce_on_hp_loss");
                }
                CardRuntimeTrigger::ModifyCost(CostModifierRule::ReducePerPower) => {
                    add_test_marker(&mut markers, "reduce_cost_per_power");
                }
                CardRuntimeTrigger::ModifyCost(CostModifierRule::ReduceOnDiscard) => {
                    add_test_marker(&mut markers, "cost_reduce_on_discard");
                }
                CardRuntimeTrigger::ModifyCost(CostModifierRule::IncreaseOnHpLoss) => {
                    add_test_marker(&mut markers, "cost_increase_on_hp_loss");
                }
                CardRuntimeTrigger::ModifyDamage(DamageModifierRule::HeavyBlade) => {
                    add_test_marker(&mut markers, "heavy_blade");
                }
                CardRuntimeTrigger::ModifyDamage(DamageModifierRule::DamagePlusMantra) => {
                    add_test_marker(&mut markers, "damage_plus_mantra");
                }
                CardRuntimeTrigger::ModifyDamage(DamageModifierRule::PerfectedStrike) => {
                    add_test_marker(&mut markers, "perfected_strike");
                }
                CardRuntimeTrigger::ModifyDamage(DamageModifierRule::Rampage) => {
                    add_test_marker(&mut markers, "rampage");
                }
                CardRuntimeTrigger::ModifyDamage(DamageModifierRule::GlassKnife) => {
                    add_test_marker(&mut markers, "glass_knife");
                }
                CardRuntimeTrigger::ModifyDamage(DamageModifierRule::RitualDagger) => {
                    add_test_marker(&mut markers, "ritual_dagger");
                }
                CardRuntimeTrigger::ModifyDamage(DamageModifierRule::SearingBlow) => {
                    add_test_marker(&mut markers, "searing_blow");
                }
                CardRuntimeTrigger::ModifyDamage(DamageModifierRule::DamageRandomXTimes) => {
                    add_test_marker(&mut markers, "damage_random_x_times");
                }
                CardRuntimeTrigger::ModifyDamage(DamageModifierRule::ClawScaling) => {
                    add_test_marker(&mut markers, "claw_scaling");
                }
                CardRuntimeTrigger::ModifyDamage(DamageModifierRule::DamagePerLightning) => {
                    add_test_marker(&mut markers, "damage_per_lightning");
                }
                CardRuntimeTrigger::ModifyDamage(DamageModifierRule::DamageFromDrawPile) => {
                    add_test_marker(&mut markers, "damage_from_draw_pile");
                }
                CardRuntimeTrigger::OnDraw(OnDrawRule::LoseEnergy) => {
                    add_test_marker(&mut markers, "lose_energy_on_draw");
                }
                CardRuntimeTrigger::OnDraw(OnDrawRule::CopySelf) => {
                    add_test_marker(&mut markers, "copy_on_draw");
                }
                CardRuntimeTrigger::OnDraw(OnDrawRule::DeusExMachina) => {
                    add_test_marker(&mut markers, "deus_ex_machina");
                }
                CardRuntimeTrigger::OnDiscard(OnDiscardRule::DrawCards) => {
                    add_test_marker(&mut markers, "draw_on_discard");
                }
                CardRuntimeTrigger::OnDiscard(OnDiscardRule::GainEnergy) => {
                    add_test_marker(&mut markers, "energy_on_discard");
                }
                CardRuntimeTrigger::OnRetain(OnRetainRule::GrowBlock) => {
                    add_test_marker(&mut markers, "grow_block_on_retain");
                }
                CardRuntimeTrigger::PostPlay(PostPlayRule::ShuffleIntoDraw) => {
                    add_test_marker(&mut markers, "shuffle_self_into_draw");
                }
                CardRuntimeTrigger::PostPlay(PostPlayRule::EndTurn) => {
                    add_test_marker(&mut markers, "end_turn");
                }
                CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Damage) => {
                    add_test_marker(&mut markers, "end_turn_damage");
                }
                CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Regret) => {
                    add_test_marker(&mut markers, "end_turn_regret");
                }
                CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Weak) => {
                    add_test_marker(&mut markers, "end_turn_weak");
                }
                CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Frail) => {
                    add_test_marker(&mut markers, "end_turn_frail");
                }
                CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::AddCopy) => {
                    add_test_marker(&mut markers, "add_copy_end_turn");
                }
                CardRuntimeTrigger::WhileInHand(WhileInHandRule::PainOnOtherCardPlayed) => {
                    add_test_marker(&mut markers, "damage_on_draw");
                }
                _ => {}
            }
        }

        if self.play_hints().x_cost {
            add_test_marker(&mut markers, "x_cost");
        }
        if self.play_hints().multi_hit {
            add_test_marker(&mut markers, "multi_hit");
        }
        if self.play_hints().draws_cards {
            add_test_marker(&mut markers, "draw");
        }
        if self.play_hints().discards_cards {
            add_test_marker(&mut markers, "discard");
        }

        match self.play_hints().block_hint {
            Some(CardBlockHint::XTimes) => add_test_marker(&mut markers, "block_x_times"),
            Some(CardBlockHint::IfSkill) => add_test_marker(&mut markers, "block_if_skill"),
            Some(CardBlockHint::IfNoBlock) => add_test_marker(&mut markers, "block_if_no_block"),
            Some(CardBlockHint::BulkCountTimesBaseBlock) => {
                add_test_marker(&mut markers, "bulk_count_times_block");
            }
            Some(CardBlockHint::UsesCardMisc) => add_test_marker(&mut markers, "uses_card_misc"),
            None => {}
        }

        match self.evoke_hint() {
            Some(CardEvokeHint::Fixed(_)) => add_test_marker(&mut markers, "evoke_orb"),
            Some(CardEvokeHint::XCost) => add_test_marker(&mut markers, "evoke_orb_x"),
            Some(CardEvokeHint::XCostPlus(_)) => add_test_marker(&mut markers, "evoke_orb_x_plus_1"),
            None => {}
        }
        if self.play_hints().channel_evoked_orb {
            add_test_marker(&mut markers, "channel_evoked");
        }

        collect_test_markers_from_effects(self.effect_data, &mut markers);
        for marker in supplemental_test_markers(self.id) {
            add_test_marker(&mut markers, marker);
        }
        markers.sort_unstable();
        markers
    }
}

#[cfg(test)]
fn add_test_marker(markers: &mut Vec<&'static str>, marker: &'static str) {
    if !markers.contains(&marker) {
        markers.push(marker);
    }
}

#[cfg(test)]
fn collect_test_markers_from_effects(effects: &[Effect], markers: &mut Vec<&'static str>) {
    for effect in effects {
        match effect {
            Effect::Simple(simple) => collect_test_markers_from_simple(simple, markers),
            Effect::Conditional(condition, then_effects, else_effects) => {
                collect_test_markers_from_conditional(condition, then_effects, else_effects, markers);
                collect_test_markers_from_effects(then_effects, markers);
                collect_test_markers_from_effects(else_effects, markers);
            }
            Effect::ChooseCards {
                source,
                action,
                post_choice_draw,
                ..
            } => {
                if matches!(source, Pile::Hand) {
                    match action {
                        ChoiceAction::Discard => {
                            add_test_marker(markers, "discard");
                        }
                        ChoiceAction::DiscardForEffect => {
                            add_test_marker(markers, "discard");
                            add_test_marker(markers, "discard_gain_energy");
                        }
                        ChoiceAction::PutOnTopAtCostZero | ChoiceAction::PutOnBottomAtCostZero => {
                            add_test_marker(markers, "setup");
                        }
                        ChoiceAction::StoreCardForNextTurnCopies => {
                            add_test_marker(markers, "meditate");
                        }
                        ChoiceAction::ExhaustAndGainEnergy => {
                            add_test_marker(markers, "discard_gain_energy");
                        }
                        _ => {}
                    }
                }
                if !matches!(post_choice_draw, AmountSource::Fixed(0)) {
                    add_test_marker(markers, "draw");
                }
            }
            Effect::ForEachInPile { filter, action, .. } => match action {
                crate::effects::declarative::BulkAction::Discard => match filter {
                    CardFilter::NonAttacks => add_test_marker(markers, "discard_non_attacks"),
                    _ => add_test_marker(markers, "discard"),
                },
                _ => {}
            },
            Effect::Discover(_) | Effect::ChooseNamedOptions(_) | Effect::ChooseScaledNamedOptions(_) => {
                add_test_marker(markers, "choice");
            }
            Effect::GenerateRandomCardsToHand { pool, .. } => {
                if matches!(pool, crate::effects::declarative::GeneratedCardPool::Skill) {
                    add_test_marker(markers, "random_skill_to_hand");
                }
            }
            Effect::GenerateRandomCardsToDraw { .. } => {}
            Effect::GenerateDiscoveryChoice { pool, .. } => {
                add_test_marker(markers, "choice");
                if matches!(
                    pool,
                    crate::effects::declarative::GeneratedCardPool::AnyColorAttackRarityWeighted
                ) {
                    add_test_marker(markers, "foreign_influence");
                }
            }
            Effect::ExtraHits(AmountSource::LivingEnemyCount) => {
                add_test_marker(markers, "damage_per_enemy");
            }
            Effect::ExtraHits(_) => add_test_marker(markers, "multi_hit"),
        }
    }
}

#[cfg(test)]
fn collect_test_markers_from_conditional(
    condition: &crate::effects::declarative::Condition,
    then_effects: &[Effect],
    else_effects: &[Effect],
    markers: &mut Vec<&'static str>,
) {
    use crate::effects::declarative::Condition as C;

    match condition {
        C::LastCardType(CardType::Attack) => {
            if effect_slice_has_gain_energy(then_effects) {
                add_test_marker(markers, "energy_if_last_attack");
            }
            if effect_slice_has_status(then_effects, Target::SelectedEnemy, crate::status_ids::sid::WEAKENED) {
                add_test_marker(markers, "weak_if_last_attack");
            }
        }
        C::LastCardType(CardType::Skill) => {
            if effect_slice_has_status(then_effects, Target::SelectedEnemy, crate::status_ids::sid::VULNERABLE) {
                add_test_marker(markers, "vuln_if_last_skill");
            }
        }
        C::InStance(Stance::Wrath) => {
            if effect_slice_has_gain_block(then_effects) {
                add_test_marker(markers, "extra_block_in_wrath");
            }
        }
        C::InStance(Stance::Calm) => {
            if effect_slice_has_draw(then_effects) && effect_slice_has_stance_change(else_effects, Stance::Calm) {
                add_test_marker(markers, "if_calm_draw_else_calm");
            }
        }
        C::EnemyAttacking => {
            if effect_slice_has_stance_change(then_effects, Stance::Calm) {
                add_test_marker(markers, "calm_if_enemy_attacking");
            }
        }
        C::EnemyHasStatus(crate::status_ids::sid::POISON) => {
            if effect_slice_has_attack(then_effects) {
                add_test_marker(markers, "double_if_poisoned");
            }
        }
        C::EnemyHasStatus(crate::status_ids::sid::WEAKENED) => {
            if effect_slice_has_gain_energy(then_effects) && effect_slice_has_draw(then_effects) {
                add_test_marker(markers, "if_weak_energy_draw");
            }
        }
        C::DiscardedThisTurn => {
            if effect_slice_has_gain_energy(then_effects) {
                add_test_marker(markers, "refund_energy_on_discard");
            }
        }
        _ => {}
    }
}

#[cfg(test)]
fn collect_test_markers_from_simple(simple: &SimpleEffect, markers: &mut Vec<&'static str>) {
    use crate::effects::declarative::BoolFlag;

    match simple {
        SimpleEffect::AddStatus(target, status, _)
        | SimpleEffect::SetStatus(target, status, _) => {
            collect_test_markers_from_status(*target, *status, markers);
        }
        SimpleEffect::MultiplyStatus(Target::SelectedEnemy, crate::status_ids::sid::POISON, factor) => {
            match factor {
                2 => add_test_marker(markers, "catalyst_double"),
                3 => add_test_marker(markers, "catalyst_triple"),
                _ => {}
            }
        }
        SimpleEffect::DrawCards(AmountSource::Fixed(10)) => {
            add_test_marker(markers, "draw");
            add_test_marker(markers, "draw_to_ten");
        }
        SimpleEffect::DrawCards(_)
        | SimpleEffect::DrawCardsThenDiscardDrawnNonZeroCost(_) => {
            add_test_marker(markers, "draw");
        }
        SimpleEffect::DrawToHandSize(amount) => match amount {
            AmountSource::Fixed(10) => add_test_marker(markers, "draw_to_ten"),
            _ => add_test_marker(markers, "draw_to_n"),
        },
        SimpleEffect::GainEnergy(_) => add_test_marker(markers, "gain_energy"),
        SimpleEffect::GainBlock(AmountSource::TotalUnblockedDamage) => {
            add_test_marker(markers, "block_from_damage");
        }
        SimpleEffect::GainBlock(AmountSource::HandSize)
        | SimpleEffect::GainBlock(AmountSource::HandSizeAtPlay)
        | SimpleEffect::GainBlock(AmountSource::HandSizeAtPlayPlus(_)) => {
            add_test_marker(markers, "block_per_card_in_hand");
        }
        SimpleEffect::GainMantra(_) => add_test_marker(markers, "mantra"),
        SimpleEffect::Scry(_) => add_test_marker(markers, "scry"),
        SimpleEffect::AddCard(card_id, pile, _) | SimpleEffect::AddCardWithMisc(card_id, pile, _, _) => {
            match (*card_id, *pile) {
                ("Shiv", Pile::Hand) => add_test_marker(markers, "add_shivs"),
                ("Smite", Pile::Hand) => add_test_marker(markers, "add_smite_to_hand"),
                ("Safety", Pile::Hand) => add_test_marker(markers, "add_safety_to_hand"),
                ("ThroughViolence", Pile::Draw) => add_test_marker(markers, "add_through_violence_to_draw"),
                ("Insight", Pile::Draw) => add_test_marker(markers, "insight_to_draw"),
                _ => {}
            }
        }
        SimpleEffect::ChannelOrb(orb, _) => match orb {
            OrbType::Lightning => add_test_marker(markers, "channel_lightning"),
            OrbType::Frost => add_test_marker(markers, "channel_frost"),
            OrbType::Dark => add_test_marker(markers, "channel_dark"),
            OrbType::Plasma => add_test_marker(markers, "channel_plasma"),
            OrbType::Empty => {}
        },
        SimpleEffect::EvokeOrb(_) => add_test_marker(markers, "evoke_orb"),
        SimpleEffect::ChangeStance(Stance::Neutral) => add_test_marker(markers, "exit_stance"),
        SimpleEffect::SetFlag(flag) => match flag {
            BoolFlag::SkipEnemyTurn => add_test_marker(markers, "skip_enemy_turn"),
            BoolFlag::NextAttackFree => add_test_marker(markers, "next_attack_free"),
            BoolFlag::BulletTime => add_test_marker(markers, "bullet_time"),
            BoolFlag::RetainHand => add_test_marker(markers, "well_laid_plans"),
            _ => {}
        },
        SimpleEffect::ObtainRandomPotion => add_test_marker(markers, "alchemize"),
        SimpleEffect::DrawRandomCardsFromPileToHand(Pile::Draw, CardFilter::Skills, _) => {
            add_test_marker(markers, "random_skill_to_hand");
        }
        SimpleEffect::DealDamage(_, AmountSource::LivingEnemyCount) => {
            add_test_marker(markers, "damage_per_enemy");
        }
        SimpleEffect::DiscardRandomCardsFromPile(_, _) => add_test_marker(markers, "discard_random"),
        _ => {}
    }
}

#[cfg(test)]
fn collect_test_markers_from_status(
    target: Target,
    status: StatusId,
    markers: &mut Vec<&'static str>,
) {
    use crate::status_ids::sid;

    match (target, status) {
        (Target::SelectedEnemy | Target::RandomEnemy, sid::VULNERABLE) => add_test_marker(markers, "vulnerable"),
        (Target::SelectedEnemy | Target::RandomEnemy, sid::WEAKENED) => add_test_marker(markers, "weak"),
        (Target::AllEnemies, sid::WEAKENED) => add_test_marker(markers, "weak_all"),
        (Target::RandomEnemy, sid::POISON) => add_test_marker(markers, "poison_random_multi"),
        (Target::SelectedEnemy, sid::POISON) => add_test_marker(markers, "poison"),
        (Target::AllEnemies, sid::POISON) => add_test_marker(markers, "poison_all"),
        (Target::SelectedEnemy, sid::BLOCK_RETURN) => add_test_marker(markers, "apply_block_return"),
        (Target::Player | Target::SelfEntity, sid::MANTRA) => add_test_marker(markers, "mantra"),
        (Target::Player | Target::SelfEntity, sid::VIGOR) => add_test_marker(markers, "vigor"),
        (Target::Player | Target::SelfEntity, sid::DEXTERITY) => add_test_marker(markers, "gain_dexterity"),
        (Target::Player | Target::SelfEntity, sid::NEXT_TURN_BLOCK) => add_test_marker(markers, "next_turn_block"),
        (Target::Player | Target::SelfEntity, sid::ENERGIZED | sid::DOPPELGANGER_ENERGY) => {
            add_test_marker(markers, "next_turn_energy");
        }
        (Target::Player | Target::SelfEntity, sid::DOPPELGANGER_DRAW | sid::DRAW_CARD | sid::DRAW) => {
            add_test_marker(markers, "draw_next_turn");
        }
        (Target::Player | Target::SelfEntity, sid::FOCUS) => add_test_marker(markers, "gain_focus"),
        (Target::Player | Target::SelfEntity, sid::THORNS) => add_test_marker(markers, "thorns"),
        (Target::Player | Target::SelfEntity, sid::AFTER_IMAGE) => add_test_marker(markers, "after_image"),
        (Target::Player | Target::SelfEntity, sid::THOUSAND_CUTS) => add_test_marker(markers, "thousand_cuts"),
        (Target::Player | Target::SelfEntity, sid::NOXIOUS_FUMES) => add_test_marker(markers, "noxious_fumes"),
        (Target::Player | Target::SelfEntity, sid::INFINITE_BLADES) => add_test_marker(markers, "infinite_blades"),
        (Target::Player | Target::SelfEntity, sid::ENVENOM) => add_test_marker(markers, "envenom"),
        (Target::Player | Target::SelfEntity, sid::ACCURACY) => add_test_marker(markers, "accuracy"),
        (Target::Player | Target::SelfEntity, sid::TOOLS_OF_THE_TRADE) => {
            add_test_marker(markers, "tools_of_the_trade");
        }
        (Target::Player | Target::SelfEntity, sid::RETAIN_CARDS) => add_test_marker(markers, "well_laid_plans"),
        (Target::Player | Target::SelfEntity, sid::RUSHDOWN) => add_test_marker(markers, "on_wrath_draw"),
        (Target::Player | Target::SelfEntity, sid::MENTAL_FORTRESS) => {
            add_test_marker(markers, "on_stance_change_block");
        }
        (Target::Player | Target::SelfEntity, sid::NIRVANA) => add_test_marker(markers, "on_scry_block"),
        (Target::Player | Target::SelfEntity, sid::BATTLE_HYMN) => add_test_marker(markers, "add_smite_to_hand"),
        (Target::Player | Target::SelfEntity, sid::WAVE_OF_THE_HAND) => {
            add_test_marker(markers, "apply_block_return");
        }
        (Target::Player | Target::SelfEntity, sid::BLUR) => add_test_marker(markers, "retain_block"),
        (Target::Player | Target::SelfEntity, sid::BURST) => add_test_marker(markers, "burst"),
        (Target::Player | Target::SelfEntity, sid::BULLET_TIME) => add_test_marker(markers, "bullet_time"),
        (Target::Player | Target::SelfEntity, sid::NEXT_ATTACK_FREE) => {
            add_test_marker(markers, "next_attack_free");
        }
        (Target::SelectedEnemy, sid::CONSTRICTED) => add_test_marker(markers, "choke"),
        (Target::SelectedEnemy, sid::CORPSE_EXPLOSION) => add_test_marker(markers, "corpse_explosion"),
        _ => {}
    }
}

#[cfg(test)]
fn effect_slice_has_status(effects: &[Effect], target: Target, status: StatusId) -> bool {
    effects.iter().any(|effect| match effect {
        Effect::Simple(SimpleEffect::AddStatus(candidate_target, candidate_status, _))
        | Effect::Simple(SimpleEffect::SetStatus(candidate_target, candidate_status, _)) => {
            *candidate_target == target && *candidate_status == status
        }
        Effect::Conditional(_, then_effects, else_effects) => {
            effect_slice_has_status(then_effects, target, status)
                || effect_slice_has_status(else_effects, target, status)
        }
        _ => false,
    })
}

#[cfg(test)]
fn effect_slice_has_gain_energy(effects: &[Effect]) -> bool {
    effects.iter().any(|effect| match effect {
        Effect::Simple(SimpleEffect::GainEnergy(_)) => true,
        Effect::Conditional(_, then_effects, else_effects) => {
            effect_slice_has_gain_energy(then_effects) || effect_slice_has_gain_energy(else_effects)
        }
        _ => false,
    })
}

#[cfg(test)]
fn effect_slice_has_gain_block(effects: &[Effect]) -> bool {
    effects.iter().any(|effect| match effect {
        Effect::Simple(SimpleEffect::GainBlock(_)) => true,
        Effect::Conditional(_, then_effects, else_effects) => {
            effect_slice_has_gain_block(then_effects) || effect_slice_has_gain_block(else_effects)
        }
        _ => false,
    })
}

#[cfg(test)]
fn effect_slice_has_draw(effects: &[Effect]) -> bool {
    effects.iter().any(|effect| match effect {
        Effect::Simple(SimpleEffect::DrawCards(_))
        | Effect::Simple(SimpleEffect::DrawCardsThenDiscardDrawnNonZeroCost(_))
        | Effect::Simple(SimpleEffect::DrawToHandSize(_)) => true,
        Effect::Conditional(_, then_effects, else_effects) => {
            effect_slice_has_draw(then_effects) || effect_slice_has_draw(else_effects)
        }
        _ => false,
    })
}

#[cfg(test)]
fn effect_slice_has_attack(effects: &[Effect]) -> bool {
    effects.iter().any(|effect| match effect {
        Effect::Simple(SimpleEffect::DealDamage(_, _)) => true,
        Effect::Conditional(_, then_effects, else_effects) => {
            effect_slice_has_attack(then_effects) || effect_slice_has_attack(else_effects)
        }
        _ => false,
    })
}

#[cfg(test)]
fn effect_slice_has_stance_change(effects: &[Effect], stance: Stance) -> bool {
    effects.iter().any(|effect| match effect {
        Effect::Simple(SimpleEffect::ChangeStance(candidate)) => *candidate == stance,
        Effect::Conditional(_, then_effects, else_effects) => {
            effect_slice_has_stance_change(then_effects, stance)
                || effect_slice_has_stance_change(else_effects, stance)
        }
        _ => false,
    })
}

#[cfg(test)]
fn supplemental_test_markers(id: &str) -> &'static [&'static str] {
    match id {
        "Rebound" | "Rebound+" => &["next_card_to_top"],
        "Weave" | "Weave+" => &["return_on_scry"],
        "PathToVictory" | "PathToVictory+" | "PressurePoints" | "PressurePoints+" => {
            &["pressure_points"]
        }
        "Judgement" | "Judgement+" => &["judgement"],
        "LessonLearned" | "LessonLearned+" => &["lesson_learned"],
        "Wish" | "Wish+" => &["wish"],
        "DeusExMachina" | "DeusExMachina+" => &["deus_ex_machina"],
        "Collect" | "Collect+" => &["mantra"],
        "StormOfSteel" | "StormOfSteel+" | "Storm of Steel" | "Storm of Steel+" => {
            &["storm_of_steel", "add_shivs"]
        }
        "Calculated Gamble" | "Calculated Gamble+" | "CalculatedGamble" | "CalculatedGamble+" => {
            &["calculated_gamble", "discard", "draw"]
        }
        "BouncingFlask" | "BouncingFlask+" => &["poison_random_multi"],
        "Piercing Wail" | "PiercingWail" | "Piercing Wail+" | "PiercingWail+" => {
            &["reduce_strength_all_temp"]
        }
        "Flechettes" | "Flechettes+" => &["flechettes", "multi_hit"],
        "Finisher" | "Finisher+" => &["finisher", "multi_hit"],
        "Meditate" | "Meditate+" => &["meditate"],
        "SecretWeapon" | "SecretWeapon+" => &["choice"],
        "SecretTechnique" | "SecretTechnique+" => &["choice"],
        "Discovery" | "Discovery+" => &["choice"],
        "Violence" | "Violence+" => &["draw"],
        "Panacea" | "Panacea+" => &["artifact"],
        "ForeignInfluence" | "ForeignInfluence+" => &["foreign_influence", "choice"],
        "Phantasmal Killer" | "Phantasmal Killer+" => &["phantasmal_killer"],
        "Scrawl" | "Scrawl+" => &["draw_to_ten"],
        _ => &[],
    }
}

fn find_declared_extra_hits(effects: &[Effect]) -> Option<AmountSource> {
    for effect in effects {
        match effect {
            Effect::ExtraHits(source) => return Some(*source),
            Effect::Conditional(_, then_effects, else_effects) => {
                if let Some(source) = find_declared_extra_hits(then_effects) {
                    return Some(source);
                }
                if let Some(source) = find_declared_extra_hits(else_effects) {
                    return Some(source);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_declared_stance_change(effects: &[Effect]) -> Option<Stance> {
    for effect in effects {
        match effect {
            Effect::Simple(SimpleEffect::ChangeStance(stance)) => return Some(*stance),
            Effect::Conditional(_, then_effects, else_effects) => {
                if let Some(stance) = find_declared_stance_change(then_effects) {
                    return Some(stance);
                }
                if let Some(stance) = find_declared_stance_change(else_effects) {
                    return Some(stance);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_unconditional_stance_change(effects: &[Effect]) -> Option<Stance> {
    effects.iter().find_map(|effect| match effect {
        Effect::Simple(SimpleEffect::ChangeStance(stance)) => Some(*stance),
        _ => None,
    })
}

fn declared_stance_name(stance: Option<Stance>) -> Option<&'static str> {
    match stance? {
        Stance::Wrath => Some("Wrath"),
        Stance::Calm => Some("Calm"),
        Stance::Neutral => Some("Neutral"),
        Stance::Divinity => Some("Divinity"),
    }
}

fn find_declared_draw_count(effects: &[Effect]) -> Option<AmountSource> {
    for effect in effects {
        match effect {
            Effect::Simple(SimpleEffect::DrawCards(amount))
            | Effect::Simple(SimpleEffect::DrawCardsThenDiscardDrawnNonZeroCost(amount)) => {
                return Some(*amount);
            }
            Effect::Conditional(_, then_effects, else_effects) => {
                if let Some(amount) = find_declared_draw_count(then_effects) {
                    return Some(amount);
                }
                if let Some(amount) = find_declared_draw_count(else_effects) {
                    return Some(amount);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_declared_all_enemy_damage(effects: &[Effect]) -> Option<AmountSource> {
    for effect in effects {
        match effect {
            Effect::Simple(SimpleEffect::DealDamage(Target::AllEnemies, amount)) => return Some(*amount),
            Effect::Conditional(_, then_effects, else_effects) => {
                if let Some(amount) = find_declared_all_enemy_damage(then_effects) {
                    return Some(amount);
                }
                if let Some(amount) = find_declared_all_enemy_damage(else_effects) {
                    return Some(amount);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_declared_primary_attack_target(effects: &[Effect]) -> Option<Target> {
    for effect in effects {
        match effect {
            Effect::Simple(SimpleEffect::DealDamage(target, _))
                if matches!(target, Target::SelectedEnemy | Target::AllEnemies | Target::RandomEnemy) =>
            {
                return Some(*target);
            }
            Effect::Conditional(_, then_effects, else_effects) => {
                if let Some(target) = find_declared_primary_attack_target(then_effects) {
                    return Some(target);
                }
                if let Some(target) = find_declared_primary_attack_target(else_effects) {
                    return Some(target);
                }
            }
            _ => {}
        }
    }
    None
}

fn has_declared_primary_block(effects: &[Effect]) -> bool {
    for effect in effects {
        match effect {
            Effect::Simple(SimpleEffect::GainBlock(AmountSource::Block)) => return true,
            Effect::Conditional(_, then_effects, else_effects) => {
                if has_declared_primary_block(then_effects) {
                    return true;
                }
                if has_declared_primary_block(else_effects) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn find_declared_choice_count(
    effects: &[Effect],
    source: Pile,
    actions: &[ChoiceAction],
) -> Option<(AmountSource, AmountSource)> {
    for effect in effects {
        match effect {
            Effect::ChooseCards {
                source: candidate_source,
                action,
                min_picks,
                max_picks,
                ..
            } if *candidate_source == source && actions.contains(action) => {
                return Some((*min_picks, *max_picks));
            }
            Effect::Conditional(_, then_effects, else_effects) => {
                if let Some(hint) = find_declared_choice_count(then_effects, source, actions) {
                    return Some(hint);
                }
                if let Some(hint) = find_declared_choice_count(else_effects, source, actions) {
                    return Some(hint);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_declared_scry_count(effects: &[Effect]) -> Option<AmountSource> {
    for effect in effects {
        match effect {
            Effect::Simple(SimpleEffect::Scry(amount)) => return Some(*amount),
            Effect::Conditional(_, then_effects, else_effects) => {
                if let Some(amount) = find_declared_scry_count(then_effects) {
                    return Some(amount);
                }
                if let Some(amount) = find_declared_scry_count(else_effects) {
                    return Some(amount);
                }
            }
            _ => {}
        }
    }
    None
}

fn collect_declared_channel_orbs(
    effects: &[Effect],
    hints: &mut Vec<(OrbType, AmountSource)>,
) {
    for effect in effects {
        match effect {
            Effect::Simple(SimpleEffect::ChannelOrb(orb_type, amount)) => {
                hints.push((*orb_type, *amount));
            }
            Effect::Conditional(_, then_effects, else_effects) => {
                collect_declared_channel_orbs(then_effects, hints);
                collect_declared_channel_orbs(else_effects, hints);
            }
            _ => {}
        }
    }
}

fn find_declared_evoke_count(effects: &[Effect]) -> Option<AmountSource> {
    for effect in effects {
        match effect {
            Effect::Simple(SimpleEffect::EvokeOrb(amount)) => return Some(*amount),
            Effect::Conditional(_, then_effects, else_effects) => {
                if let Some(amount) = find_declared_evoke_count(then_effects) {
                    return Some(amount);
                }
                if let Some(amount) = find_declared_evoke_count(else_effects) {
                    return Some(amount);
                }
            }
            _ => {}
        }
    }
    None
}

fn collect_declared_player_statuses(effects: &[Effect], statuses: &mut Vec<StatusId>) {
    for effect in effects {
        match effect {
            Effect::Simple(SimpleEffect::AddStatus(
                Target::Player | Target::SelfEntity,
                status_id,
                _,
            )) => {
                if !statuses.contains(status_id) {
                    statuses.push(*status_id);
                }
            }
            Effect::Conditional(_, then_effects, else_effects) => {
                collect_declared_player_statuses(then_effects, statuses);
                collect_declared_player_statuses(else_effects, statuses);
            }
            _ => {}
        }
    }
}

fn amount_uses_x_cost(source: &AmountSource) -> bool {
    matches!(
        source,
        AmountSource::XCost | AmountSource::XCostPlus(_) | AmountSource::MagicPlusX
    )
}

fn collect_declared_x_cost_amounts(effects: &[Effect], amounts: &mut Vec<AmountSource>) {
    for effect in effects {
        match effect {
            Effect::Simple(simple) => collect_simple_x_cost_amounts(simple, amounts),
            Effect::Conditional(_, then_effects, else_effects) => {
                collect_declared_x_cost_amounts(then_effects, amounts);
                collect_declared_x_cost_amounts(else_effects, amounts);
            }
            Effect::ChooseCards { min_picks, max_picks, .. } => {
                if amount_uses_x_cost(min_picks) {
                    amounts.push(*min_picks);
                }
                if amount_uses_x_cost(max_picks) {
                    amounts.push(*max_picks);
                }
            }
            Effect::ExtraHits(source) => {
                if amount_uses_x_cost(source) {
                    amounts.push(*source);
                }
            }
            Effect::ForEachInPile { .. }
            | Effect::Discover(_)
            | Effect::ChooseNamedOptions(_)
            | Effect::ChooseScaledNamedOptions(_)
            | Effect::GenerateRandomCardsToHand { .. }
            | Effect::GenerateRandomCardsToDraw { .. }
            | Effect::GenerateDiscoveryChoice { .. } => {}
        }
    }
}

fn collect_simple_x_cost_amounts(effect: &SimpleEffect, amounts: &mut Vec<AmountSource>) {
    match effect {
        SimpleEffect::AddStatus(_, _, source)
        | SimpleEffect::SetStatus(_, _, source)
        | SimpleEffect::DrawCards(source)
        | SimpleEffect::GainEnergy(source)
        | SimpleEffect::GainBlock(source)
        | SimpleEffect::ModifyHp(source)
        | SimpleEffect::GainMantra(source)
        | SimpleEffect::Scry(source)
        | SimpleEffect::DrawCardsThenDiscardDrawnNonZeroCost(source)
        | SimpleEffect::AddCard(_, _, source)
        | SimpleEffect::AddCardWithMisc(_, _, source, _)
        | SimpleEffect::ChannelOrb(_, source)
        | SimpleEffect::EvokeOrb(source)
        | SimpleEffect::DealDamage(_, source)
        | SimpleEffect::Judgement(source)
        | SimpleEffect::HealHp(_, source)
        | SimpleEffect::ModifyMaxHp(source)
        | SimpleEffect::ModifyMaxEnergy(source)
        | SimpleEffect::ModifyGold(source) => {
            if amount_uses_x_cost(source) {
                amounts.push(*source);
            }
        }
        SimpleEffect::MultiplyStatus(_, _, _)
        | SimpleEffect::ChangeStance(_)
        | SimpleEffect::SetFlag(_)
        | SimpleEffect::ShuffleDiscardIntoDraw
        | SimpleEffect::ExhaustRandomCardFromHand
        | SimpleEffect::CopyThisCardTo(_)
        | SimpleEffect::GainBlockIfLastHandCardType(_, _)
        | SimpleEffect::DrawToHandSize(_)
        | SimpleEffect::TriggerMarks
        | SimpleEffect::DoubleEnergy
        | SimpleEffect::IncrementCounter(_, _)
        | SimpleEffect::ModifyPlayedCardCost(_)
        | SimpleEffect::ModifyPlayedCardBlock(_)
        | SimpleEffect::ModifyPlayedCardDamage(_)
        | SimpleEffect::SetRandomHandCardCost(_)
        | SimpleEffect::ObtainRandomPotion
        | SimpleEffect::TriggerDarkPassive
        | SimpleEffect::RemoveOrbSlot
        | SimpleEffect::EvokeAndRechannelFrontOrb
        | SimpleEffect::ChannelRandomOrb(_)
        | SimpleEffect::DiscardRandomCardsFromPile(_, _)
        | SimpleEffect::PlayTopCardOfDraw
        | SimpleEffect::ResolveFission { .. }
        | SimpleEffect::RemoveEnemyBlock(_)
        | SimpleEffect::UpgradeRandomCardFromPiles(_)
        | SimpleEffect::FleeCombat => {}
        SimpleEffect::DrawRandomCardsFromPileToHand(_, _, source) => {
            if amount_uses_x_cost(source) {
                amounts.push(*source);
            }
        }
    }
}

fn simple_effect_uses_x_cost(effect: &SimpleEffect) -> bool {
    let mut amounts = Vec::new();
    collect_simple_x_cost_amounts(effect, &mut amounts);
    !amounts.is_empty()
}

fn effect_slice_uses_x_cost(effects: &[Effect]) -> bool {
    effects.iter().any(effect_uses_x_cost)
}

fn effect_uses_x_cost(effect: &Effect) -> bool {
    match effect {
        Effect::Simple(simple) => simple_effect_uses_x_cost(simple),
        Effect::Conditional(_, then_effects, else_effects) => {
            effect_slice_uses_x_cost(then_effects) || effect_slice_uses_x_cost(else_effects)
        }
        Effect::ChooseCards {
            min_picks,
            max_picks,
            ..
        } => amount_uses_x_cost(min_picks) || amount_uses_x_cost(max_picks),
        Effect::ForEachInPile { .. }
        | Effect::Discover(_)
        | Effect::ChooseNamedOptions(_)
        | Effect::ChooseScaledNamedOptions(_)
        | Effect::GenerateRandomCardsToHand { .. }
        | Effect::GenerateRandomCardsToDraw { .. }
        | Effect::GenerateDiscoveryChoice { .. } => false,
        Effect::ExtraHits(source) => amount_uses_x_cost(source),
    }
}

// ---------------------------------------------------------------------------
// Card registry — lookup by ID (including "+" suffix for upgrades)
// ---------------------------------------------------------------------------

/// Static card registry. Populated with core Watcher cards + universals.
/// Cards not in the registry fall back to defaults (cost 1, attack, enemy target).
/// Global static card registry. Built once via OnceLock, shared by all engines.
static GLOBAL_REGISTRY: std::sync::OnceLock<CardRegistry> = std::sync::OnceLock::new();

/// Get or initialize the global card registry. First call builds it; subsequent calls return &'static ref.
pub fn global_registry() -> &'static CardRegistry {
    GLOBAL_REGISTRY.get_or_init(CardRegistry::new)
}

pub fn gameplay_def(card_id: &str) -> Option<&'static crate::gameplay::GameplayDef> {
    crate::gameplay::global_registry().card(card_id)
}

pub fn gameplay_export_defs() -> Vec<crate::gameplay::GameplayDef> {
    global_registry()
        .all_card_defs()
        .iter()
        .map(|card| crate::gameplay::GameplayDef {
            domain: crate::gameplay::GameplayDomain::Card,
            id: card.id.to_string(),
            name: card.name.to_string(),
            tags: Vec::new(),
            schema: crate::gameplay::GameplaySchema::Card(crate::gameplay::CardSchema {
                card_type: Some(card.card_type),
                target: Some(card.target),
                cost: Some(card.cost),
                exhausts: card.exhaust,
                upgraded_from: if card.id.ends_with('+') {
                    Some(card.id.trim_end_matches('+').to_string())
                } else {
                    None
                },
                runtime_traits: card.runtime_traits(),
                runtime_triggers: card.runtime_triggers().to_vec(),
                play_hints: card.play_hints().clone(),
                declared_player_statuses: card
                    .declared_player_statuses()
                    .into_iter()
                    .map(|status| status.0)
                    .collect(),
                declared_effect_count: card.declared_effect_count(),
                declared_extra_hits: card.declared_extra_hits().is_some(),
                declared_stance_change: card.declared_stance_change().is_some(),
                declared_all_enemy_damage: card.declared_all_enemy_damage(),
                declared_discard_from_hand: card.declared_discard_from_hand_count().map(|(min, max)| {
                    crate::gameplay::ChoiceCountHint { min, max }
                }),
                declared_exhaust_from_hand: card.declared_exhaust_from_hand_count().map(|(min, max)| {
                    crate::gameplay::ChoiceCountHint { min, max }
                }),
                declared_scry_count: card.declared_scry_count(),
                declared_channel_orbs: card
                    .declared_channel_orbs()
                    .into_iter()
                    .map(|(orb_type, count)| crate::gameplay::OrbCountHint { orb_type, count })
                    .collect(),
                declared_evoke_count: card.declared_evoke_count(),
                uses_x_cost: card.uses_declared_x_cost(),
                declared_x_cost_amounts: card.declared_x_cost_amounts(),
            }),
            handlers: Vec::new(),
            state_fields: Vec::new(),
            has_complex_hook: card.complex_hook.is_some(),
        })
        .collect()
}

#[derive(Clone)]
pub struct CardRegistry {
    cards: HashMap<&'static str, CardDef>,
    /// CardDef indexed by numeric u16 card ID (O(1) lookup).
    id_to_def: Vec<CardDef>,
    /// String card name -> numeric u16 ID.
    name_to_id: HashMap<&'static str, u16>,
    /// Numeric u16 ID -> string card name.
    id_to_name: Vec<&'static str>,
    /// Bitset: true if this card ID is a "Strike" variant (for Perfected Strike).
    strike_flags: Vec<bool>,
}


impl CardRegistry {
    pub fn new() -> Self {
        let mut card_specs: HashMap<&'static str, CardSpec> = HashMap::new();

        watcher::register_watcher(&mut card_specs);
        ironclad::register_ironclad(&mut card_specs);
        silent::register_silent(&mut card_specs);
        defect::register_defect(&mut card_specs);
        colorless::register_colorless(&mut card_specs);
        curses::register_curses(&mut card_specs);
        status::register_status(&mut card_specs);
        temp::register_temp(&mut card_specs);

        // --- Build numeric ID mappings ---
        // Collect all names, sort so base cards come before their "+" upgrades.
        let mut names: Vec<&'static str> = card_specs.keys().copied().collect();
        names.sort_unstable_by(|a, b| {
            let a_base = a.trim_end_matches('+');
            let b_base = b.trim_end_matches('+');
            // Primary: sort by base name alphabetically
            // Secondary: non-upgraded before upgraded (shorter before longer)
            a_base.cmp(b_base).then_with(|| a.len().cmp(&b.len()))
        });

        let count = names.len();
        let mut cards = HashMap::with_capacity(count);
        let mut id_to_def = Vec::with_capacity(count);
        let mut name_to_id = HashMap::with_capacity(count);
        let mut id_to_name = Vec::with_capacity(count);
        let mut strike_flags = Vec::with_capacity(count);

        for (idx, name) in names.iter().enumerate() {
            let id = idx as u16;
            let def: CardDef = card_specs
                .remove(name)
                .unwrap_or_else(|| panic!("missing staged card def for {name}"))
                .into();
            cards.insert(*name, def.clone());
            id_to_def.push(def);
            name_to_id.insert(*name, id);
            id_to_name.push(*name);
            // Case-insensitive check for "strike" substring
            let lower = name.to_ascii_lowercase();
            strike_flags.push(lower.contains("strike"));
        }

        CardRegistry {
            cards,
            id_to_def,
            name_to_id,
            id_to_name,
            strike_flags,
        }
    }



    /// Look up a card by ID. Falls back to a default attack if not found.
    pub fn get(&self, card_id: &str) -> Option<&CardDef> {
        self.cards.get(card_id)
    }

    /// Get card or a sensible default for unknown cards.
    pub fn get_or_default(&self, card_id: &str) -> CardDef {
        if let Some(card) = self.cards.get(card_id) {
            card.clone()
        } else {
            // Unknown card: default to 1-cost attack targeting enemy, 6 damage
            CardDef {
                id: "Unknown",
                name: "Unknown",
                card_type: CardType::Attack,
                target: CardTarget::Enemy,
                cost: 1,
                base_damage: 6,
                base_block: -1,
                base_magic: -1,
                exhaust: false,
                enter_stance: None,
                metadata: CardMetadata::default(),
                effect_data: &[],
                complex_hook: None,
            }
        }
    }

    /// Check if a card ID is a known upgrade ("+" suffix).
    pub fn is_upgraded(card_id: &str) -> bool {
        card_id.ends_with('+')
    }

    /// Get the base ID (strip "+" suffix).
    pub fn base_id(card_id: &str) -> &str {
        card_id.trim_end_matches('+')
    }

    // --- Numeric ID lookup methods ---

    /// Look up the numeric u16 ID for a card name. Returns u16::MAX if not found.
    pub fn card_id(&self, name: &str) -> u16 {
        self.name_to_id.get(name).copied().unwrap_or(u16::MAX)
    }

    /// Look up a CardDef by numeric ID. O(1) array index.
    /// Panics if id is out of range — callers should use IDs from card_id().
    pub fn card_def_by_id(&self, id: u16) -> &CardDef {
        &self.id_to_def[id as usize]
    }

    /// Iterate the static card registry in deterministic numeric-id order.
    pub fn all_card_defs(&self) -> &[CardDef] {
        &self.id_to_def
    }

    /// Get all registered card names in deterministic numeric-id order.
    pub fn all_card_names(&self) -> &[&'static str] {
        &self.id_to_name
    }

    /// Look up a card's string name by numeric ID.
    /// Panics if id is out of range.
    pub fn card_name(&self, id: u16) -> &str {
        self.id_to_name[id as usize]
    }

    /// Total number of registered cards.
    pub fn card_count(&self) -> usize {
        self.id_to_def.len()
    }

    /// Create a CardInstance from a string card name.
    /// Sets def_id to u16::MAX if the name is not found.
    pub fn make_card(&self, name: &str) -> CardInstance {
        let def_id = self.card_id(name);
        let mut card = CardInstance::new(def_id);
        if let Some(def) = self.id_to_def.get(def_id as usize) {
            card.base_cost = def.cost as i8;
        }
        if name.ends_with('+') {
            card.flags |= CardInstance::FLAG_UPGRADED;
        }
        card
    }

    /// Create an upgraded CardInstance from a string card name.
    /// The name should be the base name; this sets the UPGRADED flag.
    /// For pre-registered upgraded defs (e.g. "Strike_P+"), pass the "+" name
    /// and the flag is set automatically.
    pub fn make_card_upgraded(&self, name: &str) -> CardInstance {
        let mut card = self.make_card(name);
        self.upgrade_card(&mut card);
        card
    }

    /// Returns true if the card at this numeric ID is a Strike variant.
    /// Useful for Perfected Strike without runtime string operations.
    /// Returns false for out-of-range IDs.
    pub fn is_strike(&self, id: u16) -> bool {
        self.strike_flags.get(id as usize).copied().unwrap_or(false)
    }

    /// Upgrade a card in-place: change def_id to the upgraded version and set FLAG_UPGRADED.
    pub fn upgrade_card(&self, card: &mut CardInstance) {
        if card.flags & CardInstance::FLAG_UPGRADED != 0 { return; }
        let name = self.card_name(card.def_id);
        let upgraded = format!("{}+", name);
        if let Some(&id) = self.name_to_id.get(upgraded.as_str()) {
            card.def_id = id;
            card.flags |= CardInstance::FLAG_UPGRADED;
            if let Some(def) = self.id_to_def.get(id as usize) {
                card.base_cost = def.cost as i8;
            }
        }
    }
}

impl Default for CardRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_lookup() {
        let reg = super::global_registry();
        let strike = reg.get("Strike_P").unwrap();
        assert_eq!(strike.base_damage, 6);
        assert_eq!(strike.cost, 1);
        assert_eq!(strike.card_type, CardType::Attack);
    }

    #[test]
    fn test_upgraded_lookup() {
        let reg = super::global_registry();
        let strike_plus = reg.get("Strike_P+").unwrap();
        assert_eq!(strike_plus.base_damage, 9);
    }

    #[test]
    fn test_eruption_stance() {
        let reg = super::global_registry();
        let eruption = reg.get("Eruption").unwrap();
        assert_eq!(eruption.enter_stance, Some("Wrath"));
        assert_eq!(eruption.cost, 2);

        let eruption_plus = reg.get("Eruption+").unwrap();
        assert_eq!(eruption_plus.cost, 1); // Upgrade reduces cost
    }

    #[test]
    fn test_unknown_card_default() {
        let reg = super::global_registry();
        let unknown = reg.get_or_default("SomeWeirdCard");
        assert_eq!(unknown.cost, 1);
        assert_eq!(unknown.card_type, CardType::Attack);
    }

    #[test]
    fn test_is_upgraded() {
        assert!(CardRegistry::is_upgraded("Strike_P+"));
        assert!(!CardRegistry::is_upgraded("Strike_P"));
    }

    // -----------------------------------------------------------------------
    // Helper: assert a card exists with expected base + upgraded stats
    // -----------------------------------------------------------------------
    fn assert_card(reg: &CardRegistry, id: &str, cost: i32, dmg: i32, blk: i32, mag: i32, ct: CardType) {
        let card = reg.get(id).unwrap_or_else(|| panic!("Card '{}' not found in registry", id));
        assert_eq!(card.cost, cost, "{} cost", id);
        assert_eq!(card.base_damage, dmg, "{} damage", id);
        assert_eq!(card.base_block, blk, "{} block", id);
        assert_eq!(card.base_magic, mag, "{} magic", id);
        assert_eq!(card.card_type, ct, "{} type", id);
    }

    fn assert_has_effect(reg: &CardRegistry, id: &str, effect: &str) {
        let card = reg.get(id).unwrap_or_else(|| panic!("Card '{}' not found", id));
        assert!(card.has_test_marker(effect), "{} should have effect '{}'", id, effect);
    }

    // -----------------------------------------------------------------------
    // All cards in reward pools must be registered (no fallback to Unknown)
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_pool_cards_registered() {
        let reg = super::global_registry();
        let pool_cards = [
            // Common
            "BowlingBash", "Consecrate", "Crescendo", "CrushJoints",
            "CutThroughFate", "EmptyBody", "EmptyFist", "Evaluate",
            "FlurryOfBlows", "FlyingSleeves", "FollowUp", "Halt",
            "JustLucky", "PathToVictory", "Prostrate",
            "Protect", "SashWhip", "ClearTheMind",
            // Uncommon
            "BattleHymn", "CarveReality", "Conclude", "DeceiveReality",
            "EmptyMind", "FearNoEvil", "ForeignInfluence", "Indignation",
            "InnerPeace", "LikeWater", "Meditate", "Nirvana",
            "Perseverance", "ReachHeaven", "SandsOfTime", "SignatureMove",
            "Smite", "Study", "Swivel", "TalkToTheHand",
            "Tantrum", "ThirdEye", "Wallop", "WaveOfTheHand",
            "Weave", "WheelKick", "WindmillStrike", "WreathOfFlame",
            // Rare
            "Alpha", "Blasphemy", "Brilliance", "ConjureBlade",
            "DevaForm", "Devotion", "Establishment", "Fasting2",
            "Judgement", "LessonLearned", "MasterReality",
            "MentalFortress", "Omniscience", "Ragnarok",
            "Adaptation", "Scrawl", "SpiritShield", "Vault", "Wish",
        ];
        for id in &pool_cards {
            assert!(reg.get(id).is_some(), "Card '{}' missing from registry", id);
            let upgraded = format!("{}+", id);
            assert!(reg.get(&upgraded).is_some(), "Card '{}' missing from registry", upgraded);
        }
    }

    // -----------------------------------------------------------------------
    // Common card stats (base + upgraded)
    // -----------------------------------------------------------------------
    #[test]
    fn test_consecrate_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Consecrate", 0, 5, -1, -1, CardType::Attack);
        assert_card(&reg, "Consecrate+", 0, 8, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_crescendo_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Crescendo", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Crescendo+", 0, -1, -1, -1, CardType::Skill);
        assert!(reg.get("Crescendo").unwrap().exhaust);
        assert_has_effect(&reg, "Crescendo", "retain");
        assert_eq!(reg.get("Crescendo").unwrap().enter_stance, Some("Wrath"));
    }

    #[test]
    fn test_empty_fist_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "EmptyFist", 1, 9, -1, -1, CardType::Attack);
        assert_card(&reg, "EmptyFist+", 1, 14, -1, -1, CardType::Attack);
        assert_eq!(reg.get("EmptyFist").unwrap().enter_stance, Some("Neutral"));
    }

    #[test]
    fn test_evaluate_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Evaluate", 1, -1, 6, -1, CardType::Skill);
        assert_card(&reg, "Evaluate+", 1, -1, 10, -1, CardType::Skill);
    }

    #[test]
    fn test_just_lucky_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "JustLucky", 0, 3, 2, 1, CardType::Attack);
        assert_card(&reg, "JustLucky+", 0, 4, 3, 2, CardType::Attack);
    }

    #[test]
    fn test_pressure_points_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "PathToVictory", 1, -1, -1, 8, CardType::Skill);
        assert_card(&reg, "PathToVictory+", 1, -1, -1, 11, CardType::Skill);
    }

    #[test]
    fn test_protect_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Protect", 2, -1, 12, -1, CardType::Skill);
        assert_card(&reg, "Protect+", 2, -1, 16, -1, CardType::Skill);
        assert_has_effect(&reg, "Protect", "retain");
    }

    #[test]
    fn test_sash_whip_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "SashWhip", 1, 8, -1, 1, CardType::Attack);
        assert_card(&reg, "SashWhip+", 1, 10, -1, 2, CardType::Attack);
    }

    #[test]
    fn test_tranquility_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "ClearTheMind", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "ClearTheMind+", 0, -1, -1, -1, CardType::Skill);
        assert!(reg.get("ClearTheMind").unwrap().exhaust);
        assert_eq!(reg.get("ClearTheMind").unwrap().enter_stance, Some("Calm"));
    }

    // -----------------------------------------------------------------------
    // Uncommon card stats (base + upgraded)
    // -----------------------------------------------------------------------
    #[test]
    fn test_battle_hymn_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "BattleHymn", 1, -1, -1, 1, CardType::Power);
        assert_card(&reg, "BattleHymn+", 1, -1, -1, 1, CardType::Power);
        assert_has_effect(&reg, "BattleHymn+", "innate");
    }

    #[test]
    fn test_carve_reality_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "CarveReality", 1, 6, -1, -1, CardType::Attack);
        assert_card(&reg, "CarveReality+", 1, 10, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_deceive_reality_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "DeceiveReality", 1, -1, 4, -1, CardType::Skill);
        assert_card(&reg, "DeceiveReality+", 1, -1, 7, -1, CardType::Skill);
    }

    #[test]
    fn test_empty_mind_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "EmptyMind", 1, -1, -1, 2, CardType::Skill);
        assert_card(&reg, "EmptyMind+", 1, -1, -1, 3, CardType::Skill);
        assert_eq!(reg.get("EmptyMind").unwrap().enter_stance, Some("Neutral"));
    }

    #[test]
    fn test_fear_no_evil_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "FearNoEvil", 1, 8, -1, -1, CardType::Attack);
        assert_card(&reg, "FearNoEvil+", 1, 11, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_foreign_influence_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "ForeignInfluence", 0, -1, -1, -1, CardType::Skill);
        assert!(reg.get("ForeignInfluence").unwrap().exhaust);
    }

    #[test]
    fn test_indignation_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Indignation", 1, -1, -1, 3, CardType::Skill);
        assert_card(&reg, "Indignation+", 1, -1, -1, 5, CardType::Skill);
    }

    #[test]
    fn test_like_water_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "LikeWater", 1, -1, -1, 5, CardType::Power);
        assert_card(&reg, "LikeWater+", 1, -1, -1, 7, CardType::Power);
    }

    #[test]
    fn test_meditate_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Meditate", 1, -1, -1, 1, CardType::Skill);
        assert_card(&reg, "Meditate+", 1, -1, -1, 2, CardType::Skill);
        assert_eq!(reg.get("Meditate").unwrap().enter_stance, Some("Calm"));
    }

    #[test]
    fn test_nirvana_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Nirvana", 1, -1, -1, 3, CardType::Power);
        assert_card(&reg, "Nirvana+", 1, -1, -1, 4, CardType::Power);
    }

    #[test]
    fn test_perseverance_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Perseverance", 1, -1, 5, 2, CardType::Skill);
        assert_card(&reg, "Perseverance+", 1, -1, 7, 3, CardType::Skill);
        assert_has_effect(&reg, "Perseverance", "retain");
    }

    #[test]
    fn test_reach_heaven_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "ReachHeaven", 2, 10, -1, -1, CardType::Attack);
        assert_card(&reg, "ReachHeaven+", 2, 15, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_sands_of_time_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "SandsOfTime", 4, 20, -1, -1, CardType::Attack);
        assert_card(&reg, "SandsOfTime+", 4, 26, -1, -1, CardType::Attack);
        assert_has_effect(&reg, "SandsOfTime", "retain");
    }

    #[test]
    fn test_signature_move_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "SignatureMove", 2, 30, -1, -1, CardType::Attack);
        assert_card(&reg, "SignatureMove+", 2, 40, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_study_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Study", 2, -1, -1, 1, CardType::Power);
        assert_card(&reg, "Study+", 1, -1, -1, 1, CardType::Power);
    }

    #[test]
    fn test_swivel_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Swivel", 2, -1, 8, -1, CardType::Skill);
        assert_card(&reg, "Swivel+", 2, -1, 11, -1, CardType::Skill);
    }

    #[test]
    fn test_wallop_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Wallop", 2, 9, -1, -1, CardType::Attack);
        assert_card(&reg, "Wallop+", 2, 12, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_wave_of_the_hand_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "WaveOfTheHand", 1, -1, -1, 1, CardType::Skill);
        assert_card(&reg, "WaveOfTheHand+", 1, -1, -1, 2, CardType::Skill);
    }

    #[test]
    fn test_weave_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Weave", 0, 4, -1, -1, CardType::Attack);
        assert_card(&reg, "Weave+", 0, 6, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_windmill_strike_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "WindmillStrike", 2, 7, -1, 4, CardType::Attack);
        assert_card(&reg, "WindmillStrike+", 2, 10, -1, 5, CardType::Attack);
        assert_has_effect(&reg, "WindmillStrike", "retain");
    }

    #[test]
    fn test_wreath_of_flame_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "WreathOfFlame", 1, -1, -1, 5, CardType::Skill);
        assert_card(&reg, "WreathOfFlame+", 1, -1, -1, 8, CardType::Skill);
    }

    // -----------------------------------------------------------------------
    // Rare card stats (base + upgraded)
    // -----------------------------------------------------------------------
    #[test]
    fn test_alpha_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Alpha", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Alpha+", 1, -1, -1, -1, CardType::Skill);
        assert!(reg.get("Alpha").unwrap().exhaust);
        assert_has_effect(&reg, "Alpha+", "innate");
    }

    #[test]
    fn test_blasphemy_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Blasphemy", 1, -1, -1, -1, CardType::Skill);
        assert!(reg.get("Blasphemy").unwrap().exhaust);
        assert_eq!(reg.get("Blasphemy").unwrap().enter_stance, Some("Divinity"));
        assert_has_effect(&reg, "Blasphemy+", "retain");
    }

    #[test]
    fn test_brilliance_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Brilliance", 1, 12, -1, 0, CardType::Attack);
        assert_card(&reg, "Brilliance+", 1, 16, -1, 0, CardType::Attack);
    }

    #[test]
    fn test_conjure_blade_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "ConjureBlade", -1, -1, -1, -1, CardType::Skill);
        assert!(reg.get("ConjureBlade").unwrap().exhaust);
    }

    #[test]
    fn test_deva_form_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "DevaForm", 3, -1, -1, 1, CardType::Power);
        assert_card(&reg, "DevaForm+", 3, -1, -1, 1, CardType::Power);
        assert_has_effect(&reg, "DevaForm", "ethereal");
        assert!(!reg.get("DevaForm+").unwrap().has_test_marker("ethereal"));
    }

    #[test]
    fn test_devotion_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Devotion", 1, -1, -1, 2, CardType::Power);
        assert_card(&reg, "Devotion+", 1, -1, -1, 3, CardType::Power);
    }

    #[test]
    fn test_establishment_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Establishment", 1, -1, -1, 1, CardType::Power);
        assert_has_effect(&reg, "Establishment+", "innate");
    }

    #[test]
    fn test_fasting_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Fasting2", 2, -1, -1, 3, CardType::Power);
        assert_card(&reg, "Fasting2+", 2, -1, -1, 4, CardType::Power);
    }

    #[test]
    fn test_judgement_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Judgement", 1, -1, -1, 30, CardType::Skill);
        assert_card(&reg, "Judgement+", 1, -1, -1, 40, CardType::Skill);
    }

    #[test]
    fn test_lesson_learned_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "LessonLearned", 2, 10, -1, -1, CardType::Attack);
        assert_card(&reg, "LessonLearned+", 2, 13, -1, -1, CardType::Attack);
        assert!(reg.get("LessonLearned").unwrap().exhaust);
    }

    #[test]
    fn test_master_reality_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "MasterReality", 1, -1, -1, -1, CardType::Power);
        assert_card(&reg, "MasterReality+", 0, -1, -1, -1, CardType::Power);
    }

    #[test]
    fn test_omniscience_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Omniscience", 4, -1, -1, 2, CardType::Skill);
        assert_card(&reg, "Omniscience+", 3, -1, -1, 2, CardType::Skill);
        assert!(reg.get("Omniscience").unwrap().exhaust);
    }

    #[test]
    fn test_scrawl_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Scrawl", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Scrawl+", 0, -1, -1, -1, CardType::Skill);
        assert!(reg.get("Scrawl").unwrap().exhaust);
    }

    #[test]
    fn test_spirit_shield_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "SpiritShield", 2, -1, -1, 3, CardType::Skill);
        assert_card(&reg, "SpiritShield+", 2, -1, -1, 4, CardType::Skill);
    }

    #[test]
    fn test_vault_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Vault", 3, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Vault+", 2, -1, -1, -1, CardType::Skill);
        assert!(reg.get("Vault").unwrap().exhaust);
    }

    #[test]
    fn test_wish_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Wish", 3, 3, 6, 25, CardType::Skill);
        assert_card(&reg, "Wish+", 3, 4, 8, 30, CardType::Skill);
        assert!(reg.get("Wish").unwrap().exhaust);
    }

    // -----------------------------------------------------------------------
    // Bug fixes: Tantrum shuffle + Smite exhaust
    // -----------------------------------------------------------------------
    #[test]
    fn test_tantrum_shuffle_into_draw() {
        let reg = super::global_registry();
        assert_has_effect(&reg, "Tantrum", "shuffle_self_into_draw");
        assert_has_effect(&reg, "Tantrum+", "shuffle_self_into_draw");
    }

    #[test]
    fn test_smite_exhaust() {
        let reg = super::global_registry();
        assert!(reg.get("Smite").unwrap().exhaust, "Smite should exhaust");
        assert!(reg.get("Smite+").unwrap().exhaust, "Smite+ should exhaust");
        assert_has_effect(&reg, "Smite", "retain");
    }

    // -----------------------------------------------------------------------
    // All Ironclad cards in reward pools must be registered
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_ironclad_cards_registered() {
        let reg = super::global_registry();
        let ironclad_cards = [
            // Basic
            "Strike_R", "Defend_R", "Bash",
            // Common
            "Anger", "Armaments", "Body Slam", "Clash", "Cleave",
            "Clothesline", "Flex", "Havoc", "Headbutt", "Heavy Blade",
            "Iron Wave", "Perfected Strike", "Pommel Strike", "Shrug It Off",
            "Sword Boomerang", "Thunderclap", "True Grit", "Twin Strike",
            "Warcry", "Wild Strike",
            // Uncommon
            "Battle Trance", "Blood for Blood", "Bloodletting", "Burning Pact",
            "Carnage", "Combust", "Dark Embrace", "Disarm", "Dropkick",
            "Dual Wield", "Entrench", "Evolve", "Feel No Pain", "Fire Breathing",
            "Flame Barrier", "Ghostly Armor", "Hemokinesis", "Infernal Blade",
            "Inflame", "Intimidate", "Metallicize", "Power Through", "Pummel",
            "Rage", "Rampage", "Reckless Charge", "Rupture", "Searing Blow",
            "Second Wind", "Seeing Red", "Sentinel", "Sever Soul", "Shockwave",
            "Spot Weakness", "Uppercut", "Whirlwind",
            // Rare
            "Barricade", "Berserk", "Bludgeon", "Brutality", "Corruption",
            "Demon Form", "Double Tap", "Exhume", "Feed", "Fiend Fire",
            "Immolate", "Impervious", "Juggernaut", "Limit Break", "Offering",
            "Reaper",
        ];
        for id in &ironclad_cards {
            assert!(reg.get(id).is_some(), "Ironclad card '{}' missing from registry", id);
            let upgraded = format!("{}+", id);
            assert!(reg.get(&upgraded).is_some(), "Ironclad card '{}' missing from registry", upgraded);
        }
        // Verify count: 3 basic + 20 common + 36 uncommon + 16 rare = 75
        assert_eq!(ironclad_cards.len(), 75, "Should have exactly 75 Ironclad cards");
    }

    // -----------------------------------------------------------------------
    // All Silent cards in reward pools must be registered
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_silent_cards_registered() {
        let reg = super::global_registry();
        let silent_cards = [
            // Basic
            "Strike_G", "Defend_G", "Neutralize", "Survivor",
            // Common
            "Acrobatics", "Backflip", "Bane", "Blade Dance", "Cloak and Dagger",
            "Dagger Spray", "Dagger Throw", "Deadly Poison", "Deflect",
            "Dodge and Roll", "Flying Knee", "Outmaneuver", "Piercing Wail",
            "Poisoned Stab", "Prepared", "Quick Slash", "Slice",
            "Sneaky Strike", "Sucker Punch",
            // Uncommon
            "Accuracy", "All-Out Attack", "Backstab", "Blur", "Bouncing Flask",
            "Calculated Gamble", "Caltrops", "Catalyst", "Choke", "Concentrate",
            "Crippling Cloud", "Dash", "Distraction", "Endless Agony", "Envenom",
            "Escape Plan", "Eviscerate", "Expertise", "Finisher", "Flechettes",
            "Footwork", "Heel Hook", "Infinite Blades", "Leg Sweep",
            "Masterful Stab", "Noxious Fumes", "Predator", "Reflex",
            "Riddle with Holes", "Setup", "Skewer", "Tactician", "Terror",
            "Well-Laid Plans",
            // Rare
            "A Thousand Cuts", "Adrenaline", "After Image", "Alchemize",
            "Bullet Time", "Burst", "Corpse Explosion", "Die Die Die",
            "Doppelganger", "Glass Knife", "Grand Finale", "Malaise",
            "Nightmare", "Phantasmal Killer", "Storm of Steel",
            "Tools of the Trade", "Unload", "Wraith Form",
        ];
        for id in &silent_cards {
            assert!(reg.get(id).is_some(), "Silent card '{}' missing from registry", id);
            let upgraded = format!("{}+", id);
            assert!(reg.get(&upgraded).is_some(), "Silent card '{}' missing from registry", upgraded);
        }
        // Verify count: 4 basic + 19 common + 34 uncommon + 18 rare = 75
        assert_eq!(silent_cards.len(), 75, "Should have exactly 75 Silent cards");
    }

    // -----------------------------------------------------------------------
    // Spot-check Ironclad card stats
    // -----------------------------------------------------------------------
    #[test]
    fn test_bash_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Bash", 2, 8, -1, 2, CardType::Attack);
        assert_card(&reg, "Bash+", 2, 10, -1, 3, CardType::Attack);
        assert_has_effect(&reg, "Bash", "vulnerable");
    }

    #[test]
    fn test_impervious_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Impervious", 2, -1, 30, -1, CardType::Skill);
        assert_card(&reg, "Impervious+", 2, -1, 40, -1, CardType::Skill);
        assert!(reg.get("Impervious").unwrap().exhaust);
    }

    #[test]
    fn test_corruption_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Corruption", 3, -1, -1, -1, CardType::Power);
        assert_card(&reg, "Corruption+", 2, -1, -1, -1, CardType::Power);
        assert_eq!(
            reg.get("Corruption").unwrap().effect_data,
            &[crate::effects::declarative::Effect::Simple(
                crate::effects::declarative::SimpleEffect::SetStatus(
                    crate::effects::declarative::Target::Player,
                    crate::status_ids::sid::CORRUPTION,
                    crate::effects::declarative::AmountSource::Fixed(1),
                ),
            )]
        );
    }

    // -----------------------------------------------------------------------
    // Spot-check Silent card stats
    // -----------------------------------------------------------------------
    #[test]
    fn test_neutralize_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Neutralize", 0, 3, -1, 1, CardType::Attack);
        assert_card(&reg, "Neutralize+", 0, 4, -1, 2, CardType::Attack);
        assert_has_effect(&reg, "Neutralize", "weak");
    }

    #[test]
    fn test_wraith_form_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Wraith Form", 3, -1, -1, 2, CardType::Power);
        assert_card(&reg, "Wraith Form+", 3, -1, -1, 3, CardType::Power);
        // Wraith Form now uses complex_hook, no effect tags
    }

    #[test]
    fn test_deadly_poison_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Deadly Poison", 1, -1, -1, 5, CardType::Skill);
        assert_card(&reg, "Deadly Poison+", 1, -1, -1, 7, CardType::Skill);
        assert_has_effect(&reg, "Deadly Poison", "poison");
    }

    // Defect card registration tests
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_defect_cards_registered() {
        let reg = super::global_registry();
        let defect_cards = [
            // Basic
            "Strike_B", "Defend_B", "Zap", "Dualcast",
            // Common
            "Ball Lightning", "Barrage", "Beam Cell", "Cold Snap",
            "Compile Driver", "Conserve Battery", "Coolheaded",
            "Go for the Eyes", "Hologram", "Leap", "Rebound",
            "Stack", "Steam", "Streamline", "Sweeping Beam", "Turbo", "Gash",
            // Uncommon
            "Aggregate", "Auto Shields", "Blizzard", "BootSequence",
            "Capacitor", "Chaos", "Chill", "Consume", "Darkness",
            "Defragment", "Doom and Gloom", "Double Energy", "Undo",
            "Force Field", "FTL", "Fusion", "Genetic Algorithm", "Glacier",
            "Heatsinks", "Hello World", "Lockon", "Loop",
            "Melter", "Steam Power", "Recycle", "Redo",
            "Reinforced Body", "Reprogram", "Rip and Tear", "Scrape",
            "Self Repair", "Skim", "Static Discharge", "Storm",
            "Sunder", "Tempest", "White Noise",
            // Rare
            "All For One", "Amplify", "Biased Cognition", "Buffer",
            "Core Surge", "Creative AI", "Echo Form", "Electrodynamics",
            "Fission", "Hyperbeam", "Machine Learning", "Meteor Strike",
            "Multi-Cast", "Rainbow", "Reboot", "Seek", "Thunder Strike",
        ];
        for id in &defect_cards {
            assert!(reg.get(id).is_some(), "Defect card '{}' missing", id);
            let upgraded = format!("{}+", id);
            assert!(reg.get(&upgraded).is_some(), "Defect card '{}' missing", upgraded);
        }
    }

    #[test]
    fn test_defect_orb_effects() {
        let reg = super::global_registry();
        assert_has_effect(&reg, "Zap", "channel_lightning");
        assert_has_effect(&reg, "Ball Lightning", "channel_lightning");
        assert_has_effect(&reg, "Cold Snap", "channel_frost");
        assert_has_effect(&reg, "Coolheaded", "channel_frost");
        assert_has_effect(&reg, "Darkness", "channel_dark");
        assert_has_effect(&reg, "Fusion", "channel_plasma");
        assert_has_effect(&reg, "Dualcast", "evoke_orb");
        assert_has_effect(&reg, "Defragment", "gain_focus");
    }

    #[test]
    fn test_defect_card_stats() {
        let reg = super::global_registry();
        // Basic
        assert_card(&reg, "Strike_B", 1, 6, -1, -1, CardType::Attack);
        assert_card(&reg, "Strike_B+", 1, 9, -1, -1, CardType::Attack);
        assert_card(&reg, "Defend_B", 1, -1, 5, -1, CardType::Skill);
        assert_card(&reg, "Defend_B+", 1, -1, 8, -1, CardType::Skill);
        assert_card(&reg, "Zap", 1, -1, -1, 1, CardType::Skill);
        assert_card(&reg, "Zap+", 0, -1, -1, 1, CardType::Skill);
        assert_card(&reg, "Dualcast", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Dualcast+", 0, -1, -1, -1, CardType::Skill);
        // Key uncommon/rare
        assert_card(&reg, "Glacier", 2, -1, 7, 2, CardType::Skill);
        assert_card(&reg, "Glacier+", 2, -1, 10, 2, CardType::Skill);
        assert_card(&reg, "Hyperbeam", 2, 26, -1, 3, CardType::Attack);
        assert_card(&reg, "Hyperbeam+", 2, 34, -1, 3, CardType::Attack);
        assert_card(&reg, "Echo Form", 3, -1, -1, -1, CardType::Power);
        assert_has_effect(&reg, "Echo Form", "ethereal");
        assert!(!reg.get("Echo Form+").unwrap().has_test_marker("ethereal"));
        assert_card(&reg, "Meteor Strike", 5, 24, -1, 3, CardType::Attack);
        assert_card(&reg, "Biased Cognition", 1, -1, -1, 4, CardType::Power);
        assert_card(&reg, "Biased Cognition+", 1, -1, -1, 5, CardType::Power);
    }

    // -----------------------------------------------------------------------
    // Colorless card registration tests
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_colorless_cards_registered() {
        let reg = super::global_registry();
        let colorless_cards = [
            // Uncommon
            "Bandage Up", "Blind", "Dark Shackles", "Deep Breath",
            "Discovery", "Dramatic Entrance", "Enlightenment", "Finesse",
            "Flash of Steel", "Forethought", "Good Instincts", "Impatience",
            "Jack Of All Trades", "Madness", "Mind Blast", "Panacea",
            "PanicButton", "Purity", "Swift Strike", "Trip",
            // Rare
            "Apotheosis", "Chrysalis", "HandOfGreed", "Magnetism",
            "Master of Strategy", "Mayhem", "Metamorphosis", "Panache",
            "Sadistic Nature", "Secret Technique", "Secret Weapon",
            "The Bomb", "Thinking Ahead", "Transmutation", "Violence",
            // Special
            "Ghostly", "Bite", "J.A.X.", "RitualDagger",
        ];
        for id in &colorless_cards {
            assert!(reg.get(id).is_some(), "Colorless card '{}' missing", id);
            let upgraded = format!("{}+", id);
            assert!(reg.get(&upgraded).is_some(), "Colorless card '{}' missing", upgraded);
        }
    }

    #[test]
    fn test_colorless_card_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Apotheosis", 2, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Apotheosis+", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "HandOfGreed", 2, 20, -1, 20, CardType::Attack);
        assert_card(&reg, "HandOfGreed+", 2, 25, -1, 25, CardType::Attack);
        assert_card(&reg, "Swift Strike", 0, 7, -1, -1, CardType::Attack);
        assert_card(&reg, "Ghostly", 1, -1, -1, -1, CardType::Skill);
        assert_has_effect(&reg, "Ghostly", "ethereal");
        assert!(!reg.get("Ghostly+").unwrap().has_test_marker("ethereal"));
        assert_card(&reg, "Panache", 0, -1, -1, 10, CardType::Power);
        assert_card(&reg, "Panache+", 0, -1, -1, 14, CardType::Power);
    }

    // -----------------------------------------------------------------------
    // Curse card registration tests
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_curse_cards_registered() {
        let reg = super::global_registry();
        let curse_cards = [
            "AscendersBane", "Clumsy", "CurseOfTheBell", "Decay",
            "Doubt", "Injury", "Necronomicurse", "Normality",
            "Pain", "Parasite", "Pride", "Regret", "Shame", "Writhe",
        ];
        for id in &curse_cards {
            let card = reg.get(id).unwrap_or_else(|| panic!("Curse '{}' missing", id));
            assert_eq!(card.card_type, CardType::Curse, "{} should be Curse type", id);
            assert!(card.has_test_marker("unplayable") || card.cost >= 0,
                "{} should be unplayable or have a cost", id);
        }
    }

    #[test]
    fn test_curse_effects() {
        let reg = super::global_registry();
        assert_has_effect(&reg, "Decay", "end_turn_damage");
        assert_has_effect(&reg, "Doubt", "end_turn_weak");
        assert_has_effect(&reg, "Shame", "end_turn_frail");
        assert_has_effect(&reg, "Normality", "limit_cards_per_turn");
        assert_has_effect(&reg, "Writhe", "innate");
        assert_has_effect(&reg, "Clumsy", "ethereal");
        assert_has_effect(&reg, "Necronomicurse", "unremovable");
    }

    // -----------------------------------------------------------------------
    // Status card registration tests
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_status_cards_registered() {
        let reg = super::global_registry();
        let status_cards = ["Slimed", "Wound", "Daze", "Burn", "Void"];
        for id in &status_cards {
            let card = reg.get(id).unwrap_or_else(|| panic!("Status '{}' missing", id));
            assert_eq!(card.card_type, CardType::Status, "{} should be Status type", id);
        }
    }

    #[test]
    fn test_status_effects() {
        let reg = super::global_registry();
        assert_has_effect(&reg, "Burn", "end_turn_damage");
        assert_eq!(reg.get("Burn").unwrap().base_magic, 2);
        assert_has_effect(&reg, "Burn+", "end_turn_damage");
        assert_eq!(reg.get("Burn+").unwrap().base_magic, 4);
        assert_has_effect(&reg, "Void", "lose_energy_on_draw");
        assert_has_effect(&reg, "Void", "ethereal");
        assert_has_effect(&reg, "Daze", "ethereal");
    }

    // -----------------------------------------------------------------------
    // Temp card registration tests
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_temp_cards_registered() {
        let reg = super::global_registry();
        let temp_cards = [
            "Miracle", "Smite", "Beta", "Omega", "Expunger",
            "Insight", "Safety", "ThroughViolence", "Shiv",
        ];
        for id in &temp_cards {
            assert!(reg.get(id).is_some(), "Temp card '{}' missing", id);
            let upgraded = format!("{}+", id);
            assert!(reg.get(&upgraded).is_some(), "Temp card '{}' missing", upgraded);
        }
    }

    #[test]
    fn test_temp_card_stats() {
        let reg = super::global_registry();
        assert_card(&reg, "Beta", 2, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Beta+", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Omega", 3, -1, -1, 50, CardType::Power);
        assert_card(&reg, "Omega+", 3, -1, -1, 60, CardType::Power);
        assert_card(&reg, "Shiv", 0, 4, -1, -1, CardType::Attack);
        assert_card(&reg, "Shiv+", 0, 6, -1, -1, CardType::Attack);
        assert!(reg.get("Shiv").unwrap().exhaust);
        assert_card(&reg, "Safety", 1, -1, 12, -1, CardType::Skill);
        assert_card(&reg, "Safety+", 1, -1, 16, -1, CardType::Skill);
        assert_has_effect(&reg, "Safety", "retain");
        assert_card(&reg, "ThroughViolence", 0, 20, -1, -1, CardType::Attack);
        assert_card(&reg, "ThroughViolence+", 0, 30, -1, -1, CardType::Attack);
        assert_has_effect(&reg, "ThroughViolence", "retain");
    }

    // -----------------------------------------------------------------------
    // Numeric card ID lookup tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_card_id_roundtrip() {
        let reg = super::global_registry();
        let id = reg.card_id("Strike_P");
        assert_ne!(id, u16::MAX, "Strike_P should have a valid ID");
        assert_eq!(reg.card_name(id), "Strike_P");
        assert_eq!(reg.card_def_by_id(id).base_damage, 6);
    }

    #[test]
    fn test_card_id_unknown_returns_max() {
        let reg = super::global_registry();
        assert_eq!(reg.card_id("TotallyFakeCard"), u16::MAX);
    }

    #[test]
    fn test_card_count_matches_hashmap() {
        let reg = super::global_registry();
        assert_eq!(reg.card_count(), reg.cards.len());
        assert!(reg.card_count() > 700, "Should have 700+ cards registered");
    }

    #[test]
    fn test_base_and_upgraded_consecutive_ids() {
        let reg = super::global_registry();
        let base_id = reg.card_id("Strike_P");
        let upgraded_id = reg.card_id("Strike_P+");
        assert_ne!(base_id, u16::MAX);
        assert_ne!(upgraded_id, u16::MAX);
        // Sorting puts base before upgraded, so upgraded = base + 1
        assert_eq!(upgraded_id, base_id + 1,
            "Strike_P+ should be consecutive after Strike_P");
    }

    #[test]
    fn test_all_ids_have_matching_defs() {
        let reg = super::global_registry();
        for id in 0..reg.card_count() as u16 {
            let name = reg.card_name(id);
            let def = reg.card_def_by_id(id);
            assert_eq!(def.id, name, "ID {} name mismatch", id);
            assert_eq!(reg.card_id(name), id, "Reverse lookup for '{}' failed", name);
        }
    }

    #[test]
    fn test_is_strike() {
        let reg = super::global_registry();
        assert!(reg.is_strike(reg.card_id("Strike_P")));
        assert!(reg.is_strike(reg.card_id("Strike_P+")));
        assert!(reg.is_strike(reg.card_id("Strike_R")));
        assert!(reg.is_strike(reg.card_id("Perfected Strike")));
        assert!(reg.is_strike(reg.card_id("Perfected Strike+")));
        assert!(reg.is_strike(reg.card_id("WindmillStrike")));
        assert!(reg.is_strike(reg.card_id("Swift Strike")));
        // Non-strikes
        assert!(!reg.is_strike(reg.card_id("Defend_P")));
        assert!(!reg.is_strike(reg.card_id("Eruption")));
        assert!(!reg.is_strike(reg.card_id("Bash")));
        // Out-of-range
        assert!(!reg.is_strike(u16::MAX));
    }

    #[test]
    fn test_make_card() {
        let reg = super::global_registry();
        let card = reg.make_card("Eruption");
        assert_eq!(card.def_id, reg.card_id("Eruption"));
        assert!(!card.is_upgraded());
    }

    #[test]
    fn test_make_card_upgraded() {
        let reg = super::global_registry();
        let card = reg.make_card_upgraded("Eruption+");
        assert_eq!(card.def_id, reg.card_id("Eruption+"));
        assert!(card.is_upgraded());
    }

    #[test]
    fn test_card_def_by_id_matches_get() {
        let reg = super::global_registry();
        // Every card accessible via get() should match card_def_by_id()
        for name in ["Strike_P", "Eruption", "Bash", "Neutralize", "Zap", "Apotheosis"] {
            let by_name = reg.get(name).unwrap();
            let id = reg.card_id(name);
            let by_id = reg.card_def_by_id(id);
            assert_eq!(by_name.id, by_id.id);
            assert_eq!(by_name.cost, by_id.cost);
            assert_eq!(by_name.base_damage, by_id.base_damage);
            assert_eq!(by_name.base_block, by_id.base_block);
        }
    }

    #[test]
    fn gameplay_lookup_uses_canonical_registry() {
        let def = super::gameplay_def("Strike_P").expect("card gameplay def");
        assert_eq!(def.domain, crate::gameplay::GameplayDomain::Card);
        assert_eq!(def.id, "Strike_P");
        assert!(def.card_schema().is_some());
    }

    #[test]
    fn gameplay_exports_cover_registry_cards() {
        let exports = super::gameplay_export_defs();
        assert_eq!(exports.len(), super::global_registry().card_count());
        assert!(exports.iter().any(|def| def.id == "Strike_P"));
        assert!(exports.iter().all(|def| def.domain == crate::gameplay::GameplayDomain::Card));
    }
}

#[cfg(test)]
#[path = "../tests/test_card_runtime_backend_wave3.rs"]
mod test_card_runtime_backend_wave3;
