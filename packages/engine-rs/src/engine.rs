//! Combat engine — slim orchestrator for MCTS simulations.
//!
//! Core turn loop that delegates to:
//! - card_effects: card play effect execution
//! - status_effects: end-of-turn hand card triggers
//! - combat_hooks: enemy turns, boss damage hooks

use pyo3::prelude::*;
use pyo3::types::PyDict;
use rand::seq::SliceRandom;

use crate::actions::{Action, PyAction};
use crate::cards::{CardDef, CardRegistry, CardTarget, CardType};
use crate::combat_hooks;
use crate::combat_types::CardInstance;
use crate::damage;
use crate::effects;
use crate::effects::declarative::{
    AmountSource, CardFilter, ChoiceAction, Effect, GeneratedCostRule, NamedOptionKind, Pile,
};
use crate::effects::types::CardPlayContext;
use crate::orbs::{EvokeEffect, PassiveEffect};
use crate::potions;
use crate::powers;
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

#[cfg(test)]
#[path = "tests/test_runtime_inline_cutover_wave5.rs"]
mod test_runtime_inline_cutover_wave5;

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
    PlayCardFreeFromDraw,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamedChoicePayload {
    pub kind: NamedOptionKind,
    pub amount: i32,
}

/// Card effects queued behind an interactive ScryAction. Java keeps later
/// actions pending until the Scry choice resolves; Just Lucky uses block then
/// damage in that order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeferredScryCardEffects {
    pub card_inst: CardInstance,
    pub target_idx: i32,
    pub x_value: i32,
    pub pen_nib_active: bool,
    pub vigor: i32,
    pub hand_size_at_play: usize,
    pub gain_block: bool,
    pub deal_damage: bool,
}

/// Context for an in-progress player choice.
#[derive(Debug, Clone)]
pub struct ChoiceContext {
    pub reason: ChoiceReason,
    pub options: Vec<ChoiceOption>,
    pub selected: Vec<usize>,
    pub min_picks: usize,
    pub max_picks: usize,
    /// Extra effect-specific count that should not change choice cardinality.
    /// Used for cases like Dual Wield/Nightmare where the player picks one card
    /// and the engine later duplicates it N times.
    pub aux_count: usize,
    /// Optional semantic action for declarative choice resolution.
    pub action: Option<ChoiceAction>,
    /// Optional post-choice draw handoff owned by the choice itself.
    /// Burning Pact uses this to draw after the exhaust choice resolves.
    pub post_choice_draw: i32,
    /// Optional block/damage actions queued after a Scry choice.
    pub deferred_scry_card_effects: Option<DeferredScryCardEffects>,
    /// Optional stance action queued behind an interactive card choice.
    pub deferred_stance: Option<Stance>,
    /// Optional per-option payload for named choices like Wish.
    pub named_payloads: Option<Vec<NamedChoicePayload>>,
    /// Optional post-selection cost rule for generated-card choices.
    pub generated_selected_cost_rule: Option<GeneratedCostRule>,
    /// Optional fixed cost override applied when returning selected discard cards to hand.
    pub returned_card_cost_override: Option<i8>,
    /// Whether cards returned from discard should be marked retained.
    pub retain_returned_cards: bool,
    /// Codex-style generated choice: add the selection to a random draw-pile
    /// position instead of the hand.
    pub generated_to_draw: bool,
}

/// The Rust combat engine. Wraps CombatState + card registry + RNG.
#[derive(Clone)]
pub struct CombatEngine {
    pub state: CombatState,
    pub phase: CombatPhase,
    pub card_registry: &'static CardRegistry,
    pub(crate) rng: crate::seed::StsRandom,
    /// Java's `AbstractDungeon.cardRandomRng`, used by random card placement
    /// and card-owned random choices. RunEngine seeds this with seed + floor.
    pub(crate) card_random_rng: crate::seed::StsRandom,
    /// Java's `AbstractDungeon.potionRng`, used by random potion generation.
    pub(crate) potion_rng: crate::seed::StsRandom,
    /// Java's `AbstractDungeon.miscRng`, used by Lesson Learned and event-style
    /// miscellaneous selections. It is also seeded with seed + floor.
    pub(crate) misc_rng: crate::seed::StsRandom,
    /// Per-combat enemy AI RNG. Java uses `AbstractDungeon.aiRng` consumed once per
    /// `AbstractMonster.rollMove()` and passed as `num` to `getMove(int num)` for
    /// probabilistic intent branching (JawWorm, Chosen, ~20+ enemies). Kept separate
    /// from `rng` so card/shuffle draws do not perturb intent sequences.
    pub(crate) ai_rng: crate::seed::StsRandom,
    pub choice: Option<ChoiceContext>,
    pub effect_runtime: crate::effects::runtime::EffectRuntime,
    pub(crate) nightmare_pending_copies: Vec<(CardInstance, usize)>,
    /// Omniscience autoplay items paired with the runtime-stack depth of the
    /// parent card whose action queue owns them.
    pub(crate) omniscience_autoplay: Vec<(CardInstance, usize)>,
    pub event_log: Vec<crate::effects::runtime::GameEventRecord>,
    pub runtime_played_card: Option<CardInstance>,
    pub(crate) runtime_play_target_idx: Option<i32>,
    pub(crate) runtime_play_stack: Vec<(CardInstance, i32)>,
    pub runtime_replay_window: bool,
    /// Raw energyOnUse captured for the current X-cost card. Necronomicon's
    /// queued copy reuses this value without spending energy a second time.
    pub(crate) runtime_last_x_energy_on_use: i32,
    pub(crate) runtime_x_energy_override: Option<i32>,
    pub(crate) pending_combat_start_resume: bool,
    pub(crate) pending_end_turn_resume: bool,
    pub runtime_card_total_unblocked_damage: i32,
    pub runtime_card_enemy_killed: bool,
}

impl CombatEngine {
    /// Create a new combat engine.
    pub fn new(state: CombatState, seed: u64) -> Self {
        Self::new_with_card_random_seed(state, seed, seed)
    }

    pub(crate) fn new_with_card_random_seed(
        state: CombatState,
        seed: u64,
        card_random_seed: u64,
    ) -> Self {
        let mut effect_runtime = crate::effects::runtime::EffectRuntime::default();
        effect_runtime.rebuild_from_state(&state);
        Self {
            state,
            phase: CombatPhase::NotStarted,
            card_registry: crate::cards::global_registry(),
            rng: crate::seed::StsRandom::new(seed),
            card_random_rng: crate::seed::StsRandom::new(card_random_seed),
            potion_rng: crate::seed::StsRandom::new(card_random_seed),
            misc_rng: crate::seed::StsRandom::new(card_random_seed),
            // ai_rng seeded distinctly so it is not perturbed by re-seeds of `rng`
            // when callers replay a snapshot. Mirrors Java's separate dungeon-level
            // `aiRng` distinct from `cardRandomRng`/`shuffleRng`.
            ai_rng: crate::seed::StsRandom::new(seed.wrapping_add(0xA1A1_A1A1)),
            choice: None,
            effect_runtime,
            nightmare_pending_copies: Vec::new(),
            omniscience_autoplay: Vec::new(),
            event_log: Vec::new(),
            runtime_played_card: None,
            runtime_play_target_idx: None,
            runtime_play_stack: Vec::new(),
            runtime_replay_window: false,
            runtime_last_x_energy_on_use: 0,
            runtime_x_energy_override: None,
            pending_combat_start_resume: false,
            pending_end_turn_resume: false,
            runtime_card_total_unblocked_damage: 0,
            runtime_card_enemy_killed: false,
        }
    }

    pub fn rebuild_effect_runtime(&mut self) {
        self.effect_runtime.rebuild_from_state(&self.state);
    }

    /// RNG stream counters currently tracked by this combat, keyed by the
    /// vault's canonical short names (`docs/vault/rng-system-analysis.md`).
    /// Used by `bin/trace_replay.rs` (U05) to populate `trace::PostState::rng`.
    /// The currently threaded card, cardRandom, misc, and enemy-AI streams are
    /// reported independently; absent canonical streams remain schema-legal.
    pub fn rng_counters(&self) -> std::collections::BTreeMap<String, u64> {
        let mut counters = std::collections::BTreeMap::new();
        counters.insert("card".to_string(), self.rng.counter as u64);
        counters.insert(
            "cardRandom".to_string(),
            self.card_random_rng.counter as u64,
        );
        counters.insert("potion".to_string(), self.potion_rng.counter as u64);
        counters.insert("misc".to_string(), self.misc_rng.counter as u64);
        counters.insert("ai".to_string(), self.ai_rng.counter as u64);
        counters
    }

    pub fn load_persisted_effects(
        &mut self,
        states: Vec<crate::effects::runtime::PersistedEffectState>,
    ) {
        self.effect_runtime.load_persisted_states(states);
        self.rebuild_effect_runtime();
    }

    pub fn export_persisted_effects(&self) -> Vec<crate::effects::runtime::PersistedEffectState> {
        self.effect_runtime.export_persisted_states()
    }

    pub fn emit_event(&mut self, event: crate::effects::runtime::GameEvent) {
        let mut runtime = std::mem::take(&mut self.effect_runtime);
        let mut event = event;
        if event.card_inst.is_none() {
            event.card_inst = self.runtime_played_card;
        }
        event.replay_window = self.runtime_replay_window;
        runtime.emit(self, event);
        self.effect_runtime = runtime;
    }

    pub fn take_event_log(&mut self) -> Vec<crate::effects::runtime::GameEventRecord> {
        std::mem::take(&mut self.event_log)
    }

    pub fn clear_event_log(&mut self) {
        self.event_log.clear();
    }

    fn begin_runtime_play_context(&mut self, card_inst: CardInstance, target_idx: i32) {
        if let Some(current) = self.runtime_played_card {
            self.runtime_play_stack
                .push((current, self.runtime_play_target_idx.unwrap_or(-1)));
        }
        self.runtime_played_card = Some(card_inst);
        self.runtime_play_target_idx = Some(target_idx);
    }

    fn finish_runtime_play_context(&mut self) {
        if let Some((card_inst, target_idx)) = self.runtime_play_stack.pop() {
            self.runtime_played_card = Some(card_inst);
            self.runtime_play_target_idx = Some(target_idx);
        } else {
            self.runtime_played_card = None;
            self.runtime_play_target_idx = None;
        }
    }

    fn clear_runtime_play_contexts(&mut self) {
        self.runtime_played_card = None;
        self.runtime_play_target_idx = None;
        self.runtime_play_stack.clear();
        self.omniscience_autoplay.clear();
    }

    pub fn hidden_effect_value(
        &self,
        def_id: &str,
        owner: crate::effects::runtime::EffectOwner,
        slot: usize,
    ) -> i32 {
        self.effect_runtime.hidden_value(def_id, owner, slot)
    }

    pub(crate) fn set_hidden_effect_value(
        &mut self,
        def_id: &str,
        owner: crate::effects::runtime::EffectOwner,
        slot: usize,
        value: i32,
    ) -> bool {
        self.effect_runtime
            .set_hidden_value(def_id, owner, slot, value)
    }

    // =======================================================================
    // Core API
    // =======================================================================

    /// Start combat: apply relic effects, shuffle draw pile, draw initial hand.
    pub fn start_combat(&mut self) {
        let resuming_after_choice = self.pending_combat_start_resume;
        if self.phase != CombatPhase::NotStarted && !resuming_after_choice {
            return;
        }

        if resuming_after_choice {
            self.pending_combat_start_resume = false;
        } else {

        self.rebuild_effect_runtime();

        // Source: reference/extracted/methods/relic/PenNib.java
        // Pen Nib's relic counter persists between combats; counter 9 applies
        // its double-damage power at the next battle start.
        if let Some(slot) = self.state.relics.iter().position(|id| id == "Pen Nib") {
            let counter = self.hidden_effect_value(
                "Pen Nib",
                crate::effects::runtime::EffectOwner::PlayerRelic { slot: slot as u16 },
                0,
            );
            self.state.player.set_status(sid::PEN_NIB_COUNTER, counter);
        }

        // Source: extracted FungiBeast/LouseNormal/LouseDefensive methods and
        // methods/base/AbstractMonster.java (`init` -> `rollMove`). These
        // monsters have no fixed opener; select it with empty move history.
        for enemy in self.state.enemies.iter_mut().filter(|e| matches!(e.id.as_str(),
            "FungiBeast" | "FuzzyLouseNormal" | "RedLouse"
                | "FuzzyLouseDefensive" | "GreenLouse"
                | "SlaverBlue" | "BlueSlaver" | "SlaverRed" | "RedSlaver"
                | "AcidSlime_S" | "AcidSlime_M" | "AcidSlime_L"
                | "SpikeSlime_S" | "SpikeSlime_M" | "SpikeSlime_L" | "Looter"
                | "GremlinFat" | "GremlinThief" | "GremlinWarrior"
                | "GremlinWizard" | "GremlinTsundere" | "GremlinNob"
                | "Lagavulin" | "Sentry" | "TheGuardian" | "Hexaghost"
                | "SlimeBoss" | "Apology Slime" | "ApologySlime")) {
            crate::enemies::roll_initial_move(enemy, &mut self.ai_rng);
        }

        // Apply combat-start relic + power effects via owner-aware runtime.
        self.emit_event(crate::effects::runtime::GameEvent::empty(
            crate::effects::trigger::Trigger::CombatStart,
        ));
        if self.phase == CombatPhase::AwaitingChoice {
            self.pending_combat_start_resume = true;
            return;
        }
        }

        // Channel orbs from combat-start relics (need engine context)
        if self.state.player.status(sid::CHANNEL_DARK_START) > 0 {
            self.channel_orb(crate::orbs::OrbType::Dark);
            self.state.player.set_status(sid::CHANNEL_DARK_START, 0);
        }
        if self.state.player.status(sid::CHANNEL_LIGHTNING_START) > 0 {
            self.channel_orb(crate::orbs::OrbType::Lightning);
            self.state
                .player
                .set_status(sid::CHANNEL_LIGHTNING_START, 0);
        }
        if self.state.player.status(sid::CHANNEL_PLASMA_START) > 0 {
            self.channel_orb(crate::orbs::OrbType::Plasma);
            self.state.player.set_status(sid::CHANNEL_PLASMA_START, 0);
        }

        // Shuffle draw pile
        self.state.draw_pile.shuffle(&mut self.rng);

        // Innate: move cards with "innate" tag to top of draw pile
        // Draw pile convention: index 0 = bottom, last = top
        let mut innate_indices = Vec::new();
        for (i, card) in self.state.draw_pile.iter().enumerate() {
            let def = self.card_registry.card_def_by_id(card.def_id);
            if def.runtime_traits().innate
                || card.flags & CardInstance::FLAG_INNATE != 0
            {
                innate_indices.push(i);
            }
        }
        // Remove from back to front to preserve indices, then push to end (top)
        let mut innate_cards = Vec::new();
        for &i in innate_indices.iter().rev() {
            innate_cards.push(self.state.draw_pile.remove(i));
        }
        innate_cards.reverse(); // Maintain original order
        for card in innate_cards {
            self.state.draw_pile.push(card);
        }

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
                if !potions::potion_can_use_in_combat(&self.state, potion_id) {
                    continue;
                }
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
        // Refresh the owner-aware dispatch table once per player action so
        // externally mutated state (tests, setup code, future run effects)
        // is visible before any triggered behavior fires.
        self.rebuild_effect_runtime();
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
            card_registry: self.card_registry, // &'static ref — zero-cost copy
            rng: self.rng.clone(),
            card_random_rng: self.card_random_rng.clone(),
            potion_rng: self.potion_rng.clone(),
            misc_rng: self.misc_rng.clone(),
            ai_rng: self.ai_rng.clone(),
            choice: self.choice.clone(),
            effect_runtime: self.effect_runtime.clone(),
            nightmare_pending_copies: self.nightmare_pending_copies.clone(),
            omniscience_autoplay: self.omniscience_autoplay.clone(),
            event_log: self.event_log.clone(),
            runtime_played_card: self.runtime_played_card,
            runtime_play_target_idx: self.runtime_play_target_idx,
            runtime_play_stack: self.runtime_play_stack.clone(),
            runtime_replay_window: self.runtime_replay_window,
            runtime_last_x_energy_on_use: self.runtime_last_x_energy_on_use,
            runtime_x_energy_override: self.runtime_x_energy_override,
            pending_combat_start_resume: self.pending_combat_start_resume,
            pending_end_turn_resume: self.pending_end_turn_resume,
            runtime_card_total_unblocked_damage: self.runtime_card_total_unblocked_damage,
            runtime_card_enemy_killed: self.runtime_card_enemy_killed,
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
        self.begin_choice_with_action(reason, options, min_picks, max_picks, 0, None);
    }

    pub fn begin_choice_with_named_payloads(
        &mut self,
        reason: ChoiceReason,
        options: Vec<ChoiceOption>,
        min_picks: usize,
        max_picks: usize,
        named_payloads: Vec<NamedChoicePayload>,
    ) {
        self.begin_choice_with_action(reason, options, min_picks, max_picks, 0, None);
        if let Some(choice) = self.choice.as_mut() {
            choice.named_payloads = Some(named_payloads);
        }
    }

    pub fn begin_discovery_choice(
        &mut self,
        options: Vec<ChoiceOption>,
        min_picks: usize,
        max_picks: usize,
        aux_count: usize,
        selected_cost_rule: GeneratedCostRule,
    ) {
        self.begin_choice_with_action(
            ChoiceReason::DiscoverCard,
            options,
            min_picks,
            max_picks,
            aux_count,
            None,
        );
        if let Some(choice) = self.choice.as_mut() {
            choice.generated_selected_cost_rule = Some(selected_cost_rule);
        }
    }

    pub(crate) fn begin_codex_choice(&mut self, options: Vec<ChoiceOption>) {
        self.begin_discovery_choice(options, 1, 1, 1, GeneratedCostRule::Base);
        if let Some(choice) = self.choice.as_mut() {
            choice.generated_to_draw = true;
        }
    }

    pub fn begin_choice_with_aux(
        &mut self,
        reason: ChoiceReason,
        options: Vec<ChoiceOption>,
        min_picks: usize,
        max_picks: usize,
        aux_count: usize,
    ) {
        self.begin_choice_with_action(reason, options, min_picks, max_picks, aux_count, None);
    }

    pub fn begin_choice_with_action(
        &mut self,
        reason: ChoiceReason,
        options: Vec<ChoiceOption>,
        min_picks: usize,
        max_picks: usize,
        aux_count: usize,
        action: Option<ChoiceAction>,
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
            aux_count,
            action,
            post_choice_draw: 0,
            deferred_scry_card_effects: None,
            deferred_stance: None,
            named_payloads: None,
            generated_selected_cost_rule: None,
            returned_card_cost_override: None,
            retain_returned_cards: false,
            generated_to_draw: false,
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
        let deferred_stance = ctx.deferred_stance;
        // Choice-owned autoplay (Omniscience, Havoc-style free play) must run
        // through the ordinary card pipeline, which may itself open a nested
        // choice. The prior choice has already been consumed at this point.
        self.phase = CombatPhase::PlayerTurn;

        match ctx.action {
            Some(ChoiceAction::StoreCardForNextTurnCopies) => self.resolve_nightmare(ctx),
            _ => match ctx.reason {
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
                ChoiceReason::PlayCardFreeFromDraw => self.resolve_play_card_free_from_draw(ctx),
            },
        }

        if self.choice.is_some() {
            return;
        }

        if let Some(stance) = deferred_stance {
            self.change_stance(stance);
        }

        if self.state.combat_over {
            self.clear_runtime_play_contexts();
            return;
        }
        if self.pending_combat_start_resume {
            self.start_combat();
            return;
        }
        if self.pending_end_turn_resume {
            self.end_turn();
            return;
        }
        self.drive_choice_owned_runtime();
    }

    fn drive_choice_owned_runtime(&mut self) {
        while self.phase == CombatPhase::PlayerTurn && !self.state.combat_over {
            if self.play_ready_omniscience_autoplay() {
                continue;
            }
            if self.runtime_played_card.is_some() {
                self.resume_played_card_tail_from_runtime();
                continue;
            }
            break;
        }
    }

    fn resolve_scry(&mut self, ctx: ChoiceContext) {
        let post_choice_draw = ctx.post_choice_draw;
        let deferred_card_effects = ctx.deferred_scry_card_effects;
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
        // Put kept cards back on top of draw pile (last = top, pop = draw)
        for card in to_keep.into_iter().rev() {
            self.state.draw_pile.push(card);
        }
        for card in to_discard {
            self.state.discard_pile.push(card);
        }

        // Weave: return from discard to hand on Scry
        let mut weave_indices = Vec::new();
        for (i, card_inst) in self.state.discard_pile.iter().enumerate() {
            if self
                .card_registry
                .card_name(card_inst.def_id)
                .starts_with("Weave")
            {
                weave_indices.push(i);
            }
        }
        for &i in weave_indices.iter().rev() {
            // DiscardToHandAction leaves the card in discard when the hand is
            // full; it never removes and destroys the card.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/DiscardToHandAction.java
            if self.state.hand.len() >= 10 {
                break;
            }
            let card = self.state.discard_pile.remove(i);
            self.state.hand.push(card);
        }

        // CutThroughFate.java queues DrawCardAction(1) after ScryAction, so the
        // draw cannot occur until this choice has finished resolving.
        if post_choice_draw > 0 {
            self.draw_cards(post_choice_draw);
        }
        if let Some(deferred) = deferred_card_effects {
            self.resolve_deferred_scry_card_effects(deferred);
        }
    }

    fn resolve_deferred_scry_card_effects(&mut self, deferred: DeferredScryCardEffects) {
        let card = self
            .card_registry
            .card_def_by_id(deferred.card_inst.def_id)
            .clone();
        if deferred.gain_block && card.base_block >= 0 {
            let dex = self.state.player.dexterity();
            let frail = self.state.player.is_frail();
            let block = crate::damage::calculate_block(card.base_block, dex, frail);
            self.gain_block_player(block);
        }
        if deferred.deal_damage {
            let mut ctx = crate::effects::types::CardPlayContext {
                card: &card,
                card_inst: deferred.card_inst,
                target_idx: deferred.target_idx,
                x_value: deferred.x_value,
                pen_nib_active: deferred.pen_nib_active,
                vigor: deferred.vigor,
                total_unblocked_damage: 0,
                enemy_killed: false,
                hand_size_at_play: deferred.hand_size_at_play,
                last_bulk_count: 0,
                last_drawn_card_types: Vec::new(),
            };
            crate::card_effects::execute_primary_attack(
                self,
                &mut ctx,
                crate::effects::declarative::Target::SelectedEnemy,
            );
        }
    }

    fn resolve_discard_from_hand(&mut self, ctx: ChoiceContext) {
        // Discard selected hand cards (process in reverse to maintain indices)
        let mut indices: Vec<usize> = ctx
            .selected
            .iter()
            .filter_map(|&i| {
                if let ChoiceOption::HandCard(idx) = ctx.options[i] {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
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
        let mut indices: Vec<usize> = ctx
            .selected
            .iter()
            .filter_map(|&i| {
                if let ChoiceOption::HandCard(idx) = ctx.options[i] {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        for idx in indices {
            if idx < self.state.hand.len() {
                let card = self.state.hand.remove(idx);
                self.state.exhaust_pile.push(card);
                self.trigger_card_on_exhaust(card);
            }
        }
        // Burning Pact: draw cards after exhaust choice resolves.
        if ctx.post_choice_draw > 0 {
            self.draw_cards(ctx.post_choice_draw);
        }
    }

    fn resolve_put_on_top(&mut self, ctx: ChoiceContext) {
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::HandCard(idx) = ctx.options[sel] {
                if idx < self.state.hand.len() {
                    let card = self.state.hand.remove(idx);
                    self.state.draw_pile.push(card); // last = top
                }
            }
        }
    }

    fn resolve_pick_from_discard(&mut self, ctx: ChoiceContext) {
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::DiscardCard(idx) = ctx.options[sel] {
                if idx < self.state.discard_pile.len() {
                    let card = self.state.discard_pile.remove(idx);
                    // Put on top of draw pile (Headbutt) — last = top
                    self.state.draw_pile.push(card);
                }
            }
        }
    }

    fn resolve_pick_from_draw(&mut self, ctx: ChoiceContext) {
        // Seek: move selected card(s) from draw pile to hand
        let mut indices: Vec<usize> = ctx
            .selected
            .iter()
            .filter_map(|&i| {
                if let ChoiceOption::DrawCard(idx) = ctx.options[i] {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
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
            if let ChoiceOption::GeneratedCard(preview_card) = ctx.options[sel] {
                let copies = ctx.aux_count.max(1);
                let selected_cost_rule = ctx
                    .generated_selected_cost_rule
                    .unwrap_or(GeneratedCostRule::Base);
                for _ in 0..copies {
                    let mut card = preview_card;
                    if self.state.player.status(sid::MASTER_REALITY) > 0 {
                        let card_id = self.card_registry.card_def_by_id(card.def_id).id;
                        if card_id.ends_with('+') {
                            card.flags |= crate::combat_types::CardInstance::FLAG_UPGRADED;
                        } else if !card.is_upgraded() {
                            self.card_registry.upgrade_card(&mut card);
                        }
                    }
                    let card_id = self.card_registry.card_name(card.def_id);
                    if crate::effects::interpreter::is_colorless_generation_card(card_id) {
                        card.cost = 0;
                    }
                    crate::effects::interpreter::apply_generated_cost_rule(
                        &mut card,
                        selected_cost_rule,
                    );
                    if ctx.generated_to_draw {
                        if self.state.draw_pile.is_empty() {
                            self.state.draw_pile.push(card);
                        } else {
                            // CardGroup.addToRandomSpot uses cardRandomRng and
                            // inserts before one existing index.
                            let index = self
                                .card_random_rng
                                .random((self.state.draw_pile.len() - 1) as i32)
                                as usize;
                            self.state.draw_pile.insert(index, card);
                        }
                    } else if self.state.hand.len() < 10 {
                        self.state.hand.push(card);
                    } else {
                        self.state.discard_pile.push(card);
                    }
                }
            }
        }
    }

    fn resolve_pick_option(&mut self, ctx: ChoiceContext) {
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::Named(name) = ctx.options[sel] {
                if let Some(payload) = ctx
                    .named_payloads
                    .as_ref()
                    .and_then(|payloads| payloads.get(sel))
                    .copied()
                {
                    match payload.kind {
                        NamedOptionKind::AddStatus(status) => {
                            let current = self.state.player.status(status);
                            self.state
                                .player
                                .set_status(status, current + payload.amount);
                        }
                        NamedOptionKind::GainRunGold => {
                            self.gain_run_gold(payload.amount);
                        }
                        NamedOptionKind::SetStance(stance) => {
                            self.change_stance(stance);
                        }
                    }
                    return;
                }

                match name {
                    "Strength" => {
                        let current = self.state.player.status(sid::STRENGTH);
                        self.state.player.set_status(sid::STRENGTH, current + 3);
                    }
                    "Gold" => {
                        // Combat engine can't modify run gold; no-op for MCTS
                        // (MCTS should prefer Strength or Plated Armor options)
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
                    let copies = ctx.aux_count.max(1);
                    self.add_dual_wield_copies(card, copies);
                }
            }
        }
    }

    pub(crate) fn add_dual_wield_copies(&mut self, card: CardInstance, copies: usize) {
        // DualWieldAction queues one MakeTempCardInHandAction per copy. That
        // action puts copies beyond the ten-card hand limit in the discard pile.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DualWieldAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
        for _ in 0..copies {
            if self.state.hand.len() < 10 {
                self.state.hand.push(card);
            } else {
                self.state.discard_pile.push(card);
            }
        }
    }

    fn resolve_nightmare(&mut self, ctx: ChoiceContext) {
        // Nightmare: remember the chosen card and add copies on the next turn start.
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::HandCard(idx) = ctx.options[sel] {
                if idx < self.state.hand.len() {
                    self.store_nightmare_copies(
                        self.state.hand[idx],
                        ctx.aux_count.max(1),
                    );
                }
            }
        }
    }

    pub(crate) fn store_nightmare_copies(&mut self, mut card: CardInstance, copies: usize) {
        // NightmarePower stores makeStatEquivalentCopy().resetAttributes():
        // permanent cost, upgrade, misc and freeToPlayOnce survive, while
        // transient retain/purge/exhaust flags do not.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/NightmarePower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/AbstractCard.java
        card.reset_cost_for_turn();
        card.flags &= CardInstance::FLAG_UPGRADED
            | CardInstance::FLAG_FREE
            | CardInstance::FLAG_INNATE;

        // ApplyPowerAction stacks another NightmarePower with the same ID onto
        // the existing power. AbstractPower.stackPower increases amount but
        // leaves the first power's stored card unchanged.
        if let Some((_, pending_copies)) = self.nightmare_pending_copies.first_mut() {
            *pending_copies += copies;
        } else {
            self.nightmare_pending_copies.push((card, copies));
        }
    }

    fn resolve_upgrade_card(&mut self, ctx: ChoiceContext) {
        // Armaments: upgrade selected card in hand
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::HandCard(idx) = ctx.options[sel] {
                if idx < self.state.hand.len() {
                    self.card_registry.upgrade_card(&mut self.state.hand[idx]);
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
        // Play selected card from hand for free
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::HandCard(idx) = ctx.options[sel] {
                if idx < self.state.hand.len() {
                    // Set card to free
                    self.state.hand[idx].cost = 0;
                    self.state.hand[idx].flags |= crate::combat_types::CardInstance::FLAG_FREE;
                    // Play it (target -1 = self; for targeted cards MCTS will handle)
                    let target = if self
                        .card_registry
                        .card_def_by_id(self.state.hand[idx].def_id)
                        .target
                        == CardTarget::Enemy
                    {
                        self.state
                            .targetable_enemy_indices()
                            .first()
                            .copied()
                            .unwrap_or(0) as i32
                    } else {
                        -1
                    };
                    self.play_card(idx, target);
                }
            }
        }
    }

    fn resolve_play_card_free_from_draw(&mut self, ctx: ChoiceContext) {
        // OmniscienceAction removes the selected original from draw, forces it
        // to exhaust, then queues the original plus one purge-on-use stat copy.
        // Both are autoplayed with random targets and no energy payment.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/OmniscienceAction.java
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::DrawCard(idx) = ctx.options[sel] {
                if idx < self.state.draw_pile.len() {
                    let mut card = self.state.draw_pile.remove(idx);
                    card.cost = 0;
                    card.flags |= CardInstance::FLAG_FREE | CardInstance::FLAG_EXHAUST_ON_USE;
                    let mut purge_copy = card;
                    purge_copy.flags |= CardInstance::FLAG_PURGE;
                    let parent_depth = self.runtime_play_stack.len();
                    self.omniscience_autoplay.push((card, parent_depth));
                    self.omniscience_autoplay
                        .push((purge_copy, parent_depth));
                }
            }
        }
    }

    fn play_ready_omniscience_autoplay(&mut self) -> bool {
        let depth = self.runtime_play_stack.len();
        let Some(queue_idx) = self
            .omniscience_autoplay
            .iter()
            .position(|(_, parent_depth)| *parent_depth == depth)
        else {
            return false;
        };
        let (card, _) = self.omniscience_autoplay.remove(queue_idx);
        let def = self.card_registry.card_def_by_id(card.def_id).clone();
        let target = if def.target == CardTarget::Enemy {
            let living = self.state.targetable_enemy_indices();
            if living.is_empty() {
                -1
            } else {
                // NewQueueCardAction(randomTarget=true) asks MonsterGroup for a
                // random living target on each queued play, consuming
                // cardRandomRng even when only one target exists.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
                let selected = self
                    .card_random_rng
                    .random_range(0, (living.len() - 1) as i32) as usize;
                living[selected] as i32
            }
        } else {
            -1
        };

        if !self.can_play_card_inst(&def, card) {
            if card.flags & CardInstance::FLAG_PURGE == 0
                && card.flags & CardInstance::FLAG_EXHAUST_ON_USE != 0
            {
                self.state.exhaust_pile.push(card);
                self.trigger_card_on_exhaust(card);
            }
            return true;
        }

        // Autoplay cards live outside the hand in Java. Push only for the
        // duration of the ordinary Rust play pipeline; play_card removes it
        // before hand-sensitive effects and hooks execute.
        self.state.hand.push(card);
        let hand_idx = self.state.hand.len() - 1;
        self.play_card(hand_idx, target);
        true
    }

    fn resolve_search_draw_pile(&mut self, ctx: ChoiceContext) {
        // Secret Weapon / Secret Technique: move selected card from draw pile to hand
        let mut indices: Vec<usize> = ctx
            .selected
            .iter()
            .filter_map(|&i| {
                if let ChoiceOption::DrawCard(idx) = ctx.options[i] {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        for idx in indices {
            if idx < self.state.draw_pile.len() {
                if self.state.hand.len() == 10 {
                    let card = self.state.draw_pile.remove(idx);
                    self.state.discard_pile.push(card);
                } else {
                    let card = self.state.draw_pile.remove(idx);
                    self.state.hand.push(card);
                }
            }
        }
    }

    fn resolve_return_from_discard(&mut self, ctx: ChoiceContext) {
        // Hologram / Meditate: move selected card(s) from discard to hand
        let mut indices: Vec<usize> = ctx
            .selected
            .iter()
            .filter_map(|&i| {
                if let ChoiceOption::DiscardCard(idx) = ctx.options[i] {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
        indices.sort_unstable_by(|a, b| b.cmp(a)); // remove from back to front
        for idx in indices {
            if idx < self.state.discard_pile.len() && self.state.hand.len() < 10 {
                let mut card = self.state.discard_pile.remove(idx);
                if ctx.retain_returned_cards {
                    card.set_retained(true); // Meditate marks returned cards as retained
                }
                if let Some(cost) = ctx.returned_card_cost_override {
                    card.cost = cost;
                }
                self.state.hand.push(card);
            }
        }
    }

    fn resolve_forethought(&mut self, ctx: ChoiceContext) {
        let indices: Vec<usize> = ctx
            .selected
            .iter()
            .filter_map(|&i| {
                if let ChoiceOption::HandCard(idx) = ctx.options[i] {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
        self.move_forethought_cards_to_bottom(&indices);
    }

    pub(crate) fn increase_all_claw_damage(&mut self, amount: i32) {
        // GashAction mutates the played card, then Claws in discard, draw, and
        // hand. It deliberately excludes exhaust and cards created later.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/GashAction.java
        let registry = &self.card_registry;
        let increase = |card: &mut CardInstance| {
            let def = registry.card_def_by_id(card.def_id);
            if !matches!(def.id, "Gash" | "Gash+") {
                return;
            }
            let current = if card.misc >= 0 {
                card.misc as i32
            } else {
                def.base_damage
            };
            card.misc = (current + amount).max(0).min(i16::MAX as i32) as i16;
        };

        if let Some(card) = self.runtime_played_card.as_mut() {
            increase(card);
        }
        for card in &mut self.state.discard_pile {
            increase(card);
        }
        for card in &mut self.state.draw_pile {
            increase(card);
        }
        for card in &mut self.state.hand {
            increase(card);
        }
    }

    pub(crate) fn sync_genetic_algorithm_master_deck(
        &mut self,
        before: CardInstance,
        next_block: i16,
    ) {
        // IncreaseMiscAction updates both the combat instance and the matching
        // UUID in AbstractDungeon.player.masterDeck. CardInstance has no UUID,
        // but equal same-definition/misc copies are behaviorally interchangeable,
        // so updating one matching member preserves the exact state multiset.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/IncreaseMiscAction.java
        let def = self.card_registry.card_def_by_id(before.def_id);
        let previous_block = if before.misc >= 0 {
            before.misc
        } else {
            def.base_block.max(0) as i16
        };
        if let Some(card) = self.state.master_deck.iter_mut().find(|card| {
            if card.def_id != before.def_id {
                return false;
            }
            let current = if card.misc >= 0 {
                card.misc
            } else {
                def.base_block.max(0) as i16
            };
            current == previous_block
        }) {
            card.misc = next_block;
        }
    }

    pub(crate) fn move_forethought_cards_to_bottom(&mut self, indices: &[usize]) {
        // HandCardSelectScreen.addToTop preserves click order, and each
        // moveToBottomOfDeck inserts at index zero. Capture the selected cards
        // before removing them so that both selection order and duplicate card
        // definitions are preserved.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ForethoughtAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
        let selected: Vec<CardInstance> = indices
            .iter()
            .filter_map(|&index| self.state.hand.get(index).copied())
            .collect();
        let mut removal_indices: Vec<usize> = indices
            .iter()
            .copied()
            .filter(|&index| index < self.state.hand.len())
            .collect();
        removal_indices.sort_unstable_by(|left, right| right.cmp(left));
        removal_indices.dedup();
        for index in removal_indices {
            self.state.hand.remove(index);
        }

        for mut card in selected {
            let def = self.card_registry.card_def_by_id(card.def_id);
            let permanent_cost = if card.base_cost >= 0 {
                card.base_cost
            } else {
                def.cost as i8
            };
            if permanent_cost > 0 {
                card.flags |= CardInstance::FLAG_FREE;
            }
            // Convention: last = top (pop draws), index 0 = bottom.
            self.state.draw_pile.insert(0, card);
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
                    self.trigger_card_on_exhaust(card);
                }
            }
        }
    }

    fn resolve_setup(&mut self, ctx: ChoiceContext) {
        // Setup: set card cost to 0 and put on top of draw pile (last = top)
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::HandCard(idx) = ctx.options[sel] {
                if idx < self.state.hand.len() {
                    let mut card = self.state.hand.remove(idx);
                    card.cost = 0;
                    self.state.draw_pile.push(card);
                }
            }
        }
    }

    fn resolve_discard_for_effect(&mut self, ctx: ChoiceContext) {
        // Concentrate: discard N cards, then gain energy
        let mut indices: Vec<usize> = ctx
            .selected
            .iter()
            .filter_map(|&i| {
                if let ChoiceOption::HandCard(idx) = ctx.options[i] {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        let mut discarded_cards = Vec::new();
        for idx in indices {
            if idx < self.state.hand.len() {
                let card = self.state.hand.remove(idx);
                self.state.discard_pile.push(card);
                discarded_cards.push(card);
            }
        }
        for card in discarded_cards {
            self.on_card_discarded(card);
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
        self.rebuild_effect_runtime();

        // Blasphemy delayed death.
        // Java: powers/watcher/EndTurnDeathPower.java queues LoseHPAction(99999),
        // which runs through AbstractPlayer.damage(HP_LOSS): Intangible caps to
        // 1, Buffer can reduce that to 0, Tungsten Rod applies to remaining HP
        // loss, and ordinary death/revival hooks still run.
        if self.state.blasphemy_active {
            self.state.blasphemy_active = false;
            self.player_lose_hp_from_damage(99_999);
            if self.state.combat_over {
                return;
            }
        }

        // EnergyManager.java::recharge calls addEnergy(energyMaster) with Ice
        // Cream, preserving unspent energy instead of resetting to the master amount.
        if self.state.has_relic("Ice Cream") || self.state.has_relic("IceCream") {
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
        if self.state.has_relic("Necronomicon") {
            self.state.player.set_status(sid::NECRONOMICON_USED, 0);
        }

        // All turn-start relic + power effects via owner-aware runtime.
        self.emit_event(crate::effects::runtime::GameEvent {
            kind: crate::effects::trigger::Trigger::TurnStart,
            card_type: None,
            card_inst: None,
            is_first_turn: self.state.turn == 1,
            target_idx: -1,
            enemy_idx: -1,
            potion_slot: -1,
            status_id: None,
            amount: 0,
            replay_window: false,
        });

        // WrathNextTurnPower.atStartOfTurn changes stance before normal draw.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/WrathNextTurnPower.java
        if self.state.player.status(sid::WRATH_NEXT_TURN) > 0 {
            self.state.player.set_status(sid::WRATH_NEXT_TURN, 0);
            self.change_stance(Stance::Wrath);
        }

        self.flush_pending_nightmare_copies();

        // Divinity auto-exit at start of turn
        if self.state.stance == Stance::Divinity {
            self.change_stance(Stance::Neutral);
        }

        // Block decay — Calipers loses 15 instead of all; Barricade retains all.
        // Skip block reset on turn 1 to preserve combat-start relic effects (Anchor)
        if self.state.turn > 1 {
            let barricade = self.state.player.status(sid::BARRICADE) > 0;
            let blur = self.state.player.status(sid::BLUR) > 0;
            if barricade || blur {
                // Keep all block
            } else {
                // D49 parity fix: Calipers SUBTRACTS 15 block at end of round,
                // not "cap at 15". Java: `loseBlock(15)` (AbstractCreature.loseBlock
                // truncates at 0). Pre-fix: 50 block -> Java keeps 35, Rust kept 15.
                // Matters for Body Slam / Barricade-style block stacking.
                self.state.player.block = if self.state.has_relic("Calipers") {
                    (self.state.player.block - 15).max(0)
                } else {
                    0
                };
            }
            // GameActionManager checks Blur for block retention before queued
            // atEndOfRound reduction resolves. Vault skips that entire
            // applyEndOfTurnPowers pass, so it must retain without decrementing.
            // Sources: GameActionManager.java and powers/BlurPower.java.
            if blur && !self.state.skip_enemy_turn {
                let blur_val = self.state.player.status(sid::BLUR);
                self.state
                    .player
                    .set_status(sid::BLUR, (blur_val - 1).max(0));
            }
        }

        // The skip marker must survive through the Blur/block-loss check above,
        // but applies to exactly one enemy round.
        self.state.skip_enemy_turn = false;

        // LoseStrength/LoseDexterity at end of the previous turn.
        // Turn 1 has no "previous turn", so combat-start temporary strength
        // should survive into the opening player turn.
        if self.state.turn > 1 {
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
        }

        // BiasPower.java applies a negative FocusPower at turn start. Because
        // that FocusPower is a DEBUFF, a later Artifact can block each tick.
        // Source: powers/BiasPower.java and powers/FocusPower.java.
        let bias_loss = self.state.player.status(sid::BIASED_COG_FOCUS_LOSS);
        if bias_loss > 0 {
            powers::apply_debuff(&mut self.state.player, sid::FOCUS, -bias_loss);
        }

        // === POWER HOOKS handled by the owner-aware TurnStart event above:
        // DemonForm, NoxiousFumes, Brutality, Berserk, InfiniteBlades, BattleHymn,
        // Devotion, WraithForm, DevaForm, HelloWorld, Magnetism,
        // DoppelgangerDraw, DoppelgangerEnergy
        //
        // One-shot statuses consumed by dispatch_trigger declarative effects need clearing:
        {
            let dd = self.state.player.status(sid::DOPPELGANGER_DRAW);
            if dd > 0 {
                self.state.player.set_status(sid::DOPPELGANGER_DRAW, 0);
            }
            let de = self.state.player.status(sid::DOPPELGANGER_ENERGY);
            if de > 0 {
                self.state.player.set_status(sid::DOPPELGANGER_ENERGY, 0);
            }
        }

        // LoopPower.atStartOfTurn calls both callbacks on the front orb once
        // per stack. GameActionManager clears old block synchronously after
        // invoking powers but before their queued actions resolve, so Loop's
        // Frost Block belongs to the new turn and cannot protect the intervening
        // enemy turn.
        // Java: powers/LoopPower.java and actions/GameActionManager.java.
        let loop_count = self.state.player.status(sid::LOOP).max(0);
        for _ in 0..loop_count {
            self.apply_front_orb_start_of_turn_passive();
            if self.state.combat_over {
                return;
            }
            self.apply_front_orb_end_of_turn_passive();
            if self.state.combat_over {
                return;
            }
        }

        // ---- Start-of-turn orb passives (Plasma) ----
        self.apply_orb_start_of_turn();

        // Emotion Chip: replay the full orb passive cycle on the next turn start after HP loss.
        {
            let ect = self.state.player.status(sid::EMOTION_CHIP_TRIGGER);
            if ect > 0 {
                self.state.player.set_status(sid::EMOTION_CHIP_TRIGGER, 0);
                self.apply_orb_impulse_passives();
            }
        }

        // CollectPower.onEnergyRecharge: add exactly one upgraded Miracle before
        // draw, then decrement the turn-based power by one.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CollectPower.java
        {
            let miracles = self.state.player.status(sid::COLLECT_MIRACLES);
            if miracles > 0 {
                self.add_temp_cards_to_hand("Miracle+", 1);
                self.state
                    .player
                    .set_status(sid::COLLECT_MIRACLES, miracles - 1);
            }
        }

        // Java GameActionManager queues relic atTurnStart actions before its
        // DrawCardAction; they resolve after the synchronous old-block clear.
        // Source: GameActionManager.java and AbstractPlayer.applyStartOfTurnRelics.
        self.emit_event(crate::effects::runtime::GameEvent {
            kind: crate::effects::trigger::Trigger::TurnStartPreDraw,
            card_type: None,
            card_inst: None,
            is_first_turn: self.state.turn == 1,
            target_idx: -1,
            enemy_idx: -1,
            potion_slot: -1,
            status_id: None,
            amount: 0,
            replay_window: false,
        });

        // Draw cards. SneckoEye.java raises masterHandSize by two for every
        // turn; DrawReductionPower lowers gameHandSize by exactly one while
        // present. It does not alter card-effect DrawCardActions.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DrawReductionPower.java
        let ml = self.state.player.status(sid::DRAW);
        let serpent = self.state.player.status(sid::RING_OF_SERPENT_DRAW);
        let snecko_eye = if self.state.player.status(sid::SNECKO_EYE) > 0 {
            2
        } else {
            0
        };
        let draw_reduction = i32::from(self.state.player.status(sid::DRAW_REDUCTION) > 0);
        self.draw_cards(5 + ml + serpent + snecko_eye - draw_reduction);

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

        // DrawCardNextTurnPower.atStartOfTurnPostDraw stacks its amount, draws,
        // then removes itself. This runs before ordinary post-draw powers such
        // as Foresight because Draw Card has priority 20.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DrawCardNextTurnPower.java
        let next_turn_draw = self.state.player.status(sid::SIMMERING_FURY);
        if next_turn_draw > 0 {
            self.draw_cards(next_turn_draw);
            self.state.player.set_status(sid::SIMMERING_FURY, 0);
        }

        // ---- Post-draw power effects (complex powers not in EntityDefs) ----

        // Post-draw runtime hooks that must happen before remaining turn-start setup.
        self.emit_event(crate::effects::runtime::GameEvent {
            kind: crate::effects::trigger::Trigger::TurnStartPostDraw,
            card_type: None,
            card_inst: None,
            is_first_turn: self.state.turn == 1,
            target_idx: -1,
            enemy_idx: -1,
            potion_slot: -1,
            status_id: None,
            amount: 0,
            replay_window: false,
        });
        if self.state.combat_over || self.phase == CombatPhase::AwaitingChoice {
            return;
        }

        // Foresight: scry N at start of turn (post-draw)
        {
            let foresight = self.state.player.status(sid::FORESIGHT);
            if foresight > 0 {
                self.do_scry(foresight);
            }
        }

        // Late post-draw runtime hooks (starter relics / turn-start relic setup).
        self.emit_event(crate::effects::runtime::GameEvent {
            kind: crate::effects::trigger::Trigger::TurnStartPostDrawLate,
            card_type: None,
            card_inst: None,
            is_first_turn: self.state.turn == 1,
            target_idx: -1,
            enemy_idx: -1,
            potion_slot: -1,
            status_id: None,
            amount: 0,
            replay_window: false,
        });
    }

    pub(crate) fn end_turn(&mut self) {
        if self.phase != CombatPhase::PlayerTurn {
            return;
        }
        if self.pending_end_turn_resume {
            self.pending_end_turn_resume = false;
        } else {
            self.rebuild_effect_runtime();

            // Clear Entangled (only lasts one turn)
            self.state.player.set_status(sid::ENTANGLED, 0);

            // ---- STS end-of-turn order: relics -> powers/buffs -> status cards -> discard ----

            // Unified dispatch for end-of-turn relics + powers
            self.emit_event(crate::effects::runtime::GameEvent::empty(
                crate::effects::trigger::Trigger::TurnEnd,
            ));
            if self.phase == CombatPhase::AwaitingChoice {
                self.pending_end_turn_resume = true;
                return;
            }
        }

        if self.state.combat_over {
            return;
        }

        // Rage clear at end of turn (not in POWER_DEFS)
        self.state.player.set_status(sid::RAGE, 0);

        // TempStrength revert at end of turn (not in POWER_DEFS)
        {
            let ts = self.state.player.status(sid::TEMP_STRENGTH);
            if ts > 0 {
                self.state.player.add_status(sid::STRENGTH, -ts);
                self.state.player.set_status(sid::TEMP_STRENGTH, 0);
            }
        }

        // TempStrengthLoss: restore temporary Strength loss on all enemies at end of turn
        for ei in 0..self.state.enemies.len() {
            if self.state.enemies[ei].is_alive() {
                let tsl = self.state.enemies[ei]
                    .entity
                    .status(sid::TEMP_STRENGTH_LOSS);
                if tsl > 0 {
                    self.state.enemies[ei].entity.add_status(sid::STRENGTH, tsl);
                    self.state.enemies[ei]
                        .entity
                        .set_status(sid::TEMP_STRENGTH_LOSS, 0);
                }
            }
        }

        // 3. End-of-turn hand card triggers (Burn, Decay, Regret, Doubt, Shame)
        let player_died = status_effects::process_end_turn_hand_cards(self);
        if player_died {
            self.phase = CombatPhase::CombatOver;
            return;
        }

        // 4. Discard hand — Runic Pyramid keeps ALL cards in hand (including Status/Curse).
        //    Only Ethereal cards exhaust at end of turn regardless of Runic Pyramid.
        // Source: decompiled/java-src/com/megacrit/cardcrawl/actions/common/
        // DiscardAtEndOfTurnAction.java skips DiscardAction with "Runic Pyramid".
        let _explicitly_retained = std::mem::take(&mut self.state.retained_cards);
        let mut ethereal_exhausted = 0i32;
        if self.state.has_relic("Runic Pyramid") || self.state.has_relic("RunicPyramid") {
            // Runic Pyramid: keep ALL cards except ethereal (which exhaust)
            let hand = std::mem::take(&mut self.state.hand);
            let mut kept = Vec::new();
            for card_inst in hand {
                let card = self.card_registry.card_def_by_id(card_inst.def_id);
                if card.runtime_traits().ethereal {
                    self.state.exhaust_pile.push(card_inst);
                    ethereal_exhausted += 1;
                } else {
                    // Runic Pyramid keeps cards without setting Java's retain
                    // or selfRetain flags. That distinction matters to
                    // EstablishmentPowerAction.
                    kept.push(card_inst);
                }
            }
            // Track retained cards for Establishment cost reduction
            self.state.retained_cards = kept.clone();
            self.state.hand = kept;
        } else {
            // Normal: retain tagged cards + explicitly retained (FLAG_RETAINED), exhaust ethereal, discard rest
            // Equilibrium / Well-Laid Plans: retain entire hand
            let retain_all = self.state.player.status(sid::RETAIN_HAND_FLAG) > 0;
            if retain_all {
                self.state.player.set_status(sid::RETAIN_HAND_FLAG, 0);
            }
            let hand = std::mem::take(&mut self.state.hand);
            let mut retained = Vec::new();
            for card_inst in hand {
                let card = self.card_registry.card_def_by_id(card_inst.def_id);
                // DiscardAtEndOfTurnAction first moves explicitly retained and
                // self-retaining cards to limbo, then calls
                // triggerOnEndOfPlayerTurn on the cards still in hand. Thus an
                // explicit retain can save Ethereal, but Equilibrium's blanket
                // discard suppression cannot.
                // Java: actions/common/DiscardAtEndOfTurnAction.java and
                // cards/AbstractCard.java::triggerOnEndOfPlayerTurn.
                if card.runtime_traits().retain || card_inst.is_retained() {
                    let mut retained_inst = card_inst;
                    retained_inst.set_retained(true);
                    retained.push(retained_inst);
                } else if card.runtime_traits().ethereal {
                    self.state.exhaust_pile.push(card_inst);
                    ethereal_exhausted += 1;
                } else if retain_all {
                    let mut retained_inst = card_inst;
                    retained_inst.set_retained(true);
                    retained.push(retained_inst);
                } else {
                    self.state.discard_pile.push(card_inst);
                }
            }
            // Track retained cards for Establishment cost reduction
            self.state.retained_cards = retained.clone();
            self.state.hand = retained;
        }

        // Reset temporary cost overrides back to the permanent baseline for
        // all cards that remain in combat. This mirrors Java's
        // AbstractCard.resetAttributes() at end-of-turn.
        for card in &mut self.state.draw_pile {
            card.reset_cost_for_turn();
        }
        for card in &mut self.state.discard_pile {
            card.reset_cost_for_turn();
        }
        for card in &mut self.state.exhaust_pile {
            card.reset_cost_for_turn();
        }
        for card in &mut self.state.hand {
            card.reset_cost_for_turn();
        }
        self.state.retained_cards = self.state.hand.clone();

        // on_retain hooks for retained cards
        let establishment = self.state.player.status(sid::ESTABLISHMENT);
        for card_inst in self.state.hand.iter_mut() {
            let card_def = self.card_registry.card_def_by_id(card_inst.def_id);

            // Establishment uses Java's modifyCostForCombat semantics, so the
            // retained-card discount persists across turns instead of being a
            // one-turn override that resets back to the printed cost.
            if establishment > 0 && (card_def.runtime_traits().retain || card_inst.is_retained()) {
                let current_cost = if card_inst.cost >= 0 {
                    card_inst.cost
                } else {
                    card_def.cost as i8
                };
                if current_cost >= 0 {
                    card_inst.set_permanent_cost((current_cost - establishment as i8).max(0));
                }
            }

            let (perseverance_bonus, windmill_bonus) =
                effects::card_runtime::apply_on_retain(card_inst, card_def);
            if perseverance_bonus > 0 {
                self.state
                    .player
                    .add_status(sid::PERSEVERANCE_BONUS, perseverance_bonus);
            }
            debug_assert_eq!(windmill_bonus, 0);
        }

        // RestoreRetainedCardsAction clears the one-turn `retain` marker after
        // onRetained has fired. Intrinsically self-retaining cards remain
        // retained through their CardDef trait on subsequent turns.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/RestoreRetainedCardsAction.java
        for card_inst in &mut self.state.hand {
            card_inst.set_retained(false);
        }

        // Trigger exhaust hooks for ethereal cards exhausted at end of turn
        for _ in 0..ethereal_exhausted {
            self.trigger_on_exhaust();
        }

        self.state.retained_cards = self.state.hand.clone();

        // ---- End-of-turn orb passives ----
        self.apply_orb_end_of_turn();
        if self.state.combat_over {
            return;
        }

        // Late end-of-turn runtime hooks that must happen after orb passives.
        self.emit_event(crate::effects::runtime::GameEvent::empty(
            crate::effects::trigger::Trigger::TurnEndPostOrbs,
        ));
        if self.state.combat_over {
            return;
        }

        // Constricted: deal Constricted damage to player at end of turn
        let constricted = self.state.player.status(sid::CONSTRICTED);
        if constricted > 0 {
            self.player_lose_hp_from_damage(constricted);
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
            // Decrement poison by 1
            let new_poison = player_poison - 1;
            self.state.player.set_status(sid::POISON, new_poison);
            self.player_lose_hp_from_damage(player_poison);
            if self.state.combat_over {
                return;
            }
        }

        // Check combat end (Omega may have killed enemies)
        if self.check_combat_end() {
            return;
        }

        // D50 parity fix: Java gates the ENTIRE `monsters.applyEndOfTurnPowers()`
        // pass (where VULN/WEAK/FRAIL/Intangible decrement happens) behind
        // `if (!skipMonsterTurn)`. Pre-fix Rust decremented debuffs even when
        // Vault skipped the enemy turn, so player-applied Vulnerable on enemies
        // wasted a turn under Vault. Now we gate debuff + Intangible decrement
        // inside the same branch as the enemy-turn execution.
        if self.state.skip_enemy_turn {
            // Vault skipped the enemy turn -- do NOT decrement debuffs or
            // Intangible; they persist into the next round. Matches Java.
        } else {
            combat_hooks::do_enemy_turns(self);

            // End of round: decrement debuffs on player (enemy debuffs stick
            // one turn; the justApplied flag covers the apply-turn skip per D59).
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
                self.state
                    .player
                    .set_status(sid::INTANGIBLE, intangible - 1);
            }
        }

        // Java power atEndOfRound hooks run after the enemy turn (including
        // when Vault skips that turn), not with the player's TurnEnd hooks.
        self.emit_event(crate::effects::runtime::GameEvent::empty(
            crate::effects::trigger::Trigger::RoundEnd,
        ));

        // Check combat end
        if !self.check_combat_end() {
            self.start_player_turn();
        }
    }

    // =======================================================================
    // Card Play
    // =======================================================================

    fn velvet_choker_allows_play(&self) -> bool {
        let Some(slot) = self
            .state
            .relics
            .iter()
            .position(|relic| matches!(relic.as_str(), "Velvet Choker" | "VelvetChoker"))
        else {
            return true;
        };
        self.hidden_effect_value(
            "Velvet Choker",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: slot as u16 },
            0,
        ) < 6
    }

    fn can_play_card_inst(&self, card: &CardDef, card_inst: CardInstance) -> bool {
        // Unplayable cards -- unless Medical Kit (Status) or Blue Candle (Curse)
        if card.cost == -2 || card.runtime_traits().unplayable {
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
        if !self.velvet_choker_allows_play() {
            return false;
        }

        // Normality.canPlay is polled from every card in hand and rejects the
        // candidate once cardsPlayedThisTurn.size() reaches three.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Normality.java
        if self.state.cards_played_this_turn >= 3 {
            let has_normality = self.state.hand.iter().any(|c| {
                let def = self.card_registry.card_def_by_id(c.def_id);
                def.runtime_traits().limit_cards_per_turn
            });
            if has_normality {
                return false;
            }
        }

        // ConfusionPower.java randomizes the instance when it is drawn, so
        // ordinary energy legality uses that already-materialized cost.
        let cost = self.effective_cost_inst(card, card_inst);
        if cost > self.state.energy {
            return false;
        }

        // Entangled: can't play attacks
        if self.state.player.status(sid::ENTANGLED) > 0 && card.card_type == CardType::Attack {
            return false;
        }

        if !self.card_runtime_allows_play(card, card_inst) {
            return false;
        }

        true
    }

    fn card_matches_filter_for_choice(&self, card: &CardInstance, filter: CardFilter) -> bool {
        let def = self.card_registry.card_def_by_id(card.def_id);
        match filter {
            CardFilter::All => true,
            CardFilter::NonExhume => {
                def.id.strip_suffix('+').unwrap_or(def.id) != "Exhume"
            }
            CardFilter::Attacks => def.card_type == CardType::Attack,
            CardFilter::AttackOrPower => {
                matches!(def.card_type, CardType::Attack | CardType::Power)
            }
            CardFilter::Skills => def.card_type == CardType::Skill,
            CardFilter::NonAttacks => def.card_type != CardType::Attack,
            CardFilter::ZeroCost => {
                let cost = if card.cost >= 0 {
                    card.cost as i32
                } else {
                    def.cost
                };
                cost == 0 || card.is_free()
            }
            CardFilter::Upgradeable => {
                !card.is_upgraded()
                    && self.card_registry.get(&format!("{}+", def.id)).is_some()
            }
        }
    }

    fn available_choice_cards_in_pile(&self, source: Pile, filter: CardFilter) -> usize {
        let pile = match source {
            Pile::Hand => &self.state.hand,
            Pile::Draw => &self.state.draw_pile,
            Pile::Discard => &self.state.discard_pile,
            Pile::Exhaust => &self.state.exhaust_pile,
        };
        pile.iter()
            .filter(|card| self.card_matches_filter_for_choice(card, filter))
            .count()
    }

    fn choice_min_picks_for_legality(
        &self,
        card: &CardDef,
        card_inst: CardInstance,
        min_picks: AmountSource,
    ) -> i32 {
        let ctx = CardPlayContext {
            card,
            card_inst,
            target_idx: -1,
            x_value: 0,
            pen_nib_active: false,
            vigor: 0,
            total_unblocked_damage: 0,
            enemy_killed: false,
            hand_size_at_play: self.state.hand.len().saturating_sub(1),
            last_bulk_count: 0,
            last_drawn_card_types: Vec::new(),
        };
        crate::effects::interpreter::resolve_card_amount(self, &ctx, &min_picks).max(0)
    }

    fn card_runtime_allows_play(&self, card: &CardDef, card_inst: CardInstance) -> bool {
        if !effects::card_runtime::allows_play(self, card, card_inst) {
            return false;
        }
        for effect in card.effect_data {
            if let Effect::ChooseCards {
                source,
                filter,
                min_picks,
                ..
            } = effect
            {
                if matches!(source, Pile::Draw | Pile::Discard | Pile::Exhaust) {
                    let min_required =
                        self.choice_min_picks_for_legality(card, card_inst, *min_picks);
                    // BetterDiscardPileToHandAction is a legal no-op when the
                    // discard pile is empty; Hologram itself has no canUse
                    // restriction tied to that pile.
                    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/BetterDiscardPileToHandAction.java
                    let empty_source_is_allowed = matches!(
                        card.id,
                        "Exhume" | "Exhume+" | "Hologram" | "Hologram+"
                    );
                    if min_required > 0
                        && !empty_source_is_allowed
                        && self.available_choice_cards_in_pile(*source, *filter)
                            < min_required as usize
                    {
                        return false;
                    }
                }
            }
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
        if card.card_type == CardType::Attack
            && card_inst.flags & CardInstance::FLAG_PURGE == 0
            && self.state.player.status(sid::NEXT_ATTACK_FREE) > 0
        {
            return 0;
        }
        if card.card_type == CardType::Skill && self.state.player.status(sid::CORRUPTION) > 0 {
            return 0;
        }
        // Instance cost overrides CardDef cost when set (>= 0)
        let mut cost = if card_inst.cost >= 0 {
            card_inst.cost as i32
        } else {
            card.cost
        };

        // Establishment: cost already physically reduced in end_turn on_retain loop.
        // Do NOT reduce again here to avoid double-dipping.

        cost = self.apply_card_runtime_cost_modifiers(card, card_inst, cost);

        cost
    }

    /// Effective cost for actual card play.
    /// Confusion has already randomized the instance in draw_cards.
    fn effective_cost_mut_inst(&mut self, card: &CardDef, card_inst: CardInstance) -> i32 {
        // X-cost cards: cost is consumed separately in card_effects
        if card.cost == -1 {
            return 0;
        }

        // Free overrides
        if card_inst.is_free() {
            return 0;
        }
        if card.card_type == CardType::Attack && self.state.player.status(sid::NEXT_ATTACK_FREE) > 0
        {
            return 0;
        }
        if card.card_type == CardType::Skill && self.state.player.status(sid::CORRUPTION) > 0 {
            return 0;
        }
        // Instance cost overrides CardDef cost when set (>= 0)
        let mut cost = if card_inst.cost >= 0 {
            card_inst.cost as i32
        } else {
            card.cost
        };

        // Establishment: cost already physically reduced in end_turn on_retain loop.
        // Do NOT reduce again here to avoid double-dipping.

        cost = self.apply_card_runtime_cost_modifiers(card, card_inst, cost);

        cost
    }

    fn apply_card_runtime_cost_modifiers(
        &self,
        card: &CardDef,
        card_inst: CardInstance,
        base_cost: i32,
    ) -> i32 {
        effects::card_runtime::apply_cost_modifiers(self, card, card_inst, base_cost)
    }

    pub(crate) fn play_card(&mut self, hand_idx: usize, target_idx: i32) {
        if hand_idx >= self.state.hand.len() {
            return;
        }

        let card_inst = self.state.hand[hand_idx]; // Copy, no clone needed
        let card = self.card_registry.card_def_by_id(card_inst.def_id).clone();

        if !self.can_play_card_inst(&card, card_inst) {
            return;
        }

        // Medical Kit: Status cards are played for free and exhausted
        if card.card_type == CardType::Status
            && (self.state.has_relic("Medical Kit") || self.state.has_relic("MedicalKit"))
        {
            let choke_hp_losses = self.capture_enemy_choke_hp_losses();
            self.state.hand.remove(hand_idx);
            self.state.cards_played_this_turn += 1;
            self.state.total_cards_played += 1;
            let pain_killed = status_effects::process_pain_on_card_play(self);
            self.state.exhaust_pile.push(card_inst);
            self.trigger_card_on_exhaust(card_inst);
            if pain_killed {
                self.phase = CombatPhase::CombatOver;
                return;
            }
            self.resolve_enemy_choke_hp_losses(choke_hp_losses);
            {
                let ctx = crate::effects::trigger::TriggerContext {
                    card_type: Some(card.card_type),
                    is_first_turn: self.state.turn == 1,
                    target_idx,
                };
                self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
                    crate::effects::trigger::Trigger::OnAnyCardPlayed,
                    &ctx,
                ));
            }
            return;
        }

        // Blue Candle: Curse cards are played for free, exhaust, and deal 1 HP
        if card.card_type == CardType::Curse
            && (self.state.has_relic("Blue Candle") || self.state.has_relic("BlueCandle"))
        {
            let choke_hp_losses = self.capture_enemy_choke_hp_losses();
            self.state.hand.remove(hand_idx);
            self.state.cards_played_this_turn += 1;
            self.state.total_cards_played += 1;
            let pain_killed = status_effects::process_pain_on_card_play(self);
            self.state.exhaust_pile.push(card_inst);
            self.trigger_card_on_exhaust(card_inst);
            if pain_killed {
                self.phase = CombatPhase::CombatOver;
                return;
            }
            // BlueCandle.java queues LoseHPAction(1), which uses HP_LOSS damage
            // rather than directly subtracting HP. Preserve the shared
            // Intangible/Buffer/Tungsten Rod pipeline.
            self.player_lose_hp_from_damage(1);
            self.resolve_enemy_choke_hp_losses(choke_hp_losses);
            {
                let ctx = crate::effects::trigger::TriggerContext {
                    card_type: Some(card.card_type),
                    is_first_turn: self.state.turn == 1,
                    target_idx,
                };
                self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
                    crate::effects::trigger::Trigger::OnAnyCardPlayed,
                    &ctx,
                ));
            }
            if self.state.combat_over {
                return;
            }
            return;
        }

        // Pay energy (use RNG-aware version for Confusion randomization)
        let cost = self.effective_cost_mut_inst(&card, card_inst);
        self.state.energy -= cost;

        // Remove from hand
        self.state.hand.remove(hand_idx);

        // Carry the played instance through the rest of the play pipeline so
        // self-mutating cards can write back to the exact copy that is about to
        // land in discard/exhaust.
        self.begin_runtime_play_context(card_inst, target_idx);

        // Track counters
        self.state.cards_played_this_turn += 1;
        self.state.total_cards_played += 1;
        if card.card_type == CardType::Attack {
            self.state.attacks_played_this_turn += 1;
        } else if card.card_type == CardType::Power {
            // ForceField.triggerOnCardPlayed permanently reduces every copy in
            // hand/draw/discard once per POWER card. Tracking the history also
            // implements configureCostsOnNewCard for later-created copies.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ForceField.java
            self.state.power_cards_played_this_combat += 1;
        }

        {
            let play_ctx = crate::effects::trigger::TriggerContext {
                card_type: Some(card.card_type),
                is_first_turn: self.state.turn == 1,
                target_idx,
            };
            self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
                crate::effects::trigger::Trigger::OnPlayCard,
                &play_ctx,
            ));
        }

        // ---- Java onUseCard hooks (fire BEFORE card effects resolve) ----
        // AfterImage and other pre-play triggers via unified dispatch
        {
            let pre_ctx = crate::effects::trigger::TriggerContext {
                card_type: Some(card.card_type),
                is_first_turn: self.state.turn == 1,
                target_idx,
            };
            let event = crate::effects::runtime::GameEvent::from_trigger(
                crate::effects::trigger::Trigger::OnCardPlayedPre,
                &pre_ctx,
            );
            self.emit_event(event);
            self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
                crate::effects::trigger::Trigger::OnUseCard,
                &pre_ctx,
            ));
            if card.card_type == CardType::Power {
                // HeatsinkPower/StormPower/CuriosityPower react from the set
                // that existed before the played Power's use() resolves. A
                // newly played Heatsinks or Storm must not trigger itself.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/HeatsinkPower.java
                self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
                    crate::effects::trigger::Trigger::OnPowerPlayed,
                    &pre_ctx,
                ));
            }
        }

        // AbstractPlayer.useCard invokes hand.triggerOnOtherCardPlayed after
        // the card's actions are queued. Pain adds LoseHPAction to the top, so
        // each copy resolves after onPlayCard hooks but before card effects.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Pain.java
        let pain_killed = status_effects::process_pain_on_card_play(self);
        if pain_killed {
            self.phase = CombatPhase::CombatOver;
            self.clear_runtime_play_contexts();
            return;
        }

        // Execute effects (last_card_type refers to card played BEFORE this one)
        self.execute_card_effects_with_enemy_on_use(&card, card_inst, target_idx);

        // Update last_card_type AFTER effects (so next card sees this one)
        self.state.last_card_type = Some(card.card_type);

        // Stance actions queued after an interactive card action must wait for
        // that choice to resolve. Meditate queues ChangeStanceAction after
        // MeditateAction in Java.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Meditate.java
        if let Some(stance_name) = card.enter_stance {
            let new_stance = Stance::from_str(stance_name);
            if self.phase == CombatPhase::AwaitingChoice {
                if let Some(choice) = self.choice.as_mut() {
                    choice.deferred_stance = Some(new_stance);
                }
            } else {
                self.change_stance(new_stance);
            }
        }

        // Gremlin Nob Enrage / Anger: gains Strength when player plays a SKILL.
        // D64 parity fix: Java AngerPower.java:37-42 gates on `card.type == SKILL`.
        // Pre-fix Rust used `!= Attack`, which wrongly fired on Powers too
        // (Inflame, Demon Form, Establishment, Battle Hymn all triggered
        // Gremlin Nob's Strength buff even though Java treats them as Powers,
        // not Skills). Net: Nob was ~50% more dangerous in a power-heavy deck.
        if card.card_type == CardType::Skill {
            for enemy in &mut self.state.enemies {
                if enemy.is_alive() {
                    let enrage = enemy.entity.status(sid::ENRAGE);
                    if enrage > 0 {
                        enemy.entity.add_status(sid::STRENGTH, enrage);
                    }
                }
            }
        }

        // Hex: when player plays a non-Attack card, add 1 Dazed to draw pile
        if card.card_type != CardType::Attack {
            let hex = self.state.player.status(sid::HEX);
            if hex > 0 {
                for _ in 0..hex {
                    self.state
                        .draw_pile
                        .push(self.card_registry.make_card("Dazed"));
                }
            }
        }

        // Power ApplyPowerActions resolve before UseCardAction invokes
        // onUseCard. Rebuild here so newly installed powers such as Panache
        // can observe this same card in the post-card dispatch.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
        if card.card_type == CardType::Power {
            self.install_power(&card);
        }

        // ---- Post-effects dispatch: relic + power triggers ----
        // Unified dispatch handles: relic counters (Fan, Kunai, Shuriken, etc.),
        // AfterImage (OnAnyCardPlayed), Rage (OnAttackPlayed), Heatsink, Storm,
        // Beat of Death, Slow, Forcefield, SkillBurn.
        let turn_before_after_use = self.state.turn;
        {
            let post_ctx = crate::effects::trigger::TriggerContext {
                card_type: Some(card.card_type),
                is_first_turn: self.state.turn == 1,
                target_idx,
            };
            let post_event = crate::effects::runtime::GameEvent::from_trigger(
                crate::effects::trigger::Trigger::OnCardPlayedPost,
                &post_ctx,
            );
            self.emit_event(post_event);

            let any_event = crate::effects::runtime::GameEvent::from_trigger(
                crate::effects::trigger::Trigger::OnAnyCardPlayed,
                &post_ctx,
            );
            self.emit_event(any_event);
            // Type-specific triggers
            match card.card_type {
                CardType::Attack => {
                    let attack_event = crate::effects::runtime::GameEvent::from_trigger(
                        crate::effects::trigger::Trigger::OnAttackPlayed,
                        &post_ctx,
                    );
                    self.emit_event(attack_event);
                }
                CardType::Skill => {
                    let skill_event = crate::effects::runtime::GameEvent::from_trigger(
                        crate::effects::trigger::Trigger::OnSkillPlayed,
                        &post_ctx,
                    );
                    self.emit_event(skill_event);
                }
                CardType::Power => {}
                _ => {}
            }

            self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
                crate::effects::trigger::Trigger::OnAfterUseCard,
                &post_ctx,
            ));
        }
        if self.state.combat_over || self.state.turn != turn_before_after_use {
            self.clear_runtime_play_contexts();
            return;
        }
        if self.phase != CombatPhase::PlayerTurn {
            return;
        }
        self.resume_played_card_tail_from_runtime();
    }

    fn resume_played_card_tail_from_runtime(&mut self) {
        let mut card_inst = match self.runtime_played_card {
            Some(card) => card,
            None => return,
        };
        let target_idx = self.runtime_play_target_idx.unwrap_or(-1);
        let card = self.card_registry.card_def_by_id(card_inst.def_id).clone();
        if let Some(updated) = self.runtime_played_card {
            card_inst = updated;
        }

        let mut after_card_event = crate::effects::runtime::GameEvent::from_trigger(
            crate::effects::trigger::Trigger::OnAfterCardPlayed,
            &crate::effects::trigger::TriggerContext {
                card_type: Some(card.card_type),
                is_first_turn: self.state.turn == 1,
                target_idx,
            },
        );
        after_card_event.card_inst = Some(card_inst);
        self.emit_event(after_card_event);
        if self.state.combat_over || self.phase != CombatPhase::PlayerTurn {
            self.clear_runtime_play_contexts();
            return;
        }

        {
            self.runtime_replay_window = true;
            let mut runtime = std::mem::take(&mut self.effect_runtime);
            runtime.emit_replay_window(self, card.card_type, target_idx, card_inst);
            self.effect_runtime = runtime;
        }
        self.runtime_replay_window = false;

        if let Some(updated) = self.runtime_played_card {
            card_inst = updated;
        }

        if card.card_type == CardType::Attack && self.state.player.status(sid::NEXT_ATTACK_FREE) > 0
        {
            // FreeAttackPower.onUseCard decrements one stack and removes the
            // power only at zero, so stacked Swivels cover stacked attacks.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/FreeAttackPower.java
            self.state.player.add_status(sid::NEXT_ATTACK_FREE, -1);
        }

        if !self.state.combat_over {
            let is_attack = card.card_type == CardType::Attack;
            let effective = self.effective_cost_inst(&card, card_inst);
            let meets_cost_threshold = if card.cost == -1 {
                self.runtime_last_x_energy_on_use >= 2
            } else {
                effective >= 2
            };
            if self.state.has_relic("Necronomicon")
                && is_attack
                && meets_cost_threshold
                && self.state.player.status(sid::NECRONOMICON_USED) == 0
            {
                self.state.player.set_status(sid::NECRONOMICON_USED, 1);
                // Necronomicon.java queues a same-instance copy with the
                // original energyOnUse, target, and autoplay/free flags.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Necronomicon.java
                if card.cost == -1 {
                    self.runtime_x_energy_override = Some(self.runtime_last_x_energy_on_use);
                }
                self.execute_card_effects_with_enemy_on_use(
                    &card,
                    card_inst.set_free(true),
                    target_idx,
                );
            }
        }

        let post_play_dest = effects::card_runtime::post_play_destination(&card);

        let exhausts_on_use = card.exhaust
            || card_inst.flags & CardInstance::FLAG_EXHAUST_ON_USE != 0
            || (card.card_type == CardType::Skill
                && self.state.player.status(sid::CORRUPTION) > 0);
        let spoon_saved = card_inst.flags & CardInstance::FLAG_PURGE == 0
            && card.card_type != CardType::Power
            && exhausts_on_use
            && (self.state.has_relic("Strange Spoon")
                || self.state.has_relic("StrangeSpoon"))
            // UseCardAction sets spoonProc from exactly one
            // cardRandomRng.randomBoolean(), then follows the ordinary post-use
            // destination when true.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
            && self.card_random_rng.random_boolean();

        // UseCardAction clears freeToPlayOnce before moving the card to its
        // post-play pile, so Forethought's flag applies to exactly one play.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
        let mut post_play_card = card_inst.set_free(false);
        // UseCardAction clears the one-use exhaust flag after choosing the
        // destination, including when Havoc or Omniscience supplied it.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
        post_play_card.flags &= !CardInstance::FLAG_EXHAUST_ON_USE;

        if card_inst.flags & CardInstance::FLAG_PURGE != 0 || card.card_type == CardType::Power {
            // purgeOnUse copies disappear after their effects. Power cards are
            // consumed by the normal power-play path regardless of exhaust.
        } else if exhausts_on_use && !spoon_saved {
            self.state.exhaust_pile.push(post_play_card);
            self.trigger_card_on_exhaust(post_play_card);
        } else if post_play_dest == crate::effects::types::PostPlayDestination::ShuffleIntoDraw {
            // Tantrum's UseCardAction calls hand.moveToDeck(card, true), which
            // delegates to CardGroup.addToRandomSpot and cardRandomRng. It does
            // not shuffle the existing draw pile.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
            // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
            if self.state.draw_pile.is_empty() {
                self.state.draw_pile.push(post_play_card);
            } else {
                let idx = self
                    .card_random_rng
                    .random_range(0, (self.state.draw_pile.len() - 1) as i32)
                    as usize;
                self.state.draw_pile.insert(idx, post_play_card);
            }
        } else {
            self.state.discard_pile.push(post_play_card);
        }

        self.finish_runtime_play_context();

        if post_play_dest == crate::effects::types::PostPlayDestination::EndTurn {
            self.end_turn();
            self.clear_runtime_play_contexts();
            return;
        }

        // UnceasingTop.java::onRefreshHand queues one draw only when drawing is
        // allowed; in particular, No Draw suppresses the trigger entirely.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/UnceasingTop.java
        if (self.state.has_relic("Unceasing Top") || self.state.has_relic("UnceasingTop"))
            && self.state.hand.is_empty()
            && self.state.player.status(sid::NO_DRAW) <= 0
            && (!self.state.draw_pile.is_empty() || !self.state.discard_pile.is_empty())
        {
            self.draw_cards(1);
        }

        self.check_combat_end();
    }

    /// Install a power card as a permanent status effect.
    fn install_power(&mut self, card: &CardDef) {
        let _ = card;
        self.rebuild_effect_runtime();
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

        // Potion slots can change through rewards, test setup, or direct state
        // mutation; rebuild so runtime-owned manual activations see the live slot.
        self.rebuild_effect_runtime();

        let potion_id = self.state.potions[potion_idx].clone();
        let can_use_runtime =
            crate::potions::defs::potion_uses_runtime_manual_activation(&potion_id);
        let success = if can_use_runtime {
            if !potions::potion_can_use_in_combat(&self.state, &potion_id) {
                return;
            }
            if !self.effect_runtime.has_instance(
                &potion_id,
                crate::effects::runtime::EffectOwner::PotionSlot {
                    slot: potion_idx as u8,
                },
            ) {
                self.rebuild_effect_runtime();
            }
            if potions::potion_requires_target(&potion_id)
                && (target_idx < 0 || (target_idx as usize) >= self.state.enemies.len())
            {
                false
            } else {
                self.emit_event(crate::effects::runtime::GameEvent {
                    kind: crate::effects::trigger::Trigger::ManualActivation,
                    card_type: None,
                    card_inst: None,
                    is_first_turn: self.state.turn == 1,
                    target_idx,
                    enemy_idx: target_idx,
                    potion_slot: potion_idx as i32,
                    status_id: None,
                    amount: 0,
                    replay_window: false,
                });
                true
            }
        } else {
            false
        };

        if success {
            self.emit_event(crate::effects::runtime::GameEvent {
                kind: crate::effects::trigger::Trigger::OnPotionUsed,
                card_type: None,
                card_inst: None,
                is_first_turn: self.state.turn == 1,
                target_idx,
                enemy_idx: target_idx,
                potion_slot: potion_idx as i32,
                status_id: None,
                amount: 0,
                replay_window: false,
            });

            // Entropic Brew destroys itself before its queued ObtainPotionActions
            // resolve, so its own slot can already contain a replacement here.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/ui/panels/PotionPopUp.java
            // and actions/common/ObtainPotionAction.java
            let entropic_refilled_own_slot = matches!(potion_id.as_str(), "EntropicBrew" | "Entropic Brew")
                && !self.state.has_relic("Sozu");
            if !entropic_refilled_own_slot {
                self.state.potions[potion_idx] = String::new();
            }
            self.rebuild_effect_runtime();

            // Consume potion draw (Swift Potion, etc.)
            let pd = self.state.player.status(sid::POTION_DRAW);
            if pd > 0 {
                self.state.player.set_status(sid::POTION_DRAW, 0);
                self.draw_cards(pd);
            }
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
        // incrementDiscard updates the counter and every Eviscerate before
        // queued manual-discard hooks such as Reflex resolve.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
        self.state.player.add_status(sid::DISCARDED_THIS_TURN, 1);
        effects::card_runtime::apply_stateful_cost_on_discard(self);

        let discard_effect = effects::card_runtime::apply_on_discard(self, card);

        if discard_effect.draw > 0 {
            self.draw_cards(discard_effect.draw);
        }

        if discard_effect.energy > 0 {
            self.state.energy += discard_effect.energy;
        }

        // Relic triggers via unified dispatch (Tough Bandages, Tingsha)
        {
            let ctx = crate::effects::trigger::TriggerContext::empty();
            self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
                crate::effects::trigger::Trigger::OnCardDiscard,
                &ctx,
            ));
        }
    }

    /// Called when a card is drawn into hand.
    fn on_card_drawn(&mut self, card: CardInstance) {
        effects::card_runtime::apply_on_draw(self, card);
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

        // JuggernautPower.onGainedBlock queues DamageRandomEnemyAction with
        // DamageType.THORNS for every positive block-gain event. MonsterGroup
        // selects through cardRandomRng even when only one target is alive.
        // Java: powers/JuggernautPower.java, actions/common/DamageRandomEnemyAction.java,
        // and monsters/MonsterGroup.java.
        let jugg = self.state.player.status(sid::JUGGERNAUT);
        if jugg > 0 {
            if let Some(target) = self.random_living_enemy() {
                self.deal_thorns_damage_to_enemy(target, jugg);
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
    pub(crate) fn player_lose_hp_from_damage(&mut self, amount: i32) {
        if amount <= 0 {
            return;
        }

        // LoseHPAction resolves as HP_LOSS DamageInfo. AbstractPlayer.damage
        // applies Intangible first, skips block for HP_LOSS, then lets Buffer
        // reduce positive damage to zero before Tungsten Rod's onLoseHpLast.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/LoseHPAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BufferPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/TungstenRod.java
        let intangible = self.state.player.status(sid::INTANGIBLE) > 0;
        let after_intangible = damage::apply_hp_loss(amount, intangible, false);
        let buffer = self.state.player.status(sid::BUFFER);
        if after_intangible > 0 && buffer > 0 {
            self.state.player.set_status(sid::BUFFER, buffer - 1);
            return;
        }

        let tungsten = self.state.has_relic("Tungsten Rod")
            || self.state.has_relic("TungstenRod");
        let hp_loss = damage::apply_hp_loss(after_intangible, false, tungsten);
        self.player_lose_hp(hp_loss);
    }

    pub fn player_lose_hp(&mut self, amount: i32) {
        if amount <= 0 {
            return;
        }
        self.state.player.hp -= amount;
        self.state.total_damage_taken += amount;

        self.update_cards_on_damage();

        // AbstractPlayer.damagedThisCombat increments once per positive damage
        // event, regardless of HP amount; BloodForBlood.tookDamage() reduces
        // cost once per event. The legacy status name stores that event count.
        // Source: AbstractPlayer.java::damage/updateCardsOnDamage and
        // cards/red/BloodForBlood.java.
        self.state
            .player
            .add_status(sid::HP_LOSS_THIS_COMBAT, 1);

        // Fire on_hp_loss relics via unified dispatch (Centennial Puzzle, Self-Forming Clay, Runic Cube, Red Skull)
        {
            let ctx = crate::effects::trigger::TriggerContext::empty();
            self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
                crate::effects::trigger::Trigger::OnPlayerHpLoss,
                &ctx,
            ));
        }

        // Rupture: gain Strength when losing HP
        let rupture = self.state.player.status(sid::RUPTURE);
        if rupture > 0 {
            self.state.player.add_status(sid::STRENGTH, rupture);
        }

        let rcd = self.state.player.status(sid::RUNIC_CUBE_DRAW);
        if rcd > 0 {
            self.state.player.set_status(sid::RUNIC_CUBE_DRAW, 0);
            self.draw_cards(1);
        }
        // Fairy revive check
        if self.state.player.hp <= 0 {
            self.check_fairy_revive();
        }
    }

    fn update_cards_on_damage(&mut self) {
        // AbstractPlayer.updateCardsOnDamage invokes tookDamage once on every
        // card currently in hand, discard, and draw (not exhaust), independent
        // of the damage amount. MasterfulStab.tookDamage calls updateCost(1),
        // which raises both permanent and turn cost while preserving their delta.
        // Java: characters/AbstractPlayer.java, cards/green/MasterfulStab.java,
        // and cards/AbstractCard.java::updateCost.
        let registry = self.card_registry;
        let update_pile = |pile: &mut Vec<CardInstance>| {
            for card in pile {
                let def = registry.card_def_by_id(card.def_id);
                let increases_on_damage = def.runtime_triggers().iter().any(|trigger| {
                    matches!(
                        trigger,
                        crate::effects::types::CardRuntimeTrigger::ModifyCost(
                            crate::effects::types::CostModifierRule::IncreaseOnHpLoss
                        )
                    )
                });
                if !increases_on_damage {
                    continue;
                }

                let permanent_cost = if card.base_cost >= 0 {
                    card.base_cost
                } else {
                    def.cost as i8
                };
                let cost_for_turn = if card.cost >= 0 {
                    card.cost
                } else {
                    permanent_cost
                };
                let difference = permanent_cost - cost_for_turn;
                let new_permanent = permanent_cost.saturating_add(1).max(0);
                card.base_cost = new_permanent;
                card.cost = new_permanent.saturating_sub(difference).max(0);
            }
        };

        update_pile(&mut self.state.hand);
        update_pile(&mut self.state.discard_pile);
        update_pile(&mut self.state.draw_pile);
    }

    /// Centralized healing: delegates to CombatState::heal_player.
    pub fn heal_player(&mut self, amount: i32) {
        self.state.heal_player(amount);
        if self.state.has_relic("Red Skull")
            && self.hidden_effect_value(
                "Red Skull",
                crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
                0,
            ) > 0
            && self.state.player.hp > self.state.player.max_hp / 2
        {
            self.state.player.add_status(sid::STRENGTH, -3);
            let _ = self.effect_runtime.set_hidden_value(
                "Red Skull",
                crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
                0,
                0,
            );
        }
    }

    /// Apply AbstractPlayer.gainGold while combat owns the live run state.
    pub(crate) fn gain_run_gold(&mut self, amount: i32) {
        // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
        if amount <= 0 || self.state.has_relic("Ectoplasm") {
            return;
        }
        self.state.run_gold += amount;
        self.state.pending_run_gold += amount;
        if self.state.has_relic("Bloody Idol") || self.state.has_relic("BloodyIdol") {
            // BloodyIdol.java::onGainGold heals once per gainGold call.
            self.heal_player(5);
        }
    }

    pub(crate) fn apply_player_debuff_to_enemy(
        &mut self,
        enemy_idx: usize,
        status: crate::ids::StatusId,
        amount: i32,
    ) -> bool {
        if enemy_idx >= self.state.enemies.len() {
            return false;
        }

        let mut applied_amount = amount;
        if status == sid::POISON {
            if self.state.has_relic("Snake Skull") || self.state.has_relic("SneckoSkull") {
                applied_amount += 1;
            }
        }

        let applied = powers::apply_debuff(
            &mut self.state.enemies[enemy_idx].entity,
            status,
            applied_amount,
        );
        if applied {
            self.emit_event(crate::effects::runtime::GameEvent {
                kind: crate::effects::trigger::Trigger::OnDebuffApplied,
                card_type: None,
                card_inst: self.runtime_played_card,
                is_first_turn: self.state.turn == 1,
                target_idx: enemy_idx as i32,
                enemy_idx: enemy_idx as i32,
                potion_slot: -1,
                status_id: Some(status),
                amount: applied_amount,
                replay_window: self.runtime_replay_window,
            });
        }
        if applied && status == sid::VULNERABLE && self.state.has_relic("Champion Belt") {
            let extra_applied =
                powers::apply_debuff(&mut self.state.enemies[enemy_idx].entity, sid::WEAKENED, 1);
            if extra_applied {
                self.emit_event(crate::effects::runtime::GameEvent {
                    kind: crate::effects::trigger::Trigger::OnDebuffApplied,
                    card_type: None,
                    card_inst: self.runtime_played_card,
                    is_first_turn: self.state.turn == 1,
                    target_idx: enemy_idx as i32,
                    enemy_idx: enemy_idx as i32,
                    potion_slot: -1,
                    status_id: Some(sid::WEAKENED),
                    amount: 1,
                    replay_window: self.runtime_replay_window,
                });
            }
        }
        applied
    }

    /// Check and apply revive effects (Fairy in a Bottle, Lizard Tail).
    fn check_fairy_revive(&mut self) {
        if self.state.player.hp <= 0 {
            // Java: AbstractPlayer.damage() does not invoke Fairy in a Bottle
            // or Lizard Tail while Mark of the Bloom is present.
            // Source: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
            let healing_blocked = self.state.player.status(sid::HAS_MARK_OF_BLOOM) > 0;
            // Fairy in a Bottle (potion)
            let revive_hp = if healing_blocked {
                0
            } else {
                potions::check_fairy_revive(&self.state)
            };
            if revive_hp > 0 {
                potions::consume_fairy(&mut self.state);
                self.state.player.hp = 0;
                // FairyPotion.use calls player.heal, so healing modifiers such
                // as Magic Flower apply after the percent amount is computed.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FairyPotion.java
                self.heal_player(revive_hp);
                return;
            }
            // Lizard Tail (relic): revive at 50% max HP, once per run
            if !healing_blocked
                && (self.state.has_relic("Lizard Tail") || self.state.has_relic("LizardTail"))
                && self.state.player.status(sid::LIZARD_TAIL_USED) == 0
            {
                // LizardTail.java::onTrigger calls player.heal(maxHealth / 2,
                // true), clamped to at least 1, then permanently uses up the
                // relic. RunEngine persists this status between combats; use
                // the normal heal path so modifiers such as Magic Flower apply.
                self.state.player.set_status(sid::LIZARD_TAIL_USED, 1);
                self.state.player.hp = 0;
                let heal_amount = (self.state.player.max_hp / 2).max(1);
                self.heal_player(heal_amount);
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

    /// Kill a target without routing through ordinary damage modifiers.
    ///
    /// Java's InstantKillAction sets currentHealth to zero, then calls
    /// damage(0, HP_LOSS) so monster death hooks run. Block, Flight, Slow,
    /// Invincible, and on-attacked powers do not modify the kill, and the
    /// removed HP is not counted as dealt damage.
    /// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/InstantKillAction.java
    pub(crate) fn instant_kill_enemy(&mut self, enemy_idx: usize) -> bool {
        if enemy_idx >= self.state.enemies.len()
            || self.state.enemies[enemy_idx].entity.hp <= 0
            || self.state.enemies[enemy_idx].is_escaping
        {
            return false;
        }

        let hp_before = self.state.enemies[enemy_idx].entity.hp;
        self.state.enemies[enemy_idx].entity.hp = 0;

        // Awakened One's damage override intercepts its first death and starts
        // the rebirth sequence even for InstantKillAction.
        let awakened_phase_one = matches!(
            self.state.enemies[enemy_idx].id.as_str(),
            "AwakenedOne" | "Awakened One"
        ) && self.state.enemies[enemy_idx].entity.status(sid::PHASE) == 1;
        if awakened_phase_one {
            combat_hooks::on_enemy_damaged(self, enemy_idx, hp_before);
        } else {
            self.finalize_enemy_death(enemy_idx);
        }
        true
    }

    pub fn deal_damage_to_enemy(&mut self, enemy_idx: usize, damage: i32) {
        self.deal_precomputed_normal_damage_to_enemy(enemy_idx, damage, false);
    }

    /// Deal NORMAL damage whose output was precomputed with Java's
    /// `DamageInfo.createDamageMatrix(..., true)`. Offensive/receiving power
    /// modifiers are skipped, while block and on-attacked hooks still resolve.
    pub(crate) fn deal_pure_normal_damage_to_enemy(&mut self, enemy_idx: usize, damage: i32) {
        self.deal_precomputed_normal_damage_to_enemy(enemy_idx, damage, true);
    }

    fn deal_precomputed_normal_damage_to_enemy(
        &mut self,
        enemy_idx: usize,
        damage: i32,
        pure_matrix: bool,
    ) {
        let hp_before = self.state.enemies[enemy_idx].entity.hp;
        let has_boot = self.state.has_relic("Boot");
        let enemy = &mut self.state.enemies[enemy_idx];

        // Slow: enemies with Slow take 10% more damage per card played this turn
        let damage_after_slow = if pure_matrix {
            damage
        } else {
            let slow_mult = powers::slow_damage_multiplier(&enemy.entity);
            (damage as f64 * slow_mult) as i32
        };

        // Flight: halve incoming damage while Flight > 0, decrement per hit
        let flight = enemy.entity.status(sid::FLIGHT);
        let effective_damage = if flight > 0 && !pure_matrix {
            enemy.entity.set_status(sid::FLIGHT, flight - 1);
            (damage_after_slow as f64 * 0.5) as i32
        } else {
            damage_after_slow
        };

        let blocked = enemy.entity.block.min(effective_damage);
        let mut hp_damage = effective_damage - blocked;
        enemy.entity.block -= blocked;
        // AbstractMonster.damage decrements Block before invoking player relic
        // onAttackToChangeDamage hooks. Boot.java therefore raises positive
        // post-block NORMAL damage below 5 to exactly 5.
        if has_boot && hp_damage > 0 && hp_damage < 5 {
            hp_damage = 5;
        }
        // InvinciblePower.onAttackedToChangeDamage runs after player relics.
        hp_damage = powers::apply_invincible_cap_tracked(&mut enemy.entity, hp_damage);
        enemy.entity.hp -= hp_damage;
        self.state.total_damage_dealt += hp_damage;

        // On-hit enemy reactions (only when HP damage dealt)
        if hp_damage > 0 {
            // Pure matrices skip Flight.atDamageFinalReceive, but the later
            // FlightPower.onAttacked hook still consumes one stack when its
            // half-damage survival check passes.
            if pure_matrix && flight > 0 && (hp_damage as f64 * 0.5) < hp_before as f64 {
                self.state.enemies[enemy_idx]
                    .entity
                    .set_status(sid::FLIGHT, flight - 1);
            }

            // D63 parity fix: Java CurlUpPower.onAttacked requires
            // `info.type == NORMAL && damageAmount < currentHealth && info.owner != null`.
            // This call site (`deal_damage_to_enemy`) is the NORMAL damage path --
            // THORNS / HP_LOSS go through `apply_hp_loss` or direct entity.hp writes,
            // not here -- so the type check is implicitly satisfied. The remaining
            // Java guard is the lethal-blow exemption: Curl Up should NOT trigger
            // when the hit is the killing blow (`damageAmount >= currentHealth`
            // pre-hit). Post-hit `entity.hp <= 0` is the same signal.
            let enemy_alive = self.state.enemies[enemy_idx].entity.hp > 0;

            // Curl-Up: first time hit, enemy gains block (non-lethal only).
            if enemy_alive {
                let curl_up = self.state.enemies[enemy_idx].entity.status(sid::CURL_UP);
                if curl_up > 0 {
                    self.state.enemies[enemy_idx].entity.block += curl_up;
                    self.state.enemies[enemy_idx]
                        .entity
                        .set_status(sid::CURL_UP, 0);
                }
            }

            // Malleable: gain escalating block on hit
            let malleable = self.state.enemies[enemy_idx].entity.status(sid::MALLEABLE);
            if malleable > 0 {
                self.state.enemies[enemy_idx].entity.block += malleable;
                self.state.enemies[enemy_idx]
                    .entity
                    .add_status(sid::MALLEABLE, 1);
            }

            // Shifting: gain block equal to unblocked damage
            let shifting = self.state.enemies[enemy_idx].entity.status(sid::SHIFTING);
            if shifting > 0 {
                self.state.enemies[enemy_idx].entity.block += hp_damage;
            }
        }

        self.record_enemy_hp_damage(enemy_idx, hp_damage);
    }

    pub(crate) fn record_enemy_hp_damage(&mut self, enemy_idx: usize, hp_damage: i32) {
        if hp_damage <= 0 || enemy_idx >= self.state.enemies.len() {
            return;
        }

        combat_hooks::on_enemy_damaged(self, enemy_idx, hp_damage);
        if self.state.enemies[enemy_idx].entity.hp <= 0
            && self.state.enemies[enemy_idx]
                .entity
                .status(sid::REBIRTH_PENDING)
                <= 0
        {
            self.state.enemies[enemy_idx].entity.hp = 0;
            self.finalize_enemy_death(enemy_idx);
        }
    }

    pub(crate) fn finalize_enemy_death(&mut self, enemy_idx: usize) {
        if enemy_idx >= self.state.enemies.len() {
            return;
        }

        // Source: decompiled GremlinFat/Thief/Warrior/Wizard/Tsundere.java
        // (`deathReact`). A surviving gremlin immediately changes to Escape.
        for (idx, enemy) in self.state.enemies.iter_mut().enumerate() {
            if idx != enemy_idx
                && matches!(enemy.id.as_str(),
                    "GremlinFat" | "GremlinThief" | "GremlinWarrior"
                        | "GremlinWizard" | "GremlinTsundere")
                && enemy.is_alive()
                && enemy.move_id != crate::enemies::move_ids::GREMLIN_ESCAPE
            {
                enemy.set_move(crate::enemies::move_ids::GREMLIN_ESCAPE, 0, 0, 0);
            }
        }

        // Spore Cloud: apply Vulnerable to player on death.
        let spore = self.state.enemies[enemy_idx]
            .entity
            .status(sid::SPORE_CLOUD);
        if spore > 0 {
            powers::apply_debuff(&mut self.state.player, sid::VULNERABLE, spore);
        }

        // Fire owner-aware death hooks (Gremlin Horn, The Specimen, etc.).
        let ctx = crate::effects::trigger::TriggerContext {
            card_type: None,
            is_first_turn: false,
            target_idx: enemy_idx as i32,
        };
        self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
            crate::effects::trigger::Trigger::OnEnemyDeath,
            &ctx,
        ));

        // CorpseExplosionPower.onDeath queues source-less THORNS damage equal
        // to owner.maxHealth * amount. THORNS bypasses NORMAL-only reactions.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CorpseExplosionPower.java
        let ce = self.state.enemies[enemy_idx]
            .entity
            .status(sid::CORPSE_EXPLOSION);
        if ce > 0 {
            let damage = self.state.enemies[enemy_idx].entity.max_hp * ce;
            let living = self.state.living_enemy_indices();
            for other_idx in living {
                if other_idx != enemy_idx {
                    self.deal_thorns_damage_to_enemy(other_idx, damage);
                }
            }
        }
    }

    pub(crate) fn deal_player_attack_hit_to_enemy(&mut self, enemy_idx: usize, damage: i32) -> i32 {
        if enemy_idx >= self.state.enemies.len() || !self.state.enemies[enemy_idx].is_alive() {
            return 0;
        }

        // BlockReturnPower.onAttacked runs after block is decremented but does
        // not require positive HP damage; every ordinary player attack hit
        // queues its block, including a fully blocked or zero-damage hit.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/BlockReturnPower.java
        let block_return = self.state.enemies[enemy_idx]
            .entity
            .status(sid::BLOCK_RETURN);
        let enemy_block_before = self.state.enemies[enemy_idx].entity.block;
        let hit_damage = damage;
        let block_broken = self.state.has_relic("HandDrill")
            && enemy_block_before > 0
            // Sources: AbstractCreature.java::decrementBlock calls
            // brokeBlock for both damage > block and damage == block;
            // HandDrill.java::onBlockBroken then applies 2 Vulnerable.
            && hit_damage >= enemy_block_before;
        let hp_before = self.state.enemies[enemy_idx].entity.hp;

        self.deal_damage_to_enemy(enemy_idx, hit_damage);

        if block_return > 0 {
            self.gain_block_player(block_return);
        }

        let hp_damage = (hp_before - self.state.enemies[enemy_idx].entity.hp).max(0);
        if hp_damage > 0 {
            self.emit_event(crate::effects::runtime::GameEvent {
                kind: crate::effects::trigger::Trigger::DamageResolved,
                card_type: Some(CardType::Attack),
                card_inst: None,
                is_first_turn: self.state.turn == 1,
                target_idx: enemy_idx as i32,
                enemy_idx: enemy_idx as i32,
                potion_slot: -1,
                status_id: None,
                amount: hp_damage,
                replay_window: false,
            });
        }

        if block_broken && self.state.enemies[enemy_idx].entity.block == 0 {
            self.apply_player_debuff_to_enemy(enemy_idx, sid::VULNERABLE, 2);
        }

        hp_damage
    }

    pub(crate) fn player_attack_base_damage(&self, card: &CardDef, card_inst: CardInstance) -> i32 {
        let mut damage = card.base_damage;
        if card.card_type != CardType::Attack {
            return damage;
        }

        // Source: decompiled/java-src/com/megacrit/cardcrawl/relics/StrikeDummy.java
        // atDamageModify adds exactly 3 only for cards carrying CardTags.STRIKE.
        if self.card_registry.is_strike(card_inst.def_id) && self.state.has_relic("StrikeDummy") {
            damage += 3;
        }

        let effective_cost = if card.cost == -1 {
            0
        } else if card_inst.is_free()
            || self.state.player.status(sid::NEXT_ATTACK_FREE) > 0
        {
            0
        } else if card_inst.cost >= 0 {
            card_inst.cost as i32
        } else {
            card.cost
        };
        if effective_cost == 0 && self.state.has_relic("WristBlade") {
            damage += 4;
        }

        damage
    }

    pub fn deal_damage_to_player(&mut self, damage: i32) {
        let player = &mut self.state.player;
        let blocked = player.block.min(damage);
        let hp_damage = damage - blocked;
        player.block -= blocked;

        if hp_damage > 0 {
            self.player_lose_hp(hp_damage);
        }
    }

    pub(crate) fn add_spawned_enemy(&mut self, mut enemy: crate::state::EnemyCombatState) {
        // Source: PhilosopherStone.java::onSpawnMonster grants the newly
        // spawned monster exactly 1 Strength before combat continues.
        if self.state.has_relic("Philosopher's Stone")
            || self.state.has_relic("PhilosopherStone")
        {
            enemy.entity.add_status(sid::STRENGTH, 1);
        }
        self.state.enemies.push(enemy);
    }

    pub(crate) fn deal_thorns_damage_to_player(&mut self, damage: i32) {
        // DamageInfo.THORNS still passes through final-receive powers, block,
        // Buffer, and Tungsten Rod. Torii and reactive Thorns/Flame Barrier
        // explicitly exclude THORNS damage.
        // Sources: AbstractPlayer.damage, IntangiblePlayerPower.java,
        // BufferPower.java, TungstenRod.java, and Torii.java.
        let mut incoming = damage.max(0);
        if self.state.player.status(sid::INTANGIBLE) > 0 && incoming > 1 {
            incoming = 1;
        }
        let blocked = self.state.player.block.min(incoming);
        self.state.player.block -= blocked;
        let mut hp_damage = incoming - blocked;
        if hp_damage > 0 {
            let buffer = self.state.player.status(sid::BUFFER);
            if buffer > 0 {
                self.state.player.set_status(sid::BUFFER, buffer - 1);
                hp_damage = 0;
            }
        }
        if hp_damage > 0
            && (self.state.has_relic("Tungsten Rod") || self.state.has_relic("TungstenRod"))
        {
            hp_damage -= 1;
        }
        if hp_damage > 0 {
            self.player_lose_hp(hp_damage);
        }
    }

    /// Deal DamageInfo.THORNS damage to an enemy after target-only power
    /// calculation, as used by Fire Potion. This intentionally bypasses
    /// NORMAL-only modifiers/reactions such as Slow, Vulnerable, Flight,
    /// Curl Up, and Malleable.
    pub(crate) fn deal_thorns_damage_to_enemy(&mut self, enemy_idx: usize, damage: i32) {
        if enemy_idx >= self.state.enemies.len() || !self.state.enemies[enemy_idx].is_alive() {
            return;
        }

        let mut incoming = damage.max(0);
        if self.state.enemies[enemy_idx].entity.status(sid::INTANGIBLE) > 0 && incoming > 1 {
            incoming = 1;
        }

        let blocked = self.state.enemies[enemy_idx].entity.block.min(incoming);
        self.state.enemies[enemy_idx].entity.block -= blocked;
        let mut hp_damage = incoming - blocked;

        if hp_damage > 0 {
            let buffer = self.state.enemies[enemy_idx].entity.status(sid::BUFFER);
            if buffer > 0 {
                self.state.enemies[enemy_idx]
                    .entity
                    .set_status(sid::BUFFER, buffer - 1);
                hp_damage = 0;
            }
        }
        if hp_damage > 0 {
            hp_damage = powers::apply_invincible_cap_tracked(
                &mut self.state.enemies[enemy_idx].entity,
                hp_damage,
            );
        }

        self.state.enemies[enemy_idx].entity.hp -= hp_damage;
        self.state.total_damage_dealt += hp_damage;

        // ShiftingPower.onAttacked applies to positive damage of every type.
        if hp_damage > 0 {
            let shifting = self.state.enemies[enemy_idx].entity.status(sid::SHIFTING);
            if shifting > 0 {
                self.state.enemies[enemy_idx].entity.block += hp_damage;
            }
        }

        self.record_enemy_hp_damage(enemy_idx, hp_damage);
    }

    /// Resolve a source-less `DamageInfo.HP_LOSS` hit against an enemy.
    ///
    /// HP_LOSS bypasses Block, but `AbstractMonster.damage` still applies
    /// Intangible and the target's `onAttackedToChangeDamage` powers (Buffer,
    /// then Invincible) before subtracting HP. ChokePower and MarkPower both
    /// reach this path through `LoseHPAction`.
    /// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/LoseHPAction.java
    /// Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java
    pub(crate) fn enemy_lose_hp_from_damage(&mut self, enemy_idx: usize, amount: i32) -> i32 {
        if enemy_idx >= self.state.enemies.len() || !self.state.enemies[enemy_idx].is_alive() {
            return 0;
        }

        let mut hp_damage = amount.max(0);
        if hp_damage > 0 && self.state.enemies[enemy_idx].entity.status(sid::INTANGIBLE) > 0 {
            hp_damage = 1;
        }

        if hp_damage > 0 {
            let buffer = self.state.enemies[enemy_idx].entity.status(sid::BUFFER);
            if buffer > 0 {
                self.state.enemies[enemy_idx].entity.set_status(sid::BUFFER, buffer - 1);
                hp_damage = 0;
            }
        }
        if hp_damage > 0 {
            hp_damage = powers::apply_invincible_cap_tracked(
                &mut self.state.enemies[enemy_idx].entity,
                hp_damage,
            );
        }

        let hp_before = self.state.enemies[enemy_idx].entity.hp;
        self.state.enemies[enemy_idx].entity.hp -= hp_damage;
        let actual_hp_lost = hp_damage.min(hp_before).max(0);
        self.state.total_damage_dealt += actual_hp_lost;

        // ShiftingPower.onAttacked applies to every positive damage type. Its
        // paired Strength loss/Shackled restoration is entirely blocked by
        // Artifact; otherwise record both the loss and its later restoration.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ShiftingPower.java
        if hp_damage > 0 && self.state.enemies[enemy_idx].entity.status(sid::SHIFTING) > 0 {
            let artifact = self.state.enemies[enemy_idx].entity.status(sid::ARTIFACT);
            if artifact > 0 {
                self.state.enemies[enemy_idx].entity.set_status(sid::ARTIFACT, artifact - 1);
            } else {
                self.state.enemies[enemy_idx].entity.add_status(sid::STRENGTH, -hp_damage);
                self.state.enemies[enemy_idx]
                    .entity
                    .add_status(sid::TEMP_STRENGTH_LOSS, hp_damage);
            }
        }

        self.record_enemy_hp_damage(enemy_idx, actual_hp_lost);
        actual_hp_lost
    }

    pub(crate) fn execute_card_effects_with_enemy_on_use(
        &mut self,
        card: &CardDef,
        card_inst: CardInstance,
        target_idx: i32,
    ) {
        // SharpHidePower.onUseCard queues one THORNS hit for every living
        // owner when each Attack card use is constructed, including replayed
        // and autoplayed cards. Capture before resolution so killing the
        // Guardian does not cancel its already-queued retaliation.
        // Sources: decompiled/java-src/com/megacrit/cardcrawl/powers/
        // SharpHidePower.java and actions/utility/UseCardAction.java.
        let sharp_hide_retaliations: Vec<i32> = if card.card_type == CardType::Attack {
            self.state.enemies.iter()
                .filter(|enemy| enemy.is_alive())
                .map(|enemy| enemy.entity.status(sid::SHARP_HIDE))
                .filter(|amount| *amount > 0)
                .collect()
        } else {
            Vec::new()
        };

        // ChokePower.onUseCard is invoked by the UseCardAction constructor
        // after card.use has queued the card's actions. Capture the powers that
        // existed at construction time, then resolve their LoseHPActions after
        // the card's own actions. A newly applied Choke therefore does not
        // trigger itself, while an existing stack does trigger on another Choke.
        // Sources: decompiled/java-src/com/megacrit/cardcrawl/powers/ChokePower.java
        // and actions/utility/UseCardAction.java.
        let choke_hp_losses = self.capture_enemy_choke_hp_losses();

        crate::card_effects::execute_card_effects(self, card, card_inst, target_idx);

        // card.use actions precede Sharp Hide's DamageAction in Java's queue.
        for amount in sharp_hide_retaliations {
            self.deal_thorns_damage_to_player(amount);
        }
        self.resolve_enemy_choke_hp_losses(choke_hp_losses);
    }

    fn capture_enemy_choke_hp_losses(&self) -> Vec<(usize, i32)> {
        self.state.enemies.iter()
            .enumerate()
            .filter(|(_, enemy)| enemy.is_alive())
            .filter_map(|(idx, enemy)| {
                let amount = enemy.entity.status(sid::CONSTRICTED);
                (amount > 0).then_some((idx, amount))
            })
            .collect()
    }

    fn resolve_enemy_choke_hp_losses(&mut self, pending: Vec<(usize, i32)>) {
        for (enemy_idx, amount) in pending {
            self.enemy_lose_hp_from_damage(enemy_idx, amount);
        }
    }

    // =======================================================================
    // Orb Effects
    // =======================================================================

    /// Pick a random living enemy index using cardRandomRng.
    pub(crate) fn random_living_enemy(&mut self) -> Option<usize> {
        let living = self.state.living_enemy_indices();
        if living.is_empty() {
            return None;
        }
        // AbstractDungeon.getRandomMonster delegates to MonsterGroup with
        // cardRandomRng and calls random(0, size - 1), even when size is one.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/MonsterGroup.java
        let roll = self.card_random_rng.random(living.len() as i32 - 1) as usize;
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

    /// AbstractOrb.applyLockOn raises Lightning and Dark orb damage by 50%,
    /// truncating the float result to an integer for each target independently.
    /// Java: decompiled/java-src/com/megacrit/cardcrawl/orbs/AbstractOrb.java.
    fn orb_damage_against(&self, enemy_idx: usize, damage: i32) -> i32 {
        if self.state.enemies[enemy_idx].entity.status(sid::LOCK_ON) > 0 {
            (damage as f32 * 1.5) as i32
        } else {
            damage
        }
    }

    /// Apply a single evoke effect to the game state.
    pub(crate) fn apply_evoke_effect(&mut self, effect: EvokeEffect) {
        match effect {
            EvokeEffect::LightningDamage(dmg) => {
                if self.effect_runtime.player_power_active(
                    self,
                    "electrodynamics",
                    sid::ELECTRODYNAMICS,
                ) {
                    for idx in self.state.living_enemy_indices() {
                        let orb_damage = self.orb_damage_against(idx, dmg);
                        self.deal_thorns_damage_to_enemy(idx, orb_damage);
                    }
                } else if let Some(idx) = self.random_living_enemy() {
                    let orb_damage = self.orb_damage_against(idx, dmg);
                    self.deal_thorns_damage_to_enemy(idx, orb_damage);
                }
            }
            EvokeEffect::FrostBlock(block) => {
                self.gain_block_player(block);
            }
            EvokeEffect::DarkDamage(dmg) => {
                if let Some(idx) = self.lowest_hp_enemy() {
                    let orb_damage = self.orb_damage_against(idx, dmg);
                    self.deal_thorns_damage_to_enemy(idx, orb_damage);
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
                if self.effect_runtime.player_power_active(
                    self,
                    "electrodynamics",
                    sid::ELECTRODYNAMICS,
                ) {
                    for idx in self.state.living_enemy_indices() {
                        let orb_damage = self.orb_damage_against(idx, dmg);
                        self.deal_thorns_damage_to_enemy(idx, orb_damage);
                    }
                } else if let Some(idx) = self.random_living_enemy() {
                    let orb_damage = self.orb_damage_against(idx, dmg);
                    self.deal_thorns_damage_to_enemy(idx, orb_damage);
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

    fn apply_all_orb_end_of_turn_passives(&mut self) {
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

    /// Trigger orb end-of-turn passives and apply their effects.
    fn apply_orb_end_of_turn(&mut self) {
        self.apply_all_orb_end_of_turn_passives();
        if self.state.has_relic("Cables") {
            self.apply_front_orb_end_of_turn_passive();
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

    fn apply_front_orb_start_of_turn_passive(&mut self) {
        if self.state.orb_slots.occupied_count() == 0 {
            return;
        }
        let front_orb = &self.state.orb_slots.slots[0];
        let effect = match front_orb.orb_type {
            crate::orbs::OrbType::Plasma => {
                crate::orbs::PassiveEffect::PlasmaEnergy(front_orb.base_passive)
            }
            _ => crate::orbs::PassiveEffect::None,
        };
        self.apply_passive_effect(effect);
    }

    fn apply_front_orb_end_of_turn_passive(&mut self) {
        if self.state.orb_slots.occupied_count() == 0 {
            return;
        }
        let focus = self.state.player.focus();
        let effect = {
            let front_orb = &mut self.state.orb_slots.slots[0];
            match front_orb.orb_type {
                crate::orbs::OrbType::Lightning => {
                    crate::orbs::PassiveEffect::LightningDamage(front_orb.passive_with_focus(focus))
                }
                crate::orbs::OrbType::Frost => {
                    crate::orbs::PassiveEffect::FrostBlock(front_orb.passive_with_focus(focus))
                }
                crate::orbs::OrbType::Dark => {
                    front_orb.evoke_amount += front_orb.passive_with_focus(focus);
                    crate::orbs::PassiveEffect::None
                }
                _ => crate::orbs::PassiveEffect::None,
            }
        };
        self.apply_passive_effect(effect);
    }

    pub(crate) fn trigger_dark_impulse(&mut self) {
        // DarkImpulseAction triggers every Dark orb's start/end callbacks, then
        // repeats both callbacks for the front Dark orb when Cables is owned.
        // Dark has no start callback, so each end callback adds one passive.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/DarkImpulseAction.java
        let focus = self.state.player.focus();
        for orb in self.state.orb_slots.slots.iter_mut() {
            if orb.orb_type == crate::orbs::OrbType::Dark {
                orb.evoke_amount += orb.passive_with_focus(focus);
            }
        }
        if self.state.has_relic("Cables") {
            if let Some(front_orb) = self.state.orb_slots.slots.first_mut() {
                if front_orb.orb_type == crate::orbs::OrbType::Dark {
                    front_orb.evoke_amount += front_orb.passive_with_focus(focus);
                }
            }
        }
    }

    pub(crate) fn trigger_orb_impulse(&mut self) {
        // ImpulseAction calls onStartOfTurn then onEndOfTurn on each orb in
        // slot order, then repeats both callbacks for the front orb with
        // Cables. Dark mutates synchronously while the other callbacks queue
        // effects, so collect those effects in callback order before resolving
        // them. A lethal Lightning effect clears later queued callbacks.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ImpulseAction.java
        let focus = self.state.player.focus();
        let mut effects = Vec::new();
        for orb in &mut self.state.orb_slots.slots {
            if orb.is_empty() {
                continue;
            }
            if orb.orb_type == crate::orbs::OrbType::Plasma {
                effects.push(crate::orbs::PassiveEffect::PlasmaEnergy(orb.base_passive));
            }
            match orb.orb_type {
                crate::orbs::OrbType::Lightning => effects.push(
                    crate::orbs::PassiveEffect::LightningDamage(orb.passive_with_focus(focus)),
                ),
                crate::orbs::OrbType::Frost => effects.push(
                    crate::orbs::PassiveEffect::FrostBlock(orb.passive_with_focus(focus)),
                ),
                crate::orbs::OrbType::Dark => {
                    orb.evoke_amount += orb.passive_with_focus(focus);
                }
                crate::orbs::OrbType::Plasma | crate::orbs::OrbType::Empty => {}
            }
        }
        if self.state.has_relic("Cables") {
            if let Some(front) = self.state.orb_slots.slots.first_mut() {
                if !front.is_empty() {
                    if front.orb_type == crate::orbs::OrbType::Plasma {
                        effects.push(crate::orbs::PassiveEffect::PlasmaEnergy(front.base_passive));
                    }
                    match front.orb_type {
                        crate::orbs::OrbType::Lightning => effects.push(
                            crate::orbs::PassiveEffect::LightningDamage(
                                front.passive_with_focus(focus),
                            ),
                        ),
                        crate::orbs::OrbType::Frost => effects.push(
                            crate::orbs::PassiveEffect::FrostBlock(
                                front.passive_with_focus(focus),
                            ),
                        ),
                        crate::orbs::OrbType::Dark => {
                            front.evoke_amount += front.passive_with_focus(focus);
                        }
                        crate::orbs::OrbType::Plasma | crate::orbs::OrbType::Empty => {}
                    }
                }
            }
        }
        for effect in effects {
            self.apply_passive_effect(effect);
            if self.state.player.is_dead() || self.check_combat_end() {
                return;
            }
        }
    }

    fn apply_orb_impulse_passives(&mut self) {
        if !self.state.orb_slots.has_orbs() {
            return;
        }
        self.apply_orb_start_of_turn();
        self.apply_all_orb_end_of_turn_passives();
        if self.state.has_relic("Cables") {
            self.apply_front_orb_start_of_turn_passive();
            self.apply_front_orb_end_of_turn_passive();
        }
    }

    // =======================================================================
    // Draw / Shuffle
    // =======================================================================

    pub fn draw_cards(&mut self, count: i32) -> Vec<CardType> {
        // NoDraw: skip draw entirely
        if self.state.player.status(sid::NO_DRAW) > 0 {
            return Vec::new();
        }

        let actual_count = count.max(0);

        let mut extra_draws = 0i32;
        let mut shuffles = 0;
        let mut drawn_card_types = Vec::new();

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
                shuffles += 1;
            }

            if let Some(mut drawn) = self.state.draw_pile.pop() {
                let card_def = self.card_registry.card_def_by_id(drawn.def_id);
                effects::card_runtime::initialize_stateful_cost_on_draw(
                    card_def,
                    &mut drawn,
                    self.state.player.status(sid::DISCARDED_THIS_TURN),
                );
                // ConfusionPower.java::onCardDraw consumes cardRandomRng for
                // every non-negative-cost card, permanently sets both cost
                // and costForTurn to 0..3, and clears freeToPlayOnce.
                if self.state.player.status(sid::CONFUSION) > 0 && card_def.cost >= 0 {
                    let new_cost = self.card_random_rng.random(3) as i8;
                    drawn.set_permanent_cost(new_cost);
                    if matches!(card_def.id, "Force Field" | "Force Field+") {
                        // ConfusionPower overwrites both cost and costForTurn,
                        // erasing Force Field reductions from earlier Power
                        // plays. Only later Power cards reduce this new cost.
                        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ConfusionPower.java
                        drawn.misc = self
                            .state
                            .power_cards_played_this_combat
                            .min(i16::MAX as i32) as i16;
                    }
                    drawn.flags &= !CardInstance::FLAG_FREE;
                }
                self.state.hand.push(drawn);

                // Extract card info for power triggers
                let card_type = card_def.card_type;
                drawn_card_types.push(card_type);

                // Evolve: draw extra cards when drawing a Status
                let evolve = self.state.player.status(sid::EVOLVE);
                if evolve > 0 && card_type == CardType::Status {
                    extra_draws += evolve;
                }

                // FireBreathingPower.onCardDraw uses a pure damage matrix with
                // DamageType.THORNS for both Status and Curse cards.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FireBreathingPower.java
                let fire_breathing = self.state.player.status(sid::FIRE_BREATHING);
                if fire_breathing > 0
                    && (card_type == CardType::Status || card_type == CardType::Curse)
                {
                    for i in 0..self.state.enemies.len() {
                        if self.state.enemies[i].is_targetable() {
                            self.deal_thorns_damage_to_enemy(i, fire_breathing);
                        }
                    }
                }

                self.on_card_drawn(drawn);
            }
        }

        // EmptyDeckShuffleAction constructs onShuffle relic actions before the
        // follow-up draw, but those relic actions are queued behind that draw.
        // Emit after the requested cards are in hand so Melange's ScryAction
        // sees the same remaining draw pile as Java.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DrawCardAction.java
        // and actions/common/EmptyDeckShuffleAction.java
        for _ in 0..shuffles {
            let ctx = crate::effects::trigger::TriggerContext::empty();
            self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
                crate::effects::trigger::Trigger::OnShuffle,
                &ctx,
            ));
        }

        // Evolve: draw accumulated extra cards (recursive call handles further triggers)
        if extra_draws > 0 {
            if self.phase == CombatPhase::AwaitingChoice
                && self.choice.as_ref().is_some_and(|choice| choice.reason == ChoiceReason::Scry)
            {
                if let Some(choice) = self.choice.as_mut() {
                    choice.post_choice_draw += extra_draws;
                }
            } else {
                self.draw_cards(extra_draws);
            }
        }

        drawn_card_types
    }

    /// Shuffle the draw pile (pub(crate) for card_effects).
    pub(crate) fn shuffle_draw_pile(&mut self) {
        self.state.draw_pile.shuffle(&mut self.rng);
    }

    /// Havoc/PlayTopCardAction: select a random target, shuffle if necessary,
    /// then autoplay the top card without changing its stored cost.
    pub(crate) fn play_top_card_of_draw(&mut self, exhausts: bool) {
        // Havoc.java evaluates getRandomMonster(..., cardRandomRng) while
        // constructing PlayTopCardAction, before that action inspects either pile.
        // Java: reference/extracted/methods/card/Havoc.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/PlayTopCardAction.java
        let target_idx = self.random_living_enemy().map_or(-1, |idx| idx as i32);
        self.play_top_card_of_draw_at_target(target_idx, exhausts);
    }

    pub(crate) fn play_top_card_of_draw_at_target(&mut self, target_idx: i32, exhausts: bool) {

        if self.state.draw_pile.is_empty() {
            if self.state.discard_pile.is_empty() {
                return;
            }
            let mut cards = std::mem::take(&mut self.state.discard_pile);
            self.state.draw_pile.append(&mut cards);
            self.shuffle_draw_pile();
        }

        let mut card = self.state.draw_pile.pop().expect("non-empty after shuffle");
        card.flags |= CardInstance::FLAG_FREE;
        if exhausts {
            card.flags |= CardInstance::FLAG_EXHAUST_ON_USE;
        }
        let def = self.card_registry.card_def_by_id(card.def_id).clone();
        let can_play = self.can_play_card_inst(&def, card);

        if can_play {
            self.state.hand.push(card);
            let hand_idx = self.state.hand.len() - 1;
            let plays_before = self.state.total_cards_played;
            self.play_card(hand_idx, target_idx);
            if self.state.total_cards_played > plays_before {
                return;
            }
            // A second legality check can only differ because the autoplayed
            // instance temporarily occupies the hand. Fall through to Java's
            // failed-autoplay cleanup if it remained there.
            card = self.state.hand.remove(hand_idx);
        }

        // GameActionManager marks a failed autoplay dontTriggerOnUseCard, then
        // queues UseCardAction solely to perform its normal destination move.
        let mut post_play_card = card.set_free(false);
        post_play_card.flags &= !CardInstance::FLAG_EXHAUST_ON_USE;
        if def.card_type == CardType::Power {
            return;
        }
        if !exhausts {
            self.state.discard_pile.push(post_play_card);
        } else {
            let spoon_saved = (self.state.has_relic("Strange Spoon")
                || self.state.has_relic("StrangeSpoon"))
                && self.card_random_rng.random_boolean();
            if spoon_saved {
                self.state.discard_pile.push(post_play_card);
            } else {
                self.state.exhaust_pile.push(post_play_card);
                self.trigger_card_on_exhaust(post_play_card);
            }
        }
    }

    /// Generate a random range using the engine RNG (pub(crate) for card_effects).
    pub(crate) fn rng_gen_range(&mut self, range: std::ops::Range<usize>) -> usize {
        use rand::Rng;
        self.rng.gen_range(range)
    }

    pub(crate) fn apply_madness_action(&mut self, cost: i8) {
        // MadnessAction.java first prefers any card whose costForTurn is
        // positive. It samples the entire hand through cardRandomRng and retries
        // rejected cards, consuming one counter tick per sample. If no such card
        // exists, it instead targets a card whose permanent cost is positive,
        // including one that is temporarily free this turn.
        let registry = self.card_registry;
        let costs = |card: CardInstance| {
            let def_cost = registry.card_def_by_id(card.def_id).cost as i8;
            let permanent_cost = if card.base_cost >= 0 {
                card.base_cost
            } else {
                def_cost
            };
            let cost_for_turn = if card.cost >= 0 {
                card.cost
            } else {
                permanent_cost
            };
            (cost_for_turn, permanent_cost)
        };
        let better_possible = self
            .state
            .hand
            .iter()
            .copied()
            .any(|card| costs(card).0 > 0);
        let possible = self
            .state
            .hand
            .iter()
            .copied()
            .any(|card| costs(card).1 > 0);
        if !better_possible && !possible {
            return;
        }

        loop {
            let idx = self
                .card_random_rng
                .random(self.state.hand.len() as i32 - 1) as usize;
            let (cost_for_turn, permanent_cost) = costs(self.state.hand[idx]);
            let eligible = if better_possible {
                cost_for_turn > 0
            } else {
                permanent_cost > 0
            };
            if eligible {
                self.state.hand[idx].set_permanent_cost(cost);
                return;
            }
        }
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
            let bonus =
                if self.state.has_relic("Violet Lotus") || self.state.has_relic("VioletLotus") {
                    1
                } else {
                    0
                };
            self.state.energy += 2 + bonus;
        }

        // Enter Divinity: gain 3 energy
        if new_stance == Stance::Divinity {
            self.state.energy += 3;
        }

        self.state.stance = new_stance;

        // Power + relic triggers on stance change via unified dispatch
        // (Mental Fortress block, Rushdown draw, Teardrop Locket)
        {
            let ctx = crate::effects::trigger::TriggerContext::empty();
            self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
                crate::effects::trigger::Trigger::OnStanceChange,
                &ctx,
            ));
        }

        // Flurry of Blows: return all copies from discard to hand on any stance change
        let mut flurry_indices = Vec::new();
        for (i, card_inst) in self.state.discard_pile.iter().enumerate() {
            let name = self.card_registry.card_name(card_inst.def_id);
            if name == "FlurryOfBlows" || name == "FlurryOfBlows+" {
                flurry_indices.push(i);
            }
        }
        for &i in flurry_indices.iter().rev() {
            if self.state.hand.len() < 10 {
                let card = self.state.discard_pile.remove(i);
                self.state.hand.push(card);
            }
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

    /// Lesson Learned upgrades a random card in Java's persistent master deck,
    /// not a combat pile copy. The eligible list preserves deck order and the
    /// one selection consumes exactly one miscRng tick.
    /// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/LessonLearnedAction.java
    pub(crate) fn upgrade_random_master_deck_card(&mut self) -> Option<usize> {
        let eligible: Vec<usize> = self
            .state
            .master_deck
            .iter()
            .enumerate()
            .filter_map(|(idx, card)| {
                let id = self.card_registry.card_name(card.def_id);
                (!card.is_upgraded()
                    && self.card_registry.get(&format!("{id}+")).is_some())
                .then_some(idx)
            })
            .collect();
        if eligible.is_empty() {
            return None;
        }
        let selected = self.misc_rng.random_range(0, (eligible.len() - 1) as i32) as usize;
        let deck_idx = eligible[selected];
        self.card_registry
            .upgrade_card(&mut self.state.master_deck[deck_idx]);
        Some(deck_idx)
    }

    // =======================================================================
    // Helpers: Temp Card Creation (respects Master Reality)
    // =======================================================================

    /// Get a CardInstance for a temporary card, upgrading if Master Reality is active.
    pub fn temp_card(&self, base_id: &str) -> CardInstance {
        let base = self.card_registry.get(base_id);
        let upgraded_id = format!("{}+", base_id);
        // MakeTempCard* excludes Status and Curse cards from Master Reality.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
        let master_reality_can_upgrade = self.state.player.status(sid::MASTER_REALITY) > 0
            && base.is_some_and(|def| !matches!(def.card_type, CardType::Curse | CardType::Status))
            && self.card_registry.get(&upgraded_id).is_some();
        if master_reality_can_upgrade {
            self.card_registry.make_card(&upgraded_id)
        } else {
            self.card_registry.make_card(base_id)
        }
    }

    /// Add temporary cards to hand, spilling overflow into discard.
    pub fn add_temp_cards_to_hand(&mut self, base_id: &str, amount: i32) {
        if amount <= 0 {
            return;
        }
        for _ in 0..amount {
            let card = self.temp_card(base_id);
            if self.state.hand.len() < 10 {
                self.state.hand.push(card);
            } else {
                self.state.discard_pile.push(card);
            }
        }
    }

    pub(crate) fn apply_bullet_time(&mut self) {
        // BulletTime.use queues NoDrawPower before ApplyBulletTimeAction.
        // No Draw is a debuff and may be blocked by Artifact; the cost action
        // still runs and calls setCostForTurn(-9) on cards currently in hand.
        // AbstractCard.setCostForTurn clamps that to zero only when the current
        // cost is non-negative, leaving X-cost and unplayable cards unchanged.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/BulletTime.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ApplyBulletTimeAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/AbstractCard.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/NoDrawPower.java
        crate::powers::apply_debuff(&mut self.state.player, sid::NO_DRAW, 1);

        let registry = self.card_registry;
        for card in &mut self.state.hand {
            let def = registry.card_def_by_id(card.def_id);
            let cost_for_turn = if card.cost >= 0 {
                card.cost
            } else if card.base_cost >= 0 {
                card.base_cost
            } else {
                def.cost as i8
            };
            if cost_for_turn >= 0 {
                card.set_cost_for_turn(0);
            }
        }
    }

    fn flush_pending_nightmare_copies(&mut self) {
        if self.nightmare_pending_copies.is_empty() {
            return;
        }
        let pending = std::mem::take(&mut self.nightmare_pending_copies);
        for (card, copies) in pending {
            self.add_card_instance_copies_to_hand(card, copies as i32);
        }
    }

    pub(crate) fn add_card_instance_copies_to_hand(&mut self, card: CardInstance, amount: i32) {
        if amount <= 0 {
            return;
        }
        for _ in 0..amount {
            let mut copy = card;
            copy.reset_cost_for_turn();
            if self.state.hand.len() < 10 {
                self.state.hand.push(copy);
            } else {
                self.state.discard_pile.push(copy);
            }
        }
    }

    /// Perform Scry: reveal top N cards, let player choose which to discard.
    /// Triggers Nirvana before the choice and Weave after it resolves.
    pub fn do_scry(&mut self, amount: i32) {
        // GoldenEye is applied by ScryAction's constructor, before onScry
        // powers run or the draw pile is inspected. Its bonus is exactly two.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/ScryAction.java
        let amount = if self.state.has_relic("GoldenEye") {
            amount + 2
        } else {
            amount
        };
        // ScryAction calls every power's onScry before checking for an empty
        // draw pile, so Nirvana grants raw power-owned block immediately even
        // when no selection screen opens.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/ScryAction.java
        let nirvana = self.state.player.status(sid::NIRVANA);
        if nirvana > 0 {
            self.gain_block_player(nirvana);
        }
        let to_scry = if amount == -1 {
            self.state.draw_pile.len()
        } else {
            (amount.max(0) as usize).min(self.state.draw_pile.len())
        };
        if to_scry == 0 {
            return;
        }
        // Take top N cards from draw pile (end = top) and present as choice
        let revealed: Vec<CardInstance> = self
            .state
            .draw_pile
            .drain(self.state.draw_pile.len() - to_scry..)
            .collect();
        let options: Vec<ChoiceOption> = revealed
            .into_iter()
            .map(ChoiceOption::RevealedCard)
            .collect();
        // Multi-select: player picks any subset to discard (min 0 = can keep all)
        self.begin_choice(ChoiceReason::Scry, options, 0, to_scry);
    }

    // =======================================================================
    // Exhaust Hooks
    // =======================================================================

    /// Trigger all on-exhaust power and relic hooks via unified dispatch.
    /// (FeelNoPain block, DarkEmbrace draw, Charon's Ashes damage, Dead Branch card gen)
    pub fn trigger_on_exhaust(&mut self) {
        let ctx = crate::effects::trigger::TriggerContext::empty();
        self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
            crate::effects::trigger::Trigger::OnCardExhaust,
            &ctx,
        ));
    }

    pub(crate) fn trigger_card_on_exhaust(&mut self, card_inst: CardInstance) {
        let card = self.card_registry.card_def_by_id(card_inst.def_id);
        // CardGroup.moveToExhaustPile calls relic/power onExhaust hooks before
        // the exhausted card's triggerOnExhaust hook. The order is observable
        // at the hand limit (for example Dark Embrace + Necronomicurse).
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
        self.trigger_on_exhaust();
        crate::effects::card_runtime::apply_on_exhaust(self, card, card_inst);
    }

    /// Obtain a random potion into the first empty potion slot, respecting Sozu.
    pub fn obtain_random_potion(&mut self) -> bool {
        if self.state.has_relic("Sozu") {
            return false;
        }
        let Some(slot) = self.state.potions.iter().position(|p| p.is_empty()) else {
            return false;
        };

        let pool: Vec<&'static str> = crate::potions::defs::POTION_DEFS
            .iter()
            .map(|def| def.id)
            .collect();
        if pool.is_empty() {
            return false;
        }

        let potion_id = pool[self.rng_gen_range(0..pool.len())];
        self.state.potions[slot] = potion_id.to_string();
        self.rebuild_effect_runtime();
        true
    }

    // =======================================================================
    // Orb Channel / Evoke (public API for card_effects)
    // =======================================================================

    /// Channel an orb. If slots are full, evokes the front orb first.
    /// Also tracks channeled counts for Blizzard/Thunder Strike.
    pub fn channel_orb(&mut self, orb_type: crate::orbs::OrbType) {
        // Track channeled counts for Blizzard / Thunder Strike
        match orb_type {
            crate::orbs::OrbType::Lightning => {
                self.state.player.add_status(sid::LIGHTNING_CHANNELED, 1);
            }
            crate::orbs::OrbType::Frost => {
                self.state.player.add_status(sid::FROST_CHANNELED, 1);
            }
            _ => {}
        }
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

    /// Trigger the front orb's evoke effect without removing it.
    pub fn evoke_front_orb_without_removing(&mut self) {
        let focus = self.state.player.focus();
        let effect = self.state.orb_slots.evoke_front_without_removing(focus);
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

    /// Trigger the current front orb N times, removing it only on the final evoke.
    pub fn multicast_front_orb_n(&mut self, n: usize) {
        if n == 0 || self.state.orb_slots.occupied_count() == 0 {
            return;
        }
        // MulticastAction queues `effect - 1` EvokeWithoutRemovingOrbAction
        // instances followed by one EvokeOrbAction. A lethal earlier evoke
        // clears the remaining queue, so the front orb is not removed then.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/MulticastAction.java
        for _ in 0..n.saturating_sub(1) {
            self.evoke_front_orb_without_removing();
            if self.state.combat_over {
                return;
            }
        }
        self.evoke_front_orb();
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
            // Fire on_victory relics via unified dispatch (Burning Blood, Black Blood, Meat on the Bone, Face of Cleric)
            {
                let ctx = crate::effects::trigger::TriggerContext::empty();
                self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
                    crate::effects::trigger::Trigger::CombatVictory,
                    &ctx,
                ));
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

#[cfg(test)]
mod test_relic_runtime_wave4 {
    use crate::actions::Action;
    use crate::effects::runtime::GameEvent;
    use crate::effects::trigger::Trigger;
    use crate::tests::support::{combat_state_with, end_turn, engine_with_state, make_deck_n};

    #[test]
    fn velvet_choker_blocks_seventh_play_and_resets_next_turn() {
        let mut state = combat_state_with(
            make_deck_n("Defend", 20),
            vec![crate::tests::support::enemy_no_intent("JawWorm", 120, 120)],
            20,
        );
        state.relics = vec!["PureWater".to_string(), "Velvet Choker".to_string()];
        let mut engine = engine_with_state(state);
        engine.state.hand = make_deck_n("Defend", 7);
        engine.state.draw_pile.clear();
        engine.state.discard_pile.clear();

        for expected in 1..=6 {
            let hand_before = engine.state.hand.len();
            engine.execute_action(&Action::PlayCard {
                card_idx: 0,
                target_idx: -1,
            });
            assert_eq!(engine.state.hand.len(), hand_before - 1);
            assert_eq!(engine.state.cards_played_this_turn, expected);
            assert_eq!(
                engine.hidden_effect_value(
                    "Velvet Choker",
                    crate::effects::runtime::EffectOwner::PlayerRelic { slot: 1 },
                    0,
                ),
                expected
            );
        }

        // VelvetChoker.java ignores further onPlayCard callbacks at six,
        // including callbacks from paths that do not pass the manual play gate.
        engine.emit_event(GameEvent::empty(Trigger::OnAnyCardPlayed));
        assert_eq!(
            engine.hidden_effect_value(
                "Velvet Choker",
                crate::effects::runtime::EffectOwner::PlayerRelic { slot: 1 },
                0,
            ),
            6
        );

        let hand_before_blocked = engine.state.hand.len();
        let energy_before_blocked = engine.state.energy;
        let block_before_blocked = engine.state.player.block;
        engine.execute_action(&Action::PlayCard {
            card_idx: 0,
            target_idx: -1,
        });
        assert_eq!(engine.state.hand.len(), hand_before_blocked);
        assert_eq!(engine.state.energy, energy_before_blocked);
        assert_eq!(engine.state.player.block, block_before_blocked);
        assert_eq!(engine.state.cards_played_this_turn, 6);
        assert_eq!(
            engine.hidden_effect_value(
                "Velvet Choker",
                crate::effects::runtime::EffectOwner::PlayerRelic { slot: 1 },
                0,
            ),
            6
        );

        end_turn(&mut engine);
        assert_eq!(
            engine.hidden_effect_value(
                "Velvet Choker",
                crate::effects::runtime::EffectOwner::PlayerRelic { slot: 1 },
                0,
            ),
            0
        );

        engine.state.energy = 3;
        engine.state.hand = make_deck_n("Defend", 1);
        engine.state.draw_pile.clear();
        engine.state.discard_pile.clear();
        engine.execute_action(&Action::PlayCard {
            card_idx: 0,
            target_idx: -1,
        });
        assert_eq!(engine.state.cards_played_this_turn, 1);
        assert_eq!(
            engine.hidden_effect_value(
                "Velvet Choker",
                crate::effects::runtime::EffectOwner::PlayerRelic { slot: 1 },
                0,
            ),
            1
        );

        engine.emit_event(GameEvent::empty(Trigger::CombatVictory));
        assert_eq!(
            engine.hidden_effect_value(
                "Velvet Choker",
                crate::effects::runtime::EffectOwner::PlayerRelic { slot: 1 },
                0,
            ),
            -1
        );
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

        let registry = crate::cards::global_registry();
        let deck_instances: Vec<CardInstance> =
            deck.iter().map(|name| registry.make_card(name)).collect();
        let mut state = CombatState::new(
            player_hp,
            player_max_hp,
            enemy_states,
            deck_instances,
            energy,
        );
        if let Some(r) = relics {
            state.relics = r;
        }

        RustCombatEngine {
            engine: CombatEngine::new(state, seed),
        }
    }

    #[staticmethod]
    fn from_combat_snapshot_json(snapshot_json: &str) -> PyResult<Self> {
        let snapshot: crate::training_contract::CombatSnapshotV1 =
            serde_json::from_str(snapshot_json).map_err(|err| {
                pyo3::exceptions::PyValueError::new_err(format!(
                    "Failed to parse combat snapshot JSON: {err}"
                ))
            })?;
        Ok(Self {
            engine: crate::training_contract::combat_engine_from_snapshot(&snapshot),
        })
    }

    /// Start combat (shuffle + draw initial hand).
    fn start_combat(&mut self) {
        self.engine.start_combat();
    }

    fn get_training_schema_versions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        crate::serialize_to_py_dict(
            py,
            &crate::training_contract::TrainingSchemaVersionsV1::default(),
        )
    }

    fn get_combat_training_state<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let state = crate::training_contract::combat_training_state_from_combat(
            &self.engine,
            crate::encode_combat_action,
        );
        crate::serialize_to_py_dict(py, &state)
    }

    fn get_combat_snapshot<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let snapshot = crate::training_contract::combat_snapshot_from_combat(&self.engine);
        crate::serialize_to_py_dict(py, &snapshot)
    }

    fn get_combat_snapshot_json(&self) -> PyResult<String> {
        let snapshot = crate::training_contract::combat_snapshot_from_combat(&self.engine);
        serde_json::to_string(&snapshot).map_err(|err| {
            pyo3::exceptions::PyValueError::new_err(format!(
                "Failed to serialize combat snapshot: {err}"
            ))
        })
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

    fn step_training_action<'py>(
        &mut self,
        py: Python<'py>,
        action_id: i32,
    ) -> PyResult<Bound<'py, PyDict>> {
        let action = self
            .engine
            .get_legal_actions()
            .into_iter()
            .find(|candidate| crate::encode_combat_action(candidate) == action_id)
            .ok_or_else(|| {
                pyo3::exceptions::PyValueError::new_err(format!(
                    "Illegal combat training action id {action_id}"
                ))
            })?;
        self.engine.execute_action(&action);
        let result = PyDict::new_bound(py);
        result.set_item("action_id", action_id)?;
        result.set_item("done", self.engine.is_combat_over())?;
        result.set_item("terminal", self.engine.is_combat_over())?;
        result.set_item("won", self.engine.state.player_won)?;
        result.set_item("turn", self.engine.state.turn)?;
        result.set_item("hp", self.engine.state.player.hp)?;
        result.set_item("energy", self.engine.state.energy)?;
        result.set_item(
            "legal_action_ids",
            self.engine
                .get_legal_actions()
                .iter()
                .map(crate::encode_combat_action)
                .collect::<Vec<_>>(),
        )?;
        result.set_item(
            "training_state",
            crate::serialize_to_py_dict(
                py,
                &crate::training_contract::combat_training_state_from_combat(
                    &self.engine,
                    crate::encode_combat_action,
                ),
            )?,
        )?;
        Ok(result)
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
        self.engine
            .state
            .hand
            .iter()
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
            "burn_plus" => mfx::BURN_PLUS,
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
        let deck = make_deck(&[
            "Strike",
            "Strike",
            "Strike",
            "Strike",
            "Defend",
            "Defend",
            "Defend",
            "Defend",
            "Eruption",
            "Vigilance",
        ]);

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
            .position(|c| {
                engine
                    .card_registry
                    .card_name(c.def_id)
                    .starts_with("Strike")
            })
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
            .position(|c| {
                engine
                    .card_registry
                    .card_name(c.def_id)
                    .starts_with("Defend")
            })
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
        state.draw_pile = make_deck(&["Eruption", "Strike", "Strike", "Strike", "Strike"]);

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
        state.draw_pile = make_deck(&["Vigilance", "Strike", "Strike", "Strike", "Strike"]);

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
        state.draw_pile = make_deck(&["Eruption", "Strike", "Strike", "Strike", "Strike"]);

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
            .position(|c| {
                engine
                    .card_registry
                    .card_name(c.def_id)
                    .starts_with("Defend")
            })
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
            .position(|c| {
                engine
                    .card_registry
                    .card_name(c.def_id)
                    .starts_with("Strike")
            })
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
            .position(|c| {
                engine
                    .card_registry
                    .card_name(c.def_id)
                    .starts_with("Strike")
            })
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
        state.draw_pile = make_deck(&["Strike"]); // Only 1 card
        state.discard_pile = make_deck(&["Defend", "Defend", "Defend", "Defend"]);

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
            .position(|c| {
                engine
                    .card_registry
                    .card_name(c.def_id)
                    .starts_with("Strike")
            })
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
            .position(|c| {
                engine
                    .card_registry
                    .card_name(c.def_id)
                    .starts_with("Strike")
            })
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
        state.draw_pile = make_deck(&[
            "Eruption", "Strike", "Strike", "Strike", "Defend", "Defend", "Defend",
        ]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Ensure Eruption is in hand (RNG may not have drawn it)
        if !engine
            .state
            .hand
            .iter()
            .any(|c| engine.card_registry.card_name(c.def_id) == "Eruption")
        {
            engine
                .state
                .hand
                .push(engine.card_registry.make_card("Eruption"));
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
        state.draw_pile = make_deck(&["Eruption", "Strike", "Strike", "Strike", "Defend"]);

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
        state.draw_pile = make_deck(&[
            "Prostrate",
            "Prostrate",
            "Prostrate",
            "Prostrate",
            "Prostrate",
            "Strike",
            "Strike",
            "Strike",
        ]);

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
        state.draw_pile = make_deck(&["Eruption", "Strike", "Strike", "Strike", "Strike"]);

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
        state.draw_pile = make_deck(&["Worship", "Strike", "Strike", "Strike", "Strike"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let energy_before = engine.state.energy; // 3

        // Play Worship: 5 + 5 = 10 mantra -> Divinity + 3 energy
        if let Some(idx) = engine
            .state
            .hand
            .iter()
            .position(|c| engine.card_registry.card_name(c.def_id) == "Worship")
        {
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
            .position(|c| {
                engine
                    .card_registry
                    .card_name(c.def_id)
                    .starts_with("Strike")
            })
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
        state.draw_pile = make_deck(&["Strike", "Strike", "Strike", "Defend", "Defend"]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let actions = engine.get_legal_actions();
        // Should NOT contain any Strike plays (attacks blocked by Entangle)
        let attack_actions: Vec<_> = actions
            .iter()
            .filter(|a| {
                if let Action::PlayCard { card_idx, .. } = a {
                    engine
                        .card_registry
                        .card_name(engine.state.hand[*card_idx].def_id)
                        .starts_with("Strike")
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
                    engine
                        .card_registry
                        .card_name(engine.state.hand[*card_idx].def_id)
                        .starts_with("Defend")
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
        state.draw_pile = make_deck(&["Miracle", "Strike", "Strike", "Strike", "Strike"]);

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
        assert!(engine
            .state
            .exhaust_pile
            .iter()
            .any(|c| engine.card_registry.card_name(c.def_id) == "Miracle"));
    }

    #[test]
    fn test_inner_peace_in_calm_draws() {
        let mut state = make_test_state();
        state.stance = Stance::Calm;
        state.draw_pile = make_deck(&[
            "InnerPeace",
            "Strike",
            "Strike",
            "Strike",
            "Strike",
            "Defend",
            "Defend",
            "Defend",
        ]);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Ensure InnerPeace is in hand (RNG may not have drawn it)
        if !engine
            .state
            .hand
            .iter()
            .any(|c| engine.card_registry.card_name(c.def_id) == "InnerPeace")
        {
            engine
                .state
                .hand
                .push(engine.card_registry.make_card("InnerPeace"));
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
        state.draw_pile =
            make_deck(&["InnerPeace", "Strike", "Strike", "Strike", "Strike"]);

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
        state.draw_pile = make_deck(&[
            "MentalFortress",
            "Strike",
            "Strike",
            "Strike",
            "Strike",
        ]);

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
            .iter()
            .any(|c| engine.card_registry.card_name(c.def_id) == "MentalFortress"));
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
            .position(|c| {
                engine
                    .card_registry
                    .card_name(c.def_id)
                    .starts_with("Strike")
            })
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
        state.draw_pile = make_deck(&["Halt", "Strike", "Strike", "Strike", "Strike"]);

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
