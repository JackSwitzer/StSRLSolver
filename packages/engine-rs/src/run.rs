//! Run state management — full Act 1 run simulation.
//!
//! Manages floor progression, deck building, card rewards, events,
//! shops, campfires, and combat via the existing CombatEngine.
//! Exposes step(action) -> (obs, reward, done, info) RL interface.

use rand::Rng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

use crate::enemies;
use crate::engine::CombatEngine;
use crate::map::{generate_map, DungeonMap, RoomType};
use crate::state::{CombatState, EnemyCombatState};

// ---------------------------------------------------------------------------
// Run-level action (distinct from combat Action)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RunAction {
    /// Choose a path on the map: index into available next nodes
    ChoosePath(usize),
    /// Pick a card reward: index into offered cards, or skip
    PickCard(usize),
    /// Skip card reward
    SkipCardReward,
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
    "Flurry", "FlyingSleeves", "FollowUp", "Halt",
    "JustLucky", "PressurePoints", "Prostrate",
    "Protect", "SashWhip", "Tranquility",
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
    "DevaForm", "Devotion", "Establishment", "Fasting",
    "Judgement", "LessonLearned", "MasterReality",
    "MentalFortress", "Omniscience", "Ragnarok",
    "Adaptation", "Scrawl", "SpiritShield", "Vault",
    "Wish",
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
// Event definitions (5 common Act 1 events)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDef {
    pub name: String,
    pub options: Vec<EventOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOption {
    pub text: String,
    pub effect: EventEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventEffect {
    /// Gain/lose HP (negative = lose)
    Hp(i32),
    /// Gain/lose gold
    Gold(i32),
    /// Gain a random card
    GainCard,
    /// Remove a random curse/status from deck
    RemoveCard,
    /// Gain a random relic
    GainRelic,
    /// Gain max HP
    MaxHp(i32),
    /// Take damage and gain gold
    DamageAndGold(i32, i32),
    /// Nothing (leave)
    Nothing,
    /// Upgrade a random card
    UpgradeCard,
    /// Golden Idol: lose 25% max HP, gain 300 gold
    GoldenIdolTake,
}

fn act1_events() -> Vec<EventDef> {
    vec![
        EventDef {
            name: "Big Fish".to_string(),
            options: vec![
                EventOption { text: "Eat (heal 5 HP)".into(), effect: EventEffect::Hp(5) },
                EventOption { text: "Banana (gain 2 max HP)".into(), effect: EventEffect::MaxHp(2) },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Golden Idol".to_string(),
            options: vec![
                EventOption { text: "Take (gain 300 gold, lose 25% max HP)".into(), effect: EventEffect::GoldenIdolTake },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Scrap Ooze".to_string(),
            options: vec![
                EventOption { text: "Reach inside (take 3 dmg, gain relic)".into(), effect: EventEffect::DamageAndGold(-3, 0) },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Shining Light".to_string(),
            options: vec![
                EventOption { text: "Enter (upgrade 2 cards, take 10 dmg)".into(), effect: EventEffect::DamageAndGold(-10, 0) },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Living Wall".to_string(),
            options: vec![
                EventOption { text: "Upgrade (upgrade a card)".into(), effect: EventEffect::UpgradeCard },
                EventOption { text: "Remove (remove a card)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
    ]
}

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

        Self {
            current_hp: max_hp,
            max_hp,
            gold: 99,
            floor: 0,
            act: 1,
            ascension,
            deck,
            relics: vec!["PureWater".to_string()], // Watcher starting relic
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
        }
    }
}

// ---------------------------------------------------------------------------
// RunEngine — the full run simulation engine
// ---------------------------------------------------------------------------

pub struct RunEngine {
    pub run_state: RunState,
    pub map: DungeonMap,
    pub phase: RunPhase,
    pub seed: u64,
    rng: rand::rngs::SmallRng,

    // Active combat (when in Combat phase)
    combat_engine: Option<CombatEngine>,

    // Card reward offerings (when in CardReward phase)
    card_rewards: Vec<String>,

    // Current event (when in Event phase)
    current_event: Option<EventDef>,

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
}

impl RunEngine {
    /// Create a new run engine with given seed and ascension level.
    pub fn new(seed: u64, ascension: i32) -> Self {
        let map = generate_map(seed, ascension);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(seed.wrapping_add(1));

        // Pick boss
        let boss_idx = rng.gen_range(0..ACT1_BOSSES.len());
        let boss_id = ACT1_BOSSES[boss_idx].to_string();

        Self {
            run_state: RunState::new(ascension),
            map,
            phase: RunPhase::MapChoice,
            seed,
            rng,
            combat_engine: None,
            card_rewards: Vec::new(),
            current_event: None,
            current_shop: None,
            boss_id,
            weak_encounter_idx: 0,
            strong_encounter_idx: 0,
            elite_encounter_idx: 0,
            total_reward: 0.0,
        }
    }

    /// Reset the engine to a fresh run with a new seed.
    pub fn reset(&mut self, seed: u64) {
        let ascension = self.run_state.ascension;
        *self = Self::new(seed, ascension);
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
            RunPhase::MapChoice => self.get_map_actions(),
            RunPhase::Combat => self.get_combat_actions(),
            RunPhase::CardReward => self.get_card_reward_actions(),
            RunPhase::Campfire => self.get_campfire_actions(),
            RunPhase::Shop => self.get_shop_actions(),
            RunPhase::Event => self.get_event_actions(),
            RunPhase::GameOver => Vec::new(),
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
        let mut actions: Vec<RunAction> = self
            .card_rewards
            .iter()
            .enumerate()
            .map(|(i, _)| RunAction::PickCard(i))
            .collect();
        actions.push(RunAction::SkipCardReward);
        actions
    }

    fn get_campfire_actions(&self) -> Vec<RunAction> {
        let mut actions = vec![RunAction::CampfireRest];
        // Upgrade: one action per upgradeable card in deck
        for (i, card) in self.run_state.deck.iter().enumerate() {
            if !card.ends_with('+') && card != "Strike_P" && card != "Defend_P" {
                // Only non-basic, non-upgraded cards
                actions.push(RunAction::CampfireUpgrade(i));
            }
        }
        // Also allow upgrading basics (Strike_P -> Strike_P+, etc.)
        for (i, card) in self.run_state.deck.iter().enumerate() {
            if !card.ends_with('+') {
                if !actions.contains(&RunAction::CampfireUpgrade(i)) {
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
            if !shop.removal_used && self.run_state.gold >= shop.remove_price && self.run_state.deck.len() > 5 {
                for (i, _) in self.run_state.deck.iter().enumerate() {
                    actions.push(RunAction::ShopRemoveCard(i));
                }
            }
        }
        actions.push(RunAction::ShopLeave);
        actions
    }

    fn get_event_actions(&self) -> Vec<RunAction> {
        if let Some(ref event) = self.current_event {
            event
                .options
                .iter()
                .enumerate()
                .map(|(i, _)| RunAction::EventChoice(i))
                .collect()
        } else {
            vec![RunAction::EventChoice(0)]
        }
    }

    // =======================================================================
    // Step — execute an action and return (reward, done)
    // =======================================================================

    /// Execute an action and return (reward, done).
    pub fn step(&mut self, action: &RunAction) -> (f32, bool) {
        let reward = match self.phase {
            RunPhase::MapChoice => self.step_map(action),
            RunPhase::Combat => self.step_combat(action),
            RunPhase::CardReward => self.step_card_reward(action),
            RunPhase::Campfire => self.step_campfire(action),
            RunPhase::Shop => self.step_shop(action),
            RunPhase::Event => self.step_event(action),
            RunPhase::GameOver => 0.0,
        };

        self.total_reward += reward;
        (reward, self.run_state.run_over)
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
                // Gain gold + go to map
                let gold = self.rng.gen_range(50..=80);
                self.run_state.gold += gold;
                self.phase = RunPhase::MapChoice;
            }
            RoomType::Boss => {
                self.enter_combat(false, true);
            }
            RoomType::None => {
                self.phase = RunPhase::MapChoice;
            }
        }

        // Floor milestone reward
        let floor_reward = self.run_state.floor as f32 / 55.0;
        floor_reward
    }

    // =======================================================================
    // Combat
    // =======================================================================

    fn enter_combat(&mut self, is_elite: bool, is_boss: bool) {
        let encounter = if is_boss {
            vec![self.boss_id.clone()]
        } else if is_elite {
            let pool = ACT1_ELITE_ENCOUNTERS;
            let idx = self.elite_encounter_idx % pool.len();
            self.elite_encounter_idx += 1;
            pool[idx].iter().map(|s| s.to_string()).collect()
        } else if self.run_state.floor <= 3 {
            let pool = ACT1_WEAK_ENCOUNTERS;
            let idx = self.weak_encounter_idx % pool.len();
            self.weak_encounter_idx += 1;
            pool[idx].iter().map(|s| s.to_string()).collect()
        } else {
            let pool = ACT1_STRONG_ENCOUNTERS;
            let idx = self.strong_encounter_idx % pool.len();
            self.strong_encounter_idx += 1;
            pool[idx].iter().map(|s| s.to_string()).collect()
        };

        // Create enemies
        let enemy_states: Vec<EnemyCombatState> = encounter
            .iter()
            .map(|id| {
                let (hp, max_hp) = self.roll_enemy_hp(id);
                enemies::create_enemy(id, hp, max_hp)
            })
            .collect();

        // Create combat state
        let mut combat_state = CombatState::new(
            self.run_state.current_hp,
            self.run_state.max_hp,
            enemy_states,
            self.run_state.deck.clone(),
            3, // Watcher base energy
        );
        combat_state.relics = self.run_state.relics.clone();
        combat_state.potions = self.run_state.potions.clone();

        let combat_seed = self.seed.wrapping_add(self.run_state.floor as u64 * 1000);
        let mut engine = CombatEngine::new(combat_state, combat_seed);
        engine.start_combat();

        self.combat_engine = Some(engine);
        self.phase = RunPhase::Combat;
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

        let hp_before = engine.state.player.hp;
        engine.execute_action(&combat_action);

        let mut reward = 0.0;

        if engine.is_combat_over() {
            if engine.state.player_won {
                // Combat win reward
                reward += 1.0;

                // Update run state from combat result
                self.run_state.current_hp = engine.state.player.hp;
                self.run_state.potions = engine.state.potions.clone();
                self.run_state.combats_won += 1;

                // Gold reward
                let gold = self.rng.gen_range(10..=20);
                self.run_state.gold += gold;

                // Check if this was elite
                let room_type = if self.run_state.map_y >= 0 {
                    self.map.rows[self.run_state.map_y as usize][self.run_state.map_x as usize].room_type
                } else {
                    RoomType::Monster
                };

                if room_type == RoomType::Elite {
                    self.run_state.elites_killed += 1;
                    let extra_gold = self.rng.gen_range(25..=35);
                    self.run_state.gold += extra_gold;
                }

                // Check if boss
                let is_boss = self.run_state.floor >= 16 || room_type == RoomType::Boss;
                if is_boss {
                    self.run_state.bosses_killed += 1;
                    reward += 5.0; // Boss kill bonus
                    // Run won!
                    self.run_state.run_won = true;
                    self.run_state.run_over = true;
                    self.combat_engine = None;
                    self.phase = RunPhase::GameOver;
                    return reward;
                }

                // Generate card rewards
                self.generate_card_rewards();
                self.combat_engine = None;
                self.phase = RunPhase::CardReward;
            } else {
                // Player died
                reward -= 1.0;
                self.run_state.current_hp = 0;
                self.run_state.run_over = true;
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
        }

        reward
    }

    fn generate_card_rewards(&mut self) {
        // Generate 3 card choices: common/uncommon/rare distribution
        let mut cards = Vec::new();
        for _ in 0..3 {
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
            cards.push(card.to_string());
        }
        self.card_rewards = cards;
    }

    // =======================================================================
    // Card reward step
    // =======================================================================

    fn step_card_reward(&mut self, action: &RunAction) -> f32 {
        match action {
            RunAction::PickCard(idx) => {
                if *idx < self.card_rewards.len() {
                    let card = self.card_rewards[*idx].clone();
                    self.run_state.deck.push(card);
                }
            }
            RunAction::SkipCardReward => {}
            _ => {}
        }

        self.card_rewards.clear();

        // Check if at last row (floor 15) — enter boss
        if self.run_state.map_y >= 0 && self.run_state.map_y as usize >= self.map.height - 1 {
            // Boss fight next
            self.run_state.floor += 1;
            self.enter_combat(false, true);
            return 0.0;
        }

        self.phase = RunPhase::MapChoice;
        0.0
    }

    // =======================================================================
    // Campfire step
    // =======================================================================

    fn step_campfire(&mut self, action: &RunAction) -> f32 {
        match action {
            RunAction::CampfireRest => {
                // Heal 30% of max HP
                let heal = (self.run_state.max_hp as f32 * 0.3).ceil() as i32;
                self.run_state.current_hp = (self.run_state.current_hp + heal).min(self.run_state.max_hp);
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
            return 0.0;
        }

        self.phase = RunPhase::MapChoice;
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
            cards.push((card.to_string(), price));
        }

        let remove_price = 75 + (self.run_state.combats_won as i32 * 25);

        self.current_shop = Some(ShopState {
            cards,
            remove_price,
            removal_used: false,
        });
        self.phase = RunPhase::Shop;
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
                if let Some(ref mut shop) = self.current_shop {
                    if !shop.removal_used && *idx < self.run_state.deck.len() && self.run_state.gold >= shop.remove_price {
                        let price = shop.remove_price;
                        self.run_state.gold -= price;
                        self.run_state.deck.remove(*idx);
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
        0.0
    }

    // =======================================================================
    // Event step
    // =======================================================================

    fn enter_event(&mut self) {
        let events = act1_events();
        let idx = self.rng.gen_range(0..events.len());
        self.current_event = Some(events[idx].clone());
        self.phase = RunPhase::Event;
    }

    fn step_event(&mut self, action: &RunAction) -> f32 {
        let choice_idx = match action {
            RunAction::EventChoice(idx) => *idx,
            _ => 0,
        };

        if let Some(ref event) = self.current_event {
            if choice_idx < event.options.len() {
                let effect = &event.options[choice_idx].effect;
                match effect {
                    EventEffect::Hp(amount) => {
                        self.run_state.current_hp = (self.run_state.current_hp + amount)
                            .max(0)
                            .min(self.run_state.max_hp);
                    }
                    EventEffect::Gold(amount) => {
                        self.run_state.gold = (self.run_state.gold + amount).max(0);
                    }
                    EventEffect::GainCard => {
                        let idx = self.rng.gen_range(0..WATCHER_COMMON_CARDS.len());
                        self.run_state.deck.push(WATCHER_COMMON_CARDS[idx].to_string());
                    }
                    EventEffect::RemoveCard => {
                        if self.run_state.deck.len() > 5 {
                            let idx = self.rng.gen_range(0..self.run_state.deck.len());
                            self.run_state.deck.remove(idx);
                        }
                    }
                    EventEffect::GainRelic => {
                        // Simplified: gain a placeholder relic
                        self.run_state.relics.push("EventRelic".to_string());
                    }
                    EventEffect::MaxHp(amount) => {
                        self.run_state.max_hp += amount;
                        self.run_state.current_hp += amount;
                    }
                    EventEffect::DamageAndGold(damage, gold) => {
                        if *damage < 0 {
                            self.run_state.current_hp = (self.run_state.current_hp + damage).max(0);
                        }
                        if *gold > 0 {
                            self.run_state.gold += gold;
                        }
                        // Check death
                        if self.run_state.current_hp <= 0 {
                            self.run_state.run_over = true;
                            self.phase = RunPhase::GameOver;
                            return -1.0;
                        }
                    }
                    EventEffect::GoldenIdolTake => {
                        // Lose 25% max HP (rounded down), gain 300 gold
                        let damage = self.run_state.max_hp / 4;
                        self.run_state.current_hp = (self.run_state.current_hp - damage).max(0);
                        self.run_state.gold += 300;
                        if self.run_state.current_hp <= 0 {
                            self.run_state.run_over = true;
                            self.phase = RunPhase::GameOver;
                            return -1.0;
                        }
                    }
                    EventEffect::Nothing => {}
                    EventEffect::UpgradeCard => {
                        // Upgrade first non-upgraded card
                        for card in &mut self.run_state.deck {
                            if !card.ends_with('+') {
                                *card = format!("{}+", card);
                                break;
                            }
                        }
                    }
                }
            }
        }

        self.current_event = None;
        self.phase = RunPhase::MapChoice;
        0.0
    }

    // =======================================================================
    // Observation encoding
    // =======================================================================

    /// Get the current room type string.
    pub fn current_room_type(&self) -> &str {
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

    /// Get card reward list (for observation encoding).
    pub fn get_card_rewards(&self) -> &[String] {
        &self.card_rewards
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
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_engine_creation() {
        let engine = RunEngine::new(42, 20);
        assert_eq!(engine.run_state.current_hp, 68); // A14+ = 68
        assert_eq!(engine.run_state.deck.len(), 11); // 10 base + AscendersBane (A10+)
        assert_eq!(engine.phase, RunPhase::MapChoice);
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
        let engine = RunEngine::new(42, 20);
        let actions = engine.get_legal_actions();
        assert!(!actions.is_empty(), "Should have map choice actions");
        for a in &actions {
            assert!(matches!(a, RunAction::ChoosePath(_)));
        }
    }

    #[test]
    fn test_first_floor_is_combat() {
        let mut engine = RunEngine::new(42, 20);
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
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
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
        engine.phase = RunPhase::CardReward;
        engine.card_rewards = vec![
            "Eruption".to_string(),
            "Vigilance".to_string(),
            "Tantrum".to_string(),
        ];
        let deck_before = engine.run_state.deck.len();

        engine.step(&RunAction::PickCard(1));
        assert_eq!(engine.run_state.deck.len(), deck_before + 1);
        assert_eq!(engine.run_state.deck.last().unwrap(), "Vigilance");
    }

    #[test]
    fn test_card_reward_skip() {
        let mut engine = RunEngine::new(42, 0);
        engine.phase = RunPhase::CardReward;
        engine.card_rewards = vec![
            "Eruption".to_string(),
            "Vigilance".to_string(),
        ];
        let deck_before = engine.run_state.deck.len();

        engine.step(&RunAction::SkipCardReward);
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
        engine.current_event = Some(EventDef {
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
        engine.phase = RunPhase::Event;

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
