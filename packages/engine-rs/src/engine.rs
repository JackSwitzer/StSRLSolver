//! Faithful deterministic combat engine.
//!
//! Core turn loop that delegates to:
//! - card_effects: card play effect execution
//! - status_effects: end-of-turn hand card triggers
//! - combat_hooks: enemy turns, boss damage hooks

use serde::{Deserialize, Serialize};

use crate::actions::Action;
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
#[cfg(test)]
use crate::state::EnemyCombatState;
use crate::state::{CombatState, Stance};
use crate::status_effects;
use crate::status_ids::sid;

/// Combat phase enum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoiceReason {
    Scry,
    DiscardFromHand,
    ExhaustFromHand,
    PutOnTopFromHand,
    PickFromDiscard,
    PickFromDrawPile,
    DiscoverCard,
    PickOption,
    DualWield,
    UpgradeCard,
    PickFromExhaust,
    SearchDrawPile,
    ReturnFromDiscard,
    ForethoughtPick,
    RecycleCard,
    DiscardForEffect,
    RetainFromHand,
    SetupPick,
    PlayCardFreeFromDraw,
}

/// A single option the player can choose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoiceOption {
    HandCard(usize),
    DrawCard(usize),
    DiscardCard(usize),
    RevealedCard(crate::combat_types::CardInstance),
    GeneratedCard(crate::combat_types::CardInstance),
    Named(String),
    ExhaustCard(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedChoicePayload {
    pub kind: NamedOptionKind,
    pub amount: i32,
}

/// Card effects queued behind an interactive ScryAction. Java keeps later
/// actions pending until the Scry choice resolves; Just Lucky uses block then
/// damage in that order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Card movement queued by a combat callback behind the current Java action.
///
/// These operations live on the engine rather than a card-play frame because
/// an interactive action can suspend the current card before the queue drains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DeferredCombatOp {
    MoveCard {
        card: CardInstance,
        route: DeferredCardRoute,
    },
    RemovePower {
        owner: DeferredPowerOwner,
        status: crate::ids::StatusId,
    },
    DrawCards {
        count: i32,
    },
    EscapeEnemy {
        enemy_idx: usize,
    },
    PlayTopCard {
        target_idx: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DeferredCardRoute {
    HandWithOverflowToDiscard,
    Discard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DeferredPowerOwner {
    Player,
    Enemy(usize),
}

/// Continuation point for an end-turn sequence suspended by a Java-owned
/// choice action. Nilry's Codex can pause while pre-card callbacks are still
/// draining; RetainCardPower can pause immediately before discard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum EndTurnResumeStage {
    PreCardActions,
    PostCardActions,
}

/// Actions enqueued by Java's turn-start callback walks. Relic callbacks run
/// in inventory order and power callbacks in canonical `power_order`, but
/// their addToTop/addToBot actions do not execute until every callback has
/// returned. Interactive actions leave the remaining queue intact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum TurnStartQueuedAction {
    OpeningEnergy,
    DrawCards(i32),
    GainEnergy(i32),
    LoseEnergy(i32),
    GainBlock(i32),
    HealPlayer(i32),
    ChangeStance(Stance),
    IncreaseOrbSlots(i32),
    AddPlayerStatus(crate::ids::StatusId, i32),
    AddEnemyStatus(usize, crate::ids::StatusId, i32),
    ApplyEnemyDebuff(usize, crate::ids::StatusId, i32),
    AddCardToHand(CardInstance),
    AddCardToRandomDrawSpot(CardInstance),
    ApplyPhantasmal(i32),
    ReducePlayerPower(crate::ids::StatusId, i32),
    ApplyPlayerDebuff(crate::ids::StatusId, i32),
    DamageAllEnemiesThorns(i32),
    PlayerLoseHp(i32),
    RemovePlayerPower(crate::ids::StatusId),
    UpgradeRandomCard,
    GamblingChip,
    Scry(i32),
    ShuffleDrawPile,
    MayhemWrapper,
    PlayTopCard { target_idx: i32 },
    DiscardFromHand(usize),
    GainDevotion(i32),
    EnterDivinity,
    NightmareCopies(u32),
    EnterWrathNextTurn,
    PlayerPoisonTick(i32),
    TriggerOrbImpulse,
    OrbLightning { damage: i32, hit_all: bool },
    GainMantra(i32),
    ToolboxChoice,
}

/// Java end-turn callbacks collect actions synchronously, then
/// `GameActionManager` drains the shared top/bottom queue. Keeping that queue
/// explicit prevents one callback's action from changing a later callback's
/// condition (Cloak Clasp followed by Orichalcum is the canonical example).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum EndTurnQueuedAction {
    GainBlock(i32),
    DamageAllEnemies(i32),
    PlayerLoseHp(i32),
    ApplyDexterityLoss(i32),
    AddInsightsToRandomDrawSpots(i32),
    RemovePlayerPower(crate::ids::StatusId),
    AddPlayerPower(crate::ids::StatusId, i32),
    AddPlayerStrength(i32),
    ApplyPlayerStrengthLoss(i32),
    ApplyPlayerDexterityLoss(i32),
    DamagePlayerThorns(i32),
    PlayerRegeneration(i32),
    ReduceBomb(i32),
    RetainCards(usize),
    MakeStatEquivalentCopyInDrawPile(CardInstance),
    TriggerEndOfTurnOrbs,
    ResolveLightningEndTurn { damage: i32, hit_all: bool },
    NilrysCodex,
}

/// Context for an in-progress player choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Reboot's ShuffleAllAction sequence queued behind Melange's ScryAction.
    pub deferred_shuffle_all_draw: Option<i32>,
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
fn default_next_enemy_instance_id() -> u64 {
    1
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CombatEngine {
    pub state: CombatState,
    pub phase: CombatPhase,
    #[serde(skip, default = "crate::cards::global_registry")]
    pub card_registry: &'static CardRegistry,
    /// Java's persistent `AbstractDungeon.cardRng`, used by card-library
    /// selection even when the request originates during combat.
    pub(crate) card_rng: crate::seed::StsRandom,
    /// Java's floor-local `AbstractDungeon.shuffleRng`.
    pub(crate) shuffle_rng: crate::seed::StsRandom,
    /// Java's floor-local `AbstractDungeon.monsterHpRng`, retained in combat
    /// because summoned monsters roll HP when their constructors run.
    pub(crate) monster_hp_rng: crate::seed::StsRandom,
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
    /// Java's process-global libGDX `MathUtils.random`. Presentation and
    /// gameplay call sites share this cursor, so combat must return it to the
    /// run instead of reconstructing it at room boundaries.
    pub(crate) ambient_math_rng: crate::seed::AmbientMathRng,
    /// Java's static default `java.util.Random` used by no-argument
    /// `Collections.shuffle`. Its natural Java seed is process/time-derived,
    /// so trace replay must inject its captured 48-bit state explicitly.
    java_collections_rng: crate::seed::JavaCollectionsRng,
    /// Process-global suffix allocated by `TheBombPower`'s constructor.
    the_bomb_id_offset: i32,
    /// Current `TheCollector.enemySlots` references, indexed by Java's slot
    /// keys 1 and 2. `MonsterGroup` retains dead Torch Head corpses when a
    /// replacement is spawned, so identity cannot be reconstructed by merely
    /// scanning every Torch Head in the public enemy list.
    #[serde(default)]
    pub(crate) collector_torch_slots: [Option<usize>; 2],
    /// Current `GremlinLeader.gremlins` references for Java slots 0..2.
    /// SummonGremlinAction replaces a slot reference in its constructor, then
    /// retains the old corpse in MonsterGroup when its update smart-inserts
    /// the replacement at the same draw position.
    #[serde(default)]
    pub(crate) gremlin_leader_slots: [Option<usize>; 3],
    /// Current `Reptomancer.daggers` references for Java slots 0..3.
    #[serde(default)]
    pub(crate) reptomancer_dagger_slots: [Option<usize>; 4],
    #[serde(default = "default_next_enemy_instance_id")]
    next_enemy_instance_id: u64,
    pub choice: Option<ChoiceContext>,
    pub effect_runtime: crate::effects::runtime::EffectRuntime,
    pub(crate) nightmare_pending_copies: Vec<(CardInstance, usize)>,
    /// Omniscience autoplay items paired with the runtime-stack depth of the
    /// parent card whose action queue owns them.
    pub(crate) omniscience_autoplay: Vec<(CardInstance, usize)>,
    #[serde(skip, default)]
    pub event_log: Vec<crate::effects::runtime::GameEventRecord>,
    /// Nested runtime-triggered events are queued onto the active dispatch
    /// frame so reentrant death/victory/card-play chains keep the current
    /// runtime instance set instead of seeing a temporary empty runtime.
    pub(crate) pending_runtime_events: Vec<crate::effects::runtime::GameEvent>,
    pub(crate) runtime_dispatch_active: bool,
    pub runtime_played_card: Option<CardInstance>,
    pub(crate) runtime_play_target_idx: Option<i32>,
    pub(crate) runtime_play_stack: Vec<(CardInstance, i32, bool)>,
    /// Bottom-queued card movement that resolves after the current
    /// UseCardAction, including StasisPower's held-card return.
    pub(crate) deferred_combat_ops: Vec<DeferredCombatOp>,
    /// UseCardAction.reboundCard for the active card-resolution frame.
    pub(crate) runtime_rebound_card: bool,
    pub runtime_replay_window: bool,
    /// Time Warp clears Java's queued follow-up cards immediately, but the
    /// current UseCardAction still moves its card before the early end-turn
    /// sequence runs.
    #[serde(default)]
    pub(crate) runtime_force_end_turn_after_card: bool,
    /// Raw energyOnUse captured for the current X-cost card. Necronomicon's
    /// queued copy reuses this value without spending energy a second time.
    pub(crate) runtime_last_x_energy_on_use: i32,
    pub(crate) runtime_x_energy_override: Option<i32>,
    /// True only while Java turn-start callbacks are collecting actions.
    #[serde(default)]
    pub(crate) collecting_turn_start_actions: bool,
    /// Remaining Java turn-start actions after an interactive action pauses.
    #[serde(default)]
    pub(crate) turn_start_actions: Vec<TurnStartQueuedAction>,
    #[serde(default)]
    pub(crate) pending_turn_start_resume: bool,
    #[serde(default)]
    pub(crate) end_turn_resume: Option<EndTurnResumeStage>,
    #[serde(default)]
    pub(crate) end_turn_actions: Vec<EndTurnQueuedAction>,
    pub runtime_card_total_unblocked_damage: i32,
    pub runtime_card_enemy_killed: bool,
}

impl CombatEngine {
    /// Create a deterministic unit-test combat fixture.
    ///
    /// Faithful runs must enter combat through `RunEngine`, which transfers all
    /// independently owned Java streams through `CombatRngs`.
    pub(crate) fn new(state: CombatState, seed: u64) -> Self {
        Self::new_with_card_random_seed(state, seed, seed)
    }

    /// Benchmark-only fixture constructor. This deliberately does not model a
    /// complete dungeon RNG topology; production simulation uses `RunEngine`.
    #[doc(hidden)]
    pub fn new_benchmark_fixture(state: CombatState, seed: u64) -> Self {
        Self::new(state, seed)
    }

    pub(crate) fn new_with_card_random_seed(
        state: CombatState,
        seed: u64,
        card_random_seed: u64,
    ) -> Self {
        Self::new_with_rng_streams(
            state,
            crate::seed::CombatRngs {
                card: crate::seed::StsRandom::new(seed),
                monster_hp: crate::seed::StsRandom::new(seed),
                shuffle: crate::seed::StsRandom::new(seed),
                card_random: crate::seed::StsRandom::new(card_random_seed),
                potion: crate::seed::StsRandom::new(card_random_seed),
                misc: crate::seed::StsRandom::new(card_random_seed),
                // Standalone combat fixtures have no dungeon floor from which
                // to derive Java's aiRng. RunEngine injects the real stream.
                ai: crate::seed::StsRandom::new(seed.wrapping_add(0xA1A1_A1A1)),
                ambient_math: crate::seed::AmbientMathRng::new(0),
                java_collections: crate::seed::JavaCollectionsRng::deterministic_default(),
            },
        )
    }

    pub(crate) fn new_with_rng_streams(
        mut state: CombatState,
        rngs: crate::seed::CombatRngs,
    ) -> Self {
        for (index, enemy) in state.enemies.iter_mut().enumerate() {
            enemy.runtime_instance_id = index as u64 + 1;
        }
        let next_enemy_instance_id = state.enemies.len() as u64 + 1;
        let mut effect_runtime = crate::effects::runtime::EffectRuntime::default();
        effect_runtime.rebuild_from_state(&state);
        let mut engine = Self {
            state,
            phase: CombatPhase::NotStarted,
            card_registry: crate::cards::global_registry(),
            card_rng: rngs.card,
            shuffle_rng: rngs.shuffle,
            monster_hp_rng: rngs.monster_hp,
            card_random_rng: rngs.card_random,
            potion_rng: rngs.potion,
            misc_rng: rngs.misc,
            ai_rng: rngs.ai,
            ambient_math_rng: rngs.ambient_math,
            java_collections_rng: rngs.java_collections,
            the_bomb_id_offset: 0,
            collector_torch_slots: [None, None],
            gremlin_leader_slots: [None; 3],
            reptomancer_dagger_slots: [None; 4],
            next_enemy_instance_id,
            choice: None,
            effect_runtime,
            nightmare_pending_copies: Vec::new(),
            omniscience_autoplay: Vec::new(),
            event_log: Vec::new(),
            pending_runtime_events: Vec::new(),
            runtime_dispatch_active: false,
            runtime_played_card: None,
            runtime_play_target_idx: None,
            runtime_play_stack: Vec::new(),
            deferred_combat_ops: Vec::new(),
            runtime_rebound_card: false,
            runtime_replay_window: false,
            runtime_force_end_turn_after_card: false,
            runtime_last_x_energy_on_use: 0,
            runtime_x_energy_override: None,
            collecting_turn_start_actions: false,
            turn_start_actions: Vec::new(),
            pending_turn_start_resume: false,
            end_turn_resume: None,
            end_turn_actions: Vec::new(),
            runtime_card_total_unblocked_damage: 0,
            runtime_card_enemy_killed: false,
        };
        engine.initialize_gremlin_leader_slots();
        engine.initialize_reptomancer_dagger_slots();
        engine
    }

    pub(crate) fn rng_snapshot(&self) -> crate::seed::CombatRngs {
        crate::seed::CombatRngs {
            card: self.card_rng.clone(),
            monster_hp: self.monster_hp_rng.clone(),
            shuffle: self.shuffle_rng.clone(),
            card_random: self.card_random_rng.clone(),
            potion: self.potion_rng.clone(),
            misc: self.misc_rng.clone(),
            ai: self.ai_rng.clone(),
            ambient_math: self.ambient_math_rng.clone(),
            java_collections: self.java_collections_rng.clone(),
        }
    }

    pub fn ambient_math_rng_state(&self) -> (u64, u64) {
        self.ambient_math_rng.state_tuple()
    }

    pub fn restore_ambient_math_rng_state(&mut self, state: (u64, u64)) {
        self.ambient_math_rng.restore_state(state.0, state.1);
    }

    /// Capture the raw 48-bit state behind Java's no-argument
    /// `Collections.shuffle` stream for deterministic continuation.
    pub fn java_collections_rng_state(&self) -> u64 {
        self.java_collections_rng.state()
    }

    /// Restore a captured raw 48-bit `java.util.Random` state. Values are
    /// masked to Java's 48-bit state width.
    pub fn restore_java_collections_rng_state(&mut self, state: u64) {
        self.java_collections_rng.restore_state(state);
    }

    pub fn the_bomb_id_offset(&self) -> i32 {
        self.the_bomb_id_offset
    }

    pub fn restore_the_bomb_id_offset(&mut self, value: i32) {
        self.the_bomb_id_offset = value;
    }

    pub(crate) fn shuffle_end_turn_trigger_snapshot<T>(&mut self, values: &mut [T]) {
        self.java_collections_rng.shuffle(values);
    }

    pub fn rebuild_effect_runtime(&mut self) {
        self.effect_runtime.rebuild_from_state(&self.state);
    }

    /// RNG stream counters currently tracked by this combat, keyed by the
    /// vault's canonical short names (`docs/vault/rng-system-analysis.md`).
    /// Used by `bin/trace_replay.rs` (U05) to populate `trace::PostState::rng`.
    /// The currently threaded monsterHp, shuffle, cardRandom, potion, misc, and
    /// enemy-AI streams are reported independently; absent canonical streams
    /// remain schema-legal.
    pub fn rng_counters(&self) -> std::collections::BTreeMap<String, i64> {
        let mut counters = std::collections::BTreeMap::new();
        counters.insert(
            "monsterHp".to_string(),
            i64::from(self.monster_hp_rng.counter),
        );
        counters.insert("card".to_string(), i64::from(self.card_rng.counter));
        counters.insert("shuffle".to_string(), i64::from(self.shuffle_rng.counter));
        counters.insert(
            "cardRandom".to_string(),
            i64::from(self.card_random_rng.counter),
        );
        counters.insert("potion".to_string(), i64::from(self.potion_rng.counter));
        counters.insert("misc".to_string(), i64::from(self.misc_rng.counter));
        counters.insert("ai".to_string(), i64::from(self.ai_rng.counter));
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
        let mut event = event;
        if event.card_inst.is_none() {
            event.card_inst = self.runtime_played_card;
        }
        event.replay_window = self.runtime_replay_window;
        if self.runtime_dispatch_active {
            self.pending_runtime_events.push(event);
            return;
        }
        self.with_effect_runtime(|runtime, engine| runtime.emit(engine, event));
    }

    fn emit_turn_start_relics_and_ordered_powers(
        &mut self,
        event: crate::effects::runtime::GameEvent,
    ) {
        let power_order = self.state.player.power_order.to_vec();
        self.with_effect_runtime(|runtime, engine| {
            runtime.emit_relics_then_ordered_player_power_event(
                engine,
                event,
                &power_order,
                |engine, entry| engine.collect_dynamic_turn_start_power(entry),
            );
        });
    }

    fn emit_turn_start_relics(&mut self, event: crate::effects::runtime::GameEvent) {
        self.with_effect_runtime(|runtime, engine| runtime.emit_relic_event(engine, event));
    }

    fn emit_ordered_turn_start_powers(&mut self, event: crate::effects::runtime::GameEvent) {
        let power_order = self.state.player.power_order.to_vec();
        self.with_effect_runtime(|runtime, engine| {
            runtime.emit_ordered_player_power_event(
                engine,
                event,
                &power_order,
                |engine, entry| engine.collect_dynamic_turn_start_power(entry),
            );
        });
    }

    fn collect_dynamic_turn_start_power(&mut self, entry: crate::state::PowerOrderEntry) {
        match entry {
            crate::state::PowerOrderEntry::NightTerror(instance_id) => self
                .queue_turn_start_action_bottom(TurnStartQueuedAction::NightmareCopies(
                    instance_id,
                )),
            crate::state::PowerOrderEntry::Status(sid::POISON) => {
                let amount = self.state.player.status(sid::POISON);
                if amount > 0 {
                    self.queue_turn_start_action_bottom(TurnStartQueuedAction::PlayerPoisonTick(
                        amount,
                    ));
                }
            }
            crate::state::PowerOrderEntry::Status(sid::WRATH_NEXT_TURN) => {
                self.queue_turn_start_action_bottom(TurnStartQueuedAction::EnterWrathNextTurn)
            }
            crate::state::PowerOrderEntry::Status(sid::NEXT_TURN_BLOCK) => {
                let amount = self.state.player.status(sid::NEXT_TURN_BLOCK);
                self.queue_turn_start_action_bottom(TurnStartQueuedAction::GainBlock(amount));
                self.queue_turn_start_action_bottom(TurnStartQueuedAction::RemovePlayerPower(
                    sid::NEXT_TURN_BLOCK,
                ));
            }
            crate::state::PowerOrderEntry::Status(sid::BIASED_COG_FOCUS_LOSS) => {
                let amount = self.state.player.status(sid::BIASED_COG_FOCUS_LOSS);
                self.queue_turn_start_action_bottom(TurnStartQueuedAction::ApplyPlayerDebuff(
                    sid::FOCUS,
                    -amount,
                ));
            }
            crate::state::PowerOrderEntry::Status(sid::LOOP) => {
                let amount = self.state.player.status(sid::LOOP).max(0);
                for _ in 0..amount {
                    self.collect_front_orb_impulse_actions();
                }
            }
            crate::state::PowerOrderEntry::Status(sid::END_TURN_DEATH) => {
                self.queue_turn_start_action_bottom(TurnStartQueuedAction::PlayerLoseHp(99_999));
                self.queue_turn_start_action_bottom(TurnStartQueuedAction::RemovePlayerPower(
                    sid::END_TURN_DEATH,
                ));
            }
            _ => {}
        }
    }

    pub fn take_event_log(&mut self) -> Vec<crate::effects::runtime::GameEventRecord> {
        std::mem::take(&mut self.event_log)
    }

    pub fn clear_event_log(&mut self) {
        self.event_log.clear();
    }

    pub(crate) fn validate_checkpoint_boundary(&self) -> Result<(), String> {
        if self.runtime_dispatch_active || !self.pending_runtime_events.is_empty() {
            return Err("combat runtime dispatch is active".to_string());
        }
        if self.state.combat_over {
            return Err("active combat payload is already over".to_string());
        }
        if self.collecting_turn_start_actions {
            return Err("combat checkpoint boundary is collecting turn-start actions".to_string());
        }
        self.state
            .player
            .validate_power_order()
            .map_err(|error| format!("player {error}"))?;
        for (index, enemy) in self.state.enemies.iter().enumerate() {
            enemy
                .entity
                .validate_power_order()
                .map_err(|error| format!("enemy {index} {error}"))?;
        }
        let ordered_bombs = self
            .state
            .player
            .power_order
            .iter()
            .filter_map(|entry| match entry {
                crate::state::PowerOrderEntry::TheBomb(serial) => Some(*serial),
                crate::state::PowerOrderEntry::Status(_)
                | crate::state::PowerOrderEntry::NightTerror(_) => None,
            })
            .collect::<Vec<_>>();
        let payload_bombs = self
            .state
            .pending_bombs
            .iter()
            .map(|bomb| bomb.java_serial)
            .collect::<Vec<_>>();
        if ordered_bombs.len() != payload_bombs.len()
            || ordered_bombs
                .iter()
                .any(|serial| !payload_bombs.contains(serial))
        {
            return Err(format!(
                "The Bomb power order/payload state disagrees: ordered={ordered_bombs:?}, payload={payload_bombs:?}"
            ));
        }
        let ordered_night_terrors = self
            .state
            .player
            .power_order
            .iter()
            .filter_map(|entry| match entry {
                crate::state::PowerOrderEntry::NightTerror(instance_id) => Some(*instance_id),
                crate::state::PowerOrderEntry::Status(_)
                | crate::state::PowerOrderEntry::TheBomb(_) => None,
            })
            .collect::<Vec<_>>();
        let payload_night_terrors = self
            .nightmare_pending_copies
            .iter()
            .map(|(card, _)| card.instance_id)
            .collect::<Vec<_>>();
        if ordered_night_terrors != payload_night_terrors {
            return Err(format!(
                "Night Terror power order/payload state disagrees: ordered={ordered_night_terrors:?}, payload={payload_night_terrors:?}"
            ));
        }
        if let Some(slot) = self.state.relics.iter().position(|id| id == "Pen Nib") {
            let hidden_counter = self.hidden_effect_value(
                "Pen Nib",
                crate::effects::runtime::EffectOwner::PlayerRelic { slot: slot as u16 },
                0,
            );
            let projected_counter = self.state.player.status(sid::PEN_NIB_COUNTER);
            let power_active = self.state.player.status(sid::PEN_NIB_POWER) > 0;
            if hidden_counter != projected_counter || power_active != (hidden_counter == 9) {
                return Err(format!(
                    "Pen Nib counter/power state disagrees: hidden={hidden_counter}, projected={projected_counter}, active={power_active}"
                ));
            }
        }

        match self.phase {
            CombatPhase::PlayerTurn => {
                if self.choice.is_some() {
                    return Err("player-turn combat has a stale choice payload".to_string());
                }
                if self.pending_turn_start_resume || self.end_turn_resume.is_some() {
                    return Err("player-turn combat has a stale resume continuation".to_string());
                }
                if self.runtime_played_card.is_some()
                    || self.runtime_play_target_idx.is_some()
                    || !self.runtime_play_stack.is_empty()
                    || !self.deferred_combat_ops.is_empty()
                    || !self.omniscience_autoplay.is_empty()
                    || self.runtime_rebound_card
                    || self.runtime_force_end_turn_after_card
                    || self.runtime_x_energy_override.is_some()
                {
                    return Err("player-turn combat has unresolved card runtime state".to_string());
                }
                if !self.turn_start_actions.is_empty() {
                    return Err("player-turn combat has stale turn-start actions".to_string());
                }
            }
            CombatPhase::AwaitingChoice => {
                let choice = self.choice.as_ref().ok_or_else(|| {
                    "awaiting-choice combat is missing its choice payload".to_string()
                })?;
                if choice.min_picks > choice.max_picks || choice.max_picks > choice.options.len() {
                    return Err("combat choice cardinality is invalid".to_string());
                }
                if choice.selected.len() > choice.max_picks {
                    return Err("combat choice has too many selected options".to_string());
                }
                let mut selected = std::collections::HashSet::new();
                for index in &choice.selected {
                    if *index >= choice.options.len() || !selected.insert(*index) {
                        return Err("combat choice selection indexes are invalid".to_string());
                    }
                }
                if let Some(payloads) = &choice.named_payloads {
                    if payloads.len() != choice.options.len() {
                        return Err("combat named-choice payloads are misaligned".to_string());
                    }
                }
                let resume_count = usize::from(self.pending_turn_start_resume)
                    + usize::from(self.end_turn_resume.is_some());
                if resume_count > 1 {
                    return Err("combat choice has conflicting resume continuations".to_string());
                }
                if !self.turn_start_actions.is_empty() && !self.pending_turn_start_resume {
                    return Err(
                        "combat choice has turn-start actions without a resume continuation"
                            .to_string(),
                    );
                }
            }
            CombatPhase::NotStarted | CombatPhase::EnemyTurn | CombatPhase::CombatOver => {
                return Err("combat is not at an externally actionable boundary".to_string());
            }
        }
        Ok(())
    }

    pub(crate) fn validate_card_instance_ids(&self) -> Result<(), String> {
        fn insert_unique(
            ids: &mut std::collections::HashSet<u32>,
            card: CardInstance,
            owner: &str,
        ) -> Result<(), String> {
            if card.instance_id == 0 {
                return Err(format!("{owner} has a zero card instance id"));
            }
            if !ids.insert(card.instance_id) {
                return Err(format!(
                    "independent live cards alias instance {}",
                    card.instance_id
                ));
            }
            Ok(())
        }

        let mut master_ids = std::collections::HashSet::new();
        for card in &self.state.master_deck {
            if card.instance_id == 0 || !master_ids.insert(card.instance_id) {
                return Err(
                    "combat master-deck card instance ids must be unique and nonzero".to_string(),
                );
            }
        }

        let mut live_ids = std::collections::HashSet::new();
        for card in self
            .state
            .hand
            .iter()
            .chain(&self.state.draw_pile)
            .chain(&self.state.discard_pile)
            .chain(&self.state.exhaust_pile)
        {
            insert_unique(&mut live_ids, *card, "combat pile card")?;
        }
        for action in &self.turn_start_actions {
            let card = match action {
                TurnStartQueuedAction::AddCardToHand(card)
                | TurnStartQueuedAction::AddCardToRandomDrawSpot(card) => Some(*card),
                _ => None,
            };
            if let Some(card) = card {
                insert_unique(&mut live_ids, card, "queued turn-start card")?;
            }
        }
        for enemy in &self.state.enemies {
            if enemy.stasis_card.is_some() != (enemy.entity.status(sid::STASIS_POWER) > 0) {
                return Err(format!(
                    "enemy {} Stasis card and power presence disagree",
                    enemy.id
                ));
            }
            if let Some(card) = enemy.stasis_card {
                insert_unique(&mut live_ids, card, "Stasis card")?;
            }
        }
        if let Some(choice) = &self.choice {
            for option in &choice.options {
                if let ChoiceOption::RevealedCard(card) | ChoiceOption::GeneratedCard(card) = option
                {
                    insert_unique(&mut live_ids, *card, "combat choice card")?;
                }
            }
        }
        if let Some(card) = self.runtime_played_card {
            insert_unique(&mut live_ids, card, "active played card")?;
        }
        for (card, _, _) in &self.runtime_play_stack {
            insert_unique(&mut live_ids, *card, "nested played card")?;
        }
        for op in &self.deferred_combat_ops {
            if let DeferredCombatOp::MoveCard { card, .. } = op {
                insert_unique(&mut live_ids, *card, "deferred combat card")?;
            }
        }
        for (card, _) in &self.omniscience_autoplay {
            insert_unique(&mut live_ids, *card, "queued Omniscience card")?;
        }
        for (card, _) in &self.nightmare_pending_copies {
            insert_unique(&mut live_ids, *card, "Nightmare template")?;
        }
        for card in &self.state.retained_cards {
            if card.instance_id == 0
                || !self
                    .state
                    .hand
                    .iter()
                    .any(|hand_card| hand_card.instance_id == card.instance_id)
            {
                return Err("retained-card projection is not backed by the hand".to_string());
            }
        }

        let max_id = master_ids
            .iter()
            .chain(&live_ids)
            .copied()
            .map(u64::from)
            .max()
            .unwrap_or(0);
        if self.state.next_card_instance_id <= max_id {
            return Err("combat card allocator is behind a live identity".to_string());
        }
        Ok(())
    }

    pub(crate) fn take_pending_runtime_events(
        &mut self,
    ) -> Vec<crate::effects::runtime::GameEvent> {
        std::mem::take(&mut self.pending_runtime_events)
    }

    fn with_effect_runtime<T>(
        &mut self,
        f: impl FnOnce(&mut crate::effects::runtime::EffectRuntime, &mut CombatEngine) -> T,
    ) -> T {
        let mut runtime = std::mem::take(&mut self.effect_runtime);
        self.runtime_dispatch_active = true;
        let result = f(&mut runtime, self);
        self.runtime_dispatch_active = false;
        self.effect_runtime = runtime;
        result
    }

    fn begin_runtime_play_context(&mut self, card_inst: CardInstance, target_idx: i32) {
        if let Some(current) = self.runtime_played_card {
            self.runtime_play_stack.push((
                current,
                self.runtime_play_target_idx.unwrap_or(-1),
                self.runtime_rebound_card,
            ));
        }
        self.runtime_played_card = Some(card_inst);
        self.runtime_play_target_idx = Some(target_idx);
        self.runtime_rebound_card = false;
    }

    fn finish_runtime_play_context(&mut self) {
        if let Some((card_inst, target_idx, rebound_card)) = self.runtime_play_stack.pop() {
            self.runtime_played_card = Some(card_inst);
            self.runtime_play_target_idx = Some(target_idx);
            self.runtime_rebound_card = rebound_card;
        } else {
            self.runtime_played_card = None;
            self.runtime_play_target_idx = None;
            self.runtime_rebound_card = false;
        }
    }

    fn clear_runtime_play_contexts(&mut self) {
        self.runtime_played_card = None;
        self.runtime_play_target_idx = None;
        self.runtime_play_stack.clear();
        self.runtime_rebound_card = false;
        self.runtime_force_end_turn_after_card = false;
        self.omniscience_autoplay.clear();
        if self.state.combat_over || matches!(self.phase, CombatPhase::CombatOver) {
            self.deferred_combat_ops.clear();
        }
    }

    fn drain_deferred_combat_ops(&mut self) {
        // GameActionManager.clearPostCombatActions removes Stasis' queued
        // MakeTempCard action when the killing hit ends combat.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
        if self.state.combat_over || self.state.is_victory() || self.state.is_defeat() {
            self.deferred_combat_ops.clear();
            return;
        }

        let mut shuffle_events = 0;
        for op in std::mem::take(&mut self.deferred_combat_ops) {
            match op {
                DeferredCombatOp::MoveCard { card, route } => match route {
                    DeferredCardRoute::HandWithOverflowToDiscard => {
                        // MakeTempCardInHandAction checks the hand limit again
                        // when it executes, not only when StasisPower queues it.
                        if self.state.hand.len() < 10 {
                            self.state.hand.push(card);
                        } else {
                            self.state.discard_pile.push(card);
                        }
                    }
                    DeferredCardRoute::Discard => self.state.discard_pile.push(card),
                },
                DeferredCombatOp::RemovePower { owner, status } => match owner {
                    DeferredPowerOwner::Player => self.state.player.set_status(status, 0),
                    DeferredPowerOwner::Enemy(enemy_idx) => {
                        if let Some(enemy) = self.state.enemies.get_mut(enemy_idx) {
                            enemy.entity.set_status(status, 0);
                        }
                    }
                },
                DeferredCombatOp::DrawCards { count } => {
                    self.draw_cards(count);
                }
                DeferredCombatOp::EscapeEnemy { enemy_idx } => {
                    if let Some(enemy) = self.state.enemies.get_mut(enemy_idx) {
                        enemy.is_escaping = true;
                        // EscapeAction makes the monster basically dead before
                        // its later animation marks it escaped. The compact
                        // settled state represents that terminal monster with
                        // zero HP.
                        enemy.entity.hp = 0;
                    }
                }
                DeferredCombatOp::PlayTopCard { target_idx } => {
                    // PlayTopCardAction retries after EmptyDeckShuffleAction,
                    // then sends the card through NewQueueCardAction as a
                    // normal free autoplay. Execute only after the enclosing
                    // runtime hook has returned so card/relic events reach the
                    // authoritative runtime rather than a reentrant copy.
                    // Java: actions/common/PlayTopCardAction.java;
                    // actions/utility/NewQueueCardAction.java.
                    if self.state.draw_pile.is_empty() && !self.state.discard_pile.is_empty() {
                        self.state.draw_pile = std::mem::take(&mut self.state.discard_pile);
                        self.shuffle_draw_pile();
                        shuffle_events += 1;
                    }
                    if let Some(card) = self.state.draw_pile.pop() {
                        let free_card = card.set_free(true);
                        self.state.hand.push(free_card);
                        let hand_idx = self.state.hand.len() - 1;
                        self.execute_action(&Action::PlayCard {
                            card_idx: hand_idx,
                            target_idx,
                        });
                    }
                }
            }
            if self.state.combat_over || self.state.is_victory() || self.state.is_defeat() {
                self.deferred_combat_ops.clear();
                break;
            }
        }
        // EmptyDeckShuffleAction queues relic onShuffle effects behind the
        // already-enqueued PlayTopCardActions.
        for _ in 0..shuffle_events {
            let ctx = crate::effects::trigger::TriggerContext::empty();
            self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
                crate::effects::trigger::Trigger::OnShuffle,
                &ctx,
            ));
        }
    }

    pub(crate) fn defer_draw_until_current_card_moves(&mut self, count: i32) {
        if count > 0 {
            self.deferred_combat_ops
                .push(DeferredCombatOp::DrawCards { count });
        }
    }

    pub(crate) fn rebound_current_card(&mut self) {
        self.runtime_rebound_card = true;
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
        if self.phase != CombatPhase::NotStarted {
            return;
        }
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
            self.state
                .player
                .set_status(sid::PEN_NIB_POWER, i32::from(counter == 9));
        }

        // AbstractMonster.init calls rollMove for every initialized monster,
        // including fixed openers that ignore the sampled number.
        // Java: AbstractMonster.java::init/rollMove.
        let living_enemy_count = self
            .state
            .enemies
            .iter()
            .filter(|enemy| enemy.is_alive())
            .count() as i32;
        for enemy in self
            .state
            .enemies
            .iter_mut()
            .filter(|enemy| enemy.is_alive() && enemy.needs_initial_move_roll)
        {
            if enemy.id == "Centurion" {
                enemy.entity.set_status(sid::COUNT, living_enemy_count);
            }
            crate::enemies::roll_initial_move(enemy, &mut self.ai_rng);
        }

        // AbstractPlayer.preBattlePrep initializes and shuffles the deck
        // before monster pre-battle actions and relic atPreBattle hooks.
        // Java: AbstractPlayer.java:1562-1605; CardGroup.java:917-944.
        crate::seed::card_group_shuffle(&mut self.state.draw_pile, &mut self.shuffle_rng);

        // Innate and bottled cards are moved to the top immediately after
        // the Java deck copy is shuffled.
        let mut innate_indices = Vec::new();
        for (i, card) in self.state.draw_pile.iter().enumerate() {
            let def = self.card_registry.card_def_by_id(card.def_id);
            if def.runtime_traits().innate || card.flags & CardInstance::FLAG_INNATE != 0 {
                innate_indices.push(i);
            }
        }
        let mut innate_cards = Vec::new();
        for &i in innate_indices.iter().rev() {
            innate_cards.push(self.state.draw_pile.remove(i));
        }
        innate_cards.reverse();
        self.state.draw_pile.extend(innate_cards);

        // MonsterGroup initializes every member before AbstractPlayer calls
        // usePreBattleAction on the group. Louse Curl Up therefore consumes its
        // monsterHpRng draw only after all opening aiRng draws.
        // Java: MonsterGroup.java (`initialize`, `usePreBattleAction`) and
        // AbstractPlayer.java (`preBattlePrep`).
        for enemy in self.state.enemies.iter_mut().filter(|enemy| {
            matches!(
                enemy.id.as_str(),
                "FuzzyLouseNormal" | "RedLouse" | "FuzzyLouseDefensive" | "GreenLouse"
            )
        }) {
            let curl_min = enemy.entity.status(sid::BLOCK_AMT);
            if curl_min > 0 {
                let curl_max = if curl_min == 9 { 12 } else { curl_min + 4 };
                let curl_up = self.monster_hp_rng.random_int_range(curl_min, curl_max);
                enemy.entity.set_status(sid::CURL_UP, curl_up);
                enemy.entity.set_status(sid::BLOCK_AMT, 0);
            }
        }

        self.emit_event(crate::effects::runtime::GameEvent::empty(
            crate::effects::trigger::Trigger::CombatSetup,
        ));
        // AbstractPlayer first visits every atBattleStart callback. Direct
        // mutations (Philosopher's Stone's AbstractMonster.addPower) take
        // effect during that walk, while addToTop/addToBot effects only
        // enter GameActionManager's queue. Drain the top queue afterward.
        // Java: AbstractPlayer.java::applyStartOfCombatLogic;
        // relics/PhilosopherStone.java::atBattleStart.
        self.emit_event(crate::effects::runtime::GameEvent::empty(
            crate::effects::trigger::Trigger::CombatStartDirect,
        ));
        // AbstractRoom has already queued opening energy before this walk.
        // Every atBattleStart addToTop body is collected in relic order by
        // repeated front insertion, so later relics execute first. Pre-draw
        // actions (notably Toolbox) are FIFO, while later atTurnStart relics
        // may still insert actions above all of them.
        self.turn_start_actions.clear();
        self.pending_turn_start_resume = false;
        self.collecting_turn_start_actions = true;
        // EnergyManager.prep starts the opening recharge from zero.
        self.state.energy = 0;
        self.queue_turn_start_action_bottom(TurnStartQueuedAction::OpeningEnergy);
        self.emit_event(crate::effects::runtime::GameEvent::empty(
            crate::effects::trigger::Trigger::CombatStartTop,
        ));
        let event = crate::effects::runtime::GameEvent::empty(
            crate::effects::trigger::Trigger::CombatStartPreDraw,
        );
        self.emit_event(event);

        // Start first player turn. FIFO atBattleStart actions are dispatched
        // inside the turn immediately after Java's opening DrawCardAction.
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
                if (ctx.max_picks != 1 || ctx.min_picks == 0) && ctx.selected.len() >= ctx.min_picks
                {
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
                    CardTarget::Enemy | CardTarget::SelfAndEnemy => {
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

    /// Deep clone for deterministic branch simulation.
    pub fn clone_state(&self) -> CombatEngine {
        CombatEngine {
            state: self.state.clone(),
            phase: self.phase.clone(),
            card_registry: self.card_registry, // &'static ref — zero-cost copy
            card_rng: self.card_rng.clone(),
            shuffle_rng: self.shuffle_rng.clone(),
            monster_hp_rng: self.monster_hp_rng.clone(),
            card_random_rng: self.card_random_rng.clone(),
            potion_rng: self.potion_rng.clone(),
            misc_rng: self.misc_rng.clone(),
            ai_rng: self.ai_rng.clone(),
            ambient_math_rng: self.ambient_math_rng.clone(),
            java_collections_rng: self.java_collections_rng.clone(),
            the_bomb_id_offset: self.the_bomb_id_offset,
            collector_torch_slots: self.collector_torch_slots,
            gremlin_leader_slots: self.gremlin_leader_slots,
            reptomancer_dagger_slots: self.reptomancer_dagger_slots,
            next_enemy_instance_id: self.next_enemy_instance_id,
            choice: self.choice.clone(),
            effect_runtime: self.effect_runtime.clone(),
            nightmare_pending_copies: self.nightmare_pending_copies.clone(),
            omniscience_autoplay: self.omniscience_autoplay.clone(),
            event_log: self.event_log.clone(),
            pending_runtime_events: self.pending_runtime_events.clone(),
            runtime_dispatch_active: self.runtime_dispatch_active,
            runtime_played_card: self.runtime_played_card,
            runtime_play_target_idx: self.runtime_play_target_idx,
            runtime_play_stack: self.runtime_play_stack.clone(),
            deferred_combat_ops: self.deferred_combat_ops.clone(),
            runtime_rebound_card: self.runtime_rebound_card,
            runtime_replay_window: self.runtime_replay_window,
            runtime_force_end_turn_after_card: self.runtime_force_end_turn_after_card,
            runtime_last_x_energy_on_use: self.runtime_last_x_energy_on_use,
            runtime_x_energy_override: self.runtime_x_energy_override,
            collecting_turn_start_actions: self.collecting_turn_start_actions,
            turn_start_actions: self.turn_start_actions.clone(),
            pending_turn_start_resume: self.pending_turn_start_resume,
            end_turn_resume: self.end_turn_resume,
            end_turn_actions: self.end_turn_actions.clone(),
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
            deferred_shuffle_all_draw: None,
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
                ChoiceReason::DualWield => self.resolve_dual_wield(ctx),
                ChoiceReason::UpgradeCard => self.resolve_upgrade_card(ctx),
                ChoiceReason::PickFromExhaust => self.resolve_pick_from_exhaust(ctx),
                ChoiceReason::SearchDrawPile => self.resolve_search_draw_pile(ctx),
                ChoiceReason::ReturnFromDiscard => self.resolve_return_from_discard(ctx),
                ChoiceReason::ForethoughtPick => self.resolve_forethought(ctx),
                ChoiceReason::RecycleCard => self.resolve_recycle(ctx),
                ChoiceReason::DiscardForEffect => self.resolve_discard_for_effect(ctx),
                ChoiceReason::RetainFromHand => self.resolve_retain_from_hand(ctx),
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
        if self.end_turn_resume.is_some() {
            self.end_turn();
            return;
        }
        self.drive_choice_owned_runtime();
        if self.choice.is_some() || self.state.combat_over {
            return;
        }
        if self.pending_turn_start_resume {
            self.resume_turn_start_actions();
        }
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
        let deferred_shuffle_all_draw = ctx.deferred_shuffle_all_draw;
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

        // Weave normally queues DiscardToHandAction during ScryAction. With
        // Reboot, that action is queued behind the already-present shuffle and
        // therefore finds no card in discard; do not return Weave early here.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Weave.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/DiscardToHandAction.java
        if deferred_shuffle_all_draw.is_none() {
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
                if self.state.hand.len() >= 10 {
                    break;
                }
                let card = self.state.discard_pile.remove(i);
                self.state.hand.push(card);
            }
        }

        // ShuffleAllAction's constructor queues Melange's ScryAction before
        // Reboot's ShuffleAllAction itself is added to the action manager. The
        // revealed cards must therefore finish returning to draw/discard before
        // Reboot gathers and shuffles every pile.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ShuffleAllAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Melange.java
        if let Some(draw_count) = deferred_shuffle_all_draw {
            self.continue_shuffle_all_and_draw(draw_count);
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
                target_was_attacking: deferred.target_idx >= 0
                    && self
                        .state
                        .enemies
                        .get(deferred.target_idx as usize)
                        .is_some_and(|enemy| enemy.is_attacking()),
                x_value: deferred.x_value,
                pen_nib_active: deferred.pen_nib_active,
                vigor: deferred.vigor,
                total_unblocked_damage: 0,
                enemy_killed: false,
                hand_size_at_play: deferred.hand_size_at_play,
                last_bulk_count: 0,
                last_drawn_card_types: Vec::new(),
                deferred_manual_discards: Vec::new(),
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
        // HandCardSelectScreen.selectedCards and ExhaustAction preserve the
        // player's click order in the exhaust pile. Capture that observable
        // order before removing source cards by descending hand index.
        // Java: screens/select/HandCardSelectScreen.java::selectHoveredCard;
        // actions/common/ExhaustAction.java::update;
        // cards/CardGroup.java::moveToExhaustPile.
        let selected_cards: Vec<(usize, CardInstance)> = ctx
            .selected
            .iter()
            .filter_map(|&i| {
                if let ChoiceOption::HandCard(idx) = ctx.options[i] {
                    self.state.hand.get(idx).copied().map(|card| (idx, card))
                } else {
                    None
                }
            })
            .collect();
        let mut indices: Vec<usize> = selected_cards.iter().map(|(idx, _)| *idx).collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        for idx in indices {
            if idx < self.state.hand.len() {
                self.state.hand.remove(idx);
            }
        }
        for (_, card) in selected_cards {
            self.state.exhaust_pile.push(card);
            self.trigger_card_on_exhaust(card);
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

    fn resolve_retain_from_hand(&mut self, ctx: ChoiceContext) {
        let selected_cards = ctx
            .selected
            .iter()
            .filter_map(|option_idx| match ctx.options.get(*option_idx) {
                Some(ChoiceOption::HandCard(hand_idx)) => self.state.hand.get(*hand_idx).copied(),
                _ => None,
            })
            .collect::<Vec<_>>();
        self.state
            .hand
            .retain(|card| !selected_cards.iter().any(|selected| selected == card));

        // HandCardSelectScreen removes cards as they are clicked;
        // RetainCardsAction then addToTop's selectedCards in click order.
        // Reinsert the concrete instances in `ctx.selected` order so reverse
        // multi-selection is observable in the retained hand next turn.
        // Java: actions/unique/RetainCardsAction.java and
        // screens/select/HandCardSelectScreen.java.
        for mut selected in selected_cards {
            let card = self.card_registry.card_def_by_id(selected.def_id);
            // RetainCardsAction permits selecting Ethereal cards but does not
            // set their retain flag, so they still exhaust later this turn.
            // Java: actions/unique/RetainCardsAction.java.
            if !card.runtime_traits().ethereal {
                selected.set_retained(true);
            }
            self.state.hand.push(selected);
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
                    let card = self.fresh_stat_copy(card);
                    if ctx.generated_to_draw {
                        if self.state.draw_pile.is_empty() {
                            self.state.draw_pile.push(card);
                        } else {
                            // CardGroup.addToRandomSpot uses cardRandomRng and
                            // inserts before one existing index.
                            let index = self
                                .card_random_rng
                                .random_int((self.state.draw_pile.len() - 1) as i32)
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
            if let ChoiceOption::Named(name) = &ctx.options[sel] {
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

                panic!("named choices require typed payloads: {name}");
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
            let card = self.fresh_stat_copy(card);
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
                    self.store_nightmare_copies(self.state.hand[idx], ctx.aux_count.max(1));
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
        card.flags &=
            CardInstance::FLAG_UPGRADED | CardInstance::FLAG_FREE | CardInstance::FLAG_INNATE;
        let card = self.fresh_stat_copy(card);

        // ApplyPowerAction explicitly excludes POWER_ID "Night Terror" from
        // its normal same-ID stacking path. Each play therefore appends an
        // independent NightmarePower carrying its own selected card. Java's
        // stable priority sort preserves application order for these default-
        // priority (5) instances.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ApplyPowerAction.java
        self.state.player.add_night_terror_power(card.instance_id);
        self.nightmare_pending_copies.push((card, copies));
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
                    card.flags |= CardInstance::FLAG_FREE
                        | CardInstance::FLAG_EXHAUST_ON_USE
                        | CardInstance::FLAG_AUTOPLAY;
                    let mut purge_copy = card;
                    purge_copy.flags |= CardInstance::FLAG_PURGE;
                    let purge_copy = self.fresh_stat_copy(purge_copy);
                    let parent_depth = self.runtime_play_stack.len();
                    self.omniscience_autoplay.push((card, parent_depth));
                    self.omniscience_autoplay.push((purge_copy, parent_depth));
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
        let living = self.state.targetable_enemy_indices();
        let target = if living.is_empty() {
            -1
        } else {
            // Omniscience queues every copy with randomTarget=true. Java asks
            // MonsterGroup for a random monster before canUse regardless of
            // the card's target type, including self-target cards and a
            // singleton enemy group.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
            let selected = self
                .card_random_rng
                .random_int_range(0, (living.len() - 1) as i32) as usize;
            living[selected] as i32
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
        // BetterDrawPileToHandAction moves cards in selection order and sends
        // each overflow card to discard. Secret Weapon/Technique use the same
        // movement rule for their single selection.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/BetterDrawPileToHandAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/AttackFromDeckToHandAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/SkillFromDeckToHandAction.java
        let selected_cards: Vec<(usize, CardInstance)> = ctx
            .selected
            .iter()
            .filter_map(|&choice_index| {
                if let ChoiceOption::DrawCard(draw_index) = ctx.options[choice_index] {
                    self.state
                        .draw_pile
                        .get(draw_index)
                        .copied()
                        .map(|card| (draw_index, card))
                } else {
                    None
                }
            })
            .collect();
        let mut indices: Vec<usize> = selected_cards
            .iter()
            .map(|(draw_index, _)| *draw_index)
            .collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        for idx in indices {
            if idx < self.state.draw_pile.len() {
                self.state.draw_pile.remove(idx);
            }
        }
        for (_, card) in selected_cards {
            if self.state.hand.len() >= 10 {
                self.state.discard_pile.push(card);
            } else {
                self.state.hand.push(card);
            }
        }
    }

    fn resolve_return_from_discard(&mut self, ctx: ChoiceContext) {
        // Hologram / Meditate: move selected card(s) from discard to hand
        // GridCardSelectScreen.selectedCards preserves click order, and
        // MeditateAction adds the returned cards to hand in that same order.
        // Capture the cards before removing by descending pile index so the
        // implementation detail of safe removal does not reverse the
        // observable hand order.
        // Java: screens/select/GridCardSelectScreen.java::update;
        // actions/watcher/MeditateAction.java::update.
        let available_slots = 10usize.saturating_sub(self.state.hand.len());
        let mut selected_cards: Vec<(usize, CardInstance)> = ctx
            .selected
            .iter()
            .filter_map(|&i| {
                if let ChoiceOption::DiscardCard(idx) = ctx.options[i] {
                    self.state
                        .discard_pile
                        .get(idx)
                        .copied()
                        .map(|card| (idx, card))
                } else {
                    None
                }
            })
            .take(available_slots)
            .collect();
        let mut indices: Vec<usize> = selected_cards.iter().map(|(idx, _)| *idx).collect();
        indices.sort_unstable_by(|a, b| b.cmp(a)); // remove from back to front
        for idx in indices {
            if idx < self.state.discard_pile.len() {
                self.state.discard_pile.remove(idx);
            }
        }
        for (_, mut card) in selected_cards.drain(..) {
            if ctx.retain_returned_cards {
                card.set_retained(true); // Meditate marks returned cards as retained
            }
            if let Some(cost) = ctx.returned_card_cost_override {
                card.cost = cost;
            }
            self.state.hand.push(card);
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
                card.misc
            } else {
                def.base_damage
            };
            card.misc = current.wrapping_add(amount).max(0);
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
        next_block: i32,
    ) {
        // IncreaseMiscAction updates both the combat instance and the matching
        // UUID in AbstractDungeon.player.masterDeck.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/IncreaseMiscAction.java
        if let Some(card) = self
            .state
            .master_deck
            .iter_mut()
            .find(|card| card.instance_id == before.instance_id)
        {
            card.misc = next_block;
        }
    }

    pub(crate) fn sync_ritual_dagger_master_deck(
        &mut self,
        before: CardInstance,
        next_damage: i32,
    ) {
        // RitualDaggerAction matches the played card's UUID in masterDeck and
        // raises that persistent card's misc/baseDamage.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/
        // RitualDaggerAction.java
        if let Some(card) = self
            .state
            .master_deck
            .iter_mut()
            .find(|card| card.instance_id == before.instance_id)
        {
            card.misc = next_damage;
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
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::HandCard(idx) = ctx.options[sel] {
                self.recycle_hand_card(idx);
            }
        }
    }

    pub(crate) fn recycle_hand_card(&mut self, idx: usize) {
        if idx >= self.state.hand.len() {
            return;
        }
        let card = self.state.hand.remove(idx);
        let def = self.card_registry.card_def_by_id(card.def_id);
        let cost_for_turn = if card.cost >= 0 {
            card.cost as i32
        } else {
            def.cost
        };
        // RecycleAction grants current EnergyPanel energy for an X-cost card;
        // ordinary cards grant positive costForTurn and zero/negative costs
        // grant nothing. Capture and grant before exhaust hooks resolve.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/RecycleAction.java
        let energy_gain = if cost_for_turn == -1 {
            self.state.energy
        } else {
            cost_for_turn.max(0)
        };
        self.state.energy += energy_gain;
        self.state.exhaust_pile.push(card);
        self.trigger_card_on_exhaust(card);
    }

    fn resolve_setup(&mut self, ctx: ChoiceContext) {
        if let Some(&sel) = ctx.selected.first() {
            if let ChoiceOption::HandCard(idx) = ctx.options[sel] {
                self.move_setup_card_to_top(idx);
            }
        }
    }

    pub(crate) fn move_setup_card_to_top(&mut self, idx: usize) {
        if idx >= self.state.hand.len() {
            return;
        }
        let mut card = self.state.hand.remove(idx);
        let def = self.card_registry.card_def_by_id(card.def_id);
        let permanent_cost = if card.base_cost >= 0 {
            card.base_cost
        } else {
            def.cost as i8
        };
        if permanent_cost > 0 {
            card.flags |= CardInstance::FLAG_FREE;
        }
        // moveToDeck(card, false) adds to the top; last = top in Rust.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/SetupAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/Soul.java
        self.state.draw_pile.push(card);
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

    pub(crate) fn is_collecting_turn_start_actions(&self) -> bool {
        self.collecting_turn_start_actions
    }

    pub(crate) fn queue_turn_start_action_bottom(&mut self, action: TurnStartQueuedAction) {
        self.turn_start_actions.push(action);
    }

    pub(crate) fn queue_turn_start_action_top(&mut self, action: TurnStartQueuedAction) {
        self.turn_start_actions.insert(0, action);
    }

    fn execute_turn_start_action(&mut self, action: TurnStartQueuedAction) {
        match action {
            TurnStartQueuedAction::OpeningEnergy => {
                self.state.energy += self.state.max_energy;
                self.invoke_on_energy_recharge();
            }
            TurnStartQueuedAction::DrawCards(amount) => {
                self.draw_cards(amount);
            }
            TurnStartQueuedAction::GainEnergy(amount) => self.state.energy += amount,
            TurnStartQueuedAction::LoseEnergy(amount) => {
                self.state.energy = (self.state.energy - amount).max(0);
            }
            TurnStartQueuedAction::GainBlock(amount) => self.gain_block_player(amount),
            TurnStartQueuedAction::HealPlayer(amount) => self.heal_player(amount),
            TurnStartQueuedAction::ChangeStance(stance) => self.change_stance(stance),
            TurnStartQueuedAction::IncreaseOrbSlots(amount) => {
                let before = self.state.orb_slots.max_slots;
                for _ in 0..amount.max(0) {
                    self.state.orb_slots.add_slot();
                }
                let gained = self.state.orb_slots.max_slots.saturating_sub(before) as i32;
                if gained > 0 {
                    self.state.player.add_status(sid::ORB_SLOTS, gained);
                }
            }
            TurnStartQueuedAction::AddPlayerStatus(status, amount) => {
                self.state.player.add_status(status, amount);
            }
            TurnStartQueuedAction::AddEnemyStatus(enemy_idx, status, amount) => {
                if let Some(enemy) = self.state.enemies.get_mut(enemy_idx) {
                    if enemy.is_alive() {
                        enemy.entity.add_status(status, amount);
                    }
                }
            }
            TurnStartQueuedAction::ApplyEnemyDebuff(enemy_idx, status, amount) => {
                if self
                    .state
                    .enemies
                    .get(enemy_idx)
                    .is_some_and(|enemy| enemy.is_alive())
                {
                    self.apply_player_debuff_to_enemy(enemy_idx, status, amount);
                }
            }
            TurnStartQueuedAction::AddCardToHand(card) => {
                if self.state.hand.len() < 10 {
                    self.state.hand.push(card);
                } else {
                    self.state.discard_pile.push(card);
                }
            }
            TurnStartQueuedAction::AddCardToRandomDrawSpot(card) => {
                if self.state.draw_pile.is_empty() {
                    self.state.draw_pile.push(card);
                } else {
                    let index = self
                        .card_random_rng
                        .random_int_range(0, (self.state.draw_pile.len() - 1) as i32)
                        as usize;
                    self.state.draw_pile.insert(index, card);
                }
            }
            TurnStartQueuedAction::ApplyPhantasmal(amount) => {
                let double_damage = self.state.player.status(sid::DOUBLE_DAMAGE);
                self.state.player.set_status(
                    sid::DOUBLE_DAMAGE,
                    if double_damage > 0 {
                        double_damage + amount
                    } else {
                        1
                    },
                );
            }
            TurnStartQueuedAction::ReducePlayerPower(status, amount) => {
                let current = self.state.player.status(status);
                self.state
                    .player
                    .set_status(status, (current - amount).max(0));
            }
            TurnStartQueuedAction::ApplyPlayerDebuff(status, amount) => {
                powers::apply_debuff(&mut self.state.player, status, amount);
            }
            TurnStartQueuedAction::DamageAllEnemiesThorns(amount) => {
                for enemy_idx in self.state.living_enemy_indices() {
                    self.deal_thorns_damage_to_enemy(enemy_idx, amount);
                }
            }
            TurnStartQueuedAction::PlayerLoseHp(amount) => {
                self.player_lose_hp_from_damage(amount);
            }
            TurnStartQueuedAction::RemovePlayerPower(status) => {
                self.state.player.set_status(status, 0);
                if status == sid::END_TURN_DEATH {
                    self.state.blasphemy_active = false;
                }
            }
            TurnStartQueuedAction::UpgradeRandomCard => {
                let mut upgradeable = self
                    .state
                    .hand
                    .iter()
                    .enumerate()
                    .filter_map(|(index, card)| {
                        let def = self.card_registry.card_def_by_id(card.def_id);
                        (def.card_type != crate::cards::CardType::Status
                            && self.card_registry.can_upgrade_card(card))
                        .then_some(index)
                    })
                    .collect::<Vec<_>>();
                if !upgradeable.is_empty() {
                    // UpgradeRandomCardAction shuffles the eligible CardGroup.
                    // Java: cards/CardGroup.java; actions/common/UpgradeRandomCardAction.java.
                    crate::seed::card_group_shuffle(&mut upgradeable, &mut self.shuffle_rng);
                    self.card_registry
                        .upgrade_card(&mut self.state.hand[upgradeable[0]]);
                }
            }
            TurnStartQueuedAction::GamblingChip => {
                if !self.state.hand.is_empty() {
                    let options = (0..self.state.hand.len())
                        .map(ChoiceOption::HandCard)
                        .collect::<Vec<_>>();
                    let count = options.len();
                    self.begin_choice(ChoiceReason::DiscardFromHand, options, 0, count);
                    self.state.player.set_status(sid::GAMBLING_CHIP_ACTIVE, 1);
                }
            }
            TurnStartQueuedAction::Scry(amount) => self.do_scry(amount),
            TurnStartQueuedAction::ShuffleDrawPile => {
                if self.state.draw_pile.is_empty() {
                    self.emit_event(crate::effects::runtime::GameEvent::empty(
                        crate::effects::trigger::Trigger::OnShuffle,
                    ));
                    if self.phase != CombatPhase::AwaitingChoice {
                        self.state.draw_pile = std::mem::take(&mut self.state.discard_pile);
                        self.shuffle_draw_pile();
                    }
                }
            }
            TurnStartQueuedAction::MayhemWrapper => {
                let target_idx = self.random_living_enemy().map_or(-1, |idx| idx as i32);
                self.queue_turn_start_action_bottom(TurnStartQueuedAction::PlayTopCard {
                    target_idx,
                });
            }
            TurnStartQueuedAction::PlayTopCard { target_idx } => {
                self.play_top_card_of_draw_at_target(target_idx, false);
            }
            TurnStartQueuedAction::DiscardFromHand(amount) => {
                let discard_count = amount.min(self.state.hand.len());
                if discard_count == 0 {
                    return;
                }
                if self.state.hand.len() <= discard_count {
                    while let Some(card) = self.state.hand.pop() {
                        self.state.discard_pile.push(card);
                        self.on_card_discarded(card);
                    }
                } else {
                    let options = (0..self.state.hand.len())
                        .map(ChoiceOption::HandCard)
                        .collect::<Vec<_>>();
                    self.begin_choice(
                        ChoiceReason::DiscardFromHand,
                        options,
                        discard_count,
                        discard_count,
                    );
                }
            }
            TurnStartQueuedAction::GainDevotion(amount) => {
                if self.state.mantra == 0 && amount >= 10 {
                    self.change_stance(Stance::Divinity);
                } else {
                    self.gain_mantra(amount);
                }
            }
            TurnStartQueuedAction::EnterDivinity => {
                self.state.player.set_status(sid::ENTER_DIVINITY, 0);
                self.change_stance(Stance::Divinity);
            }
            TurnStartQueuedAction::NightmareCopies(instance_id) => {
                if let Some(index) = self
                    .nightmare_pending_copies
                    .iter()
                    .position(|(card, _)| card.instance_id == instance_id)
                {
                    let (card, copies) = self.nightmare_pending_copies.remove(index);
                    self.add_card_instance_copies_to_hand(card, copies as i32);
                    self.state.player.remove_night_terror_power(instance_id);
                }
            }
            TurnStartQueuedAction::EnterWrathNextTurn => {
                self.state.player.set_status(sid::WRATH_NEXT_TURN, 0);
                self.change_stance(Stance::Wrath);
            }
            TurnStartQueuedAction::PlayerPoisonTick(amount) => {
                self.player_lose_hp_from_damage(amount);
                let current = self.state.player.status(sid::POISON);
                self.state
                    .player
                    .set_status(sid::POISON, (current - 1).max(0));
            }
            TurnStartQueuedAction::TriggerOrbImpulse => {
                self.collect_all_orb_impulse_actions();
            }
            TurnStartQueuedAction::OrbLightning { damage, hit_all } => {
                if hit_all {
                    for enemy_idx in self.state.living_enemy_indices() {
                        let orb_damage = self.orb_damage_against(enemy_idx, damage);
                        self.deal_thorns_damage_to_enemy(enemy_idx, orb_damage);
                    }
                } else if let Some(enemy_idx) = self.random_living_enemy() {
                    let orb_damage = self.orb_damage_against(enemy_idx, damage);
                    self.deal_thorns_damage_to_enemy(enemy_idx, orb_damage);
                }
            }
            TurnStartQueuedAction::GainMantra(amount) => self.gain_mantra(amount),
            TurnStartQueuedAction::ToolboxChoice => {
                // ChooseOneColorless generates its choices in update(), not in
                // the inert constructor queued by Toolbox.atBattleStartPreDraw.
                // Later turn-start callbacks therefore consume cardRandom
                // before Toolbox does.
                // Java: actions/utility/ChooseOneColorless.java::update.
                let options = crate::effects::interpreter::generate_unique_random_cards(
                    self,
                    crate::effects::declarative::GeneratedCardPool::Colorless,
                    3,
                )
                .into_iter()
                .map(ChoiceOption::GeneratedCard)
                .collect::<Vec<_>>();
                self.begin_discovery_choice(
                    options,
                    1,
                    1,
                    1,
                    crate::effects::declarative::GeneratedCostRule::Base,
                );
            }
        }
    }

    fn drain_turn_start_actions(&mut self) {
        self.pending_turn_start_resume = false;
        while !self.turn_start_actions.is_empty() {
            let action = self.turn_start_actions.remove(0);
            self.execute_turn_start_action(action);
            if self.phase == CombatPhase::AwaitingChoice {
                self.pending_turn_start_resume = true;
                return;
            }
            if self.state.is_victory() {
                // GameActionManager.clearPostCombatActions retains queued
                // DAMAGE, BLOCK, HEAL, and USE_CARD actions. These are the
                // turn-start representatives of those Java action types.
                // Java: actions/GameActionManager.java::clearPostCombatActions.
                self.turn_start_actions.retain(|queued| {
                    matches!(
                        queued,
                        TurnStartQueuedAction::DamageAllEnemiesThorns(_)
                            | TurnStartQueuedAction::OrbLightning { .. }
                            | TurnStartQueuedAction::PlayerLoseHp(_)
                            | TurnStartQueuedAction::PlayerPoisonTick(_)
                            | TurnStartQueuedAction::GainBlock(_)
                            | TurnStartQueuedAction::HealPlayer(_)
                    )
                });
            } else if self.state.is_defeat() {
                self.turn_start_actions.clear();
            }
        }
    }

    fn resume_turn_start_actions(&mut self) {
        if !self.pending_turn_start_resume || self.phase != CombatPhase::PlayerTurn {
            return;
        }
        self.drain_turn_start_actions();
    }

    /// Java's `AbstractPlayer.onEnergyRecharge` walks the sorted power list.
    /// Energized and Deva mutate energy synchronously; Collect queues its
    /// MakeTempCard/RemovePower children behind actions already collected.
    fn invoke_on_energy_recharge(&mut self) {
        for entry in self.state.player.power_order.to_vec() {
            let crate::state::PowerOrderEntry::Status(status) = entry else {
                continue;
            };
            match status {
                sid::ENERGIZED | sid::ENERGIZED_BLUE => {
                    let amount = self.state.player.status(status);
                    self.state.energy += amount;
                    self.queue_turn_start_action_bottom(TurnStartQueuedAction::RemovePlayerPower(
                        status,
                    ));
                }
                sid::DEVA_FORM => {
                    let gain = self.state.player.status(sid::DEVA_FORM_ENERGY);
                    let growth = self.state.player.status(sid::DEVA_FORM);
                    self.state.energy += gain;
                    self.state
                        .player
                        .set_status(sid::DEVA_FORM_ENERGY, gain + growth);
                }
                sid::COLLECT_MIRACLES => {
                    let amount = self.state.player.status(sid::COLLECT_MIRACLES);
                    let card = self.temp_card("Miracle+");
                    self.queue_turn_start_action_bottom(TurnStartQueuedAction::AddCardToHand(card));
                    self.queue_turn_start_action_bottom(if amount <= 1 {
                        TurnStartQueuedAction::RemovePlayerPower(sid::COLLECT_MIRACLES)
                    } else {
                        TurnStartQueuedAction::ReducePlayerPower(sid::COLLECT_MIRACLES, 1)
                    });
                }
                _ => {}
            }
        }
    }

    /// Construct Java's end-turn DrawCardAction. Its `endTurnDraw=true`
    /// constructor creates PlayerTurnEffect immediately: energy recharge and
    /// onEnergyRecharge callbacks therefore happen now, while callback-owned
    /// child actions are appended before the DrawCardAction itself.
    fn construct_normal_turn_draw(&mut self, draw_amount: i32) {
        if self.state.has_relic("Ice Cream") || self.state.has_relic("IceCream") {
            self.state.energy += self.state.max_energy;
        } else {
            self.state.energy = self.state.max_energy;
        }
        self.invoke_on_energy_recharge();
        self.queue_turn_start_action_bottom(TurnStartQueuedAction::DrawCards(draw_amount));
    }

    fn start_player_turn(&mut self) {
        self.state.turn += 1;
        self.phase = CombatPhase::PlayerTurn;
        self.rebuild_effect_runtime();

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

        let is_opening_turn = self.state.turn == 1;
        if is_opening_turn {
            // AbstractRoom enqueues the initial DrawCardAction before invoking
            // atBattleStart and atTurnStart callbacks. Top combat-start actions
            // were already drained by start_combat; FIFO combat-start bodies
            // therefore settle immediately after this draw and before any
            // TurnStart relic/power body. This is observably different for
            // equal-priority powers and for HP mutation followed by Mercury
            // Hourglass damage.
            // Java: AbstractRoom.java::update (battle opening sequence).
            let ml = self.state.player.status(sid::DRAW);
            let serpent = self.state.player.status(sid::RING_OF_SERPENT_DRAW);
            let snecko_eye = i32::from(self.state.player.status(sid::SNECKO_EYE) > 0) * 2;
            let draw_reduction = i32::from(self.state.player.status(sid::DRAW_REDUCTION) > 0);
            self.queue_turn_start_action_bottom(TurnStartQueuedAction::DrawCards(
                5 + ml + serpent + snecko_eye - draw_reduction,
            ));
            let mut event = crate::effects::runtime::GameEvent::empty(
                crate::effects::trigger::Trigger::CombatStart,
            );
            event.is_first_turn = true;
            self.emit_event(event);
            if self.state.combat_over || self.phase == CombatPhase::AwaitingChoice {
                return;
            }
        }

        // Java invokes every atTurnStart relic and power callback before its
        // shared action queue drains. The callbacks may mutate counters now,
        // while their top/bottom actions remain ordered around the later draw.
        // Java: AbstractPlayer.applyStartOfTurnRelics;
        // AbstractCreature.applyStartOfTurnPowers; GameActionManager.update.
        if !is_opening_turn {
            self.turn_start_actions.clear();
        }
        self.pending_turn_start_resume = false;
        self.collecting_turn_start_actions = true;
        // AbstractPlayer.applyStartOfTurnRelics invokes the stance callback
        // before walking relics. Divinity queues a bottom ChangeStanceAction;
        // later relic addToTop actions can therefore still precede the exit.
        if self.state.stance == Stance::Divinity {
            self.queue_turn_start_action_bottom(TurnStartQueuedAction::ChangeStance(
                Stance::Neutral,
            ));
        }
        let turn_start_event = crate::effects::runtime::GameEvent {
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
        };
        if is_opening_turn {
            self.emit_turn_start_relics(turn_start_event);
            let mut post_draw_relic_event = crate::effects::runtime::GameEvent::empty(
                crate::effects::trigger::Trigger::TurnStartPostDraw,
            );
            post_draw_relic_event.is_first_turn = true;
            self.emit_turn_start_relics(post_draw_relic_event);
            self.emit_ordered_turn_start_powers(turn_start_event);
        } else {
            self.emit_turn_start_relics_and_ordered_powers(turn_start_event);
        }

        // PoisonLoseHpAction and WrathNextTurn's stance/removal action are
        // queued at their exact positions in the ordered power callback walk.

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

        // NextTurnBlockPower.atStartOfTurn queues GainBlockAction followed by
        // removal of the power. GameActionManager clears the previous turn's
        // block before that queued gain resolves, so this block belongs to the
        // new turn and the stacked power is consumed exactly once.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/NextTurnBlockPower.java
        // NextTurnBlockPower queued its GainBlockAction and removal during the
        // ordered power callback walk above.

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
        // BiasPower's ApplyPowerAction is queued in canonical power order.

        // === POWER HOOKS handled by the owner-aware TurnStart event above:
        // DemonForm, NoxiousFumes, Brutality, Berserk, InfiniteBlades, BattleHymn,
        // Devotion, WraithForm, DevaForm, HelloWorld, Magnetism,
        // DoppelgangerDraw, DoppelgangerEnergy
        //
        // LoopPower.atStartOfTurn calls both callbacks on the front orb once
        // per stack. GameActionManager clears old block synchronously after
        // invoking powers but before their queued actions resolve, so Loop's
        // Frost Block belongs to the new turn and cannot protect the intervening
        // enemy turn.
        // Java: powers/LoopPower.java and actions/GameActionManager.java.
        // LoopPower's paired front-orb callbacks are queued in power order.

        // ---- Start-of-turn orb passives (Plasma) ----
        self.collect_orb_start_of_turn_actions();

        if !is_opening_turn {
            let ml = self.state.player.status(sid::DRAW);
            let serpent = self.state.player.status(sid::RING_OF_SERPENT_DRAW);
            let snecko_eye = i32::from(self.state.player.status(sid::SNECKO_EYE) > 0) * 2;
            let draw_reduction = i32::from(self.state.player.status(sid::DRAW_REDUCTION) > 0);
            self.construct_normal_turn_draw(5 + ml + serpent + snecko_eye - draw_reduction);
        }

        // TurnStartExtraDraw: one-shot extra draw from relics (Bag of Prep, etc.)
        let extra_draw = self.state.player.status(sid::TURN_START_EXTRA_DRAW);
        if extra_draw > 0 {
            self.queue_turn_start_action_bottom(TurnStartQueuedAction::DrawCards(extra_draw));
            self.state.player.set_status(sid::TURN_START_EXTRA_DRAW, 0);
        }

        // InkBottleDraw: one-shot extra draw from Ink Bottle relic trigger
        let ink_draw = self.state.player.status(sid::INK_BOTTLE_DRAW);
        if ink_draw > 0 {
            self.queue_turn_start_action_bottom(TurnStartQueuedAction::DrawCards(ink_draw));
            self.state.player.set_status(sid::INK_BOTTLE_DRAW, 0);
        }

        // ---- Post-draw power effects (complex powers not in EntityDefs) ----

        // Post-draw runtime hooks that must happen before remaining turn-start setup.
        if !is_opening_turn {
            self.emit_event(crate::effects::runtime::GameEvent {
                kind: crate::effects::trigger::Trigger::TurnStartPostDraw,
                card_type: None,
                card_inst: None,
                is_first_turn: false,
                target_idx: -1,
                enemy_idx: -1,
                potion_slot: -1,
                status_id: None,
                amount: 0,
                replay_window: false,
            });
        }
        self.collecting_turn_start_actions = false;
        if self.state.combat_over || self.phase == CombatPhase::AwaitingChoice {
            return;
        }

        self.drain_turn_start_actions();
    }

    pub(crate) fn schedule_the_bomb(&mut self, turns: i32, damage: i32) {
        let java_serial = self.the_bomb_id_offset;
        self.the_bomb_id_offset = self.the_bomb_id_offset.wrapping_add(1);
        self.state.pending_bombs.push(crate::state::PendingBomb {
            java_serial,
            turns: turns as i16,
            damage: damage as i16,
        });
        self.state.player.add_the_bomb_power(java_serial);
        self.sync_the_bomb_statuses();
    }

    fn sync_the_bomb_statuses(&mut self) {
        let damage = self
            .state
            .pending_bombs
            .iter()
            .map(|bomb| i32::from(bomb.damage))
            .sum();
        let turns = self
            .state
            .pending_bombs
            .iter()
            .map(|bomb| i32::from(bomb.turns))
            .min()
            .unwrap_or(0);
        self.state.player.set_status(sid::THE_BOMB, damage);
        self.state.player.set_status(sid::THE_BOMB_TURNS, turns);
    }

    pub(crate) fn queue_end_turn_action_bottom(&mut self, action: EndTurnQueuedAction) {
        self.end_turn_actions.push(action);
    }

    pub(crate) fn queue_end_turn_action_top(&mut self, action: EndTurnQueuedAction) {
        self.end_turn_actions.insert(0, action);
    }

    fn reduce_one_bomb_at_end_of_turn(&mut self, java_serial: i32) {
        let Some(index) = self
            .state
            .pending_bombs
            .iter()
            .position(|bomb| bomb.java_serial == java_serial)
        else {
            return;
        };
        let mut bomb = self.state.pending_bombs[index];
        bomb.turns -= 1;
        if bomb.turns > 0 {
            self.state.pending_bombs[index] = bomb;
            self.sync_the_bomb_statuses();
            return;
        }

        self.state.pending_bombs.remove(index);
        self.state.player.remove_the_bomb_power(java_serial);
        self.sync_the_bomb_statuses();
    }

    fn end_turn_action_survives_post_combat_clear(action: EndTurnQueuedAction) -> bool {
        matches!(
            action,
            EndTurnQueuedAction::GainBlock(_)
                | EndTurnQueuedAction::DamageAllEnemies(_)
                | EndTurnQueuedAction::PlayerLoseHp(_)
                | EndTurnQueuedAction::DamagePlayerThorns(_)
                | EndTurnQueuedAction::PlayerRegeneration(_)
                | EndTurnQueuedAction::ResolveLightningEndTurn { .. }
        )
    }

    fn execute_end_turn_action(&mut self, action: EndTurnQueuedAction) {
        match action {
            EndTurnQueuedAction::GainBlock(amount) => self.gain_block_player(amount),
            EndTurnQueuedAction::DamageAllEnemies(amount) => {
                for target in self.state.living_enemy_indices() {
                    self.deal_thorns_damage_to_enemy(target, amount);
                }
            }
            EndTurnQueuedAction::PlayerLoseHp(amount) => {
                self.player_lose_hp_from_damage(amount);
            }
            EndTurnQueuedAction::ApplyDexterityLoss(amount) => {
                crate::powers::apply_debuff(&mut self.state.player, sid::DEXTERITY, -amount);
            }
            EndTurnQueuedAction::AddInsightsToRandomDrawSpots(amount) => {
                for _ in 0..amount.max(0) {
                    let insight = self.temp_card("Insight");
                    if self.state.draw_pile.is_empty() {
                        self.state.draw_pile.push(insight);
                    } else {
                        let index = self
                            .card_random_rng
                            .random_int_range(0, (self.state.draw_pile.len() - 1) as i32)
                            as usize;
                        self.state.draw_pile.insert(index, insight);
                    }
                }
            }
            EndTurnQueuedAction::RemovePlayerPower(status) => {
                self.state.player.set_status(status, 0);
            }
            EndTurnQueuedAction::AddPlayerPower(status, amount) => {
                self.state.player.add_status(status, amount);
            }
            EndTurnQueuedAction::AddPlayerStrength(amount) => {
                self.state.player.add_status(sid::STRENGTH, amount);
            }
            EndTurnQueuedAction::ApplyPlayerStrengthLoss(amount) => {
                crate::powers::apply_debuff(&mut self.state.player, sid::STRENGTH, -amount);
                self.state.player.set_status(sid::LOSE_STRENGTH, 0);
                self.state.player.set_status(sid::TEMP_STRENGTH, 0);
            }
            EndTurnQueuedAction::ApplyPlayerDexterityLoss(amount) => {
                crate::powers::apply_debuff(&mut self.state.player, sid::DEXTERITY, -amount);
                self.state.player.set_status(sid::LOSE_DEXTERITY, 0);
            }
            EndTurnQueuedAction::DamagePlayerThorns(amount) => {
                self.deal_thorns_damage_to_player(amount);
            }
            EndTurnQueuedAction::PlayerRegeneration(amount) => {
                self.heal_player(amount);
                self.state.player.add_status(sid::REGENERATION, -1);
            }
            EndTurnQueuedAction::ReduceBomb(java_serial) => {
                self.reduce_one_bomb_at_end_of_turn(java_serial);
            }
            EndTurnQueuedAction::RetainCards(amount) => {
                let pyramid =
                    self.state.has_relic("Runic Pyramid") || self.state.has_relic("RunicPyramid");
                if amount > 0
                    && !self.state.hand.is_empty()
                    && !pyramid
                    && self.state.player.status(sid::EQUILIBRIUM) == 0
                {
                    let options = (0..self.state.hand.len())
                        .map(ChoiceOption::HandCard)
                        .collect();
                    self.begin_choice(
                        ChoiceReason::RetainFromHand,
                        options,
                        0,
                        amount.min(self.state.hand.len()),
                    );
                }
            }
            EndTurnQueuedAction::MakeStatEquivalentCopyInDrawPile(card_inst) => {
                let copy = self.fresh_stat_copy(card_inst);
                self.state.draw_pile.push(copy);
            }
            EndTurnQueuedAction::TriggerEndOfTurnOrbs => {
                self.collect_orb_end_of_turn_actions();
            }
            EndTurnQueuedAction::ResolveLightningEndTurn { damage, hit_all } => {
                // Lightning's intermediate end-turn action is a DAMAGE action
                // (LightningOrbPassiveAction normally,
                // LightningOrbEvokeAction under Electrodynamics), so it
                // survives clearPostCombatActions. Its child damage resolves
                // before the next ordinary queued action via addToTop.
                // Java: orbs/Lightning.java and actions/defect/
                // {LightningOrbPassiveAction,LightningOrbEvokeAction}.java.
                if hit_all {
                    for target in self.state.living_enemy_indices() {
                        let orb_damage = self.orb_damage_against(target, damage);
                        self.deal_thorns_damage_to_enemy(target, orb_damage);
                    }
                } else if let Some(target) = self.random_living_enemy() {
                    let orb_damage = self.orb_damage_against(target, damage);
                    self.deal_thorns_damage_to_enemy(target, orb_damage);
                }
            }
            EndTurnQueuedAction::NilrysCodex => {
                if !self.state.living_enemy_indices().is_empty() {
                    let options = crate::effects::interpreter::generate_unique_random_cards(
                        self,
                        crate::effects::declarative::GeneratedCardPool::WatcherAny,
                        3,
                    )
                    .into_iter()
                    .map(ChoiceOption::GeneratedCard)
                    .collect();
                    self.begin_codex_choice(options);
                }
            }
        }
    }

    fn drain_end_turn_actions(&mut self, resume: EndTurnResumeStage) -> bool {
        while !self.end_turn_actions.is_empty() {
            let action = self.end_turn_actions.remove(0);
            self.execute_end_turn_action(action);
            if self.state.is_victory() {
                // DamageAction/DamageAllEnemiesAction call
                // GameActionManager.clearPostCombatActions immediately after
                // monsters become basically dead. Preserve only Java's
                // Heal/GainBlock/UseCard/DAMAGE survivor classes.
                // Java: actions/GameActionManager.java::clearPostCombatActions.
                self.end_turn_actions
                    .retain(|queued| Self::end_turn_action_survives_post_combat_clear(*queued));
            }
            if self.phase == CombatPhase::AwaitingChoice {
                self.end_turn_resume = Some(resume);
                return false;
            }
            if self.state.combat_over {
                self.end_turn_actions.clear();
                return false;
            }
        }
        self.sync_the_bomb_statuses();
        true
    }

    fn collect_dynamic_player_end_turn_power(&mut self, entry: crate::state::PowerOrderEntry) {
        use crate::state::PowerOrderEntry;

        match entry {
            PowerOrderEntry::TheBomb(java_serial) => {
                if let Some(bomb) = self
                    .state
                    .pending_bombs
                    .iter()
                    .find(|bomb| bomb.java_serial == java_serial)
                    .copied()
                {
                    self.queue_end_turn_action_bottom(EndTurnQueuedAction::ReduceBomb(java_serial));
                    if bomb.turns == 1 {
                        self.queue_end_turn_action_bottom(EndTurnQueuedAction::DamageAllEnemies(
                            i32::from(bomb.damage),
                        ));
                    }
                }
            }
            PowerOrderEntry::NightTerror(_) => {}
            PowerOrderEntry::Status(status) => match status {
                sid::RAGE | sid::ENTANGLED | sid::CANNOT_CHANGE_STANCE | sid::NO_SKILLS_POWER => {
                    self.queue_end_turn_action_bottom(EndTurnQueuedAction::RemovePlayerPower(
                        status,
                    ))
                }
                sid::LOSE_STRENGTH => {
                    let amount = self.state.player.status(sid::LOSE_STRENGTH);
                    self.queue_end_turn_action_bottom(
                        EndTurnQueuedAction::ApplyPlayerStrengthLoss(amount),
                    );
                }
                sid::LOSE_DEXTERITY => {
                    let amount = self.state.player.status(sid::LOSE_DEXTERITY);
                    self.queue_end_turn_action_bottom(
                        EndTurnQueuedAction::ApplyPlayerDexterityLoss(amount),
                    );
                }
                sid::CONSTRICTED => {
                    let amount = self.state.player.status(sid::CONSTRICTED);
                    self.queue_end_turn_action_bottom(EndTurnQueuedAction::DamagePlayerThorns(
                        amount,
                    ));
                }
                sid::REGENERATION => {
                    let amount = self.state.player.status(sid::REGENERATION);
                    // RegenPower is the notable player atEndOfTurn power that
                    // uses addToTop; it resolves before all bottom actions.
                    self.queue_end_turn_action_top(EndTurnQueuedAction::PlayerRegeneration(amount));
                }
                sid::RETAIN_CARDS => {
                    let amount = self.state.player.status(sid::RETAIN_CARDS).max(0) as usize;
                    self.queue_end_turn_action_bottom(EndTurnQueuedAction::RetainCards(amount));
                }
                sid::LIVE_FOREVER => {
                    let amount = self.state.player.status(sid::LIVE_FOREVER);
                    self.queue_end_turn_action_bottom(EndTurnQueuedAction::AddPlayerPower(
                        sid::PLATED_ARMOR,
                        amount,
                    ));
                }
                // Equilibrium mutates card retain flags synchronously in its
                // callback. The discard implementation reads the still-live
                // power and applies the same non-Ethereal retention rule.
                sid::EQUILIBRIUM | sid::ESTABLISHMENT => {}
                _ => {}
            },
        }
    }

    fn collect_player_end_turn_power_actions(&mut self) {
        let power_order = self.state.player.power_order.to_vec();
        let event =
            crate::effects::runtime::GameEvent::empty(crate::effects::trigger::Trigger::TurnEnd);
        self.with_effect_runtime(|runtime, engine| {
            runtime.emit_ordered_player_power_event(
                engine,
                event,
                &power_order,
                |engine, entry| engine.collect_dynamic_player_end_turn_power(entry),
            );
        });
    }

    pub(crate) fn end_turn(&mut self) {
        if self.phase != CombatPhase::PlayerTurn {
            return;
        }
        let resume = self.end_turn_resume.take();
        match resume {
            Some(EndTurnResumeStage::PreCardActions) => {
                if !self.drain_end_turn_actions(EndTurnResumeStage::PreCardActions) {
                    return;
                }
            }
            Some(EndTurnResumeStage::PostCardActions) => {
                if !self.drain_end_turn_actions(EndTurnResumeStage::PostCardActions) {
                    return;
                }
                // The queued RetainCardsAction was the only post-card action
                // that can suspend, so resume directly at discard afterward.
            }
            None => {
                self.rebuild_effect_runtime();
                self.end_turn_actions.clear();

                // Java synchronously invokes every relic callback and the
                // three atEndOfTurnPreEndTurnCards powers, then drains their
                // shared action queue. Frozen Core channels synchronously;
                // other effects preserve addToTop/addToBot semantics here.
                self.emit_event(crate::effects::runtime::GameEvent::empty(
                    crate::effects::trigger::Trigger::TurnEndPreCard,
                ));
                self.queue_end_turn_action_bottom(EndTurnQueuedAction::TriggerEndOfTurnOrbs);
                status_effects::queue_end_turn_hand_ordinary_actions(self);
                if !self.drain_end_turn_actions(EndTurnResumeStage::PreCardActions) {
                    return;
                }
            }
        }

        if matches!(resume, Some(EndTurnResumeStage::PostCardActions)) && self.check_combat_end() {
            // A choice can suspend RetainCardsAction while later queued
            // end-turn powers remain. If one of those later DAMAGE actions is
            // lethal, Java clears DiscardAtEndOfTurnAction after the resumed
            // queue drains.
            return;
        }

        if !matches!(resume, Some(EndTurnResumeStage::PostCardActions)) {
            // Burn, Decay, Regret, Doubt, and Shame autoplay from the original
            // hand after the pre-card action queue (including orb passives).
            if status_effects::process_end_turn_card_queue(self) {
                self.phase = CombatPhase::CombatOver;
                return;
            }
            if self.state.is_victory() {
                self.check_combat_end();
                return;
            }

            // AbstractPlayer.applyEndOfTurnTriggers walks the heterogeneous
            // power list synchronously. The queued actions drain only after
            // every callback has run, preserving power priority and top/bottom
            // insertion (notably RegenPower.addToTop).
            self.collect_player_end_turn_power_actions();
            if !self.drain_end_turn_actions(EndTurnResumeStage::PostCardActions) {
                return;
            }
            if self.check_combat_end() {
                return;
            }
        }

        // DiscardAtEndOfTurnAction first removes retain/selfRetain cards to
        // limbo, then clones and Collections.shuffle's the remaining hand
        // before triggerOnEndOfPlayerTurn. Status/curse autoplay above belongs
        // to GameActionManager.callEndOfTurnActions and is neither filtered nor
        // shuffled. Ethereal callbacks addToTop, so their exhaust actions later
        // execute in reverse order of this one shuffled snapshot.
        // Java: actions/GameActionManager.java::callEndOfTurnActions;
        // actions/common/DiscardAtEndOfTurnAction.java;
        // cards/AbstractCard.java::triggerOnEndOfPlayerTurn.
        let mut end_turn_trigger_snapshot = self
            .state
            .hand
            .iter()
            .copied()
            .filter(|card_inst| {
                let card = self.card_registry.card_def_by_id(card_inst.def_id);
                !card.runtime_traits().retain && !card_inst.is_retained()
            })
            .collect::<Vec<_>>();
        self.shuffle_end_turn_trigger_snapshot(&mut end_turn_trigger_snapshot);
        let ethereal_exhaust_order = end_turn_trigger_snapshot
            .iter()
            .rev()
            .filter(|card_inst| {
                self.card_registry
                    .card_def_by_id(card_inst.def_id)
                    .runtime_traits()
                    .ethereal
            })
            .copied()
            .collect::<Vec<_>>();

        // 4. Discard hand — Runic Pyramid keeps ALL cards in hand (including Status/Curse).
        //    Only Ethereal cards exhaust at end of turn regardless of Runic Pyramid.
        // Source: decompiled/java-src/com/megacrit/cardcrawl/actions/common/
        // DiscardAtEndOfTurnAction.java skips DiscardAction with "Runic Pyramid".
        let _explicitly_retained = std::mem::take(&mut self.state.retained_cards);
        let mut ethereal_cards = Vec::new();
        if self.state.has_relic("Runic Pyramid") || self.state.has_relic("RunicPyramid") {
            // DiscardAtEndOfTurnAction first removes retain/selfRetain cards to
            // limbo even under Runic Pyramid. The relic suppresses only the
            // subsequent DiscardActions. RestoreRetainedCardsAction then uses
            // hand.addToTop (ArrayList::add), so those cards return after the
            // blanket-kept cards in their original order.
            // Java: DiscardAtEndOfTurnAction.java and
            // RestoreRetainedCardsAction.java.
            let hand = std::mem::take(&mut self.state.hand);
            let mut kept = Vec::new();
            let mut retained = Vec::new();
            for card_inst in hand {
                let card = self.card_registry.card_def_by_id(card_inst.def_id);
                if card.runtime_traits().retain || card_inst.is_retained() {
                    // Explicit retention also removes an Ethereal card before
                    // triggerOnEndOfPlayerTurn can queue its exhaust.
                    retained.push(card_inst);
                } else if card.runtime_traits().ethereal {
                    ethereal_cards.push(card_inst);
                } else {
                    // Runic Pyramid keeps cards without setting Java's retain
                    // or selfRetain flags. That distinction matters to
                    // EstablishmentPowerAction.
                    kept.push(card_inst);
                }
            }
            kept.extend(retained);
            // Track retained cards for Establishment cost reduction
            self.state.retained_cards = kept.clone();
            self.state.hand = kept;
        } else {
            // Normal: retain tagged cards + explicitly retained (FLAG_RETAINED), exhaust ethereal, discard rest
            // EquilibriumPower.atEndOfTurn tags every non-Ethereal card for
            // retention. Its amount is reduced only at end of round, so
            // stacked copies can retain the hand for multiple rounds.
            // Java: powers/EquilibriumPower.java::atEndOfTurn/atEndOfRound.
            let retain_flag = self.state.player.status(sid::RETAIN_HAND_FLAG) > 0;
            let retain_all = retain_flag || self.state.player.status(sid::EQUILIBRIUM) > 0;
            if retain_flag {
                self.state.player.set_status(sid::RETAIN_HAND_FLAG, 0);
            }
            let hand = std::mem::take(&mut self.state.hand);
            let mut retained = Vec::new();
            let mut discarded = Vec::new();
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
                    ethereal_cards.push(card_inst);
                } else if retain_all {
                    let mut retained_inst = card_inst;
                    retained_inst.set_retained(true);
                    retained.push(retained_inst);
                } else {
                    discarded.push(card_inst);
                }
            }
            // DiscardAction repeatedly removes CardGroup::getTopCard(), so
            // non-retained cards enter the discard pile in reverse hand order.
            // Java: DiscardAtEndOfTurnAction.java, DiscardAction.java.
            self.state.discard_pile.extend(discarded.into_iter().rev());
            // Track retained cards for Establishment cost reduction
            self.state.retained_cards = retained.clone();
            self.state.hand = retained;
        }

        // ExhaustSpecificCardActions execute in reverse order of the single
        // shuffled end-turn callback snapshot. Preserve that Java queue order
        // without perturbing the original order of Pyramid-retained cards.
        let ethereal_exhausted = ethereal_cards.len() as i32;
        for expected in ethereal_exhaust_order {
            if let Some(index) = ethereal_cards
                .iter()
                .position(|candidate| *candidate == expected)
            {
                self.state.exhaust_pile.push(ethereal_cards.remove(index));
            }
        }
        // Defensive fallback for any future Ethereal path excluded from the
        // callback snapshot; current Java-derived routes should consume all.
        self.state.exhaust_pile.extend(ethereal_cards);

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

            // GainStrengthPower ("Shackled") restores temporary enemy Strength
            // only after that monster has acted. Vault skips the entire
            // monsters.applyEndOfTurnPowers pass, so the loss persists when the
            // monster turn is skipped.
            // Java: powers/GainStrengthPower.java::atEndOfTurn and
            // actions/GameActionManager.java::update.
            for enemy in &mut self.state.enemies {
                if enemy.is_alive() {
                    let restore = enemy.entity.status(sid::TEMP_STRENGTH_LOSS);
                    if restore > 0 {
                        enemy.entity.add_status(sid::STRENGTH, restore);
                        enemy.entity.set_status(sid::TEMP_STRENGTH_LOSS, 0);
                    }
                }
            }

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

            // Source: decompiled/java-src/com/megacrit/cardcrawl/powers/
            // GenericStrengthUpPower.java (`atEndOfRound`). Orb Walker's
            // pre-battle power grants its stored Strength after every round.
            for enemy in &mut self.state.enemies {
                if enemy.is_alive() {
                    powers::apply_generic_strength_up(&mut enemy.entity);
                }
            }
        }

        // Java power atEndOfRound hooks run after the enemy turn (including
        // when Vault skips that turn), not with the player's TurnEnd hooks.
        self.emit_event(crate::effects::runtime::GameEvent::empty(
            crate::effects::trigger::Trigger::RoundEnd,
        ));

        // Source: decompiled/java-src/com/megacrit/cardcrawl/powers/SlowPower.java.
        // Rust stores the zero-amount installed power as sentinel 1; reset its
        // per-card amount to that sentinel after every round.
        powers::reset_slow(&mut self.state.player);
        for enemy in &mut self.state.enemies {
            powers::reset_slow(&mut enemy.entity);
        }

        // EquilibriumPower is turn-based and loses exactly one stack after
        // the enemy turn, after it has already retained the player's hand.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/EquilibriumPower.java
        powers::decrement_equilibrium(&mut self.state.player);

        // Panic Button constructs NoBlockPower with isSourceMonster=false, so
        // atEndOfRound immediately reduces its two-turn amount. It therefore
        // blocks gains for the rest of this turn and the next player turn.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/NoBlockPower.java
        let no_block = self.state.player.status(sid::NO_BLOCK);
        if no_block > 0 {
            self.state.player.set_status(sid::NO_BLOCK, no_block - 1);
        }

        // DoubleDamagePower applies to every NORMAL hit while present and is
        // turn-based, reducing once here rather than when an attack is played.
        // Java: powers/DoubleDamagePower.java::atDamageGive/atEndOfRound.
        let double_damage = self.state.player.status(sid::DOUBLE_DAMAGE);
        if double_damage > 0 {
            self.state
                .player
                .set_status(sid::DOUBLE_DAMAGE, double_damage - 1);
        }

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
            CardFilter::NonExhume => def.id.strip_suffix('+').unwrap_or(def.id) != "Exhume",
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
            CardFilter::Upgradeable => self.card_registry.can_upgrade_card(card),
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
            target_was_attacking: false,
            x_value: 0,
            pen_nib_active: false,
            vigor: 0,
            total_unblocked_damage: 0,
            enemy_killed: false,
            hand_size_at_play: self.state.hand.len().saturating_sub(1),
            last_bulk_count: 0,
            last_drawn_card_types: Vec::new(),
            deferred_manual_discards: Vec::new(),
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
                    // BetterDiscardPileToHandAction and BetterDrawPileToHandAction
                    // are legal no-ops when their source pile is empty; the
                    // corresponding cards have no canUse restriction for it.
                    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/BetterDiscardPileToHandAction.java
                    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/BetterDrawPileToHandAction.java
                    let empty_source_is_allowed = matches!(
                        card.id,
                        "Exhume" | "Exhume+" | "Hologram" | "Hologram+" | "Seek" | "Seek+"
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
            // DamageAction clears queued non-damage/non-heal/non-block actions
            // after a lethal hit. CardDef.enter_stance is the legacy/deferred
            // representation of ChangeStanceAction, so it must obey the same
            // post-combat pruning as the declarative interpreter above.
            // Java: DamageAction.java and
            // GameActionManager.java::clearPostCombatActions.
            if !self.state.is_victory() && !self.state.combat_over {
                let new_stance = Stance::from_str(stance_name);
                if self.phase == CombatPhase::AwaitingChoice {
                    if let Some(choice) = self.choice.as_mut() {
                        choice.deferred_stance = Some(new_stance);
                    }
                } else {
                    self.change_stance(new_stance);
                }
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

        // HexPower queues MakeTempCardInDrawPileAction with randomSpot=true.
        // CardGroup.addToRandomSpot consumes cardRandomRng for every Dazed
        // inserted into a nonempty pile and cannot replace the current top.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/HexPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
        if card.card_type != CardType::Attack {
            let hex = self.state.player.status(sid::HEX);
            if hex > 0 {
                for _ in 0..hex {
                    let dazed = self.temp_card("Dazed");
                    if self.state.draw_pile.is_empty() {
                        self.state.draw_pile.push(dazed);
                    } else {
                        let idx = self
                            .card_random_rng
                            .random_int((self.state.draw_pile.len() - 1) as i32)
                            as usize;
                        self.state.draw_pile.insert(idx, dazed);
                    }
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
        if self.state.combat_over {
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

        let force_end_turn = self.runtime_force_end_turn_after_card;
        if !force_end_turn {
            self.runtime_replay_window = true;
            self.with_effect_runtime(|runtime, engine| {
                runtime.emit_replay_window(engine, card.card_type, target_idx, card_inst);
            });
            self.runtime_replay_window = false;
        }

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

        if !self.state.combat_over && !force_end_turn {
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
                self.play_purge_autoplay_copy(card_inst, target_idx);
            }
        }

        let post_play_dest = effects::card_runtime::post_play_destination(&card);

        let exhausts_on_use = card.exhaust
            || card_inst.flags & CardInstance::FLAG_EXHAUST_ON_USE != 0
            || (card.card_type == CardType::Skill && self.state.player.status(sid::CORRUPTION) > 0);
        let spoon_saved = card_inst.flags & CardInstance::FLAG_PURGE == 0
            && card.card_type != CardType::Power
            && exhausts_on_use
            && (self.state.has_relic("Strange Spoon")
                || self.state.has_relic("StrangeSpoon"))
            // UseCardAction sets spoonProc from exactly one
            // cardRandomRng.randomBoolean(), then follows the ordinary post-use
            // destination when true.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
            && self.card_random_rng.random_bool();

        // UseCardAction clears freeToPlayOnce before moving the card to its
        // post-play pile, so Forethought's flag applies to exactly one play.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
        let mut post_play_card = card_inst.set_free(false);
        // UseCardAction clears the one-use exhaust flag after choosing the
        // destination, including when Havoc or Omniscience supplied it.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
        post_play_card.flags &= !(CardInstance::FLAG_EXHAUST_ON_USE | CardInstance::FLAG_AUTOPLAY);

        if card_inst.flags & CardInstance::FLAG_PURGE != 0 || card.card_type == CardType::Power {
            // purgeOnUse copies disappear after their effects. Power cards are
            // consumed by the normal power-play path regardless of exhaust.
        } else if exhausts_on_use && !spoon_saved {
            self.state.exhaust_pile.push(post_play_card);
            self.trigger_card_on_exhaust(post_play_card);
        } else if self.runtime_rebound_card {
            // ReboundPower sets UseCardAction.reboundCard for non-Powers. The
            // exhaust branch above still wins unless Strange Spoon saved the
            // card; otherwise moveToDeck(card, false) puts it on top without
            // consuming cardRandomRng, ahead of Tantrum's random destination.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ReboundPower.java
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
            self.state.draw_pile.push(post_play_card);
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
                    .random_int_range(0, (self.state.draw_pile.len() - 1) as i32)
                    as usize;
                self.state.draw_pile.insert(idx, post_play_card);
            }
        } else {
            self.state.discard_pile.push(post_play_card);
        }

        self.finish_runtime_play_context();
        self.drain_deferred_combat_ops();

        if force_end_turn || post_play_dest == crate::effects::types::PostPlayDestination::EndTurn {
            self.runtime_force_end_turn_after_card = false;
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

    /// Process a same-instance replay copy through Java's normal queued-card path.
    ///
    /// The copy is temporarily placed in the hand only to reuse the canonical
    /// play pipeline; `play_card` removes it before card-owned effects inspect
    /// combat piles. `makeStatEquivalentCopy` preserves free-to-play state but
    /// does not copy one-use exhaust or retain state.
    /// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/AbstractCard.java
    /// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    pub(crate) fn play_purge_autoplay_copy(
        &mut self,
        mut card_inst: CardInstance,
        target_idx: i32,
    ) -> bool {
        card_inst.flags &= !(CardInstance::FLAG_RETAINED | CardInstance::FLAG_EXHAUST_ON_USE);
        card_inst.flags |=
            CardInstance::FLAG_FREE | CardInstance::FLAG_PURGE | CardInstance::FLAG_AUTOPLAY;

        let card = self.card_registry.card_def_by_id(card_inst.def_id).clone();
        if !self.can_play_card_inst(&card, card_inst) {
            return false;
        }

        self.state.hand.push(card_inst);
        let hand_idx = self.state.hand.len() - 1;
        let plays_before = self.state.total_cards_played;
        self.play_card(hand_idx, target_idx);
        if self.state.total_cards_played > plays_before {
            return true;
        }

        // A hand-sensitive rule can differ only after the temporary limbo copy
        // is inserted. Failed Java autoplay copies poof instead of entering a pile.
        if hand_idx < self.state.hand.len() && self.state.hand[hand_idx] == card_inst {
            self.state.hand.remove(hand_idx);
        }
        false
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
                && (target_idx < 0
                    || (target_idx as usize) >= self.state.enemies.len()
                    || !self.state.enemies[target_idx as usize].is_targetable())
            {
                false
            } else {
                if potions::potion_requires_target(&potion_id) {
                    // PotionPopUp turns the player toward the selected monster
                    // before dispatching enemy-targeted potion actions.
                    // Java: decompiled/java-src/com/megacrit/cardcrawl/ui/panels/PotionPopUp.java
                    self.update_surrounded_facing_for_target(target_idx);
                }
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
            let entropic_refilled_own_slot =
                matches!(potion_id.as_str(), "EntropicBrew" | "Entropic Brew")
                    && !self.state.has_relic("Sozu");
            if !entropic_refilled_own_slot {
                self.state.potions[potion_idx] = String::new();
            }
            self.rebuild_effect_runtime();

            // DistilledChaosPotion.use only queues PlayTopCardActions. Drain
            // them after ManualActivation/OnPotionUsed and potion destruction,
            // matching Java's action manager and avoiding runtime reentrancy.
            if potion_id == "DistilledChaos" {
                self.drain_deferred_combat_ops();
            }

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
        let effect = self.prepare_card_discarded(card);
        self.resolve_card_discarded_effect(effect);
    }

    /// Apply synchronous discard bookkeeping and capture actions that Java
    /// queues for later resolution.
    pub(crate) fn prepare_card_discarded(
        &mut self,
        card: CardInstance,
    ) -> crate::effects::types::OnDiscardEffect {
        // incrementDiscard updates the counter and every Eviscerate before
        // queued manual-discard hooks such as Reflex resolve.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
        self.state.player.add_status(sid::DISCARDED_THIS_TURN, 1);
        effects::card_runtime::apply_stateful_cost_on_discard(self);

        effects::card_runtime::apply_on_discard(self, card)
    }

    pub(crate) fn resolve_card_discarded_effect(
        &mut self,
        discard_effect: crate::effects::types::OnDiscardEffect,
    ) {
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
        // NoBlockPower.modifyBlockLast returns zero for every block source.
        // Panic Button's own GainBlockAction resolves before the power is
        // applied, so that initial block still lands.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/NoBlockPower.java
        if amount <= 0 || self.state.player.status(sid::NO_BLOCK) > 0 {
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

    /// Centralized player-owned HP loss (bypasses block). Checks fairy revive,
    /// fires on_hp_loss relics, and triggers Rupture when positive HP is lost.
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

        let tungsten = self.state.has_relic("Tungsten Rod") || self.state.has_relic("TungstenRod");
        let hp_loss = damage::apply_hp_loss(after_intangible, false, tungsten);
        self.player_lose_hp_with_owner(hp_loss, true);
    }

    pub fn player_lose_hp(&mut self, amount: i32) {
        self.player_lose_hp_with_owner(amount, false);
    }

    fn player_lose_hp_with_owner(&mut self, amount: i32, owner_is_player: bool) {
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
        self.state.player.add_status(sid::HP_LOSS_THIS_COMBAT, 1);

        // Fire on_hp_loss relics via unified dispatch (Centennial Puzzle, Self-Forming Clay, Runic Cube, Red Skull)
        {
            let ctx = crate::effects::trigger::TriggerContext::empty();
            self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
                crate::effects::trigger::Trigger::OnPlayerHpLoss,
                &ctx,
            ));
        }

        // RupturePower.wasHPLost requires positive HP loss whose DamageInfo
        // owner is the player. Enemy attacks and enemy-owned THORNS damage still
        // fire the general HP-loss hooks above, but do not grant Strength.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/RupturePower.java
        if owner_is_player {
            let rupture = self.state.player.status(sid::RUPTURE);
            if rupture > 0 {
                self.state.player.add_status(sid::STRENGTH, rupture);
            }
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
        let red_skull = self
            .state
            .relics
            .iter()
            .position(|relic| relic == "Red Skull")
            .map(|slot| crate::effects::runtime::EffectOwner::PlayerRelic { slot: slot as u16 });
        if red_skull.is_some_and(|owner| {
            self.hidden_effect_value("Red Skull", owner, 0) > 0
                && self.state.player.hp > self.state.player.max_hp / 2
        }) {
            self.state.player.add_status(sid::STRENGTH, -3);
            let _ = self.effect_runtime.set_hidden_value(
                "Red Skull",
                red_skull.expect("checked above"),
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
        // ChampionsBelt.onTrigger is called by ApplyPowerAction only after
        // player-sourced Vulnerable survives Artifact, then applies one Weak.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ApplyPowerAction.java
        // Java: reference/extracted/methods/relic/ChampionsBelt.java
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
        let special_rebirth_death = (matches!(
            self.state.enemies[enemy_idx].id.as_str(),
            "AwakenedOne" | "Awakened One"
        ) && self.state.enemies[enemy_idx].entity.status(sid::PHASE)
            == 1)
            || self.state.enemies[enemy_idx].id == "Darkling";
        if special_rebirth_death {
            combat_hooks::on_enemy_damaged(self, enemy_idx, hp_before);
        } else {
            self.finalize_enemy_death(enemy_idx);
        }
        if self.runtime_played_card.is_none() {
            self.drain_deferred_combat_ops();
        }
        true
    }

    pub fn deal_damage_to_enemy(&mut self, enemy_idx: usize, damage: i32) {
        self.deal_precomputed_normal_damage_to_enemy(enemy_idx, damage, false);
    }

    fn apply_shifting_after_hit(&mut self, enemy_idx: usize, damage_amount: i32) {
        if damage_amount <= 0 || self.state.enemies[enemy_idx].entity.status(sid::SHIFTING) <= 0 {
            return;
        }

        // Source: decompiled/java-src/com/megacrit/cardcrawl/powers/ShiftingPower.java.
        // Every positive onAttacked amount applies equal negative Strength and,
        // absent Artifact, GainStrengthPower restores it after the monster acts.
        let artifact = self.state.enemies[enemy_idx].entity.status(sid::ARTIFACT);
        if artifact > 0 {
            self.state.enemies[enemy_idx]
                .entity
                .set_status(sid::ARTIFACT, artifact - 1);
        } else {
            // ShiftingPower calls addToTop(Strength -N), then
            // addToTop(GainStrength +N). LIFO resolution therefore installs
            // Shackled before the negative Strength power.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ShiftingPower.java
            self.state.enemies[enemy_idx]
                .entity
                .add_status(sid::TEMP_STRENGTH_LOSS, damage_amount);
            self.state.enemies[enemy_idx]
                .entity
                .add_status(sid::STRENGTH, -damage_amount);
        }
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

        // FlightPower.atDamageFinalReceive halves NORMAL damage. Its later
        // onAttacked callback sees post-block damage and queues the reduction;
        // fully blocked hits therefore do not consume Flight.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FlightPower.java
        let flight = enemy.entity.status(sid::FLIGHT);
        let effective_damage = if flight > 0 && !pure_matrix {
            (damage_after_slow as f64 * 0.5) as i32
        } else {
            damage_after_slow
        };
        // Source: reference/extracted/methods/monster/Nemesis.java (`damage`)
        // and IntangiblePower.java. Nemesis caps DamageInfo.output before
        // AbstractMonster consumes block and player relic hooks such as Boot.
        // A pure precomputed matrix bypasses IntangiblePower recalculation,
        // but not Nemesis's concrete damage override.
        let intangible_applies = !pure_matrix || enemy.id == "Nemesis";
        let effective_damage = if intangible_applies
            && enemy.entity.status(sid::INTANGIBLE) > 0
            && effective_damage > 1
        {
            1
        } else {
            effective_damage
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
        let actual_hp_lost = hp_damage.min(hp_before.max(0)).max(0);
        enemy.entity.hp = (enemy.entity.hp - hp_damage).max(0);
        self.state.total_damage_dealt += actual_hp_lost;

        // On-hit enemy reactions (only when HP damage dealt)
        if hp_damage > 0 {
            // PlatedArmorPower.wasHPLost loses one stack after unblocked,
            // non-THORNS/non-HP_LOSS damage from another creature.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/PlatedArmorPower.java
            let plated = self.state.enemies[enemy_idx]
                .entity
                .status(sid::PLATED_ARMOR);
            if plated > 0 {
                let remaining = plated - 1;
                self.state.enemies[enemy_idx]
                    .entity
                    .set_status(sid::PLATED_ARMOR, remaining);
                if remaining == 0
                    && matches!(
                        self.state.enemies[enemy_idx].id.as_str(),
                        "Shelled Parasite" | "ShelledParasite"
                    )
                {
                    // Source: PlatedArmorPower.onRemove invokes
                    // ShelledParasite.changeState("ARMOR_BREAK"), which
                    // replaces its current intent with one Stunned turn.
                    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/
                    // PlatedArmorPower.java.
                    self.state.enemies[enemy_idx].set_move_with_intent(
                        crate::enemies::move_ids::SP_STUNNED,
                        crate::combat_types::Intent::Stun,
                    );
                }
            }

            // FlightPower.onAttacked applies its half-damage survival check a
            // second time to the post-block damageAmount. That hook runs for a
            // pure matrix too even though the first receive modifier was skipped.
            if flight > 0 && (hp_damage as f64 * 0.5) < hp_before as f64 {
                let remaining = flight - 1;
                self.state.enemies[enemy_idx]
                    .entity
                    .set_status(sid::FLIGHT, remaining);
                if remaining == 0 && self.state.enemies[enemy_idx].id == "Byrd" {
                    // FlightPower.onRemove -> Byrd.changeState("GROUNDED").
                    // Java: reference/extracted/methods/monster/Byrd.java.
                    self.state.enemies[enemy_idx].set_move_with_intent(
                        crate::enemies::move_ids::BYRD_STUNNED,
                        crate::combat_types::Intent::Stun,
                    );
                }
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

            // Malleable: gain escalating block on a positive, nonlethal
            // NORMAL hit. Java checks `damageAmount < currentHealth` before
            // damage, which is equivalent to `enemy_alive` after damage.
            // Source: decompiled/java-src/com/megacrit/cardcrawl/powers/MalleablePower.java.
            let malleable = self.state.enemies[enemy_idx].entity.status(sid::MALLEABLE);
            if enemy_alive && malleable > 0 {
                self.state.enemies[enemy_idx].entity.block += malleable;
                self.state.enemies[enemy_idx]
                    .entity
                    .add_status(sid::MALLEABLE, 1);
            }

            // Source: decompiled ReactivePower.java. Only sourced, positive,
            // nonlethal NORMAL damage queues RollMoveAction; THORNS and HP_LOSS
            // use other damage paths and never reach this hook.
            if enemy_alive && self.state.enemies[enemy_idx].entity.status(sid::REACTIVE) > 0 {
                crate::enemies::writhing_mass_reactive_reroll(
                    &mut self.state.enemies[enemy_idx],
                    &mut self.ai_rng,
                );
            }

            self.apply_shifting_after_hit(enemy_idx, hp_damage);
        }

        self.record_enemy_hp_damage(enemy_idx, hp_damage);
    }

    pub(crate) fn record_enemy_hp_damage(&mut self, enemy_idx: usize, hp_damage: i32) {
        if hp_damage <= 0 || enemy_idx >= self.state.enemies.len() {
            return;
        }

        combat_hooks::on_enemy_damaged(self, enemy_idx, hp_damage);
        // Darkling.damage owns both its half-death hooks and the final
        // all-Darklings death transition, so do not finalize it a second time.
        // Source: reference/extracted/methods/monster/Darkling.java.
        if self.state.enemies[enemy_idx].id == "Darkling"
            && self.state.enemies[enemy_idx].entity.hp <= 0
        {
            return;
        }
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

        if self.state.enemies[enemy_idx].id == "Mugger" {
            // Source: decompiled/java-src/com/megacrit/cardcrawl/monsters/
            // city/Mugger.java (`die` -> `playDeathSfx`). The voice variant
            // consumes aiRng.random(2), even when no gold was stolen.
            let _ = self.ai_rng.random_int(2);
        }

        if matches!(
            self.state.enemies[enemy_idx].id.as_str(),
            "SpireShield" | "Spire Shield" | "SpireSpear" | "Spire Spear"
        ) {
            // The dying monster is skipped by Java's partner cleanup, so a
            // BackAttack on that corpse remains. The player Surrounded removal
            // and the living partner's BackAttack removal are queued actions;
            // terminal combat clears them before they resolve.
            // Java: SpireShield.java::die, SpireSpear.java::deathReact, and
            // GameActionManager.java::clearPostCombatActions.
            if self.state.player.status(sid::SURROUNDED_POWER) > 0 {
                self.deferred_combat_ops
                    .push(DeferredCombatOp::RemovePower {
                        owner: DeferredPowerOwner::Player,
                        status: sid::SURROUNDED_POWER,
                    });
            }
            for (idx, enemy) in self.state.enemies.iter().enumerate() {
                if idx != enemy_idx
                    && enemy.is_alive()
                    && matches!(
                        enemy.id.as_str(),
                        "SpireShield" | "Spire Shield" | "SpireSpear" | "Spire Spear"
                    )
                    && enemy.has_back_attack()
                {
                    self.deferred_combat_ops
                        .push(DeferredCombatOp::RemovePower {
                            owner: DeferredPowerOwner::Enemy(idx),
                            status: sid::BACK_ATTACK_POWER,
                        });
                }
            }
        }

        if matches!(
            self.state.enemies[enemy_idx].id.as_str(),
            "GremlinLeader" | "Gremlin Leader"
        ) {
            // GremlinLeader.die appends every surviving monster's EscapeAction
            // to the bottom of GameActionManager. Card actions already queued
            // behind the killing damage therefore resolve first. In
            // particular, Wheel Kick draws before the gremlins escape, while a
            // lethal hit against the encounter's actual final monster still
            // clears its queued DrawCardAction in DamageAction.
            // Java: reference/extracted/methods/monster/GremlinLeader.java::die
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAction.java
            let survivors = self
                .state
                .enemies
                .iter()
                .enumerate()
                .filter(|(idx, enemy)| *idx != enemy_idx && enemy.is_alive())
                .map(|(idx, _)| idx)
                .collect::<Vec<_>>();
            for idx in survivors {
                if self.runtime_played_card.is_some() {
                    self.deferred_combat_ops
                        .push(DeferredCombatOp::EscapeEnemy { enemy_idx: idx });
                } else {
                    self.state.enemies[idx].is_escaping = true;
                    self.state.enemies[idx].entity.hp = 0;
                }
            }
        }

        if matches!(
            self.state.enemies[enemy_idx].id.as_str(),
            "AwakenedOne" | "Awakened One"
        ) && self.state.enemies[enemy_idx].entity.status(sid::PHASE) == 2
        {
            // AwakenedOne.die queues EscapeAction for every surviving Cultist
            // after final-form death. The compact settled state marks those
            // escaped allies terminal immediately; the callback projector
            // restores the pre-queue frame when comparing useCard records.
            // Java: reference/extracted/methods/monster/AwakenedOne.java::die.
            for (idx, enemy) in self.state.enemies.iter_mut().enumerate() {
                if idx != enemy_idx && enemy.id == "Cultist" && enemy.is_alive() {
                    enemy.is_escaping = true;
                    enemy.entity.hp = 0;
                }
            }
        }

        if matches!(
            self.state.enemies[enemy_idx].id.as_str(),
            "TheCollector" | "Collector"
        ) {
            // Source: decompiled/java-src/com/megacrit/cardcrawl/monsters/
            // city/TheCollector.java (`die`). Every surviving minion receives
            // SuicideAction when the boss dies.
            let victims: Vec<usize> = self
                .state
                .enemies
                .iter()
                .enumerate()
                .filter(|(idx, enemy)| *idx != enemy_idx && enemy.is_alive())
                .map(|(idx, _)| idx)
                .collect();
            for idx in victims {
                self.state.enemies[idx].entity.hp = 0;
                self.finalize_enemy_death(idx);
            }
        }

        if self.state.enemies[enemy_idx].id == "Reptomancer" {
            // Source: decompiled/java-src/com/megacrit/cardcrawl/monsters/
            // beyond/Reptomancer.java (`die`): every surviving monster is
            // killed by SuicideAction.
            let victims: Vec<usize> = self
                .state
                .enemies
                .iter()
                .enumerate()
                .filter(|(idx, enemy)| *idx != enemy_idx && enemy.is_alive())
                .map(|(idx, _)| idx)
                .collect();
            for idx in victims {
                self.state.enemies[idx].entity.hp = 0;
                self.finalize_enemy_death(idx);
            }
        }

        // StasisPower queues its held-card return behind the current action.
        // It chooses the discard-only action only when the hand is already
        // full here; MakeTempCardInHandAction checks capacity again later.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/StasisPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
        if let Some(card) = self.state.enemies[enemy_idx].take_stasis_card() {
            let route = if self.state.hand.len() == 10 {
                DeferredCardRoute::Discard
            } else {
                DeferredCardRoute::HandWithOverflowToDiscard
            };
            self.deferred_combat_ops
                .push(DeferredCombatOp::MoveCard { card, route });
        }

        // Spore Cloud: apply Vulnerable to player on death.
        let spore = self.state.enemies[enemy_idx]
            .entity
            .status(sid::SPORE_CLOUD);
        // SporeCloudPower.onDeath returns immediately when the room is already
        // battle-ending, so the final dying monster does not queue Vulnerable.
        // Source: decompiled/java-src/com/megacrit/cardcrawl/powers/SporeCloudPower.java:36-43
        if spore > 0 && !self.state.is_victory() {
            powers::apply_debuff(&mut self.state.player, sid::VULNERABLE, spore);
        }

        // CorpseExplosionPower.onDeath queues source-less THORNS damage equal
        // to owner.maxHealth * amount before relic onMonsterDeath actions are
        // queued. Resolve it before owner-aware relic effects so The Specimen
        // selects only among monsters that survive the explosion.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CorpseExplosionPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java
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

        // Fire owner-aware death hooks (Gremlin Horn, The Specimen, etc.)
        // after earlier monster-power death actions have resolved.
        let ctx = crate::effects::trigger::TriggerContext {
            card_type: None,
            is_first_turn: false,
            target_idx: enemy_idx as i32,
        };
        self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
            crate::effects::trigger::Trigger::OnEnemyDeath,
            &ctx,
        ));
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
        // ThornsPower.onAttacked queues retaliation for every sourced NORMAL
        // hit, including zero/fully-blocked and lethal hits. Snapshot before
        // damage because killing the owner does not cancel the queued action.
        // Source: decompiled/java-src/com/megacrit/cardcrawl/powers/ThornsPower.java.
        let enemy_thorns = self.state.enemies[enemy_idx].entity.status(sid::THORNS);
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

        if enemy_thorns > 0 {
            self.deal_thorns_damage_to_player(enemy_thorns);
        }

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

        // WristBlade.atDamageModify checks the card's actual costForTurn, or
        // freeToPlayOnce only when the permanent cost is not X (-1). Combat
        // payment normalizes X to zero elsewhere, but that must not make a
        // free Skewer/Whirlwind eligible for this damage bonus.
        // Java: reference/extracted/methods/relic/WristBlade.java.
        let permanent_cost = if card_inst.base_cost >= 0 {
            card_inst.base_cost as i32
        } else {
            card.cost
        };
        let cost_for_turn = if card_inst.cost >= 0 {
            card_inst.cost as i32
        } else {
            permanent_cost
        };
        let free_to_play_once =
            card_inst.is_free() || self.state.player.status(sid::NEXT_ATTACK_FREE) > 0;
        let wrist_blade_eligible =
            cost_for_turn == 0 || (free_to_play_once && permanent_cost != -1);
        if wrist_blade_eligible && self.state.has_relic("WristBlade") {
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
        if self.state.has_relic("Philosopher's Stone") || self.state.has_relic("PhilosopherStone") {
            enemy.entity.add_status_direct(sid::STRENGTH, 1);
        }
        self.assign_enemy_instance_id(&mut enemy);
        self.state.enemies.push(enemy);
    }

    pub(crate) fn add_spawned_minion(&mut self, mut enemy: crate::state::EnemyCombatState) {
        // SpawnMonsterAction invokes every relic's onSpawnMonster before it
        // queues ApplyPowerAction(MinionPower). Philosopher's Stone therefore
        // appends Strength first; the later sorted Minion power retains that
        // equal-priority order.
        // Java: actions/common/SpawnMonsterAction.java:45-68 and
        // relics/PhilosopherStone.java:49-52.
        if self.state.has_relic("Philosopher's Stone") || self.state.has_relic("PhilosopherStone") {
            enemy.entity.add_status_direct(sid::STRENGTH, 1);
        }
        enemy.set_minion(true);
        self.assign_enemy_instance_id(&mut enemy);
        self.state.enemies.push(enemy);
    }

    pub(crate) fn assign_enemy_instance_id(&mut self, enemy: &mut crate::state::EnemyCombatState) {
        if enemy.runtime_instance_id == 0 {
            enemy.runtime_instance_id = self.next_enemy_instance_id.max(1);
            self.next_enemy_instance_id = enemy.runtime_instance_id + 1;
        }
    }

    pub(crate) fn ensure_enemy_instance_ids(&mut self) {
        for enemy in &mut self.state.enemies {
            if enemy.runtime_instance_id == 0 {
                enemy.runtime_instance_id = self.next_enemy_instance_id.max(1);
                self.next_enemy_instance_id = enemy.runtime_instance_id + 1;
            }
        }
    }

    fn initialize_gremlin_leader_slots(&mut self) {
        let Some(leader_idx) = self
            .state
            .enemies
            .iter()
            .position(|enemy| matches!(enemy.id.as_str(), "GremlinLeader" | "Gremlin Leader"))
        else {
            return;
        };

        // GremlinLeader.usePreBattleAction captures MonsterGroup members 0 and
        // 1 in slots 0 and 1, leaving slot 2 null. Normal encounter creation
        // orders those two gremlins before the Leader; the bounded fallback
        // also keeps hand-built fixtures faithful when they use that shape.
        for (slot, idx) in (0..leader_idx).take(2).enumerate() {
            self.gremlin_leader_slots[slot] = Some(idx);
        }
    }

    fn initialize_reptomancer_dagger_slots(&mut self) {
        let Some(repto_idx) = self
            .state
            .enemies
            .iter()
            .position(|enemy| enemy.id == "Reptomancer")
        else {
            return;
        };
        for (idx, enemy) in self.state.enemies.iter_mut().enumerate() {
            if enemy.id != "Dagger" {
                continue;
            }
            let marked = enemy.entity.status(crate::status_ids::sid::COUNT);
            let slot = if (1..=4).contains(&marked) {
                marked as usize - 1
            } else if idx < repto_idx {
                // Reptomancer.usePreBattleAction maps the initial left Dagger
                // to slot 1 and the initial right Dagger to slot 0.
                1
            } else {
                0
            };
            enemy
                .entity
                .set_status(crate::status_ids::sid::COUNT, slot as i32 + 1);
            self.reptomancer_dagger_slots[slot] = Some(idx);
        }
    }

    pub(crate) fn deal_thorns_damage_to_player(&mut self, damage: i32) {
        self.deal_thorns_damage_to_player_with_owner(damage, false);
    }

    pub(crate) fn deal_self_thorns_damage_to_player(&mut self, damage: i32) {
        self.deal_thorns_damage_to_player_with_owner(damage, true);
    }

    fn deal_thorns_damage_to_player_with_owner(&mut self, damage: i32, owner_is_player: bool) {
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
            self.player_lose_hp_with_owner(hp_damage, owner_is_player);
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

        let hp_before = self.state.enemies[enemy_idx].entity.hp;
        let actual_hp_lost = hp_damage.min(hp_before.max(0)).max(0);
        self.state.enemies[enemy_idx].entity.hp =
            (self.state.enemies[enemy_idx].entity.hp - hp_damage).max(0);
        self.state.total_damage_dealt += actual_hp_lost;

        // ShiftingPower.onAttacked applies to positive damage of every type.
        self.apply_shifting_after_hit(enemy_idx, hp_damage);

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

        let hp_before = self.state.enemies[enemy_idx].entity.hp;
        self.state.enemies[enemy_idx].entity.hp =
            (self.state.enemies[enemy_idx].entity.hp - hp_damage).max(0);
        let actual_hp_lost = hp_damage.min(hp_before).max(0);
        self.state.total_damage_dealt += actual_hp_lost;

        self.apply_shifting_after_hit(enemy_idx, hp_damage);

        self.record_enemy_hp_damage(enemy_idx, actual_hp_lost);
        actual_hp_lost
    }

    pub(crate) fn execute_card_effects_with_enemy_on_use(
        &mut self,
        card: &CardDef,
        card_inst: CardInstance,
        target_idx: i32,
    ) {
        self.update_surrounded_facing(card, target_idx);

        // SharpHidePower.onUseCard queues one THORNS hit for every living
        // owner when each Attack card use is constructed, including replayed
        // and autoplayed cards. Capture before resolution so killing the
        // Guardian does not cancel its already-queued retaliation.
        // Sources: decompiled/java-src/com/megacrit/cardcrawl/powers/
        // SharpHidePower.java and actions/utility/UseCardAction.java.
        let sharp_hide_retaliations: Vec<i32> = if card.card_type == CardType::Attack {
            self.state
                .enemies
                .iter()
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

    fn update_surrounded_facing(&mut self, card: &CardDef, target_idx: i32) {
        if !matches!(card.target, CardTarget::Enemy | CardTarget::SelfAndEnemy) {
            return;
        }
        self.update_surrounded_facing_for_target(target_idx);
    }

    fn update_surrounded_facing_for_target(&mut self, target_idx: i32) {
        if self.state.player.status(sid::SURROUNDED_POWER) <= 0
            || target_idx < 0
            || (target_idx as usize) >= self.state.enemies.len()
            || !self.state.enemies[target_idx as usize].is_targetable()
        {
            return;
        }

        let target_idx = target_idx as usize;
        if !matches!(
            self.state.enemies[target_idx].id.as_str(),
            "SpireShield" | "Spire Shield" | "SpireSpear" | "Spire Spear"
        ) {
            return;
        }

        // Source: decompiled/java-src/com/megacrit/cardcrawl/characters/
        // AbstractPlayer.java (`updateInput`). Targeting either flanking
        // monster turns the player toward it; AbstractMonster.applyBackAttack
        // then marks the monster on the opposite side.
        for (idx, enemy) in self.state.enemies.iter_mut().enumerate() {
            if matches!(
                enemy.id.as_str(),
                "SpireShield" | "Spire Shield" | "SpireSpear" | "Spire Spear"
            ) && enemy.is_alive()
            {
                enemy.set_back_attack(idx != target_idx);
            }
        }
    }

    fn capture_enemy_choke_hp_losses(&self) -> Vec<(usize, i32)> {
        self.state
            .enemies
            .iter()
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
        let roll = self.card_random_rng.random_int(living.len() as i32 - 1) as usize;
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

    /// Execute `TriggerEndOfTurnOrbsAction` itself: invoke every orb callback
    /// synchronously, but append the callbacks' child actions to the bottom of
    /// the shared end-turn queue. Dark mutates its evoke amount in the callback
    /// and therefore remains synchronous. Lightning's intermediate passive
    /// action is kept explicit because it is a Java `DAMAGE` survivor; Frost's
    /// GainBlockAction is likewise retained after an earlier lethal hit.
    ///
    /// This placement also matters for Pride: its ordinary action was queued
    /// behind Trigger before Trigger ran, so it stays ahead of these children.
    /// Java: actions/defect/TriggerEndOfTurnOrbsAction.java,
    /// orbs/Lightning.java, orbs/Frost.java, orbs/Dark.java.
    fn collect_orb_end_of_turn_actions(&mut self) {
        if !self.state.orb_slots.has_orbs() {
            return;
        }

        let hit_all =
            self.effect_runtime
                .player_power_active(self, "electrodynamics", sid::ELECTRODYNAMICS);
        let focus = self.state.player.focus();
        let effects = self.state.orb_slots.trigger_end_of_turn_passives(focus);
        let mut actions = Vec::new();
        for effect in effects {
            match effect {
                PassiveEffect::LightningDamage(damage) => {
                    actions.push(EndTurnQueuedAction::ResolveLightningEndTurn { damage, hit_all });
                }
                PassiveEffect::FrostBlock(block) => {
                    actions.push(EndTurnQueuedAction::GainBlock(block));
                }
                PassiveEffect::PlasmaEnergy(_) | PassiveEffect::None => {}
            }
        }

        // Gold-Plated Cables invokes the front orb callback once more after
        // all normal callbacks, preserving the same synchronous/queued split.
        if self.state.has_relic("Cables") {
            let front_effect = self.state.orb_slots.slots.first_mut().and_then(|front| {
                if front.is_empty() {
                    return None;
                }
                match front.orb_type {
                    crate::orbs::OrbType::Lightning => {
                        Some(EndTurnQueuedAction::ResolveLightningEndTurn {
                            damage: front.passive_with_focus(focus),
                            hit_all,
                        })
                    }
                    crate::orbs::OrbType::Frost => Some(EndTurnQueuedAction::GainBlock(
                        front.passive_with_focus(focus),
                    )),
                    crate::orbs::OrbType::Dark => {
                        front.evoke_amount += front.passive_with_focus(focus);
                        None
                    }
                    crate::orbs::OrbType::Plasma | crate::orbs::OrbType::Empty => None,
                }
            });
            if let Some(action) = front_effect {
                actions.push(action);
            }
        }

        self.end_turn_actions.extend(actions);
    }

    /// Collect Plasma's start callback as concrete bottom actions. Java walks
    /// every orb after relic/power callbacks, then repeats the front orb for
    /// Gold-Plated Cables; the GainEnergyActions do not execute during the walk.
    fn collect_orb_start_of_turn_actions(&mut self) {
        for orb in &self.state.orb_slots.slots {
            if orb.orb_type == crate::orbs::OrbType::Plasma {
                self.turn_start_actions
                    .push(TurnStartQueuedAction::GainEnergy(orb.base_passive));
            }
        }
        if self.state.has_relic("Cables") {
            if let Some(front) = self.state.orb_slots.slots.first() {
                if front.orb_type == crate::orbs::OrbType::Plasma {
                    self.turn_start_actions
                        .push(TurnStartQueuedAction::GainEnergy(front.base_passive));
                }
            }
        }
    }

    fn collect_one_orb_impulse_actions(&mut self, index: usize) {
        let Some(orb) = self.state.orb_slots.slots.get(index) else {
            return;
        };
        if orb.is_empty() {
            return;
        }
        let orb_type = orb.orb_type;
        let base_passive = orb.base_passive;
        let passive = orb.passive_with_focus(self.state.player.focus());

        // ImpulseAction invokes onStartOfTurn before onEndOfTurn.
        if orb_type == crate::orbs::OrbType::Plasma {
            self.queue_turn_start_action_bottom(TurnStartQueuedAction::GainEnergy(base_passive));
        }
        match orb_type {
            crate::orbs::OrbType::Lightning => {
                self.queue_turn_start_action_bottom(TurnStartQueuedAction::OrbLightning {
                    damage: passive,
                    hit_all: self.state.player.status(sid::ELECTRODYNAMICS) > 0,
                })
            }
            crate::orbs::OrbType::Frost => {
                self.queue_turn_start_action_bottom(TurnStartQueuedAction::GainBlock(passive));
            }
            crate::orbs::OrbType::Dark => {
                self.state.orb_slots.slots[index].evoke_amount += passive;
            }
            crate::orbs::OrbType::Plasma | crate::orbs::OrbType::Empty => {}
        }
    }

    fn collect_front_orb_impulse_actions(&mut self) {
        self.collect_one_orb_impulse_actions(0);
    }

    fn collect_all_orb_impulse_actions(&mut self) {
        let count = self.state.orb_slots.slots.len();
        for index in 0..count {
            self.collect_one_orb_impulse_actions(index);
        }
        if self.state.has_relic("Cables") {
            self.collect_front_orb_impulse_actions();
        }
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
                        crate::orbs::OrbType::Lightning => {
                            effects.push(crate::orbs::PassiveEffect::LightningDamage(
                                front.passive_with_focus(focus),
                            ))
                        }
                        crate::orbs::OrbType::Frost => effects.push(
                            crate::orbs::PassiveEffect::FrostBlock(front.passive_with_focus(focus)),
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
        // DrawCardAction recursively splits an overdraw into "draw current
        // pile, shuffle, draw remainder". Once a nonempty draw pile has been
        // scheduled for exhaustion, the intervening EmptyDeckShuffleAction
        // still shuffles (and consumes shuffleRng) even if the discard pile is
        // empty by the time that action runs. Java: DrawCardAction.java::update
        // and EmptyDeckShuffleAction.java::update.
        let mut forced_shuffle_after_current_pile = false;

        for draw_index in 0..actual_count {
            if self.state.hand.len() >= 10 {
                break; // Hand size limit
            }

            if self.state.draw_pile.is_empty() {
                // Shuffle discard into draw
                if self.state.discard_pile.is_empty() && !forced_shuffle_after_current_pile {
                    break; // No cards left anywhere
                }
                let mut shuffled = std::mem::take(&mut self.state.discard_pile);
                crate::seed::card_group_shuffle(&mut shuffled, &mut self.shuffle_rng);
                self.state.draw_pile = shuffled;
                shuffles += 1;
                forced_shuffle_after_current_pile = false;
                if self.state.draw_pile.is_empty() {
                    break;
                }
            }

            let draws_remaining = actual_count - draw_index;
            if draws_remaining > self.state.draw_pile.len() as i32 {
                forced_shuffle_after_current_pile = true;
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
                    let new_cost = self.card_random_rng.random_int(3) as i8;
                    drawn.set_permanent_cost(new_cost);
                    if matches!(card_def.id, "Force Field" | "Force Field+") {
                        // ConfusionPower overwrites both cost and costForTurn,
                        // erasing Force Field reductions from earlier Power
                        // plays. Only later Power cards reduce this new cost.
                        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ConfusionPower.java
                        drawn.misc = self.state.power_cards_played_this_combat.max(0);
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
                && self
                    .choice
                    .as_ref()
                    .is_some_and(|choice| choice.reason == ChoiceReason::Scry)
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
        crate::seed::card_group_shuffle(&mut self.state.draw_pile, &mut self.shuffle_rng);
    }

    /// Execute Reboot's ShuffleAllAction, explicit ShuffleAction, and draw.
    /// ShuffleAllAction's constructor fires relic hooks before its pile work;
    /// Melange can therefore pause this continuation on an interactive Scry.
    pub(crate) fn shuffle_all_and_draw(&mut self, draw_count: i32) {
        self.emit_event(crate::effects::runtime::GameEvent::empty(
            crate::effects::trigger::Trigger::OnShuffle,
        ));
        if self.phase == CombatPhase::AwaitingChoice {
            if let Some(choice) = self.choice.as_mut() {
                choice.deferred_shuffle_all_draw = Some(draw_count);
            }
            return;
        }
        self.continue_shuffle_all_and_draw(draw_count);
    }

    fn continue_shuffle_all_and_draw(&mut self, draw_count: i32) {
        // ShuffleAllAction always shuffles the discard group, even when empty.
        // CardGroup.shuffle consumes exactly one shuffleRng.randomLong().
        crate::seed::card_group_shuffle(&mut self.state.discard_pile, &mut self.shuffle_rng);

        // PutOnDeckAction selects each remaining hand card with one inclusive
        // cardRandomRng.random(size - 1), including the singleton case, then
        // puts it on top of the draw pile without firing discard callbacks.
        while !self.state.hand.is_empty() {
            let idx = self
                .card_random_rng
                .random_int((self.state.hand.len() - 1) as i32) as usize;
            let card = self.state.hand.remove(idx);
            self.state.draw_pile.push(card);
        }

        // ShuffleAllAction iterates the pre-shuffled discard group from bottom
        // to top and Soul.shuffle adds each card to the top of the draw group.
        self.state.draw_pile.append(&mut self.state.discard_pile);

        // Reboot then queues ShuffleAction(drawPile, false), which consumes a
        // second shuffleRng.randomLong() but does not trigger relics again.
        // Java: reference/extracted/methods/card/Reboot.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/PutOnDeckAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ShuffleAction.java
        crate::seed::card_group_shuffle(&mut self.state.draw_pile, &mut self.shuffle_rng);
        self.draw_cards(draw_count.max(0));
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
        card.flags |= CardInstance::FLAG_FREE | CardInstance::FLAG_AUTOPLAY;
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
                && self.card_random_rng.random_bool();
            if spoon_saved {
                self.state.discard_pile.push(post_play_card);
            } else {
                self.state.exhaust_pile.push(post_play_card);
                self.trigger_card_on_exhaust(post_play_card);
            }
        }
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
                .random_int(self.state.hand.len() as i32 - 1) as usize;
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
            // MantraPower.stackPower subtracts ten and queues removal before
            // ChangeStanceAction when no amount remains.
            // Java: powers/watcher/MantraPower.java.
            self.state.player.set_status(sid::MANTRA, self.state.mantra);
            self.change_stance(Stance::Divinity);
        } else {
            // Keep the Java-visible power synchronized with the compact scalar
            // used by combat calculations and trace projection.
            self.state.player.set_status(sid::MANTRA, self.state.mantra);
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
            .filter_map(|(idx, card)| self.card_registry.can_upgrade_card(card).then_some(idx))
            .collect();
        if eligible.is_empty() {
            return None;
        }
        let selected = self
            .misc_rng
            .random_int_range(0, (eligible.len() - 1) as i32) as usize;
        let deck_idx = eligible[selected];
        self.card_registry
            .upgrade_card(&mut self.state.master_deck[deck_idx]);
        Some(deck_idx)
    }

    // =======================================================================
    // Helpers: Temp Card Creation (respects Master Reality)
    // =======================================================================

    /// Get a CardInstance for a temporary card, upgrading if Master Reality is active.
    pub fn temp_card(&mut self, base_id: &str) -> CardInstance {
        let base = self.card_registry.get(base_id);
        let upgraded_id = format!("{}+", base_id);
        // MakeTempCard* excludes Status and Curse cards from Master Reality.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
        let master_reality_can_upgrade = self.state.player.status(sid::MASTER_REALITY) > 0
            && base.is_some_and(|def| !matches!(def.card_type, CardType::Curse | CardType::Status))
            && self.card_registry.get(&upgraded_id).is_some();
        let card = if master_reality_can_upgrade {
            self.card_registry.make_card(&upgraded_id)
        } else {
            self.card_registry.make_card(base_id)
        };
        self.fresh_stat_copy(card)
    }

    /// Java `makeStatEquivalentCopy`: preserve card state but mint a fresh
    /// identity. Replays such as Echo Form and Necronomicon intentionally keep
    /// using the original `CardInstance` instead.
    pub(crate) fn fresh_stat_copy(&mut self, mut card: CardInstance) -> CardInstance {
        card.instance_id = self.state.allocate_card_instance_id();
        card
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

    pub(crate) fn add_card_instance_copies_to_hand(&mut self, card: CardInstance, amount: i32) {
        if amount <= 0 {
            return;
        }
        for _ in 0..amount {
            let mut copy = card;
            copy.reset_cost_for_turn();
            let copy = self.fresh_stat_copy(copy);
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

    /// Roll and obtain a limited random potion into the first empty slot.
    pub fn obtain_random_potion(&mut self) -> bool {
        // Alchemize evaluates returnRandomPotion(true) while building its
        // ObtainPotionAction, before that action checks Sozu or inventory
        // capacity. The roll therefore always consumes potionRng and uses the
        // current Watcher pool's rarity/rejection algorithm.
        // Java: cards/green/Alchemize.java, dungeons/AbstractDungeon.java,
        // and actions/common/ObtainPotionAction.java.
        let potion_id = crate::potions::defs::entropic_brew::roll_limited_watcher_potion(self);

        if self.state.has_relic("Sozu") {
            return false;
        }
        let Some(slot) = self.state.potions.iter().position(|p| p.is_empty()) else {
            return false;
        };
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

    /// Evoke the first orb and re-channel that same instance, as Recursion's
    /// RedoAction does. Reusing the object preserves a Dark orb's accumulated
    /// evoke amount. The ChannelAction is non-damage work queued behind the
    /// evoke, so lethal orb damage clears it before it can resolve.
    /// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/RedoAction.java
    /// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ChannelAction.java
    /// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DarkOrbEvokeAction.java
    pub(crate) fn evoke_and_rechannel_front_orb(&mut self) {
        let Some(orb) = self
            .state
            .orb_slots
            .slots
            .first()
            .filter(|orb| !orb.is_empty())
            .cloned()
        else {
            return;
        };

        let focus = self.state.player.focus();
        let effect = self.state.orb_slots.evoke_front(focus);
        self.apply_evoke_effect(effect);
        if self.state.combat_over || self.state.is_victory() {
            return;
        }

        let Some(empty_idx) = self
            .state
            .orb_slots
            .slots
            .iter()
            .position(|slot| slot.is_empty())
        else {
            return;
        };
        let orb_type = orb.orb_type;
        self.state.orb_slots.slots[empty_idx] = orb;
        match orb_type {
            crate::orbs::OrbType::Lightning => {
                self.state.player.add_status(sid::LIGHTNING_CHANNELED, 1);
            }
            crate::orbs::OrbType::Frost => {
                self.state.player.add_status(sid::FROST_CHANNELED, 1);
            }
            _ => {}
        }
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
            self.deferred_combat_ops.clear();
            self.apply_player_on_victory();
            return true;
        }

        // Defeat: player dead
        if self.state.is_defeat() {
            self.state.combat_over = true;
            self.state.player_won = false;
            self.phase = CombatPhase::CombatOver;
            self.choice = None;
            self.deferred_combat_ops.clear();
            return true;
        }

        false
    }

    /// Run `AbstractPlayer.onVictory` callbacks. `AbstractRoom.endBattle`
    /// invokes this for both defeated encounters and Smoke Bomb escapes, even
    /// though a smoked room grants no claimable combat rewards.
    ///
    /// Java: characters/AbstractPlayer.java::onVictory and
    /// rooms/AbstractRoom.java::endBattle.
    pub(crate) fn apply_player_on_victory(&mut self) {
        // AbstractRoom.endBattle invokes Meat on the Bone's standalone
        // onTrigger before AbstractPlayer.onVictory iterates relics in owned
        // order. This matters when an earlier Burning/Black Blood heal would
        // otherwise move the player above Meat's half-HP threshold.
        // Java: rooms/AbstractRoom.java::endBattle and
        // relics/MeatOnTheBone.java::onTrigger.
        if self.state.has_relic("Meat on the Bone")
            && self.state.player.hp > 0
            && (self.state.player.hp as f32) <= (self.state.player.max_hp as f32) / 2.0
        {
            self.heal_player(12);
        }

        // Fire AbstractPlayer.onVictory relics via unified dispatch (Burning
        // Blood, Black Blood, Face of Cleric), then power callbacks.
        let ctx = crate::effects::trigger::TriggerContext::empty();
        self.emit_event(crate::effects::runtime::GameEvent::from_trigger(
            crate::effects::trigger::Trigger::CombatVictory,
            &ctx,
        ));

        // RepairPower heals only while the player is alive.
        let repair = self.state.player.status(sid::SELF_REPAIR);
        if self.state.player.hp > 0 && repair > 0 {
            self.heal_player(repair);
        }
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
    fn draw_over_total_cards_preserves_java_empty_followup_shuffle_tick() {
        // DrawCardAction(7) with five draw cards recursively schedules Draw(5),
        // EmptyDeckShuffle, Draw(2). After the one discard card is drawn, the
        // remaining Draw(1) schedules a second EmptyDeckShuffleAction even
        // though its discard group is empty. CardGroup.shuffle still consumes
        // shuffleRng.randomLong(). Java: DrawCardAction.java:73-85,
        // EmptyDeckShuffleAction.java:39-44, CardGroup.java:550-555.
        let mut state = make_test_state();
        state.hand.clear();
        state.draw_pile = make_deck(&["Strike", "Strike", "Strike", "Strike", "Strike"]);
        state.discard_pile = make_deck(&["Vigilance"]);
        let mut engine = CombatEngine::new(state, 42);
        let before = engine.shuffle_rng.counter;

        let drawn = engine.draw_cards(7);

        assert_eq!(drawn.len(), 6);
        assert_eq!(engine.state.hand.len(), 6);
        assert!(engine.state.draw_pile.is_empty());
        assert!(engine.state.discard_pile.is_empty());
        assert_eq!(engine.shuffle_rng.counter, before + 2);
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
    fn rushdown_draw_waits_until_the_stance_card_reaches_discard() {
        // RushdownPower.onChangeStance queues DrawCardAction at the bottom.
        // Eruption's UseCardAction therefore moves Eruption into discard
        // before a two-card Rushdown draw checks the piles. With Vigilance
        // already discarded, Java shuffles once and draws both cards.
        // Java: powers/watcher/RushdownPower.java,
        // actions/utility/UseCardAction.java, and DrawCardAction.java.
        let mut state = make_test_state();
        state.player.set_status(sid::RUSHDOWN, 2);
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        engine.state.stance = Stance::Calm;
        engine.state.hand = make_deck(&["Eruption"]);
        engine.state.draw_pile.clear();
        engine.state.discard_pile = make_deck(&["Vigilance"]);
        let shuffle_before = engine.shuffle_rng.counter;

        engine.execute_action(&Action::PlayCard {
            card_idx: 0,
            target_idx: 0,
        });

        let hand = engine
            .state
            .hand
            .iter()
            .map(|card| engine.card_registry.card_name(card.def_id))
            .collect::<Vec<_>>();
        assert_eq!(engine.state.stance, Stance::Wrath);
        assert_eq!(engine.shuffle_rng.counter, shuffle_before + 1);
        assert!(engine.state.discard_pile.is_empty());
        assert_eq!(hand.len(), 2);
        assert!(hand.contains(&"Eruption"));
        assert!(hand.contains(&"Vigilance"));
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
        state.draw_pile = make_deck(&["InnerPeace", "Strike", "Strike", "Strike", "Strike"]);

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
        state.draw_pile = make_deck(&["MentalFortress", "Strike", "Strike", "Strike", "Strike"]);

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
