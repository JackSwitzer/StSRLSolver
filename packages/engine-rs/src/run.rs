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
    /// Campfire: use Peace Pipe to open a one-card purge selection.
    CampfireToke,
    /// Campfire: use Girya to permanently add one lift, up to three.
    CampfireLift,
    /// Campfire: use Shovel to open a random relic reward.
    CampfireDig,
    /// Shop: buy a card (index into shop offerings)
    ShopBuyCard(usize),
    /// Shop: buy a relic (index into shop relic offerings)
    ShopBuyRelic(usize),
    /// Shop: remove a card (index into deck)
    ShopRemoveCard(usize),
    /// Shop: skip/leave shop
    ShopLeave,
    /// Event: choose an option (index)
    EventChoice(usize),
    /// Combat action: play card, use potion, or end turn
    CombatAction(crate::actions::Action),
    /// Use a potion from a run-level (non-combat) screen.
    UsePotion(usize),
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

/// Act 1 Watcher card pool for rewards.
/// Uses CardRegistry IDs (PascalCase, no spaces) to match card lookups.
/// Cards not in CardRegistry fall back to `get_or_default()` for defensive safety,
/// but the audited runtime now treats the Rust registry as canonical.
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

// CardLibrary.java::getCurse excludes Ascender's Bane, Necronomicurse,
// Curse of the Bell, and Pride from random curse generation.
const RANDOM_OBTAINABLE_CURSES: &[&str] = &[
    "Clumsy",
    "Decay",
    "Doubt",
    "Injury",
    "Normality",
    "Pain",
    "Parasite",
    "Regret",
    "Shame",
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
    /// Relics available for purchase: (relic_id, price)
    pub relics: Vec<(String, i32)>,
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
    #[serde(default)]
    pub bottled_flame_card: Option<String>,
    #[serde(default)]
    pub bottled_lightning_card: Option<String>,
    #[serde(default)]
    pub bottled_tornado_card: Option<String>,

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
    #[serde(default = "default_purge_cost")]
    pub purge_cost: i32,

    // Run outcome
    pub run_won: bool,
    pub run_over: bool,

    // Relic flags (bitfield for O(1) relic checks)
    #[serde(skip)]
    pub relic_flags: crate::relic_flags::RelicFlags,

    // Canonical owner-aware runtime state that persists across combats.
    #[serde(default)]
    pub persisted_effect_states: Vec<crate::effects::runtime::PersistedEffectState>,

    // LizardTail.java marks the relic used permanently after its one revive.
    #[serde(default)]
    pub lizard_tail_used: bool,

    // EventHelper.java mystery-room odds, stored as integer percentages.
    #[serde(default = "default_event_monster_chance")]
    pub event_monster_chance: i32,
    #[serde(default = "default_event_shop_chance")]
    pub event_shop_chance: i32,
    #[serde(default = "default_event_treasure_chance")]
    pub event_treasure_chance: i32,
}

fn default_purge_cost() -> i32 {
    75
}

fn default_event_monster_chance() -> i32 { 10 }
fn default_event_shop_chance() -> i32 { 3 }
fn default_event_treasure_chance() -> i32 { 2 }

fn adjust_run_gold_state(run_state: &mut RunState, amount: i32) {
    if amount > 0 {
        // AbstractPlayer.java::gainGold returns immediately while Ectoplasm is
        // owned. loseGold is a separate path, so negative adjustments remain valid.
        if run_state
            .relic_flags
            .has(crate::relic_flags::flag::ECTOPLASM)
        {
            return;
        }
        run_state.gold += amount;
        if run_state
            .relic_flags
            .has(crate::relic_flags::flag::BLOODY_IDOL)
            && !run_state
                .relic_flags
                .has(crate::relic_flags::flag::MARK_OF_BLOOM)
        {
            // MagicFlower.java only modifies healing during RoomPhase.COMBAT.
            run_state.current_hp = (run_state.current_hp + 5).min(run_state.max_hp);
        }
    } else if amount < 0 {
        run_state.gold = (run_state.gold + amount).max(0);
        // MawBank.java::onSpendGold permanently uses the relic up on any
        // actual gold-spending call, regardless of where the spend occurs.
        if run_state.relic_flags.has(crate::relic_flags::flag::MAW_BANK) {
            run_state.relic_flags.counters[crate::relic_flags::counter::MAW_BANK_GOLD] = -2;
        }
    }
}

fn upgrade_obtained_card_for_eggs(run_state: &RunState, card_id: &str) -> String {
    if card_id.ends_with('+') {
        return card_id.to_string();
    }
    let registry = crate::cards::global_registry();
    let Some(def) = registry.get(card_id) else {
        return card_id.to_string();
    };
    // FrozenEgg2.java, MoltenEgg2.java, and ToxicEgg2.java upgrade an
    // upgradeable Power, Attack, or Skill respectively in both preview and
    // obtain callbacks.
    let should_upgrade = match def.card_type {
        crate::cards::CardType::Attack => run_state
            .relic_flags
            .has(crate::relic_flags::flag::MOLTEN_EGG),
        crate::cards::CardType::Skill => run_state
            .relic_flags
            .has(crate::relic_flags::flag::TOXIC_EGG),
        crate::cards::CardType::Power => run_state
            .relic_flags
            .has(crate::relic_flags::flag::FROZEN_EGG),
        _ => false,
    };
    if should_upgrade {
        let upgraded = format!("{card_id}+");
        if registry.get(&upgraded).is_some() {
            return upgraded;
        }
    }
    card_id.to_string()
}

fn obtain_master_deck_card_state(run_state: &mut RunState, card_id: String) {
    let is_curse = crate::cards::global_registry()
        .get(&card_id)
        .is_some_and(|card| card.card_type == crate::cards::CardType::Curse);
    // ShowCardAndObtainEffect.java lets Omamori consume a charge and completes
    // without dispatching onObtainCard or adding the curse to the master deck.
    if is_curse
        && run_state
            .relic_flags
            .has(crate::relic_flags::flag::OMAMORI)
        && run_state.relic_flags.counters[crate::relic_flags::counter::OMAMORI_USES] > 0
    {
        run_state.relic_flags.counters[crate::relic_flags::counter::OMAMORI_USES] -= 1;
        return;
    }
    let card_id = upgrade_obtained_card_for_eggs(run_state, &card_id);
    run_state.deck.push(card_id);
    // DarkstonePeriapt.java calls increaseMaxHp(6, true) after an obtained
    // curse. AbstractCreature.java increases max HP, then heals through the
    // ordinary relic-modified healing path.
    if is_curse
        && run_state
            .relic_flags
            .has(crate::relic_flags::flag::DARKSTONE_PERIAPT)
    {
        run_state.max_hp += 6;
        if !run_state
            .relic_flags
            .has(crate::relic_flags::flag::MARK_OF_BLOOM)
        {
            // Card-obtain screens are outside combat; Magic Flower does not apply.
            run_state.current_hp = (run_state.current_hp + 6).min(run_state.max_hp);
        }
    }
    // Sources: CeramicFish.java::onObtainCard gains 9 gold for every obtained
    // card; ShowCardAndObtainEffect.java and FastCardObtainEffect.java dispatch
    // that hook for rewards, shops, event gains, duplicates, and transforms.
    if run_state
        .relic_flags
        .has(crate::relic_flags::flag::CERAMIC_FISH)
    {
        adjust_run_gold_state(run_state, 9);
    }
}

impl RunState {
    pub fn new(ascension: i32) -> Self {
        // Watcher starter deck
        let mut deck = vec![
            "Strike".to_string(),
            "Strike".to_string(),
            "Strike".to_string(),
            "Strike".to_string(),
            "Defend".to_string(),
            "Defend".to_string(),
            "Defend".to_string(),
            "Defend".to_string(),
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
            bottled_flame_card: None,
            bottled_lightning_card: None,
            bottled_tornado_card: None,
            map_x: -1,
            map_y: -1,
            has_ruby_key: false,
            has_emerald_key: false,
            has_sapphire_key: false,
            combats_won: 0,
            elites_killed: 0,
            bosses_killed: 0,
            purge_cost: default_purge_cost(),
            run_won: false,
            run_over: false,
            relic_flags,
            persisted_effect_states: Vec::new(),
            lizard_tail_used: false,
            event_monster_chance: default_event_monster_chance(),
            event_shop_chance: default_event_shop_chance(),
            event_treasure_chance: default_event_treasure_chance(),
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
    leave_only: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EventCardColor {
    Red,
    Green,
    Blue,
    Purple,
    Colorless,
    Curse,
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
    suspended_reward_screen: Option<RewardScreen>,
    pending_astrolabe_selection: bool,
    pending_astrolabe_removed: Vec<String>,
    pending_bottled_flame_selection: bool,
    pending_bottled_lightning_selection: bool,
    pending_bottled_tornado_selection: bool,
    pending_calling_bell_rewards: bool,

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
    fn compute_shop_remove_price(&self) -> i32 {
        // ShopScreen.java assigns Smiling Mask's flat cost after all other
        // purge-price discounts, so it always wins.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/shop/ShopScreen.java
        if self.run_state.relic_flags.has(crate::relic_flags::flag::SMILING_MASK) {
            return 50;
        }

        let mut remove_price = self.run_state.purge_cost;
        if self.run_state.relic_flags.has(crate::relic_flags::flag::THE_COURIER) {
            remove_price = ((remove_price as f32) * 0.8).round() as i32;
        }
        if self.run_state.relic_flags.has(crate::relic_flags::flag::MEMBERSHIP_CARD) {
            remove_price = ((remove_price as f32) * 0.5).round() as i32;
        }
        remove_price
    }

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
            suspended_reward_screen: None,
            pending_astrolabe_selection: false,
            pending_astrolabe_removed: Vec::new(),
            pending_bottled_flame_selection: false,
            pending_bottled_lightning_selection: false,
            pending_bottled_tornado_selection: false,
            pending_calling_bell_rewards: false,
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
            NeowChoiceEffect::GainGold => self.adjust_run_gold(100),
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
        let mut actions = match self.phase {
            RunPhase::Neow => self.get_neow_actions(),
            RunPhase::MapChoice => self.get_map_actions(),
            RunPhase::Combat => self.get_combat_actions(),
            RunPhase::CardReward => self.get_card_reward_actions(),
            RunPhase::Campfire => self.get_campfire_actions(),
            RunPhase::Shop => self.get_shop_actions(),
            RunPhase::Event => self.get_event_actions(),
            RunPhase::GameOver => Vec::new(),
        };
        if !matches!(self.phase, RunPhase::Combat | RunPhase::GameOver) {
            actions.extend(self.get_noncombat_potion_actions());
        }
        actions
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
                    removable_cards: self.peace_pipe_removable_indices(),
                    can_lift: self.can_lift_girya(),
                    can_dig: self.has_shovel(),
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
                removable_cards: self.peace_pipe_removable_indices(),
                can_lift: self.can_lift_girya(),
                can_dig: self.has_shovel(),
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

        // CoffeeDripper.java::canUseCampfireOption disables the exact
        // RestOption class while leaving other campfire options available.
        if !self.run_state.relic_flags.has(crate::relic_flags::flag::COFFEE_DRIPPER) {
            actions.push(RunAction::CampfireRest);
        }

        // FusionHammer.java::canUseCampfireOption disables the exact
        // SmithOption class while leaving Rest and other options available.
        if !self.run_state.relic_flags.has(crate::relic_flags::flag::FUSION_HAMMER) {
            for (i, card) in self.run_state.deck.iter().enumerate() {
                if !card.ends_with('+') {
                    actions.push(RunAction::CampfireUpgrade(i));
                }
            }
        }

        if !self.peace_pipe_removable_indices().is_empty() {
            actions.push(RunAction::CampfireToke);
        }

        if self.can_lift_girya() {
            actions.push(RunAction::CampfireLift);
        }

        if self.has_shovel() {
            actions.push(RunAction::CampfireDig);
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
            for (i, (_, price)) in shop.relics.iter().enumerate() {
                if self.run_state.gold >= *price {
                    actions.push(RunAction::ShopBuyRelic(i));
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

    fn peace_pipe_removable_indices(&self) -> Vec<usize> {
        if !self.run_state.relics.iter().any(|relic| relic == "Peace Pipe") {
            return Vec::new();
        }
        let bottled = [
            self.run_state.bottled_flame_card.as_deref(),
            self.run_state.bottled_lightning_card.as_deref(),
            self.run_state.bottled_tornado_card.as_deref(),
        ];
        self.run_state
            .deck
            .iter()
            .enumerate()
            .filter_map(|(index, card)| {
                (Self::is_purgeable_master_deck_card(card)
                    && !bottled.contains(&Some(card.as_str())))
                .then_some(index)
            })
            .collect()
    }

    fn can_lift_girya(&self) -> bool {
        self.run_state.relics.iter().any(|relic| relic == "Girya")
            && self.run_state.relic_flags.counters[crate::relic_flags::counter::GIRYA] < 3
    }

    fn has_shovel(&self) -> bool {
        self.run_state.relics.iter().any(|relic| relic == "Shovel")
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

    fn get_noncombat_potion_actions(&self) -> Vec<RunAction> {
        // FruitJuice.canUse permits every non-combat screen except WeMeetAgain.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FruitJuice.java
        if self.phase == RunPhase::Event
            && self
                .current_event
                .as_ref()
                .is_some_and(|event| event.name == "WeMeetAgain")
        {
            return Vec::new();
        }

        self.run_state
            .potions
            .iter()
            .enumerate()
            .filter_map(|(idx, potion)| {
                matches!(potion.as_str(), "Fruit Juice" | "FruitJuice")
                    .then_some(RunAction::UsePotion(idx))
            })
            .collect()
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
            if let RunAction::UsePotion(potion_idx) = action {
                self.step_noncombat_potion(*potion_idx)
            } else {
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

    fn step_noncombat_potion(&mut self, potion_idx: usize) -> f32 {
        let Some(potion_id) = self.run_state.potions.get(potion_idx).cloned() else {
            return 0.0;
        };
        match potion_id.as_str() {
            "Fruit Juice" | "FruitJuice" => {
                let amount = if self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::SACRED_BARK)
                {
                    10
                } else {
                    5
                };
                // AbstractCreature.increaseMaxHp raises maxHealth before heal.
                // MagicFlower.java does not modify this out-of-combat heal.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FruitJuice.java
                // Java: decompiled/java-src/com/megacrit/cardcrawl/core/AbstractCreature.java
                self.run_state.max_hp += amount;
                self.heal_run_player(amount);
                self.run_state.potions[potion_idx].clear();
            }
            _ => {}
        }
        0.0
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
            RoomType::Rest => self.enter_campfire(),
            RoomType::Shop => {
                self.enter_shop();
            }
            RoomType::Event => {
                self.enter_mystery_room();
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
                    self.adjust_run_gold(gold);
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

        // MawBank.java::onEnterRoom triggers for every room, including shops,
        // until onSpendGold marks the relic used up.
        if self.run_state.relic_flags.has(crate::relic_flags::flag::MAW_BANK)
            && self.run_state.relic_flags.counters
                [crate::relic_flags::counter::MAW_BANK_GOLD]
                != -2
        {
            self.adjust_run_gold(12);
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

        // Source: reference/extracted/methods/monster/JawWorm.java (constructor).
        // Ascension is only known here, so patch the fields and opening CHOMP.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "JawWorm") {
            let chomp_damage = if self.run_state.ascension >= 2 { 12 } else { 11 };
            let strength = if self.run_state.ascension >= 17 { 5 }
                else if self.run_state.ascension >= 2 { 4 } else { 3 };
            let bellow_block = if self.run_state.ascension >= 17 { 9 } else { 6 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, chomp_damage);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, strength);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT, bellow_block);
            enemy.set_move(crate::enemies::move_ids::JW_CHOMP, chomp_damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/FungiBeast.java.
        // Grow grants 3 Strength, 4 at A2, and one additional point at A17.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "FungiBeast") {
            let strength = if self.run_state.ascension >= 17 { 5 }
                else if self.run_state.ascension >= 2 { 4 } else { 3 };
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, strength);
        }

        // Sources: reference/extracted/methods/monster/LouseNormal.java and
        // LouseDefensive.java. HP is rolled separately below; constructor Bite
        // and pre-battle Curl Up also use inclusive monsterHpRng ranges.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "FuzzyLouseNormal" | "RedLouse" | "FuzzyLouseDefensive" | "GreenLouse")) {
            let bite_base = if self.run_state.ascension >= 2 { 6 } else { 5 };
            let bite_damage = bite_base + self.rng.gen_range(0..=2);
            let (curl_base, curl_width) = if self.run_state.ascension >= 17 {
                (9, 3)
            } else if self.run_state.ascension >= 7 {
                (4, 4)
            } else {
                (3, 4)
            };
            let curl_up = curl_base + self.rng.gen_range(0..=curl_width);
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, bite_damage);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 17 { 4 } else { 3 });
            enemy.entity.set_status(crate::status_ids::sid::CURL_UP, curl_up);
            enemy.set_move(crate::enemies::move_ids::LOUSE_BITE, bite_damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/SlaverBlue.java.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "SlaverBlue" | "BlueSlaver")) {
            let (stab, rake) = if self.run_state.ascension >= 2 { (13, 8) } else { (12, 7) };
            let weak = if self.run_state.ascension >= 17 { 2 } else { 1 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, stab);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, rake);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT, weak);
            enemy.set_move(crate::enemies::move_ids::BS_STAB, stab, 1, 0);
        }

        // Source: reference/extracted/methods/monster/SlaverRed.java.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "SlaverRed" | "RedSlaver")) {
            let (stab, scrape) = if self.run_state.ascension >= 2 { (14, 9) } else { (13, 8) };
            let vulnerable = if self.run_state.ascension >= 17 { 2 } else { 1 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, stab);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, scrape);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT, vulnerable);
            enemy.entity.set_status(crate::status_ids::sid::IS_FIRST_MOVE, 1);
            enemy.set_move(crate::enemies::move_ids::RS_STAB, stab, 1, 0);
        }

        // Source: reference/extracted/methods/monster/AcidSlime_S.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "AcidSlime_S") {
            let damage = if self.run_state.ascension >= 2 { 4 } else { 3 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 17 { 17 } else { 0 });
            enemy.set_move(crate::enemies::move_ids::AS_S_TACKLE, damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/AcidSlime_M.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "AcidSlime_M") {
            let (wound, normal) = if self.run_state.ascension >= 2 { (8, 12) } else { (7, 10) };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, wound);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, normal);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 17 } else { 0 });
            enemy.set_move(crate::enemies::move_ids::AS_CORROSIVE_SPIT, wound, 1, 0);
            enemy.add_effect(crate::combat_types::mfx::SLIMED, 1);
        }

        // Source: reference/extracted/methods/monster/AcidSlime_L.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "AcidSlime_L") {
            let (wound, normal) = if self.run_state.ascension >= 2 { (12, 18) } else { (11, 16) };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, wound);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, normal);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 17 } else { 0 });
            enemy.set_move(crate::enemies::move_ids::AS_CORROSIVE_SPIT, wound, 1, 0);
            enemy.add_effect(crate::combat_types::mfx::SLIMED, 2);
        }

        // Source: reference/extracted/methods/monster/SpikeSlime_S.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "SpikeSlime_S") {
            let damage = if self.run_state.ascension >= 2 { 6 } else { 5 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.set_move(crate::enemies::move_ids::SS_TACKLE, damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/SpikeSlime_M.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "SpikeSlime_M") {
            let damage = if self.run_state.ascension >= 2 { 10 } else { 8 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 17 } else { 0 });
            enemy.set_move(crate::enemies::move_ids::SS_TACKLE, damage, 1, 0);
            enemy.add_effect(crate::combat_types::mfx::SLIMED, 1);
        }

        // Source: reference/extracted/methods/monster/SpikeSlime_L.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "SpikeSlime_L") {
            let damage = if self.run_state.ascension >= 2 { 18 } else { 16 };
            let frail = if self.run_state.ascension >= 17 { 3 } else { 2 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, frail);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 17 } else { 0 });
            enemy.set_move(crate::enemies::move_ids::SS_TACKLE, damage, 1, 0);
            enemy.add_effect(crate::combat_types::mfx::SLIMED, 2);
        }

        // Source: reference/extracted/methods/monster/Looter.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Looter") {
            let (swipe, lunge) = if self.run_state.ascension >= 2 {
                (11, 14)
            } else {
                (10, 12)
            };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, swipe);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, lunge);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT, 6);
            enemy.entity.set_status(crate::status_ids::sid::TURN_COUNT,
                if self.run_state.ascension >= 17 { 20 } else { 15 });
            enemy.set_move(crate::enemies::move_ids::LOOTER_MUG, swipe, 1, 0);
        }

        // Source: reference/extracted/methods/monster/GremlinFat.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "GremlinFat") {
            let damage = if self.run_state.ascension >= 2 { 5 } else { 4 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 17 } else { 0 });
            enemy.set_move(crate::enemies::move_ids::GREMLIN_FAT_SMASH, damage, 1, 0);
            enemy.add_effect(crate::combat_types::mfx::WEAK, 1);
            if self.run_state.ascension >= 17 {
                enemy.add_effect(crate::combat_types::mfx::FRAIL, 1);
            }
        }

        // Source: reference/extracted/methods/monster/GremlinThief.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "GremlinThief") {
            let damage = if self.run_state.ascension >= 2 { 10 } else { 9 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.set_move(crate::enemies::move_ids::GREMLIN_ATTACK, damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/GremlinWarrior.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "GremlinWarrior") {
            let damage = if self.run_state.ascension >= 2 { 5 } else { 4 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(crate::status_ids::sid::ANGRY,
                if self.run_state.ascension >= 17 { 2 } else { 1 });
            enemy.set_move(crate::enemies::move_ids::GREMLIN_ATTACK, damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/GremlinWizard.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "GremlinWizard") {
            let damage = if self.run_state.ascension >= 2 { 30 } else { 25 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(crate::status_ids::sid::COUNT, 1);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 17 } else { 0 });
            enemy.set_move(crate::enemies::move_ids::GREMLIN_PROTECT, 0, 0, 0);
        }

        // Source: reference/extracted/methods/monster/GremlinTsundere.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "GremlinTsundere") {
            let damage = if self.run_state.ascension >= 2 { 8 } else { 6 };
            let block = if self.run_state.ascension >= 17 { 11 }
                else if self.run_state.ascension >= 7 { 8 } else { 7 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT, block);
            enemy.set_move(crate::enemies::move_ids::GREMLIN_TSUNDERE_PROTECT,
                0, 0, 0);
            enemy.add_effect(crate::combat_types::mfx::BLOCK_RANDOM_OTHER,
                block as i16);
        }

        // Source: reference/extracted/methods/monster/GremlinNob.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "GremlinNob") {
            let (rush, bash) = if self.run_state.ascension >= 3 {
                (16, 8)
            } else {
                (14, 6)
            };
            let enrage = if self.run_state.ascension >= 18 { 3 } else { 2 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, rush);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, bash);
            enemy.entity.set_status(crate::status_ids::sid::TURN_COUNT, enrage);
            enemy.entity.set_status(crate::status_ids::sid::IS_FIRST_MOVE, 0);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 18 { 18 } else { 0 });
            enemy.entity.set_status(crate::status_ids::sid::ENRAGE, 0);
            enemy.set_move(crate::enemies::move_ids::NOB_BELLOW, 0, 0, 0);
            enemy.add_effect(crate::combat_types::mfx::ENRAGE, enrage as i16);
        }

        // Source: reference/extracted/methods/monster/Lagavulin.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Lagavulin") {
            let damage = if self.run_state.ascension >= 3 { 20 } else { 18 };
            let debuff = if self.run_state.ascension >= 18 { 2 } else { 1 };
            enemy.entity.block = 8;
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, debuff);
            enemy.entity.set_status(crate::status_ids::sid::IS_FIRST_MOVE, 0);
            enemy.entity.set_status(crate::status_ids::sid::COUNT, 0);
            enemy.entity.set_status(crate::status_ids::sid::ATTACK_COUNT, 0);
            enemy.entity.set_status(crate::status_ids::sid::SLEEP_TURNS, 1);
            enemy.entity.set_status(crate::status_ids::sid::METALLICIZE, 8);
            enemy.set_move(crate::enemies::move_ids::LAGA_SLEEP, 0, 0, 0);
        }

        // Source: reference/extracted/methods/monster/Sentry.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Sentry") {
            let damage = if self.run_state.ascension >= 3 { 10 } else { 9 };
            let daze = if self.run_state.ascension >= 18 { 3 } else { 2 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, daze);
            enemy.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 0);
            enemy.entity.set_status(crate::status_ids::sid::IS_FIRST_MOVE,
                crate::enemies::move_ids::SENTRY_BOLT);
            enemy.entity.set_status(crate::status_ids::sid::ARTIFACT, 1);
            enemy.set_move(crate::enemies::move_ids::SENTRY_BOLT, 0, 0, 0);
            enemy.add_effect(crate::combat_types::mfx::DAZE, daze as i16);
        }

        // Source: reference/extracted/methods/monster/TheGuardian.java
        // (constructor, `useCloseUp`, and `changeState`). Ascension is only
        // available at the encounter spawn site.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "TheGuardian") {
            let threshold = if self.run_state.ascension >= 19 {
                40
            } else if self.run_state.ascension >= 9 {
                35
            } else {
                30
            };
            let (fierce_bash, roll) = if self.run_state.ascension >= 4 {
                (36, 10)
            } else {
                (32, 9)
            };
            let sharp_hide = if self.run_state.ascension >= 19 { 4 } else { 3 };
            enemy.entity.set_status(crate::status_ids::sid::PHASE, 0);
            enemy.entity.set_status(crate::status_ids::sid::MODE_SHIFT, threshold);
            enemy.entity.set_status(crate::status_ids::sid::DAMAGE_TAKEN_THIS_MODE, 0);
            enemy.entity.set_status(crate::status_ids::sid::FIERCE_BASH_DMG, fierce_bash);
            enemy.entity.set_status(crate::status_ids::sid::ROLL_DMG, roll);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT, 9);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, sharp_hide);
            enemy.entity.set_status(crate::status_ids::sid::TURN_COUNT, 20);
            enemy.set_move(crate::enemies::move_ids::GUARD_CHARGING_UP, 0, 0, 9);
        }

        // Source: reference/extracted/methods/monster/Hexaghost.java
        // (constructor). HP, attack damage, and A19 buff/card values use three
        // independent ascension thresholds.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Hexaghost") {
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 19 { 3 } else { 2 });
            enemy.entity.set_status(crate::status_ids::sid::SEAR_BURN_COUNT,
                if self.run_state.ascension >= 19 { 2 } else { 1 });
            let high_damage = self.run_state.ascension >= 4;
            enemy.entity.set_status(crate::status_ids::sid::FIRE_TACKLE_DMG,
                if high_damage { 6 } else { 5 });
            enemy.entity.set_status(crate::status_ids::sid::INFERNO_DMG,
                if high_damage { 3 } else { 2 });
        }

        // The boss passes ascension-derived large-slime constructor values to
        // its children when its split resolves.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "SlimeBoss") {
            enemy.entity.set_status(crate::status_ids::sid::FIRE_TACKLE_DMG,
                if self.run_state.ascension >= 4 { 10 } else { 9 });
            enemy.entity.set_status(crate::status_ids::sid::SLAP_DMG,
                if self.run_state.ascension >= 4 { 38 } else { 35 });
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 19 { 5 } else { 3 });
            enemy.set_move(crate::enemies::move_ids::SB_STICKY, 0, 0, 0);
            enemy.add_effect(crate::combat_types::mfx::SLIMED,
                if self.run_state.ascension >= 19 { 5 } else { 3 });
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 1 } else { 0 });
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 17 } else { 0 });
        }

        // Java Cultist.java: ctor sets ritualAmount = ascensionLevel >= 2 ? 4 : 3;
        // takeTurn() case 3 (INCANTATION) applies RitualPower(ritualAmount + 1)
        // at ascensionLevel >= 17, else RitualPower(ritualAmount).
        // create_enemy seeds the base-3 value; patch it here where ascension is known.
        if self.run_state.ascension >= 2 {
            let ritual = if self.run_state.ascension >= 17 { 5 } else { 4 };
            for enemy in enemy_states.iter_mut().filter(|e| e.id == "Cultist") {
                enemy.add_effect(crate::combat_types::mfx::RITUAL, ritual);
            }
        }

        // Sentry.java: even encounter indices open Bolt, odd indices open Beam.
        if expanded.len() == 3
            && expanded.iter().all(|id| id == "Sentry")
        {
            use crate::enemies::move_ids;
            for (idx, enemy) in enemy_states.iter_mut().enumerate() {
                enemy.entity.set_status(crate::status_ids::sid::IS_FIRST_MOVE,
                    if idx % 2 == 0 { move_ids::SENTRY_BOLT }
                    else { move_ids::SENTRY_BEAM });
            }
        }

        // Create combat state — convert deck strings to CardInstance
        let registry = crate::cards::global_registry();
        let mut deck_instances: Vec<crate::combat_types::CardInstance> = self.run_state.deck.iter()
            .map(|name| registry.make_card(name))
            .collect();
        if let Some(bottled) = self.run_state.bottled_flame_card.as_deref() {
            if let Some(card) = deck_instances.iter_mut().find(|card| {
                registry.card_name(card.def_id).trim_end_matches('+')
                    == bottled.trim_end_matches('+')
            }) {
                card.flags |= crate::combat_types::CardInstance::FLAG_INNATE;
            }
        }
        if let Some(bottled) = self.run_state.bottled_lightning_card.as_deref() {
            if let Some(card) = deck_instances.iter_mut().find(|card| {
                registry.card_name(card.def_id).trim_end_matches('+')
                    == bottled.trim_end_matches('+')
            }) {
                card.flags |= crate::combat_types::CardInstance::FLAG_INNATE;
            }
        }
        if let Some(bottled) = self.run_state.bottled_tornado_card.as_deref() {
            if let Some(card) = deck_instances.iter_mut().find(|card| {
                registry.card_name(card.def_id).trim_end_matches('+')
                    == bottled.trim_end_matches('+')
            }) {
                card.flags |= crate::combat_types::CardInstance::FLAG_INNATE;
            }
        }
        // BustedCrown.java, CoffeeDripper.java, CursedKey.java, Ectoplasm.java,
        // and FusionHammer.java each increment energyMaster once in onEquip.
        let combat_energy = 3
            + i32::from(
                self.run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::BUSTED_CROWN),
            )
            + i32::from(
                self.run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::COFFEE_DRIPPER),
            )
            + i32::from(
                self.run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::CURSED_KEY),
            )
            + i32::from(
                self.run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::ECTOPLASM),
            )
            + i32::from(
                self.run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::FUSION_HAMMER),
            );
        // DuVuDoll.java::onEquip/onMasterDeckChange counts every card whose
        // type is CURSE; atBattleStart grants that counter as Strength.
        let du_vu_curses = deck_instances
            .iter()
            .filter(|card| {
                registry.card_def_by_id(card.def_id).card_type
                    == crate::cards::CardType::Curse
            })
            .count() as i32;
        let mut combat_state = CombatState::new(
            self.run_state.current_hp,
            self.run_state.max_hp,
            enemy_states,
            deck_instances,
            combat_energy,
        );
        combat_state
            .player
            .set_status(crate::status_ids::sid::DU_VU_DOLL_CURSES, du_vu_curses);
        combat_state.player.set_status(
            crate::status_ids::sid::GIRYA_COUNTER,
            self.run_state.relic_flags.counters[crate::relic_flags::counter::GIRYA]
                as i32,
        );
        if self.run_state.lizard_tail_used {
            combat_state
                .player
                .set_status(crate::status_ids::sid::LIZARD_TAIL_USED, 1);
        }
        combat_state.relics = self.run_state.relics.clone();
        combat_state.potions = self.run_state.potions.clone();
        combat_state.relic_counters = self.run_state.relic_flags.counters;
        combat_state.run_gold = self.run_state.gold;

        let combat_seed = self.seed.wrapping_add(self.run_state.floor as u64 * 1000);
        // Java reseeds cardRandomRng to seed + floorNum on each floor.
        let card_random_seed = self.seed.wrapping_add(self.run_state.floor as u64);
        let mut engine = CombatEngine::new_with_card_random_seed(
            combat_state,
            combat_seed,
            card_random_seed,
        );
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
                // Source: reference/extracted/methods/monster/JawWorm.java:
                // setHp(40,44), or setHp(42,46) at ascension 7+ (inclusive).
                let base = if a20 { 42 } else { 40 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "Cultist" => {
                // Java Cultist.java ctor: setHp(50, 56) at ascension >= 7,
                // setHp(48, 54) below — a uniform inclusive roll, not a fixed
                // value (AbstractMonster.setHp -> monsterHpRng.random(min, max)).
                let base = if a20 { 50 } else { 48 };
                let hp = base + self.rng.gen_range(0..=6);
                (hp, hp)
            }
            "FuzzyLouseNormal" | "RedLouse" => {
                let base = if a20 { 11 } else { 10 };
                let hp = base + self.rng.gen_range(0..=5);
                (hp, hp)
            }
            "FuzzyLouseDefensive" | "GreenLouse" => {
                let base = if a20 { 12 } else { 11 };
                let hp = base + self.rng.gen_range(0..=6);
                (hp, hp)
            }
            "AcidSlime_S" => {
                let base = if a20 { 9 } else { 8 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "AcidSlime_M" => {
                let base = if a20 { 29 } else { 28 };
                let width = if a20 { 5 } else { 4 };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "AcidSlime_L" => {
                let base = if a20 { 68 } else { 65 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "SpikeSlime_S" => {
                let base = if a20 { 11 } else { 10 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "SpikeSlime_M" => {
                let base = if a20 { 29 } else { 28 };
                let width = if a20 { 5 } else { 4 };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "SpikeSlime_L" => {
                let base = if a20 { 67 } else { 64 };
                let hp = base + self.rng.gen_range(0..=6);
                (hp, hp)
            }
            "Looter" => {
                let base = if a20 { 46 } else { 44 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "GremlinFat" => {
                let base = if a20 { 14 } else { 13 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "GremlinThief" => {
                let base = if a20 { 11 } else { 10 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "GremlinWarrior" => {
                let base = if a20 { 21 } else { 20 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "GremlinWizard" => {
                let base = if a20 { 22 } else { 21 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "GremlinTsundere" => {
                let (base, width) = if a20 { (13, 4) } else { (12, 3) };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "FungiBeast" => {
                // Source: reference/extracted/methods/monster/FungiBeast.java:
                // setHp(22,28), or setHp(24,28) at ascension 7+ (inclusive).
                let base = if a20 { 24 } else { 22 };
                let hp = base + self.rng.gen_range(0..=(28 - base));
                (hp, hp)
            }
            "BlueSlaver" | "SlaverBlue" => {
                let base = if a20 { 48 } else { 46 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "RedSlaver" | "SlaverRed" => {
                let base = if a20 { 48 } else { 46 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "GremlinNob" => {
                let (base, width) = if self.run_state.ascension >= 8 {
                    (85, 5)
                } else {
                    (82, 4)
                };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "Lagavulin" => {
                let (base, width) = if self.run_state.ascension >= 8 {
                    (112, 3)
                } else {
                    (109, 2)
                };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "Sentry" => {
                let (base, width) = if self.run_state.ascension >= 8 {
                    (39, 6)
                } else {
                    (38, 4)
                };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "TheGuardian" => {
                // Source: TheGuardian.java constructor: HP changes at A9,
                // independently of the ordinary-enemy A7 HP threshold.
                let hp = if self.run_state.ascension >= 9 { 250 } else { 240 };
                (hp, hp)
            }
            "Hexaghost" => {
                // Source: Hexaghost.java constructor: HP changes at A9.
                let hp = if self.run_state.ascension >= 9 { 264 } else { 250 };
                (hp, hp)
            }
            "SlimeBoss" => {
                // Source: SlimeBoss.java constructor: HP changes at A9.
                let hp = if self.run_state.ascension >= 9 { 150 } else { 140 };
                (hp, hp)
            }
            "Apology Slime" | "ApologySlime" => {
                // Source: ApologySlime.java constructor: monsterHpRng.random(8, 12).
                let hp = self.rng.gen_range(8..=12);
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

        let escaped_with_smoke_bomb = matches!(
            &combat_action,
            crate::actions::Action::UsePotion { potion_idx, .. }
                if engine
                    .state
                    .potions
                    .get(*potion_idx)
                    .is_some_and(|id| matches!(id.as_str(), "SmokeBomb" | "Smoke Bomb"))
        );

        engine.clear_event_log();
        let hp_before = engine.state.player.hp;
        engine.execute_action(&combat_action);
        self.run_state.gold = engine.state.run_gold;

        let mut reward = 0.0;

        if engine.is_combat_over() {
            if escaped_with_smoke_bomb {
                // SmokeBomb marks the room smoked and the player escaping; it
                // is neither a victory nor player death and grants no combat
                // rewards. Persist the live combat state, then return to map.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/SmokeBomb.java
                self.run_state.current_hp = engine.state.player.hp;
                self.run_state.max_hp = engine.state.player.max_hp;
                self.run_state.potions = engine.state.potions.clone();
                self.run_state.deck = engine
                    .state
                    .master_deck
                    .iter()
                    .map(|card| engine.card_registry.card_name(card.def_id).to_string())
                    .collect();
                self.run_state.relic_flags.counters = engine.state.relic_counters;
                self.run_state.lizard_tail_used = engine
                    .state
                    .player
                    .status(crate::status_ids::sid::LIZARD_TAIL_USED)
                    > 0;
                self.run_state.persisted_effect_states = engine.export_persisted_effects();
                self.last_combat_events = engine.take_event_log();
                self.pending_event_combat = None;
                self.combat_engine = None;
                self.phase = RunPhase::MapChoice;
                self.refresh_decision_stack();
                return reward;
            }
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
                self.run_state.max_hp = engine.state.player.max_hp;
                let recovered_stolen_gold: i32 = engine.state.enemies.iter()
                    .filter(|enemy| enemy.id == "Looter"
                        && enemy.entity.hp <= 0 && !enemy.is_escaping)
                    .map(|enemy| enemy.entity.status(crate::status_ids::sid::COUNT))
                    .sum();
                adjust_run_gold_state(&mut self.run_state, recovered_stolen_gold);
                self.run_state.potions = engine.state.potions.clone();
                self.run_state.deck = engine
                    .state
                    .master_deck
                    .iter()
                    .map(|card| engine.card_registry.card_name(card.def_id).to_string())
                    .collect();
                if engine.state.pending_run_gold > 0
                    && !self
                        .run_state
                        .relic_flags
                        .has(crate::relic_flags::flag::ECTOPLASM)
                {
                    adjust_run_gold_state(&mut self.run_state, engine.state.pending_run_gold);
                }
                self.run_state.relic_flags.counters = engine.state.relic_counters;
                self.run_state.lizard_tail_used = engine
                    .state
                    .player
                    .status(crate::status_ids::sid::LIZARD_TAIL_USED)
                    > 0;
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
                    self.adjust_run_gold(gold);
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
                        self.adjust_run_gold(extra_gold);
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
                self.run_state.lizard_tail_used = engine
                    .state
                    .player
                    .status(crate::status_ids::sid::LIZARD_TAIL_USED)
                    > 0;
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
                card_id: self.upgrade_reward_card_if_needed(card),
            });
        }
        cards
    }

    fn card_reward_choice_count(&self) -> usize {
        let question_card_bonus = if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::QUESTION_CARD)
        {
            1
        } else {
            0
        };
        let busted_crown_penalty = if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::BUSTED_CROWN)
        {
            2
        } else {
            0
        };
        (3 + question_card_bonus - busted_crown_penalty).max(1) as usize
    }

    fn build_dream_catcher_reward_screen(&mut self) {
        // CampfireSleepEffect.java opens AbstractDungeon.getRewardCards after
        // resting with Dream Catcher, so normal reward-count relic callbacks
        // (Question Card and Busted Crown) still apply.
        let choices = self.generate_card_reward_choices(self.card_reward_choice_count());
        self.reward_screen = Some(RewardScreen {
            source: RewardScreenSource::Event,
            ordered: true,
            active_item: None,
            items: vec![RewardItem {
                index: 0,
                kind: RewardItemKind::CardChoice,
                state: RewardItemState::Available,
                label: "dream_catcher_reward".to_string(),
                claimable: true,
                active: false,
                skip_allowed: true,
                skip_label: Some("Skip".to_string()),
                choices,
            }],
        });
    }

    fn build_peace_pipe_selection_screen(&mut self) {
        // PeacePipe.java adds an active TokeOption only when the purgeable,
        // non-bottled master-deck group is nonempty. CampfireTokeEffect then
        // opens an exact one-card purge selection from that same group.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/PeacePipe.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/vfx/campfire/CampfireTokeEffect.java
        let choices = self
            .peace_pipe_removable_indices()
            .into_iter()
            .map(|index| RewardChoice::Card {
                index,
                card_id: self.run_state.deck[index].clone(),
            })
            .collect();
        self.reward_screen = Some(RewardScreen {
            source: RewardScreenSource::Campfire,
            ordered: true,
            active_item: None,
            items: vec![RewardItem {
                index: 0,
                kind: RewardItemKind::CardChoice,
                state: RewardItemState::Available,
                label: "deck_selection_peace_pipe".to_string(),
                claimable: true,
                active: false,
                skip_allowed: false,
                skip_label: None,
                choices,
            }],
        });
    }

    fn roll_shovel_relic_id(&mut self) -> String {
        // Exordium.java sets relic tier odds to 50 common / 33 uncommon / 17
        // rare. CampfireDigEffect uses that tier roll and returnRandomRelic;
        // this run layer preserves those semantic tiers on its shared stream.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/Exordium.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/vfx/campfire/CampfireDigEffect.java
        const COMMON: &[&str] = &[
            "Akabeko", "Anchor", "Art of War", "Bag of Marbles",
            "Bag of Preparation", "Blood Vial", "Boot", "Bronze Scales",
            "Lantern", "Vajra",
        ];
        const UNCOMMON: &[&str] = &[
            "Blue Candle", "Darkstone Periapt", "Eternal Feather", "InkBottle",
            "Kunai", "Letter Opener", "Ornamental Fan",
        ];
        const RARE: &[&str] = &[
            "Bird Faced Urn", "Calipers", "Du-Vu Doll", "FossilizedHelix",
            "Ginger", "Ice Cream", "Incense Burner", "Old Coin",
            "Thread and Needle", "Tough Bandages", "TungstenRod",
        ];
        let roll = self.rng.gen_range(0..100);
        let pool = if roll < 50 {
            COMMON
        } else if roll < 83 {
            UNCOMMON
        } else {
            RARE
        };
        let registry = gameplay_registry();
        let candidates: Vec<&str> = pool
            .iter()
            .copied()
            .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
            .filter(|id| *id != "Old Coin" || self.run_state.floor <= 48)
            .filter(|id| !self.run_state.relics.iter().any(|owned| owned == id))
            .collect();
        if candidates.is_empty() {
            "Circlet".to_string()
        } else {
            candidates[self.rng.gen_range(0..candidates.len())].to_string()
        }
    }

    fn build_shovel_reward_screen(&mut self) {
        let relic = self.roll_shovel_relic_id();
        self.reward_screen = Some(RewardScreen {
            source: RewardScreenSource::Campfire,
            ordered: true,
            active_item: None,
            items: vec![RewardItem {
                index: 0,
                kind: RewardItemKind::Relic,
                state: RewardItemState::Available,
                label: relic,
                claimable: true,
                active: false,
                skip_allowed: false,
                skip_label: None,
                choices: Vec::new(),
            }],
        });
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
        // BustedCrown.java subtracts two through changeNumberOfCardsInReward;
        // QuestionCard adds one through the same ordered relic callback chain.
        let card_choice_count = self.card_reward_choice_count();

        for _ in 0..card_reward_count {
            items.push(RewardItem {
                index: items.len(),
                kind: RewardItemKind::CardChoice,
                state: RewardItemState::Available,
                label: "card_reward".to_string(),
                claimable: items.is_empty(),
                active: false,
                skip_allowed: true,
                skip_label: Some("Skip".to_string()),
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
        // CursedKey.java::onChestOpen obtains one random curse for non-boss
        // chests. Boss relic rewards use their separate boss-chest path.
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::CURSED_KEY)
        {
            let curse = RANDOM_OBTAINABLE_CURSES
                [self.rng.gen_range(0..RANDOM_OBTAINABLE_CURSES.len())]
                .to_string();
            obtain_master_deck_card_state(&mut self.run_state, curse);
        }
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
                // Matryoshka.java consumes relicRng.randomBoolean(0.75f):
                // COMMON on true, otherwise UNCOMMON.
                label: self.roll_matryoshka_relic_id(),
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
            let mut choices = item.choices.clone();
            // Sources: CardRewardScreen.java displays the ordinary Skip button
            // and a separate SingingBowlButton; SingingBowlButton::onClick
            // increases max HP by 2 and removes this card reward.
            if item.kind == RewardItemKind::CardChoice
                && self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::SINGING_BOWL)
            {
                choices.push(RewardChoice::Named {
                    index: choices.len(),
                    label: "Singing Bowl".to_string(),
                });
            }
            self.decision_stack.push(DecisionFrame::RewardChoice(RewardChoiceFrame {
                item_index,
                item_kind: item.kind,
                skip_allowed: item.skip_allowed,
                choices,
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
        let chose_astrolabe =
            matches!(&choice, RewardChoice::Named { label, .. } if label == "Astrolabe");
        let chose_calling_bell =
            matches!(&choice, RewardChoice::Named { label, .. } if label == "Calling Bell");
        let astrolabe_card_pick = item_label == "deck_selection_astrolabe";
        let bottled_flame_card_pick = item_label == "deck_selection_bottled_flame";
        let bottled_lightning_card_pick = item_label == "deck_selection_bottled_lightning";
        let bottled_tornado_card_pick = item_label == "deck_selection_bottled_tornado";

        match (kind, choice) {
            (RewardItemKind::CardChoice, RewardChoice::Card { card_id, .. }) => {
                if bottled_tornado_card_pick {
                    self.run_state.bottled_tornado_card = Some(card_id);
                    self.pending_bottled_tornado_selection = false;
                } else if bottled_lightning_card_pick {
                    self.run_state.bottled_lightning_card = Some(card_id);
                    self.pending_bottled_lightning_selection = false;
                } else if bottled_flame_card_pick {
                    self.run_state.bottled_flame_card = Some(card_id);
                    self.pending_bottled_flame_selection = false;
                } else if astrolabe_card_pick {
                    if let Some(RewardChoice::Card { index, .. }) =
                        choice_frame.choices.get(choice_index)
                    {
                        if let Some(removed) = self.remove_master_deck_card(*index) {
                            self.pending_astrolabe_removed.push(removed);
                        }
                    }
                } else if !self.resolve_event_deck_selection_choice(&item_label, &choice_frame, choice_index)
                {
                    self.add_card_reward(card_id);
                }
            }
            (RewardItemKind::CardChoice, RewardChoice::Named { label, .. })
                if label == "Singing Bowl" =>
            {
                self.run_state.max_hp += 2;
                self.run_state.current_hp += 2;
            }
            (RewardItemKind::Relic, RewardChoice::Named { label, .. }) => {
                self.add_relic_reward(&label);
            }
            (RewardItemKind::Potion, RewardChoice::Named { label, .. }) => {
                self.add_potion_reward(&label);
            }
            (RewardItemKind::Gold, RewardChoice::Named { label, .. }) => {
                if let Ok(amount) = label.parse::<i32>() {
                    self.adjust_run_gold(amount.max(0));
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

        if astrolabe_card_pick {
            if self.pending_astrolabe_removed.len() == 3 {
                let originals = std::mem::take(&mut self.pending_astrolabe_removed);
                self.astrolabe_transform_cards(originals);
                self.pending_astrolabe_selection = false;
            } else {
                self.build_astrolabe_selection_screen();
            }
        } else if chose_astrolabe && self.pending_astrolabe_selection {
            self.build_astrolabe_selection_screen();
        } else if chose_calling_bell && self.pending_calling_bell_rewards {
            self.build_calling_bell_reward_screen();
        }
        if (bottled_flame_card_pick
            || bottled_lightning_card_pick
            || bottled_tornado_card_pick)
            && self.suspended_reward_screen.is_some()
        {
            self.reward_screen = self.suspended_reward_screen.take();
        }
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
            "deck_selection_peace_pipe" => {
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
        let source = screen.source.clone();
        let suspend_for_bottle = screen.items.len() > 1
            && matches!(
                label.as_str(),
                "Bottled Flame" | "Bottled Lightning" | "Bottled Tornado"
            );

        match kind {
            RewardItemKind::Relic => self.add_relic_reward(&label),
            RewardItemKind::Potion => self.add_potion_reward(&label),
            RewardItemKind::Gold => {
                if let Ok(amount) = label.parse::<i32>() {
                    self.adjust_run_gold(amount.max(0));
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
        if suspend_for_bottle {
            self.suspended_reward_screen = self.reward_screen.clone();
        }
        if label == "Bottled Flame" && self.pending_bottled_flame_selection {
            self.build_bottled_flame_selection_screen(source);
        } else if label == "Bottled Lightning" && self.pending_bottled_lightning_selection {
            self.build_bottled_lightning_selection_screen(source);
        } else if label == "Bottled Tornado" && self.pending_bottled_tornado_selection {
            self.build_bottled_tornado_selection_screen(source);
        } else if label == "Calling Bell" && self.pending_calling_bell_rewards {
            self.build_calling_bell_reward_screen();
        }
    }

    fn add_card_reward(&mut self, card_id: String) {
        let upgraded = self.upgrade_reward_card_if_needed(&card_id);
        obtain_master_deck_card_state(&mut self.run_state, upgraded);
    }

    fn upgrade_reward_card_if_needed(&self, card_id: &str) -> String {
        upgrade_obtained_card_for_eggs(&self.run_state, card_id)
    }

    fn refresh_visible_card_rewards_for_eggs(&mut self) {
        let run_state = &self.run_state;
        let Some(screen) = self.reward_screen.as_mut() else {
            return;
        };
        for choice in screen.items.iter_mut().flat_map(|item| item.choices.iter_mut()) {
            if let RewardChoice::Card { card_id, .. } = choice {
                *card_id = upgrade_obtained_card_for_eggs(run_state, card_id);
            }
        }
    }

    fn add_relic_reward(&mut self, relic_id: &str) {
        // BossRelicSelectScreen instant-obtains HolyWater into slot zero; its
        // canSpawn guard means that slot contains PureWater in a valid run.
        // Preserve the same replacement semantics without duplicating relics.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/screens/select/BossRelicSelectScreen.java
        if relic_id == "HolyWater" {
            if let Some(slot) = self
                .run_state
                .relics
                .iter()
                .position(|owned| owned == "PureWater")
            {
                self.run_state.relics[slot] = relic_id.to_string();
            } else {
                self.run_state.relics.push(relic_id.to_string());
            }
        } else {
            self.run_state.relics.push(relic_id.to_string());
        }
        self.run_state.relic_flags.rebuild(&self.run_state.relics);
        self.run_state.relic_flags.init_relic_counter(relic_id);

        if matches!(relic_id, "Frozen Egg 2" | "Molten Egg 2" | "Toxic Egg 2") {
            // Each Egg's onEquip revisits cards already present on the combat
            // reward screen through onPreviewObtainCard.
            self.refresh_visible_card_rewards_for_eggs();
        }

        match relic_id {
            "Astrolabe" => self.prepare_astrolabe_on_equip(),
            "Bottled Flame" => {
                self.pending_bottled_flame_selection = !self.bottled_flame_choices().is_empty();
            }
            "Bottled Lightning" => {
                self.pending_bottled_lightning_selection =
                    !self.bottled_lightning_choices().is_empty();
            }
            "Bottled Tornado" => {
                self.pending_bottled_tornado_selection =
                    !self.bottled_tornado_choices().is_empty();
            }
            "Calling Bell" => self.pending_calling_bell_rewards = true,
            "Lee's Waffle" => {
                // Waffle.java::onEquip increases max HP by 7 without healing,
                // then separately heals by maxHealth. Mark of the Bloom blocks
                // that heal while leaving the max-HP increase intact.
                self.run_state.max_hp += 7;
                if !self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::MARK_OF_BLOOM)
                {
                    self.run_state.current_hp = self.run_state.max_hp;
                }
            }
            "Membership Card" => {
                // StoreRelic.java immediately applies ShopScreen's rounded
                // 0.5 discount to all visible offers and the purge price.
                if let Some(shop) = self.current_shop.as_mut() {
                    for (_, price) in &mut shop.cards {
                        *price = ((*price as f32) * 0.5).round() as i32;
                    }
                    for (_, price) in &mut shop.relics {
                        *price = ((*price as f32) * 0.5).round() as i32;
                    }
                    shop.remove_price = ((shop.remove_price as f32) * 0.5).round() as i32;
                }
            }
            "Old Coin" | "OldCoin" => {
                // OldCoin.java::onEquip calls AbstractPlayer.gainGold(300),
                // which also preserves Ectoplasm's gain-gold prohibition.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/OldCoin.java
                self.adjust_run_gold(300);
            }
            "Mango" => {
                // Mango.java::onEquip calls increaseMaxHp(14, true): max HP
                // always rises, while Mark of the Bloom can block the heal.
                self.run_state.max_hp += 14;
                if !self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::MARK_OF_BLOOM)
                {
                    self.run_state.current_hp =
                        (self.run_state.current_hp + 14).min(self.run_state.max_hp);
                }
            }
            "Pear" => {
                // Source: reference/extracted/methods/relic/Pear.java
                // onEquip calls increaseMaxHp(10, true): max HP always rises,
                // while Mark of the Bloom can prevent the accompanying heal.
                self.run_state.max_hp += 10;
                if !self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::MARK_OF_BLOOM)
                {
                    self.run_state.current_hp =
                        (self.run_state.current_hp + 10).min(self.run_state.max_hp);
                }
            }
            "Potion Belt" | "PotionBelt" => {
                // Source: reference/extracted/methods/relic/PotionBelt.java
                // onEquip increments potionSlots by two and appends two empty
                // PotionSlot entries without disturbing existing potions.
                self.run_state.max_potions += 2;
                self.run_state.potions.push(String::new());
                self.run_state.potions.push(String::new());
            }
            "Strawberry" => {
                // Source: reference/extracted/methods/relic/Strawberry.java
                // onEquip calls increaseMaxHp(7, true): max HP always rises,
                // while Mark of the Bloom can prevent the accompanying heal.
                self.run_state.max_hp += 7;
                if !self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::MARK_OF_BLOOM)
                {
                    self.run_state.current_hp =
                        (self.run_state.current_hp + 7).min(self.run_state.max_hp);
                }
            }
            // Source: reference/extracted/methods/relic/Whetstone.java upgrades
            // up to two upgradable ATTACK cards and checks bottled upgrades.
            "Whetstone" => self.upgrade_random_cards_by_type(crate::cards::CardType::Attack, 2),
            // Source: reference/extracted/methods/relic/WarPaint.java upgrades
            // up to two upgradable SKILL cards and checks bottled upgrades.
            "War Paint" | "WarPaint" => {
                self.upgrade_random_cards_by_type(crate::cards::CardType::Skill, 2)
            }
            // D27 partial fix: Pandora's Box. Java replaces all Strikes and Defends
            // in master_deck with `returnTrulyRandomCard()` from the player's class
            // common pool. We remove the starter basics now (the "transform" side);
            // filling with random commons needs the class-aware card pool + a seeded
            // card_random_rng stream (D52) that we haven't built yet. Until that
            // lands, post-Pandora runs have a smaller starter deck instead of
            // silently-unchanged basics. This is strictly closer to Java than the
            // pre-fix behavior (silently do nothing).
            "Pandora's Box" | "PandorasBox" => {
                self.run_state
                    .deck
                    .retain(|card| !matches!(card.as_str(), "Strike" | "Defend"));
            }
            _ => {}
        }
    }

    fn astrolabe_transform_cards(&mut self, originals: Vec<String>) {
        // AbstractDungeon.transformCard(card, true, miscRng) selects from the
        // character's available rarity pools and upgrades the result.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
        let registry = crate::cards::global_registry();
        for original in originals {
            let original = original.trim_end_matches('+');
            let colors: &[EventCardColor] = if self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::PRISMATIC_SHARD)
            {
                &[
                    EventCardColor::Red,
                    EventCardColor::Green,
                    EventCardColor::Blue,
                    EventCardColor::Purple,
                ]
            } else {
                &[EventCardColor::Purple]
            };
            let mut candidates = Vec::new();
            for color in colors {
                for rarity in [
                    EventCardRarity::Common,
                    EventCardRarity::Uncommon,
                    EventCardRarity::Rare,
                ] {
                    candidates.extend(matching_event_cards(*color, rarity));
                }
            }
            candidates.retain(|candidate| {
                !candidate.ends_with('+')
                    && candidate != original
                    && registry.get(candidate).is_some()
            });
            if candidates.is_empty() {
                continue;
            }
            let transformed = &candidates[self.rng.gen_range(0..candidates.len())];
            let upgraded = format!("{transformed}+");
            obtain_master_deck_card_state(&mut self.run_state, if registry.get(&upgraded).is_some() {
                upgraded
            } else {
                transformed.clone()
            });
        }
    }

    fn prepare_astrolabe_on_equip(&mut self) {
        let purgeable: Vec<usize> = self
            .run_state
            .deck
            .iter()
            .enumerate()
            .filter_map(|(idx, card)| Self::is_purgeable_master_deck_card(card).then_some(idx))
            .collect();
        self.pending_astrolabe_removed.clear();
        if purgeable.is_empty() {
            self.pending_astrolabe_selection = false;
        } else if purgeable.len() <= 3 {
            let mut originals = Vec::with_capacity(purgeable.len());
            for idx in purgeable.into_iter().rev() {
                originals.push(self.run_state.deck.remove(idx));
            }
            originals.reverse();
            self.astrolabe_transform_cards(originals);
            self.pending_astrolabe_selection = false;
        } else {
            self.pending_astrolabe_selection = true;
        }
    }

    fn build_astrolabe_selection_screen(&mut self) {
        let choices = self
            .run_state
            .deck
            .iter()
            .enumerate()
            .filter(|(_, card)| Self::is_purgeable_master_deck_card(card))
            .map(|(index, card_id)| RewardChoice::Card {
                index,
                card_id: card_id.clone(),
            })
            .collect();
        let mut screen = RewardScreen {
            source: RewardScreenSource::BossCombat,
            ordered: true,
            active_item: None,
            items: vec![RewardItem {
                index: 0,
                kind: RewardItemKind::CardChoice,
                state: RewardItemState::Available,
                label: "deck_selection_astrolabe".to_string(),
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

    fn bottled_flame_choices(&self) -> Vec<RewardChoice> {
        self.bottled_card_choices(crate::cards::CardType::Attack)
    }

    fn bottled_lightning_choices(&self) -> Vec<RewardChoice> {
        self.bottled_card_choices(crate::cards::CardType::Skill)
    }

    fn bottled_tornado_choices(&self) -> Vec<RewardChoice> {
        self.bottled_card_choices(crate::cards::CardType::Power)
    }

    fn bottled_card_choices(&self, card_type: crate::cards::CardType) -> Vec<RewardChoice> {
        let registry = crate::cards::global_registry();
        self.run_state
            .deck
            .iter()
            .enumerate()
            .filter(|(_, card_id)| Self::is_purgeable_master_deck_card(card_id))
            .filter(|(_, card_id)| {
                registry
                    .get(card_id)
                    .is_some_and(|card| card.card_type == card_type)
            })
            .map(|(index, card_id)| RewardChoice::Card {
                index,
                card_id: card_id.clone(),
            })
            .collect()
    }

    fn build_bottled_flame_selection_screen(&mut self, source: RewardScreenSource) {
        self.build_bottled_card_selection_screen(
            source,
            "deck_selection_bottled_flame",
            self.bottled_flame_choices(),
        );
    }

    fn build_bottled_lightning_selection_screen(&mut self, source: RewardScreenSource) {
        self.build_bottled_card_selection_screen(
            source,
            "deck_selection_bottled_lightning",
            self.bottled_lightning_choices(),
        );
    }

    fn build_bottled_tornado_selection_screen(&mut self, source: RewardScreenSource) {
        self.build_bottled_card_selection_screen(
            source,
            "deck_selection_bottled_tornado",
            self.bottled_tornado_choices(),
        );
    }

    fn build_bottled_card_selection_screen(
        &mut self,
        source: RewardScreenSource,
        label: &str,
        choices: Vec<RewardChoice>,
    ) {
        let mut screen = RewardScreen {
            source,
            ordered: true,
            active_item: None,
            items: vec![RewardItem {
                index: 0,
                kind: RewardItemKind::CardChoice,
                state: RewardItemState::Available,
                label: label.to_string(),
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

    fn roll_calling_bell_tier_relic(&mut self, pool: &[&str]) -> String {
        let registry = gameplay_registry();
        let candidates: Vec<&str> = pool
            .iter()
            .copied()
            .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
            .filter(|id| *id != "Ancient Tea Set" || self.run_state.floor <= 48)
            .filter(|id| *id != "Old Coin" || self.run_state.floor <= 48)
            .filter(|id| *id != "Smiling Mask" || self.run_state.floor <= 48)
            .filter(|id| {
                *id != "Peace Pipe"
                    || (self.run_state.floor < 48 && self.campfire_relic_count() < 2)
            })
            .filter(|id| {
                *id != "Girya"
                    || (self.run_state.floor < 48 && self.campfire_relic_count() < 2)
            })
            .filter(|id| {
                *id != "Shovel"
                    || (self.run_state.floor < 48 && self.campfire_relic_count() < 2)
            })
            .filter(|id| *id != "Bottled Flame" || self.can_spawn_bottled_flame())
            .filter(|id| *id != "Bottled Lightning" || self.can_spawn_bottled_lightning())
            .filter(|id| *id != "Bottled Tornado" || self.can_spawn_bottled_tornado())
            .filter(|id| !self.run_state.relics.iter().any(|owned| owned == id))
            .collect();
        if candidates.is_empty() {
            return "Circlet".to_string();
        }
        candidates[self.rng.gen_range(0..candidates.len())].to_string()
    }

    fn build_calling_bell_reward_screen(&mut self) {
        // CallingBell.java opens one mandatory Curse of the Bell confirmation,
        // then one screenless COMMON, UNCOMMON, and RARE relic reward.
        const COMMON: &[&str] = &[
            "Akabeko",
            "Anchor",
            "Ancient Tea Set",
            "Art of War",
            "Bag of Marbles",
            "Bag of Preparation",
            "Blood Vial",
            "Boot",
            "Bronze Scales",
            "Smiling Mask",
            // Vajra.java uses canonical ID "Vajra" and COMMON tier.
            "Vajra",
        ];
        const UNCOMMON: &[&str] = &[
            "Blue Candle",
            "Bottled Flame",
            "Bottled Lightning",
            "Bottled Tornado",
            "Darkstone Periapt",
            "Eternal Feather",
            "Frozen Egg 2",
            "Molten Egg 2",
            "Ornamental Fan",
            "Pantograph",
            "Shuriken",
            "Toxic Egg 2",
        ];
        const RARE: &[&str] = &[
            "Bird Faced Urn",
            "Calipers",
            "Du-Vu Doll",
            "FossilizedHelix",
            "Girya",
            "Ginger",
            "Ice Cream",
            "Incense Burner",
            "Old Coin",
            "Peace Pipe",
            "Pocketwatch",
            "Shovel",
            "Torii",
            "Turnip",
            // ThreadAndNeedle.java constructs a RARE relic.
            "Thread and Needle",
            "Tough Bandages",
            "TungstenRod",
        ];
        let common = self.roll_calling_bell_tier_relic(COMMON);
        let uncommon = self.roll_calling_bell_tier_relic(UNCOMMON);
        let rare = self.roll_calling_bell_tier_relic(RARE);
        let mut screen = RewardScreen {
            source: RewardScreenSource::BossCombat,
            ordered: true,
            active_item: None,
            items: vec![
                RewardItem {
                    index: 0,
                    kind: RewardItemKind::CardChoice,
                    state: RewardItemState::Available,
                    label: "calling_bell_curse".to_string(),
                    claimable: true,
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: vec![RewardChoice::Card {
                        index: 0,
                        card_id: "CurseOfTheBell".to_string(),
                    }],
                },
                RewardItem {
                    index: 1,
                    kind: RewardItemKind::Relic,
                    state: RewardItemState::Available,
                    label: common,
                    claimable: false,
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: Vec::new(),
                },
                RewardItem {
                    index: 2,
                    kind: RewardItemKind::Relic,
                    state: RewardItemState::Available,
                    label: uncommon,
                    claimable: false,
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: Vec::new(),
                },
                RewardItem {
                    index: 3,
                    kind: RewardItemKind::Relic,
                    state: RewardItemState::Available,
                    label: rare,
                    claimable: false,
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: Vec::new(),
                },
            ],
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
        self.pending_calling_bell_rewards = false;
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

    fn adjust_run_gold(&mut self, amount: i32) {
        // BloodyIdol.java::onGainGold heals once per gold-gain call,
        // independent of the amount gained.
        adjust_run_gold_state(&mut self.run_state, amount);
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

        // MagicFlower.java only modifies healing in RoomPhase.COMBAT; all
        // RunEngine healing paths represented here occur outside combat.
        self.run_state.current_hp = (self.run_state.current_hp + amount).min(self.run_state.max_hp);
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
            let original = self.run_state.deck[deck_idx].clone();
            let upgraded = format!("{original}+");
            self.run_state.deck[deck_idx] = upgraded.clone();
            // Whetstone.java calls bottledCardUpgradeCheck after each ATTACK
            // upgrade. RunEngine stores the bottle by card ID, so keep a
            // uniquely represented bottled attack synchronized.
            if card_type == crate::cards::CardType::Attack
                && self.run_state.bottled_flame_card.as_deref() == Some(original.as_str())
            {
                self.run_state.bottled_flame_card = Some(upgraded);
            } else if card_type == crate::cards::CardType::Skill
                && self.run_state.bottled_lightning_card.as_deref() == Some(original.as_str())
            {
                // WarPaint.java performs the same bottledCardUpgradeCheck for
                // each SKILL it upgrades.
                self.run_state.bottled_lightning_card = Some(upgraded);
            }
        }
    }

    fn can_spawn_bottled_flame(&self) -> bool {
        self.can_spawn_bottled_card(crate::cards::CardType::Attack)
    }

    fn can_spawn_bottled_lightning(&self) -> bool {
        self.can_spawn_bottled_card(crate::cards::CardType::Skill)
    }

    fn can_spawn_bottled_tornado(&self) -> bool {
        let registry = crate::cards::global_registry();
        self.run_state.deck.iter().any(|card_id| {
            registry
                .get(card_id)
                .is_some_and(|card| card.card_type == crate::cards::CardType::Power)
        })
    }

    fn can_spawn_bottled_card(&self, card_type: crate::cards::CardType) -> bool {
        let registry = crate::cards::global_registry();
        self.run_state.deck.iter().any(|card_id| {
            registry
                .get(card_id)
                .is_some_and(|card| card.card_type == card_type)
                && event_card_rarity(card_id) != Some(EventCardRarity::Basic)
        })
    }

    fn roll_reward_relic_id(&mut self) -> String {
        self.roll_reward_relic_id_with_shop_exclusions(false)
    }

    fn campfire_relic_count(&self) -> usize {
        self.run_state
            .relics
            .iter()
            .filter(|relic| matches!(relic.as_str(), "Peace Pipe" | "Shovel" | "Girya"))
            .count()
    }

    fn roll_shop_reward_relic_id(&mut self) -> String {
        self.roll_reward_relic_id_with_shop_exclusions(true)
    }

    fn roll_reward_relic_id_with_shop_exclusions(&mut self, in_shop: bool) -> String {
        const RELIC_REWARD_POOL: &[&str] = &[
            // RelicLibrary.java registers Ancient Tea Set at COMMON tier;
            // AncientTeaSet.java::canSpawn excludes floors after 48.
            "Ancient Tea Set",
            // RelicLibrary.java registers Art of War, whose constructor in
            // relics/ArtOfWar.java assigns it to the COMMON tier.
            "Art of War",
            // RelicLibrary.java registers Akabeko, whose constructor in
            // relics/Akabeko.java assigns it to the COMMON tier.
            "Akabeko",
            "Vajra",
            "Anchor",
            // BagOfMarbles.java uses the canonical ID "Bag of Marbles" and
            // assigns the relic to the COMMON tier.
            "Bag of Marbles",
            // BagOfPreparation.java uses this canonical ID and COMMON tier.
            "Bag of Preparation",
            // BirdFacedUrn.java uses this canonical ID and RARE tier.
            "Bird Faced Urn",
            // BloodVial.java uses this canonical ID and COMMON tier.
            "Blood Vial",
            // BlueCandle.java uses this canonical ID and UNCOMMON tier.
            "Blue Candle",
            // Boot.java uses canonical ID "Boot" and COMMON tier.
            "Boot",
            // BottledFlame.java uses this canonical ID and UNCOMMON tier.
            "Bottled Flame",
            // BottledLightning.java uses this canonical ID and UNCOMMON tier.
            "Bottled Lightning",
            // BottledTornado.java uses this canonical ID and UNCOMMON tier.
            "Bottled Tornado",
            // BronzeScales.java uses this canonical ID and COMMON tier.
            "Bronze Scales",
            // Calipers.java uses this canonical ID and RARE tier.
            "Calipers",
            // CentennialPuzzle.java uses this canonical ID and COMMON tier.
            "Centennial Puzzle",
            // CeramicFish.java uses canonical ID "CeramicFish", COMMON tier,
            // and canSpawn excludes non-endless runs after floor 48.
            "CeramicFish",
            // DarkstonePeriapt.java uses this canonical ID, UNCOMMON tier, and
            // canSpawn excludes non-endless runs after floor 48.
            "Darkstone Periapt",
            // DreamCatcher.java uses this canonical ID, COMMON tier, and
            // canSpawn excludes non-endless runs after floor 48.
            "Dream Catcher",
            // DuVuDoll.java uses this canonical ID and RARE tier.
            "Du-Vu Doll",
            // EternalFeather.java uses this canonical ID and UNCOMMON tier.
            "Eternal Feather",
            // Ginger.java uses this canonical ID and RARE tier.
            "Ginger",
            // FossilizedHelix.java uses canonical ID "FossilizedHelix" and RARE tier.
            "FossilizedHelix",
            // IceCream.java uses canonical ID "Ice Cream" and RARE tier.
            "Ice Cream",
            // IncenseBurner.java uses canonical ID "Incense Burner" and RARE tier.
            "Incense Burner",
            // InkBottle.java uses canonical ID "InkBottle" and UNCOMMON tier.
            "InkBottle",
            // Kunai.java uses canonical ID "Kunai" and UNCOMMON tier.
            "Kunai",
            // Lantern.java uses canonical ID "Lantern" and COMMON tier.
            "Lantern",
            // LetterOpener.java uses canonical ID "Letter Opener" and UNCOMMON tier.
            "Letter Opener",
            // LizardTail.java uses canonical ID "Lizard Tail" and RARE tier.
            "Lizard Tail",
            // MagicFlower.java uses canonical ID "Magic Flower" and RARE tier.
            "Magic Flower",
            // Mango.java uses canonical ID "Mango" and RARE tier.
            "Mango",
            // OldCoin.java uses canonical ID "Old Coin", RARE tier, and
            // canSpawn excludes non-endless runs after floor 48.
            "Old Coin",
            // SmilingMask.java uses canonical ID "Smiling Mask", COMMON tier,
            // and canSpawn excludes non-endless runs after floor 48.
            "Smiling Mask",
            // PeacePipe.java uses canonical ID "Peace Pipe", RARE tier, and
            // canSpawn requires floor < 48 and fewer than two campfire relics.
            "Peace Pipe",
            // Girya.java uses canonical ID "Girya", RARE tier, and the same
            // floor and campfire-relic-count spawn gates as Peace Pipe.
            "Girya",
            // Shovel.java uses canonical ID "Shovel", RARE tier, and the same
            // floor and campfire-relic-count spawn gates as Peace Pipe.
            "Shovel",
            // Pocketwatch.java uses canonical ID "Pocketwatch" and RARE tier.
            "Pocketwatch",
            // Torii.java uses canonical ID "Torii" and RARE tier.
            "Torii",
            // Turnip.java uses canonical ID "Turnip" and RARE tier.
            "Turnip",
            // TungstenRod.java uses canonical ID "TungstenRod" and RARE tier.
            "TungstenRod",
            // Matryoshka.java uses canonical ID "Matryoshka", UNCOMMON tier,
            // and canSpawn excludes non-endless runs after floor 40.
            "Matryoshka",
            // MawBank.java uses canonical ID "MawBank", COMMON tier, and
            // canSpawn excludes floors after 48 and the current shop room.
            "MawBank",
            // MealTicket.java uses canonical ID "MealTicket", COMMON tier,
            // and canSpawn excludes non-endless runs after floor 48.
            "MealTicket",
            // MeatOnTheBone.java uses canonical ID "Meat on the Bone",
            // UNCOMMON tier, and a floor-48 spawn cutoff.
            "Meat on the Bone",
            // HappyFlower.java uses canonical ID "Happy Flower" and COMMON tier.
            "Happy Flower",
            // JuzuBracelet.java uses canonical ID "Juzu Bracelet", COMMON tier,
            // and canSpawn excludes non-endless runs after floor 48.
            "Juzu Bracelet",
            // MercuryHourglass.java uses canonical ID "Mercury Hourglass" and
            // UNCOMMON tier.
            "Mercury Hourglass",
            // MummifiedHand.java uses canonical ID "Mummified Hand" and
            // UNCOMMON tier.
            "Mummified Hand",
            // Nunchaku.java uses canonical ID "Nunchaku" and COMMON tier.
            "Nunchaku",
            // OddlySmoothStone.java uses canonical ID "Oddly Smooth Stone"
            // and COMMON tier.
            "Oddly Smooth Stone",
            // Orichalcum.java uses canonical ID "Orichalcum" and COMMON tier.
            "Orichalcum",
            // OrnamentalFan.java uses canonical ID "Ornamental Fan" and
            // UNCOMMON tier.
            "Ornamental Fan",
            // Pantograph.java uses canonical ID "Pantograph" and UNCOMMON tier.
            "Pantograph",
            // Shuriken.java uses canonical ID "Shuriken" and UNCOMMON tier.
            "Shuriken",
            // PenNib.java uses canonical ID "Pen Nib" and COMMON tier.
            "Pen Nib",
            // Pear.java uses canonical ID "Pear" and UNCOMMON tier.
            "Pear",
            // QuestionCard.java uses canonical ID "Question Card", UNCOMMON
            // tier, and canSpawn excludes non-endless runs after floor 48.
            "Question Card",
            // PrayerWheel.java uses canonical ID "Prayer Wheel", RARE tier,
            // and canSpawn excludes non-endless runs after floor 48.
            "Prayer Wheel",
            // PotionBelt.java uses canonical ID "Potion Belt", COMMON tier,
            // and canSpawn excludes non-endless runs after floor 48.
            "Potion Belt",
            // PreservedInsect.java uses canonical ID "PreservedInsect",
            // COMMON tier, and canSpawn excludes non-endless runs after 52.
            "PreservedInsect",
            // RegalPillow.java uses canonical ID "Regal Pillow", COMMON tier,
            // and canSpawn excludes non-endless runs after floor 48.
            "Regal Pillow",
            // SingingBowl.java uses canonical ID "Singing Bowl", UNCOMMON
            // tier, and canSpawn excludes non-endless runs after floor 48.
            "Singing Bowl",
            // Strawberry.java uses canonical ID "Strawberry" and COMMON tier.
            "Strawberry",
            // ThreadAndNeedle.java uses canonical ID "Thread and Needle" and
            // RARE tier.
            "Thread and Needle",
            // Whetstone.java uses canonical ID "Whetstone" and COMMON tier.
            "Whetstone",
            // WarPaint.java uses canonical ID "War Paint" and COMMON tier.
            "War Paint",
            // FrozenEgg2.java, MoltenEgg2.java, and ToxicEgg2.java use spaced
            // canonical IDs, UNCOMMON tier, and a floor-48 spawn cutoff.
            "Frozen Egg 2",
            "Molten Egg 2",
            "Toxic Egg 2",
        ];

        let mut candidates: Vec<&str> = RELIC_REWARD_POOL
            .iter()
            .copied()
            .filter(|relic| *relic != "Ancient Tea Set" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "CeramicFish" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Darkstone Periapt" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Dream Catcher" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Matryoshka" || self.run_state.floor <= 40)
            .filter(|relic| *relic != "MawBank" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "MealTicket" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Meat on the Bone" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Old Coin" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Smiling Mask" || self.run_state.floor <= 48)
            .filter(|relic| {
                *relic != "Peace Pipe"
                    || (self.run_state.floor < 48 && self.campfire_relic_count() < 2)
            })
            .filter(|relic| {
                *relic != "Girya"
                    || (self.run_state.floor < 48 && self.campfire_relic_count() < 2)
            })
            .filter(|relic| {
                *relic != "Shovel"
                    || (self.run_state.floor < 48 && self.campfire_relic_count() < 2)
            })
            .filter(|relic| {
                !in_shop || !matches!(*relic, "Old Coin" | "Smiling Mask")
            })
            .filter(|relic| *relic != "Juzu Bracelet" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Question Card" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Prayer Wheel" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Potion Belt" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "PreservedInsect" || self.run_state.floor <= 52)
            .filter(|relic| *relic != "Regal Pillow" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Singing Bowl" || self.run_state.floor <= 48)
            .filter(|relic| {
                !matches!(*relic, "Frozen Egg 2" | "Molten Egg 2" | "Toxic Egg 2")
                    || self.run_state.floor <= 48
            })
            .filter(|relic| *relic != "Bottled Flame" || self.can_spawn_bottled_flame())
            .filter(|relic| {
                *relic != "Bottled Lightning" || self.can_spawn_bottled_lightning()
            })
            .filter(|relic| *relic != "Bottled Tornado" || self.can_spawn_bottled_tornado())
            .filter(|relic| !self.run_state.relics.iter().any(|owned| owned == relic))
            .collect();
        if candidates.is_empty() {
            candidates.extend(
                RELIC_REWARD_POOL
                    .iter()
                    .copied()
                    .filter(|relic| *relic != "Ancient Tea Set" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "CeramicFish" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Darkstone Periapt" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Dream Catcher" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Matryoshka" || self.run_state.floor <= 40)
                    .filter(|relic| *relic != "MawBank" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "MealTicket" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Meat on the Bone" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Old Coin" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Smiling Mask" || self.run_state.floor <= 48)
                    .filter(|relic| {
                        *relic != "Peace Pipe"
                            || (self.run_state.floor < 48
                                && self.campfire_relic_count() < 2)
                    })
                    .filter(|relic| {
                        *relic != "Girya"
                            || (self.run_state.floor < 48
                                && self.campfire_relic_count() < 2)
                    })
                    .filter(|relic| {
                        *relic != "Shovel"
                            || (self.run_state.floor < 48
                                && self.campfire_relic_count() < 2)
                    })
                    .filter(|relic| {
                        !in_shop || !matches!(*relic, "Old Coin" | "Smiling Mask")
                    })
                    .filter(|relic| *relic != "Juzu Bracelet" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Question Card" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Prayer Wheel" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Potion Belt" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "PreservedInsect" || self.run_state.floor <= 52)
                    .filter(|relic| *relic != "Regal Pillow" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Singing Bowl" || self.run_state.floor <= 48)
                    .filter(|relic| {
                        !matches!(*relic, "Frozen Egg 2" | "Molten Egg 2" | "Toxic Egg 2")
                            || self.run_state.floor <= 48
                    })
                    .filter(|relic| *relic != "Bottled Flame" || self.can_spawn_bottled_flame())
                    .filter(|relic| {
                        *relic != "Bottled Lightning" || self.can_spawn_bottled_lightning()
                    })
                    .filter(|relic| {
                        *relic != "Bottled Tornado" || self.can_spawn_bottled_tornado()
                    }),
            );
        }

        let idx = self.rng.gen_range(0..candidates.len());
        candidates[idx].to_string()
    }

    fn roll_matryoshka_relic_id(&mut self) -> String {
        // Matryoshka.java::onChestOpen consumes the 75% tier roll before the
        // tier-specific reward is generated. RunEngine currently uses one
        // shared run RNG, so this preserves semantic tiering and call order.
        const COMMON: &[&str] = &[
            "Akabeko", "Anchor", "Ancient Tea Set", "Art of War",
            "Bag of Marbles", "Bag of Preparation", "Blood Vial", "Boot",
            "Bronze Scales", "CeramicFish", "Dream Catcher", "Lantern", "Vajra",
        ];
        const UNCOMMON: &[&str] = &[
            "Blue Candle", "Bottled Flame", "Bottled Lightning", "Bottled Tornado",
            "Darkstone Periapt", "Eternal Feather", "Frozen Egg 2", "InkBottle",
            "Kunai", "Letter Opener", "Matryoshka", "Molten Egg 2",
            "Ornamental Fan", "Toxic Egg 2",
        ];

        let pool = if self.rng.gen_bool(0.75) { COMMON } else { UNCOMMON };
        let registry = gameplay_registry();
        let candidates: Vec<&str> = pool
            .iter()
            .copied()
            .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
            .filter(|id| !self.run_state.relics.iter().any(|owned| owned == id))
            .collect();
        if candidates.is_empty() {
            return "Circlet".to_string();
        }
        candidates[self.rng.gen_range(0..candidates.len())].to_string()
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
            "Ambrosia",
            "BottledMiracle",
            "StancePotion",
            "Ancient Potion",
            "BlessingOfTheForge",
            "ColorlessPotion",
            "CultistPotion",
            "DistilledChaos",
            "DuplicationPotion",
            "EntropicBrew",
            "EssenceOfSteel",
            "Explosive Potion",
            "FairyPotion",
            "FruitJuice",
            "GamblersBrew",
            "LiquidBronze",
            "LiquidMemories",
            "Regen Potion",
            "SmokeBomb",
            "SneckoOil",
            "SpeedPotion",
            "SteroidPotion",
            "PowerPotion",
            "SkillPotion",
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
            "Busted Crown",
            "Calling Bell",
            "Coffee Dripper",
            "Cursed Key",
            "Ectoplasm",
            "Fusion Hammer",
            "Philosopher's Stone",
            "Velvet Choker",
            "Snecko Eye",
            "HolyWater",
            "VioletLotus",
            "Astrolabe",
            "Black Star",
        ];

        let registry = gameplay_registry();
        // HolyWater.java canSpawn() requires PureWater. Java's boss-relic
        // selection therefore cannot offer the Watcher upgrade after its
        // starter relic has already been replaced.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/HolyWater.java
        let has_pure_water = self
            .run_state
            .relics
            .iter()
            .any(|owned| owned == "PureWater");
        let act = self.run_state.act;
        let can_spawn = |id: &&str| {
            (*id != "HolyWater" || has_pure_water)
                && (*id != "Ectoplasm" || act <= 1)
        };
        let mut candidates: Vec<&str> = BOSS_RELIC_POOL
            .iter()
            .copied()
            .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
            .filter(|id| !self.run_state.relics.iter().any(|owned| owned == id))
            .filter(can_spawn)
            .collect();
        if candidates.len() < count {
            candidates = BOSS_RELIC_POOL
                .iter()
                .copied()
                .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
                .filter(can_spawn)
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

    fn enter_campfire(&mut self) {
        // EternalFeather.java::onEnterRoom heals floor(masterDeck.size / 5) * 3
        // on RestRoom entry, before the player chooses a campfire option.
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::ETERNAL_FEATHER)
        {
            let heal = (self.run_state.deck.len() / 5 * 3) as i32;
            self.heal_run_player(heal);
        }
        // AncientTeaSet.java::onEnterRestRoom arms the relic on room entry,
        // regardless of whether the player subsequently rests or upgrades.
        if self
            .run_state
            .relics
            .iter()
            .any(|relic| relic == "Ancient Tea Set")
        {
            self.run_state.relic_flags.counters
                [crate::relic_flags::counter::ANCIENT_TEA_SET] = 1;
        }
        self.phase = RunPhase::Campfire;
    }

    fn step_campfire(&mut self, action: &RunAction) -> f32 {
        match action {
            RunAction::CampfireRest => {
                if !self.run_state.relic_flags.has(crate::relic_flags::flag::MARK_OF_BLOOM) {
                    // Source: CampfireSleepEffect.java truncates maxHealth *
                    // 0.3f to int, then adds Regal Pillow's exact 15.
                    let mut heal = (self.run_state.max_hp as f32 * 0.3) as i32;
                    if self.run_state.relic_flags.has(crate::relic_flags::flag::REGAL_PILLOW) {
                        heal += 15;
                    }
                    self.run_state.current_hp = (self.run_state.current_hp + heal).min(self.run_state.max_hp);
                }
                // CampfireSleepEffect.java opens a card reward only after the
                // Rest option resolves; entering the room or upgrading does not.
                if self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::DREAM_CATCHER)
                {
                    self.build_dream_catcher_reward_screen();
                    self.phase = RunPhase::CardReward;
                    self.refresh_decision_stack();
                    return 0.0;
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
            RunAction::CampfireToke => {
                self.build_peace_pipe_selection_screen();
                self.phase = RunPhase::CardReward;
                self.refresh_decision_stack();
                return 0.0;
            }
            RunAction::CampfireLift => {
                // CampfireLiftEffect increments Girya.counter exactly once;
                // LiftOption is usable only while the counter is below three.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/vfx/campfire/CampfireLiftEffect.java
                let counter = &mut self.run_state.relic_flags.counters
                    [crate::relic_flags::counter::GIRYA];
                *counter = (*counter + 1).min(3);
            }
            RunAction::CampfireDig => {
                self.build_shovel_reward_screen();
                self.phase = RunPhase::CardReward;
                self.refresh_decision_stack();
                return 0.0;
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
                ((price as f32) * 0.5).round() as i32
            } else {
                price
            };
            cards.push((self.upgrade_reward_card_if_needed(card), final_price));
        }

        let remove_price = self.compute_shop_remove_price();

        // ShopScreen.java::initRelics creates two ordinary tier rolls followed
        // by one guaranteed SHOP-tier relic. The run RNG is still shared, but
        // the visible slot semantics and purchase path match Java.
        let mut relics = Vec::new();
        for _ in 0..2 {
            let relic = self.roll_shop_reward_relic_id();
            let price = self.rng.gen_range(143..=158);
            let final_price = if self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::MEMBERSHIP_CARD)
            {
                ((price as f32) * 0.5).round() as i32
            } else {
                price
            };
            relics.push((relic, final_price));
        }
        const SHOP_RELICS: &[&str] = &[
            "TheAbacus", "Brimstone", "Chemical X", "ClockworkSouvenir", "Frozen Eye", "HandDrill", "Lee's Waffle", "Medical Kit", "Melange",
            "Membership Card",
            "OrangePellets", "Runic Capacitor", "Sling", "Strange Spoon",
            "TwistedFunnel",
        ];
        let registry = gameplay_registry();
        let candidates: Vec<&str> = SHOP_RELICS
            .iter()
            .copied()
            .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
            .filter(|id| !self.run_state.relics.iter().any(|owned| owned == id))
            .collect();
        let shop_relic = if candidates.is_empty() {
            "Circlet"
        } else {
            candidates[self.rng.gen_range(0..candidates.len())]
        };
        // AbstractRelic.java::getPrice returns 150 for SHOP tier; StoreRelic
        // applies merchantRng.random(0.95f, 1.05f).
        let shop_price = self.rng.gen_range(143..=158);
        let shop_price = if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::MEMBERSHIP_CARD)
        {
            ((shop_price as f32) * 0.5).round() as i32
        } else {
            shop_price
        };
        relics.push((shop_relic.to_string(), shop_price));

        self.current_shop = Some(ShopState {
            cards,
            relics,
            remove_price,
            removal_used: false,
        });
        self.phase = RunPhase::Shop;
        self.refresh_decision_stack();

        // Meal Ticket: heal 15 on shop enter
        if self.run_state.relic_flags.has(crate::relic_flags::flag::MEAL_TICKET)
            && !self.run_state.relic_flags.has(crate::relic_flags::flag::MARK_OF_BLOOM)
        {
            // Meal Ticket heals in a shop, not during combat, so Magic Flower
            // does not modify the 15 points.
            self.run_state.current_hp = (self.run_state.current_hp + 15).min(self.run_state.max_hp);
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
                            if self.run_state.relic_flags.has(crate::relic_flags::flag::MAW_BANK) {
                                self.run_state.relic_flags.counters
                                    [crate::relic_flags::counter::MAW_BANK_GOLD] = -2;
                            }
                            obtain_master_deck_card_state(&mut self.run_state, card);
                            shop.cards.remove(*idx);
                        }
                    }
                }
                // Stay in shop for more purchases
                return 0.0;
            }
            RunAction::ShopBuyRelic(idx) => {
                let purchase = self.current_shop.as_ref().and_then(|shop| {
                    shop.relics.get(*idx).cloned().filter(|(_, price)| {
                        self.run_state.gold >= *price
                    })
                });
                if let Some((relic, price)) = purchase {
                    self.adjust_run_gold(-price);
                    self.add_relic_reward(&relic);
                    if let Some(shop) = self.current_shop.as_mut() {
                        shop.relics.remove(*idx);
                    }
                }
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
                    if self.run_state.relic_flags.has(crate::relic_flags::flag::MAW_BANK) {
                        self.run_state.relic_flags.counters
                            [crate::relic_flags::counter::MAW_BANK_GOLD] = -2;
                    }
                    self.remove_master_deck_card(*idx);
                    self.run_state.purge_cost += 25;
                    let next_remove_price = self.compute_shop_remove_price();
                    if let Some(shop) = self.current_shop.as_mut() {
                        shop.remove_price = next_remove_price;
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

    fn build_match_and_keep_board(&mut self) -> Vec<String> {
        let color = self.current_event_player_color();
        let mut unique_cards = vec![
            self.random_match_and_keep_colored_card(color, EventCardRarity::Rare),
            self.random_match_and_keep_colored_card(color, EventCardRarity::Uncommon),
            self.random_match_and_keep_colored_card(color, EventCardRarity::Common),
        ];
        if self.run_state.ascension >= 15 {
            unique_cards.push(self.random_match_and_keep_curse());
            unique_cards.push(self.random_match_and_keep_curse());
        } else {
            unique_cards.push(
                self.random_match_and_keep_colored_card(
                    EventCardColor::Colorless,
                    EventCardRarity::Uncommon,
                ),
            );
            unique_cards.push(self.random_match_and_keep_curse());
        }
        unique_cards.push(self.match_and_keep_starter_card(color).to_string());

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

    fn current_event_player_color(&self) -> EventCardColor {
        let relics = &self.run_state.relics;
        if relics.iter().any(|relic| matches!(relic.as_str(), "Burning Blood" | "Black Blood")) {
            return EventCardColor::Red;
        }
        if relics
            .iter()
            .any(|relic| matches!(relic.as_str(), "Ring of the Snake" | "Ring of the Serpent"))
        {
            return EventCardColor::Green;
        }
        if relics
            .iter()
            .any(|relic| matches!(relic.as_str(), "Cracked Core" | "Nuclear Battery"))
        {
            return EventCardColor::Blue;
        }
        if relics
            .iter()
            .any(|relic| {
                matches!(
                    relic.as_str(),
                    "PureWater" | "HolyWater" | "VioletLotus" | "Damaru"
                )
            })
        {
            return EventCardColor::Purple;
        }

        let deck = &self.run_state.deck;
        if deck.iter().any(|card| matches!(card.as_str(), "Bash" | "Strike" | "Defend")) {
            return EventCardColor::Red;
        }
        if deck
            .iter()
            .any(|card| matches!(card.as_str(), "Neutralize" | "Strike" | "Defend"))
        {
            return EventCardColor::Green;
        }
        if deck.iter().any(|card| matches!(card.as_str(), "Zap" | "Strike" | "Defend")) {
            return EventCardColor::Blue;
        }
        if deck
            .iter()
            .any(|card| matches!(card.as_str(), "Eruption" | "Strike" | "Defend"))
        {
            return EventCardColor::Purple;
        }

        EventCardColor::Purple
    }

    fn match_and_keep_starter_card(&self, color: EventCardColor) -> &'static str {
        match color {
            EventCardColor::Red => "Bash",
            EventCardColor::Green => "Neutralize",
            EventCardColor::Blue => "Zap",
            EventCardColor::Purple | EventCardColor::Colorless | EventCardColor::Curse => "Eruption",
        }
    }

    fn random_match_and_keep_colored_card(
        &mut self,
        color: EventCardColor,
        rarity: EventCardRarity,
    ) -> String {
        let mut candidates = matching_event_cards(color, rarity);
        if candidates.is_empty() {
            candidates = match (color, rarity) {
                (EventCardColor::Purple, EventCardRarity::Common) => {
                    WATCHER_COMMON_CARDS.iter().map(|id| (*id).to_string()).collect()
                }
                (EventCardColor::Purple, EventCardRarity::Uncommon) => {
                    WATCHER_UNCOMMON_CARDS.iter().map(|id| (*id).to_string()).collect()
                }
                (EventCardColor::Purple, EventCardRarity::Rare) => {
                    WATCHER_RARE_CARDS.iter().map(|id| (*id).to_string()).collect()
                }
                (EventCardColor::Colorless, EventCardRarity::Uncommon) => {
                    MATCH_AND_KEEP_COLORLESS_UNCOMMON_CARDS
                        .iter()
                        .map(|id| (*id).to_string())
                        .collect()
                }
                _ => vec![self.match_and_keep_starter_card(color).to_string()],
            };
        }
        let idx = self.rng.gen_range(0..candidates.len());
        candidates[idx].clone()
    }

    fn random_match_and_keep_curse(&mut self) -> String {
        let idx = self.rng.gen_range(0..MATCH_AND_KEEP_CURSES.len());
        MATCH_AND_KEEP_CURSES[idx].to_string()
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
        let options = if state.leave_only {
            vec![crate::events::TypedEventOption::supported(
                "Leave",
                crate::events::EventProgram::from_ops(vec![crate::events::EventProgramOp::nothing()]),
                crate::events::EventEffect::Nothing,
            )]
        } else {
            vec![
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
            ]
        };
        self.current_event = Some(TypedEventDef {
            name: "Scrap Ooze".to_string(),
            options,
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
                    leave_only: false,
                });
                self.sync_scrap_ooze_event();
            }
        }
    }

    fn enter_mystery_room(&mut self) {
        // EventHelper.java::roll fills a 100-entry table in this order:
        // MONSTER, SHOP, TREASURE, then EVENT. (Elite entries require the
        // Deadly Events modifier, which the standard run engine does not use.)
        let roll = self.next_event_roll_100() as i32;
        let monster_end = self.run_state.event_monster_chance.min(100);
        let shop_end = (monster_end + self.run_state.event_shop_chance).min(100);
        let treasure_end = (shop_end + self.run_state.event_treasure_chance).min(100);

        #[derive(Clone, Copy, PartialEq, Eq)]
        enum MysteryResult { Monster, Shop, Treasure, Event }

        let rolled = if roll < monster_end {
            MysteryResult::Monster
        } else if roll < shop_end {
            MysteryResult::Shop
        } else if roll < treasure_end {
            MysteryResult::Treasure
        } else {
            MysteryResult::Event
        };

        // EventHelper resets MONSTER_CHANCE on a monster roll before Juzu
        // converts that result to EVENT; other rolls ramp it by 10 points.
        let result = if rolled == MysteryResult::Monster {
            self.run_state.event_monster_chance = 10;
            if self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::JUZU_BRACELET)
            {
                MysteryResult::Event
            } else {
                MysteryResult::Monster
            }
        } else {
            self.run_state.event_monster_chance += 10;
            rolled
        };
        self.run_state.event_shop_chance = if result == MysteryResult::Shop {
            3
        } else {
            self.run_state.event_shop_chance + 3
        };
        self.run_state.event_treasure_chance = if result == MysteryResult::Treasure {
            2
        } else {
            self.run_state.event_treasure_chance + 2
        };

        match result {
            MysteryResult::Monster => self.enter_combat(false, false),
            MysteryResult::Shop => self.enter_shop(),
            MysteryResult::Treasure => {
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
                    let gold = self.rng.gen_range(50..=80);
                    self.adjust_run_gold(gold);
                    self.phase = RunPhase::MapChoice;
                }
            }
            MysteryResult::Event => self.enter_event(),
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
                        obtain_master_deck_card_state(&mut self.run_state, first_card);
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

        if state_snapshot.leave_only {
            self.current_event = None;
            self.scrap_ooze_state = None;
            self.phase = RunPhase::MapChoice;
            self.refresh_decision_stack();
            return 0.0;
        }

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
                    let relic_id = self.roll_reward_relic_id();
                    self.add_relic_reward(&relic_id);
                    self.scrap_ooze_state = Some(ScrapOozeState {
                        damage: state_snapshot.damage,
                        relic_chance: state_snapshot.relic_chance,
                        leave_only: true,
                    });
                    self.sync_scrap_ooze_event();
                    self.phase = RunPhase::Event;
                } else {
                    self.scrap_ooze_state = Some(ScrapOozeState {
                        damage: state_snapshot.damage + 1,
                        relic_chance: state_snapshot.relic_chance + 10,
                        leave_only: false,
                    });
                    self.sync_scrap_ooze_event();
                    self.phase = RunPhase::Event;
                }
                self.refresh_decision_stack();
                0.0
            }
            1 => {
                self.scrap_ooze_state = Some(ScrapOozeState {
                    damage: state_snapshot.damage,
                    relic_chance: state_snapshot.relic_chance,
                    leave_only: true,
                });
                self.sync_scrap_ooze_event();
                self.phase = RunPhase::Event;
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
                self.adjust_run_gold(*amount);
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
                self.adjust_run_gold(amount);
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
                self.adjust_run_gold(*gold);
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
                self.adjust_run_gold(-50);
                let owner_wins = self.rng.gen_bool(0.3);
                if (*bet_on_owner && owner_wins) || (!*bet_on_owner && !owner_wins) {
                    let payout = if *bet_on_owner { 250 } else { 100 };
                    self.adjust_run_gold(payout);
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
                    obtain_master_deck_card_state(
                        &mut self.run_state,
                        WATCHER_COMMON_CARDS[idx].to_string(),
                    );
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
                    obtain_master_deck_card_state(
                        &mut self.run_state,
                        WATCHER_COMMON_CARDS[card_idx].to_string(),
                    );
                }
            }
            EventDeckMutation::DuplicateCard { count } => {
                for _ in 0..*count {
                    if !self.run_state.deck.is_empty() {
                        let idx = self.rng.gen_range(0..self.run_state.deck.len());
                        let card = self.run_state.deck[idx].clone();
                        obtain_master_deck_card_state(&mut self.run_state, card);
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
                self.adjust_run_gold(*amount);
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
                obtain_master_deck_card_state(&mut self.run_state, label.clone());
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
            "Blue Candle",
            "Bottled Flame",
            "Bottled Lightning",
            "Bottled Tornado",
            "DataDisk",
            // OrnamentalFan.java declares canonical ID "Ornamental Fan".
            "Ornamental Fan",
        ];

        let registry = gameplay_registry();
        let mut candidates: Vec<&str> = UNCOMMON_EVENT_RELIC_POOL
            .iter()
            .copied()
            .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
            .filter(|id| *id != "Bottled Flame" || self.can_spawn_bottled_flame())
            .filter(|id| *id != "Bottled Lightning" || self.can_spawn_bottled_lightning())
            .filter(|id| *id != "Bottled Tornado" || self.can_spawn_bottled_tornado())
            .filter(|id| !self.run_state.relics.iter().any(|owned| owned == id))
            .collect();
        if candidates.is_empty() {
            candidates = UNCOMMON_EVENT_RELIC_POOL
                .iter()
                .copied()
                .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
                .filter(|id| *id != "Bottled Flame" || self.can_spawn_bottled_flame())
                .filter(|id| *id != "Bottled Lightning" || self.can_spawn_bottled_lightning())
                .filter(|id| *id != "Bottled Tornado" || self.can_spawn_bottled_tornado())
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
            "Bird Faced Urn",
            "Calipers",
            "Ice Cream",
            "Incense Burner",
            // ThreadAndNeedle.java declares canonical ID "Thread and Needle"
            // and RARE tier.
            "Thread and Needle",
            "Tough Bandages",
            "TungstenRod",
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

    /// RNG stream counters currently tracked by this run, keyed by the
    /// vault's canonical short names (`docs/vault/rng-system-analysis.md`).
    /// Used by `bin/trace_replay.rs` (U05) to populate `trace::PostState::rng`.
    ///
    /// While in combat, delegates to `CombatEngine::rng_counters` (which
    /// tracks `card`/`ai` distinctly); outside combat, only the run-level
    /// catch-all `rng` counter is available and is reported as `card`
    /// (today's engine does not yet separate map/shop/event/relic/etc.
    /// streams — see `docs/vault/rng-system-analysis.md` for the full
    /// 13-stream target this will grow into).
    pub fn rng_counters(&self) -> HashMap<String, u64> {
        if let Some(combat) = &self.combat_engine {
            return combat.rng_counters().into_iter().collect();
        }
        let mut counters = HashMap::new();
        counters.insert("card".to_string(), self.rng.counter as u64);
        counters
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
                skip_label: Some("Skip".to_string()),
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
    pub(crate) fn debug_enter_shop(&mut self) {
        self.enter_shop();
    }

    #[cfg(test)]
    pub(crate) fn debug_enter_specific_combat(&mut self, enemies: &[&str]) {
        self.enter_specific_combat(enemies.iter().map(|enemy| (*enemy).to_string()).collect());
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
    pub(crate) fn debug_combat_engine_mut(&mut self) -> &mut CombatEngine {
        self.combat_engine
            .as_mut()
            .expect("expected active combat engine")
    }

    #[cfg(test)]
    pub(crate) fn debug_resolve_current_combat_outcome(&mut self) -> f32 {
        self.step_combat(&RunAction::CombatAction(crate::actions::Action::EndTurn))
    }

    #[cfg(test)]
    pub(crate) fn debug_set_campfire_phase(&mut self) {
        self.enter_campfire();
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

#[derive(Debug, Clone)]
struct EventCardCatalogEntry {
    id: String,
    name: Option<String>,
    rarity: EventCardRarity,
    color: EventCardColor,
}

fn parse_event_card_catalog() -> Vec<EventCardCatalogEntry> {
    include_str!("../content/generated-cards.txt")
        .split("\n)\n")
        .filter_map(|block| {
            let Some((_, id_rest)) = block.split_once("id=\"") else {
                return None;
            };
            let Some((card_id, _)) = id_rest.split_once('"') else {
                return None;
            };
            let card_name = block
                .split_once("name=\"")
                .and_then(|(_, rest)| rest.split_once('"'))
                .map(|(name, _)| name.to_string());
            let rarity = block
                .split_once("rarity=CardRarity.")
                .and_then(|(_, rarity_rest)| {
                    Some(
                        match rarity_rest
                            .split(|ch: char| !matches!(ch, 'A'..='Z' | '_'))
                            .next()
                            .unwrap_or("")
                        {
                            "CURSE" => EventCardRarity::Curse,
                            "BASIC" => EventCardRarity::Basic,
                            "COMMON" => EventCardRarity::Common,
                            "SPECIAL" => EventCardRarity::Special,
                            "UNCOMMON" => EventCardRarity::Uncommon,
                            "RARE" => EventCardRarity::Rare,
                            _ => return None,
                        },
                    )
                })?;
            let color = if let Some((_, color_rest)) = block.split_once("color=CardColor.") {
                match color_rest
                    .split(|ch: char| !matches!(ch, 'A'..='Z' | '_'))
                    .next()
                    .unwrap_or("")
                {
                    "RED" => EventCardColor::Red,
                    "GREEN" => EventCardColor::Green,
                    "BLUE" => EventCardColor::Blue,
                    "PURPLE" => EventCardColor::Purple,
                    "COLORLESS" => EventCardColor::Colorless,
                    "CURSE" => EventCardColor::Curse,
                    _ => EventCardColor::Purple,
                }
            } else {
                EventCardColor::Purple
            };

            Some(EventCardCatalogEntry {
                id: card_id.to_string(),
                name: card_name,
                rarity,
                color,
            })
        })
        .collect()
}

fn build_event_card_rarity_map() -> HashMap<String, EventCardRarity> {
    let mut map = HashMap::new();
    for entry in parse_event_card_catalog() {
        map.insert(entry.id, entry.rarity);
        if let Some(card_name) = entry.name {
            map.insert(card_name, entry.rarity);
        }
    }
    map
}

fn event_card_color(card_id: &str) -> Option<EventCardColor> {
    static CARD_COLORS: OnceLock<HashMap<String, EventCardColor>> = OnceLock::new();

    CARD_COLORS
        .get_or_init(build_event_card_color_map)
        .get(card_id.trim_end_matches('+'))
        .copied()
}

fn build_event_card_color_map() -> HashMap<String, EventCardColor> {
    let mut map = HashMap::new();
    for entry in parse_event_card_catalog() {
        map.insert(entry.id, entry.color);
        if let Some(card_name) = entry.name {
            map.insert(card_name, entry.color);
        }
    }
    map
}

fn matching_event_cards(color: EventCardColor, rarity: EventCardRarity) -> Vec<String> {
    let registry = gameplay_registry();
    let mut cards: Vec<String> = registry
        .defs_for_domain(GameplayDomain::Card)
        .filter(|def| {
            event_card_color(def.id.as_str()) == Some(color)
                && event_card_rarity(def.id.as_str()) == Some(rarity)
        })
        .map(|def| def.id.clone())
        .collect();
    cards.sort();
    cards
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
    fn fruit_juice_is_usable_outside_combat_except_we_meet_again() {
        // Source-derived (verify potion/FruitJuice): canUse only rejects an
        // ended combat turn and WeMeetAgain; use increases max HP by constant
        // potency five through increaseMaxHp. Sacred Bark doubles potency;
        // MagicFlower.java does not modify healing outside RoomPhase.COMBAT.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FruitJuice.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/core/AbstractCreature.java
        let mut engine = RunEngine::new(42, 20);
        resolve_opening_neow(&mut engine);
        engine.run_state.current_hp = 40;
        engine.run_state.max_hp = 80;
        engine.run_state.potions[0] = "FruitJuice".to_string();
        engine.run_state.relics.push("SacredBark".to_string());
        engine.run_state.relics.push("Magic Flower".to_string());
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);

        assert!(engine
            .get_legal_actions()
            .contains(&RunAction::UsePotion(0)));
        let result = engine.step_with_result(&RunAction::UsePotion(0));

        assert!(result.action_accepted);
        assert_eq!(engine.run_state.max_hp, 90);
        assert_eq!(engine.run_state.current_hp, 50);
        assert!(engine.run_state.potions[0].is_empty());
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);

        engine.run_state.potions[0] = "Fruit Juice".to_string();
        engine.debug_set_event_state(EventDef {
            name: "WeMeetAgain".to_string(),
            options: vec![EventOption {
                text: "Leave".to_string(),
                effect: EventEffect::Nothing,
            }],
        });
        assert!(!engine
            .get_legal_actions()
            .contains(&RunAction::UsePotion(0)));
    }

    #[test]
    fn smoke_bomb_escapes_without_losing_run_or_receiving_combat_rewards() {
        // Source-derived (verify potion/SmokeBomb): use marks the current room
        // smoked and the player escaping. It does not kill the player or win
        // the fight. Combat-owned state still persists when the room exits.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/SmokeBomb.java
        let mut engine = RunEngine::new(42, 20);
        resolve_opening_neow(&mut engine);
        let max_before = engine.run_state.max_hp;
        let hp_before = engine.run_state.current_hp;
        engine.run_state.potions[0] = "FruitJuice".to_string();
        engine.run_state.potions[1] = "SmokeBomb".to_string();
        engine.enter_specific_combat(vec!["JawWorm".to_string()]);

        let fruit = RunAction::CombatAction(crate::actions::Action::UsePotion {
            potion_idx: 0,
            target_idx: -1,
        });
        assert!(engine.step_with_result(&fruit).action_accepted);
        let smoke = RunAction::CombatAction(crate::actions::Action::UsePotion {
            potion_idx: 1,
            target_idx: -1,
        });
        let result = engine.step_with_result(&smoke);

        assert!(result.action_accepted);
        assert_eq!(result.reward, 0.0);
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
        assert!(!engine.run_state.run_over);
        assert_eq!(engine.run_state.combats_won, 0);
        assert_eq!(engine.run_state.max_hp, max_before + 5);
        assert_eq!(engine.run_state.current_hp, hp_before + 5);
        assert!(engine.run_state.potions[0].is_empty());
        assert!(engine.run_state.potions[1].is_empty());
        assert!(engine.combat_engine.is_none());
        assert!(engine.reward_screen.is_none());
    }

    #[test]
    fn jaw_worm_constructor_ranges_and_ascension_stats_match_java() {
        // Source: reference/extracted/methods/monster/JawWorm.java (constructor):
        // HP 40..44 (42..46 at A7), 11/12 Chomp at A2, 3/4/5 Strength at
        // A0/A2/A17, and Bellow block 6/9 at A0/A17.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("JawWorm").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("JawWorm").0);
        }
        assert!(low_hp.iter().all(|hp| (40..=44).contains(hp)));
        assert!(high_hp.iter().all(|hp| (42..=46).contains(hp)));
        assert!(low_hp.contains(&40) && low_hp.contains(&44));
        assert!(high_hp.contains(&42) && high_hp.contains(&46));

        for (ascension, damage, strength, block) in
            [(0, 11, 3, 6), (2, 12, 4, 6), (17, 12, 5, 9)]
        {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["JawWorm".to_string()]);
            let enemy = &engine.combat_engine.as_ref().unwrap().state.enemies[0];
            assert_eq!(enemy.move_damage(), damage);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), damage);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), strength);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), block);
        }
    }

    #[test]
    fn fungi_beast_constructor_ranges_and_ascension_stats_match_java() {
        // Source: reference/extracted/methods/monster/FungiBeast.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("FungiBeast").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("FungiBeast").0);
        }
        assert!(low_hp.iter().all(|hp| (22..=28).contains(hp)));
        assert!(high_hp.iter().all(|hp| (24..=28).contains(hp)));
        assert!(low_hp.contains(&22) && low_hp.contains(&28));
        assert!(high_hp.contains(&24) && high_hp.contains(&28));

        for (ascension, strength) in [(0, 3), (2, 4), (17, 5)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["FungiBeast".to_string()]);
            let enemy = &engine.combat_engine.as_ref().unwrap().state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), strength);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::SPORE_CLOUD), 2);
            assert!(enemy.move_history.is_empty());
            assert!(matches!(enemy.move_id,
                crate::enemies::move_ids::FB_BITE | crate::enemies::move_ids::FB_GROW));
            assert_eq!(engine.combat_engine.as_ref().unwrap().ai_rng.counter, 1);
        }
    }

    #[test]
    fn louse_constructor_rolls_and_a17_rules_match_java() {
        // Sources: reference/extracted/methods/monster/LouseNormal.java and
        // LouseDefensive.java.
        let mut red_low_hp = std::collections::HashSet::new();
        let mut red_high_hp = std::collections::HashSet::new();
        let mut green_low_hp = std::collections::HashSet::new();
        let mut green_high_hp = std::collections::HashSet::new();
        let mut low_bites = std::collections::HashSet::new();
        let mut low_curl = std::collections::HashSet::new();
        let mut a17_bites = std::collections::HashSet::new();
        let mut a17_curl = std::collections::HashSet::new();

        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            red_low_hp.insert(low.roll_enemy_hp("FuzzyLouseNormal").0);
            green_low_hp.insert(low.roll_enemy_hp("FuzzyLouseDefensive").0);
            low.enter_specific_combat(vec!["FuzzyLouseNormal".to_string()]);
            let enemy = &low.combat_engine.as_ref().unwrap().state.enemies[0];
            low_bites.insert(enemy.entity.status(crate::status_ids::sid::STARTING_DMG));
            low_curl.insert(enemy.entity.status(crate::status_ids::sid::CURL_UP));

            let mut high = RunEngine::new(seed, 17);
            red_high_hp.insert(high.roll_enemy_hp("FuzzyLouseNormal").0);
            green_high_hp.insert(high.roll_enemy_hp("FuzzyLouseDefensive").0);
            high.enter_specific_combat(vec!["FuzzyLouseDefensive".to_string()]);
            let enemy = &high.combat_engine.as_ref().unwrap().state.enemies[0];
            a17_bites.insert(enemy.entity.status(crate::status_ids::sid::STARTING_DMG));
            a17_curl.insert(enemy.entity.status(crate::status_ids::sid::CURL_UP));
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), 4);
            assert_eq!(high.combat_engine.as_ref().unwrap().ai_rng.counter, 1);
        }

        assert_eq!(red_low_hp, (10..=15).collect());
        assert_eq!(red_high_hp, (11..=16).collect());
        assert_eq!(green_low_hp, (11..=17).collect());
        assert_eq!(green_high_hp, (12..=18).collect());
        assert_eq!(low_bites, (5..=7).collect());
        assert_eq!(a17_bites, (6..=8).collect());
        assert_eq!(low_curl, (3..=7).collect());
        assert_eq!(a17_curl, (9..=12).collect());

        let mut a7 = RunEngine::new(42, 7);
        a7.enter_specific_combat(vec!["FuzzyLouseNormal".to_string()]);
        let curl = a7.combat_engine.as_ref().unwrap().state.enemies[0]
            .entity.status(crate::status_ids::sid::CURL_UP);
        assert!((4..=8).contains(&curl));
    }

    #[test]
    fn blue_slaver_constructor_and_ascension_stats_match_java() {
        // Source: reference/extracted/methods/monster/SlaverBlue.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("SlaverBlue").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("SlaverBlue").0);
        }
        assert_eq!(low_hp, (46..=50).collect());
        assert_eq!(high_hp, (48..=52).collect());

        for (ascension, stab, rake, weak) in
            [(0, 12, 7, 1), (2, 13, 8, 1), (17, 13, 8, 2)]
        {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["SlaverBlue".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), stab);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), rake);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), weak);
            assert!(matches!(enemy.move_id,
                crate::enemies::move_ids::BS_STAB | crate::enemies::move_ids::BS_RAKE));
            assert_eq!(combat.ai_rng.counter, 1);
        }
    }

    #[test]
    fn red_slaver_constructor_and_first_turn_match_java() {
        // Source: reference/extracted/methods/monster/SlaverRed.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("SlaverRed").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("SlaverRed").0);
        }
        assert_eq!(low_hp, (46..=50).collect());
        assert_eq!(high_hp, (48..=52).collect());

        for (ascension, stab, scrape, vulnerable) in
            [(0, 13, 8, 1), (2, 14, 9, 1), (17, 14, 9, 2)]
        {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["SlaverRed".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.move_id, crate::enemies::move_ids::RS_STAB);
            assert_eq!(enemy.move_damage(), stab);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), scrape);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), vulnerable);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::IS_FIRST_MOVE), 0);
            assert_eq!(combat.ai_rng.counter, 1);
        }
    }

    #[test]
    fn acid_slime_s_initial_rng_then_direct_alternation_matches_java() {
        // Source: reference/extracted/methods/monster/AcidSlime_S.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("AcidSlime_S").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("AcidSlime_S").0);
        }
        assert_eq!(low_hp, (8..=12).collect());
        assert_eq!(high_hp, (9..=13).collect());

        for (ascension, damage, initial_ticks) in [(0, 3, 2), (2, 4, 2), (17, 4, 1)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["AcidSlime_S".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let initial = combat.state.enemies[0].move_id;
            assert!(matches!(initial,
                crate::enemies::move_ids::AS_S_TACKLE
                    | crate::enemies::move_ids::AS_S_LICK));
            assert_eq!(combat.state.enemies[0].entity.status(
                crate::status_ids::sid::STARTING_DMG), damage);
            assert_eq!(combat.ai_rng.counter, initial_ticks);

            engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
            let combat = engine.combat_engine.as_ref().unwrap();
            let expected = if initial == crate::enemies::move_ids::AS_S_TACKLE {
                crate::enemies::move_ids::AS_S_LICK
            } else {
                crate::enemies::move_ids::AS_S_TACKLE
            };
            assert_eq!(combat.state.enemies[0].move_id, expected);
            assert_eq!(combat.ai_rng.counter, initial_ticks,
                "takeTurn directly sets the next move without RollMoveAction");
        }
    }

    #[test]
    fn acid_slime_m_ranges_stats_and_conditional_rng_match_java() {
        // Source: reference/extracted/methods/monster/AcidSlime_M.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("AcidSlime_M").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("AcidSlime_M").0);
        }
        assert_eq!(low_hp, (28..=32).collect());
        assert_eq!(high_hp, (29..=34).collect());

        for (ascension, wound, normal, marker) in
            [(0, 7, 10, 0), (2, 8, 12, 0), (17, 8, 12, 17)]
        {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["AcidSlime_M".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), wound);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), normal);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), marker);
            assert_eq!(combat.ai_rng.counter, 1);
        }

        // Drive a real enemy turn where the 30..69 window follows Normal
        // Tackle with Java's conditional 0.4 draw.
        let seed = (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            let num = rng.random(99);
            (30..70).contains(&num)
        }).unwrap();
        let mut probe = crate::seed::StsRandom::new(seed);
        let _ = probe.random(99);
        let expected_wound = probe.random_float() < 0.4;

        let mut engine = RunEngine::new(42, 0);
        engine.enter_specific_combat(vec!["AcidSlime_M".to_string()]);
        {
            let combat = engine.combat_engine.as_mut().unwrap();
            let enemy = &mut combat.state.enemies[0];
            enemy.move_history.clear();
            enemy.set_move(crate::enemies::move_ids::AS_TACKLE, 10, 1, 0);
            enemy.move_effects.clear();
            combat.ai_rng = crate::seed::StsRandom::new(seed);
        }
        engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = engine.combat_engine.as_ref().unwrap();
        assert_eq!(combat.ai_rng.counter, 2);
        assert_eq!(combat.state.enemies[0].move_id, if expected_wound {
            crate::enemies::move_ids::AS_CORROSIVE_SPIT
        } else {
            crate::enemies::move_ids::AS_LICK
        });
    }

    #[test]
    fn acid_slime_l_stats_ai_and_half_hp_split_match_java() {
        // Source: reference/extracted/methods/monster/AcidSlime_L.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("AcidSlime_L").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("AcidSlime_L").0);
        }
        assert_eq!(low_hp, (65..=69).collect());
        assert_eq!(high_hp, (68..=72).collect());

        for (ascension, wound, normal, marker) in
            [(0, 11, 16, 0), (2, 12, 18, 0), (17, 12, 18, 17)]
        {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["AcidSlime_L".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), wound);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), normal);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), marker);
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut engine = RunEngine::new(42, 17);
        engine.enter_specific_combat(vec!["AcidSlime_L".to_string()]);
        let (half_hp, ticks_before) = {
            let combat = engine.combat_engine.as_mut().unwrap();
            let enemy = &mut combat.state.enemies[0];
            let half = enemy.entity.max_hp / 2;
            enemy.entity.hp = half + 1;
            let ticks = combat.ai_rng.counter;
            combat.deal_damage_to_enemy(0, 1);
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::AS_SPLIT);
            (half, ticks)
        };
        engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = engine.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_eq!(combat.state.enemies.len(), 3);
        for child in &combat.state.enemies[1..] {
            assert_eq!(child.id, "AcidSlime_M");
            assert_eq!((child.entity.hp, child.entity.max_hp), (half_hp, half_hp));
            assert_eq!(child.entity.status(crate::status_ids::sid::STARTING_DMG), 8);
            assert_eq!(child.entity.status(crate::status_ids::sid::STR_AMT), 12);
            assert_eq!(child.entity.status(crate::status_ids::sid::BLOCK_AMT), 17);
        }
        assert_eq!(combat.ai_rng.counter, ticks_before + 2,
            "each spawned medium slime initializes with one aiRng roll");
    }

    #[test]
    fn spike_slime_s_ranges_damage_and_ai_ticks_match_java() {
        // Source: reference/extracted/methods/monster/SpikeSlime_S.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("SpikeSlime_S").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("SpikeSlime_S").0);
        }
        assert_eq!(low_hp, (10..=14).collect());
        assert_eq!(high_hp, (11..=15).collect());

        for (ascension, damage) in [(0, 5), (2, 6)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["SpikeSlime_S".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::SS_TACKLE);
            assert_eq!(combat.state.enemies[0].move_damage(), damage);
            assert_eq!(combat.ai_rng.counter, 1);

            engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::SS_TACKLE);
            assert_eq!(combat.ai_rng.counter, 2);
        }
    }

    #[test]
    fn spike_slime_m_stats_tackle_and_ai_ticks_match_java() {
        // Source: reference/extracted/methods/monster/SpikeSlime_M.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("SpikeSlime_M").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("SpikeSlime_M").0);
        }
        assert_eq!(low_hp, (28..=32).collect());
        assert_eq!(high_hp, (29..=34).collect());

        for (ascension, damage, marker) in [(0, 8, 0), (2, 10, 0), (17, 10, 17)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["SpikeSlime_M".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), damage);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), marker);
            assert!(matches!(enemy.move_id,
                crate::enemies::move_ids::SS_TACKLE | crate::enemies::move_ids::SS_LICK));
            assert_eq!(combat.ai_rng.counter, 1,
                "AbstractMonster.init performs one opening roll");
        }

        let mut engine = RunEngine::new(42, 0);
        engine.enter_specific_combat(vec!["SpikeSlime_M".to_string()]);
        let ticks_before = {
            let combat = engine.combat_engine.as_mut().unwrap();
            let enemy = &mut combat.state.enemies[0];
            enemy.move_history.clear();
            enemy.set_move(crate::enemies::move_ids::SS_TACKLE, 8, 1, 0);
            enemy.move_effects.clear();
            enemy.add_effect(crate::combat_types::mfx::SLIMED, 1);
            combat.ai_rng.counter
        };
        engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = engine.combat_engine.as_ref().unwrap();
        assert_eq!(combat.ai_rng.counter, ticks_before + 1,
            "RollMoveAction consumes one aiRng draw after Tackle");
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Slimed").count(), 1);
    }

    #[test]
    fn spike_slime_l_stats_tackle_ai_and_half_hp_split_match_java() {
        // Source: reference/extracted/methods/monster/SpikeSlime_L.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("SpikeSlime_L").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("SpikeSlime_L").0);
        }
        assert_eq!(low_hp, (64..=70).collect());
        assert_eq!(high_hp, (67..=73).collect());

        for (ascension, damage, frail, marker) in
            [(0, 16, 2, 0), (2, 18, 2, 0), (17, 18, 3, 17)]
        {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["SpikeSlime_L".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), damage);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), frail);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), marker);
            assert_eq!(combat.ai_rng.counter, 1,
                "AbstractMonster.init performs one opening roll");
        }

        let mut engine = RunEngine::new(42, 0);
        engine.enter_specific_combat(vec!["SpikeSlime_L".to_string()]);
        let ticks_before = {
            let combat = engine.combat_engine.as_mut().unwrap();
            let enemy = &mut combat.state.enemies[0];
            enemy.move_history.clear();
            enemy.set_move(crate::enemies::move_ids::SS_TACKLE, 16, 1, 0);
            enemy.move_effects.clear();
            enemy.add_effect(crate::combat_types::mfx::SLIMED, 2);
            combat.ai_rng.counter
        };
        engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = engine.combat_engine.as_ref().unwrap();
        assert_eq!(combat.ai_rng.counter, ticks_before + 1);
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Slimed").count(), 2);

        let mut engine = RunEngine::new(42, 17);
        engine.enter_specific_combat(vec!["SpikeSlime_L".to_string()]);
        let (half_hp, ticks_before) = {
            let combat = engine.combat_engine.as_mut().unwrap();
            let enemy = &mut combat.state.enemies[0];
            let half = enemy.entity.max_hp / 2;
            enemy.entity.hp = half + 1;
            let ticks = combat.ai_rng.counter;
            combat.deal_damage_to_enemy(0, 1);
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::SS_SPLIT);
            (half, ticks)
        };
        engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = engine.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_eq!(combat.state.enemies.len(), 3);
        for child in &combat.state.enemies[1..] {
            assert_eq!(child.id, "SpikeSlime_M");
            assert_eq!((child.entity.hp, child.entity.max_hp), (half_hp, half_hp));
            assert_eq!(child.entity.status(crate::status_ids::sid::STARTING_DMG), 10);
            assert_eq!(child.entity.status(crate::status_ids::sid::BLOCK_AMT), 17);
        }
        assert_eq!(combat.ai_rng.counter, ticks_before + 2,
            "each spawned medium slime initializes with one aiRng roll");
    }

    #[test]
    fn looter_stats_gold_theft_moves_and_ai_ticks_match_java() {
        // Source: reference/extracted/methods/monster/Looter.java and
        // decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAction.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("Looter").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("Looter").0);
        }
        assert_eq!(low_hp, (44..=48).collect());
        assert_eq!(high_hp, (46..=50).collect());

        for (ascension, swipe, lunge, gold) in
            [(0, 10, 12, 15), (2, 11, 14, 15), (17, 11, 14, 20)]
        {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["Looter".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), swipe);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), lunge);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), 6);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::TURN_COUNT), gold);
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let seed = (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            let _ = rng.random_float();
            rng.random_float() < 0.5
        }).unwrap();
        let mut engine = RunEngine::new(42, 0);
        engine.enter_specific_combat(vec!["Looter".to_string()]);
        engine.combat_engine.as_mut().unwrap().ai_rng = crate::seed::StsRandom::new(seed);

        engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(engine.run_state.gold, 84);
        {
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::LOOTER_MUG);
            assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::COUNT), 15);
            assert_eq!(combat.ai_rng.counter, 1,
                "first Mug consumes only its 0.6 dialogue boolean");
        }

        engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(engine.run_state.gold, 69);
        {
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::LOOTER_SMOKE_BOMB);
            assert_eq!(combat.state.enemies[0].move_block(), 6);
            assert_eq!(combat.ai_rng.counter, 2,
                "second Mug consumes only its 0.5 branch boolean");
        }

        engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = engine.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.enemies[0].entity.block, 6);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LOOTER_ESCAPE);
        assert!(!combat.state.enemies[0].is_escaping,
            "Smoke Bomb announces Escape; the following turn performs it");
        assert_eq!(combat.ai_rng.counter, 2);

        let mut refund = RunEngine::new(42, 0);
        refund.enter_specific_combat(vec!["Looter".to_string()]);
        refund.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        refund.combat_engine.as_mut().unwrap().state.enemies[0].entity.hp = 0;
        refund.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        assert!((109..=119).contains(&refund.run_state.gold),
            "death returns 15 stolen gold before the normal 10..=20 reward");
    }

    #[test]
    fn gremlin_fat_stats_debuffs_ai_ticks_and_ally_death_escape_match_java() {
        // Source: reference/extracted/methods/monster/GremlinFat.java and the
        // full source's `deathReact`/`takeTurn` escape case.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("GremlinFat").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("GremlinFat").0);
        }
        assert_eq!(low_hp, (13..=17).collect());
        assert_eq!(high_hp, (14..=18).collect());

        for (ascension, damage, frail) in [(0, 4, false), (2, 5, false), (17, 5, true)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["GremlinFat".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.move_id, crate::enemies::move_ids::GREMLIN_FAT_SMASH);
            assert_eq!(enemy.move_damage(), damage);
            assert_eq!(enemy.effect(crate::combat_types::mfx::WEAK), Some(1));
            assert_eq!(enemy.effect(crate::combat_types::mfx::FRAIL),
                frail.then_some(1));
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut a17 = RunEngine::new(42, 17);
        a17.enter_specific_combat(vec!["GremlinFat".to_string()]);
        a17.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = a17.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.player.status(crate::status_ids::sid::WEAKENED), 1);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::FRAIL), 1);
        assert_eq!(combat.ai_rng.counter, 2,
            "the normal attack queues exactly one RollMoveAction");

        let mut group = RunEngine::new(42, 0);
        group.enter_specific_combat(vec!["GremlinFat".to_string(),
            "GremlinThief".to_string()]);
        let combat = group.combat_engine.as_mut().unwrap();
        combat.state.enemies[1].entity.hp = 1;
        combat.deal_damage_to_enemy(1, 1);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ESCAPE);
        crate::combat_hooks::do_enemy_turns(combat);
        assert!(combat.state.enemies[0].is_escaping);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
    }

    #[test]
    fn gremlin_thief_stats_direct_repeat_and_ally_death_escape_match_java() {
        // Source: reference/extracted/methods/monster/GremlinThief.java and the
        // full source's `deathReact`/`takeTurn` escape case.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("GremlinThief").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("GremlinThief").0);
        }
        assert_eq!(low_hp, (10..=14).collect());
        assert_eq!(high_hp, (11..=15).collect());

        for (ascension, damage) in [(0, 9), (2, 10)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["GremlinThief".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies[0].move_damage(), damage);
            assert_eq!(combat.ai_rng.counter, 1);

            engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::GREMLIN_ATTACK);
            assert_eq!(combat.state.enemies[0].move_damage(), damage);
            assert_eq!(combat.ai_rng.counter, 1,
                "Puncture sets Puncture directly without RollMoveAction");
        }

        let mut group = RunEngine::new(42, 0);
        group.enter_specific_combat(vec!["GremlinThief".to_string(),
            "GremlinWarrior".to_string()]);
        let combat = group.combat_engine.as_mut().unwrap();
        combat.state.enemies[1].entity.hp = 1;
        combat.deal_damage_to_enemy(1, 1);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ESCAPE);
        crate::combat_hooks::do_enemy_turns(combat);
        assert!(combat.state.enemies[0].is_escaping);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
    }

    #[test]
    fn gremlin_warrior_stats_angry_direct_repeat_and_escape_match_java() {
        // Source: reference/extracted/methods/monster/GremlinWarrior.java and
        // the full source's `deathReact` escape behavior.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("GremlinWarrior").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("GremlinWarrior").0);
        }
        assert_eq!(low_hp, (20..=24).collect());
        assert_eq!(high_hp, (21..=25).collect());

        for (ascension, damage, angry) in [(0, 4, 1), (2, 5, 1), (17, 5, 2)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["GremlinWarrior".to_string()]);
            {
                let combat = engine.combat_engine.as_mut().unwrap();
                assert_eq!(combat.state.enemies[0].move_damage(), damage);
                assert_eq!(combat.state.enemies[0].entity.status(
                    crate::status_ids::sid::ANGRY), angry);
                assert_eq!(combat.ai_rng.counter, 1);
                combat.deal_damage_to_enemy(0, 1);
                assert_eq!(combat.state.enemies[0].entity.status(
                    crate::status_ids::sid::STRENGTH), angry);
            }

            engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::GREMLIN_ATTACK);
            assert_eq!(combat.ai_rng.counter, 1,
                "Scratch sets Scratch directly without RollMoveAction");
        }

        let mut group = RunEngine::new(42, 0);
        group.enter_specific_combat(vec!["GremlinWarrior".to_string(),
            "GremlinThief".to_string()]);
        let combat = group.combat_engine.as_mut().unwrap();
        combat.state.enemies[1].entity.hp = 1;
        combat.deal_damage_to_enemy(1, 1);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ESCAPE);
        crate::combat_hooks::do_enemy_turns(combat);
        assert!(combat.state.enemies[0].is_escaping);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
    }

    #[test]
    fn gremlin_wizard_charge_cycle_a17_repeat_and_escape_match_java() {
        // Source: reference/extracted/methods/monster/GremlinWizard.java and
        // the full source's initial charge and `deathReact` fields.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("GremlinWizard").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("GremlinWizard").0);
        }
        assert_eq!(low_hp, (21..=25).collect());
        assert_eq!(high_hp, (22..=26).collect());

        for (ascension, damage, marker) in [(0, 25, 0), (2, 30, 0), (17, 30, 17)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["GremlinWizard".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.move_id, crate::enemies::move_ids::GREMLIN_PROTECT);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), damage);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::COUNT), 1);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), marker);
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut low = RunEngine::new(42, 0);
        low.enter_specific_combat(vec!["GremlinWizard".to_string()]);
        low.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(low.combat_engine.as_ref().unwrap().state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_PROTECT);
        low.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(low.combat_engine.as_ref().unwrap().state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ATTACK);
        low.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = low.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_PROTECT);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::COUNT), 0);
        assert_eq!(combat.ai_rng.counter, 1,
            "all charge/blast transitions use direct SetMoveAction calls");

        let mut a17 = RunEngine::new(42, 17);
        a17.enter_specific_combat(vec!["GremlinWizard".to_string()]);
        a17.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        a17.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        a17.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = a17.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ATTACK,
            "A17 repeats Ultimate Blast immediately");
        assert_eq!(combat.ai_rng.counter, 1);

        let mut group = RunEngine::new(42, 0);
        group.enter_specific_combat(vec!["GremlinWizard".to_string(),
            "GremlinThief".to_string()]);
        let combat = group.combat_engine.as_mut().unwrap();
        combat.state.enemies[1].entity.hp = 1;
        combat.deal_damage_to_enemy(1, 1);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ESCAPE);
        crate::combat_hooks::do_enemy_turns(combat);
        assert!(combat.state.enemies[0].is_escaping);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
    }

    #[test]
    fn gremlin_tsundere_random_protect_solo_bash_and_escape_match_java() {
        // Source: reference/extracted/methods/monster/GremlinTsundere.java and
        // decompiled GainBlockRandomMonsterAction.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("GremlinTsundere").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("GremlinTsundere").0);
        }
        assert_eq!(low_hp, (12..=15).collect());
        assert_eq!(high_hp, (13..=17).collect());

        for (ascension, damage, block) in [(0, 6, 7), (2, 8, 7), (7, 8, 8), (17, 8, 11)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["GremlinTsundere".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.move_id,
                crate::enemies::move_ids::GREMLIN_TSUNDERE_PROTECT);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), damage);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), block);
            assert_eq!(enemy.effect(crate::combat_types::mfx::BLOCK_RANDOM_OTHER),
                Some(block as i16));
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut solo = RunEngine::new(42, 0);
        solo.enter_specific_combat(vec!["GremlinTsundere".to_string()]);
        {
            let combat = solo.combat_engine.as_mut().unwrap();
            crate::combat_hooks::do_enemy_turns(combat);
            assert_eq!(combat.state.enemies[0].entity.block, 7);
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::GREMLIN_TSUNDERE_BASH);
            assert_eq!(combat.ai_rng.counter, 1,
                "no eligible ally means self-block without an aiRng draw");
        }

        let mut group = RunEngine::new(42, 17);
        group.enter_specific_combat(vec!["GremlinTsundere".to_string(),
            "GremlinThief".to_string(), "GremlinWarrior".to_string()]);
        let opening_ticks = group.combat_engine.as_ref().unwrap().ai_rng.counter;
        let combat = group.combat_engine.as_mut().unwrap();
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.block, 0);
        assert_eq!(combat.state.enemies[1..].iter()
            .filter(|enemy| enemy.entity.block == 11).count(), 1);
        assert_eq!(combat.ai_rng.counter, opening_ticks + 1,
            "two eligible allies require exactly one random-target draw");

        let mut escape = RunEngine::new(42, 0);
        escape.enter_specific_combat(vec!["GremlinTsundere".to_string(),
            "GremlinThief".to_string()]);
        let combat = escape.combat_engine.as_mut().unwrap();
        combat.state.enemies[1].entity.hp = 1;
        combat.deal_damage_to_enemy(1, 1);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ESCAPE);
        crate::combat_hooks::do_enemy_turns(combat);
        assert!(combat.state.enemies[0].is_escaping);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
    }

    #[test]
    fn gremlin_nob_stats_bellow_ai_enrage_and_a18_history_match_java() {
        // Source: reference/extracted/methods/monster/GremlinNob.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("GremlinNob").0);
            let mut high = RunEngine::new(seed, 8);
            high_hp.insert(high.roll_enemy_hp("GremlinNob").0);
        }
        assert_eq!(low_hp, (82..=86).collect());
        assert_eq!(high_hp, (85..=90).collect());

        for (ascension, rush, bash, enrage, marker) in
            [(0, 14, 6, 2, 0), (3, 16, 8, 2, 0), (18, 16, 8, 3, 18)]
        {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["GremlinNob".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.move_id, crate::enemies::move_ids::NOB_BELLOW);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), rush);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), bash);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::TURN_COUNT), enrage);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), marker);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ENRAGE), 0);
            assert_eq!(enemy.effect(crate::combat_types::mfx::ENRAGE),
                Some(enrage as i16));
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut a18 = RunEngine::new(42, 18);
        a18.enter_specific_combat(vec!["GremlinNob".to_string()]);
        a18.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        {
            let combat = a18.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ENRAGE), 3);
            assert_eq!(enemy.move_id, crate::enemies::move_ids::NOB_SKULL_BASH,
                "A18 forces Bash when neither of the previous two moves was Bash");
            assert_eq!(enemy.move_damage(), 8);
            assert_eq!(enemy.effect(crate::combat_types::mfx::VULNERABLE), Some(2));
            assert_eq!(combat.ai_rng.counter, 2);
        }

        {
            let combat = a18.combat_engine.as_mut().unwrap();
            let defend = combat.card_registry.make_card("Defend");
            combat.state.hand.clear();
            combat.state.hand.push(defend);
            combat.state.energy = 3;
        }
        a18.step(&RunAction::CombatAction(crate::actions::Action::PlayCard {
            card_idx: 0,
            target_idx: -1,
        }));
        assert_eq!(a18.combat_engine.as_ref().unwrap().state.enemies[0]
            .entity.status(crate::status_ids::sid::STRENGTH), 3,
            "the Enrage installed by Bellow triggers on a Skill");
    }

    #[test]
    fn lagavulin_stats_natural_wake_damage_stun_and_siphon_match_java() {
        // Source: reference/extracted/methods/monster/Lagavulin.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("Lagavulin").0);
            let mut high = RunEngine::new(seed, 8);
            high_hp.insert(high.roll_enemy_hp("Lagavulin").0);
        }
        assert_eq!(low_hp, (109..=111).collect());
        assert_eq!(high_hp, (112..=115).collect());

        for (ascension, damage, debuff) in [(0, 18, 1), (3, 20, 1), (18, 20, 2)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["Lagavulin".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.move_id, crate::enemies::move_ids::LAGA_SLEEP);
            assert_eq!(enemy.entity.block, 8);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::METALLICIZE), 8);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), damage);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), debuff);
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut natural = RunEngine::new(42, 18);
        natural.enter_specific_combat(vec!["Lagavulin".to_string()]);
        let combat = natural.combat_engine.as_mut().unwrap();
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_SLEEP);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_SLEEP);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_ATTACK);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::METALLICIZE), 0);
        assert_eq!(combat.ai_rng.counter, 3,
            "the first two idle turns roll; natural wake on idle three does not");

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_ATTACK);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_SIPHON);
        assert_eq!(combat.state.enemies[0].effect(crate::combat_types::mfx::SIPHON_STR),
            Some(2));
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::STRENGTH), -2);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::DEXTERITY), -2);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_ATTACK);
        assert_eq!(combat.ai_rng.counter, 6);

        let mut hit_wake = RunEngine::new(42, 0);
        hit_wake.enter_specific_combat(vec!["Lagavulin".to_string()]);
        let combat = hit_wake.combat_engine.as_mut().unwrap();
        combat.deal_damage_to_enemy(0, 8);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_SLEEP,
            "fully blocked damage does not change currentHealth or wake Lagavulin");
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_STUN);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::METALLICIZE), 0);
        let ticks = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_ATTACK);
        assert_eq!(combat.ai_rng.counter, ticks + 1,
            "the stunned wake turn queues one RollMoveAction");
    }

    #[test]
    fn sentry_stats_stagger_daze_artifact_and_ai_ticks_match_java() {
        // Source: reference/extracted/methods/monster/Sentry.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("Sentry").0);
            let mut high = RunEngine::new(seed, 8);
            high_hp.insert(high.roll_enemy_hp("Sentry").0);
        }
        assert_eq!(low_hp, (38..=42).collect());
        assert_eq!(high_hp, (39..=45).collect());

        for (ascension, damage, daze) in [(0, 9, 2), (3, 10, 2), (18, 10, 3)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["Sentry".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.move_id, crate::enemies::move_ids::SENTRY_BOLT);
            assert_eq!(enemy.move_damage(), 0);
            assert_eq!(enemy.effect(crate::combat_types::mfx::DAZE),
                Some(daze as i16));
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), damage);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ARTIFACT), 1);
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut single = RunEngine::new(42, 0);
        single.enter_specific_combat(vec!["Sentry".to_string()]);
        single.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = single.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Daze").count(), 2);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SENTRY_BEAM);
        assert_eq!(combat.state.enemies[0].move_damage(), 9);
        assert_eq!(combat.ai_rng.counter, 2);

        let mut trio = RunEngine::new(42, 18);
        trio.enter_specific_combat(vec!["Sentry".to_string(), "Sentry".to_string(),
            "Sentry".to_string()]);
        {
            let combat = trio.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies.iter().map(|enemy| enemy.move_id)
                .collect::<Vec<_>>(), vec![
                    crate::enemies::move_ids::SENTRY_BOLT,
                    crate::enemies::move_ids::SENTRY_BEAM,
                    crate::enemies::move_ids::SENTRY_BOLT,
                ]);
            assert_eq!(combat.ai_rng.counter, 3);
        }
        let hp_before = trio.combat_engine.as_ref().unwrap().state.player.hp;
        trio.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = trio.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.player.hp, hp_before - 10);
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Daze").count(), 6);
        assert_eq!(combat.state.enemies.iter().map(|enemy| enemy.move_id)
            .collect::<Vec<_>>(), vec![
                crate::enemies::move_ids::SENTRY_BEAM,
                crate::enemies::move_ids::SENTRY_BOLT,
                crate::enemies::move_ids::SENTRY_BEAM,
            ]);
        assert_eq!(combat.ai_rng.counter, 6);
    }

    #[test]
    fn guardian_stats_modes_cycles_sharp_hide_and_ai_ticks_match_java() {
        // Sources: reference/extracted/methods/monster/TheGuardian.java and
        // decompiled/.../powers/SharpHidePower.java. Expected values and move
        // transitions are re-derived from the constructors and `use*` methods.
        for (ascension, hp, threshold, fierce, roll, sharp) in [
            (0, 240, 30, 32, 9, 3),
            (4, 240, 30, 36, 10, 3),
            (9, 250, 35, 36, 10, 3),
            (19, 250, 40, 36, 10, 4),
        ] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["TheGuardian".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!((enemy.entity.hp, enemy.entity.max_hp), (hp, hp));
            assert_eq!(enemy.entity.status(crate::status_ids::sid::MODE_SHIFT), threshold);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::FIERCE_BASH_DMG), fierce);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ROLL_DMG), roll);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), sharp);
            assert_eq!(enemy.move_id, crate::enemies::move_ids::GUARD_CHARGING_UP);
            assert_eq!(enemy.move_block(), 9);
            assert_eq!(combat.ai_rng.counter, 1,
                "the constructor's rollMove consumes exactly one aiRng value");
        }

        let mut offensive = RunEngine::new(42, 0);
        offensive.enter_specific_combat(vec!["TheGuardian".to_string()]);
        let combat = offensive.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 200;
        combat.state.player.max_hp = 200;

        crate::combat_hooks::do_enemy_turns(combat); // Charge Up -> Fierce Bash
        assert_eq!(combat.state.enemies[0].entity.block, 9);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_FIERCE_BASH);
        assert_eq!(combat.state.enemies[0].move_damage(), 32);
        crate::combat_hooks::do_enemy_turns(combat); // Fierce Bash -> Vent Steam
        assert_eq!(combat.state.player.hp, 168);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_VENT_STEAM);
        crate::combat_hooks::do_enemy_turns(combat); // Vent Steam -> Whirlwind
        assert_eq!(combat.state.player.status(crate::status_ids::sid::WEAKENED), 2);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::VULNERABLE), 2);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_WHIRLWIND);
        crate::combat_hooks::do_enemy_turns(combat); // Whirlwind -> Charge Up
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_CHARGING_UP);
        assert_eq!(combat.ai_rng.counter, 1,
            "Guardian's direct setMove cycle consumes no later aiRng values");

        let mut defensive = RunEngine::new(42, 19);
        defensive.enter_specific_combat(vec!["TheGuardian".to_string()]);
        let combat = defensive.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 200;
        combat.state.player.max_hp = 200;

        combat.deal_damage_to_enemy(0, 39);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::DAMAGE_TAKEN_THIS_MODE), 39);
        combat.state.enemies[0].entity.block = 1;
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_CHARGING_UP,
            "blocked damage does not count toward Mode Shift");
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::PHASE), 1);
        assert_eq!(combat.state.enemies[0].entity.block, 20);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::MODE_SHIFT), 50);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_CLOSE_UP);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::SHARP_HIDE), 0,
            "Sharp Hide is applied by Close Up, not by the mode change");

        crate::combat_hooks::do_enemy_turns(combat); // Close Up -> Roll Attack
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::SHARP_HIDE), 4);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_ROLL_ATTACK);

        // SharpHidePower.onUseCard retaliates once for an Attack card even
        // when the Guardian blocks all of that card's damage.
        combat.phase = crate::engine::CombatPhase::PlayerTurn;
        combat.state.enemies[0].entity.block = 100;
        combat.state.player.block = 3;
        combat.state.energy = 3;
        let strike = combat.card_registry.make_card("Strike");
        combat.state.hand.push(strike);
        let hand_idx = combat.state.hand.len() - 1;
        let hp_before = combat.state.player.hp;
        let guardian_hp_before = combat.state.enemies[0].entity.hp;
        combat.play_card(hand_idx, 0);
        assert_eq!(combat.state.enemies[0].entity.hp, guardian_hp_before);
        assert_eq!(combat.state.player.hp, hp_before - 1);
        assert_eq!(combat.state.player.block, 0);

        combat.state.relics.push("Tungsten Rod".to_string());
        let strike = combat.card_registry.make_card("Strike");
        combat.state.hand.push(strike);
        let hp_before = combat.state.player.hp;
        combat.play_card(combat.state.hand.len() - 1, 0);
        assert_eq!(combat.state.player.hp, hp_before - 3,
            "Tungsten Rod reduces Sharp Hide's THORNS HP damage by one");

        combat.state.relics.retain(|id| id != "Tungsten Rod");
        combat.state.relics.push("Necronomicon".to_string());
        combat.state.enemies[0].entity.block = 200;
        combat.state.energy = 3;
        let bash = combat.card_registry.make_card("Bash");
        combat.state.hand.push(bash);
        let hp_before = combat.state.player.hp;
        combat.play_card(combat.state.hand.len() - 1, 0);
        assert_eq!(combat.state.player.hp, hp_before - 8,
            "Necronomicon's replay constructs a second Sharp Hide onUseCard hit");

        crate::combat_hooks::do_enemy_turns(combat); // Roll Attack -> Twin Slam
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_TWIN_SLAM);
        combat.state.player.set_status(crate::status_ids::sid::THORNS, 3);
        let guardian_hp_before = combat.state.enemies[0].entity.hp;
        crate::combat_hooks::do_enemy_turns(combat); // Offensive Mode, Twin Slam -> Whirlwind
        assert_eq!(combat.state.enemies[0].entity.hp, guardian_hp_before - 6);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::DAMAGE_TAKEN_THIS_MODE), 6,
            "Offensive Mode resolves before Twin Slam, so both Thorns hits count");
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::PHASE), 0);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::SHARP_HIDE), 0);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_WHIRLWIND);
        assert_eq!(combat.ai_rng.counter, 1);

        let mut lethal = RunEngine::new(42, 19);
        lethal.enter_specific_combat(vec!["TheGuardian".to_string()]);
        let combat = lethal.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].entity.hp = 40;
        combat.deal_damage_to_enemy(0, 40);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_ne!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_CLOSE_UP,
            "TheGuardian.damage excludes lethal hits with !isDying");
    }

    #[test]
    fn hexaghost_stats_divider_cycle_burn_upgrade_and_ai_ticks_match_java() {
        // Sources: reference/extracted/methods/monster/Hexaghost.java and
        // decompiled/java-src/com/megacrit/cardcrawl/actions/unique/
        // BurnIncreaseAction.java.
        for (ascension, hp, tackle, inferno, strength, burns) in [
            (0, 250, 5, 2, 2, 1),
            (4, 250, 6, 3, 2, 1),
            (9, 264, 6, 3, 2, 1),
            (19, 264, 6, 3, 3, 2),
        ] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["Hexaghost".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!((enemy.entity.hp, enemy.entity.max_hp), (hp, hp));
            assert_eq!(enemy.entity.status(crate::status_ids::sid::FIRE_TACKLE_DMG), tackle);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::INFERNO_DMG), inferno);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), strength);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::SEAR_BURN_COUNT), burns);
            assert_eq!(enemy.move_id, crate::enemies::move_ids::HEX_ACTIVATE);
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut engine = RunEngine::new(42, 0);
        engine.enter_specific_combat(vec!["Hexaghost".to_string()]);
        let combat = engine.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 77;
        combat.state.player.max_hp = 500;

        crate::combat_hooks::do_enemy_turns(combat); // Activate -> Divider
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_DIVIDER);
        assert_eq!(combat.state.enemies[0].move_damage(), 77 / 12 + 1);
        assert_eq!(combat.state.enemies[0].move_hits(), 6);
        assert_eq!(combat.ai_rng.counter, 1,
            "Activate sets Divider directly without RollMoveAction");

        combat.state.player.hp = 500;
        let seed_burn = combat.card_registry.make_card("Burn");
        combat.state.draw_pile.push(seed_burn);
        crate::combat_hooks::do_enemy_turns(combat); // Divider -> Sear(0)
        assert_eq!(combat.state.player.hp, 458);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_SEAR);
        assert_eq!(combat.state.enemies[0].effect(crate::combat_types::mfx::BURN), Some(1));
        assert_eq!(combat.ai_rng.counter, 2);

        crate::combat_hooks::do_enemy_turns(combat); // Sear(0) -> Tackle(1)
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_TACKLE);
        assert_eq!(combat.state.enemies[0].move_damage(), 5);
        crate::combat_hooks::do_enemy_turns(combat); // Tackle(1) -> Sear(2)
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_SEAR);
        crate::combat_hooks::do_enemy_turns(combat); // Sear(2) -> Inflame(3)
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_INFLAME);
        crate::combat_hooks::do_enemy_turns(combat); // Inflame(3) -> Tackle(4)
        assert_eq!(combat.state.enemies[0].entity.block, 12);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::STRENGTH), 2);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_TACKLE);
        crate::combat_hooks::do_enemy_turns(combat); // Tackle(4) -> Sear(5)
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_SEAR);
        crate::combat_hooks::do_enemy_turns(combat); // Sear(5) -> Inferno(6)
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_INFERNO);
        assert_eq!(combat.state.enemies[0].move_damage(), 2);
        assert_eq!(combat.state.enemies[0].move_hits(), 6);
        assert_eq!(combat.ai_rng.counter, 8);

        crate::combat_hooks::do_enemy_turns(combat); // Inferno -> Sear(0)
        let draw_names: Vec<&str> = combat.state.draw_pile.iter()
            .map(|card| combat.card_registry.card_name(card.def_id)).collect();
        let discard_names: Vec<&str> = combat.state.discard_pile.iter()
            .map(|card| combat.card_registry.card_name(card.def_id)).collect();
        assert!(draw_names.iter().all(|name| *name != "Burn"));
        assert!(discard_names.iter().all(|name| *name != "Burn"));
        assert_eq!(draw_names.iter().chain(discard_names.iter())
            .filter(|name| **name == "Burn+").count(), 7,
            "three Sear Burns plus the seeded Burn are upgraded, then three Burn+ are added");
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_SEAR);
        assert_eq!(combat.state.enemies[0].effect(crate::combat_types::mfx::BURN_PLUS), Some(1));
        assert_eq!(combat.ai_rng.counter, 9);

        crate::combat_hooks::do_enemy_turns(combat); // upgraded Sear -> Tackle
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Burn+").count(), 7);
        assert_eq!(combat.state.draw_pile.iter().chain(combat.state.discard_pile.iter())
            .filter(|card| combat.card_registry.card_name(card.def_id) == "Burn+").count(), 8);

        let mut a19 = RunEngine::new(42, 19);
        a19.enter_specific_combat(vec!["Hexaghost".to_string()]);
        let combat = a19.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 200;
        crate::combat_hooks::do_enemy_turns(combat);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].effect(crate::combat_types::mfx::BURN), Some(2));
    }

    #[test]
    fn hexaghost_visual_helpers_are_not_targetable_combat_entities() {
        // Sources: decompiled/java-src/com/megacrit/cardcrawl/monsters/
        // exordium/HexaghostBody.java and HexaghostOrb.java. They are rendering
        // helpers owned by Hexaghost, not AbstractMonster subclasses.
        let mut engine = RunEngine::new(42, 0);
        engine.enter_specific_combat(vec!["Hexaghost".to_string()]);
        let combat = engine.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.enemies.len(), 1);
        assert_eq!(combat.state.enemies[0].id, "Hexaghost");
        assert!(!crate::enemies::known_enemy_ids().iter().any(|(id, _)|
            matches!(*id, "HexaghostBody" | "HexaghostOrb")));
    }

    #[test]
    fn slime_boss_stats_direct_cycle_delayed_split_and_child_order_match_java() {
        // Source: reference/extracted/methods/monster/SlimeBoss.java
        // (constructor, `getMove`, `takeTurn`, and `damage`).
        for (ascension, hp, tackle, slam, slimed) in [
            (0, 140, 9, 35, 3),
            (4, 140, 10, 38, 3),
            (9, 150, 10, 38, 3),
            (19, 150, 10, 38, 5),
        ] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["SlimeBoss".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!((enemy.entity.hp, enemy.entity.max_hp), (hp, hp));
            assert_eq!(enemy.entity.status(crate::status_ids::sid::FIRE_TACKLE_DMG), tackle);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::SLAP_DMG), slam);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), slimed);
            assert_eq!(enemy.move_id, crate::enemies::move_ids::SB_STICKY);
            assert_eq!(enemy.effect(crate::combat_types::mfx::SLIMED), Some(slimed as i16));
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut cycle = RunEngine::new(42, 19);
        cycle.enter_specific_combat(vec!["SlimeBoss".to_string()]);
        let combat = cycle.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 200;
        combat.state.player.max_hp = 200;
        crate::combat_hooks::do_enemy_turns(combat); // Sticky -> Prep Slam
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Slimed").count(), 5);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SB_PREP_SLAM);
        crate::combat_hooks::do_enemy_turns(combat); // Prep Slam -> Slam
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SB_SLAM);
        assert_eq!(combat.state.enemies[0].move_damage(), 38);
        crate::combat_hooks::do_enemy_turns(combat); // Slam -> Sticky
        assert_eq!(combat.state.player.hp, 162);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SB_STICKY);
        assert_eq!(combat.state.enemies[0].effect(crate::combat_types::mfx::SLIMED), Some(5));
        assert_eq!(combat.ai_rng.counter, 1,
            "Sticky, Prep Slam, and Slam all set the next move directly");

        let mut split = RunEngine::new(42, 19);
        split.enter_specific_combat(vec!["SlimeBoss".to_string()]);
        let combat = split.combat_engine.as_mut().unwrap();
        combat.deal_damage_to_enemy(0, 75);
        assert_eq!(combat.state.enemies[0].entity.hp, 75);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SB_SPLIT);
        assert_eq!(combat.state.enemies.len(), 1,
            "damage only interrupts the intent; Split has not taken its turn");
        assert_eq!(combat.ai_rng.counter, 1);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_eq!(combat.state.enemies.len(), 3);
        assert_eq!(combat.state.enemies[1].id, "SpikeSlime_L");
        assert_eq!(combat.state.enemies[2].id, "AcidSlime_L");
        for child in &combat.state.enemies[1..] {
            assert_eq!((child.entity.hp, child.entity.max_hp), (75, 75));
        }
        assert_eq!(combat.state.enemies[1].entity.status(
            crate::status_ids::sid::STARTING_DMG), 18);
        assert_eq!(combat.state.enemies[1].entity.status(
            crate::status_ids::sid::STR_AMT), 3);
        assert_eq!(combat.state.enemies[2].entity.status(
            crate::status_ids::sid::STARTING_DMG), 12);
        assert_eq!(combat.state.enemies[2].entity.status(
            crate::status_ids::sid::STR_AMT), 18);
        assert_eq!(combat.state.enemies[1].entity.status(
            crate::status_ids::sid::BLOCK_AMT), 17);
        assert_eq!(combat.state.enemies[2].entity.status(
            crate::status_ids::sid::BLOCK_AMT), 17);
        assert_eq!(combat.ai_rng.counter, 3,
            "the spawned Spike then Acid each consume one opening aiRng value");

        let mut lethal = RunEngine::new(42, 0);
        lethal.enter_specific_combat(vec!["SlimeBoss".to_string()]);
        let combat = lethal.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].entity.hp = 10;
        combat.deal_damage_to_enemy(0, 10);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_eq!(combat.state.enemies.len(), 1,
            "SlimeBoss.damage excludes lethal hits with !isDying");
    }

    #[test]
    fn apology_slime_hp_opening_boolean_alternation_and_ai_ticks_match_java() {
        // Source: reference/extracted/methods/monster/ApologySlime.java.
        let mut hp_values = std::collections::HashSet::new();
        let mut attack_seed = None;
        let mut debuff_seed = None;
        for seed in 1..=256 {
            let mut engine = RunEngine::new(seed, 0);
            engine.enter_specific_combat(vec!["Apology Slime".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            hp_values.insert(enemy.entity.hp);
            assert_eq!(combat.ai_rng.counter, 2,
                "rollMove and getMove.randomBoolean each consume one aiRng value");
            match enemy.move_id {
                crate::enemies::move_ids::APOLOGY_TACKLE => attack_seed.get_or_insert(seed),
                crate::enemies::move_ids::APOLOGY_DEBUFF => debuff_seed.get_or_insert(seed),
                move_id => panic!("unexpected Apology Slime opener {move_id}"),
            };
        }
        assert_eq!(hp_values, (8..=12).collect());

        let mut attack = RunEngine::new(attack_seed.expect("attack opener"), 0);
        attack.enter_specific_combat(vec!["Apology Slime".to_string()]);
        let combat = attack.combat_engine.as_mut().unwrap();
        let hp_before = combat.state.player.hp;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, hp_before - 3);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::APOLOGY_DEBUFF);
        assert_eq!(combat.state.enemies[0].effect(crate::combat_types::mfx::WEAK), Some(1));
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::WEAKENED), 1);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::APOLOGY_TACKLE);
        assert_eq!(combat.ai_rng.counter, 2,
            "the direct alternating cycle consumes no later aiRng values");

        let mut debuff = RunEngine::new(debuff_seed.expect("debuff opener"), 0);
        debuff.enter_specific_combat(vec!["Apology Slime".to_string()]);
        let combat = debuff.combat_engine.as_mut().unwrap();
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::WEAKENED), 1);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::APOLOGY_TACKLE);
        assert_eq!(combat.ai_rng.counter, 2);
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
            "Strike".to_string(),
            "Eruption".to_string(),
        ];

        engine.step(&RunAction::CampfireUpgrade(1));
        assert_eq!(engine.run_state.deck[1], "Eruption+");
    }

    #[test]
    fn ancient_tea_set_arms_on_rest_room_entry_and_fires_in_the_next_combat() {
        // Source-derived (verify relic/Ancient Tea Set): AncientTeaSet.java
        // arms counter -2 in onEnterRestRoom, then its first atTurnStart gains
        // exactly 2 energy and resets the counter.
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.relics.push("Ancient Tea Set".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);

        engine.enter_campfire();
        assert_eq!(
            engine.run_state.relic_flags.counters
                [crate::relic_flags::counter::ANCIENT_TEA_SET],
            1
        );

        engine.enter_specific_combat(vec!["JawWorm".to_string()]);
        let combat = engine.combat_engine.as_ref().expect("combat should start");
        assert_eq!(combat.state.energy, 5);
        assert_eq!(
            combat.state.relic_counters[crate::relic_flags::counter::ANCIENT_TEA_SET],
            0
        );
    }

    #[test]
    fn bloody_idol_swaps_at_forgotten_altar_and_heals_once_per_gold_gain() {
        // Source-derived (verify relic/Bloody Idol): ForgottenAltar.java swaps
        // Golden Idol for Bloody Idol; BloodyIdol.java::onGainGold heals 5 once
        // for each successful gold-gain call.
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.relics.push("Golden Idol".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.current_event = crate::events::typed_events_for_act(2)
            .into_iter()
            .find(|event| event.name == "Forgotten Altar");
        engine.phase = RunPhase::Event;

        assert!(engine
            .step_with_result(&RunAction::EventChoice(0))
            .action_accepted);
        assert!(!engine
            .run_state
            .relics
            .iter()
            .any(|relic| relic == "Golden Idol"));
        assert!(engine
            .step_with_result(&RunAction::SelectRewardItem(0))
            .action_accepted);
        assert!(engine
            .run_state
            .relics
            .iter()
            .any(|relic| relic == "Bloody Idol"));

        engine.run_state.current_hp = 40;
        let gold = engine.run_state.gold;
        engine.adjust_run_gold(75);
        assert_eq!(engine.run_state.gold, gold + 75);
        assert_eq!(engine.run_state.current_hp, 45);

        engine.adjust_run_gold(-10);
        assert_eq!(engine.run_state.current_hp, 45);
        engine.run_state.relic_flags.set(crate::relic_flags::flag::ECTOPLASM);
        let gold = engine.run_state.gold;
        engine.adjust_run_gold(50);
        assert_eq!(engine.run_state.gold, gold);
        assert_eq!(engine.run_state.current_hp, 45);
    }

    #[test]
    fn bottled_flame_marks_one_selected_master_deck_card_innate_in_combat() {
        // Source-derived (verify relic/Bottled Flame): the selected master-deck
        // card has inBottleFlame=true and is therefore moved into the opening
        // hand by AbstractPlayer.initializeStarterDeck.
        let mut engine = RunEngine::new(17, 0);
        engine.run_state.deck = vec![
            "Strike".to_string(),
            "Strike".to_string(),
            "Strike".to_string(),
            "Defend".to_string(),
            "Defend".to_string(),
            "Defend".to_string(),
            "Defend".to_string(),
            "Vigilance".to_string(),
            "Eruption".to_string(),
            "Wallop".to_string(),
        ];
        engine.run_state.relics.push("Bottled Flame".to_string());
        engine.run_state.bottled_flame_card = Some("Wallop".to_string());

        engine.enter_specific_combat(vec!["JawWorm".to_string()]);
        let combat = engine.combat_engine.as_ref().expect("combat should start");
        assert!(combat
            .state
            .hand
            .iter()
            .any(|card| combat.card_registry.card_name(card.def_id) == "Wallop"));
    }

    #[test]
    fn bottled_lightning_marks_one_selected_master_deck_card_innate_in_combat() {
        // Source-derived (verify relic/Bottled Lightning): the selected Skill
        // gets inBottleLightning=true and enters the opening hand.
        let mut engine = RunEngine::new(23, 0);
        engine.run_state.deck = vec![
            "Strike".to_string(),
            "Strike".to_string(),
            "Strike".to_string(),
            "Defend".to_string(),
            "Defend".to_string(),
            "Defend".to_string(),
            "Eruption".to_string(),
            "Vigilance".to_string(),
            "Wallop".to_string(),
            "ThirdEye".to_string(),
        ];
        engine.run_state.relics.push("Bottled Lightning".to_string());
        engine.run_state.bottled_lightning_card = Some("ThirdEye".to_string());

        engine.enter_specific_combat(vec!["JawWorm".to_string()]);
        let combat = engine.combat_engine.as_ref().expect("combat should start");
        assert!(combat
            .state
            .hand
            .iter()
            .any(|card| combat.card_registry.card_name(card.def_id) == "ThirdEye"));
    }

    #[test]
    fn bottled_tornado_marks_one_selected_master_deck_card_innate_in_combat() {
        // Source-derived (verify relic/Bottled Tornado): the selected Power
        // gets inBottleTornado=true and enters the opening hand.
        let mut engine = RunEngine::new(29, 0);
        engine.run_state.deck = vec![
            "Strike".to_string(),
            "Strike".to_string(),
            "Strike".to_string(),
            "Defend".to_string(),
            "Defend".to_string(),
            "Defend".to_string(),
            "Eruption".to_string(),
            "Vigilance".to_string(),
            "Wallop".to_string(),
            "Devotion".to_string(),
        ];
        engine.run_state.relics.push("Bottled Tornado".to_string());
        engine.run_state.bottled_tornado_card = Some("Devotion".to_string());

        engine.enter_specific_combat(vec!["JawWorm".to_string()]);
        let combat = engine.combat_engine.as_ref().expect("combat should start");
        assert!(combat
            .state
            .hand
            .iter()
            .any(|card| combat.card_registry.card_name(card.def_id) == "Devotion"));
    }

    #[test]
    fn busted_crown_increases_watcher_master_energy_for_combat() {
        // Source-derived (verify relic/Busted Crown):
        // BustedCrown.java::onEquip increments energyMaster exactly once.
        let mut engine = RunEngine::new(31, 0);
        engine.run_state.relics.push("Busted Crown".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);

        engine.enter_specific_combat(vec!["JawWorm".to_string()]);
        let combat = engine.combat_engine.as_ref().expect("combat should start");
        assert_eq!(combat.state.max_energy, 4);
        assert_eq!(combat.state.energy, 4);
    }

    #[test]
    fn coffee_dripper_increases_watcher_master_energy_for_combat() {
        // Source-derived (verify relic/Coffee Dripper):
        // CoffeeDripper.java::onEquip increments energyMaster exactly once.
        let mut engine = RunEngine::new(37, 0);
        engine.run_state.relics.push("Coffee Dripper".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);

        engine.enter_specific_combat(vec!["JawWorm".to_string()]);
        let combat = engine.combat_engine.as_ref().expect("combat should start");
        assert_eq!(combat.state.max_energy, 4);
        assert_eq!(combat.state.energy, 4);
    }

    #[test]
    fn cursed_key_increases_watcher_master_energy_for_combat() {
        // Source-derived (verify relic/Cursed Key):
        // CursedKey.java::onEquip increments energyMaster exactly once.
        let mut engine = RunEngine::new(41, 0);
        engine.run_state.relics.push("Cursed Key".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);

        engine.enter_specific_combat(vec!["JawWorm".to_string()]);
        let combat = engine.combat_engine.as_ref().expect("combat should start");
        assert_eq!(combat.state.max_energy, 4);
        assert_eq!(combat.state.energy, 4);
    }

    #[test]
    fn ectoplasm_increases_watcher_master_energy_for_combat() {
        // Source-derived (verify relic/Ectoplasm):
        // Ectoplasm.java::onEquip increments energyMaster exactly once.
        let mut engine = RunEngine::new(43, 0);
        engine.run_state.relics.push("Ectoplasm".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);

        engine.enter_specific_combat(vec!["JawWorm".to_string()]);
        let combat = engine.combat_engine.as_ref().expect("combat should start");
        assert_eq!(combat.state.max_energy, 4);
        assert_eq!(combat.state.energy, 4);
    }

    #[test]
    fn fusion_hammer_increases_watcher_master_energy_for_combat() {
        // Source-derived (verify relic/Fusion Hammer):
        // FusionHammer.java::onEquip increments energyMaster exactly once.
        let mut engine = RunEngine::new(45, 0);
        engine.run_state.relics.push("Fusion Hammer".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);

        engine.enter_specific_combat(vec!["JawWorm".to_string()]);
        let combat = engine.combat_engine.as_ref().expect("combat should start");
        assert_eq!(combat.state.max_energy, 4);
        assert_eq!(combat.state.energy, 4);
    }

    #[test]
    fn du_vu_doll_counts_actual_master_deck_curses_for_combat_strength() {
        // Source-derived (verify relic/Du-Vu Doll): DuVuDoll.java counts every
        // CURSE-typed master-deck card, then grants that count at battle start.
        let mut engine = RunEngine::new(47, 20);
        engine.run_state.relics.push("Du-Vu Doll".to_string());
        engine.run_state.deck.extend([
            "Regret".to_string(),
            "CurseOfTheBell".to_string(),
        ]);

        engine.enter_specific_combat(vec!["JawWorm".to_string()]);
        let combat = engine.combat_engine.as_ref().expect("combat should start");
        // Ascension 20 supplies Ascender's Bane; all three cards are CURSE type.
        assert_eq!(
            combat
                .state
                .player
                .status(crate::status_ids::sid::DU_VU_DOLL_CURSES),
            3
        );
        assert_eq!(combat.state.player.strength(), 3);
        assert_eq!(
            combat.hidden_effect_value(
                "Du-Vu Doll",
                crate::effects::runtime::EffectOwner::PlayerRelic { slot: 1 },
                0,
            ),
            3
        );
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
            "Strike".to_string(),
            "Strike".to_string(),
            "Strike".to_string(),
            "Strike".to_string(),
            "Defend".to_string(),
            "Defend".to_string(),
            "Defend".to_string(),
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

    // =========================================================================
    // D27 Pandora's Box -- partial fix regression test.
    // Java: replaces all Strikes/Defends in masterDeck with random commons.
    // Rust (partial): removes Strikes/Defends on equip; filling with random
    // commons deferred until D52 RNG stream partition lands.
    // =========================================================================

    #[test]
    fn pandora_box_removes_strikes_and_defends_from_master_deck_on_equip() {
        let mut engine = RunEngine::new(42, 0);
        // Watcher starter at this entry point is 4 Strike_P + 4 Defend_P + 2
        // class cards -- but RunEngine::new seeds Ironclad by default. For the
        // D27 assertion we only care that Strike_X / Defend_X get purged, not
        // which class. Inject a mixed-class deck to exercise all color arms.
        engine.run_state.deck = vec![
            "Strike".to_string(),
            "Strike".to_string(),
            "Defend".to_string(),
            "Defend".to_string(),
            "Bash".to_string(), // non-starter, should survive
            "ShrugItOff".to_string(), // non-starter, should survive
        ];
        engine.add_relic_reward("Pandora's Box");
        assert!(
            !engine
                .run_state
                .deck
                .iter()
                .any(|c| matches!(c.as_str(), "Strike" | "Defend")),
            "Pandora's Box should remove all Strikes/Defends; deck was {:?}",
            engine.run_state.deck
        );
        assert!(
            engine.run_state.deck.contains(&"Bash".to_string())
                && engine.run_state.deck.contains(&"ShrugItOff".to_string()),
            "Non-starter cards must be preserved; deck was {:?}",
            engine.run_state.deck
        );
        // Partial-fix caveat: Java replaces the removed basics 1-for-1 with
        // random commons. We do NOT yet add commons back (needs class-common
        // pool + card_random_rng from D52). Assert deck size equals the
        // non-starter count to catch any future silent change.
        assert_eq!(
            engine.run_state.deck.len(),
            2,
            "partial fix: 4 basics removed, 2 non-starters kept, nothing added yet"
        );
    }

    #[test]
    fn pandora_box_partial_fix_handles_deck_without_basics() {
        // Edge case: if all Strikes/Defends were already removed (e.g. by
        // Neow+shops), Pandora's Box should be a no-op and not crash.
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.deck = vec!["Bash".to_string(), "ShrugItOff".to_string()];
        let pre_len = engine.run_state.deck.len();
        engine.add_relic_reward("Pandora's Box");
        assert_eq!(engine.run_state.deck.len(), pre_len);
    }
}
