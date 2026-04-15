//! Canonical runtime snapshot API for universal gameplay work.
//!
//! This module gives every migration wave the same read surface across combat
//! effects, run-level reward screens, decision state, and recent event traces.
//! It does not replace execution yet; it establishes the shared interface that
//! worker slices can target while we retire engine-specific internals.

use std::collections::BTreeMap;

use crate::decision::{DecisionContext, DecisionState, RewardScreen};
use crate::effects::entity_def::EntityKind;
use crate::effects::runtime::{
    EffectOwner, EntityInstance, EventRecordPhase, GameEventRecord, PersistedEffectState,
    PersistenceScope,
};
use crate::engine::CombatEngine;
use crate::gameplay::registry::{global_registry, GameplayRegistry};
use crate::gameplay::types::{
    EffectOp, GameplayDef, GameplayDomain, GameplayEventKind, GameplayOwner,
    GameplayProgram, Lifetime, StateVisibility,
};
use crate::run::RunEngine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameplayRuntimeScope {
    Combat,
    Run,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayRuntimeValue {
    pub field_id: String,
    pub value: i16,
    pub visibility: StateVisibility,
    pub persistence: PersistenceScope,
    pub lifetime: Lifetime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayRuntimeInstance {
    pub instance_id: u32,
    pub runtime_order: usize,
    pub domain: GameplayDomain,
    pub def_id: String,
    pub owner: GameplayOwner,
    pub persistence: PersistenceScope,
    pub program: GameplayProgram,
    pub values: Vec<GameplayRuntimeValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayPersistedState {
    pub domain: Option<GameplayDomain>,
    pub def_id: String,
    pub values: Vec<GameplayRuntimeValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayRuntimeEventRecord {
    pub phase: EventRecordPhase,
    pub event: GameplayEventKind,
    pub owner: Option<GameplayOwner>,
    pub def_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayRuntimeSnapshot {
    pub scope: GameplayRuntimeScope,
    pub instances: Vec<GameplayRuntimeInstance>,
    pub persisted_states: Vec<GameplayPersistedState>,
    pub reward_screen: Option<RewardScreen>,
    pub decision_state: Option<DecisionState>,
    pub decision_context: Option<DecisionContext>,
    pub decision_program: Option<GameplayProgram>,
    pub recent_events: Vec<GameplayRuntimeEventRecord>,
}

pub trait GameplayRuntimeSource {
    fn gameplay_registry(&self) -> &'static GameplayRegistry {
        global_registry()
    }

    fn gameplay_runtime_snapshot(&self) -> GameplayRuntimeSnapshot;
}

impl GameplayRuntimeSource for CombatEngine {
    fn gameplay_runtime_snapshot(&self) -> GameplayRuntimeSnapshot {
        GameplayRuntimeSnapshot {
            scope: GameplayRuntimeScope::Combat,
            instances: runtime_instances_from_effects(self.effect_runtime.instances(), global_registry()),
            persisted_states: self
                .effect_runtime
                .persisted_states()
                .iter()
                .map(|state| persisted_state_from_effect(state, global_registry()))
                .collect(),
            reward_screen: None,
            decision_state: None,
            decision_context: None,
            decision_program: None,
            recent_events: self.event_log.iter().map(event_record_from_effect).collect(),
        }
    }
}

impl GameplayRuntimeSource for RunEngine {
    fn gameplay_runtime_snapshot(&self) -> GameplayRuntimeSnapshot {
        let mut instances = Vec::new();
        let mut persisted_states = self
            .run_state
            .persisted_effect_states
            .iter()
            .map(|state| persisted_state_from_effect(state, global_registry()))
            .collect::<Vec<_>>();
        let reward_screen = self.current_reward_screen();
        let decision_state = Some(self.current_decision_state());
        let decision_context = Some(self.current_decision_context());
        let mut recent_events = self
            .last_combat_events()
            .iter()
            .map(event_record_from_effect)
            .collect::<Vec<_>>();

        if let Some(combat) = self.get_combat_engine() {
            instances.extend(runtime_instances_from_effects(
                combat.effect_runtime.instances(),
                global_registry(),
            ));
            persisted_states = combat
                .effect_runtime
                .persisted_states()
                .iter()
                .map(|state| persisted_state_from_effect(state, global_registry()))
                .collect();
            recent_events = combat.event_log.iter().map(event_record_from_effect).collect();
        }

        let decision_program = decision_program_from_snapshot(
            decision_state.as_ref(),
            reward_screen.as_ref(),
            decision_context.as_ref(),
        );

        GameplayRuntimeSnapshot {
            scope: GameplayRuntimeScope::Run,
            instances,
            persisted_states,
            reward_screen,
            decision_state,
            decision_context,
            decision_program,
            recent_events,
        }
    }
}

fn runtime_instances_from_effects(
    instances: &[EntityInstance],
    registry: &GameplayRegistry,
) -> Vec<GameplayRuntimeInstance> {
    let mut player_power_install_order: u16 = 0;
    let mut enemy_power_install_orders: BTreeMap<u16, u16> = BTreeMap::new();

    instances
        .iter()
        .enumerate()
        .map(|(runtime_order, instance)| {
            let domain = domain_from_entity_kind(instance.def.kind);
            let owner = match instance.owner {
                EffectOwner::PlayerRelic { slot } => GameplayOwner::PlayerRelic { slot },
                EffectOwner::PlayerPower => {
                    let owner = GameplayOwner::PlayerPower {
                        install_order: player_power_install_order,
                    };
                    player_power_install_order += 1;
                    owner
                }
                EffectOwner::EnemyPower { enemy_idx } => {
                    let next = enemy_power_install_orders.entry(enemy_idx).or_insert(0);
                    let owner = GameplayOwner::EnemyPower {
                        enemy_idx,
                        install_order: *next,
                    };
                    *next += 1;
                    owner
                }
                EffectOwner::PotionSlot { slot } => GameplayOwner::PotionSlot { slot },
                EffectOwner::RunEffect => GameplayOwner::Run,
            };

            GameplayRuntimeInstance {
                instance_id: instance.instance_id,
                runtime_order,
                domain,
                def_id: instance.def.id.to_string(),
                owner,
                persistence: instance.state.persistence,
                program: registry
                    .get(domain, instance.def.id)
                    .map(|def| def.program())
                    .unwrap_or_else(|| GameplayProgram::adapted_legacy(vec![
                        EffectOp::DeclareDefinition {
                            domain,
                            id: instance.def.id.to_string(),
                            name: instance.def.name.to_string(),
                        },
                        EffectOp::LegacyAdapter {
                            label: instance.def.name.to_string(),
                            reason: "missing registry definition".to_string(),
                        },
                    ])),
                values: runtime_values(
                    registry.get(domain, instance.def.id),
                    instance.state.persistence,
                    &instance.state.as_vec(),
                ),
            }
        })
        .collect()
}

fn persisted_state_from_effect(
    state: &PersistedEffectState,
    registry: &GameplayRegistry,
) -> GameplayPersistedState {
    let domain = registry
        .defs()
        .iter()
        .find(|def| def.id == state.def_id)
        .map(|def| def.domain);
    GameplayPersistedState {
        domain,
        def_id: state.def_id.clone(),
        values: runtime_values(
            domain.and_then(|domain| registry.get(domain, &state.def_id)),
            PersistenceScope::Run,
            &state.values,
        ),
    }
}

fn event_record_from_effect(record: &GameEventRecord) -> GameplayRuntimeEventRecord {
    GameplayRuntimeEventRecord {
        phase: record.phase,
        event: GameplayEventKind::from(record.event),
        owner: record.owner.map(owner_from_effect_owner),
        def_id: record.def_id.map(|id| id.to_string()),
    }
}

fn decision_program_from_snapshot(
    decision_state: Option<&DecisionState>,
    reward_screen: Option<&RewardScreen>,
    decision_context: Option<&DecisionContext>,
) -> Option<GameplayProgram> {
    let decision_state = decision_state?;
    match decision_state.kind {
        crate::decision::DecisionKind::NeowChoice => decision_context
            .and_then(|context| context.neow.as_ref())
            .map(gameplay_program_for_neow_context),
        crate::decision::DecisionKind::RewardScreen => {
            reward_screen.map(gameplay_program_for_reward_screen)
        }
        crate::decision::DecisionKind::EventOption => decision_context
            .and_then(|context| context.event.as_ref())
            .map(gameplay_program_for_event_context),
        crate::decision::DecisionKind::ShopAction => decision_context
            .and_then(|context| context.shop.as_ref())
            .map(gameplay_program_for_shop_context),
        crate::decision::DecisionKind::CampfireAction => decision_context
            .and_then(|context| context.campfire.as_ref())
            .map(gameplay_program_for_campfire_context),
        crate::decision::DecisionKind::MapPath => decision_context
            .and_then(|context| context.map.as_ref())
            .map(gameplay_program_for_map_context),
        crate::decision::DecisionKind::CombatChoice => decision_context
            .and_then(|context| context.combat.as_ref())
            .map(|combat| gameplay_program_for_combat_choice(&combat.choice)),
        crate::decision::DecisionKind::CombatAction | crate::decision::DecisionKind::GameOver => {
            None
        }
    }
}

fn gameplay_program_for_reward_screen(screen: &RewardScreen) -> GameplayProgram {
    GameplayProgram::canonical(vec![EffectOp::RewardScreen {
        label: format!("{:?}", screen.source),
        source: format!("{:?}", screen.source),
        ordered: screen.ordered,
        active_item: screen.active_item,
        item_count: screen.items.len(),
    }])
}

fn gameplay_program_for_neow_context(context: &crate::decision::NeowDecisionContext) -> GameplayProgram {
    GameplayProgram::canonical(vec![EffectOp::OpenChoice {
        label: "neow".to_string(),
        option_count: context.options.len(),
    }])
}

fn gameplay_program_for_event_context(context: &crate::decision::EventDecisionContext) -> GameplayProgram {
    GameplayProgram::canonical(vec![EffectOp::OpenChoice {
        label: context.name.clone(),
        option_count: context.options.len(),
    }])
}

fn gameplay_program_for_shop_context(context: &crate::decision::ShopDecisionContext) -> GameplayProgram {
    GameplayProgram::canonical(vec![EffectOp::OpenChoice {
        label: "shop".to_string(),
        option_count: context.offers.len() + 2,
    }])
}

fn gameplay_program_for_campfire_context(
    context: &crate::decision::CampfireDecisionContext,
) -> GameplayProgram {
    GameplayProgram::canonical(vec![EffectOp::OpenChoice {
        label: "campfire".to_string(),
        option_count: context.upgradable_cards.len() + if context.can_rest { 1 } else { 0 },
    }])
}

fn gameplay_program_for_map_context(context: &crate::decision::MapDecisionContext) -> GameplayProgram {
    GameplayProgram::canonical(vec![EffectOp::OpenChoice {
        label: "map path".to_string(),
        option_count: context.available_paths,
    }])
}

fn gameplay_program_for_combat_choice(
    context: &crate::decision::CombatChoiceContext,
) -> GameplayProgram {
    GameplayProgram::canonical(vec![EffectOp::OpenChoice {
        label: context.reason.clone().unwrap_or_else(|| "combat choice".to_string()),
        option_count: context.option_count,
    }])
}

fn domain_from_entity_kind(kind: EntityKind) -> GameplayDomain {
    match kind {
        EntityKind::Relic => GameplayDomain::Relic,
        EntityKind::Power => GameplayDomain::Power,
        EntityKind::Potion => GameplayDomain::Potion,
    }
}

fn owner_from_effect_owner(owner: EffectOwner) -> GameplayOwner {
    match owner {
        EffectOwner::PlayerRelic { slot } => GameplayOwner::PlayerRelic { slot },
        EffectOwner::PlayerPower => GameplayOwner::PlayerPower { install_order: 0 },
        EffectOwner::EnemyPower { enemy_idx } => GameplayOwner::EnemyPower {
            enemy_idx,
            install_order: 0,
        },
        EffectOwner::PotionSlot { slot } => GameplayOwner::PotionSlot { slot },
        EffectOwner::RunEffect => GameplayOwner::Run,
    }
}

fn runtime_values(
    def: Option<&GameplayDef>,
    persistence: PersistenceScope,
    values: &[i16],
) -> Vec<GameplayRuntimeValue> {
    values
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            let field = def.and_then(|def| def.state_fields.get(idx)).copied();
            GameplayRuntimeValue {
                field_id: field
                    .map(|field| field.id.to_string())
                    .unwrap_or_else(|| format!("slot_{idx}")),
                value: *value,
                visibility: field.map(|field| field.visibility).unwrap_or(StateVisibility::Hidden),
                persistence: field.map(|field| field.persistence).unwrap_or(persistence),
                lifetime: field.map(|field| field.lifetime).unwrap_or(match persistence {
                    PersistenceScope::Combat => Lifetime::Combat,
                    PersistenceScope::Run => Lifetime::Run,
                }),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision::RewardItemKind;
    use crate::map::RoomType;
    use crate::run::RunPhase;
    use crate::status_ids::sid;
    use crate::tests::support::{engine_with, run_engine};

    fn set_first_reachable_room(engine: &mut RunEngine, room_type: RoomType) {
        let start = engine.map.get_start_nodes()[0];
        let (x, y) = (start.x, start.y);
        engine.map.rows[y][x].room_type = room_type;
    }

    #[test]
    fn combat_snapshot_exposes_active_runtime_instances() {
        let mut engine = engine_with(crate::tests::support::make_deck(&["Strike_R"]), 20, 5);
        engine.state.relics.push("OrangePellets".to_string());
        engine.state.potions[0] = "Block Potion".to_string();
        engine.rebuild_effect_runtime();

        let snapshot = engine.gameplay_runtime_snapshot();
        assert_eq!(snapshot.scope, GameplayRuntimeScope::Combat);
        assert!(snapshot
            .instances
            .iter()
            .any(|instance| instance.domain == GameplayDomain::Relic && instance.def_id == "OrangePellets"));
        assert!(snapshot
            .instances
            .iter()
            .any(|instance| instance.domain == GameplayDomain::Potion));
        assert!(snapshot
            .instances
            .iter()
            .any(|instance| instance.runtime_order == 0 && matches!(instance.owner, GameplayOwner::PlayerRelic { slot: 0 })));
        assert!(snapshot
            .instances
            .iter()
            .any(|instance| instance.program.is_legacy_adapted()));
    }

    #[test]
    fn run_snapshot_exposes_reward_screen_and_decision_state() {
        let mut engine = run_engine(42, 20);
        engine.debug_set_card_reward_screen(vec!["Wallop".to_string(), "Scrawl".to_string()]);

        let snapshot = engine.gameplay_runtime_snapshot();
        assert_eq!(snapshot.scope, GameplayRuntimeScope::Run);
        assert!(snapshot.decision_state.is_some());
        assert!(snapshot.decision_context.is_some());
        let screen = snapshot.reward_screen.expect("reward screen should be present");
        assert_eq!(screen.items.len(), 1);
        assert_eq!(screen.items[0].kind, RewardItemKind::CardChoice);
        let decision_program = snapshot.decision_program.expect("decision program should be present");
        assert!(decision_program
            .steps
            .iter()
            .any(|step| matches!(step, EffectOp::RewardScreen { .. })));
    }

    #[test]
    fn combat_snapshot_assigns_stable_power_install_order() {
        let mut engine = engine_with(crate::tests::support::make_deck(&["Strike_R"]), 20, 5);
        engine.state.player.set_status(sid::THOUSAND_CUTS, 1);
        engine.state.player.set_status(sid::PANACHE, 1);
        engine.rebuild_effect_runtime();

        let snapshot = engine.gameplay_runtime_snapshot();
        let power_orders: Vec<u16> = snapshot
            .instances
            .iter()
            .filter_map(|instance| match instance.owner {
                GameplayOwner::PlayerPower { install_order } => Some(install_order),
                _ => None,
            })
            .collect();
        assert_eq!(power_orders, vec![0, 1]);
    }

    #[test]
    fn run_snapshot_matches_combat_snapshot_when_in_combat() {
        let mut engine = run_engine(42, 20);
        set_first_reachable_room(&mut engine, RoomType::Monster);
        let action = engine.get_legal_actions()[0].clone();
        let _ = engine.step_with_result(&action);
        assert_eq!(engine.current_phase(), RunPhase::Combat);

        let run_snapshot = engine.gameplay_runtime_snapshot();
        let combat_snapshot = engine.get_combat_engine().expect("combat engine").gameplay_runtime_snapshot();

        assert_eq!(run_snapshot.instances, combat_snapshot.instances);
        assert_eq!(run_snapshot.persisted_states, combat_snapshot.persisted_states);
        assert_eq!(run_snapshot.recent_events, combat_snapshot.recent_events);
        assert!(run_snapshot.decision_state.is_some());
        assert!(run_snapshot.decision_context.is_some());
        assert_eq!(run_snapshot.decision_program, combat_snapshot.decision_program);
    }

    #[test]
    fn runtime_values_use_declared_field_metadata() {
        let mut engine = engine_with(crate::tests::support::make_deck(&["Strike_R"]), 20, 5);
        engine.state.relics.push("Happy Flower".to_string());
        engine.rebuild_effect_runtime();
        engine.execute_action(&crate::actions::Action::EndTurn);
        engine.execute_action(&crate::actions::Action::EndTurn);

        let snapshot = engine.gameplay_runtime_snapshot();
        let happy_flower = snapshot
            .instances
            .iter()
            .find(|instance| instance.def_id == "Happy Flower")
            .expect("happy flower runtime instance");

        assert_eq!(happy_flower.values[0].field_id, "counter");
        assert_eq!(happy_flower.values[0].visibility, StateVisibility::Observable);
    }
}
