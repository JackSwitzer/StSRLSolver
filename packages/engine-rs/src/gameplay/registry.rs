use std::collections::HashMap;
use std::sync::OnceLock;

use crate::effects::entity_def::{EntityDef, EntityKind};
use crate::gameplay::types::{
    EventSchema, GameplayDef, GameplayDomain, GameplayEventKind, GameplayHandler, GameplaySchema,
    GameplayProgram, GameplayStateField, Lifetime, PotionSchema, PowerSchema, RelicSchema,
    RunEffectSchema, StateVisibility,
};

#[derive(Debug, Clone)]
pub struct GameplayRegistry {
    defs: Vec<GameplayDef>,
    by_domain_and_id: HashMap<(GameplayDomain, String), usize>,
}

impl GameplayRegistry {
    pub fn new() -> Self {
        let mut defs = Vec::new();
        defs.extend(card_defs());
        defs.extend(entity_defs(
            GameplayDomain::Relic,
            crate::relics::defs::RELIC_DEFS,
        ));
        defs.extend(entity_defs(
            GameplayDomain::Power,
            crate::powers::defs::POWER_DEFS,
        ));
        defs.extend(entity_defs(
            GameplayDomain::Potion,
            crate::potions::defs::POTION_DEFS,
        ));
        defs.extend(event_defs());
        defs.extend(enemy_defs());
        defs.extend(run_effect_defs());

        let by_domain_and_id = defs
            .iter()
            .enumerate()
            .map(|(idx, def)| ((def.domain, def.id.clone()), idx))
            .collect();

        Self {
            defs,
            by_domain_and_id,
        }
    }

    pub fn defs(&self) -> &[GameplayDef] {
        &self.defs
    }

    pub fn defs_for_domain(&self, domain: GameplayDomain) -> impl Iterator<Item = &GameplayDef> {
        self.defs.iter().filter(move |def| def.domain == domain)
    }

    pub fn count_for_domain(&self, domain: GameplayDomain) -> usize {
        self.defs_for_domain(domain).count()
    }

    pub fn contains(&self, domain: GameplayDomain, id: &str) -> bool {
        self.by_domain_and_id.contains_key(&(domain, id.to_string()))
    }

    pub fn get(&self, domain: GameplayDomain, id: &str) -> Option<&GameplayDef> {
        self.by_domain_and_id
            .get(&(domain, id.to_string()))
            .and_then(|idx| self.defs.get(*idx))
    }

    pub fn card(&self, id: &str) -> Option<&GameplayDef> {
        self.get(GameplayDomain::Card, id)
    }

    pub fn enemy(&self, id: &str) -> Option<&GameplayDef> {
        self.get(GameplayDomain::Enemy, id)
    }

    pub fn relic(&self, id: &str) -> Option<&GameplayDef> {
        self.get(GameplayDomain::Relic, id)
    }

    pub fn power(&self, id: &str) -> Option<&GameplayDef> {
        self.get(GameplayDomain::Power, id)
    }

    pub fn potion(&self, id: &str) -> Option<&GameplayDef> {
        self.get(GameplayDomain::Potion, id)
    }

    pub fn defs_for_tag<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = &'a GameplayDef> + 'a {
        self.defs
            .iter()
            .filter(move |def| def.tags.iter().any(|candidate| candidate == tag))
    }

    pub fn program(&self, domain: GameplayDomain, id: &str) -> Option<GameplayProgram> {
        self.get(domain, id).map(|def| def.program())
    }
}

impl Default for GameplayRegistry {
    fn default() -> Self {
        Self::new()
    }
}

static GLOBAL_GAMEPLAY_REGISTRY: OnceLock<GameplayRegistry> = OnceLock::new();

pub fn global_registry() -> &'static GameplayRegistry {
    GLOBAL_GAMEPLAY_REGISTRY.get_or_init(GameplayRegistry::new)
}

fn card_defs() -> Vec<GameplayDef> {
    crate::cards::gameplay_export_defs()
}

fn entity_defs(domain: GameplayDomain, defs: &[&EntityDef]) -> Vec<GameplayDef> {
    defs.iter()
        .map(|def| GameplayDef {
            domain,
            id: def.id.to_string(),
            name: def.name.to_string(),
            tags: tags_for_entity_kind(domain, def.kind),
            schema: match def.kind {
                EntityKind::Relic => GameplaySchema::Relic(RelicSchema {
                    inventory_item: true,
                }),
                EntityKind::Power => GameplaySchema::Power(PowerSchema {
                    status_guard: def.status_guard.map(|status| status.0),
                }),
                EntityKind::Potion => GameplaySchema::Potion(PotionSchema {
                    target_required: crate::potions::potion_requires_target(def.name)
                        || crate::potions::potion_requires_target(def.id),
                    manual_activation: def
                        .triggers
                        .iter()
                        .any(|handler| handler.trigger == crate::effects::trigger::Trigger::ManualActivation),
                }),
            },
            handlers: def
                .triggers
                .iter()
                .map(|handler| GameplayHandler {
                    event: GameplayEventKind::from(handler.trigger),
                    label: format!("{:?}", handler.trigger),
                })
                .collect(),
            state_fields: state_fields_for_entity(def.id),
            has_complex_hook: def.complex_hook.is_some(),
        })
        .collect()
}

fn tags_for_entity_kind(domain: GameplayDomain, kind: EntityKind) -> Vec<String> {
    let mut tags = vec![format!("{domain:?}"), format!("{kind:?}")];
    tags.sort();
    tags.dedup();
    tags
}

fn state_fields_for_entity(def_id: &str) -> Vec<GameplayStateField> {
    match def_id {
        "Nunchaku" | "InkBottle" | "Happy Flower" | "Incense Burner" | "Sundial" => vec![
            GameplayStateField {
                id: "counter",
                visibility: StateVisibility::Observable,
                persistence: crate::effects::runtime::PersistenceScope::Run,
                lifetime: Lifetime::Run,
            },
        ],
        "OrangePellets" | "Pocketwatch" | "panache" | "StoneCalendar" | "Inserter" => vec![
            GameplayStateField {
                id: "counter",
                visibility: StateVisibility::Observable,
                persistence: crate::effects::runtime::PersistenceScope::Combat,
                lifetime: Lifetime::Combat,
            },
        ],
        _ => Vec::new(),
    }
}

fn event_defs() -> Vec<GameplayDef> {
    let mut defs = Vec::new();
    for act in [1, 2, 3] {
        defs.extend(
            crate::events::typed_events_for_act(act)
                .into_iter()
                .map(|event| GameplayDef {
                    domain: GameplayDomain::Event,
                    id: format!("act{act}:{}", event.name),
                    name: event.name.clone(),
                    tags: vec![format!("act:{act}")],
                    schema: GameplaySchema::Event(EventSchema {
                        option_count: event.options.len(),
                        act: Some(act),
                        shrine: false,
                    }),
                    handlers: Vec::new(),
                    state_fields: Vec::new(),
                    has_complex_hook: false,
                }),
        );
    }
    defs.extend(
        crate::events::typed_shrine_events()
            .into_iter()
            .map(|event| GameplayDef {
                domain: GameplayDomain::Event,
                id: format!("shrine:{}", event.name),
                name: event.name.clone(),
                tags: vec!["shrine".to_string()],
                schema: GameplaySchema::Event(EventSchema {
                    option_count: event.options.len(),
                    act: None,
                    shrine: true,
                }),
                handlers: Vec::new(),
                state_fields: Vec::new(),
                has_complex_hook: false,
            }),
    );
    defs
}

fn enemy_defs() -> Vec<GameplayDef> {
    crate::enemies::gameplay_export_defs()
}

fn run_effect_defs() -> Vec<GameplayDef> {
    vec![GameplayDef {
        domain: GameplayDomain::RunEffect,
        id: "run:core".to_string(),
        name: "Core Run Effects".to_string(),
        tags: vec!["run".to_string()],
        schema: GameplaySchema::RunEffect(RunEffectSchema {
            source: "RunEngine".to_string(),
        }),
        handlers: Vec::new(),
        state_fields: Vec::new(),
        has_complex_hook: false,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn gameplay_registry_exposes_core_domains() {
        let registry = GameplayRegistry::new();
        for domain in [
            GameplayDomain::Card,
            GameplayDomain::Relic,
            GameplayDomain::Power,
            GameplayDomain::Potion,
            GameplayDomain::Enemy,
            GameplayDomain::Event,
            GameplayDomain::RunEffect,
        ] {
            assert!(
                registry.defs_for_domain(domain).next().is_some(),
                "missing domain {domain:?}"
            );
        }
    }

    #[test]
    fn gameplay_registry_exports_existing_entity_defs() {
        let registry = GameplayRegistry::new();
        assert!(registry.get(GameplayDomain::Relic, "OrangePellets").is_some());
        assert!(registry.get(GameplayDomain::Power, "thousand_cuts").is_some());
        assert!(registry.get(GameplayDomain::Potion, "SneckoOil").is_some());
        assert!(registry.get(GameplayDomain::Card, "Strike_P").is_some());
    }

    #[test]
    fn gameplay_registry_programs_are_introspectable() {
        let registry = GameplayRegistry::new();

        let strike_program = registry
            .program(GameplayDomain::Card, "Strike_P")
            .expect("strike program");
        assert_eq!(strike_program.source, crate::gameplay::GameplayProgramSource::Canonical);
        assert!(strike_program
            .steps
            .iter()
            .any(|step| matches!(step, crate::gameplay::EffectOp::PlayCard { .. })));

        let event_program = registry
            .defs_for_domain(GameplayDomain::Event)
            .next()
            .expect("event def")
            .program();
        assert_eq!(event_program.source, crate::gameplay::GameplayProgramSource::Canonical);
        assert!(event_program
            .steps
            .iter()
            .any(|step| matches!(step, crate::gameplay::EffectOp::OpenChoice { .. })));

        let pellets_program = registry
            .program(GameplayDomain::Relic, "OrangePellets")
            .expect("orange pellets program");
        assert!(pellets_program
            .steps
            .iter()
            .any(|step| matches!(step, crate::gameplay::EffectOp::BindHandler { .. })));
    }

    #[test]
    fn gameplay_registry_event_ids_are_unique() {
        let registry = GameplayRegistry::new();
        let ids: BTreeSet<_> = registry
            .defs_for_domain(GameplayDomain::Event)
            .map(|def| def.id.clone())
            .collect();
        assert_eq!(
            ids.len(),
            registry.defs_for_domain(GameplayDomain::Event).count()
        );
    }

    #[test]
    fn typed_registry_lookups_match_domain_getters() {
        let registry = GameplayRegistry::new();
        assert_eq!(registry.card("Strike_P"), registry.get(GameplayDomain::Card, "Strike_P"));
        assert_eq!(registry.enemy("JawWorm"), registry.get(GameplayDomain::Enemy, "JawWorm"));
        assert_eq!(registry.relic("OrangePellets"), registry.get(GameplayDomain::Relic, "OrangePellets"));
        assert_eq!(registry.power("thousand_cuts"), registry.get(GameplayDomain::Power, "thousand_cuts"));
        assert_eq!(registry.potion("SneckoOil"), registry.get(GameplayDomain::Potion, "SneckoOil"));
        assert!(registry.contains(GameplayDomain::Card, "Strike_P"));
        assert!(registry.count_for_domain(GameplayDomain::Enemy) > 0);
    }

    #[test]
    fn domain_exports_feed_global_registry_honestly() {
        let registry = GameplayRegistry::new();
        let exported_cards = crate::cards::gameplay_export_defs();
        let exported_enemies = crate::enemies::gameplay_export_defs();

        assert_eq!(registry.count_for_domain(GameplayDomain::Card), exported_cards.len());
        assert_eq!(registry.count_for_domain(GameplayDomain::Enemy), exported_enemies.len());
        assert_eq!(registry.card("Strike_P"), exported_cards.iter().find(|def| def.id == "Strike_P"));
        assert_eq!(registry.enemy("JawWorm"), exported_enemies.iter().find(|def| def.id == "JawWorm"));
    }
}
