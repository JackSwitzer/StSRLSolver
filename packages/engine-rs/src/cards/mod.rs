//! Card data and effects — minimal card registry for the core turn loop.
//!
//! Only implements cards needed for the fast MCTS path. The Python engine
//! handles the full ~350 card catalog with all edge cases.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::combat_types::CardInstance;
use crate::effects::declarative::{
    AmountSource, ChoiceAction, Effect, Pile, SimpleEffect, Target,
};
use crate::orbs::OrbType;
use crate::state::Stance;

mod prelude;
mod watcher;
mod ironclad;
mod silent;
pub(crate) mod defect;
mod colorless;
mod curses;
mod status;
mod temp;

/// Insert a card definition into the registry map.
pub(crate) fn insert(map: &mut HashMap<&'static str, CardDef>, card: CardDef) {
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
    /// Special effect tags for the engine to check
    pub effects: &'static [&'static str],
    /// Declarative effect data for the new interpreter (parallel to string tags during migration).
    /// Empty slice = use old card_effects.rs dispatch.
    #[serde(skip)]
    pub effect_data: &'static [crate::effects::declarative::Effect],
    /// Complex on-play hook for irreducible effects (Pressure Points, Judgement, etc.).
    /// None = no complex hook.
    #[serde(skip)]
    pub complex_hook: Option<crate::effects::registry::OnPlayFn>,
}

impl CardDef {
    /// Is this card an unplayable status/curse?
    pub fn is_unplayable(&self) -> bool {
        self.cost == -2
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
            Effect::Simple(SimpleEffect::DealDamage(target, AmountSource::Damage))
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
            tags: card.effects.iter().map(|tag| (*tag).to_string()).collect(),
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
    /// Precomputed effect flags per card ID for O(1) hook dispatch.
    effect_flags_vec: Vec<crate::effects::EffectFlags>,
}


impl CardRegistry {
    pub fn new() -> Self {
        let mut cards = HashMap::new();

        watcher::register_watcher(&mut cards);
        ironclad::register_ironclad(&mut cards);
        silent::register_silent(&mut cards);
        defect::register_defect(&mut cards);
        colorless::register_colorless(&mut cards);
        curses::register_curses(&mut cards);
        status::register_status(&mut cards);
        temp::register_temp(&mut cards);

        // --- Build numeric ID mappings ---
        // Collect all names, sort so base cards come before their "+" upgrades.
        let mut names: Vec<&'static str> = cards.keys().copied().collect();
        names.sort_unstable_by(|a, b| {
            let a_base = a.trim_end_matches('+');
            let b_base = b.trim_end_matches('+');
            // Primary: sort by base name alphabetically
            // Secondary: non-upgraded before upgraded (shorter before longer)
            a_base.cmp(b_base).then_with(|| a.len().cmp(&b.len()))
        });

        let count = names.len();
        let mut id_to_def = Vec::with_capacity(count);
        let mut name_to_id = HashMap::with_capacity(count);
        let mut id_to_name = Vec::with_capacity(count);
        let mut strike_flags = Vec::with_capacity(count);

        for (idx, name) in names.iter().enumerate() {
            let id = idx as u16;
            let def = cards[name].clone();
            id_to_def.push(def);
            name_to_id.insert(*name, id);
            id_to_name.push(*name);
            // Case-insensitive check for "strike" substring
            let lower = name.to_ascii_lowercase();
            strike_flags.push(lower.contains("strike"));
        }

        let effect_flags_vec = id_to_def
            .iter()
            .map(|def| crate::effects::build_effect_flags(def.effects))
            .collect();

        CardRegistry { cards, id_to_def, name_to_id, id_to_name, strike_flags, effect_flags_vec }
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
                effects: &[],
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

    /// Get precomputed effect flags for a card ID. Returns EMPTY for unknown IDs.
    #[inline]
    pub fn effect_flags(&self, id: u16) -> crate::effects::EffectFlags {
        self.effect_flags_vec
            .get(id as usize)
            .copied()
            .unwrap_or(crate::effects::EffectFlags::EMPTY)
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
        assert!(card.effects.contains(&effect), "{} should have effect '{}'", id, effect);
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
        assert!(!reg.get("DevaForm+").unwrap().effects.contains(&"ethereal"));
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
        assert_card(&reg, "Wish", 3, -1, -1, 3, CardType::Skill);
        assert_card(&reg, "Wish+", 3, -1, -1, 4, CardType::Skill);
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
        assert_has_effect(&reg, "Corruption", "corruption");
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
        assert!(!reg.get("Echo Form+").unwrap().effects.contains(&"ethereal"));
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
        assert!(!reg.get("Ghostly+").unwrap().effects.contains(&"ethereal"));
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
            assert!(card.effects.contains(&"unplayable") || card.cost >= 0,
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
