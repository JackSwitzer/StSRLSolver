use crate::cards::{CardTarget, CardType};
use crate::effects::declarative::AmountSource;
use crate::orbs::OrbType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameplayDomain {
    Card,
    Relic,
    Power,
    Potion,
    Enemy,
    Event,
    RunEffect,
}

impl GameplayDomain {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Card => "card",
            Self::Relic => "relic",
            Self::Power => "power",
            Self::Potion => "potion",
            Self::Enemy => "enemy",
            Self::Event => "event",
            Self::RunEffect => "run_effect",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameplayOwner {
    Player,
    PlayerRelic { slot: u16 },
    PlayerPower { install_order: u16 },
    Enemy { enemy_idx: u16 },
    EnemyPower { enemy_idx: u16, install_order: u16 },
    PotionSlot { slot: u8 },
    Run,
    RewardScreen,
    EventContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateVisibility {
    Hidden,
    Observable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifetime {
    Instant,
    UntilEvent(GameplayEventKind),
    Combat,
    Run,
    Charges(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameplayProgramSource {
    Canonical,
    AdaptedLegacy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameplayStateField {
    pub id: &'static str,
    pub visibility: StateVisibility,
    pub persistence: crate::effects::runtime::PersistenceScope,
    pub lifetime: Lifetime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameplayEventKind {
    CombatStart,
    CombatStartPreDraw,
    TurnStart,
    TurnStartPostDraw,
    TurnEnd,
    CombatVictory,
    CombatDefeat,
    CardOffered,
    CardChosen,
    CardPlayRequested,
    CardCostResolved,
    CardPrePlay,
    CardEffectResolved,
    CardResolved,
    ReplayRequested,
    ReplayResolved,
    CardDrawn,
    CardDiscarded,
    CardExhausted,
    DrawPileShuffled,
    OutgoingHitCalculated,
    OutgoingHitResolved,
    IncomingHitCalculated,
    IncomingHitResolved,
    HpLoss,
    BlockGained,
    BlockBroken,
    StatusApplied,
    DebuffApplied,
    StatusRemoved,
    StatusChanged,
    OrbChanneled,
    OrbPassiveTriggered,
    OrbEvoked,
    OrbSlotsChanged,
    PotionActivated,
    PotionResolved,
    PotionConsumed,
    ReviveTriggered,
    EnemySpawned,
    EnemyIntentSet,
    EnemyDeath,
    EntityRevived,
    StanceChanged,
    CombatRewardsCreated,
    RewardItemClaimed,
    RewardItemSkipped,
    RoomEntered,
    EventOptionChosen,
    ShopPurchase,
    ShopRemoval,
    CampfireAction,
    MapAdvance,
    Legacy(crate::effects::trigger::Trigger),
}

impl From<crate::effects::trigger::Trigger> for GameplayEventKind {
    fn from(value: crate::effects::trigger::Trigger) -> Self {
        match value {
            crate::effects::trigger::Trigger::CombatStart => Self::CombatStart,
            crate::effects::trigger::Trigger::CombatStartPreDraw => Self::CombatStartPreDraw,
            crate::effects::trigger::Trigger::TurnStart => Self::TurnStart,
            crate::effects::trigger::Trigger::TurnStartPostDraw => Self::TurnStartPostDraw,
            crate::effects::trigger::Trigger::TurnEnd => Self::TurnEnd,
            crate::effects::trigger::Trigger::CombatVictory => Self::CombatVictory,
            crate::effects::trigger::Trigger::OnCardPlayedPre => Self::CardPrePlay,
            crate::effects::trigger::Trigger::OnCardPlayedPost => Self::CardResolved,
            crate::effects::trigger::Trigger::OnCardExhaust => Self::CardExhausted,
            crate::effects::trigger::Trigger::OnCardDiscard => Self::CardDiscarded,
            crate::effects::trigger::Trigger::OnPlayerHpLoss => Self::HpLoss,
            crate::effects::trigger::Trigger::OnEnemyDeath => Self::EnemyDeath,
            crate::effects::trigger::Trigger::OnShuffle => Self::DrawPileShuffled,
            crate::effects::trigger::Trigger::OnStanceChange => Self::StanceChanged,
            crate::effects::trigger::Trigger::OnPotionUsed => Self::PotionConsumed,
            crate::effects::trigger::Trigger::EnemyTurnStart => Self::EnemyIntentSet,
            crate::effects::trigger::Trigger::ManualActivation => Self::PotionActivated,
            crate::effects::trigger::Trigger::DamageResolved => Self::OutgoingHitResolved,
            crate::effects::trigger::Trigger::OnDebuffApplied => Self::DebuffApplied,
            crate::effects::trigger::Trigger::OnBlockBroken => Self::BlockBroken,
            crate::effects::trigger::Trigger::DamageCalculation => Self::OutgoingHitCalculated,
            other => Self::Legacy(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayHandler {
    pub event: GameplayEventKind,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayProgram {
    pub source: GameplayProgramSource,
    pub steps: Vec<EffectOp>,
}

impl GameplayProgram {
    pub fn canonical(steps: Vec<EffectOp>) -> Self {
        Self {
            source: GameplayProgramSource::Canonical,
            steps,
        }
    }

    pub fn adapted_legacy(steps: Vec<EffectOp>) -> Self {
        Self {
            source: GameplayProgramSource::AdaptedLegacy,
            steps,
        }
    }

    pub fn is_legacy_adapted(&self) -> bool {
        matches!(self.source, GameplayProgramSource::AdaptedLegacy)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectOp {
    DeclareDefinition {
        domain: GameplayDomain,
        id: String,
        name: String,
    },
    EmitEvent {
        event: GameplayEventKind,
        label: String,
    },
    BindHandler {
        event: GameplayEventKind,
        label: String,
    },
    InstallStateField(GameplayStateField),
    OpenChoice {
        label: String,
        option_count: usize,
    },
    RewardScreen {
        label: String,
        source: String,
        ordered: bool,
        active_item: Option<usize>,
        item_count: usize,
    },
    PlayCard {
        card_type: Option<CardType>,
        target: Option<CardTarget>,
        cost: Option<i32>,
        exhausts: bool,
        upgraded_from: Option<String>,
        declared_effect_count: usize,
        declared_extra_hits: bool,
        declared_stance_change: bool,
        declared_all_enemy_damage: Option<AmountSource>,
        declared_discard_from_hand: Option<ChoiceCountHint>,
        declared_exhaust_from_hand: Option<ChoiceCountHint>,
        declared_scry_count: Option<AmountSource>,
        declared_channel_orbs: Vec<OrbCountHint>,
        declared_evoke_count: Option<AmountSource>,
        uses_x_cost: bool,
        declared_x_cost_amounts: Vec<AmountSource>,
    },
    LegacyAdapter {
        label: String,
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChoiceCountHint {
    pub min: AmountSource,
    pub max: AmountSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrbCountHint {
    pub orb_type: OrbType,
    pub count: AmountSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CardSchema {
    pub card_type: Option<CardType>,
    pub target: Option<CardTarget>,
    pub cost: Option<i32>,
    pub exhausts: bool,
    pub upgraded_from: Option<String>,
    pub declared_effect_count: usize,
    pub declared_extra_hits: bool,
    pub declared_stance_change: bool,
    pub declared_all_enemy_damage: Option<AmountSource>,
    pub declared_discard_from_hand: Option<ChoiceCountHint>,
    pub declared_exhaust_from_hand: Option<ChoiceCountHint>,
    pub declared_scry_count: Option<AmountSource>,
    pub declared_channel_orbs: Vec<OrbCountHint>,
    pub declared_evoke_count: Option<AmountSource>,
    pub uses_x_cost: bool,
    pub declared_x_cost_amounts: Vec<AmountSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RelicSchema {
    pub inventory_item: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PowerSchema {
    pub status_guard: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PotionSchema {
    pub target_required: bool,
    pub manual_activation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnemySchema {
    pub move_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EventSchema {
    pub option_count: usize,
    pub act: Option<i32>,
    pub shrine: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunEffectSchema {
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameplaySchema {
    Card(CardSchema),
    Relic(RelicSchema),
    Power(PowerSchema),
    Potion(PotionSchema),
    Enemy(EnemySchema),
    Event(EventSchema),
    RunEffect(RunEffectSchema),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayDef {
    pub domain: GameplayDomain,
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
    pub schema: GameplaySchema,
    pub handlers: Vec<GameplayHandler>,
    pub state_fields: Vec<GameplayStateField>,
    pub has_complex_hook: bool,
}

impl GameplayDef {
    pub fn program_source(&self) -> GameplayProgramSource {
        match self.domain {
            GameplayDomain::Event | GameplayDomain::RunEffect => GameplayProgramSource::Canonical,
            GameplayDomain::Card
            | GameplayDomain::Relic
            | GameplayDomain::Power
            | GameplayDomain::Potion
            | GameplayDomain::Enemy => GameplayProgramSource::AdaptedLegacy,
        }
    }

    pub fn program(&self) -> GameplayProgram {
        let mut steps = vec![EffectOp::DeclareDefinition {
            domain: self.domain,
            id: self.id.clone(),
            name: self.name.clone(),
        }];

        steps.extend(
            self.state_fields
                .iter()
                .copied()
                .map(EffectOp::InstallStateField),
        );

        steps.extend(self.handlers.iter().map(|handler| EffectOp::BindHandler {
            event: handler.event,
            label: handler.label.clone(),
        }));

        match &self.schema {
            GameplaySchema::Card(schema) => {
                steps.push(EffectOp::EmitEvent {
                    event: GameplayEventKind::CardPlayRequested,
                    label: self.name.clone(),
                });
                steps.push(EffectOp::EmitEvent {
                    event: GameplayEventKind::CardPrePlay,
                    label: self.name.clone(),
                });
                steps.push(EffectOp::PlayCard {
                    card_type: schema.card_type,
                    target: schema.target,
                    cost: schema.cost,
                    exhausts: schema.exhausts,
                    upgraded_from: schema.upgraded_from.clone(),
                    declared_effect_count: schema.declared_effect_count,
                    declared_extra_hits: schema.declared_extra_hits,
                    declared_stance_change: schema.declared_stance_change,
                    declared_all_enemy_damage: schema.declared_all_enemy_damage,
                    declared_discard_from_hand: schema.declared_discard_from_hand,
                    declared_exhaust_from_hand: schema.declared_exhaust_from_hand,
                    declared_scry_count: schema.declared_scry_count,
                    declared_channel_orbs: schema.declared_channel_orbs.clone(),
                    declared_evoke_count: schema.declared_evoke_count,
                    uses_x_cost: schema.uses_x_cost,
                    declared_x_cost_amounts: schema.declared_x_cost_amounts.clone(),
                });
                steps.push(EffectOp::EmitEvent {
                    event: GameplayEventKind::CardEffectResolved,
                    label: self.name.clone(),
                });
                steps.push(EffectOp::EmitEvent {
                    event: GameplayEventKind::CardResolved,
                    label: self.name.clone(),
                });
            }
            GameplaySchema::Event(schema) => {
                steps.push(EffectOp::OpenChoice {
                    label: self.name.clone(),
                    option_count: schema.option_count,
                });
                steps.push(EffectOp::LegacyAdapter {
                    label: self.name.clone(),
                    reason: if schema.shrine {
                        "event shrine flow".to_string()
                    } else {
                        "event option flow".to_string()
                    },
                });
            }
            GameplaySchema::RunEffect(schema) => {
                steps.push(EffectOp::LegacyAdapter {
                    label: schema.source.clone(),
                    reason: "run effect adapter".to_string(),
                });
            }
            GameplaySchema::Relic(_)
            | GameplaySchema::Power(_)
            | GameplaySchema::Potion(_)
            | GameplaySchema::Enemy(_) => {
                steps.push(EffectOp::LegacyAdapter {
                    label: self.name.clone(),
                    reason: "legacy-adapted entity runtime".to_string(),
                });
            }
        }

        match self.program_source() {
            GameplayProgramSource::Canonical => GameplayProgram::canonical(steps),
            GameplayProgramSource::AdaptedLegacy => GameplayProgram::adapted_legacy(steps),
        }
    }

    pub fn state_field(&self, id: &str) -> Option<&GameplayStateField> {
        self.state_fields.iter().find(|field| field.id == id)
    }

    pub fn handles(&self, event: GameplayEventKind) -> bool {
        self.handlers.iter().any(|handler| handler.event == event)
    }

    pub fn card_schema(&self) -> Option<&CardSchema> {
        match &self.schema {
            GameplaySchema::Card(schema) => Some(schema),
            _ => None,
        }
    }

    pub fn enemy_schema(&self) -> Option<&EnemySchema> {
        match &self.schema {
            GameplaySchema::Enemy(schema) => Some(schema),
            _ => None,
        }
    }
}
