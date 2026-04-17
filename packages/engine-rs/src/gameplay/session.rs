//! Canonical decision/session facade for universal gameplay work.
//!
//! This lifts both combat-only and full-run engines onto the same action,
//! context, and runtime-snapshot surface so migration waves can target one
//! coordinator API instead of reaching into engine-specific internals.

use crate::decision::{build_combat_context, DecisionAction, DecisionContext, DecisionKind};
use crate::engine::{CombatEngine, CombatPhase};
use crate::gameplay::runtime::{
    GameplayRuntimeEventRecord, GameplayRuntimeScope, GameplayRuntimeSnapshot, GameplayRuntimeSource,
};
use crate::obs::{encode_combat_state_v2_from_combat, COMBAT_OBS_VERSION};
use crate::run::RunEngine;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayDecisionState {
    pub scope: GameplayRuntimeScope,
    pub kind: DecisionKind,
    pub terminal: bool,
    pub phase_label: String,
    pub room_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GameplayStepResult {
    pub action_accepted: bool,
    pub reward_delta: f32,
    pub terminal: bool,
    pub decision_state: GameplayDecisionState,
    pub decision_context: DecisionContext,
    pub legal_actions: Vec<DecisionAction>,
    pub runtime: GameplayRuntimeSnapshot,
    pub event_trace: Vec<GameplayRuntimeEventRecord>,
    pub combat_obs_v2: Option<Vec<f32>>,
    pub combat_obs_version: Option<u32>,
}

pub trait GameplaySession: GameplayRuntimeSource {
    fn gameplay_decision_state(&self) -> GameplayDecisionState;
    fn gameplay_decision_context(&self) -> DecisionContext;
    fn gameplay_legal_actions(&self) -> Vec<DecisionAction>;
    fn gameplay_step(&mut self, action: &DecisionAction) -> GameplayStepResult;
}

impl GameplaySession for CombatEngine {
    fn gameplay_decision_state(&self) -> GameplayDecisionState {
        GameplayDecisionState {
            scope: GameplayRuntimeScope::Combat,
            kind: combat_decision_kind(self),
            terminal: self.is_combat_over(),
            phase_label: format!("Combat::{:?}", self.phase),
            room_type: None,
        }
    }

    fn gameplay_decision_context(&self) -> DecisionContext {
        DecisionContext {
            kind: combat_decision_kind(self),
            neow: None,
            combat: Some(build_combat_context(self)),
            reward_screen: None,
            map: None,
            event: None,
            shop: None,
            campfire: None,
        }
    }

    fn gameplay_legal_actions(&self) -> Vec<DecisionAction> {
        self.get_legal_actions()
            .into_iter()
            .map(DecisionAction::Combat)
            .collect()
    }

    fn gameplay_step(&mut self, action: &DecisionAction) -> GameplayStepResult {
        let legal_before = self.gameplay_legal_actions();
        let action_accepted = legal_before.contains(action);
        self.clear_event_log();

        if action_accepted {
            if let DecisionAction::Combat(combat_action) = action {
                self.execute_action(combat_action);
            }
        }

        let runtime = self.gameplay_runtime_snapshot();
        GameplayStepResult {
            action_accepted,
            reward_delta: 0.0,
            terminal: self.is_combat_over(),
            decision_state: self.gameplay_decision_state(),
            decision_context: self.gameplay_decision_context(),
            legal_actions: self.gameplay_legal_actions(),
            event_trace: runtime.recent_events.clone(),
            runtime,
            combat_obs_v2: Some(encode_combat_state_v2_from_combat(self).to_vec()),
            combat_obs_version: Some(COMBAT_OBS_VERSION),
        }
    }
}

impl GameplaySession for RunEngine {
    fn gameplay_decision_state(&self) -> GameplayDecisionState {
        let state = self.current_decision_state();
        GameplayDecisionState {
            scope: GameplayRuntimeScope::Run,
            kind: state.kind,
            terminal: state.terminal,
            phase_label: format!("Run::{:?}", self.current_phase()),
            room_type: Some(state.room_type),
        }
    }

    fn gameplay_decision_context(&self) -> DecisionContext {
        self.current_decision_context()
    }

    fn gameplay_legal_actions(&self) -> Vec<DecisionAction> {
        self.get_legal_decision_actions()
    }

    fn gameplay_step(&mut self, action: &DecisionAction) -> GameplayStepResult {
        let result = self.step_with_result(&action.to_run_action());
        let runtime = self.gameplay_runtime_snapshot();
        let combat_obs_v2 = result.combat_obs_v2;
        let combat_obs_version = combat_obs_v2
            .as_ref()
            .map(|_| result.combat_obs_version);
        GameplayStepResult {
            action_accepted: result.action_accepted,
            reward_delta: result.reward,
            terminal: result.done,
            decision_state: self.gameplay_decision_state(),
            decision_context: self.gameplay_decision_context(),
            legal_actions: self.gameplay_legal_actions(),
            event_trace: runtime.recent_events.clone(),
            runtime,
            combat_obs_v2,
            combat_obs_version,
        }
    }
}

fn combat_decision_kind(engine: &CombatEngine) -> DecisionKind {
    match engine.phase {
        CombatPhase::AwaitingChoice => DecisionKind::CombatChoice,
        CombatPhase::CombatOver => DecisionKind::GameOver,
        _ => DecisionKind::CombatAction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Action;
    use crate::tests::support::{engine_with, make_deck, run_engine};

    #[test]
    fn combat_session_exposes_unified_action_surface() {
        let engine = engine_with(make_deck(&["Strike_R"]), 20, 5);
        let legal = engine.gameplay_legal_actions();

        assert!(legal.iter().all(|action| matches!(action, DecisionAction::Combat(_))));
        assert!(legal.contains(&DecisionAction::Combat(Action::EndTurn)));
        assert_eq!(engine.gameplay_decision_state().scope, GameplayRuntimeScope::Combat);
    }

    #[test]
    fn run_session_exposes_reward_screen_actions() {
        let mut engine = run_engine(42, 20);
        engine.debug_set_card_reward_screen(vec!["Wallop".to_string(), "Scrawl".to_string()]);

        let legal = engine.gameplay_legal_actions();
        assert_eq!(engine.gameplay_decision_state().scope, GameplayRuntimeScope::Run);
        assert!(legal.contains(&DecisionAction::ClaimRewardItem { item_index: 0 }));
    }

    #[test]
    fn combat_session_step_returns_runtime_snapshot_and_obs() {
        let mut engine = engine_with(make_deck(&["Strike_R"]), 20, 5);
        let result = engine.gameplay_step(&DecisionAction::Combat(Action::EndTurn));

        assert!(result.action_accepted);
        assert_eq!(result.runtime.scope, GameplayRuntimeScope::Combat);
        assert!(result.combat_obs_v2.is_some());
        assert_eq!(result.combat_obs_version, Some(COMBAT_OBS_VERSION));
    }
}
