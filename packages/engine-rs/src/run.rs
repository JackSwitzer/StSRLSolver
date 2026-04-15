//! Run state management — full Act 1 run simulation.
//!
//! Manages floor progression, deck building, card rewards, events,
//! shops, campfires, and combat via the existing CombatEngine.
//! Exposes step(action) -> (obs, reward, done, info) RL interface.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::decision::{
    build_combat_context, build_shop_context, CampfireDecisionContext, CombatContext,
    DecisionAction, DecisionContext, DecisionFrame, DecisionKind, DecisionStack, DecisionState,
    EventDecisionContext, EventOptionContext, MapDecisionContext, NeowDecisionContext,
    NeowOptionContext, RewardChoice,
    RewardChoiceFrame, RewardItem, RewardItemKind, RewardItemState, RewardScreen,
    RewardScreenSource,
};
use crate::enemies;
use crate::engine::CombatEngine;
use crate::gameplay::registry::global_registry as gameplay_registry;
use crate::gameplay::types::GameplayDomain;
use crate::map::{generate_map, DungeonMap, RoomType};
use crate::state::{CombatState, EnemyCombatState};

static NOTE_FOR_YOURSELF_CARD: OnceLock<Mutex<String>> = OnceLock::new();

fn note_for_yourself_slot() -> &'static Mutex<String> {
    NOTE_FOR_YOURSELF_CARD.get_or_init(|| Mutex::new("IronWave".to_string()))
}

fn current_note_for_yourself_card() -> String {
    note_for_yourself_slot()
        .lock()
        .expect("note-for-yourself mutex poisoned")
        .clone()
}

fn set_note_for_yourself_card(card_id: String) {
    *note_for_yourself_slot()
        .lock()
        .expect("note-for-yourself mutex poisoned") = card_id;
}

// ---------------------------------------------------------------------------
// Run-level action (distinct from combat Action)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RunAction {
    /// Choose one of the opening Neow options.
    ChooseNeowOption(usize),
    /// Choose a path on the map: index into available next nodes
    ChoosePath(usize),
    /// Click or claim a reward item in the current ordered reward screen.
    SelectRewardItem(usize),
    /// Choose an option within the currently active reward item.
    ChooseRewardOption {
        item_index: usize,
        choice_index: usize,
    },
    /// Skip a skippable reward item.
    SkipRewardItem(usize),
    /// Campfire: rest (heal 30% max HP)
    CampfireRest,
    /// Campfire: upgrade a card (index into deck)
    CampfireUpgrade(usize),
    /// Shop: buy a card (index into shop offerings)
    ShopBuyCard(usize),
    /// Shop: remove a card (index into deck)
    ShopRemoveCard(usize),
    /// Shop: skip/leave shop
    ShopLeave,
    /// Event: choose an option (index)
    EventChoice(usize),
    /// Combat action: play card, use potion, or end turn
    CombatAction(crate::actions::Action),
}

// ---------------------------------------------------------------------------
// Decision phase
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunPhase {
    /// Opening Neow choice before the map starts.
    Neow,
    /// Choose next room on map
    MapChoice,
    /// In combat
    Combat,
    /// Picking card reward after combat
    CardReward,
    /// At campfire
    Campfire,
    /// In shop
    Shop,
    /// In event
    Event,
    /// Run is over (win or loss)
    GameOver,
}

// ---------------------------------------------------------------------------
// Card pool for rewards
// ---------------------------------------------------------------------------

/// Simplified Act 1 Watcher card pool for rewards.
/// Uses CardRegistry IDs (PascalCase, no spaces) to match card lookups.
/// Cards not in CardRegistry fall back to get_or_default() (1-cost 6-damage attack).
/// This is intentional for the fast MCTS path — full card effects are in the Python engine.
const WATCHER_COMMON_CARDS: &[&str] = &[
    "BowlingBash", "Consecrate", "Crescendo", "CrushJoints",
    "CutThroughFate", "EmptyBody", "EmptyFist", "Evaluate",
    "FlurryOfBlows", "FlyingSleeves", "FollowUp", "Halt",
    "JustLucky", "PathToVictory", "Prostrate",
    "Protect", "SashWhip", "ClearTheMind",
];

const WATCHER_UNCOMMON_CARDS: &[&str] = &[
    "BattleHymn", "CarveReality", "Conclude", "DeceiveReality",
    "EmptyMind", "FearNoEvil", "ForeignInfluence", "Indignation",
    "InnerPeace", "LikeWater", "Meditate", "Nirvana",
    "Perseverance", "ReachHeaven", "SandsOfTime", "SignatureMove",
    "Smite", "Study", "Swivel", "TalkToTheHand",
    "Tantrum", "ThirdEye", "Wallop", "WaveOfTheHand",
    "Weave", "WheelKick", "WindmillStrike", "WreathOfFlame",
];

const WATCHER_RARE_CARDS: &[&str] = &[
    "Alpha", "Blasphemy", "Brilliance", "ConjureBlade",
    "DevaForm", "Devotion", "Establishment", "Fasting2",
    "Judgement", "LessonLearned", "MasterReality",
    "MentalFortress", "Omniscience", "Ragnarok",
    "Adaptation", "Scrawl", "SpiritShield", "Vault",
    "Wish",
];

const MATCH_AND_KEEP_COLORLESS_UNCOMMON_CARDS: &[&str] = &[
    "Blind",
    "Dark Shackles",
    "Deep Breath",
    "Discovery",
    "Enlightenment",
    "Finesse",
    "Forethought",
    "Impatience",
    "Madness",
    "Panacea",
    "PanicButton",
    "Purity",
    "Trip",
];

const MATCH_AND_KEEP_CURSES: &[&str] = &[
    "Clumsy",
    "Decay",
    "Doubt",
    "Injury",
    "Normality",
    "Pain",
    "Parasite",
    "Regret",
    "Writhe",
];

// ---------------------------------------------------------------------------
// Act 1 encounter pools (simplified)
// ---------------------------------------------------------------------------

const ACT1_WEAK_ENCOUNTERS: &[&[&str]] = &[
    &["Cultist"],
    &["JawWorm"],
    &["FuzzyLouseNormal", "FuzzyLouseDefensive"],
    &["AcidSlime_S", "SpikeSlime_M"],
];

const ACT1_STRONG_ENCOUNTERS: &[&[&str]] = &[
    &["BlueSlaver"],
    &["RedSlaver"],
    &["FuzzyLouseNormal", "FuzzyLouseDefensive", "FuzzyLouseNormal"],
    &["FungiBeast", "FungiBeast"],
    &["AcidSlime_L"],
    &["SpikeSlime_L"],
    &["AcidSlime_M", "SpikeSlime_M"],
];

const ACT1_ELITE_ENCOUNTERS: &[&[&str]] = &[
    &["GremlinNob"],
    &["Lagavulin"],
    &["Sentry", "Sentry", "Sentry"],
];

const ACT1_BOSSES: &[&str] = &["TheGuardian", "Hexaghost", "SlimeBoss"];

// ---------------------------------------------------------------------------
// Act 2 encounter pools
// ---------------------------------------------------------------------------

const ACT2_WEAK_ENCOUNTERS: &[&[&str]] = &[
    &["Byrd", "Byrd"],
    &["Chosen"],
    &["ShelledParasite"],
    &["Cultist", "Chosen"],
];

const ACT2_STRONG_ENCOUNTERS: &[&[&str]] = &[
    &["SnakePlant"],
    &["Centurion", "Mystic"],
    &["Cultist", "Cultist", "Cultist"],
    &["Byrd", "Byrd", "Byrd"],
    &["Chosen", "Cultist"],
    &["ShelledParasite", "FungiBeast"],
];

const ACT2_ELITE_ENCOUNTERS: &[&[&str]] = &[
    &["GremlinLeader"],
    &["BookOfStabbing"],
    &["TaskMaster"],
];

// ---------------------------------------------------------------------------
// Act 3 encounter pools
// ---------------------------------------------------------------------------

const ACT3_WEAK_ENCOUNTERS: &[&[&str]] = &[
    &["Darkling", "Darkling", "Darkling"],
    &["OrbWalker"],
    &["Repulsor"],
];

const ACT3_STRONG_ENCOUNTERS: &[&[&str]] = &[
    &["WrithingMass"],
    &["GiantHead"],
    &["Nemesis"],
    &["Reptomancer"],
    &["Transient"],
    &["Maw"],
    &["SpireGrowth"],
];

const ACT3_ELITE_ENCOUNTERS: &[&[&str]] = &[
    &["GiantHead"],
    &["Nemesis"],
    &["Reptomancer"],
];

// ---------------------------------------------------------------------------
// Act 4 encounters
// ---------------------------------------------------------------------------

const ACT4_ELITE_ENCOUNTERS: &[&[&str]] = &[
    &["SpireShield", "SpireSpear"],
];

// ---------------------------------------------------------------------------
// Event definitions
// ---------------------------------------------------------------------------

// Events are defined in the events module
use crate::events::{
    typed_events_for_act, EventDeckMutation, EventProgramOp, EventReward, EventRuntimeStatus,
    TypedEventDef,
};


// ---------------------------------------------------------------------------
// Shop state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopState {
    /// Cards available for purchase: (card_id, price)
    pub cards: Vec<(String, i32)>,
    /// Card removal price
    pub remove_price: i32,
    /// Whether the player has already used their one card removal this shop visit
    pub removal_used: bool,
}

// ---------------------------------------------------------------------------
// RunState — full run state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunState {
    pub current_hp: i32,
    pub max_hp: i32,
    pub gold: i32,
    pub floor: i32,
    pub act: i32,
    pub ascension: i32,
    pub deck: Vec<String>,
    pub relics: Vec<String>,
    pub potions: Vec<String>,
    pub max_potions: usize,

    // Map state
    pub map_x: i32,   // -1 before first move
    pub map_y: i32,   // -1 before first move

    // Keys
    pub has_ruby_key: bool,
    pub has_emerald_key: bool,
    pub has_sapphire_key: bool,

    // Stats
    pub combats_won: i32,
    pub elites_killed: i32,
    pub bosses_killed: i32,

    // Run outcome
    pub run_won: bool,
    pub run_over: bool,

    // Relic flags (bitfield for O(1) relic checks)
    #[serde(skip)]
    pub relic_flags: crate::relic_flags::RelicFlags,

    // Canonical owner-aware runtime state that persists across combats.
    #[serde(default)]
    pub persisted_effect_states: Vec<crate::effects::runtime::PersistedEffectState>,
}

impl RunState {
    pub fn new(ascension: i32) -> Self {
        // Watcher starter deck
        let mut deck = vec![
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Defend_P".to_string(),
            "Defend_P".to_string(),
            "Defend_P".to_string(),
            "Defend_P".to_string(),
            "Eruption".to_string(),
            "Vigilance".to_string(),
        ];

        // Ascension 10+: add Ascender's Bane (unplayable curse) to starter deck
        if ascension >= 10 {
            deck.push("AscendersBane".to_string());
        }

        let max_hp = if ascension >= 14 { 68 } else { 72 };

        let relics = vec!["PureWater".to_string()];
        let mut relic_flags = crate::relic_flags::RelicFlags::default();
        relic_flags.rebuild(&relics);

        Self {
            current_hp: max_hp,
            max_hp,
            gold: 99,
            floor: 0,
            act: 1,
            ascension,
            deck,
            relics,
            potions: vec!["".to_string(); 3],
            max_potions: 3,
            map_x: -1,
            map_y: -1,
            has_ruby_key: false,
            has_emerald_key: false,
            has_sapphire_key: false,
            combats_won: 0,
            elites_killed: 0,
            bosses_killed: 0,
            run_won: false,
            run_over: false,
            relic_flags,
            persisted_effect_states: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineStepResult {
    pub reward: f32,
    pub done: bool,
    pub phase: RunPhase,
    pub action_accepted: bool,
    pub legal_actions: Vec<RunAction>,
    pub decision_state: DecisionState,
    pub decision_context: DecisionContext,
    pub legal_decision_actions: Vec<DecisionAction>,
    pub combat_events: Vec<crate::effects::runtime::GameEventRecord>,
    pub combat_obs_v2: Option<Vec<f32>>,
    pub combat_obs_version: u32,
    pub combat_context: Option<CombatContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PendingEventCombat {
    enemies: Vec<String>,
    on_win: crate::events::EventProgram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchAndKeepScreen {
    Intro,
    RuleExplanation,
    Play,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MatchAndKeepState {
    screen: MatchAndKeepScreen,
    board: Vec<String>,
    first_pick: Option<usize>,
    attempts_left: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScrapOozeState {
    damage: i32,
    relic_chance: usize,
}

enum EventProgramFlow {
    Continue,
    ContinueEvent(TypedEventDef),
    Died,
    EndRunVictory,
    StartCombat(PendingEventCombat),
    StartBossCombat,
    StartFinalAct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventCardRarity {
    Curse,
    Basic,
    Common,
    Special,
    Uncommon,
    Rare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NeowChoiceEffect {
    GainCards,
    GainGold,
    UpgradeRandomCard,
    GainRelic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NeowChoiceOption {
    label: String,
    effect: NeowChoiceEffect,
}

// ---------------------------------------------------------------------------
// RunEngine — the full run simulation engine
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct RunEngine {
    pub run_state: RunState,
    pub map: DungeonMap,
    pub phase: RunPhase,
    pub seed: u64,
    rng: crate::seed::StsRandom,

    // Active combat (when in Combat phase)
    combat_engine: Option<CombatEngine>,

    // Ordered reward screen state (when in CardReward phase).
    reward_screen: Option<RewardScreen>,

    // Canonical stack of active decisions.
    decision_stack: DecisionStack,

    // Current event (when in Event phase)
    current_event: Option<TypedEventDef>,

    // Scripted event fight that resumes into an event-owned on-win program.
    pending_event_combat: Option<PendingEventCombat>,

    // Dynamic event runtime sidecars for stateful event flows.
    match_and_keep_state: Option<MatchAndKeepState>,
    scrap_ooze_state: Option<ScrapOozeState>,
    forced_event_rolls: Vec<usize>,

    // Current shop (when in Shop phase)
    current_shop: Option<ShopState>,

    // Boss for this act
    boss_id: String,

    // Encounter counters
    weak_encounter_idx: usize,
    strong_encounter_idx: usize,
    elite_encounter_idx: usize,

    // Reward tracking
    pub total_reward: f32,
    last_combat_events: Vec<crate::effects::runtime::GameEventRecord>,

    // Opening Neow choice before the map starts.
    neow_options: Vec<NeowChoiceOption>,
}

impl RunEngine {
    /// Create a new run engine with given seed and ascension level.
    pub fn new(seed: u64, ascension: i32) -> Self {
        let map = generate_map(seed, ascension);
        let mut rng = crate::seed::StsRandom::new(seed.wrapping_add(1));

        // Pick boss
        let boss_idx = rng.gen_range(0..ACT1_BOSSES.len());
        let boss_id = ACT1_BOSSES[boss_idx].to_string();

        let mut engine = Self {
            run_state: RunState::new(ascension),
            map,
            phase: RunPhase::Neow,
            seed,
            rng,
            combat_engine: None,
            reward_screen: None,
            decision_stack: DecisionStack::new(),
            current_event: None,
            pending_event_combat: None,
            match_and_keep_state: None,
            scrap_ooze_state: None,
            forced_event_rolls: Vec::new(),
            current_shop: None,
            boss_id,
            weak_encounter_idx: 0,
            strong_encounter_idx: 0,
            elite_encounter_idx: 0,
            total_reward: 0.0,
            last_combat_events: Vec::new(),
            neow_options: Vec::new(),
        };
        engine.neow_options = engine.build_neow_options();
        engine.refresh_decision_stack();
        engine
    }

    /// Reset the engine to a fresh run with a new seed.
    pub fn reset(&mut self, seed: u64) {
        let ascension = self.run_state.ascension;
        *self = Self::new(seed, ascension);
    }

    fn build_neow_options(&mut self) -> Vec<NeowChoiceOption> {
        let mut options = vec![
            NeowChoiceOption {
                label: "Gain 100 gold".to_string(),
                effect: NeowChoiceEffect::GainGold,
            },
            NeowChoiceOption {
                label: "Gain 3 random cards".to_string(),
                effect: NeowChoiceEffect::GainCards,
            },
            NeowChoiceOption {
                label: "Upgrade a random card".to_string(),
                effect: NeowChoiceEffect::UpgradeRandomCard,
            },
            NeowChoiceOption {
                label: "Gain a random relic".to_string(),
                effect: NeowChoiceEffect::GainRelic,
            },
        ];

        for i in (1..options.len()).rev() {
            let j = self.rng.gen_range(0..=i);
            options.swap(i, j);
        }

        options
    }

    fn apply_neow_choice(&mut self, effect: NeowChoiceEffect) {
        match effect {
            NeowChoiceEffect::GainCards => self.apply_event_deck_mutation(&EventDeckMutation::GainCard { count: 3 }),
            NeowChoiceEffect::GainGold => self.run_state.gold += 100,
            NeowChoiceEffect::UpgradeRandomCard => self.upgrade_random_cards(1),
            NeowChoiceEffect::GainRelic => {
                let relic = self.roll_reward_relic_id();
                self.add_relic_reward(&relic);
            }
        }
    }

    fn upgrade_random_cards(&mut self, count: usize) {
        let registry = crate::cards::global_registry();
        let mut eligible: Vec<usize> = self
            .run_state
            .deck
            .iter()
            .enumerate()
            .filter_map(|(idx, card_id)| {
                if card_id.ends_with('+') {
                    return None;
                }
                let upgraded = format!("{card_id}+");
                if registry.get(&upgraded).is_some() {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        for _ in 0..count {
            if eligible.is_empty() {
                break;
            }
            let pick = self.rng.gen_range(0..eligible.len());
            let deck_idx = eligible.swap_remove(pick);
            self.run_state.deck[deck_idx] = format!("{}+", self.run_state.deck[deck_idx]);
        }
    }

    /// Get the current phase.
    pub fn current_phase(&self) -> RunPhase {
        self.phase
    }

    /// Is the run over?
    pub fn is_done(&self) -> bool {
        self.run_state.run_over
    }

    /// Get the boss name for this act.
    pub fn boss_name(&self) -> &str {
        &self.boss_id
    }

    // =======================================================================
    // Legal actions
    // =======================================================================

    /// Get all legal actions in the current phase.
    pub fn get_legal_actions(&self) -> Vec<RunAction> {
        match self.phase {
            RunPhase::Neow => self.get_neow_actions(),
            RunPhase::MapChoice => self.get_map_actions(),
            RunPhase::Combat => self.get_combat_actions(),
            RunPhase::CardReward => self.get_card_reward_actions(),
            RunPhase::Campfire => self.get_campfire_actions(),
            RunPhase::Shop => self.get_shop_actions(),
            RunPhase::Event => self.get_event_actions(),
            RunPhase::GameOver => Vec::new(),
        }
    }

    pub fn get_legal_decision_actions(&self) -> Vec<DecisionAction> {
        self.get_legal_actions()
            .iter()
            .map(|action| DecisionAction::from_run_action(action, self.phase))
            .collect()
    }

    pub fn current_decision_state(&self) -> DecisionState {
        DecisionState {
            kind: self.current_decision_kind(),
            phase: self.phase,
            terminal: self.run_state.run_over,
            room_type: self.current_room_type().to_string(),
        }
    }

    pub fn current_decision_context(&self) -> DecisionContext {
        DecisionContext {
            kind: self.current_decision_kind(),
            neow: if self.phase == RunPhase::Neow {
                Some(NeowDecisionContext {
                    options: self
                        .neow_options
                        .iter()
                        .enumerate()
                        .map(|(index, option)| NeowOptionContext {
                            index,
                            label: option.label.clone(),
                        })
                        .collect(),
                })
            } else {
                None
            },
            combat: if self.phase == RunPhase::Combat {
                self.current_combat_context()
            } else {
                None
            },
            reward_screen: self.current_reward_screen(),
            map: if self.phase == RunPhase::MapChoice {
                Some(MapDecisionContext {
                    available_paths: self.get_map_actions().len(),
                })
            } else {
                None
            },
            event: if self.phase == RunPhase::Event {
                self.current_event.as_ref().map(|event| EventDecisionContext {
                    name: event.name.clone(),
                    options: event
                        .options
                        .iter()
                        .enumerate()
                        .map(|(index, option)| EventOptionContext {
                            index,
                            label: option.text.clone(),
                        })
                        .collect(),
                })
            } else {
                None
            },
            shop: if self.phase == RunPhase::Shop {
                self.current_shop
                    .as_ref()
                    .map(|shop| build_shop_context(shop, self.run_state.gold, self.run_state.deck.len()))
            } else {
                None
            },
            campfire: if self.phase == RunPhase::Campfire {
                Some(CampfireDecisionContext {
                    can_rest: !self.run_state.relic_flags.has(crate::relic_flags::flag::COFFEE_DRIPPER),
                    upgradable_cards: if self
                        .run_state
                        .relic_flags
                        .has(crate::relic_flags::flag::FUSION_HAMMER)
                    {
                        Vec::new()
                    } else {
                        self.run_state
                            .deck
                            .iter()
                            .enumerate()
                            .filter_map(|(i, card)| (!card.ends_with('+')).then_some(i))
                            .collect()
                    },
                })
            } else {
                None
            },
        }
    }

    pub fn current_decision_kind(&self) -> DecisionKind {
        if let Some(kind) = self.decision_stack.current_kind() {
            return kind;
        }
        match self.phase {
            RunPhase::Neow => DecisionKind::NeowChoice,
            RunPhase::MapChoice => DecisionKind::MapPath,
            RunPhase::Combat => {
                if self
                    .combat_engine
                    .as_ref()
                    .and_then(|combat| combat.choice.as_ref())
                    .is_some()
                {
                    DecisionKind::CombatChoice
                } else {
                    DecisionKind::CombatAction
                }
            }
            RunPhase::CardReward => DecisionKind::RewardScreen,
            RunPhase::Campfire => DecisionKind::CampfireAction,
            RunPhase::Shop => DecisionKind::ShopAction,
            RunPhase::Event => DecisionKind::EventOption,
            RunPhase::GameOver => DecisionKind::GameOver,
        }
    }

    fn refresh_decision_stack(&mut self) {
        let reward_choice = self.decision_stack.current_reward_choice().cloned();

        self.decision_stack.clear();
        let root = match self.phase {
            RunPhase::Neow => DecisionFrame::Neow(NeowDecisionContext {
                options: self
                    .neow_options
                    .iter()
                    .enumerate()
                    .map(|(index, option)| NeowOptionContext {
                        index,
                        label: option.label.clone(),
                    })
                    .collect(),
            }),
            RunPhase::MapChoice => DecisionFrame::Map(MapDecisionContext {
                available_paths: self.get_map_actions().len(),
            }),
            RunPhase::Combat => {
                if let Some(context) = self.current_combat_context() {
                    if context.choice.active {
                        DecisionFrame::CombatChoice(context.choice)
                    } else {
                        DecisionFrame::Combat(context)
                    }
                } else {
                    DecisionFrame::GameOver
                }
            }
            RunPhase::CardReward => DecisionFrame::RewardScreen {
                source: self
                    .reward_screen
                    .as_ref()
                    .map(|screen| screen.source.clone())
                    .unwrap_or(RewardScreenSource::Unknown),
            },
            RunPhase::Campfire => DecisionFrame::Campfire(CampfireDecisionContext {
                can_rest: !self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::COFFEE_DRIPPER),
                upgradable_cards: if self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::FUSION_HAMMER)
                {
                    Vec::new()
                } else {
                    self.run_state
                        .deck
                        .iter()
                        .enumerate()
                        .filter_map(|(i, card)| (!card.ends_with('+')).then_some(i))
                        .collect()
                },
            }),
            RunPhase::Shop => self
                .current_shop
                .as_ref()
                .map(|shop| DecisionFrame::Shop(build_shop_context(shop, self.run_state.gold, self.run_state.deck.len())))
                .unwrap_or(DecisionFrame::GameOver),
            RunPhase::Event => self
                .current_event
                .as_ref()
                .map(|event| DecisionFrame::Event(EventDecisionContext {
                    name: event.name.clone(),
                    options: event
                        .options
                        .iter()
                        .enumerate()
                        .map(|(index, option)| EventOptionContext {
                            index,
                            label: option.text.clone(),
                        })
                        .collect(),
                }))
                .unwrap_or(DecisionFrame::GameOver),
            RunPhase::GameOver => DecisionFrame::GameOver,
        };
        self.decision_stack.push(root);
        if let Some(choice) = reward_choice {
            if matches!(self.phase, RunPhase::CardReward) {
                self.decision_stack.push(DecisionFrame::RewardChoice(choice));
            }
        }
    }

    fn get_map_actions(&self) -> Vec<RunAction> {
        if self.run_state.map_y < 0 {
            // First move: choose from starting nodes
            let starts = self.map.get_start_nodes();
            starts.iter().enumerate().map(|(i, _)| RunAction::ChoosePath(i)).collect()
        } else {
            let x = self.run_state.map_x as usize;
            let y = self.run_state.map_y as usize;
            let next = self.map.get_next_nodes(x, y);
            next.iter().enumerate().map(|(i, _)| RunAction::ChoosePath(i)).collect()
        }
    }

    fn get_neow_actions(&self) -> Vec<RunAction> {
        self.neow_options
            .iter()
            .enumerate()
            .map(|(i, _)| RunAction::ChooseNeowOption(i))
            .collect()
    }

    fn get_combat_actions(&self) -> Vec<RunAction> {
        if let Some(ref engine) = self.combat_engine {
            engine
                .get_legal_actions()
                .into_iter()
                .map(RunAction::CombatAction)
                .collect()
        } else {
            Vec::new()
        }
    }

    fn get_card_reward_actions(&self) -> Vec<RunAction> {
        let Some(screen) = self.reward_screen.as_ref() else {
            return Vec::new();
        };

        if let Some(choice_frame) = self.decision_stack.current_reward_choice() {
            let Some(item) = screen.items.get(choice_frame.item_index) else {
                return Vec::new();
            };
            let mut actions: Vec<RunAction> = choice_frame
                .choices
                .iter()
                .enumerate()
                .map(|(choice_index, _)| RunAction::ChooseRewardOption {
                    item_index: item.index,
                    choice_index,
                })
                .collect();
            if choice_frame.skip_allowed {
                actions.push(RunAction::SkipRewardItem(item.index));
            }
            return actions;
        }

        let mut actions = Vec::new();
        for item in &screen.items {
            if item.claimable {
                actions.push(RunAction::SelectRewardItem(item.index));
                if item.skip_allowed {
                    actions.push(RunAction::SkipRewardItem(item.index));
                }
            }
        }
        actions
    }

    fn get_campfire_actions(&self) -> Vec<RunAction> {
        let mut actions = Vec::new();

        // Coffee Dripper blocks resting
        if !self.run_state.relic_flags.has(crate::relic_flags::flag::COFFEE_DRIPPER) {
            actions.push(RunAction::CampfireRest);
        }

        // Fusion Hammer blocks upgrading
        if !self.run_state.relic_flags.has(crate::relic_flags::flag::FUSION_HAMMER) {
            for (i, card) in self.run_state.deck.iter().enumerate() {
                if !card.ends_with('+') {
                    actions.push(RunAction::CampfireUpgrade(i));
                }
            }
        }

        actions
    }

    fn get_shop_actions(&self) -> Vec<RunAction> {
        let mut actions = Vec::new();
        if let Some(ref shop) = self.current_shop {
            for (i, (_, price)) in shop.cards.iter().enumerate() {
                if self.run_state.gold >= *price {
                    actions.push(RunAction::ShopBuyCard(i));
                }
            }
            if !shop.removal_used && self.run_state.gold >= shop.remove_price {
                for (i, card_id) in self.run_state.deck.iter().enumerate() {
                    if Self::is_purgeable_master_deck_card(card_id) {
                        actions.push(RunAction::ShopRemoveCard(i));
                    }
                }
            }
        }
        actions.push(RunAction::ShopLeave);
        actions
    }

    fn is_purgeable_master_deck_card(card_id: &str) -> bool {
        !matches!(card_id, "Necronomicurse" | "CurseOfTheBell" | "AscendersBane")
    }

    fn get_event_actions(&self) -> Vec<RunAction> {
        if let Some(state) = &self.match_and_keep_state {
            if matches!(state.screen, MatchAndKeepScreen::Play) {
                return (0..state.board.len())
                    .filter(|idx| Some(*idx) != state.first_pick)
                    .map(RunAction::EventChoice)
                    .collect();
            }
        }
        if let Some(ref event) = self.current_event {
            event
                .options
                .iter()
                .enumerate()
                .map(|(i, _)| RunAction::EventChoice(i))
                .collect()
        } else {
            Vec::new()
        }
    }

    // =======================================================================
    // Step — execute an action and return (reward, done)
    // =======================================================================

    /// Execute an action and return (reward, done).
    pub fn step(&mut self, action: &RunAction) -> (f32, bool) {
        let result = self.step_with_result(action);
        (result.reward, result.done)
    }

    /// Execute an action and return the stable post-step RL contract.
    pub fn step_with_result(&mut self, action: &RunAction) -> EngineStepResult {
        self.last_combat_events.clear();
        let legal_before = self.get_legal_actions();
        let action_accepted = legal_before.contains(action);
        let reward = if action_accepted {
            match self.phase {
                RunPhase::Neow => self.step_neow(action),
                RunPhase::MapChoice => self.step_map(action),
                RunPhase::Combat => self.step_combat(action),
                RunPhase::CardReward => self.step_card_reward(action),
                RunPhase::Campfire => self.step_campfire(action),
                RunPhase::Shop => self.step_shop(action),
                RunPhase::Event => self.step_event(action),
                RunPhase::GameOver => 0.0,
            }
        } else {
            0.0
        };

        self.refresh_decision_stack();
        self.total_reward += reward;
        let combat_obs_v2 = if self.phase == RunPhase::Combat {
            Some(crate::obs::encode_combat_state_v2(self).to_vec())
        } else {
            None
        };
        let combat_context = if self.phase == RunPhase::Combat {
            self.current_combat_context()
        } else {
            None
        };
        let decision_state = self.current_decision_state();
        let decision_context = self.current_decision_context();
        EngineStepResult {
            reward,
            done: self.run_state.run_over,
            phase: self.phase,
            action_accepted,
            legal_actions: self.get_legal_actions(),
            decision_state,
            decision_context,
            legal_decision_actions: self.get_legal_decision_actions(),
            combat_events: self.last_combat_events.clone(),
            combat_obs_v2,
            combat_obs_version: crate::obs::COMBAT_OBS_VERSION,
            combat_context,
        }
    }

    // =======================================================================
    // Map step
    // =======================================================================

    fn step_map(&mut self, action: &RunAction) -> f32 {
        let path_idx = match action {
            RunAction::ChoosePath(idx) => *idx,
            _ => return 0.0,
        };

        let (next_x, next_y, room_type) = if self.run_state.map_y < 0 {
            // First move: pick starting node
            let starts: Vec<_> = self.map.get_start_nodes().iter().map(|n| (n.x, n.y, n.room_type)).collect();
            if path_idx >= starts.len() {
                return 0.0;
            }
            starts[path_idx]
        } else {
            let x = self.run_state.map_x as usize;
            let y = self.run_state.map_y as usize;
            let next: Vec<_> = self.map.get_next_nodes(x, y).iter().map(|n| (n.x, n.y, n.room_type)).collect();
            if path_idx >= next.len() {
                return 0.0;
            }
            next[path_idx]
        };

        self.run_state.map_x = next_x as i32;
        self.run_state.map_y = next_y as i32;
        self.run_state.floor += 1;

        // Enter the room
        match room_type {
            RoomType::Monster => self.enter_combat(false, false),
            RoomType::Elite => self.enter_combat(true, false),
            RoomType::Rest => {
                self.phase = RunPhase::Campfire;
            }
            RoomType::Shop => {
                self.enter_shop();
            }
            RoomType::Event => {
                self.enter_event();
            }
            RoomType::Treasure => {
                if self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::MATRYOSHKA)
                    && self.run_state.relic_flags.counters
                        [crate::relic_flags::counter::MATRYOSHKA_USES]
                        > 0
                {
                    self.build_treasure_reward_screen();
                    self.phase = RunPhase::CardReward;
                } else {
                    // Gain gold + go to map
                    let gold = self.rng.gen_range(50..=80);
                    self.run_state.gold += gold;
                    self.phase = RunPhase::MapChoice;
                }
            }
            RoomType::Boss => {
                self.enter_combat(false, true);
            }
            RoomType::None => {
                self.phase = RunPhase::MapChoice;
            }
        }

        // Maw Bank: +12g per non-shop floor
        if room_type != RoomType::Shop
            && self.run_state.relic_flags.has(crate::relic_flags::flag::MAW_BANK)
            && !self.run_state.relic_flags.has(crate::relic_flags::flag::ECTOPLASM)
        {
            self.run_state.gold += 12;
        }

        // Floor milestone reward
        let floor_reward = self.run_state.floor as f32 / 55.0;
        floor_reward
    }

    fn step_neow(&mut self, action: &RunAction) -> f32 {
        let choice_idx = match action {
            RunAction::ChooseNeowOption(idx) => *idx,
            _ => return 0.0,
        };
        let Some(choice) = self.neow_options.get(choice_idx).cloned() else {
            return 0.0;
        };

        self.apply_neow_choice(choice.effect);
        self.neow_options.clear();
        self.phase = RunPhase::MapChoice;
        self.refresh_decision_stack();
        0.0
    }

    // =======================================================================
    // Combat
    // =======================================================================

    fn enter_combat(&mut self, is_elite: bool, is_boss: bool) {
        let act = self.run_state.act;
        let encounter = if is_boss {
            vec![self.boss_id.clone()]
        } else if is_elite {
            let pool = match act {
                2 => ACT2_ELITE_ENCOUNTERS,
                3 => ACT3_ELITE_ENCOUNTERS,
                4 => ACT4_ELITE_ENCOUNTERS,
                _ => ACT1_ELITE_ENCOUNTERS,
            };
            let idx = self.elite_encounter_idx % pool.len();
            self.elite_encounter_idx += 1;
            pool[idx].iter().map(|s| s.to_string()).collect()
        } else {
            // Weak encounters for early floors in the act, strong otherwise
            let act_floor = self.run_state.floor % 17; // relative floor within act
            let is_weak = act_floor <= 3;
            let pool = match (act, is_weak) {
                (2, true) => ACT2_WEAK_ENCOUNTERS,
                (2, false) => ACT2_STRONG_ENCOUNTERS,
                (3, true) => ACT3_WEAK_ENCOUNTERS,
                (3, false) => ACT3_STRONG_ENCOUNTERS,
                (_, true) => ACT1_WEAK_ENCOUNTERS,
                (_, false) => ACT1_STRONG_ENCOUNTERS,
            };
            let counter = if is_weak { &mut self.weak_encounter_idx } else { &mut self.strong_encounter_idx };
            let idx = *counter % pool.len();
            *counter += 1;
            pool[idx].iter().map(|s| s.to_string()).collect()
        };
        self.enter_specific_combat(encounter);
    }

    fn enter_specific_combat(&mut self, encounter: Vec<String>) {
        // Expand composite encounters (DonuAndDeca → two enemies)
        let expanded: Vec<String> = encounter.iter().flat_map(|id| {
            match id.as_str() {
                "DonuAndDeca" => vec!["Donu".to_string(), "Deca".to_string()],
                _ => vec![id.clone()],
            }
        }).collect();

        // Create enemies
        let mut enemy_states: Vec<EnemyCombatState> = expanded
            .iter()
            .map(|id| {
                let (hp, max_hp) = self.roll_enemy_hp(id);
                enemies::create_enemy(id, hp, max_hp)
            })
            .collect();

        // Sentry stagger: middle sentry starts on Beam, others on Bolt
        if expanded.len() == 3
            && expanded.iter().all(|id| id == "Sentry")
        {
            use crate::enemies::move_ids;
            enemy_states[1].set_move(move_ids::SENTRY_BEAM, 9, 1, 0);
            enemy_states[1].add_effect(crate::combat_types::mfx::DAZE, 2);
        }

        // Create combat state — convert deck strings to CardInstance
        let registry = crate::cards::global_registry();
        let deck_instances: Vec<crate::combat_types::CardInstance> = self.run_state.deck.iter()
            .map(|name| registry.make_card(name))
            .collect();
        let mut combat_state = CombatState::new(
            self.run_state.current_hp,
            self.run_state.max_hp,
            enemy_states,
            deck_instances,
            3, // Watcher base energy
        );
        combat_state.relics = self.run_state.relics.clone();
        combat_state.potions = self.run_state.potions.clone();
        combat_state.relic_counters = self.run_state.relic_flags.counters;

        let combat_seed = self.seed.wrapping_add(self.run_state.floor as u64 * 1000);
        let mut engine = CombatEngine::new(combat_state, combat_seed);
        engine.load_persisted_effects(self.run_state.persisted_effect_states.clone());
        engine.start_combat();

        self.combat_engine = Some(engine);
        self.phase = RunPhase::Combat;
        self.refresh_decision_stack();
    }

    fn roll_enemy_hp(&mut self, enemy_id: &str) -> (i32, i32) {
        let a20 = self.run_state.ascension >= 7;
        match enemy_id {
            "JawWorm" => {
                let hp = if a20 { 44 } else { 40 };
                (hp, hp)
            }
            "Cultist" => {
                let hp = if a20 { 50 } else { 48 };
                (hp, hp)
            }
            "FuzzyLouseNormal" | "FuzzyLouseDefensive" | "RedLouse" | "GreenLouse" => {
                let base = if a20 { 11 } else { 10 };
                let hp = base + self.rng.gen_range(0..=5);
                (hp, hp)
            }
            "AcidSlime_S" => {
                let hp = if a20 { 9 } else { 8 };
                (hp, hp)
            }
            "AcidSlime_M" => {
                let hp = if a20 { 32 } else { 28 };
                (hp, hp)
            }
            "AcidSlime_L" => {
                let hp = if a20 { 70 } else { 65 };
                (hp, hp)
            }
            "SpikeSlime_S" => {
                let hp = if a20 { 13 } else { 11 };
                (hp, hp)
            }
            "SpikeSlime_M" => {
                let hp = if a20 { 32 } else { 28 };
                (hp, hp)
            }
            "SpikeSlime_L" => {
                let hp = if a20 { 70 } else { 65 };
                (hp, hp)
            }
            "FungiBeast" => {
                let hp = if a20 { 24 } else { 22 };
                (hp, hp)
            }
            "BlueSlaver" | "SlaverBlue" => {
                let hp = if a20 { 48 } else { 46 };
                (hp, hp)
            }
            "RedSlaver" | "SlaverRed" => {
                let hp = if a20 { 48 } else { 46 };
                (hp, hp)
            }
            "GremlinNob" => {
                let hp = if a20 { 110 } else { 106 };
                (hp, hp)
            }
            "Lagavulin" => {
                let hp = if a20 { 112 } else { 109 };
                (hp, hp)
            }
            "Sentry" => {
                let hp = if a20 { 39 } else { 38 };
                (hp, hp)
            }
            "TheGuardian" => {
                let hp = if a20 { 250 } else { 240 };
                (hp, hp)
            }
            "Hexaghost" => {
                let hp = if a20 { 264 } else { 250 };
                (hp, hp)
            }
            "SlimeBoss" => {
                let hp = if a20 { 150 } else { 140 };
                (hp, hp)
            }
            // Act 2 enemies
            "Byrd" => {
                let hp = if a20 { 28 } else { 25 };
                (hp, hp)
            }
            "Chosen" => {
                let hp = if a20 { 99 } else { 95 };
                (hp, hp)
            }
            "ShelledParasite" => {
                let hp = if a20 { 73 } else { 68 };
                (hp, hp)
            }
            "SnakePlant" => {
                let hp = if a20 { 79 } else { 75 };
                (hp, hp)
            }
            "Centurion" => {
                let hp = if a20 { 80 } else { 76 };
                (hp, hp)
            }
            "Mystic" => {
                let hp = if a20 { 52 } else { 48 };
                (hp, hp)
            }
            "GremlinLeader" => {
                let hp = if a20 { 162 } else { 140 };
                (hp, hp)
            }
            "BookOfStabbing" => {
                let hp = if a20 { 168 } else { 160 };
                (hp, hp)
            }
            "TaskMaster" => {
                let hp = if a20 { 64 } else { 60 };
                (hp, hp)
            }
            "BronzeAutomaton" => {
                let hp = if a20 { 320 } else { 300 };
                (hp, hp)
            }
            "TheCollector" => {
                let hp = if a20 { 318 } else { 282 };
                (hp, hp)
            }
            "TheChamp" => {
                let hp = if a20 { 440 } else { 420 };
                (hp, hp)
            }
            // Act 3 enemies
            "Darkling" => {
                let hp = if a20 { 55 } else { 48 };
                (hp, hp)
            }
            "OrbWalker" => {
                let hp = if a20 { 100 } else { 90 };
                (hp, hp)
            }
            "Repulsor" => {
                let hp = if a20 { 36 } else { 29 };
                (hp, hp)
            }
            "WrithingMass" => {
                let hp = if a20 { 175 } else { 160 };
                (hp, hp)
            }
            "GiantHead" => {
                let hp = if a20 { 520 } else { 500 };
                (hp, hp)
            }
            "Nemesis" => {
                let hp = if a20 { 200 } else { 185 };
                (hp, hp)
            }
            "Reptomancer" => {
                let hp = if a20 { 210 } else { 190 };
                (hp, hp)
            }
            "Transient" => {
                let hp = if a20 { 1000 } else { 999 };
                (hp, hp)
            }
            "Maw" => {
                let hp = if a20 { 310 } else { 300 };
                (hp, hp)
            }
            "SpireGrowth" => {
                let hp = if a20 { 190 } else { 170 };
                (hp, hp)
            }
            "AwakenedOne" => {
                let hp = if a20 { 320 } else { 300 };
                (hp, hp)
            }
            "TimeEater" => {
                let hp = if a20 { 480 } else { 456 };
                (hp, hp)
            }
            "DonuAndDeca" | "Donu" | "Deca" => {
                let hp = if a20 { 282 } else { 250 };
                (hp, hp)
            }
            // Act 4 enemies
            "SpireShield" | "SpireSpear" => {
                let hp = if a20 { 220 } else { 200 };
                (hp, hp)
            }
            "CorruptHeart" => {
                let hp = if a20 { 800 } else { 750 };
                (hp, hp)
            }
            _ => (40, 40),
        }
    }

    fn step_combat(&mut self, action: &RunAction) -> f32 {
        let combat_action = match action {
            RunAction::CombatAction(a) => a.clone(),
            _ => return 0.0,
        };

        let engine = match self.combat_engine.as_mut() {
            Some(e) => e,
            None => return 0.0,
        };
        let combat_enemy_ids: Vec<String> = engine
            .state
            .enemies
            .iter()
            .map(|enemy| enemy.id.clone())
            .collect();

        engine.clear_event_log();
        let hp_before = engine.state.player.hp;
        engine.execute_action(&combat_action);

        let mut reward = 0.0;

        if engine.is_combat_over() {
            if engine.state.player_won {
                // Combat win reward
                reward += 1.0;

                // Self Repair: heal at end of combat
                let self_repair = engine.state.player.status(crate::status_ids::sid::SELF_REPAIR);
                if self_repair > 0 {
                    engine.state.heal_player(self_repair);
                }

                // Update run state from combat result
                self.run_state.current_hp = engine.state.player.hp;
                self.run_state.potions = engine.state.potions.clone();
                if engine.state.pending_run_gold > 0
                    && !self
                        .run_state
                        .relic_flags
                        .has(crate::relic_flags::flag::ECTOPLASM)
                {
                    self.run_state.gold += engine.state.pending_run_gold;
                }
                self.run_state.relic_flags.counters = engine.state.relic_counters;
                self.run_state.persisted_effect_states = engine.export_persisted_effects();
                self.run_state.combats_won += 1;
                self.last_combat_events = engine.take_event_log();

                if let Some(branch) = self.pending_event_combat.take() {
                    let mut reward_items = Vec::new();
                    let mut died = false;
                    let mut run_won = false;
                    let mut next_event = None;
                    let mut next_combat = None;
                    let mut start_boss_combat = false;
                    let mut start_final_act = false;
                    match self.apply_event_program(&branch.on_win, &mut reward_items) {
                        EventProgramFlow::Continue => {}
                        EventProgramFlow::ContinueEvent(event) => {
                            next_event = Some(event);
                        }
                        EventProgramFlow::Died => {
                            died = true;
                        }
                        EventProgramFlow::EndRunVictory => {
                            run_won = true;
                        }
                        EventProgramFlow::StartCombat(branch) => {
                            next_combat = Some(branch);
                        }
                        EventProgramFlow::StartBossCombat => {
                            start_boss_combat = true;
                        }
                        EventProgramFlow::StartFinalAct => {
                            start_final_act = true;
                        }
                    }

                    self.combat_engine = None;
                    if run_won {
                        self.resolve_terminal_run_victory();
                    } else if died {
                        reward -= 1.0;
                        self.run_state.current_hp = 0;
                        self.run_state.run_over = true;
                        self.phase = RunPhase::GameOver;
                    } else if let Some(event) = next_event {
                        self.current_event = Some(event);
                        self.phase = RunPhase::Event;
                    } else if let Some(branch) = next_combat {
                        self.current_event = None;
                        let resolved_enemies = self.resolve_event_combat_enemies(&branch.enemies);
                        self.pending_event_combat = Some(branch);
                        self.enter_specific_combat(resolved_enemies);
                    } else if start_boss_combat {
                        self.current_event = None;
                        self.pending_event_combat = None;
                        self.run_state.floor = 16;
                        self.enter_specific_combat(vec![self.boss_id.clone()]);
                    } else if start_final_act {
                        self.current_event = None;
                        self.pending_event_combat = None;
                        self.start_final_act();
                    } else if !reward_items.is_empty() {
                        self.build_event_reward_screen(reward_items);
                        self.phase = RunPhase::CardReward;
                    } else {
                        self.phase = RunPhase::MapChoice;
                    }
                    self.refresh_decision_stack();
                    return reward;
                }

                // Gold reward is an automatic combat outcome, not a selectable reward action.
                if !self.run_state.relic_flags.has(crate::relic_flags::flag::ECTOPLASM) {
                    let mut gold = self.rng.gen_range(10..=20);
                    if self.run_state.relic_flags.has(crate::relic_flags::flag::GOLDEN_IDOL) {
                        gold = (gold as f32 * 1.25) as i32;
                    }
                    self.run_state.gold += gold;
                }

                // Check if this was elite
                let room_type = if self.run_state.map_y >= 0 {
                    self.map.rows[self.run_state.map_y as usize][self.run_state.map_x as usize].room_type
                } else {
                    RoomType::Monster
                };

                if room_type == RoomType::Elite {
                    self.run_state.elites_killed += 1;
                    if !self.run_state.relic_flags.has(crate::relic_flags::flag::ECTOPLASM) {
                        let extra_gold = self.rng.gen_range(25..=35);
                        self.run_state.gold += extra_gold;
                    }
                }

                // Check if boss
                let is_boss = self.run_state.floor >= 16 || room_type == RoomType::Boss;
                if is_boss {
                    self.run_state.bosses_killed += 1;
                    reward += 5.0; // Boss kill bonus
                    if self.run_state.act == 4
                        && combat_enemy_ids.len() == 1
                        && combat_enemy_ids[0] == "CorruptHeart"
                    {
                        self.combat_engine = None;
                        self.resolve_terminal_run_victory();
                        return reward;
                    }
                    self.build_boss_reward_screen();
                    self.combat_engine = None;
                    self.phase = RunPhase::CardReward;
                    return reward;
                }

                // Build the ordered post-combat reward screen.
                self.build_combat_reward_screen(room_type);
                self.combat_engine = None;
                self.phase = RunPhase::CardReward;
            } else {
                // Player died
                reward -= 1.0;
                self.run_state.current_hp = 0;
                self.run_state.persisted_effect_states = engine.export_persisted_effects();
                self.last_combat_events = engine.take_event_log();
                self.run_state.run_over = true;
                self.pending_event_combat = None;
                self.combat_engine = None;
                self.phase = RunPhase::GameOver;
            }
        } else {
            // HP-based damage signal
            let hp_after = engine.state.player.hp;
            if hp_after < hp_before {
                let damage_ratio = (hp_before - hp_after) as f32 / self.run_state.max_hp as f32;
                reward -= damage_ratio * 0.5;
            }
            self.last_combat_events = engine.take_event_log();
        }

        reward
    }

    fn generate_card_reward_choices(&mut self, count: usize) -> Vec<RewardChoice> {
        let mut cards = Vec::new();
        for choice_index in 0..count {
            let roll: f32 = self.rng.gen();
            let card = if roll < 0.6 {
                // Common
                let idx = self.rng.gen_range(0..WATCHER_COMMON_CARDS.len());
                WATCHER_COMMON_CARDS[idx]
            } else if roll < 0.93 {
                // Uncommon
                let idx = self.rng.gen_range(0..WATCHER_UNCOMMON_CARDS.len());
                WATCHER_UNCOMMON_CARDS[idx]
            } else {
                // Rare
                let idx = self.rng.gen_range(0..WATCHER_RARE_CARDS.len());
                WATCHER_RARE_CARDS[idx]
            };
            cards.push(RewardChoice::Card {
                index: choice_index,
                card_id: card.to_string(),
            });
        }
        cards
    }

    fn build_combat_reward_screen(&mut self, room_type: RoomType) {
        let mut items = Vec::new();

        if room_type == RoomType::Elite {
            let relic_count = if self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::BLACK_STAR)
            {
                2
            } else {
                1
            };
            for reward_idx in 0..relic_count {
                items.push(RewardItem {
                    index: items.len(),
                    kind: RewardItemKind::Relic,
                    state: RewardItemState::Available,
                    label: self.roll_reward_relic_id(),
                    claimable: reward_idx == 0,
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: Vec::new(),
                });
            }
        }

        if self.should_offer_potion_reward(room_type) {
            items.push(RewardItem {
                index: items.len(),
                kind: RewardItemKind::Potion,
                state: RewardItemState::Available,
                label: self.roll_reward_potion_id(),
                claimable: items.is_empty(),
                active: false,
                skip_allowed: true,
                skip_label: Some("Skip".to_string()),
                choices: Vec::new(),
            });
        }

        let card_reward_count = if room_type == RoomType::Monster
            && self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::PRAYER_WHEEL)
        {
            2
        } else {
            1
        };
        let card_choice_count = if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::QUESTION_CARD)
        {
            4
        } else {
            3
        };

        for _ in 0..card_reward_count {
            items.push(RewardItem {
                index: items.len(),
                kind: RewardItemKind::CardChoice,
                state: RewardItemState::Available,
                label: "card_reward".to_string(),
                claimable: items.is_empty(),
                active: false,
                skip_allowed: true,
                skip_label: Some(if self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::SINGING_BOWL)
                {
                    "+2 Max HP".to_string()
                } else {
                    "Skip".to_string()
                }),
                choices: self.generate_card_reward_choices(card_choice_count),
            });
        }

        let mut screen = RewardScreen {
            source: if self.run_state.floor >= 16 {
                RewardScreenSource::BossCombat
            } else {
                RewardScreenSource::Combat
            },
            ordered: true,
            active_item: None,
            items,
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
    }

    fn build_boss_reward_screen(&mut self) {
        let choices = self
            .roll_boss_relic_choices(3)
            .into_iter()
            .enumerate()
            .map(|(index, relic_id)| RewardChoice::Named {
                index,
                label: relic_id,
            })
            .collect();

        let mut screen = RewardScreen {
            source: RewardScreenSource::BossCombat,
            ordered: true,
            active_item: None,
            items: vec![RewardItem {
                index: 0,
                kind: RewardItemKind::Relic,
                state: RewardItemState::Available,
                label: "boss_relic_reward".to_string(),
                claimable: true,
                active: false,
                skip_allowed: false,
                skip_label: None,
                choices,
            }],
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
    }

    fn build_treasure_reward_screen(&mut self) {
        let gold = self.rng.gen_range(50..=80);
        let extra_relic = self.run_state.relic_flags.counters
            [crate::relic_flags::counter::MATRYOSHKA_USES]
            > 0;
        if extra_relic {
            self.run_state.relic_flags.counters[crate::relic_flags::counter::MATRYOSHKA_USES] -= 1;
        }

        let mut items = vec![
            RewardItem {
                index: 0,
                kind: RewardItemKind::Gold,
                state: RewardItemState::Available,
                label: gold.to_string(),
                claimable: true,
                active: false,
                skip_allowed: false,
                skip_label: None,
                choices: Vec::new(),
            },
            RewardItem {
                index: 1,
                kind: RewardItemKind::Relic,
                state: RewardItemState::Available,
                label: self.roll_reward_relic_id(),
                claimable: false,
                active: false,
                skip_allowed: false,
                skip_label: None,
                choices: Vec::new(),
            },
        ];

        if extra_relic {
            items.push(RewardItem {
                index: items.len(),
                kind: RewardItemKind::Relic,
                state: RewardItemState::Available,
                label: self.roll_reward_relic_id(),
                claimable: false,
                active: false,
                skip_allowed: false,
                skip_label: None,
                choices: Vec::new(),
            });
        }

        let mut screen = RewardScreen {
            source: RewardScreenSource::Treasure,
            ordered: true,
            active_item: None,
            items,
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
    }

    // =======================================================================
    // Card reward step
    // =======================================================================

    fn step_card_reward(&mut self, action: &RunAction) -> f32 {
        match action {
            RunAction::SelectRewardItem(item_index) => self.select_reward_item(*item_index),
            RunAction::ChooseRewardOption {
                item_index,
                choice_index,
            } => self.choose_reward_option(*item_index, *choice_index),
            RunAction::SkipRewardItem(item_index) => self.skip_reward_item(*item_index),
            _ => {}
        }

        let reward_complete = self.reward_screen.as_ref().is_none_or(|screen| {
            screen
                .items
                .iter()
                .all(|item| item.state != RewardItemState::Available)
        });
        if reward_complete {
            let source = self
                .reward_screen
                .as_ref()
                .map(|screen| screen.source.clone())
                .unwrap_or(RewardScreenSource::Unknown);
            self.reward_screen = None;

            if source == RewardScreenSource::BossCombat {
                self.run_state.run_won = true;
                self.run_state.run_over = true;
                self.phase = RunPhase::GameOver;
                self.refresh_decision_stack();
                return 0.0;
            }

            // Check if at last row (floor 15) — enter boss
            if self.run_state.map_y >= 0 && self.run_state.map_y as usize >= self.map.height - 1 {
                // Boss fight next
                self.run_state.floor += 1;
                self.enter_combat(false, true);
                self.refresh_decision_stack();
                return 0.0;
            }

            self.phase = RunPhase::MapChoice;
        }
        self.refresh_decision_stack();
        0.0
    }

    fn select_reward_item(&mut self, item_index: usize) {
        let Some(screen) = self.reward_screen.as_ref() else {
            return;
        };
        if self.decision_stack.current_reward_choice().is_some() {
            return;
        }
        let Some(item) = screen.items.get(item_index) else {
            return;
        };
        if item.index != item_index || !item.claimable || item.state != RewardItemState::Available {
            return;
        }

        if !item.choices.is_empty() {
            self.decision_stack.push(DecisionFrame::RewardChoice(RewardChoiceFrame {
                item_index,
                item_kind: item.kind,
                skip_allowed: item.skip_allowed,
                choices: item.choices.clone(),
            }));
            return;
        }

        self.claim_reward_item(item_index);
    }

    fn choose_reward_option(&mut self, item_index: usize, choice_index: usize) {
        let Some(choice_frame) = self.decision_stack.current_reward_choice().cloned() else {
            return;
        };
        if choice_frame.item_index != item_index {
            return;
        }
        let Some(screen) = self.reward_screen.as_ref() else {
            return;
        };
        let Some(item) = screen.items.get(item_index) else {
            return;
        };
        if item.state != RewardItemState::Available {
            return;
        }
        let item_label = item.label.clone();
        let Some(choice) = choice_frame.choices.get(choice_index).cloned() else {
            return;
        };
        let kind = choice_frame.item_kind;

        match (kind, choice) {
            (RewardItemKind::CardChoice, RewardChoice::Card { card_id, .. }) => {
                if !self.resolve_event_deck_selection_choice(&item_label, &choice_frame, choice_index)
                {
                    self.add_card_reward(card_id);
                }
            }
            (RewardItemKind::Relic, RewardChoice::Named { label, .. }) => {
                self.add_relic_reward(&label);
            }
            (RewardItemKind::Potion, RewardChoice::Named { label, .. }) => {
                self.add_potion_reward(&label);
            }
            (RewardItemKind::Gold, RewardChoice::Named { label, .. }) => {
                if let Ok(amount) = label.parse::<i32>() {
                    self.run_state.gold += amount.max(0);
                }
            }
            (_, RewardChoice::Named { .. }) => {}
            _ => {}
        }

        if let Some(screen) = self.reward_screen.as_mut() {
            if let Some(item) = screen.items.get_mut(item_index) {
                item.state = RewardItemState::Claimed;
            }
            Self::refresh_reward_screen(screen);
        }
        self.decision_stack.pop();
    }

    fn resolve_event_deck_selection_choice(
        &mut self,
        label: &str,
        choice_frame: &RewardChoiceFrame,
        choice_index: usize,
    ) -> bool {
        let Some(RewardChoice::Card {
            index: deck_index,
            card_id,
        }) = choice_frame.choices.get(choice_index)
        else {
            return false;
        };

        match label {
            "deck_selection_purge" => {
                self.remove_master_deck_card(*deck_index);
                true
            }
            "deck_selection_bonfire_offer" => {
                self.remove_master_deck_card(*deck_index);
                self.resolve_bonfire_offer(card_id);
                true
            }
            "deck_selection_note_for_yourself" => {
                self.remove_master_deck_card(*deck_index);
                set_note_for_yourself_card(card_id.clone());
                true
            }
            _ => false,
        }
    }

    fn remove_master_deck_card(&mut self, deck_index: usize) -> Option<String> {
        if deck_index >= self.run_state.deck.len() {
            return None;
        }
        let removed = self.run_state.deck.remove(deck_index);
        self.apply_master_deck_removal_hook(&removed);
        Some(removed)
    }

    fn apply_master_deck_removal_hook(&mut self, card_id: &str) {
        if card_id == "Parasite" {
            self.run_state.max_hp = (self.run_state.max_hp - 3).max(1);
            self.run_state.current_hp = self.run_state.current_hp.min(self.run_state.max_hp);
        }
    }

    fn resolve_bonfire_offer(&mut self, card_id: &str) {
        match event_card_rarity(card_id) {
            Some(EventCardRarity::Curse) => {
                if self
                    .run_state
                    .relics
                    .iter()
                    .any(|relic| matches!(relic.as_str(), "Spirit Poop" | "SpiritPoop"))
                {
                    self.add_relic_reward("Circlet");
                } else {
                    self.add_relic_reward("Spirit Poop");
                }
            }
            Some(EventCardRarity::Basic) => {}
            Some(EventCardRarity::Common) | Some(EventCardRarity::Special) => {
                self.heal_run_player(5);
            }
            Some(EventCardRarity::Uncommon) => {
                let missing_hp = self.run_state.max_hp - self.run_state.current_hp;
                self.heal_run_player(missing_hp);
            }
            Some(EventCardRarity::Rare) => {
                self.run_state.max_hp += 10;
                self.run_state.current_hp = self.run_state.max_hp;
            }
            None => {}
        }
    }

    fn skip_reward_item(&mut self, item_index: usize) {
        let active_choice = self.decision_stack.current_reward_choice().cloned();
        let Some(screen) = self.reward_screen.as_ref() else {
            return;
        };
        let Some(item) = screen.items.get(item_index) else {
            return;
        };
        if item.index != item_index || !item.skip_allowed || item.state != RewardItemState::Available {
            return;
        }
        if let Some(choice_frame) = active_choice.as_ref() {
            if choice_frame.item_index != item_index || !choice_frame.skip_allowed {
                return;
            }
        } else if !item.claimable {
            return;
        }

        if item.kind == RewardItemKind::CardChoice
            && self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::SINGING_BOWL)
        {
            self.run_state.max_hp += 2;
            self.run_state.current_hp += 2;
        }

        if let Some(screen) = self.reward_screen.as_mut() {
            if let Some(item) = screen.items.get_mut(item_index) {
                item.state = RewardItemState::Skipped;
            }
            Self::refresh_reward_screen(screen);
        }
        if active_choice.is_some() {
            self.decision_stack.pop();
        }
    }

    fn claim_reward_item(&mut self, item_index: usize) {
        let Some(screen) = self.reward_screen.as_ref() else {
            return;
        };
        let Some(item) = screen.items.get(item_index) else {
            return;
        };
        let kind = item.kind;
        let label = item.label.clone();

        match kind {
            RewardItemKind::Relic => self.add_relic_reward(&label),
            RewardItemKind::Potion => self.add_potion_reward(&label),
            RewardItemKind::Gold => {
                if let Ok(amount) = label.parse::<i32>() {
                    self.run_state.gold += amount.max(0);
                }
            }
            _ => {}
        }

        if let Some(screen) = self.reward_screen.as_mut() {
            if let Some(item) = screen.items.get_mut(item_index) {
                item.state = RewardItemState::Claimed;
            }
            Self::refresh_reward_screen(screen);
        }
    }

    fn add_card_reward(&mut self, card_id: String) {
        let upgraded = self.upgrade_reward_card_if_needed(&card_id);
        self.run_state.deck.push(upgraded);
        // Ceramic Fish: +9g on card add
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::CERAMIC_FISH)
            && !self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::ECTOPLASM)
        {
            self.run_state.gold += 9;
        }
    }

    fn upgrade_reward_card_if_needed(&self, card_id: &str) -> String {
        if card_id.ends_with('+') {
            return card_id.to_string();
        }

        let registry = crate::cards::global_registry();
        let Some(def) = registry.get(card_id) else {
            return card_id.to_string();
        };

        let should_upgrade = match def.card_type {
            crate::cards::CardType::Attack => self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::MOLTEN_EGG),
            crate::cards::CardType::Skill => self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::TOXIC_EGG),
            crate::cards::CardType::Power => self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::FROZEN_EGG),
            _ => false,
        };
        if !should_upgrade {
            return card_id.to_string();
        }

        let upgraded = format!("{card_id}+");
        if registry.get(&upgraded).is_some() {
            upgraded
        } else {
            card_id.to_string()
        }
    }

    fn add_relic_reward(&mut self, relic_id: &str) {
        self.run_state.relics.push(relic_id.to_string());
        self.run_state.relic_flags.rebuild(&self.run_state.relics);
        self.run_state.relic_flags.init_relic_counter(relic_id);

        match relic_id {
            "Whetstone" => self.upgrade_random_cards_by_type(crate::cards::CardType::Attack, 2),
            "WarPaint" => self.upgrade_random_cards_by_type(crate::cards::CardType::Skill, 2),
            _ => {}
        }
    }

    fn remove_relic_reward(&mut self, relic_id: &str) {
        let matches = |owned: &str| match relic_id {
            "Golden Idol" | "GoldenIdol" => matches!(owned, "Golden Idol" | "GoldenIdol"),
            _ => owned == relic_id,
        };

        if let Some(index) = self
            .run_state
            .relics
            .iter()
            .position(|owned| matches(owned.as_str()))
        {
            self.run_state.relics.remove(index);
            self.run_state.relic_flags.rebuild(&self.run_state.relics);
        }
    }

    fn heal_run_player(&mut self, amount: i32) {
        if amount <= 0 {
            return;
        }
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::MARK_OF_BLOOM)
        {
            return;
        }

        let mut heal = amount;
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::MAGIC_FLOWER)
        {
            heal = (heal as f32 * 1.5) as i32;
        }

        self.run_state.current_hp = (self.run_state.current_hp + heal).min(self.run_state.max_hp);
    }

    fn add_potion_reward(&mut self, potion_id: &str) {
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::SOZU)
        {
            return;
        }
        if let Some(slot) = self.run_state.potions.iter().position(|p| p.is_empty()) {
            self.run_state.potions[slot] = potion_id.to_string();
        }
    }

    fn upgrade_random_cards_by_type(&mut self, card_type: crate::cards::CardType, count: usize) {
        let registry = crate::cards::global_registry();
        let mut eligible: Vec<usize> = self
            .run_state
            .deck
            .iter()
            .enumerate()
            .filter_map(|(idx, card_id)| {
                if card_id.ends_with('+') {
                    return None;
                }
                let def = registry.get(card_id)?;
                if def.card_type == card_type && registry.get(&format!("{card_id}+")).is_some() {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        for _ in 0..count {
            if eligible.is_empty() {
                break;
            }
            let pick = self.rng.gen_range(0..eligible.len());
            let deck_idx = eligible.swap_remove(pick);
            self.run_state.deck[deck_idx] = format!("{}+", self.run_state.deck[deck_idx]);
        }
    }

    fn roll_reward_relic_id(&mut self) -> String {
        const RELIC_REWARD_POOL: &[&str] = &[
            "Vajra",
            "Anchor",
            "BagOfMarbles",
            "QuestionCard",
            "PrayerWheel",
            "SingingBowl",
            "Whetstone",
            "WarPaint",
            "MoltenEgg2",
            "ToxicEgg2",
            "FrozenEgg2",
        ];

        let mut candidates: Vec<&str> = RELIC_REWARD_POOL
            .iter()
            .copied()
            .filter(|relic| !self.run_state.relics.iter().any(|owned| owned == relic))
            .collect();
        if candidates.is_empty() {
            candidates.extend(RELIC_REWARD_POOL.iter().copied());
        }

        let idx = self.rng.gen_range(0..candidates.len());
        candidates[idx].to_string()
    }

    fn roll_reward_potion_id(&mut self) -> String {
        const POTION_REWARD_POOL: &[&str] = &[
            "Block Potion",
            "Dexterity Potion",
            "Energy Potion",
            "Fire Potion",
            "Fear Potion",
            "Strength Potion",
            "Swift Potion",
            "Weak Potion",
        ];

        let registry = gameplay_registry();
        let candidates: Vec<&str> = POTION_REWARD_POOL
            .iter()
            .copied()
            .filter(|id| {
                registry.get(GameplayDomain::Potion, id).is_some()
                    || crate::potions::defs::potion_def_by_runtime_id(id).is_some()
            })
            .collect();
        if candidates.is_empty() {
            return "Block Potion".to_string();
        }
        let idx = self.rng.gen_range(0..candidates.len());
        candidates[idx].to_string()
    }

    fn roll_boss_relic_choices(&mut self, count: usize) -> Vec<String> {
        const BOSS_RELIC_POOL: &[&str] = &[
            "Philosopher's Stone",
            "Velvet Choker",
            "Snecko Eye",
        ];

        let registry = gameplay_registry();
        let mut candidates: Vec<&str> = BOSS_RELIC_POOL
            .iter()
            .copied()
            .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
            .filter(|id| !self.run_state.relics.iter().any(|owned| owned == id))
            .collect();
        if candidates.len() < count {
            candidates = BOSS_RELIC_POOL
                .iter()
                .copied()
                .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
                .collect();
        }
        if candidates.is_empty() {
            return Vec::new();
        }

        let mut picks = Vec::new();
        while !candidates.is_empty() && picks.len() < count {
            let idx = self.rng.gen_range(0..candidates.len());
            picks.push(candidates.swap_remove(idx).to_string());
        }
        picks
    }

    fn should_offer_potion_reward(&mut self, room_type: RoomType) -> bool {
        if room_type == RoomType::Boss {
            return false;
        }
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::SOZU)
        {
            return false;
        }
        if !self.run_state.potions.iter().any(|p| p.is_empty()) {
            return false;
        }
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::WHITE_BEAST)
        {
            return true;
        }
        if room_type == RoomType::Elite {
            return true;
        }
        self.rng.gen_bool(0.4)
    }

    fn can_enter_final_act(&self) -> bool {
        self.run_state.has_ruby_key
            && self.run_state.has_emerald_key
            && self.run_state.has_sapphire_key
    }

    fn resolve_terminal_run_victory(&mut self) {
        self.current_event = None;
        self.pending_event_combat = None;
        self.reward_screen = None;
        self.combat_engine = None;
        self.run_state.run_won = true;
        self.run_state.run_over = true;
        self.phase = RunPhase::GameOver;
    }

    fn start_final_act(&mut self) {
        self.run_state.act = 4;
        self.boss_id = "CorruptHeart".to_string();
        self.pending_event_combat = Some(PendingEventCombat {
            enemies: vec!["CorruptHeart".to_string()],
            on_win: crate::events::EventProgram::from_ops(vec![EventProgramOp::StartBossCombat]),
        });
        self.enter_specific_combat(vec![
            "SpireShield".to_string(),
            "SpireSpear".to_string(),
        ]);
    }

    fn refresh_reward_screen(screen: &mut RewardScreen) {
        for item in &mut screen.items {
            item.claimable = false;
            item.active = false;
        }

        if screen.ordered {
            if let Some(item) = screen
                .items
                .iter_mut()
                .find(|item| item.state == RewardItemState::Available)
            {
                item.claimable = true;
            }
        } else {
            for item in &mut screen.items {
                if item.state == RewardItemState::Available {
                    item.claimable = true;
                }
            }
        }
    }

    // =======================================================================
    // Campfire step
    // =======================================================================

    fn step_campfire(&mut self, action: &RunAction) -> f32 {
        match action {
            RunAction::CampfireRest => {
                if !self.run_state.relic_flags.has(crate::relic_flags::flag::MARK_OF_BLOOM) {
                    let mut heal = (self.run_state.max_hp as f32 * 0.3).ceil() as i32;
                    // Regal Pillow: +15 campfire heal
                    if self.run_state.relic_flags.has(crate::relic_flags::flag::REGAL_PILLOW) {
                        heal += 15;
                    }
                    // Magic Flower: 1.5x healing
                    if self.run_state.relic_flags.has(crate::relic_flags::flag::MAGIC_FLOWER) {
                        heal = (heal as f32 * 1.5) as i32;
                    }
                    self.run_state.current_hp = (self.run_state.current_hp + heal).min(self.run_state.max_hp);
                }
            }
            RunAction::CampfireUpgrade(idx) => {
                if *idx < self.run_state.deck.len() {
                    let card = &self.run_state.deck[*idx];
                    if !card.ends_with('+') {
                        let upgraded = format!("{}+", card);
                        self.run_state.deck[*idx] = upgraded;
                    }
                }
            }
            _ => {}
        }

        // Check if at last row — enter boss
        if self.run_state.map_y >= 0 && self.run_state.map_y as usize >= self.map.height - 1 {
            self.run_state.floor += 1;
            self.enter_combat(false, true);
            self.refresh_decision_stack();
            return 0.0;
        }

        self.phase = RunPhase::MapChoice;
        self.refresh_decision_stack();
        0.0
    }

    // =======================================================================
    // Shop step
    // =======================================================================

    fn enter_shop(&mut self) {
        // Generate shop cards (5 cards) and removal option
        let mut cards = Vec::new();
        for _ in 0..5 {
            let roll: f32 = self.rng.gen();
            let card = if roll < 0.5 {
                let idx = self.rng.gen_range(0..WATCHER_COMMON_CARDS.len());
                WATCHER_COMMON_CARDS[idx]
            } else if roll < 0.85 {
                let idx = self.rng.gen_range(0..WATCHER_UNCOMMON_CARDS.len());
                WATCHER_UNCOMMON_CARDS[idx]
            } else {
                let idx = self.rng.gen_range(0..WATCHER_RARE_CARDS.len());
                WATCHER_RARE_CARDS[idx]
            };
            let price = if roll < 0.5 {
                self.rng.gen_range(45..=80)
            } else if roll < 0.85 {
                self.rng.gen_range(68..=120)
            } else {
                self.rng.gen_range(135..=200)
            };
            // Membership Card: 50% shop discount
            let final_price = if self.run_state.relic_flags.has(crate::relic_flags::flag::MEMBERSHIP_CARD) {
                price / 2
            } else {
                price
            };
            cards.push((card.to_string(), final_price));
        }

        let mut remove_price = 75 + (self.run_state.combats_won as i32 * 25);
        // Smiling Mask: card removal always costs 50g
        if self.run_state.relic_flags.has(crate::relic_flags::flag::SMILING_MASK) {
            remove_price = 50;
        }
        // Membership Card discount on removal too
        if self.run_state.relic_flags.has(crate::relic_flags::flag::MEMBERSHIP_CARD) {
            remove_price /= 2;
        }

        self.current_shop = Some(ShopState {
            cards,
            remove_price,
            removal_used: false,
        });
        self.phase = RunPhase::Shop;
        self.refresh_decision_stack();

        // Meal Ticket: heal 15 on shop enter
        if self.run_state.relic_flags.has(crate::relic_flags::flag::MEAL_TICKET)
            && !self.run_state.relic_flags.has(crate::relic_flags::flag::MARK_OF_BLOOM)
        {
            let mut heal = 15;
            if self.run_state.relic_flags.has(crate::relic_flags::flag::MAGIC_FLOWER) {
                heal = (heal as f32 * 1.5) as i32;
            }
            self.run_state.current_hp = (self.run_state.current_hp + heal).min(self.run_state.max_hp);
        }
    }

    fn step_shop(&mut self, action: &RunAction) -> f32 {
        match action {
            RunAction::ShopBuyCard(idx) => {
                if let Some(ref mut shop) = self.current_shop {
                    if *idx < shop.cards.len() {
                        let (card, price) = shop.cards[*idx].clone();
                        if self.run_state.gold >= price {
                            self.run_state.gold -= price;
                            self.run_state.deck.push(card);
                            shop.cards.remove(*idx);
                        }
                    }
                }
                // Stay in shop for more purchases
                return 0.0;
            }
            RunAction::ShopRemoveCard(idx) => {
                let remove_price = self.current_shop.as_ref().and_then(|shop| {
                    if !shop.removal_used
                        && *idx < self.run_state.deck.len()
                        && Self::is_purgeable_master_deck_card(&self.run_state.deck[*idx])
                        && self.run_state.gold >= shop.remove_price
                    {
                        Some(shop.remove_price)
                    } else {
                        None
                    }
                });
                if let Some(price) = remove_price {
                    self.run_state.gold -= price;
                    self.remove_master_deck_card(*idx);
                    if let Some(shop) = self.current_shop.as_mut() {
                        shop.removal_used = true;
                    }
                }
                // Stay in shop
                return 0.0;
            }
            RunAction::ShopLeave => {}
            _ => {}
        }

        self.current_shop = None;
        self.phase = RunPhase::MapChoice;
        self.refresh_decision_stack();
        0.0
    }

    // =======================================================================
    // Event step
    // =======================================================================

    fn deck_has_attack_with_base_damage_at_least(&self, threshold: i32) -> bool {
        let registry = crate::cards::global_registry();
        self.run_state.deck.iter().any(|card_id| {
            registry.get(card_id.as_str()).is_some_and(|card| {
                card.card_type == crate::cards::CardType::Attack && card.base_damage >= threshold
            })
        })
    }

    fn normalize_event_runtime_statuses(&self, event: &mut TypedEventDef) {
        if event.name != "Golden Wing" {
            return;
        }

        if let Some(option) = event.options.get_mut(1) {
            if self.deck_has_attack_with_base_damage_at_least(10) {
                option.status = EventRuntimeStatus::Supported;
            } else {
                option.status = EventRuntimeStatus::Blocked {
                    reason: "requires a deck attack with base damage 10 or more".to_string(),
                };
            }
        }
    }

    fn normalize_event_runtime_state(&self, event: &mut TypedEventDef) {
        self.normalize_event_runtime_statuses(event);
        if event.name == "Dead Adventurer" {
            *event = crate::events::dead_adventurer_event(self.run_state.ascension);
        }
    }

    fn random_card_from_pool(&mut self, pool: &[&str]) -> String {
        let idx = self.rng.gen_range(0..pool.len());
        pool[idx].to_string()
    }

    fn build_match_and_keep_board(&mut self) -> Vec<String> {
        let mut unique_cards = vec![
            self.random_card_from_pool(WATCHER_RARE_CARDS),
            self.random_card_from_pool(WATCHER_UNCOMMON_CARDS),
            self.random_card_from_pool(WATCHER_COMMON_CARDS),
        ];
        if self.run_state.ascension >= 15 {
            unique_cards.push(self.random_card_from_pool(MATCH_AND_KEEP_CURSES));
            unique_cards.push(self.random_card_from_pool(MATCH_AND_KEEP_CURSES));
        } else {
            unique_cards.push(self.random_card_from_pool(MATCH_AND_KEEP_COLORLESS_UNCOMMON_CARDS));
            unique_cards.push(self.random_card_from_pool(MATCH_AND_KEEP_CURSES));
        }
        unique_cards.push("Eruption".to_string());

        let mut board = unique_cards.clone();
        board.extend(unique_cards);
        for idx in (1..board.len()).rev() {
            let swap_idx = self.rng.gen_range(0..=idx);
            board.swap(idx, swap_idx);
        }
        board
    }

    fn match_and_keep_label(&self, card_id: &str) -> String {
        gameplay_registry()
            .card(card_id)
            .map(|def| def.name.clone())
            .unwrap_or_else(|| card_id.to_string())
    }

    fn sync_match_and_keep_event(&mut self) {
        let Some(state) = &self.match_and_keep_state else {
            return;
        };

        let options = match state.screen {
            MatchAndKeepScreen::Intro => vec![crate::events::TypedEventOption::supported(
                "Hear the rules",
                crate::events::EventProgram::from_ops(vec![crate::events::EventProgramOp::nothing()]),
                crate::events::EventEffect::Nothing,
            )],
            MatchAndKeepScreen::RuleExplanation => {
                vec![crate::events::TypedEventOption::supported(
                    "Begin the match game",
                    crate::events::EventProgram::from_ops(vec![crate::events::EventProgramOp::nothing()]),
                    crate::events::EventEffect::Nothing,
                )]
            }
            MatchAndKeepScreen::Play => state
                .board
                .iter()
                .enumerate()
                .map(|(idx, card_id)| {
                    let label = if state.first_pick == Some(idx) {
                        format!(
                            "Revealed slot {}: {} ({} attempts left)",
                            idx + 1,
                            self.match_and_keep_label(card_id),
                            state.attempts_left
                        )
                    } else {
                        format!("Pick slot {} ({} attempts left)", idx + 1, state.attempts_left)
                    };
                    crate::events::TypedEventOption::supported(
                        label,
                        crate::events::EventProgram::from_ops(vec![crate::events::EventProgramOp::nothing()]),
                        crate::events::EventEffect::Nothing,
                    )
                })
                .collect(),
            MatchAndKeepScreen::Complete => vec![crate::events::TypedEventOption::supported(
                "Leave",
                crate::events::EventProgram::from_ops(vec![crate::events::EventProgramOp::nothing()]),
                crate::events::EventEffect::Nothing,
            )],
        };

        self.current_event = Some(TypedEventDef {
            name: "Match and Keep!".to_string(),
            options,
        });
    }

    fn sync_scrap_ooze_event(&mut self) {
        let Some(state) = &self.scrap_ooze_state else {
            return;
        };
        self.current_event = Some(TypedEventDef {
            name: "Scrap Ooze".to_string(),
            options: vec![
                crate::events::TypedEventOption::supported(
                    format!(
                        "Reach inside (take {} dmg, {}% relic chance)",
                        state.damage, state.relic_chance
                    ),
                    crate::events::EventProgram::from_ops(vec![crate::events::EventProgramOp::nothing()]),
                    crate::events::EventEffect::DamageAndGold(-state.damage, 0),
                ),
                crate::events::TypedEventOption::supported(
                    "Leave",
                    crate::events::EventProgram::from_ops(vec![crate::events::EventProgramOp::nothing()]),
                    crate::events::EventEffect::Nothing,
                ),
            ],
        });
    }

    fn next_event_roll_100(&mut self) -> usize {
        if !self.forced_event_rolls.is_empty() {
            return self.forced_event_rolls.remove(0).min(99);
        }
        self.rng.gen_range(0..100)
    }

    fn initialize_dynamic_event_state(&mut self) {
        self.match_and_keep_state = None;
        self.scrap_ooze_state = None;
        if let Some(event_name) = self.current_event.as_ref().map(|event| event.name.clone()) {
            if event_name == "Match and Keep!" {
                self.match_and_keep_state = Some(MatchAndKeepState {
                    screen: MatchAndKeepScreen::Intro,
                    board: self.build_match_and_keep_board(),
                    first_pick: None,
                    attempts_left: 5,
                });
                self.sync_match_and_keep_event();
            } else if event_name == "Scrap Ooze" {
                self.scrap_ooze_state = Some(ScrapOozeState {
                    damage: if self.run_state.ascension >= 15 { 5 } else { 3 },
                    relic_chance: 25,
                });
                self.sync_scrap_ooze_event();
            }
        }
    }

    fn enter_event(&mut self) {
        let events = typed_events_for_act(self.run_state.act);
        let idx = self.rng.gen_range(0..events.len());
        let mut event = events[idx].clone();
        self.normalize_event_runtime_state(&mut event);
        self.current_event = Some(event);
        self.initialize_dynamic_event_state();
        self.phase = RunPhase::Event;
        self.refresh_decision_stack();
    }

    fn step_match_and_keep(&mut self, choice_idx: usize) -> f32 {
        let Some(state) = self.match_and_keep_state.as_mut() else {
            return 0.0;
        };

        match state.screen {
            MatchAndKeepScreen::Intro => {
                state.screen = MatchAndKeepScreen::RuleExplanation;
                self.sync_match_and_keep_event();
                self.phase = RunPhase::Event;
                self.refresh_decision_stack();
                0.0
            }
            MatchAndKeepScreen::RuleExplanation => {
                state.screen = MatchAndKeepScreen::Play;
                self.sync_match_and_keep_event();
                self.phase = RunPhase::Event;
                self.refresh_decision_stack();
                0.0
            }
            MatchAndKeepScreen::Play => {
                if choice_idx >= state.board.len() || state.first_pick == Some(choice_idx) {
                    self.refresh_decision_stack();
                    return 0.0;
                }

                if let Some(first_idx) = state.first_pick.take() {
                    let first_card = state.board[first_idx].clone();
                    let second_card = state.board[choice_idx].clone();
                    if first_card == second_card {
                        self.run_state.deck.push(first_card);
                        let (hi, lo) = if first_idx > choice_idx {
                            (first_idx, choice_idx)
                        } else {
                            (choice_idx, first_idx)
                        };
                        state.board.remove(hi);
                        state.board.remove(lo);
                    }
                    state.attempts_left = state.attempts_left.saturating_sub(1);
                    if state.attempts_left == 0 || state.board.is_empty() {
                        state.screen = MatchAndKeepScreen::Complete;
                    }
                } else {
                    state.first_pick = Some(choice_idx);
                }

                self.sync_match_and_keep_event();
                self.phase = RunPhase::Event;
                self.refresh_decision_stack();
                0.0
            }
            MatchAndKeepScreen::Complete => {
                self.current_event = None;
                self.match_and_keep_state = None;
                self.phase = RunPhase::MapChoice;
                self.refresh_decision_stack();
                0.0
            }
        }
    }

    fn step_scrap_ooze(&mut self, choice_idx: usize) -> f32 {
        let Some(state_snapshot) = self.scrap_ooze_state.clone() else {
            return 0.0;
        };

        match choice_idx {
            0 => {
                self.run_state.current_hp = (self.run_state.current_hp - state_snapshot.damage).max(0);
                if self.run_state.current_hp <= 0 {
                    self.current_event = None;
                    self.scrap_ooze_state = None;
                    self.run_state.run_over = true;
                    self.phase = RunPhase::GameOver;
                    self.refresh_decision_stack();
                    return -1.0;
                }

                let threshold = 100usize.saturating_sub(state_snapshot.relic_chance.min(100));
                let roll = self.next_event_roll_100();
                if roll >= threshold {
                    let mut reward_items = Vec::new();
                    self.apply_event_reward(
                        &EventReward::Relic {
                            label: "random relic".to_string(),
                        },
                        &mut reward_items,
                    );
                    self.current_event = None;
                    self.scrap_ooze_state = None;
                    self.build_event_reward_screen(reward_items);
                    self.phase = RunPhase::CardReward;
                } else {
                    self.scrap_ooze_state = Some(ScrapOozeState {
                        damage: state_snapshot.damage + 1,
                        relic_chance: state_snapshot.relic_chance + 10,
                    });
                    self.sync_scrap_ooze_event();
                    self.phase = RunPhase::Event;
                }
                self.refresh_decision_stack();
                0.0
            }
            1 => {
                self.current_event = None;
                self.scrap_ooze_state = None;
                self.phase = RunPhase::MapChoice;
                self.refresh_decision_stack();
                0.0
            }
            _ => 0.0,
        }
    }

    fn step_event(&mut self, action: &RunAction) -> f32 {
        let choice_idx = match action {
            RunAction::EventChoice(idx) => *idx,
            _ => 0,
        };

        if self.scrap_ooze_state.is_some() {
            return self.step_scrap_ooze(choice_idx);
        }
        if self.match_and_keep_state.is_some() {
            return self.step_match_and_keep(choice_idx);
        }

        let Some(event) = self.current_event.clone() else {
            self.phase = RunPhase::MapChoice;
            self.refresh_decision_stack();
            return 0.0;
        };
        let Some(option) = event.options.get(choice_idx).cloned() else {
            self.current_event = None;
            self.phase = RunPhase::MapChoice;
            self.refresh_decision_stack();
            return 0.0;
        };

        let mut reward_items = Vec::new();
        let mut died = false;
        let mut run_won = false;
        let mut combat_branch = None;
        let mut next_event = None;
        let mut start_boss_combat = false;
        let mut start_final_act = false;
        let blocked = matches!(option.status, EventRuntimeStatus::Blocked { .. });

        if !blocked {
            match self.apply_event_program(&option.program, &mut reward_items) {
                EventProgramFlow::Continue => {}
                EventProgramFlow::ContinueEvent(event) => {
                    next_event = Some(event);
                }
                EventProgramFlow::Died => {
                    died = true;
                }
                EventProgramFlow::EndRunVictory => {
                    run_won = true;
                }
                EventProgramFlow::StartCombat(branch) => {
                    combat_branch = Some(branch);
                }
                EventProgramFlow::StartBossCombat => {
                    start_boss_combat = true;
                }
                EventProgramFlow::StartFinalAct => {
                    start_final_act = true;
                }
            }
        }

        if let Some(branch) = combat_branch {
            self.current_event = None;
            let resolved_enemies = self.resolve_event_combat_enemies(&branch.enemies);
            self.pending_event_combat = Some(branch.clone());
            self.enter_specific_combat(resolved_enemies);
            self.refresh_decision_stack();
            return 0.0;
        }

        if let Some(event) = next_event {
            self.current_event = Some(event);
            self.phase = RunPhase::Event;
            self.refresh_decision_stack();
            return 0.0;
        }

        if start_boss_combat {
            self.current_event = None;
            self.pending_event_combat = None;
            self.run_state.floor = 16;
            self.enter_specific_combat(vec![self.boss_id.clone()]);
            self.refresh_decision_stack();
            return 0.0;
        }

        if start_final_act {
            self.current_event = None;
            self.pending_event_combat = None;
            self.start_final_act();
            self.refresh_decision_stack();
            return 0.0;
        }

        self.current_event = None;
        if died {
            self.run_state.run_over = true;
            self.phase = RunPhase::GameOver;
            self.refresh_decision_stack();
            return -1.0;
        }

        if run_won {
            self.resolve_terminal_run_victory();
            self.refresh_decision_stack();
            return 0.0;
        }

        if !reward_items.is_empty() {
            self.build_event_reward_screen(reward_items);
            self.phase = RunPhase::CardReward;
        } else {
            self.phase = RunPhase::MapChoice;
        }
        self.refresh_decision_stack();
        0.0
    }

    fn apply_event_program_op(
        &mut self,
        op: &EventProgramOp,
        reward_items: &mut Vec<RewardItem>,
    ) -> EventProgramFlow {
        match op {
            EventProgramOp::ContinueEvent { event } => {
                EventProgramFlow::ContinueEvent((**event).clone())
            }
            EventProgramOp::ResolveFinalAct => {
                if self.can_enter_final_act() {
                    EventProgramFlow::StartFinalAct
                } else {
                    EventProgramFlow::EndRunVictory
                }
            }
            EventProgramOp::CombatBranch { enemies, on_win } => {
                EventProgramFlow::StartCombat(PendingEventCombat {
                    enemies: enemies.clone(),
                    on_win: *on_win.clone(),
                })
            }
            EventProgramOp::StartBossCombat => EventProgramFlow::StartBossCombat,
            EventProgramOp::RandomOutcomeTable { outcomes } => {
                if outcomes.is_empty() {
                    return EventProgramFlow::Continue;
                }
                let idx = self.rng.gen_range(0..outcomes.len());
                self.apply_event_program(&outcomes[idx], reward_items)
            }
            EventProgramOp::DeckSelection { label } => {
                if self.run_state.deck.is_empty() {
                    return EventProgramFlow::Continue;
                }
                let mut choices = self
                    .run_state
                    .deck
                    .iter()
                    .enumerate()
                    .map(|(index, card_id)| RewardChoice::Card {
                        index,
                        card_id: card_id.clone(),
                    })
                    .collect::<Vec<_>>();
                if label == "deck_selection_note_for_yourself" {
                    let reward_card = self.upgrade_reward_card_if_needed(&current_note_for_yourself_card());
                    choices.push(RewardChoice::Card {
                        index: choices.len(),
                        card_id: reward_card,
                    });
                }
                reward_items.push(RewardItem {
                    index: reward_items.len(),
                    kind: RewardItemKind::CardChoice,
                    state: RewardItemState::Available,
                    label: label.clone(),
                    claimable: reward_items.is_empty(),
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices,
                });
                EventProgramFlow::Continue
            }
            EventProgramOp::AdjustHp { amount } => {
                if *amount > 0 {
                    self.heal_run_player(*amount);
                } else {
                    self.run_state.current_hp =
                        (self.run_state.current_hp + amount).max(0).min(self.run_state.max_hp);
                }
                if self.run_state.current_hp <= 0 {
                    EventProgramFlow::Died
                } else {
                    EventProgramFlow::Continue
                }
            }
            EventProgramOp::AdjustHpByAscension { base, asc15 } => {
                let amount = if self.run_state.ascension >= 15 {
                    *asc15
                } else {
                    *base
                };
                if amount > 0 {
                    self.heal_run_player(amount);
                } else {
                    self.run_state.current_hp =
                        (self.run_state.current_hp + amount).max(0).min(self.run_state.max_hp);
                }
                if self.run_state.current_hp <= 0 {
                    EventProgramFlow::Died
                } else {
                    EventProgramFlow::Continue
                }
            }
            EventProgramOp::HealPercentHp { percent } => {
                let heal = (self.run_state.max_hp * percent) / 100;
                self.heal_run_player(heal);
                EventProgramFlow::Continue
            }
            EventProgramOp::AdjustHpPercentByAscension {
                heal,
                base_percent,
                asc15_percent,
            } => {
                let percent = if self.run_state.ascension >= 15 {
                    *asc15_percent
                } else {
                    *base_percent
                };
                let amount = (self.run_state.max_hp * percent) / 100;
                if *heal {
                    self.heal_run_player(amount);
                    EventProgramFlow::Continue
                } else {
                    self.run_state.current_hp = (self.run_state.current_hp - amount).max(0);
                    if self.run_state.current_hp <= 0 {
                        EventProgramFlow::Died
                    } else {
                        EventProgramFlow::Continue
                    }
                }
            }
            EventProgramOp::HealToFull => {
                let missing_hp = self.run_state.max_hp - self.run_state.current_hp;
                self.heal_run_player(missing_hp);
                EventProgramFlow::Continue
            }
            EventProgramOp::AdjustGold { amount } => {
                self.run_state.gold = (self.run_state.gold + amount).max(0);
                EventProgramFlow::Continue
            }
            EventProgramOp::AdjustGoldByAct {
                exordium,
                city,
                beyond,
            } => {
                let amount = match self.run_state.act {
                    2 => *city,
                    3 => *beyond,
                    _ => *exordium,
                };
                self.run_state.gold = (self.run_state.gold + amount).max(0);
                EventProgramFlow::Continue
            }
            EventProgramOp::AdjustMaxHp { amount } => {
                self.run_state.max_hp = (self.run_state.max_hp + amount).max(1);
                self.run_state.current_hp =
                    (self.run_state.current_hp + amount).max(0).min(self.run_state.max_hp);
                if self.run_state.current_hp <= 0 {
                    EventProgramFlow::Died
                } else {
                    EventProgramFlow::Continue
                }
            }
            EventProgramOp::AdjustMaxHpPercent { percent } => {
                let amount = if *percent >= 0 {
                    (self.run_state.max_hp * percent) / 100
                } else {
                    -((self.run_state.max_hp * (-percent)) / 100)
                };
                self.run_state.max_hp = (self.run_state.max_hp + amount).max(1);
                self.run_state.current_hp = self.run_state.current_hp.min(self.run_state.max_hp);
                if self.run_state.current_hp <= 0 {
                    EventProgramFlow::Died
                } else {
                    EventProgramFlow::Continue
                }
            }
            EventProgramOp::DamageAndGold { damage, gold } => {
                if *damage < 0 {
                    self.run_state.current_hp = (self.run_state.current_hp + damage).max(0);
                }
                self.run_state.gold = (self.run_state.gold + gold).max(0);
                if self.run_state.current_hp <= 0 {
                    EventProgramFlow::Died
                } else {
                    EventProgramFlow::Continue
                }
            }
            EventProgramOp::LosePercentHp { percent } => {
                let loss = ((self.run_state.max_hp * percent) / 100).max(1);
                self.run_state.current_hp = (self.run_state.current_hp - loss).max(0);
                if self.run_state.current_hp <= 0 {
                    EventProgramFlow::Died
                } else {
                    EventProgramFlow::Continue
                }
            }
            EventProgramOp::ResolveJoustBet { bet_on_owner } => {
                self.run_state.gold = (self.run_state.gold - 50).max(0);
                let owner_wins = self.rng.gen_bool(0.3);
                if (*bet_on_owner && owner_wins) || (!*bet_on_owner && !owner_wins) {
                    let payout = if *bet_on_owner { 250 } else { 100 };
                    self.run_state.gold += payout;
                }
                EventProgramFlow::Continue
            }
            EventProgramOp::RemoveRelic { label } => {
                self.remove_relic_reward(label);
                EventProgramFlow::Continue
            }
            EventProgramOp::DeckMutation(mutation) => {
                self.apply_event_deck_mutation(mutation);
                EventProgramFlow::Continue
            }
            EventProgramOp::Reward(reward) => {
                self.apply_event_reward(reward, reward_items);
                EventProgramFlow::Continue
            }
            EventProgramOp::Nothing | EventProgramOp::BlockedPlaceholder { .. } => {
                EventProgramFlow::Continue
            }
        }
    }


    fn apply_event_program(
        &mut self,
        program: &crate::events::EventProgram,
        reward_items: &mut Vec<RewardItem>,
    ) -> EventProgramFlow {
        for op in &program.ops {
            match self.apply_event_program_op(op, reward_items) {
                EventProgramFlow::Continue => {}
                other => return other,
            }
        }
        EventProgramFlow::Continue
    }

    fn apply_event_deck_mutation(&mut self, mutation: &EventDeckMutation) {
        match mutation {
            EventDeckMutation::GainCard { count } => {
                for _ in 0..*count {
                    let idx = self.rng.gen_range(0..WATCHER_COMMON_CARDS.len());
                    self.run_state.deck.push(WATCHER_COMMON_CARDS[idx].to_string());
                }
            }
            EventDeckMutation::RemoveCard { count } => {
                for _ in 0..*count {
                    if self.run_state.deck.len() > 5 {
                        let idx = self.rng.gen_range(0..self.run_state.deck.len());
                        self.remove_master_deck_card(idx);
                    }
                }
            }
            EventDeckMutation::TransformCard { count } => {
                for _ in 0..*count {
                    if self.run_state.deck.len() > 5 {
                        let idx = self.rng.gen_range(0..self.run_state.deck.len());
                        self.remove_master_deck_card(idx);
                    }
                    let card_idx = self.rng.gen_range(0..WATCHER_COMMON_CARDS.len());
                    self.run_state
                        .deck
                        .push(WATCHER_COMMON_CARDS[card_idx].to_string());
                }
            }
            EventDeckMutation::DuplicateCard { count } => {
                for _ in 0..*count {
                    if !self.run_state.deck.is_empty() {
                        let idx = self.rng.gen_range(0..self.run_state.deck.len());
                        let card = self.run_state.deck[idx].clone();
                        self.run_state.deck.push(card);
                    }
                }
            }
            EventDeckMutation::UpgradeCard { count } => {
                for _ in 0..*count {
                    if let Some(card) = self.run_state.deck.iter_mut().find(|card| !card.ends_with('+'))
                    {
                        *card = format!("{card}+");
                    }
                }
            }
        }
    }

    fn apply_event_reward(&mut self, reward: &EventReward, reward_items: &mut Vec<RewardItem>) {
        match reward {
            EventReward::Gold { amount } => {
                self.run_state.gold = (self.run_state.gold + amount).max(0);
            }
            EventReward::MaxHp { amount } => {
                self.run_state.max_hp = (self.run_state.max_hp + amount).max(1);
                self.run_state.current_hp =
                    (self.run_state.current_hp + amount).max(0).min(self.run_state.max_hp);
            }
            EventReward::Relic { label } => {
                let relic_id = self.resolve_event_relic_reward_label(label);
                reward_items.push(RewardItem {
                    index: reward_items.len(),
                    kind: RewardItemKind::Relic,
                    state: RewardItemState::Available,
                    label: relic_id,
                    claimable: reward_items.is_empty(),
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: Vec::new(),
                });
            }
            EventReward::Potion { count } => {
                for _ in 0..*count {
                    reward_items.push(RewardItem {
                        index: reward_items.len(),
                        kind: RewardItemKind::Potion,
                        state: RewardItemState::Available,
                        label: self.roll_reward_potion_id(),
                        claimable: reward_items.is_empty(),
                        active: false,
                        skip_allowed: false,
                        skip_label: None,
                        choices: Vec::new(),
                    });
                }
            }
            EventReward::Card { count } => {
                for _ in 0..*count {
                    reward_items.push(RewardItem {
                        index: reward_items.len(),
                        kind: RewardItemKind::CardChoice,
                        state: RewardItemState::Available,
                        label: "event_card_reward".to_string(),
                        claimable: reward_items.is_empty(),
                        active: false,
                        skip_allowed: false,
                        skip_label: None,
                        choices: self.generate_card_reward_choices(3),
                    });
                }
            }
            EventReward::StoredNoteCard => {
                let card_id = self.upgrade_reward_card_if_needed(&current_note_for_yourself_card());
                reward_items.push(RewardItem {
                    index: reward_items.len(),
                    kind: RewardItemKind::CardChoice,
                    state: RewardItemState::Available,
                    label: "event_stored_note_reward".to_string(),
                    claimable: reward_items.is_empty(),
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: vec![RewardChoice::Card { index: 0, card_id }],
                });
            }
            EventReward::SpecificCards { labels } => {
                reward_items.push(RewardItem {
                    index: reward_items.len(),
                    kind: RewardItemKind::CardChoice,
                    state: RewardItemState::Available,
                    label: "event_specific_card_reward".to_string(),
                    claimable: reward_items.is_empty(),
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: labels
                        .iter()
                        .enumerate()
                        .map(|(index, card_id)| RewardChoice::Card {
                            index,
                            card_id: card_id.clone(),
                        })
                        .collect(),
                });
            }
            EventReward::Curse { label } => {
                self.run_state.deck.push(label.clone());
            }
            EventReward::Nothing => {}
        }
    }

    fn resolve_event_combat_enemies(&mut self, enemies: &[String]) -> Vec<String> {
        if enemies.len() == 1 && enemies[0] == "MindBloomAct1Boss" {
            const ACT1_BOSS_POOL: &[&str] = &["TheGuardian", "Hexaghost", "SlimeBoss"];
            let idx = self.rng.gen_range(0..ACT1_BOSS_POOL.len());
            return vec![ACT1_BOSS_POOL[idx].to_string()];
        }

        enemies.to_vec()
    }

    fn resolve_event_relic_reward_label(&mut self, label: &str) -> String {
        match label {
            "random relic" => self.roll_reward_relic_id(),
            "rare relic" => self.roll_rare_event_relic_id(),
            "uncommon relic" => self.roll_uncommon_event_relic_id(),
            "Cursed Tome reward" => self.roll_cursed_tome_book_id(),
            other => other.to_string(),
        }
    }

    fn roll_uncommon_event_relic_id(&mut self) -> String {
        const UNCOMMON_EVENT_RELIC_POOL: &[&str] = &[
            "Anchor",
            "BagOfMarbles",
            "ClockworkSouvenir",
            "DataDisk",
            "HappyFlower",
            "Lantern",
            "OrnamentalFan",
            "ThreadAndNeedle",
        ];

        let registry = gameplay_registry();
        let mut candidates: Vec<&str> = UNCOMMON_EVENT_RELIC_POOL
            .iter()
            .copied()
            .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
            .filter(|id| !self.run_state.relics.iter().any(|owned| owned == id))
            .collect();
        if candidates.is_empty() {
            candidates = UNCOMMON_EVENT_RELIC_POOL
                .iter()
                .copied()
                .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
                .collect();
        }
        if candidates.is_empty() {
            return self.roll_reward_relic_id();
        }

        let idx = self.rng.gen_range(0..candidates.len());
        candidates[idx].to_string()
    }

    fn roll_rare_event_relic_id(&mut self) -> String {
        const RARE_EVENT_RELIC_POOL: &[&str] = &[
            "Calipers",
            "Ice Cream",
            "Incense Burner",
            "Tough Bandages",
            "Tungsten Rod",
        ];

        let registry = gameplay_registry();
        let mut candidates: Vec<&str> = RARE_EVENT_RELIC_POOL
            .iter()
            .copied()
            .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
            .filter(|id| !self.run_state.relics.iter().any(|owned| owned == id))
            .collect();
        if candidates.is_empty() {
            candidates = RARE_EVENT_RELIC_POOL
                .iter()
                .copied()
                .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
                .collect();
        }
        if candidates.is_empty() {
            return self.roll_reward_relic_id();
        }

        let idx = self.rng.gen_range(0..candidates.len());
        candidates[idx].to_string()
    }

    fn roll_cursed_tome_book_id(&mut self) -> String {
        const CURSED_TOME_BOOKS: &[&str] = &["Necronomicon", "Enchiridion", "Nilry's Codex"];

        let registry = gameplay_registry();
        let mut candidates: Vec<&str> = CURSED_TOME_BOOKS
            .iter()
            .copied()
            .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
            .filter(|id| !self.run_state.relics.iter().any(|owned| owned == id))
            .collect();
        if candidates.is_empty() {
            candidates = CURSED_TOME_BOOKS
                .iter()
                .copied()
                .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
                .collect();
        }
        if candidates.is_empty() {
            return "Circlet".to_string();
        }

        let idx = self.rng.gen_range(0..candidates.len());
        candidates[idx].to_string()
    }

    fn build_event_reward_screen(&mut self, mut items: Vec<RewardItem>) {
        for (index, item) in items.iter_mut().enumerate() {
            item.index = index;
            item.claimable = false;
            item.active = false;
        }

        let mut screen = RewardScreen {
            source: RewardScreenSource::Event,
            ordered: true,
            active_item: None,
            items,
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
    }

    // =======================================================================
    // Observation encoding
    // =======================================================================

    /// Get the current room type string.
    pub fn current_room_type(&self) -> &str {
        if self.phase == RunPhase::Neow {
            return "neow";
        }
        if self.run_state.map_y < 0 || self.run_state.map_x < 0 {
            return "none";
        }
        let x = self.run_state.map_x as usize;
        let y = self.run_state.map_y as usize;
        if y < self.map.height && x < self.map.width {
            self.map.rows[y][x].room_type.as_str()
        } else {
            "none"
        }
    }

    /// Get the currently visible card choices from the reward screen.
    pub fn get_card_rewards(&self) -> Vec<String> {
        self.reward_screen
            .as_ref()
            .and_then(|screen| {
                self.decision_stack
                    .current_reward_choice()
                    .and_then(|choice| screen.items.get(choice.item_index))
                    .or_else(|| {
                        screen
                            .items
                            .iter()
                            .find(|item| item.kind == RewardItemKind::CardChoice)
                    })
            })
            .map(|item| {
                item.choices
                    .iter()
                    .filter_map(|choice| match choice {
                        RewardChoice::Card { card_id, .. } => Some(card_id.clone()),
                        RewardChoice::Named { label, .. } => Some(label.clone()),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get event options count.
    pub fn event_option_count(&self) -> usize {
        self.current_event.as_ref().map_or(0, |e| e.options.len())
    }

    /// Get shop state for observation.
    pub fn get_shop(&self) -> Option<&ShopState> {
        self.current_shop.as_ref()
    }

    /// Get the combat engine reference (for combat observation encoding).
    pub fn get_combat_engine(&self) -> Option<&CombatEngine> {
        self.combat_engine.as_ref()
    }

    pub fn current_combat_context(&self) -> Option<CombatContext> {
        self.combat_engine.as_ref().map(build_combat_context)
    }

    pub fn current_reward_screen(&self) -> Option<RewardScreen> {
        if self.phase != RunPhase::CardReward {
            return None;
        }
        let mut screen = self.reward_screen.clone()?;
        screen.active_item = self
            .decision_stack
            .current_reward_choice()
            .map(|choice| choice.item_index);
        for item in &mut screen.items {
            item.active = screen.active_item == Some(item.index);
        }
        Some(screen)
    }

    pub fn decision_stack_depth(&self) -> usize {
        self.decision_stack.depth()
    }

    pub(crate) fn pending_event_combat_summary(&self) -> Option<String> {
        self.pending_event_combat
            .as_ref()
            .map(|branch| format!("{branch:?}"))
    }

    #[cfg(test)]
    pub(crate) fn debug_current_enemy_ids(&self) -> Vec<String> {
        self.combat_engine
            .as_ref()
            .map(|engine| {
                engine
                    .state
                    .enemies
                    .iter()
                    .map(|enemy| enemy.id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn current_reward_choice_count(&self) -> usize {
        self.decision_stack
            .current_reward_choice()
            .map(|choice| choice.choices.len())
            .unwrap_or(0)
    }

    pub fn current_choice_count(&self) -> usize {
        match self.phase {
            RunPhase::Neow => self.neow_options.len(),
            RunPhase::MapChoice => self.get_map_actions().len(),
            RunPhase::Combat => self
                .combat_engine
                .as_ref()
                .and_then(|combat| combat.choice.as_ref())
                .map(|choice| choice.options.len())
                .unwrap_or(0),
            RunPhase::CardReward => self.current_reward_choice_count(),
            RunPhase::Campfire => self.get_campfire_actions().len(),
            RunPhase::Shop => self.get_shop_actions().len(),
            RunPhase::Event => self.get_event_actions().len(),
            RunPhase::GameOver => 0,
        }
    }

    pub fn last_combat_events(&self) -> &[crate::effects::runtime::GameEventRecord] {
        &self.last_combat_events
    }

    #[cfg(test)]
    pub(crate) fn debug_set_card_reward_screen(&mut self, rewards: Vec<String>) {
        let mut screen = RewardScreen {
            source: RewardScreenSource::Combat,
            ordered: true,
            active_item: None,
            items: vec![RewardItem {
                index: 0,
                kind: RewardItemKind::CardChoice,
                state: RewardItemState::Available,
                label: "card_reward".to_string(),
                claimable: true,
                active: false,
                skip_allowed: true,
                skip_label: Some(if self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::SINGING_BOWL)
                {
                    "+2 Max HP".to_string()
                } else {
                    "Skip".to_string()
                }),
                choices: rewards
                    .into_iter()
                    .enumerate()
                    .map(|(index, card_id)| RewardChoice::Card { index, card_id })
                    .collect(),
            }],
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
        self.phase = RunPhase::CardReward;
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_set_reward_screen(&mut self, mut screen: RewardScreen) {
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
        self.phase = RunPhase::CardReward;
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_build_combat_reward_screen(&mut self, room_type: RoomType) {
        self.build_combat_reward_screen(room_type);
        self.phase = RunPhase::CardReward;
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_build_boss_reward_screen(&mut self) {
        self.build_boss_reward_screen();
        self.phase = RunPhase::CardReward;
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_build_treasure_reward_screen(&mut self) {
        self.build_treasure_reward_screen();
        self.phase = RunPhase::CardReward;
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_set_shop_state(&mut self, shop: ShopState) {
        self.current_shop = Some(shop);
        self.phase = RunPhase::Shop;
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_set_event_state(&mut self, event: crate::events::EventDef) {
        let mut event = TypedEventDef::from(event);
        self.normalize_event_runtime_state(&mut event);
        self.current_event = Some(event);
        self.initialize_dynamic_event_state();
        self.phase = RunPhase::Event;
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_set_typed_event_state(&mut self, event: TypedEventDef) {
        let mut event = event;
        self.normalize_event_runtime_state(&mut event);
        self.current_event = Some(event);
        self.initialize_dynamic_event_state();
        self.phase = RunPhase::Event;
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_current_event(&self) -> Option<TypedEventDef> {
        self.current_event.clone()
    }

    #[cfg(test)]
    pub(crate) fn debug_current_note_for_yourself_card(&self) -> String {
        current_note_for_yourself_card()
    }

    #[cfg(test)]
    pub(crate) fn debug_set_note_for_yourself_card(&mut self, card_id: &str) {
        set_note_for_yourself_card(card_id.to_string());
    }

    #[cfg(test)]
    pub(crate) fn debug_reset_note_for_yourself_card(&mut self) {
        set_note_for_yourself_card("IronWave".to_string());
    }

    #[cfg(test)]
    pub(crate) fn debug_clear_event_state(&mut self) {
        self.current_event = None;
        self.match_and_keep_state = None;
        self.scrap_ooze_state = None;
        self.phase = RunPhase::Event;
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_force_event_rolls(&mut self, rolls: &[usize]) {
        self.forced_event_rolls = rolls.to_vec();
    }

    #[cfg(test)]
    pub(crate) fn debug_match_and_keep_board(&self) -> Option<Vec<String>> {
        self.match_and_keep_state.as_ref().map(|state| state.board.clone())
    }

    #[cfg(test)]
    pub(crate) fn debug_match_and_keep_attempts_left(&self) -> Option<usize> {
        self.match_and_keep_state
            .as_ref()
            .map(|state| state.attempts_left)
    }

    #[cfg(test)]
    pub(crate) fn debug_force_current_combat_outcome(&mut self, player_won: bool) {
        let combat = self
            .combat_engine
            .as_mut()
            .expect("expected active combat to force outcome");
        combat.state.combat_over = true;
        combat.state.player_won = player_won;
    }

    #[cfg(test)]
    pub(crate) fn debug_resolve_current_combat_outcome(&mut self) -> f32 {
        self.step_combat(&RunAction::CombatAction(crate::actions::Action::EndTurn))
    }

    #[cfg(test)]
    pub(crate) fn debug_set_campfire_phase(&mut self) {
        self.phase = RunPhase::Campfire;
        self.refresh_decision_stack();
    }
}

fn event_card_rarity(card_id: &str) -> Option<EventCardRarity> {
    static CARD_RARITIES: OnceLock<HashMap<String, EventCardRarity>> = OnceLock::new();

    CARD_RARITIES
        .get_or_init(build_event_card_rarity_map)
        .get(card_id.trim_end_matches('+'))
        .copied()
}

fn build_event_card_rarity_map() -> HashMap<String, EventCardRarity> {
    let mut map = HashMap::new();
    for line in include_str!("../../engine/content/cards.py").lines() {
        let Some((_, rest)) = line.split_once("id=\"") else {
            continue;
        };
        let Some((card_id, _)) = rest.split_once('"') else {
            continue;
        };
        let Some((_, rarity_rest)) = line.split_once("rarity=CardRarity.") else {
            continue;
        };
        let rarity_token = rarity_rest
            .split(|ch: char| !matches!(ch, 'A'..='Z' | '_'))
            .next()
            .unwrap_or("");
        let rarity = match rarity_token {
            "CURSE" => EventCardRarity::Curse,
            "BASIC" => EventCardRarity::Basic,
            "COMMON" => EventCardRarity::Common,
            "SPECIAL" => EventCardRarity::Special,
            "UNCOMMON" => EventCardRarity::Uncommon,
            "RARE" => EventCardRarity::Rare,
            _ => continue,
        };
        map.insert(card_id.to_string(), rarity);
    }
    map
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------


#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventDef, EventEffect, EventOption};

    fn resolve_opening_neow(engine: &mut RunEngine) {
        if engine.current_phase() == RunPhase::Neow {
            let action = engine
                .current_decision_context()
                .neow
                .and_then(|neow| {
                    neow.options
                        .iter()
                        .position(|option| option.label == "Gain 100 gold")
                })
                .map(|idx| RunAction::ChooseNeowOption(idx))
                .unwrap_or_else(|| engine.get_legal_actions()[0].clone());
            let (reward, done) = engine.step(&action);
            assert_eq!(reward, 0.0);
            assert!(!done);
            assert_eq!(engine.current_phase(), RunPhase::MapChoice);
        }
    }

    #[test]
    fn test_run_engine_creation() {
        let engine = RunEngine::new(42, 20);
        assert_eq!(engine.run_state.current_hp, 68); // A14+ = 68
        assert_eq!(engine.run_state.deck.len(), 11); // 10 base + AscendersBane (A10+)
        assert_eq!(engine.phase, RunPhase::Neow);
        assert_eq!(engine.current_choice_count(), 4);
        assert!(!engine.is_done());
    }

    #[test]
    fn test_run_engine_reset() {
        let mut engine = RunEngine::new(42, 20);
        engine.run_state.current_hp = 10;
        engine.reset(99);
        assert_eq!(engine.run_state.current_hp, 68);
        assert_eq!(engine.seed, 99);
    }

    #[test]
    fn test_map_choice_actions() {
        let mut engine = RunEngine::new(42, 20);
        resolve_opening_neow(&mut engine);
        let actions = engine.get_legal_actions();
        assert!(!actions.is_empty(), "Should have map choice actions");
        for a in &actions {
            assert!(matches!(a, RunAction::ChoosePath(_)));
        }
    }

    #[test]
    fn test_first_floor_is_combat() {
        let mut engine = RunEngine::new(42, 20);
        resolve_opening_neow(&mut engine);
        let actions = engine.get_legal_actions();
        assert!(!actions.is_empty());

        // Take first path
        let (_, done) = engine.step(&actions[0]);
        assert!(!done);
        assert_eq!(engine.phase, RunPhase::Combat);
        assert_eq!(engine.run_state.floor, 1);
    }

    #[test]
    fn test_full_run_terminates() {
        // Run a full game with random actions — it should eventually terminate
        let mut engine = RunEngine::new(42, 20);
        let mut rng = crate::seed::StsRandom::new(42);
        let mut steps = 0;
        let max_steps = 50_000;

        while !engine.is_done() && steps < max_steps {
            let actions = engine.get_legal_actions();
            if actions.is_empty() {
                break;
            }
            let idx = rng.gen_range(0..actions.len());
            engine.step(&actions[idx]);
            steps += 1;
        }

        assert!(
            engine.is_done() || steps >= max_steps,
            "Run should terminate. Steps: {}, Floor: {}",
            steps,
            engine.run_state.floor
        );
    }

    #[test]
    fn test_campfire_heal() {
        let mut engine = RunEngine::new(42, 0);
        // Simulate entering a campfire
        engine.run_state.current_hp = 50;
        engine.run_state.max_hp = 72;
        engine.phase = RunPhase::Campfire;

        let actions = engine.get_legal_actions();
        assert!(actions.contains(&RunAction::CampfireRest));

        engine.step(&RunAction::CampfireRest);
        // Should heal 30% of 72 = 21.6 -> 22
        assert_eq!(engine.run_state.current_hp, 72); // 50 + 22 = 72 (capped at max)
    }

    #[test]
    fn test_campfire_upgrade() {
        let mut engine = RunEngine::new(42, 0);
        engine.phase = RunPhase::Campfire;
        engine.run_state.deck = vec![
            "Strike_P".to_string(),
            "Eruption".to_string(),
        ];

        engine.step(&RunAction::CampfireUpgrade(1));
        assert_eq!(engine.run_state.deck[1], "Eruption+");
    }

    #[test]
    fn test_card_reward_pick() {
        let mut engine = RunEngine::new(42, 0);
        engine.debug_set_card_reward_screen(vec![
            "Eruption".to_string(),
            "Vigilance".to_string(),
            "Tantrum".to_string(),
        ]);
        let deck_before = engine.run_state.deck.len();

        engine.step(&RunAction::SelectRewardItem(0));
        assert_eq!(
            engine.current_reward_screen().as_ref().and_then(|screen| screen.active_item),
            Some(0)
        );
        assert!(engine
            .get_legal_actions()
            .iter()
            .all(|action| matches!(action, RunAction::ChooseRewardOption { item_index: 0, .. } | RunAction::SkipRewardItem(0))));
        engine.step(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 1,
        });
        assert_eq!(engine.run_state.deck.len(), deck_before + 1);
        assert_eq!(engine.run_state.deck.last().unwrap(), "Vigilance");
    }

    #[test]
    fn test_card_reward_skip() {
        let mut engine = RunEngine::new(42, 0);
        engine.debug_set_card_reward_screen(vec![
            "Eruption".to_string(),
            "Vigilance".to_string(),
        ]);
        let deck_before = engine.run_state.deck.len();

        engine.step(&RunAction::SkipRewardItem(0));
        assert_eq!(engine.run_state.deck.len(), deck_before);
    }

    #[test]
    fn test_event_choices() {
        let mut engine = RunEngine::new(42, 0);
        engine.enter_event();
        assert_eq!(engine.phase, RunPhase::Event);

        let actions = engine.get_legal_actions();
        assert!(!actions.is_empty());
        assert!(actions.iter().all(|a| matches!(a, RunAction::EventChoice(_))));
    }

    #[test]
    fn test_shop_mechanics() {
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.gold = 500;
        engine.enter_shop();
        assert_eq!(engine.phase, RunPhase::Shop);

        let actions = engine.get_legal_actions();
        assert!(actions.contains(&RunAction::ShopLeave));

        // Leave shop
        engine.step(&RunAction::ShopLeave);
        assert_eq!(engine.phase, RunPhase::MapChoice);
    }

    #[test]
    fn test_a10_deck_contains_ascenders_bane() {
        // A10+ should have AscendersBane in starter deck
        let engine = RunEngine::new(42, 10);
        assert!(
            engine.run_state.deck.contains(&"AscendersBane".to_string()),
            "A10 deck should contain AscendersBane"
        );
        assert_eq!(engine.run_state.deck.len(), 11); // 10 base + 1 curse

        // Below A10 should not have it
        let engine_low = RunEngine::new(42, 9);
        assert!(
            !engine_low.run_state.deck.contains(&"AscendersBane".to_string()),
            "A9 deck should not contain AscendersBane"
        );
        assert_eq!(engine_low.run_state.deck.len(), 10);
    }

    #[test]
    fn test_golden_idol_costs_hp() {
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.max_hp = 72;
        engine.run_state.current_hp = 72;
        let gold_before = engine.run_state.gold;

        // Set up Golden Idol event manually
        engine.debug_set_event_state(EventDef {
            name: "Golden Idol".to_string(),
            options: vec![
                EventOption {
                    text: "Take".into(),
                    effect: EventEffect::GoldenIdolTake,
                },
                EventOption {
                    text: "Leave".into(),
                    effect: EventEffect::Nothing,
                },
            ],
        });

        engine.step(&RunAction::EventChoice(0));

        // Should lose 25% of 72 = 18 HP
        assert_eq!(engine.run_state.current_hp, 72 - 18);
        // Should gain 300 gold
        assert_eq!(engine.run_state.gold, gold_before + 300);
    }

    #[test]
    fn test_shop_removal_only_once() {
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.gold = 10000;
        engine.run_state.deck = vec![
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Defend_P".to_string(),
            "Defend_P".to_string(),
            "Defend_P".to_string(),
            "Eruption".to_string(),
            "Vigilance".to_string(),
            "Tantrum".to_string(),
        ];
        engine.enter_shop();

        // Should have removal options available
        let actions = engine.get_legal_actions();
        let has_remove = actions.iter().any(|a| matches!(a, RunAction::ShopRemoveCard(_)));
        assert!(has_remove, "Should offer card removal before first use");

        // Remove a card
        engine.step(&RunAction::ShopRemoveCard(0));
        assert_eq!(engine.run_state.deck.len(), 9);

        // After removal, should NOT have removal options
        let actions_after = engine.get_legal_actions();
        let has_remove_after = actions_after.iter().any(|a| matches!(a, RunAction::ShopRemoveCard(_)));
        assert!(!has_remove_after, "Should not offer card removal after first use");
    }
}
