//! Combat engine — slim orchestrator for MCTS simulations.
//!
//! Core turn loop that delegates to:
//! - card_effects: card play effect execution
//! - status_effects: end-of-turn hand card triggers
//! - combat_hooks: enemy turns, boss damage hooks

use pyo3::prelude::*;
use rand::seq::SliceRandom;

use crate::actions::{Action, PyAction};
use crate::cards::{CardDef, CardRegistry, CardTarget, CardType};
use crate::combat_hooks;
use crate::combat_types::CardInstance;
use crate::damage;
use crate::enemies;
use crate::orbs::{EvokeEffect, PassiveEffect};
use crate::potions;
use crate::powers;
use crate::relics;
use crate::state::{CombatState, EnemyCombatState, PyCombatState, Stance};
use crate::status_effects;
use crate::status_ids::sid;

/// Combat phase enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatPhase {
    NotStarted,
    PlayerTurn,
    EnemyTurn,
    AwaitingChoice,
    CombatOver,
}

/// Why we're awaiting a player choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChoiceReason {
    Scry,
    DiscardFromHand,
    ExhaustFromHand,
    PutOnTopFromHand,
    PickFromDiscard,
    PickFromDrawPile,
    DiscoverCard,
    PickOption,
    PlayCardFree,
    DualWield,
    UpgradeCard,
    PickFromExhaust,
    SearchDrawPile,
    ReturnFromDiscard,
    ForethoughtPick,
    RecycleCard,
    DiscardForEffect,
    SetupPick,
}

/// A single option the player can choose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChoiceOption {
    HandCard(usize),
    DrawCard(usize),
    DiscardCard(usize),
    RevealedCard(crate::combat_types::CardInstance),
    GeneratedCard(crate::combat_types::CardInstance),
    Named(&'static str),
    ExhaustCard(usize),
}

/// Context for an in-progress player choice.
#[derive(Debug, Clone)]
pub struct ChoiceContext {
    pub reason: ChoiceReason,
    pub options: Vec<ChoiceOption>,
    pub selected: Vec<usize>,
    pub min_picks: usize,
    pub max_picks: usize,
}

/// The Rust combat engine. Wraps CombatState + card registry + RNG.
#[derive(Clone)]
pub struct CombatEngine {
    pub state: CombatState,
    pub phase: CombatPhase,
    pub card_registry: CardRegistry,
    pub(crate) rng: crate::seed::StsRandom,
    pub choice: Option<ChoiceContext>,
}

impl CombatEngine {
    /// Create a new combat engine.
    pub fn new(state: CombatState, seed: u64) -> Self {
        Self {
            state,
            phase: CombatPhase::NotStarted,
            card_registry: CardRegistry::new(),
            rng: crate::seed::StsRandom::new(seed),
            choice: None,
        }
    }

    // =======================================================================
    // Core API
    // =======================================================================

    /// Start combat: apply relic effects, shuffle draw pile, draw initial hand.
    pub fn start_combat(&mut self) {
        if self.phase != CombatPhase::NotStarted {
            return;
        }

        // Apply combat-start relic effects
        relics::apply_combat_start_relics(&mut self.state);

        // Channel orbs from combat-start relics (need engine context)
        if self.state.player.status(sid::CHANNEL_DARK_START) > 0 {
            self.channel_orb(crate::orbs::OrbType::Dark);
            self.state.player.set_status(sid::CHANNEL_DARK_START, 0);
        }
        if self.state.player.status(sid::CHANNEL_LIGHTNING_START) > 0 {
            self.channel_orb(crate::orbs::OrbType::Lightning);
            self.state.player.set_status(sid::CHANNEL_LIGHTNING_START, 0);
        }
        if self.state.player.status(sid::CHANNEL_PLASMA_START) > 0 {
            self.channel_orb(crate::orbs::OrbType::Plasma);
            self.state.player.set_status(sid::CHANNEL_PLASMA_START, 0);
        }

        // Shuffle draw pile
        self.state.draw_pile.shuffle(&mut self.rng);

        // Start first player turn
        self.start_player_turn();
    }

    /// Check if combat is over.
    pub fn is_combat_over(&self) -> bool {
        self.state.combat_over
    }

    /// Get all legal actions from the current state.
    pub fn get_legal_actions(&self) -> Vec<Action> {
        // If awaiting a choice, only choice actions are legal
        if self.phase == CombatPhase::AwaitingChoice {
            if let Some(ref ctx) = self.choice {
                let mut actions = Vec::new();
                for i in 0..ctx.options.len() {
                    if !ctx.selected.contains(&i) {
                        actions.push(Action::Choose(i));
                    }
                }
                if ctx.max_picks != 1 && ctx.selected.len() >= ctx.min_picks {
                    actions.push(Action::ConfirmSelection);
                }
                return actions;
            }
        }

        if self.phase != CombatPhase::PlayerTurn || self.state.combat_over {
            return Vec::new();
        }

        let mut actions = Vec::new();
        let living = self.state.targetable_enemy_indices();

        // Card plays
        for (hand_idx, card_inst) in self.state.hand.iter().enumerate() {
            let card = self.card_registry.card_def_by_id(card_inst.def_id);
            if self.can_play_card_inst(card, *card_inst) {
                match card.target {
                    CardTarget::Enemy => {
                        for &enemy_idx in &living {
                            actions.push(Action::PlayCard {
                                card_idx: hand_idx,
                                target_idx: enemy_idx as i32,
                            });
                        }
                    }
                    _ => {
                        actions.push(Action::PlayCard {
                            card_idx: hand_idx,
                            target_idx: -1,
                        });
                    }
                }
            }
        }

        // Potion uses
        for (pot_idx, potion_id) in self.state.potions.iter().enumerate() {
            if !potion_id.is_empty() {
                if potions::potion_requires_target(potion_id) {
                    // Targeted potion: one action per living enemy
                    for &enemy_idx in &living {
                        actions.push(Action::UsePotion {
                            potion_idx: pot_idx,
                            target_idx: enemy_idx as i32,
                        });
                    }
                } else {
                    actions.push(Action::UsePotion {
                        potion_idx: pot_idx,
                        target_idx: -1,
                    });
                }
            }
        }

        // End turn is always legal
        actions.push(Action::EndTurn);

        actions
    }

    /// Execute an action.
    pub fn execute_action(&mut self, action: &Action) {
        match action {
            Action::EndTurn => self.end_turn(),
            Action::PlayCard {
                card_idx,
                target_idx,
            } => self.play_card(*card_idx, *target_idx),
            Action::UsePotion {
                potion_idx,
                target_idx,
            } => self.use_potion(*potion_idx, *target_idx),
            Action::Choose(idx) => self.execute_choose(*idx),
            Action::ConfirmSelection => self.execute_confirm_selection(),
        }
    }

    /// Deep clone for MCTS tree search.
    pub fn clone_state(&self) -> CombatEngine {
        CombatEngine {
            state: self.state.clone(),
            phase: self.phase.clone(),
            card_registry: CardRegistry::new(), // Registry is stateless, cheap to recreate
            rng: self.rng.clone(),
            choice: self.choice.clone(),
        }
    }

    // =======================================================================
    // Interactive Choice System
    // =======================================================================

    /// Begin an interactive choice. Sets phase to AwaitingChoice.
    pub fn begin_choice(
        &mut self,
        reason: ChoiceReason,
        options: Vec<ChoiceOption>,
        min_picks: usize,
        max_picks: usize,
    ) {
        if options.is_empty() {
            return; // Nothing to choose from
        }
        if self.phase == CombatPhase::AwaitingChoice {
            return; // Don't overwrite an active choice
        }
        self.phase = CombatPhase::AwaitingChoice;
        self.choice = Some(ChoiceContext {
            reason,
            options,
            selected: Vec::new(),
            min_picks,
            max_picks,
        });
    }

    /// Handle Choose(idx) action.
    fn execute_choose(&mut self, idx: usize) {
        let is_single = {
            let ctx = match self.choice.as_mut() {
                Some(c) => c,
                None => return,
            };
            if idx >= ctx.options.len() || ctx.selected.contains(&idx) {
                return;
            }
            ctx.selected.push(idx);
            ctx.max_picks == 1
        };

        if is_single {
            self.resolve_choice();
        }
    }

    /// Handle ConfirmSelection action (multi-select finalization).
    fn execute_confirm_selection(&mut self) {
        if self.choice.is_some() {
            self.resolve_choice();
        }
    }

    /// Resolve the current choice and return to PlayerTurn.
    fn resolve_choice(&mut self) {
        let ctx = match self.choice.take() {
            Some(c) => c,
            None => return,
        };

        match ctx.reason {
            ChoiceReason::Scry => self.resolve_scry(ctx),
            ChoiceReason::DiscardFromHand => self.resolve_discard_from_hand(ctx),
            ChoiceReason::ExhaustFromHand => self.resolve_exhaust_from_hand(ctx),
            ChoiceReason::PutOnTopFromHand => self.resolve_put_on_top(ctx),
            ChoiceReason::PickFromDiscard => self.resolve_pick_from_discard(ctx),
            ChoiceReason::PickFromDrawPile => self.resolve_pick_from_draw(ctx),
            ChoiceReason::DiscoverCard => self.resolve_discover(ctx),
            ChoiceReason::PickOption => self.resolve_pick_option(ctx),
            ChoiceReason::PlayCardFree => self.resolve_play_card_free(ctx),
            ChoiceReason::DualWield => self.resolve_dual_wield(ctx),
            ChoiceReason::UpgradeCard => self.resolve_upgrade_card(ctx),
            ChoiceReason::PickFromExhaust => self.resolve_pick_from_exhaust(ctx),
            ChoiceReason::SearchDrawPile => self.resolve_search_draw_pile(ctx),
            ChoiceReason::ReturnFromDiscard => self.resolve_return_from_discard(ctx),
            ChoiceReason::ForethoughtPick => self.resolve_forethought(ctx),
            ChoiceReason::RecycleCard => self.resolve_recycle(ctx),
            ChoiceReason::DiscardForEffect => self.resolve_discard_for_effect(ctx),
            ChoiceReason::SetupPick => self.resolve_setup(ctx),
        }

        self.phase = CombatPhase::PlayerTurn;
    }

    fn resolve_scry(&mut self, ctx: ChoiceContext) {
        // Selected indices are cards to discard from the revealed set.
        // The revealed cards were taken from top of draw pile and stored as options.
        // Non-selected go back on top of draw pile, selected go to discard.
        let mut to_discard = Vec::new();
        let mut to_keep = Vec::new();
        for (i, opt) in ctx.options.into_iter().enumerate() {
            if let ChoiceOption::RevealedCard(card) = opt {
                if ctx.selected.contains(&i) {
                    to_discard.push(card);
                } else {
                    to_keep.push(card);
                }
            }
        }
        // Put kept cards back on top of draw pile (front = top)
        for card in to_keep.into_iter().rev() {
            self.state.draw_pile.insert(0, card);
        }
        for card in to_discard {
            self.state.discard_pile.push(card);
        }

        // Nirvana: gain block when scrying
        let nirvana = self.state.player.status(sid::NIRVANA);
        if nirvana > 0 {
            self.gain_block_player(nirvana);
        }

        // Weave: return from discard to hand on Scry
        let mut weave_indices = Vec::new();
        for (i, card_inst) in self.state.discard_pile.iter().enumerate() {
            if self.card_registry.card_name(card_inst.def_id).starts_with("Weave") {
                weave_indices.push(i);
            }
        }
        for &i in weave_indices.iter().rev() {
            let card = self.state.discard_pile.remove(i);
            if self.state.hand.len() < 10 {
                self.state.hand.push(card);
            }
        }
    }

    fn resolve_discard_from_hand(&mut self, ctx: ChoiceContext) {
        // Discard selected hand cards (process in reverse to maintain indices)
        let mut indices: Vec<usize> = ctx.selected.iter().filter_map(|&i| {
            if let ChoiceOption::HandCard(idx) = ctx.options[i] { Some(idx) } else { None }
        }).collect();
        indices.sort_unstable_by(|a, b| b.cmp(a)); // Reverse order
        let discard_count = indices.len();
        let mut discarded_cards = Vec::new();
        for idx in indices {
            if idx < self.state.hand.len() {
                let card = self.state.hand.remove(idx);
                self.state.discard_pile.push(card);
                discarded_cards.push(card);
            }
        }
        // Fire on_card_discarded hooks for each discarded card
        for card in discarded_cards {
            self.on_card_discarded(card);
        }
        // Gambling Chip: redraw equal to discarded count
        if self.state.player.status(sid::GAMBLING_CHIP_ACTIVE) > 0 {
            self.state.player.set_status(sid::GAMBLING_CHIP_ACTIVE, 0);
            if discard_count > 0 {
                self.draw_cards(discard_count as i32);
            }
        }
    }

    fn resolve_exhaust_from_hand(&mut self, ctx: ChoiceContext) {
        let mut indices: Vec<usize> = ctx.selected.iter().filter_map(|&i| {
            if let ChoiceOption::HandCard(idx) = ctx.options[i] { Some(idx) } else { None }
        }).collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        for idx in indices {
            if idx < self.state.hand.len() {
                let card = self.state.hand.remove(idx);
                self.state.exhaust_pile.push(card);
                self.trigger_on_exhaust();
            }
        }
    }

    fn resolve_put_on_top(&mut self, ctx: ChoiceContext) {
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::HandCard(idx) = ctx.options[sel] {
                if idx < self.state.hand.len() {
                    let card = self.state.hand.remove(idx);
                    self.state.draw_pile.insert(0, card);
                }
            }
        }
    }

    fn resolve_pick_from_discard(&mut self, ctx: ChoiceContext) {
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::DiscardCard(idx) = ctx.options[sel] {
                if idx < self.state.discard_pile.len() {
                    let card = self.state.discard_pile.remove(idx);
                    // Put on top of draw pile (Headbutt)
                    self.state.draw_pile.insert(0, card);
                }
            }
        }
    }

    fn resolve_pick_from_draw(&mut self, ctx: ChoiceContext) {
        // Seek: move selected card(s) from draw pile to hand
        let mut indices: Vec<usize> = ctx.selected.iter().filter_map(|&i| {
            if let ChoiceOption::DrawCard(idx) = ctx.options[i] { Some(idx) } else { None }
        }).collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        for idx in indices {
            if idx < self.state.draw_pile.len() && self.state.hand.len() < 10 {
                let card = self.state.draw_pile.remove(idx);
                self.state.hand.push(card);
            }
        }
    }

    fn resolve_discover(&mut self, ctx: ChoiceContext) {
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::GeneratedCard(card) = ctx.options[sel] {
                if self.state.hand.len() < 10 {
                    self.state.hand.push(card);
                }
            }
        }
    }

    fn resolve_pick_option(&mut self, ctx: ChoiceContext) {
        // Wish: Named options [Strength, Gold, Plated Armor]
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::Named(name) = ctx.options[sel] {
                match name {
                    "Strength" => {
                        let current = self.state.player.status(sid::STRENGTH);
                        self.state.player.set_status(sid::STRENGTH, current + 3);
                    }
                    "Gold" => {
                        // Can't modify run gold from combat engine; grant 3 Plated Armor instead
                        let current = self.state.player.status(sid::PLATED_ARMOR);
                        self.state.player.set_status(sid::PLATED_ARMOR, current + 3);
                    }
                    "Plated Armor" => {
                        let current = self.state.player.status(sid::PLATED_ARMOR);
                        self.state.player.set_status(sid::PLATED_ARMOR, current + 3);
                    }
                    _ => {}
                }
            }
        }
    }

    fn resolve_dual_wield(&mut self, ctx: ChoiceContext) {
        // Dual Wield: duplicate selected card in hand
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::HandCard(idx) = ctx.options[sel] {
                if idx < self.state.hand.len() {
                    let card = self.state.hand[idx];
                    // base_magic determines copy count (1 base, 2 upgraded)
                    let copies = ctx.max_picks.max(1);
                    for _ in 0..copies {
                        if self.state.hand.len() >= 10 { break; }
                        self.state.hand.push(card);
                    }
                }
            }
        }
    }

    fn resolve_upgrade_card(&mut self, ctx: ChoiceContext) {
        // Armaments: upgrade selected card in hand
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::HandCard(idx) = ctx.options[sel] {
                if idx < self.state.hand.len() {
                    self.state.hand[idx].flags |= 0x04; // UPGRADED flag
                }
            }
        }
    }

    fn resolve_pick_from_exhaust(&mut self, ctx: ChoiceContext) {
        // Exhume: move selected card from exhaust pile to hand
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::ExhaustCard(idx) = ctx.options[sel] {
                if idx < self.state.exhaust_pile.len() && self.state.hand.len() < 10 {
                    let card = self.state.exhaust_pile.remove(idx);
                    self.state.hand.push(card);
                }
            }
        }
    }

    fn resolve_play_card_free(&mut self, ctx: ChoiceContext) {
        // Omniscience: play selected card from hand for free, twice
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::HandCard(idx) = ctx.options[sel] {
                if idx < self.state.hand.len() {
                    // Set card to free
                    self.state.hand[idx].cost = 0;
                    self.state.hand[idx].flags |= crate::combat_types::CardInstance::FLAG_FREE;
                    // Play it (target -1 = self; for targeted cards MCTS will handle)
                    let target = if self.card_registry.card_def_by_id(self.state.hand[idx].def_id).target == CardTarget::Enemy {
                        self.state.targetable_enemy_indices().first().copied().unwrap_or(0) as i32
                    } else {
                        -1
                    };
                    self.play_card(idx, target);
                }
            }
        }
    }

    fn resolve_search_draw_pile(&mut self, ctx: ChoiceContext) {
        // Secret Weapon / Secret Technique: move selected card from draw pile to hand
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::DrawCard(idx) = ctx.options[sel] {
                if idx < self.state.draw_pile.len() && self.state.hand.len() < 10 {
                    let card = self.state.draw_pile.remove(idx);
                    self.state.hand.push(card);
                }
            }
        }
    }

    fn resolve_return_from_discard(&mut self, ctx: ChoiceContext) {
        // Hologram: move selected card from discard to hand (not top of draw)
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::DiscardCard(idx) = ctx.options[sel] {
                if idx < self.state.discard_pile.len() && self.state.hand.len() < 10 {
                    let card = self.state.discard_pile.remove(idx);
                    self.state.hand.push(card);
                }
            }
        }
    }

    fn resolve_forethought(&mut self, ctx: ChoiceContext) {
        // Forethought: put selected card(s) on bottom of draw pile at 0 cost
        let mut indices: Vec<usize> = ctx.selected.iter().filter_map(|&i| {
            if let ChoiceOption::HandCard(idx) = ctx.options[i] { Some(idx) } else { None }
        }).collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        for idx in indices {
            if idx < self.state.hand.len() {
                let mut card = self.state.hand.remove(idx);
                card.cost = 0;
                // Bottom of draw pile = end of vec (top = index 0 or beginning)
                self.state.draw_pile.push(card);
            }
        }
    }

    fn resolve_recycle(&mut self, ctx: ChoiceContext) {
        // Recycle: exhaust selected card, gain its cost as energy
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::HandCard(idx) = ctx.options[sel] {
                if idx < self.state.hand.len() {
                    let card = self.state.hand.remove(idx);
                    // cost == -1 means "use CardDef base cost"
                    let effective_cost = if card.cost >= 0 {
                        card.cost as i32
                    } else {
                        self.card_registry.card_def_by_id(card.def_id).cost.max(0)
                    };
                    self.state.energy += effective_cost;
                    self.state.exhaust_pile.push(card);
                    self.trigger_on_exhaust();
                }
            }
        }
    }

    fn resolve_setup(&mut self, ctx: ChoiceContext) {
        // Setup: set card cost to 0 and put on top of draw pile
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::HandCard(idx) = ctx.options[sel] {
                if idx < self.state.hand.len() {
                    let mut card = self.state.hand.remove(idx);
                    card.cost = 0;
                    self.state.draw_pile.insert(0, card);
                }
            }
        }
    }

    fn resolve_discard_for_effect(&mut self, ctx: ChoiceContext) {
        // Concentrate: discard N cards, then gain energy
        let mut indices: Vec<usize> = ctx.selected.iter().filter_map(|&i| {
            if let ChoiceOption::HandCard(idx) = ctx.options[i] { Some(idx) } else { None }
        }).collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        for idx in indices {
            if idx < self.state.hand.len() {
                let card = self.state.hand.remove(idx);
                self.state.discard_pile.push(card);
            }
        }
        // Concentrate gives 2 energy after discarding
        self.state.energy += 2;
    }

    // =======================================================================
    // Turn Flow
    // =======================================================================

    fn start_player_turn(&mut self) {
        self.state.turn += 1;
        self.phase = CombatPhase::PlayerTurn;

        // Blasphemy: die at start of turn
        if self.state.blasphemy_active {
            self.state.blasphemy_active = false;
            self.state.player.hp = 0;
            self.state.combat_over = true;
            self.state.player_won = false;
            self.phase = CombatPhase::CombatOver;
            self.choice = None;
            return;
        }

        // Reset energy — Ice Cream preserves unspent energy
        if relics::has_ice_cream(&self.state) {
            self.state.energy += self.state.max_energy;
        } else {
            self.state.energy = self.state.max_energy;
        }

        // Reset turn counters
        self.state.cards_played_this_turn = 0;
        self.state.attacks_played_this_turn = 0;
        self.state.last_card_type = None;

        // Reset per-turn statuses
        self.state.player.set_status(sid::WAVE_OF_THE_HAND, 0);
        self.state.player.set_status(sid::DISCARDED_THIS_TURN, 0);

        // Necronomicon reset
        relics::necronomicon_reset(&mut self.state);

        // All turn-start relic effects (Lantern, Happy Flower, Mercury Hourglass, etc.)
        relics::apply_turn_start_relics(&mut self.state);

        // Divinity auto-exit at start of turn
        if self.state.stance == Stance::Divinity {
            self.change_stance(Stance::Neutral);
        }

        // Block decay — Calipers retains up to 15, Barricade retains all
        // Skip block reset on turn 1 to preserve combat-start relic effects (Anchor)
        if self.state.turn > 1 {
            let barricade = self.state.player.status(sid::BARRICADE) > 0;
            let blur = self.state.player.status(sid::BLUR) > 0;
            if barricade || blur {
                // Keep all block
            } else {
                let retained = relics::calipers_block_retention(&self.state, self.state.player.block);
                self.state.player.block = retained;
            }
            // Blur: decrement after use (Java: BlurPower is turn-based, decrements at end of round)
            if blur {
                let blur_val = self.state.player.status(sid::BLUR);
                self.state.player.set_status(sid::BLUR, (blur_val - 1).max(0));
            }
        }

        // LoseStrength/LoseDexterity at end of previous turn
        let lose_str = self.state.player.status(sid::LOSE_STRENGTH);
        if lose_str > 0 {
            self.state.player.add_status(sid::STRENGTH, -lose_str);
            self.state.player.set_status(sid::LOSE_STRENGTH, 0);
        }
        let lose_dex = self.state.player.status(sid::LOSE_DEXTERITY);
        if lose_dex > 0 {
            self.state.player.add_status(sid::DEXTERITY, -lose_dex);
            self.state.player.set_status(sid::LOSE_DEXTERITY, 0);
        }

        // === POWER HOOKS: start of turn ===
        // Dispatch collects all power effects; engine applies them in correct order.
        // Hooks that mutate entity directly (DemonForm/Strength, WraithForm/Dex) do so
        // inside the hook fn. One-shot hooks (Doppelganger, EnterDivinity) clear themselves.
        let fx = powers::registry::dispatch_turn_start(&mut self.state.player);

        // Pre-draw: energy from hooks (DevaForm, Berserk, DoppelgangerEnergy)
        self.state.energy += fx.energy + fx.doppelganger_energy;

        // ---- Start-of-turn orb passives (Plasma) ----
        self.apply_orb_start_of_turn();

        // Pre-draw: add temp cards to hand (BattleHymn smites)
        for _ in 0..fx.add_smites {
            let smite_id = self.temp_card("Smite");
            if self.state.hand.len() < 10 {
                self.state.hand.push(smite_id);
            }
        }

        // Draw cards (default 5 + Draw/Machine Learning power + Ring of the Serpent)
        let ml = self.state.player.status(sid::DRAW);
        let serpent = self.state.player.status(sid::RING_OF_SERPENT_DRAW);
        self.draw_cards(5 + ml + serpent);

        // TurnStartExtraDraw: one-shot extra draw from relics (Bag of Prep, etc.)
        let extra_draw = self.state.player.status(sid::TURN_START_EXTRA_DRAW);
        if extra_draw > 0 {
            self.draw_cards(extra_draw);
            self.state.player.set_status(sid::TURN_START_EXTRA_DRAW, 0);
        }

        // InkBottleDraw: one-shot extra draw from Ink Bottle relic trigger
        let ink_draw = self.state.player.status(sid::INK_BOTTLE_DRAW);
        if ink_draw > 0 {
            self.draw_cards(ink_draw);
            self.state.player.set_status(sid::INK_BOTTLE_DRAW, 0);
        }

        // ---- Post-draw power effects ----

        // Devotion: gain Mantra (Java: atStartOfTurnPostDraw)
        if fx.mantra_gain > 0 {
            self.gain_mantra(fx.mantra_gain);
        }

        // CreativeAI: add random Power card to hand (MCTS: add "Smite")
        for _ in 0..fx.add_creative_ai_cards {
            if self.state.hand.len() < 10 {
                let smite_id = self.temp_card("Smite");
                self.state.hand.push(smite_id);
            }
        }

        // DoppelgangerDraw: consume extra draws
        if fx.doppelganger_draw > 0 {
            self.draw_cards(fx.doppelganger_draw);
        }

        // Magnetism + HelloWorld: add Strikes to hand
        for _ in 0..fx.add_strikes {
            if self.state.hand.len() < 10 {
                let strike_id = self.temp_card("Strike");
                self.state.hand.push(strike_id);
            }
        }

        // EnterDivinity (Damaru relic)
        if fx.enter_divinity {
            self.change_stance(Stance::Divinity);
        }

        // Mayhem: add top card(s) of draw pile to hand
        for _ in 0..fx.mayhem_draw {
            if self.state.hand.len() < 10 {
                if let Some(card_id) = self.state.draw_pile.pop() {
                    self.state.hand.push(card_id);
                }
            }
        }

        // NoxiousFumes: apply Poison to all living enemies
        if fx.poison_all_enemies > 0 {
            for ei in 0..self.state.enemies.len() {
                if self.state.enemies[ei].is_alive() {
                    self.state.enemies[ei].entity.add_status(sid::POISON, fx.poison_all_enemies);
                }
            }
        }

        // Brutality: draw + HP loss (draw from hook, HP loss applied here)
        if fx.hp_loss > 0 {
            self.draw_cards(fx.draw);
            self.player_lose_hp(fx.hp_loss);
            if self.state.combat_over {
                return;
            }
        }

        // InfiniteBlades: add Shivs to hand
        for _ in 0..fx.add_shivs {
            let shiv_id = self.temp_card("Shiv");
            if self.state.hand.len() < 10 {
                self.state.hand.push(shiv_id);
            }
        }

        // ToolsOfTheTrade: draw then player chooses card to discard
        if fx.tools_of_the_trade_draw > 0 {
            self.draw_cards(fx.tools_of_the_trade_draw);
            if fx.tools_of_the_trade_discard > 0 && !self.state.hand.is_empty() {
                let options: Vec<ChoiceOption> = (0..self.state.hand.len())
                    .map(|i| ChoiceOption::HandCard(i))
                    .collect();
                self.begin_choice(ChoiceReason::DiscardFromHand, options, 1, 1);
                return; // Pause turn start; resumes after choice
            }
        }

        // WarpedTongs: upgrade a random card in hand each turn
        if self.state.has_relic("WarpedTongs") && !self.state.hand.is_empty() {
            let idx = self.rng.random(self.state.hand.len() as i32 - 1) as usize;
            self.card_registry.upgrade_card(&mut self.state.hand[idx]);
        }

        // Gambling Chip: at start of each turn, player chooses cards to discard and redraws
        if self.state.has_relic("Gambling Chip") || self.state.has_relic("GamblingChip") {
            if !self.state.hand.is_empty() {
                let options: Vec<ChoiceOption> = (0..self.state.hand.len())
                    .map(|i| ChoiceOption::HandCard(i))
                    .collect();
                let n = options.len();
                self.begin_choice(ChoiceReason::DiscardFromHand, options, 0, n);
                // After confirm, resolve_discard_from_hand will discard selected cards.
                // We need to redraw equal count -- handled in resolve_discard_from_hand
                // by checking if the reason originated from Gambling Chip.
                // For now, we store a flag so resolve knows to draw replacements.
                self.state.player.set_status(sid::GAMBLING_CHIP_ACTIVE, 1);
            }
        }
    }

    fn end_turn(&mut self) {
        if self.phase != CombatPhase::PlayerTurn {
            return;
        }

        // Clear Entangled (only lasts one turn)
        self.state.player.set_status(sid::ENTANGLED, 0);

        // ---- STS end-of-turn order: relics -> powers/buffs -> status cards -> discard ----

        // 1. End-of-turn relic triggers
        relics::apply_turn_end_relics(&mut self.state);

        // 2. End-of-turn power triggers (via hook dispatch)
        let in_calm = self.state.stance == Stance::Calm;
        let efx = powers::registry::dispatch_turn_end(&mut self.state.player, in_calm);

        // Block gains (Metallicize, PlatedArmor, LikeWater)
        if efx.block_gain > 0 {
            self.gain_block_player(efx.block_gain);
        }

        // Study: add Insight(s) to draw pile
        for _ in 0..efx.add_insights {
            let insight_id = self.temp_card("Insight");
            self.state.draw_pile.push(insight_id);
        }

        // Omega: deal damage to all living enemies
        if efx.omega_damage > 0 {
            let living = self.state.living_enemy_indices();
            for idx in living {
                self.deal_damage_to_enemy(idx, efx.omega_damage);
            }
        }

        // Combust: lose HP first (death check), then deal damage to all enemies
        if efx.combust_hp_loss > 0 {
            self.player_lose_hp(efx.combust_hp_loss);
            if self.state.combat_over {
                return;
            }
            let living = self.state.living_enemy_indices();
            for idx in living {
                self.deal_damage_to_enemy(idx, efx.combust_damage);
            }
        }

        // TempStrength revert and Rage clear are handled inside the hook fns
        // NOTE: Regeneration stays inline (fires after Constricted/orb passives)

        // TempStrengthLoss: restore temporary Strength loss on all enemies at end of turn
        for ei in 0..self.state.enemies.len() {
            if self.state.enemies[ei].is_alive() {
                let tsl = self.state.enemies[ei].entity.status(sid::TEMP_STRENGTH_LOSS);
                if tsl > 0 {
                    self.state.enemies[ei].entity.add_status(sid::STRENGTH, tsl);
                    self.state.enemies[ei].entity.set_status(sid::TEMP_STRENGTH_LOSS, 0);
                }
            }
        }

        // 3. End-of-turn hand card triggers (Burn, Decay, Regret, Doubt, Shame)
        let player_died = status_effects::process_end_turn_hand_cards(
            &mut self.state,
            &self.card_registry,
        );
        if player_died {
            self.phase = CombatPhase::CombatOver;
            return;
        }

        // 4. Discard hand — Runic Pyramid keeps ALL cards in hand (including Status/Curse).
        //    Only Ethereal cards exhaust at end of turn regardless of Runic Pyramid.
        let _explicitly_retained = std::mem::take(&mut self.state.retained_cards);
        let mut ethereal_exhausted = 0i32;
        if relics::has_runic_pyramid(&self.state) {
            // Runic Pyramid: keep ALL cards except ethereal (which exhaust)
            let hand = std::mem::take(&mut self.state.hand);
            let mut kept = Vec::new();
            for card_inst in hand {
                let card = self.card_registry.card_def_by_id(card_inst.def_id);
                if card.effects.contains(&"ethereal") {
                    self.state.exhaust_pile.push(card_inst);
                    ethereal_exhausted += 1;
                } else {
                    let mut retained_inst = card_inst;
                    retained_inst.set_retained(true);
                    kept.push(retained_inst);
                }
            }
            // Track retained cards for Establishment cost reduction
            self.state.retained_cards = kept.clone();
            self.state.hand = kept;
        } else {
            // Normal: retain tagged cards + explicitly retained (FLAG_RETAINED), exhaust ethereal, discard rest
            let hand = std::mem::take(&mut self.state.hand);
            let mut retained = Vec::new();
            for card_inst in hand {
                let card = self.card_registry.card_def_by_id(card_inst.def_id);
                if card.effects.contains(&"retain") || card_inst.is_retained() {
                    let mut retained_inst = card_inst;
                    retained_inst.set_retained(true);
                    retained.push(retained_inst);
                } else if card.effects.contains(&"ethereal") {
                    self.state.exhaust_pile.push(card_inst);
                    ethereal_exhausted += 1;
                } else {
                    self.state.discard_pile.push(card_inst);
                }
            }
            // Track retained cards for Establishment cost reduction
            self.state.retained_cards = retained.clone();
            self.state.hand = retained;
        }

        // on_retain hooks for retained cards
        let establishment = self.state.player.status(sid::ESTABLISHMENT);
        for card_inst in self.state.hand.iter_mut() {
            let card_def = self.card_registry.card_def_by_id(card_inst.def_id);

            // Establishment: reduce retained card cost
            if establishment > 0 {
                card_inst.cost = (card_inst.cost - establishment as i8).max(0);
            }

            // Sands of Time: reduce cost on retain
            if card_def.effects.contains(&"reduce_cost_on_retain") {
                card_inst.cost = (card_inst.cost - 1).max(0);
            }

            // Perseverance: grow block bonus on retain
            if card_def.effects.contains(&"grow_block_on_retain") {
                self.state.player.add_status(sid::PERSEVERANCE_BONUS, card_def.base_magic);
            }

            // Windmill Strike: grow damage bonus on retain
            if card_def.effects.contains(&"grow_damage_on_retain") {
                self.state.player.add_status(sid::WINDMILL_STRIKE_BONUS, card_def.base_magic);
            }
        }

        // Trigger exhaust hooks for ethereal cards exhausted at end of turn
        for _ in 0..ethereal_exhausted {
            self.trigger_on_exhaust();
        }

        // ---- End-of-turn orb passives ----
        self.apply_orb_end_of_turn();
        if self.state.combat_over {
            return;
        }

        // Loop: trigger front orb passive again
        let loop_count = self.state.player.status(sid::LOOP);
        if loop_count > 0 && self.state.orb_slots.has_orbs() {
            let focus = self.state.player.focus();
            let front = &mut self.state.orb_slots.slots[0];
            if !front.is_empty() {
                let effect = match front.orb_type {
                    crate::orbs::OrbType::Lightning => {
                        let damage = front.passive_with_focus(focus);
                        PassiveEffect::LightningDamage(damage)
                    }
                    crate::orbs::OrbType::Frost => {
                        let block = front.passive_with_focus(focus);
                        PassiveEffect::FrostBlock(block)
                    }
                    crate::orbs::OrbType::Dark => {
                        let gain = front.passive_with_focus(focus);
                        front.evoke_amount += gain;
                        PassiveEffect::None
                    }
                    crate::orbs::OrbType::Plasma => {
                        PassiveEffect::PlasmaEnergy(front.base_passive)
                    }
                    crate::orbs::OrbType::Empty => PassiveEffect::None,
                };
                self.apply_passive_effect(effect);
                if self.state.combat_over {
                    return;
                }
            }
        }

        // FrozenCore: at end of turn, if no orbs in slots, channel 1 Frost
        if self.state.player.status(sid::FROZEN_CORE_TRIGGER) > 0 {
            if self.state.orb_slots.occupied_count() == 0 {
                self.channel_orb(crate::orbs::OrbType::Frost);
            }
        }

        // Constricted: deal Constricted damage to player at end of turn
        let constricted = self.state.player.status(sid::CONSTRICTED);
        if constricted > 0 {
            let intangible = self.state.player.status(sid::INTANGIBLE) > 0;
            let has_tungsten = self.state.has_relic("Tungsten Rod");
            let hp_loss = damage::apply_hp_loss(constricted, intangible, has_tungsten);
            self.player_lose_hp(hp_loss);
            if self.state.combat_over {
                return;
            }
        }

        // Player Regeneration: heal and decrement (Java: RegenerationPower.atEndOfTurn)
        let regen = self.state.player.status(sid::REGENERATION);
        if regen > 0 {
            self.heal_player(regen);
            self.state.player.add_status(sid::REGENERATION, -1);
        }

        // Player poison tick (before enemy turns)
        let player_poison = self.state.player.status(sid::POISON);
        if player_poison > 0 {
            let intangible = self.state.player.status(sid::INTANGIBLE) > 0;
            let tungsten_rod = self.state.has_relic("Tungsten Rod");
            let hp_loss = damage::apply_hp_loss(player_poison, intangible, tungsten_rod);
            // Decrement poison by 1
            let new_poison = player_poison - 1;
            self.state.player.set_status(sid::POISON, new_poison);
            self.player_lose_hp(hp_loss);
            if self.state.combat_over {
                return;
            }
        }

        // Check combat end (Omega may have killed enemies)
        if self.check_combat_end() {
            return;
        }

        // Enemy turns (skip if Vault was played)
        if self.state.skip_enemy_turn {
            self.state.skip_enemy_turn = false;
        } else {
            combat_hooks::do_enemy_turns(self);
        }

        // End of round: decrement debuffs on player
        powers::decrement_debuffs(&mut self.state.player);

        // End of round: decrement debuffs on enemies
        for enemy in &mut self.state.enemies {
            if !enemy.entity.is_dead() {
                powers::decrement_debuffs(&mut enemy.entity);
                // Decrement enemy Intangible (Nemesis cycling)
                let intang = enemy.entity.status(sid::INTANGIBLE);
                if intang > 0 {
                    enemy.entity.set_status(sid::INTANGIBLE, intang - 1);
                }
            }
        }

        // Decrement player Intangible
        let intangible = self.state.player.status(sid::INTANGIBLE);
        if intangible > 0 {
            self.state.player.set_status(sid::INTANGIBLE, intangible - 1);
        }

        // Check combat end
        if !self.check_combat_end() {
            self.start_player_turn();
        }
    }

    // =======================================================================
    // Card Play
    // =======================================================================

    fn can_play_card_inst(&self, card: &CardDef, card_inst: CardInstance) -> bool {
        // Unplayable cards -- unless Medical Kit (Status) or Blue Candle (Curse)
        if card.cost == -2 || card.effects.contains(&"unplayable") {
            if card.card_type == CardType::Status
                && (self.state.has_relic("Medical Kit") || self.state.has_relic("MedicalKit"))
            {
                // Medical Kit: Status cards become playable (exhaust on play, cost 0)
            } else if card.card_type == CardType::Curse
                && (self.state.has_relic("Blue Candle") || self.state.has_relic("BlueCandle"))
            {
                // Blue Candle: Curse cards become playable (1 HP + exhaust, cost 0)
            } else {
                return false;
            }
        }

        // Velvet Choker: max 6 cards per turn
        if !relics::velvet_choker_can_play(&self.state) {
            return false;
        }

        // Energy check — Confusion: any card could cost 0-3, so playable if energy >= 0
        if self.state.player.status(sid::CONFUSION) > 0 && card.cost >= 0 {
            if self.state.energy < 0 {
                return false;
            }
        } else {
            let cost = self.effective_cost_inst(card, card_inst);
            if cost > self.state.energy {
                return false;
            }
        }

        // Entangled: can't play attacks
        if self.state.player.status(sid::ENTANGLED) > 0 && card.card_type == CardType::Attack {
            return false;
        }

        // Signature Move: only playable if no other attacks in hand
        if card.effects.contains(&"only_attack_in_hand") {
            let other_attacks = self.state.hand.iter().filter(|c| {
                let other_card = self.card_registry.card_def_by_id(c.def_id);
                other_card.card_type == CardType::Attack && c.def_id != card_inst.def_id
            }).count();
            if other_attacks > 0 {
                return false;
            }
        }

        // Clash: only playable if hand contains only attacks
        if card.effects.contains(&"only_attacks_in_hand") {
            let has_non_attack = self.state.hand.iter().any(|c| {
                let other_card = self.card_registry.card_def_by_id(c.def_id);
                other_card.card_type != CardType::Attack
            });
            if has_non_attack {
                return false;
            }
        }

        // Grand Finale: only playable if draw pile is empty
        if card.effects.contains(&"only_empty_draw") && !self.state.draw_pile.is_empty() {
            return false;
        }

        true
    }

    /// Resolve the effective cost of a card instance.
    ///
    /// Priority: X-cost -> free overrides -> Confusion -> instance cost -> CardDef cost.
    /// CardInstance.cost == -1 means "use CardDef base cost" (the default).
    /// When a card's cost is modified at runtime (Streamline, Madness, etc.),
    /// the instance cost is set to a non-negative value which takes precedence.
    fn effective_cost_inst(&self, card: &CardDef, card_inst: CardInstance) -> i32 {
        // X-cost cards: cost is consumed separately in card_effects
        if card.cost == -1 {
            return 0;
        }

        // Free overrides
        if card_inst.is_free() {
            return 0;
        }
        if card.card_type == CardType::Attack && self.state.player.status(sid::NEXT_ATTACK_FREE) > 0 {
            return 0;
        }
        if card.card_type == CardType::Skill && self.state.player.status(sid::CORRUPTION) > 0 {
            return 0;
        }
        if self.state.player.status(sid::BULLET_TIME) > 0 {
            return 0;
        }

        // Confusion/SneckoEye: MCTS approximation (deterministic midpoint)
        if self.state.player.status(sid::CONFUSION) > 0 {
            return 1;
        }

        // Instance cost overrides CardDef cost when set (>= 0)
        let mut cost = if card_inst.cost >= 0 {
            card_inst.cost as i32
        } else {
            card.cost
        };

        // Establishment: retained cards cost 1 less per stack
        let establishment = self.state.player.status(sid::ESTABLISHMENT);
        if establishment > 0 && card_inst.is_retained() {
            cost = (cost - establishment).max(0);
        }

        cost
    }

    /// Effective cost with RNG for actual card play (Confusion randomization).
    /// Called from play_card where &mut self is available.
    fn effective_cost_mut_inst(&mut self, card: &CardDef, card_inst: CardInstance) -> i32 {
        // X-cost cards: cost is consumed separately in card_effects
        if card.cost == -1 {
            return 0;
        }

        // Free overrides
        if card_inst.is_free() {
            return 0;
        }
        if card.card_type == CardType::Attack && self.state.player.status(sid::NEXT_ATTACK_FREE) > 0 {
            return 0;
        }
        if card.card_type == CardType::Skill && self.state.player.status(sid::CORRUPTION) > 0 {
            return 0;
        }
        if self.state.player.status(sid::BULLET_TIME) > 0 {
            return 0;
        }

        // Confusion/SneckoEye: randomize card costs 0-3
        if self.state.player.status(sid::CONFUSION) > 0 {
            return self.rng.random(3);
        }

        // Instance cost overrides CardDef cost when set (>= 0)
        let mut cost = if card_inst.cost >= 0 {
            card_inst.cost as i32
        } else {
            card.cost
        };

        // Establishment: retained cards cost 1 less per stack
        let establishment = self.state.player.status(sid::ESTABLISHMENT);
        if establishment > 0 && card_inst.is_retained() {
            cost = (cost - establishment).max(0);
        }

        cost
    }

    fn play_card(&mut self, hand_idx: usize, target_idx: i32) {
        if hand_idx >= self.state.hand.len() {
            return;
        }

        let card_inst = self.state.hand[hand_idx]; // Copy, no clone needed
        let card = self.card_registry.card_def_by_id(card_inst.def_id).clone();
        let card_id = self.card_registry.card_name(card_inst.def_id).to_string();

        if !self.can_play_card_inst(&card, card_inst) {
            return;
        }

        // Medical Kit: Status cards are played for free and exhausted
        if card.card_type == CardType::Status
            && (self.state.has_relic("Medical Kit") || self.state.has_relic("MedicalKit"))
        {
            self.state.hand.remove(hand_idx);
            self.state.cards_played_this_turn += 1;
            self.state.total_cards_played += 1;
            self.state.exhaust_pile.push(card_inst);
            self.trigger_on_exhaust();
            relics::on_card_played(&mut self.state, card.card_type);
            return;
        }

        // Blue Candle: Curse cards are played for free, exhaust, and deal 1 HP
        if card.card_type == CardType::Curse
            && (self.state.has_relic("Blue Candle") || self.state.has_relic("BlueCandle"))
        {
            self.state.hand.remove(hand_idx);
            self.state.cards_played_this_turn += 1;
            self.state.total_cards_played += 1;
            self.state.exhaust_pile.push(card_inst);
            self.trigger_on_exhaust();
            self.player_lose_hp(1);
            relics::on_card_played(&mut self.state, card.card_type);
            if self.state.combat_over { return; }
            return;
        }

        // Pay energy (use RNG-aware version for Confusion randomization)
        let cost = self.effective_cost_mut_inst(&card, card_inst);
        self.state.energy -= cost;

        // Remove from hand
        self.state.hand.remove(hand_idx);

        // Track counters
        self.state.cards_played_this_turn += 1;
        self.state.total_cards_played += 1;
        if card.card_type == CardType::Attack {
            self.state.attacks_played_this_turn += 1;
        }

        // ---- Java onUseCard hooks (fire BEFORE card effects resolve) ----

        // AfterImage: gain block per card played (via hook dispatch, pre-effects)
        let pre_fx = powers::registry::dispatch_on_card_played_pre(&self.state.player);
        if pre_fx.block_gain > 0 {
            self.gain_block_player(pre_fx.block_gain);
        }

        // Execute effects (last_card_type refers to card played BEFORE this one)
        crate::card_effects::execute_card_effects(self, &card, card_inst, target_idx);

        // Envenom: when Attack deals unblocked damage, apply Poison to target
        // MCTS approximation: apply Envenom Poison to target after every attack card
        let envenom = self.state.player.status(sid::ENVENOM);
        if envenom > 0 && card.card_type == CardType::Attack && target_idx >= 0 {
            let tidx = target_idx as usize;
            if tidx < self.state.enemies.len() && self.state.enemies[tidx].is_alive() {
                self.state.enemies[tidx].entity.add_status(sid::POISON, envenom);
            }
        }

        // Sadistic Nature: deal damage when debuff applied to enemy
        // MCTS approximation: deal Sadistic damage per debuff-applying attack
        let sadistic = self.state.player.status(sid::SADISTIC);
        if sadistic > 0 && card.card_type == CardType::Attack && target_idx >= 0 {
            // Check if card applies debuffs (Weak, Vulnerable, Poison via effects)
            let applies_debuff = card.effects.iter().any(|e| {
                *e == "weak" || *e == "vulnerable" || *e == "poison"
                    || *e == "weak_all" || *e == "vulnerable_all"
            });
            if applies_debuff {
                let tidx = target_idx as usize;
                if tidx < self.state.enemies.len() && self.state.enemies[tidx].is_alive() {
                    self.deal_damage_to_enemy(tidx, sadistic);
                }
            }
        }

        // Electrodynamics: when playing an Attack, channel Lightning for each living enemy
        if card.card_type == CardType::Attack && self.state.player.status(sid::ELECTRODYNAMICS) > 0 {
            let count = self.state.living_enemy_indices().len();
            for _ in 0..count {
                self.channel_orb(crate::orbs::OrbType::Lightning);
            }
        }

        // Update last_card_type AFTER effects (so next card sees this one)
        self.state.last_card_type = Some(card.card_type);

        // Stance change from card
        if let Some(stance_name) = card.enter_stance {
            let new_stance = Stance::from_str(stance_name);
            self.change_stance(new_stance);
        }

        // Gremlin Nob Enrage: gains Strength when player plays non-Attack
        if card.card_type != CardType::Attack {
            for enemy in &mut self.state.enemies {
                if enemy.is_alive() {
                    let enrage = enemy.entity.status(sid::ENRAGE);
                    if enrage > 0 {
                        enemy.entity.add_status(sid::STRENGTH, enrage);
                    }
                }
            }
        }

        // Hex: when player plays a non-Attack card, add 1 Daze to draw pile
        if card.card_type != CardType::Attack {
            let hex = self.state.player.status(sid::HEX);
            if hex > 0 {
                for _ in 0..hex {
                    self.state.draw_pile.push(self.card_registry.make_card("Daze"));
                }
            }
        }

        // Pain curse: deal 1 HP loss per Pain card in hand on every card play
        let pain_killed = status_effects::process_pain_on_card_play(
            &mut self.state,
            &self.card_registry,
        );
        if pain_killed {
            self.phase = CombatPhase::CombatOver;
            return;
        }

        // All on-card-play relic effects (Fan, Kunai, Shuriken, Nunchaku, etc.)
        relics::on_card_played(&mut self.state, card.card_type);

        // ---- Power hooks on card play (AFTER effects) ----
        // Note: After Image and Beat of Death fire BEFORE effects (Java onUseCard order)

        // A Thousand Cuts: deal damage to ALL living enemies per card played
        let thousand_cuts_dmg = powers::get_thousand_cuts_damage(&self.state.player);
        if thousand_cuts_dmg > 0 {
            let living = self.state.living_enemy_indices();
            for idx in living {
                self.deal_damage_to_enemy(idx, thousand_cuts_dmg);
            }
        }

        // Rage: gain block when playing an Attack (via hook dispatch, post-effects)
        let post_fx = powers::registry::dispatch_on_card_played_post(&self.state.player, card.card_type == CardType::Attack);
        if post_fx.block_gain > 0 {
            self.gain_block_player(post_fx.block_gain);
        }

        // Beat of Death: enemies with this power deal damage to player AFTER card played (Java: onAfterUseCard)
        for ei in 0..self.state.enemies.len() {
            if self.state.enemies[ei].is_alive() {
                let bod = powers::get_beat_of_death_damage(&self.state.enemies[ei].entity);
                if bod > 0 {
                    let intangible = self.state.player.status(sid::INTANGIBLE) > 0;
                    let has_torii = self.state.has_relic("Torii");
                    let has_tungsten = self.state.has_relic("Tungsten Rod");
                    let has_odd_mushroom = self.state.has_relic("Odd Mushroom");
                    let result = damage::calculate_incoming_damage(
                        bod,
                        self.state.player.block,
                        self.state.stance == Stance::Wrath,
                        self.state.player.is_vulnerable(),
                        intangible,
                        has_torii,
                        has_tungsten,
                        has_odd_mushroom,
                    );
                    self.state.player.block = result.block_remaining;
                    if result.hp_loss > 0 {
                        self.state.player.hp -= result.hp_loss;
                        self.state.total_damage_taken += result.hp_loss;
                    }
                }
            }
        }

        // Slow: increment counter on enemies with Slow power
        for ei in 0..self.state.enemies.len() {
            if self.state.enemies[ei].is_alive() {
                powers::increment_slow(&mut self.state.enemies[ei].entity);
            }
        }

        // TimeWarp: increment card counter; at 12 end turn + enemy gains Strength
        for ei in 0..self.state.enemies.len() {
            if self.state.enemies[ei].is_alive() {
                let triggered = powers::increment_time_warp(&mut self.state.enemies[ei].entity);
                if triggered {
                    self.state.enemies[ei].entity.add_status(sid::STRENGTH, 2);
                    self.end_turn();
                    return;
                }
            }
        }

        // Panache: every 5 cards played, deal damage to all enemies
        let panache_dmg = powers::check_panache(&mut self.state.player);
        if panache_dmg > 0 {
            let living = self.state.living_enemy_indices();
            for idx in living {
                self.deal_damage_to_enemy(idx, panache_dmg);
            }
        }

        // Consume NextAttackFree after playing an attack
        if card.card_type == CardType::Attack && self.state.player.status(sid::NEXT_ATTACK_FREE) > 0 {
            self.state.player.set_status(sid::NEXT_ATTACK_FREE, 0);
        }

        // EchoForm: replay first N cards played this turn (stacking)
        let echo_count = self.state.player.status(sid::ECHO_FORM);
        if echo_count > 0
            && self.state.cards_played_this_turn <= echo_count
            && card.card_type != CardType::Power
            && !self.state.combat_over
        {
            crate::card_effects::execute_card_effects(self, &card, card_inst, target_idx);
        }

        // Double Tap: replay next Attack (Java: DoubleTapPower.onUseCard)
        if card.card_type == CardType::Attack && !self.state.combat_over {
            let dt = self.state.player.status(sid::DOUBLE_TAP);
            if dt > 0 {
                self.state.player.add_status(sid::DOUBLE_TAP, -1);
                crate::card_effects::execute_card_effects(self, &card, card_inst, target_idx);
            }
        }

        // Burst: replay next Skill (Java: BurstPower.onUseCard)
        if card.card_type == CardType::Skill && !self.state.combat_over {
            let burst = self.state.player.status(sid::BURST);
            if burst > 0 {
                self.state.player.add_status(sid::BURST, -1);
                crate::card_effects::execute_card_effects(self, &card, card_inst, target_idx);
            }
        }

        // Necronomicon: replay first 2+-cost Attack once per turn
        if !self.state.combat_over {
            let is_attack = card.card_type == CardType::Attack;
            if relics::necronomicon_should_trigger(&self.state, card.cost, is_attack) {
                relics::necronomicon_mark_used(&mut self.state);
                crate::card_effects::execute_card_effects(self, &card, card_inst, target_idx);
            }
        }

        // Curiosity: Awakened One gains Strength when player plays a Power
        if card.card_type == CardType::Power {
            for i in 0..self.state.enemies.len() {
                let curiosity = self.state.enemies[i].entity.status(sid::CURIOSITY);
                if curiosity > 0 && self.state.enemies[i].is_alive() {
                    self.state.enemies[i].entity.add_status(sid::STRENGTH, curiosity);
                }
            }
        }

        // SkillBurn (Book of Stabbing): deal damage to player when playing a Skill
        if card.card_type == CardType::Skill {
            for i in 0..self.state.enemies.len() {
                let sb = self.state.enemies[i].entity.status(sid::SKILL_BURN);
                if sb > 0 && self.state.enemies[i].is_alive() {
                    self.player_lose_hp(sb);
                }
            }
        }

        // Forcefield: decrement on enemies after each card play
        for i in 0..self.state.enemies.len() {
            let ff = self.state.enemies[i].entity.status(sid::FORCEFIELD);
            if ff > 0 && self.state.enemies[i].is_alive() {
                self.state.enemies[i].entity.add_status(sid::FORCEFIELD, -1);
            }
        }

        // Wave of the Hand is now handled inside gain_block_player() automatically.

        // Power card: install status effect instead of going to discard
        if card.card_type == CardType::Power {
            self.install_power(&card);
            // Storm: channel Lightning when playing a Power
            if powers::should_storm_channel(&self.state.player) {
                self.channel_orb(crate::orbs::OrbType::Lightning);
            }
            // Heatsink: draw cards when playing a Power
            let heatsink_draw = powers::get_heatsink_draw(&self.state.player);
            if heatsink_draw > 0 {
                self.draw_cards(heatsink_draw);
            }
            // MummifiedHand: when Power card played, random card in hand costs 0 this turn
            // MCTS approximation: reduce energy cost of cheapest card in hand by setting its cost
            // to 0 is not feasible without per-card cost tracking; grant 1 energy instead
            if self.state.player.status(sid::MUMMIFIED_HAND_TRIGGER) > 0 && !self.state.hand.is_empty() {
                self.state.energy += 1;
            }
            // Powers don't go to any pile
        } else if card.effects.contains(&"shuffle_self_into_draw") {
            // Tantrum: goes to draw pile instead of discard
            // (already handled in execute_card_effects, don't double-add)
        } else if card.exhaust
            || (card.card_type == CardType::Skill
                && self.state.player.status(sid::CORRUPTION) > 0)
        {
            // Strange Spoon: 50% chance exhaust -> shuffle into draw pile
            if self.state.has_relic("Strange Spoon") || self.state.has_relic("StrangeSpoon") {
                if self.rng.random(1) == 0 {
                    self.state.draw_pile.push(card_inst);
                } else {
                    self.state.exhaust_pile.push(card_inst);
                    self.trigger_on_exhaust();
                }
            } else {
                self.state.exhaust_pile.push(card_inst);
                self.trigger_on_exhaust();
            }
        } else {
            self.state.discard_pile.push(card_inst);
        }

        // Conclude: end the turn immediately after playing
        // Let end_turn() handle the remaining hand (respects retain/ethereal)
        if card.effects.contains(&"end_turn") {
            self.end_turn();
            return;
        }

        // Unceasing Top: draw when hand is empty
        while relics::unceasing_top_should_draw(&self.state) {
            self.draw_cards(1);
        }

        // Check combat end after card play
        self.check_combat_end();
    }

    /// Install a power card as a permanent status effect.
    /// Uses the unified power registry for tag->StatusId lookup.
    fn install_power(&mut self, card: &CardDef) {
        for effect in card.effects {
            // Registry lookup: covers ~40 powers with the same pattern
            if let Some(entry) = powers::registry::lookup_by_tag(effect) {
                let amt = card.base_magic.max(1);
                self.state.player.add_status(entry.status_id, amt);
                continue;
            }
            // Special cases that need engine context
            match *effect {
                "fasting" => {
                    let amount = card.base_magic.max(1);
                    self.state.player.add_status(sid::STRENGTH, amount);
                    self.state.player.add_status(sid::DEXTERITY, amount);
                    self.state.max_energy -= 1;
                    self.state.energy = self.state.energy.min(self.state.max_energy);
                }
                "master_reality" => {
                    self.state.player.set_status(sid::MASTER_REALITY, 1);
                }
                "gain_orb_slots" => {
                    let slots = card.base_magic.max(1);
                    for _ in 0..slots {
                        self.state.orb_slots.add_slot();
                    }
                }
                _ => {}
            }
        }
    }

    // =======================================================================
    // Potion Use
    // =======================================================================

    fn use_potion(&mut self, potion_idx: usize, target_idx: i32) {
        if potion_idx >= self.state.potions.len() {
            return;
        }
        if self.state.potions[potion_idx].is_empty() {
            return;
        }

        let potion_id = self.state.potions[potion_idx].clone();

        // Apply potion effect
        let success = potions::apply_potion(&mut self.state, &potion_id, target_idx);

        if success {
            // Consume the potion slot
            self.state.potions[potion_idx] = String::new();

            // Toy Ornithopter: heal 5 on potion use
            relics::toy_ornithopter_on_potion(&mut self.state);
        }

        // Check combat end (potions can kill enemies)
        self.check_combat_end();
    }

    // =======================================================================
    // Hook Dispatch: on_card_discarded
    // =======================================================================

    /// Called when a card is manually discarded from hand (card effects, choices).
    /// NOT called for end-of-turn discard (matches real game behavior).
    pub fn on_card_discarded(&mut self, card: CardInstance) {
        // Extract card info before mutable borrows
        let (has_draw, has_energy, magic) = {
            let card_def = self.card_registry.card_def_by_id(card.def_id);
            (
                card_def.effects.contains(&"draw_on_discard"),
                card_def.effects.contains(&"energy_on_discard"),
                card_def.base_magic,
            )
        };

        // Reflex: draw N cards when discarded from hand
        if has_draw {
            self.draw_cards(magic);
        }

        // Tactician: gain N energy when discarded from hand
        if has_energy {
            self.state.energy += magic;
        }

        // Track discard count this turn (for Sneaky Strike, Eviscerate)
        self.state.player.add_status(sid::DISCARDED_THIS_TURN, 1);

        // Relic triggers
        relics::tough_bandages_on_discard(&mut self.state);
        relics::tingsha_on_discard(&mut self.state);
    }

    // =======================================================================
    // Centralized Mutations
    // =======================================================================

    /// Centralized player block gain. Fires Juggernaut and Wave of the Hand reactions.
    /// Callers pass the FINAL computed amount (dexterity/frail already applied).
    pub fn gain_block_player(&mut self, amount: i32) {
        if amount <= 0 {
            return;
        }
        self.state.player.block += amount;

        // Juggernaut: deal damage to first living enemy equal to block gained
        let jugg = self.state.player.status(sid::JUGGERNAUT);
        if jugg > 0 {
            let targets = self.state.targetable_enemy_indices();
            if let Some(&target) = targets.first() {
                self.deal_damage_to_enemy(target, jugg);
            }
        }

        // Wave of the Hand: apply Weak to all enemies when gaining block
        let wave = self.state.player.status(sid::WAVE_OF_THE_HAND);
        if wave > 0 {
            for i in 0..self.state.enemies.len() {
                if self.state.enemies[i].is_targetable() {
                    powers::apply_debuff(&mut self.state.enemies[i].entity, sid::WEAKENED, wave);
                }
            }
        }
    }

    /// Centralized player HP loss (bypasses block). Checks fairy revive, fires on_hp_loss relics,
    /// and triggers Rupture.
    pub fn player_lose_hp(&mut self, amount: i32) {
        if amount <= 0 {
            return;
        }
        self.state.player.hp -= amount;
        self.state.total_damage_taken += amount;

        // Fire on_hp_loss relics (Centennial Puzzle, Self-Forming Clay, Runic Cube, Red Skull, Emotion Chip)
        relics::on_hp_loss(&mut self.state, amount);

        // Rupture: gain Strength when losing HP
        let rupture = self.state.player.status(sid::RUPTURE);
        if rupture > 0 {
            self.state.player.add_status(sid::STRENGTH, rupture);
        }

        // Fairy revive check
        if self.state.player.hp <= 0 {
            self.check_fairy_revive();
        }
    }

    /// Centralized healing: delegates to CombatState::heal_player.
    pub fn heal_player(&mut self, amount: i32) {
        self.state.heal_player(amount);
    }

    /// Check and apply revive effects (Fairy in a Bottle, Lizard Tail).
    fn check_fairy_revive(&mut self) {
        if self.state.player.hp <= 0 {
            // Fairy in a Bottle (potion)
            let revive_hp = potions::check_fairy_revive(&self.state);
            if revive_hp > 0 {
                potions::consume_fairy(&mut self.state);
                self.state.player.hp = revive_hp;
                return;
            }
            // Lizard Tail (relic): revive at 50% max HP, once per run
            if (self.state.has_relic("Lizard Tail") || self.state.has_relic("LizardTail"))
                && self.state.player.status(sid::LIZARD_TAIL_USED) == 0
            {
                self.state.player.set_status(sid::LIZARD_TAIL_USED, 1);
                self.state.player.hp = self.state.player.max_hp / 2;
                return;
            }
            // No revive available
            self.state.player.hp = 0;
            self.state.combat_over = true;
            self.state.player_won = false;
            self.phase = CombatPhase::CombatOver;
            self.choice = None;
        }
    }

    // =======================================================================
    // Damage Dealing / Taking
    // =======================================================================

    pub fn deal_damage_to_enemy(&mut self, enemy_idx: usize, damage: i32) {
        let enemy = &mut self.state.enemies[enemy_idx];

        // Slow: enemies with Slow take 10% more damage per card played this turn
        let slow_mult = powers::slow_damage_multiplier(&enemy.entity);
        let damage_after_slow = (damage as f64 * slow_mult) as i32;

        // Flight: halve incoming damage while Flight > 0, decrement per hit
        let flight = enemy.entity.status(sid::FLIGHT);
        let effective_damage = if flight > 0 {
            enemy.entity.set_status(sid::FLIGHT, flight - 1);
            (damage_after_slow as f64 * 0.5) as i32
        } else {
            damage_after_slow
        };

        // Invincible: cap total HP loss per turn (Heart, Donu, Deca)
        let capped_damage = powers::apply_invincible_cap_tracked(&mut enemy.entity, effective_damage);

        let blocked = enemy.entity.block.min(capped_damage);
        let hp_damage = capped_damage - blocked;
        enemy.entity.block -= blocked;
        enemy.entity.hp -= hp_damage;
        self.state.total_damage_dealt += hp_damage;

        // On-hit enemy reactions (only when HP damage dealt)
        if hp_damage > 0 {
            // Curl-Up: first time hit, enemy gains block
            let curl_up = self.state.enemies[enemy_idx].entity.status(sid::CURL_UP);
            if curl_up > 0 {
                self.state.enemies[enemy_idx].entity.block += curl_up;
                self.state.enemies[enemy_idx].entity.set_status(sid::CURL_UP, 0);
            }

            // Malleable: gain escalating block on hit
            let malleable = self.state.enemies[enemy_idx].entity.status(sid::MALLEABLE);
            if malleable > 0 {
                self.state.enemies[enemy_idx].entity.block += malleable;
                self.state.enemies[enemy_idx].entity.add_status(sid::MALLEABLE, 1);
            }

            // Sharp Hide: deal retaliation damage to player when attacked
            let sharp_hide = self.state.enemies[enemy_idx].entity.status(sid::SHARP_HIDE);
            if sharp_hide > 0 {
                self.state.player.hp -= sharp_hide;
                self.state.total_damage_taken += sharp_hide;
            }

            // Shifting: gain block equal to unblocked damage
            let shifting = self.state.enemies[enemy_idx].entity.status(sid::SHIFTING);
            if shifting > 0 {
                self.state.enemies[enemy_idx].entity.block += hp_damage;
            }
        }

        if self.state.enemies[enemy_idx].entity.hp <= 0 {
            self.state.enemies[enemy_idx].entity.hp = 0;
            // SporeCloud: apply Vulnerable to player on death (Java: SporeCloudPower.onDeath)
            let spore = self.state.enemies[enemy_idx].entity.status(sid::SPORE_CLOUD);
            if spore > 0 {
                powers::apply_debuff(&mut self.state.player, sid::VULNERABLE, spore);
            }
            // Fire on_enemy_death relics (Gremlin Horn, The Specimen)
            relics::on_enemy_death(&mut self.state, enemy_idx);
        }

        // Boss damage hooks
        combat_hooks::on_enemy_damaged(self, enemy_idx, hp_damage);
    }

    #[allow(dead_code)]
    pub fn deal_damage_to_player(&mut self, damage: i32) {
        let player = &mut self.state.player;
        let blocked = player.block.min(damage);
        let hp_damage = damage - blocked;
        player.block -= blocked;

        if hp_damage > 0 {
            self.player_lose_hp(hp_damage);
        }
    }

    // =======================================================================
    // Orb Effects
    // =======================================================================

    /// Pick a random living enemy index using the combat RNG.
    fn random_living_enemy(&mut self) -> Option<usize> {
        let living = self.state.living_enemy_indices();
        if living.is_empty() {
            return None;
        }
        if living.len() == 1 {
            return Some(living[0]);
        }
        let roll = self.rng.random(living.len() as i32 - 1) as usize;
        Some(living[roll])
    }

    /// Pick the living enemy with the lowest HP.
    fn lowest_hp_enemy(&self) -> Option<usize> {
        let living = self.state.living_enemy_indices();
        living
            .iter()
            .copied()
            .min_by_key(|&i| self.state.enemies[i].entity.hp)
    }

    /// Apply a single evoke effect to the game state.
    pub(crate) fn apply_evoke_effect(&mut self, effect: EvokeEffect) {
        match effect {
            EvokeEffect::LightningDamage(dmg) => {
                if let Some(idx) = self.random_living_enemy() {
                    self.deal_damage_to_enemy(idx, dmg);
                }
            }
            EvokeEffect::FrostBlock(block) => {
                self.gain_block_player(block);
            }
            EvokeEffect::DarkDamage(dmg) => {
                if let Some(idx) = self.lowest_hp_enemy() {
                    self.deal_damage_to_enemy(idx, dmg);
                }
            }
            EvokeEffect::PlasmaEnergy(energy) => {
                self.state.energy += energy;
            }
            EvokeEffect::None => {}
        }
    }

    /// Apply a single passive effect to the game state.
    fn apply_passive_effect(&mut self, effect: PassiveEffect) {
        match effect {
            PassiveEffect::LightningDamage(dmg) => {
                if let Some(idx) = self.random_living_enemy() {
                    self.deal_damage_to_enemy(idx, dmg);
                }
            }
            PassiveEffect::FrostBlock(block) => {
                self.gain_block_player(block);
            }
            PassiveEffect::PlasmaEnergy(energy) => {
                self.state.energy += energy;
            }
            PassiveEffect::None => {}
        }
    }

    /// Trigger orb end-of-turn passives and apply their effects.
    fn apply_orb_end_of_turn(&mut self) {
        if !self.state.orb_slots.has_orbs() {
            return;
        }
        let focus = self.state.player.focus();
        let effects = self.state.orb_slots.trigger_end_of_turn_passives(focus);
        for effect in effects {
            self.apply_passive_effect(effect);
            if self.state.player.is_dead() || self.check_combat_end() {
                return;
            }
        }
    }

    /// Trigger orb start-of-turn passives (Plasma) and apply their effects.
    fn apply_orb_start_of_turn(&mut self) {
        if !self.state.orb_slots.has_orbs() {
            return;
        }
        let effects = self.state.orb_slots.trigger_start_of_turn_passives();
        for effect in effects {
            self.apply_passive_effect(effect);
        }
    }

    // =======================================================================
    // Draw / Shuffle
    // =======================================================================

    pub fn draw_cards(&mut self, count: i32) {
        // NoDraw: skip draw entirely
        if self.state.player.status(sid::NO_DRAW) > 0 {
            return;
        }

        // DrawReduction: reduce draw count
        let draw_reduction = self.state.player.status(sid::DRAW_REDUCTION);
        let actual_count = (count - draw_reduction).max(0);

        let mut extra_draws = 0i32;

        for _ in 0..actual_count {
            if self.state.hand.len() >= 10 {
                break; // Hand size limit
            }

            if self.state.draw_pile.is_empty() {
                // Shuffle discard into draw
                if self.state.discard_pile.is_empty() {
                    break; // No cards left anywhere
                }
                let mut shuffled = std::mem::take(&mut self.state.discard_pile);
                shuffled.shuffle(&mut self.rng);
                self.state.draw_pile = shuffled;
                // Fire on_shuffle relics (Sundial, The Abacus)
                relics::on_shuffle(&mut self.state);
            }

            if let Some(drawn) = self.state.draw_pile.pop() {
                self.state.hand.push(drawn);

                // Extract card info before mutable borrows
                let (card_type, has_void, has_copy) = {
                    let card_def = self.card_registry.card_def_by_id(drawn.def_id);
                    (
                        card_def.card_type,
                        card_def.effects.contains(&"lose_energy_on_draw"),
                        card_def.effects.contains(&"copy_on_draw"),
                    )
                };

                // Evolve: draw extra cards when drawing a Status
                let evolve = self.state.player.status(sid::EVOLVE);
                if evolve > 0 && card_type == CardType::Status {
                    extra_draws += evolve;
                }

                // Fire Breathing: damage all enemies when drawing Status or Curse
                let fire_breathing = self.state.player.status(sid::FIRE_BREATHING);
                if fire_breathing > 0 && (card_type == CardType::Status || card_type == CardType::Curse) {
                    for i in 0..self.state.enemies.len() {
                        if self.state.enemies[i].is_targetable() {
                            self.deal_damage_to_enemy(i, fire_breathing);
                        }
                    }
                }

                // Void: lose 1 energy when drawn
                if has_void {
                    self.state.energy = (self.state.energy - 1).max(0);
                }

                // Endless Agony: add a copy to hand when drawn
                if has_copy && self.state.hand.len() < 10 {
                    self.state.hand.push(drawn);
                }
            }
        }

        // Evolve: draw accumulated extra cards (recursive call handles further triggers)
        if extra_draws > 0 {
            self.draw_cards(extra_draws);
        }
    }

    /// Shuffle the draw pile (pub(crate) for card_effects).
    pub(crate) fn shuffle_draw_pile(&mut self) {
        self.state.draw_pile.shuffle(&mut self.rng);
    }

    /// Generate a random range using the engine RNG (pub(crate) for card_effects).
    pub(crate) fn rng_gen_range(&mut self, range: std::ops::Range<usize>) -> usize {
        use rand::Rng;
        self.rng.gen_range(range)
    }

    // =======================================================================
    // Stance
    // =======================================================================

    pub fn change_stance(&mut self, new_stance: Stance) {
        let old_stance = self.state.stance;
        if old_stance == new_stance {
            return;
        }

        // Exit Calm: gain 2 energy (+ Violet Lotus bonus)
        if old_stance == Stance::Calm {
            let bonus = relics::violet_lotus_calm_exit_bonus(&self.state);
            self.state.energy += 2 + bonus;
        }

        // Enter Divinity: gain 3 energy
        if new_stance == Stance::Divinity {
            self.state.energy += 3;
        }

        self.state.stance = new_stance;

        // -- Power triggers on stance change (via hook dispatch) --
        let entering_wrath = new_stance == Stance::Wrath;
        let sfx = powers::registry::dispatch_on_stance_change(&self.state.player, entering_wrath);
        if sfx.block_gain > 0 {
            self.gain_block_player(sfx.block_gain);
        }
        if sfx.draw > 0 {
            self.draw_cards(sfx.draw);
        }
    }

    /// Gain mantra and check for Divinity entry (10+ mantra).
    pub fn gain_mantra(&mut self, amount: i32) {
        self.state.mantra += amount;
        self.state.mantra_gained += amount;
        if self.state.mantra >= 10 {
            self.state.mantra -= 10;
            self.change_stance(Stance::Divinity);
        }
    }

    // =======================================================================
    // Helpers: Temp Card Creation (respects Master Reality)
    // =======================================================================

    /// Get a CardInstance for a temporary card, upgrading if Master Reality is active.
    pub fn temp_card(&self, base_id: &str) -> CardInstance {
        if self.state.player.status(sid::MASTER_REALITY) > 0 {
            self.card_registry.make_card(&format!("{}+", base_id))
        } else {
            self.card_registry.make_card(base_id)
        }
    }

    /// Perform Scry: reveal top N cards, let player choose which to discard.
    /// Triggers Nirvana (on_scry block) and Weave (return_on_scry) in resolve_scry.
    pub fn do_scry(&mut self, amount: i32) {
        let to_scry = (amount as usize).min(self.state.draw_pile.len());
        if to_scry == 0 {
            return;
        }
        // Take top N cards from draw pile (end = top) and present as choice
        let revealed: Vec<CardInstance> = self.state.draw_pile
            .drain(self.state.draw_pile.len() - to_scry..)
            .collect();
        let options: Vec<ChoiceOption> = revealed.into_iter()
            .map(ChoiceOption::RevealedCard)
            .collect();
        // Multi-select: player picks any subset to discard (min 0 = can keep all)
        self.begin_choice(ChoiceReason::Scry, options, 0, to_scry);
    }

    // =======================================================================
    // Exhaust Hooks
    // =======================================================================

    /// Trigger all on-exhaust power and relic hooks.
    pub fn trigger_on_exhaust(&mut self) {
        // Power hooks via dispatch (FeelNoPain block, DarkEmbrace draw)
        let efx = powers::registry::dispatch_on_exhaust(&self.state.player);
        if efx.block_gain > 0 {
            self.gain_block_player(efx.block_gain);
        }
        if efx.draw > 0 {
            self.draw_cards(efx.draw);
        }

        // Charon's Ashes (relic): deal 3 damage to all enemies on exhaust
        relics::charons_ashes_on_exhaust(&mut self.state);

        // Dead Branch (relic): add a random card to hand on exhaust
        if relics::dead_branch_on_exhaust(&self.state) {
            let temp = self.temp_card("Strike");
            if self.state.hand.len() < 10 {
                self.state.hand.push(temp);
            }
        }
    }

    // =======================================================================
    // Orb Channel / Evoke (public API for card_effects)
    // =======================================================================

    /// Channel an orb. If slots are full, evokes the front orb first.
    pub fn channel_orb(&mut self, orb_type: crate::orbs::OrbType) {
        let focus = self.state.player.focus();
        let evoke_effect = self.state.orb_slots.channel(orb_type, focus);
        self.apply_evoke_effect(evoke_effect);
    }

    /// Evoke the front orb.
    pub fn evoke_front_orb(&mut self) {
        let focus = self.state.player.focus();
        let effect = self.state.orb_slots.evoke_front(focus);
        self.apply_evoke_effect(effect);
    }

    /// Evoke the front orb N times.
    pub fn evoke_front_orb_n(&mut self, n: usize) {
        let focus = self.state.player.focus();
        let effects = self.state.orb_slots.evoke_front_n(n, focus);
        for effect in effects {
            self.apply_evoke_effect(effect);
            if self.state.combat_over {
                return;
            }
        }
    }

    /// Evoke all orbs.
    pub fn evoke_all_orbs(&mut self) {
        let focus = self.state.player.focus();
        let effects = self.state.orb_slots.evoke_all(focus);
        for effect in effects {
            self.apply_evoke_effect(effect);
            if self.state.combat_over {
                return;
            }
        }
    }

    /// Initialize Defect orb slots (3 by default).
    pub fn init_defect_orbs(&mut self, num_slots: usize) {
        self.state.orb_slots = crate::orbs::OrbSlots::new(num_slots);
    }

    // =======================================================================
    // Combat End Check
    // =======================================================================

    pub fn check_combat_end(&mut self) -> bool {
        if self.state.combat_over {
            return true;
        }

        // Victory: all enemies dead
        if self.state.is_victory() {
            self.state.combat_over = true;
            self.state.player_won = true;
            self.phase = CombatPhase::CombatOver;
            self.choice = None;
            // Fire on_victory relics (Burning Blood, Black Blood, Meat on the Bone, Face of Cleric)
            let heal = relics::on_victory(&mut self.state);
            if heal > 0 {
                self.heal_player(heal);
            }
            return true;
        }

        // Defeat: player dead
        if self.state.is_defeat() {
            self.state.combat_over = true;
            self.state.player_won = false;
            self.phase = CombatPhase::CombatOver;
            self.choice = None;
            return true;
        }

        false
    }
}

// ===========================================================================
// PyO3 Bindings — RustCombatEngine exposed to Python
// ===========================================================================

#[pyclass(name = "RustCombatEngine")]
pub struct RustCombatEngine {
    engine: CombatEngine,
}

#[pymethods]
impl RustCombatEngine {
    /// Create a new Rust combat engine.
    ///
    /// Args:
    ///     player_hp: Player's current HP
    ///     player_max_hp: Player's maximum HP
    ///     energy: Starting energy per turn
    ///     deck: List of card IDs (strings)
    ///     enemies: List of (id, hp, max_hp, move_damage, move_hits) tuples
    ///     seed: RNG seed for shuffling
    ///     relics: Optional list of relic IDs
    #[new]
    #[pyo3(signature = (player_hp, player_max_hp, energy, deck, enemies, seed=42, relics=None))]
    fn new_py(
        player_hp: i32,
        player_max_hp: i32,
        energy: i32,
        deck: Vec<String>,
        enemies: Vec<(String, i32, i32, i32, i32)>,
        seed: u64,
        relics: Option<Vec<String>>,
    ) -> Self {
        let enemy_states: Vec<EnemyCombatState> = enemies
            .into_iter()
            .map(|(id, hp, max_hp, move_damage, move_hits)| {
                let mut e = EnemyCombatState::new(&id, hp, max_hp);
                e.set_move(1, move_damage, move_hits, 0);
                e
            })
            .collect();

        let registry = CardRegistry::new();
        let deck_instances: Vec<CardInstance> = deck.iter()
            .map(|name| registry.make_card(name))
            .collect();
        let mut state = CombatState::new(player_hp, player_max_hp, enemy_states, deck_instances, energy);
        if let Some(r) = relics {
            state.relics = r;
        }

        RustCombatEngine {
            engine: CombatEngine::new(state, seed),
        }
    }

    /// Start combat (shuffle + draw initial hand).
    fn start_combat(&mut self) {
        self.engine.start_combat();
    }

    /// Get legal actions as a list of Action objects.
    fn get_legal_actions(&self) -> Vec<PyAction> {
        self.engine
            .get_legal_actions()
            .into_iter()
            .map(|a| PyAction { inner: a })
            .collect()
    }

    /// Execute an action.
    fn take_action(&mut self, action: &PyAction) {
        self.engine.execute_action(&action.inner);
    }

    /// Check if combat is over.
    fn is_combat_over(&self) -> bool {
        self.engine.is_combat_over()
    }

    /// Check if player won.
    fn is_victory(&self) -> bool {
        self.engine.state.player_won
    }

    /// Get the current combat state as a Python CombatState object.
    fn get_state(&self) -> PyCombatState {
        PyCombatState {
            inner: self.engine.state.clone(),
        }
    }

    /// Get player HP.
    #[getter]
    fn player_hp(&self) -> i32 {
        self.engine.state.player.hp
    }

    /// Get player block.
    #[getter]
    fn player_block(&self) -> i32 {
        self.engine.state.player.block
    }

    /// Get current energy.
    #[getter]
    fn energy(&self) -> i32 {
        self.engine.state.energy
    }

    /// Get current turn number.
    #[getter]
    fn turn(&self) -> i32 {
        self.engine.state.turn
    }

    /// Get the current hand as list of card IDs.
    #[getter]
    fn hand(&self) -> Vec<String> {
        self.engine.state.hand.iter()
            .map(|c| self.engine.card_registry.card_name(c.def_id).to_string())
            .collect()
    }

    /// Get stance as string.
    #[getter]
    fn stance(&self) -> &str {
        self.engine.state.stance.as_str()
    }

    /// Set an enemy's next move (for external AI control).
    fn set_enemy_move(
        &mut self,
        enemy_idx: usize,
        move_id: i32,
        damage: i32,
        hits: i32,
        block: i32,
    ) {
        if enemy_idx < self.engine.state.enemies.len() {
            self.engine.state.enemies[enemy_idx].set_move(move_id, damage, hits, block);
        }
    }

    /// Set an enemy move effect (e.g., "weak" -> 2).
    fn set_enemy_move_effect(&mut self, enemy_idx: usize, effect: &str, amount: i32) {
        use crate::combat_types::mfx;
        let eid = match effect {
            "weak" => mfx::WEAK,
            "vulnerable" => mfx::VULNERABLE,
            "frail" => mfx::FRAIL,
            "strength" => mfx::STRENGTH,
            "ritual" => mfx::RITUAL,
            "entangle" => mfx::ENTANGLE,
            "slimed" => mfx::SLIMED,
            "daze" => mfx::DAZE,
            "burn" => mfx::BURN,
            "burn_upgrade" => mfx::BURN_UPGRADE,
            "siphon_str" => mfx::SIPHON_STR,
            "siphon_dex" => mfx::SIPHON_DEX,
            "remove_debuffs" => mfx::REMOVE_DEBUFFS,
            "heal_to_half" => mfx::HEAL_TO_HALF,
            "heal" => mfx::HEAL,
            "artifact" => mfx::ARTIFACT,
            "confused" => mfx::CONFUSED,
            "constrict" => mfx::CONSTRICT,
            "dexterity_down" | "dex_down" => mfx::DEX_DOWN,
            "draw_reduction" => mfx::DRAW_REDUCTION,
            "hex" => mfx::HEX,
            "painful_stabs" => mfx::PAINFUL_STABS,
            "stasis" => mfx::STASIS,
            "strength_bonus" => mfx::STRENGTH_BONUS,
            "strength_down" => mfx::STRENGTH_DOWN,
            "thorns" => mfx::THORNS,
            "void" => mfx::VOID,
            "wound" => mfx::WOUND,
            "beat_of_death" => mfx::BEAT_OF_DEATH,
            "poison" => mfx::POISON,
            _ => return,
        };
        if enemy_idx < self.engine.state.enemies.len() {
            self.engine.state.enemies[enemy_idx].add_effect(eid, amount as i16);
        }
    }

    /// Deep clone for MCTS (returns a new independent engine).
    fn clone_for_mcts(&self) -> RustCombatEngine {
        RustCombatEngine {
            engine: self.engine.clone_state(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "RustCombatEngine(hp={}/{}, energy={}, turn={}, hand={}, enemies={}, over={})",
            self.engine.state.player.hp,
            self.engine.state.player.max_hp,
            self.engine.state.energy,
            self.engine.state.turn,
            self.engine.state.hand.len(),
            self.engine.state.enemies.len(),
            self.engine.state.combat_over,
        )
    }
}

// ===========================================================================
// Rust-only tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::support::make_deck;

    fn make_test_state() -> CombatState {
        let deck = make_deck(&["Strike_P", "Strike_P", "Strike_P", "Strike_P", "Defend_P", "Defend_P", "Defend_P", "Defend_P", "Eruption", "Vigilance"]);

        let mut enemy = EnemyCombatState::new("JawWorm", 44, 44);
        enemy.set_move(1, 11, 1, 0); // 11 damage attack

        CombatState::new(80, 80, vec![enemy], deck, 3)
    }

    #[test]
    fn test_start_combat_draws_hand() {
        let state = make_test_state();
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        assert_eq!(engine.state.hand.len(), 5);
        assert_eq!(engine.state.draw_pile.len(), 5);
        assert_eq!(engine.state.turn, 1);
        assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    }

    #[test]
    fn test_legal_actions_include_end_turn() {
        let state = make_test_state();
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let actions = engine.get_legal_actions();
        assert!(actions.contains(&Action::EndTurn));
        assert!(!actions.is_empty());
    }

    #[test]
    fn test_play_strike_deals_damage() {
        let state = make_test_state();
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;

        // Find a Strike in hand and play it
        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id).starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });

        // Strike deals 6 damage
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 6);
        assert_eq!(engine.state.energy, 2); // Spent 1 energy
        assert_eq!(engine.state.hand.len(), 4); // Card removed from hand
    }

    #[test]
    fn test_play_defend_gives_block() {
        let state = make_test_state();
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let defend_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id).starts_with("Defend"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: defend_idx,
            target_idx: -1,
        });

        // Defend gives 5 block
        assert_eq!(engine.state.player.block, 5);
    }

    #[test]
    fn test_eruption_enters_wrath() {
        let mut state = make_test_state();
        // Ensure Eruption is in the deck and will be drawn
        state.draw_pile = make_deck(&["Eruption", "Strike_P", "Strike_P", "Strike_P", "Strike_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let eruption_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id) == "Eruption")
            .unwrap();

        engine.execute_action(&Action::PlayCard {
            card_idx: eruption_idx,
            target_idx: 0,
        });

        assert_eq!(engine.state.stance, Stance::Wrath);
    }

    #[test]
    fn test_vigilance_enters_calm() {
        let mut state = make_test_state();
        state.draw_pile = make_deck(&["Vigilance", "Strike_P", "Strike_P", "Strike_P", "Strike_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let vig_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id) == "Vigilance")
            .unwrap();

        engine.execute_action(&Action::PlayCard {
            card_idx: vig_idx,
            target_idx: -1,
        });

        assert_eq!(engine.state.stance, Stance::Calm);
    }

    #[test]
    fn test_calm_exit_grants_energy() {
        let mut state = make_test_state();
        state.stance = Stance::Calm;
        state.draw_pile = make_deck(&["Eruption", "Strike_P", "Strike_P", "Strike_P", "Strike_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_energy = engine.state.energy; // Should be 3

        let eruption_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id) == "Eruption")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: eruption_idx,
            target_idx: 0,
        });

        // Eruption costs 2, but exiting Calm grants +2, net 0 change
        assert_eq!(engine.state.energy, initial_energy - 2 + 2);
        assert_eq!(engine.state.stance, Stance::Wrath);
    }

    #[test]
    fn test_end_turn_enemy_attacks() {
        let state = make_test_state();
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.player.hp;

        engine.execute_action(&Action::EndTurn);

        // Enemy attacks for 11 damage (Jaw Worm)
        assert_eq!(engine.state.player.hp, initial_hp - 11);
        assert_eq!(engine.state.turn, 2); // New turn started
    }

    #[test]
    fn test_block_absorbs_damage() {
        let state = make_test_state();
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Play Defend first
        let defend_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id).starts_with("Defend"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: defend_idx,
            target_idx: -1,
        });

        assert_eq!(engine.state.player.block, 5);
        let initial_hp = engine.state.player.hp;

        engine.execute_action(&Action::EndTurn);

        // 11 damage - 5 block = 6 HP lost
        assert_eq!(engine.state.player.hp, initial_hp - 6);
    }

    #[test]
    fn test_enemy_death_ends_combat() {
        let mut state = make_test_state();
        state.enemies[0].entity.hp = 5; // Low HP enemy

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id).starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });

        // 6 damage kills 5 HP enemy
        assert!(engine.state.combat_over);
        assert!(engine.state.player_won);
    }

    #[test]
    fn test_player_death_ends_combat() {
        let mut state = make_test_state();
        state.enemies[0].set_move(1, 100, 1, 0); // Lethal damage

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        engine.execute_action(&Action::EndTurn);

        assert!(engine.state.combat_over);
        assert!(!engine.state.player_won);
        assert_eq!(engine.state.player.hp, 0);
    }

    #[test]
    fn test_wrath_doubles_outgoing_damage() {
        let mut state = make_test_state();
        state.stance = Stance::Wrath;

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;

        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id).starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });

        // Divinity auto-exits at turn start, so stance is Neutral.
        // But we set Wrath which doesn't auto-exit.
        // Wait -- Divinity auto-exits, Wrath does not. Let me check.
        // start_player_turn only exits Divinity, not Wrath. Good.
        // Strike in Wrath: 6 * 2.0 = 12
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 12);
    }

    #[test]
    fn test_wrath_doubles_incoming_damage() {
        let mut state = make_test_state();
        state.stance = Stance::Wrath;

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Wrath doesn't auto-exit at turn start (only Divinity does)
        assert_eq!(engine.state.stance, Stance::Wrath);

        let initial_hp = engine.state.player.hp;
        engine.execute_action(&Action::EndTurn);

        // Enemy: 11 * 2.0 (Wrath incoming) = 22 damage
        assert_eq!(engine.state.player.hp, initial_hp - 22);
    }

    #[test]
    fn test_shuffle_on_empty_draw() {
        let mut state = make_test_state();
        state.draw_pile = make_deck(&["Strike_P"]); // Only 1 card
        state.discard_pile = make_deck(&["Defend_P", "Defend_P", "Defend_P", "Defend_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Should draw 1 from draw, then shuffle 4 from discard, draw 4 more
        assert_eq!(engine.state.hand.len(), 5);
        assert!(engine.state.discard_pile.is_empty());
    }

    #[test]
    fn test_vulnerability_increases_damage() {
        let mut state = make_test_state();
        state.enemies[0].entity.set_status(sid::VULNERABLE, 2);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;

        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id).starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });

        // 6 * 1.5 = 9
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 9);
    }

    #[test]
    fn test_strength_adds_damage() {
        let mut state = make_test_state();
        state.player.set_status(sid::STRENGTH, 3);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;

        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id).starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });

        // 6 + 3 = 9
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 9);
    }

    #[test]
    fn test_debuff_decrement_on_end_round() {
        let mut state = make_test_state();
        state.player.set_status(sid::WEAKENED, 2);
        state.player.set_status(sid::VULNERABLE, 1);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();
        engine.execute_action(&Action::EndTurn);

        // After one round: Weakened 2->1, Vulnerable 1->removed
        assert_eq!(engine.state.player.status(sid::WEAKENED), 1);
        assert_eq!(engine.state.player.status(sid::VULNERABLE), 0);
    }

    #[test]
    fn test_poison_ticks_on_enemies() {
        let mut state = make_test_state();
        state.enemies[0].entity.set_status(sid::POISON, 5);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;
        engine.execute_action(&Action::EndTurn);

        // Poison deals 5 HP to enemy, then decrements to 4.
        // Enemy also attacks player for 11, but that doesn't affect enemy HP.
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 5);
        assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 4);
    }

    #[test]
    fn test_no_actions_when_combat_over() {
        let mut state = make_test_state();
        state.combat_over = true;

        let engine = CombatEngine::new(state, 42);
        let actions = engine.get_legal_actions();
        assert!(actions.is_empty());
    }

    // =================================================================
    // Phase A: Power trigger tests
    // =================================================================

    #[test]
    fn test_rushdown_draws_on_wrath_entry() {
        let mut state = make_test_state();
        // Give player Rushdown power (draw 2 on Wrath entry)
        state.player.set_status(sid::RUSHDOWN, 2);
        state.draw_pile = make_deck(&["Eruption", "Strike_P", "Strike_P", "Strike_P", "Defend_P", "Defend_P", "Defend_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Ensure Eruption is in hand (RNG may not have drawn it)
        if !engine.state.hand.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Eruption") {
            engine.state.hand.push(engine.card_registry.make_card("Eruption"));
        }

        let hand_size_before = engine.state.hand.len();

        // Find and play Eruption (enters Wrath)
        let eruption_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id) == "Eruption")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: eruption_idx,
            target_idx: 0,
        });

        assert_eq!(engine.state.stance, Stance::Wrath);
        // Rushdown drew 2 cards: hand was (N-1) after playing Eruption, now N-1+2
        assert_eq!(engine.state.hand.len(), hand_size_before - 1 + 2);
    }

    #[test]
    fn test_mental_fortress_blocks_on_stance_change() {
        let mut state = make_test_state();
        // Give player MentalFortress power (4 block on stance change)
        state.player.set_status(sid::MENTAL_FORTRESS, 4);
        state.draw_pile = make_deck(&["Eruption", "Strike_P", "Strike_P", "Strike_P", "Defend_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        assert_eq!(engine.state.player.block, 0);

        // Play Eruption -> enters Wrath, MentalFortress triggers
        let eruption_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id) == "Eruption")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: eruption_idx,
            target_idx: 0,
        });

        assert_eq!(engine.state.stance, Stance::Wrath);
        assert_eq!(engine.state.player.block, 4); // MentalFortress block
    }

    #[test]
    fn test_mantra_accumulation_to_divinity() {
        let mut state = make_test_state();
        state.draw_pile = make_deck(&["Prostrate", "Prostrate", "Prostrate", "Prostrate", "Prostrate", "Strike_P", "Strike_P", "Strike_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        assert_eq!(engine.state.mantra, 0);
        assert_eq!(engine.state.stance, Stance::Neutral);

        // Play Prostrate cards to accumulate mantra
        // Each gives 2 mantra (base_magic=2)
        for i in 0..5 {
            if let Some(idx) = engine
                .state
                .hand
                .iter()
                .position(|c| engine.card_registry.card_name(c.def_id) == "Prostrate")
            {
                engine.execute_action(&Action::PlayCard {
                    card_idx: idx,
                    target_idx: -1,
                });

                if i < 4 {
                    // After 4 plays: 8 mantra, not yet Divinity
                    assert_ne!(engine.state.stance, Stance::Divinity);
                } else {
                    // After 5 plays: 10 mantra -> Divinity!
                    assert_eq!(engine.state.stance, Stance::Divinity);
                    assert_eq!(engine.state.mantra, 0); // Reset after entering Divinity
                }
            }
        }
    }

    #[test]
    fn test_violet_lotus_extra_energy_on_calm_exit() {
        let mut state = make_test_state();
        state.stance = Stance::Calm;
        state.relics.push("Violet Lotus".to_string());
        state.draw_pile = make_deck(&["Eruption", "Strike_P", "Strike_P", "Strike_P", "Strike_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_energy = engine.state.energy; // 3

        let eruption_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id) == "Eruption")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: eruption_idx,
            target_idx: 0,
        });

        // Eruption costs 2, Calm exit gives +2, Violet Lotus gives +1, net +1
        assert_eq!(engine.state.energy, initial_energy - 2 + 2 + 1);
    }

    #[test]
    fn test_divinity_auto_exit_gives_energy() {
        let mut state = make_test_state();
        // Pre-set mantra to 5 so only one Worship needed to enter Divinity
        state.mantra = 5;
        state.draw_pile = make_deck(&["Worship", "Strike_P", "Strike_P", "Strike_P", "Strike_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let energy_before = engine.state.energy; // 3

        // Play Worship: 5 + 5 = 10 mantra -> Divinity + 3 energy
        if let Some(idx) = engine.state.hand.iter().position(|c| engine.card_registry.card_name(c.def_id) == "Worship") {
            engine.execute_action(&Action::PlayCard {
                card_idx: idx,
                target_idx: -1,
            });
        }
        assert_eq!(engine.state.stance, Stance::Divinity);
        assert_eq!(engine.state.mantra, 0);
        // Energy: 3 base - 2 Worship cost + 3 Divinity bonus = 4
        assert_eq!(engine.state.energy, energy_before - 2 + 3);

        // End turn + start new turn: Divinity auto-exits to Neutral
        engine.execute_action(&Action::EndTurn);
        assert_eq!(engine.state.stance, Stance::Neutral);
    }

    #[test]
    fn test_potion_use_fire_potion() {
        let mut state = make_test_state();
        state.potions[0] = "Fire Potion".to_string();

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;

        engine.execute_action(&Action::UsePotion {
            potion_idx: 0,
            target_idx: 0,
        });

        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 20);
        assert!(engine.state.potions[0].is_empty()); // Consumed
    }

    #[test]
    fn test_potion_use_energy_potion() {
        let mut state = make_test_state();
        state.potions[0] = "Energy Potion".to_string();

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_energy = engine.state.energy;

        engine.execute_action(&Action::UsePotion {
            potion_idx: 0,
            target_idx: -1,
        });

        assert_eq!(engine.state.energy, initial_energy + 2);
    }

    #[test]
    fn test_relic_vajra_at_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Vajra".to_string());

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        assert_eq!(engine.state.player.strength(), 1);

        // Strike should now deal 7 damage (6 + 1 Str)
        let initial_hp = engine.state.enemies[0].entity.hp;
        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id).starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 7);
    }

    #[test]
    fn test_relic_lantern_first_turn_energy() {
        let mut state = make_test_state();
        state.relics.push("Lantern".to_string());

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Turn 1: should have 3 + 1 = 4 energy
        assert_eq!(engine.state.energy, 4);
    }

    #[test]
    fn test_fairy_auto_revive() {
        let mut state = make_test_state();
        state.enemies[0].set_move(1, 200, 1, 0); // Lethal damage
        state.potions[0] = "FairyPotion".to_string();

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        engine.execute_action(&Action::EndTurn);

        // Fairy should revive at 30% of 80 = 24 HP
        assert_eq!(engine.state.player.hp, 24);
        assert!(!engine.state.combat_over);
        assert!(engine.state.potions[0].is_empty()); // Consumed
    }

    #[test]
    fn test_enemy_slimed_cards_to_discard() {
        let mut state = make_test_state();
        state.enemies[0].set_move(1, 0, 0, 0);
        state.enemies[0].add_effect(crate::combat_types::mfx::SLIMED, 3);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        engine.execute_action(&Action::EndTurn);

        // 5 cards from hand + 3 Slimed cards from enemy
        let slimed_count = engine
            .state
            .discard_pile
            .iter()
            .filter(|c| engine.card_registry.card_name(c.def_id) == "Slimed")
            .count();
        assert_eq!(slimed_count, 3);
    }

    #[test]
    fn test_entangle_prevents_attacks() {
        let mut state = make_test_state();
        state.player.set_status(sid::ENTANGLED, 1);
        state.draw_pile = make_deck(&["Strike_P", "Strike_P", "Strike_P", "Defend_P", "Defend_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let actions = engine.get_legal_actions();
        // Should NOT contain any Strike plays (attacks blocked by Entangle)
        let attack_actions: Vec<_> = actions
            .iter()
            .filter(|a| {
                if let Action::PlayCard { card_idx, .. } = a {
                    engine.card_registry.card_name(engine.state.hand[*card_idx].def_id).starts_with("Strike")
                } else {
                    false
                }
            })
            .collect();
        assert!(attack_actions.is_empty());

        // But Defend should still be playable
        let defend_actions: Vec<_> = actions
            .iter()
            .filter(|a| {
                if let Action::PlayCard { card_idx, .. } = a {
                    engine.card_registry.card_name(engine.state.hand[*card_idx].def_id).starts_with("Defend")
                } else {
                    false
                }
            })
            .collect();
        assert!(!defend_actions.is_empty());
    }

    #[test]
    fn test_miracle_gives_energy() {
        let mut state = make_test_state();
        state.draw_pile = make_deck(&["Miracle", "Strike_P", "Strike_P", "Strike_P", "Strike_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_energy = engine.state.energy;
        let miracle_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id) == "Miracle")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: miracle_idx,
            target_idx: -1,
        });

        // Miracle costs 0, gives 1 energy
        assert_eq!(engine.state.energy, initial_energy + 1);
        // Miracle exhausts
        assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Miracle"));
    }

    #[test]
    fn test_inner_peace_in_calm_draws() {
        let mut state = make_test_state();
        state.stance = Stance::Calm;
        state.draw_pile = make_deck(&["InnerPeace", "Strike_P", "Strike_P", "Strike_P", "Strike_P", "Defend_P", "Defend_P", "Defend_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Ensure InnerPeace is in hand (RNG may not have drawn it)
        if !engine.state.hand.iter().any(|c| engine.card_registry.card_name(c.def_id) == "InnerPeace") {
            engine.state.hand.push(engine.card_registry.make_card("InnerPeace"));
        }

        let hand_before = engine.state.hand.len();

        let ip_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id) == "InnerPeace")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: ip_idx,
            target_idx: -1,
        });

        // In Calm: draws 3 (base_magic=3). Hand was (N-1) after playing, now N-1+3
        assert_eq!(engine.state.hand.len(), hand_before - 1 + 3);
        // Stays in Calm (doesn't change stance when drawing)
        assert_eq!(engine.state.stance, Stance::Calm);
    }

    #[test]
    fn test_inner_peace_not_calm_enters_calm() {
        let mut state = make_test_state();
        state.stance = Stance::Neutral;
        state.draw_pile = make_deck(&["InnerPeace", "Strike_P", "Strike_P", "Strike_P", "Strike_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let hand_before = engine.state.hand.len();

        let ip_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id) == "InnerPeace")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: ip_idx,
            target_idx: -1,
        });

        // Not in Calm: enters Calm, no draw
        assert_eq!(engine.state.stance, Stance::Calm);
        assert_eq!(engine.state.hand.len(), hand_before - 1); // Only removed the played card
    }

    #[test]
    fn test_power_card_not_in_discard() {
        let mut state = make_test_state();
        state.draw_pile = make_deck(&["MentalFortress", "Strike_P", "Strike_P", "Strike_P", "Strike_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let mf_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id) == "MentalFortress")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: mf_idx,
            target_idx: -1,
        });

        // Power card should NOT be in discard pile
        assert!(!engine
            .state
            .discard_pile
            .iter().any(|c| engine.card_registry.card_name(c.def_id) == "MentalFortress"));
        // MentalFortress status installed
        assert_eq!(engine.state.player.status(sid::MENTAL_FORTRESS), 4);
    }

    #[test]
    fn test_vigor_consumed_on_attack() {
        let mut state = make_test_state();
        state.player.set_status(sid::VIGOR, 8); // Akabeko

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;

        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id).starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });

        // Strike deals 6 + 8 vigor = 14 damage
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 14);
        // Vigor consumed
        assert_eq!(engine.state.player.status(sid::VIGOR), 0);
    }

    #[test]
    fn test_enemy_advances_moves_each_turn() {
        let state = make_test_state();
        // Jaw Worm starts with Chomp (11 damage)
        assert_eq!(state.enemies[0].move_id, 1); // CHOMP

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        engine.execute_action(&Action::EndTurn);

        // After first enemy turn, enemy should have rolled next move
        // Jaw Worm after Chomp -> Bellow (no damage, gains strength)
        assert_ne!(engine.state.enemies[0].move_id, -1);
        // The move should have changed from Chomp
        assert!(engine.state.enemies[0].move_history.contains(&1));
    }

    #[test]
    fn test_targeted_potion_in_legal_actions() {
        let mut state = make_test_state();
        state.potions[0] = "Fire Potion".to_string();

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let actions = engine.get_legal_actions();
        let potion_actions: Vec<_> = actions
            .iter()
            .filter(|a| matches!(a, Action::UsePotion { .. }))
            .collect();

        // Fire Potion requires target, so should have 1 action per living enemy
        assert_eq!(potion_actions.len(), 1); // One enemy alive
        if let Action::UsePotion { target_idx, .. } = potion_actions[0] {
            assert_eq!(*target_idx, 0); // Targets enemy 0
        }
    }

    #[test]
    fn test_halt_extra_block_in_wrath() {
        let mut state = make_test_state();
        state.stance = Stance::Wrath;
        state.draw_pile = make_deck(&["Halt", "Strike_P", "Strike_P", "Strike_P", "Strike_P"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let halt_idx = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id) == "Halt")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: halt_idx,
            target_idx: -1,
        });

        // Halt: 3 base block + 9 extra in Wrath = 12 total
        assert_eq!(engine.state.player.block, 12);
    }
}
