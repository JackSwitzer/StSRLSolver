//! Owner-aware runtime for relic, power, and potion entity definitions.
//!
//! This is the combat-facing execution kernel for EntityDefs. It installs
//! concrete runtime instances for the current combat, precomputes handler
//! lists by event kind, owns hidden per-instance state, and emits a
//! deterministic event trace for debugging / RL inspection.

use smallvec::SmallVec;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

use crate::cards::CardType;
use crate::combat_types::CardInstance;
use crate::effects::declarative::{AmountSource, Effect, Pile, SimpleEffect, Target};
use crate::effects::entity_def::EntityDef;
use crate::effects::trigger::{Trigger, TriggerCondition, TriggerContext};
use crate::engine::CombatEngine;
use crate::ids::StatusId;
use crate::status_ids::sid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectOwner {
    PlayerRelic { slot: u16 },
    PlayerPower,
    EnemyPower { enemy_idx: u16 },
    PotionSlot { slot: u8 },
    RunEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistenceScope {
    Combat,
    Run,
}

impl Hash for Trigger {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*self as u8).hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectState {
    pub persistence: PersistenceScope,
    values: SmallVec<[i16; 4]>,
}

impl EffectState {
    pub fn new(persistence: PersistenceScope) -> Self {
        Self {
            persistence,
            values: SmallVec::new(),
        }
    }

    pub fn from_values(persistence: PersistenceScope, values: Vec<i16>) -> Self {
        Self {
            persistence,
            values: SmallVec::from_vec(values),
        }
    }

    pub fn get(&self, slot: usize) -> i32 {
        self.values.get(slot).copied().unwrap_or(0) as i32
    }

    pub fn set(&mut self, slot: usize, value: i32) {
        while self.values.len() <= slot {
            self.values.push(0);
        }
        self.values[slot] = value as i16;
    }

    pub fn add(&mut self, slot: usize, delta: i32) {
        let next = self.get(slot) + delta;
        self.set(slot, next);
    }

    pub fn as_vec(&self) -> Vec<i16> {
        self.values.to_vec()
    }
}

impl Default for EffectState {
    fn default() -> Self {
        Self::new(PersistenceScope::Combat)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedEffectState {
    pub def_id: String,
    pub values: Vec<i16>,
}

#[derive(Debug, Clone)]
pub struct EntityInstance {
    pub instance_id: u32,
    pub def: &'static EntityDef,
    pub owner: EffectOwner,
    pub state: EffectState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameEvent {
    pub kind: Trigger,
    pub card_type: Option<CardType>,
    pub card_inst: Option<CardInstance>,
    pub is_first_turn: bool,
    pub target_idx: i32,
    pub enemy_idx: i32,
    pub potion_slot: i32,
    pub status_id: Option<StatusId>,
    pub amount: i32,
    pub replay_window: bool,
}

impl GameEvent {
    pub fn empty(kind: Trigger) -> Self {
        Self {
            kind,
            card_type: None,
            card_inst: None,
            is_first_turn: false,
            target_idx: -1,
            enemy_idx: -1,
            potion_slot: -1,
            status_id: None,
            amount: 0,
            replay_window: false,
        }
    }

    pub fn from_trigger(kind: Trigger, ctx: &TriggerContext) -> Self {
        Self {
            kind,
            card_type: ctx.card_type,
            card_inst: None,
            is_first_turn: ctx.is_first_turn,
            target_idx: ctx.target_idx,
            enemy_idx: -1,
            potion_slot: -1,
            status_id: None,
            amount: 0,
            replay_window: false,
        }
    }

    pub fn trigger_context(&self) -> TriggerContext {
        TriggerContext {
            card_type: self.card_type,
            is_first_turn: self.is_first_turn,
            target_idx: self.target_idx,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventRecordPhase {
    Emitted,
    Handled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectExecutionPhase {
    Declarative,
    Hook,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEventRecord {
    pub phase: EventRecordPhase,
    pub event: Trigger,
    pub owner: Option<EffectOwner>,
    pub def_id: Option<&'static str>,
    pub execution: Option<EffectExecutionPhase>,
    pub card_type: Option<CardType>,
    pub is_first_turn: bool,
    pub target_idx: i32,
    pub enemy_idx: i32,
    pub potion_slot: i32,
    pub status_id: Option<StatusId>,
    pub amount: i32,
}

impl GameEventRecord {
    pub fn emitted(event: GameEvent) -> Self {
        Self {
            phase: EventRecordPhase::Emitted,
            event: event.kind,
            owner: None,
            def_id: None,
            execution: None,
            card_type: event.card_type,
            is_first_turn: event.is_first_turn,
            target_idx: event.target_idx,
            enemy_idx: event.enemy_idx,
            potion_slot: event.potion_slot,
            status_id: event.status_id,
            amount: event.amount,
        }
    }

    pub fn handled(
        event: &GameEvent,
        owner: EffectOwner,
        def_id: &'static str,
        execution: EffectExecutionPhase,
    ) -> Self {
        Self {
            phase: EventRecordPhase::Handled,
            event: event.kind,
            owner: Some(owner),
            def_id: Some(def_id),
            execution: Some(execution),
            card_type: event.card_type,
            is_first_turn: event.is_first_turn,
            target_idx: event.target_idx,
            enemy_idx: event.enemy_idx,
            potion_slot: event.potion_slot,
            status_id: event.status_id,
            amount: event.amount,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DispatchTable {
    handlers: Vec<Vec<usize>>,
}

impl DispatchTable {
    fn new() -> Self {
        let len = Trigger::OnPoisonApplied as usize + 1;
        Self {
            handlers: vec![Vec::new(); len],
        }
    }

    fn clear(&mut self) {
        for handlers in &mut self.handlers {
            handlers.clear();
        }
    }

    fn add(&mut self, trigger: Trigger, instance_idx: usize) {
        self.handlers[trigger as usize].push(instance_idx);
    }

    fn handlers_for(&self, trigger: Trigger) -> Vec<usize> {
        self.handlers[trigger as usize].clone()
    }
}

impl Default for DispatchTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct EffectRuntime {
    next_instance_id: u32,
    instances: Vec<EntityInstance>,
    dispatch: DispatchTable,
    persisted_states: Vec<PersistedEffectState>,
}

impl EffectRuntime {
    pub fn rebuild_from_state(&mut self, state: &crate::state::CombatState) {
        let mut old_instances = std::mem::take(&mut self.instances);
        let persisted_states = self.persisted_states.clone();

        for (slot, relic_id) in state.relics.iter().enumerate() {
            if let Some(def) = crate::relics::defs::relic_def_by_id(relic_id) {
                let owner = EffectOwner::PlayerRelic {
                    slot: slot as u16,
                };
                let mut effect_state = take_or_default_state(
                    &mut old_instances,
                    owner,
                    def,
                    &persisted_states,
                );
                seed_hidden_relic_state(state, def, &mut effect_state);
                let instance_id = self.alloc_instance_id();
                self.instances.push(EntityInstance {
                    instance_id,
                    def,
                    owner,
                    state: effect_state,
                });
            }
        }

        for def in crate::powers::defs::RUNTIME_PLAYER_POWER_DEFS
            .iter()
            .copied()
            .chain([
                &crate::powers::defs::DEF_ENVENOM,
                &crate::powers::defs::DEF_ELECTRODYNAMICS,
            ])
        {
            if let Some(guard) = def.status_guard {
                if state.player.status(guard) <= 0 {
                    continue;
                }
            } else {
                continue;
            }
            let owner = EffectOwner::PlayerPower;
            let effect_state = take_or_default_state(
                &mut old_instances,
                owner,
                def,
                &persisted_states,
            );
            let instance_id = self.alloc_instance_id();
            self.instances.push(EntityInstance {
                instance_id,
                def,
                owner,
                state: effect_state,
            });
        }

        for (enemy_idx, enemy) in state.enemies.iter().enumerate() {
            for def in crate::powers::defs::RUNTIME_ENEMY_POWER_DEFS
                .iter()
                .copied()
                .chain(std::iter::once(&crate::powers::defs::DEF_TIME_WARP))
            {
                if !enemy_power_is_active(def, enemy) {
                    continue;
                }
                let owner = EffectOwner::EnemyPower {
                    enemy_idx: enemy_idx as u16,
                };
                let effect_state = take_or_default_state(
                    &mut old_instances,
                    owner,
                    def,
                    &persisted_states,
                );
                let instance_id = self.alloc_instance_id();
                self.instances.push(EntityInstance {
                    instance_id,
                    def,
                    owner,
                    state: effect_state,
                });
            }
        }

        for (slot, potion_id) in state.potions.iter().enumerate() {
            if potion_id.is_empty() {
                continue;
            }
            if let Some(def) = crate::potions::defs::potion_def_by_runtime_id(potion_id) {
                let owner = EffectOwner::PotionSlot { slot: slot as u8 };
                let effect_state = take_or_default_state(
                    &mut old_instances,
                    owner,
                    def,
                    &persisted_states,
                );
                let instance_id = self.alloc_instance_id();
                self.instances.push(EntityInstance {
                    instance_id,
                    def,
                    owner,
                    state: effect_state,
                });
            }
        }

        self.rebuild_dispatch();
    }

    pub fn load_persisted_states(&mut self, states: Vec<PersistedEffectState>) {
        self.persisted_states = states;
    }

    pub fn export_persisted_states(&self) -> Vec<PersistedEffectState> {
        self.instances
            .iter()
            .filter(|instance| instance.state.persistence == PersistenceScope::Run)
            .map(|instance| PersistedEffectState {
                def_id: instance.def.id.to_string(),
                values: instance.state.as_vec(),
            })
            .collect()
    }

    pub fn instances(&self) -> &[EntityInstance] {
        &self.instances
    }

    pub fn persisted_states(&self) -> &[PersistedEffectState] {
        &self.persisted_states
    }

    pub fn emit(&mut self, engine: &mut CombatEngine, event: GameEvent) {
        engine.event_log.push(GameEventRecord::emitted(event));
        let handlers = self.dispatch.handlers_for(event.kind);
        for idx in handlers {
            if idx >= self.instances.len() {
                continue;
            }
            self.execute_instance(engine, idx, &event);
            if engine.state.combat_over {
                break;
            }
        }
        self.persisted_states = self.export_persisted_states();
    }

    pub fn emit_hooks_for_defs(
        &mut self,
        engine: &mut CombatEngine,
        event: GameEvent,
        def_ids: &[&str],
    ) {
        engine.event_log.push(GameEventRecord::emitted(event));
        for idx in 0..self.instances.len() {
            let def = self.instances[idx].def;
            if !def_ids.iter().any(|candidate| *candidate == def.id) {
                continue;
            }
            self.execute_instance_hook_only(engine, idx, &event);
            if engine.state.combat_over {
                break;
            }
        }
        self.persisted_states = self.export_persisted_states();
    }

    pub fn emit_replay_window(
        &mut self,
        engine: &mut CombatEngine,
        card_type: CardType,
        target_idx: i32,
        card_inst: CardInstance,
    ) {
        let ctx = TriggerContext {
            card_type: Some(card_type),
            is_first_turn: engine.state.turn == 1,
            target_idx,
        };

        let mut post_event = GameEvent::from_trigger(Trigger::OnCardPlayedPost, &ctx);
        post_event.card_inst = Some(card_inst);
        post_event.replay_window = true;
        self.emit_hooks_for_replay_defs(engine, post_event);
        if engine.state.combat_over {
            return;
        }

        match card_type {
            CardType::Attack => {
                let mut attack_event = GameEvent::from_trigger(Trigger::OnAttackPlayed, &ctx);
                attack_event.card_inst = Some(card_inst);
                attack_event.replay_window = true;
                self.emit_hooks_for_replay_defs(engine, attack_event);
            }
            CardType::Skill => {
                let mut skill_event = GameEvent::from_trigger(Trigger::OnSkillPlayed, &ctx);
                skill_event.card_inst = Some(card_inst);
                skill_event.replay_window = true;
                self.emit_hooks_for_replay_defs(engine, skill_event);
            }
            _ => {}
        }
    }

    pub fn hidden_value(&self, def_id: &str, owner: EffectOwner, slot: usize) -> i32 {
        self.instances
            .iter()
            .find(|instance| instance.def.id == def_id && instance.owner == owner)
            .map(|instance| instance.state.get(slot))
            .unwrap_or(0)
    }

    pub fn set_hidden_value(
        &mut self,
        def_id: &str,
        owner: EffectOwner,
        slot: usize,
        value: i32,
    ) -> bool {
        if let Some(instance) = self
            .instances
            .iter_mut()
            .find(|instance| instance.def.id == def_id && instance.owner == owner)
        {
            instance.state.set(slot, value);
            return true;
        }
        false
    }

    pub fn has_instance(&self, def_id: &str, owner: EffectOwner) -> bool {
        self.instances
            .iter()
            .any(|instance| instance.def.id == def_id && instance.owner == owner)
    }

    pub fn player_power_active(
        &self,
        engine: &CombatEngine,
        def_id: &str,
        guard: StatusId,
    ) -> bool {
        self.has_instance(def_id, EffectOwner::PlayerPower) || engine.state.player.status(guard) > 0
    }

    fn emit_hooks_for_replay_defs(
        &mut self,
        engine: &mut CombatEngine,
        event: GameEvent,
    ) {
        engine.event_log.push(GameEventRecord::emitted(event));
        for idx in 0..self.instances.len() {
            let def_id = self.instances[idx].def.id;
            let matches = match event.kind {
                Trigger::OnCardPlayedPost => def_id == "echo_form",
                Trigger::OnAttackPlayed => def_id == "double_tap",
                Trigger::OnSkillPlayed => def_id == "burst",
                _ => false,
            };
            if !matches {
                continue;
            }
            self.execute_instance_hook_only(engine, idx, &event);
            if engine.state.combat_over {
                break;
            }
        }
        self.persisted_states = self.export_persisted_states();
    }

    fn alloc_instance_id(&mut self) -> u32 {
        let next = self.next_instance_id;
        self.next_instance_id += 1;
        next
    }

    fn rebuild_dispatch(&mut self) {
        self.dispatch.clear();
        for (idx, instance) in self.instances.iter().enumerate() {
            for trigger in instance.def.triggers {
                self.dispatch.add(trigger.trigger, idx);
            }
        }
    }

    fn execute_instance(&mut self, engine: &mut CombatEngine, instance_idx: usize, event: &GameEvent) {
        let owner = self.instances[instance_idx].owner;
        let def = self.instances[instance_idx].def;

        if !owner_is_active(engine, owner, def) {
            return;
        }
        if !owner_matches_event(owner, event) {
            return;
        }

        let mut hook_should_fire = false;
        for trigger in def.triggers {
            if trigger.trigger != event.kind {
                continue;
            }
            if !self.check_condition(engine, instance_idx, &trigger.condition, owner, event) {
                continue;
            }
            if let Some((counter_sid, threshold)) = trigger.counter {
                let next = self.read_status(engine, instance_idx, owner, counter_sid) + 1;
                if next >= threshold {
                    self.write_status(engine, instance_idx, owner, counter_sid, 0);
                } else {
                    self.write_status(engine, instance_idx, owner, counter_sid, next);
                    continue;
                }
            }
            hook_should_fire = true;
            engine
                .event_log
                .push(GameEventRecord::handled(
                    event,
                    owner,
                    def.id,
                    EffectExecutionPhase::Declarative,
                ));
            self.execute_effects(engine, instance_idx, owner, trigger.effects, event);
            if engine.state.combat_over {
                return;
            }
        }

        if let Some(hook) = def.complex_hook {
            if hook_should_fire {
                engine
                    .event_log
                    .push(GameEventRecord::handled(
                        event,
                        owner,
                        def.id,
                        EffectExecutionPhase::Hook,
                    ));
                let state = &mut self.instances[instance_idx].state;
                hook(engine, owner, event, state);
            }
        }
    }

    fn execute_instance_hook_only(
        &mut self,
        engine: &mut CombatEngine,
        instance_idx: usize,
        event: &GameEvent,
    ) {
        let owner = self.instances[instance_idx].owner;
        let def = self.instances[instance_idx].def;

        if !owner_is_active(engine, owner, def) {
            return;
        }
        if !owner_matches_event(owner, event) {
            return;
        }

        let mut hook_should_fire = def.triggers.is_empty();
        for trigger in def.triggers {
            if trigger.trigger != event.kind {
                continue;
            }
            if !self.check_condition(engine, instance_idx, &trigger.condition, owner, event) {
                continue;
            }
            if let Some((counter_sid, threshold)) = trigger.counter {
                let next = self.read_status(engine, instance_idx, owner, counter_sid) + 1;
                if next >= threshold {
                    self.write_status(engine, instance_idx, owner, counter_sid, 0);
                } else {
                    self.write_status(engine, instance_idx, owner, counter_sid, next);
                    continue;
                }
            }
            hook_should_fire = true;
            break;
        }

        if let Some(hook) = def.complex_hook {
            if hook_should_fire {
                engine.event_log.push(GameEventRecord::handled(
                    event,
                    owner,
                    def.id,
                    EffectExecutionPhase::Hook,
                ));
                let state = &mut self.instances[instance_idx].state;
                hook(engine, owner, event, state);
            }
        }
    }

    fn check_condition(
        &self,
        engine: &CombatEngine,
        instance_idx: usize,
        cond: &TriggerCondition,
        owner: EffectOwner,
        event: &GameEvent,
    ) -> bool {
        match cond {
            TriggerCondition::Always => true,
            TriggerCondition::FirstTurn => event.is_first_turn,
            TriggerCondition::NotFirstTurn => !event.is_first_turn,
            TriggerCondition::NoBlock => engine.state.player.block == 0,
            TriggerCondition::CounterReached => true,
            TriggerCondition::InStance(stance) => engine.state.stance == *stance,
            TriggerCondition::HasStatus(status_id) => {
                self.read_status(engine, instance_idx, owner, *status_id) > 0
            }
            TriggerCondition::HpBelow(pct) => {
                let threshold = (engine.state.player.max_hp * (*pct as i32)) / 100;
                engine.state.player.hp <= threshold
            }
            TriggerCondition::HandEmpty => engine.state.hand.is_empty(),
            TriggerCondition::CardTypeIs(card_type) => event.card_type == Some(*card_type),
            TriggerCondition::IsBossFight => is_boss_fight(engine),
            TriggerCondition::IsEliteFight => is_elite_fight(engine),
            TriggerCondition::IsEliteOrBossFight => is_elite_fight(engine) || is_boss_fight(engine),
        }
    }

    fn execute_effects(
        &mut self,
        engine: &mut CombatEngine,
        instance_idx: usize,
        owner: EffectOwner,
        effects: &[Effect],
        event: &GameEvent,
    ) {
        for effect in effects {
            match effect {
                Effect::Simple(simple) => {
                    self.execute_simple(engine, instance_idx, owner, *simple, event);
                }
                Effect::Conditional(cond, then_effects, else_effects) => {
                    let branch = if self.evaluate_effect_condition(engine, cond, event) {
                        then_effects
                    } else {
                        else_effects
                    };
                    self.execute_effects(engine, instance_idx, owner, branch, event);
                }
                _ => {}
            }
            if engine.state.combat_over {
                return;
            }
        }
    }

    fn execute_simple(
        &mut self,
        engine: &mut CombatEngine,
        instance_idx: usize,
        owner: EffectOwner,
        simple: SimpleEffect,
        event: &GameEvent,
    ) {
        match simple {
            SimpleEffect::AddStatus(target, status_id, amount_src) => {
                let amount = self.resolve_amount(engine, instance_idx, owner, amount_src);
                self.add_status(engine, instance_idx, owner, target, status_id, amount, event);
            }
            SimpleEffect::SetStatus(target, status_id, amount_src) => {
                let amount = self.resolve_amount(engine, instance_idx, owner, amount_src);
                self.set_status_for_target(
                    engine,
                    instance_idx,
                    owner,
                    target,
                    status_id,
                    amount,
                    event,
                );
            }
            SimpleEffect::MultiplyStatus(target, status_id, multiplier) => {
                let current =
                    self.read_target_status(engine, instance_idx, owner, target, status_id, event);
                if current > 0 {
                    self.set_status_for_target(
                        engine,
                        instance_idx,
                        owner,
                        target,
                        status_id,
                        current * multiplier,
                        event,
                    );
                }
            }
            SimpleEffect::DrawCards(amount_src) => {
                let count = self.resolve_amount(engine, instance_idx, owner, amount_src);
                if count > 0 {
                    engine.draw_cards(count);
                }
            }
            SimpleEffect::DrawToHandSize(amount_src) => {
                let target = self.resolve_amount(engine, instance_idx, owner, amount_src);
                let to_draw = (target - engine.state.hand.len() as i32).max(0);
                if to_draw > 0 {
                    engine.draw_cards(to_draw);
                }
            }
            SimpleEffect::ModifyPlayedCardCost(amount_src) => {
                let delta = self.resolve_amount(engine, instance_idx, owner, amount_src);
                if let Some(mut card) = engine.runtime_played_card {
                    let current = if card.cost >= 0 {
                        card.cost as i32
                    } else {
                        engine.card_registry.card_def_by_id(card.def_id).cost
                    };
                    let next = (current + delta).max(0) as i8;
                    card.cost = next;
                    engine.runtime_played_card = Some(card);
                }
            }
            SimpleEffect::ModifyPlayedCardBlock(amount_src) => {
                let delta = self.resolve_amount(engine, instance_idx, owner, amount_src);
                if let Some(mut card) = engine.runtime_played_card {
                    let current = if card.misc >= 0 {
                        card.misc as i32
                    } else {
                        engine.card_registry
                            .card_def_by_id(card.def_id)
                            .base_block
                            .max(0)
                    };
                    let next = (current + delta).max(0) as i16;
                    card.misc = next;
                    engine.runtime_played_card = Some(card);
                }
            }
            SimpleEffect::ModifyPlayedCardDamage(amount_src) => {
                let delta = self.resolve_amount(engine, instance_idx, owner, amount_src);
                if let Some(mut card) = engine.runtime_played_card {
                    let current = if card.misc >= 0 {
                        card.misc as i32
                    } else {
                        engine.card_registry.card_def_by_id(card.def_id).base_damage
                    };
                    let next = (current + delta).max(0) as i16;
                    card.misc = next;
                    engine.runtime_played_card = Some(card);
                }
            }
            SimpleEffect::GainEnergy(amount_src) => {
                let amount = self.resolve_amount(engine, instance_idx, owner, amount_src);
                engine.state.energy += amount;
            }
            SimpleEffect::DoubleEnergy => {
                engine.state.energy *= 2;
            }
            SimpleEffect::GainBlock(amount_src) => {
                let base = self.resolve_amount(engine, instance_idx, owner, amount_src);
                if base <= 0 {
                    return;
                }
                match owner {
                    EffectOwner::EnemyPower { enemy_idx } => {
                        let idx = enemy_idx as usize;
                        if idx < engine.state.enemies.len() && engine.state.enemies[idx].is_alive() {
                            engine.state.enemies[idx].entity.block += base;
                        }
                    }
                    EffectOwner::PotionSlot { .. } => {
                        engine.state.player.block += base;
                    }
                    _ => {
                        let dex = engine.state.player.dexterity();
                        let frail = engine.state.player.is_frail();
                        let block = crate::damage::calculate_block(base, dex, frail);
                        engine.gain_block_player(block);
                    }
                }
            }
            SimpleEffect::ModifyHp(amount_src) => {
                let amount = self.resolve_amount(engine, instance_idx, owner, amount_src);
                if amount > 0 {
                    engine.heal_player(amount);
                } else if amount < 0 {
                    engine.player_lose_hp(-amount);
                }
            }
            SimpleEffect::RemoveEnemyBlock(target) => {
                match target {
                    Target::SelectedEnemy => {
                        if event.target_idx >= 0
                            && (event.target_idx as usize) < engine.state.enemies.len()
                        {
                            engine.state.enemies[event.target_idx as usize].entity.block = 0;
                        }
                    }
                    Target::AllEnemies => {
                        for idx in engine.state.living_enemy_indices() {
                            engine.state.enemies[idx].entity.block = 0;
                        }
                    }
                    Target::RandomEnemy => {
                        let living = engine.state.living_enemy_indices();
                        if !living.is_empty() {
                            let idx = living[engine.rng_gen_range(0..living.len())];
                            engine.state.enemies[idx].entity.block = 0;
                        }
                    }
                    Target::Player | Target::SelfEntity => {}
                }
            }
            SimpleEffect::GainMantra(amount_src) => {
                let amount = self.resolve_amount(engine, instance_idx, owner, amount_src);
                engine.gain_mantra(amount);
            }
            SimpleEffect::Scry(amount_src) => {
                let amount = self.resolve_amount(engine, instance_idx, owner, amount_src);
                engine.do_scry(amount);
            }
            SimpleEffect::AddCard(card_name, pile, amount_src) => {
                let count = self.resolve_amount(engine, instance_idx, owner, amount_src).max(0);
                if pile == Pile::Hand {
                    engine.add_temp_cards_to_hand(card_name, count);
                } else {
                    for _ in 0..count {
                        let card = engine.temp_card(card_name);
                        push_to_pile(engine, pile, card);
                    }
                }
                if pile == Pile::Draw && count > 0 {
                    engine.shuffle_draw_pile();
                }
            }
            SimpleEffect::AddCardWithMisc(card_name, pile, amount_src, misc_src) => {
                let count = self.resolve_amount(engine, instance_idx, owner, amount_src).max(0);
                let misc = self.resolve_amount(engine, instance_idx, owner, misc_src).max(0) as i16;
                if pile == Pile::Hand {
                    for _ in 0..count {
                        let mut card = engine.temp_card(card_name);
                        card.misc = misc;
                        if engine.state.hand.len() < 10 {
                            engine.state.hand.push(card);
                        } else {
                            engine.state.discard_pile.push(card);
                        }
                    }
                } else {
                    for _ in 0..count {
                        let mut card = engine.temp_card(card_name);
                        card.misc = misc;
                        push_to_pile(engine, pile, card);
                    }
                }
                if pile == Pile::Draw && count > 0 {
                    engine.shuffle_draw_pile();
                }
            }
            SimpleEffect::CopyThisCardTo(_pile) => {}
            SimpleEffect::ChannelOrb(orb_type, amount_src) => {
                let count = self.resolve_amount(engine, instance_idx, owner, amount_src).max(0);
                for _ in 0..count {
                    engine.channel_orb(orb_type);
                }
            }
            SimpleEffect::ChannelRandomOrb(amount_src) => {
                let count = self.resolve_amount(engine, instance_idx, owner, amount_src).max(0);
                let orb_types = [
                    crate::orbs::OrbType::Lightning,
                    crate::orbs::OrbType::Frost,
                    crate::orbs::OrbType::Dark,
                    crate::orbs::OrbType::Plasma,
                ];
                for _ in 0..count {
                    let idx = engine.rng_gen_range(0..orb_types.len());
                    engine.channel_orb(orb_types[idx]);
                }
            }
            SimpleEffect::EvokeOrb(amount_src) => {
                let count = self.resolve_amount(engine, instance_idx, owner, amount_src).max(0);
                if count > 0 {
                    engine.evoke_front_orb_n(count as usize);
                }
            }
            SimpleEffect::ChangeStance(stance) => {
                engine.change_stance(stance);
            }
            SimpleEffect::SetFlag(flag) => {
                match flag {
                    crate::effects::declarative::BoolFlag::NoDraw => {
                        engine.state.player.set_status(sid::NO_DRAW, 1);
                    }
                    crate::effects::declarative::BoolFlag::RetainHand => {
                        engine.state.player.set_status(sid::RETAIN_CARDS, 1);
                    }
                    crate::effects::declarative::BoolFlag::SkipEnemyTurn => {
                        engine.state.skip_enemy_turn = true;
                    }
                    crate::effects::declarative::BoolFlag::NextAttackFree => {
                        engine.state.player.set_status(sid::NEXT_ATTACK_FREE, 1);
                    }
                    crate::effects::declarative::BoolFlag::Blasphemy => {
                        engine.state.blasphemy_active = true;
                    }
                    crate::effects::declarative::BoolFlag::BulletTime => {
                        engine.state.player.set_status(sid::NO_DRAW, 1);
                        engine.state.player.set_status(sid::BULLET_TIME, 1);
                    }
                }
            }
            SimpleEffect::ShuffleDiscardIntoDraw => {
                let mut cards = std::mem::take(&mut engine.state.discard_pile);
                engine.state.draw_pile.append(&mut cards);
                engine.shuffle_draw_pile();
            }
            SimpleEffect::DealDamage(target, amount_src) => {
                let amount = self.resolve_amount(engine, instance_idx, owner, amount_src);
                if amount <= 0 {
                    return;
                }
                self.deal_damage(engine, owner, target, amount, event);
            }
            SimpleEffect::Judgement(amount_src) => {
                let threshold = self.resolve_amount(engine, instance_idx, owner, amount_src);
                let idx = event.target_idx;
                if idx >= 0 && (idx as usize) < engine.state.enemies.len() {
                    let tidx = idx as usize;
                    if engine.state.enemies[tidx].entity.hp <= threshold
                        && engine.state.enemies[tidx].is_alive()
                    {
                        let lethal =
                            engine.state.enemies[tidx].entity.hp + engine.state.enemies[tidx].entity.block;
                        engine.deal_damage_to_enemy(tidx, lethal);
                    }
                }
            }
            SimpleEffect::TriggerMarks => {
                let living = engine.state.living_enemy_indices();
                let mut any_killed = false;
                let mut total_mark_damage = 0;
                for idx in living {
                    let mark = engine.state.enemies[idx].entity.status(sid::MARK);
                    if mark > 0 {
                        engine.state.enemies[idx].entity.hp -= mark;
                        engine.state.total_damage_dealt += mark;
                        total_mark_damage += mark;
                        if engine.state.enemies[idx].entity.hp <= 0 {
                            engine.state.enemies[idx].entity.hp = 0;
                            any_killed = true;
                        }
                        engine.record_enemy_hp_damage(idx, mark);
                    }
                }
                engine.runtime_card_total_unblocked_damage += total_mark_damage;
                if any_killed {
                    engine.runtime_card_enemy_killed = true;
                }
            }
            SimpleEffect::HealHp(target, amount_src) => {
                let amount = self.resolve_amount(engine, instance_idx, owner, amount_src);
                if amount <= 0 {
                    return;
                }
                self.heal_hp(engine, owner, target, amount, event);
            }
            SimpleEffect::IncrementCounter(status_id, _threshold) => {
                let next = self.read_status(engine, instance_idx, owner, status_id) + 1;
                self.write_status(engine, instance_idx, owner, status_id, next);
            }
            SimpleEffect::ModifyMaxHp(amount_src) => {
                let amount = self.resolve_amount(engine, instance_idx, owner, amount_src);
                engine.state.player.max_hp = (engine.state.player.max_hp + amount).max(1);
                if engine.state.player.hp > engine.state.player.max_hp {
                    engine.state.player.hp = engine.state.player.max_hp;
                }
            }
            SimpleEffect::ModifyMaxEnergy(amount_src) => {
                let amount = self.resolve_amount(engine, instance_idx, owner, amount_src);
                engine.state.max_energy = (engine.state.max_energy + amount).max(0);
                engine.state.energy = engine.state.energy.min(engine.state.max_energy);
            }
            SimpleEffect::PlayTopCardOfDraw => {}
            SimpleEffect::ModifyGold(_amount_src) => {}
            SimpleEffect::FleeCombat => {
                engine.state.combat_over = true;
            }
        }
    }

    fn evaluate_effect_condition(
        &self,
        engine: &CombatEngine,
        cond: &crate::effects::declarative::Condition,
        event: &GameEvent,
    ) -> bool {
        match *cond {
            crate::effects::declarative::Condition::InStance(stance) => engine.state.stance == stance,
            crate::effects::declarative::Condition::EnemyAttacking => {
                let idx = event.target_idx;
                idx >= 0
                    && (idx as usize) < engine.state.enemies.len()
                    && engine.state.enemies[idx as usize].is_attacking()
            }
            crate::effects::declarative::Condition::EnemyHasStatus(status_id) => {
                let idx = event.target_idx;
                idx >= 0
                    && (idx as usize) < engine.state.enemies.len()
                    && engine.state.enemies[idx as usize].entity.status(status_id) > 0
            }
            crate::effects::declarative::Condition::LastCardType(card_type) => {
                engine.state.last_card_type == Some(card_type)
            }
            crate::effects::declarative::Condition::PlayerHasStatus(status_id) => {
                engine.state.player.status(status_id) > 0
            }
            crate::effects::declarative::Condition::NoBlock => engine.state.player.block == 0,
            crate::effects::declarative::Condition::EnemyKilled => engine.runtime_card_enemy_killed,
            crate::effects::declarative::Condition::DiscardedThisTurn => {
                engine.state.player.status(crate::status_ids::sid::DISCARDED_THIS_TURN) > 0
            }
        }
    }

    fn resolve_amount(
        &self,
        engine: &CombatEngine,
        instance_idx: usize,
        owner: EffectOwner,
        amount_src: AmountSource,
    ) -> i32 {
        match amount_src {
            AmountSource::Magic => 0,
            AmountSource::Block => 0,
            AmountSource::Damage => 0,
            AmountSource::Fixed(value) => value,
            AmountSource::XCost => 0,
            AmountSource::XCostPlus(bonus) => bonus,
            AmountSource::MagicPlusX => 0,
            AmountSource::LivingEnemyCount => engine.state.living_enemy_indices().len() as i32,
            AmountSource::OrbCount => engine.state.orb_slots.occupied_count() as i32,
            AmountSource::UniqueOrbCount => {
                let mut has_lightning = false;
                let mut has_frost = false;
                let mut has_dark = false;
                let mut has_plasma = false;
                for orb in &engine.state.orb_slots.slots {
                    match orb.orb_type {
                        crate::orbs::OrbType::Lightning => has_lightning = true,
                        crate::orbs::OrbType::Frost => has_frost = true,
                        crate::orbs::OrbType::Dark => has_dark = true,
                        crate::orbs::OrbType::Plasma => has_plasma = true,
                        crate::orbs::OrbType::Empty => {}
                    }
                }
                (has_lightning as i32)
                    + (has_frost as i32)
                    + (has_dark as i32)
                    + (has_plasma as i32)
            }
            AmountSource::HandSize => engine.state.hand.len() as i32,
            AmountSource::PlayerBlock => engine.state.player.block,
            AmountSource::DiscardPileSize => engine.state.discard_pile.len() as i32,
            AmountSource::CardMisc => 0,
            AmountSource::StatusValue(status_id) => {
                self.read_status(engine, instance_idx, owner, status_id)
            }
            AmountSource::PercentMaxHp(pct) => (engine.state.player.max_hp * pct) / 100,
            AmountSource::DrawPileDivN(n) => {
                if n > 0 {
                    engine.state.draw_pile.len() as i32 / n
                } else {
                    0
                }
            }
            // Runtime entity handlers do not execute in the card-play pipeline, so these
            // card-specific sources are intentionally inert on the owner-aware effect runtime.
            AmountSource::HandSizeAtPlay => 0,
            AmountSource::HandSizeAtPlayPlus(bonus) => bonus,
            AmountSource::DrawPileSize => engine.state.draw_pile.len() as i32,
            AmountSource::AttacksThisTurn => engine.state.attacks_played_this_turn,
            AmountSource::SkillsInHand => engine
                .state
                .hand
                .iter()
                .filter(|card| engine.card_registry.card_def_by_id(card.def_id).card_type == CardType::Skill)
                .count() as i32,
            AmountSource::PotionPotency => match owner {
                EffectOwner::PotionSlot { slot } => {
                    let idx = slot as usize;
                    if idx < engine.state.potions.len() {
                        crate::potions::effective_potency_runtime(
                            &engine.state,
                            &engine.state.potions[idx],
                        )
                    } else {
                        0
                    }
                }
                _ => 1,
            },
            AmountSource::TotalUnblockedDamage => engine.runtime_card_total_unblocked_damage.max(0),
        }
    }

    fn read_status(
        &self,
        engine: &CombatEngine,
        instance_idx: usize,
        owner: EffectOwner,
        status_id: StatusId,
    ) -> i32 {
        let def_id = self.instances[instance_idx].def.id;
        if let Some(slot) = hidden_status_slot(def_id, status_id) {
            return self.instances[instance_idx].state.get(slot);
        }
        match owner {
            EffectOwner::EnemyPower { enemy_idx } => {
                let idx = enemy_idx as usize;
                if idx < engine.state.enemies.len() {
                    engine.state.enemies[idx].entity.status(status_id)
                } else {
                    0
                }
            }
            _ => engine.state.player.status(status_id),
        }
    }

    fn write_status(
        &mut self,
        engine: &mut CombatEngine,
        instance_idx: usize,
        owner: EffectOwner,
        status_id: StatusId,
        value: i32,
    ) {
        let def_id = self.instances[instance_idx].def.id;
        if let Some(slot) = hidden_status_slot(def_id, status_id) {
            self.instances[instance_idx].state.set(slot, value);
            return;
        }
        match owner {
            EffectOwner::EnemyPower { enemy_idx } => {
                let idx = enemy_idx as usize;
                if idx < engine.state.enemies.len() {
                    engine.state.enemies[idx].entity.set_status(status_id, value);
                }
            }
            _ => engine.state.player.set_status(status_id, value),
        }
    }

    fn add_status(
        &mut self,
        engine: &mut CombatEngine,
        instance_idx: usize,
        owner: EffectOwner,
        target: Target,
        status_id: StatusId,
        amount: i32,
        event: &GameEvent,
    ) {
        match target {
            Target::Player => {
                if is_hidden_status_for_def(self.instances[instance_idx].def.id, status_id) {
                    let current = self.instances[instance_idx]
                        .state
                        .get(hidden_status_slot(self.instances[instance_idx].def.id, status_id).unwrap());
                    self.write_status(engine, instance_idx, owner, status_id, current + amount);
                } else {
                    engine.state.player.add_status(status_id, amount);
                }
            }
            Target::SelfEntity => match owner {
                EffectOwner::EnemyPower { enemy_idx } => {
                    let idx = enemy_idx as usize;
                    if idx < engine.state.enemies.len() {
                        engine.state.enemies[idx].entity.add_status(status_id, amount);
                    }
                }
                _ => engine.state.player.add_status(status_id, amount),
            },
            Target::SelectedEnemy => {
                let idx = event.target_idx.max(0) as usize;
                if idx < engine.state.enemies.len() {
                    if is_debuff(status_id) {
                        match owner {
                            EffectOwner::EnemyPower { .. } => {
                                crate::powers::apply_debuff(
                                    &mut engine.state.enemies[idx].entity,
                                    status_id,
                                    amount,
                                );
                            }
                            _ => {
                                engine.apply_player_debuff_to_enemy(idx, status_id, amount);
                            }
                        }
                    } else {
                        engine.state.enemies[idx].entity.add_status(status_id, amount);
                    }
                }
            }
            Target::AllEnemies => {
                let living = engine.state.living_enemy_indices();
                for idx in living {
                    if is_debuff(status_id) {
                        match owner {
                            EffectOwner::EnemyPower { .. } => {
                                crate::powers::apply_debuff(
                                    &mut engine.state.enemies[idx].entity,
                                    status_id,
                                    amount,
                                );
                            }
                            _ => {
                                engine.apply_player_debuff_to_enemy(idx, status_id, amount);
                            }
                        }
                    } else {
                        engine.state.enemies[idx].entity.add_status(status_id, amount);
                    }
                }
            }
            Target::RandomEnemy => {
                let living = engine.state.living_enemy_indices();
                if !living.is_empty() {
                    let idx = living[engine.rng_gen_range(0..living.len())];
                    if is_debuff(status_id) {
                        match owner {
                            EffectOwner::EnemyPower { .. } => {
                                crate::powers::apply_debuff(
                                    &mut engine.state.enemies[idx].entity,
                                    status_id,
                                    amount,
                                );
                            }
                            _ => {
                                engine.apply_player_debuff_to_enemy(idx, status_id, amount);
                            }
                        }
                    } else {
                        engine.state.enemies[idx].entity.add_status(status_id, amount);
                    }
                }
            }
        }
    }

    fn set_status_for_target(
        &mut self,
        engine: &mut CombatEngine,
        instance_idx: usize,
        owner: EffectOwner,
        target: Target,
        status_id: StatusId,
        value: i32,
        event: &GameEvent,
    ) {
        match target {
            Target::Player => self.write_status(engine, instance_idx, owner, status_id, value),
            Target::SelfEntity => match owner {
                EffectOwner::EnemyPower { enemy_idx } => {
                    let idx = enemy_idx as usize;
                    if idx < engine.state.enemies.len() {
                        engine.state.enemies[idx].entity.set_status(status_id, value);
                    }
                }
                _ => engine.state.player.set_status(status_id, value),
            },
            Target::SelectedEnemy => {
                let idx = event.target_idx.max(0) as usize;
                if idx < engine.state.enemies.len() {
                    engine.state.enemies[idx].entity.set_status(status_id, value);
                }
            }
            Target::AllEnemies => {
                let living = engine.state.living_enemy_indices();
                for idx in living {
                    engine.state.enemies[idx].entity.set_status(status_id, value);
                }
            }
            Target::RandomEnemy => {
                let living = engine.state.living_enemy_indices();
                if !living.is_empty() {
                    let idx = living[engine.rng_gen_range(0..living.len())];
                    engine.state.enemies[idx].entity.set_status(status_id, value);
                }
            }
        }
    }

    fn read_target_status(
        &self,
        engine: &CombatEngine,
        instance_idx: usize,
        owner: EffectOwner,
        target: Target,
        status_id: StatusId,
        event: &GameEvent,
    ) -> i32 {
        match target {
            Target::Player => self.read_status(engine, instance_idx, owner, status_id),
            Target::SelfEntity => match owner {
                EffectOwner::EnemyPower { enemy_idx } => {
                    let idx = enemy_idx as usize;
                    if idx < engine.state.enemies.len() {
                        engine.state.enemies[idx].entity.status(status_id)
                    } else {
                        0
                    }
                }
                _ => engine.state.player.status(status_id),
            },
            Target::SelectedEnemy => {
                let idx = event.target_idx.max(0) as usize;
                if idx < engine.state.enemies.len() {
                    engine.state.enemies[idx].entity.status(status_id)
                } else {
                    0
                }
            }
            Target::AllEnemies | Target::RandomEnemy => 0,
        }
    }

    fn deal_damage(
        &mut self,
        engine: &mut CombatEngine,
        owner: EffectOwner,
        target: Target,
        amount: i32,
        event: &GameEvent,
    ) {
        match target {
            Target::Player => engine.deal_damage_to_player(amount),
            Target::SelfEntity => {
                if let EffectOwner::EnemyPower { enemy_idx } = owner {
                    let idx = enemy_idx as usize;
                    if idx < engine.state.enemies.len() && engine.state.enemies[idx].is_alive() {
                        engine.deal_damage_to_enemy(idx, amount);
                    }
                }
            }
            Target::SelectedEnemy => {
                if event.target_idx >= 0 {
                    let idx = event.target_idx as usize;
                    if idx < engine.state.enemies.len() {
                        engine.deal_damage_to_enemy(idx, amount);
                    }
                }
            }
            Target::AllEnemies => {
                let living = engine.state.living_enemy_indices();
                for idx in living {
                    engine.deal_damage_to_enemy(idx, amount);
                }
            }
            Target::RandomEnemy => {
                let living = engine.state.living_enemy_indices();
                if !living.is_empty() {
                    let idx = living[engine.rng_gen_range(0..living.len())];
                    engine.deal_damage_to_enemy(idx, amount);
                }
            }
        }
    }

    fn heal_hp(
        &mut self,
        engine: &mut CombatEngine,
        owner: EffectOwner,
        target: Target,
        amount: i32,
        event: &GameEvent,
    ) {
        match target {
            Target::Player => engine.heal_player(amount),
            Target::SelfEntity => {
                if let EffectOwner::EnemyPower { enemy_idx } = owner {
                    let idx = enemy_idx as usize;
                    if idx < engine.state.enemies.len() {
                        let enemy = &mut engine.state.enemies[idx].entity;
                        enemy.hp = (enemy.hp + amount).min(enemy.max_hp);
                    }
                } else {
                    engine.heal_player(amount);
                }
            }
            Target::SelectedEnemy => {
                if event.target_idx >= 0 {
                    let idx = event.target_idx as usize;
                    if idx < engine.state.enemies.len() {
                        let enemy = &mut engine.state.enemies[idx].entity;
                        enemy.hp = (enemy.hp + amount).min(enemy.max_hp);
                    }
                }
            }
            Target::AllEnemies => {
                let living = engine.state.living_enemy_indices();
                for idx in living {
                    let enemy = &mut engine.state.enemies[idx].entity;
                    enemy.hp = (enemy.hp + amount).min(enemy.max_hp);
                }
            }
            Target::RandomEnemy => {
                let living = engine.state.living_enemy_indices();
                if !living.is_empty() {
                    let idx = living[engine.rng_gen_range(0..living.len())];
                    let enemy = &mut engine.state.enemies[idx].entity;
                    enemy.hp = (enemy.hp + amount).min(enemy.max_hp);
                }
            }
        }
    }
}

fn take_or_default_state(
    old_instances: &mut Vec<EntityInstance>,
    owner: EffectOwner,
    def: &'static EntityDef,
    persisted_states: &[PersistedEffectState],
) -> EffectState {
    if let Some(pos) = old_instances
        .iter()
        .position(|instance| instance.owner == owner && instance.def.id == def.id)
    {
        return old_instances.swap_remove(pos).state;
    }
    if default_persistence_for(def.id) == PersistenceScope::Run {
        if let Some(saved) = persisted_states.iter().find(|state| state.def_id == def.id) {
            return EffectState::from_values(PersistenceScope::Run, saved.values.clone());
        }
    }
    EffectState::new(default_persistence_for(def.id))
}

fn seed_hidden_relic_state(
    state: &crate::state::CombatState,
    def: &'static EntityDef,
    effect_state: &mut EffectState,
) {
    match def.id {
        "Du-Vu Doll" if effect_state.get(0) == 0 => {
            effect_state.set(0, state.player.status(sid::DU_VU_DOLL_CURSES));
        }
        "Girya" if effect_state.get(0) == 0 => {
            effect_state.set(0, state.player.status(sid::GIRYA_COUNTER));
        }
        _ => {}
    }
}

fn default_persistence_for(def_id: &str) -> PersistenceScope {
    match def_id {
        "Nunchaku" | "InkBottle" | "Happy Flower" | "Incense Burner" | "Sundial" => PersistenceScope::Run,
        _ => PersistenceScope::Combat,
    }
}

fn owner_is_active(engine: &CombatEngine, owner: EffectOwner, def: &EntityDef) -> bool {
    match owner {
        EffectOwner::PlayerRelic { .. } => engine.state.has_relic(def.id),
        EffectOwner::PlayerPower => {
            if let Some(guard) = def.status_guard {
                engine.state.player.status(guard) > 0
            } else {
                true
            }
        }
        EffectOwner::EnemyPower { enemy_idx } => {
            let idx = enemy_idx as usize;
            if idx >= engine.state.enemies.len() || !engine.state.enemies[idx].is_alive() {
                return false;
            }
            if let Some(guard) = def.status_guard {
                engine.state.enemies[idx].entity.status(guard) > 0
            } else {
                enemy_power_is_active(def, &engine.state.enemies[idx])
            }
        }
        EffectOwner::PotionSlot { slot } => {
            let idx = slot as usize;
            idx < engine.state.potions.len() && !engine.state.potions[idx].is_empty()
        }
        EffectOwner::RunEffect => true,
    }
}

fn owner_matches_event(owner: EffectOwner, event: &GameEvent) -> bool {
    match owner {
        EffectOwner::PotionSlot { slot }
            if matches!(event.kind, Trigger::ManualActivation | Trigger::OnPotionUsed) =>
        {
            event.potion_slot < 0 || event.potion_slot == slot as i32
        }
        _ => true,
    }
}

fn enemy_power_is_active(def: &EntityDef, enemy: &crate::state::EnemyCombatState) -> bool {
    if let Some(guard) = def.status_guard {
        return enemy.entity.status(guard) > 0;
    }
    false
}

fn is_hidden_status_for_def(def_id: &str, status_id: StatusId) -> bool {
    hidden_status_slot(def_id, status_id).is_some()
}

fn hidden_status_slot(def_id: &str, status_id: StatusId) -> Option<usize> {
    match (def_id, status_id) {
        ("Ornamental Fan", sid::ORNAMENTAL_FAN_COUNTER) => Some(0),
        ("Kunai", sid::KUNAI_COUNTER) => Some(0),
        ("Shuriken", sid::SHURIKEN_COUNTER) => Some(0),
        ("Nunchaku", sid::NUNCHAKU_COUNTER) => Some(0),
        ("InkBottle", sid::INK_BOTTLE_COUNTER) => Some(0),
        ("Happy Flower", sid::HAPPY_FLOWER_COUNTER) => Some(0),
        ("Incense Burner", sid::INCENSE_BURNER_COUNTER) => Some(0),
        ("Sundial", sid::SUNDIAL_COUNTER) => Some(0),
        ("Letter Opener", sid::LETTER_OPENER_COUNTER) => Some(0),
        ("StoneCalendar", sid::STONE_CALENDAR_COUNTER) => Some(0),
        ("Inserter", sid::INSERTER_COUNTER) => Some(0),
        ("Velvet Choker", sid::VELVET_CHOKER_COUNTER) => Some(0),
        ("OrangePellets", sid::OP_ATTACK) => Some(0),
        ("OrangePellets", sid::OP_SKILL) => Some(1),
        ("OrangePellets", sid::OP_POWER) => Some(2),
        ("Pocketwatch", sid::POCKETWATCH_COUNTER) => Some(0),
        ("Pocketwatch", sid::POCKETWATCH_FIRST_TURN) => Some(1),
        ("panache", sid::PANACHE_COUNT) => Some(0),
        ("Du-Vu Doll", sid::DU_VU_DOLL_CURSES) => Some(0),
        ("Girya", sid::GIRYA_COUNTER) => Some(0),
        _ => None,
    }
}

fn is_boss_fight(engine: &CombatEngine) -> bool {
    engine.state.enemies.iter().any(|enemy| {
        matches!(
            enemy.id.as_str(),
            "Hexaghost"
                | "SlimeBoss"
                | "TheGuardian"
                | "BronzeAutomaton"
                | "TheCollector"
                | "TheChamp"
                | "AwakenedOne"
                | "TimeEater"
                | "Donu"
                | "Deca"
                | "TheHeart"
                | "CorruptHeart"
                | "SpireShield"
                | "SpireSpear"
        )
    })
}

fn is_elite_fight(engine: &CombatEngine) -> bool {
    if engine.state.player.status(sid::SLING_ELITE) > 0
        || engine.state.player.status(sid::PRESERVED_INSECT_ELITE) > 0
    {
        return true;
    }
    engine.state.enemies.iter().any(|enemy| {
        matches!(
            enemy.id.as_str(),
            "GremlinNob" | "Lagavulin" | "Sentry" | "BookOfStabbing" | "GremlinLeader"
                | "TaskMaster" | "Nemesis" | "Reptomancer" | "GiantHead"
        )
    })
}

fn push_to_pile(
    engine: &mut CombatEngine,
    pile: Pile,
    card: crate::combat_types::CardInstance,
) {
    match pile {
        Pile::Hand => {
            if engine.state.hand.len() < 10 {
                engine.state.hand.push(card);
            }
        }
        Pile::Draw => engine.state.draw_pile.push(card),
        Pile::Discard => engine.state.discard_pile.push(card),
        Pile::Exhaust => engine.state.exhaust_pile.push(card),
    }
}

fn is_debuff(status_id: StatusId) -> bool {
    status_id == sid::WEAKENED
        || status_id == sid::VULNERABLE
        || status_id == sid::FRAIL
        || status_id == sid::POISON
        || status_id == sid::CONSTRICTED
}
