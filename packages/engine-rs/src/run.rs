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
    /// Shop: buy a potion (index into shop potion offerings)
    ShopBuyPotion(usize),
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

/// Watcher source card pools used by rewards, shops, and truly-random cards.
/// CardLibrary.addPurpleCards excludes BASIC cards and sorts the registered
/// cards into these rarity pools during AbstractDungeon.initializeCardPools.
/// Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/CardLibrary.java
/// Uses CardRegistry IDs (PascalCase, no spaces) to match card lookups.
/// Cards not in CardRegistry fall back to `get_or_default()` for defensive safety,
/// but the audited runtime now treats the Rust registry as canonical.
const WATCHER_COMMON_CARDS: &[&str] = &[
    "BowlingBash", "Consecrate", "Crescendo", "CrushJoints",
    "CutThroughFate", "EmptyBody", "EmptyFist", "Evaluate",
    "FlurryOfBlows", "FlyingSleeves", "FollowUp", "Halt",
    "JustLucky", "PathToVictory", "Prostrate",
    "Protect", "SashWhip", "ClearTheMind", "ThirdEye",
];

const WATCHER_UNCOMMON_CARDS: &[&str] = &[
    "Adaptation", "BattleHymn", "CarveReality", "Collect", "Conclude",
    "DeceiveReality", "EmptyMind", "Fasting2", "FearNoEvil",
    "ForeignInfluence", "Indignation", "InnerPeace", "LikeWater",
    "Meditate", "MentalFortress", "Nirvana", "Perseverance", "Pray",
    "ReachHeaven", "Sanctity", "SandsOfTime", "SignatureMove", "Study",
    "Swivel", "TalkToTheHand", "Tantrum", "Vengeance", "Wallop",
    "WaveOfTheHand", "Weave", "WheelKick", "WindmillStrike",
    "Wireheading", "Worship", "WreathOfFlame",
];

const WATCHER_RARE_CARDS: &[&str] = &[
    "Alpha", "Blasphemy", "Brilliance", "ConjureBlade", "DeusExMachina",
    "DevaForm", "Devotion", "Establishment", "Judgement", "LessonLearned",
    "MasterReality", "Omniscience", "Ragnarok", "Scrawl", "SpiritShield",
    "Vault", "Wish",
];

// AbstractDungeon.addColorlessCards supplies these non-special cards to the
// merchant's fixed uncommon and rare colorless slots.
// Java: decompiled/java-src/com/megacrit/cardcrawl/shop/Merchant.java
const SHOP_COLORLESS_UNCOMMON_CARDS: &[&str] = &[
    "Bandage Up", "Blind", "Dark Shackles", "Deep Breath", "Discovery",
    "Dramatic Entrance", "Enlightenment", "Finesse", "Flash of Steel",
    "Forethought", "Good Instincts", "Impatience", "Jack Of All Trades",
    "Madness", "Mind Blast", "Panacea", "PanicButton", "Purity",
    "Swift Strike", "Trip",
];

const SHOP_COLORLESS_RARE_CARDS: &[&str] = &[
    "Apotheosis", "Chrysalis", "HandOfGreed", "Magnetism",
    "Master of Strategy", "Mayhem", "Metamorphosis", "Panache",
    "Sadistic Nature", "Secret Technique", "Secret Weapon", "The Bomb",
    "Thinking Ahead", "Transmutation", "Violence",
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
    &["Shelled Parasite"],
    &["Cultist", "Chosen"],
];

const ACT2_STRONG_ENCOUNTERS: &[&[&str]] = &[
    &["SnakePlant"],
    &["Centurion", "Healer"],
    &["Cultist", "Cultist", "Cultist"],
    &["Byrd", "Byrd", "Byrd"],
    &["Chosen", "Cultist"],
    &["Shelled Parasite", "FungiBeast"],
];

const ACT2_ELITE_ENCOUNTERS: &[&[&str]] = &[
    &["GremlinLeader"],
    &["BookOfStabbing"],
    &["SlaverBlue", "SlaverBoss", "SlaverRed"],
];

// ---------------------------------------------------------------------------
// Act 3 encounter pools
// ---------------------------------------------------------------------------

const ACT3_WEAK_ENCOUNTERS: &[&[&str]] = &[
    &["Darkling", "Darkling", "Darkling"],
    &["Orb Walker"],
    &["3 Shapes"],
];

const ACT3_STRONG_ENCOUNTERS: &[&[&str]] = &[
    &["WrithingMass"],
    &["GiantHead"],
    &["Nemesis"],
    &["Reptomancer"],
    &["Transient"],
    &["Maw"],
    &["Serpent"],
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
    /// Potions available for purchase: (potion_id, price)
    #[serde(default)]
    pub potions: Vec<(String, i32)>,
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
    /// Per-card persistent state aligned with `deck`. The string deck remains
    /// the public/run-action surface; this snapshot carries Java CardSave.misc
    /// across combats for cards such as Genetic Algorithm.
    #[serde(default)]
    pub deck_card_states: Vec<crate::combat_types::CardInstance>,
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

fn golden_idol_combat_gold(base_gold: i32, has_golden_idol: bool) -> i32 {
    if !has_golden_idol {
        return base_gold;
    }

    // RewardItem.java::applyGoldBonus adds MathUtils.round(base * 0.25f)
    // to non-treasure, non-stolen gold rewards.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/rewards/RewardItem.java
    base_gold + ((base_gold as f32) * 0.25).round() as i32
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
    // MoltenEgg2 calls canUpgrade on the obtained card. An already-upgraded
    // Searing Blow therefore advances one more level because its canUpgrade()
    // is always true; the string ID remains the static `+` definition while
    // CardInstance.misc preserves the exact counter.
    // Java: MoltenEgg2.java and cards/red/SearingBlow.java.
    let extra_searing_upgrade = card_id == "Searing Blow+"
        && run_state
            .relic_flags
            .has(crate::relic_flags::flag::MOLTEN_EGG);
    let card_id = upgrade_obtained_card_for_eggs(run_state, &card_id);
    run_state.reconcile_deck_card_states();
    run_state.deck.push(card_id.clone());
    let registry = crate::cards::global_registry();
    let mut card_state = registry.make_card(&card_id);
    if extra_searing_upgrade {
        registry.upgrade_card(&mut card_state);
    }
    run_state.deck_card_states.push(card_state);
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

        let registry = crate::cards::global_registry();
        let deck_card_states = deck
            .iter()
            .map(|card_id| registry.make_card(card_id))
            .collect();

        Self {
            current_hp: max_hp,
            max_hp,
            gold: 99,
            floor: 0,
            act: 1,
            ascension,
            deck,
            deck_card_states,
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

    fn reconcile_deck_card_states(&mut self) {
        let registry = crate::cards::global_registry();
        let mut prior = std::mem::take(&mut self.deck_card_states);
        let mut reconciled = Vec::with_capacity(self.deck.len());

        for card_id in &self.deck {
            let mut card = registry.make_card(card_id);
            let base_id = card_id.trim_end_matches('+');
            if let Some(index) = prior.iter().position(|candidate| {
                candidate.def_id != u16::MAX
                    && registry
                        .card_name(candidate.def_id)
                        .trim_end_matches('+')
                        == base_id
            }) {
                // Upgrade operations keep misc; transforms construct a fresh
                // card and therefore do not match by base ID.
                card.misc = prior.remove(index).misc;
            }
            reconciled.push(card);
        }

        self.deck_card_states = reconciled;
    }

    fn upgrade_deck_card(&mut self, index: usize) -> Option<(String, String)> {
        if index >= self.deck.len() {
            return None;
        }
        self.reconcile_deck_card_states();
        let registry = crate::cards::global_registry();
        if !registry.can_upgrade_card(&self.deck_card_states[index]) {
            return None;
        }
        let original = self.deck[index].clone();
        registry.upgrade_card(&mut self.deck_card_states[index]);
        let upgraded = registry
            .card_name(self.deck_card_states[index].def_id)
            .to_string();
        self.deck[index] = upgraded.clone();
        Some((original, upgraded))
    }

    fn combat_deck_instances(&mut self) -> Vec<crate::combat_types::CardInstance> {
        self.reconcile_deck_card_states();
        self.deck_card_states.clone()
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
    NeowsLament,
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
    pending_empty_cage_removals: usize,

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
            pending_empty_cage_removals: 0,
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
                // NeowReward category 1 includes THREE_ENEMY_KILL, which
                // obtains NeowsLament under canonical ID NeowsBlessing.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/neow/NeowReward.java
                label: "Enemies in your next three combats have 1 HP".to_string(),
                effect: NeowChoiceEffect::NeowsLament,
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
            NeowChoiceEffect::NeowsLament => self.add_relic_reward("NeowsBlessing"),
        }
    }

    fn upgrade_random_cards(&mut self, count: usize) {
        let registry = crate::cards::global_registry();
        let mut eligible: Vec<usize> = self
            .run_state
            .deck
            .iter()
            .enumerate()
            .filter_map(|(idx, card_id)| registry.can_upgrade_name(card_id).then_some(idx))
            .collect();

        for _ in 0..count {
            if eligible.is_empty() {
                break;
            }
            let pick = self.rng.gen_range(0..eligible.len());
            let deck_idx = eligible.swap_remove(pick);
            self.run_state.upgrade_deck_card(deck_idx);
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
                            .filter_map(|(i, card)| {
                                crate::cards::global_registry()
                                    .can_upgrade_name(card)
                                    .then_some(i)
                            })
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
                        .filter_map(|(i, card)| {
                            crate::cards::global_registry()
                                .can_upgrade_name(card)
                                .then_some(i)
                        })
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
        self.available_map_destinations()
            .iter()
            .enumerate()
            .map(|(i, _)| RunAction::ChoosePath(i))
            .collect()
    }

    fn available_map_destinations(&self) -> Vec<(usize, usize, RoomType, bool)> {
        if self.run_state.map_y < 0 {
            return self
                .map
                .get_start_nodes()
                .iter()
                .map(|node| (node.x, node.y, node.room_type, false))
                .collect();
        }

        let x = self.run_state.map_x as usize;
        let y = self.run_state.map_y as usize;
        let normal = self.map.get_next_nodes(x, y);
        let mut destinations = normal
            .iter()
            .map(|node| (node.x, node.y, node.room_type, false))
            .collect::<Vec<_>>();
        let can_fly = self
            .run_state
            .relics
            .iter()
            .any(|relic| relic == "WingedGreaves")
            && self.run_state.relic_flags.counters
                [crate::relic_flags::counter::WINGED_GREAVES]
                > 0;
        let Some(target_y) = normal.first().map(|node| node.y) else {
            return destinations;
        };
        if can_fly {
            // MapRoomNode.wingedIsConnectedTo accepts every visible node whose
            // y matches an ordinary outgoing edge. Ordinary edges stay first;
            // only the additional nodes consume Winged Greaves charges.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/map/MapRoomNode.java
            for node in self.map.get_nodes_at_floor(target_y) {
                if node.room_type == RoomType::None
                    || destinations
                        .iter()
                        .any(|(nx, ny, _, _)| *nx == node.x && *ny == node.y)
                {
                    continue;
                }
                destinations.push((node.x, node.y, node.room_type, true));
            }
        }
        destinations
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
                if crate::cards::global_registry().can_upgrade_name(card) {
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
            let can_obtain_potion = !self.run_state.relic_flags.has(crate::relic_flags::flag::SOZU)
                && self.run_state.potions.iter().any(|potion| potion.is_empty());
            if can_obtain_potion {
                for (i, (_, price)) in shop.potions.iter().enumerate() {
                    if self.run_state.gold >= *price {
                        actions.push(RunAction::ShopBuyPotion(i));
                    }
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
                if self
                    .run_state
                    .relics
                    .iter()
                    .any(|relic| relic == "Toy Ornithopter")
                {
                    // ToyOrnithopter.java heals 5 immediately outside combat
                    // whenever a potion is used.
                    // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/ToyOrnithopter.java
                    self.heal_run_player(5);
                }
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

        let destinations = self.available_map_destinations();
        let Some(&(next_x, next_y, room_type, used_winged_greaves)) =
            destinations.get(path_idx)
        else {
            return 0.0;
        };

        if used_winged_greaves {
            let counter = &mut self.run_state.relic_flags.counters
                [crate::relic_flags::counter::WINGED_GREAVES];
            *counter -= 1;
            if *counter <= 0 {
                *counter = -2;
            }
        }

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
                // AbstractChest.open always creates a non-boss relic reward;
                // optional gold is a separate ordered reward. Keep the reward
                // screen available even without Matryoshka so chest callbacks
                // such as NlothsMask can mutate the relic reward.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/rewards/chests/AbstractChest.java
                self.build_treasure_reward_screen();
                self.phase = RunPhase::CardReward;
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
        // Expand composite encounters. MonsterHelper constructs Gremlin Leader
        // with two independent weighted miscRng gremlins before the Leader.
        // Source: decompiled/java-src/com/megacrit/cardcrawl/helpers/MonsterHelper.java.
        let mut expanded = Vec::new();
        for id in &encounter {
            match id.as_str() {
                "DonuAndDeca" => {
                    expanded.push("Donu".to_string());
                    expanded.push("Deca".to_string());
                }
                "GremlinLeader" | "Gremlin Leader" => {
                    const GREMLIN_POOL: [&str; 8] = [
                        "GremlinWarrior", "GremlinWarrior",
                        "GremlinThief", "GremlinThief",
                        "GremlinFat", "GremlinFat",
                        "GremlinTsundere", "GremlinWizard",
                    ];
                    for _ in 0..2 {
                        let index = self.rng.gen_range(0..GREMLIN_POOL.len());
                        expanded.push(GREMLIN_POOL[index].to_string());
                    }
                    expanded.push("GremlinLeader".to_string());
                }
                "Reptomancer" => {
                    // Source: decompiled/java-src/com/megacrit/cardcrawl/
                    // helpers/MonsterHelper.java encounter case: left dagger,
                    // Reptomancer, right dagger.
                    expanded.push("SnakeDagger".to_string());
                    expanded.push("Reptomancer".to_string());
                    expanded.push("SnakeDagger".to_string());
                }
                "3 Shapes" | "4 Shapes" => {
                    // Sources: decompiled TheBeyond.java and
                    // MonsterHelper.java (`spawnShapes`). Draw without
                    // replacement from two of each Ancient Shape.
                    let count = if id.as_str() == "3 Shapes" { 3 } else { 4 };
                    let mut pool = vec![
                        "Repulsor", "Repulsor", "Exploder", "Exploder",
                        "Spiker", "Spiker",
                    ];
                    for _ in 0..count {
                        let index = self.rng.gen_range(0..pool.len());
                        expanded.push(pool.remove(index).to_string());
                    }
                }
                _ => expanded.push(id.clone()),
            }
        }

        // Create enemies
        let mut enemy_states: Vec<EnemyCombatState> = expanded
            .iter()
            .map(|id| {
                let (hp, max_hp) = self.roll_enemy_hp(id);
                enemies::create_enemy(id, hp, max_hp)
            })
            .collect();

        if enemy_states.iter().any(|enemy| enemy.id == "GremlinLeader") {
            for enemy in enemy_states.iter_mut().filter(|enemy| matches!(enemy.id.as_str(),
                "GremlinFat" | "GremlinThief" | "GremlinWarrior"
                    | "GremlinWizard" | "GremlinTsundere"))
            {
                enemy.is_minion = true;
            }
        }

        if enemy_states.iter().any(|enemy| enemy.id == "Reptomancer") {
            // Source: reference/extracted/methods/monster/Reptomancer.java
            // (`usePreBattleAction`) gives MinionPower to every other monster
            // in its encounter.
            for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id != "Reptomancer") {
                enemy.is_minion = true;
            }
        }

        // AwakenedOne.usePreBattleAction applies independent ascension
        // thresholds: +2 Strength at A4, 320 HP at A9 (rolled above), and
        // Curiosity 2 / Regenerate 15 at A19. create_enemy has no ascension,
        // so these belong at the run spawn site.
        // Java: reference/extracted/methods/monster/AwakenedOne.java
        for enemy in enemy_states
            .iter_mut()
            .filter(|enemy| matches!(enemy.id.as_str(), "AwakenedOne" | "Awakened One"))
        {
            let high_ai = self.run_state.ascension >= 19;
            enemy
                .entity
                .set_status(crate::status_ids::sid::CURIOSITY, if high_ai { 2 } else { 1 });
            enemy.entity.set_status(
                crate::status_ids::sid::REGENERATION,
                if high_ai { 15 } else { 10 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STRENGTH,
                if self.run_state.ascension >= 4 { 2 } else { 0 },
            );
        }

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

        // Source: reference/extracted/methods/monster/BanditBear.java.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "BanditBear" | "Bear")) {
            let (maul, lunge) = if self.run_state.ascension >= 2 {
                (20, 10)
            } else {
                (18, 9)
            };
            let dexterity_down = if self.run_state.ascension >= 17 { 4 } else { 2 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, maul);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, lunge);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT, dexterity_down);
            enemy.set_move(crate::enemies::move_ids::BEAR_HUG, 0, 0, 0);
            enemy.move_effects.clear();
            enemy.add_effect(crate::combat_types::mfx::DEX_DOWN, dexterity_down as i16);
        }

        // Source: reference/extracted/methods/monster/BanditPointy.java.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "BanditChild" | "BanditPointy" | "Pointy")) {
            let damage = if self.run_state.ascension >= 2 { 6 } else { 5 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.set_move(crate::enemies::move_ids::POINTY_STAB, damage, 2, 0);
        }

        // Source: reference/extracted/methods/monster/Byrd.java (constructor).
        // Damage changes at A2, while the stored Flight amount changes at A17.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Byrd") {
            let peck_count = if self.run_state.ascension >= 2 { 6 } else { 5 };
            let swoop_damage = if self.run_state.ascension >= 2 { 14 } else { 12 };
            let flight = if self.run_state.ascension >= 17 { 4 } else { 3 };
            enemy.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, 1);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, peck_count);
            enemy.entity.set_status(crate::status_ids::sid::SLASH_DMG, swoop_damage);
            enemy.entity.set_status(crate::status_ids::sid::HEAD_SLAM_DMG, 3);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT, flight);
            enemy.entity.set_status(crate::status_ids::sid::FLIGHT, flight);
            enemy.set_move(crate::enemies::move_ids::BYRD_PECK, 1, peck_count, 0);
        }

        // Source: reference/extracted/methods/monster/ShelledParasite.java.
        // Damage changes at A2, its opener is forced to Fell at A17, and
        // usePreBattleAction grants both 14 Plated Armor and 14 block.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "Shelled Parasite" | "ShelledParasite"))
        {
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 7 } else { 6 });
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 2 { 21 } else { 18 });
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 2 { 12 } else { 10 });
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 });
            enemy.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy.entity.set_status(crate::status_ids::sid::PLATED_ARMOR, 14);
            enemy.entity.block = 14;
        }

        // Sources: reference/extracted/methods/monster/SnakePlant.java and
        // decompiled/java-src/com/megacrit/cardcrawl/powers/MalleablePower.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "SnakePlant") {
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 8 } else { 7 });
            enemy.entity.set_status(crate::status_ids::sid::MALLEABLE, 3);
            // Preserve MalleablePower.basePower separately from its growing amount.
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT, 3);
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 });
        }

        // Source: reference/extracted/methods/monster/Snecko.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Snecko") {
            enemy.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 18 } else { 15 });
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 2 { 10 } else { 8 });
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 });
        }

        // Source: reference/extracted/methods/monster/SphericGuardian.java.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "SphericGuardian" | "Spheric Guardian"))
        {
            enemy.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy.entity.set_status(crate::status_ids::sid::FIRST_TURN, 1);
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 11 } else { 10 });
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 35 } else { 25 });
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, 15);
            enemy.entity.set_status(crate::status_ids::sid::BARRICADE, 1);
            enemy.entity.set_status(crate::status_ids::sid::ARTIFACT, 3);
            enemy.entity.block = 40;
        }

        // Source: reference/extracted/methods/monster/TheCollector.java.
        // Fireball/Strength change at A4, HP and base Block at A9, while A19
        // independently raises Strength, Mega Debuff, and effective Buff Block.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "TheCollector" | "Collector"))
        {
            enemy.entity.set_status(crate::status_ids::sid::FIREBALL_DMG,
                if self.run_state.ascension >= 4 { 21 } else { 18 });
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 19 { 5 }
                else if self.run_state.ascension >= 4 { 4 } else { 3 });
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 19 { 23 }
                else if self.run_state.ascension >= 9 { 18 } else { 15 });
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 19 { 5 } else { 3 });
            enemy.entity.set_status(crate::status_ids::sid::TURN_COUNT, 0);
            enemy.entity.set_status(crate::status_ids::sid::USED_MEGA_DEBUFF, 0);
            enemy.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy.entity.set_status(crate::status_ids::sid::COUNT, 0);
        }

        // Source: reference/extracted/methods/monster/Centurion.java.
        let monster_count = enemy_states.len() as i32;
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Centurion") {
            let (slash, fury) = if self.run_state.ascension >= 2 {
                (14, 7)
            } else {
                (12, 6)
            };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, slash);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, fury);
            enemy.entity.set_status(crate::status_ids::sid::ATTACK_COUNT, 3);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 20 } else { 15 });
            enemy.entity.set_status(crate::status_ids::sid::COUNT, monster_count);
            enemy.set_move(crate::enemies::move_ids::CENT_SLASH, slash, 1, 0);
        }

        // Source: reference/extracted/methods/monster/Champ.java (constructor
        // and `getMove`). Its A4/A9/A19 thresholds are independent of HP.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "Champ" | "TheChamp")) {
            let high_damage = self.run_state.ascension >= 4;
            enemy.entity.set_status(crate::status_ids::sid::SLASH_DMG,
                if high_damage { 18 } else { 16 });
            enemy.entity.set_status(crate::status_ids::sid::SLAP_DMG,
                if high_damage { 14 } else { 12 });
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 19 { 4 }
                else if self.run_state.ascension >= 4 { 3 } else { 2 });
            enemy.entity.set_status(crate::status_ids::sid::FORGE_AMT,
                if self.run_state.ascension >= 19 { 7 }
                else if self.run_state.ascension >= 9 { 6 } else { 5 });
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 19 { 20 }
                else if self.run_state.ascension >= 9 { 18 } else { 15 });
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 19 { 1 } else { 0 });
            enemy.entity.set_status(crate::status_ids::sid::NUM_TURNS, 0);
            enemy.entity.set_status(crate::status_ids::sid::FORGE_TIMES, 0);
            enemy.entity.set_status(crate::status_ids::sid::THRESHOLD_REACHED, 0);
        }

        // Source: reference/extracted/methods/monster/Chosen.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Chosen") {
            let high_damage = self.run_state.ascension >= 2;
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if high_damage { 21 } else { 18 });
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if high_damage { 12 } else { 10 });
            enemy.entity.set_status(crate::status_ids::sid::SLAP_DMG,
                if high_damage { 6 } else { 5 });
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 });
            enemy.entity.set_status(crate::status_ids::sid::FIRST_TURN, 1);
            enemy.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 0);
        }

        // Source: reference/extracted/methods/monster/CorruptHeart.java
        // (constructor and `usePreBattleAction`). A4 damage, A9 HP, and A19
        // powers are independent thresholds; HP alone cannot encode them.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "CorruptHeart" | "Corrupt Heart")) {
            let high_damage = self.run_state.ascension >= 4;
            enemy.entity.set_status(crate::status_ids::sid::BLOOD_HIT_COUNT,
                if high_damage { 15 } else { 12 });
            enemy.entity.set_status(crate::status_ids::sid::ECHO_DMG,
                if high_damage { 45 } else { 40 });
            enemy.entity.set_status(crate::status_ids::sid::INVINCIBLE,
                if self.run_state.ascension >= 19 { 200 } else { 300 });
            enemy.entity.set_status(crate::status_ids::sid::BEAT_OF_DEATH,
                if self.run_state.ascension >= 19 { 2 } else { 1 });
            enemy.entity.set_status(crate::status_ids::sid::MOVE_COUNT, 0);
            enemy.entity.set_status(crate::status_ids::sid::BUFF_COUNT, 0);
            enemy.entity.set_status(crate::status_ids::sid::IS_FIRST_MOVE, 1);
        }

        // Source: reference/extracted/methods/monster/SpireShield.java
        // (constructor and `usePreBattleAction`). Damage, HP, and Artifact use
        // independent A3/A8/A18 thresholds. Shield begins to the player's left,
        // so it is the initial Back Attack while the player faces right.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "SpireShield" | "Spire Shield"))
        {
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 3 { 14 } else { 12 });
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 3 { 38 } else { 34 });
            enemy.entity.set_status(crate::status_ids::sid::ARTIFACT,
                if self.run_state.ascension >= 18 { 2 } else { 1 });
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 18 { 1 } else { 0 });
            enemy.entity.set_status(crate::status_ids::sid::MOVE_COUNT, 0);
            enemy.back_attack = true;
        }
        // Source: reference/extracted/methods/monster/SpireSpear.java
        // (constructor and `usePreBattleAction`). Burn Strike and Skewer change
        // at A3; Artifact and Burn destination change independently at A18.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "SpireSpear" | "Spire Spear"))
        {
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 3 { 6 } else { 5 });
            enemy.entity.set_status(crate::status_ids::sid::SKEWER_COUNT,
                if self.run_state.ascension >= 3 { 4 } else { 3 });
            enemy.entity.set_status(crate::status_ids::sid::ARTIFACT,
                if self.run_state.ascension >= 18 { 2 } else { 1 });
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 18 { 1 } else { 0 });
            enemy.entity.set_status(crate::status_ids::sid::MOVE_COUNT, 0);
        }
        if enemy_states.iter().any(|enemy| matches!(enemy.id.as_str(),
            "SpireShield" | "Spire Shield"))
        {
            for enemy in enemy_states.iter_mut().filter(|enemy| matches!(enemy.id.as_str(),
                "SpireSpear" | "Spire Spear"))
            {
                enemy.back_attack = false;
            }
        }

        // Source: reference/extracted/methods/monster/Darkling.java.
        // HP is rolled separately; Nip has its own inclusive monsterHpRng
        // roll, and Chomp/Harden change at independent ascension thresholds.
        for (index, enemy) in enemy_states.iter_mut().enumerate()
            .filter(|(_, enemy)| enemy.id == "Darkling")
        {
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 9 } else { 8 });
            let nip_base = if self.run_state.ascension >= 2 { 9 } else { 7 };
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                nip_base + self.rng.gen_range(0..=4));
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 });
            enemy.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy.entity.set_status(crate::status_ids::sid::COUNT, index as i32);
            enemy.entity.set_status(crate::status_ids::sid::REGROW, 1);
        }

        // Source: reference/extracted/methods/monster/Deca.java. Damage changes
        // at A4, HP at A9, and Artifact/Square's Plated Armor at A19.
        for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id == "Deca") {
            let beam = if self.run_state.ascension >= 4 { 12 } else { 10 };
            enemy.entity.set_status(crate::status_ids::sid::BEAM_DMG, beam);
            enemy.entity.set_status(crate::status_ids::sid::ARTIFACT,
                if self.run_state.ascension >= 19 { 3 } else { 2 });
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 19 { 1 } else { 0 });
            enemy.set_move(crate::enemies::move_ids::DECA_BEAM, beam, 2, 0);
            enemy.add_effect(crate::combat_types::mfx::DAZE, 2);
        }

        // Source: reference/extracted/methods/monster/Donu.java. Damage changes
        // at A4, HP at A9, and pre-battle Artifact at A19.
        for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id == "Donu") {
            let beam = if self.run_state.ascension >= 4 { 12 } else { 10 };
            enemy.entity.set_status(crate::status_ids::sid::BEAM_DMG, beam);
            enemy.entity.set_status(crate::status_ids::sid::ARTIFACT,
                if self.run_state.ascension >= 19 { 3 } else { 2 });
            enemy.set_move(crate::enemies::move_ids::DONU_CIRCLE, 0, 0, 0);
            enemy.add_effect(crate::combat_types::mfx::STRENGTH, 3);
            enemy.add_effect(crate::combat_types::mfx::STRENGTH_ALL_ALLIES, 3);
        }

        // Source: reference/extracted/methods/monster/Exploder.java. Damage
        // changes at A2; ExplosivePower(3) and turnCount start before combat.
        for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id == "Exploder") {
            let damage = if self.run_state.ascension >= 2 { 11 } else { 9 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(crate::status_ids::sid::TURN_COUNT, 0);
            enemy.entity.set_status(crate::status_ids::sid::EXPLOSIVE, 3);
            enemy.set_move(crate::enemies::move_ids::EXPLODER_ATTACK, damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/GiantHead.java. The
        // zero-amount Slow power uses sentinel 1 in Rust; A18 decrements count
        // once in usePreBattleAction before the opening rollMove.
        for enemy in enemy_states.iter_mut().filter(|enemy| matches!(enemy.id.as_str(),
            "GiantHead" | "Giant Head"))
        {
            enemy.entity.set_status(crate::status_ids::sid::COUNT,
                if self.run_state.ascension >= 18 { 4 } else { 5 });
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DEATH_DMG,
                if self.run_state.ascension >= 3 { 40 } else { 30 });
            enemy.entity.set_status(crate::status_ids::sid::SLOW, 1);
        }

        // Source: reference/extracted/methods/monster/TimeEater.java.
        // Damage changes at A4, HP at A9, and the extra move payloads at A19.
        for enemy in enemy_states.iter_mut().filter(|enemy| matches!(enemy.id.as_str(),
            "TimeEater" | "Time Eater"))
        {
            enemy.entity.set_status(crate::status_ids::sid::REVERB_DMG,
                if self.run_state.ascension >= 4 { 8 } else { 7 });
            enemy.entity.set_status(crate::status_ids::sid::HEAD_SLAM_DMG,
                if self.run_state.ascension >= 4 { 32 } else { 26 });
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 19 { 1 } else { 0 });
            enemy.entity.set_status(crate::status_ids::sid::USED_HASTE, 0);
            enemy.entity.set_status(crate::status_ids::sid::TIME_WARP_ACTIVE, 1);
        }

        // Source: reference/extracted/methods/monster/Maw.java. Maw has fixed
        // 300 HP; only Slam changes at A2, while Drool and Roar change at A17.
        for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id == "Maw") {
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 30 } else { 25 });
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 17 { 5 } else { 3 });
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 5 } else { 3 });
            enemy.entity.set_status(crate::status_ids::sid::TURN_COUNT, 1);
            enemy.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 0);
        }

        // Source: reference/extracted/methods/monster/Nemesis.java. Fire
        // damage changes at A3 and the Burn payload changes at A18.
        for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id == "Nemesis") {
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 3 { 7 } else { 6 });
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 18 { 5 } else { 3 });
            enemy.entity.set_status(crate::status_ids::sid::SCYTHE_COOLDOWN, 0);
            enemy.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 1);
        }

        // Source: reference/extracted/methods/monster/OrbWalker.java. Attack
        // values change at A2; GenericStrengthUp changes from 3 to 5 at A17.
        for enemy in enemy_states.iter_mut().filter(|enemy| matches!(enemy.id.as_str(),
            "OrbWalker" | "Orb Walker"))
        {
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 11 } else { 10 });
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 2 { 16 } else { 15 });
            enemy.entity.set_status(crate::status_ids::sid::GENERIC_STRENGTH_UP,
                if self.run_state.ascension >= 17 { 5 } else { 3 });
        }

        // Source: reference/extracted/methods/monster/Repulsor.java. Its
        // attack changes at A2; HP is rolled separately below at A7.
        for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id == "Repulsor") {
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 13 } else { 11 });
        }

        // Source: reference/extracted/methods/monster/Spiker.java.
        for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id == "Spiker") {
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 9 } else { 7 });
            let starting_thorns = if self.run_state.ascension >= 2 { 4 } else { 3 };
            enemy.entity.set_status(crate::status_ids::sid::THORNS,
                starting_thorns + if self.run_state.ascension >= 17 { 3 } else { 0 });
            enemy.entity.set_status(crate::status_ids::sid::COUNT, 0);
        }

        // Source: reference/extracted/methods/monster/SpireGrowth.java. The
        // Java class uses canonical ID `Serpent`; damage changes at A2,
        // Constricted at A17, and HP is patched separately below at A7.
        for enemy in enemy_states.iter_mut().filter(|enemy| matches!(enemy.id.as_str(),
            "Serpent" | "SpireGrowth" | "Spire Growth"))
        {
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 18 } else { 16 });
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 2 { 25 } else { 22 });
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 12 } else { 10 });
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 });
            enemy.entity.set_status(crate::status_ids::sid::COUNT, 0);
        }

        // Source: reference/extracted/methods/monster/Reptomancer.java.
        let repto_others = enemy_states.iter().filter(|enemy| enemy.id != "Reptomancer"
            && enemy.is_alive()).count() as i32;
        for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id == "Reptomancer") {
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 3 { 16 } else { 13 });
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 3 { 34 } else { 30 });
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 18 { 2 } else { 1 });
            enemy.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy.entity.set_status(crate::status_ids::sid::COUNT, repto_others);
        }

        // Source: reference/extracted/methods/monster/BanditLeader.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "BanditLeader") {
            let (cross_slash, agonizing_slash) = if self.run_state.ascension >= 2 {
                (17, 12)
            } else {
                (15, 10)
            };
            let weak = if self.run_state.ascension >= 17 { 3 } else { 2 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, cross_slash);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, agonizing_slash);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT, weak);
            enemy.set_move(crate::enemies::move_ids::BANDIT_MOCK, 0, 0, 0);
        }

        // Source: reference/extracted/methods/monster/BookOfStabbing.java.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "BookOfStabbing" | "Book of Stabbing")) {
            let (stab, big_stab) = if self.run_state.ascension >= 3 {
                (7, 24)
            } else {
                (6, 21)
            };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, stab);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, big_stab);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 18 { 1 } else { 0 });
            enemy.entity.set_status(crate::status_ids::sid::STAB_COUNT, 0);
            enemy.entity.set_status(crate::status_ids::sid::PAINFUL_STABS, 1);
        }

        // Source: reference/extracted/methods/monster/Taskmaster.java.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "SlaverBoss" | "TaskMaster" | "Taskmaster"))
        {
            let wounds = if self.run_state.ascension >= 18 { 3 }
                else if self.run_state.ascension >= 3 { 2 } else { 1 };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, 7);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT, wounds);
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 18 { 1 } else { 0 });
        }

        // Source: reference/extracted/methods/monster/BronzeAutomaton.java.
        for enemy in enemy_states.iter_mut().filter(|e| matches!(e.id.as_str(),
            "BronzeAutomaton" | "Bronze Automaton")) {
            let (flail, beam, strength) = if self.run_state.ascension >= 4 {
                (8, 50, 4)
            } else {
                (7, 45, 3)
            };
            enemy.entity.set_status(crate::status_ids::sid::FLAIL_DMG, flail);
            enemy.entity.set_status(crate::status_ids::sid::BEAM_DMG, beam);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, strength);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 9 { 12 } else { 9 });
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 19 { 1 } else { 0 });
            enemy.entity.set_status(crate::status_ids::sid::FIRST_TURN, 1);
            enemy.entity.set_status(crate::status_ids::sid::NUM_TURNS, 0);
            enemy.entity.set_status(crate::status_ids::sid::ARTIFACT, 3);
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

        // Source: reference/extracted/methods/monster/Mugger.java. The attack
        // fields change at A2; gold theft and Smoke Bomb block change at A17.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Mugger") {
            let (swipe, big_swipe) = if self.run_state.ascension >= 2 {
                (11, 18)
            } else {
                (10, 16)
            };
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG, swipe);
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, big_swipe);
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 17 } else { 11 });
            enemy.entity.set_status(crate::status_ids::sid::TURN_COUNT,
                if self.run_state.ascension >= 17 { 20 } else { 15 });
            enemy.entity.set_status(crate::status_ids::sid::ATTACK_COUNT, 0);
            enemy.set_move(crate::enemies::move_ids::MUGGER_MUG, swipe, 1, 0);
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

        // Source: reference/extracted/methods/monster/GremlinLeader.java.
        // STARTING_DMG is an internal marker carrying ascension for later
        // SummonGremlinAction construction; the Leader's Stab is always 6x3.
        let alive_gremlins = enemy_states.iter().filter(|enemy| enemy.is_minion
            && enemy.is_alive()).count() as i32;
        for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id == "GremlinLeader") {
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 18 { 5 }
                else if self.run_state.ascension >= 3 { 4 } else { 3 });
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 18 { 10 } else { 6 });
            enemy.entity.set_status(crate::status_ids::sid::COUNT, alive_gremlins);
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                self.run_state.ascension);
        }

        // Source: reference/extracted/methods/monster/Healer.java. COUNT is
        // refreshed from group missing HP before each later RollMoveAction.
        let missing_hp = enemy_states.iter().filter(|enemy| enemy.is_alive())
            .map(|enemy| enemy.entity.max_hp - enemy.entity.hp).sum();
        for enemy in enemy_states.iter_mut().filter(|enemy| matches!(enemy.id.as_str(),
            "Healer" | "Mystic"))
        {
            enemy.entity.set_status(crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 9 } else { 8 });
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 17 { 4 }
                else if self.run_state.ascension >= 2 { 3 } else { 2 });
            enemy.entity.set_status(crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 20 } else { 16 });
            enemy.entity.set_status(crate::status_ids::sid::COUNT, missing_hp);
            enemy.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 });
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
        let mut deck_instances = self.run_state.combat_deck_instances();
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
        // FusionHammer.java, RunicDome.java, VelvetChoker.java, and
        // PhilosopherStone.java and Sozu.java each increment energyMaster once
        // in onEquip.
        let slavers_collar_energy = i32::from(
            self.run_state.relics.iter().any(|relic| relic == "SlaversCollar")
                && expanded.iter().any(|enemy| matches!(enemy.as_str(),
                    "GremlinNob" | "Lagavulin" | "Sentry" | "BookOfStabbing"
                    | "GremlinLeader" | "SlaverBoss" | "TaskMaster" | "Taskmaster"
                    | "Nemesis" | "Reptomancer"
                    | "GiantHead" | "Hexaghost" | "SlimeBoss" | "TheGuardian"
                    | "BronzeAutomaton" | "TheCollector" | "TheChamp" | "AwakenedOne"
                    | "TimeEater" | "Donu" | "Deca" | "TheHeart" | "CorruptHeart"
                    | "SpireShield" | "SpireSpear")),
        );
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
            )
            + i32::from(
                self.run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::RUNIC_DOME),
            )
            + i32::from(
                self.run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::VELVET_CHOKER),
            )
            + i32::from(
                self.run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::PHILOSOPHERS_STONE),
            )
            + i32::from(
                self.run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::SOZU),
            )
            // SlaversCollar.beforeEnergyPrep increments only for this combat;
            // onVictory decrements it again, so RunState does not persist it.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/SlaversCollar.java
            + slavers_collar_energy;
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
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::PRISMATIC_SHARD)
        {
            // PrismaticShard.java::onEquip gives Watcher one master orb slot.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/PrismaticShard.java
            combat_state.orb_slots = crate::orbs::OrbSlots::new(1);
        }
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
            "Mugger" => {
                // Source: reference/extracted/methods/monster/Mugger.java:
                // inclusive 48..52, or 50..54 at ascension 7.
                let base = if a20 { 50 } else { 48 };
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
            "BanditBear" | "Bear" => {
                let base = if a20 { 40 } else { 38 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "BanditChild" | "BanditPointy" | "Pointy" => {
                let hp = if a20 { 34 } else { 30 };
                (hp, hp)
            }
            "BanditLeader" => {
                let base = if a20 { 37 } else { 35 };
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
                // Source: reference/extracted/methods/monster/Byrd.java:
                // setHp(25,31), or setHp(26,33) at ascension 7+.
                let (base, width) = if a20 { (26, 7) } else { (25, 6) };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "Chosen" => {
                // Source: Chosen.java constructor: inclusive setHp(95,99),
                // or setHp(98,103) at ascension 7+.
                let (base, width) = if a20 { (98, 5) } else { (95, 4) };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "Shelled Parasite" | "ShelledParasite" => {
                // Source: ShelledParasite.java: inclusive 68..72, or 70..75
                // at ascension 7.
                let (base, width) = if self.run_state.ascension >= 7 {
                    (70, 5)
                } else {
                    (68, 4)
                };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "SnakePlant" => {
                // Source: reference/extracted/methods/monster/SnakePlant.java:
                // inclusive 75..79, or 78..82 at ascension 7+.
                let base = if a20 { 78 } else { 75 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "Centurion" => {
                // Source: reference/extracted/methods/monster/Centurion.java:
                // setHp(76,80), or setHp(78,83) at ascension 7+.
                let (base, width) = if a20 { (78, 5) } else { (76, 4) };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "Healer" | "Mystic" => {
                // Source: reference/extracted/methods/monster/Healer.java:
                // inclusive 48..56, or 50..58 at ascension 7.
                let base = if self.run_state.ascension >= 7 { 50 } else { 48 };
                let hp = base + self.rng.gen_range(0..=8);
                (hp, hp)
            }
            "GremlinLeader" => {
                // Source: reference/extracted/methods/monster/GremlinLeader.java:
                // inclusive 140..148, or 145..155 at ascension 8.
                let (base, width) = if self.run_state.ascension >= 8 {
                    (145, 10)
                } else {
                    (140, 8)
                };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "BookOfStabbing" => {
                let base = if self.run_state.ascension >= 8 { 168 } else { 160 };
                let hp = base + self.rng.gen_range(0..=4);
                (hp, hp)
            }
            "SlaverBoss" | "TaskMaster" | "Taskmaster" => {
                // Source: Taskmaster.java: inclusive 54..60, or 57..64 at A8.
                let (base, width) = if self.run_state.ascension >= 8 {
                    (57, 7)
                } else {
                    (54, 6)
                };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "Snecko" => {
                // Source: reference/extracted/methods/monster/Snecko.java:
                // inclusive 114..120, or 120..125 at ascension 7+.
                let (base, width) = if self.run_state.ascension >= 7 {
                    (120, 5)
                } else {
                    (114, 6)
                };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "SphericGuardian" | "Spheric Guardian" => {
                // Source: reference/extracted/methods/monster/SphericGuardian.java:
                // the constructor passes a fixed 20 HP and never calls setHp.
                (20, 20)
            }
            "BronzeAutomaton" => {
                let hp = if self.run_state.ascension >= 9 { 320 } else { 300 };
                (hp, hp)
            }
            "BronzeOrb" | "Bronze Orb" => {
                let base = if self.run_state.ascension >= 9 { 54 } else { 52 };
                let hp = base + self.rng.gen_range(0..=6);
                (hp, hp)
            }
            "TorchHead" | "Torch Head" => {
                // Source: reference/extracted/methods/monster/TorchHead.java:
                // inclusive 38..40, raised to inclusive 40..45 at A9.
                let (base, width) = if self.run_state.ascension >= 9 {
                    (40, 5)
                } else {
                    (38, 2)
                };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "TheCollector" => {
                // Source: reference/extracted/methods/monster/TheCollector.java:
                // fixed 282 HP, raised to fixed 300 at ascension 9.
                let hp = if self.run_state.ascension >= 9 { 300 } else { 282 };
                (hp, hp)
            }
            "Champ" | "TheChamp" => {
                // Source: Champ.java changes boss HP at A9, not the ordinary
                // monster A7 threshold represented by `a20` above.
                let hp = if self.run_state.ascension >= 9 { 440 } else { 420 };
                (hp, hp)
            }
            // Act 3 enemies
            "Darkling" => {
                // Source: Darkling.java uses inclusive 48..56 / 50..59 rolls.
                let (base, width) = if a20 { (50, 9) } else { (48, 8) };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "OrbWalker" | "Orb Walker" => {
                // Source: reference/extracted/methods/monster/OrbWalker.java:
                // inclusive 90..96, or 92..102 at ascension 7.
                let (base, width) = if self.run_state.ascension >= 7 {
                    (92, 10)
                } else {
                    (90, 6)
                };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "Repulsor" => {
                // Source: reference/extracted/methods/monster/Repulsor.java:
                // inclusive 29..35, or 31..38 at ascension 7.
                let (base, width) = if self.run_state.ascension >= 7 {
                    (31, 7)
                } else {
                    (29, 6)
                };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "Spiker" => {
                // Source: reference/extracted/methods/monster/Spiker.java:
                // inclusive 42..56, or 44..60 at ascension 7+.
                let (base, width) = if self.run_state.ascension >= 7 {
                    (44, 16)
                } else {
                    (42, 14)
                };
                let hp = base + self.rng.gen_range(0..=width);
                (hp, hp)
            }
            "Exploder" => {
                // Source: reference/extracted/methods/monster/Exploder.java.
                // A7 changes fixed 30 HP to an inclusive 30..35 roll.
                let hp = if self.run_state.ascension >= 7 {
                    30 + self.rng.gen_range(0..=5)
                } else {
                    30
                };
                (hp, hp)
            }
            "WrithingMass" => {
                let hp = if a20 { 175 } else { 160 };
                (hp, hp)
            }
            "GiantHead" => {
                // Source: reference/extracted/methods/monster/GiantHead.java:
                // the fixed HP increase is at ascension 8.
                let hp = if self.run_state.ascension >= 8 { 520 } else { 500 };
                (hp, hp)
            }
            "Nemesis" => {
                // Source: reference/extracted/methods/monster/Nemesis.java:
                // fixed HP changes at ascension 8, not ascension 7.
                let hp = if self.run_state.ascension >= 8 { 200 } else { 185 };
                (hp, hp)
            }
            "Reptomancer" => {
                // Source: Reptomancer.java: inclusive 180..190, or 190..200
                // at ascension 8. The constructor's earlier roll is a
                // run-stream detail; semantic HP uses the final setHp range.
                let base = if self.run_state.ascension >= 8 { 190 } else { 180 };
                let hp = base + self.rng.gen_range(0..=10);
                (hp, hp)
            }
            "SnakeDagger" | "Snake Dagger" => {
                // Source: reference/extracted/methods/monster/SnakeDagger.java.
                let hp = 20 + self.rng.gen_range(0..=5);
                (hp, hp)
            }
            "Transient" => {
                let hp = if a20 { 1000 } else { 999 };
                (hp, hp)
            }
            "Maw" => {
                // Source: reference/extracted/methods/monster/Maw.java: the
                // constructor passes fixed 300 HP at every ascension.
                (300, 300)
            }
            "Serpent" | "SpireGrowth" | "Spire Growth" => {
                // Source: SpireGrowth.java: fixed 170 HP, or 190 at A7.
                let hp = if self.run_state.ascension >= 7 { 190 } else { 170 };
                (hp, hp)
            }
            "AwakenedOne" => {
                let hp = if self.run_state.ascension >= 9 { 320 } else { 300 };
                (hp, hp)
            }
            "TimeEater" => {
                // Source: reference/extracted/methods/monster/TimeEater.java:
                // fixed 456 HP, raised to fixed 480 at ascension 9.
                let hp = if self.run_state.ascension >= 9 { 480 } else { 456 };
                (hp, hp)
            }
            "DonuAndDeca" | "Donu" | "Deca" => {
                // Source: Deca.java (and Donu.java) changes HP at A9.
                let hp = if self.run_state.ascension >= 9 { 265 } else { 250 };
                (hp, hp)
            }
            // Act 4 enemies
            "SpireShield" | "Spire Shield" => {
                // Source: reference/extracted/methods/monster/SpireShield.java:
                // fixed 110 HP, raised to fixed 125 at ascension 8.
                let hp = if self.run_state.ascension >= 8 { 125 } else { 110 };
                (hp, hp)
            }
            "SpireSpear" | "Spire Spear" => {
                // Source: reference/extracted/methods/monster/SpireSpear.java:
                // fixed 160 HP, raised to fixed 180 at ascension 8.
                let hp = if self.run_state.ascension >= 8 { 180 } else { 160 };
                (hp, hp)
            }
            "CorruptHeart" => {
                // Source: CorruptHeart.java changes max HP at A9.
                let hp = if self.run_state.ascension >= 9 { 800 } else { 750 };
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
                self.run_state.deck_card_states = engine.state.master_deck.clone();
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

                // Update run state from combat result
                self.run_state.current_hp = engine.state.player.hp;
                self.run_state.max_hp = engine.state.player.max_hp;
                let recovered_stolen_gold: i32 = engine.state.enemies.iter()
                    .filter(|enemy| matches!(enemy.id.as_str(), "Looter" | "Mugger")
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
                self.run_state.deck_card_states = engine.state.master_deck.clone();
                // Combat gainGold calls update run_gold immediately. It was
                // synchronized above after the accepted action, so replaying
                // pending_run_gold here would award the same gold twice.
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

                let room_type = if self.run_state.map_y >= 0 {
                    self.map.rows[self.run_state.map_y as usize][self.run_state.map_x as usize].room_type
                } else {
                    RoomType::Monster
                };
                let is_boss = self.run_state.floor >= 16 || room_type == RoomType::Boss;
                let is_final_heart = self.run_state.act == 4
                    && combat_enemy_ids.len() == 1
                    && combat_enemy_ids[0] == "CorruptHeart";

                // AbstractRoom.java creates exactly one gold reward band per
                // ordinary, elite, or boss room. RewardItem.java then applies
                // Golden Idol's rounded 25% bonus to that complete base amount.
                // The run engine auto-claims combat gold because it has no gold
                // reward decision item, preserving the same final run state.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/rooms/AbstractRoom.java
                // Java: decompiled/java-src/com/megacrit/cardcrawl/rewards/RewardItem.java
                if !is_final_heart {
                    let base_gold = if is_boss {
                        let rolled = self.rng.gen_range(95..=105);
                        if self.run_state.ascension >= 13 {
                            ((rolled as f32) * 0.75).round() as i32
                        } else {
                            rolled
                        }
                    } else if room_type == RoomType::Elite {
                        self.rng.gen_range(25..=35)
                    } else {
                        self.rng.gen_range(10..=20)
                    };
                    let gold = golden_idol_combat_gold(
                        base_gold,
                        self.run_state
                            .relic_flags
                            .has(crate::relic_flags::flag::GOLDEN_IDOL),
                    );
                    self.adjust_run_gold(gold);
                }

                if room_type == RoomType::Elite {
                    self.run_state.elites_killed += 1;
                }

                // Check if boss
                if is_boss {
                    self.run_state.bosses_killed += 1;
                    reward += 5.0; // Boss kill bonus
                    if is_final_heart {
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
        let prismatic = self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::PRISMATIC_SHARD);
        let rare_chance = if self
            .run_state
            .relics
            .iter()
            .any(|relic| relic == "Nloth's Gift")
        {
            // NlothsGift.java::changeRareCardRewardChance multiplies the
            // room's rare chance by exactly three; AbstractRoom leaves the
            // uncommon chance unchanged, so the extra rare share displaces
            // common cards.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/NlothsGift.java
            0.21
        } else {
            0.07
        };
        let uncommon_chance = 0.33;
        let common_chance = 1.0 - uncommon_chance - rare_chance;
        for choice_index in 0..count {
            let roll: f32 = self.rng.gen();
            let rarity = if roll < common_chance {
                EventCardRarity::Common
            } else if roll < common_chance + uncommon_chance {
                EventCardRarity::Uncommon
            } else {
                EventCardRarity::Rare
            };
            let card = if prismatic {
                // AbstractDungeon.getRewardCards calls getAnyColorCard for a
                // PrismaticShard owner, preserving the rolled rarity while
                // sampling red, green, blue, and purple source pools.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
                let mut candidates = [
                    EventCardColor::Red,
                    EventCardColor::Green,
                    EventCardColor::Blue,
                    EventCardColor::Purple,
                ]
                .iter()
                .flat_map(|color| matching_event_cards(*color, rarity))
                .filter(|card_id| {
                    crate::cards::global_registry().get(card_id.as_str()).is_some()
                        && !cards.iter().any(|choice| {
                            matches!(choice, RewardChoice::Card { card_id: chosen, .. } if chosen == card_id)
                        })
                })
                .collect::<Vec<_>>();
                if candidates.is_empty() {
                    candidates = matching_event_cards(EventCardColor::Purple, rarity);
                }
                candidates[self.rng.gen_range(0..candidates.len())].clone()
            } else {
                let pool = match rarity {
                    EventCardRarity::Common => WATCHER_COMMON_CARDS,
                    EventCardRarity::Uncommon => WATCHER_UNCOMMON_CARDS,
                    EventCardRarity::Rare => WATCHER_RARE_CARDS,
                    _ => unreachable!("reward rarity is common, uncommon, or rare"),
                };
                pool[self.rng.gen_range(0..pool.len())].to_string()
            };
            cards.push(RewardChoice::Card {
                index: choice_index,
                card_id: self.upgrade_reward_card_if_needed(&card),
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
            "Lantern", "Omamori", "Vajra",
        ];
        const UNCOMMON: &[&str] = &[
            "Blue Candle", "Darkstone Periapt", "Eternal Feather", "InkBottle",
            "Kunai", "Letter Opener", "Ornamental Fan", "White Beast Statue",
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
            .filter(|id| *id != "Omamori" || self.run_state.floor <= 48)
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

        let mut items = Vec::new();

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

        items.push(RewardItem {
            index: items.len(),
            kind: RewardItemKind::Gold,
            state: RewardItemState::Available,
            label: gold.to_string(),
            claimable: items.is_empty(),
            active: false,
            skip_allowed: false,
            skip_label: None,
            choices: Vec::new(),
        });
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

        let has_nloths_mask = self
            .run_state
            .relics
            .iter()
            .any(|relic| relic == "NlothsMask");
        if has_nloths_mask
            && self.run_state.relic_flags.counters
                [crate::relic_flags::counter::NLOTHS_MASK]
                > 0
        {
            // NlothsMask.java::onChestOpenAfter runs after Matryoshka's
            // onChestOpen reward and the chest's own relic reward are queued.
            // AbstractRoom.removeOneRelicFromRewards removes the first relic.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/NlothsMask.java
            self.run_state.relic_flags.counters[crate::relic_flags::counter::NLOTHS_MASK] = -2;
            if let Some(index) = items
                .iter()
                .position(|item| item.kind == RewardItemKind::Relic)
            {
                items.remove(index);
            }
        }

        for (index, item) in items.iter_mut().enumerate() {
            item.index = index;
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

            if source == RewardScreenSource::Shop {
                self.phase = RunPhase::Shop;
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
        let chose_tiny_house =
            matches!(&choice, RewardChoice::Named { label, .. } if label == "Tiny House");
        let chose_empty_cage =
            matches!(&choice, RewardChoice::Named { label, .. } if label == "Empty Cage");
        let astrolabe_card_pick = item_label == "deck_selection_astrolabe";
        let bottled_flame_card_pick = item_label == "deck_selection_bottled_flame";
        let bottled_lightning_card_pick = item_label == "deck_selection_bottled_lightning";
        let bottled_tornado_card_pick = item_label == "deck_selection_bottled_tornado";
        let empty_cage_card_pick = item_label == "deck_selection_empty_cage";

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
        } else if chose_tiny_house {
            self.build_tiny_house_reward_screen();
        } else if chose_empty_cage && self.pending_empty_cage_removals > 0 {
            self.build_empty_cage_selection_screen();
        } else if empty_cage_card_pick && self.pending_empty_cage_removals > 0 {
            self.build_empty_cage_selection_screen();
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
            "deck_selection_empty_cage" => {
                self.remove_master_deck_card(*deck_index);
                self.pending_empty_cage_removals =
                    self.pending_empty_cage_removals.saturating_sub(1);
                true
            }
            "deck_selection_dollys_mirror" => {
                obtain_master_deck_card_state(&mut self.run_state, card_id.clone());
                true
            }
            _ => false,
        }
    }

    fn remove_master_deck_card(&mut self, deck_index: usize) -> Option<String> {
        if deck_index >= self.run_state.deck.len() {
            return None;
        }
        self.run_state.reconcile_deck_card_states();
        let removed = self.run_state.deck.remove(deck_index);
        self.run_state.deck_card_states.remove(deck_index);
        self.apply_master_deck_removal_hook(&removed);
        Some(removed)
    }

    fn apply_master_deck_removal_hook(&mut self, card_id: &str) {
        if card_id == "Parasite" {
            // Parasite.onRemoveFromMasterDeck calls decreaseMaxHealth(3),
            // which floors max HP at one and clamps current HP to the new max.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Parasite.java
            // Java: decompiled/java-src/com/megacrit/cardcrawl/core/AbstractCreature.java
            self.run_state.max_hp = (self.run_state.max_hp - 3).max(1);
            self.run_state.current_hp = self.run_state.current_hp.min(self.run_state.max_hp);
        } else if card_id == "Necronomicurse" {
            // onRemoveFromMasterDeck constructs NecronomicurseEffect, whose
            // constructor immediately adds a fresh curse to the master deck.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Necronomicurse.java
            // Java: decompiled/java-src/com/megacrit/cardcrawl/vfx/NecronomicurseEffect.java
            obtain_master_deck_card_state(
                &mut self.run_state,
                "Necronomicurse".to_string(),
            );
        }
    }

    fn resolve_bonfire_offer(&mut self, card_id: &str) {
        match event_card_rarity(card_id) {
            Some(EventCardRarity::Curse) => {
                // Bonfire.java grants Spirit Poop for the first offered Curse
                // and Circlet when the special relic is already owned.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/events/shrines/Bonfire.java
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

        // RewardItem.claimReward leaves a potion reward unclaimed when the
        // inventory is full, but Sozu's earlier branch returns true and
        // consumes the reward harmlessly.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/rewards/RewardItem.java
        if kind == RewardItemKind::Potion
            && !self.run_state.relic_flags.has(crate::relic_flags::flag::SOZU)
            && !self.run_state.potions.iter().any(|potion| potion.is_empty())
        {
            return;
        }

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
            // Circlet.java gives every fallback copy its own presentation-only
            // counter and has no gameplay hook; duplicate Circlets therefore
            // remain separate owned relic entries.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Circlet.java
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
            "Necronomicon" => {
                // Necronomicon.java::onEquip obtains one Necronomicurse via
                // ShowCardAndObtainEffect, including Omamori interception.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Necronomicon.java
                obtain_master_deck_card_state(
                    &mut self.run_state,
                    "Necronomicurse".to_string(),
                );
            }
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
                    for (_, price) in &mut shop.potions {
                        *price = ((*price as f32) * 0.5).round() as i32;
                    }
                    shop.remove_price = ((shop.remove_price as f32) * 0.5).round() as i32;
                }
            }
            "Cauldron" => self.build_cauldron_reward_screen(),
            "DollysMirror" => self.build_dollys_mirror_selection_screen(),
            "Orrery" => self.build_orrery_reward_screen(),
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
            "Tiny House" | "TinyHouse" => {
                // TinyHouse.java shuffles all upgradeable master-deck cards,
                // upgrades exactly the first, then increases max HP by 5.
                self.upgrade_one_random_master_deck_card();
                self.run_state.max_hp += 5;
                self.heal_run_player(5);
            }
            "Empty Cage" | "EmptyCage" => self.prepare_empty_cage_on_equip(),
            // Source: reference/extracted/methods/relic/Whetstone.java upgrades
            // up to two upgradable ATTACK cards and checks bottled upgrades.
            "Whetstone" => self.upgrade_random_cards_by_type(crate::cards::CardType::Attack, 2),
            // Source: reference/extracted/methods/relic/WarPaint.java upgrades
            // up to two upgradable SKILL cards and checks bottled upgrades.
            "War Paint" | "WarPaint" => {
                self.upgrade_random_cards_by_type(crate::cards::CardType::Skill, 2)
            }
            "Pandora's Box" | "PandorasBox" => self.apply_pandoras_box(),
            _ => {}
        }
    }

    fn build_cauldron_reward_screen(&mut self) {
        // Cauldron.onEquip queues exactly five PotionHelper random potions and
        // opens CombatRewardScreen over the shop. No card reward survives, and
        // the player returns to the same shop after resolving the five items.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Cauldron.java
        let mut items: Vec<RewardItem> = (0..5)
            .map(|index| RewardItem {
                index,
                kind: RewardItemKind::Potion,
                state: RewardItemState::Available,
                label: self.roll_reward_potion_id(),
                claimable: index == 0,
                active: false,
                skip_allowed: true,
                skip_label: Some("Skip".to_string()),
                choices: Vec::new(),
            })
            .collect();
        let mut screen = RewardScreen {
            source: RewardScreenSource::Shop,
            ordered: true,
            active_item: None,
            items: std::mem::take(&mut items),
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
        self.phase = RunPhase::CardReward;
    }

    fn build_dollys_mirror_selection_screen(&mut self) {
        // DollysMirror.onEquip opens a mandatory one-card master-deck grid.
        // update makes a stat-equivalent copy, clears all bottle flags, and
        // obtains the copy normally before returning to the shop.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/DollysMirror.java
        let choices = self.run_state.deck.iter().cloned().enumerate()
            .map(|(index, card_id)| RewardChoice::Card { index, card_id })
            .collect();
        let mut screen = RewardScreen {
            source: RewardScreenSource::Shop,
            ordered: true,
            active_item: None,
            items: vec![RewardItem {
                index: 0,
                kind: RewardItemKind::CardChoice,
                state: RewardItemState::Available,
                label: "deck_selection_dollys_mirror".to_string(),
                claimable: true,
                active: false,
                skip_allowed: false,
                skip_label: None,
                choices,
            }],
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
        self.phase = RunPhase::CardReward;
    }

    fn build_orrery_reward_screen(&mut self) {
        // Orrery.onEquip queues four card rewards. CombatRewardScreen then
        // appends ShopRoom's ordinary card reward, producing five independent
        // choices before returning to the same shop.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Orrery.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/screens/CombatRewardScreen.java
        let choice_count = self.card_reward_choice_count();
        let mut items = (0..5)
            .map(|index| RewardItem {
                index,
                kind: RewardItemKind::CardChoice,
                state: RewardItemState::Available,
                label: "orrery_card_reward".to_string(),
                claimable: index == 0,
                active: false,
                skip_allowed: true,
                skip_label: Some("Skip".to_string()),
                choices: self.generate_card_reward_choices(choice_count),
            })
            .collect::<Vec<_>>();
        let mut screen = RewardScreen {
            source: RewardScreenSource::Shop,
            ordered: true,
            active_item: None,
            items: std::mem::take(&mut items),
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
        self.phase = RunPhase::CardReward;
    }

    fn apply_pandoras_box(&mut self) {
        // PandorasBox.java removes cards by the STARTER_STRIKE/STARTER_DEFEND
        // tags, then returnTrulyRandomCard draws uniformly from the complete
        // class source pools. Confirmation obtains each previewed card through
        // FastCardObtainEffect, preserving Egg and on-obtain relic callbacks.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/PandorasBox.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
        let before = self.run_state.deck.len();
        self.run_state.deck.retain(|card| {
            !matches!(card.as_str(), "Strike" | "Strike+" | "Defend" | "Defend+")
        });
        let count = before - self.run_state.deck.len();
        if count == 0 {
            return;
        }

        let pool_len = WATCHER_COMMON_CARDS.len()
            + WATCHER_UNCOMMON_CARDS.len()
            + WATCHER_RARE_CARDS.len();
        for _ in 0..count {
            let idx = self.rng.gen_range(0..pool_len);
            let card = if idx < WATCHER_COMMON_CARDS.len() {
                WATCHER_COMMON_CARDS[idx]
            } else if idx < WATCHER_COMMON_CARDS.len() + WATCHER_UNCOMMON_CARDS.len() {
                WATCHER_UNCOMMON_CARDS[idx - WATCHER_COMMON_CARDS.len()]
            } else {
                WATCHER_RARE_CARDS[idx - WATCHER_COMMON_CARDS.len() - WATCHER_UNCOMMON_CARDS.len()]
            };
            obtain_master_deck_card_state(&mut self.run_state, card.to_string());
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

    fn prepare_empty_cage_on_equip(&mut self) {
        // CardGroup.getPurgeableCards excludes exactly these three curses.
        // EmptyCage.java removes the whole pool immediately when it has at
        // most two entries, otherwise it requires exactly two selections.
        let purgeable: Vec<usize> = self
            .run_state
            .deck
            .iter()
            .enumerate()
            .filter_map(|(idx, card)| Self::is_purgeable_master_deck_card(card).then_some(idx))
            .collect();
        if purgeable.len() <= 2 {
            for idx in purgeable.into_iter().rev() {
                self.remove_master_deck_card(idx);
            }
            self.pending_empty_cage_removals = 0;
        } else {
            self.pending_empty_cage_removals = 2;
        }
    }

    fn build_empty_cage_selection_screen(&mut self) {
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
                label: "deck_selection_empty_cage".to_string(),
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
            .filter(|id| *id != "Omamori" || self.run_state.floor <= 48)
            .filter(|id| *id != "Smiling Mask" || self.run_state.floor <= 48)
            .filter(|id| *id != "Tiny Chest" || self.run_state.floor <= 35)
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
            // Omamori.java constructs a COMMON relic and canSpawn excludes
            // non-endless runs after floor 48.
            "Omamori",
            "Smiling Mask",
            "Tiny Chest",
            "Toy Ornithopter",
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
            "Sundial",
            "Toxic Egg 2",
            "Yang",
            "White Beast Statue",
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
            "StoneCalendar",
            "Tingsha",
            "Torii",
            "Turnip",
            "Unceasing Top",
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

    fn build_tiny_house_reward_screen(&mut self) {
        // TinyHouse.java adds gold first, then a miscRng-selected potion, and
        // opens the current room's ordered combat reward screen.
        let potion = self.roll_reward_potion_id();
        let mut screen = RewardScreen {
            source: RewardScreenSource::BossCombat,
            ordered: true,
            active_item: None,
            items: vec![
                RewardItem {
                    index: 0,
                    kind: RewardItemKind::Gold,
                    state: RewardItemState::Available,
                    label: "50".to_string(),
                    claimable: true,
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: Vec::new(),
                },
                RewardItem {
                    index: 1,
                    kind: RewardItemKind::Potion,
                    state: RewardItemState::Available,
                    label: potion,
                    claimable: false,
                    active: false,
                    skip_allowed: true,
                    skip_label: Some("Skip".to_string()),
                    choices: Vec::new(),
                },
            ],
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
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
            if relic_id == "Necronomicon" {
                // onUnequip removes the first Necronomicurse after the relic
                // has left the player, so the curse's removal hook does not
                // recreate it.
                if let Some(card_index) = self
                    .run_state
                    .deck
                    .iter()
                    .position(|card| card == "Necronomicurse")
                {
                    self.run_state.deck.remove(card_index);
                }
            }
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
                let def = registry.get(card_id)?;
                if def.card_type == card_type && registry.can_upgrade_name(card_id) {
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
            let Some((original, upgraded)) = self.run_state.upgrade_deck_card(deck_idx) else {
                continue;
            };
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

    fn upgrade_one_random_master_deck_card(&mut self) {
        let registry = crate::cards::global_registry();
        let eligible: Vec<usize> = self
            .run_state
            .deck
            .iter()
            .enumerate()
            .filter_map(|(idx, card_id)| {
                registry.can_upgrade_name(card_id).then_some(idx)
            })
            .collect();
        if eligible.is_empty() {
            return;
        }

        // TinyHouse.java consumes miscRng.randomLong() to seed a shuffle and
        // upgrades the first result. RunEngine's shared run stream preserves
        // the semantic random choice until run-level streams are split.
        let deck_idx = eligible[self.rng.gen_range(0..eligible.len())];
        let Some((original, upgraded)) = self.run_state.upgrade_deck_card(deck_idx) else {
            return;
        };
        if self.run_state.bottled_flame_card.as_deref() == Some(original.as_str()) {
            self.run_state.bottled_flame_card = Some(upgraded.clone());
        }
        if self.run_state.bottled_lightning_card.as_deref() == Some(original.as_str()) {
            self.run_state.bottled_lightning_card = Some(upgraded.clone());
        }
        if self.run_state.bottled_tornado_card.as_deref() == Some(original.as_str()) {
            self.run_state.bottled_tornado_card = Some(upgraded);
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
            // DeadBranch.java uses canonical ID "Dead Branch" and RARE tier.
            "Dead Branch",
            // GamblingChip.java uses canonical ID "Gambling Chip" and RARE tier.
            "Gambling Chip",
            // BloodVial.java uses this canonical ID and COMMON tier.
            "Blood Vial",
            // BlueCandle.java uses this canonical ID and UNCOMMON tier.
            "Blue Candle",
            // HornCleat.java uses canonical ID "HornCleat" and UNCOMMON tier.
            "HornCleat",
            // GremlinHorn.java uses canonical ID "Gremlin Horn" and UNCOMMON tier.
            "Gremlin Horn",
            // Boot.java uses canonical ID "Boot" and COMMON tier.
            "Boot",
            // ToyOrnithopter.java uses canonical ID "Toy Ornithopter" and
            // COMMON tier.
            "Toy Ornithopter",
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
            // TinyChest.java uses canonical ID "Tiny Chest", COMMON tier, and
            // canSpawn permits non-endless runs through floor 35.
            "Tiny Chest",
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
            // StoneCalendar.java uses canonical ID "StoneCalendar" and RARE tier.
            "StoneCalendar",
            // Tingsha.java uses canonical ID "Tingsha" and RARE tier.
            "Tingsha",
            // Torii.java uses canonical ID "Torii" and RARE tier.
            "Torii",
            // Turnip.java uses canonical ID "Turnip" and RARE tier.
            "Turnip",
            // UnceasingTop.java uses canonical ID "Unceasing Top" and RARE tier.
            "Unceasing Top",
            // ToughBandages.java uses canonical ID "Tough Bandages" and RARE tier.
            "Tough Bandages",
            // TungstenRod.java uses canonical ID "TungstenRod" and RARE tier.
            "TungstenRod",
            // Matryoshka.java uses canonical ID "Matryoshka", UNCOMMON tier,
            // and canSpawn excludes non-endless runs after floor 40.
            "Matryoshka",
            // WingBoots.java declares canonical ID WingedGreaves at RARE tier
            // and canSpawn excludes non-endless runs after floor 40.
            "WingedGreaves",
            // Courier.java declares canonical ID "The Courier", UNCOMMON
            // tier, and excludes floors after 48 plus the current shop room.
            "The Courier",
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
            // Omamori.java uses canonical ID "Omamori", COMMON tier, and
            // canSpawn excludes non-endless runs after floor 48.
            "Omamori",
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
            // Sundial.java uses canonical ID "Sundial" and UNCOMMON tier.
            "Sundial",
            // Duality.java declares canonical ID "Yang" and UNCOMMON tier.
            "Yang",
            // WhiteBeast.java declares canonical ID "White Beast Statue" at
            // UNCOMMON tier; AbstractRoom forces potion chance to 100.
            "White Beast Statue",
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
            .filter(|relic| *relic != "WingedGreaves" || self.run_state.floor <= 40)
            .filter(|relic| *relic != "The Courier" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "MawBank" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "MealTicket" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Meat on the Bone" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Old Coin" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Omamori" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Smiling Mask" || self.run_state.floor <= 48)
            .filter(|relic| *relic != "Tiny Chest" || self.run_state.floor <= 35)
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
                !in_shop || !matches!(*relic, "Old Coin" | "Smiling Mask" | "The Courier")
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
                    .filter(|relic| *relic != "WingedGreaves" || self.run_state.floor <= 40)
                    .filter(|relic| *relic != "The Courier" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "MawBank" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "MealTicket" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Meat on the Bone" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Old Coin" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Omamori" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Smiling Mask" || self.run_state.floor <= 48)
                    .filter(|relic| *relic != "Tiny Chest" || self.run_state.floor <= 35)
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
                        !in_shop || !matches!(*relic, "Old Coin" | "Smiling Mask" | "The Courier")
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
            "Bronze Scales", "CeramicFish", "Dream Catcher", "Lantern", "Omamori", "Vajra",
        ];
        const UNCOMMON: &[&str] = &[
            "Blue Candle", "Bottled Flame", "Bottled Lightning", "Bottled Tornado",
            "Darkstone Periapt", "Eternal Feather", "Frozen Egg 2", "InkBottle",
            "Kunai", "Letter Opener", "Matryoshka", "Molten Egg 2",
            "Ornamental Fan", "Toxic Egg 2", "White Beast Statue",
        ];

        let pool = if self.rng.gen_bool(0.75) { COMMON } else { UNCOMMON };
        let registry = gameplay_registry();
        let candidates: Vec<&str> = pool
            .iter()
            .copied()
            .filter(|id| registry.get(GameplayDomain::Relic, id).is_some())
            .filter(|id| *id != "Omamori" || self.run_state.floor <= 48)
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
            // RunicDome.java constructs canonical ID "Runic Dome" at BOSS
            // tier and increments energyMaster on equip.
            "Runic Dome",
            // RunicPyramid.java constructs canonical ID "Runic Pyramid" at
            // BOSS tier; DiscardAtEndOfTurnAction.java supplies its behavior.
            "Runic Pyramid",
            // SacredBark.java constructs canonical ID "SacredBark" at BOSS
            // tier and reinitializes all currently owned potion data on equip.
            "SacredBark",
            "Velvet Choker",
            "Snecko Eye",
            // Sozu.java constructs canonical ID "Sozu" at BOSS tier and
            // increments energyMaster on equip.
            "Sozu",
            // TinyHouse.java constructs canonical ID "Tiny House" at BOSS
            // tier and opens its own follow-up reward screen on equip.
            "Tiny House",
            // EmptyCage.java constructs canonical ID "Empty Cage" at BOSS
            // tier and opens an exact two-card purge selection when needed.
            "Empty Cage",
            "HolyWater",
            "VioletLotus",
            "Astrolabe",
            "Black Star",
            // PandorasBox.java constructs canonical ID "Pandora's Box" at
            // BOSS tier and replaces starter-tagged Strikes and Defends.
            "Pandora's Box",
            // SlaversCollar.java constructs canonical ID SlaversCollar at
            // BOSS tier and changes energyMaster only in elite/boss combats.
            "SlaversCollar",
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
        // AbstractRoom.addPotionToRewards rolls independently of Sozu and
        // available inventory slots. RewardItem handles both at claim time.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/rooms/AbstractRoom.java
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
                self.run_state.upgrade_deck_card(*idx);
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

    fn rounded_shop_price(price: i32, multiplier: f32) -> i32 {
        ((price as f32) * multiplier).round() as i32
    }

    fn apply_shop_entry_discounts(&self, mut price: i32) -> i32 {
        // ShopScreen.init applies these to already-priced stock in this exact
        // order, rounding after every multiplier.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/shop/ShopScreen.java
        if self.run_state.ascension >= 16 {
            price = Self::rounded_shop_price(price, 1.1);
        }
        if self.run_state.relic_flags.has(crate::relic_flags::flag::THE_COURIER) {
            price = Self::rounded_shop_price(price, 0.8);
        }
        if self.run_state.relic_flags.has(crate::relic_flags::flag::MEMBERSHIP_CARD) {
            price = Self::rounded_shop_price(price, 0.5);
        }
        price
    }

    fn apply_shop_replacement_discounts(&self, mut price: i32) -> i32 {
        // StoreRelic/StorePotion round each replacement discount after the
        // merchant variance has already been rounded.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/shop/ShopScreen.java
        if self.run_state.relic_flags.has(crate::relic_flags::flag::THE_COURIER) {
            price = Self::rounded_shop_price(price, 0.8);
        }
        if self.run_state.relic_flags.has(crate::relic_flags::flag::MEMBERSHIP_CARD) {
            price = Self::rounded_shop_price(price, 0.5);
        }
        price
    }

    fn roll_shop_colored_card(
        &mut self,
        card_type: crate::cards::CardType,
        exclude: Option<&str>,
    ) -> (String, i32) {
        // ShopRoom disables reward alternation: 3% rare, 37% uncommon, 60%
        // common. getCardFromPool falls through when a rarity has no card of
        // the requested type (notably Watcher common powers).
        // Java: decompiled/java-src/com/megacrit/cardcrawl/rooms/ShopRoom.java
        let roll = self.rng.gen_range(0..100);
        let ordered: [(&[&str], i32); 3] = if roll < 3 {
            [(WATCHER_RARE_CARDS, 150), (WATCHER_UNCOMMON_CARDS, 75), (WATCHER_COMMON_CARDS, 50)]
        } else if roll < 40 {
            [(WATCHER_UNCOMMON_CARDS, 75), (WATCHER_RARE_CARDS, 150), (WATCHER_COMMON_CARDS, 50)]
        } else {
            [(WATCHER_COMMON_CARDS, 50), (WATCHER_UNCOMMON_CARDS, 75), (WATCHER_RARE_CARDS, 150)]
        };
        let registry = crate::cards::global_registry();
        for (pool, base_price) in ordered {
            let candidates: Vec<&str> = pool.iter().copied()
                .filter(|id| exclude != Some(*id))
                .filter(|id| registry.get(id).is_some_and(|card| card.card_type == card_type))
                .collect();
            if !candidates.is_empty() {
                let card = candidates[self.rng.gen_range(0..candidates.len())];
                let price = self.rng.gen_range((base_price * 9 / 10)..=(base_price * 11 / 10));
                return (card.to_string(), price);
            }
        }
        ("Scrawl".to_string(), self.rng.gen_range(135..=165))
    }

    fn roll_shop_colorless_card(&mut self, rare: bool) -> (String, i32) {
        let (pool, base_price) = if rare {
            (SHOP_COLORLESS_RARE_CARDS, 150)
        } else {
            (SHOP_COLORLESS_UNCOMMON_CARDS, 75)
        };
        let card = pool[self.rng.gen_range(0..pool.len())];
        // ShopScreen.initCards multiplies colorless prices by 1.2 before the
        // float-to-int truncation.
        let min = ((base_price as f32) * 0.9 * 1.2) as i32;
        let max = ((base_price as f32) * 1.1 * 1.2) as i32;
        (card.to_string(), self.rng.gen_range(min..=max))
    }

    fn is_shop_colorless_card(card_id: &str) -> bool {
        let card_id = card_id.strip_suffix('+').unwrap_or(card_id);
        SHOP_COLORLESS_UNCOMMON_CARDS.contains(&card_id)
            || SHOP_COLORLESS_RARE_CARDS.contains(&card_id)
    }

    fn roll_courier_replacement_card(&mut self, purchased: &str) -> (String, i32) {
        let (card, raw_price) = if Self::is_shop_colorless_card(purchased) {
            let rare = self.rng.gen_bool(0.3);
            self.roll_shop_colorless_card(rare)
        } else {
            let base_id = purchased.strip_suffix('+').unwrap_or(purchased);
            let card_type = crate::cards::global_registry().get(base_id)
                .map(|card| card.card_type)
                .unwrap_or(crate::cards::CardType::Skill);
            self.roll_shop_colored_card(card_type, None)
        };
        // ShopScreen.setPrice applies all card multipliers in float and then
        // truncates once, unlike relic/potion refill rounding.
        let mut multiplier = 1.0;
        if self.run_state.relic_flags.has(crate::relic_flags::flag::THE_COURIER) {
            multiplier *= 0.8;
        }
        if self.run_state.relic_flags.has(crate::relic_flags::flag::MEMBERSHIP_CARD) {
            multiplier *= 0.5;
        }
        (self.upgrade_reward_card_if_needed(&card), ((raw_price as f32) * multiplier) as i32)
    }

    fn potion_base_shop_price(potion_id: &str) -> i32 {
        let key: String = potion_id.chars().filter(|ch| ch.is_ascii_alphanumeric())
            .flat_map(|ch| ch.to_lowercase()).collect();
        if matches!(key.as_str(), "ambrosia" | "cultistpotion" | "fruitjuice" | "sneckooil" | "fairypotion" | "smokebomb" | "entropicbrew") {
            100
        } else if matches!(key.as_str(), "stancepotion" | "regenpotion" | "ancientpotion" | "liquidbronze" | "gamblersbrew" | "essenceofsteel" | "duplicationpotion" | "distilledchaos" | "liquidmemories") {
            75
        } else {
            50
        }
    }

    fn roll_shop_potion(&mut self) -> (String, i32) {
        let potion = self.roll_reward_potion_id();
        let base = Self::potion_base_shop_price(&potion);
        let min = ((base as f32) * 0.95).round() as i32;
        let max = ((base as f32) * 1.05).round() as i32;
        (potion, self.rng.gen_range(min..=max))
    }

    fn relic_base_shop_price(relic_id: &str) -> i32 {
        // AbstractRelic.getPrice: COMMON=150, UNCOMMON=250, RARE=300.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/AbstractRelic.java
        if matches!(relic_id,
            "Blue Candle" | "Bottled Flame" | "Bottled Lightning" | "Bottled Tornado"
            | "The Courier" | "Darkstone Periapt" | "Yang" | "Eternal Feather"
            | "Frozen Egg 2" | "InkBottle" | "Kunai" | "Letter Opener" | "Matryoshka"
            | "Meat on the Bone" | "Mercury Hourglass" | "Molten Egg 2" | "Mummified Hand"
            | "Ornamental Fan" | "Pantograph" | "Pear" | "Question Card" | "Shuriken"
            | "Singing Bowl" | "Sundial" | "Toxic Egg 2" | "White Beast Statue") {
            250
        } else if matches!(relic_id,
            "Bird Faced Urn" | "Calipers" | "Du-Vu Doll" | "FossilizedHelix" | "Ginger"
            | "Girya" | "Ice Cream" | "Incense Burner" | "Lizard Tail" | "Magic Flower"
            | "Mango" | "Old Coin" | "Peace Pipe" | "Pocketwatch" | "Prayer Wheel"
            | "Shovel" | "StoneCalendar" | "Thread and Needle" | "Tingsha" | "Torii"
            | "Tough Bandages" | "TungstenRod" | "Turnip" | "Unceasing Top"
            | "WingedGreaves") {
            300
        } else {
            150
        }
    }

    fn roll_relic_merchant_price(&mut self, relic_id: &str) -> i32 {
        let base = Self::relic_base_shop_price(relic_id);
        let min = ((base as f32) * 0.95).round() as i32;
        let max = ((base as f32) * 1.05).round() as i32;
        self.rng.gen_range(min..=max)
    }

    fn roll_courier_replacement_relic(&mut self) -> (String, i32) {
        // StoreRelic excludes these four on every Courier refill.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/shop/StoreRelic.java
        let relic = (0..64).map(|_| self.roll_shop_reward_relic_id())
            .find(|id| !matches!(id.as_str(), "Old Coin" | "Smiling Mask" | "MawBank" | "The Courier"))
            .unwrap_or_else(|| "Circlet".to_string());
        let raw_price = self.roll_relic_merchant_price(&relic);
        (relic, self.apply_shop_replacement_discounts(raw_price))
    }

    fn enter_shop(&mut self) {
        // Merchant.java creates two attacks, two skills, one power, then one
        // uncommon and one rare colorless card. ShopScreen chooses one of the
        // five colored cards for the half-price sale before global discounts.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/shop/Merchant.java
        let mut cards = Vec::new();
        let first_attack = self.roll_shop_colored_card(crate::cards::CardType::Attack, None);
        let second_attack = self.roll_shop_colored_card(crate::cards::CardType::Attack, Some(first_attack.0.as_str()));
        let first_skill = self.roll_shop_colored_card(crate::cards::CardType::Skill, None);
        let second_skill = self.roll_shop_colored_card(crate::cards::CardType::Skill, Some(first_skill.0.as_str()));
        cards.extend([first_attack, second_attack, first_skill, second_skill]);
        cards.push(self.roll_shop_colored_card(crate::cards::CardType::Power, None));
        let sale_idx = self.rng.gen_range(0..5);
        cards[sale_idx].1 /= 2;
        cards.push(self.roll_shop_colorless_card(false));
        cards.push(self.roll_shop_colorless_card(true));
        for (card, price) in &mut cards {
            *card = self.upgrade_reward_card_if_needed(card);
            *price = self.apply_shop_entry_discounts(*price);
        }

        let remove_price = self.compute_shop_remove_price();

        // ShopScreen.java::initRelics creates two ordinary tier rolls followed
        // by one guaranteed SHOP-tier relic. The run RNG is still shared, but
        // the visible slot semantics and purchase path match Java.
        let mut relics = Vec::new();
        for _ in 0..2 {
            let relic = self.roll_shop_reward_relic_id();
            let price = self.roll_relic_merchant_price(&relic);
            let final_price = self.apply_shop_entry_discounts(price);
            relics.push((relic, final_price));
        }
        const SHOP_RELICS: &[&str] = &[
            "TheAbacus", "Brimstone", "Cauldron", "Chemical X", "ClockworkSouvenir", "DollysMirror", "Frozen Eye", "HandDrill", "Lee's Waffle", "Medical Kit", "Melange", "Orrery",
            "Membership Card",
            "OrangePellets", "Runic Capacitor", "Sling", "Strange Spoon",
            "PrismaticShard", "Toolbox", "TwistedFunnel",
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
        let shop_price = self.apply_shop_entry_discounts(shop_price);
        relics.push((shop_relic.to_string(), shop_price));

        let potions = (0..3).map(|_| {
            let (potion, price) = self.roll_shop_potion();
            (potion, self.apply_shop_entry_discounts(price))
        }).collect();

        self.current_shop = Some(ShopState {
            cards,
            relics,
            potions,
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
                let purchase = self.current_shop.as_ref().and_then(|shop| {
                    shop.cards.get(*idx).cloned()
                        .filter(|(_, price)| self.run_state.gold >= *price)
                });
                if let Some((card, price)) = purchase {
                    self.run_state.gold -= price;
                    if self.run_state.relic_flags.has(crate::relic_flags::flag::MAW_BANK) {
                        self.run_state.relic_flags.counters[crate::relic_flags::counter::MAW_BANK_GOLD] = -2;
                    }
                    obtain_master_deck_card_state(&mut self.run_state, card.clone());
                    let replacement = self.run_state.relic_flags.has(crate::relic_flags::flag::THE_COURIER)
                        .then(|| self.roll_courier_replacement_card(&card));
                    if let Some(shop) = self.current_shop.as_mut() {
                        if let Some(replacement) = replacement { shop.cards[*idx] = replacement; }
                        else { shop.cards.remove(*idx); }
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
                    let replacement = (relic == "The Courier"
                        || self.run_state.relic_flags.has(crate::relic_flags::flag::THE_COURIER))
                        .then(|| self.roll_courier_replacement_relic());
                    if let Some(shop) = self.current_shop.as_mut() {
                        if let Some(replacement) = replacement { shop.relics[*idx] = replacement; }
                        else { shop.relics.remove(*idx); }
                    }
                }
                return 0.0;
            }
            RunAction::ShopBuyPotion(idx) => {
                let purchase = self.current_shop.as_ref().and_then(|shop| {
                    shop.potions.get(*idx).cloned().filter(|(_, price)| {
                        self.run_state.gold >= *price
                            && !self.run_state.relic_flags.has(crate::relic_flags::flag::SOZU)
                            && self.run_state.potions.iter().any(|potion| potion.is_empty())
                    })
                });
                if let Some((potion, price)) = purchase {
                    self.run_state.gold -= price;
                    if self.run_state.relic_flags.has(crate::relic_flags::flag::MAW_BANK) {
                        self.run_state.relic_flags.counters[crate::relic_flags::counter::MAW_BANK_GOLD] = -2;
                    }
                    self.add_potion_reward(&potion);
                    let replacement = self.run_state.relic_flags.has(crate::relic_flags::flag::THE_COURIER)
                        .then(|| {
                            let (potion, raw_price) = self.roll_shop_potion();
                            (potion, self.apply_shop_replacement_discounts(raw_price))
                        });
                    if let Some(shop) = self.current_shop.as_mut() {
                        if let Some(replacement) = replacement { shop.potions[*idx] = replacement; }
                        else { shop.potions.remove(*idx); }
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

    fn normalize_event_runtime_state(&mut self, event: &mut TypedEventDef) {
        self.normalize_event_runtime_statuses(event);
        if event.name == "Dead Adventurer" {
            *event = crate::events::dead_adventurer_event(self.run_state.ascension);
        } else if event.name == "Golden Idol" && event.options.len() == 3 {
            // GoldenIdolEvent.java calculates both displayed consequences from
            // maxHealth when the event is constructed.
            let (damage_percent, max_hp_percent) = if self.run_state.ascension >= 15 {
                (35, 10)
            } else {
                (25, 8)
            };
            let damage = (self.run_state.max_hp * damage_percent) / 100;
            let max_hp_loss = ((self.run_state.max_hp * max_hp_percent) / 100).max(1);
            event.options[1].text = format!("Take {damage} damage");
            event.options[2].text = format!("Lose {max_hp_loss} max HP");
        } else if event.name == "FaceTrader" && event.options.len() == 3 {
            // FaceTrader.java snapshots maxHealth/10 (minimum one) and the
            // A15-dependent gold reward when constructing the event.
            let damage = (self.run_state.max_hp / 10).max(1);
            let gold = if self.run_state.ascension >= 15 { 50 } else { 75 };
            event.options[0].text = format!("Touch (take {damage} damage, gain {gold} gold)");
        } else if event.name == "N'loth" && event.options.len() == 3 {
            // Nloth.java copies and shuffles the player's relic list, then
            // offers the first two entries. RunEngine's shared run RNG cannot
            // reproduce miscRng tick counts, but preserves the semantic random
            // sample without replacement required by the event.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/events/shrines/Nloth.java
            let mut candidates = self.run_state.relics.clone();
            if candidates.len() < 2 {
                for option in event.options.iter_mut().take(2) {
                    option.status = EventRuntimeStatus::Blocked {
                        reason: "requires at least two owned relics".to_string(),
                    };
                }
                return;
            }
            let first_index = self.rng.gen_range(0..candidates.len());
            let first = candidates.remove(first_index);
            let second_index = self.rng.gen_range(0..candidates.len());
            let second = candidates.remove(second_index);
            let already_has_gift = self
                .run_state
                .relics
                .iter()
                .any(|relic| relic == "Nloth's Gift");
            for (option, relic_id) in event.options.iter_mut().take(2).zip([first, second]) {
                option.text = format!("Trade {relic_id} for N'loth's Gift");
                option.program = crate::events::nloth_trade_program(&relic_id, already_has_gift);
            }
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
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::SSSERPENT_HEAD)
        {
            // SsserpentHead.java::onEnterRoom gains 50 gold whenever the map
            // room is an EventRoom. This fires before EventRoom.onPlayerEntry
            // reveals whether the mystery room becomes combat, shop, treasure,
            // or an event.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/SsserpentHead.java
            self.adjust_run_gold(50);
        }

        // EventHelper.java::roll fills a 100-entry table in this order:
        // MONSTER, SHOP, TREASURE, then EVENT. (Elite entries require the
        // Deadly Events modifier, which the standard run engine does not use.)
        let roll = self.next_event_roll_100() as i32;
        let force_tiny_chest = if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::TINY_CHEST)
        {
            let counter = &mut self.run_state.relic_flags.counters
                [crate::relic_flags::counter::TINY_CHEST];
            *counter += 1;
            if *counter == 4 {
                *counter = 0;
                true
            } else {
                false
            }
        } else {
            false
        };
        let monster_end = self.run_state.event_monster_chance.min(100);
        let shop_end = (monster_end + self.run_state.event_shop_chance).min(100);
        let treasure_end = (shop_end + self.run_state.event_treasure_chance).min(100);

        #[derive(Clone, Copy, PartialEq, Eq)]
        enum MysteryResult { Monster, Shop, Treasure, Event }

        let rolled = if force_tiny_chest {
            // EventHelper.java consumes eventRng.random() before incrementing
            // Tiny Chest and overriding the fourth mystery result to TREASURE.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/EventHelper.java
            MysteryResult::Treasure
        } else if roll < monster_end {
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
                self.build_treasure_reward_screen();
                self.phase = RunPhase::CardReward;
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

        if let Some(mut event) = next_event {
            self.normalize_event_runtime_state(&mut event);
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
            EventProgramOp::AdjustMaxHpPercentByAscension {
                base_percent,
                asc15_percent,
                minimum_loss,
            } => {
                let percent = if self.run_state.ascension >= 15 {
                    *asc15_percent
                } else {
                    *base_percent
                };
                let amount = if percent >= 0 {
                    (self.run_state.max_hp * percent) / 100
                } else {
                    -((self.run_state.max_hp * (-percent)) / 100).max(*minimum_loss)
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
            EventProgramOp::ResolveFaceTraderTouch => {
                // FaceTrader.java gains gold before applying maxHealth/10
                // NORMAL damage. Outside combat this resolves to the same raw
                // HP loss after Ectoplasm/Bloody Idol gold hooks.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/events/shrines/FaceTrader.java
                let gold = if self.run_state.ascension >= 15 { 50 } else { 75 };
                let damage = (self.run_state.max_hp / 10).max(1);
                self.adjust_run_gold(gold);
                self.run_state.current_hp = (self.run_state.current_hp - damage).max(0);
                if self.run_state.current_hp <= 0 {
                    EventProgramFlow::Died
                } else {
                    EventProgramFlow::Continue
                }
            }
            EventProgramOp::ObtainRandomFace => {
                // FaceTrader.java::getRandomFace filters these exact canonical
                // IDs and falls back to Circlet only when all five are owned.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/events/shrines/FaceTrader.java
                const FACE_IDS: &[&str] = &[
                    "CultistMask",
                    "FaceOfCleric",
                    "GremlinMask",
                    "NlothsMask",
                    "SsserpentHead",
                ];
                let candidates = FACE_IDS
                    .iter()
                    .copied()
                    .filter(|face| !self.run_state.relics.iter().any(|owned| owned == face))
                    .collect::<Vec<_>>();
                let face = if candidates.is_empty() {
                    "Circlet"
                } else {
                    candidates[self.rng.gen_range(0..candidates.len())]
                };
                self.add_relic_reward(face);
                EventProgramFlow::Continue
            }
            EventProgramOp::RemoveRelic { label } => {
                self.remove_relic_reward(label);
                EventProgramFlow::Continue
            }
            EventProgramOp::ObtainRelic { label } => {
                let obtained = if self.run_state.relics.iter().any(|owned| owned == label) {
                    // DrugDealer.java explicitly substitutes Circlet when its
                    // special relic is already owned.
                    "Circlet"
                } else {
                    label.as_str()
                };
                self.add_relic_reward(obtained);
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
                // Upgrade-all effects snapshot the deck and upgrade each card at
                // most once. Searing Blow remains upgradeable afterward, but the
                // same action must not repeatedly select that one copy.
                let indices: Vec<_> = self
                    .run_state
                    .deck
                    .iter()
                    .enumerate()
                    .filter_map(|(index, card)| {
                        crate::cards::global_registry()
                            .can_upgrade_name(card)
                            .then_some(index)
                    })
                    .take(*count)
                    .collect();
                for index in indices {
                    self.run_state.upgrade_deck_card(index);
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
            EventReward::UniqueRelicOrCirclet { label } => {
                let relic_id = if self.run_state.relics.iter().any(|owned| owned == label) {
                    "Circlet".to_string()
                } else {
                    label.clone()
                };
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

        // CursedTome.java::randomBook considers these exact three SPECIAL
        // relics, excludes owned books, and uses Circlet only when all three
        // are already owned. They must not be filtered through ordinary relic
        // reward registry metadata.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/events/city/CursedTome.java
        let candidates: Vec<&str> = CURSED_TOME_BOOKS
            .iter()
            .copied()
            .filter(|id| !self.run_state.relics.iter().any(|owned| owned == id))
            .collect();
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
    pub(crate) fn debug_is_off_color_reward_card(card_id: &str) -> bool {
        matches!(
            event_card_color(card_id),
            Some(EventCardColor::Red | EventCardColor::Green | EventCardColor::Blue)
        )
    }

    #[cfg(test)]
    pub(crate) fn debug_prepare_winged_path_source(&mut self) -> usize {
        for y in (0..self.map.height.saturating_sub(1)).rev() {
            for x in 0..self.map.width {
                let normal = self.map.get_next_nodes(x, y);
                if normal.is_empty() {
                    continue;
                }
                let normal_coords = normal
                    .iter()
                    .map(|node| (node.x, node.y))
                    .collect::<Vec<_>>();
                let normal_len = normal.len();
                let target_y = normal[0].y;
                let has_winged_extra = self.map.get_nodes_at_floor(target_y).iter().any(|node| {
                    node.room_type != RoomType::None
                        && !normal_coords.contains(&(node.x, node.y))
                });
                if has_winged_extra {
                    self.run_state.map_x = x as i32;
                    self.run_state.map_y = y as i32;
                    self.phase = RunPhase::MapChoice;
                    self.refresh_decision_stack();
                    return normal_len;
                }
            }
        }
        panic!("generated map has no source with an off-edge next-row node");
    }

    #[cfg(test)]
    pub(crate) fn debug_enter_mystery_room(&mut self) {
        self.enter_mystery_room();
        self.refresh_decision_stack();
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

/// ApplyStasisAction rarity priority: Rare > Uncommon > Common > any.
pub(crate) fn stasis_card_rarity_rank(card_id: &str) -> u8 {
    match event_card_rarity(card_id) {
        Some(EventCardRarity::Rare) => 3,
        Some(EventCardRarity::Uncommon) => 2,
        Some(EventCardRarity::Common) => 1,
        _ => 0,
    }
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
    fn bandit_bear_stats_direct_cycle_and_ai_ticks_match_java() {
        // Source: reference/extracted/methods/monster/BanditBear.java.
        // Constructor: HP 38..42 (40..44 at A7), attacks 18/9 (20/10 at
        // A2), and Bear Hug applies -2 Dexterity (-4 at A17). getMove consumes
        // the opening roll, while each takeTurn uses SetMoveAction directly:
        // Hug -> Lunge -> Maul -> Lunge, with no further aiRng calls.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("BanditBear").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("BanditBear").0);
        }
        assert_eq!(low_hp, (38..=42).collect());
        assert_eq!(high_hp, (40..=44).collect());

        for (ascension, maul, lunge, dexterity_down) in
            [(0, 18, 9, 2), (2, 20, 10, 2), (17, 20, 10, 4)]
        {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["BanditBear".to_string()]);
            let combat = run.combat_engine.as_mut().unwrap();
            assert_eq!(combat.ai_rng.counter, 1);
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BEAR_HUG);
            assert_eq!(combat.state.enemies[0].effect(crate::combat_types::mfx::DEX_DOWN),
                Some(dexterity_down as i16));

            let hp_before = combat.state.player.hp;
            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before);
            assert_eq!(combat.state.player.status(crate::status_ids::sid::DEXTERITY),
                -dexterity_down);
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BEAR_LUNGE);
            assert_eq!(combat.state.enemies[0].move_damage(), lunge);
            assert_eq!(combat.state.enemies[0].move_block(), 9);
            assert_eq!(combat.ai_rng.counter, 1);

            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before - lunge);
            assert_eq!(combat.state.enemies[0].entity.block, 9);
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BEAR_MAUL);
            assert_eq!(combat.state.enemies[0].move_damage(), maul);
            assert_eq!(combat.ai_rng.counter, 1);

            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before - lunge - maul);
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BEAR_LUNGE);
            assert_eq!(combat.ai_rng.counter, 1);
        }
    }

    #[test]
    fn bandit_child_canonical_id_stats_and_direct_repeat_match_java() {
        // Source: reference/extracted/methods/monster/BanditPointy.java.
        // BanditChild is the canonical ID. The constructor sets 30 HP (34 at
        // A7) and a 5x2 attack (6x2 at A2). getMove consumes the opening tick;
        // takeTurn repeats the attack with SetMoveAction and consumes no RNG.
        for (ascension, hp, damage) in [(0, 30, 5), (2, 30, 6), (7, 34, 6)] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["BanditChild".to_string()]);
            let combat = run.combat_engine.as_mut().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.id, "BanditChild");
            assert_eq!(enemy.entity.hp, hp);
            assert_eq!(enemy.move_id, crate::enemies::move_ids::POINTY_STAB);
            assert_eq!(enemy.move_damage(), damage);
            assert_eq!(enemy.move_hits(), 2);
            assert_eq!(combat.ai_rng.counter, 1);

            let hp_before = combat.state.player.hp;
            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before - damage * 2);
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::POINTY_STAB);
            assert_eq!(combat.ai_rng.counter, 1);

            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before - damage * 4);
            assert_eq!(combat.ai_rng.counter, 1);
        }
    }

    #[test]
    fn bandit_leader_stats_direct_pattern_and_a17_repeat_match_java() {
        // Source: reference/extracted/methods/monster/BanditLeader.java.
        // HP is 35..39 (37..41 at A7); Cross/Agonizing Slash are 15/10
        // (17/12 at A2), and Weak is two (three at A17). Only getMove's Mock
        // opener rolls; takeTurn directly installs every later intent. A17
        // permits a second consecutive Cross Slash, but never a third.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("BanditLeader").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("BanditLeader").0);
        }
        assert_eq!(low_hp, (35..=39).collect());
        assert_eq!(high_hp, (37..=41).collect());

        for (ascension, cross, agonize, weak, after_first_cross) in [
            (0, 15, 10, 2, crate::enemies::move_ids::BANDIT_AGONIZE),
            (2, 17, 12, 2, crate::enemies::move_ids::BANDIT_AGONIZE),
            (17, 17, 12, 3, crate::enemies::move_ids::BANDIT_CROSS_SLASH),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["BanditLeader".to_string()]);
            let combat = run.combat_engine.as_mut().unwrap();
            assert_eq!(combat.ai_rng.counter, 1);
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BANDIT_MOCK);

            let hp_before = combat.state.player.hp;
            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before);
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BANDIT_AGONIZE);
            assert_eq!(combat.state.enemies[0].move_damage(), agonize);
            assert_eq!(combat.state.enemies[0].effect(crate::combat_types::mfx::WEAK),
                Some(weak as i16));
            assert_eq!(combat.ai_rng.counter, 1);

            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before - agonize);
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BANDIT_CROSS_SLASH);
            assert_eq!(combat.state.enemies[0].move_damage(), cross);
            assert_eq!(combat.ai_rng.counter, 1);

            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before - agonize - cross);
            assert_eq!(combat.state.enemies[0].move_id, after_first_cross);
            assert_eq!(combat.ai_rng.counter, 1);

            if ascension >= 17 {
                combat.execute_action(&crate::actions::Action::EndTurn);
                assert_eq!(combat.state.player.hp, hp_before - agonize - cross * 2);
                assert_eq!(combat.state.enemies[0].move_id,
                    crate::enemies::move_ids::BANDIT_AGONIZE);
                assert_eq!(combat.ai_rng.counter, 1);
            }
        }
    }

    #[test]
    fn book_of_stabbing_stats_opening_rng_and_painful_stabs_match_java() {
        // Source: reference/extracted/methods/monster/BookOfStabbing.java.
        // HP is 160..164 (168..172 at A8), attacks are 6/21 (7/24 at A3),
        // and every combat begins with Painful Stabs. AbstractMonster.rollMove
        // consumes one opening aiRng value before getMove chooses Stab/Big Stab.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("BookOfStabbing").0);
            let mut high = RunEngine::new(seed, 8);
            high_hp.insert(high.roll_enemy_hp("BookOfStabbing").0);
        }
        assert_eq!(low_hp, (160..=164).collect());
        assert_eq!(high_hp, (168..=172).collect());

        for (ascension, stab, big_stab, a18) in
            [(0, 6, 21, 0), (3, 7, 24, 0), (8, 7, 24, 0), (18, 7, 24, 1)]
        {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["BookOfStabbing".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), stab);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), big_stab);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), a18);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::PAINFUL_STABS), 1);
            assert_eq!(combat.ai_rng.counter, 1);
            match enemy.move_id {
                crate::enemies::move_ids::BOOK_STAB => {
                    assert_eq!(enemy.move_damage(), stab);
                    assert_eq!(enemy.move_hits(),
                        enemy.entity.status(crate::status_ids::sid::STAB_COUNT));
                }
                crate::enemies::move_ids::BOOK_BIG_STAB => {
                    assert_eq!(enemy.move_damage(), big_stab);
                    assert_eq!(enemy.move_hits(), 1);
                }
                other => panic!("unexpected Book opener {other}"),
            }
        }

        // PainfulStabsPower.onInflictDamage adds one Wound for each hit that
        // deals HP damage. A fully blocked first hit must not add one.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/PainfulStabsPower.java
        let mut run = RunEngine::new(42, 0);
        run.enter_specific_combat(vec!["BookOfStabbing".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::BOOK_STAB, 6, 2, 0);
        combat.state.player.block = 6;
        combat.execute_action(&crate::actions::Action::EndTurn);
        let wounds = combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Wound").count();
        assert_eq!(wounds, 1);
    }

    #[test]
    fn taskmaster_stats_wounds_strength_group_and_ticks_match_java() {
        // Sources: reference/extracted/methods/monster/Taskmaster.java and
        // decompiled/java-src/com/megacrit/cardcrawl/helpers/MonsterHelper.java
        // (`Slavers` encounter).
        let mut low_hp = std::collections::BTreeSet::new();
        let mut high_hp = std::collections::BTreeSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("SlaverBoss").0);
            let mut high = RunEngine::new(seed, 8);
            high_hp.insert(high.roll_enemy_hp("SlaverBoss").0);
        }
        assert_eq!(low_hp, (54..=60).collect());
        assert_eq!(high_hp, (57..=64).collect());

        for (ascension, hp_range, wounds, strength_each_turn) in [
            (0, 54..=60, 1, 0),
            (3, 54..=60, 2, 0),
            (8, 57..=64, 2, 0),
            (18, 57..=64, 3, 1),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["SlaverBoss".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let taskmaster = &combat.state.enemies[0];
            assert_eq!(taskmaster.id, "SlaverBoss");
            assert!(hp_range.contains(&taskmaster.entity.hp));
            assert_eq!(taskmaster.move_id,
                crate::enemies::move_ids::TASK_SCOURING_WHIP);
            assert_eq!(taskmaster.move_damage(), 7);
            assert_eq!(taskmaster.effect(crate::combat_types::mfx::WOUND),
                Some(wounds));
            assert_eq!(taskmaster.effect(crate::combat_types::mfx::STRENGTH),
                (strength_each_turn > 0).then_some(strength_each_turn));
            assert!(matches!(taskmaster.intent,
                crate::combat_types::Intent::AttackDebuff { .. }));
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut a18 = RunEngine::new(42, 18);
        a18.enter_specific_combat(vec!["SlaverBoss".to_string()]);
        let combat = a18.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.discard_pile.clear();
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 493);
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Wound").count(), 3);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::STRENGTH), 1);
        assert_eq!(combat.ai_rng.counter, ai_before + 1);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 485,
            "the second fixed-seven attack receives the first Strength stack");
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Wound").count(), 6);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::STRENGTH), 2);
        assert_eq!(combat.ai_rng.counter, ai_before + 2);

        let mut group = RunEngine::new(42, 0);
        group.enter_specific_combat(vec![
            "SlaverBlue".to_string(),
            "SlaverBoss".to_string(),
            "SlaverRed".to_string(),
        ]);
        let combat = group.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.enemies.iter().map(|enemy| enemy.id.as_str())
            .collect::<Vec<_>>(), ["SlaverBlue", "SlaverBoss", "SlaverRed"]);
        assert_eq!(combat.ai_rng.counter, 3,
            "each Slavers group member consumes its opening rollMove draw");
    }

    #[test]
    fn snake_plant_stats_ai_spores_malleable_and_ticks_match_java() {
        // Source: reference/extracted/methods/monster/SnakePlant.java.
        // HP is uniformly inclusive, Chomp changes at A2, its AI changes at
        // A17, and every opening/turn-ending rollMove consumes one aiRng tick.
        let mut low_hp = std::collections::BTreeSet::new();
        let mut high_hp = std::collections::BTreeSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("SnakePlant").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("SnakePlant").0);
        }
        assert_eq!(low_hp, (75..=79).collect());
        assert_eq!(high_hp, (78..=82).collect());

        for (ascension, damage, high_ai, hp_range) in [
            (0, 7, 0, 75..=79),
            (2, 8, 0, 75..=79),
            (7, 8, 0, 78..=82),
            (17, 8, 1, 78..=82),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["SnakePlant".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert!(hp_range.contains(&enemy.entity.hp));
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), damage);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::MALLEABLE), 3);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), 3);
            assert_eq!(enemy.entity.status(
                crate::status_ids::sid::HIGH_ASCENSION_AI), high_ai);
            assert!(matches!(enemy.move_id,
                crate::enemies::move_ids::SNAKE_CHOMP
                    | crate::enemies::move_ids::SNAKE_SPORES));
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut attack = RunEngine::new(42, 2);
        attack.enter_specific_combat(vec!["SnakePlant".to_string()]);
        let combat = attack.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::SNAKE_CHOMP, 8, 3, 0);
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 476);
        assert_eq!(combat.ai_rng.counter, ai_before + 1);

        let mut spores = RunEngine::new(42, 0);
        spores.enter_specific_combat(vec!["SnakePlant".to_string()]);
        let combat = spores.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::SNAKE_SPORES, 0, 0, 0);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::FRAIL, 2);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::WEAK, 2);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::FRAIL), 2);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::WEAKENED), 2);

        // MalleablePower.onAttacked gains basePower block only from positive,
        // nonlethal NORMAL HP damage, increments its amount, then resets the
        // amount to basePower at the end of the monster turn.
        // Source: decompiled/java-src/com/megacrit/cardcrawl/powers/MalleablePower.java.
        let mut malleable = RunEngine::new(42, 0);
        malleable.enter_specific_combat(vec!["SnakePlant".to_string()]);
        let combat = malleable.combat_engine.as_mut().unwrap();
        let hp = combat.state.enemies[0].entity.hp;
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(combat.state.enemies[0].entity.hp, hp - 1);
        assert_eq!(combat.state.enemies[0].entity.block, 3);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::MALLEABLE), 4);
        combat.deal_damage_to_enemy(0, 5);
        assert_eq!(combat.state.enemies[0].entity.hp, hp - 3);
        assert_eq!(combat.state.enemies[0].entity.block, 4);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::MALLEABLE), 5);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::MALLEABLE), 3);

        let mut lethal = RunEngine::new(42, 0);
        lethal.enter_specific_combat(vec!["SnakePlant".to_string()]);
        let combat = lethal.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].entity.hp = 1;
        combat.state.enemies[0].entity.block = 0;
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(combat.state.enemies[0].entity.block, 0,
            "a lethal hit must not trigger Malleable");
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::MALLEABLE), 3);
    }

    #[test]
    fn snecko_stats_ai_debuffs_confusion_and_ticks_match_java() {
        // Source: reference/extracted/methods/monster/Snecko.java.
        // HP is uniformly inclusive, both attacks change at A2, Tail adds
        // Weak at A17, and the forced Glare still consumes rollMove's AI draw.
        let mut low_hp = std::collections::BTreeSet::new();
        let mut high_hp = std::collections::BTreeSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("Snecko").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("Snecko").0);
        }
        assert_eq!(low_hp, (114..=120).collect());
        assert_eq!(high_hp, (120..=125).collect());

        for (ascension, bite, tail, high_ai, hp_range) in [
            (0, 15, 8, 0, 114..=120),
            (2, 18, 10, 0, 114..=120),
            (7, 18, 10, 0, 120..=125),
            (17, 18, 10, 1, 120..=125),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Snecko".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert!(hp_range.contains(&enemy.entity.hp));
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), bite);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), tail);
            assert_eq!(enemy.entity.status(
                crate::status_ids::sid::HIGH_ASCENSION_AI), high_ai);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::FIRST_MOVE), 0);
            assert_eq!(enemy.move_id, crate::enemies::move_ids::SNECKO_GLARE);
            assert_eq!(enemy.effect(crate::combat_types::mfx::CONFUSED), Some(1));
            assert_eq!(combat.ai_rng.counter, 1);
        }

        // Glare installs Confusion only when it resolves. ConfusionPower then
        // consumes cardRandomRng once for every newly drawn non-X card, leaves
        // X-cost cards alone, and randomizes each eligible card to 0..=3.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ConfusionPower.java.
        let mut glare = RunEngine::new(42, 0);
        glare.enter_specific_combat(vec!["Snecko".to_string()]);
        let combat = glare.combat_engine.as_mut().unwrap();
        combat.state.hand.clear();
        combat.state.discard_pile.clear();
        combat.state.draw_pile = [
            "ConjureBlade", "Strike", "Defend", "Vigilance", "Eruption",
        ].into_iter().map(|id| combat.card_registry.make_card(id)).collect();
        let card_random_before = combat.card_random_rng.counter;
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::CONFUSION), 1);
        assert_eq!(combat.card_random_rng.counter, card_random_before + 4);
        let x_cost = combat.state.hand.iter().find(|card|
            combat.card_registry.card_name(card.def_id) == "ConjureBlade").unwrap();
        assert_eq!(x_cost.cost, -1);
        assert!(combat.state.hand.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) != "ConjureBlade")
            .all(|card| (0..=3).contains(&card.cost)));
        assert_eq!(combat.ai_rng.counter, 2);

        // Glare uses ApplyPowerAction, so Artifact negates Confusion and is
        // consumed before the following draw can randomize any card costs.
        let mut artifact = RunEngine::new(42, 0);
        artifact.enter_specific_combat(vec!["Snecko".to_string()]);
        let combat = artifact.combat_engine.as_mut().unwrap();
        combat.state.player.set_status(crate::status_ids::sid::ARTIFACT, 1);
        let card_random_before = combat.card_random_rng.counter;
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::ARTIFACT), 0);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::CONFUSION), 0);
        assert_eq!(combat.card_random_rng.counter, card_random_before);

        let mut a17 = RunEngine::new(42, 17);
        a17.enter_specific_combat(vec!["Snecko".to_string()]);
        let combat = a17.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        crate::enemies::roll_next_move_with_num_and_rng(
            &mut combat.state.enemies[0], 0, &mut crate::seed::StsRandom::new(0));
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SNECKO_TAIL);
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 490);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::WEAKENED), 2);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::VULNERABLE), 2);

        let mut bite = RunEngine::new(42, 0);
        bite.enter_specific_combat(vec!["Snecko".to_string()]);
        let combat = bite.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        crate::enemies::roll_next_move_with_num_and_rng(
            &mut combat.state.enemies[0], 40, &mut crate::seed::StsRandom::new(0));
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SNECKO_BITE);
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 485);
    }

    #[test]
    fn spheric_guardian_stats_cycle_barricade_artifact_and_ticks_match_java() {
        // Source: reference/extracted/methods/monster/SphericGuardian.java.
        // The constructor is fixed at 20 HP. A2 raises all attacks to 11,
        // while A17 raises only Activate's block from 25 to 35.
        for (ascension, damage, activate_block) in [
            (0, 10, 25), (2, 11, 25), (17, 11, 35),
        ] {
            let mut run = RunEngine::new(42, ascension);
            assert_eq!(run.roll_enemy_hp("SphericGuardian"), (20, 20));
            run.enter_specific_combat(vec!["SphericGuardian".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!((enemy.entity.hp, enemy.entity.max_hp), (20, 20));
            assert_eq!(enemy.entity.block, 40);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BARRICADE), 1);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ARTIFACT), 3);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), damage);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), activate_block);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), 15);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::FIRST_MOVE), 0);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::FIRST_TURN), 1);
            assert_eq!(enemy.move_id,
                crate::enemies::move_ids::SPHER_INITIAL_BLOCK);
            assert_eq!(enemy.move_block(), activate_block);
            assert_eq!(combat.ai_rng.counter, 1);
        }

        // BarricadePower preserves every accumulated block value. The exact
        // source cycle is Activate -> Frail Attack -> two-hit Slam -> Harden,
        // then Slam/Harden alternation; each queued RollMoveAction costs one
        // AI RNG tick even though getMove ignores its integer.
        let mut cycle = RunEngine::new(42, 17);
        cycle.enter_specific_combat(vec!["SphericGuardian".to_string()]);
        let combat = cycle.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.enemies[0].entity.block, 75);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SPHER_FRAIL_ATTACK);
        assert_eq!(combat.ai_rng.counter, 2);

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 489);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::FRAIL), 5);
        assert_eq!(combat.state.enemies[0].entity.block, 75);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SPHER_BIG_ATTACK);
        assert_eq!(combat.ai_rng.counter, 3);

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 467);
        assert_eq!(combat.state.enemies[0].entity.block, 75);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SPHER_BLOCK_ATTACK);
        assert_eq!(combat.ai_rng.counter, 4);

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 456);
        assert_eq!(combat.state.enemies[0].entity.block, 90);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SPHER_BIG_ATTACK);
        assert_eq!(combat.ai_rng.counter, 5);

        // usePreBattleAction's three Artifact stacks negate the first three
        // debuffs without installing them, then allow the fourth through.
        let entity = &mut combat.state.enemies[0].entity;
        for remaining in [2, 1, 0] {
            assert!(!crate::powers::apply_debuff(
                entity, crate::status_ids::sid::WEAKENED, 1));
            assert_eq!(entity.status(crate::status_ids::sid::ARTIFACT), remaining);
            assert_eq!(entity.status(crate::status_ids::sid::WEAKENED), 0);
        }
        assert!(crate::powers::apply_debuff(
            entity, crate::status_ids::sid::WEAKENED, 1));
        assert_eq!(entity.status(crate::status_ids::sid::WEAKENED), 1);

        // Frail is also an ApplyPowerAction and therefore respects player Artifact.
        let mut protected = RunEngine::new(42, 0);
        protected.enter_specific_combat(vec!["SphericGuardian".to_string()]);
        let combat = protected.combat_engine.as_mut().unwrap();
        combat.execute_action(&crate::actions::Action::EndTurn);
        combat.state.player.set_status(crate::status_ids::sid::ARTIFACT, 1);
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::ARTIFACT), 0);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::FRAIL), 0);
    }

    #[test]
    fn bronze_automaton_stats_cycle_a19_recovery_and_spawn_order_match_java() {
        // Source: reference/extracted/methods/monster/BronzeAutomaton.java.
        // HP changes at A9; Flail/Beam/Strength at A4; block at A9; and only
        // A19 replaces the post-Beam Stunned turn with Boost. Artifact is 3.
        for (ascension, hp, flail, beam, strength, block, high_ai) in [
            (0, 300, 7, 45, 3, 9, 0),
            (4, 300, 8, 50, 4, 9, 0),
            (9, 320, 8, 50, 4, 12, 0),
            (19, 320, 8, 50, 4, 12, 1),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["BronzeAutomaton".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.hp, hp);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::FLAIL_DMG), flail);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BEAM_DMG), beam);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), strength);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), block);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::HIGH_ASCENSION_AI), high_ai);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ARTIFACT), 3);
            assert_eq!(enemy.move_id, crate::enemies::move_ids::BA_SPAWN_ORBS);
            assert_eq!(combat.ai_rng.counter, 1);

            let mut pattern = enemy.clone();
            let mut secondary = crate::seed::StsRandom::new(0);
            let expected = [
                crate::enemies::move_ids::BA_FLAIL,
                crate::enemies::move_ids::BA_BOOST,
                crate::enemies::move_ids::BA_FLAIL,
                crate::enemies::move_ids::BA_BOOST,
                crate::enemies::move_ids::BA_HYPER_BEAM,
                if high_ai > 0 {
                    crate::enemies::move_ids::BA_BOOST
                } else {
                    crate::enemies::move_ids::BA_STUNNED
                },
            ];
            for expected_move in expected {
                crate::enemies::roll_next_move(&mut pattern, &mut secondary);
                assert_eq!(pattern.move_id, expected_move);
            }
        }

        // SpawnMonsterAction initializes each minion before the Automaton's
        // queued RollMoveAction: opening Automaton tick + two Orb ticks + the
        // Automaton's next-move tick = four. Both Orbs receive MinionPower.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/SpawnMonsterAction.java
        let mut run = RunEngine::new(42, 19);
        run.enter_specific_combat(vec!["BronzeAutomaton".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.enemies.len(), 3);
        assert!(combat.state.enemies[1].is_minion);
        assert!(combat.state.enemies[2].is_minion);
        assert_eq!(combat.state.enemies[1].entity.status(crate::status_ids::sid::COUNT), 0);
        assert_eq!(combat.state.enemies[2].entity.status(crate::status_ids::sid::COUNT), 1);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::BA_FLAIL);
        assert_eq!(combat.ai_rng.counter, 4);
    }

    #[test]
    fn bronze_orb_hp_ai_support_and_stasis_lifecycle_match_java() {
        // Source: reference/extracted/methods/monster/BronzeOrb.java.
        // HP is 52..58 (54..60 at A9), Beam deals 8, Support gives the
        // Automaton 12 Block, and the first >=25 AI roll uses Stasis once.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("BronzeOrb").0);
            let mut high = RunEngine::new(seed, 9);
            high_hp.insert(high.roll_enemy_hp("BronzeOrb").0);
        }
        assert_eq!(low_hp, (52..=58).collect());
        assert_eq!(high_hp, (54..=60).collect());

        let deck = crate::tests::support::make_deck_n("Defend", 20);
        let automaton = crate::enemies::create_enemy("BronzeAutomaton", 300, 300);
        let orb = crate::enemies::create_enemy("BronzeOrb", 52, 52);
        let state = crate::state::CombatState::new(80, 80, vec![automaton, orb], deck, 3);
        let mut support = crate::engine::CombatEngine::new(state, 42);
        support.start_combat();
        support.state.enemies[0].set_move(
            crate::enemies::move_ids::BA_STUNNED, 0, 0, 0);
        support.state.enemies[1].set_move(
            crate::enemies::move_ids::BO_SUPPORT, 0, 0, 12);
        support.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(support.state.enemies[0].entity.block, 12);
        assert_eq!(support.state.enemies[1].entity.block, 0);

        // ApplyStasisAction chooses from draw before discard, prioritizes Rare,
        // consumes one cardRandomRng tick, and StasisPower returns the exact
        // held card to hand when the Orb dies.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ApplyStasisAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/StasisPower.java
        let mut run = RunEngine::new(42, 0);
        run.enter_specific_combat(vec!["BronzeOrb".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.state.draw_pile = ["Strike", "CutThroughFate", "ForeignInfluence", "Omniscience"]
            .into_iter()
            .map(|id| combat.card_registry.make_card(id))
            .collect();
        combat.state.discard_pile.clear();
        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::BO_STASIS, 0, 0, 0);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::STASIS, 1);
        let card_random_before = combat.card_random_rng.counter;
        combat.execute_action(&crate::actions::Action::EndTurn);
        let held = combat.state.enemies[0].stasis_card
            .expect("Stasis should hold one card");
        assert_eq!(combat.card_registry.card_name(held.def_id), "Omniscience");
        assert_eq!(combat.card_random_rng.counter, card_random_before + 1);
        assert!(!combat.state.draw_pile.iter().chain(&combat.state.discard_pile)
            .chain(&combat.state.hand)
            .any(|card| combat.card_registry.card_name(card.def_id) == "Omniscience"));

        assert!(combat.instant_kill_enemy(0));
        assert!(combat.state.hand.iter()
            .any(|card| combat.card_registry.card_name(card.def_id) == "Omniscience"));
        assert!(combat.state.enemies[0].stasis_card.is_none());
    }

    #[test]
    fn byrd_stats_rng_flight_grounding_and_fly_up_match_java() {
        // Sources: reference/extracted/methods/monster/Byrd.java and
        // decompiled/java-src/com/megacrit/cardcrawl/powers/FlightPower.java.
        // HP changes at A7, Peck/Swoop at A2, and Flight at A17. The opening
        // always consumes rollMove's integer plus randomBoolean(0.375).
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("Byrd").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("Byrd").0);
        }
        assert_eq!(low_hp, (25..=31).collect());
        assert_eq!(high_hp, (26..=33).collect());

        for (ascension, pecks, swoop, flight) in [
            (0, 5, 12, 3),
            (2, 6, 14, 3),
            (7, 6, 14, 3),
            (17, 6, 14, 4),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Byrd".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), pecks);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::SLASH_DMG), swoop);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), flight);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::FLIGHT), flight);
            assert_eq!(combat.ai_rng.counter, 2);

            let mut forced_swoop = enemy.clone();
            forced_swoop.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 0);
            crate::enemies::roll_initial_move_with_num_and_rng(
                &mut forced_swoop, 60, &mut crate::seed::StsRandom::new(0));
            assert_eq!(forced_swoop.move_id, crate::enemies::move_ids::BYRD_SWOOP);
            assert_eq!(forced_swoop.move_damage(), swoop);
        }

        // FlightPower.onAttacked receives post-block damage: a fully blocked
        // hit consumes nothing. A positive nonlethal hit consumes one stack,
        // and atStartOfTurn restores the stored amount while Flight remains.
        let mut reset = RunEngine::new(42, 0);
        reset.enter_specific_combat(vec!["Byrd".to_string()]);
        let combat = reset.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].entity.block = 10;
        combat.deal_damage_to_enemy(0, 10);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::FLIGHT), 3);
        combat.state.enemies[0].entity.block = 0;
        combat.deal_damage_to_enemy(0, 2);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::FLIGHT), 2);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::FLIGHT), 3);

        // Removing the final stack invokes changeState("GROUNDED") immediately.
        // Stunned rolls Headbutt; Headbutt installs Fly Up directly without an
        // RNG draw; Fly Up reapplies Flight and then performs one normal roll.
        let mut grounded = RunEngine::new(42, 0);
        grounded.enter_specific_combat(vec!["Byrd".to_string()]);
        let combat = grounded.combat_engine.as_mut().unwrap();
        for remaining in [2, 1, 0] {
            combat.deal_damage_to_enemy(0, 2);
            assert_eq!(combat.state.enemies[0].entity.status(
                crate::status_ids::sid::FLIGHT), remaining);
        }
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::BYRD_STUNNED);

        let before_stunned = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::BYRD_HEADBUTT);
        assert_eq!(combat.ai_rng.counter, before_stunned + 1);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::BYRD_FLY_UP);
        assert_eq!(combat.ai_rng.counter, before_stunned + 1);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::FLIGHT), 3);
        assert!(matches!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::BYRD_PECK
                | crate::enemies::move_ids::BYRD_SWOOP
                | crate::enemies::move_ids::BYRD_CAW));
        assert_eq!(combat.ai_rng.counter, before_stunned + 2);
    }

    #[test]
    fn shelled_parasite_stats_ai_vampire_plating_and_stun_match_java() {
        // Sources: reference/extracted/methods/monster/ShelledParasite.java,
        // PlatedArmorPower.java, and VampireDamageAction.java.
        let mut low_hp = std::collections::BTreeSet::new();
        let mut high_hp = std::collections::BTreeSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("Shelled Parasite").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("Shelled Parasite").0);
        }
        assert_eq!(low_hp, (68..=72).collect());
        assert_eq!(high_hp, (70..=75).collect());

        for (ascension, hp_range, double, fell, suck, high_ai, ai_ticks) in [
            (0, 68..=72, 6, 18, 10, 0, 2),
            (2, 68..=72, 7, 21, 12, 0, 2),
            (7, 70..=75, 7, 21, 12, 0, 2),
            (17, 70..=75, 7, 21, 12, 1, 1),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Shelled Parasite".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let parasite = &combat.state.enemies[0];
            assert_eq!(parasite.id, "Shelled Parasite");
            assert!(hp_range.contains(&parasite.entity.hp));
            assert_eq!(parasite.entity.status(crate::status_ids::sid::STARTING_DMG),
                double);
            assert_eq!(parasite.entity.status(crate::status_ids::sid::STR_AMT), fell);
            assert_eq!(parasite.entity.status(crate::status_ids::sid::BLOCK_AMT), suck);
            assert_eq!(parasite.entity.status(crate::status_ids::sid::HIGH_ASCENSION_AI),
                high_ai);
            assert_eq!(parasite.entity.status(crate::status_ids::sid::PLATED_ARMOR), 14);
            assert_eq!(parasite.entity.block, 14);
            assert_eq!(parasite.entity.status(crate::status_ids::sid::FIRST_MOVE), 0);
            if ascension >= 17 {
                assert_eq!(parasite.move_id, crate::enemies::move_ids::SP_FELL);
                assert_eq!(parasite.move_damage(), fell);
                assert_eq!(parasite.effect(crate::combat_types::mfx::FRAIL), Some(2));
            } else {
                assert!(matches!(parasite.move_id,
                    crate::enemies::move_ids::SP_DOUBLE_STRIKE
                        | crate::enemies::move_ids::SP_LIFE_SUCK));
            }
            assert_eq!(combat.ai_rng.counter, ai_ticks);
        }

        let mut fell_run = RunEngine::new(42, 17);
        fell_run.enter_specific_combat(vec!["Shelled Parasite".to_string()]);
        let combat = fell_run.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::SP_FELL, 21, 1, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::FRAIL, 2);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 479);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::FRAIL), 2);

        let mut vampire = RunEngine::new(42, 2);
        vampire.enter_specific_combat(vec!["Shelled Parasite".to_string()]);
        let combat = vampire.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.player.block = 5;
        combat.state.enemies[0].entity.hp = 50;
        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::SP_LIFE_SUCK, 12, 1, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::HEAL, 12);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 493);
        assert_eq!(combat.state.enemies[0].entity.hp, 57,
            "VampireDamageAction heals exactly lastDamageTaken after block");
        assert_eq!(combat.state.enemies[0].entity.block, 14,
            "Plated Armor grants its amount after the monster turn");

        let mut break_armor = RunEngine::new(42, 0);
        break_armor.enter_specific_combat(vec!["Shelled Parasite".to_string()]);
        let combat = break_armor.combat_engine.as_mut().unwrap();
        let hp_before = combat.state.enemies[0].entity.hp;
        combat.deal_damage_to_enemy(0, 5);
        assert_eq!(combat.state.enemies[0].entity.hp, hp_before);
        assert_eq!(combat.state.enemies[0].entity.block, 9);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::PLATED_ARMOR), 14,
            "fully blocked damage does not reduce Plated Armor");

        combat.state.enemies[0].entity.block = 0;
        combat.state.enemies[0].entity.set_status(crate::status_ids::sid::PLATED_ARMOR, 1);
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::PLATED_ARMOR), 0);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SP_STUNNED);
        assert!(matches!(combat.state.enemies[0].intent,
            crate::combat_types::Intent::Stun));

        let middle_seed = (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            (20..60).contains(&rng.random(99))
        }).unwrap();
        combat.ai_rng = crate::seed::StsRandom::new(middle_seed);
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 500,
            "armor break Stunned turn performs no attack");
        assert_eq!(combat.ai_rng.counter, ai_before + 1);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SP_DOUBLE_STRIKE);
        assert_eq!(combat.state.enemies[0].move_history.last().copied(),
            Some(crate::enemies::move_ids::SP_FELL),
            "Stunned takeTurn installs Fell before its RollMoveAction");
    }

    #[test]
    fn centurion_stats_party_aware_ai_and_random_protect_match_java() {
        // Source: reference/extracted/methods/monster/Centurion.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("Centurion").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("Centurion").0);
        }
        assert_eq!(low_hp, (76..=80).collect());
        assert_eq!(high_hp, (78..=83).collect());

        for (ascension, slash, fury, block) in [
            (0, 12, 6, 15),
            (2, 14, 7, 15),
            (7, 14, 7, 15),
            (17, 14, 7, 20),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Centurion".to_string(), "Healer".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), slash);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), fury);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ATTACK_COUNT), 3);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), block);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::COUNT), 2);
            assert!(matches!(enemy.move_id,
                crate::enemies::move_ids::CENT_SLASH
                    | crate::enemies::move_ids::CENT_PROTECT));
            assert_eq!(combat.ai_rng.counter, 2);
        }

        // getMove's >=65 window chooses Protect with a living ally and Fury
        // without one. Both fall back to Slash after two repeats.
        let mut paired = crate::enemies::create_enemy("Centurion", 80, 80);
        paired.entity.set_status(crate::status_ids::sid::COUNT, 2);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut paired, 65, &mut crate::seed::StsRandom::new(0));
        assert_eq!(paired.move_id, crate::enemies::move_ids::CENT_PROTECT);
        assert_eq!(paired.move_block(), 0);
        assert_eq!(paired.effect(crate::combat_types::mfx::BLOCK_RANDOM_OTHER), Some(15));

        let mut solo = crate::enemies::create_enemy("Centurion", 80, 80);
        solo.entity.set_status(crate::status_ids::sid::COUNT, 1);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut solo, 65, &mut crate::seed::StsRandom::new(0));
        assert_eq!(solo.move_id, crate::enemies::move_ids::CENT_FURY);
        assert_eq!(solo.move_damage(), 6);
        assert_eq!(solo.move_hits(), 3);

        let mut repeated_protect = crate::enemies::create_enemy("Centurion", 80, 80);
        repeated_protect.move_history.push(crate::enemies::move_ids::CENT_PROTECT);
        repeated_protect.set_move(crate::enemies::move_ids::CENT_PROTECT, 0, 0, 0);
        crate::enemies::roll_next_move_with_num(&mut repeated_protect, 99);
        assert_eq!(repeated_protect.move_id, crate::enemies::move_ids::CENT_SLASH);

        let mut repeated_slash = crate::enemies::create_enemy("Centurion", 80, 80);
        repeated_slash.move_history.push(crate::enemies::move_ids::CENT_SLASH);
        repeated_slash.set_move(crate::enemies::move_ids::CENT_SLASH, 12, 1, 0);
        crate::enemies::roll_next_move_with_num(&mut repeated_slash, 0);
        assert_eq!(repeated_slash.move_id, crate::enemies::move_ids::CENT_PROTECT);

        // GainBlockRandomMonsterAction excludes the source and consumes one
        // aiRng draw even with exactly one candidate. The following queued
        // RollMoveAction consumes the second draw.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/GainBlockRandomMonsterAction.java
        let mut centurion = crate::enemies::create_enemy("Centurion", 80, 80);
        centurion.set_move(crate::enemies::move_ids::CENT_PROTECT, 0, 0, 0);
        centurion.add_effect(crate::combat_types::mfx::BLOCK_RANDOM_OTHER, 15);
        let ally = crate::state::EnemyCombatState::new("Ally", 20, 20);
        let state = crate::state::CombatState::new(
            80, 80, vec![centurion, ally], Vec::new(), 3);
        let mut combat = crate::engine::CombatEngine::new(state, 42);
        crate::combat_hooks::do_enemy_turns(&mut combat);
        assert_eq!(combat.state.enemies[0].entity.block, 0);
        assert_eq!(combat.state.enemies[1].entity.block, 15);
        assert_eq!(combat.ai_rng.counter, 2);

        // With no eligible target the same action blocks the source and skips
        // the random-target draw; only RollMoveAction advances aiRng.
        let mut centurion = crate::enemies::create_enemy("Centurion", 80, 80);
        centurion.set_move(crate::enemies::move_ids::CENT_PROTECT, 0, 0, 0);
        centurion.add_effect(crate::combat_types::mfx::BLOCK_RANDOM_OTHER, 15);
        let state = crate::state::CombatState::new(
            80, 80, vec![centurion], Vec::new(), 3);
        let mut combat = crate::engine::CombatEngine::new(state, 42);
        crate::combat_hooks::do_enemy_turns(&mut combat);
        assert_eq!(combat.state.enemies[0].entity.block, 15);
        assert_eq!(combat.ai_rng.counter, 1);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::COUNT), 1);
    }

    #[test]
    fn healer_stats_missing_hp_ai_group_effects_and_ticks_match_java() {
        // Source: reference/extracted/methods/monster/Healer.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("Healer").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("Healer").0);
        }
        assert_eq!(low_hp, (48..=56).collect());
        assert_eq!(high_hp, (50..=58).collect());

        for (ascension, hp_range, damage, strength, heal, high_ai) in [
            (0, 48..=56, 8, 2, 16, 0),
            (2, 48..=56, 9, 3, 16, 0),
            (7, 50..=58, 9, 3, 16, 0),
            (17, 50..=58, 9, 4, 20, 1),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Centurion".to_string(), "Healer".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let healer = &combat.state.enemies[1];
            assert_eq!(healer.id, "Healer");
            assert!(hp_range.contains(&healer.entity.hp));
            assert_eq!(healer.entity.status(crate::status_ids::sid::STARTING_DMG), damage);
            assert_eq!(healer.entity.status(crate::status_ids::sid::STR_AMT), strength);
            assert_eq!(healer.entity.status(crate::status_ids::sid::BLOCK_AMT), heal);
            assert_eq!(healer.entity.status(crate::status_ids::sid::HIGH_ASCENSION_AI), high_ai);
            assert_eq!(healer.entity.status(crate::status_ids::sid::COUNT), 0);
            assert!(matches!(healer.move_id,
                crate::enemies::move_ids::MYSTIC_ATTACK
                    | crate::enemies::move_ids::MYSTIC_BUFF));
            assert_eq!(combat.ai_rng.counter, 2);
        }

        let mut heal = crate::enemies::create_enemy("Healer", 56, 56);
        heal.entity.set_status(crate::status_ids::sid::COUNT, 16);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut heal, 99, &mut crate::seed::StsRandom::new(0));
        assert_eq!(heal.move_id, crate::enemies::move_ids::MYSTIC_HEAL);
        assert_eq!(heal.effect(crate::combat_types::mfx::HEAL_ALL), Some(16));

        let mut attack = crate::enemies::create_enemy("Healer", 56, 56);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut attack, 40, &mut crate::seed::StsRandom::new(0));
        assert_eq!(attack.move_id, crate::enemies::move_ids::MYSTIC_ATTACK);
        assert_eq!(attack.effect(crate::combat_types::mfx::FRAIL), Some(2));

        let mut buff = crate::enemies::create_enemy("Healer", 56, 56);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut buff, 39, &mut crate::seed::StsRandom::new(0));
        assert_eq!(buff.move_id, crate::enemies::move_ids::MYSTIC_BUFF);
        assert_eq!(buff.effect(crate::combat_types::mfx::STRENGTH), Some(2));
        assert_eq!(buff.effect(crate::combat_types::mfx::STRENGTH_ALL_ALLIES), Some(2));

        let mut a17 = crate::enemies::create_enemy("Healer", 58, 58);
        a17.entity.set_status(crate::status_ids::sid::STARTING_DMG, 9);
        a17.entity.set_status(crate::status_ids::sid::STR_AMT, 4);
        a17.entity.set_status(crate::status_ids::sid::BLOCK_AMT, 20);
        a17.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI, 1);
        a17.entity.set_status(crate::status_ids::sid::COUNT, 20);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut a17, 39, &mut crate::seed::StsRandom::new(0));
        assert_eq!(a17.move_id, crate::enemies::move_ids::MYSTIC_BUFF,
            "A17 heals only when total missing HP is strictly above twenty");
        a17.entity.set_status(crate::status_ids::sid::COUNT, 21);
        crate::enemies::roll_next_move_with_num(&mut a17, 99);
        assert_eq!(a17.move_id, crate::enemies::move_ids::MYSTIC_HEAL);
        assert_eq!(a17.effect(crate::combat_types::mfx::HEAL_ALL), Some(20));

        let mut low_repeat = crate::enemies::create_enemy("Healer", 56, 56);
        low_repeat.set_move(crate::enemies::move_ids::MYSTIC_ATTACK, 8, 1, 0);
        crate::enemies::roll_next_move_with_num(&mut low_repeat, 99);
        assert_eq!(low_repeat.move_id, crate::enemies::move_ids::MYSTIC_ATTACK,
            "below A17 a single previous Attack does not block another");
        let mut high_repeat = low_repeat.clone();
        high_repeat.move_history.clear();
        high_repeat.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI, 1);
        crate::enemies::roll_next_move_with_num(&mut high_repeat, 99);
        assert_eq!(high_repeat.move_id, crate::enemies::move_ids::MYSTIC_BUFF,
            "A17 blocks Attack after only one previous Attack");

        let mut healing_run = RunEngine::new(42, 17);
        healing_run.enter_specific_combat(vec!["Centurion".to_string(), "Healer".to_string()]);
        let combat = healing_run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].entity.hp -= 25;
        combat.state.enemies[1].entity.hp -= 10;
        combat.state.enemies[0].move_id = -1;
        combat.state.enemies[1].entity.set_status(crate::status_ids::sid::COUNT, 35);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut combat.state.enemies[1], 99, &mut crate::seed::StsRandom::new(0));
        let hp_before = [combat.state.enemies[0].entity.hp,
            combat.state.enemies[1].entity.hp];
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.hp, hp_before[0] + 20);
        assert_eq!(combat.state.enemies[1].entity.hp,
            (hp_before[1] + 20).min(combat.state.enemies[1].entity.max_hp));
        assert_eq!(combat.ai_rng.counter, ai_before + 1);

        let mut buff_run = RunEngine::new(42, 17);
        buff_run.enter_specific_combat(vec!["Centurion".to_string(), "Healer".to_string()]);
        let combat = buff_run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].move_id = -1;
        combat.state.enemies[1].entity.set_status(crate::status_ids::sid::COUNT, 0);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut combat.state.enemies[1], 0, &mut crate::seed::StsRandom::new(0));
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::STRENGTH), 4);
        assert_eq!(combat.state.enemies[1].entity.status(crate::status_ids::sid::STRENGTH), 4);
    }

    #[test]
    fn maw_stats_rng_branches_turn_count_and_effects_match_java() {
        // Source: reference/extracted/methods/monster/Maw.java (`constructor`,
        // `getMove`, and `takeTurn`). HP is fixed even at high ascension.
        for ascension in [0, 2, 7, 17, 20] {
            let mut run = RunEngine::new(42, ascension);
            assert_eq!(run.roll_enemy_hp("Maw"), (300, 300));
        }

        for (ascension, slam, strength, terrify) in [
            (0, 25, 3, 3),
            (2, 30, 3, 3),
            (17, 30, 5, 5),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Maw".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let maw = &combat.state.enemies[0];
            assert_eq!((maw.entity.hp, maw.entity.max_hp), (300, 300));
            assert_eq!(maw.move_id, crate::enemies::move_ids::MAW_ROAR);
            assert_eq!(maw.entity.status(crate::status_ids::sid::TURN_COUNT), 2);
            assert_eq!(maw.entity.status(crate::status_ids::sid::STARTING_DMG), slam);
            assert_eq!(maw.entity.status(crate::status_ids::sid::STR_AMT), strength);
            assert_eq!(maw.entity.status(crate::status_ids::sid::BLOCK_AMT), terrify);
            assert_eq!(maw.effect(crate::combat_types::mfx::WEAK), Some(terrify as i16));
            assert_eq!(maw.effect(crate::combat_types::mfx::FRAIL), Some(terrify as i16));
            assert_eq!(combat.ai_rng.counter, 1,
                "AbstractMonster.init opening roll consumes one aiRng value");
        }

        let mut after_roar = crate::enemies::create_enemy("Maw", 300, 300);
        after_roar.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 1);
        after_roar.entity.set_status(crate::status_ids::sid::TURN_COUNT, 2);
        crate::enemies::roll_next_move_with_num(&mut after_roar, 49);
        assert_eq!(after_roar.move_id, crate::enemies::move_ids::MAW_NOM);
        assert_eq!((after_roar.move_damage(), after_roar.move_hits()), (5, 1));
        assert_eq!(after_roar.entity.status(crate::status_ids::sid::TURN_COUNT), 3);

        let mut high_after_roar = crate::enemies::create_enemy("Maw", 300, 300);
        high_after_roar.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 1);
        high_after_roar.entity.set_status(crate::status_ids::sid::TURN_COUNT, 2);
        high_after_roar.entity.set_status(crate::status_ids::sid::STARTING_DMG, 30);
        crate::enemies::roll_next_move_with_num(&mut high_after_roar, 50);
        assert_eq!(high_after_roar.move_id, crate::enemies::move_ids::MAW_SLAM);
        assert_eq!(high_after_roar.move_damage(), 30);

        let mut slam_low = crate::enemies::create_enemy("Maw", 300, 300);
        slam_low.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 1);
        slam_low.entity.set_status(crate::status_ids::sid::TURN_COUNT, 3);
        slam_low.set_move(crate::enemies::move_ids::MAW_SLAM, 25, 1, 0);
        crate::enemies::roll_next_move_with_num(&mut slam_low, 49);
        assert_eq!(slam_low.move_id, crate::enemies::move_ids::MAW_NOM);
        assert_eq!((slam_low.move_damage(), slam_low.move_hits()), (5, 2));

        let mut slam_high = crate::enemies::create_enemy("Maw", 300, 300);
        slam_high.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 1);
        slam_high.entity.set_status(crate::status_ids::sid::TURN_COUNT, 3);
        slam_high.set_move(crate::enemies::move_ids::MAW_SLAM, 25, 1, 0);
        crate::enemies::roll_next_move_with_num(&mut slam_high, 50);
        assert_eq!(slam_high.move_id, crate::enemies::move_ids::MAW_DROOL);
        assert_eq!(slam_high.effect(crate::combat_types::mfx::STRENGTH), Some(3));

        let mut a17_run = RunEngine::new(42, 17);
        a17_run.enter_specific_combat(vec!["Maw".to_string()]);
        let combat = a17_run.combat_engine.as_mut().unwrap();
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::WEAKENED), 5);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::FRAIL), 5);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::FIRST_MOVE), 1);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::TURN_COUNT), 3);
        assert_eq!(combat.ai_rng.counter, ai_before + 1);

        combat.state.enemies[0].set_move(crate::enemies::move_ids::MAW_DROOL, 0, 0, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::STRENGTH, 5);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::STRENGTH), 5);
        assert_eq!(combat.ai_rng.counter, ai_before + 2);
    }

    #[test]
    fn nemesis_stats_full_ai_burns_and_rng_ticks_match_java() {
        // Source: reference/extracted/methods/monster/Nemesis.java
        // (`constructor`, `getMove`, and `takeTurn`).
        for (ascension, hp, fire, burns) in [
            (0, 185, 6, 3),
            (3, 185, 7, 3),
            (7, 185, 7, 3),
            (8, 200, 7, 3),
            (18, 200, 7, 5),
        ] {
            let mut run = RunEngine::new(42, ascension);
            assert_eq!(run.roll_enemy_hp("Nemesis"), (hp, hp));
            run.enter_specific_combat(vec!["Nemesis".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let nemesis = &combat.state.enemies[0];
            assert_eq!((nemesis.entity.hp, nemesis.entity.max_hp), (hp, hp));
            assert_eq!(nemesis.entity.status(crate::status_ids::sid::STARTING_DMG), fire);
            assert_eq!(nemesis.entity.status(crate::status_ids::sid::BLOCK_AMT), burns);
            assert_eq!(nemesis.entity.status(crate::status_ids::sid::FIRST_MOVE), 0);
            assert_eq!(nemesis.entity.status(crate::status_ids::sid::SCYTHE_COOLDOWN), -1);
            match nemesis.move_id {
                crate::enemies::move_ids::NEM_TRI_ATTACK => {
                    assert_eq!((nemesis.move_damage(), nemesis.move_hits()), (fire, 3));
                }
                crate::enemies::move_ids::NEM_BURN => {
                    assert_eq!(nemesis.effect(crate::combat_types::mfx::BURN), Some(burns as i16));
                }
                other => panic!("invalid source opener {other}"),
            }
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut tri = crate::enemies::create_enemy("Nemesis", 200, 200);
        tri.entity.set_status(crate::status_ids::sid::STARTING_DMG, 7);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut tri, 49, &mut crate::seed::StsRandom::new(0));
        assert_eq!(tri.move_id, crate::enemies::move_ids::NEM_TRI_ATTACK);
        assert_eq!((tri.move_damage(), tri.move_hits()), (7, 3));
        assert_eq!(tri.entity.status(crate::status_ids::sid::SCYTHE_COOLDOWN), -1);

        let mut burn = crate::enemies::create_enemy("Nemesis", 200, 200);
        burn.entity.set_status(crate::status_ids::sid::BLOCK_AMT, 5);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut burn, 50, &mut crate::seed::StsRandom::new(0));
        assert_eq!(burn.move_id, crate::enemies::move_ids::NEM_BURN);
        assert_eq!(burn.effect(crate::combat_types::mfx::BURN), Some(5));

        crate::enemies::roll_next_move_with_num(&mut tri, 29);
        assert_eq!(tri.move_id, crate::enemies::move_ids::NEM_SCYTHE);
        assert_eq!(tri.move_damage(), 45);
        assert_eq!(tri.entity.status(crate::status_ids::sid::SCYTHE_COOLDOWN), 2);

        let no_boolean_seed = (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            (30..65).contains(&rng.random(99))
        }).unwrap();
        let mut run = RunEngine::new(42, 18);
        run.enter_specific_combat(vec!["Nemesis".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].set_move(crate::enemies::move_ids::NEM_BURN, 0, 0, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::BURN, 5);
        combat.ai_rng = crate::seed::StsRandom::new(no_boolean_seed);
        run.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = run.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Burn").count(), 5);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::INTANGIBLE), 1);
        assert_eq!(combat.ai_rng.counter, 1,
            "the mid window without two Tri Attacks consumes only RollMoveAction");
    }

    #[test]
    fn orb_walker_stats_ai_burn_zones_strength_and_ticks_match_java() {
        // Sources: reference/extracted/methods/monster/OrbWalker.java and
        // decompiled GenericStrengthUpPower.java plus
        // MakeTempCardInDiscardAndDeckAction.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=512 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("Orb Walker").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("Orb Walker").0);
        }
        assert_eq!(low_hp, (90..=96).collect());
        assert_eq!(high_hp, (92..=102).collect());

        for (ascension, hp_range, laser, claw, growth) in [
            (0, 90..=96, 10, 15, 3),
            (2, 90..=96, 11, 16, 3),
            (7, 92..=102, 11, 16, 3),
            (17, 92..=102, 11, 16, 5),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Orb Walker".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let walker = &combat.state.enemies[0];
            assert_eq!(walker.id, "Orb Walker");
            assert!(hp_range.contains(&walker.entity.hp));
            assert_eq!(walker.entity.status(crate::status_ids::sid::STARTING_DMG), laser);
            assert_eq!(walker.entity.status(crate::status_ids::sid::STR_AMT), claw);
            assert_eq!(walker.entity.status(crate::status_ids::sid::GENERIC_STRENGTH_UP), growth);
            match walker.move_id {
                crate::enemies::move_ids::OW_LASER => {
                    assert_eq!(walker.move_damage(), laser);
                    assert_eq!(walker.effect(crate::combat_types::mfx::BURN_DRAW_DISCARD), Some(1));
                    assert!(matches!(walker.intent,
                        crate::combat_types::Intent::AttackDebuff { .. }));
                }
                crate::enemies::move_ids::OW_CLAW => assert_eq!(walker.move_damage(), claw),
                other => panic!("invalid Orb Walker opener {other}"),
            }
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut low = crate::enemies::create_enemy("Orb Walker", 96, 96);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut low, 39, &mut crate::seed::StsRandom::new(0));
        assert_eq!(low.move_id, crate::enemies::move_ids::OW_CLAW);
        crate::enemies::roll_next_move_with_num(&mut low, 39);
        assert_eq!(low.move_id, crate::enemies::move_ids::OW_CLAW);
        crate::enemies::roll_next_move_with_num(&mut low, 39);
        assert_eq!(low.move_id, crate::enemies::move_ids::OW_LASER);

        let mut high = crate::enemies::create_enemy("Orb Walker", 96, 96);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut high, 40, &mut crate::seed::StsRandom::new(0));
        assert_eq!(high.move_id, crate::enemies::move_ids::OW_LASER);
        crate::enemies::roll_next_move_with_num(&mut high, 40);
        assert_eq!(high.move_id, crate::enemies::move_ids::OW_LASER);
        crate::enemies::roll_next_move_with_num(&mut high, 40);
        assert_eq!(high.move_id, crate::enemies::move_ids::OW_CLAW);

        let mut run = RunEngine::new(42, 17);
        run.enter_specific_combat(vec!["Orb Walker".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].set_move(crate::enemies::move_ids::OW_LASER, 11, 1, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(
            crate::combat_types::mfx::BURN_DRAW_DISCARD, 1);
        let filler = combat.card_registry.make_card("Strike");
        combat.state.draw_pile.push(filler);
        let ai_before = combat.ai_rng.counter;
        let card_random_before = combat.card_random_rng.counter;
        run.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = run.combat_engine.as_ref().unwrap();
        let draw_burns = combat.state.draw_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Burn").count();
        let hand_burns = combat.state.hand.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Burn").count();
        let discard_burns = combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Burn").count();
        assert_eq!((draw_burns + hand_burns, discard_burns), (1, 1));
        assert_eq!(combat.card_random_rng.counter, card_random_before + 1);
        assert_eq!(combat.ai_rng.counter, ai_before + 1);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::STRENGTH), 5);

        let player_hp = combat.state.player.hp;
        let combat = run.combat_engine.as_mut().unwrap();
        let burn_id = combat.card_registry.make_card("Burn").def_id;
        combat.state.hand.retain(|card| card.def_id != burn_id);
        combat.state.enemies[0].set_move(crate::enemies::move_ids::OW_CLAW, 16, 1, 0);
        combat.state.enemies[0].move_effects.clear();
        run.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = run.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.player.hp, player_hp - 21,
            "Claw uses the five Strength gained after the previous round");
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::STRENGTH), 10);

        let mut pair = RunEngine::new(42, 17);
        pair.enter_specific_combat(vec!["Orb Walker".to_string(), "Orb Walker".to_string()]);
        let combat = pair.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.enemies.len(), 2);
        assert!(combat.state.enemies.iter().all(|enemy| enemy.id == "Orb Walker"));
        assert_eq!(combat.ai_rng.counter, 2,
            "each event Orb Walker consumes its own opening roll");
    }

    #[test]
    fn spiker_stats_ai_thorns_execution_retaliation_and_shapes_match_java() {
        // Source: reference/extracted/methods/monster/Spiker.java.
        // HP is uniformly inclusive; A2 raises damage and starting Thorns;
        // A17 adds three more starting Thorns.
        let mut low_hp = std::collections::BTreeSet::new();
        let mut high_hp = std::collections::BTreeSet::new();
        for seed in 1..=512 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("Spiker").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("Spiker").0);
        }
        assert_eq!(low_hp, (42..=56).collect());
        assert_eq!(high_hp, (44..=60).collect());

        for (ascension, damage, thorns, hp_range) in [
            (0, 7, 3, 42..=56),
            (2, 9, 4, 42..=56),
            (7, 9, 4, 44..=60),
            (17, 9, 7, 44..=60),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Spiker".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert!(hp_range.contains(&enemy.entity.hp));
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), damage);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::THORNS), thorns);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::COUNT), 0);
            match enemy.move_id {
                crate::enemies::move_ids::SPIKER_ATTACK => {
                    assert_eq!(enemy.move_damage(), damage);
                }
                crate::enemies::move_ids::SPIKER_BUFF => {
                    assert_eq!(enemy.effect(crate::combat_types::mfx::THORNS), Some(2));
                }
                other => panic!("invalid Spiker opener {other}"),
            }
            assert_eq!(combat.ai_rng.counter, 1);
        }

        // takeTurn increments thornsCount and applies +2 Thorns before its
        // queued RollMoveAction. After six executed buffs, getMove always
        // attacks regardless of the roll or previous intent.
        let mut buffs = RunEngine::new(42, 0);
        buffs.enter_specific_combat(vec!["Spiker".to_string()]);
        let combat = buffs.combat_engine.as_mut().unwrap();
        let ai_before = combat.ai_rng.counter;
        for executed in 1..=6 {
            combat.state.enemies[0].move_effects.clear();
            combat.state.enemies[0].set_move(
                crate::enemies::move_ids::SPIKER_BUFF, 0, 0, 0);
            combat.state.enemies[0].add_effect(crate::combat_types::mfx::THORNS, 2);
            crate::combat_hooks::do_enemy_turns(combat);
            assert_eq!(combat.state.enemies[0].entity.status(
                crate::status_ids::sid::COUNT), executed);
            assert_eq!(combat.state.enemies[0].entity.status(
                crate::status_ids::sid::THORNS), 3 + executed * 2);
        }
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SPIKER_ATTACK);
        assert_eq!(combat.ai_rng.counter, ai_before + 6);

        // ThornsPower.onAttacked retaliates once per sourced NORMAL attack
        // hit even when the attack is zero, fully blocked, or lethal. Its
        // THORNS DamageInfo is blockable and does not trigger from non-attacks.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ThornsPower.java.
        let mut retaliation = RunEngine::new(42, 17);
        retaliation.enter_specific_combat(vec!["Spiker".to_string()]);
        let combat = retaliation.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 100;
        combat.state.player.max_hp = 100;
        combat.deal_player_attack_hit_to_enemy(0, 0);
        assert_eq!(combat.state.player.hp, 93);
        combat.state.player.block = 7;
        combat.deal_player_attack_hit_to_enemy(0, 1);
        assert_eq!(combat.state.player.hp, 93);
        assert_eq!(combat.state.player.block, 0);
        let hp_before = combat.state.player.hp;
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(combat.state.player.hp, hp_before,
            "non-attack damage must not invoke ThornsPower.onAttacked");

        let mut lethal = RunEngine::new(42, 17);
        lethal.enter_specific_combat(vec!["Spiker".to_string()]);
        let combat = lethal.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 100;
        combat.state.enemies[0].entity.hp = 1;
        combat.deal_player_attack_hit_to_enemy(0, 1);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_eq!(combat.state.player.hp, 93,
            "lethal attack still resolves the queued Thorns hit");

        // TheBeyond and MonsterHelper make Spiker reachable only through the
        // three/four-Shape groups, drawing without replacement from two of
        // each Shape. Run-level RNG is checked semantically per AGENTS.md.
        let mut saw_spiker = false;
        for seed in 1..=64 {
            let mut group = RunEngine::new(seed, 0);
            group.enter_specific_combat(vec!["3 Shapes".to_string()]);
            let combat = group.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies.len(), 3);
            for id in ["Spiker", "Repulsor", "Exploder"] {
                let count = combat.state.enemies.iter()
                    .filter(|enemy| enemy.id == id).count();
                assert!(count <= 2);
                saw_spiker |= id == "Spiker" && count > 0;
            }
            assert!(combat.state.enemies.iter().all(|enemy|
                matches!(enemy.id.as_str(), "Spiker" | "Repulsor" | "Exploder")));
        }
        assert!(saw_spiker, "canonical Shapes groups must make Spiker reachable");
    }

    #[test]
    fn repulsor_stats_ai_draw_dazes_and_rng_ticks_match_java() {
        // Sources: reference/extracted/methods/monster/Repulsor.java,
        // decompiled/java-src/com/megacrit/cardcrawl/actions/common/
        // MakeTempCardInDrawPileAction.java, and CardGroup.java.
        let mut low_hp = std::collections::BTreeSet::new();
        let mut high_hp = std::collections::BTreeSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("Repulsor").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("Repulsor").0);
        }
        assert_eq!(low_hp, (29..=35).collect());
        assert_eq!(high_hp, (31..=38).collect());

        for (ascension, hp_range, damage) in [
            (0, 29..=35, 11),
            (2, 29..=35, 13),
            (7, 31..=38, 13),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Repulsor".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let repulsor = &combat.state.enemies[0];
            assert!(hp_range.contains(&repulsor.entity.hp));
            assert_eq!(repulsor.entity.status(crate::status_ids::sid::STARTING_DMG),
                damage);
            match repulsor.move_id {
                crate::enemies::move_ids::REPULSOR_ATTACK => {
                    assert_eq!(repulsor.move_damage(), damage);
                }
                crate::enemies::move_ids::REPULSOR_DAZE => {
                    assert_eq!(repulsor.effect(crate::combat_types::mfx::DAZE_DRAW),
                        Some(2));
                    assert!(matches!(repulsor.intent,
                        crate::combat_types::Intent::Debuff { .. }));
                }
                other => panic!("invalid Repulsor opener {other}"),
            }
            assert_eq!(combat.ai_rng.counter, 1,
                "AbstractMonster.init consumes one opening rollMove draw");
        }

        let mut daze_run = RunEngine::new(42, 0);
        daze_run.enter_specific_combat(vec!["Repulsor".to_string()]);
        let combat = daze_run.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.draw_pile.clear();
        combat.state.discard_pile.clear();
        for _ in 0..3 {
            combat.state.draw_pile.push(combat.card_registry.make_card("Strike"));
        }
        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::REPULSOR_DAZE, 0, 0, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::DAZE_DRAW, 2);
        combat.state.enemies[0].intent = crate::combat_types::Intent::Debuff {
            effects: crate::combat_types::fx::DAZE,
        };
        let ai_before = combat.ai_rng.counter;
        let card_random_before = combat.card_random_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.draw_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Dazed").count(), 2);
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Dazed").count(), 0);
        assert_eq!(combat.card_random_rng.counter, card_random_before + 2,
            "each random draw-pile insertion consumes cardRandomRng");
        assert_eq!(combat.ai_rng.counter, ai_before + 1,
            "takeTurn always queues one RollMoveAction");

        let mut attack_run = RunEngine::new(42, 2);
        attack_run.enter_specific_combat(vec!["Repulsor".to_string()]);
        let combat = attack_run.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::REPULSOR_ATTACK, 13, 1, 0);
        combat.state.enemies[0].move_effects.clear();
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 487);
        assert_eq!(combat.ai_rng.counter, ai_before + 1);
    }

    #[test]
    fn serpent_stats_ai_constricted_artifact_and_damage_type_match_java() {
        // Sources: reference/extracted/methods/monster/SpireGrowth.java and
        // decompiled/java-src/com/megacrit/cardcrawl/powers/ConstrictedPower.java.
        for (ascension, hp, tackle, smash, constrict, high_ai) in [
            (0, 170, 16, 22, 10, 0),
            (2, 170, 18, 25, 10, 0),
            (7, 190, 18, 25, 10, 0),
            (17, 190, 18, 25, 12, 1),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Serpent".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let serpent = &combat.state.enemies[0];
            assert_eq!(serpent.id, "Serpent");
            assert_eq!((serpent.entity.hp, serpent.entity.max_hp), (hp, hp));
            assert_eq!(serpent.entity.status(crate::status_ids::sid::STARTING_DMG),
                tackle);
            assert_eq!(serpent.entity.status(crate::status_ids::sid::STR_AMT), smash);
            assert_eq!(serpent.entity.status(crate::status_ids::sid::BLOCK_AMT),
                constrict);
            assert_eq!(serpent.entity.status(crate::status_ids::sid::HIGH_ASCENSION_AI),
                high_ai);
            if ascension >= 17 {
                assert_eq!(serpent.move_id, crate::enemies::move_ids::SG_CONSTRICT);
                assert_eq!(serpent.effect(crate::combat_types::mfx::CONSTRICT),
                    Some(12));
            } else {
                assert!(matches!(serpent.move_id,
                    crate::enemies::move_ids::SG_QUICK_TACKLE
                        | crate::enemies::move_ids::SG_CONSTRICT));
            }
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut blocked = RunEngine::new(42, 17);
        blocked.enter_specific_combat(vec!["Serpent".to_string()]);
        let combat = blocked.combat_engine.as_mut().unwrap();
        combat.state.player.set_status(crate::status_ids::sid::ARTIFACT, 1);
        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::SG_CONSTRICT, 0, 0, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::CONSTRICT, 12);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::ARTIFACT), 0);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::CONSTRICTED), 0,
            "ApplyPowerAction is blocked by Artifact");

        let mut applied = RunEngine::new(42, 0);
        applied.enter_specific_combat(vec!["Serpent".to_string()]);
        let combat = applied.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::SG_CONSTRICT, 0, 0, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::CONSTRICT, 10);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::CONSTRICTED), 10);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::COUNT), 1,
            "the next getMove observes the installed Constricted power");

        // ConstrictedPower deals DamageInfo.THORNS at player end of turn: five
        // block absorbs five of twelve, then Tungsten Rod removes one more.
        let mut damage = RunEngine::new(42, 17);
        damage.enter_specific_combat(vec!["Serpent".to_string()]);
        let combat = damage.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.player.block = 5;
        combat.state.player.set_status(crate::status_ids::sid::CONSTRICTED, 12);
        combat.state.relics.push("Tungsten Rod".to_string());
        combat.state.enemies[0].move_id = -1;
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 494);
        assert_eq!(combat.state.player.block, 0);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::CONSTRICTED), 12);
    }

    #[test]
    fn champ_rng_forge_delayed_anger_and_execute_cadence_match_java() {
        // Source: reference/extracted/methods/monster/Champ.java (`getMove`,
        // `takeTurn`, and constructor).
        let mut defensive = crate::enemies::create_enemy("Champ", 420, 420);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut defensive, 15, &mut crate::seed::StsRandom::new(0));
        assert_eq!(defensive.move_id, crate::enemies::move_ids::CHAMP_DEFENSIVE);
        assert_eq!(defensive.move_block(), 15);
        assert_eq!(defensive.effect(crate::combat_types::mfx::METALLICIZE), Some(5));
        assert_eq!(defensive.entity.status(crate::status_ids::sid::FORGE_TIMES), 1);

        let mut gloat = crate::enemies::create_enemy("Champ", 420, 420);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut gloat, 16, &mut crate::seed::StsRandom::new(0));
        assert_eq!(gloat.move_id, crate::enemies::move_ids::CHAMP_GLOAT);
        assert_eq!(gloat.effect(crate::combat_types::mfx::STRENGTH), Some(2));

        let mut slap = crate::enemies::create_enemy("Champ", 420, 420);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut slap, 31, &mut crate::seed::StsRandom::new(0));
        assert_eq!(slap.move_id, crate::enemies::move_ids::CHAMP_FACE_SLAP);

        let mut slash = crate::enemies::create_enemy("Champ", 420, 420);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut slash, 56, &mut crate::seed::StsRandom::new(0));
        assert_eq!(slash.move_id, crate::enemies::move_ids::CHAMP_HEAVY_SLASH);

        // A19 widens only the Defensive Stance window from <=15 to <=30.
        let mut a19 = crate::enemies::create_enemy("Champ", 440, 440);
        a19.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI, 1);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut a19, 30, &mut crate::seed::StsRandom::new(0));
        assert_eq!(a19.move_id, crate::enemies::move_ids::CHAMP_DEFENSIVE);

        // Forge is capped at two uses and cannot repeat immediately.
        crate::enemies::roll_next_move_with_num(&mut defensive, 0);
        assert_ne!(defensive.move_id, crate::enemies::move_ids::CHAMP_DEFENSIVE);
        crate::enemies::roll_next_move_with_num(&mut defensive, 0);
        assert_eq!(defensive.move_id, crate::enemies::move_ids::CHAMP_DEFENSIVE);
        assert_eq!(defensive.entity.status(crate::status_ids::sid::FORGE_TIMES), 2);
        crate::enemies::roll_next_move_with_num(&mut defensive, 0);
        crate::enemies::roll_next_move_with_num(&mut defensive, 0);
        assert_ne!(defensive.move_id, crate::enemies::move_ids::CHAMP_DEFENSIVE);
        assert_eq!(defensive.entity.status(crate::status_ids::sid::FORGE_TIMES), 2);

        // Exactly half HP does not trigger the threshold.
        let mut half = crate::enemies::create_enemy("Champ", 210, 420);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut half, 99, &mut crate::seed::StsRandom::new(0));
        assert_ne!(half.move_id, crate::enemies::move_ids::CHAMP_ANGER);

        // Damage below half does not change the current intent or consume RNG.
        // The current action resolves first; its queued RollMoveAction selects
        // Anger. Anger then cleans debuffs/Shackled before adding 3*Strength.
        let mut run = RunEngine::new(42, 0);
        run.enter_specific_combat(vec!["TheChamp".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::CHAMP_FACE_SLAP, 12, 1, 0);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::FRAIL, 2);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::VULNERABLE, 2);
        let ai_before_damage = combat.ai_rng.counter;
        combat.deal_damage_to_enemy(0, 211);
        assert_eq!(combat.state.enemies[0].entity.hp, 209);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::CHAMP_FACE_SLAP);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::THRESHOLD_REACHED), 0);
        assert_eq!(combat.ai_rng.counter, ai_before_damage);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::CHAMP_ANGER);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::THRESHOLD_REACHED), 1);
        assert_eq!(combat.ai_rng.counter, ai_before_damage + 1);

        combat.state.enemies[0].entity.set_status(crate::status_ids::sid::STRENGTH, -4);
        combat.state.enemies[0].entity.set_status(
            crate::status_ids::sid::TEMP_STRENGTH_LOSS, 4);
        combat.state.enemies[0].entity.set_status(crate::status_ids::sid::WEAKENED, 2);
        combat.state.enemies[0].entity.set_status(crate::status_ids::sid::POISON, 2);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::STRENGTH), 6);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::TEMP_STRENGTH_LOSS), 0);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::WEAKENED), 0);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::POISON), 0);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::CHAMP_EXECUTE);

        // Execute is selected only when neither of the prior two moves was
        // Execute: one Execute, two normal intents, then Execute again.
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_ne!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::CHAMP_EXECUTE);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_ne!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::CHAMP_EXECUTE);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::CHAMP_EXECUTE);

        // Defensive Stance grants its immediate block and installs
        // Metallicize, which also fires at the end of the same enemy round.
        let mut forge_run = RunEngine::new(42, 0);
        forge_run.enter_specific_combat(vec!["TheChamp".to_string()]);
        let combat = forge_run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::CHAMP_DEFENSIVE, 0, 0, 15);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::METALLICIZE, 5);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::METALLICIZE), 5);
        assert_eq!(combat.state.enemies[0].entity.block, 20);
    }

    #[test]
    fn chosen_stats_a17_opener_rng_table_and_move_effects_match_java() {
        // Source: reference/extracted/methods/monster/Chosen.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("Chosen").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("Chosen").0);
        }
        assert_eq!(low_hp, (95..=99).collect());
        assert_eq!(high_hp, (98..=103).collect());

        for (ascension, zap, debilitate, poke, opener) in [
            (0, 18, 10, 5, crate::enemies::move_ids::CHOSEN_POKE),
            (2, 21, 12, 6, crate::enemies::move_ids::CHOSEN_POKE),
            (7, 21, 12, 6, crate::enemies::move_ids::CHOSEN_POKE),
            (17, 21, 12, 6, crate::enemies::move_ids::CHOSEN_HEX),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Chosen".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), zap);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), debilitate);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::SLAP_DMG), poke);
            assert_eq!(enemy.move_id, opener);
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut normal = crate::enemies::create_enemy("Chosen", 95, 95);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut normal, 99, &mut crate::seed::StsRandom::new(0));
        assert_eq!(normal.move_id, crate::enemies::move_ids::CHOSEN_POKE);
        crate::enemies::roll_next_move_with_num(&mut normal, 99);
        assert_eq!(normal.move_id, crate::enemies::move_ids::CHOSEN_HEX);
        crate::enemies::roll_next_move_with_num(&mut normal, 49);
        assert_eq!(normal.move_id, crate::enemies::move_ids::CHOSEN_DEBILITATE);
        assert_eq!(normal.effect(crate::combat_types::mfx::VULNERABLE), Some(2));
        crate::enemies::roll_next_move_with_num(&mut normal, 39);
        assert_eq!(normal.move_id, crate::enemies::move_ids::CHOSEN_ZAP);

        let mut drain = crate::enemies::create_enemy("Chosen", 95, 95);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut drain, 99, &mut crate::seed::StsRandom::new(0));
        crate::enemies::roll_next_move_with_num(&mut drain, 99);
        crate::enemies::roll_next_move_with_num(&mut drain, 50);
        assert_eq!(drain.move_id, crate::enemies::move_ids::CHOSEN_DRAIN);
        assert_eq!(drain.effect(crate::combat_types::mfx::WEAK), Some(3));
        assert_eq!(drain.effect(crate::combat_types::mfx::STRENGTH), Some(3));
        crate::enemies::roll_next_move_with_num(&mut drain, 40);
        assert_eq!(drain.move_id, crate::enemies::move_ids::CHOSEN_POKE);

        let mut high_ai = crate::enemies::create_enemy("Chosen", 98, 98);
        high_ai.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI, 1);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut high_ai, 0, &mut crate::seed::StsRandom::new(0));
        assert_eq!(high_ai.move_id, crate::enemies::move_ids::CHOSEN_HEX);
        crate::enemies::roll_next_move_with_num(&mut high_ai, 50);
        assert_eq!(high_ai.move_id, crate::enemies::move_ids::CHOSEN_DRAIN);

        // takeTurn effects: Hex applies Hex 1; Drain applies Weak 3 and gives
        // Chosen Strength 3; Debilitate attacks then applies Vulnerable 2.
        let mut run = RunEngine::new(42, 0);
        run.enter_specific_combat(vec!["Chosen".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::CHOSEN_HEX, 0, 0, 0);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::HEX, 1);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::HEX), 1);

        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::CHOSEN_DRAIN, 0, 0, 0);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::WEAK, 3);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::STRENGTH, 3);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::WEAKENED), 3);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::STRENGTH), 3);

        combat.state.enemies[0].set_move(
            crate::enemies::move_ids::CHOSEN_DEBILITATE, 10, 1, 0);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::VULNERABLE, 2);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::VULNERABLE), 2);
    }

    #[test]
    fn corrupt_heart_stats_rng_debilitate_and_buff_cycle_match_java() {
        // Source: reference/extracted/methods/monster/CorruptHeart.java.
        for (ascension, hp, echo, blood_hits, invincible, beat) in [
            (0, 750, 40, 12, 300, 1),
            (4, 750, 45, 15, 300, 1),
            (9, 800, 45, 15, 300, 1),
            (19, 800, 45, 15, 200, 2),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["CorruptHeart".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!((enemy.entity.hp, enemy.entity.max_hp), (hp, hp));
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ECHO_DMG), echo);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOOD_HIT_COUNT),
                blood_hits);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::INVINCIBLE), invincible);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BEAT_OF_DEATH), beat);
            assert_eq!(enemy.move_id, crate::enemies::move_ids::HEART_DEBILITATE);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::MOVE_COUNT), 0);
            assert_eq!(combat.ai_rng.counter, 1,
                "AbstractMonster.rollMove consumes the opening num");
        }

        let seed_for = |expected: bool| (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            rng.random_boolean() == expected
        }).unwrap();
        for (boolean, first_attack) in [
            (true, crate::enemies::move_ids::HEART_BLOOD_SHOTS),
            (false, crate::enemies::move_ids::HEART_ECHO),
        ] {
            let mut enemy = crate::enemies::create_enemy("CorruptHeart", 750, 750);
            let mut rng = crate::seed::StsRandom::new(seed_for(boolean));
            crate::enemies::roll_initial_move_with_num_and_rng(&mut enemy, 37, &mut rng);
            assert_eq!(enemy.move_id, crate::enemies::move_ids::HEART_DEBILITATE);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::MOVE_COUNT), 0);
            assert_eq!(rng.counter, 0);

            crate::enemies::roll_next_move_with_num_and_rng(&mut enemy, 37, &mut rng);
            assert_eq!(enemy.move_id, first_attack);
            assert_eq!(rng.counter, 1,
                "cycle slot zero consumes CorruptHeart.getMove randomBoolean");
            crate::enemies::roll_next_move_with_num_and_rng(&mut enemy, 37, &mut rng);
            assert_eq!(enemy.move_id, if first_attack ==
                crate::enemies::move_ids::HEART_ECHO {
                    crate::enemies::move_ids::HEART_BLOOD_SHOTS
                } else {
                    crate::enemies::move_ids::HEART_ECHO
                });
            assert_eq!(rng.counter, 1);
            crate::enemies::roll_next_move_with_num_and_rng(&mut enemy, 37, &mut rng);
            assert_eq!(enemy.move_id, crate::enemies::move_ids::HEART_BUFF);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::MOVE_COUNT), 3);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BUFF_COUNT), 0);
            assert_eq!(rng.counter, 1);
        }

        let mut run = RunEngine::new(42, 0);
        run.enter_specific_combat(vec!["CorruptHeart".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.draw_pile = crate::tests::support::make_deck(&["Strike", "Defend"]);
        let card_random_before = combat.card_random_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::VULNERABLE), 2);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::WEAKENED), 2);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::FRAIL), 2);
        let mut status_cards: Vec<&str> = combat.state.draw_pile.iter()
            .map(|card| combat.card_registry.card_name(card.def_id))
            .filter(|id| matches!(*id, "Dazed" | "Slimed" | "Wound" | "Burn" | "Void"))
            .collect();
        status_cards.sort_unstable();
        assert_eq!(status_cards, ["Burn", "Dazed", "Slimed", "Void", "Wound"]);
        assert_eq!(combat.card_random_rng.counter, card_random_before + 5);
        assert_eq!(combat.ai_rng.counter, 3,
            "next attack consumes rollMove num plus getMove boolean");

        combat.state.enemies[0].entity.set_status(crate::status_ids::sid::STRENGTH, -5);
        let expected_strength = [2, 4, 6, 18, 70];
        for (stage, expected) in expected_strength.into_iter().enumerate() {
            combat.state.enemies[0].set_move(
                crate::enemies::move_ids::HEART_BUFF, 0, 0, 0);
            crate::combat_hooks::do_enemy_turns(combat);
            assert_eq!(combat.state.enemies[0].entity.status(
                crate::status_ids::sid::STRENGTH), expected);
            assert_eq!(combat.state.enemies[0].entity.status(
                crate::status_ids::sid::BUFF_COUNT), stage as i32 + 1);
            match stage {
                0 => assert_eq!(combat.state.enemies[0].entity.status(
                    crate::status_ids::sid::ARTIFACT), 2),
                1 => assert_eq!(combat.state.enemies[0].entity.status(
                    crate::status_ids::sid::BEAT_OF_DEATH), 2),
                2 => assert_eq!(combat.state.enemies[0].entity.status(
                    crate::status_ids::sid::PAINFUL_STABS), 1),
                _ => {}
            }
        }
    }

    #[test]
    fn snake_dagger_hp_init_wound_suicide_and_spawn_rng_match_java() {
        // Source: reference/extracted/methods/monster/SnakeDagger.java.
        let mut hp_values = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut run = RunEngine::new(seed, 0);
            hp_values.insert(run.roll_enemy_hp("SnakeDagger").0);
        }
        assert_eq!(hp_values, (20..=25).collect());

        let mut solo = RunEngine::new(42, 0);
        solo.enter_specific_combat(vec!["SnakeDagger".to_string()]);
        let combat = solo.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.discard_pile.clear();
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SD_WOUND);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::FIRST_MOVE), 0);
        assert_eq!(combat.ai_rng.counter, 1);

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 491);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SD_EXPLODE);
        assert_eq!(combat.ai_rng.counter, 2);
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Wound").count(), 1);

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 466);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert!(combat.state.combat_over && combat.state.player_won);
        assert_eq!(combat.ai_rng.counter, 2,
            "solo LoseHPAction clears the queued RollMoveAction");

        let mut group = RunEngine::new(42, 0);
        group.enter_specific_combat(vec!["Reptomancer".to_string()]);
        let combat = group.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.discard_pile.clear();
        assert_eq!(combat.state.enemies.iter().map(|enemy| enemy.id.as_str())
            .collect::<Vec<_>>(), ["SnakeDagger", "Reptomancer", "SnakeDagger"]);
        assert!(combat.state.enemies[0].is_minion);
        assert!(combat.state.enemies[2].is_minion);
        // Keep only the left dagger active so this SnakeDagger test isolates
        // its queued grouped RollMoveAction from Reptomancer's random AI.
        combat.state.enemies[1].move_id = -1;
        combat.state.enemies[2].move_id = -1;
        let ai_before = combat.ai_rng.counter;

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies.len(), 3);
        assert_eq!(combat.ai_rng.counter, ai_before + 1);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SD_EXPLODE);
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Wound").count(), 1);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 466);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_eq!(combat.ai_rng.counter, ai_before + 2,
            "with Reptomancer alive, the dead dagger's queued roll still executes");
    }

    #[test]
    fn reptomancer_group_stats_spawns_capacity_and_death_match_java() {
        // Sources: reference/extracted/methods/monster/Reptomancer.java and
        // decompiled/java-src/com/megacrit/cardcrawl/helpers/MonsterHelper.java.
        // The canonical encounter starts with two initialized minion daggers;
        // A18 summons two per action, otherwise one, into four tracked slots.
        for (ascension, hp_range, strike, bite, per_spawn) in [
            (0, 180..=190, 13, 30, 1),
            (3, 180..=190, 16, 34, 1),
            (8, 190..=200, 16, 34, 1),
            (18, 190..=200, 16, 34, 2),
        ] {
            let mut hp_values = std::collections::BTreeSet::new();
            for seed in 1..=256 {
                let mut run = RunEngine::new(seed, ascension);
                run.enter_specific_combat(vec!["Reptomancer".to_string()]);
                let combat = run.combat_engine.as_ref().unwrap();
                assert_eq!(combat.state.enemies.iter().map(|enemy| enemy.id.as_str())
                    .collect::<Vec<_>>(),
                    ["SnakeDagger", "Reptomancer", "SnakeDagger"]);
                let repto = &combat.state.enemies[1];
                hp_values.insert(repto.entity.hp);
                assert_eq!(repto.entity.status(crate::status_ids::sid::STARTING_DMG),
                    strike);
                assert_eq!(repto.entity.status(crate::status_ids::sid::STR_AMT), bite);
                assert_eq!(repto.entity.status(crate::status_ids::sid::BLOCK_AMT),
                    per_spawn);
                assert_eq!(repto.entity.status(crate::status_ids::sid::COUNT), 2);
                assert_eq!(repto.entity.status(crate::status_ids::sid::FIRST_MOVE), 0);
                assert_eq!(repto.move_id, crate::enemies::move_ids::REPTO_SPAWN);
                assert!(matches!(repto.intent, crate::combat_types::Intent::Unknown));
                assert!(combat.state.enemies[0].is_minion);
                assert!(combat.state.enemies[2].is_minion);
                assert_eq!(combat.ai_rng.counter, 3);
            }
            assert_eq!(hp_values, hp_range.collect());
        }

        let mut baseline = RunEngine::new(42, 0);
        baseline.enter_specific_combat(vec!["Reptomancer".to_string()]);
        let combat = baseline.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.discard_pile.clear();
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies.len(), 4);
        assert_eq!(combat.ai_rng.counter, ai_before + 4,
            "two original daggers, one spawned dagger, and Reptomancer each roll");
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SD_EXPLODE);
        assert_eq!(combat.state.enemies[2].move_id,
            crate::enemies::move_ids::SD_EXPLODE);
        assert_eq!(combat.state.enemies[3].move_id,
            crate::enemies::move_ids::SD_WOUND);
        assert!(combat.state.enemies[3].is_minion);
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Wound").count(), 2);

        let mut a18 = RunEngine::new(42, 18);
        a18.enter_specific_combat(vec!["Reptomancer".to_string()]);
        let combat = a18.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies.len(), 5);
        assert_eq!(combat.ai_rng.counter, ai_before + 5,
            "A18 initializes two spawned daggers before Reptomancer rolls");
        assert!(combat.state.enemies[3..].iter().all(|enemy|
            enemy.id == "SnakeDagger" && enemy.is_minion));

        // All four tracked slots are occupied, so another explicit summon
        // creates nothing even though takeTurn still queues a normal roll.
        for (idx, enemy) in combat.state.enemies.iter_mut().enumerate() {
            if idx != 1 {
                enemy.move_id = -1;
            }
        }
        combat.state.enemies[1].set_move(
            crate::enemies::move_ids::REPTO_SPAWN, 0, 0, 0);
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies.len(), 5);
        assert_eq!(combat.ai_rng.counter, ai_before + 1);

        // Source: full Reptomancer.java `die`: every surviving group member
        // receives HideHealthBarAction followed by SuicideAction.
        let repto_hp = combat.state.enemies[1].entity.hp;
        combat.deal_damage_to_enemy(1, repto_hp);
        assert!(combat.state.enemies.iter().all(|enemy| enemy.entity.hp == 0));
        assert!(combat.state.enemies.iter().all(|enemy| !enemy.is_escaping));
        assert!(combat.check_combat_end());
        assert!(combat.state.player_won);
    }

    #[test]
    fn darkling_stats_recursive_ai_and_half_dead_revival_match_java() {
        // Source: reference/extracted/methods/monster/Darkling.java.
        for (ascension, hp_range, chomp, nip_range, high_ai) in [
            (0, 48..=56, 8, 7..=11, 0),
            (2, 48..=56, 9, 9..=13, 0),
            (7, 50..=59, 9, 9..=13, 0),
            (17, 50..=59, 9, 9..=13, 1),
        ] {
            let mut hp_values = std::collections::HashSet::new();
            let mut nip_values = std::collections::HashSet::new();
            for seed in 1..=256 {
                let mut run = RunEngine::new(seed, ascension);
                run.enter_specific_combat(vec!["Darkling".to_string()]);
                let combat = run.combat_engine.as_ref().unwrap();
                let enemy = &combat.state.enemies[0];
                hp_values.insert(enemy.entity.hp);
                nip_values.insert(enemy.entity.status(crate::status_ids::sid::STR_AMT));
                assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), chomp);
                assert_eq!(enemy.entity.status(crate::status_ids::sid::HIGH_ASCENSION_AI),
                    high_ai);
                assert_eq!(enemy.entity.status(crate::status_ids::sid::REGROW), 1);
                assert_eq!(combat.ai_rng.counter, 1);
            }
            assert_eq!(hp_values, hp_range.collect());
            assert_eq!(nip_values, nip_range.collect());
        }

        let mut low = crate::enemies::create_enemy("Darkling", 48, 48);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut low, 49, &mut crate::seed::StsRandom::new(0));
        assert_eq!(low.move_id, crate::enemies::move_ids::DARK_HARDEN);
        assert_eq!(low.move_block(), 12);
        assert_eq!(low.effect(crate::combat_types::mfx::STRENGTH), None);

        let mut high = crate::enemies::create_enemy("Darkling", 50, 50);
        high.entity.set_status(crate::status_ids::sid::STARTING_DMG, 9);
        high.entity.set_status(crate::status_ids::sid::STR_AMT, 13);
        high.entity.set_status(crate::status_ids::sid::HIGH_ASCENSION_AI, 1);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut high, 49, &mut crate::seed::StsRandom::new(0));
        assert_eq!(high.move_id, crate::enemies::move_ids::DARK_HARDEN);
        assert_eq!(high.effect(crate::combat_types::mfx::STRENGTH), Some(2));

        let mut even = crate::enemies::create_enemy("Darkling", 48, 48);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut even, 50, &mut crate::seed::StsRandom::new(0));
        let mut no_reroll = crate::seed::StsRandom::new(7);
        crate::enemies::roll_next_move_with_num_and_rng(&mut even, 0, &mut no_reroll);
        assert_eq!(even.move_id, crate::enemies::move_ids::DARK_CHOMP);
        assert_eq!((even.move_damage(), even.move_hits()), (8, 2));
        assert_eq!(no_reroll.counter, 0);

        let seed_for_range = |low: i32, high: i32| (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            (low..=high).contains(&rng.random_range(40, 99))
        }).unwrap();
        let mut odd = crate::enemies::create_enemy("Darkling", 48, 48);
        odd.entity.set_status(crate::status_ids::sid::COUNT, 1);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut odd, 50, &mut crate::seed::StsRandom::new(0));
        let mut reroll = crate::seed::StsRandom::new(seed_for_range(40, 69));
        crate::enemies::roll_next_move_with_num_and_rng(&mut odd, 0, &mut reroll);
        assert_eq!(odd.move_id, crate::enemies::move_ids::DARK_HARDEN);
        assert_eq!(reroll.counter, 1,
            "odd group index recursively rerolls num 0 into 40..=99");

        let seed_for_full = (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            (40..=69).contains(&rng.random(99))
        }).unwrap();
        let mut repeated_nip = crate::enemies::create_enemy("Darkling", 48, 48);
        repeated_nip.entity.set_status(crate::status_ids::sid::FIRST_MOVE, 0);
        repeated_nip.move_id = crate::enemies::move_ids::DARK_NIP;
        repeated_nip.move_history = vec![crate::enemies::move_ids::DARK_NIP];
        let mut full_reroll = crate::seed::StsRandom::new(seed_for_full);
        crate::enemies::roll_next_move_with_num_and_rng(
            &mut repeated_nip, 99, &mut full_reroll);
        assert_eq!(repeated_nip.move_id, crate::enemies::move_ids::DARK_HARDEN);
        assert_eq!(full_reroll.counter, 1,
            "two Nips recursively reroll through the full 0..=99 table");

        let mut revive = RunEngine::new(42, 17);
        revive.run_state.relics.push("Philosopher's Stone".to_string());
        revive.enter_specific_combat(vec![
            "Darkling".to_string(), "Darkling".to_string(), "Darkling".to_string()]);
        let combat = revive.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        let stored_chomp = combat.state.enemies[0]
            .entity.status(crate::status_ids::sid::STARTING_DMG);
        combat.state.enemies[0].entity.set_status(crate::status_ids::sid::STRENGTH, 5);
        combat.state.enemies[0].entity.set_status(crate::status_ids::sid::POISON, 3);
        combat.state.enemies[0].entity.set_status(crate::status_ids::sid::ARTIFACT, 2);
        combat.deal_damage_to_enemy(0, 999);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::REBIRTH_PENDING), 1);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::DARK_WAIT);
        assert!(!combat.state.enemies[0].is_targetable());
        for status in [crate::status_ids::sid::STRENGTH,
            crate::status_ids::sid::POISON, crate::status_ids::sid::ARTIFACT,
            crate::status_ids::sid::REGROW]
        {
            assert_eq!(combat.state.enemies[0].entity.status(status), 0);
        }
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::STARTING_DMG), stored_chomp);

        combat.deal_damage_to_enemy(1, 999);
        combat.state.enemies[2].is_escaping = true;
        let ai_before_wait = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.ai_rng.counter, ai_before_wait + 2);
        for darkling in &combat.state.enemies[..2] {
            assert_eq!(darkling.entity.hp, 0);
            assert_eq!(darkling.move_id, crate::enemies::move_ids::DARK_REINCARNATE);
        }

        crate::combat_hooks::do_enemy_turns(combat);
        for darkling in &combat.state.enemies[..2] {
            assert_eq!(darkling.entity.hp, darkling.entity.max_hp / 2);
            assert_eq!(darkling.entity.status(crate::status_ids::sid::REBIRTH_PENDING), 0);
            assert_eq!(darkling.entity.status(crate::status_ids::sid::REGROW), 1);
            assert_eq!(darkling.entity.status(crate::status_ids::sid::STRENGTH), 1,
                "PhilosopherStone.onSpawnMonster fires after revival");
        }

        let mut finish = RunEngine::new(99, 0);
        finish.enter_specific_combat(vec![
            "Darkling".to_string(), "Darkling".to_string(), "Darkling".to_string()]);
        let combat = finish.combat_engine.as_mut().unwrap();
        for idx in 0..3 {
            combat.deal_damage_to_enemy(idx, 999);
        }
        assert!(combat.state.enemies.iter().all(|enemy|
            enemy.entity.hp == 0
                && enemy.entity.status(crate::status_ids::sid::REBIRTH_PENDING) == 0));
        assert!(combat.state.is_victory());
    }

    #[test]
    fn deca_thresholds_cycle_team_block_and_plated_armor_match_java() {
        // Source: reference/extracted/methods/monster/Deca.java.
        for (ascension, hp, beam, artifact, high_ai) in [
            (0, 250, 10, 2, 0),
            (4, 250, 12, 2, 0),
            (9, 265, 12, 2, 0),
            (19, 265, 12, 3, 1),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Deca".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let deca = &combat.state.enemies[0];
            assert_eq!((deca.entity.hp, deca.entity.max_hp), (hp, hp));
            assert_eq!(deca.entity.status(crate::status_ids::sid::BEAM_DMG), beam);
            assert_eq!(deca.entity.status(crate::status_ids::sid::ARTIFACT), artifact);
            assert_eq!(deca.entity.status(crate::status_ids::sid::HIGH_ASCENSION_AI), high_ai);
            assert_eq!(deca.move_id, crate::enemies::move_ids::DECA_BEAM);
            assert_eq!((deca.move_damage(), deca.move_hits()), (beam, 2));
            assert_eq!(deca.effect(crate::combat_types::mfx::DAZE), Some(2));
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut base = RunEngine::new(42, 0);
        base.enter_specific_combat(vec!["Deca".to_string()]);
        let combat = base.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.discard_pile.clear();
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 480);
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Dazed").count(), 2);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::DECA_SQUARE);
        assert_eq!(combat.state.enemies[0].move_block(), 16);
        assert_eq!(combat.state.enemies[0].effect(
            crate::combat_types::mfx::BLOCK_ALL_ALLIES), Some(16));
        assert_eq!(combat.state.enemies[0].effect(
            crate::combat_types::mfx::PLATED_ARMOR_ALL), None);
        assert_eq!(combat.ai_rng.counter, 2);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.block, 16);
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::DECA_BEAM);
        assert_eq!(combat.ai_rng.counter, 3);

        let mut high = RunEngine::new(42, 19);
        high.enter_specific_combat(vec!["DonuAndDeca".to_string()]);
        let combat = high.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.enemies[1].set_move(
            crate::enemies::move_ids::DECA_SQUARE, 0, 0, 16);
        combat.state.enemies[1].add_effect(
            crate::combat_types::mfx::BLOCK_ALL_ALLIES, 16);
        combat.state.enemies[1].add_effect(
            crate::combat_types::mfx::PLATED_ARMOR_ALL, 3);
        crate::combat_hooks::do_enemy_turns(combat);
        for enemy in &combat.state.enemies {
            assert_eq!(enemy.entity.status(crate::status_ids::sid::PLATED_ARMOR), 3);
            assert_eq!(enemy.entity.block, 19,
                "Square's 16 block resolves before Plated Armor's end-turn 3");
        }

        combat.state.enemies[0].entity.block = 0;
        let hp_before = combat.state.enemies[0].entity.hp;
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(combat.state.enemies[0].entity.hp, hp_before - 1);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::PLATED_ARMOR), 2,
            "unblocked NORMAL damage removes one Plated Armor");

        combat.state.enemies[1].set_move(
            crate::enemies::move_ids::DECA_SQUARE, 0, 0, 16);
        combat.state.enemies[1].add_effect(
            crate::combat_types::mfx::BLOCK_ALL_ALLIES, 16);
        combat.state.enemies[1].add_effect(
            crate::combat_types::mfx::PLATED_ARMOR_ALL, 3);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.status(
            crate::status_ids::sid::PLATED_ARMOR), 5);
        assert_eq!(combat.state.enemies[1].entity.status(
            crate::status_ids::sid::PLATED_ARMOR), 6);
        assert_eq!(combat.state.enemies[0].entity.block, 21);
        assert_eq!(combat.state.enemies[1].entity.block, 22);
    }

    #[test]
    fn donu_thresholds_cycle_and_team_strength_match_java() {
        // Source: reference/extracted/methods/monster/Donu.java.
        for (ascension, hp, beam, artifact) in [
            (0, 250, 10, 2),
            (4, 250, 12, 2),
            (9, 265, 12, 2),
            (19, 265, 12, 3),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Donu".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let donu = &combat.state.enemies[0];
            assert_eq!((donu.entity.hp, donu.entity.max_hp), (hp, hp));
            assert_eq!(donu.entity.status(crate::status_ids::sid::BEAM_DMG), beam);
            assert_eq!(donu.entity.status(crate::status_ids::sid::ARTIFACT), artifact);
            assert_eq!(donu.move_id, crate::enemies::move_ids::DONU_CIRCLE);
            assert_eq!(donu.effect(crate::combat_types::mfx::STRENGTH), Some(3));
            assert_eq!(donu.effect(crate::combat_types::mfx::STRENGTH_ALL_ALLIES), Some(3));
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut run = RunEngine::new(42, 0);
        run.enter_specific_combat(vec!["DonuAndDeca".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.discard_pile.clear();
        assert_eq!(combat.ai_rng.counter, 2,
            "Donu and Deca each consume one initialization roll");

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 474,
            "Donu's Circle Strength modifies Deca's later Beam in group order");
        for enemy in &combat.state.enemies {
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STRENGTH), 3);
        }
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::DONU_BEAM);
        assert_eq!(combat.state.enemies[1].move_id,
            crate::enemies::move_ids::DECA_SQUARE);
        assert_eq!(combat.ai_rng.counter, 4);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 448);
        assert!(combat.state.enemies.iter().all(|enemy| enemy.entity.block == 16));
        assert_eq!(combat.state.enemies[0].move_id,
            crate::enemies::move_ids::DONU_CIRCLE);
        assert_eq!(combat.state.enemies[1].move_id,
            crate::enemies::move_ids::DECA_BEAM);
        assert_eq!(combat.ai_rng.counter, 6);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 416);
        for enemy in &combat.state.enemies {
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STRENGTH), 6);
        }
        assert_eq!(combat.state.discard_pile.iter().filter(|card|
            combat.card_registry.card_name(card.def_id) == "Dazed").count(), 4);
        assert_eq!(combat.ai_rng.counter, 8);
    }

    #[test]
    fn giant_head_thresholds_rng_countdown_damage_and_slow_match_java() {
        // Sources: reference/extracted/methods/monster/GiantHead.java and
        // decompiled/java-src/com/megacrit/cardcrawl/powers/SlowPower.java.
        for (ascension, hp, death_damage, count_after_opening) in [
            (0, 500, 30, 4),
            (3, 500, 40, 4),
            (7, 500, 40, 4),
            (8, 520, 40, 4),
            (18, 520, 40, 3),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["GiantHead".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let head = &combat.state.enemies[0];
            assert_eq!((head.entity.hp, head.entity.max_hp), (hp, hp));
            assert_eq!(head.entity.status(crate::status_ids::sid::STARTING_DEATH_DMG),
                death_damage);
            assert_eq!(head.entity.status(crate::status_ids::sid::COUNT),
                count_after_opening);
            assert_eq!(head.entity.status(crate::status_ids::sid::SLOW), 1,
                "sentinel one represents SlowPower's installed amount zero");
            assert!(matches!(head.move_id,
                crate::enemies::move_ids::GH_GLARE
                    | crate::enemies::move_ids::GH_COUNT));
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut rng = crate::seed::StsRandom::new(0);
        let mut low_roll = crate::enemies::create_enemy("GiantHead", 500, 500);
        crate::enemies::roll_initial_move_with_num_and_rng(&mut low_roll, 0, &mut rng);
        assert_eq!(low_roll.move_id, crate::enemies::move_ids::GH_GLARE);
        assert_eq!(low_roll.effect(crate::combat_types::mfx::WEAK), Some(1));
        assert_eq!(low_roll.entity.status(crate::status_ids::sid::COUNT), 4);

        let mut high_roll = crate::enemies::create_enemy("GiantHead", 500, 500);
        crate::enemies::roll_initial_move_with_num_and_rng(&mut high_roll, 99, &mut rng);
        assert_eq!(high_roll.move_id, crate::enemies::move_ids::GH_COUNT);

        // A single previous Glare does not force alternation; only lastTwoMoves
        // blocks the low-roll Glare branch.
        low_roll.move_history.clear();
        low_roll.entity.set_status(crate::status_ids::sid::COUNT, 4);
        crate::enemies::roll_next_move_with_num(&mut low_roll, 0);
        assert_eq!(low_roll.move_id, crate::enemies::move_ids::GH_GLARE);
        crate::enemies::roll_next_move_with_num(&mut low_roll, 0);
        assert_eq!(low_roll.move_id, crate::enemies::move_ids::GH_COUNT);

        let mut death = crate::enemies::create_enemy("GiantHead", 500, 500);
        death.entity.set_status(crate::status_ids::sid::COUNT, 1);
        let expected_damage = [30, 35, 40, 45, 50, 55, 60, 60];
        for expected in expected_damage {
            crate::enemies::roll_next_move_with_num(&mut death, 0);
            assert_eq!(death.move_id, crate::enemies::move_ids::GH_IT_IS_TIME);
            assert_eq!(death.move_damage(), expected);
        }

        // Slow increments after each card. With four Strength, the two Strikes
        // have raw damage ten and receive +10%, then +20%, from earlier cards.
        let mut run = RunEngine::new(7, 0);
        run.enter_specific_combat(vec!["GiantHead".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.state.player.set_status(crate::status_ids::sid::STRENGTH, 4);
        combat.state.hand = ["Defend", "Strike", "Strike"]
            .iter().map(|name| combat.card_registry.make_card(name)).collect();
        combat.state.energy = 10;
        combat.state.draw_pile.clear();
        assert_eq!(combat.state.enemies[0].entity.hp, 500);

        combat.execute_action(&crate::actions::Action::PlayCard {
            card_idx: 0, target_idx: -1 });
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::SLOW), 2);
        combat.execute_action(&crate::actions::Action::PlayCard {
            card_idx: 0, target_idx: 0 });
        assert_eq!(combat.state.enemies[0].entity.hp, 489);
        combat.execute_action(&crate::actions::Action::PlayCard {
            card_idx: 0, target_idx: 0 });
        assert_eq!(combat.state.enemies[0].entity.hp, 477);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::SLOW), 4);

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::SLOW), 1,
            "SlowPower.atEndOfRound resets amount without removing the power");
        assert_eq!(combat.ai_rng.counter, 2);
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
    fn mugger_stats_gold_theft_direct_moves_and_ai_ticks_match_java() {
        // Sources: reference/extracted/methods/monster/Mugger.java and
        // decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAction.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=256 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("Mugger").0);
            let mut high = RunEngine::new(seed, 7);
            high_hp.insert(high.roll_enemy_hp("Mugger").0);
        }
        assert_eq!(low_hp, (48..=52).collect());
        assert_eq!(high_hp, (50..=54).collect());

        for (ascension, swipe, big_swipe, block, gold, hp_range) in [
            (0, 10, 16, 11, 15, 48..=52),
            (2, 11, 18, 11, 15, 48..=52),
            (7, 11, 18, 11, 15, 50..=54),
            (17, 11, 18, 17, 20, 50..=54),
        ] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["Mugger".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert!(hp_range.contains(&enemy.entity.hp));
            assert_eq!(enemy.move_id, crate::enemies::move_ids::MUGGER_MUG);
            assert_eq!(enemy.move_damage(), swipe);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG), swipe);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), big_swipe);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), block);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::TURN_COUNT), gold);
            assert_eq!(combat.ai_rng.counter, 1,
                "AbstractMonster.init consumes the only getMove roll");
        }

        let smoke_seed = (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            let _ = rng.random(2); // first Mug voice
            let _ = rng.random(2); // second Mug voice
            let _ = rng.random_float(); // second-Mug dialogue
            rng.random_float() < 0.5 // Smoke Bomb branch
        }).unwrap();
        let mut engine = RunEngine::new(42, 0);
        engine.enter_specific_combat(vec!["Mugger".to_string()]);
        engine.combat_engine.as_mut().unwrap().ai_rng = crate::seed::StsRandom::new(smoke_seed);

        engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(engine.run_state.gold, 84);
        {
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::MUGGER_MUG);
            assert_eq!(combat.state.enemies[0].entity.status(crate::status_ids::sid::COUNT), 15);
            assert_eq!(combat.ai_rng.counter, 1,
                "first Mug consumes only playSfx's aiRng.random(2)");
        }

        engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(engine.run_state.gold, 69);
        {
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::MUGGER_SMOKE_BOMB);
            assert_eq!(combat.state.enemies[0].move_block(), 11);
            assert_eq!(combat.ai_rng.counter, 4,
                "second Mug consumes voice, dialogue, and route booleans");
        }

        engine.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        {
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies[0].entity.block, 11);
            assert_eq!(combat.state.enemies[0].move_id,
                crate::enemies::move_ids::MUGGER_ESCAPE);
            assert!(!combat.state.enemies[0].is_escaping);
            assert_eq!(combat.ai_rng.counter, 4,
                "direct SetMove actions consume no RollMoveAction tick");
        }

        let big_seed = (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            let _ = rng.random(2);
            let _ = rng.random(2);
            let _ = rng.random_float();
            rng.random_float() >= 0.5
        }).unwrap();
        let mut big = crate::enemies::create_enemy("Mugger", 52, 52);
        let mut big_rng = crate::seed::StsRandom::new(big_seed);
        crate::enemies::act2::advance_mugger_after_turn(&mut big, &mut big_rng);
        crate::enemies::act2::advance_mugger_after_turn(&mut big, &mut big_rng);
        assert_eq!(big.move_id, crate::enemies::move_ids::MUGGER_BIG_SWIPE);
        assert_eq!(big.move_damage(), 16);
        assert_eq!(big_rng.counter, 4);
        crate::enemies::act2::advance_mugger_after_turn(&mut big, &mut big_rng);
        assert_eq!(big.move_id, crate::enemies::move_ids::MUGGER_SMOKE_BOMB);
        assert_eq!(big_rng.counter, 5, "Big Swipe consumes its voice roll");

        let mut a17_smoke = crate::enemies::create_enemy("Mugger", 54, 54);
        a17_smoke.entity.set_status(crate::status_ids::sid::BLOCK_AMT, 17);
        let mut a17_rng = crate::seed::StsRandom::new(smoke_seed);
        crate::enemies::act2::advance_mugger_after_turn(&mut a17_smoke, &mut a17_rng);
        crate::enemies::act2::advance_mugger_after_turn(&mut a17_smoke, &mut a17_rng);
        assert_eq!(a17_smoke.move_id, crate::enemies::move_ids::MUGGER_SMOKE_BOMB);
        assert_eq!(a17_smoke.move_block(), 17,
            "A17 adds six to the constructor's eleven escape block");

        let mut death = RunEngine::new(42, 0);
        death.enter_specific_combat(vec!["Mugger".to_string()]);
        let combat = death.combat_engine.as_mut().unwrap();
        let before_death = combat.ai_rng.counter;
        let hp = combat.state.enemies[0].entity.hp;
        combat.deal_damage_to_enemy(0, hp);
        assert_eq!(combat.ai_rng.counter, before_death + 1,
            "Mugger.die consumes one aiRng voice variant");

        let mut refund = RunEngine::new(42, 0);
        refund.enter_specific_combat(vec!["Mugger".to_string()]);
        refund.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        refund.combat_engine.as_mut().unwrap().state.enemies[0].entity.hp = 0;
        refund.step(&RunAction::CombatAction(crate::actions::Action::EndTurn));
        assert!((109..=119).contains(&refund.run_state.gold),
            "death returns 15 stolen gold before the normal 10..=20 reward");
    }

    #[test]
    fn gremlin_leader_encounter_ai_summons_encourage_and_death_match_java() {
        // Sources: reference/extracted/methods/monster/GremlinLeader.java,
        // decompiled/java-src/com/megacrit/cardcrawl/helpers/MonsterHelper.java,
        // and decompiled/java-src/com/megacrit/cardcrawl/actions/unique/
        // SummonGremlinAction.java.
        let mut low_hp = std::collections::HashSet::new();
        let mut high_hp = std::collections::HashSet::new();
        for seed in 1..=512 {
            let mut low = RunEngine::new(seed, 0);
            low_hp.insert(low.roll_enemy_hp("GremlinLeader").0);
            let mut high = RunEngine::new(seed, 8);
            high_hp.insert(high.roll_enemy_hp("GremlinLeader").0);
        }
        assert_eq!(low_hp, (140..=148).collect());
        assert_eq!(high_hp, (145..=155).collect());

        for (ascension, hp_range, strength, block) in [
            (0, 140..=148, 3, 6),
            (3, 140..=148, 4, 6),
            (8, 145..=155, 4, 6),
            (18, 145..=155, 5, 10),
        ] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["GremlinLeader".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies.len(), 3);
            assert!(combat.state.enemies[..2].iter().all(|enemy| enemy.is_minion));
            assert!(combat.state.enemies[..2].iter().all(|enemy| matches!(enemy.id.as_str(),
                "GremlinWarrior" | "GremlinThief" | "GremlinFat"
                    | "GremlinTsundere" | "GremlinWizard")));
            let leader = &combat.state.enemies[2];
            assert_eq!(leader.id, "GremlinLeader");
            assert!(hp_range.contains(&leader.entity.hp));
            assert_eq!(leader.entity.status(crate::status_ids::sid::STR_AMT), strength);
            assert_eq!(leader.entity.status(crate::status_ids::sid::BLOCK_AMT), block);
            assert_eq!(leader.entity.status(crate::status_ids::sid::COUNT), 2);
            assert_eq!(leader.entity.status(crate::status_ids::sid::STARTING_DMG), ascension);
            assert!(matches!(leader.move_id,
                crate::enemies::move_ids::GL_ENCOURAGE
                    | crate::enemies::move_ids::GL_STAB));
            assert_eq!(combat.ai_rng.counter, 3,
                "two gremlins and the Leader each consume one opening roll");
        }

        // With one ally and a repeated Rally, the low branch recursively draws
        // 50..99 and consumes exactly one additional aiRng tick.
        let mut recursive = crate::enemies::create_enemy("GremlinLeader", 140, 140);
        recursive.entity.set_status(crate::status_ids::sid::COUNT, 1);
        let mut ai_rng = crate::seed::StsRandom::new(9);
        crate::enemies::roll_next_move_with_num_and_rng(&mut recursive, 0, &mut ai_rng);
        assert!(matches!(recursive.move_id,
            crate::enemies::move_ids::GL_ENCOURAGE
                | crate::enemies::move_ids::GL_STAB));
        assert_eq!(ai_rng.counter, 1);

        let mut encourage = RunEngine::new(42, 18);
        encourage.enter_specific_combat(vec!["GremlinLeader".to_string()]);
        let combat = encourage.combat_engine.as_mut().unwrap();
        for minion in &mut combat.state.enemies[..2] {
            minion.move_id = -1;
        }
        let leader = &mut combat.state.enemies[2];
        leader.move_history.clear();
        leader.set_move(crate::enemies::move_ids::GL_STAB, 6, 3, 0);
        leader.entity.set_status(crate::status_ids::sid::COUNT, 2);
        crate::enemies::roll_next_move_with_num(leader, 0);
        assert_eq!(leader.move_id, crate::enemies::move_ids::GL_ENCOURAGE);
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[2].entity.status(crate::status_ids::sid::STRENGTH), 5);
        assert_eq!(combat.state.enemies[2].entity.block, 0,
            "Encourage never gives the Leader block");
        for minion in &combat.state.enemies[..2] {
            assert_eq!(minion.entity.status(crate::status_ids::sid::STRENGTH), 5);
            assert_eq!(minion.entity.block, 10);
        }
        assert_eq!(combat.ai_rng.counter, ai_before + 2,
            "Encourage quote and queued RollMove each consume one tick");

        let mut rally = RunEngine::new(42, 18);
        rally.enter_specific_combat(vec!["GremlinLeader".to_string()]);
        let combat = rally.combat_engine.as_mut().unwrap();
        for minion in &mut combat.state.enemies[..2] {
            minion.entity.hp = 0;
        }
        combat.state.enemies[2].set_move(crate::enemies::move_ids::GL_RALLY, 0, 0, 0);
        combat.state.enemies[2].move_effects.clear();
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies.len(), 5);
        let summons = &combat.state.enemies[3..];
        assert!(summons.iter().all(|enemy| enemy.is_minion && enemy.is_alive()));
        assert!(summons.iter().all(|enemy| matches!(enemy.id.as_str(),
            "GremlinWarrior" | "GremlinThief" | "GremlinFat"
                | "GremlinTsundere" | "GremlinWizard")));
        assert_eq!(combat.state.enemies[2].entity.status(crate::status_ids::sid::COUNT), 2);
        assert_eq!(combat.ai_rng.counter, ai_before + 5,
            "two weighted type rolls, two init rolls, then Leader RollMove");

        let mut death = RunEngine::new(42, 0);
        death.enter_specific_combat(vec!["GremlinLeader".to_string()]);
        let combat = death.combat_engine.as_mut().unwrap();
        combat.deal_damage_to_enemy(2, 999);
        assert_eq!(combat.state.enemies[2].entity.hp, 0);
        assert!(combat.state.enemies[..2].iter().all(|enemy|
            enemy.is_escaping && enemy.entity.hp == 0));
        assert!(combat.state.is_victory());
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
            combat.card_registry.card_name(card.def_id) == "Dazed").count(), 2);
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
            combat.card_registry.card_name(card.def_id) == "Dazed").count(), 6);
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
        // Source: CampfireSleepEffect.java casts maxHealth * 0.3f to int,
        // so 72 max HP heals 21 rather than rounding 21.6 up to 22.
        assert_eq!(engine.run_state.current_hp, 71);
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
    fn searing_blow_remains_campfire_upgradeable_and_persists_exact_level() {
        // SearingBlow.canUpgrade always returns true; each call increments
        // timesUpgraded even though the canonical card ID remains Searing Blow.
        // Java: reference/extracted/methods/card/SearingBlow.java
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.deck = vec!["Searing Blow+".to_string()];

        for expected_level in [2, 3] {
            engine.phase = RunPhase::Campfire;
            assert!(engine
                .get_legal_actions()
                .contains(&RunAction::CampfireUpgrade(0)));
            engine.step(&RunAction::CampfireUpgrade(0));
            assert_eq!(engine.run_state.deck[0], "Searing Blow+");
            assert_eq!(engine.run_state.deck_card_states[0].misc, expected_level);
        }

        engine.debug_enter_specific_combat(&["JawWorm"]);
        assert_eq!(
            engine
                .get_combat_engine()
                .expect("combat active")
                .state
                .master_deck[0]
                .misc,
            3,
        );

        let mut obtained = RunState::new(0);
        obtained.deck.clear();
        obtained.deck_card_states.clear();
        obtained.relics.push("Molten Egg 2".to_string());
        obtained.relic_flags.rebuild(&obtained.relics);
        obtain_master_deck_card_state(&mut obtained, "Searing Blow+".to_string());
        assert_eq!(obtained.deck_card_states[0].misc, 2);
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
    fn golden_idol_event_obtains_relic_then_offers_source_consequences() {
        // GoldenIdolEvent.java first obtains Golden Idol (or Circlet), then
        // offers Injury, 25/35% damage, or 8/10% max-HP loss. It never grants
        // 300 gold merely for taking the idol.
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.max_hp = 72;
        engine.run_state.current_hp = 72;
        let gold_before = engine.run_state.gold;
        let event = crate::events::typed_events_for_act(1)
            .into_iter()
            .find(|event| event.name == "Golden Idol")
            .expect("Golden Idol event");
        engine.debug_set_typed_event_state(event.clone());

        assert!(engine
            .step_with_result(&RunAction::EventChoice(0))
            .action_accepted);
        assert_eq!(engine.current_phase(), RunPhase::Event);
        assert_eq!(engine.run_state.current_hp, 72);
        assert_eq!(engine.run_state.gold, gold_before);
        assert!(engine
            .run_state
            .relics
            .iter()
            .any(|relic| relic == "Golden Idol"));
        let consequence = engine.debug_current_event().expect("consequence screen");
        assert_eq!(consequence.options.len(), 3);
        assert_eq!(consequence.options[1].text, "Take 18 damage");
        assert_eq!(consequence.options[2].text, "Lose 5 max HP");

        let deck_before = engine.run_state.deck.len();
        assert!(engine
            .step_with_result(&RunAction::EventChoice(0))
            .action_accepted);
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
        assert_eq!(engine.run_state.deck.len(), deck_before + 1);
        assert_eq!(engine.run_state.deck.last().map(String::as_str), Some("Injury"));

        let mut damage = RunEngine::new(43, 0);
        damage.run_state.max_hp = 72;
        damage.run_state.current_hp = 72;
        damage.debug_set_typed_event_state(event.clone());
        damage.step(&RunAction::EventChoice(0));
        damage.step(&RunAction::EventChoice(1));
        assert_eq!(damage.run_state.current_hp, 54);

        let mut asc15 = RunEngine::new(44, 15);
        asc15.run_state.max_hp = 100;
        asc15.run_state.current_hp = 100;
        asc15.debug_set_typed_event_state(event.clone());
        asc15.step(&RunAction::EventChoice(0));
        asc15.step(&RunAction::EventChoice(2));
        assert_eq!(asc15.run_state.max_hp, 90);
        assert_eq!(asc15.run_state.current_hp, 90);

        let mut duplicate = RunEngine::new(45, 0);
        duplicate.run_state.relics.push("Golden Idol".to_string());
        duplicate.run_state.relic_flags.rebuild(&duplicate.run_state.relics);
        duplicate.debug_set_typed_event_state(event);
        duplicate.step(&RunAction::EventChoice(0));
        assert_eq!(
            duplicate
                .run_state
                .relics
                .iter()
                .filter(|relic| relic.as_str() == "Golden Idol")
                .count(),
            1
        );
        assert!(duplicate.run_state.relics.iter().any(|relic| relic == "Circlet"));
    }

    #[test]
    fn golden_idol_combat_gold_uses_one_room_band_and_rounded_bonus() {
        // AbstractRoom.java awards exactly one 10..20, 25..35, or 95..105
        // room band (boss gold is 75% at A13). RewardItem.java then adds
        // round(base * 0.25), so a base reward of 10 becomes 13, not 12.
        fn resolve_gold(seed: u64, ascension: i32, room_type: RoomType, idol: bool) -> i32 {
            let mut engine = RunEngine::new(seed, ascension);
            engine.run_state.floor = if room_type == RoomType::Boss { 16 } else { 1 };
            engine.run_state.map_x = 0;
            engine.run_state.map_y = 0;
            engine.map.rows[0][0].room_type = room_type;
            if idol {
                engine.run_state.relics.push("Golden Idol".to_string());
                engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
            }
            let gold_before = engine.run_state.gold;
            engine.debug_enter_specific_combat(&["Cultist"]);
            engine.debug_force_current_combat_outcome(true);
            engine.debug_resolve_current_combat_outcome();
            engine.run_state.gold - gold_before
        }

        assert_eq!(golden_idol_combat_gold(10, true), 13);
        for (room_type, range) in [
            (RoomType::Monster, 10..=20),
            (RoomType::Elite, 25..=35),
            (RoomType::Boss, 95..=105),
        ] {
            let base = resolve_gold(77, 0, room_type, false);
            let idol = resolve_gold(77, 0, room_type, true);
            assert!(range.contains(&base), "{room_type:?} base gold was {base}");
            assert_eq!(idol, golden_idol_combat_gold(base, true));
        }

        let a13_boss = resolve_gold(79, 13, RoomType::Boss, false);
        assert!((71..=79).contains(&a13_boss));
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

    #[test]
    fn pandora_box_replaces_every_starter_tagged_card_one_for_one() {
        // PandorasBox.java removes STARTER_STRIKE/STARTER_DEFEND cards, rolls
        // one returnTrulyRandomCard per removal, previews relic callbacks, and
        // obtains all replacements through FastCardObtainEffect.
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.deck = vec![
            "Strike".to_string(),
            "Strike+".to_string(),
            "Defend".to_string(),
            "Defend+".to_string(),
            "Perfected Strike".to_string(),
            "Eruption".to_string(),
            "Vigilance".to_string(),
        ];
        engine.run_state.gold = 100;
        engine.run_state.relics.extend([
            "Frozen Egg 2".to_string(),
            "Molten Egg 2".to_string(),
            "Toxic Egg 2".to_string(),
            "CeramicFish".to_string(),
        ]);
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.add_relic_reward("Pandora's Box");
        assert_eq!(engine.run_state.deck.len(), 7);
        assert_eq!(&engine.run_state.deck[..3], &["Perfected Strike", "Eruption", "Vigilance"]);
        assert!(engine.run_state.deck.iter().all(|card| {
            !matches!(card.as_str(), "Strike" | "Strike+" | "Defend" | "Defend+")
        }));
        assert!(engine.run_state.deck[3..].iter().all(|card| card.ends_with('+')));
        assert!(engine.run_state.deck[3..].iter().all(|card| {
            let base = card.trim_end_matches('+');
            WATCHER_COMMON_CARDS.contains(&base)
                || WATCHER_UNCOMMON_CARDS.contains(&base)
                || WATCHER_RARE_CARDS.contains(&base)
        }));
        assert_eq!(engine.run_state.gold, 136);
    }

    #[test]
    fn pandora_box_without_starter_tags_leaves_the_deck_unchanged() {
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.deck = vec!["Perfected Strike".to_string(), "Eruption".to_string()];
        let before = engine.run_state.deck.clone();
        engine.add_relic_reward("Pandora's Box");
        assert_eq!(engine.run_state.deck, before);
    }

    #[test]
    fn necronomicurse_normal_master_deck_removal_recreates_the_curse() {
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.deck = vec!["Necronomicurse".to_string()];
        engine.run_state.reconcile_deck_card_states();

        assert_eq!(
            engine.remove_master_deck_card(0),
            Some("Necronomicurse".to_string())
        );
        assert_eq!(engine.run_state.deck, vec!["Necronomicurse"]);
    }
}
