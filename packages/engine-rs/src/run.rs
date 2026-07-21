//! Run state management for complete seeded simulations.
//!
//! Manages floor progression, deck building, card rewards, events,
//! shops, campfires, and combat via the existing CombatEngine.
//! Exposes one canonical `GameAction` and training-neutral `StepOutcome` API.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::OnceLock;

use crate::decision::{
    build_combat_context, build_shop_context, CampfireDecisionContext, ChestDecisionContext,
    CombatContext, DecisionContext, DecisionFrame, DecisionKind, DecisionStack, DecisionState,
    EventDecisionContext, EventOptionContext, MapDecisionContext, MapPathContext,
    NeowDecisionContext, NeowOptionContext, PotionSlotContext, RewardChoice, RewardChoiceFrame,
    RewardItem, RewardItemKind, RewardItemState, RewardKeyColor, RewardScreen, RewardScreenSource,
    TransitionDecisionContext,
};
use crate::enemies;
use crate::engine::CombatEngine;
use crate::gameplay::registry::global_registry as gameplay_registry;
use crate::gameplay::types::GameplayDomain;
use crate::map::{generate_map_with_rng_for_run, generate_the_ending_map, DungeonMap, RoomType};
use crate::state::{CombatState, EnemyCombatState};

// ---------------------------------------------------------------------------
// Canonical game action
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameAction {
    /// Choose one of the opening Neow options.
    ChooseNeowOption(usize),
    /// Choose a path on the map: index into available next nodes
    ChoosePath(usize),
    /// Open the generated chest in the current treasure room.
    OpenChest,
    /// Proceed from the treasure room without opening its chest.
    LeaveChest,
    /// Click or claim a reward item in the current ordered reward screen.
    SelectRewardItem(usize),
    /// Choose an option within the currently active reward item.
    ChooseRewardOption {
        item_index: usize,
        choice_index: usize,
    },
    /// Skip a skippable reward item.
    SkipRewardItem(usize),
    /// Proceed from an unordered reward screen, abandoning all remaining items.
    LeaveRewards,
    /// Resolve the current typed room/run continuation.
    Proceed,
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
    /// Campfire: claim the Ruby Key through Recall.
    CampfireRecall,
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
    /// Discard an occupied potion slot without using the potion.
    DiscardPotion(usize),
}

impl GameAction {
    /// Stable ordering for deterministic replay, search frontiers, and action
    /// serialization. This ordering is independent of enum layout.
    pub fn canonical_sort_key(&self) -> (u8, i32, i32, i32) {
        match self {
            Self::ChooseNeowOption(index) => (0, *index as i32, 0, 0),
            Self::ChoosePath(index) => (1, *index as i32, 0, 0),
            Self::OpenChest => (2, 0, 0, 0),
            Self::LeaveChest => (2, 1, 0, 0),
            Self::CombatAction(action) => match action {
                crate::actions::Action::PlayCard {
                    card_idx,
                    target_idx,
                } => (3, 0, *card_idx as i32, *target_idx),
                crate::actions::Action::UsePotion {
                    potion_idx,
                    target_idx,
                } => (3, 1, *potion_idx as i32, *target_idx),
                crate::actions::Action::Choose(index) => (3, 2, *index as i32, 0),
                crate::actions::Action::ConfirmSelection => (3, 3, 0, 0),
                crate::actions::Action::EndTurn => (3, 4, 0, 0),
            },
            Self::SelectRewardItem(index) => (4, *index as i32, 0, 0),
            Self::ChooseRewardOption {
                item_index,
                choice_index,
            } => (5, *item_index as i32, *choice_index as i32, 0),
            Self::SkipRewardItem(index) => (6, *index as i32, 0, 0),
            Self::LeaveRewards => (6, i32::MAX, 0, 0),
            Self::Proceed => (6, i32::MAX - 1, 0, 0),
            Self::CampfireRest => (7, 0, 0, 0),
            Self::CampfireUpgrade(index) => (7, 1, *index as i32, 0),
            Self::CampfireToke => (7, 2, 0, 0),
            Self::CampfireLift => (7, 3, 0, 0),
            Self::CampfireDig => (7, 4, 0, 0),
            Self::CampfireRecall => (7, 5, 0, 0),
            Self::ShopBuyCard(index) => (8, 0, *index as i32, 0),
            Self::ShopBuyRelic(index) => (8, 1, *index as i32, 0),
            Self::ShopBuyPotion(index) => (8, 2, *index as i32, 0),
            Self::ShopRemoveCard(index) => (8, 3, *index as i32, 0),
            Self::ShopLeave => (8, 4, 0, 0),
            Self::EventChoice(index) => (9, *index as i32, 0, 0),
            Self::UsePotion(index) => (10, 0, *index as i32, 0),
            Self::DiscardPotion(index) => (10, 1, *index as i32, 0),
        }
    }
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
    /// A generated treasure chest is waiting to be opened or left behind.
    Chest,
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
    /// A completed room is waiting for an explicit Proceed action.
    Transition,
    /// Run is over (win or loss)
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunContinuation {
    ActThreeSecondBoss,
    SpireHeart,
    NextAct,
    TrueVictory,
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
    "Consecrate",
    "BowlingBash",
    "FlyingSleeves",
    "Halt",
    "JustLucky",
    "FlurryOfBlows",
    "Protect",
    "ThirdEye",
    "Crescendo",
    "ClearTheMind",
    "EmptyBody",
    "SashWhip",
    "CutThroughFate",
    "FollowUp",
    "PathToVictory",
    "CrushJoints",
    "Evaluate",
    "Prostrate",
    "EmptyFist",
];

const WATCHER_UNCOMMON_CARDS: &[&str] = &[
    "WheelKick",
    "Vengeance",
    "Wireheading",
    "Sanctity",
    "TalkToTheHand",
    "BattleHymn",
    "Indignation",
    "WindmillStrike",
    "ForeignInfluence",
    "LikeWater",
    "Fasting2",
    "CarveReality",
    "Wallop",
    "WreathOfFlame",
    "Collect",
    "InnerPeace",
    "Adaptation",
    "DeceiveReality",
    "MentalFortress",
    "ReachHeaven",
    "FearNoEvil",
    "SandsOfTime",
    "WaveOfTheHand",
    "Study",
    "Meditate",
    "Perseverance",
    "Swivel",
    "Worship",
    "Conclude",
    "Tantrum",
    "Nirvana",
    "EmptyMind",
    "Weave",
    "SignatureMove",
    "Pray",
];

const WATCHER_RARE_CARDS: &[&str] = &[
    "DeusExMachina",
    "DevaForm",
    "SpiritShield",
    "Establishment",
    "Omniscience",
    "Wish",
    "Alpha",
    "Vault",
    "Scrawl",
    "LessonLearned",
    "Ragnarok",
    "Blasphemy",
    "Devotion",
    "Brilliance",
    "MasterReality",
    "ConjureBlade",
    "Judgement",
];

/// Java keeps both mutable working pools and source pools for the entire run.
/// Reward generation samples the working pools without removal; transforms
/// mutate the source pools and can therefore change later source-pool order.
/// Source: AbstractDungeon.initializeCardPools/srcTransformCard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CardPools {
    common: Vec<String>,
    uncommon: Vec<String>,
    rare: Vec<String>,
    colorless: Vec<String>,
    source_common: Vec<String>,
    source_uncommon: Vec<String>,
    source_rare: Vec<String>,
}

// AbstractDungeon.addColorlessCards traverses CardLibrary's Java 8 HashMap
// and adds each eligible card to the top. This is the resulting all-unlocked
// mutable colorlessCardPool order used by returnColorlessCard.
// Java: AbstractDungeon.java::addColorlessCards/returnColorlessCard.
const COLORLESS_CARD_POOL: &[&str] = &[
    "Dark Shackles",
    "Sadistic Nature",
    "PanicButton",
    "Trip",
    "Dramatic Entrance",
    "Impatience",
    "The Bomb",
    "Blind",
    "Bandage Up",
    "Secret Technique",
    "Deep Breath",
    "Violence",
    "Panache",
    "Secret Weapon",
    "Apotheosis",
    "Mayhem",
    "HandOfGreed",
    "Flash of Steel",
    "Forethought",
    "Enlightenment",
    "Purity",
    "Panacea",
    "Transmutation",
    "Chrysalis",
    "Discovery",
    "Finesse",
    "Magnetism",
    "Master of Strategy",
    "Good Instincts",
    "Swift Strike",
    "Jack Of All Trades",
    "Metamorphosis",
    "Mind Blast",
    "Thinking Ahead",
    "Madness",
];

impl CardPools {
    fn watcher_with_locked_cards(locked_cards: &[String]) -> Self {
        fn available(ids: &[&str], locked_cards: &[String]) -> Vec<String> {
            // Watcher.getCardPool delegates to CardLibrary.addPurpleCards,
            // which filters UnlockTracker locks before pool construction.
            // Java: Watcher.java:179-190; CardLibrary.java:1172-1179.
            ids.iter()
                .filter(|id| !locked_cards.iter().any(|locked| locked.as_str() == **id))
                .map(|id| (*id).to_string())
                .collect()
        }

        let common: Vec<String> = available(WATCHER_COMMON_CARDS, locked_cards);
        let uncommon: Vec<String> = available(WATCHER_UNCOMMON_CARDS, locked_cards);
        let rare: Vec<String> = available(WATCHER_RARE_CARDS, locked_cards);
        Self {
            // src*CardPool uses addToBottom, which inserts at index zero.
            source_common: common.iter().rev().cloned().collect(),
            source_uncommon: uncommon.iter().rev().cloned().collect(),
            source_rare: rare.iter().rev().cloned().collect(),
            common,
            uncommon,
            rare,
            colorless: COLORLESS_CARD_POOL
                .iter()
                .map(|id| (*id).to_string())
                .collect(),
        }
    }

    fn working(&self, rarity: EventCardRarity) -> &[String] {
        match rarity {
            EventCardRarity::Common => &self.common,
            EventCardRarity::Uncommon => &self.uncommon,
            EventCardRarity::Rare => &self.rare,
            _ => &[],
        }
    }
}

// Dungeon event pools preserve Java insertion order because event selection
// indexes the eligible copy directly. Regular and generic shrine pools are
// rebuilt on act entry; selected one-time shrines remain absent for the run.
// Java: AbstractDungeon.java::generateEvent/getEvent/getShrine,
// Exordium.java/TheCity.java/TheBeyond.java::initialize*List.
const ACT_ONE_EVENTS: &[&str] = &[
    "Big Fish",
    "The Cleric",
    "Dead Adventurer",
    "Golden Idol",
    "Golden Wing",
    "World of Goop",
    "Liars Game",
    "Living Wall",
    "Mushrooms",
    "Scrap Ooze",
    "Shining Light",
];
const ACT_TWO_EVENTS: &[&str] = &[
    "Addict",
    "Back to Basics",
    "Beggar",
    "Colosseum",
    "Cursed Tome",
    "Drug Dealer",
    "Forgotten Altar",
    "Ghosts",
    "Masked Bandits",
    "Nest",
    "The Library",
    "The Mausoleum",
    "Vampires",
];
const ACT_THREE_EVENTS: &[&str] = &[
    "Falling",
    "MindBloom",
    "The Moai Head",
    "Mysterious Sphere",
    "SensoryStone",
    "Tomb of Lord Red Mask",
    "Winding Halls",
];
const ACT_ONE_SHRINES: &[&str] = &[
    "Match and Keep!",
    "Golden Shrine",
    "Transmorgrifier",
    "Purifier",
    "Upgrade Shrine",
    "Wheel of Change",
];
const LATER_ACT_SHRINES: &[&str] = &[
    "Match and Keep!",
    "Wheel of Change",
    "Golden Shrine",
    "Transmorgrifier",
    "Purifier",
    "Upgrade Shrine",
];
const ONE_TIME_SHRINES: &[&str] = &[
    "Accursed Blacksmith",
    "Bonfire Elementals",
    "Designer",
    "Duplicator",
    "FaceTrader",
    "Fountain of Cleansing",
    "Knowing Skull",
    "Lab",
    "N'loth",
    "NoteForYourself",
    "SecretPortal",
    "The Joust",
    "WeMeetAgain",
    "The Woman in Blue",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EventPools {
    regular: Vec<String>,
    shrines: Vec<String>,
    one_time_shrines: Vec<String>,
}

impl EventPools {
    fn watcher(ascension: i32, highest_unlocked_ascension: i32, is_daily_run: bool) -> Self {
        let note_for_yourself_available = !is_daily_run
            && (ascension == 0 || (ascension < 15 && ascension < highest_unlocked_ascension));
        Self {
            regular: ACT_ONE_EVENTS.iter().map(|id| (*id).to_string()).collect(),
            shrines: ACT_ONE_SHRINES.iter().map(|id| (*id).to_string()).collect(),
            one_time_shrines: ONE_TIME_SHRINES
                .iter()
                .filter(|id| **id != "NoteForYourself" || note_for_yourself_available)
                .map(|id| (*id).to_string())
                .collect(),
        }
    }

    fn reset_for_act(&mut self, act: i32) {
        let regular = match act {
            2 => ACT_TWO_EVENTS,
            3 => ACT_THREE_EVENTS,
            _ => ACT_ONE_EVENTS,
        };
        let shrines = if act == 1 {
            ACT_ONE_SHRINES
        } else {
            LATER_ACT_SHRINES
        };
        self.regular = regular.iter().map(|id| (*id).to_string()).collect();
        self.shrines = shrines.iter().map(|id| (*id).to_string()).collect();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum RelicTier {
    Common,
    Uncommon,
    Rare,
    Shop,
    Boss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum ChestSize {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct GeneratedChest {
    size: ChestSize,
    gold_reward: bool,
    relic_tier: RelicTier,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum PendingChest {
    Treasure(GeneratedChest),
    Boss { relic_choices: Vec<String> },
}

impl PendingChest {
    fn size_label(&self) -> &'static str {
        match self {
            Self::Treasure(chest) => chest.size_label(),
            Self::Boss { .. } => "boss",
        }
    }
}

impl GeneratedChest {
    fn size_label(self) -> &'static str {
        match self.size {
            ChestSize::Small => "small",
            ChestSize::Medium => "medium",
            ChestSize::Large => "large",
        }
    }

    fn base_gold(self) -> i32 {
        match self.size {
            ChestSize::Small => 25,
            ChestSize::Medium => 50,
            ChestSize::Large => 75,
        }
    }
}

// RelicLibrary.populateRelicPool traverses Java 8 HashMaps, not constructor
// insertion order. These all-unlocked Watcher vectors are the exact resulting
// pre-shuffle order. Java: RelicLibrary.java::populateRelicPool.
const WATCHER_COMMON_RELICS: &[&str] = &[
    "Whetstone",
    "Boot",
    "Blood Vial",
    "MealTicket",
    "Pen Nib",
    "Akabeko",
    "Lantern",
    "Regal Pillow",
    "Bag of Preparation",
    "Ancient Tea Set",
    "Smiling Mask",
    "Potion Belt",
    "PreservedInsect",
    "Omamori",
    "MawBank",
    "Art of War",
    "Toy Ornithopter",
    "CeramicFish",
    "Vajra",
    "Centennial Puzzle",
    "Strawberry",
    "Happy Flower",
    "Oddly Smooth Stone",
    "War Paint",
    "Bronze Scales",
    "Juzu Bracelet",
    "Dream Catcher",
    "Nunchaku",
    "Tiny Chest",
    "Orichalcum",
    "Anchor",
    "Bag of Marbles",
    "Damaru",
];

const WATCHER_UNCOMMON_RELICS: &[&str] = &[
    "Bottled Tornado",
    "Sundial",
    "Kunai",
    "Pear",
    "Blue Candle",
    "Eternal Feather",
    "StrikeDummy",
    "Singing Bowl",
    "Matryoshka",
    "InkBottle",
    "The Courier",
    "Frozen Egg 2",
    "Ornamental Fan",
    "Bottled Lightning",
    "Gremlin Horn",
    "HornCleat",
    "Toxic Egg 2",
    "Letter Opener",
    "Question Card",
    "Bottled Flame",
    "Shuriken",
    "Molten Egg 2",
    "Meat on the Bone",
    "Darkstone Periapt",
    "Mummified Hand",
    "Pantograph",
    "White Beast Statue",
    "Mercury Hourglass",
    "Yang",
    "TeardropLocket",
];

const WATCHER_RARE_RELICS: &[&str] = &[
    "Ginger",
    "Old Coin",
    "Bird Faced Urn",
    "Unceasing Top",
    "Torii",
    "StoneCalendar",
    "Shovel",
    "WingedGreaves",
    "Thread and Needle",
    "Turnip",
    "Ice Cream",
    "Calipers",
    "Lizard Tail",
    "Prayer Wheel",
    "Girya",
    "Dead Branch",
    "Du-Vu Doll",
    "Pocketwatch",
    "Mango",
    "Incense Burner",
    "Gambling Chip",
    "Peace Pipe",
    "CaptainsWheel",
    "FossilizedHelix",
    "TungstenRod",
    "CloakClasp",
    "GoldenEye",
];

const WATCHER_SHOP_RELICS: &[&str] = &[
    "Sling",
    "HandDrill",
    "Toolbox",
    "Chemical X",
    "Lee's Waffle",
    "Orrery",
    "DollysMirror",
    "OrangePellets",
    "PrismaticShard",
    "ClockworkSouvenir",
    "Frozen Eye",
    "TheAbacus",
    "Medical Kit",
    "Cauldron",
    "Strange Spoon",
    "Membership Card",
    "Melange",
];

const WATCHER_BOSS_RELICS: &[&str] = &[
    "Fusion Hammer",
    "Velvet Choker",
    "Runic Dome",
    "SlaversCollar",
    "Snecko Eye",
    "Pandora's Box",
    "Cursed Key",
    "Busted Crown",
    "Ectoplasm",
    "Tiny House",
    "Sozu",
    "Philosopher's Stone",
    "Astrolabe",
    "Black Star",
    "SacredBark",
    "Empty Cage",
    "Runic Pyramid",
    "Calling Bell",
    "Coffee Dripper",
    "HolyWater",
    "VioletLotus",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PotionRarity {
    Common,
    Uncommon,
    Rare,
}

// PotionHelper.getPotions(WATCHER, false) order is gameplay-significant: both
// Neow and Tiny House index this list directly, while normal rewards repeatedly
// sample it after a separate rarity roll.
// Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
const WATCHER_POTIONS: &[(&str, PotionRarity)] = &[
    ("BottledMiracle", PotionRarity::Common),
    ("StancePotion", PotionRarity::Uncommon),
    ("Ambrosia", PotionRarity::Rare),
    ("Block Potion", PotionRarity::Common),
    ("Dexterity Potion", PotionRarity::Common),
    ("Energy Potion", PotionRarity::Common),
    ("Explosive Potion", PotionRarity::Common),
    ("Fire Potion", PotionRarity::Common),
    ("Strength Potion", PotionRarity::Common),
    ("Swift Potion", PotionRarity::Common),
    ("Weak Potion", PotionRarity::Common),
    ("FearPotion", PotionRarity::Common),
    ("AttackPotion", PotionRarity::Common),
    ("SkillPotion", PotionRarity::Common),
    ("PowerPotion", PotionRarity::Common),
    ("ColorlessPotion", PotionRarity::Common),
    ("SteroidPotion", PotionRarity::Common),
    ("SpeedPotion", PotionRarity::Common),
    ("BlessingOfTheForge", PotionRarity::Common),
    ("Regen Potion", PotionRarity::Uncommon),
    ("Ancient Potion", PotionRarity::Uncommon),
    ("LiquidBronze", PotionRarity::Uncommon),
    ("GamblersBrew", PotionRarity::Uncommon),
    ("EssenceOfSteel", PotionRarity::Uncommon),
    ("DuplicationPotion", PotionRarity::Uncommon),
    ("DistilledChaos", PotionRarity::Uncommon),
    ("LiquidMemories", PotionRarity::Uncommon),
    ("CultistPotion", PotionRarity::Rare),
    ("Fruit Juice", PotionRarity::Rare),
    ("SneckoOil", PotionRarity::Rare),
    ("FairyPotion", PotionRarity::Rare),
    ("SmokeBomb", PotionRarity::Rare),
    ("EntropicBrew", PotionRarity::Rare),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RelicPools {
    common: Vec<String>,
    uncommon: Vec<String>,
    rare: Vec<String>,
    shop: Vec<String>,
    boss: Vec<String>,
}

impl RelicPools {
    fn watcher_with_locked_relics(
        rng: &mut crate::seed::StsRandom,
        locked_relics: &[String],
    ) -> Self {
        fn shuffled(
            ids: &[&str],
            locked_relics: &[String],
            rng: &mut crate::seed::StsRandom,
        ) -> Vec<String> {
            // RelicLibrary.populateRelicPool checks UnlockTracker before
            // AbstractDungeon.initializeRelicList shuffles each tier.
            // Java: RelicLibrary.java:628-662; AbstractDungeon.java:1211-1231.
            let mut pool = ids
                .iter()
                .filter(|id| !locked_relics.iter().any(|locked| locked.as_str() == **id))
                .map(|id| (*id).to_string())
                .collect::<Vec<_>>();
            let seed = rng.random_long_unbounded();
            crate::seed::java_util_shuffle(&mut pool, seed);
            pool
        }
        Self {
            common: shuffled(WATCHER_COMMON_RELICS, locked_relics, rng),
            uncommon: shuffled(WATCHER_UNCOMMON_RELICS, locked_relics, rng),
            rare: shuffled(WATCHER_RARE_RELICS, locked_relics, rng),
            shop: shuffled(WATCHER_SHOP_RELICS, locked_relics, rng),
            boss: shuffled(WATCHER_BOSS_RELICS, locked_relics, rng),
        }
    }

    fn pool_mut(&mut self, tier: RelicTier) -> &mut Vec<String> {
        match tier {
            RelicTier::Common => &mut self.common,
            RelicTier::Uncommon => &mut self.uncommon,
            RelicTier::Rare => &mut self.rare,
            RelicTier::Shop => &mut self.shop,
            RelicTier::Boss => &mut self.boss,
        }
    }
}

// AbstractDungeon.addColorlessCards supplies these non-special cards to the
// merchant's fixed uncommon and rare colorless slots.
// Java: decompiled/java-src/com/megacrit/cardcrawl/shop/Merchant.java
const SHOP_COLORLESS_UNCOMMON_CARDS: &[&str] = &[
    "Bandage Up",
    "Blind",
    "Dark Shackles",
    "Deep Breath",
    "Discovery",
    "Dramatic Entrance",
    "Enlightenment",
    "Finesse",
    "Flash of Steel",
    "Forethought",
    "Good Instincts",
    "Impatience",
    "Jack Of All Trades",
    "Madness",
    "Mind Blast",
    "Panacea",
    "PanicButton",
    "Purity",
    "Swift Strike",
    "Trip",
];

const SHOP_COLORLESS_RARE_CARDS: &[&str] = &[
    "Apotheosis",
    "Chrysalis",
    "HandOfGreed",
    "Magnetism",
    "Master of Strategy",
    "Mayhem",
    "Metamorphosis",
    "Panache",
    "Sadistic Nature",
    "Secret Technique",
    "Secret Weapon",
    "The Bomb",
    "Thinking Ahead",
    "Transmutation",
    "Violence",
];

// CardLibrary.java::getCurse excludes Ascender's Bane, Necronomicurse,
// Curse of the Bell, and Pride from random curse generation.
const RANDOM_OBTAINABLE_CURSES: &[&str] = &[
    // Java 8 HashMap traversal order from CardLibrary.curses.entrySet().
    "Regret",
    "Injury",
    "Shame",
    "Parasite",
    "Normality",
    "Doubt",
    "Writhe",
    "Pain",
    "Decay",
    "Clumsy",
];

// ---------------------------------------------------------------------------
// Seeded encounter queues
// ---------------------------------------------------------------------------

type WeightedEncounter = (&'static str, f32);

const ACT1_WEAK_ENCOUNTERS: &[WeightedEncounter] = &[
    ("Cultist", 2.0),
    ("Jaw Worm", 2.0),
    ("2 Louse", 2.0),
    ("Small Slimes", 2.0),
];

const ACT1_STRONG_ENCOUNTERS: &[WeightedEncounter] = &[
    ("Blue Slaver", 2.0),
    ("Gremlin Gang", 1.0),
    ("Looter", 2.0),
    ("Large Slime", 2.0),
    ("Lots of Slimes", 1.0),
    ("Exordium Thugs", 1.5),
    ("Exordium Wildlife", 1.5),
    ("Red Slaver", 1.0),
    ("3 Louse", 2.0),
    ("2 Fungi Beasts", 2.0),
];

const ACT1_ELITE_ENCOUNTERS: &[WeightedEncounter] = &[
    ("Gremlin Nob", 1.0),
    ("Lagavulin", 1.0),
    ("3 Sentries", 1.0),
];

const ACT1_BOSSES: &[&str] = &["TheGuardian", "Hexaghost", "SlimeBoss"];

// ---------------------------------------------------------------------------
// Act 2 encounter pools
// ---------------------------------------------------------------------------

const ACT2_WEAK_ENCOUNTERS: &[WeightedEncounter] = &[
    ("Spheric Guardian", 2.0),
    ("Chosen", 2.0),
    ("Shell Parasite", 2.0),
    ("3 Byrds", 2.0),
    ("2 Thieves", 2.0),
];

const ACT2_STRONG_ENCOUNTERS: &[WeightedEncounter] = &[
    ("Chosen and Byrds", 2.0),
    ("Sentry and Sphere", 2.0),
    ("Snake Plant", 6.0),
    ("Snecko", 4.0),
    ("Centurion and Healer", 6.0),
    ("Cultist and Chosen", 3.0),
    ("3 Cultists", 3.0),
    ("Shelled Parasite and Fungi", 3.0),
];

const ACT2_ELITE_ENCOUNTERS: &[WeightedEncounter] = &[
    ("Gremlin Leader", 1.0),
    ("Slavers", 1.0),
    ("Book of Stabbing", 1.0),
];

const ACT2_BOSSES: &[&str] = &["BronzeAutomaton", "TheCollector", "TheChamp"];

// ---------------------------------------------------------------------------
// Act 3 encounter pools
// ---------------------------------------------------------------------------

const ACT3_WEAK_ENCOUNTERS: &[WeightedEncounter] =
    &[("3 Darklings", 2.0), ("Orb Walker", 2.0), ("3 Shapes", 2.0)];

const ACT3_STRONG_ENCOUNTERS: &[WeightedEncounter] = &[
    ("Spire Growth", 1.0),
    ("Transient", 1.0),
    ("4 Shapes", 1.0),
    ("Maw", 1.0),
    ("Sphere and 2 Shapes", 1.0),
    ("Jaw Worm Horde", 1.0),
    ("3 Darklings", 1.0),
    ("Writhing Mass", 1.0),
];

const ACT3_ELITE_ENCOUNTERS: &[WeightedEncounter] =
    &[("Giant Head", 2.0), ("Nemesis", 2.0), ("Reptomancer", 2.0)];

const ACT3_BOSSES: &[&str] = &["AwakenedOne", "TimeEater", "DonuAndDeca"];

fn boss_seen_id_for_encounter(id: &str) -> Option<BossSeenId> {
    match id {
        "TheGuardian" => Some(BossSeenId::Guardian),
        "Hexaghost" => Some(BossSeenId::Ghost),
        "SlimeBoss" => Some(BossSeenId::Slime),
        "TheChamp" => Some(BossSeenId::Champ),
        "BronzeAutomaton" => Some(BossSeenId::Automaton),
        "TheCollector" => Some(BossSeenId::Collector),
        "AwakenedOne" => Some(BossSeenId::Crow),
        "DonuAndDeca" => Some(BossSeenId::Donut),
        "TimeEater" => Some(BossSeenId::Wizard),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Act 4 encounters
// ---------------------------------------------------------------------------

const ACT4_ELITE_ENCOUNTER: &str = "Shield and Spear";

fn encounter_pools_for_act(
    act: i32,
) -> (
    &'static [WeightedEncounter],
    usize,
    &'static [WeightedEncounter],
    &'static [WeightedEncounter],
) {
    match act {
        2 => (
            ACT2_WEAK_ENCOUNTERS,
            2,
            ACT2_STRONG_ENCOUNTERS,
            ACT2_ELITE_ENCOUNTERS,
        ),
        3 => (
            ACT3_WEAK_ENCOUNTERS,
            2,
            ACT3_STRONG_ENCOUNTERS,
            ACT3_ELITE_ENCOUNTERS,
        ),
        _ => (
            ACT1_WEAK_ENCOUNTERS,
            3,
            ACT1_STRONG_ENCOUNTERS,
            ACT1_ELITE_ENCOUNTERS,
        ),
    }
}

fn roll_weighted_encounter(
    encounters: &[WeightedEncounter],
    rng: &mut crate::seed::StsRandom,
) -> &'static str {
    // MonsterInfo.normalizeWeights stable-sorts by weight before accumulating
    // normalized f32 weights. Equal-weight entries retain declaration order.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/MonsterInfo.java
    let mut sorted = encounters.to_vec();
    sorted.sort_by(|left, right| left.1.total_cmp(&right.1));
    let total = sorted.iter().fold(0.0_f32, |sum, entry| sum + entry.1);
    let roll = rng.random_f32();
    let mut current = 0.0_f32;
    for (name, weight) in sorted {
        current += weight / total;
        if roll < current {
            return name;
        }
    }
    "ERROR"
}

fn populate_encounter_queue(
    queue: &mut VecDeque<String>,
    encounters: &[WeightedEncounter],
    count: usize,
    elites: bool,
    rng: &mut crate::seed::StsRandom,
) {
    for _ in 0..count {
        loop {
            let candidate = roll_weighted_encounter(encounters, rng);
            let repeats_last = queue.back().is_some_and(|last| last == candidate);
            let repeats_two_back = !elites
                && queue.len() > 1
                && queue
                    .get(queue.len() - 2)
                    .is_some_and(|previous| previous == candidate);
            if !repeats_last && !repeats_two_back {
                queue.push_back(candidate.to_string());
                break;
            }
        }
    }
}

fn first_strong_exclusions(act: i32, previous: &str) -> &'static [&'static str] {
    match (act, previous) {
        (1, "Looter") => &["Exordium Thugs"],
        (1, "Blue Slaver") => &["Red Slaver", "Exordium Thugs"],
        (1, "2 Louse") => &["3 Louse"],
        (1, "Small Slimes") => &["Large Slime", "Lots of Slimes"],
        (2, "Spheric Guardian") => &["Sentry and Sphere"],
        (2, "3 Byrds") => &["Chosen and Byrds"],
        (2, "Chosen") => &["Chosen and Byrds", "Cultist and Chosen"],
        (3, "3 Darklings") => &["3 Darklings"],
        (3, "Orb Walker") => &["Orb Walker"],
        (3, "3 Shapes") => &["4 Shapes"],
        _ => &[],
    }
}

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
// Profile input and cross-run outputs
// ---------------------------------------------------------------------------

// UnlockTracker.refresh registers these IDs as locked when their preference is
// absent or not fully unlocked. Keep the source order for recorder diagnostics.
// Java: decompiled/java-src/com/megacrit/cardcrawl/unlock/UnlockTracker.java:173-240.
const JAVA_FRESH_LOCKED_CARDS: &[&str] = &[
    "Havoc",
    "Sentinel",
    "Exhume",
    "Wild Strike",
    "Evolve",
    "Immolate",
    "Heavy Blade",
    "Spot Weakness",
    "Limit Break",
    "Concentrate",
    "Setup",
    "Grand Finale",
    "Cloak And Dagger",
    "Accuracy",
    "Storm of Steel",
    "Bane",
    "Catalyst",
    "Corpse Explosion",
    "Rebound",
    "Undo",
    "Echo Form",
    "Turbo",
    "Sunder",
    "Meteor Strike",
    "Hyperbeam",
    "Recycle",
    "Core Surge",
    "Prostrate",
    "Blasphemy",
    "Devotion",
    "ForeignInfluence",
    "Alpha",
    "MentalFortress",
    "SpiritShield",
    "Wish",
    "Wireheading",
];

const JAVA_FRESH_LOCKED_RELICS: &[&str] = &[
    "Omamori",
    "Prayer Wheel",
    "Shovel",
    "Art of War",
    "The Courier",
    "Pandora's Box",
    "Blue Candle",
    "Dead Branch",
    "Singing Bowl",
    "Du-Vu Doll",
    "Smiling Mask",
    "Tiny Chest",
    "Cables",
    "DataDisk",
    "Emotion Chip",
    "Runic Capacitor",
    "Turnip",
    "Symbiotic Virus",
    "Akabeko",
    "Yang",
    "CeramicFish",
    "StrikeDummy",
    "TeardropLocket",
    "CloakClasp",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    pub note_for_yourself_card: String,
    /// NoteForYourself.java compares the current ascension with this profile
    /// value when deciding whether the one-time shrine enters the run pool.
    #[serde(default = "default_highest_unlocked_ascension")]
    pub highest_unlocked_ascension: i32,
    #[serde(default)]
    pub is_daily_run: bool,
    /// Profile unlock controlling whether the three-key/Act 4 route exists.
    #[serde(default = "default_final_act_available")]
    pub final_act_available: bool,
    /// Bosses already encountered by this profile. Dungeon constructors use
    /// this to select the first unseen boss without consuming monster RNG.
    #[serde(default = "BossSeenSnapshot::all_seen")]
    pub bosses_seen: BossSeenSnapshot,
    /// Authoritative `UnlockTracker.lockedCards` snapshot. Standard Watcher
    /// colored pools exclude these IDs during dungeon initialization.
    #[serde(default)]
    pub locked_cards: Vec<String>,
    /// Authoritative `UnlockTracker.lockedRelics` snapshot. Java excludes
    /// these IDs while populating each relic tier, before shuffling the pool.
    /// The default preserves the simulator's historical all-unlocked profile.
    #[serde(default)]
    pub locked_relics: Vec<String>,
}

/// Java preference keys used by the three standard dungeon constructors.
///
/// Java: Exordium.java, TheCity.java, and TheBeyond.java `initializeBoss`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BossSeenId {
    Guardian,
    Ghost,
    Slime,
    Champ,
    Automaton,
    Collector,
    Crow,
    Donut,
    Wizard,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BossSeenSnapshot {
    seen: Vec<BossSeenId>,
}

impl BossSeenSnapshot {
    pub fn fresh() -> Self {
        Self { seen: Vec::new() }
    }

    pub fn all_seen() -> Self {
        Self {
            seen: vec![
                BossSeenId::Guardian,
                BossSeenId::Ghost,
                BossSeenId::Slime,
                BossSeenId::Champ,
                BossSeenId::Automaton,
                BossSeenId::Collector,
                BossSeenId::Crow,
                BossSeenId::Donut,
                BossSeenId::Wizard,
            ],
        }
    }

    pub fn contains(&self, boss: BossSeenId) -> bool {
        self.seen.contains(&boss)
    }

    fn insert(&mut self, boss: BossSeenId) {
        if !self.seen.contains(&boss) {
            self.seen.push(boss);
            self.seen.sort_unstable();
        }
    }
}

impl Default for BossSeenSnapshot {
    fn default() -> Self {
        Self::all_seen()
    }
}

fn default_highest_unlocked_ascension() -> i32 {
    20
}
fn default_final_act_available() -> bool {
    true
}

impl Default for ProfileSnapshot {
    fn default() -> Self {
        Self::all_unlocked()
    }
}

impl ProfileSnapshot {
    /// Simulator-friendly profile with the full content catalog unlocked and
    /// all standard bosses seen. Callers reproducing a fresh Java profile must
    /// use `fresh()` instead of relying on an implicit assumption.
    pub fn all_unlocked() -> Self {
        Self {
            note_for_yourself_card: "IronWave".to_string(),
            highest_unlocked_ascension: default_highest_unlocked_ascension(),
            is_daily_run: false,
            final_act_available: default_final_act_available(),
            bosses_seen: BossSeenSnapshot::all_seen(),
            locked_cards: Vec::new(),
            locked_relics: Vec::new(),
        }
    }

    /// New-profile inputs matching the lock lists seeded by
    /// `UnlockTracker.refresh()` when no unlock preferences are present.
    pub fn fresh() -> Self {
        Self {
            note_for_yourself_card: "IronWave".to_string(),
            highest_unlocked_ascension: 0,
            is_daily_run: false,
            final_act_available: false,
            bosses_seen: BossSeenSnapshot::fresh(),
            locked_cards: JAVA_FRESH_LOCKED_CARDS
                .iter()
                .map(|id| (*id).to_string())
                .collect(),
            locked_relics: JAVA_FRESH_LOCKED_RELICS
                .iter()
                .map(|id| (*id).to_string())
                .collect(),
        }
    }

    pub fn with_note_for_yourself_card(card_id: impl Into<String>) -> Self {
        Self {
            note_for_yourself_card: card_id.into(),
            highest_unlocked_ascension: default_highest_unlocked_ascension(),
            is_daily_run: false,
            final_act_available: default_final_act_available(),
            bosses_seen: BossSeenSnapshot::all_seen(),
            locked_cards: Vec::new(),
            locked_relics: Vec::new(),
        }
    }

    pub fn apply_update(&mut self, update: &ProfileUpdate) {
        match update {
            ProfileUpdate::StoreNoteForYourselfCard { card_id } => {
                self.note_for_yourself_card = card_id.clone();
            }
            ProfileUpdate::MarkBossSeen { boss } => self.bosses_seen.insert(*boss),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileUpdate {
    StoreNoteForYourselfCard { card_id: String },
    MarkBossSeen { boss: BossSeenId },
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
    /// Java gates Secret Portal on CardCrawlGame.playtime >= 800 seconds.
    /// Simulation callers may advance or inject this external clock explicitly.
    #[serde(default)]
    pub playtime_seconds: f32,
    pub deck: Vec<String>,
    /// Per-card persistent state aligned with `deck`. The string deck remains
    /// the public/run-action surface; this snapshot carries Java CardSave.misc
    /// across combats for cards such as Genetic Algorithm.
    #[serde(default)]
    pub deck_card_states: Vec<crate::combat_types::CardInstance>,
    /// Next deterministic identity for a newly obtained master-deck card.
    #[serde(default = "default_next_card_instance_id")]
    pub next_card_instance_id: u64,
    pub relics: Vec<String>,
    /// Empty strings represent Java's inert PotionSlot placeholder objects.
    /// Source: reference/extracted/methods/potion/PotionSlot.java.
    pub potions: Vec<String>,
    pub max_potions: usize,
    #[serde(default)]
    pub bottled_flame_card_instance_id: Option<u32>,
    #[serde(default)]
    pub bottled_lightning_card_instance_id: Option<u32>,
    #[serde(default)]
    pub bottled_tornado_card_instance_id: Option<u32>,

    // Map state
    pub map_x: i32, // Java's synthetic pre-map node is (0, -1)
    pub map_y: i32, // -1 before first move

    // Keys
    pub has_ruby_key: bool,
    pub has_emerald_key: bool,
    pub has_sapphire_key: bool,

    // Stats
    pub combats_won: i32,
    pub elites_killed: i32,
    pub bosses_killed: i32,
    /// AbstractDungeon.cardBlizzRandomizer persists across reward screens and
    /// acts, resets to 5 after a rare, and bottoms out at -40 after commons.
    #[serde(default = "default_card_blizz_randomizer")]
    pub card_blizz_randomizer: i32,
    /// AbstractRoom.blizzardPotionMod persists within an act, moving by ten
    /// after every potion reward success/failure, and resets on act entry.
    #[serde(default)]
    pub potion_blizz_randomizer: i32,
    #[serde(default = "default_purge_cost")]
    pub purge_cost: i32,

    // Run outcome
    pub run_won: bool,
    pub run_over: bool,

    // Relic presence bits and cross-combat counters. Presence bits are
    // rebuilt after checkpoint restore, while counters are causal state.
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

fn default_next_card_instance_id() -> u64 {
    1
}

fn take_run_card_instance_id(next: &mut u64) -> u32 {
    let current = (*next).max(1);
    let instance_id = u32::try_from(current).expect("card instance identity space exhausted");
    *next = current + 1;
    instance_id
}

fn default_card_blizz_randomizer() -> i32 {
    5
}

fn default_event_monster_chance() -> i32 {
    10
}
fn default_event_shop_chance() -> i32 {
    3
}
fn default_event_treasure_chance() -> i32 {
    2
}

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
        if run_state
            .relic_flags
            .has(crate::relic_flags::flag::MAW_BANK)
        {
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
        && run_state.relic_flags.has(crate::relic_flags::flag::OMAMORI)
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
    card_state.instance_id = run_state.allocate_card_instance_id();
    if extra_searing_upgrade {
        registry.upgrade_card(&mut card_state);
    }
    run_state.deck_card_states.push(card_state);
    apply_master_deck_obtain_hooks(run_state, is_curse);
}

fn apply_master_deck_obtain_hooks(run_state: &mut RunState, is_curse: bool) {
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

fn obtain_master_deck_stat_copy(
    run_state: &mut RunState,
    mut source: crate::combat_types::CardInstance,
) {
    let registry = crate::cards::global_registry();
    let is_curse =
        registry.card_def_by_id(source.def_id).card_type == crate::cards::CardType::Curse;
    if is_curse
        && run_state.relic_flags.has(crate::relic_flags::flag::OMAMORI)
        && run_state.relic_flags.counters[crate::relic_flags::counter::OMAMORI_USES] > 0
    {
        run_state.relic_flags.counters[crate::relic_flags::counter::OMAMORI_USES] -= 1;
        return;
    }
    let should_upgrade = match registry.card_def_by_id(source.def_id).card_type {
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
    if should_upgrade && registry.can_upgrade_card(&source) {
        registry.upgrade_card(&mut source);
    }
    source.instance_id = run_state.allocate_card_instance_id();
    source.flags &= crate::combat_types::CardInstance::FLAG_UPGRADED
        | crate::combat_types::CardInstance::FLAG_INNATE;
    run_state
        .deck
        .push(registry.card_name(source.def_id).to_string());
    run_state.deck_card_states.push(source);
    apply_master_deck_obtain_hooks(run_state, is_curse);
}

impl RunState {
    pub fn new(ascension: i32) -> Self {
        // Watcher keeps Java's class-specific starter IDs in persistent and
        // combat state. Generic Strike/Defend remain registry aliases only.
        // Java: Strike_Purple.java::ID, Defend_Watcher.java::ID.
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

        let registry = crate::cards::global_registry();
        let deck_card_states = deck
            .iter()
            .enumerate()
            .map(|(index, card_id)| {
                registry
                    .make_card(card_id)
                    .with_instance_id(index as u32 + 1)
            })
            .collect();
        let next_card_instance_id = deck.len() as u64 + 1;

        Self {
            current_hp: max_hp,
            max_hp,
            gold: 99,
            floor: 0,
            act: 1,
            ascension,
            playtime_seconds: 0.0,
            deck,
            deck_card_states,
            next_card_instance_id,
            relics,
            potions: vec!["".to_string(); 3],
            max_potions: 3,
            bottled_flame_card_instance_id: None,
            bottled_lightning_card_instance_id: None,
            bottled_tornado_card_instance_id: None,
            // Exordium installs `new MapRoomNode(0, -1)` before Neow.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/Exordium.java:59
            map_x: 0,
            map_y: -1,
            has_ruby_key: false,
            has_emerald_key: false,
            has_sapphire_key: false,
            combats_won: 0,
            elites_killed: 0,
            bosses_killed: 0,
            card_blizz_randomizer: default_card_blizz_randomizer(),
            potion_blizz_randomizer: 0,
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
        let mut next_instance_id = self.next_card_instance_id.max(1);

        for card_id in &self.deck {
            let mut card = registry.make_card(card_id);
            let base_id = card_id.trim_end_matches('+');
            if let Some(index) = prior.iter().position(|candidate| {
                candidate.def_id != u16::MAX
                    && registry.card_name(candidate.def_id).trim_end_matches('+') == base_id
            }) {
                // Upgrade operations keep misc; transforms construct a fresh
                // card and therefore do not match by base ID.
                let prior_card = prior.remove(index);
                card.instance_id = prior_card.instance_id;
                card.misc = prior_card.misc;
            }
            if card.instance_id == 0 {
                card.instance_id = take_run_card_instance_id(&mut next_instance_id);
            }
            reconciled.push(card);
        }

        self.deck_card_states = reconciled;
        self.next_card_instance_id = next_instance_id;
    }

    fn allocate_card_instance_id(&mut self) -> u32 {
        take_run_card_instance_id(&mut self.next_card_instance_id)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionStatus {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunOutcome {
    Victory,
    Defeat,
}

#[derive(Debug, Clone)]
pub struct DecisionSnapshot {
    pub state: DecisionState,
    pub context: DecisionContext,
    pub legal_actions: Vec<GameAction>,
}

/// Training-neutral result of one canonical simulator action.
#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub status: ActionStatus,
    pub terminal: Option<RunOutcome>,
    pub next_decision: DecisionSnapshot,
    pub events: Vec<crate::effects::runtime::GameEventRecord>,
}

impl StepOutcome {
    pub fn accepted(&self) -> bool {
        self.status == ActionStatus::Accepted
    }

    pub fn is_terminal(&self) -> bool {
        self.terminal.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PendingEventCombat {
    enemies: Vec<String>,
    on_win: crate::events::EventProgram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum MatchAndKeepScreen {
    Intro,
    RuleExplanation,
    Play,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MatchAndKeepState {
    screen: MatchAndKeepScreen,
    board: Vec<String>,
    first_pick: Option<usize>,
    attempts_left: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ScrapOozeState {
    damage: i32,
    relic_chance: usize,
    leave_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum SpireHeartScreen {
    Intro,
    Middle,
    Middle2,
    Death,
    GoToEnding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct SpireHeartState {
    screen: SpireHeartScreen,
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
enum CardRewardContext {
    Standard,
    Elite,
    Rest,
    Shop,
    Boss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum EncounterQueueKind {
    Hallway,
    Elite,
}

impl CardRewardContext {
    fn base_chances(self) -> (i32, i32) {
        match self {
            Self::Standard | Self::Rest => (3, 37),
            Self::Elite => (10, 40),
            Self::Shop => (9, 37),
            Self::Boss => (100, 0),
        }
    }

    fn applies_relic_rarity_modifiers(self) -> bool {
        !matches!(self, Self::Rest | Self::Shop | Self::Boss)
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum NeowReward {
    RandomColorlessRare,
    ThreeCards,
    OneRandomRareCard,
    RemoveCard,
    UpgradeCard,
    RandomColorless,
    TransformCard,
    ThreePotions,
    RandomCommonRelic,
    TenPercentHpBonus,
    HundredGold,
    NeowsLament,
    RemoveTwo,
    TransformTwo,
    OneRareRelic,
    ThreeRareCards,
    TwoFiftyGold,
    TwentyPercentHpBonus,
    BossRelic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum NeowDrawback {
    None,
    TenPercentHpLoss,
    NoGold,
    Curse,
    PercentDamage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum NeowDeckChoice {
    Remove,
    Upgrade,
    Transform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct PendingNeowDeckSelection {
    choice: NeowDeckChoice,
    remaining: usize,
    skip_allowed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum NeowFrame {
    Intro,
    Choice,
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PendingEventDeckSelection {
    label: String,
    remaining: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct NeowChoiceOption {
    category: u8,
    label: String,
    reward: NeowReward,
    drawback: NeowDrawback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct NeowSelectionWitness {
    options: Vec<NeowChoiceOption>,
    selected_index: usize,
}

impl NeowReward {
    fn java_id(self) -> &'static str {
        match self {
            Self::RandomColorlessRare => "RANDOM_COLORLESS_2",
            Self::ThreeCards => "THREE_CARDS",
            Self::OneRandomRareCard => "ONE_RANDOM_RARE_CARD",
            Self::RemoveCard => "REMOVE_CARD",
            Self::UpgradeCard => "UPGRADE_CARD",
            Self::RandomColorless => "RANDOM_COLORLESS",
            Self::TransformCard => "TRANSFORM_CARD",
            Self::ThreePotions => "THREE_SMALL_POTIONS",
            Self::RandomCommonRelic => "RANDOM_COMMON_RELIC",
            Self::TenPercentHpBonus => "TEN_PERCENT_HP_BONUS",
            Self::HundredGold => "HUNDRED_GOLD",
            Self::NeowsLament => "THREE_ENEMY_KILL",
            Self::RemoveTwo => "REMOVE_TWO",
            Self::TransformTwo => "TRANSFORM_TWO_CARDS",
            Self::OneRareRelic => "ONE_RARE_RELIC",
            Self::ThreeRareCards => "THREE_RARE_CARDS",
            Self::TwoFiftyGold => "TWO_FIFTY_GOLD",
            Self::TwentyPercentHpBonus => "TWENTY_PERCENT_HP_BONUS",
            Self::BossRelic => "BOSS_RELIC",
        }
    }
}

impl NeowDrawback {
    fn java_id(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::TenPercentHpLoss => "TEN_PERCENT_HP_LOSS",
            Self::NoGold => "NO_GOLD",
            Self::Curse => "CURSE",
            Self::PercentDamage => "PERCENT_DAMAGE",
        }
    }
}

// ---------------------------------------------------------------------------
// RunEngine — the full run simulation engine
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize, Deserialize)]
pub struct RunEngine {
    pub run_state: RunState,
    pub map: DungeonMap,
    pub phase: RunPhase,
    pub seed: u64,
    profile: ProfileSnapshot,
    profile_updates: Vec<ProfileUpdate>,
    persistent_rngs: crate::seed::PersistentRngs,
    floor_rngs: crate::seed::FloorRngs,
    ambient_math_rng: crate::seed::AmbientMathRng,
    java_collections_rng: crate::seed::JavaCollectionsRng,
    neow_rng: crate::seed::StsRandom,
    map_rng: crate::seed::StsRandom,
    card_pools: CardPools,
    relic_pools: RelicPools,
    event_pools: EventPools,

    // Active combat (when in Combat phase)
    combat_engine: Option<CombatEngine>,

    // TreasureRoom creates the chest on entry, but opening is a separate
    // player action. Hidden reward details remain internal until then.
    pending_chest: Option<PendingChest>,

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
    pending_neow_deck_selection: Option<PendingNeowDeckSelection>,
    pending_event_deck_selection: Option<PendingEventDeckSelection>,
    pending_relic_followup_source: RewardScreenSource,
    pending_continuation: Option<RunContinuation>,

    // Canonical stack of active decisions.
    decision_stack: DecisionStack,

    // Current event (when in Event phase)
    current_event: Option<TypedEventDef>,

    // Scripted event fight that resumes into an event-owned on-win program.
    pending_event_combat: Option<PendingEventCombat>,

    // Dynamic event runtime sidecars for stateful event flows.
    match_and_keep_state: Option<MatchAndKeepState>,
    scrap_ooze_state: Option<ScrapOozeState>,
    spire_heart_state: Option<SpireHeartState>,
    forced_event_rolls: Vec<usize>,

    // Current shop (when in Shop phase)
    current_shop: Option<ShopState>,

    // Boss for this act
    boss_id: String,
    boss_sequence: VecDeque<String>,
    // EventRoom replaces itself with the rolled concrete room before combat.
    // Keep that resolved identity with the combat so reward and checkpoint
    // behavior never has to re-read the original map symbol.
    active_combat_room_type: Option<RoomType>,
    active_combat_has_emerald_key: bool,

    // Java generates seeded encounter-key queues once per act, then consumes
    // them from the front as rooms are entered.
    encounter_queue_act: i32,
    monster_encounter_queue: VecDeque<String>,
    elite_encounter_queue: VecDeque<String>,
    active_encounter_queue: Option<EncounterQueueKind>,

    // Reward tracking
    #[serde(skip, default)]
    last_combat_events: Vec<crate::effects::runtime::GameEventRecord>,

    // Opening Neow choice before the map starts.
    neow_frame: Option<NeowFrame>,
    neow_options: Vec<NeowChoiceOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    neow_selection_witness: Option<NeowSelectionWitness>,
}

impl RunEngine {
    fn compute_shop_remove_price(&self) -> i32 {
        // ShopScreen.java assigns Smiling Mask's flat cost after all other
        // purge-price discounts, so it always wins.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/shop/ShopScreen.java
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::SMILING_MASK)
        {
            return 50;
        }

        let mut remove_price = self.run_state.purge_cost;
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::THE_COURIER)
        {
            remove_price = ((remove_price as f32) * 0.8).round() as i32;
        }
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::MEMBERSHIP_CARD)
        {
            remove_price = ((remove_price as f32) * 0.5).round() as i32;
        }
        remove_price
    }

    /// Create a new run engine with given seed and ascension level.
    pub fn new(seed: u64, ascension: i32) -> Self {
        Self::new_with_ambient_seed(seed, ascension, 0)
    }

    /// Create a deterministic simulator run with an explicit state-local
    /// libGDX `MathUtils.random` seed. Java initializes this ambient stream
    /// from process time, not from the run seed, so callers that need exact
    /// oracle replay must inject the captured state/outcome instead.
    pub fn new_with_ambient_seed(seed: u64, ascension: i32, ambient_seed: u64) -> Self {
        Self::new_with_profile_and_ambient_seed(
            seed,
            ascension,
            ProfileSnapshot::default(),
            ambient_seed,
        )
    }

    /// Create a run from both process-global RNG states that Java does not
    /// derive from the dungeon seed. Oracle replay must use this constructor
    /// when the recorder supplies these initial conditions.
    pub fn new_with_ambient_states(
        seed: u64,
        ascension: i32,
        ambient_math_state: (u64, u64),
        java_collections_state: u64,
    ) -> Self {
        let mut engine = Self::new(seed, ascension);
        engine.ambient_math_rng =
            crate::seed::AmbientMathRng::from_state(ambient_math_state.0, ambient_math_state.1);
        engine
            .java_collections_rng
            .restore_state(java_collections_state);
        engine
    }

    pub fn ambient_math_rng_state(&self) -> (u64, u64) {
        self.combat_engine.as_ref().map_or_else(
            || self.ambient_math_rng.state_tuple(),
            CombatEngine::ambient_math_rng_state,
        )
    }

    pub fn restore_ambient_math_rng_state(&mut self, ambient_state: (u64, u64)) {
        self.ambient_math_rng
            .restore_state(ambient_state.0, ambient_state.1);
        if let Some(combat) = self.combat_engine.as_mut() {
            combat.restore_ambient_math_rng_state(ambient_state);
        }
    }

    pub fn java_collections_rng_state(&self) -> u64 {
        self.combat_engine.as_ref().map_or_else(
            || self.java_collections_rng.state(),
            CombatEngine::java_collections_rng_state,
        )
    }

    pub fn restore_java_collections_rng_state(&mut self, state: u64) {
        self.java_collections_rng.restore_state(state);
        if let Some(combat) = self.combat_engine.as_mut() {
            combat.restore_java_collections_rng_state(state);
        }
    }

    /// Rebuild indexes that are deliberately excluded from causal checkpoint
    /// data. The relic bitset and decision stack are projections of canonical
    /// inventory/phase state; event records are telemetry and never affect a
    /// future action.
    pub(crate) fn rebuild_checkpoint_projections(&mut self) {
        self.run_state.relic_flags.rebuild(&self.run_state.relics);
        self.last_combat_events.clear();
        self.refresh_decision_stack();
    }

    pub(crate) fn validate_checkpoint_state(&self) -> Result<(), String> {
        if let Some(combat) = self.combat_engine.as_ref() {
            combat.validate_checkpoint_boundary()?;
        }
        if (self.phase == RunPhase::Combat) != self.combat_engine.is_some() {
            return Err("combat phase and active combat payload disagree".to_string());
        }
        if let Some(witness) = self.neow_selection_witness.as_ref() {
            if witness.selected_index >= witness.options.len() {
                return Err("Neow selection witness index is out of bounds".to_string());
            }
        }
        match (self.phase, self.neow_frame) {
            (RunPhase::Neow, Some(NeowFrame::Intro)) => {
                if !self.neow_options.is_empty() || self.neow_selection_witness.is_some() {
                    return Err("Neow intro frame must not expose generated options".to_string());
                }
            }
            (RunPhase::Neow, Some(NeowFrame::Choice)) => {
                if self.neow_options.len() != 4 || self.neow_selection_witness.is_some() {
                    return Err(
                        "Neow choice frame must expose exactly four unselected options".to_string(),
                    );
                }
            }
            (RunPhase::Neow, Some(NeowFrame::Exit)) => {
                if !self.neow_options.is_empty()
                    || self.neow_selection_witness.is_none()
                    || self.reward_screen.is_some()
                    || self.pending_neow_deck_selection.is_some()
                {
                    return Err("Neow exit frame has inconsistent pending state".to_string());
                }
            }
            (RunPhase::Neow, None) => {
                return Err("Neow phase is missing its decision frame".to_string());
            }
            (RunPhase::CardReward, Some(NeowFrame::Exit)) => {
                if !self.neow_options.is_empty()
                    || self.neow_selection_witness.is_none()
                    || self.reward_screen.is_none()
                {
                    return Err("Neow reward overlay has inconsistent pending state".to_string());
                }
            }
            // Test/support entry points can replace the opening room directly.
            // An unselected frame is then inert; selected Neow state is not.
            (_, Some(_)) if self.neow_selection_witness.is_some() => {
                return Err("Neow selection state exists outside the opening flow".to_string());
            }
            (_, Some(_)) => {}
            (_, None) if self.neow_selection_witness.is_some() => {
                return Err("Neow selection witness exists outside the opening flow".to_string());
            }
            (_, None) => {}
        }
        match self.phase {
            RunPhase::Chest if self.pending_chest.is_none() => {
                return Err("chest phase is missing its pending chest".to_string());
            }
            RunPhase::CardReward if self.reward_screen.is_none() => {
                return Err("reward phase is missing its reward screen".to_string());
            }
            RunPhase::Shop if self.current_shop.is_none() => {
                return Err("shop phase is missing its shop inventory".to_string());
            }
            RunPhase::Event if self.current_event.is_none() => {
                return Err("event phase is missing its event definition".to_string());
            }
            RunPhase::Transition if self.pending_continuation.is_none() => {
                return Err("transition phase is missing its continuation".to_string());
            }
            RunPhase::GameOver if !self.run_state.run_over => {
                return Err("game-over phase is missing its terminal run state".to_string());
            }
            _ => {}
        }
        let current_event_name = self.current_event.as_ref().map(|event| event.name.as_str());
        if self.phase == RunPhase::Event {
            if (current_event_name == Some("Match and Keep!"))
                != self.match_and_keep_state.is_some()
            {
                return Err("Match and Keep event and runtime sidecar disagree".to_string());
            }
            if (current_event_name == Some("Scrap Ooze")) != self.scrap_ooze_state.is_some() {
                return Err("Scrap Ooze event and runtime sidecar disagree".to_string());
            }
            if (current_event_name == Some("Spire Heart")) != self.spire_heart_state.is_some() {
                return Err("Spire Heart event and runtime sidecar disagree".to_string());
            }
        } else if self.match_and_keep_state.is_some()
            || self.scrap_ooze_state.is_some()
            || self.spire_heart_state.is_some()
        {
            return Err("stateful event sidecar exists outside an event phase".to_string());
        }
        if let Some(choice) = self.decision_stack.current_reward_choice() {
            if self.phase != RunPhase::CardReward {
                return Err("reward choice exists outside a reward phase".to_string());
            }
            let screen = self
                .reward_screen
                .as_ref()
                .ok_or_else(|| "reward choice is missing its reward screen".to_string())?;
            let item = screen
                .items
                .get(choice.item_index)
                .ok_or_else(|| "reward choice item index is invalid".to_string())?;
            let mut expected_choices = item.choices.clone();
            if item.kind == RewardItemKind::CardChoice
                && self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::SINGING_BOWL)
            {
                expected_choices.push(RewardChoice::Named {
                    index: expected_choices.len(),
                    label: "Singing Bowl".to_string(),
                });
            }
            if item.index != choice.item_index
                || !item.claimable
                || item.state != RewardItemState::Available
                || item.kind != choice.item_kind
                || item.skip_allowed != choice.skip_allowed
                || expected_choices != choice.choices
            {
                return Err("reward choice is not backed by its reward-screen item".to_string());
            }
        }
        if self.run_state.deck.len() != self.run_state.deck_card_states.len() {
            return Err("master-deck names and instances are misaligned".to_string());
        }
        let mut ids = std::collections::HashSet::new();
        for card in &self.run_state.deck_card_states {
            if card.instance_id == 0 || !ids.insert(card.instance_id) {
                return Err("master-deck card instance ids must be unique and nonzero".to_string());
            }
        }
        let max_run_id = self
            .run_state
            .deck_card_states
            .iter()
            .map(|card| u64::from(card.instance_id))
            .max()
            .unwrap_or(0);
        if self.run_state.next_card_instance_id <= max_run_id {
            return Err("master-deck card allocator is behind a live identity".to_string());
        }
        if let Some(combat) = self.combat_engine.as_ref() {
            ids.clear();
            for card in &combat.state.master_deck {
                if card.instance_id == 0 || !ids.insert(card.instance_id) {
                    return Err(
                        "combat master-deck card instance ids must be unique and nonzero"
                            .to_string(),
                    );
                }
            }
            combat.validate_card_instance_ids()?;
        }
        Ok(())
    }

    /// Create a run from immutable save/profile inputs.
    pub fn new_with_profile(seed: u64, ascension: i32, profile: ProfileSnapshot) -> Self {
        Self::new_with_profile_and_ambient_seed(seed, ascension, profile, 0)
    }

    pub fn new_with_profile_and_ambient_seed(
        seed: u64,
        ascension: i32,
        profile: ProfileSnapshot,
        ambient_seed: u64,
    ) -> Self {
        // Exordium seeds mapRng with Settings.seed + actNum.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/Exordium.java:56
        let (map, map_rng) = generate_map_with_rng_for_run(
            seed.wrapping_add(1),
            ascension,
            profile.final_act_available,
        );
        let mut persistent_rngs = crate::seed::PersistentRngs::new(seed);
        // AbstractDungeon.initializeRelicList shuffles these five persistent
        // tier pools before Neow is resolved.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:1227-1231
        // Daily runs make Settings.treatEverythingAsUnlocked() true, so raw
        // profile locks do not constrain either library population path.
        // Java: Settings.java:629-635.
        let (locked_cards, locked_relics) = if profile.is_daily_run {
            (&[][..], &[][..])
        } else {
            (&profile.locked_cards[..], &profile.locked_relics[..])
        };
        let relic_pools =
            RelicPools::watcher_with_locked_relics(&mut persistent_rngs.relic, locked_relics);
        let card_pools = CardPools::watcher_with_locked_cards(locked_cards);
        let event_pools = EventPools::watcher(
            ascension,
            profile.highest_unlocked_ascension,
            profile.is_daily_run,
        );
        let mut floor_rngs = crate::seed::FloorRngs::new(seed);
        // Initial dungeon music selection consumes one miscRng draw. This is
        // reset on the first room transition, but remains visible after Neow.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/audio/MainMusic.java:61-83
        floor_rngs.misc.random_int(1);

        let mut engine = Self {
            run_state: RunState::new(ascension),
            map,
            phase: RunPhase::Neow,
            seed,
            profile,
            profile_updates: Vec::new(),
            persistent_rngs,
            floor_rngs,
            ambient_math_rng: crate::seed::AmbientMathRng::new(ambient_seed),
            java_collections_rng: crate::seed::JavaCollectionsRng::deterministic_default(),
            neow_rng: crate::seed::StsRandom::new(seed),
            map_rng,
            card_pools,
            relic_pools,
            event_pools,
            combat_engine: None,
            pending_chest: None,
            reward_screen: None,
            suspended_reward_screen: None,
            pending_astrolabe_selection: false,
            pending_astrolabe_removed: Vec::new(),
            pending_bottled_flame_selection: false,
            pending_bottled_lightning_selection: false,
            pending_bottled_tornado_selection: false,
            pending_calling_bell_rewards: false,
            pending_empty_cage_removals: 0,
            pending_neow_deck_selection: None,
            pending_event_deck_selection: None,
            pending_relic_followup_source: RewardScreenSource::BossCombat,
            pending_continuation: None,
            decision_stack: DecisionStack::new(),
            current_event: None,
            pending_event_combat: None,
            match_and_keep_state: None,
            scrap_ooze_state: None,
            spire_heart_state: None,
            forced_event_rolls: Vec::new(),
            current_shop: None,
            boss_id: String::new(),
            boss_sequence: VecDeque::new(),
            active_combat_room_type: None,
            active_combat_has_emerald_key: false,
            encounter_queue_act: 0,
            monster_encounter_queue: VecDeque::new(),
            elite_encounter_queue: VecDeque::new(),
            active_encounter_queue: None,
            last_combat_events: Vec::new(),
            neow_frame: Some(NeowFrame::Intro),
            neow_options: Vec::new(),
            neow_selection_witness: None,
        };
        engine.generate_encounter_queues(1);
        engine.roll_boss_sequence_for_act(1);
        engine.refresh_decision_stack();
        engine
    }

    /// Reset the engine to a fresh run with a new seed.
    pub fn reset(&mut self, seed: u64) {
        let ascension = self.run_state.ascension;
        let profile = self.profile.clone();
        let ambient_math_state = self.ambient_math_rng_state();
        let java_collections_state = self.java_collections_rng_state();
        *self = Self::new_with_profile(seed, ascension, profile);
        self.restore_ambient_math_rng_state(ambient_math_state);
        self.restore_java_collections_rng_state(java_collections_state);
    }

    /// Immutable inputs supplied when this simulation root was created.
    pub fn profile_snapshot(&self) -> &ProfileSnapshot {
        &self.profile
    }

    /// Explicit cross-run writes produced by this simulation.
    pub fn profile_updates(&self) -> &[ProfileUpdate] {
        &self.profile_updates
    }

    fn current_note_for_yourself_card(&self) -> &str {
        self.profile_updates
            .iter()
            .rev()
            .find_map(|update| match update {
                ProfileUpdate::StoreNoteForYourselfCard { card_id } => Some(card_id.as_str()),
                ProfileUpdate::MarkBossSeen { .. } => None,
            })
            .unwrap_or(&self.profile.note_for_yourself_card)
    }

    fn store_note_for_yourself_card(&mut self, card_id: String) {
        self.profile_updates
            .push(ProfileUpdate::StoreNoteForYourselfCard { card_id });
    }

    fn mark_boss_seen(&mut self, boss: BossSeenId) {
        let already_pending = self.profile_updates.iter().any(
            |update| matches!(update, ProfileUpdate::MarkBossSeen { boss: queued } if *queued == boss),
        );
        if !self.profile.bosses_seen.contains(boss) && !already_pending {
            self.profile_updates
                .push(ProfileUpdate::MarkBossSeen { boss });
        }
    }

    #[cfg(test)]
    fn boss_floor_for_act(act: i32) -> i32 {
        act * 17 - 1
    }

    fn map_seed_for_act(&self, act: i32) -> u64 {
        match act {
            2 => self.seed.wrapping_add((act as u64) * 100),
            3 => self.seed.wrapping_add((act as u64) * 200),
            4 => self.seed.wrapping_add((act as u64) * 300),
            _ => self.seed.wrapping_add(act as u64),
        }
    }

    fn advance_card_rng_for_act_transition(&mut self) {
        let target = match self.persistent_rngs.card.counter {
            1..=249 => 250,
            251..=499 => 500,
            501..=749 => 750,
            _ => return,
        };
        while self.persistent_rngs.card.counter < target {
            self.persistent_rngs.card.random_bool();
        }
    }

    fn wait_for_continuation(&mut self, continuation: RunContinuation) {
        self.pending_continuation = Some(continuation);
        self.reward_screen = None;
        self.phase = RunPhase::Transition;
    }

    fn step_transition(&mut self, action: &GameAction) {
        if !matches!(action, GameAction::Proceed) {
            return;
        }
        let Some(continuation) = self.pending_continuation.take() else {
            return;
        };
        match continuation {
            RunContinuation::ActThreeSecondBoss => {
                self.run_state.floor += 1;
                self.reset_floor_rngs();
                self.boss_id = self
                    .boss_sequence
                    .front()
                    .cloned()
                    .expect("second Act 3 boss requires a remaining frontier");
                self.enter_combat(false, true);
            }
            RunContinuation::SpireHeart => {
                self.run_state.floor += 1;
                self.reset_floor_rngs();
                self.enter_spire_heart_event();
            }
            RunContinuation::NextAct => self.transition_to_next_act(),
            RunContinuation::TrueVictory => {
                // ProceedButton.goToTrueVictoryRoom installs this explicit node
                // before nextRoomTransition increments the floor.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/ui/buttons/ProceedButton.java
                self.run_state.map_x = 3;
                self.run_state.map_y = 4;
                self.run_state.floor += 1;
                self.reset_floor_rngs();
                self.resolve_terminal_run_victory();
            }
        }
        ()
    }

    fn transition_to_next_act(&mut self) {
        let next_act = self.run_state.act + 1;
        debug_assert!(matches!(next_act, 2 | 3));

        // TreasureRoomBoss already advanced floorNum through nextRoomTransition;
        // dungeonTransitionSetup only increments the act and heals the player.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/ui/buttons/ProceedButton.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:2552-2584
        self.advance_card_rng_for_act_transition();
        self.run_state.act = next_act;
        self.run_state.map_x = 0;
        self.run_state.map_y = -1;
        self.run_state.event_monster_chance = default_event_monster_chance();
        self.run_state.event_shop_chance = default_event_shop_chance();
        self.run_state.event_treasure_chance = default_event_treasure_chance();
        self.run_state.potion_blizz_randomizer = 0;
        self.event_pools.reset_for_act(next_act);

        let heal_amount = if self.run_state.ascension >= 5 {
            (((self.run_state.max_hp - self.run_state.current_hp) as f32) * 0.75).round() as i32
        } else {
            self.run_state.max_hp
        };
        self.heal_run_player(heal_amount);

        let map_seed = self.map_seed_for_act(next_act);
        let (map, map_rng) = generate_map_with_rng_for_run(
            map_seed,
            self.run_state.ascension,
            self.profile.final_act_available && !self.run_state.has_emerald_key,
        );
        self.map = map;
        self.map_rng = map_rng;
        self.generate_encounter_queues(next_act);
        self.roll_boss_sequence_for_act(next_act);
        // MainMusic selects one of two dungeon tracks on Act 2/3 entry using
        // the still-live miscRng; the next room transition replaces this stream.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/audio/MainMusic.java:61-83
        self.floor_rngs.misc.random_int(1);

        self.current_event = None;
        self.pending_event_combat = None;
        self.match_and_keep_state = None;
        self.scrap_ooze_state = None;
        self.spire_heart_state = None;
        self.current_shop = None;
        self.reward_screen = None;
        self.suspended_reward_screen = None;
        self.combat_engine = None;
        self.active_combat_room_type = None;
        self.pending_continuation = None;
        self.phase = RunPhase::MapChoice;
    }

    fn enter_spire_heart_event(&mut self) {
        let event = typed_events_for_act(3)
            .into_iter()
            .find(|event| event.name == "Spire Heart")
            .expect("typed Spire Heart event must exist");
        self.current_event = Some(event);
        self.spire_heart_state = Some(SpireHeartState {
            screen: SpireHeartScreen::Intro,
        });
        self.sync_spire_heart_event();
        self.pending_event_combat = None;
        self.reward_screen = None;
        self.combat_engine = None;
        self.active_combat_room_type = None;
        self.phase = RunPhase::Event;
    }

    fn reset_floor_rngs(&mut self) {
        // AbstractDungeon.nextRoomTransition rebuilds these five room-scoped
        // streams from Settings.seed + floorNum. potionRng remains persistent.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:1737-1741
        let floor_seed = self.seed.wrapping_add(self.run_state.floor as u64);
        self.floor_rngs = crate::seed::FloorRngs::new(floor_seed);
    }

    fn enter_boss_treasure_room(&mut self) {
        // ProceedButton routes the defeated boss room to TreasureRoomBoss via
        // nextRoomTransition before TreasureRoomBoss creates its BossChest.
        // Boss relic choices and onEquip effects therefore see seed + new floor.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/ui/buttons/ProceedButton.java:179-186
        // Java: decompiled/java-src/com/megacrit/cardcrawl/rooms/TreasureRoomBoss.java:57-65
        self.run_state.floor += 1;
        self.reset_floor_rngs();
    }

    fn build_neow_options(&mut self) -> Vec<NeowChoiceOption> {
        // NeowEvent.blessing creates one reward from each category in fixed
        // order using its own Random(Settings.seed) stream. Category 2 first
        // rolls a drawback, then filters its reward list around that drawback.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/neow/NeowEvent.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/neow/NeowReward.java
        (0..4)
            .map(|category| self.build_neow_option(category))
            .collect()
    }

    fn build_neow_option(&mut self, category: usize) -> NeowChoiceOption {
        const CATEGORY_ZERO: &[NeowReward] = &[
            NeowReward::ThreeCards,
            NeowReward::OneRandomRareCard,
            NeowReward::RemoveCard,
            NeowReward::UpgradeCard,
            NeowReward::TransformCard,
            NeowReward::RandomColorless,
        ];
        const CATEGORY_ONE: &[NeowReward] = &[
            NeowReward::ThreePotions,
            NeowReward::RandomCommonRelic,
            NeowReward::TenPercentHpBonus,
            NeowReward::NeowsLament,
            NeowReward::HundredGold,
        ];
        const DRAWBACKS: &[NeowDrawback] = &[
            NeowDrawback::TenPercentHpLoss,
            NeowDrawback::NoGold,
            NeowDrawback::Curse,
            NeowDrawback::PercentDamage,
        ];

        let (reward, drawback) = match category {
            0 => (self.roll_neow_reward(CATEGORY_ZERO), NeowDrawback::None),
            1 => (self.roll_neow_reward(CATEGORY_ONE), NeowDrawback::None),
            2 => {
                let drawback =
                    DRAWBACKS[self.neow_rng.random_int(DRAWBACKS.len() as i32 - 1) as usize];
                let mut rewards = vec![NeowReward::RandomColorlessRare];
                if drawback != NeowDrawback::Curse {
                    rewards.push(NeowReward::RemoveTwo);
                }
                rewards.extend([NeowReward::OneRareRelic, NeowReward::ThreeRareCards]);
                if drawback != NeowDrawback::NoGold {
                    rewards.push(NeowReward::TwoFiftyGold);
                }
                rewards.push(NeowReward::TransformTwo);
                if drawback != NeowDrawback::TenPercentHpLoss {
                    rewards.push(NeowReward::TwentyPercentHpBonus);
                }
                (self.roll_neow_reward(&rewards), drawback)
            }
            3 => (
                self.roll_neow_reward(&[NeowReward::BossRelic]),
                NeowDrawback::None,
            ),
            _ => unreachable!("Neow has exactly four reward categories"),
        };
        let hp_bonus = (self.run_state.max_hp as f32 * 0.1) as i32;
        NeowChoiceOption {
            category: category as u8,
            label: Self::neow_option_label(reward, drawback, hp_bonus),
            reward,
            drawback,
        }
    }

    fn roll_neow_reward(&mut self, rewards: &[NeowReward]) -> NeowReward {
        rewards[self.neow_rng.random_int(rewards.len() as i32 - 1) as usize]
    }

    fn neow_option_label(reward: NeowReward, drawback: NeowDrawback, hp_bonus: i32) -> String {
        let drawback = match drawback {
            NeowDrawback::None => String::new(),
            NeowDrawback::TenPercentHpLoss => format!("Lose {hp_bonus} Max HP. "),
            NeowDrawback::NoGold => "Lose all Gold. ".to_string(),
            NeowDrawback::Curse => "Obtain a Curse. ".to_string(),
            NeowDrawback::PercentDamage => "Lose 30% of current HP. ".to_string(),
        };
        let reward = match reward {
            NeowReward::RandomColorlessRare => {
                "Choose 1 of 3 random Rare Colorless cards to obtain.".to_string()
            }
            NeowReward::ThreeCards => "Choose 1 of 3 cards to obtain.".to_string(),
            NeowReward::OneRandomRareCard => "Obtain a random Rare card.".to_string(),
            NeowReward::RemoveCard => "Remove a card from your deck.".to_string(),
            NeowReward::UpgradeCard => "Upgrade a card.".to_string(),
            NeowReward::RandomColorless => {
                "Choose 1 of 3 random Colorless cards to obtain.".to_string()
            }
            NeowReward::TransformCard => "Transform a card.".to_string(),
            NeowReward::ThreePotions => "Obtain 3 Potions.".to_string(),
            NeowReward::RandomCommonRelic => "Obtain a random Common Relic.".to_string(),
            NeowReward::TenPercentHpBonus => format!("Gain {hp_bonus} Max HP."),
            NeowReward::HundredGold => "Gain 100 gold".to_string(),
            NeowReward::NeowsLament => "Enemies in your next three combats have 1 HP".to_string(),
            NeowReward::RemoveTwo => "Remove 2 cards from your deck.".to_string(),
            NeowReward::TransformTwo => "Transform 2 cards.".to_string(),
            NeowReward::OneRareRelic => "Obtain a random Rare Relic.".to_string(),
            NeowReward::ThreeRareCards => "Choose 1 of 3 Rare cards to obtain.".to_string(),
            NeowReward::TwoFiftyGold => "Gain 250 gold".to_string(),
            NeowReward::TwentyPercentHpBonus => format!("Gain {} Max HP.", hp_bonus * 2),
            NeowReward::BossRelic => "Lose your starting Relic. Obtain a Boss Relic.".to_string(),
        };
        format!("{drawback}{reward}")
    }

    fn apply_neow_choice(&mut self, choice: NeowChoiceOption) {
        let deferred_curse = choice.drawback == NeowDrawback::Curse;
        match choice.drawback {
            NeowDrawback::None | NeowDrawback::Curse => {}
            NeowDrawback::TenPercentHpLoss => {
                let hp_bonus = (self.run_state.max_hp as f32 * 0.1) as i32;
                self.run_state.max_hp = (self.run_state.max_hp - hp_bonus).max(1);
                self.run_state.current_hp = self.run_state.current_hp.min(self.run_state.max_hp);
            }
            NeowDrawback::NoGold => self.run_state.gold = 0,
            NeowDrawback::PercentDamage => {
                let damage = self.run_state.current_hp / 10 * 3;
                self.run_state.current_hp = (self.run_state.current_hp - damage).max(0);
            }
        }

        self.apply_neow_reward(choice.reward);
        if deferred_curse {
            let curse = RANDOM_OBTAINABLE_CURSES[self
                .persistent_rngs
                .card
                .random_int(RANDOM_OBTAINABLE_CURSES.len() as i32 - 1)
                as usize];
            obtain_master_deck_card_state(&mut self.run_state, curse.to_string());
        }
    }

    fn apply_neow_reward(&mut self, reward: NeowReward) {
        let hp_bonus = (self.run_state.max_hp as f32 * 0.1) as i32;
        match reward {
            NeowReward::RandomColorlessRare => {
                let choices = self.generate_neow_card_choices(true, true);
                self.build_neow_card_reward_screen(choices);
            }
            NeowReward::ThreeCards => {
                let choices = self.generate_neow_card_choices(false, false);
                self.build_neow_card_reward_screen(choices);
            }
            NeowReward::OneRandomRareCard => {
                let card = self.roll_neow_card(WATCHER_RARE_CARDS);
                obtain_master_deck_card_state(&mut self.run_state, card);
            }
            NeowReward::RemoveCard => {
                self.start_neow_deck_selection(NeowDeckChoice::Remove, 1, true);
            }
            NeowReward::UpgradeCard => {
                self.start_neow_deck_selection(NeowDeckChoice::Upgrade, 1, false);
            }
            NeowReward::RandomColorless => {
                let choices = self.generate_neow_card_choices(true, false);
                self.build_neow_card_reward_screen(choices);
            }
            NeowReward::TransformCard => {
                self.start_neow_deck_selection(NeowDeckChoice::Transform, 1, false);
            }
            NeowReward::ThreePotions => self.build_neow_potion_reward_screen(),
            NeowReward::RandomCommonRelic => {
                let relic = self.roll_neow_common_relic();
                self.obtain_neow_relic(&relic);
            }
            NeowReward::TenPercentHpBonus => {
                self.run_state.max_hp += hp_bonus;
                self.run_state.current_hp += hp_bonus;
            }
            NeowReward::HundredGold => self.adjust_run_gold(100),
            NeowReward::NeowsLament => self.add_relic_reward("NeowsBlessing"),
            NeowReward::RemoveTwo => {
                self.start_neow_deck_selection(NeowDeckChoice::Remove, 2, false);
            }
            NeowReward::TransformTwo => {
                self.start_neow_deck_selection(NeowDeckChoice::Transform, 2, false);
            }
            NeowReward::OneRareRelic => {
                let relic = self.roll_neow_rare_relic();
                self.obtain_neow_relic(&relic);
            }
            NeowReward::ThreeRareCards => {
                let choices = self.generate_neow_card_choices(false, true);
                self.build_neow_card_reward_screen(choices);
            }
            NeowReward::TwoFiftyGold => self.adjust_run_gold(250),
            NeowReward::TwentyPercentHpBonus => {
                self.run_state.max_hp += hp_bonus * 2;
                self.run_state.current_hp += hp_bonus * 2;
            }
            NeowReward::BossRelic => {
                if !self.run_state.relics.is_empty() {
                    self.run_state.relics.remove(0);
                    self.run_state.relic_flags.rebuild(&self.run_state.relics);
                }
                if let Some(relic) = self.roll_boss_relic_choices(1).into_iter().next() {
                    self.obtain_neow_relic(&relic);
                }
            }
        }
    }

    fn generate_neow_card_choices(
        &mut self,
        colorless: bool,
        rare_only: bool,
    ) -> Vec<RewardChoice> {
        let mut cards = Vec::with_capacity(3);
        for index in 0..3 {
            // NeowReward rolls rarity even when rareOnly subsequently forces
            // RARE, so each displayed card consumes the same two Neow RNG
            // calls: rarity, then pool index.
            let rolled_uncommon = self.neow_rng.random_f32() < 0.33;
            let pool = if colorless {
                if rare_only {
                    SHOP_COLORLESS_RARE_CARDS
                } else if rolled_uncommon {
                    SHOP_COLORLESS_UNCOMMON_CARDS
                } else {
                    // Colorless COMMON results are promoted to UNCOMMON.
                    SHOP_COLORLESS_UNCOMMON_CARDS
                }
            } else if rare_only {
                WATCHER_RARE_CARDS
            } else if rolled_uncommon {
                WATCHER_UNCOMMON_CARDS
            } else {
                WATCHER_COMMON_CARDS
            };

            let mut card = self.roll_neow_card(pool);
            while cards.iter().any(
                |choice| matches!(choice, RewardChoice::Card { card_id, .. } if card_id == &card),
            ) {
                card = self.roll_neow_card(pool);
            }
            cards.push(RewardChoice::Card {
                index,
                card_id: card,
            });
        }
        cards
    }

    fn roll_neow_card(&mut self, pool: &[&str]) -> String {
        pool[self.neow_rng.random_int(pool.len() as i32 - 1) as usize].to_string()
    }

    fn build_neow_card_reward_screen(&mut self, choices: Vec<RewardChoice>) {
        self.reward_screen = Some(RewardScreen {
            source: RewardScreenSource::Event,
            ordered: true,
            active_item: None,
            items: vec![RewardItem {
                index: 0,
                kind: RewardItemKind::CardChoice,
                state: RewardItemState::Available,
                label: "neow_card_reward".to_string(),
                claimable: true,
                active: false,
                skip_allowed: true,
                skip_label: Some("Skip".to_string()),
                choices,
            }],
        });
        self.phase = RunPhase::CardReward;
    }

    fn build_neow_potion_reward_screen(&mut self) {
        let mut screen = RewardScreen {
            source: RewardScreenSource::Event,
            ordered: false,
            active_item: None,
            items: (0..3)
                .map(|index| RewardItem {
                    index,
                    kind: RewardItemKind::Potion,
                    state: RewardItemState::Available,
                    label: self.roll_neow_potion_id(),
                    claimable: true,
                    active: false,
                    skip_allowed: true,
                    skip_label: Some("Skip".to_string()),
                    choices: Vec::new(),
                })
                .collect(),
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
        self.phase = RunPhase::CardReward;
    }

    fn roll_neow_potion_id(&mut self) -> String {
        let index = self
            .persistent_rngs
            .potion
            .random_int(WATCHER_POTIONS.len() as i32 - 1) as usize;
        WATCHER_POTIONS[index].0.to_string()
    }

    fn start_neow_deck_selection(
        &mut self,
        choice: NeowDeckChoice,
        remaining: usize,
        skip_allowed: bool,
    ) {
        self.pending_neow_deck_selection = Some(PendingNeowDeckSelection {
            choice,
            remaining,
            skip_allowed,
        });
        self.build_neow_deck_selection_screen();
    }

    fn build_neow_deck_selection_screen(&mut self) {
        let Some(pending) = self.pending_neow_deck_selection else {
            return;
        };
        let choices = self
            .run_state
            .deck
            .iter()
            .enumerate()
            .filter(|(index, card_id)| match pending.choice {
                NeowDeckChoice::Upgrade => {
                    crate::cards::global_registry().can_upgrade_name(card_id)
                }
                NeowDeckChoice::Remove | NeowDeckChoice::Transform => {
                    self.is_removable_master_deck_index(*index)
                }
            })
            .map(|(index, card_id)| RewardChoice::Card {
                index,
                card_id: card_id.clone(),
            })
            .collect();
        let label = match pending.choice {
            NeowDeckChoice::Remove => "deck_selection_neow_remove",
            NeowDeckChoice::Upgrade => "deck_selection_neow_upgrade",
            NeowDeckChoice::Transform => "deck_selection_neow_transform",
        };
        let mut screen = RewardScreen {
            source: RewardScreenSource::Event,
            ordered: true,
            active_item: None,
            items: vec![RewardItem {
                index: 0,
                kind: RewardItemKind::CardChoice,
                state: RewardItemState::Available,
                label: label.to_string(),
                claimable: true,
                active: false,
                skip_allowed: pending.skip_allowed,
                skip_label: pending.skip_allowed.then(|| "Skip".to_string()),
                choices,
            }],
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
        self.phase = RunPhase::CardReward;
    }

    fn transform_neow_card(&mut self, original: &str) -> Option<String> {
        let base = original.trim_end_matches('+');
        let color = event_card_color(base)?;
        let mut candidates = match color {
            EventCardColor::Colorless => SHOP_COLORLESS_UNCOMMON_CARDS
                .iter()
                .chain(SHOP_COLORLESS_RARE_CARDS)
                .map(|card| (*card).to_string())
                .collect::<Vec<_>>(),
            EventCardColor::Curse => RANDOM_OBTAINABLE_CURSES
                .iter()
                .map(|card| (*card).to_string())
                .collect::<Vec<_>>(),
            _ => WATCHER_COMMON_CARDS
                .iter()
                .chain(WATCHER_UNCOMMON_CARDS)
                .chain(WATCHER_RARE_CARDS)
                .map(|card| (*card).to_string())
                .collect::<Vec<_>>(),
        };
        candidates.retain(|candidate| candidate != base);
        (!candidates.is_empty()).then(|| {
            candidates[self.neow_rng.random_int(candidates.len() as i32 - 1) as usize].clone()
        })
    }

    fn roll_neow_common_relic(&mut self) -> String {
        self.draw_relic_from_pool(RelicTier::Common, false, false)
    }

    fn roll_neow_rare_relic(&mut self) -> String {
        self.draw_relic_from_pool(RelicTier::Rare, false, false)
    }

    fn obtain_neow_relic(&mut self, relic: &str) {
        self.pending_relic_followup_source = RewardScreenSource::Event;
        self.add_relic_reward(relic);
        if relic == "Astrolabe" && self.pending_astrolabe_selection {
            self.build_astrolabe_selection_screen();
            self.phase = RunPhase::CardReward;
        } else if relic == "Calling Bell" && self.pending_calling_bell_rewards {
            self.build_calling_bell_reward_screen();
            self.phase = RunPhase::CardReward;
        } else if matches!(relic, "Empty Cage" | "EmptyCage")
            && self.pending_empty_cage_removals > 0
        {
            self.build_empty_cage_selection_screen();
            self.phase = RunPhase::CardReward;
        } else if matches!(relic, "Tiny House" | "TinyHouse") {
            self.build_tiny_house_reward_screen();
            self.phase = RunPhase::CardReward;
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
    pub fn get_legal_actions(&self) -> Vec<GameAction> {
        let mut actions = match self.phase {
            RunPhase::Neow => self.get_neow_actions(),
            RunPhase::MapChoice => self.get_map_actions(),
            RunPhase::Chest => vec![GameAction::OpenChest, GameAction::LeaveChest],
            RunPhase::Combat => self.get_combat_actions(),
            RunPhase::CardReward => self.get_card_reward_actions(),
            RunPhase::Campfire => self.get_campfire_actions(),
            RunPhase::Shop => self.get_shop_actions(),
            RunPhase::Event => self.get_event_actions(),
            RunPhase::Transition => vec![GameAction::Proceed],
            RunPhase::GameOver => Vec::new(),
        };
        if !matches!(self.phase, RunPhase::Combat | RunPhase::GameOver) {
            actions.extend(self.get_noncombat_potion_actions());
        }
        actions.extend(self.get_potion_discard_actions());
        actions
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
            potion_slots: self
                .combat_engine
                .as_ref()
                .filter(|_| self.phase == RunPhase::Combat)
                .map(|combat| combat.state.potions.as_slice())
                .unwrap_or(self.run_state.potions.as_slice())
                .iter()
                .enumerate()
                .map(|(slot, potion)| PotionSlotContext {
                    slot,
                    potion_id: (!potion.is_empty()).then(|| potion.clone()),
                })
                .collect(),
            neow: if self.phase == RunPhase::Neow && self.neow_frame == Some(NeowFrame::Choice) {
                Some(NeowDecisionContext {
                    options: self
                        .neow_options
                        .iter()
                        .enumerate()
                        .map(|(index, option)| NeowOptionContext {
                            index,
                            category: option.category,
                            reward_id: option.reward.java_id().to_string(),
                            drawback_id: option.drawback.java_id().to_string(),
                            label: option.label.clone(),
                        })
                        .collect(),
                })
            } else {
                None
            },
            chest: if self.phase == RunPhase::Chest {
                self.pending_chest
                    .as_ref()
                    .map(|chest| ChestDecisionContext {
                        size: chest.size_label().to_string(),
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
                Some(self.map_decision_context())
            } else {
                None
            },
            event: if self.phase == RunPhase::Event {
                self.current_event
                    .as_ref()
                    .map(|event| EventDecisionContext {
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
                self.current_shop.as_ref().map(|shop| {
                    build_shop_context(shop, self.run_state.gold, self.run_state.deck.len())
                })
            } else {
                None
            },
            campfire: if self.phase == RunPhase::Campfire {
                Some(CampfireDecisionContext {
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
                    can_recall: self.can_recall_at_campfire(),
                })
            } else {
                None
            },
            transition: match (self.phase, self.neow_frame) {
                (RunPhase::Neow, Some(NeowFrame::Intro)) => Some(TransitionDecisionContext {
                    continuation: "NeowIntro".to_string(),
                }),
                (RunPhase::Neow, Some(NeowFrame::Exit)) => Some(TransitionDecisionContext {
                    continuation: "NeowExit".to_string(),
                }),
                (RunPhase::Transition, _) => Some(TransitionDecisionContext {
                    continuation: format!(
                        "{:?}",
                        self.pending_continuation
                            .expect("transition phase requires a continuation")
                    ),
                }),
                _ => None,
            },
        }
    }

    pub(crate) fn oracle_neow_witness(
        &self,
    ) -> Option<(Vec<NeowOptionContext>, Option<NeowOptionContext>)> {
        let to_context = |index: usize, option: &NeowChoiceOption| NeowOptionContext {
            index,
            category: option.category,
            reward_id: option.reward.java_id().to_string(),
            drawback_id: option.drawback.java_id().to_string(),
            label: option.label.clone(),
        };
        match (self.phase, self.neow_frame) {
            (RunPhase::Neow, Some(NeowFrame::Intro)) => return Some((Vec::new(), None)),
            (RunPhase::Neow, Some(NeowFrame::Choice)) => {
                return Some((
                    self.neow_options
                        .iter()
                        .enumerate()
                        .map(|(index, option)| to_context(index, option))
                        .collect(),
                    None,
                ));
            }
            _ => {}
        }
        self.neow_selection_witness.as_ref().map(|witness| {
            let options: Vec<_> = witness
                .options
                .iter()
                .enumerate()
                .map(|(index, option)| to_context(index, option))
                .collect();
            let selected = options.get(witness.selected_index).cloned();
            (options, selected)
        })
    }

    pub fn current_decision_kind(&self) -> DecisionKind {
        if let Some(kind) = self.decision_stack.current_kind() {
            return kind;
        }
        match self.phase {
            RunPhase::Neow => match self.neow_frame {
                Some(NeowFrame::Choice) => DecisionKind::NeowChoice,
                Some(NeowFrame::Intro | NeowFrame::Exit) => DecisionKind::Proceed,
                None => DecisionKind::GameOver,
            },
            RunPhase::MapChoice => DecisionKind::MapPath,
            RunPhase::Chest => DecisionKind::ChestAction,
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
            RunPhase::Transition => DecisionKind::Proceed,
            RunPhase::GameOver => DecisionKind::GameOver,
        }
    }

    fn refresh_decision_stack(&mut self) {
        let reward_choice = self.decision_stack.current_reward_choice().cloned();

        self.decision_stack.clear();
        let root = match self.phase {
            RunPhase::Neow => match self.neow_frame {
                Some(NeowFrame::Intro) => DecisionFrame::Transition(TransitionDecisionContext {
                    continuation: "NeowIntro".to_string(),
                }),
                Some(NeowFrame::Choice) => DecisionFrame::Neow(NeowDecisionContext {
                    options: self
                        .neow_options
                        .iter()
                        .enumerate()
                        .map(|(index, option)| NeowOptionContext {
                            index,
                            category: option.category,
                            reward_id: option.reward.java_id().to_string(),
                            drawback_id: option.drawback.java_id().to_string(),
                            label: option.label.clone(),
                        })
                        .collect(),
                }),
                Some(NeowFrame::Exit) => DecisionFrame::Transition(TransitionDecisionContext {
                    continuation: "NeowExit".to_string(),
                }),
                None => DecisionFrame::GameOver,
            },
            RunPhase::MapChoice => DecisionFrame::Map(self.map_decision_context()),
            RunPhase::Chest => self
                .pending_chest
                .as_ref()
                .map(|chest| {
                    DecisionFrame::Chest(ChestDecisionContext {
                        size: chest.size_label().to_string(),
                    })
                })
                .unwrap_or(DecisionFrame::GameOver),
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
                can_recall: self.can_recall_at_campfire(),
            }),
            RunPhase::Shop => self
                .current_shop
                .as_ref()
                .map(|shop| {
                    DecisionFrame::Shop(build_shop_context(
                        shop,
                        self.run_state.gold,
                        self.run_state.deck.len(),
                    ))
                })
                .unwrap_or(DecisionFrame::GameOver),
            RunPhase::Event => self
                .current_event
                .as_ref()
                .map(|event| {
                    DecisionFrame::Event(EventDecisionContext {
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
                })
                .unwrap_or(DecisionFrame::GameOver),
            RunPhase::Transition => DecisionFrame::Transition(TransitionDecisionContext {
                continuation: format!(
                    "{:?}",
                    self.pending_continuation
                        .expect("transition phase requires a continuation")
                ),
            }),
            RunPhase::GameOver => DecisionFrame::GameOver,
        };
        self.decision_stack.push(root);
        if let Some(choice) = reward_choice {
            if matches!(self.phase, RunPhase::CardReward) {
                self.decision_stack
                    .push(DecisionFrame::RewardChoice(choice));
            }
        }
    }

    fn get_map_actions(&self) -> Vec<GameAction> {
        self.available_map_destinations()
            .iter()
            .enumerate()
            .map(|(i, _)| GameAction::ChoosePath(i))
            .collect()
    }

    fn map_decision_context(&self) -> MapDecisionContext {
        let destinations = self.available_map_destinations();
        MapDecisionContext {
            available_paths: destinations.len(),
            paths: destinations
                .into_iter()
                .enumerate()
                .map(
                    |(choice, (x, y, room_type, uses_winged_greaves))| MapPathContext {
                        choice,
                        x,
                        y,
                        room_type: format!("{room_type:?}"),
                        uses_winged_greaves,
                        has_emerald_key: usize::try_from(y)
                            .ok()
                            .and_then(|y| self.map.rows.get(y))
                            .and_then(|row| usize::try_from(x).ok().and_then(|x| row.get(x)))
                            .is_some_and(|node| node.has_emerald_key),
                    },
                )
                .collect(),
        }
    }

    fn available_map_destinations(&self) -> Vec<(i32, i32, RoomType, bool)> {
        if self.run_state.map_y < 0 {
            return self
                .map
                .get_start_nodes()
                .iter()
                .map(|node| (node.x as i32, node.y as i32, node.room_type, false))
                .collect();
        }

        // MapGenerator does not contain the standard boss. After the top-row
        // room, Java's map exposes one synthetic boss click at `(-1, 15)`.
        // Act 4 reaches `(3, 4)` through the Heart room's Proceed button.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/ui/buttons/ProceedButton.java
        if self.run_state.act <= 3 && self.run_state.map_y as usize >= self.map.height - 1 {
            return vec![(-1, 15, RoomType::Boss, false)];
        }

        let x = self.run_state.map_x as usize;
        let y = self.run_state.map_y as usize;
        let normal = self.map.get_next_nodes(x, y);
        let mut destinations = normal
            .iter()
            .map(|node| (node.x as i32, node.y as i32, node.room_type, false))
            .collect::<Vec<_>>();
        let can_fly = self
            .run_state
            .relics
            .iter()
            .any(|relic| relic == "WingedGreaves")
            && self.run_state.relic_flags.counters[crate::relic_flags::counter::WINGED_GREAVES] > 0;
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
                        .any(|(nx, ny, _, _)| *nx == node.x as i32 && *ny == node.y as i32)
                {
                    continue;
                }
                destinations.push((node.x as i32, node.y as i32, node.room_type, true));
            }
        }
        destinations
    }

    fn get_neow_actions(&self) -> Vec<GameAction> {
        match self.neow_frame {
            Some(NeowFrame::Intro | NeowFrame::Exit) => vec![GameAction::Proceed],
            Some(NeowFrame::Choice) => self
                .neow_options
                .iter()
                .enumerate()
                .map(|(i, _)| GameAction::ChooseNeowOption(i))
                .collect(),
            None => Vec::new(),
        }
    }

    fn get_combat_actions(&self) -> Vec<GameAction> {
        if let Some(ref engine) = self.combat_engine {
            engine
                .get_legal_actions()
                .into_iter()
                .map(GameAction::CombatAction)
                .collect()
        } else {
            Vec::new()
        }
    }

    fn get_card_reward_actions(&self) -> Vec<GameAction> {
        let Some(screen) = self.reward_screen.as_ref() else {
            return Vec::new();
        };

        if let Some(choice_frame) = self.decision_stack.current_reward_choice() {
            let Some(item) = screen.items.get(choice_frame.item_index) else {
                return Vec::new();
            };
            let mut actions: Vec<GameAction> = choice_frame
                .choices
                .iter()
                .enumerate()
                .map(|(choice_index, _)| GameAction::ChooseRewardOption {
                    item_index: item.index,
                    choice_index,
                })
                .collect();
            if choice_frame.skip_allowed {
                actions.push(GameAction::SkipRewardItem(item.index));
            }
            return actions;
        }

        let mut actions = Vec::new();
        for item in &screen.items {
            if item.claimable {
                actions.push(GameAction::SelectRewardItem(item.index));
            }
        }
        if !screen.ordered {
            actions.push(GameAction::LeaveRewards);
        }
        actions
    }

    fn get_campfire_actions(&self) -> Vec<GameAction> {
        let mut actions = Vec::new();

        // CoffeeDripper.java::canUseCampfireOption disables the exact
        // RestOption class while leaving other campfire options available.
        if !self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::COFFEE_DRIPPER)
        {
            actions.push(GameAction::CampfireRest);
        }

        // FusionHammer.java::canUseCampfireOption disables the exact
        // SmithOption class while leaving Rest and other options available.
        if !self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::FUSION_HAMMER)
        {
            for (i, card) in self.run_state.deck.iter().enumerate() {
                if crate::cards::global_registry().can_upgrade_name(card) {
                    actions.push(GameAction::CampfireUpgrade(i));
                }
            }
        }

        if !self.peace_pipe_removable_indices().is_empty() {
            actions.push(GameAction::CampfireToke);
        }

        if self.can_lift_girya() {
            actions.push(GameAction::CampfireLift);
        }

        if self.has_shovel() {
            actions.push(GameAction::CampfireDig);
        }

        if self.can_recall_at_campfire() {
            actions.push(GameAction::CampfireRecall);
        }

        actions
    }

    fn get_shop_actions(&self) -> Vec<GameAction> {
        let mut actions = Vec::new();
        if let Some(ref shop) = self.current_shop {
            for (i, (_, price)) in shop.cards.iter().enumerate() {
                if self.run_state.gold >= *price {
                    actions.push(GameAction::ShopBuyCard(i));
                }
            }
            for (i, (_, price)) in shop.relics.iter().enumerate() {
                if self.run_state.gold >= *price {
                    actions.push(GameAction::ShopBuyRelic(i));
                }
            }
            let can_obtain_potion = !self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::SOZU)
                && self
                    .run_state
                    .potions
                    .iter()
                    .any(|potion| potion.is_empty());
            if can_obtain_potion {
                for (i, (_, price)) in shop.potions.iter().enumerate() {
                    if self.run_state.gold >= *price {
                        actions.push(GameAction::ShopBuyPotion(i));
                    }
                }
            }
            if !shop.removal_used && self.run_state.gold >= shop.remove_price {
                for i in 0..self.run_state.deck.len() {
                    if self.is_removable_master_deck_index(i) {
                        actions.push(GameAction::ShopRemoveCard(i));
                    }
                }
            }
        }
        actions.push(GameAction::ShopLeave);
        actions
    }

    fn is_purgeable_master_deck_card(card_id: &str) -> bool {
        !matches!(
            card_id,
            "Necronomicurse" | "CurseOfTheBell" | "AscendersBane"
        )
    }

    fn is_removable_master_deck_index(&self, index: usize) -> bool {
        let Some(card_id) = self.run_state.deck.get(index) else {
            return false;
        };
        if !Self::is_purgeable_master_deck_card(card_id) {
            return false;
        }
        let Some(card) = self.run_state.deck_card_states.get(index) else {
            return false;
        };
        ![
            self.run_state.bottled_flame_card_instance_id,
            self.run_state.bottled_lightning_card_instance_id,
            self.run_state.bottled_tornado_card_instance_id,
        ]
        .contains(&Some(card.instance_id))
    }

    fn peace_pipe_removable_indices(&self) -> Vec<usize> {
        if !self
            .run_state
            .relics
            .iter()
            .any(|relic| relic == "Peace Pipe")
        {
            return Vec::new();
        }
        self.run_state
            .deck
            .iter()
            .enumerate()
            .filter_map(|(index, card)| {
                (Self::is_purgeable_master_deck_card(card)
                    && self.is_removable_master_deck_index(index))
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

    fn can_recall_at_campfire(&self) -> bool {
        self.profile.final_act_available && !self.run_state.has_ruby_key
    }

    fn get_event_actions(&self) -> Vec<GameAction> {
        if let Some(state) = &self.match_and_keep_state {
            if matches!(state.screen, MatchAndKeepScreen::Play) {
                return (0..state.board.len())
                    .filter(|idx| Some(*idx) != state.first_pick)
                    .map(GameAction::EventChoice)
                    .collect();
            }
        }
        if let Some(ref event) = self.current_event {
            event
                .options
                .iter()
                .enumerate()
                .map(|(i, _)| GameAction::EventChoice(i))
                .collect()
        } else {
            Vec::new()
        }
    }

    fn get_noncombat_potion_actions(&self) -> Vec<GameAction> {
        // Fruit Juice, Blood Potion, and Entropic Brew override canUse so they
        // are available outside combat except during WeMeetAgain.
        if self.potion_overlay_blocked() {
            return Vec::new();
        }

        self.run_state
            .potions
            .iter()
            .enumerate()
            .filter_map(|(idx, potion)| {
                matches!(
                    potion.as_str(),
                    "Fruit Juice"
                        | "FruitJuice"
                        | "BloodPotion"
                        | "Blood Potion"
                        | "EntropicBrew"
                        | "Entropic Brew"
                )
                .then_some(GameAction::UsePotion(idx))
            })
            .collect()
    }

    fn potion_overlay_blocked(&self) -> bool {
        self.phase == RunPhase::GameOver
            || (self.phase == RunPhase::Event
                && self
                    .current_event
                    .as_ref()
                    .is_some_and(|event| event.name == "WeMeetAgain"))
    }

    fn get_potion_discard_actions(&self) -> Vec<GameAction> {
        if self.potion_overlay_blocked() {
            return Vec::new();
        }
        let potions = self
            .combat_engine
            .as_ref()
            .filter(|_| self.phase == RunPhase::Combat)
            .map(|combat| combat.state.potions.as_slice())
            .unwrap_or(self.run_state.potions.as_slice());
        potions
            .iter()
            .enumerate()
            .filter_map(|(slot, potion)| {
                (!potion.is_empty()).then_some(GameAction::DiscardPotion(slot))
            })
            .collect()
    }

    // =======================================================================
    // Step — execute one canonical action
    // =======================================================================

    /// Execute one canonical action without observations, search metadata, or
    /// shaped training rewards.
    pub fn step_game(&mut self, action: &GameAction) -> StepOutcome {
        let accepted = self.apply_game_action(action);
        StepOutcome {
            status: if accepted {
                ActionStatus::Accepted
            } else {
                ActionStatus::Rejected
            },
            terminal: self
                .run_state
                .run_over
                .then_some(if self.run_state.run_won {
                    RunOutcome::Victory
                } else {
                    RunOutcome::Defeat
                }),
            next_decision: DecisionSnapshot {
                state: self.current_decision_state(),
                context: self.current_decision_context(),
                legal_actions: self.get_legal_actions(),
            },
            events: if accepted {
                self.last_combat_events.clone()
            } else {
                Vec::new()
            },
        }
    }

    fn apply_game_action(&mut self, action: &GameAction) -> bool {
        if !self.get_legal_actions().contains(action) {
            return false;
        }

        self.last_combat_events.clear();
        match action {
            GameAction::UsePotion(potion_idx) => self.step_noncombat_potion(*potion_idx),
            GameAction::DiscardPotion(potion_idx) => {
                self.step_discard_potion(*potion_idx);
            }
            _ => match self.phase {
                RunPhase::Neow => self.step_neow(action),
                RunPhase::MapChoice => self.step_map(action),
                RunPhase::Chest => self.step_chest(action),
                RunPhase::Combat => self.step_combat(action),
                RunPhase::CardReward => self.step_card_reward(action),
                RunPhase::Campfire => self.step_campfire(action),
                RunPhase::Shop => self.step_shop(action),
                RunPhase::Event => self.step_event(action),
                RunPhase::Transition => self.step_transition(action),
                RunPhase::GameOver => {}
            },
        }
        self.refresh_decision_stack();
        true
    }

    fn step_noncombat_potion(&mut self, potion_idx: usize) {
        let Some(potion_id) = self.run_state.potions.get(potion_idx).cloned() else {
            return;
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
                self.trigger_noncombat_potion_used_relics();
            }
            "BloodPotion" | "Blood Potion" => {
                let potency = if self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::SACRED_BARK)
                {
                    40
                } else {
                    20
                };
                // BloodPotion.java casts the max-HP percentage to int.
                self.heal_run_player(self.run_state.max_hp * potency / 100);
                self.run_state.potions[potion_idx].clear();
                self.trigger_noncombat_potion_used_relics();
            }
            "EntropicBrew" | "Entropic Brew" => {
                self.run_state.potions[potion_idx].clear();
                if !self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::SOZU)
                {
                    // EntropicBrew generates one candidate per potion slot even
                    // after the inventory becomes full; failed obtains still
                    // consume the potion RNG draws.
                    for _ in 0..self.run_state.max_potions {
                        let potion = self.roll_reward_potion_id();
                        self.add_potion_reward(&potion);
                    }
                }
                self.trigger_noncombat_potion_used_relics();
            }
            _ => {}
        }
    }

    fn trigger_noncombat_potion_used_relics(&mut self) {
        if self
            .run_state
            .relics
            .iter()
            .any(|relic| relic == "Toy Ornithopter")
        {
            // ToyOrnithopter.java heals only for use, never discard.
            self.heal_run_player(5);
        }
    }

    fn step_discard_potion(&mut self, potion_idx: usize) {
        if self.phase == RunPhase::Combat {
            if let Some(combat) = self.combat_engine.as_mut() {
                if let Some(slot) = combat.state.potions.get_mut(potion_idx) {
                    slot.clear();
                }
            }
        }
        if let Some(slot) = self.run_state.potions.get_mut(potion_idx) {
            slot.clear();
        }
    }

    // =======================================================================
    // Map step
    // =======================================================================

    fn step_map(&mut self, action: &GameAction) {
        let path_idx = match action {
            GameAction::ChoosePath(idx) => *idx,
            _ => return,
        };

        let destinations = self.available_map_destinations();
        let Some(&(next_x, next_y, room_type, used_winged_greaves)) = destinations.get(path_idx)
        else {
            return;
        };

        if used_winged_greaves {
            let counter = &mut self.run_state.relic_flags.counters
                [crate::relic_flags::counter::WINGED_GREAVES];
            *counter -= 1;
            if *counter <= 0 {
                *counter = -2;
            }
        }

        // Java keeps the current encounter at the front of its persistent
        // queue for the room's full lifetime and removes it only while leaving
        // that room. This also covers EventRoom rolls that became MonsterRoom.
        // Java: AbstractDungeon.java::nextRoomTransition,
        // AbstractDungeon.java::getMonsterForRoomCreation.
        self.consume_active_encounter_queue();

        self.run_state.map_x = next_x;
        self.run_state.map_y = next_y;
        self.run_state.floor += 1;
        self.reset_floor_rngs();

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
                self.enter_treasure_room();
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
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::MAW_BANK)
            && self.run_state.relic_flags.counters[crate::relic_flags::counter::MAW_BANK_GOLD] != -2
        {
            self.adjust_run_gold(12);
        }
    }

    fn step_neow(&mut self, action: &GameAction) {
        match (self.neow_frame, action) {
            (Some(NeowFrame::Intro), GameAction::Proceed) => {
                // NeowEvent's intro button calls blessing(), which constructs
                // categories 0 through 3 at this exact decision boundary.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/neow/NeowEvent.java:160-181
                self.neow_options = self.build_neow_options();
                self.neow_frame = Some(NeowFrame::Choice);
            }
            (Some(NeowFrame::Choice), GameAction::ChooseNeowOption(choice_idx)) => {
                let Some(choice) = self.neow_options.get(*choice_idx).cloned() else {
                    return;
                };
                self.neow_selection_witness = Some(NeowSelectionWitness {
                    options: self.neow_options.clone(),
                    selected_index: *choice_idx,
                });
                // Java moves to screen 99 before any card/reward overlay closes.
                self.neow_frame = Some(NeowFrame::Exit);
                self.neow_options.clear();
                self.apply_neow_choice(choice);
            }
            (Some(NeowFrame::Exit), GameAction::Proceed) => {
                self.neow_frame = None;
                self.neow_selection_witness = None;
                self.phase = RunPhase::MapChoice;
            }
            _ => return,
        }
    }

    // =======================================================================
    // Combat
    // =======================================================================

    fn generate_encounter_queues(&mut self, act: i32) {
        self.encounter_queue_act = act;
        self.monster_encounter_queue.clear();
        self.elite_encounter_queue.clear();

        if act == 4 {
            self.monster_encounter_queue
                .extend(std::iter::repeat_n(ACT4_ELITE_ENCOUNTER.to_string(), 3));
            self.elite_encounter_queue
                .extend(std::iter::repeat_n(ACT4_ELITE_ENCOUNTER.to_string(), 3));
            return;
        }

        let (weak, weak_count, strong, elites) = encounter_pools_for_act(act);
        populate_encounter_queue(
            &mut self.monster_encounter_queue,
            weak,
            weak_count,
            false,
            &mut self.persistent_rngs.monster,
        );

        let previous = self
            .monster_encounter_queue
            .back()
            .cloned()
            .unwrap_or_default();
        let exclusions = first_strong_exclusions(act, &previous);
        loop {
            let candidate = roll_weighted_encounter(strong, &mut self.persistent_rngs.monster);
            if !exclusions.contains(&candidate) {
                self.monster_encounter_queue
                    .push_back(candidate.to_string());
                break;
            }
        }

        populate_encounter_queue(
            &mut self.monster_encounter_queue,
            strong,
            12,
            false,
            &mut self.persistent_rngs.monster,
        );
        populate_encounter_queue(
            &mut self.elite_encounter_queue,
            elites,
            10,
            true,
            &mut self.persistent_rngs.monster,
        );
    }

    fn roll_boss_sequence_for_act(&mut self, act: i32) {
        if act == 4 {
            self.boss_sequence = std::iter::repeat_n("CorruptHeart".to_string(), 3).collect();
            self.boss_id = "CorruptHeart".to_string();
            return;
        }

        // Standard dungeon constructors choose the first unseen boss without
        // consuming monsterRng. Daily runs and all-seen profiles shuffle the
        // complete list using one randomLong-backed java.util.Random.
        // Java: Exordium.java, TheCity.java, TheBeyond.java `initializeBoss`.
        let (shuffle_order, unseen_priority): (&[&str], &[(&str, BossSeenId)]) = match act {
            2 => (
                ACT2_BOSSES,
                &[
                    ("TheChamp", BossSeenId::Champ),
                    ("BronzeAutomaton", BossSeenId::Automaton),
                    ("TheCollector", BossSeenId::Collector),
                ],
            ),
            3 => (
                ACT3_BOSSES,
                &[
                    ("AwakenedOne", BossSeenId::Crow),
                    ("DonuAndDeca", BossSeenId::Donut),
                    ("TimeEater", BossSeenId::Wizard),
                ],
            ),
            _ => (
                ACT1_BOSSES,
                &[
                    ("TheGuardian", BossSeenId::Guardian),
                    ("Hexaghost", BossSeenId::Ghost),
                    ("SlimeBoss", BossSeenId::Slime),
                ],
            ),
        };

        let mut bosses = if !self.profile.is_daily_run {
            unseen_priority
                .iter()
                .find(|(_, key)| !self.profile.bosses_seen.contains(*key))
                .map(|(boss, _)| vec![*boss])
                .unwrap_or_else(|| shuffle_order.to_vec())
        } else {
            shuffle_order.to_vec()
        };
        if bosses.len() == 3 {
            let shuffle_seed = self.persistent_rngs.monster.random_long_unbounded();
            crate::seed::java_util_shuffle(&mut bosses, shuffle_seed);
        } else {
            bosses.push(bosses[0]);
        }
        self.boss_sequence = bosses.iter().map(|boss| (*boss).to_string()).collect();
        self.boss_id = self
            .boss_sequence
            .front()
            .cloned()
            .expect("standard acts have three bosses");
    }

    fn ensure_encounter_queues_for_act(&mut self, act: i32) {
        if self.encounter_queue_act != act {
            self.generate_encounter_queues(act);
            self.roll_boss_sequence_for_act(act);
        }
    }

    fn consume_active_encounter_queue(&mut self) {
        match self.active_encounter_queue.take() {
            Some(EncounterQueueKind::Hallway) => {
                self.monster_encounter_queue.pop_front();
            }
            Some(EncounterQueueKind::Elite) => {
                self.elite_encounter_queue.pop_front();
            }
            None => {}
        }
    }

    fn enter_combat(&mut self, is_elite: bool, is_boss: bool) {
        let act = self.run_state.act;
        self.active_combat_has_emerald_key = is_elite
            && self.profile.final_act_available
            && !self.run_state.has_emerald_key
            && self.run_state.map_y >= 0
            && self.map.rows[self.run_state.map_y as usize][self.run_state.map_x as usize]
                .has_emerald_key;
        let encounter = if is_boss {
            self.active_encounter_queue = None;
            let boss = self
                .boss_sequence
                .pop_front()
                .unwrap_or_else(|| self.boss_id.clone());
            self.boss_id = boss.clone();
            if let Some(seen_id) = boss_seen_id_for_encounter(&boss) {
                // MonsterRoomBoss marks the metric/profile before battle, so a
                // subsequent loss must not erase this cross-run update.
                self.mark_boss_seen(seen_id);
            }
            vec![boss]
        } else if is_elite {
            self.ensure_encounter_queues_for_act(act);
            if self.elite_encounter_queue.is_empty() {
                if act == 4 {
                    self.elite_encounter_queue
                        .push_back(ACT4_ELITE_ENCOUNTER.to_string());
                } else {
                    let (_, _, _, elites) = encounter_pools_for_act(act);
                    populate_encounter_queue(
                        &mut self.elite_encounter_queue,
                        elites,
                        10,
                        true,
                        &mut self.persistent_rngs.monster,
                    );
                }
            }
            self.active_encounter_queue = Some(EncounterQueueKind::Elite);
            vec![self
                .elite_encounter_queue
                .front()
                .cloned()
                .expect("elite encounter queue was refilled")]
        } else {
            self.ensure_encounter_queues_for_act(act);
            if self.monster_encounter_queue.is_empty() {
                if act == 4 {
                    self.monster_encounter_queue
                        .push_back(ACT4_ELITE_ENCOUNTER.to_string());
                } else {
                    let (_, _, strong, _) = encounter_pools_for_act(act);
                    populate_encounter_queue(
                        &mut self.monster_encounter_queue,
                        strong,
                        12,
                        false,
                        &mut self.persistent_rngs.monster,
                    );
                }
            }
            self.active_encounter_queue = Some(EncounterQueueKind::Hallway);
            vec![self
                .monster_encounter_queue
                .front()
                .cloned()
                .expect("monster encounter queue was refilled")]
        };
        let room_type = if is_boss {
            RoomType::Boss
        } else if is_elite {
            RoomType::Elite
        } else {
            RoomType::Monster
        };
        self.start_specific_combat(encounter, room_type);
    }

    fn random_louse_id(&mut self) -> String {
        if self.floor_rngs.misc.random_bool() {
            "FuzzyLouseNormal".to_string()
        } else {
            "FuzzyLouseDefensive".to_string()
        }
    }

    fn construct_random_louse(&mut self) -> EnemyCombatState {
        // MonsterHelper.getLouse chooses the type and immediately runs that
        // constructor before the next array element is evaluated.
        let id = self.random_louse_id();
        self.construct_enemy_for_run(&id)
    }

    fn random_slaver_id(&mut self) -> String {
        if self.floor_rngs.misc.random_bool() {
            "RedSlaver".to_string()
        } else {
            "BlueSlaver".to_string()
        }
    }

    fn random_ancient_shape_id(&mut self) -> String {
        match self.floor_rngs.misc.random_int(2) {
            0 => "Spiker".to_string(),
            1 => "Repulsor".to_string(),
            _ => "Exploder".to_string(),
        }
    }

    fn construct_random_weak_wildlife(&mut self) -> EnemyCombatState {
        // bottomGetWeakWildlife constructs all three candidates before the
        // miscRng selection. Discarded candidates still consume constructor
        // monsterHpRng draws; only the chosen Louse later receives Curl Up.
        // Java: helpers/MonsterHelper.java::bottomGetWeakWildlife.
        let louse_id = self.random_louse_id();
        let mut candidates = vec![
            self.construct_enemy_for_run(&louse_id),
            self.construct_enemy_for_run("SpikeSlime_M"),
            self.construct_enemy_for_run("AcidSlime_M"),
        ];
        let index = self.floor_rngs.misc.random_int(2) as usize;
        candidates.remove(index)
    }

    fn construct_random_strong_humanoid(&mut self) -> EnemyCombatState {
        // Java constructs Cultist, the color-selected Slaver, and Looter in
        // that order before choosing which fully constructed instance survives.
        // Java: helpers/MonsterHelper.java::bottomGetStrongHumanoid.
        let cultist = self.construct_enemy_for_run("Cultist");
        let slaver_id = self.random_slaver_id();
        let slaver = self.construct_enemy_for_run(&slaver_id);
        let looter = self.construct_enemy_for_run("Looter");
        let mut candidates = vec![cultist, slaver, looter];
        let index = self.floor_rngs.misc.random_int(2) as usize;
        candidates.remove(index)
    }

    fn construct_random_strong_wildlife(&mut self) -> EnemyCombatState {
        // bottomGetStrongWildlife constructs both candidates, then selects by
        // miscRng.random(0, 1), which is not the randomBoolean overload.
        // Java: helpers/MonsterHelper.java::bottomGetStrongWildlife.
        let mut candidates = vec![
            self.construct_enemy_for_run("FungiBeast"),
            self.construct_enemy_for_run("JawWorm"),
        ];
        let index = self.floor_rngs.misc.random_int(1) as usize;
        candidates.remove(index)
    }

    #[cfg(test)]
    fn enter_specific_combat(&mut self, encounter: Vec<String>) {
        self.active_combat_has_emerald_key = false;
        self.reset_floor_rngs();
        let mapped_room_type = (self.run_state.map_y >= 0)
            .then(|| {
                self.map
                    .rows
                    .get(self.run_state.map_y as usize)
                    .and_then(|row| row.get(self.run_state.map_x as usize))
                    .map(|node| node.room_type)
            })
            .flatten()
            .filter(|room_type| {
                matches!(
                    room_type,
                    RoomType::Monster | RoomType::Elite | RoomType::Boss
                )
            });
        let room_type = mapped_room_type.unwrap_or_else(|| {
            if self.run_state.floor == Self::boss_floor_for_act(self.run_state.act) {
                RoomType::Boss
            } else {
                RoomType::Monster
            }
        });
        self.start_specific_combat(encounter, room_type);
    }

    fn start_event_combat(&mut self, encounter: Vec<String>) {
        // Event combats stay in the current EventRoom. AbstractEvent.enterCombat
        // and AbstractImageEvent.enterCombatFromImage initialize monsters and
        // combat state without calling nextRoomTransition, so all five floor
        // streams retain event-side consumption.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/events/AbstractEvent.java:85-93
        // Java: decompiled/java-src/com/megacrit/cardcrawl/events/AbstractImageEvent.java:76-88
        self.active_combat_has_emerald_key = false;
        self.start_specific_combat(encounter, RoomType::Event);
    }

    fn start_specific_combat(&mut self, encounter: Vec<String>, room_type: RoomType) {
        self.active_combat_room_type = Some(room_type);

        // Expand composite encounters. MonsterHelper constructs Gremlin Leader
        // with two independent weighted miscRng gremlins before the Leader.
        // Source: decompiled/java-src/com/megacrit/cardcrawl/helpers/MonsterHelper.java.
        let mut expanded = Vec::new();
        let mut preconstructed = HashMap::<usize, EnemyCombatState>::new();
        for id in &encounter {
            match id.as_str() {
                "Jaw Worm" => expanded.push("JawWorm".to_string()),
                "2 Louse" | "3 Louse" => {
                    let count = if id == "2 Louse" { 2 } else { 3 };
                    for _ in 0..count {
                        let louse = self.construct_random_louse();
                        let index = expanded.len();
                        expanded.push(louse.id.clone());
                        preconstructed.insert(index, louse);
                    }
                }
                "Small Slimes" => {
                    if self.floor_rngs.misc.random_bool() {
                        expanded.push("SpikeSlime_S".to_string());
                        expanded.push("AcidSlime_M".to_string());
                    } else {
                        expanded.push("AcidSlime_S".to_string());
                        expanded.push("SpikeSlime_M".to_string());
                    }
                }
                "Blue Slaver" => expanded.push("BlueSlaver".to_string()),
                "Red Slaver" => expanded.push("RedSlaver".to_string()),
                "Gremlin Gang" => {
                    let mut pool = vec![
                        "GremlinWarrior",
                        "GremlinWarrior",
                        "GremlinThief",
                        "GremlinThief",
                        "GremlinFat",
                        "GremlinFat",
                        "GremlinTsundere",
                        "GremlinWizard",
                    ];
                    for _ in 0..4 {
                        let index =
                            self.floor_rngs.misc.random_int((pool.len() - 1) as i32) as usize;
                        expanded.push(pool.remove(index).to_string());
                    }
                }
                "Large Slime" => {
                    expanded.push(if self.floor_rngs.misc.random_bool() {
                        "AcidSlime_L".to_string()
                    } else {
                        "SpikeSlime_L".to_string()
                    });
                }
                "Lots of Slimes" => {
                    let mut pool = vec![
                        "SpikeSlime_S",
                        "SpikeSlime_S",
                        "SpikeSlime_S",
                        "AcidSlime_S",
                        "AcidSlime_S",
                    ];
                    while !pool.is_empty() {
                        let index =
                            self.floor_rngs.misc.random_int((pool.len() - 1) as i32) as usize;
                        expanded.push(pool.remove(index).to_string());
                    }
                }
                "Exordium Thugs" => {
                    let weak = self.construct_random_weak_wildlife();
                    let weak_index = expanded.len();
                    expanded.push(weak.id.clone());
                    preconstructed.insert(weak_index, weak);

                    let strong = self.construct_random_strong_humanoid();
                    let strong_index = expanded.len();
                    expanded.push(strong.id.clone());
                    preconstructed.insert(strong_index, strong);
                }
                "Exordium Wildlife" => {
                    let strong = self.construct_random_strong_wildlife();
                    let strong_index = expanded.len();
                    expanded.push(strong.id.clone());
                    preconstructed.insert(strong_index, strong);

                    let weak = self.construct_random_weak_wildlife();
                    let weak_index = expanded.len();
                    expanded.push(weak.id.clone());
                    preconstructed.insert(weak_index, weak);
                }
                "2 Fungi Beasts" => {
                    expanded.push("FungiBeast".to_string());
                    expanded.push("FungiBeast".to_string());
                }
                "Gremlin Nob" => expanded.push("GremlinNob".to_string()),
                "3 Sentries" => {
                    expanded.extend(std::iter::repeat_n("Sentry".to_string(), 3));
                }
                "Spheric Guardian" => expanded.push("SphericGuardian".to_string()),
                "Shell Parasite" => expanded.push("Shelled Parasite".to_string()),
                "3 Byrds" => {
                    expanded.extend(std::iter::repeat_n("Byrd".to_string(), 3));
                }
                "2 Thieves" => {
                    expanded.push("Looter".to_string());
                    expanded.push("Mugger".to_string());
                }
                "Chosen and Byrds" => {
                    expanded.push("Byrd".to_string());
                    expanded.push("Chosen".to_string());
                }
                "Sentry and Sphere" => {
                    expanded.push("Sentry".to_string());
                    expanded.push("SphericGuardian".to_string());
                }
                "Snake Plant" => expanded.push("SnakePlant".to_string()),
                "Centurion and Healer" => {
                    expanded.push("Centurion".to_string());
                    expanded.push("Healer".to_string());
                }
                "Cultist and Chosen" => {
                    expanded.push("Cultist".to_string());
                    expanded.push("Chosen".to_string());
                }
                "3 Cultists" => {
                    expanded.extend(std::iter::repeat_n("Cultist".to_string(), 3));
                }
                "Shelled Parasite and Fungi" => {
                    expanded.push("Shelled Parasite".to_string());
                    expanded.push("FungiBeast".to_string());
                }
                "Slavers" => {
                    expanded.push("BlueSlaver".to_string());
                    expanded.push("SlaverBoss".to_string());
                    expanded.push("RedSlaver".to_string());
                }
                "Book of Stabbing" => expanded.push("BookOfStabbing".to_string()),
                "3 Darklings" => {
                    expanded.extend(std::iter::repeat_n("Darkling".to_string(), 3));
                }
                "Orb Walker" => expanded.push("Orb Walker".to_string()),
                "Spire Growth" => expanded.push("Serpent".to_string()),
                "Sphere and 2 Shapes" => {
                    expanded.push(self.random_ancient_shape_id());
                    expanded.push(self.random_ancient_shape_id());
                    expanded.push("SphericGuardian".to_string());
                }
                "Jaw Worm Horde" => {
                    expanded.extend(std::iter::repeat_n("JawWorm".to_string(), 3));
                }
                "Writhing Mass" => expanded.push("WrithingMass".to_string()),
                "Giant Head" => expanded.push("GiantHead".to_string()),
                "Shield and Spear" => {
                    expanded.push("SpireShield".to_string());
                    expanded.push("SpireSpear".to_string());
                }
                "DonuAndDeca" => {
                    expanded.push("Deca".to_string());
                    expanded.push("Donu".to_string());
                }
                "AwakenedOne" | "Awakened One" => {
                    expanded.push("Cultist".to_string());
                    expanded.push("Cultist".to_string());
                    expanded.push("AwakenedOne".to_string());
                }
                "GremlinLeader" | "Gremlin Leader" => {
                    const GREMLIN_POOL: [&str; 8] = [
                        "GremlinWarrior",
                        "GremlinWarrior",
                        "GremlinThief",
                        "GremlinThief",
                        "GremlinFat",
                        "GremlinFat",
                        "GremlinTsundere",
                        "GremlinWizard",
                    ];
                    for _ in 0..2 {
                        let index = self
                            .floor_rngs
                            .misc
                            .random_int((GREMLIN_POOL.len() - 1) as i32)
                            as usize;
                        let gremlin = self.construct_enemy_for_run(GREMLIN_POOL[index]);
                        let expanded_index = expanded.len();
                        expanded.push(gremlin.id.clone());
                        preconstructed.insert(expanded_index, gremlin);
                    }
                    expanded.push("GremlinLeader".to_string());
                }
                "Reptomancer" => {
                    // Source: decompiled/java-src/com/megacrit/cardcrawl/
                    // helpers/MonsterHelper.java encounter case: left dagger,
                    // Reptomancer, right dagger.
                    expanded.push("Dagger".to_string());
                    expanded.push("Reptomancer".to_string());
                    expanded.push("Dagger".to_string());
                }
                "3 Shapes" | "4 Shapes" => {
                    // Sources: decompiled TheBeyond.java and
                    // MonsterHelper.java (`spawnShapes`). Draw without
                    // replacement from two of each Ancient Shape.
                    let count = if id.as_str() == "3 Shapes" { 3 } else { 4 };
                    let mut pool = vec![
                        "Repulsor", "Repulsor", "Exploder", "Exploder", "Spiker", "Spiker",
                    ];
                    for _ in 0..count {
                        let index =
                            self.floor_rngs.misc.random_int((pool.len() - 1) as i32) as usize;
                        expanded.push(pool.remove(index).to_string());
                    }
                }
                _ => expanded.push(id.clone()),
            }
        }

        // Create enemies
        let mut enemy_states = Vec::with_capacity(expanded.len());
        for (index, id) in expanded.iter().enumerate() {
            let enemy = preconstructed
                .remove(&index)
                .unwrap_or_else(|| self.construct_enemy_for_run(id));
            enemy_states.push(enemy);
        }

        if enemy_states.iter().any(|enemy| enemy.id == "GremlinLeader") {
            for enemy in enemy_states.iter_mut().filter(|enemy| {
                matches!(
                    enemy.id.as_str(),
                    "GremlinFat"
                        | "GremlinThief"
                        | "GremlinWarrior"
                        | "GremlinWizard"
                        | "GremlinTsundere"
                )
            }) {
                enemy.is_minion = true;
            }
        }

        if enemy_states.iter().any(|enemy| enemy.id == "Reptomancer") {
            // Source: reference/extracted/methods/monster/Reptomancer.java
            // (`usePreBattleAction`) gives MinionPower to every other monster
            // in its encounter.
            for enemy in enemy_states
                .iter_mut()
                .filter(|enemy| enemy.id != "Reptomancer")
            {
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
            enemy.entity.set_status(
                crate::status_ids::sid::CURIOSITY,
                if high_ai { 2 } else { 1 },
            );
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
            let chomp_damage = if self.run_state.ascension >= 2 {
                12
            } else {
                11
            };
            let strength = if self.run_state.ascension >= 17 {
                5
            } else if self.run_state.ascension >= 2 {
                4
            } else {
                3
            };
            let bellow_block = if self.run_state.ascension >= 17 { 9 } else { 6 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, chomp_damage);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, strength);
            enemy
                .entity
                .set_status(crate::status_ids::sid::BLOCK_AMT, bellow_block);
            enemy.set_move(crate::enemies::move_ids::JW_CHOMP, chomp_damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/FungiBeast.java.
        // Grow grants 3 Strength, 4 at A2, and one additional point at A17.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "FungiBeast") {
            let strength = if self.run_state.ascension >= 17 {
                5
            } else if self.run_state.ascension >= 2 {
                4
            } else {
                3
            };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, strength);
        }

        // Sources: reference/extracted/methods/monster/LouseNormal.java and
        // LouseDefensive.java. HP and Bite were consumed together by
        // construct_enemy_for_run. Store the pre-battle Curl Up lower bound;
        // CombatEngine draws it only after every monster's init() AI roll.
        for enemy in enemy_states.iter_mut().filter(|e| {
            matches!(
                e.id.as_str(),
                "FuzzyLouseNormal" | "RedLouse" | "FuzzyLouseDefensive" | "GreenLouse"
            )
        }) {
            let bite_damage = enemy.entity.status(crate::status_ids::sid::STARTING_DMG);
            let curl_min = if self.run_state.ascension >= 17 {
                9
            } else if self.run_state.ascension >= 7 {
                4
            } else {
                3
            };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, bite_damage);
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 17 { 4 } else { 3 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::BLOCK_AMT, curl_min);
            enemy.entity.set_status(crate::status_ids::sid::CURL_UP, 0);
            enemy.set_move(crate::enemies::move_ids::LOUSE_BITE, bite_damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/SlaverBlue.java.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "SlaverBlue" | "BlueSlaver"))
        {
            let (stab, rake) = if self.run_state.ascension >= 2 {
                (13, 8)
            } else {
                (12, 7)
            };
            let weak = if self.run_state.ascension >= 17 { 2 } else { 1 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, stab);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, rake);
            enemy
                .entity
                .set_status(crate::status_ids::sid::BLOCK_AMT, weak);
            enemy.set_move(crate::enemies::move_ids::BS_STAB, stab, 1, 0);
        }

        // Source: reference/extracted/methods/monster/SlaverRed.java.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "SlaverRed" | "RedSlaver"))
        {
            let (stab, scrape) = if self.run_state.ascension >= 2 {
                (14, 9)
            } else {
                (13, 8)
            };
            let vulnerable = if self.run_state.ascension >= 17 { 2 } else { 1 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, stab);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, scrape);
            enemy
                .entity
                .set_status(crate::status_ids::sid::BLOCK_AMT, vulnerable);
            enemy
                .entity
                .set_status(crate::status_ids::sid::IS_FIRST_MOVE, 1);
            enemy.set_move(crate::enemies::move_ids::RS_STAB, stab, 1, 0);
        }

        // Source: reference/extracted/methods/monster/BanditBear.java.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "BanditBear" | "Bear"))
        {
            let (maul, lunge) = if self.run_state.ascension >= 2 {
                (20, 10)
            } else {
                (18, 9)
            };
            let dexterity_down = if self.run_state.ascension >= 17 { 4 } else { 2 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, maul);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, lunge);
            enemy
                .entity
                .set_status(crate::status_ids::sid::BLOCK_AMT, dexterity_down);
            enemy.set_move(crate::enemies::move_ids::BEAR_HUG, 0, 0, 0);
            enemy.move_effects.clear();
            enemy.add_effect(crate::combat_types::mfx::DEX_DOWN, dexterity_down as i16);
        }

        // Source: reference/extracted/methods/monster/BanditPointy.java.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "BanditChild" | "BanditPointy" | "Pointy"))
        {
            let damage = if self.run_state.ascension >= 2 { 6 } else { 5 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.set_move(crate::enemies::move_ids::POINTY_STAB, damage, 2, 0);
        }

        // Source: reference/extracted/methods/monster/Byrd.java (constructor).
        // Damage changes at A2, while the stored Flight amount changes at A17.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Byrd") {
            let peck_count = if self.run_state.ascension >= 2 { 6 } else { 5 };
            let swoop_damage = if self.run_state.ascension >= 2 {
                14
            } else {
                12
            };
            let flight = if self.run_state.ascension >= 17 { 4 } else { 3 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, 1);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, peck_count);
            enemy
                .entity
                .set_status(crate::status_ids::sid::SLASH_DMG, swoop_damage);
            enemy
                .entity
                .set_status(crate::status_ids::sid::HEAD_SLAM_DMG, 3);
            enemy
                .entity
                .set_status(crate::status_ids::sid::BLOCK_AMT, flight);
            enemy
                .entity
                .set_status(crate::status_ids::sid::FLIGHT, flight);
            enemy.set_move(crate::enemies::move_ids::BYRD_PECK, 1, peck_count, 0);
        }

        // Source: reference/extracted/methods/monster/ShelledParasite.java.
        // Damage changes at A2, its opener is forced to Fell at A17, and
        // usePreBattleAction grants both 14 Plated Armor and 14 block.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "Shelled Parasite" | "ShelledParasite"))
        {
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 7 } else { 6 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 2 {
                    21
                } else {
                    18
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 2 {
                    12
                } else {
                    10
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy
                .entity
                .set_status(crate::status_ids::sid::PLATED_ARMOR, 14);
            enemy.entity.block = 14;
        }

        // Sources: reference/extracted/methods/monster/SnakePlant.java and
        // decompiled/java-src/com/megacrit/cardcrawl/powers/MalleablePower.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "SnakePlant") {
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 8 } else { 7 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::MALLEABLE, 3);
            // Preserve MalleablePower.basePower separately from its growing amount.
            enemy
                .entity
                .set_status(crate::status_ids::sid::BLOCK_AMT, 3);
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 },
            );
        }

        // Source: reference/extracted/methods/monster/Snecko.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Snecko") {
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 {
                    18
                } else {
                    15
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 2 { 10 } else { 8 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 },
            );
        }

        // Source: reference/extracted/methods/monster/SphericGuardian.java.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "SphericGuardian" | "Spheric Guardian"))
        {
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_TURN, 1);
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 {
                    11
                } else {
                    10
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 {
                    35
                } else {
                    25
                },
            );
            enemy.entity.set_status(crate::status_ids::sid::STR_AMT, 15);
            enemy
                .entity
                .set_status(crate::status_ids::sid::BARRICADE, 1);
            enemy.entity.set_status(crate::status_ids::sid::ARTIFACT, 3);
            enemy.entity.block = 40;
        }

        // Source: reference/extracted/methods/monster/TheCollector.java.
        // Fireball/Strength change at A4, HP and base Block at A9, while A19
        // independently raises Strength, Mega Debuff, and effective Buff Block.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "TheCollector" | "Collector"))
        {
            enemy.entity.set_status(
                crate::status_ids::sid::FIREBALL_DMG,
                if self.run_state.ascension >= 4 {
                    21
                } else {
                    18
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 19 {
                    5
                } else if self.run_state.ascension >= 4 {
                    4
                } else {
                    3
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 19 {
                    23
                } else if self.run_state.ascension >= 9 {
                    18
                } else {
                    15
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 19 { 5 } else { 3 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::TURN_COUNT, 0);
            enemy
                .entity
                .set_status(crate::status_ids::sid::USED_MEGA_DEBUFF, 0);
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
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
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, slash);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, fury);
            enemy
                .entity
                .set_status(crate::status_ids::sid::ATTACK_COUNT, 3);
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 {
                    20
                } else {
                    15
                },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::COUNT, monster_count);
            enemy.set_move(crate::enemies::move_ids::CENT_SLASH, slash, 1, 0);
        }

        // Source: reference/extracted/methods/monster/Champ.java (constructor
        // and `getMove`). Its A4/A9/A19 thresholds are independent of HP.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "Champ" | "TheChamp"))
        {
            let high_damage = self.run_state.ascension >= 4;
            enemy.entity.set_status(
                crate::status_ids::sid::SLASH_DMG,
                if high_damage { 18 } else { 16 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::SLAP_DMG,
                if high_damage { 14 } else { 12 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 19 {
                    4
                } else if self.run_state.ascension >= 4 {
                    3
                } else {
                    2
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::FORGE_AMT,
                if self.run_state.ascension >= 19 {
                    7
                } else if self.run_state.ascension >= 9 {
                    6
                } else {
                    5
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 19 {
                    20
                } else if self.run_state.ascension >= 9 {
                    18
                } else {
                    15
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 19 { 1 } else { 0 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::NUM_TURNS, 0);
            enemy
                .entity
                .set_status(crate::status_ids::sid::FORGE_TIMES, 0);
            enemy
                .entity
                .set_status(crate::status_ids::sid::THRESHOLD_REACHED, 0);
        }

        // Source: reference/extracted/methods/monster/Chosen.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Chosen") {
            let high_damage = self.run_state.ascension >= 2;
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if high_damage { 21 } else { 18 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if high_damage { 12 } else { 10 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::SLAP_DMG,
                if high_damage { 6 } else { 5 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_TURN, 1);
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_MOVE, 0);
        }

        // Source: reference/extracted/methods/monster/CorruptHeart.java
        // (constructor and `usePreBattleAction`). A4 damage, A9 HP, and A19
        // powers are independent thresholds; HP alone cannot encode them.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "CorruptHeart" | "Corrupt Heart"))
        {
            let high_damage = self.run_state.ascension >= 4;
            enemy.entity.set_status(
                crate::status_ids::sid::BLOOD_HIT_COUNT,
                if high_damage { 15 } else { 12 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::ECHO_DMG,
                if high_damage { 45 } else { 40 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::INVINCIBLE,
                if self.run_state.ascension >= 19 {
                    200
                } else {
                    300
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::BEAT_OF_DEATH,
                if self.run_state.ascension >= 19 { 2 } else { 1 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::MOVE_COUNT, 0);
            enemy
                .entity
                .set_status(crate::status_ids::sid::BUFF_COUNT, 0);
            enemy
                .entity
                .set_status(crate::status_ids::sid::IS_FIRST_MOVE, 1);
        }

        // Source: reference/extracted/methods/monster/SpireShield.java
        // (constructor and `usePreBattleAction`). Damage, HP, and Artifact use
        // independent A3/A8/A18 thresholds. Shield begins to the player's left,
        // so it is the initial Back Attack while the player faces right.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "SpireShield" | "Spire Shield"))
        {
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 3 {
                    14
                } else {
                    12
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 3 {
                    38
                } else {
                    34
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::ARTIFACT,
                if self.run_state.ascension >= 18 { 2 } else { 1 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 18 { 1 } else { 0 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::MOVE_COUNT, 0);
            enemy.back_attack = true;
        }
        // Source: reference/extracted/methods/monster/SpireSpear.java
        // (constructor and `usePreBattleAction`). Burn Strike and Skewer change
        // at A3; Artifact and Burn destination change independently at A18.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "SpireSpear" | "Spire Spear"))
        {
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 3 { 6 } else { 5 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::SKEWER_COUNT,
                if self.run_state.ascension >= 3 { 4 } else { 3 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::ARTIFACT,
                if self.run_state.ascension >= 18 { 2 } else { 1 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 18 { 1 } else { 0 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::MOVE_COUNT, 0);
        }
        if enemy_states
            .iter()
            .any(|enemy| matches!(enemy.id.as_str(), "SpireShield" | "Spire Shield"))
        {
            for enemy in enemy_states
                .iter_mut()
                .filter(|enemy| matches!(enemy.id.as_str(), "SpireSpear" | "Spire Spear"))
            {
                enemy.back_attack = false;
            }
        }

        // Source: reference/extracted/methods/monster/Darkling.java.
        // HP and Nip were consumed together by construct_enemy_for_run;
        // Chomp/Harden change at independent ascension thresholds.
        for (index, enemy) in enemy_states
            .iter_mut()
            .enumerate()
            .filter(|(_, enemy)| enemy.id == "Darkling")
        {
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 9 } else { 8 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy
                .entity
                .set_status(crate::status_ids::sid::COUNT, index as i32);
            enemy.entity.set_status(crate::status_ids::sid::REGROW, 1);
        }

        // Source: reference/extracted/methods/monster/Deca.java. Damage changes
        // at A4, HP at A9, and Artifact/Square's Plated Armor at A19.
        for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id == "Deca") {
            let beam = if self.run_state.ascension >= 4 {
                12
            } else {
                10
            };
            enemy
                .entity
                .set_status(crate::status_ids::sid::BEAM_DMG, beam);
            enemy.entity.set_status(
                crate::status_ids::sid::ARTIFACT,
                if self.run_state.ascension >= 19 { 3 } else { 2 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 19 { 1 } else { 0 },
            );
            enemy.set_move(crate::enemies::move_ids::DECA_BEAM, beam, 2, 0);
            enemy.add_effect(crate::combat_types::mfx::DAZE, 2);
        }

        // Source: reference/extracted/methods/monster/Donu.java. Damage changes
        // at A4, HP at A9, and pre-battle Artifact at A19.
        for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id == "Donu") {
            let beam = if self.run_state.ascension >= 4 {
                12
            } else {
                10
            };
            enemy
                .entity
                .set_status(crate::status_ids::sid::BEAM_DMG, beam);
            enemy.entity.set_status(
                crate::status_ids::sid::ARTIFACT,
                if self.run_state.ascension >= 19 { 3 } else { 2 },
            );
            enemy.set_move(crate::enemies::move_ids::DONU_CIRCLE, 0, 0, 0);
            enemy.add_effect(crate::combat_types::mfx::STRENGTH, 3);
            enemy.add_effect(crate::combat_types::mfx::STRENGTH_ALL_ALLIES, 3);
        }

        // Source: reference/extracted/methods/monster/Exploder.java. Damage
        // changes at A2; ExplosivePower(3) and turnCount start before combat.
        for enemy in enemy_states
            .iter_mut()
            .filter(|enemy| enemy.id == "Exploder")
        {
            let damage = if self.run_state.ascension >= 2 { 11 } else { 9 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy
                .entity
                .set_status(crate::status_ids::sid::TURN_COUNT, 0);
            enemy
                .entity
                .set_status(crate::status_ids::sid::EXPLOSIVE, 3);
            enemy.set_move(crate::enemies::move_ids::EXPLODER_ATTACK, damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/GiantHead.java. The
        // zero-amount Slow power uses sentinel 1 in Rust; A18 decrements count
        // once in usePreBattleAction before the opening rollMove.
        for enemy in enemy_states
            .iter_mut()
            .filter(|enemy| matches!(enemy.id.as_str(), "GiantHead" | "Giant Head"))
        {
            enemy.entity.set_status(
                crate::status_ids::sid::COUNT,
                if self.run_state.ascension >= 18 { 4 } else { 5 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DEATH_DMG,
                if self.run_state.ascension >= 3 {
                    40
                } else {
                    30
                },
            );
            enemy.entity.set_status(crate::status_ids::sid::SLOW, 1);
        }

        // Source: reference/extracted/methods/monster/TimeEater.java.
        // Damage changes at A4, HP at A9, and the extra move payloads at A19.
        for enemy in enemy_states
            .iter_mut()
            .filter(|enemy| matches!(enemy.id.as_str(), "TimeEater" | "Time Eater"))
        {
            enemy.entity.set_status(
                crate::status_ids::sid::REVERB_DMG,
                if self.run_state.ascension >= 4 { 8 } else { 7 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::HEAD_SLAM_DMG,
                if self.run_state.ascension >= 4 {
                    32
                } else {
                    26
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 19 { 1 } else { 0 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::USED_HASTE, 0);
            enemy
                .entity
                .set_status(crate::status_ids::sid::TIME_WARP_ACTIVE, 1);
        }

        // Source: reference/extracted/methods/monster/Transient.java. Damage
        // changes at A2, while Fading lasts six turns only at A17+.
        for enemy in enemy_states
            .iter_mut()
            .filter(|enemy| enemy.id == "Transient")
        {
            let damage = if self.run_state.ascension >= 2 {
                40
            } else {
                30
            };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy
                .entity
                .set_status(crate::status_ids::sid::ATTACK_COUNT, 0);
            enemy.entity.set_status(
                crate::status_ids::sid::FADING,
                if self.run_state.ascension >= 17 { 6 } else { 5 },
            );
            enemy.entity.set_status(crate::status_ids::sid::SHIFTING, 1);
        }

        // Source: reference/extracted/methods/monster/Maw.java. Maw has fixed
        // 300 HP; only Slam changes at A2, while Drool and Roar change at A17.
        for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id == "Maw") {
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 {
                    30
                } else {
                    25
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 17 { 5 } else { 3 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 { 5 } else { 3 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::TURN_COUNT, 1);
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_MOVE, 0);
        }

        // Source: reference/extracted/methods/monster/Nemesis.java. Fire
        // damage changes at A3 and the Burn payload changes at A18.
        for enemy in enemy_states
            .iter_mut()
            .filter(|enemy| enemy.id == "Nemesis")
        {
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 3 { 7 } else { 6 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 18 { 5 } else { 3 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::SCYTHE_COOLDOWN, 0);
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
        }

        // Source: reference/extracted/methods/monster/OrbWalker.java. Attack
        // values change at A2; GenericStrengthUp changes from 3 to 5 at A17.
        for enemy in enemy_states
            .iter_mut()
            .filter(|enemy| matches!(enemy.id.as_str(), "OrbWalker" | "Orb Walker"))
        {
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 {
                    11
                } else {
                    10
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 2 {
                    16
                } else {
                    15
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::GENERIC_STRENGTH_UP,
                if self.run_state.ascension >= 17 { 5 } else { 3 },
            );
        }

        // Source: reference/extracted/methods/monster/Repulsor.java. Its
        // attack changes at A2; HP is rolled separately below at A7.
        for enemy in enemy_states
            .iter_mut()
            .filter(|enemy| enemy.id == "Repulsor")
        {
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 {
                    13
                } else {
                    11
                },
            );
        }

        // Source: reference/extracted/methods/monster/Spiker.java.
        for enemy in enemy_states.iter_mut().filter(|enemy| enemy.id == "Spiker") {
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 9 } else { 7 },
            );
            let starting_thorns = if self.run_state.ascension >= 2 { 4 } else { 3 };
            enemy.entity.set_status(
                crate::status_ids::sid::THORNS,
                starting_thorns + if self.run_state.ascension >= 17 { 3 } else { 0 },
            );
            enemy.entity.set_status(crate::status_ids::sid::COUNT, 0);
        }

        // Source: reference/extracted/methods/monster/WrithingMass.java.
        // Damage changes at A2; HP changes separately at A7 below.
        for enemy in enemy_states
            .iter_mut()
            .filter(|enemy| matches!(enemy.id.as_str(), "WrithingMass" | "Writhing Mass"))
        {
            let high = self.run_state.ascension >= 2;
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if high { 38 } else { 32 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, if high { 9 } else { 7 });
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if high { 16 } else { 15 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::HEAD_SLAM_DMG,
                if high { 12 } else { 10 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy
                .entity
                .set_status(crate::status_ids::sid::USED_MEGA_DEBUFF, 0);
            enemy.entity.set_status(crate::status_ids::sid::REACTIVE, 1);
            enemy
                .entity
                .set_status(crate::status_ids::sid::MALLEABLE, 3);
        }

        // Source: reference/extracted/methods/monster/SpireGrowth.java. The
        // Java class uses canonical ID `Serpent`; damage changes at A2,
        // Constricted at A17, and HP is patched separately below at A7.
        for enemy in enemy_states.iter_mut().filter(|enemy| {
            matches!(
                enemy.id.as_str(),
                "Serpent" | "SpireGrowth" | "Spire Growth"
            )
        }) {
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 {
                    18
                } else {
                    16
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 2 {
                    25
                } else {
                    22
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 {
                    12
                } else {
                    10
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 },
            );
            enemy.entity.set_status(crate::status_ids::sid::COUNT, 0);
        }

        // Source: reference/extracted/methods/monster/Reptomancer.java.
        let repto_others = enemy_states
            .iter()
            .filter(|enemy| enemy.id != "Reptomancer" && enemy.is_alive())
            .count() as i32;
        for enemy in enemy_states
            .iter_mut()
            .filter(|enemy| enemy.id == "Reptomancer")
        {
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 3 {
                    16
                } else {
                    13
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 3 {
                    34
                } else {
                    30
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 18 { 2 } else { 1 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
            enemy
                .entity
                .set_status(crate::status_ids::sid::COUNT, repto_others);
        }

        // Source: reference/extracted/methods/monster/BanditLeader.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "BanditLeader") {
            let (cross_slash, agonizing_slash) = if self.run_state.ascension >= 2 {
                (17, 12)
            } else {
                (15, 10)
            };
            let weak = if self.run_state.ascension >= 17 { 3 } else { 2 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, cross_slash);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, agonizing_slash);
            enemy
                .entity
                .set_status(crate::status_ids::sid::BLOCK_AMT, weak);
            enemy.set_move(crate::enemies::move_ids::BANDIT_MOCK, 0, 0, 0);
        }

        // Source: reference/extracted/methods/monster/BookOfStabbing.java.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "BookOfStabbing" | "Book of Stabbing"))
        {
            let (stab, big_stab) = if self.run_state.ascension >= 3 {
                (7, 24)
            } else {
                (6, 21)
            };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, stab);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, big_stab);
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 18 { 1 } else { 0 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::STAB_COUNT, 0);
            enemy
                .entity
                .set_status(crate::status_ids::sid::PAINFUL_STABS, 1);
        }

        // Source: reference/extracted/methods/monster/Taskmaster.java.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "SlaverBoss" | "TaskMaster" | "Taskmaster"))
        {
            let wounds = if self.run_state.ascension >= 18 {
                3
            } else if self.run_state.ascension >= 3 {
                2
            } else {
                1
            };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, 7);
            enemy
                .entity
                .set_status(crate::status_ids::sid::BLOCK_AMT, wounds);
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 18 { 1 } else { 0 },
            );
        }

        // Source: reference/extracted/methods/monster/BronzeAutomaton.java.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| matches!(e.id.as_str(), "BronzeAutomaton" | "Bronze Automaton"))
        {
            let (flail, beam, strength) = if self.run_state.ascension >= 4 {
                (8, 50, 4)
            } else {
                (7, 45, 3)
            };
            enemy
                .entity
                .set_status(crate::status_ids::sid::FLAIL_DMG, flail);
            enemy
                .entity
                .set_status(crate::status_ids::sid::BEAM_DMG, beam);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, strength);
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 9 { 12 } else { 9 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 19 { 1 } else { 0 },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_TURN, 1);
            enemy
                .entity
                .set_status(crate::status_ids::sid::NUM_TURNS, 0);
            enemy.entity.set_status(crate::status_ids::sid::ARTIFACT, 3);
        }

        // Source: reference/extracted/methods/monster/AcidSlime_S.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "AcidSlime_S") {
            let damage = if self.run_state.ascension >= 2 { 4 } else { 3 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 17 {
                    17
                } else {
                    0
                },
            );
            enemy.set_move(crate::enemies::move_ids::AS_S_TACKLE, damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/AcidSlime_M.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "AcidSlime_M") {
            let (wound, normal) = if self.run_state.ascension >= 2 {
                (8, 12)
            } else {
                (7, 10)
            };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, wound);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, normal);
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 {
                    17
                } else {
                    0
                },
            );
            enemy.set_move(crate::enemies::move_ids::AS_CORROSIVE_SPIT, wound, 1, 0);
            enemy.add_effect(crate::combat_types::mfx::SLIMED, 1);
        }

        // Source: reference/extracted/methods/monster/AcidSlime_L.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "AcidSlime_L") {
            let (wound, normal) = if self.run_state.ascension >= 2 {
                (12, 18)
            } else {
                (11, 16)
            };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, wound);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, normal);
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 {
                    17
                } else {
                    0
                },
            );
            enemy.set_move(crate::enemies::move_ids::AS_CORROSIVE_SPIT, wound, 1, 0);
            enemy.add_effect(crate::combat_types::mfx::SLIMED, 2);
        }

        // Source: reference/extracted/methods/monster/SpikeSlime_S.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "SpikeSlime_S") {
            let damage = if self.run_state.ascension >= 2 { 6 } else { 5 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.set_move(crate::enemies::move_ids::SS_TACKLE, damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/SpikeSlime_M.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "SpikeSlime_M") {
            let damage = if self.run_state.ascension >= 2 { 10 } else { 8 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 {
                    17
                } else {
                    0
                },
            );
            enemy.set_move(crate::enemies::move_ids::SS_TACKLE, damage, 1, 0);
            enemy.add_effect(crate::combat_types::mfx::SLIMED, 1);
        }

        // Source: reference/extracted/methods/monster/SpikeSlime_L.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "SpikeSlime_L") {
            let damage = if self.run_state.ascension >= 2 {
                18
            } else {
                16
            };
            let frail = if self.run_state.ascension >= 17 { 3 } else { 2 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, frail);
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 {
                    17
                } else {
                    0
                },
            );
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
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, swipe);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, lunge);
            enemy
                .entity
                .set_status(crate::status_ids::sid::BLOCK_AMT, 6);
            enemy.entity.set_status(
                crate::status_ids::sid::TURN_COUNT,
                if self.run_state.ascension >= 17 {
                    20
                } else {
                    15
                },
            );
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
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, swipe);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, big_swipe);
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 {
                    17
                } else {
                    11
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::TURN_COUNT,
                if self.run_state.ascension >= 17 {
                    20
                } else {
                    15
                },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::ATTACK_COUNT, 0);
            enemy.set_move(crate::enemies::move_ids::MUGGER_MUG, swipe, 1, 0);
        }

        // Source: reference/extracted/methods/monster/GremlinFat.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "GremlinFat") {
            let damage = if self.run_state.ascension >= 2 { 5 } else { 4 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 {
                    17
                } else {
                    0
                },
            );
            enemy.set_move(crate::enemies::move_ids::GREMLIN_FAT_SMASH, damage, 1, 0);
            enemy.add_effect(crate::combat_types::mfx::WEAK, 1);
            if self.run_state.ascension >= 17 {
                enemy.add_effect(crate::combat_types::mfx::FRAIL, 1);
            }
        }

        // Source: reference/extracted/methods/monster/GremlinThief.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "GremlinThief") {
            let damage = if self.run_state.ascension >= 2 { 10 } else { 9 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.set_move(crate::enemies::move_ids::GREMLIN_ATTACK, damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/GremlinWarrior.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "GremlinWarrior") {
            let damage = if self.run_state.ascension >= 2 { 5 } else { 4 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(
                crate::status_ids::sid::ANGRY,
                if self.run_state.ascension >= 17 { 2 } else { 1 },
            );
            enemy.set_move(crate::enemies::move_ids::GREMLIN_ATTACK, damage, 1, 0);
        }

        // Source: reference/extracted/methods/monster/GremlinWizard.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "GremlinWizard") {
            let damage = if self.run_state.ascension >= 2 {
                30
            } else {
                25
            };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy.entity.set_status(crate::status_ids::sid::COUNT, 1);
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 {
                    17
                } else {
                    0
                },
            );
            enemy.set_move(crate::enemies::move_ids::GREMLIN_PROTECT, 0, 0, 0);
        }

        // Source: reference/extracted/methods/monster/GremlinTsundere.java.
        for enemy in enemy_states
            .iter_mut()
            .filter(|e| e.id == "GremlinTsundere")
        {
            let damage = if self.run_state.ascension >= 2 { 8 } else { 6 };
            let block = if self.run_state.ascension >= 17 {
                11
            } else if self.run_state.ascension >= 7 {
                8
            } else {
                7
            };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy
                .entity
                .set_status(crate::status_ids::sid::BLOCK_AMT, block);
            enemy.set_move(crate::enemies::move_ids::GREMLIN_TSUNDERE_PROTECT, 0, 0, 0);
            enemy.add_effect(crate::combat_types::mfx::BLOCK_RANDOM_OTHER, block as i16);
        }

        // Source: reference/extracted/methods/monster/GremlinLeader.java.
        // STARTING_DMG is an internal marker carrying ascension for later
        // SummonGremlinAction construction; the Leader's Stab is always 6x3.
        let alive_gremlins = enemy_states
            .iter()
            .filter(|enemy| enemy.is_minion && enemy.is_alive())
            .count() as i32;
        for enemy in enemy_states
            .iter_mut()
            .filter(|enemy| enemy.id == "GremlinLeader")
        {
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 18 {
                    5
                } else if self.run_state.ascension >= 3 {
                    4
                } else {
                    3
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 18 {
                    10
                } else {
                    6
                },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::COUNT, alive_gremlins);
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                self.run_state.ascension,
            );
        }

        // Source: reference/extracted/methods/monster/Healer.java. COUNT is
        // refreshed from group missing HP before each later RollMoveAction.
        let missing_hp = enemy_states
            .iter()
            .filter(|enemy| enemy.is_alive())
            .map(|enemy| enemy.entity.max_hp - enemy.entity.hp)
            .sum();
        for enemy in enemy_states
            .iter_mut()
            .filter(|enemy| matches!(enemy.id.as_str(), "Healer" | "Mystic"))
        {
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 9 } else { 8 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 17 {
                    4
                } else if self.run_state.ascension >= 2 {
                    3
                } else {
                    2
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 {
                    20
                } else {
                    16
                },
            );
            enemy
                .entity
                .set_status(crate::status_ids::sid::COUNT, missing_hp);
            enemy.entity.set_status(
                crate::status_ids::sid::HIGH_ASCENSION_AI,
                if self.run_state.ascension >= 17 { 1 } else { 0 },
            );
        }

        // Source: reference/extracted/methods/monster/GremlinNob.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "GremlinNob") {
            let (rush, bash) = if self.run_state.ascension >= 3 {
                (16, 8)
            } else {
                (14, 6)
            };
            let enrage = if self.run_state.ascension >= 18 { 3 } else { 2 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, rush);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, bash);
            enemy
                .entity
                .set_status(crate::status_ids::sid::TURN_COUNT, enrage);
            enemy
                .entity
                .set_status(crate::status_ids::sid::IS_FIRST_MOVE, 0);
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 18 {
                    18
                } else {
                    0
                },
            );
            enemy.entity.set_status(crate::status_ids::sid::ENRAGE, 0);
            enemy.set_move(crate::enemies::move_ids::NOB_BELLOW, 0, 0, 0);
            enemy.add_effect(crate::combat_types::mfx::ENRAGE, enrage as i16);
        }

        // Source: reference/extracted/methods/monster/Lagavulin.java and
        // helpers/MonsterHelper.java (`Lagavulin Event` -> Lagavulin(false)).
        for (index, enemy) in enemy_states
            .iter_mut()
            .enumerate()
            .filter(|(_, enemy)| enemy.id == "Lagavulin")
        {
            let damage = if self.run_state.ascension >= 3 {
                20
            } else {
                18
            };
            let debuff = if self.run_state.ascension >= 18 { 2 } else { 1 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, debuff);
            enemy.entity.set_status(crate::status_ids::sid::COUNT, 0);
            enemy
                .entity
                .set_status(crate::status_ids::sid::ATTACK_COUNT, 0);
            if expanded[index] == "Lagavulin Event" {
                enemy.entity.block = 0;
                enemy
                    .entity
                    .set_status(crate::status_ids::sid::IS_FIRST_MOVE, 1);
                enemy
                    .entity
                    .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
                enemy
                    .entity
                    .set_status(crate::status_ids::sid::SLEEP_TURNS, 0);
                enemy
                    .entity
                    .set_status(crate::status_ids::sid::METALLICIZE, 0);
                enemy.set_move(crate::enemies::move_ids::LAGA_SIPHON, 0, 0, 0);
            } else {
                enemy.entity.block = 8;
                enemy
                    .entity
                    .set_status(crate::status_ids::sid::IS_FIRST_MOVE, 0);
                enemy
                    .entity
                    .set_status(crate::status_ids::sid::FIRST_MOVE, 0);
                enemy
                    .entity
                    .set_status(crate::status_ids::sid::SLEEP_TURNS, 1);
                enemy
                    .entity
                    .set_status(crate::status_ids::sid::METALLICIZE, 8);
                enemy.set_move(crate::enemies::move_ids::LAGA_SLEEP, 0, 0, 0);
            }
        }

        // Source: reference/extracted/methods/monster/Sentry.java.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Sentry") {
            let damage = if self.run_state.ascension >= 3 { 10 } else { 9 };
            let daze = if self.run_state.ascension >= 18 { 3 } else { 2 };
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, damage);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, daze);
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIRST_MOVE, 0);
            enemy.entity.set_status(
                crate::status_ids::sid::IS_FIRST_MOVE,
                crate::enemies::move_ids::SENTRY_BOLT,
            );
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
            enemy
                .entity
                .set_status(crate::status_ids::sid::MODE_SHIFT, threshold);
            enemy
                .entity
                .set_status(crate::status_ids::sid::DAMAGE_TAKEN_THIS_MODE, 0);
            enemy
                .entity
                .set_status(crate::status_ids::sid::FIERCE_BASH_DMG, fierce_bash);
            enemy
                .entity
                .set_status(crate::status_ids::sid::ROLL_DMG, roll);
            enemy
                .entity
                .set_status(crate::status_ids::sid::BLOCK_AMT, 9);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, sharp_hide);
            enemy
                .entity
                .set_status(crate::status_ids::sid::TURN_COUNT, 20);
            enemy.set_move(crate::enemies::move_ids::GUARD_CHARGING_UP, 0, 0, 9);
        }

        // Source: reference/extracted/methods/monster/Hexaghost.java
        // (constructor). HP, attack damage, and A19 buff/card values use three
        // independent ascension thresholds.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "Hexaghost") {
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 19 { 3 } else { 2 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::SEAR_BURN_COUNT,
                if self.run_state.ascension >= 19 { 2 } else { 1 },
            );
            let high_damage = self.run_state.ascension >= 4;
            enemy.entity.set_status(
                crate::status_ids::sid::FIRE_TACKLE_DMG,
                if high_damage { 6 } else { 5 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::INFERNO_DMG,
                if high_damage { 3 } else { 2 },
            );
        }

        // The boss passes ascension-derived large-slime constructor values to
        // its children when its split resolves.
        for enemy in enemy_states.iter_mut().filter(|e| e.id == "SlimeBoss") {
            enemy.entity.set_status(
                crate::status_ids::sid::FIRE_TACKLE_DMG,
                if self.run_state.ascension >= 4 { 10 } else { 9 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::SLAP_DMG,
                if self.run_state.ascension >= 4 {
                    38
                } else {
                    35
                },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STR_AMT,
                if self.run_state.ascension >= 19 { 5 } else { 3 },
            );
            enemy.set_move(crate::enemies::move_ids::SB_STICKY, 0, 0, 0);
            enemy.add_effect(
                crate::combat_types::mfx::SLIMED,
                if self.run_state.ascension >= 19 { 5 } else { 3 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::STARTING_DMG,
                if self.run_state.ascension >= 2 { 1 } else { 0 },
            );
            enemy.entity.set_status(
                crate::status_ids::sid::BLOCK_AMT,
                if self.run_state.ascension >= 17 {
                    17
                } else {
                    0
                },
            );
        }

        // Java Cultist.java: ctor sets ritualAmount = ascensionLevel >= 2 ? 4 : 3;
        // takeTurn() case 3 (INCANTATION) applies RitualPower(ritualAmount + 1)
        // at ascensionLevel >= 17, else RitualPower(ritualAmount).
        // create_enemy seeds the base-3 value; patch it here where ascension is known.
        if self.run_state.ascension >= 2 {
            let ritual = if self.run_state.ascension >= 17 { 5 } else { 4 };
            for enemy in enemy_states.iter_mut().filter(|e| e.id == "Cultist") {
                enemy
                    .entity
                    .set_status(crate::status_ids::sid::STR_AMT, ritual as i32);
                enemy.add_effect(crate::combat_types::mfx::RITUAL, ritual);
            }
        }

        // Sentry.java: even encounter indices open Bolt, odd indices open Beam.
        if expanded.len() == 3 && expanded.iter().all(|id| id == "Sentry") {
            use crate::enemies::move_ids;
            for (idx, enemy) in enemy_states.iter_mut().enumerate() {
                enemy.entity.set_status(
                    crate::status_ids::sid::IS_FIRST_MOVE,
                    if idx % 2 == 0 {
                        move_ids::SENTRY_BOLT
                    } else {
                        move_ids::SENTRY_BEAM
                    },
                );
            }
        }

        // Create combat state — convert deck strings to CardInstance
        let registry = crate::cards::global_registry();
        let mut deck_instances = self.run_state.combat_deck_instances();
        if let Some(bottled) = self.run_state.bottled_flame_card_instance_id {
            if let Some(card) = deck_instances
                .iter_mut()
                .find(|card| card.instance_id == bottled)
            {
                card.flags |= crate::combat_types::CardInstance::FLAG_INNATE;
            }
        }
        if let Some(bottled) = self.run_state.bottled_lightning_card_instance_id {
            if let Some(card) = deck_instances
                .iter_mut()
                .find(|card| card.instance_id == bottled)
            {
                card.flags |= crate::combat_types::CardInstance::FLAG_INNATE;
            }
        }
        if let Some(bottled) = self.run_state.bottled_tornado_card_instance_id {
            if let Some(card) = deck_instances
                .iter_mut()
                .find(|card| card.instance_id == bottled)
            {
                card.flags |= crate::combat_types::CardInstance::FLAG_INNATE;
            }
        }
        // BustedCrown.java, CoffeeDripper.java, CursedKey.java, Ectoplasm.java,
        // FusionHammer.java, MarkOfPain.java, RunicDome.java, VelvetChoker.java, and
        // PhilosopherStone.java and Sozu.java each increment energyMaster once
        // in onEquip.
        let slavers_collar_energy = i32::from(
            self.run_state
                .relics
                .iter()
                .any(|relic| relic == "SlaversCollar")
                && expanded.iter().any(|enemy| {
                    matches!(
                        enemy.as_str(),
                        "GremlinNob"
                            | "Lagavulin"
                            | "Sentry"
                            | "BookOfStabbing"
                            | "GremlinLeader"
                            | "SlaverBoss"
                            | "TaskMaster"
                            | "Taskmaster"
                            | "Nemesis"
                            | "Reptomancer"
                            | "GiantHead"
                            | "Hexaghost"
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
                }),
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
                    .relics
                    .iter()
                    .any(|relic| relic == "Mark of Pain"),
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
                registry.card_def_by_id(card.def_id).card_type == crate::cards::CardType::Curse
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
            self.run_state.relic_flags.counters[crate::relic_flags::counter::GIRYA] as i32,
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

        if self.active_combat_has_emerald_key {
            // MonsterRoomElite applies one shared mapRng-selected buff after
            // monster pre-battle actions and before player relic setup.
            // Java: MonsterRoomElite.java::applyEmeraldEliteBuff,
            // AbstractPlayer.java::preBattlePrep.
            let buff = self.map_rng.random_int(3);
            for enemy in &mut combat_state.enemies {
                match buff {
                    0 => enemy
                        .entity
                        .add_status(crate::status_ids::sid::STRENGTH, self.run_state.act + 1),
                    1 => {
                        let increase = ((enemy.entity.max_hp as f32) * 0.25).round() as i32;
                        enemy.entity.max_hp += increase;
                        enemy.entity.hp += increase;
                    }
                    2 => enemy.entity.add_status(
                        crate::status_ids::sid::METALLICIZE,
                        self.run_state.act * 2 + 2,
                    ),
                    _ => enemy.entity.add_status(
                        crate::status_ids::sid::REGENERATION,
                        1 + self.run_state.act * 2,
                    ),
                }
            }
        }

        let mut engine = CombatEngine::new_with_rng_streams(
            combat_state,
            self.floor_rngs.combat_snapshot_with_globals(
                &self.persistent_rngs,
                self.ambient_math_rng.clone(),
                self.java_collections_rng.clone(),
            ),
        );
        engine.load_persisted_effects(self.run_state.persisted_effect_states.clone());
        engine.start_combat();

        self.combat_engine = Some(engine);
        self.phase = RunPhase::Combat;
        self.refresh_decision_stack();
    }

    fn construct_enemy_for_run(&mut self, enemy_id: &str) -> EnemyCombatState {
        let canonical_id = match enemy_id {
            // MonsterHelper maps the Dead Adventurer event key to the normal
            // Lagavulin constructor with the asleep flag disabled.
            // Java: helpers/MonsterHelper.java (`Lagavulin Event`).
            "Lagavulin Event" => "Lagavulin",
            other => other,
        };
        let (hp, max_hp) = self.roll_enemy_hp(canonical_id);
        let mut enemy = enemies::create_enemy_with_ambient(
            canonical_id,
            hp,
            max_hp,
            &mut self.ambient_math_rng,
        );

        // These values are constructor-owned and therefore interleave with HP
        // member-by-member. Later pre-battle draws such as Curl Up remain in
        // the encounter setup phase after every constructor has completed.
        if matches!(
            canonical_id,
            "FuzzyLouseNormal" | "RedLouse" | "FuzzyLouseDefensive" | "GreenLouse"
        ) {
            let bite_base = if self.run_state.ascension >= 2 { 6 } else { 5 };
            let bite_damage = bite_base + self.floor_rngs.monster_hp.random_int(2);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STARTING_DMG, bite_damage);
        } else if canonical_id == "Darkling" {
            let nip_base = if self.run_state.ascension >= 2 { 9 } else { 7 };
            let nip_damage = nip_base + self.floor_rngs.monster_hp.random_int(4);
            enemy
                .entity
                .set_status(crate::status_ids::sid::STR_AMT, nip_damage);
        }

        enemy
    }

    fn roll_fixed_set_hp(&mut self, hp: i32) -> (i32, i32) {
        // AbstractMonster.setHp(hp) delegates to setHp(hp, hp), and the latter
        // always calls monsterHpRng.random(min, max), including width one.
        let rolled = self.floor_rngs.monster_hp.random_int_range(hp, hp);
        (rolled, rolled)
    }

    fn roll_enemy_hp(&mut self, enemy_id: &str) -> (i32, i32) {
        let a20 = self.run_state.ascension >= 7;
        match enemy_id {
            "JawWorm" => {
                // Source: reference/extracted/methods/monster/JawWorm.java:
                // setHp(40,44), or setHp(42,46) at ascension 7+ (inclusive).
                let base = if a20 { 42 } else { 40 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "Cultist" => {
                // Java Cultist.java ctor: setHp(50, 56) at ascension >= 7,
                // setHp(48, 54) below — a uniform inclusive roll, not a fixed
                // value (AbstractMonster.setHp -> monsterHpRng.random(min, max)).
                let base = if a20 { 50 } else { 48 };
                let hp = base + self.floor_rngs.monster_hp.random_int(6);
                (hp, hp)
            }
            "FuzzyLouseNormal" | "RedLouse" => {
                let base = if a20 { 11 } else { 10 };
                let hp = base + self.floor_rngs.monster_hp.random_int(5);
                (hp, hp)
            }
            "FuzzyLouseDefensive" | "GreenLouse" => {
                let base = if a20 { 12 } else { 11 };
                let hp = base + self.floor_rngs.monster_hp.random_int(6);
                (hp, hp)
            }
            "AcidSlime_S" => {
                let base = if a20 { 9 } else { 8 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "AcidSlime_M" => {
                let base = if a20 { 29 } else { 28 };
                let width = if a20 { 5 } else { 4 };
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "AcidSlime_L" => {
                let base = if a20 { 68 } else { 65 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "SpikeSlime_S" => {
                let base = if a20 { 11 } else { 10 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "SpikeSlime_M" => {
                let base = if a20 { 29 } else { 28 };
                let width = if a20 { 5 } else { 4 };
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "SpikeSlime_L" => {
                let base = if a20 { 67 } else { 64 };
                let hp = base + self.floor_rngs.monster_hp.random_int(6);
                (hp, hp)
            }
            "Looter" => {
                let base = if a20 { 46 } else { 44 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "Mugger" => {
                // Source: reference/extracted/methods/monster/Mugger.java:
                // inclusive 48..52, or 50..54 at ascension 7.
                let base = if a20 { 50 } else { 48 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "GremlinFat" => {
                let base = if a20 { 14 } else { 13 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "GremlinThief" => {
                let base = if a20 { 11 } else { 10 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "GremlinWarrior" => {
                let base = if a20 { 21 } else { 20 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "GremlinWizard" => {
                let base = if a20 { 22 } else { 21 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "GremlinTsundere" => {
                let (base, width) = if a20 { (13, 4) } else { (12, 3) };
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "FungiBeast" => {
                // Source: reference/extracted/methods/monster/FungiBeast.java:
                // setHp(22,28), or setHp(24,28) at ascension 7+ (inclusive).
                let base = if a20 { 24 } else { 22 };
                let hp = base + self.floor_rngs.monster_hp.random_int(28 - base);
                (hp, hp)
            }
            "BlueSlaver" | "SlaverBlue" => {
                let base = if a20 { 48 } else { 46 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "RedSlaver" | "SlaverRed" => {
                let base = if a20 { 48 } else { 46 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "BanditBear" | "Bear" => {
                let base = if a20 { 40 } else { 38 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "BanditChild" | "BanditPointy" | "Pointy" => {
                let hp = if a20 { 34 } else { 30 };
                self.roll_fixed_set_hp(hp)
            }
            "BanditLeader" => {
                let base = if a20 { 37 } else { 35 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "GremlinNob" => {
                let (base, width) = if self.run_state.ascension >= 8 {
                    (85, 5)
                } else {
                    (82, 4)
                };
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "Lagavulin" => {
                let (base, width) = if self.run_state.ascension >= 8 {
                    (112, 3)
                } else {
                    (109, 2)
                };
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "Sentry" => {
                let (base, width) = if self.run_state.ascension >= 8 {
                    (39, 6)
                } else {
                    (38, 4)
                };
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "TheGuardian" => {
                // Source: TheGuardian.java constructor: HP changes at A9,
                // independently of the ordinary-enemy A7 HP threshold.
                let hp = if self.run_state.ascension >= 9 {
                    250
                } else {
                    240
                };
                self.roll_fixed_set_hp(hp)
            }
            "Hexaghost" => {
                // Source: Hexaghost.java constructor: HP changes at A9.
                let hp = if self.run_state.ascension >= 9 {
                    264
                } else {
                    250
                };
                self.roll_fixed_set_hp(hp)
            }
            "SlimeBoss" => {
                // Source: SlimeBoss.java constructor: HP changes at A9.
                let hp = if self.run_state.ascension >= 9 {
                    150
                } else {
                    140
                };
                self.roll_fixed_set_hp(hp)
            }
            "Apology Slime" | "ApologySlime" => {
                // Source: ApologySlime.java constructor: monsterHpRng.random(8, 12).
                let hp = self.floor_rngs.monster_hp.random_int_range(8, 12);
                (hp, hp)
            }
            // Act 2 enemies
            "Byrd" => {
                // Source: reference/extracted/methods/monster/Byrd.java:
                // setHp(25,31), or setHp(26,33) at ascension 7+.
                let (base, width) = if a20 { (26, 7) } else { (25, 6) };
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "Chosen" => {
                // Source: Chosen.java constructor: inclusive setHp(95,99),
                // or setHp(98,103) at ascension 7+.
                let (base, width) = if a20 { (98, 5) } else { (95, 4) };
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
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
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "SnakePlant" => {
                // Source: reference/extracted/methods/monster/SnakePlant.java:
                // inclusive 75..79, or 78..82 at ascension 7+.
                let base = if a20 { 78 } else { 75 };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "Centurion" => {
                // Source: reference/extracted/methods/monster/Centurion.java:
                // setHp(76,80), or setHp(78,83) at ascension 7+.
                let (base, width) = if a20 { (78, 5) } else { (76, 4) };
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "Healer" | "Mystic" => {
                // Source: reference/extracted/methods/monster/Healer.java:
                // inclusive 48..56, or 50..58 at ascension 7.
                let base = if self.run_state.ascension >= 7 {
                    50
                } else {
                    48
                };
                let hp = base + self.floor_rngs.monster_hp.random_int(8);
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
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "BookOfStabbing" => {
                let base = if self.run_state.ascension >= 8 {
                    168
                } else {
                    160
                };
                let hp = base + self.floor_rngs.monster_hp.random_int(4);
                (hp, hp)
            }
            "SlaverBoss" | "TaskMaster" | "Taskmaster" => {
                // Taskmaster's constructor rolls 54..60 for `super`, then
                // `setHp` immediately replaces it with the ascension range.
                // Both calls consume monsterHpRng even below A8.
                // Source: decompiled/.../monsters/city/Taskmaster.java.
                let _discarded_constructor_hp = self.floor_rngs.monster_hp.random_int_range(54, 60);
                let (base, width) = if self.run_state.ascension >= 8 {
                    (57, 7)
                } else {
                    (54, 6)
                };
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
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
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "SphericGuardian" | "Spheric Guardian" => {
                // Source: reference/extracted/methods/monster/SphericGuardian.java:
                // the constructor passes a fixed 20 HP and never calls setHp.
                (20, 20)
            }
            "BronzeAutomaton" => {
                let hp = if self.run_state.ascension >= 9 {
                    320
                } else {
                    300
                };
                self.roll_fixed_set_hp(hp)
            }
            "BronzeOrb" | "Bronze Orb" => {
                // BronzeOrb rolls once for `super`, then again in `setHp`.
                // Source: decompiled/.../monsters/city/BronzeOrb.java.
                let _discarded_constructor_hp = self.floor_rngs.monster_hp.random_int_range(52, 58);
                let base = if self.run_state.ascension >= 9 {
                    54
                } else {
                    52
                };
                let hp = base + self.floor_rngs.monster_hp.random_int(6);
                (hp, hp)
            }
            "TorchHead" | "Torch Head" => {
                // Source: reference/extracted/methods/monster/TorchHead.java:
                // `super` consumes 38..40 before `setHp` consumes the final
                // inclusive 38..40, raised to 40..45 at A9.
                let _discarded_constructor_hp = self.floor_rngs.monster_hp.random_int_range(38, 40);
                let (base, width) = if self.run_state.ascension >= 9 {
                    (40, 5)
                } else {
                    (38, 2)
                };
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "TheCollector" => {
                // Source: reference/extracted/methods/monster/TheCollector.java:
                // fixed 282 HP, raised to fixed 300 at ascension 9.
                let hp = if self.run_state.ascension >= 9 {
                    300
                } else {
                    282
                };
                self.roll_fixed_set_hp(hp)
            }
            "Champ" | "TheChamp" => {
                // Source: Champ.java changes boss HP at A9, not the ordinary
                // monster A7 threshold represented by `a20` above.
                let hp = if self.run_state.ascension >= 9 {
                    440
                } else {
                    420
                };
                self.roll_fixed_set_hp(hp)
            }
            // Act 3 enemies
            "Darkling" => {
                // Source: Darkling.java uses inclusive 48..56 / 50..59 rolls.
                let (base, width) = if a20 { (50, 9) } else { (48, 8) };
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "OrbWalker" | "Orb Walker" => {
                // Source: reference/extracted/methods/monster/OrbWalker.java:
                // `super` consumes 90..96 before `setHp` consumes the final
                // inclusive 90..96, or 92..102 at ascension 7.
                let _discarded_constructor_hp = self.floor_rngs.monster_hp.random_int_range(90, 96);
                let (base, width) = if self.run_state.ascension >= 7 {
                    (92, 10)
                } else {
                    (90, 6)
                };
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
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
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
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
                let hp = base + self.floor_rngs.monster_hp.random_int(width);
                (hp, hp)
            }
            "Exploder" => {
                // Source: reference/extracted/methods/monster/Exploder.java.
                // setHp always consumes one draw, including A0's 30..30.
                if self.run_state.ascension >= 7 {
                    let hp = self.floor_rngs.monster_hp.random_int_range(30, 35);
                    (hp, hp)
                } else {
                    self.roll_fixed_set_hp(30)
                }
            }
            "WrithingMass" => {
                // Source: WrithingMass.java uses fixed 160, raised at A7.
                let hp = if self.run_state.ascension >= 7 {
                    175
                } else {
                    160
                };
                self.roll_fixed_set_hp(hp)
            }
            "GiantHead" => {
                // Source: reference/extracted/methods/monster/GiantHead.java:
                // the fixed HP increase is at ascension 8.
                let hp = if self.run_state.ascension >= 8 {
                    520
                } else {
                    500
                };
                self.roll_fixed_set_hp(hp)
            }
            "Nemesis" => {
                // Source: reference/extracted/methods/monster/Nemesis.java:
                // fixed HP changes at ascension 8, not ascension 7.
                let hp = if self.run_state.ascension >= 8 {
                    200
                } else {
                    185
                };
                self.roll_fixed_set_hp(hp)
            }
            "Reptomancer" => {
                // Source: Reptomancer.java: inclusive 180..190, or 190..200
                // at ascension 8. The constructor's earlier 180..190 roll is
                // overwritten by `setHp`, but still consumes monsterHpRng.
                let _discarded_constructor_hp =
                    self.floor_rngs.monster_hp.random_int_range(180, 190);
                let base = if self.run_state.ascension >= 8 {
                    190
                } else {
                    180
                };
                let hp = base + self.floor_rngs.monster_hp.random_int(10);
                (hp, hp)
            }
            "Dagger" => {
                // Source: reference/extracted/methods/monster/SnakeDagger.java.
                let hp = 20 + self.floor_rngs.monster_hp.random_int(5);
                (hp, hp)
            }
            "Transient" => {
                // Source: reference/extracted/methods/monster/Transient.java:
                // fixed 999 HP at every ascension.
                (999, 999)
            }
            "Maw" => {
                // Source: reference/extracted/methods/monster/Maw.java: the
                // constructor passes fixed 300 HP at every ascension.
                (300, 300)
            }
            "Serpent" | "SpireGrowth" | "Spire Growth" => {
                // Source: SpireGrowth.java: fixed 170 HP, or 190 at A7.
                let hp = if self.run_state.ascension >= 7 {
                    190
                } else {
                    170
                };
                self.roll_fixed_set_hp(hp)
            }
            "AwakenedOne" | "Awakened One" => {
                let hp = if self.run_state.ascension >= 9 {
                    320
                } else {
                    300
                };
                self.roll_fixed_set_hp(hp)
            }
            "TimeEater" => {
                // Source: reference/extracted/methods/monster/TimeEater.java:
                // fixed 456 HP, raised to fixed 480 at ascension 9.
                let hp = if self.run_state.ascension >= 9 {
                    480
                } else {
                    456
                };
                self.roll_fixed_set_hp(hp)
            }
            "DonuAndDeca" | "Donu" | "Deca" => {
                // Source: Deca.java (and Donu.java) changes HP at A9.
                let hp = if self.run_state.ascension >= 9 {
                    265
                } else {
                    250
                };
                self.roll_fixed_set_hp(hp)
            }
            // Act 4 enemies
            "SpireShield" | "Spire Shield" => {
                // Source: reference/extracted/methods/monster/SpireShield.java:
                // fixed 110 HP, raised to fixed 125 at ascension 8.
                let hp = if self.run_state.ascension >= 8 {
                    125
                } else {
                    110
                };
                self.roll_fixed_set_hp(hp)
            }
            "SpireSpear" | "Spire Spear" => {
                // Source: reference/extracted/methods/monster/SpireSpear.java:
                // fixed 160 HP, raised to fixed 180 at ascension 8.
                let hp = if self.run_state.ascension >= 8 {
                    180
                } else {
                    160
                };
                self.roll_fixed_set_hp(hp)
            }
            "CorruptHeart" => {
                // Source: CorruptHeart.java changes max HP at A9.
                let hp = if self.run_state.ascension >= 9 {
                    800
                } else {
                    750
                };
                self.roll_fixed_set_hp(hp)
            }
            _ => (40, 40),
        }
    }

    fn step_combat(&mut self, action: &GameAction) {
        let combat_action = match action {
            GameAction::CombatAction(a) => a.clone(),
            _ => return,
        };

        let engine = match self.combat_engine.as_mut() {
            Some(e) => e,
            None => return,
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
        engine.execute_action(&combat_action);
        // cardRng and potionRng remain persistent; the other five streams are
        // floor-owned. Return the complete combat snapshot atomically.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:397,422,1737-1741
        let combat_rngs = engine.rng_snapshot();
        let (ambient_math_rng, java_collections_rng) =
            combat_rngs.absorb_into(&mut self.persistent_rngs, &mut self.floor_rngs);
        self.ambient_math_rng = ambient_math_rng;
        self.java_collections_rng = java_collections_rng;
        self.run_state.gold = engine.state.run_gold;

        if engine.is_combat_over() {
            let completed_room_type = self
                .active_combat_room_type
                .take()
                .expect("active combat must retain its resolved room type");
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
                self.run_state.next_card_instance_id = engine.state.next_card_instance_id;
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
                return;
            }
            if engine.state.player_won {
                // Update run state from combat result
                self.run_state.current_hp = engine.state.player.hp;
                self.run_state.max_hp = engine.state.player.max_hp;
                let recovered_stolen_gold: i32 = engine
                    .state
                    .enemies
                    .iter()
                    .filter(|enemy| {
                        matches!(enemy.id.as_str(), "Looter" | "Mugger")
                            && enemy.entity.hp <= 0
                            && !enemy.is_escaping
                    })
                    .map(|enemy| enemy.entity.status(crate::status_ids::sid::COUNT))
                    .sum();
                self.run_state.potions = engine.state.potions.clone();
                self.run_state.deck = engine
                    .state
                    .master_deck
                    .iter()
                    .map(|card| engine.card_registry.card_name(card.def_id).to_string())
                    .collect();
                self.run_state.deck_card_states = engine.state.master_deck.clone();
                self.run_state.next_card_instance_id = engine.state.next_card_instance_id;
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
                        self.start_event_combat(resolved_enemies);
                    } else if start_boss_combat {
                        self.current_event = None;
                        self.pending_event_combat = None;
                        self.run_state.map_x = -1;
                        self.run_state.map_y = 15;
                        self.run_state.floor += 1;
                        self.reset_floor_rngs();
                        self.enter_combat(false, true);
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
                    return;
                }

                let room_type = completed_room_type;
                let is_boss = room_type == RoomType::Boss;
                let is_final_heart = self.run_state.act == 4
                    && combat_enemy_ids.len() == 1
                    && combat_enemy_ids[0] == "CorruptHeart";
                // AbstractRoom.addGoldToRewards constructs the boss gold item
                // before MonsterRoomBoss suppresses the Act 3/4 reward screen.
                // Those final-act values are discarded, but miscRng still
                // advances once for every boss victory.
                // Java: AbstractRoom.java:286-331.
                if is_boss && self.run_state.act >= 3 {
                    let _hidden_gold = self.roll_boss_gold_reward();
                }

                if is_boss && self.run_state.act == 3 {
                    self.run_state.bosses_killed += 1;
                    let continuation =
                        if self.run_state.ascension >= 20 && self.boss_sequence.len() == 2 {
                            RunContinuation::ActThreeSecondBoss
                        } else {
                            RunContinuation::SpireHeart
                        };
                    // MonsterRoomBoss is complete now; Proceed owns the next
                    // floor increment, five floor-stream resets, and room entry.
                    // Java: ProceedButton.java:100-113,199-219.
                    self.combat_engine = None;
                    self.wait_for_continuation(continuation);
                    self.refresh_decision_stack();
                    return;
                }

                if room_type == RoomType::Elite {
                    self.run_state.elites_killed += 1;
                }

                // Check if boss
                if is_boss {
                    self.run_state.bosses_killed += 1;
                    if is_final_heart {
                        self.combat_engine = None;
                        self.wait_for_continuation(RunContinuation::TrueVictory);
                        return;
                    }
                    self.build_combat_reward_screen(RoomType::Boss, recovered_stolen_gold);
                    self.combat_engine = None;
                    self.phase = RunPhase::CardReward;
                    return;
                }

                // Build the ordered post-combat reward screen.
                self.build_combat_reward_screen(room_type, recovered_stolen_gold);
                self.combat_engine = None;
                self.phase = RunPhase::CardReward;
            } else {
                // Player died
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
            self.last_combat_events = engine.take_event_log();
        }
    }

    fn generate_card_reward_choices(
        &mut self,
        count: usize,
        context: CardRewardContext,
    ) -> Vec<RewardChoice> {
        // AbstractDungeon.getRewardCards first rolls every rarity/card pair,
        // retrying duplicate IDs, and only afterward performs natural upgrade
        // rolls and relic preview callbacks over the copied result list.
        // Java: AbstractDungeon.java:1413-1468.
        let mut selected = Vec::<(String, EventCardRarity)>::new();
        let prismatic = self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::PRISMATIC_SHARD);
        let (base_rare, base_uncommon) = context.base_chances();
        let rare_chance = if context.applies_relic_rarity_modifiers()
            && self
                .run_state
                .relics
                .iter()
                .any(|relic| relic == "Nloth's Gift")
        {
            base_rare * 3
        } else {
            base_rare
        };

        for _ in 0..count {
            let roll =
                self.persistent_rngs.card.random_int(99) + self.run_state.card_blizz_randomizer;
            let rarity = if matches!(context, CardRewardContext::Boss) || roll < rare_chance {
                EventCardRarity::Rare
            } else if roll < rare_chance + base_uncommon {
                EventCardRarity::Uncommon
            } else {
                EventCardRarity::Common
            };
            match rarity {
                EventCardRarity::Rare => self.run_state.card_blizz_randomizer = 5,
                EventCardRarity::Common => {
                    self.run_state.card_blizz_randomizer =
                        (self.run_state.card_blizz_randomizer - 1).max(-40);
                }
                EventCardRarity::Uncommon => {}
                _ => unreachable!("reward rarity is common, uncommon, or rare"),
            }

            let card = loop {
                let candidate = if prismatic {
                    // CardLibrary.getAnyColorCard constructs all colored and
                    // colorless candidates, shuffles through randomLong(), then
                    // CardGroup.getRandomCard filters and sorts by card ID before
                    // the inclusive selection draw. The shuffle cannot change
                    // the sorted result, but its wrapper draw remains causal.
                    self.persistent_rngs.card.random_long_unbounded();
                    let candidates = prismatic_reward_candidates(rarity);
                    let java_id =
                        &candidates[self.persistent_rngs.card.random_index(candidates.len())];
                    runtime_card_id_for_java_id(java_id).to_string()
                } else {
                    let pool = self.card_pools.working(rarity);
                    pool[self.persistent_rngs.card.random_index(pool.len())].clone()
                };
                if selected.iter().all(|(chosen, _)| chosen != &candidate) {
                    break candidate;
                }
            };
            selected.push((card, rarity));
        }

        let upgrade_chance = match self.run_state.act {
            1 => 0.0,
            2 => {
                if self.run_state.ascension >= 12 {
                    0.125
                } else {
                    0.25
                }
            }
            _ => {
                if self.run_state.ascension >= 12 {
                    0.25
                } else {
                    0.5
                }
            }
        };
        let registry = crate::cards::global_registry();
        selected
            .into_iter()
            .enumerate()
            .map(|(index, (card, rarity))| {
                let naturally_upgraded = rarity != EventCardRarity::Rare
                    && self.persistent_rngs.card.random_bool_chance(upgrade_chance)
                    && registry.can_upgrade_name(&card);
                let card_id = if naturally_upgraded {
                    format!("{card}+")
                } else {
                    self.upgrade_reward_card_if_needed(&card)
                };
                RewardChoice::Card { index, card_id }
            })
            .collect()
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

    fn generate_colorless_reward_choices(&mut self, count: usize) -> Vec<RewardChoice> {
        // AbstractDungeon.getColorlessRewardCards rolls rare/uncommon at 30%
        // with cardRng, then selects from the matching colorless pool with the
        // same stream and retries duplicate identities. It performs no natural
        // upgrade roll; RewardItem preview relic callbacks are represented by
        // upgrade_reward_card_if_needed.
        // Java: AbstractDungeon.java::getColorlessRewardCards.
        let mut selected_base = Vec::<String>::with_capacity(count);
        let mut choices = Vec::with_capacity(count);
        for index in 0..count {
            let rare = self.persistent_rngs.card.random_bool_chance(0.3);
            let pool = if rare {
                SHOP_COLORLESS_RARE_CARDS
            } else {
                SHOP_COLORLESS_UNCOMMON_CARDS
            };
            let card = loop {
                let candidate = pool[self.persistent_rngs.card.random_index(pool.len())];
                if selected_base.iter().all(|chosen| chosen != candidate) {
                    break candidate.to_string();
                }
            };
            let card_id = self.upgrade_reward_card_if_needed(&card);
            selected_base.push(card);
            choices.push(RewardChoice::Card { index, card_id });
        }
        choices
    }

    fn build_dream_catcher_reward_screen(&mut self) {
        // CampfireSleepEffect.java opens AbstractDungeon.getRewardCards after
        // resting with Dream Catcher, so normal reward-count relic callbacks
        // (Question Card and Busted Crown) still apply.
        let choices = self
            .generate_card_reward_choices(self.card_reward_choice_count(), CardRewardContext::Rest);
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
        let tier = self.roll_relic_tier();
        self.draw_relic_from_pool(tier, false, false)
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

    fn build_combat_reward_screen(&mut self, room_type: RoomType, stolen_gold: i32) {
        let base_gold = if room_type == RoomType::Boss {
            self.roll_boss_gold_reward()
        } else if room_type == RoomType::Elite {
            self.persistent_rngs.treasure.random_int_range(25, 35)
        } else {
            self.persistent_rngs.treasure.random_int_range(10, 20)
        };
        let gold = golden_idol_combat_gold(
            base_gold,
            self.run_state
                .relic_flags
                .has(crate::relic_flags::flag::GOLDEN_IDOL),
        );
        // Looter/Mugger death appends a distinct reward before AbstractRoom
        // generates ordinary combat rewards. The theft is not refunded until
        // the player claims this item.
        // Java: AbstractRoom.java:631-638, Looter.java:174-187,
        // Mugger.java:167-178, RewardItem.java:104-111.
        let mut items = Vec::new();
        if stolen_gold > 0 {
            items.push(RewardItem {
                index: items.len(),
                kind: RewardItemKind::StolenGold,
                state: RewardItemState::Available,
                label: stolen_gold.to_string(),
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
            claimable: false,
            active: false,
            skip_allowed: false,
            skip_label: None,
            choices: Vec::new(),
        });

        if room_type == RoomType::Elite {
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
            if self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::BLACK_STAR)
            {
                items.push(RewardItem {
                    index: items.len(),
                    kind: RewardItemKind::Relic,
                    state: RewardItemState::Available,
                    label: self.roll_noncamp_reward_relic_id(),
                    claimable: false,
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: Vec::new(),
                });
            }
            if self.active_combat_has_emerald_key {
                items.push(RewardItem {
                    index: items.len(),
                    kind: RewardItemKind::Key {
                        color: RewardKeyColor::Emerald,
                        linked_item_index: None,
                    },
                    state: RewardItemState::Available,
                    label: "Emerald Key".to_string(),
                    claimable: false,
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: Vec::new(),
                });
            }
        }

        if self.should_offer_potion_reward(room_type, items.len()) {
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
                choices: self.generate_card_reward_choices(
                    card_choice_count,
                    if room_type == RoomType::Elite {
                        CardRewardContext::Elite
                    } else if room_type == RoomType::Boss {
                        CardRewardContext::Boss
                    } else {
                        CardRewardContext::Standard
                    },
                ),
            });
        }

        let mut screen = RewardScreen {
            source: if room_type == RoomType::Boss {
                RewardScreenSource::BossCombat
            } else {
                RewardScreenSource::Combat
            },
            ordered: false,
            active_item: None,
            items,
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
    }

    fn roll_boss_gold_reward(&mut self) -> i32 {
        let rolled = 100 + self.floor_rngs.misc.random_int_range(-5, 5);
        if self.run_state.ascension >= 13 {
            ((rolled as f32) * 0.75).round() as i32
        } else {
            rolled
        }
    }

    fn enter_boss_chest(&mut self) {
        // BossChest constructs and removes its three persistent-pool relics
        // when the treasure room is entered, before the chest is opened.
        // Closing the boss-relic screen reuses these same instances.
        // Java: BossChest.java::<init>, BossRelicSelectScreen::relicSkipLogic.
        let relic_choices = self.roll_boss_relic_choices(3);
        self.reward_screen = None;
        self.pending_chest = Some(PendingChest::Boss { relic_choices });
        self.phase = RunPhase::Chest;
    }

    fn build_boss_reward_screen(&mut self, relic_choices: Vec<String>) {
        self.pending_relic_followup_source = RewardScreenSource::BossCombat;
        let choices = relic_choices
            .into_iter()
            .enumerate()
            .map(|(index, relic_id)| RewardChoice::Named {
                index,
                label: relic_id,
            })
            .collect();

        let mut screen = RewardScreen {
            source: RewardScreenSource::BossRelic,
            ordered: true,
            active_item: None,
            items: vec![RewardItem {
                index: 0,
                kind: RewardItemKind::Relic,
                state: RewardItemState::Available,
                label: "boss_relic_reward".to_string(),
                claimable: true,
                active: false,
                skip_allowed: true,
                skip_label: Some("Skip".to_string()),
                choices,
            }],
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
    }

    fn enter_treasure_room(&mut self) {
        // TreasureRoom.onPlayerEntry creates and randomizes the chest. The
        // chest click and the room's Proceed button remain separate actions.
        // Java: TreasureRoom.java::onPlayerEntry, AbstractChest.java::open.
        self.reward_screen = None;
        self.pending_chest = Some(PendingChest::Treasure(self.generate_chest()));
        self.phase = RunPhase::Chest;
    }

    fn step_chest(&mut self, action: &GameAction) {
        match action {
            GameAction::OpenChest => {
                let Some(chest) = self.pending_chest.clone() else {
                    return;
                };
                match chest {
                    PendingChest::Treasure(chest) => {
                        self.pending_chest = None;
                        self.build_treasure_reward_screen_for(chest)
                    }
                    PendingChest::Boss { relic_choices } => {
                        self.build_boss_reward_screen(relic_choices)
                    }
                }
                self.phase = RunPhase::CardReward;
            }
            GameAction::LeaveChest => {
                let pending = self.pending_chest.take();
                self.reward_screen = None;
                if matches!(pending, Some(PendingChest::Boss { .. })) {
                    if self.run_state.act < 3 {
                        self.wait_for_continuation(RunContinuation::NextAct);
                    } else {
                        self.wait_for_continuation(RunContinuation::SpireHeart);
                    }
                } else {
                    self.phase = RunPhase::MapChoice;
                }
            }
            _ => {}
        }
        ()
    }

    #[cfg(test)]
    fn build_treasure_reward_screen(&mut self) {
        let chest = self.generate_chest();
        self.build_treasure_reward_screen_for(chest);
    }

    fn build_treasure_reward_screen_for(&mut self, chest: GeneratedChest) {
        let extra_relic =
            self.run_state.relic_flags.counters[crate::relic_flags::counter::MATRYOSHKA_USES] > 0;
        if extra_relic {
            let counter = &mut self.run_state.relic_flags.counters
                [crate::relic_flags::counter::MATRYOSHKA_USES];
            *counter -= 1;
            if *counter == 0 {
                *counter = -2;
            }
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

        if chest.gold_reward {
            let base = chest.base_gold() as f32;
            let gold = self
                .persistent_rngs
                .treasure
                .random_f32_range(base * 0.9, base * 1.1)
                .round() as i32;
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
        }

        // CursedKey.onChestOpen marks the chest cursed. AbstractChest.open
        // consumes the random curse after optional gold generation and before
        // adding the chest relic reward.
        // Java: CursedKey.java, AbstractChest.java:65-95.
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::CURSED_KEY)
        {
            let curse = RANDOM_OBTAINABLE_CURSES[self
                .persistent_rngs
                .card
                .random_index(RANDOM_OBTAINABLE_CURSES.len())]
            .to_string();
            obtain_master_deck_card_state(&mut self.run_state, curse);
        }

        let mut chest_relic_index = Some(items.len());
        items.push(RewardItem {
            index: items.len(),
            kind: RewardItemKind::Relic,
            state: RewardItemState::Available,
            label: self.draw_relic_from_pool(chest.relic_tier, false, false),
            claimable: false,
            active: false,
            skip_allowed: false,
            skip_label: None,
            choices: Vec::new(),
        });

        if self.profile.final_act_available && !self.run_state.has_sapphire_key {
            items.push(RewardItem {
                index: items.len(),
                kind: RewardItemKind::Key {
                    color: RewardKeyColor::Sapphire,
                    linked_item_index: chest_relic_index,
                },
                state: RewardItemState::Available,
                label: "Sapphire Key".to_string(),
                claimable: false,
                active: false,
                skip_allowed: false,
                skip_label: None,
                choices: Vec::new(),
            });
        }

        let has_nloths_mask = self
            .run_state
            .relics
            .iter()
            .any(|relic| relic == "NlothsMask");
        if has_nloths_mask
            && self.run_state.relic_flags.counters[crate::relic_flags::counter::NLOTHS_MASK] > 0
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
                if items.get(index).is_some_and(|item| {
                    matches!(
                        item.kind,
                        RewardItemKind::Key {
                            color: RewardKeyColor::Sapphire,
                            linked_item_index: Some(linked),
                        } if linked == index
                    )
                }) {
                    items.remove(index);
                }
                chest_relic_index = chest_relic_index.and_then(|linked| {
                    if index == linked {
                        None
                    } else if index < linked {
                        Some(linked - 1)
                    } else {
                        Some(linked)
                    }
                });
            }
        }

        for item in &mut items {
            if let RewardItemKind::Key {
                linked_item_index: Some(linked),
                ..
            } = &mut item.kind
            {
                *linked = chest_relic_index.expect("surviving Sapphire key must link to relic");
            }
        }

        for (index, item) in items.iter_mut().enumerate() {
            item.index = index;
        }

        let mut screen = RewardScreen {
            source: RewardScreenSource::Treasure,
            ordered: false,
            active_item: None,
            items,
        };
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
    }

    fn generate_chest(&mut self) -> GeneratedChest {
        // Re-verified: getRandomChest consumes one inclusive 0..99 roll, then
        // the selected chest constructor consumes one correlated gold/tier
        // roll. Gold amount is deferred until open.
        // Java: AbstractDungeon.java::getRandomChest,
        // AbstractChest.java::randomizeReward.
        let size = match self.persistent_rngs.treasure.random_int(99) {
            0..=49 => ChestSize::Small,
            50..=82 => ChestSize::Medium,
            _ => ChestSize::Large,
        };
        let reward_roll = self.persistent_rngs.treasure.random_int(99);
        let (gold_reward, relic_tier) = match size {
            ChestSize::Small => (
                reward_roll < 50,
                if reward_roll < 75 {
                    RelicTier::Common
                } else {
                    RelicTier::Uncommon
                },
            ),
            ChestSize::Medium => (
                reward_roll < 35,
                if reward_roll < 35 {
                    RelicTier::Common
                } else if reward_roll < 85 {
                    RelicTier::Uncommon
                } else {
                    RelicTier::Rare
                },
            ),
            ChestSize::Large => (
                reward_roll < 50,
                if reward_roll < 75 {
                    RelicTier::Uncommon
                } else {
                    RelicTier::Rare
                },
            ),
        };
        GeneratedChest {
            size,
            gold_reward,
            relic_tier,
        }
    }

    // =======================================================================
    // Card reward step
    // =======================================================================

    fn step_card_reward(&mut self, action: &GameAction) {
        let leave_requested = matches!(action, GameAction::LeaveRewards);
        match action {
            GameAction::SelectRewardItem(item_index) => self.select_reward_item(*item_index),
            GameAction::ChooseRewardOption {
                item_index,
                choice_index,
            } => self.choose_reward_option(*item_index, *choice_index),
            GameAction::SkipRewardItem(item_index) => self.skip_reward_item(*item_index),
            GameAction::LeaveRewards => {
                if let Some(screen) = self.reward_screen.as_mut() {
                    for item in &mut screen.items {
                        if item.state == RewardItemState::Available {
                            item.state = RewardItemState::Skipped;
                        }
                    }
                }
            }
            _ => {}
        }

        if self.phase != RunPhase::CardReward {
            self.refresh_decision_stack();
            return;
        }

        let reward_complete = self.reward_screen.as_ref().is_none_or(|screen| {
            leave_requested
                || (screen.ordered
                    && screen
                        .items
                        .iter()
                        .all(|item| item.state != RewardItemState::Available))
        });
        if reward_complete {
            let source = self
                .reward_screen
                .as_ref()
                .map(|screen| screen.source.clone())
                .unwrap_or(RewardScreenSource::Unknown);
            self.reward_screen = None;

            if source == RewardScreenSource::BossCombat {
                self.enter_boss_treasure_room();
                self.enter_boss_chest();
                self.refresh_decision_stack();
                return;
            }

            if source == RewardScreenSource::BossRelic {
                self.pending_chest = None;
                if self.run_state.act < 3 {
                    self.wait_for_continuation(RunContinuation::NextAct);
                } else {
                    self.wait_for_continuation(RunContinuation::SpireHeart);
                }
                self.refresh_decision_stack();
                return;
            }

            if source == RewardScreenSource::Shop {
                self.phase = RunPhase::Shop;
                self.refresh_decision_stack();
                return;
            }

            if self.neow_frame == Some(NeowFrame::Exit) && self.neow_selection_witness.is_some() {
                // Neow's screen 99 remains underneath card, potion, and grid
                // overlays. Completing the overlay returns to its final Proceed.
                self.pending_neow_deck_selection = None;
                self.phase = RunPhase::Neow;
                self.refresh_decision_stack();
                return;
            }

            // A top-row reward overlay returns to Java's synthetic boss click.
            if self.run_state.map_y >= 0 && self.run_state.map_y as usize >= self.map.height - 1 {
                self.phase = RunPhase::MapChoice;
                self.refresh_decision_stack();
                return;
            }

            self.phase = RunPhase::MapChoice;
        }
        self.refresh_decision_stack();
        ()
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
            self.decision_stack
                .push(DecisionFrame::RewardChoice(RewardChoiceFrame {
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
        let screen_source = screen.source.clone();
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
        let neow_deck_card_pick = matches!(
            item_label.as_str(),
            "deck_selection_neow_remove"
                | "deck_selection_neow_upgrade"
                | "deck_selection_neow_transform"
        );

        match (kind, choice) {
            (RewardItemKind::CardChoice, RewardChoice::Card { index, card_id }) => {
                if bottled_tornado_card_pick {
                    self.run_state.reconcile_deck_card_states();
                    self.run_state.bottled_tornado_card_instance_id = self
                        .run_state
                        .deck_card_states
                        .get(index)
                        .map(|card| card.instance_id);
                    self.pending_bottled_tornado_selection = false;
                } else if bottled_lightning_card_pick {
                    self.run_state.reconcile_deck_card_states();
                    self.run_state.bottled_lightning_card_instance_id = self
                        .run_state
                        .deck_card_states
                        .get(index)
                        .map(|card| card.instance_id);
                    self.pending_bottled_lightning_selection = false;
                } else if bottled_flame_card_pick {
                    self.run_state.reconcile_deck_card_states();
                    self.run_state.bottled_flame_card_instance_id = self
                        .run_state
                        .deck_card_states
                        .get(index)
                        .map(|card| card.instance_id);
                    self.pending_bottled_flame_selection = false;
                } else if astrolabe_card_pick {
                    if let Some(RewardChoice::Card { index, .. }) =
                        choice_frame.choices.get(choice_index)
                    {
                        if let Some(removed) = self.remove_master_deck_card(*index) {
                            self.pending_astrolabe_removed.push(removed);
                        }
                    }
                } else if !self.resolve_event_deck_selection_choice(
                    &item_label,
                    &choice_frame,
                    choice_index,
                ) {
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
                self.pending_relic_followup_source = screen_source;
                self.add_relic_reward(&label);
            }
            (RewardItemKind::Potion, RewardChoice::Named { label, .. }) => {
                self.add_potion_reward(&label);
            }
            (
                RewardItemKind::Gold | RewardItemKind::StolenGold,
                RewardChoice::Named { label, .. },
            ) => {
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
        } else if neow_deck_card_pick {
            if self
                .pending_neow_deck_selection
                .is_some_and(|pending| pending.remaining > 0)
            {
                self.build_neow_deck_selection_screen();
            } else {
                self.pending_neow_deck_selection = None;
            }
        }
        let event_selection_remaining = self
            .pending_event_deck_selection
            .as_ref()
            .filter(|pending| pending.label == item_label)
            .map(|pending| pending.remaining);
        match event_selection_remaining {
            Some(remaining) if remaining > 0 => {
                self.build_pending_event_deck_selection_screen();
            }
            Some(_) => {
                self.pending_event_deck_selection = None;
            }
            None => {}
        }
        if (bottled_flame_card_pick || bottled_lightning_card_pick || bottled_tornado_card_pick)
            && self.suspended_reward_screen.is_some()
        {
            self.reward_screen = self.suspended_reward_screen.take();
        }
    }

    fn event_deck_selection_item(
        &self,
        label: &str,
        index: usize,
        claimable: bool,
    ) -> Option<RewardItem> {
        let restrict_to_removable = matches!(
            label,
            "deck_selection_purge"
                | "deck_selection_event_remove"
                | "deck_selection_event_transform"
                | "deck_selection_designer_full"
        );
        let restrict_to_upgradeable = label == "deck_selection_event_upgrade";
        let mut choices = self
            .run_state
            .deck
            .iter()
            .enumerate()
            .filter(|(deck_index, _)| {
                (!restrict_to_removable || self.is_removable_master_deck_index(*deck_index))
                    && (!restrict_to_upgradeable
                        || crate::cards::global_registry()
                            .can_upgrade_name(&self.run_state.deck[*deck_index]))
            })
            .map(|(deck_index, card_id)| RewardChoice::Card {
                index: deck_index,
                card_id: card_id.clone(),
            })
            .collect::<Vec<_>>();
        if choices.is_empty() {
            return None;
        }
        if label == "deck_selection_note_for_yourself" {
            let reward_card =
                self.upgrade_reward_card_if_needed(self.current_note_for_yourself_card());
            choices.push(RewardChoice::Card {
                index: choices.len(),
                card_id: reward_card,
            });
        }
        Some(RewardItem {
            index,
            kind: RewardItemKind::CardChoice,
            state: RewardItemState::Available,
            label: label.to_string(),
            claimable,
            active: false,
            skip_allowed: false,
            skip_label: None,
            choices,
        })
    }

    fn build_pending_event_deck_selection_screen(&mut self) {
        let Some(pending) = self.pending_event_deck_selection.as_ref() else {
            return;
        };
        let label = pending.label.clone();
        let Some(item) = self.event_deck_selection_item(&label, 0, true) else {
            self.pending_event_deck_selection = None;
            return;
        };
        self.reward_screen = Some(RewardScreen {
            source: RewardScreenSource::Event,
            ordered: true,
            active_item: None,
            items: vec![item],
        });
        self.phase = RunPhase::CardReward;
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
                self.store_note_for_yourself_card(card_id.clone());
                true
            }
            "deck_selection_empty_cage" => {
                self.remove_master_deck_card(*deck_index);
                self.pending_empty_cage_removals =
                    self.pending_empty_cage_removals.saturating_sub(1);
                true
            }
            "deck_selection_dollys_mirror" => {
                self.run_state.reconcile_deck_card_states();
                let Some(card) = self.run_state.deck_card_states.get(*deck_index).copied() else {
                    return false;
                };
                obtain_master_deck_stat_copy(&mut self.run_state, card);
                true
            }
            "deck_selection_event_duplicate" => {
                self.run_state.reconcile_deck_card_states();
                let Some(card) = self.run_state.deck_card_states.get(*deck_index).copied() else {
                    return false;
                };
                obtain_master_deck_stat_copy(&mut self.run_state, card);
                true
            }
            "deck_selection_event_remove" => {
                self.remove_master_deck_card(*deck_index);
                true
            }
            "deck_selection_event_upgrade" => {
                self.run_state.upgrade_deck_card(*deck_index);
                true
            }
            "deck_selection_event_transform" => {
                if let Some(original) = self.remove_master_deck_card(*deck_index) {
                    if let Some(transformed) = self.transform_card_with_misc(&original, false) {
                        obtain_master_deck_card_state(&mut self.run_state, transformed);
                    }
                }
                if let Some(pending) = self.pending_event_deck_selection.as_mut() {
                    if pending.label == label {
                        pending.remaining = pending.remaining.saturating_sub(1);
                    }
                }
                true
            }
            "deck_selection_designer_full" => {
                self.remove_master_deck_card(*deck_index);
                let mut indices = self
                    .run_state
                    .deck
                    .iter()
                    .enumerate()
                    .filter_map(|(index, card)| {
                        crate::cards::global_registry()
                            .can_upgrade_name(card)
                            .then_some(index)
                    })
                    .collect::<Vec<_>>();
                // REMOVE_AND_UPGRADE shuffles after the selected card is
                // removed, even when the resulting list is empty.
                let seed = self.floor_rngs.misc.random_long_unbounded();
                crate::seed::java_util_shuffle(&mut indices, seed);
                if let Some(index) = indices.first().copied() {
                    self.run_state.upgrade_deck_card(index);
                }
                true
            }
            "deck_selection_neow_remove" => {
                if self.remove_master_deck_card(*deck_index).is_some() {
                    if let Some(pending) = self.pending_neow_deck_selection.as_mut() {
                        pending.remaining = pending.remaining.saturating_sub(1);
                    }
                }
                true
            }
            "deck_selection_neow_upgrade" => {
                if self.run_state.upgrade_deck_card(*deck_index).is_some() {
                    if let Some(pending) = self.pending_neow_deck_selection.as_mut() {
                        pending.remaining = pending.remaining.saturating_sub(1);
                    }
                }
                true
            }
            "deck_selection_neow_transform" => {
                if let Some(original) = self.remove_master_deck_card(*deck_index) {
                    if let Some(transformed) = self.transform_neow_card(&original) {
                        obtain_master_deck_card_state(&mut self.run_state, transformed);
                    }
                    if let Some(pending) = self.pending_neow_deck_selection.as_mut() {
                        pending.remaining = pending.remaining.saturating_sub(1);
                    }
                }
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
        if !self.is_removable_master_deck_index(deck_index) {
            return None;
        }
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
        let skipped_neow_selection = matches!(
            item.label.as_str(),
            "deck_selection_neow_remove"
                | "deck_selection_neow_upgrade"
                | "deck_selection_neow_transform"
        );
        if item.index != item_index
            || !item.skip_allowed
            || item.state != RewardItemState::Available
        {
            return;
        }
        if let Some(choice_frame) = active_choice.as_ref() {
            if choice_frame.item_index != item_index || !choice_frame.skip_allowed {
                return;
            }
        } else if !item.claimable {
            return;
        }

        // BossRelicSelectScreen's cancel button closes the boss chest and
        // returns to TreasureRoomBoss. The same three constructor-generated
        // choices are available if the chest is opened again.
        // Java: BossRelicSelectScreen.java::relicSkipLogic, BossChest.java::close.
        if active_choice.is_some() && screen.source == RewardScreenSource::BossRelic {
            self.decision_stack.pop();
            self.reward_screen = None;
            self.phase = RunPhase::Chest;
            return;
        }

        // SkipCardButton closes CardRewardScreen without taking its backing
        // RewardItem. The player may click that card reward again or abandon
        // the entire combat reward screen with Proceed.
        // Java: CardRewardScreen.java::takeReward, SkipCardButton.java::update.
        if active_choice.is_some() && !screen.ordered && item.kind == RewardItemKind::CardChoice {
            self.decision_stack.pop();
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
        if skipped_neow_selection {
            self.pending_neow_deck_selection = None;
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
        let linked_to_disable = match kind {
            RewardItemKind::Key {
                linked_item_index, ..
            } => linked_item_index,
            _ => screen
                .items
                .iter()
                .find_map(|candidate| match candidate.kind {
                    RewardItemKind::Key {
                        linked_item_index: Some(linked),
                        ..
                    } if linked == item_index => Some(candidate.index),
                    _ => None,
                }),
        };
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
            && !self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::SOZU)
            && !self
                .run_state
                .potions
                .iter()
                .any(|potion| potion.is_empty())
        {
            return;
        }

        match kind {
            RewardItemKind::Relic => {
                self.pending_relic_followup_source = source.clone();
                self.add_relic_reward(&label);
            }
            RewardItemKind::Potion => self.add_potion_reward(&label),
            RewardItemKind::Gold | RewardItemKind::StolenGold => {
                if let Ok(amount) = label.parse::<i32>() {
                    self.adjust_run_gold(amount.max(0));
                }
            }
            RewardItemKind::Key { color, .. } => match color {
                RewardKeyColor::Ruby => self.run_state.has_ruby_key = true,
                RewardKeyColor::Emerald => self.run_state.has_emerald_key = true,
                RewardKeyColor::Sapphire => self.run_state.has_sapphire_key = true,
            },
            _ => {}
        }

        if let Some(screen) = self.reward_screen.as_mut() {
            if let Some(item) = screen.items.get_mut(item_index) {
                item.state = RewardItemState::Claimed;
            }
            if let Some(linked_index) = linked_to_disable {
                if let Some(linked) = screen.items.get_mut(linked_index) {
                    linked.state = RewardItemState::Disabled;
                }
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
        for choice in screen
            .items
            .iter_mut()
            .flat_map(|item| item.choices.iter_mut())
        {
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
                self.pending_bottled_tornado_selection = !self.bottled_tornado_choices().is_empty();
            }
            "Calling Bell" => self.pending_calling_bell_rewards = true,
            "Necronomicon" => {
                // Necronomicon.java::onEquip obtains one Necronomicurse via
                // ShowCardAndObtainEffect, including Omamori interception.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Necronomicon.java
                obtain_master_deck_card_state(&mut self.run_state, "Necronomicurse".to_string());
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
            ordered: false,
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
        let choices = self
            .run_state
            .deck
            .iter()
            .cloned()
            .enumerate()
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
                choices: self.generate_card_reward_choices(choice_count, CardRewardContext::Shop),
            })
            .collect::<Vec<_>>();
        let mut screen = RewardScreen {
            source: RewardScreenSource::Shop,
            ordered: false,
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
        self.run_state.reconcile_deck_card_states();
        let names = std::mem::take(&mut self.run_state.deck);
        let states = std::mem::take(&mut self.run_state.deck_card_states);
        let mut count = 0;
        for (name, state) in names.into_iter().zip(states) {
            if is_starter_strike_or_defend(&name) {
                count += 1;
            } else {
                self.run_state.deck.push(name);
                self.run_state.deck_card_states.push(state);
            }
        }
        if count == 0 {
            return;
        }

        let pool_len =
            WATCHER_COMMON_CARDS.len() + WATCHER_UNCOMMON_CARDS.len() + WATCHER_RARE_CARDS.len();
        for _ in 0..count {
            let idx = self.floor_rngs.card_random.random_index(pool_len);
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
        for original in originals {
            if let Some(transformed) = self.transform_card_with_misc(&original, true) {
                obtain_master_deck_card_state(&mut self.run_state, transformed);
            }
        }
    }

    fn transform_card_with_misc(&mut self, original: &str, auto_upgrade: bool) -> Option<String> {
        let registry = crate::cards::global_registry();
        let original = original.trim_end_matches('+');
        let color = event_card_color(original)?;
        let mut candidates = match color {
            EventCardColor::Colorless => [EventCardRarity::Uncommon, EventCardRarity::Rare]
                .into_iter()
                .flat_map(|rarity| matching_event_cards(EventCardColor::Colorless, rarity))
                .collect::<Vec<_>>(),
            EventCardColor::Curse => RANDOM_OBTAINABLE_CURSES
                .iter()
                .map(|card| (*card).to_string())
                .collect::<Vec<_>>(),
            _ => {
                let colors = if self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::PRISMATIC_SHARD)
                {
                    vec![
                        EventCardColor::Red,
                        EventCardColor::Green,
                        EventCardColor::Blue,
                        EventCardColor::Purple,
                    ]
                } else {
                    vec![color]
                };
                colors
                    .into_iter()
                    .flat_map(|candidate_color| {
                        [
                            EventCardRarity::Common,
                            EventCardRarity::Uncommon,
                            EventCardRarity::Rare,
                        ]
                        .into_iter()
                        .flat_map(move |rarity| matching_event_cards(candidate_color, rarity))
                    })
                    .collect::<Vec<_>>()
            }
        };
        candidates.retain(|candidate| {
            !candidate.ends_with('+') && candidate != original && registry.get(candidate).is_some()
        });
        if candidates.is_empty() {
            return None;
        }
        let mut transformed =
            candidates[self.floor_rngs.misc.random_index(candidates.len())].clone();
        if auto_upgrade {
            let upgraded = format!("{transformed}+");
            if registry.get(&upgraded).is_some() {
                transformed = upgraded;
            }
        }
        Some(transformed)
    }

    fn prepare_astrolabe_on_equip(&mut self) {
        let purgeable: Vec<usize> = self
            .run_state
            .deck
            .iter()
            .enumerate()
            .filter_map(|(idx, _)| self.is_removable_master_deck_index(idx).then_some(idx))
            .collect();
        self.pending_astrolabe_removed.clear();
        if purgeable.is_empty() {
            self.pending_astrolabe_selection = false;
        } else if purgeable.len() <= 3 {
            let mut originals = Vec::with_capacity(purgeable.len());
            for idx in purgeable.into_iter().rev() {
                if let Some(original) = self.remove_master_deck_card(idx) {
                    originals.push(original);
                }
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
            .filter_map(|(idx, _)| self.is_removable_master_deck_index(idx).then_some(idx))
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
            .filter(|(index, _)| self.is_removable_master_deck_index(*index))
            .map(|(index, card_id)| RewardChoice::Card {
                index,
                card_id: card_id.clone(),
            })
            .collect();
        let mut screen = RewardScreen {
            source: self.pending_relic_followup_source.clone(),
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
            .filter(|(index, _)| self.is_removable_master_deck_index(*index))
            .map(|(index, card_id)| RewardChoice::Card {
                index,
                card_id: card_id.clone(),
            })
            .collect();
        let mut screen = RewardScreen {
            source: self.pending_relic_followup_source.clone(),
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

    fn build_calling_bell_reward_screen(&mut self) {
        // CallingBell.java opens one mandatory Curse of the Bell confirmation,
        // then one screenless COMMON, UNCOMMON, and RARE relic reward.
        let common = self.draw_relic_from_pool(RelicTier::Common, false, false);
        let uncommon = self.draw_relic_from_pool(RelicTier::Uncommon, false, false);
        let rare = self.draw_relic_from_pool(RelicTier::Rare, false, false);
        let mut screen = RewardScreen {
            source: self.pending_relic_followup_source.clone(),
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
        // TinyHouse.java indexes PotionHelper's initialized list directly with
        // miscRng; it does not use the ordinary potion reward rarity loop.
        let potion = self.roll_tiny_house_potion_id();
        let mut screen = RewardScreen {
            source: self.pending_relic_followup_source.clone(),
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

        // Whetstone and War Paint seed java.util.Random from exactly one
        // miscRng.randomLong(), then upgrade the first two shuffled cards.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Whetstone.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/WarPaint.java
        let seed = self.floor_rngs.misc.random_long_unbounded();
        crate::seed::java_util_shuffle(&mut eligible, seed);
        for deck_idx in eligible.into_iter().take(count) {
            self.run_state.upgrade_deck_card(deck_idx);
        }
    }

    fn upgrade_one_random_master_deck_card(&mut self) {
        let registry = crate::cards::global_registry();
        let mut eligible: Vec<usize> = self
            .run_state
            .deck
            .iter()
            .enumerate()
            .filter_map(|(idx, card_id)| registry.can_upgrade_name(card_id).then_some(idx))
            .collect();
        // TinyHouse.java consumes one miscRng.randomLong(), shuffles through
        // java.util.Random even when no card is eligible.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/TinyHouse.java
        let seed = self.floor_rngs.misc.random_long_unbounded();
        crate::seed::java_util_shuffle(&mut eligible, seed);
        if let Some(&deck_idx) = eligible.first() {
            self.run_state.upgrade_deck_card(deck_idx);
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

    fn roll_relic_tier(&mut self) -> RelicTier {
        let roll = self.persistent_rngs.relic.random_int(99);
        if roll < 50 {
            RelicTier::Common
        } else if roll < 83 {
            RelicTier::Uncommon
        } else {
            RelicTier::Rare
        }
    }

    fn campfire_relic_count(&self) -> usize {
        self.run_state
            .relics
            .iter()
            .filter(|relic| matches!(relic.as_str(), "Peace Pipe" | "Shovel" | "Girya"))
            .count()
    }

    fn roll_shop_relic_tier(&mut self) -> RelicTier {
        let roll = self.persistent_rngs.merchant.random_int(99);
        if roll < 48 {
            RelicTier::Common
        } else if roll < 82 {
            RelicTier::Uncommon
        } else {
            RelicTier::Rare
        }
    }

    fn relic_can_spawn(&self, relic_id: &str, in_shop: bool) -> bool {
        let floor = self.run_state.floor;
        if matches!(
            relic_id,
            "Ancient Tea Set"
                | "CeramicFish"
                | "Dream Catcher"
                | "Juzu Bracelet"
                | "MealTicket"
                | "Omamori"
                | "Potion Belt"
                | "Regal Pillow"
                | "MawBank"
                | "Smiling Mask"
                | "Darkstone Periapt"
                | "The Courier"
                | "Frozen Egg 2"
                | "Meat on the Bone"
                | "Molten Egg 2"
                | "Question Card"
                | "Singing Bowl"
                | "Toxic Egg 2"
                | "Prayer Wheel"
        ) && floor > 48
        {
            return false;
        }
        if relic_id == "PreservedInsect" && floor > 52 {
            return false;
        }
        if relic_id == "Tiny Chest" && floor > 35 {
            return false;
        }
        if matches!(relic_id, "Matryoshka" | "WingedGreaves") && floor > 40 {
            return false;
        }
        if in_shop
            && matches!(
                relic_id,
                "MawBank" | "Smiling Mask" | "The Courier" | "Old Coin"
            )
        {
            return false;
        }
        if relic_id == "Old Coin" && floor > 48 {
            return false;
        }
        if matches!(relic_id, "Girya" | "Peace Pipe" | "Shovel")
            && (floor >= 48 || self.campfire_relic_count() >= 2)
        {
            return false;
        }
        if relic_id == "Bottled Flame" && !self.can_spawn_bottled_flame() {
            return false;
        }
        if relic_id == "Bottled Lightning" && !self.can_spawn_bottled_lightning() {
            return false;
        }
        if relic_id == "Bottled Tornado" && !self.can_spawn_bottled_tornado() {
            return false;
        }
        if relic_id == "Ectoplasm" && self.run_state.act > 1 {
            return false;
        }
        if relic_id == "HolyWater"
            && !self
                .run_state
                .relics
                .iter()
                .any(|relic| relic == "PureWater")
        {
            return false;
        }
        true
    }

    fn pop_relic_candidate(&mut self, tier: RelicTier, from_end: bool) -> String {
        let candidate = {
            let pool = self.relic_pools.pool_mut(tier);
            if pool.is_empty() {
                None
            } else if tier == RelicTier::Boss || !from_end {
                Some(pool.remove(0))
            } else {
                pool.pop()
            }
        };
        if let Some(candidate) = candidate {
            return candidate;
        }
        match tier {
            RelicTier::Common => self.pop_relic_candidate(RelicTier::Uncommon, false),
            RelicTier::Uncommon => self.pop_relic_candidate(RelicTier::Rare, false),
            RelicTier::Shop => self.pop_relic_candidate(RelicTier::Uncommon, false),
            RelicTier::Rare => "Circlet".to_string(),
            RelicTier::Boss => "Red Circlet".to_string(),
        }
    }

    fn draw_relic_from_pool(&mut self, tier: RelicTier, from_end: bool, in_shop: bool) -> String {
        let mut use_end = from_end;
        loop {
            let candidate = self.pop_relic_candidate(tier, use_end);
            if matches!(candidate.as_str(), "Circlet" | "Red Circlet")
                || self.relic_can_spawn(&candidate, in_shop)
            {
                return candidate;
            }
            // Both Java entry points retry through returnEndRandomRelicKey
            // after permanently discarding a failed canSpawn candidate.
            use_end = true;
        }
    }

    fn roll_reward_relic_id(&mut self) -> String {
        let tier = self.roll_relic_tier();
        self.draw_relic_from_pool(tier, false, false)
    }

    fn roll_screenless_relic_id(&mut self) -> String {
        let tier = self.roll_relic_tier();
        loop {
            let relic = self.draw_relic_from_pool(tier, false, false);
            if !matches!(
                relic.as_str(),
                "Bottled Flame" | "Bottled Lightning" | "Bottled Tornado" | "Whetstone"
            ) {
                return relic;
            }
        }
    }

    fn roll_noncamp_reward_relic_id(&mut self) -> String {
        let tier = self.roll_relic_tier();
        loop {
            let relic = self.draw_relic_from_pool(tier, false, false);
            if !matches!(relic.as_str(), "Peace Pipe" | "Shovel" | "Girya") {
                return relic;
            }
        }
    }

    fn roll_shop_reward_relic_id(&mut self) -> String {
        let tier = self.roll_shop_relic_tier();
        self.draw_relic_from_pool(tier, true, true)
    }

    fn roll_matryoshka_relic_id(&mut self) -> String {
        // Matryoshka consumes one 75% tier roll, then uses the ordinary
        // front-removal path without screenless/non-campfire exclusions.
        // Java: Matryoshka.java::onChestOpen.
        let tier = if self.persistent_rngs.relic.random_bool_chance(0.75) {
            RelicTier::Common
        } else {
            RelicTier::Uncommon
        };
        self.draw_relic_from_pool(tier, false, false)
    }

    fn roll_reward_potion_id(&mut self) -> String {
        // PotionHelper.getPotions(WATCHER, false) is the exact candidate
        // order. Selection samples this full list repeatedly until the result
        // matches the independently rolled rarity; duplicates are allowed.
        // Java: PotionHelper.java::getPotions,
        // AbstractDungeon.java::returnRandomPotion.
        let rarity_roll = self.persistent_rngs.potion.random_int(99);
        let rarity = if rarity_roll < 65 {
            PotionRarity::Common
        } else if rarity_roll < 90 {
            PotionRarity::Uncommon
        } else {
            PotionRarity::Rare
        };
        loop {
            let index = self
                .persistent_rngs
                .potion
                .random_index(WATCHER_POTIONS.len());
            let (id, candidate_rarity) = WATCHER_POTIONS[index];
            if candidate_rarity == rarity {
                return id.to_string();
            }
        }
    }

    fn roll_tiny_house_potion_id(&mut self) -> String {
        let index = self.floor_rngs.misc.random_index(WATCHER_POTIONS.len());
        WATCHER_POTIONS[index].0.to_string()
    }

    fn roll_boss_relic_choices(&mut self, count: usize) -> Vec<String> {
        // BossChest removes three consecutive entries from the persistent boss
        // pool. It consumes no relic RNG after the startup shuffle.
        // Java: BossChest.java::<init>, AbstractDungeon.returnRandomRelicKey.
        (0..count)
            .map(|_| self.draw_relic_from_pool(RelicTier::Boss, false, false))
            .collect()
    }

    fn should_offer_potion_reward(
        &mut self,
        room_type: RoomType,
        pre_potion_reward_count: usize,
    ) -> bool {
        // AbstractRoom.addPotionToRewards rolls independently of Sozu and
        // available inventory slots. RewardItem handles both at claim time.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/rooms/AbstractRoom.java
        let mut chance = 40 + self.run_state.potion_blizz_randomizer;
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::WHITE_BEAST)
        {
            chance = 100;
        }
        // Black Star plus the Emerald Key can fill four pre-potion reward
        // slots. Java forces chance zero but still consumes the chance draw.
        if pre_potion_reward_count >= 4 {
            chance = 0;
        }
        // Standard MonsterRoom, MonsterRoomElite, Act 1/2 MonsterRoomBoss, and
        // EventRoom all use the same base chance. The room parameter remains
        // explicit because Act 3/4 boss flow skips reward construction.
        debug_assert!(matches!(
            room_type,
            RoomType::Monster | RoomType::Elite | RoomType::Boss | RoomType::Event
        ));
        // Java always consumes this roll, including the 0% and 100% cases.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/rooms/AbstractRoom.java:592-619
        let offered = self.persistent_rngs.potion.random_int(99) < chance;
        if offered {
            self.run_state.potion_blizz_randomizer -= 10;
        } else {
            self.run_state.potion_blizz_randomizer += 10;
        }
        offered
    }

    fn can_enter_final_act(&self) -> bool {
        self.profile.final_act_available
            && self.run_state.has_ruby_key
            && self.run_state.has_emerald_key
            && self.run_state.has_sapphire_key
    }

    fn resolve_terminal_run_victory(&mut self) {
        self.current_event = None;
        self.spire_heart_state = None;
        self.pending_event_combat = None;
        self.reward_screen = None;
        self.combat_engine = None;
        self.active_combat_room_type = None;
        self.run_state.run_won = true;
        self.run_state.run_over = true;
        self.phase = RunPhase::GameOver;
    }

    fn start_final_act(&mut self) {
        // SpireHeart's key branch performs the same dungeon-transition setup
        // as standard acts, then TheEnding installs a fixed zero-draw map and
        // a fresh mapRng seeded with Settings.seed + actNum * 300.
        // Java: AbstractDungeon.java::dungeonTransitionSetup,
        // TheEnding.java::<init>/generateSpecialMap.
        self.advance_card_rng_for_act_transition();
        self.run_state.act = 4;
        self.run_state.map_x = 0;
        self.run_state.map_y = -1;
        self.run_state.event_monster_chance = default_event_monster_chance();
        self.run_state.event_shop_chance = default_event_shop_chance();
        self.run_state.event_treasure_chance = default_event_treasure_chance();
        self.run_state.potion_blizz_randomizer = 0;
        let heal_amount = if self.run_state.ascension >= 5 {
            (((self.run_state.max_hp - self.run_state.current_hp) as f32) * 0.75).round() as i32
        } else {
            self.run_state.max_hp
        };
        self.heal_run_player(heal_amount);
        self.map = generate_the_ending_map();
        self.map_rng = crate::seed::StsRandom::new(self.seed.wrapping_add(1200));
        self.boss_id = "CorruptHeart".to_string();
        self.boss_sequence = std::iter::repeat_n("CorruptHeart".to_string(), 3).collect();
        self.encounter_queue_act = 4;
        self.monster_encounter_queue.clear();
        self.elite_encounter_queue.clear();
        self.monster_encounter_queue
            .extend(std::iter::repeat_n(ACT4_ELITE_ENCOUNTER.to_string(), 3));
        self.elite_encounter_queue
            .extend(std::iter::repeat_n(ACT4_ELITE_ENCOUNTER.to_string(), 3));
        self.current_event = None;
        self.spire_heart_state = None;
        self.pending_event_combat = None;
        self.reward_screen = None;
        self.pending_chest = None;
        self.combat_engine = None;
        self.active_combat_room_type = None;
        self.active_combat_has_emerald_key = false;
        self.phase = RunPhase::MapChoice;
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
            self.run_state.relic_flags.counters[crate::relic_flags::counter::ANCIENT_TEA_SET] = 1;
        }
        self.phase = RunPhase::Campfire;
    }

    fn step_campfire(&mut self, action: &GameAction) {
        match action {
            GameAction::CampfireRest => {
                if !self
                    .run_state
                    .relic_flags
                    .has(crate::relic_flags::flag::MARK_OF_BLOOM)
                {
                    // Source: CampfireSleepEffect.java truncates maxHealth *
                    // 0.3f to int, then adds Regal Pillow's exact 15.
                    let mut heal = (self.run_state.max_hp as f32 * 0.3) as i32;
                    if self
                        .run_state
                        .relic_flags
                        .has(crate::relic_flags::flag::REGAL_PILLOW)
                    {
                        heal += 15;
                    }
                    self.run_state.current_hp =
                        (self.run_state.current_hp + heal).min(self.run_state.max_hp);
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
                    return;
                }
            }
            GameAction::CampfireUpgrade(idx) => {
                self.run_state.upgrade_deck_card(*idx);
            }
            GameAction::CampfireToke => {
                self.build_peace_pipe_selection_screen();
                self.phase = RunPhase::CardReward;
                self.refresh_decision_stack();
                return;
            }
            GameAction::CampfireLift => {
                // CampfireLiftEffect increments Girya.counter exactly once;
                // LiftOption is usable only while the counter is below three.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/vfx/campfire/CampfireLiftEffect.java
                let counter =
                    &mut self.run_state.relic_flags.counters[crate::relic_flags::counter::GIRYA];
                *counter = (*counter + 1).min(3);
            }
            GameAction::CampfireDig => {
                self.build_shovel_reward_screen();
                self.phase = RunPhase::CardReward;
                self.refresh_decision_stack();
                return;
            }
            GameAction::CampfireRecall => {
                self.run_state.has_ruby_key = true;
            }
            _ => {}
        }

        // The top-row campfire completes before the player clicks Java's
        // synthetic boss node `(-1, 15)` on the map.
        if self.run_state.map_y >= 0 && self.run_state.map_y as usize >= self.map.height - 1 {
            self.phase = RunPhase::MapChoice;
            self.refresh_decision_stack();
            return;
        }

        self.phase = RunPhase::MapChoice;
        self.refresh_decision_stack();
        ()
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
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::THE_COURIER)
        {
            price = Self::rounded_shop_price(price, 0.8);
        }
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::MEMBERSHIP_CARD)
        {
            price = Self::rounded_shop_price(price, 0.5);
        }
        price
    }

    fn apply_shop_replacement_discounts(&self, mut price: i32) -> i32 {
        // StoreRelic/StorePotion round each replacement discount after the
        // merchant variance has already been rounded.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/shop/ShopScreen.java
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::THE_COURIER)
        {
            price = Self::rounded_shop_price(price, 0.8);
        }
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::MEMBERSHIP_CARD)
        {
            price = Self::rounded_shop_price(price, 0.5);
        }
        price
    }

    fn roll_shop_card_rarity(&mut self) -> EventCardRarity {
        // ShopRoom disables reward alternation but still uses the persistent
        // Blizzard offset: 9% rare, then 37% uncommon. Shop generation does
        // not mutate the offset.
        // Java: AbstractDungeon.java::rollRarity,
        // ShopRoom.java::{constructor,getCardRarity}.
        let roll = self.persistent_rngs.card.random_int(99) + self.run_state.card_blizz_randomizer;
        if roll < 9 {
            EventCardRarity::Rare
        } else if roll < 46 {
            EventCardRarity::Uncommon
        } else {
            EventCardRarity::Common
        }
    }

    fn shop_rarity_fallbacks(
        rarity: EventCardRarity,
        card_type: crate::cards::CardType,
    ) -> &'static [EventCardRarity] {
        use crate::cards::CardType;
        use EventCardRarity::{Common, Rare, Uncommon};
        match (rarity, card_type) {
            (Rare, _) => &[Rare, Uncommon, Common],
            (Uncommon, CardType::Power) => &[Uncommon, Rare],
            (Uncommon, _) => &[Uncommon, Common],
            (Common, CardType::Power) => &[Common, Uncommon, Rare],
            (Common, _) => &[Common],
            _ => &[],
        }
    }

    fn roll_shop_colored_card_with_source(
        &mut self,
        card_type: crate::cards::CardType,
        use_card_rng: bool,
    ) -> (String, EventCardRarity) {
        let rolled_rarity = self.roll_shop_card_rarity();
        let registry = crate::cards::global_registry();
        for &rarity in Self::shop_rarity_fallbacks(rolled_rarity, card_type) {
            let mut candidates = self
                .card_pools
                .working(rarity)
                .iter()
                .filter(|id| {
                    registry
                        .get(id)
                        .is_some_and(|card| card.card_type == card_type)
                })
                .cloned()
                .collect::<Vec<_>>();
            if candidates.is_empty() {
                continue;
            }
            // CardGroup.getRandomCard(type, ...) sorts by cardID for both the
            // counted cardRng path and the ambient MathUtils path.
            candidates.sort();
            let index = if use_card_rng {
                self.persistent_rngs.card.random_index(candidates.len())
            } else {
                self.ambient_math_rng
                    .random_int(candidates.len() as i32 - 1) as usize
            };
            return (candidates[index].clone(), rarity);
        }
        ("Scrawl".to_string(), EventCardRarity::Rare)
    }

    fn roll_shop_colored_card(
        &mut self,
        card_type: crate::cards::CardType,
    ) -> (String, EventCardRarity) {
        self.roll_shop_colored_card_with_source(card_type, true)
    }

    fn roll_shop_colorless_card(&mut self, rare: bool) -> (String, EventCardRarity) {
        let (pool, rarity) = if rare {
            (SHOP_COLORLESS_RARE_CARDS, EventCardRarity::Rare)
        } else {
            (SHOP_COLORLESS_UNCOMMON_CARDS, EventCardRarity::Uncommon)
        };
        let mut candidates = pool.to_vec();
        candidates.sort();
        let card = candidates[self.persistent_rngs.card.random_index(candidates.len())];
        (card.to_string(), rarity)
    }

    fn roll_shop_card_price(&mut self, rarity: EventCardRarity, colorless: bool) -> i32 {
        self.roll_shop_card_price_raw(rarity, colorless) as i32
    }

    fn roll_shop_card_price_raw(&mut self, rarity: EventCardRarity, colorless: bool) -> f32 {
        let base_price = match rarity {
            EventCardRarity::Common => 50,
            EventCardRarity::Uncommon => 75,
            EventCardRarity::Rare => 150,
            _ => unreachable!("shop cards use standard rarities"),
        };
        let colorless_multiplier = if colorless { 1.2 } else { 1.0 };
        (base_price as f32)
            * self.persistent_rngs.merchant.random_f32_range(0.9, 1.1)
            * colorless_multiplier
    }

    fn is_shop_colorless_card(card_id: &str) -> bool {
        let card_id = card_id.strip_suffix('+').unwrap_or(card_id);
        SHOP_COLORLESS_UNCOMMON_CARDS.contains(&card_id)
            || SHOP_COLORLESS_RARE_CARDS.contains(&card_id)
    }

    fn roll_courier_replacement_card(&mut self, purchased: &str) -> (String, i32) {
        let (card, rarity, colorless) = if Self::is_shop_colorless_card(purchased) {
            let rare = self.persistent_rngs.merchant.random_f32() < 0.3;
            let (card, rarity) = self.roll_shop_colorless_card(rare);
            (card, rarity, true)
        } else {
            let base_id = purchased.strip_suffix('+').unwrap_or(purchased);
            let card_type = crate::cards::global_registry()
                .get(base_id)
                .map(|card| card.card_type)
                .unwrap_or(crate::cards::CardType::Skill);
            let (card, rarity) = self.roll_shop_colored_card_with_source(card_type, false);
            (card, rarity, false)
        };
        let raw_price = self.roll_shop_card_price_raw(rarity, colorless);
        // ShopScreen.setPrice applies all card multipliers in float and then
        // truncates once, unlike relic/potion refill rounding.
        let mut multiplier = 1.0;
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::THE_COURIER)
        {
            multiplier *= 0.8;
        }
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::MEMBERSHIP_CARD)
        {
            multiplier *= 0.5;
        }
        (
            self.upgrade_reward_card_if_needed(&card),
            (raw_price * multiplier) as i32,
        )
    }

    fn consume_shop_purchase_ambient_draws(&mut self, card_purchase: bool) {
        // ShopScreen.purchaseCard additionally resets speechTimer. All three
        // successful purchase types then roll a voice line, select one of five
        // buy messages, and construct a speech bubble with a random side and x
        // position. These visual calls share MathUtils.random with gameplay-
        // significant card selection, so omitting them changes future Courier
        // refills.
        // Java: ShopScreen.java::{purchaseCard,playBuySfx,getBuyMsg,createSpeech},
        // StoreRelic.java::purchaseRelic, StorePotion.java::purchasePotion.
        if card_purchase {
            let _ = self.ambient_math_rng.random_f32_range(40.0, 60.0);
        }
        let _ = self.ambient_math_rng.random_int(2);
        let _ = self.ambient_math_rng.random_int(4);
        let _ = self.ambient_math_rng.random_bool();
        let _ = self.ambient_math_rng.random_f32_range(660.0, 1260.0);
    }

    fn potion_base_shop_price(potion_id: &str) -> i32 {
        let key: String = potion_id
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .flat_map(|ch| ch.to_lowercase())
            .collect();
        if matches!(
            key.as_str(),
            "ambrosia"
                | "cultistpotion"
                | "fruitjuice"
                | "sneckooil"
                | "fairypotion"
                | "smokebomb"
                | "entropicbrew"
        ) {
            100
        } else if matches!(
            key.as_str(),
            "stancepotion"
                | "regenpotion"
                | "ancientpotion"
                | "liquidbronze"
                | "gamblersbrew"
                | "essenceofsteel"
                | "duplicationpotion"
                | "distilledchaos"
                | "liquidmemories"
        ) {
            75
        } else {
            50
        }
    }

    fn roll_shop_potion(&mut self) -> (String, i32) {
        let potion = self.roll_reward_potion_id();
        let base = Self::potion_base_shop_price(&potion);
        let price = ((base as f32) * self.persistent_rngs.merchant.random_f32_range(0.95, 1.05))
            .round() as i32;
        (potion, price)
    }

    fn relic_base_shop_price(relic_id: &str) -> i32 {
        // AbstractRelic.getPrice: COMMON=150, UNCOMMON=250, RARE=300.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/AbstractRelic.java
        if matches!(
            relic_id,
            "Blue Candle"
                | "Bottled Flame"
                | "Bottled Lightning"
                | "Bottled Tornado"
                | "The Courier"
                | "Darkstone Periapt"
                | "Yang"
                | "Eternal Feather"
                | "Frozen Egg 2"
                | "InkBottle"
                | "Kunai"
                | "Letter Opener"
                | "Matryoshka"
                | "Meat on the Bone"
                | "Mercury Hourglass"
                | "Molten Egg 2"
                | "Mummified Hand"
                | "Ornamental Fan"
                | "Pantograph"
                | "Pear"
                | "Question Card"
                | "Shuriken"
                | "Singing Bowl"
                | "Sundial"
                | "Toxic Egg 2"
                | "White Beast Statue"
                | "StrikeDummy"
                | "TeardropLocket"
        ) {
            250
        } else if matches!(
            relic_id,
            "Bird Faced Urn"
                | "Calipers"
                | "Du-Vu Doll"
                | "FossilizedHelix"
                | "Ginger"
                | "Girya"
                | "Ice Cream"
                | "Incense Burner"
                | "Lizard Tail"
                | "Magic Flower"
                | "Mango"
                | "Old Coin"
                | "Peace Pipe"
                | "Pocketwatch"
                | "Prayer Wheel"
                | "Shovel"
                | "StoneCalendar"
                | "Thread and Needle"
                | "Tingsha"
                | "Torii"
                | "Tough Bandages"
                | "TungstenRod"
                | "Turnip"
                | "Unceasing Top"
                | "WingedGreaves"
                | "CaptainsWheel"
                | "CloakClasp"
                | "GoldenEye"
        ) {
            300
        } else {
            150
        }
    }

    fn roll_relic_merchant_price(&mut self, relic_id: &str) -> i32 {
        let base = Self::relic_base_shop_price(relic_id);
        ((base as f32) * self.persistent_rngs.merchant.random_f32_range(0.95, 1.05)).round() as i32
    }

    fn roll_courier_replacement_relic(&mut self) -> (String, i32) {
        // StoreRelic excludes these four on every Courier refill.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/shop/StoreRelic.java
        let relic = (0..64)
            .map(|_| self.roll_shop_reward_relic_id())
            .find(|id| {
                !matches!(
                    id.as_str(),
                    "Old Coin" | "Smiling Mask" | "MawBank" | "The Courier"
                )
            })
            .unwrap_or_else(|| "Circlet".to_string());
        let raw_price = self.roll_relic_merchant_price(&relic);
        (relic, self.apply_shop_replacement_discounts(raw_price))
    }

    fn enter_shop(&mut self) {
        // Merchant.java creates two attacks, two skills, one power, then one
        // uncommon and one rare colorless card. ShopScreen chooses one of the
        // five colored cards for the half-price sale before global discounts.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/shop/Merchant.java
        let mut generated_cards = Vec::new();
        let first_attack = self.roll_shop_colored_card(crate::cards::CardType::Attack);
        let second_attack = loop {
            let candidate = self.roll_shop_colored_card(crate::cards::CardType::Attack);
            if candidate.0 != first_attack.0 {
                break candidate;
            }
        };
        let first_skill = self.roll_shop_colored_card(crate::cards::CardType::Skill);
        let second_skill = loop {
            let candidate = self.roll_shop_colored_card(crate::cards::CardType::Skill);
            if candidate.0 != first_skill.0 {
                break candidate;
            }
        };
        generated_cards.extend([first_attack, second_attack, first_skill, second_skill]);
        generated_cards.push(self.roll_shop_colored_card(crate::cards::CardType::Power));
        generated_cards.push(self.roll_shop_colorless_card(false));
        generated_cards.push(self.roll_shop_colorless_card(true));

        // Merchant generates every card before ShopScreen prices the five
        // colored cards, then the two colorless cards, then rolls the sale.
        // Rejected duplicate generation attempts therefore consume no
        // merchantRng draws.
        // Java: Merchant.java::<init>, ShopScreen.java::initCards.
        let mut cards = generated_cards
            .into_iter()
            .enumerate()
            .map(|(index, (card, rarity))| {
                let price = self.roll_shop_card_price(rarity, index >= 5);
                (card, price)
            })
            .collect::<Vec<_>>();
        let sale_idx = self.persistent_rngs.merchant.random_int(4) as usize;
        cards[sale_idx].1 /= 2;
        for (card, price) in &mut cards {
            *card = self.upgrade_reward_card_if_needed(card);
            *price = self.apply_shop_entry_discounts(*price);
        }

        let remove_price = self.compute_shop_remove_price();

        // ShopScreen.java::initRelics creates two merchantRng tier rolls
        // followed by one guaranteed SHOP-tier relic. All identities are
        // removed from the ends of the persistent relic pools.
        let mut relics = Vec::new();
        for _ in 0..2 {
            let relic = self.roll_shop_reward_relic_id();
            let price = self.roll_relic_merchant_price(&relic);
            let final_price = self.apply_shop_entry_discounts(price);
            relics.push((relic, final_price));
        }
        let shop_relic = self.draw_relic_from_pool(RelicTier::Shop, true, true);
        // AbstractRelic.java::getPrice returns 150 for SHOP tier; StoreRelic
        // applies merchantRng.random(0.95f, 1.05f).
        let shop_price =
            (150.0 * self.persistent_rngs.merchant.random_f32_range(0.95, 1.05)).round() as i32;
        let shop_price = self.apply_shop_entry_discounts(shop_price);
        relics.push((shop_relic, shop_price));

        let potions = (0..3)
            .map(|_| {
                let (potion, price) = self.roll_shop_potion();
                (potion, self.apply_shop_entry_discounts(price))
            })
            .collect();

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
        if self
            .run_state
            .relic_flags
            .has(crate::relic_flags::flag::MEAL_TICKET)
            && !self
                .run_state
                .relic_flags
                .has(crate::relic_flags::flag::MARK_OF_BLOOM)
        {
            // Meal Ticket heals in a shop, not during combat, so Magic Flower
            // does not modify the 15 points.
            self.run_state.current_hp = (self.run_state.current_hp + 15).min(self.run_state.max_hp);
        }
    }

    fn step_shop(&mut self, action: &GameAction) {
        match action {
            GameAction::ShopBuyCard(idx) => {
                let purchase = self.current_shop.as_ref().and_then(|shop| {
                    shop.cards
                        .get(*idx)
                        .cloned()
                        .filter(|(_, price)| self.run_state.gold >= *price)
                });
                if let Some((card, price)) = purchase {
                    self.run_state.gold -= price;
                    if self
                        .run_state
                        .relic_flags
                        .has(crate::relic_flags::flag::MAW_BANK)
                    {
                        self.run_state.relic_flags.counters
                            [crate::relic_flags::counter::MAW_BANK_GOLD] = -2;
                    }
                    obtain_master_deck_card_state(&mut self.run_state, card.clone());
                    let replacement = self
                        .run_state
                        .relic_flags
                        .has(crate::relic_flags::flag::THE_COURIER)
                        .then(|| self.roll_courier_replacement_card(&card));
                    if let Some(shop) = self.current_shop.as_mut() {
                        if let Some(replacement) = replacement {
                            shop.cards[*idx] = replacement;
                        } else {
                            shop.cards.remove(*idx);
                        }
                    }
                    self.consume_shop_purchase_ambient_draws(true);
                }
                // Stay in shop for more purchases
                return;
            }
            GameAction::ShopBuyRelic(idx) => {
                let purchase = self.current_shop.as_ref().and_then(|shop| {
                    shop.relics
                        .get(*idx)
                        .cloned()
                        .filter(|(_, price)| self.run_state.gold >= *price)
                });
                if let Some((relic, price)) = purchase {
                    self.adjust_run_gold(-price);
                    self.add_relic_reward(&relic);
                    self.consume_shop_purchase_ambient_draws(false);
                    let replacement = (relic == "The Courier"
                        || self
                            .run_state
                            .relic_flags
                            .has(crate::relic_flags::flag::THE_COURIER))
                    .then(|| self.roll_courier_replacement_relic());
                    if let Some(shop) = self.current_shop.as_mut() {
                        if let Some(replacement) = replacement {
                            shop.relics[*idx] = replacement;
                        } else {
                            shop.relics.remove(*idx);
                        }
                    }
                }
                return;
            }
            GameAction::ShopBuyPotion(idx) => {
                let purchase = self.current_shop.as_ref().and_then(|shop| {
                    shop.potions.get(*idx).cloned().filter(|(_, price)| {
                        self.run_state.gold >= *price
                            && !self
                                .run_state
                                .relic_flags
                                .has(crate::relic_flags::flag::SOZU)
                            && self
                                .run_state
                                .potions
                                .iter()
                                .any(|potion| potion.is_empty())
                    })
                });
                if let Some((potion, price)) = purchase {
                    self.run_state.gold -= price;
                    if self
                        .run_state
                        .relic_flags
                        .has(crate::relic_flags::flag::MAW_BANK)
                    {
                        self.run_state.relic_flags.counters
                            [crate::relic_flags::counter::MAW_BANK_GOLD] = -2;
                    }
                    self.add_potion_reward(&potion);
                    self.consume_shop_purchase_ambient_draws(false);
                    let replacement = self
                        .run_state
                        .relic_flags
                        .has(crate::relic_flags::flag::THE_COURIER)
                        .then(|| {
                            let (potion, raw_price) = self.roll_shop_potion();
                            (potion, self.apply_shop_replacement_discounts(raw_price))
                        });
                    if let Some(shop) = self.current_shop.as_mut() {
                        if let Some(replacement) = replacement {
                            shop.potions[*idx] = replacement;
                        } else {
                            shop.potions.remove(*idx);
                        }
                    }
                }
                return;
            }
            GameAction::ShopRemoveCard(idx) => {
                let remove_price = self.current_shop.as_ref().and_then(|shop| {
                    if !shop.removal_used
                        && *idx < self.run_state.deck.len()
                        && self.is_removable_master_deck_index(*idx)
                        && self.run_state.gold >= shop.remove_price
                    {
                        Some(shop.remove_price)
                    } else {
                        None
                    }
                });
                if let Some(price) = remove_price {
                    self.run_state.gold -= price;
                    if self
                        .run_state
                        .relic_flags
                        .has(crate::relic_flags::flag::MAW_BANK)
                    {
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
                return;
            }
            GameAction::ShopLeave => {}
            _ => {}
        }

        self.current_shop = None;
        self.phase = RunPhase::MapChoice;
        self.refresh_decision_stack();
        ()
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
        if event.name == "Dead Adventurer"
            && event
                .options
                .first()
                .is_some_and(|option| option.text == "Initialize Dead Adventurer")
        {
            let mut rewards = [
                crate::events::DeadAdventurerReward::Gold,
                crate::events::DeadAdventurerReward::Nothing,
                crate::events::DeadAdventurerReward::Relic,
            ];
            let seed = self.floor_rngs.misc.random_long_unbounded();
            crate::seed::java_util_shuffle(&mut rewards, seed);
            let enemy = match self.floor_rngs.misc.random_int(2) {
                0 => "3 Sentries",
                1 => "Gremlin Nob",
                _ => "Lagavulin Event",
            };
            *event = crate::events::dead_adventurer_event_with_state(
                self.run_state.ascension,
                rewards,
                enemy,
            );
        } else if event.name == "World of Goop" && event.options.len() == 2 {
            // GoopPuddle constructs this value before either option is chosen,
            // then caps it by current gold. The roll therefore happens even
            // when the player gathers the puddle gold instead.
            // Java: events/exordium/GoopPuddle.java::<init>/goldLoss.
            let (minimum, maximum) = if self.run_state.ascension >= 15 {
                (35, 75)
            } else {
                (20, 50)
            };
            let loss = self
                .floor_rngs
                .misc
                .random_int_range(minimum, maximum)
                .min(self.run_state.gold);
            event.options[1].text = format!("Leave (lose {loss} gold)");
            event.options[1].program =
                crate::events::EventProgram::from_ops(vec![crate::events::EventProgramOp::gold(
                    -loss,
                )]);
        } else if event.name == "Shining Light" && event.options.len() == 2 {
            let percent = if self.run_state.ascension >= 15 {
                30
            } else {
                20
            };
            let damage = (self.run_state.max_hp * percent + 50) / 100;
            event.options[0].text = format!("Enter (upgrade 2 random cards, take {damage} damage)");
        } else if event.name == "Falling" && event.options.len() == 1 {
            let choice = self.build_falling_choice_event();
            event.options[0].program = crate::events::EventProgram::from_ops(vec![
                crate::events::EventProgramOp::continue_event(choice),
            ]);
        } else if event.name == "WeMeetAgain" && event.options.len() == 4 {
            self.run_state.reconcile_deck_card_states();

            // Constructor order is potion shuffle, gold roll, then card
            // shuffle. Each offered identity remains fixed until a choice.
            // Java: events/shrines/WeMeetAgain.java::<init>.
            let mut potion_candidates = self
                .run_state
                .potions
                .iter()
                .enumerate()
                .filter_map(|(slot, potion)| (!potion.is_empty()).then_some((slot, potion.clone())))
                .collect::<Vec<_>>();
            let potion_offer = if potion_candidates.is_empty() {
                None
            } else {
                let seed = self.floor_rngs.misc.random_long_unbounded();
                crate::seed::java_util_shuffle(&mut potion_candidates, seed);
                potion_candidates.into_iter().next()
            };

            let gold_offer = if self.run_state.gold < 50 {
                None
            } else {
                Some(
                    self.floor_rngs
                        .misc
                        .random_int_range(50, self.run_state.gold.min(150)),
                )
            };

            let registry = crate::cards::global_registry();
            let mut card_candidates = self
                .run_state
                .deck
                .iter()
                .enumerate()
                .filter_map(|(index, card_id)| {
                    let def = registry.get(card_id)?;
                    (def.card_type != crate::cards::CardType::Curse
                        && event_card_rarity(card_id) != Some(EventCardRarity::Basic))
                    .then_some((
                        self.run_state.deck_card_states[index].instance_id,
                        card_id.clone(),
                    ))
                })
                .collect::<Vec<_>>();
            let card_offer = if card_candidates.is_empty() {
                None
            } else {
                let seed = self.floor_rngs.misc.random_long_unbounded();
                crate::seed::java_util_shuffle(&mut card_candidates, seed);
                card_candidates.into_iter().next()
            };

            let option = |text: String, ops: Vec<crate::events::EventProgramOp>, legal: bool| {
                if legal {
                    crate::events::TypedEventOption::supported(
                        text,
                        crate::events::EventProgram::from_ops(ops),
                        crate::events::EventEffect::GainRelic,
                    )
                } else {
                    crate::events::TypedEventOption::blocked(
                        text,
                        crate::events::EventProgram::from_ops(ops),
                        crate::events::EventEffect::Nothing,
                        "We Meet Again offer is unavailable",
                    )
                }
            };
            let potion_option = potion_offer.map_or_else(
                || option("No potion to give".to_string(), vec![], false),
                |(slot, potion)| {
                    option(
                        format!("Give potion: {potion}"),
                        vec![
                            crate::events::EventProgramOp::remove_potion_slot(slot),
                            crate::events::EventProgramOp::obtain_random_screenless_relic(),
                        ],
                        true,
                    )
                },
            );
            let gold_option = gold_offer.map_or_else(
                || option("Not enough gold to give".to_string(), vec![], false),
                |amount| {
                    option(
                        format!("Give {amount} gold"),
                        vec![
                            crate::events::EventProgramOp::gold(-amount),
                            crate::events::EventProgramOp::obtain_random_screenless_relic(),
                        ],
                        true,
                    )
                },
            );
            let card_option = card_offer.map_or_else(
                || option("No nonbasic card to give".to_string(), vec![], false),
                |(instance_id, card_id)| {
                    option(
                        format!("Give card: {card_id}"),
                        vec![
                            crate::events::EventProgramOp::remove_offered_deck_card_instance(
                                instance_id,
                            ),
                            crate::events::EventProgramOp::obtain_random_screenless_relic(),
                        ],
                        true,
                    )
                },
            );
            event.options = vec![
                potion_option,
                gold_option,
                card_option,
                crate::events::TypedEventOption::supported(
                    "Leave",
                    crate::events::EventProgram::from_ops(vec![
                        crate::events::EventProgramOp::nothing(),
                    ]),
                    crate::events::EventEffect::Nothing,
                ),
            ];
        } else if event.name == "Designer"
            && event
                .options
                .first()
                .is_some_and(|option| option.text.starts_with("Adjustment (pay gold"))
        {
            // Designer snapshots both variants in its constructor, before the
            // forced intro click reveals the priced main screen.
            // Java: events/shrines/Designer.java::<init>/buttonEffect.
            self.run_state.reconcile_deck_card_states();
            let adjustment_upgrades_one = self.floor_rngs.misc.random_bool();
            let cleanup_removes_one = self.floor_rngs.misc.random_bool();
            let main = self.build_designer_main_event(adjustment_upgrades_one, cleanup_removes_one);
            *event = TypedEventDef {
                name: "Designer".to_string(),
                options: vec![crate::events::TypedEventOption::supported(
                    "Proceed",
                    crate::events::EventProgram::from_ops(vec![
                        crate::events::EventProgramOp::continue_event(main),
                    ]),
                    crate::events::EventEffect::Nothing,
                )],
            };
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
            let gold = if self.run_state.ascension >= 15 {
                50
            } else {
                75
            };
            event.options[0].text = format!("Touch (take {damage} damage, gain {gold} gold)");
        } else if event.name == "Vampires" && event.options.len() == 3 {
            let max_hp_loss = ((self.run_state.max_hp * 30) + 99) / 100;
            event.options[0].text =
                format!("Accept (lose {max_hp_loss} max HP, remove Strikes, gain 5 Bites)");
            if self
                .run_state
                .relics
                .iter()
                .any(|relic| relic == "Blood Vial")
            {
                event.options[1].status = EventRuntimeStatus::Supported;
            } else {
                event.options[1].status = EventRuntimeStatus::Blocked {
                    reason: "requires Blood Vial".to_string(),
                };
            }
        } else if event.name == "N'loth" && event.options.len() == 3 {
            // Nloth.java copies and shuffles the player's relic list, then
            // offers the first two entries after one miscRng-seeded Java
            // shuffle.
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
            let seed = self.floor_rngs.misc.random_long_unbounded();
            crate::seed::java_util_shuffle(&mut candidates, seed);
            let first = candidates[0].clone();
            let second = candidates[1].clone();
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

    fn build_designer_main_event(
        &self,
        adjustment_upgrades_one: bool,
        cleanup_removes_one: bool,
    ) -> TypedEventDef {
        let (adjust_cost, cleanup_cost, full_cost, hp_loss) = if self.run_state.ascension >= 15 {
            (50, 75, 110, 5)
        } else {
            (40, 60, 90, 3)
        };
        let removable_count = (0..self.run_state.deck.len())
            .filter(|index| self.is_removable_master_deck_index(*index))
            .count();
        let has_upgradeable = self
            .run_state
            .deck
            .iter()
            .any(|card| crate::cards::global_registry().can_upgrade_name(card));

        let adjustment_ops = if adjustment_upgrades_one {
            vec![
                crate::events::EventProgramOp::gold(-adjust_cost),
                crate::events::EventProgramOp::deck_selection("deck_selection_event_upgrade"),
            ]
        } else {
            vec![
                crate::events::EventProgramOp::gold(-adjust_cost),
                crate::events::EventProgramOp::upgrade_random_cards(2),
            ]
        };
        let cleanup_ops = if cleanup_removes_one {
            vec![
                crate::events::EventProgramOp::gold(-cleanup_cost),
                crate::events::EventProgramOp::deck_selection("deck_selection_event_remove"),
            ]
        } else {
            vec![
                crate::events::EventProgramOp::gold(-cleanup_cost),
                crate::events::EventProgramOp::deck_selection_many(
                    "deck_selection_event_transform",
                    2,
                ),
            ]
        };
        let options = vec![
            (
                if adjustment_upgrades_one {
                    format!("Pay {adjust_cost} gold: choose 1 card to upgrade")
                } else {
                    format!("Pay {adjust_cost} gold: upgrade 2 random cards")
                },
                adjustment_ops,
                self.run_state.gold >= adjust_cost && has_upgradeable,
                crate::events::EventEffect::UpgradeCard,
            ),
            (
                if cleanup_removes_one {
                    format!("Pay {cleanup_cost} gold: choose 1 card to remove")
                } else {
                    format!("Pay {cleanup_cost} gold: choose 2 cards to transform")
                },
                cleanup_ops,
                self.run_state.gold >= cleanup_cost
                    && removable_count >= if cleanup_removes_one { 1 } else { 2 },
                crate::events::EventEffect::RemoveCard,
            ),
            (
                format!("Pay {full_cost} gold: remove 1 card, then upgrade 1 random card"),
                vec![
                    crate::events::EventProgramOp::gold(-full_cost),
                    crate::events::EventProgramOp::deck_selection("deck_selection_designer_full"),
                ],
                self.run_state.gold >= full_cost && removable_count >= 1,
                crate::events::EventEffect::RemoveCard,
            ),
            (
                format!("Get punched: lose {hp_loss} HP"),
                vec![crate::events::EventProgramOp::hp(-hp_loss)],
                true,
                crate::events::EventEffect::Hp(-hp_loss),
            ),
        ];
        TypedEventDef {
            name: "Designer".to_string(),
            options: options
                .into_iter()
                .map(|(text, ops, legal, effect)| {
                    if legal {
                        crate::events::TypedEventOption::supported(
                            text,
                            crate::events::EventProgram::from_ops(ops),
                            effect,
                        )
                    } else {
                        crate::events::TypedEventOption::blocked(
                            text,
                            crate::events::EventProgram::from_ops(ops),
                            effect,
                            "Designer option requirements are not met",
                        )
                    }
                })
                .collect(),
        }
    }

    fn falling_card_of_type(&mut self, card_type: crate::cards::CardType) -> Option<(u32, String)> {
        self.run_state.reconcile_deck_card_states();
        let registry = crate::cards::global_registry();
        let eligible = self
            .run_state
            .deck
            .iter()
            .enumerate()
            .filter_map(|(index, card_id)| {
                (self.is_removable_master_deck_index(index)
                    && registry
                        .get(card_id)
                        .is_some_and(|def| def.card_type == card_type))
                .then_some(index)
            })
            .collect::<Vec<_>>();
        if eligible.is_empty() {
            return None;
        }
        let index = eligible[self.floor_rngs.misc.random_index(eligible.len())];
        Some((
            self.run_state.deck_card_states[index].instance_id,
            self.run_state.deck[index].clone(),
        ))
    }

    fn build_falling_choice_event(&mut self) -> TypedEventDef {
        // Falling.setCards tests all three types first, then selects in ATTACK,
        // SKILL, POWER order. The dialog itself is SKILL, POWER, ATTACK.
        // Java: events/beyond/Falling.java::setCards/buttonEffect and
        // helpers/CardHelper.java::returnCardOfType.
        let attack = self.falling_card_of_type(crate::cards::CardType::Attack);
        let skill = self.falling_card_of_type(crate::cards::CardType::Skill);
        let power = self.falling_card_of_type(crate::cards::CardType::Power);

        if attack.is_none() && skill.is_none() && power.is_none() {
            return TypedEventDef {
                name: "Falling".to_string(),
                options: vec![crate::events::TypedEventOption::supported(
                    "Continue",
                    crate::events::EventProgram::from_ops(vec![
                        crate::events::EventProgramOp::nothing(),
                    ]),
                    crate::events::EventEffect::Nothing,
                )],
            };
        }

        let option = |kind: &str, selected: Option<(u32, String)>| match selected {
            Some((instance_id, card_id)) => crate::events::TypedEventOption::supported(
                format!("Lose {kind}: {card_id}"),
                crate::events::EventProgram::from_ops(vec![
                    crate::events::EventProgramOp::remove_deck_card_instance(instance_id),
                ]),
                crate::events::EventEffect::RemoveCard,
            ),
            None => crate::events::TypedEventOption::blocked(
                format!("No removable {kind}"),
                crate::events::EventProgram::from_ops(vec![
                    crate::events::EventProgramOp::nothing(),
                ]),
                crate::events::EventEffect::Nothing,
                format!("deck has no removable {kind}"),
            ),
        };
        TypedEventDef {
            name: "Falling".to_string(),
            options: vec![
                option("skill", skill),
                option("power", power),
                option("attack", attack),
            ],
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
            unique_cards.push(self.random_match_and_keep_colorless_uncommon());
            unique_cards.push(self.random_match_and_keep_curse());
        }
        unique_cards.push(self.match_and_keep_starter_card(color).to_string());

        // Every relic previews each of the six cards before the stat-equivalent
        // copies are created. Egg upgrades are the gameplay-visible preview
        // hooks represented by the run layer.
        for card in &mut unique_cards {
            *card = self.upgrade_reward_card_if_needed(card);
        }

        let mut board = unique_cards.clone();
        board.extend(unique_cards);
        // GremlinMatchGame consumes one miscRng.randomLong() and delegates the
        // permutation to java.util.Random/Collections.shuffle.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/events/shrines/GremlinMatchGame.java:56
        let seed = self.floor_rngs.misc.random_long_unbounded();
        crate::seed::java_util_shuffle(&mut board, seed);
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
        if relics
            .iter()
            .any(|relic| matches!(relic.as_str(), "Burning Blood" | "Black Blood"))
        {
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
        if relics.iter().any(|relic| {
            matches!(
                relic.as_str(),
                "PureWater" | "HolyWater" | "VioletLotus" | "Damaru"
            )
        }) {
            return EventCardColor::Purple;
        }

        let deck = &self.run_state.deck;
        if deck
            .iter()
            .any(|card| matches!(card.as_str(), "Bash" | "Strike_R" | "Defend_R"))
        {
            return EventCardColor::Red;
        }
        if deck
            .iter()
            .any(|card| matches!(card.as_str(), "Neutralize" | "Strike_G" | "Defend_G"))
        {
            return EventCardColor::Green;
        }
        if deck
            .iter()
            .any(|card| matches!(card.as_str(), "Zap" | "Strike_B" | "Defend_B"))
        {
            return EventCardColor::Blue;
        }
        if deck
            .iter()
            .any(|card| matches!(card.as_str(), "Eruption" | "Strike_P" | "Defend_P"))
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
            EventCardColor::Purple | EventCardColor::Colorless | EventCardColor::Curse => {
                "Eruption"
            }
        }
    }

    fn random_match_and_keep_colored_card(
        &mut self,
        color: EventCardColor,
        rarity: EventCardRarity,
    ) -> String {
        let mut candidates = if color == EventCardColor::Purple {
            self.card_pools.working(rarity).to_vec()
        } else {
            matching_event_cards(color, rarity)
        };
        if candidates.is_empty() {
            candidates = match (color, rarity) {
                (EventCardColor::Purple, EventCardRarity::Common) => WATCHER_COMMON_CARDS
                    .iter()
                    .map(|id| (*id).to_string())
                    .collect(),
                (EventCardColor::Purple, EventCardRarity::Uncommon) => WATCHER_UNCOMMON_CARDS
                    .iter()
                    .map(|id| (*id).to_string())
                    .collect(),
                (EventCardColor::Purple, EventCardRarity::Rare) => WATCHER_RARE_CARDS
                    .iter()
                    .map(|id| (*id).to_string())
                    .collect(),
                _ => vec![self.match_and_keep_starter_card(color).to_string()],
            };
        }
        let idx = self.persistent_rngs.card.random_index(candidates.len());
        candidates[idx].clone()
    }

    fn random_match_and_keep_curse(&mut self) -> String {
        let idx = self
            .persistent_rngs
            .card
            .random_index(RANDOM_OBTAINABLE_CURSES.len());
        RANDOM_OBTAINABLE_CURSES[idx].to_string()
    }

    fn random_match_and_keep_colorless_uncommon(&mut self) -> String {
        // returnColorlessCard shuffles the complete mutable colorless pool
        // from one shuffleRng.randomLong(), then returns the first card with
        // the requested rarity without removing it.
        // Java: AbstractDungeon.java::returnColorlessCard.
        let seed = self.floor_rngs.shuffle.random_long_unbounded();
        crate::seed::java_util_shuffle(&mut self.card_pools.colorless, seed);
        self.card_pools
            .colorless
            .iter()
            .find(|card| event_card_rarity(card) == Some(EventCardRarity::Uncommon))
            .cloned()
            .unwrap_or_else(|| "Swift Strike".to_string())
    }

    fn sync_match_and_keep_event(&mut self) {
        let Some(state) = &self.match_and_keep_state else {
            return;
        };

        let options = match state.screen {
            MatchAndKeepScreen::Intro => vec![crate::events::TypedEventOption::supported(
                "Hear the rules",
                crate::events::EventProgram::from_ops(vec![
                    crate::events::EventProgramOp::nothing(),
                ]),
                crate::events::EventEffect::Nothing,
            )],
            MatchAndKeepScreen::RuleExplanation => {
                vec![crate::events::TypedEventOption::supported(
                    "Begin the match game",
                    crate::events::EventProgram::from_ops(vec![
                        crate::events::EventProgramOp::nothing(),
                    ]),
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
                        format!(
                            "Pick slot {} ({} attempts left)",
                            idx + 1,
                            state.attempts_left
                        )
                    };
                    crate::events::TypedEventOption::supported(
                        label,
                        crate::events::EventProgram::from_ops(vec![
                            crate::events::EventProgramOp::nothing(),
                        ]),
                        crate::events::EventEffect::Nothing,
                    )
                })
                .collect(),
            MatchAndKeepScreen::Complete => vec![crate::events::TypedEventOption::supported(
                "Leave",
                crate::events::EventProgram::from_ops(vec![
                    crate::events::EventProgramOp::nothing(),
                ]),
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
                crate::events::EventProgram::from_ops(vec![
                    crate::events::EventProgramOp::nothing(),
                ]),
                crate::events::EventEffect::Nothing,
            )]
        } else {
            vec![
                crate::events::TypedEventOption::supported(
                    format!(
                        "Reach inside (take {} dmg, {}% relic chance)",
                        state.damage, state.relic_chance
                    ),
                    crate::events::EventProgram::from_ops(vec![
                        crate::events::EventProgramOp::nothing(),
                    ]),
                    crate::events::EventEffect::DamageAndGold(-state.damage, 0),
                ),
                crate::events::TypedEventOption::supported(
                    "Leave",
                    crate::events::EventProgram::from_ops(vec![
                        crate::events::EventProgramOp::nothing(),
                    ]),
                    crate::events::EventEffect::Nothing,
                ),
            ]
        };
        self.current_event = Some(TypedEventDef {
            name: "Scrap Ooze".to_string(),
            options,
        });
    }

    fn sync_spire_heart_event(&mut self) {
        let Some(screen) = self.spire_heart_state.map(|state| state.screen) else {
            return;
        };
        let label = match screen {
            SpireHeartScreen::Intro => "Approach the Heart",
            SpireHeartScreen::Middle => "Strike the Heart",
            SpireHeartScreen::Middle2 => "Continue",
            SpireHeartScreen::Death => "Complete the run",
            SpireHeartScreen::GoToEnding => "Enter Act 4",
        };
        self.current_event = Some(TypedEventDef {
            name: "Spire Heart".to_string(),
            options: vec![crate::events::TypedEventOption::supported(
                label,
                crate::events::EventProgram::from_ops(vec![
                    crate::events::EventProgramOp::nothing(),
                ]),
                crate::events::EventEffect::Nothing,
            )],
        });
    }

    fn next_event_roll_100(&mut self) -> usize {
        if !self.forced_event_rolls.is_empty() {
            return self.forced_event_rolls.remove(0).min(99);
        }
        // AbstractDungeon.nextRoomTransition reconstructs a seed+counter
        // duplicate, rolls the mystery-room type, then commits that duplicate.
        let mut duplicate =
            crate::seed::StsRandom::with_counter(self.seed, self.persistent_rngs.event.counter);
        let roll = (duplicate.random_f32() * 100.0) as usize;
        self.persistent_rngs.event = duplicate;
        roll
    }

    fn initialize_dynamic_event_state(&mut self) {
        self.match_and_keep_state = None;
        self.scrap_ooze_state = None;
        self.spire_heart_state = None;
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
            } else if event_name == "Spire Heart" {
                self.spire_heart_state = Some(SpireHeartState {
                    screen: SpireHeartScreen::Intro,
                });
                self.sync_spire_heart_event();
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
            let counter =
                &mut self.run_state.relic_flags.counters[crate::relic_flags::counter::TINY_CHEST];
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
        enum MysteryResult {
            Monster,
            Shop,
            Treasure,
            Event,
        }

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
                self.enter_treasure_room();
            }
            MysteryResult::Event => self.enter_event(),
        }
    }

    fn deck_contains_curse(&self) -> bool {
        let registry = crate::cards::global_registry();
        self.run_state.deck.iter().any(|card_id| {
            let base_id = card_id.trim_end_matches('+');
            !matches!(
                base_id,
                "Necronomicurse" | "CurseOfTheBell" | "AscendersBane"
            ) && registry
                .get(card_id)
                .is_some_and(|def| def.card_type == crate::cards::CardType::Curse)
        })
    }

    fn regular_event_is_eligible(&self, event_name: &str) -> bool {
        match event_name {
            "Dead Adventurer" | "Mushrooms" => self.run_state.floor > 6,
            "The Moai Head" => {
                self.run_state.relics.iter().any(|id| id == "Golden Idol")
                    || (self.run_state.current_hp as f32 / self.run_state.max_hp as f32) <= 0.5
            }
            "The Cleric" => self.run_state.gold >= 35,
            "Beggar" => self.run_state.gold >= 75,
            "Colosseum" => self.run_state.map_y > (self.map.rows.len() / 2) as i32,
            _ => true,
        }
    }

    fn one_time_shrine_is_eligible(&self, event_name: &str) -> bool {
        match event_name {
            "Fountain of Cleansing" => self.deck_contains_curse(),
            "Designer" => matches!(self.run_state.act, 2 | 3) && self.run_state.gold >= 75,
            "Duplicator" => matches!(self.run_state.act, 2 | 3),
            "FaceTrader" => matches!(self.run_state.act, 1 | 2),
            "Knowing Skull" => self.run_state.act == 2 && self.run_state.current_hp > 12,
            // The duplicated city comparison in N'loth.java/getShrine still
            // reduces to Act 2 plus at least two owned relics.
            "N'loth" => self.run_state.act == 2 && self.run_state.relics.len() >= 2,
            "The Joust" => self.run_state.act == 2 && self.run_state.gold >= 50,
            "The Woman in Blue" => self.run_state.gold >= 50,
            "SecretPortal" => self.run_state.act == 3 && self.run_state.playtime_seconds >= 800.0,
            _ => true,
        }
    }

    fn take_regular_event(&mut self, rng: &mut crate::seed::StsRandom) -> Option<String> {
        let eligible: Vec<String> = self
            .event_pools
            .regular
            .iter()
            .filter(|event_name| self.regular_event_is_eligible(event_name))
            .cloned()
            .collect();
        if eligible.is_empty() {
            return None;
        }
        let selected = eligible[rng.random_index(eligible.len())].clone();
        if let Some(index) = self
            .event_pools
            .regular
            .iter()
            .position(|event_name| event_name == &selected)
        {
            self.event_pools.regular.remove(index);
        }
        Some(selected)
    }

    fn take_shrine_event(&mut self, rng: &mut crate::seed::StsRandom) -> Option<String> {
        let mut eligible = self.event_pools.shrines.clone();
        eligible.extend(
            self.event_pools
                .one_time_shrines
                .iter()
                .filter(|event_name| self.one_time_shrine_is_eligible(event_name))
                .cloned(),
        );
        if eligible.is_empty() {
            return None;
        }
        let selected = eligible[rng.random_index(eligible.len())].clone();
        self.event_pools
            .shrines
            .retain(|event_name| event_name != &selected);
        self.event_pools
            .one_time_shrines
            .retain(|event_name| event_name != &selected);
        Some(selected)
    }

    fn take_event_key(&mut self, rng: &mut crate::seed::StsRandom) -> Option<String> {
        // Re-verified: generateEvent always consumes random(1.0f) before it
        // checks pool availability; getEvent/getShrine then consume exactly one
        // inclusive index draw. The EventRoom duplicate is discarded afterward.
        let choose_shrine = rng.random_f32_range(0.0, 1.0) < 0.25;
        if choose_shrine {
            if !self.event_pools.shrines.is_empty() || !self.event_pools.one_time_shrines.is_empty()
            {
                return self.take_shrine_event(rng);
            }
            return self.take_regular_event(rng);
        }
        self.take_regular_event(rng)
            .or_else(|| self.take_shrine_event(rng))
    }

    fn typed_event_for_pool_key(event_name: &str) -> Option<TypedEventDef> {
        let display_name = match event_name {
            "Ghosts" => "Council of Ghosts",
            "MindBloom" => "Mind Bloom",
            "SensoryStone" => "Sensory Stone",
            "SecretPortal" => "Secret Portal",
            other => other,
        };
        (1..=3)
            .flat_map(typed_events_for_act)
            .chain(crate::events::typed_shrine_events())
            .find(|event| event.name == display_name)
    }

    fn enter_event(&mut self) {
        // EventRoom.onPlayerEntry uses another seed+counter duplicate for event
        // selection and deliberately does not commit it to AbstractDungeon.eventRng.
        let mut duplicate =
            crate::seed::StsRandom::with_counter(self.seed, self.persistent_rngs.event.counter);
        let event_name = self
            .take_event_key(&mut duplicate)
            .expect("Java event pools must contain an eligible event or shrine");
        let mut event = Self::typed_event_for_pool_key(&event_name)
            .unwrap_or_else(|| panic!("missing typed event for Java pool key {event_name}"));
        self.normalize_event_runtime_state(&mut event);
        self.current_event = Some(event);
        self.initialize_dynamic_event_state();
        self.phase = RunPhase::Event;
        self.refresh_decision_stack();
    }

    fn step_match_and_keep(&mut self, choice_idx: usize) {
        let Some(state) = self.match_and_keep_state.as_mut() else {
            return;
        };

        match state.screen {
            MatchAndKeepScreen::Intro => {
                state.screen = MatchAndKeepScreen::RuleExplanation;
                self.sync_match_and_keep_event();
                self.phase = RunPhase::Event;
                self.refresh_decision_stack();
                ()
            }
            MatchAndKeepScreen::RuleExplanation => {
                state.screen = MatchAndKeepScreen::Play;
                self.sync_match_and_keep_event();
                self.phase = RunPhase::Event;
                self.refresh_decision_stack();
                ()
            }
            MatchAndKeepScreen::Play => {
                if choice_idx >= state.board.len() || state.first_pick == Some(choice_idx) {
                    self.refresh_decision_stack();
                    return;
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
                ()
            }
            MatchAndKeepScreen::Complete => {
                self.current_event = None;
                self.match_and_keep_state = None;
                self.phase = RunPhase::MapChoice;
                self.refresh_decision_stack();
                ()
            }
        }
    }

    fn step_scrap_ooze(&mut self, choice_idx: usize) {
        let Some(state_snapshot) = self.scrap_ooze_state.clone() else {
            return;
        };

        if state_snapshot.leave_only {
            self.current_event = None;
            self.scrap_ooze_state = None;
            self.phase = RunPhase::MapChoice;
            self.refresh_decision_stack();
            return;
        }

        match choice_idx {
            0 => {
                self.run_state.current_hp =
                    (self.run_state.current_hp - state_snapshot.damage).max(0);
                if self.run_state.current_hp <= 0 {
                    self.current_event = None;
                    self.scrap_ooze_state = None;
                    self.run_state.run_over = true;
                    self.phase = RunPhase::GameOver;
                    self.refresh_decision_stack();
                    return;
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
                ()
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
                ()
            }
            _ => (),
        }
    }

    fn consume_watcher_spire_heart_slash_rng(&mut self) {
        // Watcher supplies six effects in this order. SpireHeart first rolls
        // each delay, then DamageHeartEffect's constructor rolls heavy-blunt
        // rotation (when applicable), x jitter, and y jitter.
        // Java: characters/Watcher.java::getSpireHeartSlashEffect,
        // events/beyond/SpireHeart.java::buttonEffect,
        // vfx/DamageHeartEffect.java::<init>/loadImage.
        for blunt_heavy in [false, true, false, true, true, false] {
            let _delay = self.ambient_math_rng.random_f32_range(0.05, 0.2);
            if blunt_heavy {
                let _rotation = self.ambient_math_rng.random_f32_range(0.0, 360.0);
            }
            let _x = self.ambient_math_rng.random_f32_range(-150.0, 150.0);
            let _y = self.ambient_math_rng.random_f32_range(-150.0, 150.0);
        }
    }

    fn step_spire_heart(&mut self, choice_idx: usize) {
        if choice_idx != 0 {
            return;
        }
        let Some(screen) = self.spire_heart_state.map(|state| state.screen) else {
            return;
        };

        match screen {
            SpireHeartScreen::Intro => {
                self.spire_heart_state = Some(SpireHeartState {
                    screen: SpireHeartScreen::Middle,
                });
            }
            SpireHeartScreen::Middle => {
                self.consume_watcher_spire_heart_slash_rng();
                self.spire_heart_state = Some(SpireHeartState {
                    screen: SpireHeartScreen::Middle2,
                });
            }
            SpireHeartScreen::Middle2 => {
                self.spire_heart_state = Some(SpireHeartState {
                    screen: if self.can_enter_final_act() {
                        SpireHeartScreen::GoToEnding
                    } else {
                        SpireHeartScreen::Death
                    },
                });
            }
            SpireHeartScreen::Death => {
                self.current_event = None;
                self.spire_heart_state = None;
                self.resolve_terminal_run_victory();
                self.refresh_decision_stack();
                return;
            }
            SpireHeartScreen::GoToEnding => {
                self.current_event = None;
                self.spire_heart_state = None;
                // DoorUnlockScreen's frame/render VFX share process-global
                // MathUtils state and are intentionally outside the causal
                // headless core. Oracle replay restores the captured settled
                // ambient state at the next causal checkpoint.
                self.start_final_act();
                self.refresh_decision_stack();
                return;
            }
        }

        self.sync_spire_heart_event();
        self.phase = RunPhase::Event;
        self.refresh_decision_stack();
    }

    fn step_event(&mut self, action: &GameAction) {
        let choice_idx = match action {
            GameAction::EventChoice(idx) => *idx,
            _ => 0,
        };

        if self.spire_heart_state.is_some() {
            return self.step_spire_heart(choice_idx);
        }
        if self.scrap_ooze_state.is_some() {
            return self.step_scrap_ooze(choice_idx);
        }
        if self.match_and_keep_state.is_some() {
            return self.step_match_and_keep(choice_idx);
        }

        let Some(event) = self.current_event.clone() else {
            self.phase = RunPhase::MapChoice;
            self.refresh_decision_stack();
            return;
        };
        let Some(option) = event.options.get(choice_idx).cloned() else {
            self.current_event = None;
            self.phase = RunPhase::MapChoice;
            self.refresh_decision_stack();
            return;
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
            self.start_event_combat(resolved_enemies);
            self.refresh_decision_stack();
            return;
        }

        if let Some(mut event) = next_event {
            self.normalize_event_runtime_state(&mut event);
            self.current_event = Some(event);
            self.phase = RunPhase::Event;
            self.refresh_decision_stack();
            return;
        }

        if start_boss_combat {
            self.current_event = None;
            self.pending_event_combat = None;
            // SecretPortal replaces the next room node with the Act 3 boss;
            // nextRoomTransition still advances floorNum by exactly one.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/events/beyond/SecretPortal.java:43
            self.run_state.map_x = -1;
            self.run_state.map_y = 15;
            self.run_state.floor += 1;
            self.reset_floor_rngs();
            self.enter_combat(false, true);
            self.refresh_decision_stack();
            return;
        }

        if start_final_act {
            self.current_event = None;
            self.pending_event_combat = None;
            self.start_final_act();
            self.refresh_decision_stack();
            return;
        }

        self.current_event = None;
        if died {
            self.run_state.run_over = true;
            self.phase = RunPhase::GameOver;
            self.refresh_decision_stack();
            return;
        }

        if run_won {
            self.resolve_terminal_run_victory();
            self.refresh_decision_stack();
            return;
        }

        if !reward_items.is_empty() {
            self.build_event_reward_screen(reward_items);
            self.phase = RunPhase::CardReward;
        } else {
            self.phase = RunPhase::MapChoice;
        }
        self.refresh_decision_stack();
        ()
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
            EventProgramOp::CombatBranch {
                enemies,
                on_win,
                precombat_random_gold,
            } => {
                let mut on_win = *on_win.clone();
                if let Some((min_gold, max_gold)) = precombat_random_gold {
                    let gold = self.floor_rngs.misc.random_int_range(*min_gold, *max_gold);
                    on_win.ops.insert(0, EventProgramOp::gain_gold_reward(gold));
                }
                self.resolve_random_event_reward_labels(&mut on_win);
                EventProgramFlow::StartCombat(PendingEventCombat {
                    enemies: enemies.clone(),
                    on_win,
                })
            }
            EventProgramOp::PrepareCombatBranch {
                enemies,
                on_win,
                precombat_random_gold,
            } => {
                let mut on_win = *on_win.clone();
                if let Some((min_gold, max_gold)) = precombat_random_gold {
                    let gold = self.floor_rngs.misc.random_int_range(*min_gold, *max_gold);
                    on_win.ops.insert(0, EventProgramOp::gain_gold_reward(gold));
                }
                let event_name = self
                    .current_event
                    .as_ref()
                    .map(|event| event.name.clone())
                    .unwrap_or_else(|| "Prepared Combat".to_string());
                EventProgramFlow::ContinueEvent(TypedEventDef {
                    name: event_name,
                    options: vec![crate::events::TypedEventOption::supported(
                        "Fight",
                        crate::events::EventProgram::from_ops(vec![EventProgramOp::CombatBranch {
                            enemies: enemies.clone(),
                            on_win: Box::new(on_win),
                            precombat_random_gold: None,
                        }]),
                        crate::events::EventEffect::Nothing,
                    )],
                })
            }
            EventProgramOp::StartBossCombat => EventProgramFlow::StartBossCombat,
            EventProgramOp::RandomOutcomeTable { outcomes } => {
                if outcomes.is_empty() {
                    return EventProgramFlow::Continue;
                }
                let idx = self.floor_rngs.misc.random_index(outcomes.len());
                self.apply_event_program(&outcomes[idx], reward_items)
            }
            EventProgramOp::RandomBooleanOutcome {
                if_true,
                if_false,
                force_true_at_ascension15,
            } => {
                // TheMausoleum always consumes randomBoolean before A15 forces
                // the result true. Keep this general typed operation explicit
                // so cursor behavior cannot be hidden in a weighted int table.
                let rolled = self.floor_rngs.misc.random_bool();
                let result =
                    rolled || (*force_true_at_ascension15 && self.run_state.ascension >= 15);
                self.apply_event_program(if result { if_true } else { if_false }, reward_items)
            }
            EventProgramOp::DeckSelection { label } => {
                if let Some(item) = self.event_deck_selection_item(
                    label,
                    reward_items.len(),
                    reward_items.is_empty(),
                ) {
                    reward_items.push(item);
                }
                EventProgramFlow::Continue
            }
            EventProgramOp::DeckSelectionMany { label, count } => {
                if *count == 0 {
                    return EventProgramFlow::Continue;
                }
                if let Some(item) = self.event_deck_selection_item(
                    label,
                    reward_items.len(),
                    reward_items.is_empty(),
                ) {
                    self.pending_event_deck_selection = Some(PendingEventDeckSelection {
                        label: label.clone(),
                        remaining: *count,
                    });
                    reward_items.push(item);
                }
                EventProgramFlow::Continue
            }
            EventProgramOp::AdjustHp { amount } => {
                if *amount > 0 {
                    self.heal_run_player(*amount);
                } else {
                    self.run_state.current_hp = (self.run_state.current_hp + amount)
                        .max(0)
                        .min(self.run_state.max_hp);
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
                    self.run_state.current_hp = (self.run_state.current_hp + amount)
                        .max(0)
                        .min(self.run_state.max_hp);
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
            EventProgramOp::LoseHpPercentRoundedByAscension {
                base_percent,
                asc15_percent,
            } => {
                let percent = if self.run_state.ascension >= 15 {
                    *asc15_percent
                } else {
                    *base_percent
                };
                let damage = (self.run_state.max_hp * percent + 50) / 100;
                self.run_state.current_hp = (self.run_state.current_hp - damage).max(0);
                if self.run_state.current_hp <= 0 {
                    EventProgramFlow::Died
                } else {
                    EventProgramFlow::Continue
                }
            }
            EventProgramOp::ConsumeMiscLong => {
                let _ = self.floor_rngs.misc.random_long_unbounded();
                EventProgramFlow::Continue
            }
            EventProgramOp::ConsumeAmbientIntPerCurrentGold { max_inclusive } => {
                // MaskedBandits.stealGold selects a random bandit once per
                // penny before the player's gold is removed. MonsterGroup's
                // no-RNG overload uses process-global MathUtils.random, whose
                // later shop-card draws are gameplay-visible.
                // Java: events/city/MaskedBandits.java::stealGold,
                // monsters/MonsterGroup.java::getRandomMonster.
                for _ in 0..self.run_state.gold.max(0) {
                    let _ = self.ambient_math_rng.random_int(*max_inclusive);
                }
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
                self.run_state.current_hp = (self.run_state.current_hp + amount)
                    .max(0)
                    .min(self.run_state.max_hp);
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
            EventProgramOp::LoseMaxHpPercentCeil { percent } => {
                let loss = ((self.run_state.max_hp * (*percent).max(0)) + 99) / 100;
                let loss = loss.min(self.run_state.max_hp.saturating_sub(1));
                self.run_state.max_hp = (self.run_state.max_hp - loss).max(1);
                self.run_state.current_hp = self.run_state.current_hp.min(self.run_state.max_hp);
                EventProgramFlow::Continue
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
                let owner_wins = self.floor_rngs.misc.random_bool_chance(0.3);
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
                let gold = if self.run_state.ascension >= 15 {
                    50
                } else {
                    75
                };
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
                let mut candidates = FACE_IDS
                    .iter()
                    .copied()
                    .filter(|face| !self.run_state.relics.iter().any(|owned| owned == face))
                    .collect::<Vec<_>>();
                if candidates.is_empty() {
                    candidates.push("Circlet");
                }
                let seed = self.floor_rngs.misc.random_long_unbounded();
                crate::seed::java_util_shuffle(&mut candidates, seed);
                let face = candidates[0];
                self.add_relic_reward(face);
                EventProgramFlow::Continue
            }
            EventProgramOp::ReplaceStarterStrikes { replacement, count } => {
                self.run_state.reconcile_deck_card_states();
                let mut remove = self
                    .run_state
                    .deck
                    .iter()
                    .enumerate()
                    .filter_map(|(index, card_id)| is_starter_strike(card_id).then_some(index))
                    .collect::<Vec<_>>();
                remove.reverse();
                for index in remove {
                    self.remove_master_deck_card(index);
                }
                for _ in 0..*count {
                    obtain_master_deck_card_state(&mut self.run_state, replacement.clone());
                }
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
            EventProgramOp::UpgradeRandomCards { count } => {
                let mut indices = self
                    .run_state
                    .deck
                    .iter()
                    .enumerate()
                    .filter_map(|(index, card)| {
                        crate::cards::global_registry()
                            .can_upgrade_name(card)
                            .then_some(index)
                    })
                    .collect::<Vec<_>>();
                // ShiningLight and Designer consume the misc seed even when
                // the eligible list is empty or has one card.
                let seed = self.floor_rngs.misc.random_long_unbounded();
                crate::seed::java_util_shuffle(&mut indices, seed);
                for index in indices.into_iter().take(*count) {
                    self.run_state.upgrade_deck_card(index);
                }
                EventProgramFlow::Continue
            }
            EventProgramOp::RemoveDeckCardInstance { instance_id } => {
                self.run_state.reconcile_deck_card_states();
                if let Some(index) = self
                    .run_state
                    .deck_card_states
                    .iter()
                    .position(|card| card.instance_id == *instance_id)
                {
                    self.remove_master_deck_card(index);
                }
                EventProgramFlow::Continue
            }
            EventProgramOp::RemoveOfferedDeckCardInstance { instance_id } => {
                self.run_state.reconcile_deck_card_states();
                if let Some(index) = self
                    .run_state
                    .deck_card_states
                    .iter()
                    .position(|card| card.instance_id == *instance_id)
                {
                    let removed = self.run_state.deck.remove(index);
                    self.run_state.deck_card_states.remove(index);
                    for bottle in [
                        &mut self.run_state.bottled_flame_card_instance_id,
                        &mut self.run_state.bottled_lightning_card_instance_id,
                        &mut self.run_state.bottled_tornado_card_instance_id,
                    ] {
                        if *bottle == Some(*instance_id) {
                            *bottle = None;
                        }
                    }
                    self.apply_master_deck_removal_hook(&removed);
                }
                EventProgramFlow::Continue
            }
            EventProgramOp::RemoveAllRemovableCurses => {
                self.run_state.reconcile_deck_card_states();
                let registry = crate::cards::global_registry();
                for index in (0..self.run_state.deck.len()).rev() {
                    let is_curse = registry
                        .get(&self.run_state.deck[index])
                        .is_some_and(|def| def.card_type == crate::cards::CardType::Curse);
                    if is_curse && self.is_removable_master_deck_index(index) {
                        self.remove_master_deck_card(index);
                    }
                }
                EventProgramFlow::Continue
            }
            EventProgramOp::RemovePotionSlot { slot } => {
                if let Some(potion) = self.run_state.potions.get_mut(*slot) {
                    potion.clear();
                }
                EventProgramFlow::Continue
            }
            EventProgramOp::ObtainRandomScreenlessRelic => {
                let relic = self.roll_screenless_relic_id();
                self.add_relic_reward(&relic);
                EventProgramFlow::Continue
            }
            EventProgramOp::Reward(reward) => {
                self.apply_event_reward(reward, reward_items);
                EventProgramFlow::Continue
            }
            EventProgramOp::Nothing => EventProgramFlow::Continue,
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
                    let idx = self
                        .floor_rngs
                        .card_random
                        .random_index(WATCHER_COMMON_CARDS.len());
                    obtain_master_deck_card_state(
                        &mut self.run_state,
                        WATCHER_COMMON_CARDS[idx].to_string(),
                    );
                }
            }
            EventDeckMutation::RemoveCard { count } => {
                for _ in 0..*count {
                    if self.run_state.deck.len() > 5 {
                        let idx = self.floor_rngs.misc.random_index(self.run_state.deck.len());
                        self.remove_master_deck_card(idx);
                    }
                }
            }
            EventDeckMutation::TransformCard { count } => {
                for _ in 0..*count {
                    if self.run_state.deck.len() > 5 {
                        let idx = self.floor_rngs.misc.random_index(self.run_state.deck.len());
                        self.remove_master_deck_card(idx);
                    }
                    let card_idx = self
                        .floor_rngs
                        .card_random
                        .random_index(WATCHER_COMMON_CARDS.len());
                    obtain_master_deck_card_state(
                        &mut self.run_state,
                        WATCHER_COMMON_CARDS[card_idx].to_string(),
                    );
                }
            }
            EventDeckMutation::DuplicateCard { count } => {
                for _ in 0..*count {
                    if !self.run_state.deck.is_empty() {
                        let idx = self.floor_rngs.misc.random_index(self.run_state.deck.len());
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
            EventReward::GoldReward { amount } => {
                let amount = (*amount).max(0);
                reward_items.push(RewardItem {
                    index: reward_items.len(),
                    kind: RewardItemKind::Gold,
                    state: RewardItemState::Available,
                    label: amount.to_string(),
                    claimable: reward_items.is_empty(),
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: Vec::new(),
                });
            }
            EventReward::MaxHp { amount } => {
                self.run_state.max_hp = (self.run_state.max_hp + amount).max(1);
                self.run_state.current_hp = (self.run_state.current_hp + amount)
                    .max(0)
                    .min(self.run_state.max_hp);
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
                        choices: self.generate_card_reward_choices(3, CardRewardContext::Standard),
                    });
                }
            }
            EventReward::ColorlessCard { count } => {
                for _ in 0..*count {
                    reward_items.push(RewardItem {
                        index: reward_items.len(),
                        kind: RewardItemKind::CardChoice,
                        state: RewardItemState::Available,
                        label: "event_colorless_card_reward".to_string(),
                        claimable: reward_items.is_empty(),
                        active: false,
                        skip_allowed: false,
                        skip_label: None,
                        choices: self
                            .generate_colorless_reward_choices(self.card_reward_choice_count()),
                    });
                }
            }
            EventReward::StoredNoteCard => {
                let card_id =
                    self.upgrade_reward_card_if_needed(self.current_note_for_yourself_card());
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
            EventReward::SpecificCardCopiesByAscension {
                label,
                base_count,
                asc15_count,
            } => {
                let count = if self.run_state.ascension >= 15 {
                    *asc15_count
                } else {
                    *base_count
                };
                for _ in 0..count {
                    obtain_master_deck_card_state(&mut self.run_state, label.clone());
                }
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
            let mut bosses = ACT1_BOSS_POOL.to_vec();
            let seed = self.floor_rngs.misc.random_long_unbounded();
            crate::seed::java_util_shuffle(&mut bosses, seed);
            return vec![bosses[0].to_string()];
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

    fn resolve_random_event_reward_labels(&mut self, program: &mut crate::events::EventProgram) {
        for op in &mut program.ops {
            let EventProgramOp::Reward(crate::events::EventReward::Relic { label }) = op else {
                continue;
            };
            if matches!(
                label.as_str(),
                "random relic" | "rare relic" | "uncommon relic" | "Cursed Tome reward"
            ) {
                *label = self.resolve_event_relic_reward_label(label);
            }
        }
    }

    fn roll_uncommon_event_relic_id(&mut self) -> String {
        self.draw_relic_from_pool(RelicTier::Uncommon, false, false)
    }

    fn roll_rare_event_relic_id(&mut self) -> String {
        self.draw_relic_from_pool(RelicTier::Rare, false, false)
    }

    fn roll_cursed_tome_book_id(&mut self) -> String {
        const CURSED_TOME_BOOKS: &[&str] = &["Necronomicon", "Enchiridion", "Nilry's Codex"];

        // CursedTome.java::randomBook considers these exact three SPECIAL
        // relics, excludes owned books, and uses Circlet only when all three
        // are already owned. They must not be filtered through ordinary relic
        // reward registry metadata.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/events/city/CursedTome.java
        let mut candidates: Vec<&str> = CURSED_TOME_BOOKS
            .iter()
            .copied()
            .filter(|id| !self.run_state.relics.iter().any(|owned| owned == id))
            .collect();
        if candidates.is_empty() {
            candidates.push("Circlet");
        }
        let idx = self.floor_rngs.misc.random_index(candidates.len());
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

    /// Inspect the current canonical shop state.
    pub fn get_shop(&self) -> Option<&ShopState> {
        self.current_shop.as_ref()
    }

    /// Inspect the current canonical combat state.
    pub fn get_combat_engine(&self) -> Option<&CombatEngine> {
        self.combat_engine.as_ref()
    }

    /// RNG stream counters currently tracked by this run, keyed by the
    /// vault's canonical short names (`docs/vault/rng-system-analysis.md`).
    /// Used by `bin/trace_replay.rs` (U05) to populate `trace::PostState::rng`.
    ///
    /// Java owns seven persistent dungeon streams, five streams reset from
    /// `seed + floor`, and one act-local map stream. Report all thirteen at
    /// every action so a missing stream can never hide a parity divergence.
    ///
    /// Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:388-401,1737-1741
    pub fn rng_counters(&self) -> HashMap<String, i64> {
        let mut counters = HashMap::with_capacity(13);
        counters.insert(
            "card".to_string(),
            i64::from(self.persistent_rngs.card.counter),
        );
        counters.insert(
            "monster".to_string(),
            i64::from(self.persistent_rngs.monster.counter),
        );
        counters.insert(
            "event".to_string(),
            i64::from(self.persistent_rngs.event.counter),
        );
        counters.insert(
            "relic".to_string(),
            i64::from(self.persistent_rngs.relic.counter),
        );
        counters.insert(
            "treasure".to_string(),
            i64::from(self.persistent_rngs.treasure.counter),
        );
        counters.insert(
            "potion".to_string(),
            i64::from(self.persistent_rngs.potion.counter),
        );
        counters.insert(
            "merchant".to_string(),
            i64::from(self.persistent_rngs.merchant.counter),
        );
        counters.insert(
            "monsterHp".to_string(),
            i64::from(self.floor_rngs.monster_hp.counter),
        );
        counters.insert("ai".to_string(), i64::from(self.floor_rngs.ai.counter));
        counters.insert(
            "shuffle".to_string(),
            i64::from(self.floor_rngs.shuffle.counter),
        );
        counters.insert(
            "cardRandom".to_string(),
            i64::from(self.floor_rngs.card_random.counter),
        );
        counters.insert("misc".to_string(), i64::from(self.floor_rngs.misc.counter));
        counters.insert("map".to_string(), i64::from(self.map_rng.counter));

        if let Some(combat) = &self.combat_engine {
            counters.insert("card".to_string(), i64::from(combat.card_rng.counter));
            counters.insert(
                "monsterHp".to_string(),
                i64::from(combat.monster_hp_rng.counter),
            );
            counters.insert("potion".to_string(), i64::from(combat.potion_rng.counter));
            counters.insert("ai".to_string(), i64::from(combat.ai_rng.counter));
            counters.insert("shuffle".to_string(), i64::from(combat.shuffle_rng.counter));
            counters.insert(
                "cardRandom".to_string(),
                i64::from(combat.card_random_rng.counter),
            );
            counters.insert("misc".to_string(), i64::from(combat.misc_rng.counter));
        }
        counters
    }

    /// Exact raw state for every counted StS RNG owned by the run.
    ///
    /// Counters alone cannot prove deterministic continuation because bounded
    /// rejection may consume multiple `RandomXS128` transitions inside one
    /// wrapper call. Active combat owns seven of these streams temporarily,
    /// so its snapshot replaces the corresponding run-level values here.
    pub(crate) fn rng_state_tuples(&self) -> std::collections::BTreeMap<String, (u64, u64, i32)> {
        let mut states = std::collections::BTreeMap::new();
        states.insert("card".to_string(), self.persistent_rngs.card.state_tuple());
        states.insert(
            "monster".to_string(),
            self.persistent_rngs.monster.state_tuple(),
        );
        states.insert(
            "event".to_string(),
            self.persistent_rngs.event.state_tuple(),
        );
        states.insert(
            "relic".to_string(),
            self.persistent_rngs.relic.state_tuple(),
        );
        states.insert(
            "treasure".to_string(),
            self.persistent_rngs.treasure.state_tuple(),
        );
        states.insert(
            "potion".to_string(),
            self.persistent_rngs.potion.state_tuple(),
        );
        states.insert(
            "merchant".to_string(),
            self.persistent_rngs.merchant.state_tuple(),
        );
        states.insert(
            "monsterHp".to_string(),
            self.floor_rngs.monster_hp.state_tuple(),
        );
        states.insert("ai".to_string(), self.floor_rngs.ai.state_tuple());
        states.insert("shuffle".to_string(), self.floor_rngs.shuffle.state_tuple());
        states.insert(
            "cardRandom".to_string(),
            self.floor_rngs.card_random.state_tuple(),
        );
        states.insert("misc".to_string(), self.floor_rngs.misc.state_tuple());
        states.insert("map".to_string(), self.map_rng.state_tuple());
        states.insert("neow".to_string(), self.neow_rng.state_tuple());

        if let Some(combat) = &self.combat_engine {
            states.insert("card".to_string(), combat.card_rng.state_tuple());
            states.insert("monsterHp".to_string(), combat.monster_hp_rng.state_tuple());
            states.insert("potion".to_string(), combat.potion_rng.state_tuple());
            states.insert("ai".to_string(), combat.ai_rng.state_tuple());
            states.insert("shuffle".to_string(), combat.shuffle_rng.state_tuple());
            states.insert(
                "cardRandom".to_string(),
                combat.card_random_rng.state_tuple(),
            );
            states.insert("misc".to_string(), combat.misc_rng.state_tuple());
        }
        states
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

    #[cfg(test)]
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
            RunPhase::Neow => match self.neow_frame {
                Some(NeowFrame::Choice) => self.neow_options.len(),
                Some(NeowFrame::Intro | NeowFrame::Exit) => 1,
                None => 0,
            },
            RunPhase::MapChoice => self.get_map_actions().len(),
            RunPhase::Chest => self.pending_chest.is_some().then_some(2).unwrap_or(0),
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
            RunPhase::Transition => 1,
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
        if matches!(&screen.source, RewardScreenSource::BossRelic) {
            self.enter_boss_treasure_room();
        }
        Self::refresh_reward_screen(&mut screen);
        self.reward_screen = Some(screen);
        self.phase = RunPhase::CardReward;
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_build_combat_reward_screen(&mut self, room_type: RoomType) {
        self.build_combat_reward_screen(room_type, 0);
        self.phase = RunPhase::CardReward;
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_build_boss_reward_screen(&mut self) {
        self.enter_boss_treasure_room();
        self.enter_boss_chest();
        self.step_chest(&GameAction::OpenChest);
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_build_treasure_reward_screen(&mut self) {
        self.build_treasure_reward_screen();
        self.phase = RunPhase::CardReward;
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_relic_pool_lengths(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.relic_pools.common.len(),
            self.relic_pools.uncommon.len(),
            self.relic_pools.rare.len(),
            self.relic_pools.shop.len(),
            self.relic_pools.boss.len(),
        )
    }

    #[cfg(test)]
    pub(crate) fn debug_relic_pool_contains(&self, relic_id: &str) -> bool {
        [
            &self.relic_pools.common,
            &self.relic_pools.uncommon,
            &self.relic_pools.rare,
            &self.relic_pools.shop,
            &self.relic_pools.boss,
        ]
        .into_iter()
        .any(|pool| pool.iter().any(|candidate| candidate == relic_id))
    }

    #[cfg(test)]
    pub(crate) fn debug_card_pool_lengths(&self) -> (usize, usize, usize) {
        (
            self.card_pools.common.len(),
            self.card_pools.uncommon.len(),
            self.card_pools.rare.len(),
        )
    }

    #[cfg(test)]
    pub(crate) fn debug_card_pool_contains(&self, card_id: &str) -> bool {
        [
            &self.card_pools.common,
            &self.card_pools.uncommon,
            &self.card_pools.rare,
        ]
        .into_iter()
        .any(|pool| pool.iter().any(|candidate| candidate == card_id))
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
    pub(crate) fn debug_enter_current_boss_combat(&mut self) {
        self.reset_floor_rngs();
        self.enter_combat(false, true);
    }

    #[cfg(test)]
    pub(crate) fn debug_enter_boss_room(&mut self) {
        self.enter_combat(false, true);
    }

    #[cfg(test)]
    pub(crate) fn debug_roll_boss_sequence_for_act(&mut self, act: i32) {
        self.roll_boss_sequence_for_act(act);
    }

    #[cfg(test)]
    pub(crate) fn debug_boss_sequence(&self) -> Vec<String> {
        self.boss_sequence.iter().cloned().collect()
    }

    #[cfg(test)]
    pub(crate) fn debug_set_persistent_monster_rng(&mut self, rng: crate::seed::StsRandom) {
        self.persistent_rngs.monster = rng;
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
    pub(crate) fn debug_clear_event_state(&mut self) {
        self.current_event = None;
        self.match_and_keep_state = None;
        self.scrap_ooze_state = None;
        self.spire_heart_state = None;
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
                    node.room_type != RoomType::None && !normal_coords.contains(&(node.x, node.y))
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
        self.match_and_keep_state
            .as_ref()
            .map(|state| state.board.clone())
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
    pub(crate) fn debug_set_floor_rng_counters(&mut self, counters: [i32; 5]) {
        let seed = self.seed.wrapping_add(self.run_state.floor as u64);
        self.floor_rngs.monster_hp = crate::seed::StsRandom::with_counter(seed, counters[0]);
        self.floor_rngs.ai = crate::seed::StsRandom::with_counter(seed, counters[1]);
        self.floor_rngs.shuffle = crate::seed::StsRandom::with_counter(seed, counters[2]);
        self.floor_rngs.card_random = crate::seed::StsRandom::with_counter(seed, counters[3]);
        self.floor_rngs.misc = crate::seed::StsRandom::with_counter(seed, counters[4]);
    }

    #[cfg(test)]
    pub(crate) fn debug_advance_persistent_card_rng_once(&mut self) {
        self.persistent_rngs.card.random_bool();
    }

    #[cfg(test)]
    pub(crate) fn debug_start_final_act(&mut self) {
        self.start_final_act();
        self.refresh_decision_stack();
    }

    #[cfg(test)]
    pub(crate) fn debug_peek_map_rng_int3(&self) -> i32 {
        let mut copy = self.map_rng.clone();
        copy.random_int(3)
    }

    #[cfg(test)]
    pub(crate) fn debug_set_active_emerald_elite(&mut self, active: bool) {
        self.active_combat_has_emerald_key = active;
    }

    #[cfg(test)]
    pub(crate) fn debug_floor_rng_states(&self) -> [(u64, u64, i32); 5] {
        let rngs = self
            .combat_engine
            .as_ref()
            .map(CombatEngine::rng_snapshot)
            .unwrap_or_else(|| self.floor_rngs.combat_snapshot(&self.persistent_rngs));
        [
            rngs.monster_hp.state_tuple(),
            rngs.ai.state_tuple(),
            rngs.shuffle.state_tuple(),
            rngs.card_random.state_tuple(),
            rngs.misc.state_tuple(),
        ]
    }

    #[cfg(test)]
    pub(crate) fn debug_resolve_current_combat_outcome(&mut self) {
        self.step_combat(&GameAction::CombatAction(crate::actions::Action::EndTurn))
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

fn generated_card_definition_blocks() -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = None::<String>;
    for line in include_str!("../content/generated-cards.txt").lines() {
        if current.is_none() && line.contains("= Card(") && !line.trim_start().starts_with('#') {
            current = Some(format!("{line}\n"));
            continue;
        }
        let Some(block) = current.as_mut() else {
            continue;
        };
        block.push_str(line);
        block.push('\n');
        if line.trim() == ")" {
            blocks.push(current.take().expect("active card definition"));
        }
    }
    blocks
}

fn parse_event_card_catalog() -> Vec<EventCardCatalogEntry> {
    generated_card_definition_blocks()
        .into_iter()
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

fn runtime_card_id_for_java_id(java_id: &str) -> &str {
    match java_id {
        "Cloak And Dagger" => "Cloak and Dagger",
        "PiercingWail" => "Piercing Wail",
        "Underhanded Strike" => "Sneaky Strike",
        "Crippling Poison" => "Crippling Cloud",
        "Venomology" => "Alchemize",
        _ => java_id,
    }
}

fn prismatic_reward_candidates(rarity: EventCardRarity) -> Vec<String> {
    let registry = crate::cards::global_registry();
    let mut candidates = parse_event_card_catalog()
        .into_iter()
        .filter(|entry| {
            let runtime_id = runtime_card_id_for_java_id(&entry.id);
            let def = registry.get(runtime_id);
            entry.rarity == rarity
                && matches!(
                    entry.color,
                    EventCardColor::Red
                        | EventCardColor::Green
                        | EventCardColor::Blue
                        | EventCardColor::Purple
                        | EventCardColor::Colorless
                )
                && !entry.id.ends_with('+')
                && def.is_some_and(|card| {
                    !matches!(
                        card.card_type,
                        crate::cards::CardType::Status | crate::cards::CardType::Curse
                    )
                })
        })
        .map(|entry| entry.id)
        .collect::<Vec<_>>();
    candidates.sort();
    candidates
}

fn build_event_card_rarity_map() -> HashMap<String, EventCardRarity> {
    let mut map = HashMap::new();
    for entry in parse_event_card_catalog() {
        map.insert(entry.id, entry.rarity);
        if let Some(card_name) = entry.name {
            map.insert(card_name, entry.rarity);
        }
    }
    for card_id in canonical_starter_ids() {
        map.insert(card_id.to_string(), EventCardRarity::Basic);
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
    for (suffix, color) in [
        ("R", EventCardColor::Red),
        ("G", EventCardColor::Green),
        ("B", EventCardColor::Blue),
        ("P", EventCardColor::Purple),
    ] {
        map.insert(format!("Strike_{suffix}"), color);
        map.insert(format!("Defend_{suffix}"), color);
    }
    map
}

fn canonical_starter_ids() -> impl Iterator<Item = &'static str> {
    [
        "Strike_R", "Strike_G", "Strike_B", "Strike_P", "Defend_R", "Defend_G", "Defend_B",
        "Defend_P",
    ]
    .into_iter()
}

fn is_starter_strike_or_defend(card_id: &str) -> bool {
    let base = card_id.trim_end_matches('+');
    matches!(base, "Strike" | "Defend") || canonical_starter_ids().any(|id| id == base)
}

fn is_starter_strike(card_id: &str) -> bool {
    let base = card_id.trim_end_matches('+');
    matches!(
        base,
        "Strike" | "Strike_R" | "Strike_G" | "Strike_B" | "Strike_P"
    )
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

    fn card_choice_ids(choices: &[RewardChoice]) -> Vec<&str> {
        choices
            .iter()
            .map(|choice| match choice {
                RewardChoice::Card { card_id, .. } => card_id.as_str(),
                _ => panic!("expected only card reward choices"),
            })
            .collect()
    }

    #[test]
    fn card_reward_sequences_match_java_working_pools_and_blizzard_state() {
        // AbstractDungeon.getRewardCards rolls rarity and selection for every
        // card before the natural-upgrade pass. Common results decrement the
        // persistent Blizzard offset; uncommon leaves it unchanged.
        // Re-verified: Act 1's 0% upgrade chance still consumes randomBoolean
        // once for every non-rare preview.
        // Java: AbstractDungeon.java:1413-1468, AbstractRoom.java:148-185.
        for (seed, context, expected, expected_counter, expected_blizzard) in [
            (
                42,
                CardRewardContext::Standard,
                vec!["Swivel", "JustLucky", "CutThroughFate"],
                9,
                3,
            ),
            (
                73,
                CardRewardContext::Standard,
                vec!["CarveReality", "SashWhip", "SandsOfTime"],
                9,
                4,
            ),
            (
                73,
                CardRewardContext::Elite,
                vec!["DevaForm", "SashWhip", "SandsOfTime"],
                8,
                4,
            ),
        ] {
            let mut engine = RunEngine::new(seed, 0);
            let choices = engine.generate_card_reward_choices(3, context);
            assert_eq!(card_choice_ids(&choices), expected, "seed {seed}");
            assert_eq!(
                engine.persistent_rngs.card.counter, expected_counter,
                "seed {seed}"
            );
            assert_eq!(
                engine.run_state.card_blizz_randomizer, expected_blizzard,
                "seed {seed}"
            );
        }
    }

    #[test]
    fn watcher_working_and_source_card_pools_match_java_order() {
        // CardLibrary.addPurpleCards iterates the registered HashMap order into
        // the working pools. AbstractDungeon then copies each working pool into
        // its source pool through CardGroup.addToBottom, which inserts at zero.
        // Java: CardLibrary.java::addPurpleCards,
        // AbstractDungeon.java::initializeCardPools, CardGroup.java::addToBottom.
        let pools = CardPools::watcher_with_locked_cards(&[]);

        assert_eq!(pools.common.len(), 19);
        assert_eq!(pools.uncommon.len(), 35);
        assert_eq!(pools.rare.len(), 17);
        assert_eq!(
            pools.source_common,
            pools.common.iter().rev().cloned().collect::<Vec<_>>()
        );
        assert_eq!(
            pools.source_uncommon,
            pools.uncommon.iter().rev().cloned().collect::<Vec<_>>()
        );
        assert_eq!(
            pools.source_rare,
            pools.rare.iter().rev().cloned().collect::<Vec<_>>()
        );
        assert_eq!(pools.common.first().map(String::as_str), Some("Consecrate"));
        assert_eq!(pools.common.last().map(String::as_str), Some("EmptyFist"));
        assert_eq!(
            pools.uncommon.first().map(String::as_str),
            Some("WheelKick")
        );
        assert_eq!(pools.uncommon.last().map(String::as_str), Some("Pray"));
        assert_eq!(
            pools.rare.first().map(String::as_str),
            Some("DeusExMachina")
        );
        assert_eq!(pools.rare.last().map(String::as_str), Some("Judgement"));
    }

    #[test]
    fn watcher_relic_pools_shuffle_once_and_persist() {
        // RelicLibrary supplies 33/30/27/17/21 all-unlocked Watcher entries,
        // then initializeRelicList consumes exactly five randomLong calls in
        // common/uncommon/rare/shop/boss order.
        // Java: RelicLibrary.java::populateRelicPool,
        // AbstractDungeon.java::initializeRelicList.
        assert_eq!(WATCHER_COMMON_RELICS.len(), 33);
        assert_eq!(WATCHER_UNCOMMON_RELICS.len(), 30);
        assert_eq!(WATCHER_RARE_RELICS.len(), 27);
        assert_eq!(WATCHER_SHOP_RELICS.len(), 17);
        assert_eq!(WATCHER_BOSS_RELICS.len(), 21);

        let first = RunEngine::new(42, 0);
        let second = RunEngine::new(42, 0);
        assert_eq!(first.persistent_rngs.relic.counter, 5);
        assert_eq!(first.relic_pools, second.relic_pools);
        assert_eq!(first.relic_pools.common.len(), 33);
        assert_eq!(first.relic_pools.uncommon.len(), 30);
        assert_eq!(first.relic_pools.rare.len(), 27);
        assert_eq!(first.relic_pools.shop.len(), 17);
        assert_eq!(first.relic_pools.boss.len(), 21);
    }

    #[test]
    fn relic_retry_discards_failed_front_then_draws_from_the_back() {
        // A failed canSpawn item is permanently removed. Both Java draw entry
        // points then retry through returnEndRandomRelicKey, so this injected
        // [bad, A, C] pool returns C and leaves only A without an RNG draw.
        // Java: AbstractDungeon.java::{returnRandomRelicKey,returnEndRandomRelicKey}.
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.floor = 36;
        engine.relic_pools.common = vec![
            "Tiny Chest".to_string(),
            "Anchor".to_string(),
            "Vajra".to_string(),
        ];
        let counter_before = engine.persistent_rngs.relic.counter;

        let relic = engine.draw_relic_from_pool(RelicTier::Common, false, false);

        assert_eq!(relic, "Vajra");
        assert_eq!(engine.relic_pools.common, vec!["Anchor"]);
        assert_eq!(engine.persistent_rngs.relic.counter, counter_before);
    }

    #[test]
    fn card_reward_duplicate_retry_and_nloths_gift_match_java() {
        // Duplicate reward IDs retry only the pool-selection draw. N'loth's
        // Gift triples the room's base rare integer (3 -> 9 in a normal room),
        // not a global floating-point probability.
        // Re-verified: the rejected seed-3 duplicate consumes an extra cardRng
        // selection tick but no extra rarity or natural-upgrade tick.
        // Java: AbstractDungeon.java:1424-1468, NlothsGift.java:22-24.
        let mut duplicate = RunEngine::new(3, 0);
        let choices = duplicate.generate_card_reward_choices(3, CardRewardContext::Standard);
        assert_eq!(
            card_choice_ids(&choices),
            vec!["FlurryOfBlows", "PathToVictory", "Protect"],
        );
        assert_eq!(duplicate.persistent_rngs.card.counter, 10);

        let mut gift = RunEngine::new(73, 0);
        gift.run_state.relics.push("Nloth's Gift".to_string());
        gift.run_state.relic_flags.rebuild(&gift.run_state.relics);
        let choices = gift.generate_card_reward_choices(3, CardRewardContext::Standard);
        assert_eq!(
            card_choice_ids(&choices),
            vec!["DevaForm", "SashWhip", "SandsOfTime"],
        );
        assert_eq!(gift.persistent_rngs.card.counter, 8);
        assert_eq!(gift.run_state.card_blizz_randomizer, 4);
    }

    #[test]
    fn act_two_natural_upgrade_and_prismatic_paths_match_java() {
        // Act 2 A0 uses a 25% natural upgrade chance. Prismatic Shard includes
        // colorless cards and consumes randomLong for a Java shuffle before
        // the sorted-card-ID selection draw on every attempt.
        // Re-verified: rare previews skip randomBoolean entirely, while a
        // non-rare 0%/25% preview always consumes it before relic callbacks.
        // Java: TheCity.java:72-85, CardLibrary.java:1064-1082,
        // CardGroup.java:491-505,554-556.
        let mut act_two = RunEngine::new(0, 0);
        act_two.run_state.act = 2;
        let choices = act_two.generate_card_reward_choices(3, CardRewardContext::Standard);
        assert_eq!(
            card_choice_ids(&choices),
            vec!["FlyingSleeves", "JustLucky", "SignatureMove+"],
        );
        assert_eq!(act_two.persistent_rngs.card.counter, 9);

        let mut prismatic = RunEngine::new(42, 0);
        prismatic
            .run_state
            .relics
            .push("PrismaticShard".to_string());
        prismatic
            .run_state
            .relic_flags
            .rebuild(&prismatic.run_state.relics);
        let choices = prismatic.generate_card_reward_choices(3, CardRewardContext::Standard);
        assert_eq!(
            card_choice_ids(&choices),
            vec!["Masterful Stab", "Evaluate", "Calculated Gamble"],
        );
        assert_eq!(prismatic.persistent_rngs.card.counter, 12);
        assert_eq!(prismatic.run_state.card_blizz_randomizer, 4);
    }

    #[test]
    fn prismatic_all_unlocked_candidate_membership_matches_java() {
        // CardLibrary.getAnyColorCard includes every registered non-status,
        // non-curse card of the rolled rarity and sorts by Java cardID. Keep
        // these counts and alias-sensitive members explicit because bounded
        // selection changes when even one candidate is missing.
        // Java: CardLibrary.java::getAnyColorCard,
        // CardGroup.java::getRandomCard(boolean, rarity).
        let common = prismatic_reward_candidates(EventCardRarity::Common);
        let uncommon = prismatic_reward_candidates(EventCardRarity::Uncommon);
        let rare = prismatic_reward_candidates(EventCardRarity::Rare);
        assert_eq!(common.len(), 76);
        assert_eq!(uncommon.len(), 160);
        assert_eq!(rare.len(), 84);
        for id in ["Cloak And Dagger", "PiercingWail", "Underhanded Strike"] {
            assert!(common.iter().any(|candidate| candidate == id), "{id}");
        }
        for id in ["Capacitor", "Crippling Poison", "Fasting2"] {
            assert!(uncommon.iter().any(|candidate| candidate == id), "{id}");
        }
        for id in ["Vault", "Venomology"] {
            assert!(rare.iter().any(|candidate| candidate == id), "{id}");
        }
    }

    #[test]
    fn rare_reward_resets_blizzard_offset_for_later_screens() {
        // A rare resets cardBlizzRandomizer to 5 after rarity resolution, and
        // the state persists between reward screens.
        // Re-verified: the reset happens before card selection, not after the
        // later natural-upgrade pass.
        // Java: AbstractDungeon.java:1424-1440.
        let mut engine = RunEngine::new(42, 0);
        let mut sixth = Vec::new();
        for reward_index in 0..6 {
            let choices = engine.generate_card_reward_choices(3, CardRewardContext::Standard);
            if reward_index == 5 {
                sixth = card_choice_ids(&choices)
                    .into_iter()
                    .map(str::to_string)
                    .collect();
            }
        }
        assert_eq!(sixth, vec!["JustLucky", "Vault", "Adaptation"]);
        assert_eq!(engine.persistent_rngs.card.counter, 53);
        assert_eq!(engine.run_state.card_blizz_randomizer, 5);
    }

    #[test]
    fn dream_catcher_disables_nloths_gift_rarity_alternation() {
        // RestRoom.getCardRarity calls the non-alternating overload. The
        // Blizzard-adjusted roll still applies, but N'loth's Gift cannot turn
        // this roll of eight from uncommon into rare.
        // Java: RestRoom.java::getCardRarity,
        // AbstractRoom.java::getCardRarity(int, boolean).
        let card_seed = (0_u64..10_000)
            .find(|seed| crate::seed::StsRandom::new(*seed).random_int(99) == 3)
            .expect("fixture seed for Blizzard-adjusted roll eight");

        let mut combat_reward = RunEngine::new(42, 0);
        combat_reward.persistent_rngs.card = crate::seed::StsRandom::new(card_seed);
        combat_reward
            .run_state
            .relics
            .push("Nloth's Gift".to_string());
        let standard = combat_reward.generate_card_reward_choices(1, CardRewardContext::Standard);
        let standard_id = card_choice_ids(&standard).remove(0);
        assert_eq!(event_card_rarity(&standard_id), Some(EventCardRarity::Rare));

        let mut rest_reward = RunEngine::new(42, 0);
        rest_reward.persistent_rngs.card = crate::seed::StsRandom::new(card_seed);
        rest_reward
            .run_state
            .relics
            .push("Nloth's Gift".to_string());
        let rest = rest_reward.generate_card_reward_choices(1, CardRewardContext::Rest);
        let rest_id = card_choice_ids(&rest).remove(0);
        assert_eq!(event_card_rarity(&rest_id), Some(EventCardRarity::Uncommon));
    }

    #[test]
    fn potion_pity_moves_by_ten_and_consumes_every_chance_roll() {
        // A roll equal to the current chance fails (`<`, not `<=`), raising
        // the next chance by ten. A later success lowers it by ten again.
        // Java: AbstractRoom.java::addPotionToRewards.
        let mut engine = RunEngine::new(42, 0);
        engine.persistent_rngs.potion = crate::seed::StsRandom::new(18_448);

        assert!(!engine.should_offer_potion_reward(RoomType::Monster, 0));
        assert_eq!(engine.run_state.potion_blizz_randomizer, 10);
        assert!(engine.should_offer_potion_reward(RoomType::Monster, 0));
        assert_eq!(engine.run_state.potion_blizz_randomizer, 0);
        assert_eq!(engine.persistent_rngs.potion.counter, 2);
    }

    #[test]
    fn potion_rarity_rejection_uses_the_full_watcher_pool() {
        // Rarity 64 is Common. Candidate index one is Stance Potion
        // (Uncommon), so Java rejects it and consumes another full-pool draw;
        // index zero then accepts Bottled Miracle.
        // Java: AbstractDungeon.java::returnRandomPotion,
        // PotionHelper.java::getPotions.
        let mut engine = RunEngine::new(42, 0);
        engine.persistent_rngs.potion = crate::seed::StsRandom::new(52_510);

        assert_eq!(engine.roll_reward_potion_id(), "BottledMiracle");
        assert_eq!(engine.persistent_rngs.potion.counter, 3);
    }

    fn resolve_opening_neow(engine: &mut RunEngine) {
        if engine.current_phase() == RunPhase::Neow {
            let intro = engine.step_game(&GameAction::Proceed);
            assert!(intro.accepted());
            assert!(!intro.is_terminal());

            let outcome = engine.step_game(&GameAction::ChooseNeowOption(1));
            assert!(outcome.accepted());
            assert!(!outcome.is_terminal());
            while engine.current_phase() == RunPhase::CardReward {
                let actions = engine.get_legal_actions();
                let action = actions
                    .iter()
                    .find(|action| matches!(action, GameAction::SkipRewardItem(_)))
                    .or_else(|| {
                        actions
                            .iter()
                            .find(|action| matches!(action, GameAction::SelectRewardItem(_)))
                    })
                    .or_else(|| {
                        actions
                            .iter()
                            .find(|action| matches!(action, GameAction::ChooseRewardOption { .. }))
                    })
                    .or_else(|| {
                        actions
                            .iter()
                            .find(|action| matches!(action, GameAction::LeaveRewards))
                    })
                    .cloned()
                    .expect("Neow follow-up must expose a reward action");
                engine.step_game(&action);
            }
            assert_eq!(engine.current_phase(), RunPhase::Neow);
            let exit = engine.step_game(&GameAction::Proceed);
            assert!(exit.accepted());
            assert!(!exit.is_terminal());
            assert_eq!(engine.current_phase(), RunPhase::MapChoice);
        }
    }

    #[test]
    fn test_run_engine_creation() {
        let engine = RunEngine::new(42, 20);
        assert_eq!(engine.run_state.current_hp, 68); // A14+ = 68
        assert_eq!(engine.run_state.deck.len(), 11); // 10 base + AscendersBane (A10+)
        assert_eq!(engine.phase, RunPhase::Neow);
        assert_eq!(engine.current_decision_kind(), DecisionKind::Proceed);
        assert_eq!(engine.current_choice_count(), 1);
        assert_eq!(engine.get_legal_actions(), vec![GameAction::Proceed]);
        assert!(!engine.is_done());
    }

    #[test]
    fn neow_builds_four_java_categories_on_its_dedicated_rng() {
        // Four NeowReward constructors consume one category draw each, while
        // category 2 consumes its drawback draw first: five total calls.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/neow/NeowEvent.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/neow/NeowReward.java
        for seed in [0, 4, 42, 57_554_006_466] {
            let mut engine = RunEngine::new(seed, 0);
            assert!(engine.neow_options.is_empty());
            assert_eq!(engine.neow_rng.counter, 0);
            assert!(engine.step_game(&GameAction::Proceed).accepted());
            assert_eq!(engine.neow_options.len(), 4);
            assert_eq!(engine.neow_rng.counter, 5);
            assert_eq!(engine.rng_counters()["card"], 0);
        }
    }

    #[test]
    fn neow_dialog_frames_are_checkpoint_safe_and_reject_out_of_phase_actions() {
        // NeowEvent commits the intro, reward selection, and screen-99 exit as
        // three separate inputs. Only the middle commit constructs rewards.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/neow/NeowEvent.java:160-219,236-238
        let mut engine = RunEngine::new(4, 0);

        let intro_checkpoint = crate::checkpoint::CoreCheckpoint::capture(&engine).unwrap();
        let intro_rng = engine.neow_rng.clone();
        assert!(!engine
            .step_game(&GameAction::ChooseNeowOption(1))
            .accepted());
        assert_eq!(engine.neow_rng, intro_rng);
        assert_eq!(
            crate::checkpoint::CoreCheckpoint::capture(&engine).unwrap(),
            intro_checkpoint
        );
        let intro_restored = intro_checkpoint.restore().unwrap();
        assert_eq!(
            intro_restored.get_legal_actions(),
            vec![GameAction::Proceed]
        );

        assert!(engine.step_game(&GameAction::Proceed).accepted());
        assert_eq!(engine.current_decision_kind(), DecisionKind::NeowChoice);
        assert_eq!(engine.get_legal_actions().len(), 4);
        let choice_checkpoint = crate::checkpoint::CoreCheckpoint::capture(&engine).unwrap();
        let choice_rng = engine.neow_rng.clone();
        assert!(!engine.step_game(&GameAction::Proceed).accepted());
        assert_eq!(engine.neow_rng, choice_rng);
        assert_eq!(
            crate::checkpoint::CoreCheckpoint::capture(&engine).unwrap(),
            choice_checkpoint
        );
        assert_eq!(
            choice_checkpoint.restore().unwrap().get_legal_actions(),
            engine.get_legal_actions()
        );

        assert!(engine
            .step_game(&GameAction::ChooseNeowOption(1))
            .accepted());
        assert_eq!(engine.current_phase(), RunPhase::Neow);
        assert_eq!(engine.current_decision_kind(), DecisionKind::Proceed);
        assert_eq!(engine.get_legal_actions(), vec![GameAction::Proceed]);
        let exit_checkpoint = crate::checkpoint::CoreCheckpoint::capture(&engine).unwrap();
        let exit_rng = engine.neow_rng.clone();
        assert!(!engine
            .step_game(&GameAction::ChooseNeowOption(1))
            .accepted());
        assert_eq!(engine.neow_rng, exit_rng);
        assert_eq!(
            crate::checkpoint::CoreCheckpoint::capture(&engine).unwrap(),
            exit_checkpoint
        );
        assert_eq!(
            exit_checkpoint.restore().unwrap().get_legal_actions(),
            vec![GameAction::Proceed]
        );

        assert!(engine.step_game(&GameAction::Proceed).accepted());
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    }

    #[test]
    fn secret_portal_uses_normal_floor_and_room_rng_transitions() {
        // Re-verified against SecretPortal.java and
        // AbstractDungeon.nextRoomTransition: redirecting the node does not
        // rewrite floorNum to the canonical boss floor.
        let mut engine = RunEngine::new(73, 0);
        engine.run_state.act = 3;
        engine.run_state.floor = 42;
        let portal = typed_events_for_act(3)
            .into_iter()
            .find(|event| event.name == "Secret Portal")
            .expect("Secret Portal must be registered");
        engine.debug_set_typed_event_state(portal);

        assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
        assert_eq!(engine.phase, RunPhase::Combat);
        assert_eq!(engine.run_state.floor, 43);

        engine.debug_force_current_combat_outcome(true);
        engine.debug_resolve_current_combat_outcome();
        assert_eq!(engine.phase, RunPhase::Transition);
        assert_eq!(engine.run_state.floor, 43);
        assert!(engine.step_game(&GameAction::Proceed).accepted());
        assert_eq!(engine.phase, RunPhase::Event);
        assert_eq!(engine.run_state.floor, 44);

        let room_state = crate::seed::StsRandom::new(117).state_tuple();
        assert_eq!(engine.floor_rngs.monster_hp.state_tuple(), room_state);
        assert_eq!(engine.floor_rngs.ai.state_tuple(), room_state);
        assert_eq!(engine.floor_rngs.shuffle.state_tuple(), room_state);
        assert_eq!(engine.floor_rngs.card_random.state_tuple(), room_state);
        assert_eq!(engine.floor_rngs.misc.state_tuple(), room_state);
        assert_eq!(
            engine.persistent_rngs.potion.state_tuple(),
            crate::seed::StsRandom::new(73).state_tuple()
        );
    }

    #[test]
    fn top_row_campfire_returns_to_one_synthetic_boss_map_action() {
        // ProceedButton's boss click installs MapRoomNode(-1, 15); completing
        // the top campfire does not enter combat by itself.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/ui/buttons/ProceedButton.java:180-187
        let mut engine = RunEngine::new(73, 0);
        resolve_opening_neow(&mut engine);
        engine.run_state.map_x = 3;
        engine.run_state.map_y = engine.map.height as i32 - 1;
        engine.run_state.floor = 15;
        engine.phase = RunPhase::Campfire;

        assert!(engine.step_game(&GameAction::CampfireRest).accepted());
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
        assert_eq!(engine.get_legal_actions(), vec![GameAction::ChoosePath(0)]);
        let path = &engine.map_decision_context().paths[0];
        assert_eq!((path.x, path.y), (-1, 15));
        assert_eq!(path.room_type, "Boss");
        assert!(!path.has_emerald_key);

        let checkpoint = crate::checkpoint::CoreCheckpoint::capture(&engine).unwrap();
        let mut restored = checkpoint.restore().unwrap();
        assert!(restored.step_game(&GameAction::ChoosePath(0)).accepted());
        assert_eq!(
            (restored.run_state.map_x, restored.run_state.map_y),
            (-1, 15)
        );
        assert_eq!(restored.run_state.floor, 16);
        assert_eq!(restored.current_phase(), RunPhase::Combat);
    }

    #[test]
    fn dream_catcher_overlay_returns_to_the_same_synthetic_boss_click() {
        // CampfireSleepEffect opens Dream Catcher's CardRewardScreen first;
        // closing that overlay returns to the map, not directly to the boss.
        // Java: CampfireSleepEffect.java and ProceedButton.java.
        let mut engine = RunEngine::new(73, 0);
        resolve_opening_neow(&mut engine);
        engine.run_state.relics.push("Dream Catcher".to_string());
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);
        engine.run_state.map_x = 3;
        engine.run_state.map_y = engine.map.height as i32 - 1;
        engine.phase = RunPhase::Campfire;

        assert!(engine.step_game(&GameAction::CampfireRest).accepted());
        assert_eq!(engine.current_phase(), RunPhase::CardReward);
        assert!(engine
            .step_game(&GameAction::SelectRewardItem(0))
            .accepted());
        assert!(engine.step_game(&GameAction::SkipRewardItem(0)).accepted());
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
        assert_eq!(engine.get_legal_actions(), vec![GameAction::ChoosePath(0)]);
        let path = &engine.map_decision_context().paths[0];
        assert_eq!((path.x, path.y, path.room_type.as_str()), (-1, 15, "Boss"));
    }

    #[test]
    fn noncombat_room_entry_resets_every_floor_rng() {
        // Re-verified against AbstractDungeon.nextRoomTransition: reset happens
        // for every room type, not only when a combat engine is constructed.
        let mut engine = RunEngine::new(73, 0);
        resolve_opening_neow(&mut engine);
        let (x, y, _, _) = engine.available_map_destinations()[0];
        engine.map.rows[y as usize][x as usize].room_type = RoomType::Rest;

        engine.floor_rngs.monster_hp.random_int(9);
        engine.floor_rngs.ai.random_int(9);
        engine.floor_rngs.shuffle.random_int(9);
        engine.floor_rngs.card_random.random_int(9);
        engine.floor_rngs.misc.random_int(9);

        assert!(engine.step_game(&GameAction::ChoosePath(0)).accepted());
        assert_eq!(engine.phase, RunPhase::Campfire);
        assert_eq!(engine.run_state.floor, 1);

        let expected = crate::seed::StsRandom::new(74).state_tuple();
        assert_eq!(engine.floor_rngs.monster_hp.state_tuple(), expected);
        assert_eq!(engine.floor_rngs.ai.state_tuple(), expected);
        assert_eq!(engine.floor_rngs.shuffle.state_tuple(), expected);
        assert_eq!(engine.floor_rngs.card_random.state_tuple(), expected);
        assert_eq!(engine.floor_rngs.misc.state_tuple(), expected);
    }

    #[test]
    fn combat_absorb_returns_persistent_card_rng_without_touching_shuffle_rng() {
        // ForeignInfluenceAction consumes cardRandomRng twice and persistent
        // cardRng once per candidate attempt. CardGroup's library shuffle is
        // seeded from cardRandomRng, so floor-local shuffleRng stays untouched.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/ForeignInfluenceAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/CardLibrary.java:1054-1061
        let mut engine = RunEngine::new(73, 0);
        engine.debug_enter_specific_combat(&["JawWorm"]);
        let combat = engine.combat_engine.as_mut().expect("combat");
        combat.state.hand = vec![combat.card_registry.make_card("ForeignInfluence")];
        combat.state.energy = 3;

        let card_before = engine.persistent_rngs.card.counter;
        let card_random_before = combat.card_random_rng.counter;
        let shuffle_before = combat.shuffle_rng.state_tuple();
        assert!(engine
            .step_game(&GameAction::CombatAction(
                crate::actions::Action::PlayCard {
                    card_idx: 0,
                    target_idx: -1,
                }
            ))
            .accepted());

        let attempts = engine.persistent_rngs.card.counter - card_before;
        assert!(attempts >= 3);
        assert_eq!(
            engine.floor_rngs.card_random.counter - card_random_before,
            attempts * 2,
        );
        assert_eq!(engine.floor_rngs.shuffle.state_tuple(), shuffle_before);
        let combat = engine
            .combat_engine
            .as_ref()
            .expect("combat remains active");
        assert_eq!(
            combat.card_rng.state_tuple(),
            engine.persistent_rngs.card.state_tuple()
        );
    }

    #[test]
    fn rng_counter_exports_preserve_java_signed_int_overflow() {
        let mut engine = RunEngine::new(42, 0);
        engine.persistent_rngs.card.counter = i32::MAX;
        engine.persistent_rngs.card.random_int(0);
        assert_eq!(engine.rng_counters()["card"], i64::from(i32::MIN));

        engine.debug_enter_specific_combat(&["JawWorm"]);
        engine.combat_engine.as_mut().unwrap().ai_rng.counter = i32::MIN;
        assert_eq!(engine.rng_counters()["ai"], i64::from(i32::MIN));
    }

    #[test]
    fn mystery_room_type_roll_commits_only_the_event_stream() {
        // EventHelper.roll(Random) consumes one unit float. The duplicate is
        // committed to AbstractDungeon.eventRng after room-type generation.
        // EventRoom then selects through another seed+counter duplicate and
        // deliberately leaves the committed stream unchanged.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/
        // AbstractDungeon.java:1755 and rooms/EventRoom.java:26.
        let mut engine = RunEngine::new(42, 0);
        let misc_before = engine.floor_rngs.misc.state_tuple();
        let expected_events = ["Big Fish", "Lab"];

        for prior_counter in 0..2 {
            let mut room_duplicate = crate::seed::StsRandom::with_counter(42, prior_counter);
            let expected_roll = (room_duplicate.random_f32() * 100.0) as usize;
            assert_eq!(engine.next_event_roll_100(), expected_roll);
            assert_eq!(engine.persistent_rngs.event, room_duplicate);

            let committed = engine.persistent_rngs.event.clone();
            engine.enter_event();
            assert_eq!(
                engine.current_event.as_ref().expect("event selected").name,
                expected_events[prior_counter as usize],
            );
            assert_eq!(
                engine.persistent_rngs.event, committed,
                "EventRoom selection must discard its duplicate RNG",
            );
        }

        assert_eq!(engine.floor_rngs.misc.state_tuple(), misc_before);
    }

    #[test]
    fn watcher_a0_event_pools_match_java_order_and_lifecycle() {
        // Re-verified: each dungeon constructor clears/rebuilds only eventList
        // and shrineList; specialOneTimeEventList is passed to the next act.
        // Java: AbstractDungeon.java:2552-2568, CardCrawlGame.java:1121-1127,
        // and each dungeon's initializeEventList/initializeShrineList.
        let mut pools = EventPools::watcher(0, 20, false);
        assert_eq!(pools.regular, ACT_ONE_EVENTS);
        assert_eq!(pools.shrines, ACT_ONE_SHRINES);
        assert_eq!(pools.one_time_shrines, ONE_TIME_SHRINES);

        pools.one_time_shrines.retain(|event| event != "Lab");
        pools.reset_for_act(2);
        assert_eq!(pools.regular, ACT_TWO_EVENTS);
        assert_eq!(pools.shrines, LATER_ACT_SHRINES);
        assert!(!pools.one_time_shrines.iter().any(|event| event == "Lab"));

        let daily = EventPools::watcher(0, 20, true);
        assert!(!daily
            .one_time_shrines
            .iter()
            .any(|event| event == "NoteForYourself"));

        pools.reset_for_act(3);
        assert_eq!(pools.regular, ACT_THREE_EVENTS);
        assert_eq!(pools.shrines, LATER_ACT_SHRINES);
        assert!(!pools.one_time_shrines.iter().any(|event| event == "Lab"));
    }

    #[test]
    fn event_selection_removes_only_the_selected_java_pool_key() {
        // Seed 42's discarded EventRoom duplicate selects Big Fish, then Lab
        // after the next room-type tick. Lab is one-time; Big Fish is regular.
        // Re-verified: getEvent/getShrine remove the selected key from its
        // owning persistent pool, while ineligible keys are not removed.
        // Java: AbstractDungeon.java:1854-1978.
        let mut engine = RunEngine::new(42, 0);

        engine.next_event_roll_100();
        engine.enter_event();
        assert_eq!(engine.current_event.as_ref().unwrap().name, "Big Fish");
        assert!(!engine
            .event_pools
            .regular
            .iter()
            .any(|event| event == "Big Fish"));
        assert_eq!(
            engine.event_pools.one_time_shrines.len(),
            ONE_TIME_SHRINES.len()
        );

        engine.next_event_roll_100();
        engine.enter_event();
        assert_eq!(engine.current_event.as_ref().unwrap().name, "Lab");
        assert!(!engine
            .event_pools
            .one_time_shrines
            .iter()
            .any(|event| event == "Lab"));
        assert!(engine
            .event_pools
            .shrines
            .iter()
            .any(|event| event == "Golden Shrine"));
    }

    #[test]
    fn java_event_eligibility_uses_run_state_without_removing_failed_candidates() {
        // Re-verified: getEvent/getShrine build an eligible temporary list and
        // remove only the selected key from the source pool.
        // Java: AbstractDungeon.java:1871-1978.
        let mut engine = RunEngine::new(7, 0);
        assert!(!engine.regular_event_is_eligible("Dead Adventurer"));
        assert!(!engine.one_time_shrine_is_eligible("Fountain of Cleansing"));
        assert!(!engine.one_time_shrine_is_eligible("SecretPortal"));

        engine.run_state.floor = 7;
        engine.run_state.deck.push("Regret".to_string());
        engine.run_state.act = 3;
        engine.run_state.playtime_seconds = 800.0;
        assert!(engine.regular_event_is_eligible("Dead Adventurer"));
        assert!(engine.one_time_shrine_is_eligible("Fountain of Cleansing"));
        assert!(engine.one_time_shrine_is_eligible("SecretPortal"));
        assert!(engine
            .event_pools
            .one_time_shrines
            .iter()
            .any(|event| event == "Fountain of Cleansing"));
    }

    #[test]
    fn fountain_ignores_java_special_unremovable_curses() {
        // AbstractPlayer.isCursed excludes these three Curse-typed cards from
        // Fountain eligibility by ID comparison.
        // Java: AbstractPlayer.java:739-746.
        let mut engine = RunEngine::new(7, 0);
        engine.run_state.deck = vec![
            "Necronomicurse".to_string(),
            "CurseOfTheBell".to_string(),
            "AscendersBane".to_string(),
        ];
        assert!(!engine.deck_contains_curse());
        assert!(!engine.one_time_shrine_is_eligible("Fountain of Cleansing"));

        engine.run_state.deck.push("Regret".to_string());
        assert!(engine.deck_contains_curse());
        assert!(engine.one_time_shrine_is_eligible("Fountain of Cleansing"));
    }

    #[test]
    fn act_transition_uses_java_card_counter_buckets_and_music_tick() {
        // AbstractDungeon.dungeonTransitionSetup advances open card-counter
        // intervals to 250/500/750; MainMusic consumes one misc draw afterward.
        let cases = [
            (0, 0),
            (1, 250),
            (249, 250),
            (250, 250),
            (251, 500),
            (499, 500),
            (500, 500),
            (501, 750),
            (749, 750),
            (750, 750),
        ];
        for (start, expected) in cases {
            let mut engine = RunEngine::new(42, 0);
            engine.persistent_rngs.card = crate::seed::StsRandom::with_counter(42, start);
            engine.floor_rngs.misc = crate::seed::StsRandom::new(99);
            engine.transition_to_next_act();
            assert_eq!(
                engine.persistent_rngs.card.counter, expected,
                "start {start}"
            );
            assert_eq!(engine.floor_rngs.misc.counter, 1, "start {start}");
            assert_eq!(engine.run_state.act, 2);
        }
    }

    #[test]
    fn neow_remove_is_an_agent_choice_and_can_be_cancelled() {
        // REMOVE_CARD opens the purgeable master-deck grid with canCancel=true;
        // choosing the Neow option itself must not silently remove a card.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/neow/NeowReward.java:132-143,238-241
        let seed = 57_554_006_466;
        let mut chosen = RunEngine::new(seed, 0);
        let deck_before = chosen.run_state.deck.clone();
        assert!(chosen.step_game(&GameAction::Proceed).accepted());
        chosen.step_game(&GameAction::ChooseNeowOption(0));
        assert_eq!(chosen.current_phase(), RunPhase::CardReward);
        assert_eq!(chosen.run_state.deck, deck_before);

        chosen.step_game(&GameAction::SelectRewardItem(0));
        chosen.step_game(&GameAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        });
        assert_eq!(chosen.current_phase(), RunPhase::Neow);
        assert_eq!(chosen.run_state.deck.len(), deck_before.len() - 1);
        assert!(chosen.step_game(&GameAction::Proceed).accepted());
        assert_eq!(chosen.current_phase(), RunPhase::MapChoice);

        let mut cancelled = RunEngine::new(seed, 0);
        assert!(cancelled.step_game(&GameAction::Proceed).accepted());
        cancelled.step_game(&GameAction::ChooseNeowOption(0));
        // The screen-level action opens the grid; Skip belongs to that nested
        // cancellable grid, not to the parent reward row.
        cancelled.step_game(&GameAction::SelectRewardItem(0));
        cancelled.step_game(&GameAction::SkipRewardItem(0));
        assert_eq!(cancelled.current_phase(), RunPhase::Neow);
        assert_eq!(cancelled.run_state.deck, deck_before);
        assert!(cancelled.step_game(&GameAction::Proceed).accepted());
        assert_eq!(cancelled.current_phase(), RunPhase::MapChoice);
    }

    #[test]
    fn neow_transform_replaces_the_selected_card_through_the_reward_surface() {
        // TRANSFORM_CARD opens one non-cancellable purgeable-card selection,
        // removes that exact card, and obtains the Neow-RNG-selected result.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/neow/NeowReward.java:144-151,265-268
        let seed = (0..512)
            .find(|seed| {
                let mut engine = RunEngine::new(*seed, 0);
                assert!(engine.step_game(&GameAction::Proceed).accepted());
                engine.neow_options[0].reward == NeowReward::TransformCard
            })
            .expect("category zero must reach TRANSFORM_CARD");
        let mut engine = RunEngine::new(seed, 0);
        let deck_before = engine.run_state.deck.clone();
        assert!(engine.step_game(&GameAction::Proceed).accepted());
        engine.step_game(&GameAction::ChooseNeowOption(0));
        engine.step_game(&GameAction::SelectRewardItem(0));
        engine.step_game(&GameAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        });

        assert_eq!(engine.current_phase(), RunPhase::Neow);
        assert_eq!(engine.run_state.deck.len(), deck_before.len());
        assert_ne!(engine.run_state.deck, deck_before);
        assert!(engine.step_game(&GameAction::Proceed).accepted());
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    }

    #[test]
    fn encounter_queues_match_java_lengths_and_repeat_rejection_rules() {
        // Each act generates its weak prefix, one exclusion-filtered first
        // strong encounter, twelve more strong encounters, and ten elites.
        // Hallways reject the previous two keys; elites reject the previous
        // key. Java: AbstractDungeon.java:1047-1084 and each act's
        // generateMonsters implementation.
        let mut engine = RunEngine::new(0, 0);
        for act in 1..=3 {
            for seed in 0..64 {
                engine.persistent_rngs.monster = crate::seed::StsRandom::new(seed);
                engine.generate_encounter_queues(act);

                let expected_monsters = if act == 1 { 16 } else { 15 };
                assert_eq!(engine.monster_encounter_queue.len(), expected_monsters);
                assert_eq!(engine.elite_encounter_queue.len(), 10);

                let weak_count = if act == 1 { 3 } else { 2 };
                let monsters = engine.monster_encounter_queue.iter().collect::<Vec<_>>();
                for index in 1..monsters.len() {
                    if index == weak_count {
                        // populateFirstStrongEnemy uses only the act-specific
                        // exclusion list, not populateMonsterList's generic
                        // previous/two-back rejection.
                        continue;
                    }
                    assert_ne!(monsters[index], monsters[index - 1]);
                    if index > 1 {
                        assert_ne!(monsters[index], monsters[index - 2]);
                    }
                }

                let elites = engine.elite_encounter_queue.iter().collect::<Vec<_>>();
                for index in 1..elites.len() {
                    assert_ne!(elites[index], elites[index - 1]);
                }

                let previous = monsters[weak_count - 1];
                let first_strong = monsters[weak_count];
                assert!(
                    !first_strong_exclusions(act, previous).contains(&first_strong.as_str()),
                    "act {act} seed {seed}: {first_strong} followed excluded {previous}",
                );
            }
        }
    }

    #[test]
    fn encounter_queue_peeks_until_the_next_accepted_map_transition() {
        // Room creation reads index zero without removal. Java removes that
        // encounter only in nextRoomTransition, after combat and rewards, when
        // the player actually leaves the room.
        // Java: AbstractDungeon.java::getMonsterForRoomCreation,
        // AbstractDungeon.java::nextRoomTransition.
        let mut engine = RunEngine::new(42, 0);
        let expected = engine
            .monster_encounter_queue
            .front()
            .cloned()
            .expect("act one hallway queue");
        let initial_len = engine.monster_encounter_queue.len();

        engine.enter_combat(false, false);
        assert_eq!(
            engine.active_encounter_queue,
            Some(EncounterQueueKind::Hallway)
        );
        assert_eq!(engine.monster_encounter_queue.front(), Some(&expected));
        assert_eq!(engine.monster_encounter_queue.len(), initial_len);

        // Simulate a resolved room. An illegal map action cannot retire the
        // encounter; the first accepted transition does so exactly once.
        engine.combat_engine = None;
        engine.phase = RunPhase::MapChoice;
        engine.step_map(&GameAction::ChoosePath(usize::MAX));
        assert_eq!(engine.monster_encounter_queue.len(), initial_len);
        assert_eq!(
            engine.active_encounter_queue,
            Some(EncounterQueueKind::Hallway)
        );

        engine.step_map(&GameAction::ChoosePath(0));
        assert_eq!(engine.monster_encounter_queue.len(), initial_len - 1);
        assert_ne!(engine.monster_encounter_queue.front(), Some(&expected));
    }

    #[test]
    fn mystery_monster_uses_the_same_persistent_hallway_queue() {
        // EventRoom replaces itself with a real MonsterRoom after a monster
        // roll, so Java's ordinary hallway queue remains authoritative.
        // Java: EventRoom.java::onPlayerEntry,
        // AbstractDungeon.java::nextRoomTransition.
        let mut engine = RunEngine::new(73, 0);
        let expected = engine
            .monster_encounter_queue
            .front()
            .cloned()
            .expect("act one hallway queue");
        let initial_len = engine.monster_encounter_queue.len();
        engine.debug_force_event_rolls(&[0]);

        engine.enter_mystery_room();

        assert_eq!(
            engine.active_encounter_queue,
            Some(EncounterQueueKind::Hallway)
        );
        assert_eq!(engine.monster_encounter_queue.front(), Some(&expected));
        assert_eq!(engine.monster_encounter_queue.len(), initial_len);
        assert_eq!(engine.phase, RunPhase::Combat);
    }

    #[test]
    fn boss_sequence_persists_until_boss_room_entry() {
        // The all-bosses-seen branch shuffles all three IDs once, stores the
        // full list, and removes index zero only when MonsterRoomBoss is
        // entered. Keeping the tail is required for saves and A20's second
        // Act 3 boss.
        // Java: Exordium.java::initializeBoss,
        // MonsterRoomBoss.java::onPlayerEntry.
        let mut engine = RunEngine::new(42, 0);
        let initial = engine.boss_sequence.clone();
        assert_eq!(initial.len(), 3);
        assert_eq!(initial.front(), Some(&engine.boss_id));

        engine.enter_combat(false, true);

        assert_eq!(engine.boss_sequence.len(), 2);
        assert_eq!(
            engine.boss_sequence,
            initial.into_iter().skip(1).collect::<VecDeque<_>>()
        );
        assert_eq!(engine.active_combat_room_type, Some(RoomType::Boss));
    }

    #[test]
    fn courier_colored_refill_uses_ambient_math_rng_only_for_identity() {
        // ShopScreen asks rollRarity() to consume one cardRng draw, then calls
        // getCardFromPool(..., false), whose sorted identity selection uses
        // MathUtils.random. Price variance remains on merchantRng.
        // Java: ShopScreen.java::purchaseCard,
        // CardGroup.java::getRandomCard(type, useRng).
        let mut ambient_zero = RunEngine::new_with_ambient_seed(42, 0, 0);
        let mut ambient_one = RunEngine::new_with_ambient_seed(42, 0, 1);
        for engine in [&mut ambient_zero, &mut ambient_one] {
            engine.persistent_rngs.card = crate::seed::StsRandom::new(0);
            engine.persistent_rngs.merchant = crate::seed::StsRandom::new(73);
        }

        let zero_named_before = (
            ambient_zero.persistent_rngs.card.clone(),
            ambient_zero.persistent_rngs.merchant.clone(),
        );
        let one_named_before = (
            ambient_one.persistent_rngs.card.clone(),
            ambient_one.persistent_rngs.merchant.clone(),
        );
        assert_eq!(zero_named_before, one_named_before);

        let zero = ambient_zero.roll_courier_replacement_card("Strike_P");
        let one = ambient_one.roll_courier_replacement_card("Strike_P");

        assert_eq!(zero.0, "CrushJoints");
        assert_eq!(one.0, "FlurryOfBlows");
        assert_eq!(ambient_zero.persistent_rngs.card.counter, 1);
        assert_eq!(ambient_one.persistent_rngs.card.counter, 1);
        assert_eq!(ambient_zero.persistent_rngs.merchant.counter, 1);
        assert_eq!(ambient_one.persistent_rngs.merchant.counter, 1);
        assert_eq!(
            ambient_zero.persistent_rngs.card,
            ambient_one.persistent_rngs.card
        );
        assert_eq!(
            ambient_zero.persistent_rngs.merchant,
            ambient_one.persistent_rngs.merchant
        );
        assert_ne!(
            ambient_zero.ambient_math_rng.state_tuple(),
            ambient_one.ambient_math_rng.state_tuple()
        );
    }

    #[test]
    fn rng_ambient_002_jaw_worm_constructor_advances_process_global_mathutils_before_courier() {
        // AbstractMonster first constructs BobEffect, then JawWorm randomizes
        // its Spine animation time: two MathUtils.random() calls in order.
        // CardGroup.getRandomCard(false) later uses that same process-global
        // stream for Courier replacement identity.
        // Java: monsters/AbstractMonster.java, vfx/BobEffect.java,
        // monsters/exordium/JawWorm.java::<init>,
        // cards/CardGroup.java::getRandomCard(boolean).
        let mut engine = RunEngine::new_with_ambient_seed(73, 0, 0);
        let mut oracle = engine.ambient_math_rng.clone();
        let _ = oracle.random_f32();
        let _ = oracle.random_f32();

        engine.enter_specific_combat(vec!["JawWorm".to_string()]);

        assert_eq!(engine.ambient_math_rng_state(), oracle.state_tuple());
    }

    #[test]
    fn courier_consecutive_colored_purchases_include_post_purchase_mathutils_draws() {
        // purchaseCard replaces Courier stock first, then consumes speechTimer,
        // playBuySfx, getBuyMsg, speech-side, and speech-position MathUtils
        // draws. The next replacement identity therefore starts five wrapper
        // calls later.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/shop/ShopScreen.java
        // lines 589-635, 1007-1013, 1037-1040, and 1076-1084.
        let mut engine = RunEngine::new_with_ambient_seed(42, 0, 0);
        engine.run_state.relics.push("The Courier".to_string());
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);
        engine.run_state.gold = 10_000;
        engine.current_shop = Some(ShopState {
            cards: vec![("Strike_P".to_string(), 1)],
            relics: Vec::new(),
            potions: Vec::new(),
            remove_price: 75,
            removal_used: false,
        });

        let mut oracle = engine.clone();
        let first = oracle.roll_courier_replacement_card("Strike_P");
        let _ = oracle.ambient_math_rng.random_f32_range(40.0, 60.0);
        let _ = oracle.ambient_math_rng.random_int(2);
        let _ = oracle.ambient_math_rng.random_int(4);
        let _ = oracle.ambient_math_rng.random_bool();
        let _ = oracle.ambient_math_rng.random_f32_range(660.0, 1260.0);

        engine.step_shop(&GameAction::ShopBuyCard(0));
        assert_eq!(engine.current_shop.as_ref().unwrap().cards[0], first);
        assert_eq!(
            engine.ambient_math_rng_state(),
            oracle.ambient_math_rng_state()
        );

        let second = oracle.roll_courier_replacement_card(&first.0);
        let _ = oracle.ambient_math_rng.random_f32_range(40.0, 60.0);
        let _ = oracle.ambient_math_rng.random_int(2);
        let _ = oracle.ambient_math_rng.random_int(4);
        let _ = oracle.ambient_math_rng.random_bool();
        let _ = oracle.ambient_math_rng.random_f32_range(660.0, 1260.0);

        engine.step_shop(&GameAction::ShopBuyCard(0));
        assert_eq!(engine.current_shop.as_ref().unwrap().cards[0], second);
        assert_eq!(
            engine.ambient_math_rng_state(),
            oracle.ambient_math_rng_state()
        );
        assert_eq!(engine.persistent_rngs.card, oracle.persistent_rngs.card);
        assert_eq!(
            engine.persistent_rngs.merchant,
            oracle.persistent_rngs.merchant
        );
    }

    #[test]
    fn relic_and_potion_purchases_consume_four_shop_speech_draws() {
        // StoreRelic.purchaseRelic and StorePotion.purchasePotion call
        // playBuySfx, getBuyMsg, and createSpeech, consuming two ints, a
        // boolean, and a float in that order. Unlike card purchases, neither
        // resets ShopScreen.speechTimer here.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/shop/StoreRelic.java
        // and StorePotion.java.
        for is_relic in [true, false] {
            let mut engine = RunEngine::new_with_ambient_seed(42, 0, 0);
            engine.run_state.gold = 10_000;
            engine.current_shop = Some(ShopState {
                cards: Vec::new(),
                relics: is_relic
                    .then(|| vec![("Anchor".to_string(), 1)])
                    .unwrap_or_default(),
                potions: (!is_relic)
                    .then(|| vec![("Dexterity Potion".to_string(), 1)])
                    .unwrap_or_default(),
                remove_price: 75,
                removal_used: false,
            });
            let mut oracle = engine.ambient_math_rng.clone();
            let _ = oracle.random_int(2);
            let _ = oracle.random_int(4);
            let _ = oracle.random_bool();
            let _ = oracle.random_f32_range(660.0, 1260.0);

            let action = if is_relic {
                GameAction::ShopBuyRelic(0)
            } else {
                GameAction::ShopBuyPotion(0)
            };
            engine.step_shop(&action);
            assert_eq!(engine.ambient_math_rng_state(), oracle.state_tuple());
        }
    }

    #[test]
    fn courier_card_refill_truncates_once_after_all_float_discounts() {
        // Re-verified: ShopScreen.setPrice keeps tmpPrice as a float through
        // the Courier and Membership Card multipliers, then casts once.
        // Java: ShopScreen.java:657-668.
        let mut found_rounding_boundary = false;
        for merchant_seed in 0..256 {
            let mut engine = RunEngine::new_with_ambient_seed(42, 0, 0);
            engine
                .run_state
                .relics
                .extend(["The Courier".to_string(), "Membership Card".to_string()]);
            engine
                .run_state
                .relic_flags
                .rebuild(&engine.run_state.relics);
            engine.persistent_rngs.card = crate::seed::StsRandom::new(0);
            engine.persistent_rngs.merchant = crate::seed::StsRandom::new(merchant_seed);

            let mut oracle = engine.persistent_rngs.merchant.clone();
            let (card, actual_price) = engine.roll_courier_replacement_card("Strike_P");
            let base_id = card.strip_suffix('+').unwrap_or(&card);
            let base_price = if WATCHER_COMMON_CARDS.contains(&base_id) {
                50.0
            } else if WATCHER_UNCOMMON_CARDS.contains(&base_id) {
                75.0
            } else {
                150.0
            };
            let rolled_price = base_price * oracle.random_f32_range(0.9, 1.1);
            let java_price = (rolled_price * 0.8 * 0.5) as i32;
            let stale_two_stage_price = ((rolled_price as i32 as f32) * 0.8 * 0.5) as i32;
            assert_eq!(actual_price, java_price);
            assert_eq!(engine.persistent_rngs.merchant, oracle);

            if java_price != stale_two_stage_price {
                found_rounding_boundary = true;
                break;
            }
        }
        assert!(
            found_rounding_boundary,
            "fixture range must expose the old truncation bug"
        );
    }

    #[test]
    fn initial_shop_stream_order_matches_java_minimum_fixture() {
        // Merchant generates all seven cards first. ShopScreen then consumes
        // seven card prices, one sale index, two ordinary relic tier rolls,
        // three relic prices, and three potion prices. Relic identities come
        // from persistent queues and consume no relic RNG.
        // Java: Merchant.java::<init>, ShopScreen.java::{initCards,initRelics,initPotions}.
        let mut engine = RunEngine::new(4, 0);
        let before = (
            engine.persistent_rngs.card.counter,
            engine.persistent_rngs.merchant.counter,
            engine.persistent_rngs.potion.counter,
            engine.persistent_rngs.relic.counter,
        );
        engine.enter_shop();
        let delta = (
            engine.persistent_rngs.card.counter - before.0,
            engine.persistent_rngs.merchant.counter - before.1,
            engine.persistent_rngs.potion.counter - before.2,
            engine.persistent_rngs.relic.counter - before.3,
        );
        assert_eq!(delta, (12, 16, 6, 0));
    }

    #[test]
    fn canonical_encounter_keys_expand_through_monster_helper_shapes() {
        // MonsterHelper keeps encounter selection separate from concrete group
        // construction. These cases guard the previously incomplete Act 2
        // weak group and the fixed Act 4 elite group.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/MonsterHelper.java
        let mut engine = RunEngine::new(11, 0);
        engine.enter_specific_combat(vec!["3 Byrds".to_string()]);
        assert_eq!(
            engine.debug_current_enemy_ids(),
            vec!["Byrd".to_string(), "Byrd".to_string(), "Byrd".to_string()],
        );

        engine.enter_specific_combat(vec!["Shield and Spear".to_string()]);
        assert_eq!(
            engine.debug_current_enemy_ids(),
            vec!["SpireShield".to_string(), "SpireSpear".to_string()],
        );
    }

    #[test]
    fn every_seeded_encounter_key_builds_its_java_group_size() {
        // Source: each act's generateWeak/Strong/Elites methods plus
        // MonsterHelper.getEncounter.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/Exordium.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/TheCity.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/TheBeyond.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/MonsterHelper.java
        let encounter_sizes = [
            ("Cultist", 1),
            ("Jaw Worm", 1),
            ("2 Louse", 2),
            ("Small Slimes", 2),
            ("Blue Slaver", 1),
            ("Gremlin Gang", 4),
            ("Looter", 1),
            ("Large Slime", 1),
            ("Lots of Slimes", 5),
            ("Exordium Thugs", 2),
            ("Exordium Wildlife", 2),
            ("Red Slaver", 1),
            ("3 Louse", 3),
            ("2 Fungi Beasts", 2),
            ("Gremlin Nob", 1),
            ("Lagavulin", 1),
            ("3 Sentries", 3),
            ("Spheric Guardian", 1),
            ("Chosen", 1),
            ("Shell Parasite", 1),
            ("3 Byrds", 3),
            ("2 Thieves", 2),
            ("Chosen and Byrds", 2),
            ("Sentry and Sphere", 2),
            ("Snake Plant", 1),
            ("Snecko", 1),
            ("Centurion and Healer", 2),
            ("Cultist and Chosen", 2),
            ("3 Cultists", 3),
            ("Shelled Parasite and Fungi", 2),
            ("Gremlin Leader", 3),
            ("Slavers", 3),
            ("Book of Stabbing", 1),
            ("3 Darklings", 3),
            ("Orb Walker", 1),
            ("3 Shapes", 3),
            ("Spire Growth", 1),
            ("Transient", 1),
            ("4 Shapes", 4),
            ("Maw", 1),
            ("Sphere and 2 Shapes", 3),
            ("Jaw Worm Horde", 3),
            ("Writhing Mass", 1),
            ("Giant Head", 1),
            ("Nemesis", 1),
            ("Reptomancer", 3),
            ("Shield and Spear", 2),
        ];

        let mut engine = RunEngine::new(19, 20);
        for (encounter, expected_size) in encounter_sizes {
            engine.enter_specific_combat(vec![encounter.to_string()]);
            let combat = engine
                .get_combat_engine()
                .unwrap_or_else(|| panic!("{encounter} did not enter combat"));
            assert_eq!(
                combat.state.enemies.len(),
                expected_size,
                "{encounter} group size",
            );
            assert!(
                combat.state.enemies.iter().all(|enemy| enemy.entity.hp > 0),
                "{encounter} contained an invalid enemy",
            );
        }
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
            .contains(&GameAction::UsePotion(0)));
        let result = engine.step_game(&GameAction::UsePotion(0));

        assert!(result.accepted());
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
            .contains(&GameAction::UsePotion(0)));
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

        let fruit = GameAction::CombatAction(crate::actions::Action::UsePotion {
            potion_idx: 0,
            target_idx: -1,
        });
        assert!(engine.step_game(&fruit).accepted());
        let smoke = GameAction::CombatAction(crate::actions::Action::UsePotion {
            potion_idx: 1,
            target_idx: -1,
        });
        let result = engine.step_game(&smoke);

        assert!(result.accepted());
        assert!(!result.is_terminal());
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

        for (ascension, damage, strength, block) in [(0, 11, 3, 6), (2, 12, 4, 6), (17, 12, 5, 9)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["JawWorm".to_string()]);
            let enemy = &engine.combat_engine.as_ref().unwrap().state.enemies[0];
            assert_eq!(enemy.move_damage(), damage);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                damage
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STR_AMT),
                strength
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                block
            );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STR_AMT),
                strength
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::SPORE_CLOUD), 2);
            assert!(enemy.move_history.is_empty());
            assert!(matches!(
                enemy.move_id,
                crate::enemies::move_ids::FB_BITE | crate::enemies::move_ids::FB_GROW
            ));
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
            .entity
            .status(crate::status_ids::sid::CURL_UP);
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

        for (ascension, stab, rake, weak) in [(0, 12, 7, 1), (2, 13, 8, 1), (17, 13, 8, 2)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["SlaverBlue".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                stab
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), rake);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), weak);
            assert!(matches!(
                enemy.move_id,
                crate::enemies::move_ids::BS_STAB | crate::enemies::move_ids::BS_RAKE
            ));
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

        for (ascension, stab, scrape, vulnerable) in [(0, 13, 8, 1), (2, 14, 9, 1), (17, 14, 9, 2)]
        {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["SlaverRed".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.move_id, crate::enemies::move_ids::RS_STAB);
            assert_eq!(enemy.move_damage(), stab);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), scrape);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                vulnerable
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::IS_FIRST_MOVE),
                0
            );
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
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BEAR_HUG
            );
            assert_eq!(
                combat.state.enemies[0].effect(crate::combat_types::mfx::DEX_DOWN),
                Some(dexterity_down as i16)
            );

            let hp_before = combat.state.player.hp;
            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before);
            assert_eq!(
                combat
                    .state
                    .player
                    .status(crate::status_ids::sid::DEXTERITY),
                -dexterity_down
            );
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BEAR_LUNGE
            );
            assert_eq!(combat.state.enemies[0].move_damage(), lunge);
            assert_eq!(combat.state.enemies[0].move_block(), 9);
            assert_eq!(combat.ai_rng.counter, 1);

            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before - lunge);
            assert_eq!(combat.state.enemies[0].entity.block, 9);
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BEAR_MAUL
            );
            assert_eq!(combat.state.enemies[0].move_damage(), maul);
            assert_eq!(combat.ai_rng.counter, 1);

            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before - lunge - maul);
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BEAR_LUNGE
            );
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
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::POINTY_STAB
            );
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
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BANDIT_MOCK
            );

            let hp_before = combat.state.player.hp;
            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before);
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BANDIT_AGONIZE
            );
            assert_eq!(combat.state.enemies[0].move_damage(), agonize);
            assert_eq!(
                combat.state.enemies[0].effect(crate::combat_types::mfx::WEAK),
                Some(weak as i16)
            );
            assert_eq!(combat.ai_rng.counter, 1);

            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before - agonize);
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::BANDIT_CROSS_SLASH
            );
            assert_eq!(combat.state.enemies[0].move_damage(), cross);
            assert_eq!(combat.ai_rng.counter, 1);

            combat.execute_action(&crate::actions::Action::EndTurn);
            assert_eq!(combat.state.player.hp, hp_before - agonize - cross);
            assert_eq!(combat.state.enemies[0].move_id, after_first_cross);
            assert_eq!(combat.ai_rng.counter, 1);

            if ascension >= 17 {
                combat.execute_action(&crate::actions::Action::EndTurn);
                assert_eq!(combat.state.player.hp, hp_before - agonize - cross * 2);
                assert_eq!(
                    combat.state.enemies[0].move_id,
                    crate::enemies::move_ids::BANDIT_AGONIZE
                );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                stab
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STR_AMT),
                big_stab
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), a18);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::PAINFUL_STABS),
                1
            );
            assert_eq!(combat.ai_rng.counter, 1);
            match enemy.move_id {
                crate::enemies::move_ids::BOOK_STAB => {
                    assert_eq!(enemy.move_damage(), stab);
                    assert_eq!(
                        enemy.move_hits(),
                        enemy.entity.status(crate::status_ids::sid::STAB_COUNT)
                    );
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
        combat.state.enemies[0].set_move(crate::enemies::move_ids::BOOK_STAB, 6, 2, 0);
        combat.state.player.block = 6;
        combat.execute_action(&crate::actions::Action::EndTurn);
        let wounds = combat
            .state
            .discard_pile
            .iter()
            .filter(|card| combat.card_registry.card_name(card.def_id) == "Wound")
            .count();
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
            assert_eq!(
                taskmaster.move_id,
                crate::enemies::move_ids::TASK_SCOURING_WHIP
            );
            assert_eq!(taskmaster.move_damage(), 7);
            assert_eq!(
                taskmaster.effect(crate::combat_types::mfx::WOUND),
                Some(wounds)
            );
            assert_eq!(
                taskmaster.effect(crate::combat_types::mfx::STRENGTH),
                (strength_each_turn > 0).then_some(strength_each_turn)
            );
            assert!(matches!(
                taskmaster.intent,
                crate::combat_types::Intent::AttackDebuff { .. }
            ));
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
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Wound")
                .count(),
            3
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::STRENGTH),
            1
        );
        assert_eq!(combat.ai_rng.counter, ai_before + 1);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.player.hp, 485,
            "the second fixed-seven attack receives the first Strength stack"
        );
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Wound")
                .count(),
            6
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::STRENGTH),
            2
        );
        assert_eq!(combat.ai_rng.counter, ai_before + 2);

        let mut group = RunEngine::new(42, 0);
        group.enter_specific_combat(vec![
            "SlaverBlue".to_string(),
            "SlaverBoss".to_string(),
            "SlaverRed".to_string(),
        ]);
        let combat = group.combat_engine.as_ref().unwrap();
        assert_eq!(
            combat
                .state
                .enemies
                .iter()
                .map(|enemy| enemy.id.as_str())
                .collect::<Vec<_>>(),
            ["SlaverBlue", "SlaverBoss", "SlaverRed"]
        );
        assert_eq!(
            combat.ai_rng.counter, 3,
            "each Slavers group member consumes its opening rollMove draw"
        );
    }

    #[test]
    fn overwritten_monster_constructor_hp_rolls_preserve_java_rng_state() {
        // Each constructor below passes a random HP to `super`, then replaces
        // it with `setHp`. The discarded first draw is still observable in
        // every later monsterHpRng result.
        // Sources: Taskmaster.java, BronzeOrb.java, TorchHead.java,
        // OrbWalker.java, and Reptomancer.java.
        let cases = [
            ("SlaverBoss", 8, (54, 60), (57, 64)),
            ("BronzeOrb", 9, (52, 58), (54, 60)),
            ("TorchHead", 9, (38, 40), (40, 45)),
            ("Orb Walker", 7, (90, 96), (92, 102)),
            ("Reptomancer", 8, (180, 190), (190, 200)),
        ];

        for (enemy_id, ascension, constructor_range, final_range) in cases {
            let mut run = RunEngine::new(42, ascension);
            let mut oracle = run.floor_rngs.monster_hp.clone();
            let before = run.floor_rngs.monster_hp.counter;
            let _ = oracle.random_int_range(constructor_range.0, constructor_range.1);
            let expected_hp = oracle.random_int_range(final_range.0, final_range.1);

            assert_eq!(run.roll_enemy_hp(enemy_id), (expected_hp, expected_hp));
            assert_eq!(run.floor_rngs.monster_hp.counter, before + 2, "{enemy_id}");
            assert_eq!(
                run.floor_rngs.monster_hp.state_tuple(),
                oracle.state_tuple(),
                "{enemy_id}"
            );
        }
    }

    #[test]
    fn fixed_set_hp_constructors_consume_one_java_monster_hp_draw() {
        // AbstractMonster.setHp(int) delegates to setHp(int, int), whose
        // width-one random call still advances monsterHpRng.
        // Java: monsters/AbstractMonster.java::setHp.
        let cases = [
            ("BanditPointy", 0, 30),
            ("TheGuardian", 0, 240),
            ("Hexaghost", 0, 250),
            ("SlimeBoss", 0, 140),
            ("BronzeAutomaton", 0, 300),
            ("TheCollector", 0, 282),
            ("TheChamp", 0, 420),
            ("Exploder", 0, 30),
            ("WrithingMass", 0, 160),
            ("GiantHead", 0, 500),
            ("Nemesis", 0, 185),
            ("SpireGrowth", 0, 170),
            ("AwakenedOne", 0, 300),
            ("TimeEater", 0, 456),
            ("Deca", 0, 250),
            ("Donu", 0, 250),
            ("SpireShield", 0, 110),
            ("SpireSpear", 0, 160),
            ("CorruptHeart", 0, 750),
        ];

        for (enemy_id, ascension, hp) in cases {
            let mut run = RunEngine::new(42, ascension);
            let mut oracle = run.floor_rngs.monster_hp.clone();
            let expected = oracle.random_int_range(hp, hp);

            assert_eq!(
                run.roll_enemy_hp(enemy_id),
                (expected, expected),
                "{enemy_id}"
            );
            assert_eq!(
                run.floor_rngs.monster_hp.state_tuple(),
                oracle.state_tuple(),
                "{enemy_id}"
            );
            assert_eq!(run.floor_rngs.monster_hp.counter, 1, "{enemy_id}");
        }

        for enemy_id in ["SphericGuardian", "Transient", "Maw"] {
            let mut run = RunEngine::new(42, 0);
            let before = run.floor_rngs.monster_hp.state_tuple();
            let _ = run.roll_enemy_hp(enemy_id);
            assert_eq!(
                run.floor_rngs.monster_hp.state_tuple(),
                before,
                "{enemy_id}"
            );
        }
    }

    #[test]
    fn louse_and_darkling_constructor_subdraws_are_member_interleaved() {
        // Java constructors consume HP then Bite/Nip for each member before
        // moving to the next instance. Louse Curl Up is a later pre-battle pass.
        // Java: LouseNormal.java, LouseDefensive.java, and Darkling.java.
        let mut louse_run = RunEngine::new(42, 0);
        let mut louse_misc = crate::seed::StsRandom::new(42);
        let louse_ids: Vec<&str> = (0..3)
            .map(|_| {
                if louse_misc.random_bool() {
                    "FuzzyLouseNormal"
                } else {
                    "FuzzyLouseDefensive"
                }
            })
            .collect();
        let mut louse_hp = louse_run.floor_rngs.monster_hp.clone();
        let mut expected_louse = Vec::new();
        for id in &louse_ids {
            let hp = if *id == "FuzzyLouseNormal" {
                louse_hp.random_int_range(10, 15)
            } else {
                louse_hp.random_int_range(11, 17)
            };
            let bite = louse_hp.random_int_range(5, 7);
            expected_louse.push((hp, bite));
        }
        let curls: Vec<i32> = (0..3).map(|_| louse_hp.random_int_range(3, 7)).collect();

        louse_run.enter_specific_combat(vec!["3 Louse".to_string()]);
        let louse_combat = louse_run.combat_engine.as_ref().unwrap();
        assert_eq!(
            louse_combat.misc_rng.state_tuple(),
            louse_misc.state_tuple()
        );
        assert_eq!(
            louse_combat.monster_hp_rng.state_tuple(),
            louse_hp.state_tuple()
        );
        for (index, enemy) in louse_combat.state.enemies.iter().enumerate() {
            assert_eq!(enemy.id, louse_ids[index]);
            assert_eq!(enemy.entity.hp, expected_louse[index].0);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                expected_louse[index].1
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::CURL_UP),
                curls[index]
            );
        }

        let mut darkling_run = RunEngine::new(42, 0);
        let mut darkling_hp = darkling_run.floor_rngs.monster_hp.clone();
        let expected_darklings: Vec<(i32, i32)> = (0..3)
            .map(|_| {
                (
                    darkling_hp.random_int_range(48, 56),
                    darkling_hp.random_int_range(7, 11),
                )
            })
            .collect();
        darkling_run.enter_specific_combat(vec!["3 Darklings".to_string()]);
        let darkling_combat = darkling_run.combat_engine.as_ref().unwrap();
        assert_eq!(
            darkling_combat.monster_hp_rng.state_tuple(),
            darkling_hp.state_tuple()
        );
        for (enemy, (hp, nip)) in darkling_combat.state.enemies.iter().zip(expected_darklings) {
            assert_eq!(enemy.entity.hp, hp);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), nip);
        }
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                damage
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::MALLEABLE), 3);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), 3);
            assert_eq!(
                enemy
                    .entity
                    .status(crate::status_ids::sid::HIGH_ASCENSION_AI),
                high_ai
            );
            assert!(matches!(
                enemy.move_id,
                crate::enemies::move_ids::SNAKE_CHOMP | crate::enemies::move_ids::SNAKE_SPORES
            ));
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut attack = RunEngine::new(42, 2);
        attack.enter_specific_combat(vec!["SnakePlant".to_string()]);
        let combat = attack.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].set_move(crate::enemies::move_ids::SNAKE_CHOMP, 8, 3, 0);
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 476);
        assert_eq!(combat.ai_rng.counter, ai_before + 1);

        let mut spores = RunEngine::new(42, 0);
        spores.enter_specific_combat(vec!["SnakePlant".to_string()]);
        let combat = spores.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].set_move(crate::enemies::move_ids::SNAKE_SPORES, 0, 0, 0);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::FRAIL, 2);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::WEAK, 2);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::FRAIL), 2);
        assert_eq!(
            combat.state.player.status(crate::status_ids::sid::WEAKENED),
            2
        );

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
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::MALLEABLE),
            4
        );
        combat.deal_damage_to_enemy(0, 5);
        assert_eq!(combat.state.enemies[0].entity.hp, hp - 3);
        assert_eq!(combat.state.enemies[0].entity.block, 4);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::MALLEABLE),
            5
        );
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::MALLEABLE),
            3
        );

        let mut lethal = RunEngine::new(42, 0);
        lethal.enter_specific_combat(vec!["SnakePlant".to_string()]);
        let combat = lethal.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].entity.hp = 1;
        combat.state.enemies[0].entity.block = 0;
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(
            combat.state.enemies[0].entity.block, 0,
            "a lethal hit must not trigger Malleable"
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::MALLEABLE),
            3
        );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                bite
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), tail);
            assert_eq!(
                enemy
                    .entity
                    .status(crate::status_ids::sid::HIGH_ASCENSION_AI),
                high_ai
            );
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
        combat.state.draw_pile = ["ConjureBlade", "Strike", "Defend", "Vigilance", "Eruption"]
            .into_iter()
            .map(|id| combat.card_registry.make_card(id))
            .collect();
        let card_random_before = combat.card_random_rng.counter;
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(
            combat
                .state
                .player
                .status(crate::status_ids::sid::CONFUSION),
            1
        );
        assert_eq!(combat.card_random_rng.counter, card_random_before + 4);
        let x_cost = combat
            .state
            .hand
            .iter()
            .find(|card| combat.card_registry.card_name(card.def_id) == "ConjureBlade")
            .unwrap();
        assert_eq!(x_cost.cost, -1);
        assert!(combat
            .state
            .hand
            .iter()
            .filter(|card| combat.card_registry.card_name(card.def_id) != "ConjureBlade")
            .all(|card| (0..=3).contains(&card.cost)));
        assert_eq!(combat.ai_rng.counter, 2);

        // Glare uses ApplyPowerAction, so Artifact negates Confusion and is
        // consumed before the following draw can randomize any card costs.
        let mut artifact = RunEngine::new(42, 0);
        artifact.enter_specific_combat(vec!["Snecko".to_string()]);
        let combat = artifact.combat_engine.as_mut().unwrap();
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::ARTIFACT, 1);
        let card_random_before = combat.card_random_rng.counter;
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(
            combat.state.player.status(crate::status_ids::sid::ARTIFACT),
            0
        );
        assert_eq!(
            combat
                .state
                .player
                .status(crate::status_ids::sid::CONFUSION),
            0
        );
        assert_eq!(combat.card_random_rng.counter, card_random_before);

        let mut a17 = RunEngine::new(42, 17);
        a17.enter_specific_combat(vec!["Snecko".to_string()]);
        let combat = a17.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        crate::enemies::roll_next_move_with_num_and_rng(
            &mut combat.state.enemies[0],
            0,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SNECKO_TAIL
        );
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 490);
        assert_eq!(
            combat.state.player.status(crate::status_ids::sid::WEAKENED),
            2
        );
        assert_eq!(
            combat
                .state
                .player
                .status(crate::status_ids::sid::VULNERABLE),
            2
        );

        let mut bite = RunEngine::new(42, 0);
        bite.enter_specific_combat(vec!["Snecko".to_string()]);
        let combat = bite.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        crate::enemies::roll_next_move_with_num_and_rng(
            &mut combat.state.enemies[0],
            40,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SNECKO_BITE
        );
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 485);
    }

    #[test]
    fn spheric_guardian_stats_cycle_barricade_artifact_and_ticks_match_java() {
        // Source: reference/extracted/methods/monster/SphericGuardian.java.
        // The constructor is fixed at 20 HP. A2 raises all attacks to 11,
        // while A17 raises only Activate's block from 25 to 35.
        for (ascension, damage, activate_block) in [(0, 10, 25), (2, 11, 25), (17, 11, 35)] {
            let mut run = RunEngine::new(42, ascension);
            assert_eq!(run.roll_enemy_hp("SphericGuardian"), (20, 20));
            run.enter_specific_combat(vec!["SphericGuardian".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!((enemy.entity.hp, enemy.entity.max_hp), (20, 20));
            assert_eq!(enemy.entity.block, 40);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BARRICADE), 1);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ARTIFACT), 3);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                damage
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                activate_block
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), 15);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::FIRST_MOVE), 0);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::FIRST_TURN), 1);
            assert_eq!(enemy.move_id, crate::enemies::move_ids::SPHER_INITIAL_BLOCK);
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
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SPHER_FRAIL_ATTACK
        );
        assert_eq!(combat.ai_rng.counter, 2);

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 489);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::FRAIL), 5);
        assert_eq!(combat.state.enemies[0].entity.block, 75);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SPHER_BIG_ATTACK
        );
        assert_eq!(combat.ai_rng.counter, 3);

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 467);
        assert_eq!(combat.state.enemies[0].entity.block, 75);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SPHER_BLOCK_ATTACK
        );
        assert_eq!(combat.ai_rng.counter, 4);

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 456);
        assert_eq!(combat.state.enemies[0].entity.block, 90);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SPHER_BIG_ATTACK
        );
        assert_eq!(combat.ai_rng.counter, 5);

        // usePreBattleAction's three Artifact stacks negate the first three
        // debuffs without installing them, then allow the fourth through.
        let entity = &mut combat.state.enemies[0].entity;
        for remaining in [2, 1, 0] {
            assert!(!crate::powers::apply_debuff(
                entity,
                crate::status_ids::sid::WEAKENED,
                1
            ));
            assert_eq!(entity.status(crate::status_ids::sid::ARTIFACT), remaining);
            assert_eq!(entity.status(crate::status_ids::sid::WEAKENED), 0);
        }
        assert!(crate::powers::apply_debuff(
            entity,
            crate::status_ids::sid::WEAKENED,
            1
        ));
        assert_eq!(entity.status(crate::status_ids::sid::WEAKENED), 1);

        // Frail is also an ApplyPowerAction and therefore respects player Artifact.
        let mut protected = RunEngine::new(42, 0);
        protected.enter_specific_combat(vec!["SphericGuardian".to_string()]);
        let combat = protected.combat_engine.as_mut().unwrap();
        combat.execute_action(&crate::actions::Action::EndTurn);
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::ARTIFACT, 1);
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(
            combat.state.player.status(crate::status_ids::sid::ARTIFACT),
            0
        );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::FLAIL_DMG),
                flail
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BEAM_DMG), beam);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STR_AMT),
                strength
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                block
            );
            assert_eq!(
                enemy
                    .entity
                    .status(crate::status_ids::sid::HIGH_ASCENSION_AI),
                high_ai
            );
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
        assert_eq!(
            combat.state.enemies[1]
                .entity
                .status(crate::status_ids::sid::COUNT),
            0
        );
        assert_eq!(
            combat.state.enemies[2]
                .entity
                .status(crate::status_ids::sid::COUNT),
            1
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::BA_FLAIL
        );
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
        support.state.enemies[0].set_move(crate::enemies::move_ids::BA_STUNNED, 0, 0, 0);
        support.state.enemies[1].set_move(crate::enemies::move_ids::BO_SUPPORT, 0, 0, 12);
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
        combat.state.draw_pile = [
            "Strike",
            "CutThroughFate",
            "ForeignInfluence",
            "Omniscience",
        ]
        .into_iter()
        .map(|id| combat.card_registry.make_card(id))
        .collect();
        combat.state.discard_pile.clear();
        combat.state.enemies[0].set_move(crate::enemies::move_ids::BO_STASIS, 0, 0, 0);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::STASIS, 1);
        let card_random_before = combat.card_random_rng.counter;
        combat.execute_action(&crate::actions::Action::EndTurn);
        let held = combat.state.enemies[0]
            .stasis_card
            .expect("Stasis should hold one card");
        assert_eq!(combat.card_registry.card_name(held.def_id), "Omniscience");
        assert_eq!(combat.card_random_rng.counter, card_random_before + 1);
        assert!(!combat
            .state
            .draw_pile
            .iter()
            .chain(&combat.state.discard_pile)
            .chain(&combat.state.hand)
            .any(|card| combat.card_registry.card_name(card.def_id) == "Omniscience"));

        assert!(combat.instant_kill_enemy(0));
        assert!(combat
            .state
            .hand
            .iter()
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

        for (ascension, pecks, swoop, flight) in
            [(0, 5, 12, 3), (2, 6, 14, 3), (7, 6, 14, 3), (17, 6, 14, 4)]
        {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Byrd".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), pecks);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::SLASH_DMG),
                swoop
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                flight
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::FLIGHT), flight);
            assert_eq!(combat.ai_rng.counter, 2);

            let mut forced_swoop = enemy.clone();
            forced_swoop
                .entity
                .set_status(crate::status_ids::sid::FIRST_MOVE, 0);
            crate::enemies::roll_initial_move_with_num_and_rng(
                &mut forced_swoop,
                60,
                &mut crate::seed::StsRandom::new(0),
            );
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
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::FLIGHT),
            3
        );
        combat.state.enemies[0].entity.block = 0;
        combat.deal_damage_to_enemy(0, 2);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::FLIGHT),
            2
        );
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::FLIGHT),
            3
        );

        // Removing the final stack invokes changeState("GROUNDED") immediately.
        // Stunned rolls Headbutt; Headbutt installs Fly Up directly without an
        // RNG draw; Fly Up reapplies Flight and then performs one normal roll.
        let mut grounded = RunEngine::new(42, 0);
        grounded.enter_specific_combat(vec!["Byrd".to_string()]);
        let combat = grounded.combat_engine.as_mut().unwrap();
        for remaining in [2, 1, 0] {
            combat.deal_damage_to_enemy(0, 2);
            assert_eq!(
                combat.state.enemies[0]
                    .entity
                    .status(crate::status_ids::sid::FLIGHT),
                remaining
            );
        }
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::BYRD_STUNNED
        );

        let before_stunned = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::BYRD_HEADBUTT
        );
        assert_eq!(combat.ai_rng.counter, before_stunned + 1);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::BYRD_FLY_UP
        );
        assert_eq!(combat.ai_rng.counter, before_stunned + 1);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::FLIGHT),
            3
        );
        assert!(matches!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::BYRD_PECK
                | crate::enemies::move_ids::BYRD_SWOOP
                | crate::enemies::move_ids::BYRD_CAW
        ));
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
            assert_eq!(
                parasite.entity.status(crate::status_ids::sid::STARTING_DMG),
                double
            );
            assert_eq!(
                parasite.entity.status(crate::status_ids::sid::STR_AMT),
                fell
            );
            assert_eq!(
                parasite.entity.status(crate::status_ids::sid::BLOCK_AMT),
                suck
            );
            assert_eq!(
                parasite
                    .entity
                    .status(crate::status_ids::sid::HIGH_ASCENSION_AI),
                high_ai
            );
            assert_eq!(
                parasite.entity.status(crate::status_ids::sid::PLATED_ARMOR),
                14
            );
            assert_eq!(parasite.entity.block, 14);
            assert_eq!(
                parasite.entity.status(crate::status_ids::sid::FIRST_MOVE),
                0
            );
            if ascension >= 17 {
                assert_eq!(parasite.move_id, crate::enemies::move_ids::SP_FELL);
                assert_eq!(parasite.move_damage(), fell);
                assert_eq!(parasite.effect(crate::combat_types::mfx::FRAIL), Some(2));
            } else {
                assert!(matches!(
                    parasite.move_id,
                    crate::enemies::move_ids::SP_DOUBLE_STRIKE
                        | crate::enemies::move_ids::SP_LIFE_SUCK
                ));
            }
            assert_eq!(combat.ai_rng.counter, ai_ticks);
        }

        let mut fell_run = RunEngine::new(42, 17);
        fell_run.enter_specific_combat(vec!["Shelled Parasite".to_string()]);
        let combat = fell_run.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.enemies[0].set_move(crate::enemies::move_ids::SP_FELL, 21, 1, 0);
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
        combat.state.enemies[0].set_move(crate::enemies::move_ids::SP_LIFE_SUCK, 12, 1, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::HEAL, 12);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 493);
        assert_eq!(
            combat.state.enemies[0].entity.hp, 57,
            "VampireDamageAction heals exactly lastDamageTaken after block"
        );
        assert_eq!(
            combat.state.enemies[0].entity.block, 14,
            "Plated Armor grants its amount after the monster turn"
        );

        let mut break_armor = RunEngine::new(42, 0);
        break_armor.enter_specific_combat(vec!["Shelled Parasite".to_string()]);
        let combat = break_armor.combat_engine.as_mut().unwrap();
        let hp_before = combat.state.enemies[0].entity.hp;
        combat.deal_damage_to_enemy(0, 5);
        assert_eq!(combat.state.enemies[0].entity.hp, hp_before);
        assert_eq!(combat.state.enemies[0].entity.block, 9);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::PLATED_ARMOR),
            14,
            "fully blocked damage does not reduce Plated Armor"
        );

        combat.state.enemies[0].entity.block = 0;
        combat.state.enemies[0]
            .entity
            .set_status(crate::status_ids::sid::PLATED_ARMOR, 1);
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::PLATED_ARMOR),
            0
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SP_STUNNED
        );
        assert!(matches!(
            combat.state.enemies[0].intent,
            crate::combat_types::Intent::Stun
        ));

        let middle_seed = (1..10_000)
            .find(|&seed| {
                let mut rng = crate::seed::StsRandom::new(seed);
                (20..60).contains(&rng.random_int(99))
            })
            .unwrap();
        combat.ai_rng = crate::seed::StsRandom::new(middle_seed);
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.player.hp, 500,
            "armor break Stunned turn performs no attack"
        );
        assert_eq!(combat.ai_rng.counter, ai_before + 1);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SP_DOUBLE_STRIKE
        );
        assert_eq!(
            combat.state.enemies[0].move_history.last().copied(),
            Some(crate::enemies::move_ids::SP_FELL),
            "Stunned takeTurn installs Fell before its RollMoveAction"
        );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                slash
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), fury);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ATTACK_COUNT), 3);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                block
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::COUNT), 2);
            assert!(matches!(
                enemy.move_id,
                crate::enemies::move_ids::CENT_SLASH | crate::enemies::move_ids::CENT_PROTECT
            ));
            assert_eq!(combat.ai_rng.counter, 2);
        }

        // getMove's >=65 window chooses Protect with a living ally and Fury
        // without one. Both fall back to Slash after two repeats.
        let mut paired = crate::enemies::create_enemy("Centurion", 80, 80);
        paired.entity.set_status(crate::status_ids::sid::COUNT, 2);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut paired,
            65,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(paired.move_id, crate::enemies::move_ids::CENT_PROTECT);
        assert_eq!(paired.move_block(), 0);
        assert_eq!(
            paired.effect(crate::combat_types::mfx::BLOCK_RANDOM_OTHER),
            Some(15)
        );

        let mut solo = crate::enemies::create_enemy("Centurion", 80, 80);
        solo.entity.set_status(crate::status_ids::sid::COUNT, 1);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut solo,
            65,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(solo.move_id, crate::enemies::move_ids::CENT_FURY);
        assert_eq!(solo.move_damage(), 6);
        assert_eq!(solo.move_hits(), 3);

        let mut repeated_protect = crate::enemies::create_enemy("Centurion", 80, 80);
        repeated_protect
            .move_history
            .push(crate::enemies::move_ids::CENT_PROTECT);
        repeated_protect.set_move(crate::enemies::move_ids::CENT_PROTECT, 0, 0, 0);
        crate::enemies::roll_next_move_with_num(&mut repeated_protect, 99);
        assert_eq!(
            repeated_protect.move_id,
            crate::enemies::move_ids::CENT_SLASH
        );

        let mut repeated_slash = crate::enemies::create_enemy("Centurion", 80, 80);
        repeated_slash
            .move_history
            .push(crate::enemies::move_ids::CENT_SLASH);
        repeated_slash.set_move(crate::enemies::move_ids::CENT_SLASH, 12, 1, 0);
        crate::enemies::roll_next_move_with_num(&mut repeated_slash, 0);
        assert_eq!(
            repeated_slash.move_id,
            crate::enemies::move_ids::CENT_PROTECT
        );

        // GainBlockRandomMonsterAction excludes the source and consumes one
        // aiRng draw even with exactly one candidate. The following queued
        // RollMoveAction consumes the second draw.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/GainBlockRandomMonsterAction.java
        let mut centurion = crate::enemies::create_enemy("Centurion", 80, 80);
        centurion.set_move(crate::enemies::move_ids::CENT_PROTECT, 0, 0, 0);
        centurion.add_effect(crate::combat_types::mfx::BLOCK_RANDOM_OTHER, 15);
        let ally = crate::state::EnemyCombatState::new("Ally", 20, 20);
        let state = crate::state::CombatState::new(80, 80, vec![centurion, ally], Vec::new(), 3);
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
        let state = crate::state::CombatState::new(80, 80, vec![centurion], Vec::new(), 3);
        let mut combat = crate::engine::CombatEngine::new(state, 42);
        crate::combat_hooks::do_enemy_turns(&mut combat);
        assert_eq!(combat.state.enemies[0].entity.block, 15);
        assert_eq!(combat.ai_rng.counter, 1);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::COUNT),
            1
        );
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
            assert_eq!(
                healer.entity.status(crate::status_ids::sid::STARTING_DMG),
                damage
            );
            assert_eq!(
                healer.entity.status(crate::status_ids::sid::STR_AMT),
                strength
            );
            assert_eq!(
                healer.entity.status(crate::status_ids::sid::BLOCK_AMT),
                heal
            );
            assert_eq!(
                healer
                    .entity
                    .status(crate::status_ids::sid::HIGH_ASCENSION_AI),
                high_ai
            );
            assert_eq!(healer.entity.status(crate::status_ids::sid::COUNT), 0);
            assert!(matches!(
                healer.move_id,
                crate::enemies::move_ids::MYSTIC_ATTACK | crate::enemies::move_ids::MYSTIC_BUFF
            ));
            assert_eq!(combat.ai_rng.counter, 2);
        }

        let mut heal = crate::enemies::create_enemy("Healer", 56, 56);
        heal.entity.set_status(crate::status_ids::sid::COUNT, 16);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut heal,
            99,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(heal.move_id, crate::enemies::move_ids::MYSTIC_HEAL);
        assert_eq!(heal.effect(crate::combat_types::mfx::HEAL_ALL), Some(16));

        let mut attack = crate::enemies::create_enemy("Healer", 56, 56);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut attack,
            40,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(attack.move_id, crate::enemies::move_ids::MYSTIC_ATTACK);
        assert_eq!(attack.effect(crate::combat_types::mfx::FRAIL), Some(2));

        let mut buff = crate::enemies::create_enemy("Healer", 56, 56);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut buff,
            39,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(buff.move_id, crate::enemies::move_ids::MYSTIC_BUFF);
        assert_eq!(buff.effect(crate::combat_types::mfx::STRENGTH), Some(2));
        assert_eq!(
            buff.effect(crate::combat_types::mfx::STRENGTH_ALL_ALLIES),
            Some(2)
        );

        let mut a17 = crate::enemies::create_enemy("Healer", 58, 58);
        a17.entity
            .set_status(crate::status_ids::sid::STARTING_DMG, 9);
        a17.entity.set_status(crate::status_ids::sid::STR_AMT, 4);
        a17.entity.set_status(crate::status_ids::sid::BLOCK_AMT, 20);
        a17.entity
            .set_status(crate::status_ids::sid::HIGH_ASCENSION_AI, 1);
        a17.entity.set_status(crate::status_ids::sid::COUNT, 20);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut a17,
            39,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(
            a17.move_id,
            crate::enemies::move_ids::MYSTIC_BUFF,
            "A17 heals only when total missing HP is strictly above twenty"
        );
        a17.entity.set_status(crate::status_ids::sid::COUNT, 21);
        crate::enemies::roll_next_move_with_num(&mut a17, 99);
        assert_eq!(a17.move_id, crate::enemies::move_ids::MYSTIC_HEAL);
        assert_eq!(a17.effect(crate::combat_types::mfx::HEAL_ALL), Some(20));

        let mut low_repeat = crate::enemies::create_enemy("Healer", 56, 56);
        low_repeat.set_move(crate::enemies::move_ids::MYSTIC_ATTACK, 8, 1, 0);
        crate::enemies::roll_next_move_with_num(&mut low_repeat, 99);
        assert_eq!(
            low_repeat.move_id,
            crate::enemies::move_ids::MYSTIC_ATTACK,
            "below A17 a single previous Attack does not block another"
        );
        let mut high_repeat = low_repeat.clone();
        high_repeat.move_history.clear();
        high_repeat
            .entity
            .set_status(crate::status_ids::sid::HIGH_ASCENSION_AI, 1);
        crate::enemies::roll_next_move_with_num(&mut high_repeat, 99);
        assert_eq!(
            high_repeat.move_id,
            crate::enemies::move_ids::MYSTIC_BUFF,
            "A17 blocks Attack after only one previous Attack"
        );

        let mut healing_run = RunEngine::new(42, 17);
        healing_run.enter_specific_combat(vec!["Centurion".to_string(), "Healer".to_string()]);
        let combat = healing_run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].entity.hp -= 25;
        combat.state.enemies[1].entity.hp -= 10;
        combat.state.enemies[0].move_id = -1;
        combat.state.enemies[1]
            .entity
            .set_status(crate::status_ids::sid::COUNT, 35);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut combat.state.enemies[1],
            99,
            &mut crate::seed::StsRandom::new(0),
        );
        let hp_before = [
            combat.state.enemies[0].entity.hp,
            combat.state.enemies[1].entity.hp,
        ];
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.hp, hp_before[0] + 20);
        assert_eq!(
            combat.state.enemies[1].entity.hp,
            (hp_before[1] + 20).min(combat.state.enemies[1].entity.max_hp)
        );
        assert_eq!(combat.ai_rng.counter, ai_before + 1);

        let mut buff_run = RunEngine::new(42, 17);
        buff_run.enter_specific_combat(vec!["Centurion".to_string(), "Healer".to_string()]);
        let combat = buff_run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].move_id = -1;
        combat.state.enemies[1]
            .entity
            .set_status(crate::status_ids::sid::COUNT, 0);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut combat.state.enemies[1],
            0,
            &mut crate::seed::StsRandom::new(0),
        );
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::STRENGTH),
            4
        );
        assert_eq!(
            combat.state.enemies[1]
                .entity
                .status(crate::status_ids::sid::STRENGTH),
            4
        );
    }

    #[test]
    fn maw_stats_rng_branches_turn_count_and_effects_match_java() {
        // Source: reference/extracted/methods/monster/Maw.java (`constructor`,
        // `getMove`, and `takeTurn`). HP is fixed even at high ascension.
        for ascension in [0, 2, 7, 17, 20] {
            let mut run = RunEngine::new(42, ascension);
            assert_eq!(run.roll_enemy_hp("Maw"), (300, 300));
        }

        for (ascension, slam, strength, terrify) in [(0, 25, 3, 3), (2, 30, 3, 3), (17, 30, 5, 5)] {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Maw".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let maw = &combat.state.enemies[0];
            assert_eq!((maw.entity.hp, maw.entity.max_hp), (300, 300));
            assert_eq!(maw.move_id, crate::enemies::move_ids::MAW_ROAR);
            assert_eq!(maw.entity.status(crate::status_ids::sid::TURN_COUNT), 2);
            assert_eq!(
                maw.entity.status(crate::status_ids::sid::STARTING_DMG),
                slam
            );
            assert_eq!(maw.entity.status(crate::status_ids::sid::STR_AMT), strength);
            assert_eq!(
                maw.entity.status(crate::status_ids::sid::BLOCK_AMT),
                terrify
            );
            assert_eq!(
                maw.effect(crate::combat_types::mfx::WEAK),
                Some(terrify as i16)
            );
            assert_eq!(
                maw.effect(crate::combat_types::mfx::FRAIL),
                Some(terrify as i16)
            );
            assert_eq!(
                combat.ai_rng.counter, 1,
                "AbstractMonster.init opening roll consumes one aiRng value"
            );
        }

        let mut after_roar = crate::enemies::create_enemy("Maw", 300, 300);
        after_roar
            .entity
            .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
        after_roar
            .entity
            .set_status(crate::status_ids::sid::TURN_COUNT, 2);
        crate::enemies::roll_next_move_with_num(&mut after_roar, 49);
        assert_eq!(after_roar.move_id, crate::enemies::move_ids::MAW_NOM);
        assert_eq!((after_roar.move_damage(), after_roar.move_hits()), (5, 1));
        assert_eq!(
            after_roar.entity.status(crate::status_ids::sid::TURN_COUNT),
            3
        );

        let mut high_after_roar = crate::enemies::create_enemy("Maw", 300, 300);
        high_after_roar
            .entity
            .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
        high_after_roar
            .entity
            .set_status(crate::status_ids::sid::TURN_COUNT, 2);
        high_after_roar
            .entity
            .set_status(crate::status_ids::sid::STARTING_DMG, 30);
        crate::enemies::roll_next_move_with_num(&mut high_after_roar, 50);
        assert_eq!(high_after_roar.move_id, crate::enemies::move_ids::MAW_SLAM);
        assert_eq!(high_after_roar.move_damage(), 30);

        let mut slam_low = crate::enemies::create_enemy("Maw", 300, 300);
        slam_low
            .entity
            .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
        slam_low
            .entity
            .set_status(crate::status_ids::sid::TURN_COUNT, 3);
        slam_low.set_move(crate::enemies::move_ids::MAW_SLAM, 25, 1, 0);
        crate::enemies::roll_next_move_with_num(&mut slam_low, 49);
        assert_eq!(slam_low.move_id, crate::enemies::move_ids::MAW_NOM);
        assert_eq!((slam_low.move_damage(), slam_low.move_hits()), (5, 2));

        let mut slam_high = crate::enemies::create_enemy("Maw", 300, 300);
        slam_high
            .entity
            .set_status(crate::status_ids::sid::FIRST_MOVE, 1);
        slam_high
            .entity
            .set_status(crate::status_ids::sid::TURN_COUNT, 3);
        slam_high.set_move(crate::enemies::move_ids::MAW_SLAM, 25, 1, 0);
        crate::enemies::roll_next_move_with_num(&mut slam_high, 50);
        assert_eq!(slam_high.move_id, crate::enemies::move_ids::MAW_DROOL);
        assert_eq!(
            slam_high.effect(crate::combat_types::mfx::STRENGTH),
            Some(3)
        );

        let mut a17_run = RunEngine::new(42, 17);
        a17_run.enter_specific_combat(vec!["Maw".to_string()]);
        let combat = a17_run.combat_engine.as_mut().unwrap();
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.player.status(crate::status_ids::sid::WEAKENED),
            5
        );
        assert_eq!(combat.state.player.status(crate::status_ids::sid::FRAIL), 5);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::FIRST_MOVE),
            1
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::TURN_COUNT),
            3
        );
        assert_eq!(combat.ai_rng.counter, ai_before + 1);

        combat.state.enemies[0].set_move(crate::enemies::move_ids::MAW_DROOL, 0, 0, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::STRENGTH, 5);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::STRENGTH),
            5
        );
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
            assert_eq!(
                nemesis.entity.status(crate::status_ids::sid::STARTING_DMG),
                fire
            );
            assert_eq!(
                nemesis.entity.status(crate::status_ids::sid::BLOCK_AMT),
                burns
            );
            assert_eq!(nemesis.entity.status(crate::status_ids::sid::FIRST_MOVE), 0);
            assert_eq!(
                nemesis
                    .entity
                    .status(crate::status_ids::sid::SCYTHE_COOLDOWN),
                -1
            );
            match nemesis.move_id {
                crate::enemies::move_ids::NEM_TRI_ATTACK => {
                    assert_eq!((nemesis.move_damage(), nemesis.move_hits()), (fire, 3));
                }
                crate::enemies::move_ids::NEM_BURN => {
                    assert_eq!(
                        nemesis.effect(crate::combat_types::mfx::BURN),
                        Some(burns as i16)
                    );
                }
                other => panic!("invalid source opener {other}"),
            }
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut tri = crate::enemies::create_enemy("Nemesis", 200, 200);
        tri.entity
            .set_status(crate::status_ids::sid::STARTING_DMG, 7);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut tri,
            49,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(tri.move_id, crate::enemies::move_ids::NEM_TRI_ATTACK);
        assert_eq!((tri.move_damage(), tri.move_hits()), (7, 3));
        assert_eq!(
            tri.entity.status(crate::status_ids::sid::SCYTHE_COOLDOWN),
            -1
        );

        let mut burn = crate::enemies::create_enemy("Nemesis", 200, 200);
        burn.entity.set_status(crate::status_ids::sid::BLOCK_AMT, 5);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut burn,
            50,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(burn.move_id, crate::enemies::move_ids::NEM_BURN);
        assert_eq!(burn.effect(crate::combat_types::mfx::BURN), Some(5));

        crate::enemies::roll_next_move_with_num(&mut tri, 29);
        assert_eq!(tri.move_id, crate::enemies::move_ids::NEM_SCYTHE);
        assert_eq!(tri.move_damage(), 45);
        assert_eq!(
            tri.entity.status(crate::status_ids::sid::SCYTHE_COOLDOWN),
            2
        );

        let no_boolean_seed = (1..10_000)
            .find(|&seed| {
                let mut rng = crate::seed::StsRandom::new(seed);
                (30..65).contains(&rng.random_int(99))
            })
            .unwrap();
        let mut run = RunEngine::new(42, 18);
        run.enter_specific_combat(vec!["Nemesis".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].set_move(crate::enemies::move_ids::NEM_BURN, 0, 0, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::BURN, 5);
        combat.ai_rng = crate::seed::StsRandom::new(no_boolean_seed);
        run.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = run.combat_engine.as_ref().unwrap();
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Burn")
                .count(),
            5
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::INTANGIBLE),
            1
        );
        assert_eq!(
            combat.ai_rng.counter, 1,
            "the mid window without two Tri Attacks consumes only RollMoveAction"
        );
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
            assert_eq!(
                walker.entity.status(crate::status_ids::sid::STARTING_DMG),
                laser
            );
            assert_eq!(walker.entity.status(crate::status_ids::sid::STR_AMT), claw);
            assert_eq!(
                walker
                    .entity
                    .status(crate::status_ids::sid::GENERIC_STRENGTH_UP),
                growth
            );
            match walker.move_id {
                crate::enemies::move_ids::OW_LASER => {
                    assert_eq!(walker.move_damage(), laser);
                    assert_eq!(
                        walker.effect(crate::combat_types::mfx::BURN_DRAW_DISCARD),
                        Some(1)
                    );
                    assert!(matches!(
                        walker.intent,
                        crate::combat_types::Intent::AttackDebuff { .. }
                    ));
                }
                crate::enemies::move_ids::OW_CLAW => assert_eq!(walker.move_damage(), claw),
                other => panic!("invalid Orb Walker opener {other}"),
            }
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut low = crate::enemies::create_enemy("Orb Walker", 96, 96);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut low,
            39,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(low.move_id, crate::enemies::move_ids::OW_CLAW);
        crate::enemies::roll_next_move_with_num(&mut low, 39);
        assert_eq!(low.move_id, crate::enemies::move_ids::OW_CLAW);
        crate::enemies::roll_next_move_with_num(&mut low, 39);
        assert_eq!(low.move_id, crate::enemies::move_ids::OW_LASER);

        let mut high = crate::enemies::create_enemy("Orb Walker", 96, 96);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut high,
            40,
            &mut crate::seed::StsRandom::new(0),
        );
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
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::BURN_DRAW_DISCARD, 1);
        let filler = combat.card_registry.make_card("Strike");
        combat.state.draw_pile.push(filler);
        let ai_before = combat.ai_rng.counter;
        let card_random_before = combat.card_random_rng.counter;
        run.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = run.combat_engine.as_ref().unwrap();
        let draw_burns = combat
            .state
            .draw_pile
            .iter()
            .filter(|card| combat.card_registry.card_name(card.def_id) == "Burn")
            .count();
        let hand_burns = combat
            .state
            .hand
            .iter()
            .filter(|card| combat.card_registry.card_name(card.def_id) == "Burn")
            .count();
        let discard_burns = combat
            .state
            .discard_pile
            .iter()
            .filter(|card| combat.card_registry.card_name(card.def_id) == "Burn")
            .count();
        assert_eq!((draw_burns + hand_burns, discard_burns), (1, 1));
        assert_eq!(combat.card_random_rng.counter, card_random_before + 1);
        assert_eq!(combat.ai_rng.counter, ai_before + 1);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::STRENGTH),
            5
        );

        let player_hp = combat.state.player.hp;
        let combat = run.combat_engine.as_mut().unwrap();
        let burn_id = combat.card_registry.make_card("Burn").def_id;
        combat.state.hand.retain(|card| card.def_id != burn_id);
        combat.state.enemies[0].set_move(crate::enemies::move_ids::OW_CLAW, 16, 1, 0);
        combat.state.enemies[0].move_effects.clear();
        run.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = run.combat_engine.as_ref().unwrap();
        assert_eq!(
            combat.state.player.hp,
            player_hp - 21,
            "Claw uses the five Strength gained after the previous round"
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::STRENGTH),
            10
        );

        let mut pair = RunEngine::new(42, 17);
        pair.enter_specific_combat(vec!["Orb Walker".to_string(), "Orb Walker".to_string()]);
        let combat = pair.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.enemies.len(), 2);
        assert!(combat
            .state
            .enemies
            .iter()
            .all(|enemy| enemy.id == "Orb Walker"));
        assert_eq!(
            combat.ai_rng.counter, 2,
            "each event Orb Walker consumes its own opening roll"
        );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                damage
            );
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
            combat.state.enemies[0].set_move(crate::enemies::move_ids::SPIKER_BUFF, 0, 0, 0);
            combat.state.enemies[0].add_effect(crate::combat_types::mfx::THORNS, 2);
            crate::combat_hooks::do_enemy_turns(combat);
            assert_eq!(
                combat.state.enemies[0]
                    .entity
                    .status(crate::status_ids::sid::COUNT),
                executed
            );
            assert_eq!(
                combat.state.enemies[0]
                    .entity
                    .status(crate::status_ids::sid::THORNS),
                3 + executed * 2
            );
        }
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SPIKER_ATTACK
        );
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
        assert_eq!(
            combat.state.player.hp, hp_before,
            "non-attack damage must not invoke ThornsPower.onAttacked"
        );

        let mut lethal = RunEngine::new(42, 17);
        lethal.enter_specific_combat(vec!["Spiker".to_string()]);
        let combat = lethal.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 100;
        combat.state.enemies[0].entity.hp = 1;
        combat.deal_player_attack_hit_to_enemy(0, 1);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_eq!(
            combat.state.player.hp, 93,
            "lethal attack still resolves the queued Thorns hit"
        );

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
                let count = combat
                    .state
                    .enemies
                    .iter()
                    .filter(|enemy| enemy.id == id)
                    .count();
                assert!(count <= 2);
                saw_spiker |= id == "Spiker" && count > 0;
            }
            assert!(combat
                .state
                .enemies
                .iter()
                .all(|enemy| matches!(enemy.id.as_str(), "Spiker" | "Repulsor" | "Exploder")));
        }
        assert!(
            saw_spiker,
            "canonical Shapes groups must make Spiker reachable"
        );
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

        for (ascension, hp_range, damage) in [(0, 29..=35, 11), (2, 29..=35, 13), (7, 31..=38, 13)]
        {
            let mut run = RunEngine::new(42, ascension);
            run.enter_specific_combat(vec!["Repulsor".to_string()]);
            let combat = run.combat_engine.as_ref().unwrap();
            let repulsor = &combat.state.enemies[0];
            assert!(hp_range.contains(&repulsor.entity.hp));
            assert_eq!(
                repulsor.entity.status(crate::status_ids::sid::STARTING_DMG),
                damage
            );
            match repulsor.move_id {
                crate::enemies::move_ids::REPULSOR_ATTACK => {
                    assert_eq!(repulsor.move_damage(), damage);
                }
                crate::enemies::move_ids::REPULSOR_DAZE => {
                    assert_eq!(
                        repulsor.effect(crate::combat_types::mfx::DAZE_DRAW),
                        Some(2)
                    );
                    assert!(matches!(
                        repulsor.intent,
                        crate::combat_types::Intent::Debuff { .. }
                    ));
                }
                other => panic!("invalid Repulsor opener {other}"),
            }
            assert_eq!(
                combat.ai_rng.counter, 1,
                "AbstractMonster.init consumes one opening rollMove draw"
            );
        }

        let mut daze_run = RunEngine::new(42, 0);
        daze_run.enter_specific_combat(vec!["Repulsor".to_string()]);
        let combat = daze_run.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.draw_pile.clear();
        combat.state.discard_pile.clear();
        for _ in 0..3 {
            combat
                .state
                .draw_pile
                .push(combat.card_registry.make_card("Strike"));
        }
        combat.state.enemies[0].set_move(crate::enemies::move_ids::REPULSOR_DAZE, 0, 0, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::DAZE_DRAW, 2);
        combat.state.enemies[0].intent = crate::combat_types::Intent::Debuff {
            effects: crate::combat_types::fx::DAZE,
        };
        let ai_before = combat.ai_rng.counter;
        let card_random_before = combat.card_random_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat
                .state
                .draw_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Dazed")
                .count(),
            2
        );
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Dazed")
                .count(),
            0
        );
        assert_eq!(
            combat.card_random_rng.counter,
            card_random_before + 2,
            "each random draw-pile insertion consumes cardRandomRng"
        );
        assert_eq!(
            combat.ai_rng.counter,
            ai_before + 1,
            "takeTurn always queues one RollMoveAction"
        );

        let mut attack_run = RunEngine::new(42, 2);
        attack_run.enter_specific_combat(vec!["Repulsor".to_string()]);
        let combat = attack_run.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.enemies[0].set_move(crate::enemies::move_ids::REPULSOR_ATTACK, 13, 1, 0);
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
            assert_eq!(
                serpent.entity.status(crate::status_ids::sid::STARTING_DMG),
                tackle
            );
            assert_eq!(
                serpent.entity.status(crate::status_ids::sid::STR_AMT),
                smash
            );
            assert_eq!(
                serpent.entity.status(crate::status_ids::sid::BLOCK_AMT),
                constrict
            );
            assert_eq!(
                serpent
                    .entity
                    .status(crate::status_ids::sid::HIGH_ASCENSION_AI),
                high_ai
            );
            if ascension >= 17 {
                assert_eq!(serpent.move_id, crate::enemies::move_ids::SG_CONSTRICT);
                assert_eq!(
                    serpent.effect(crate::combat_types::mfx::CONSTRICT),
                    Some(12)
                );
            } else {
                assert!(matches!(
                    serpent.move_id,
                    crate::enemies::move_ids::SG_QUICK_TACKLE
                        | crate::enemies::move_ids::SG_CONSTRICT
                ));
            }
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut blocked = RunEngine::new(42, 17);
        blocked.enter_specific_combat(vec!["Serpent".to_string()]);
        let combat = blocked.combat_engine.as_mut().unwrap();
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::ARTIFACT, 1);
        combat.state.enemies[0].set_move(crate::enemies::move_ids::SG_CONSTRICT, 0, 0, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::CONSTRICT, 12);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.player.status(crate::status_ids::sid::ARTIFACT),
            0
        );
        assert_eq!(
            combat
                .state
                .player
                .status(crate::status_ids::sid::CONSTRICTED),
            0,
            "ApplyPowerAction is blocked by Artifact"
        );

        let mut applied = RunEngine::new(42, 0);
        applied.enter_specific_combat(vec!["Serpent".to_string()]);
        let combat = applied.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].set_move(crate::enemies::move_ids::SG_CONSTRICT, 0, 0, 0);
        combat.state.enemies[0].move_effects.clear();
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::CONSTRICT, 10);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat
                .state
                .player
                .status(crate::status_ids::sid::CONSTRICTED),
            10
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::COUNT),
            1,
            "the next getMove observes the installed Constricted power"
        );

        // ConstrictedPower deals DamageInfo.THORNS at player end of turn: five
        // block absorbs five of twelve, then Tungsten Rod removes one more.
        let mut damage = RunEngine::new(42, 17);
        damage.enter_specific_combat(vec!["Serpent".to_string()]);
        let combat = damage.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.player.block = 5;
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::CONSTRICTED, 12);
        combat.state.relics.push("Tungsten Rod".to_string());
        combat.state.enemies[0].move_id = -1;
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 494);
        assert_eq!(combat.state.player.block, 0);
        assert_eq!(
            combat
                .state
                .player
                .status(crate::status_ids::sid::CONSTRICTED),
            12
        );
    }

    #[test]
    fn champ_rng_forge_delayed_anger_and_execute_cadence_match_java() {
        // Source: reference/extracted/methods/monster/Champ.java (`getMove`,
        // `takeTurn`, and constructor).
        let mut defensive = crate::enemies::create_enemy("Champ", 420, 420);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut defensive,
            15,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(defensive.move_id, crate::enemies::move_ids::CHAMP_DEFENSIVE);
        assert_eq!(defensive.move_block(), 15);
        assert_eq!(
            defensive.effect(crate::combat_types::mfx::METALLICIZE),
            Some(5)
        );
        assert_eq!(
            defensive.entity.status(crate::status_ids::sid::FORGE_TIMES),
            1
        );

        let mut gloat = crate::enemies::create_enemy("Champ", 420, 420);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut gloat,
            16,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(gloat.move_id, crate::enemies::move_ids::CHAMP_GLOAT);
        assert_eq!(gloat.effect(crate::combat_types::mfx::STRENGTH), Some(2));

        let mut slap = crate::enemies::create_enemy("Champ", 420, 420);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut slap,
            31,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(slap.move_id, crate::enemies::move_ids::CHAMP_FACE_SLAP);

        let mut slash = crate::enemies::create_enemy("Champ", 420, 420);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut slash,
            56,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(slash.move_id, crate::enemies::move_ids::CHAMP_HEAVY_SLASH);

        // A19 widens only the Defensive Stance window from <=15 to <=30.
        let mut a19 = crate::enemies::create_enemy("Champ", 440, 440);
        a19.entity
            .set_status(crate::status_ids::sid::HIGH_ASCENSION_AI, 1);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut a19,
            30,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(a19.move_id, crate::enemies::move_ids::CHAMP_DEFENSIVE);

        // Forge is capped at two uses and cannot repeat immediately.
        crate::enemies::roll_next_move_with_num(&mut defensive, 0);
        assert_ne!(defensive.move_id, crate::enemies::move_ids::CHAMP_DEFENSIVE);
        crate::enemies::roll_next_move_with_num(&mut defensive, 0);
        assert_eq!(defensive.move_id, crate::enemies::move_ids::CHAMP_DEFENSIVE);
        assert_eq!(
            defensive.entity.status(crate::status_ids::sid::FORGE_TIMES),
            2
        );
        crate::enemies::roll_next_move_with_num(&mut defensive, 0);
        crate::enemies::roll_next_move_with_num(&mut defensive, 0);
        assert_ne!(defensive.move_id, crate::enemies::move_ids::CHAMP_DEFENSIVE);
        assert_eq!(
            defensive.entity.status(crate::status_ids::sid::FORGE_TIMES),
            2
        );

        // Exactly half HP does not trigger the threshold.
        let mut half = crate::enemies::create_enemy("Champ", 210, 420);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut half,
            99,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_ne!(half.move_id, crate::enemies::move_ids::CHAMP_ANGER);

        // Damage below half does not change the current intent or consume RNG.
        // The current action resolves first; its queued RollMoveAction selects
        // Anger. Anger then cleans debuffs/Shackled before adding 3*Strength.
        let mut run = RunEngine::new(42, 0);
        run.enter_specific_combat(vec!["TheChamp".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].set_move(crate::enemies::move_ids::CHAMP_FACE_SLAP, 12, 1, 0);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::FRAIL, 2);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::VULNERABLE, 2);
        let ai_before_damage = combat.ai_rng.counter;
        combat.deal_damage_to_enemy(0, 211);
        assert_eq!(combat.state.enemies[0].entity.hp, 209);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::CHAMP_FACE_SLAP
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::THRESHOLD_REACHED),
            0
        );
        assert_eq!(combat.ai_rng.counter, ai_before_damage);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::CHAMP_ANGER
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::THRESHOLD_REACHED),
            1
        );
        assert_eq!(combat.ai_rng.counter, ai_before_damage + 1);

        combat.state.enemies[0]
            .entity
            .set_status(crate::status_ids::sid::STRENGTH, -4);
        combat.state.enemies[0]
            .entity
            .set_status(crate::status_ids::sid::TEMP_STRENGTH_LOSS, 4);
        combat.state.enemies[0]
            .entity
            .set_status(crate::status_ids::sid::WEAKENED, 2);
        combat.state.enemies[0]
            .entity
            .set_status(crate::status_ids::sid::POISON, 2);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::STRENGTH),
            6
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::TEMP_STRENGTH_LOSS),
            0
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::WEAKENED),
            0
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::POISON),
            0
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::CHAMP_EXECUTE
        );

        // Execute is selected only when neither of the prior two moves was
        // Execute: one Execute, two normal intents, then Execute again.
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_ne!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::CHAMP_EXECUTE
        );
        crate::combat_hooks::do_enemy_turns(combat);
        assert_ne!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::CHAMP_EXECUTE
        );
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::CHAMP_EXECUTE
        );

        // Defensive Stance grants its immediate block and installs
        // Metallicize, which also fires at the end of the same enemy round.
        let mut forge_run = RunEngine::new(42, 0);
        forge_run.enter_specific_combat(vec!["TheChamp".to_string()]);
        let combat = forge_run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].set_move(crate::enemies::move_ids::CHAMP_DEFENSIVE, 0, 0, 15);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::METALLICIZE, 5);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::METALLICIZE),
            5
        );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                zap
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STR_AMT),
                debilitate
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::SLAP_DMG), poke);
            assert_eq!(enemy.move_id, opener);
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut normal = crate::enemies::create_enemy("Chosen", 95, 95);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut normal,
            99,
            &mut crate::seed::StsRandom::new(0),
        );
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
            &mut drain,
            99,
            &mut crate::seed::StsRandom::new(0),
        );
        crate::enemies::roll_next_move_with_num(&mut drain, 99);
        crate::enemies::roll_next_move_with_num(&mut drain, 50);
        assert_eq!(drain.move_id, crate::enemies::move_ids::CHOSEN_DRAIN);
        assert_eq!(drain.effect(crate::combat_types::mfx::WEAK), Some(3));
        assert_eq!(drain.effect(crate::combat_types::mfx::STRENGTH), Some(3));
        crate::enemies::roll_next_move_with_num(&mut drain, 40);
        assert_eq!(drain.move_id, crate::enemies::move_ids::CHOSEN_POKE);

        let mut high_ai = crate::enemies::create_enemy("Chosen", 98, 98);
        high_ai
            .entity
            .set_status(crate::status_ids::sid::HIGH_ASCENSION_AI, 1);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut high_ai,
            0,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(high_ai.move_id, crate::enemies::move_ids::CHOSEN_HEX);
        crate::enemies::roll_next_move_with_num(&mut high_ai, 50);
        assert_eq!(high_ai.move_id, crate::enemies::move_ids::CHOSEN_DRAIN);

        // takeTurn effects: Hex applies Hex 1; Drain applies Weak 3 and gives
        // Chosen Strength 3; Debilitate attacks then applies Vulnerable 2.
        let mut run = RunEngine::new(42, 0);
        run.enter_specific_combat(vec!["Chosen".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].set_move(crate::enemies::move_ids::CHOSEN_HEX, 0, 0, 0);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::HEX, 1);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.status(crate::status_ids::sid::HEX), 1);

        combat.state.enemies[0].set_move(crate::enemies::move_ids::CHOSEN_DRAIN, 0, 0, 0);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::WEAK, 3);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::STRENGTH, 3);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.player.status(crate::status_ids::sid::WEAKENED),
            3
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::STRENGTH),
            3
        );

        combat.state.enemies[0].set_move(crate::enemies::move_ids::CHOSEN_DEBILITATE, 10, 1, 0);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::VULNERABLE, 2);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat
                .state
                .player
                .status(crate::status_ids::sid::VULNERABLE),
            2
        );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOOD_HIT_COUNT),
                blood_hits
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::INVINCIBLE),
                invincible
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BEAT_OF_DEATH),
                beat
            );
            assert_eq!(enemy.move_id, crate::enemies::move_ids::HEART_DEBILITATE);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::MOVE_COUNT), 0);
            assert_eq!(
                combat.ai_rng.counter, 1,
                "AbstractMonster.rollMove consumes the opening num"
            );
        }

        let seed_for = |expected: bool| {
            (1..10_000)
                .find(|&seed| {
                    let mut rng = crate::seed::StsRandom::new(seed);
                    rng.random_bool() == expected
                })
                .unwrap()
        };
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
            assert_eq!(
                rng.counter, 1,
                "cycle slot zero consumes CorruptHeart.getMove randomBoolean"
            );
            crate::enemies::roll_next_move_with_num_and_rng(&mut enemy, 37, &mut rng);
            assert_eq!(
                enemy.move_id,
                if first_attack == crate::enemies::move_ids::HEART_ECHO {
                    crate::enemies::move_ids::HEART_BLOOD_SHOTS
                } else {
                    crate::enemies::move_ids::HEART_ECHO
                }
            );
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
        assert_eq!(
            combat
                .state
                .player
                .status(crate::status_ids::sid::VULNERABLE),
            2
        );
        assert_eq!(
            combat.state.player.status(crate::status_ids::sid::WEAKENED),
            2
        );
        assert_eq!(combat.state.player.status(crate::status_ids::sid::FRAIL), 2);
        let mut status_cards: Vec<&str> = combat
            .state
            .draw_pile
            .iter()
            .map(|card| combat.card_registry.card_name(card.def_id))
            .filter(|id| matches!(*id, "Dazed" | "Slimed" | "Wound" | "Burn" | "Void"))
            .collect();
        status_cards.sort_unstable();
        assert_eq!(status_cards, ["Burn", "Dazed", "Slimed", "Void", "Wound"]);
        assert_eq!(combat.card_random_rng.counter, card_random_before + 5);
        assert_eq!(
            combat.ai_rng.counter, 3,
            "next attack consumes rollMove num plus getMove boolean"
        );

        combat.state.enemies[0]
            .entity
            .set_status(crate::status_ids::sid::STRENGTH, -5);
        let expected_strength = [2, 4, 6, 18, 70];
        for (stage, expected) in expected_strength.into_iter().enumerate() {
            combat.state.enemies[0].set_move(crate::enemies::move_ids::HEART_BUFF, 0, 0, 0);
            crate::combat_hooks::do_enemy_turns(combat);
            assert_eq!(
                combat.state.enemies[0]
                    .entity
                    .status(crate::status_ids::sid::STRENGTH),
                expected
            );
            assert_eq!(
                combat.state.enemies[0]
                    .entity
                    .status(crate::status_ids::sid::BUFF_COUNT),
                stage as i32 + 1
            );
            match stage {
                0 => assert_eq!(
                    combat.state.enemies[0]
                        .entity
                        .status(crate::status_ids::sid::ARTIFACT),
                    2
                ),
                1 => assert_eq!(
                    combat.state.enemies[0]
                        .entity
                        .status(crate::status_ids::sid::BEAT_OF_DEATH),
                    2
                ),
                2 => assert_eq!(
                    combat.state.enemies[0]
                        .entity
                        .status(crate::status_ids::sid::PAINFUL_STABS),
                    1
                ),
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
            hp_values.insert(run.roll_enemy_hp("Dagger").0);
        }
        assert_eq!(hp_values, (20..=25).collect());

        let mut solo = RunEngine::new(42, 0);
        solo.enter_specific_combat(vec!["Dagger".to_string()]);
        let combat = solo.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.discard_pile.clear();
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SD_WOUND
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::FIRST_MOVE),
            0
        );
        assert_eq!(combat.ai_rng.counter, 1);

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 491);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SD_EXPLODE
        );
        assert_eq!(combat.ai_rng.counter, 2);
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Wound")
                .count(),
            1
        );

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 466);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert!(combat.state.combat_over && combat.state.player_won);
        assert_eq!(
            combat.ai_rng.counter, 2,
            "solo LoseHPAction clears the queued RollMoveAction"
        );

        let mut group = RunEngine::new(42, 0);
        group.enter_specific_combat(vec!["Reptomancer".to_string()]);
        let combat = group.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.discard_pile.clear();
        assert_eq!(
            combat
                .state
                .enemies
                .iter()
                .map(|enemy| enemy.id.as_str())
                .collect::<Vec<_>>(),
            ["Dagger", "Reptomancer", "Dagger"]
        );
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
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SD_EXPLODE
        );
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Wound")
                .count(),
            1
        );

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 466);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_eq!(
            combat.ai_rng.counter,
            ai_before + 2,
            "with Reptomancer alive, the dead dagger's queued roll still executes"
        );
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
                assert_eq!(
                    combat
                        .state
                        .enemies
                        .iter()
                        .map(|enemy| enemy.id.as_str())
                        .collect::<Vec<_>>(),
                    ["Dagger", "Reptomancer", "Dagger"]
                );
                let repto = &combat.state.enemies[1];
                hp_values.insert(repto.entity.hp);
                assert_eq!(
                    repto.entity.status(crate::status_ids::sid::STARTING_DMG),
                    strike
                );
                assert_eq!(repto.entity.status(crate::status_ids::sid::STR_AMT), bite);
                assert_eq!(
                    repto.entity.status(crate::status_ids::sid::BLOCK_AMT),
                    per_spawn
                );
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
        assert_eq!(
            combat.ai_rng.counter,
            ai_before + 4,
            "two original daggers, one spawned dagger, and Reptomancer each roll"
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SD_EXPLODE
        );
        assert_eq!(
            combat.state.enemies[2].move_id,
            crate::enemies::move_ids::SD_EXPLODE
        );
        assert_eq!(
            combat.state.enemies[3].move_id,
            crate::enemies::move_ids::SD_WOUND
        );
        assert!(combat.state.enemies[3].is_minion);
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Wound")
                .count(),
            2
        );

        let mut a18 = RunEngine::new(42, 18);
        a18.enter_specific_combat(vec!["Reptomancer".to_string()]);
        let combat = a18.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies.len(), 5);
        assert_eq!(
            combat.ai_rng.counter,
            ai_before + 5,
            "A18 initializes two spawned daggers before Reptomancer rolls"
        );
        assert!(combat.state.enemies[3..]
            .iter()
            .all(|enemy| enemy.id == "Dagger" && enemy.is_minion));

        // All four tracked slots are occupied, so another explicit summon
        // creates nothing even though takeTurn still queues a normal roll.
        for (idx, enemy) in combat.state.enemies.iter_mut().enumerate() {
            if idx != 1 {
                enemy.move_id = -1;
            }
        }
        combat.state.enemies[1].set_move(crate::enemies::move_ids::REPTO_SPAWN, 0, 0, 0);
        let ai_before = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies.len(), 5);
        assert_eq!(combat.ai_rng.counter, ai_before + 1);

        // Source: full Reptomancer.java `die`: every surviving group member
        // receives HideHealthBarAction followed by SuicideAction.
        let repto_hp = combat.state.enemies[1].entity.hp;
        combat.deal_damage_to_enemy(1, repto_hp);
        assert!(combat
            .state
            .enemies
            .iter()
            .all(|enemy| enemy.entity.hp == 0));
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
                assert_eq!(
                    enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                    chomp
                );
                assert_eq!(
                    enemy
                        .entity
                        .status(crate::status_ids::sid::HIGH_ASCENSION_AI),
                    high_ai
                );
                assert_eq!(enemy.entity.status(crate::status_ids::sid::REGROW), 1);
                assert_eq!(combat.ai_rng.counter, 1);
            }
            assert_eq!(hp_values, hp_range.collect());
            assert_eq!(nip_values, nip_range.collect());
        }

        let mut low = crate::enemies::create_enemy("Darkling", 48, 48);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut low,
            49,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(low.move_id, crate::enemies::move_ids::DARK_HARDEN);
        assert_eq!(low.move_block(), 12);
        assert_eq!(low.effect(crate::combat_types::mfx::STRENGTH), None);

        let mut high = crate::enemies::create_enemy("Darkling", 50, 50);
        high.entity
            .set_status(crate::status_ids::sid::STARTING_DMG, 9);
        high.entity.set_status(crate::status_ids::sid::STR_AMT, 13);
        high.entity
            .set_status(crate::status_ids::sid::HIGH_ASCENSION_AI, 1);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut high,
            49,
            &mut crate::seed::StsRandom::new(0),
        );
        assert_eq!(high.move_id, crate::enemies::move_ids::DARK_HARDEN);
        assert_eq!(high.effect(crate::combat_types::mfx::STRENGTH), Some(2));

        let mut even = crate::enemies::create_enemy("Darkling", 48, 48);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut even,
            50,
            &mut crate::seed::StsRandom::new(0),
        );
        let mut no_reroll = crate::seed::StsRandom::new(7);
        crate::enemies::roll_next_move_with_num_and_rng(&mut even, 0, &mut no_reroll);
        assert_eq!(even.move_id, crate::enemies::move_ids::DARK_CHOMP);
        assert_eq!((even.move_damage(), even.move_hits()), (8, 2));
        assert_eq!(no_reroll.counter, 0);

        let seed_for_range = |low: i32, high: i32| {
            (1..10_000)
                .find(|&seed| {
                    let mut rng = crate::seed::StsRandom::new(seed);
                    (low..=high).contains(&rng.random_int_range(40, 99))
                })
                .unwrap()
        };
        let mut odd = crate::enemies::create_enemy("Darkling", 48, 48);
        odd.entity.set_status(crate::status_ids::sid::COUNT, 1);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut odd,
            50,
            &mut crate::seed::StsRandom::new(0),
        );
        let mut reroll = crate::seed::StsRandom::new(seed_for_range(40, 69));
        crate::enemies::roll_next_move_with_num_and_rng(&mut odd, 0, &mut reroll);
        assert_eq!(odd.move_id, crate::enemies::move_ids::DARK_HARDEN);
        assert_eq!(
            reroll.counter, 1,
            "odd group index recursively rerolls num 0 into 40..=99"
        );

        let seed_for_full = (1..10_000)
            .find(|&seed| {
                let mut rng = crate::seed::StsRandom::new(seed);
                (40..=69).contains(&rng.random_int(99))
            })
            .unwrap();
        let mut repeated_nip = crate::enemies::create_enemy("Darkling", 48, 48);
        repeated_nip
            .entity
            .set_status(crate::status_ids::sid::FIRST_MOVE, 0);
        repeated_nip.move_id = crate::enemies::move_ids::DARK_NIP;
        repeated_nip.move_history = vec![crate::enemies::move_ids::DARK_NIP];
        let mut full_reroll = crate::seed::StsRandom::new(seed_for_full);
        crate::enemies::roll_next_move_with_num_and_rng(&mut repeated_nip, 99, &mut full_reroll);
        assert_eq!(repeated_nip.move_id, crate::enemies::move_ids::DARK_HARDEN);
        assert_eq!(
            full_reroll.counter, 1,
            "two Nips recursively reroll through the full 0..=99 table"
        );

        let mut revive = RunEngine::new(42, 17);
        revive
            .run_state
            .relics
            .push("Philosopher's Stone".to_string());
        revive.enter_specific_combat(vec![
            "Darkling".to_string(),
            "Darkling".to_string(),
            "Darkling".to_string(),
        ]);
        let combat = revive.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        let stored_chomp = combat.state.enemies[0]
            .entity
            .status(crate::status_ids::sid::STARTING_DMG);
        combat.state.enemies[0]
            .entity
            .set_status(crate::status_ids::sid::STRENGTH, 5);
        combat.state.enemies[0]
            .entity
            .set_status(crate::status_ids::sid::POISON, 3);
        combat.state.enemies[0]
            .entity
            .set_status(crate::status_ids::sid::ARTIFACT, 2);
        combat.deal_damage_to_enemy(0, 999);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::REBIRTH_PENDING),
            1
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::DARK_WAIT
        );
        assert!(!combat.state.enemies[0].is_targetable());
        for status in [
            crate::status_ids::sid::STRENGTH,
            crate::status_ids::sid::POISON,
            crate::status_ids::sid::ARTIFACT,
            crate::status_ids::sid::REGROW,
        ] {
            assert_eq!(combat.state.enemies[0].entity.status(status), 0);
        }
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::STARTING_DMG),
            stored_chomp
        );

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
            assert_eq!(
                darkling
                    .entity
                    .status(crate::status_ids::sid::REBIRTH_PENDING),
                0
            );
            assert_eq!(darkling.entity.status(crate::status_ids::sid::REGROW), 1);
            assert_eq!(
                darkling.entity.status(crate::status_ids::sid::STRENGTH),
                1,
                "PhilosopherStone.onSpawnMonster fires after revival"
            );
        }

        let mut finish = RunEngine::new(99, 0);
        finish.enter_specific_combat(vec![
            "Darkling".to_string(),
            "Darkling".to_string(),
            "Darkling".to_string(),
        ]);
        let combat = finish.combat_engine.as_mut().unwrap();
        for idx in 0..3 {
            combat.deal_damage_to_enemy(idx, 999);
        }
        assert!(combat.state.enemies.iter().all(|enemy| enemy.entity.hp == 0
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
            assert_eq!(
                deca.entity.status(crate::status_ids::sid::ARTIFACT),
                artifact
            );
            assert_eq!(
                deca.entity
                    .status(crate::status_ids::sid::HIGH_ASCENSION_AI),
                high_ai
            );
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
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Dazed")
                .count(),
            2
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::DECA_SQUARE
        );
        assert_eq!(combat.state.enemies[0].move_block(), 16);
        assert_eq!(
            combat.state.enemies[0].effect(crate::combat_types::mfx::BLOCK_ALL_ALLIES),
            Some(16)
        );
        assert_eq!(
            combat.state.enemies[0].effect(crate::combat_types::mfx::PLATED_ARMOR_ALL),
            None
        );
        assert_eq!(combat.ai_rng.counter, 2);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.block, 16);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::DECA_BEAM
        );
        assert_eq!(combat.ai_rng.counter, 3);

        let mut high = RunEngine::new(42, 19);
        high.enter_specific_combat(vec!["DonuAndDeca".to_string()]);
        let combat = high.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.enemies[0].set_move(crate::enemies::move_ids::DECA_SQUARE, 0, 0, 16);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::BLOCK_ALL_ALLIES, 16);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::PLATED_ARMOR_ALL, 3);
        crate::combat_hooks::do_enemy_turns(combat);
        for enemy in &combat.state.enemies {
            assert_eq!(enemy.entity.status(crate::status_ids::sid::PLATED_ARMOR), 3);
            assert_eq!(
                enemy.entity.block, 19,
                "Square's 16 block resolves before Plated Armor's end-turn 3"
            );
        }

        combat.state.enemies[0].entity.block = 0;
        let hp_before = combat.state.enemies[0].entity.hp;
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(combat.state.enemies[0].entity.hp, hp_before - 1);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::PLATED_ARMOR),
            2,
            "unblocked NORMAL damage removes one Plated Armor"
        );

        combat.state.enemies[0].set_move(crate::enemies::move_ids::DECA_SQUARE, 0, 0, 16);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::BLOCK_ALL_ALLIES, 16);
        combat.state.enemies[0].add_effect(crate::combat_types::mfx::PLATED_ARMOR_ALL, 3);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::PLATED_ARMOR),
            5
        );
        assert_eq!(
            combat.state.enemies[1]
                .entity
                .status(crate::status_ids::sid::PLATED_ARMOR),
            6
        );
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
            assert_eq!(
                donu.entity.status(crate::status_ids::sid::ARTIFACT),
                artifact
            );
            assert_eq!(donu.move_id, crate::enemies::move_ids::DONU_CIRCLE);
            assert_eq!(donu.effect(crate::combat_types::mfx::STRENGTH), Some(3));
            assert_eq!(
                donu.effect(crate::combat_types::mfx::STRENGTH_ALL_ALLIES),
                Some(3)
            );
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut run = RunEngine::new(42, 0);
        run.enter_specific_combat(vec!["DonuAndDeca".to_string()]);
        let combat = run.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        combat.state.discard_pile.clear();
        assert_eq!(
            combat.ai_rng.counter, 2,
            "Donu and Deca each consume one initialization roll"
        );
        assert_eq!(
            combat
                .state
                .enemies
                .iter()
                .map(|enemy| enemy.id.as_str())
                .collect::<Vec<_>>(),
            vec!["Deca", "Donu"]
        );

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.player.hp, 480,
            "Deca's opening Beam resolves before Donu's Circle in Java group order"
        );
        for enemy in &combat.state.enemies {
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STRENGTH), 3);
        }
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::DECA_SQUARE
        );
        assert_eq!(
            combat.state.enemies[1].move_id,
            crate::enemies::move_ids::DONU_BEAM
        );
        assert_eq!(combat.ai_rng.counter, 4);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 454);
        assert!(combat
            .state
            .enemies
            .iter()
            .all(|enemy| enemy.entity.block == 16));
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::DECA_BEAM
        );
        assert_eq!(
            combat.state.enemies[1].move_id,
            crate::enemies::move_ids::DONU_CIRCLE
        );
        assert_eq!(combat.ai_rng.counter, 6);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, 428);
        for enemy in &combat.state.enemies {
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STRENGTH), 6);
        }
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Dazed")
                .count(),
            4
        );
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
            assert_eq!(
                head.entity
                    .status(crate::status_ids::sid::STARTING_DEATH_DMG),
                death_damage
            );
            assert_eq!(
                head.entity.status(crate::status_ids::sid::COUNT),
                count_after_opening
            );
            assert_eq!(
                head.entity.status(crate::status_ids::sid::SLOW),
                1,
                "sentinel one represents SlowPower's installed amount zero"
            );
            assert!(matches!(
                head.move_id,
                crate::enemies::move_ids::GH_GLARE | crate::enemies::move_ids::GH_COUNT
            ));
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
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::STRENGTH, 4);
        combat.state.hand = ["Defend", "Strike", "Strike"]
            .iter()
            .map(|name| combat.card_registry.make_card(name))
            .collect();
        combat.state.energy = 10;
        combat.state.draw_pile.clear();
        assert_eq!(combat.state.enemies[0].entity.hp, 500);

        combat.execute_action(&crate::actions::Action::PlayCard {
            card_idx: 0,
            target_idx: -1,
        });
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::SLOW),
            2
        );
        combat.execute_action(&crate::actions::Action::PlayCard {
            card_idx: 0,
            target_idx: 0,
        });
        assert_eq!(combat.state.enemies[0].entity.hp, 489);
        combat.execute_action(&crate::actions::Action::PlayCard {
            card_idx: 0,
            target_idx: 0,
        });
        assert_eq!(combat.state.enemies[0].entity.hp, 477);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::SLOW),
            4
        );

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::SLOW),
            1,
            "SlowPower.atEndOfRound resets amount without removing the power"
        );
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
            assert!(matches!(
                initial,
                crate::enemies::move_ids::AS_S_TACKLE | crate::enemies::move_ids::AS_S_LICK
            ));
            assert_eq!(
                combat.state.enemies[0]
                    .entity
                    .status(crate::status_ids::sid::STARTING_DMG),
                damage
            );
            assert_eq!(combat.ai_rng.counter, initial_ticks);

            engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
            let combat = engine.combat_engine.as_ref().unwrap();
            let expected = if initial == crate::enemies::move_ids::AS_S_TACKLE {
                crate::enemies::move_ids::AS_S_LICK
            } else {
                crate::enemies::move_ids::AS_S_TACKLE
            };
            assert_eq!(combat.state.enemies[0].move_id, expected);
            assert_eq!(
                combat.ai_rng.counter, initial_ticks,
                "takeTurn directly sets the next move without RollMoveAction"
            );
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

        for (ascension, wound, normal, marker) in [(0, 7, 10, 0), (2, 8, 12, 0), (17, 8, 12, 17)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["AcidSlime_M".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                wound
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), normal);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                marker
            );
            assert_eq!(combat.ai_rng.counter, 1);
        }

        // Drive a real enemy turn where the 30..69 window follows Normal
        // Tackle with Java's conditional 0.4 draw.
        let seed = (1..10_000)
            .find(|&seed| {
                let mut rng = crate::seed::StsRandom::new(seed);
                let num = rng.random_int(99);
                (30..70).contains(&num)
            })
            .unwrap();
        let mut probe = crate::seed::StsRandom::new(seed);
        let _ = probe.random_int(99);
        let expected_wound = probe.random_f32() < 0.4;

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
        engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = engine.combat_engine.as_ref().unwrap();
        assert_eq!(combat.ai_rng.counter, 2);
        assert_eq!(
            combat.state.enemies[0].move_id,
            if expected_wound {
                crate::enemies::move_ids::AS_CORROSIVE_SPIT
            } else {
                crate::enemies::move_ids::AS_LICK
            }
        );
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

        for (ascension, wound, normal, marker) in [(0, 11, 16, 0), (2, 12, 18, 0), (17, 12, 18, 17)]
        {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["AcidSlime_L".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                wound
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), normal);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                marker
            );
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
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::AS_SPLIT
            );
            (half, ticks)
        };
        engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
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
        assert_eq!(
            combat.ai_rng.counter,
            ticks_before + 2,
            "each spawned medium slime initializes with one aiRng roll"
        );
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
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::SS_TACKLE
            );
            assert_eq!(combat.state.enemies[0].move_damage(), damage);
            assert_eq!(combat.ai_rng.counter, 1);

            engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::SS_TACKLE
            );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                damage
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                marker
            );
            assert!(matches!(
                enemy.move_id,
                crate::enemies::move_ids::SS_TACKLE | crate::enemies::move_ids::SS_LICK
            ));
            assert_eq!(
                combat.ai_rng.counter, 1,
                "AbstractMonster.init performs one opening roll"
            );
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
        engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = engine.combat_engine.as_ref().unwrap();
        assert_eq!(
            combat.ai_rng.counter,
            ticks_before + 1,
            "RollMoveAction consumes one aiRng draw after Tackle"
        );
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Slimed")
                .count(),
            1
        );
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

        for (ascension, damage, frail, marker) in [(0, 16, 2, 0), (2, 18, 2, 0), (17, 18, 3, 17)] {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["SpikeSlime_L".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                damage
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), frail);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                marker
            );
            assert_eq!(
                combat.ai_rng.counter, 1,
                "AbstractMonster.init performs one opening roll"
            );
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
        engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = engine.combat_engine.as_ref().unwrap();
        assert_eq!(combat.ai_rng.counter, ticks_before + 1);
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Slimed")
                .count(),
            2
        );

        let mut engine = RunEngine::new(42, 17);
        engine.enter_specific_combat(vec!["SpikeSlime_L".to_string()]);
        let (half_hp, ticks_before) = {
            let combat = engine.combat_engine.as_mut().unwrap();
            let enemy = &mut combat.state.enemies[0];
            let half = enemy.entity.max_hp / 2;
            enemy.entity.hp = half + 1;
            let ticks = combat.ai_rng.counter;
            combat.deal_damage_to_enemy(0, 1);
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::SS_SPLIT
            );
            (half, ticks)
        };
        engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = engine.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_eq!(combat.state.enemies.len(), 3);
        for child in &combat.state.enemies[1..] {
            assert_eq!(child.id, "SpikeSlime_M");
            assert_eq!((child.entity.hp, child.entity.max_hp), (half_hp, half_hp));
            assert_eq!(
                child.entity.status(crate::status_ids::sid::STARTING_DMG),
                10
            );
            assert_eq!(child.entity.status(crate::status_ids::sid::BLOCK_AMT), 17);
        }
        assert_eq!(
            combat.ai_rng.counter,
            ticks_before + 2,
            "each spawned medium slime initializes with one aiRng roll"
        );
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

        for (ascension, swipe, lunge, gold) in [(0, 10, 12, 15), (2, 11, 14, 15), (17, 11, 14, 20)]
        {
            let mut engine = RunEngine::new(42, ascension);
            engine.enter_specific_combat(vec!["Looter".to_string()]);
            let combat = engine.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                swipe
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), lunge);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::BLOCK_AMT), 6);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::TURN_COUNT),
                gold
            );
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let seed = (1..10_000)
            .find(|&seed| {
                let mut rng = crate::seed::StsRandom::new(seed);
                let _ = rng.random_f32();
                rng.random_f32() < 0.5
            })
            .unwrap();
        let mut engine = RunEngine::new(42, 0);
        engine.enter_specific_combat(vec!["Looter".to_string()]);
        engine.combat_engine.as_mut().unwrap().ai_rng = crate::seed::StsRandom::new(seed);

        engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(engine.run_state.gold, 84);
        {
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::LOOTER_MUG
            );
            assert_eq!(
                combat.state.enemies[0]
                    .entity
                    .status(crate::status_ids::sid::COUNT),
                15
            );
            assert_eq!(
                combat.ai_rng.counter, 1,
                "first Mug consumes only its 0.6 dialogue boolean"
            );
        }

        engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(engine.run_state.gold, 69);
        {
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::LOOTER_SMOKE_BOMB
            );
            assert_eq!(combat.state.enemies[0].move_block(), 6);
            assert_eq!(
                combat.ai_rng.counter, 2,
                "second Mug consumes only its 0.5 branch boolean"
            );
        }

        engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = engine.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.enemies[0].entity.block, 6);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LOOTER_ESCAPE
        );
        assert!(
            !combat.state.enemies[0].is_escaping,
            "Smoke Bomb announces Escape; the following turn performs it"
        );
        assert_eq!(combat.ai_rng.counter, 2);

        let mut refund = RunEngine::new(42, 0);
        refund.enter_specific_combat(vec!["Looter".to_string()]);
        refund.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        refund.combat_engine.as_mut().unwrap().state.enemies[0]
            .entity
            .hp = 0;
        refund.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(
            refund.run_state.gold, 84,
            "Looter death stages a reward instead of refunding theft immediately"
        );
        let rewards = refund.current_reward_screen().unwrap();
        assert_eq!(rewards.items[0].kind, RewardItemKind::StolenGold);
        assert_eq!(rewards.items[0].label, "15");
        assert_eq!(rewards.items[1].kind, RewardItemKind::Gold);
        refund.step_game(&GameAction::SelectRewardItem(0));
        assert_eq!(refund.run_state.gold, 99);
        refund.step_game(&GameAction::SelectRewardItem(1));
        assert!((109..=119).contains(&refund.run_state.gold));
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                swipe
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STR_AMT),
                big_swipe
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                block
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::TURN_COUNT),
                gold
            );
            assert_eq!(
                combat.ai_rng.counter, 1,
                "AbstractMonster.init consumes the only getMove roll"
            );
        }

        let smoke_seed = (1..10_000)
            .find(|&seed| {
                let mut rng = crate::seed::StsRandom::new(seed);
                let _ = rng.random_int(2); // first Mug voice
                let _ = rng.random_int(2); // second Mug voice
                let _ = rng.random_f32(); // second-Mug dialogue
                rng.random_f32() < 0.5 // Smoke Bomb branch
            })
            .unwrap();
        let mut engine = RunEngine::new(42, 0);
        engine.enter_specific_combat(vec!["Mugger".to_string()]);
        engine.combat_engine.as_mut().unwrap().ai_rng = crate::seed::StsRandom::new(smoke_seed);

        engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(engine.run_state.gold, 84);
        {
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::MUGGER_MUG
            );
            assert_eq!(
                combat.state.enemies[0]
                    .entity
                    .status(crate::status_ids::sid::COUNT),
                15
            );
            assert_eq!(
                combat.ai_rng.counter, 1,
                "first Mug consumes only playSfx's aiRng.random_int(2)"
            );
        }

        engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(engine.run_state.gold, 69);
        {
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::MUGGER_SMOKE_BOMB
            );
            assert_eq!(combat.state.enemies[0].move_block(), 11);
            assert_eq!(
                combat.ai_rng.counter, 4,
                "second Mug consumes voice, dialogue, and route booleans"
            );
        }

        engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        {
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(combat.state.enemies[0].entity.block, 11);
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::MUGGER_ESCAPE
            );
            assert!(!combat.state.enemies[0].is_escaping);
            assert_eq!(
                combat.ai_rng.counter, 4,
                "direct SetMove actions consume no RollMoveAction tick"
            );
        }

        let big_seed = (1..10_000)
            .find(|&seed| {
                let mut rng = crate::seed::StsRandom::new(seed);
                let _ = rng.random_int(2);
                let _ = rng.random_int(2);
                let _ = rng.random_f32();
                rng.random_f32() >= 0.5
            })
            .unwrap();
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
        a17_smoke
            .entity
            .set_status(crate::status_ids::sid::BLOCK_AMT, 17);
        let mut a17_rng = crate::seed::StsRandom::new(smoke_seed);
        crate::enemies::act2::advance_mugger_after_turn(&mut a17_smoke, &mut a17_rng);
        crate::enemies::act2::advance_mugger_after_turn(&mut a17_smoke, &mut a17_rng);
        assert_eq!(
            a17_smoke.move_id,
            crate::enemies::move_ids::MUGGER_SMOKE_BOMB
        );
        assert_eq!(
            a17_smoke.move_block(),
            17,
            "A17 adds six to the constructor's eleven escape block"
        );

        let mut death = RunEngine::new(42, 0);
        death.enter_specific_combat(vec!["Mugger".to_string()]);
        let combat = death.combat_engine.as_mut().unwrap();
        let before_death = combat.ai_rng.counter;
        let hp = combat.state.enemies[0].entity.hp;
        combat.deal_damage_to_enemy(0, hp);
        assert_eq!(
            combat.ai_rng.counter,
            before_death + 1,
            "Mugger.die consumes one aiRng voice variant"
        );

        let mut refund = RunEngine::new(42, 0);
        refund.enter_specific_combat(vec!["Mugger".to_string()]);
        refund.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        refund.combat_engine.as_mut().unwrap().state.enemies[0]
            .entity
            .hp = 0;
        refund.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(
            refund.run_state.gold, 84,
            "Mugger death stages a reward instead of refunding theft immediately"
        );
        let rewards = refund.current_reward_screen().unwrap();
        assert_eq!(rewards.items[0].kind, RewardItemKind::StolenGold);
        assert_eq!(rewards.items[0].label, "15");
        assert_eq!(rewards.items[1].kind, RewardItemKind::Gold);
        refund.step_game(&GameAction::SelectRewardItem(0));
        assert_eq!(refund.run_state.gold, 99);
        refund.step_game(&GameAction::SelectRewardItem(1));
        assert!((109..=119).contains(&refund.run_state.gold));
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
            assert!(combat.state.enemies[..2]
                .iter()
                .all(|enemy| enemy.is_minion));
            assert!(combat.state.enemies[..2].iter().all(|enemy| matches!(
                enemy.id.as_str(),
                "GremlinWarrior"
                    | "GremlinThief"
                    | "GremlinFat"
                    | "GremlinTsundere"
                    | "GremlinWizard"
            )));
            let leader = &combat.state.enemies[2];
            assert_eq!(leader.id, "GremlinLeader");
            assert!(hp_range.contains(&leader.entity.hp));
            assert_eq!(
                leader.entity.status(crate::status_ids::sid::STR_AMT),
                strength
            );
            assert_eq!(
                leader.entity.status(crate::status_ids::sid::BLOCK_AMT),
                block
            );
            assert_eq!(leader.entity.status(crate::status_ids::sid::COUNT), 2);
            assert_eq!(
                leader.entity.status(crate::status_ids::sid::STARTING_DMG),
                ascension
            );
            assert!(matches!(
                leader.move_id,
                crate::enemies::move_ids::GL_ENCOURAGE | crate::enemies::move_ids::GL_STAB
            ));
            assert_eq!(
                combat.ai_rng.counter, 3,
                "two gremlins and the Leader each consume one opening roll"
            );
        }

        // With one ally and a repeated Rally, the low branch recursively draws
        // 50..99 and consumes exactly one additional aiRng tick.
        let mut recursive = crate::enemies::create_enemy("GremlinLeader", 140, 140);
        recursive
            .entity
            .set_status(crate::status_ids::sid::COUNT, 1);
        let mut ai_rng = crate::seed::StsRandom::new(9);
        crate::enemies::roll_next_move_with_num_and_rng(&mut recursive, 0, &mut ai_rng);
        assert!(matches!(
            recursive.move_id,
            crate::enemies::move_ids::GL_ENCOURAGE | crate::enemies::move_ids::GL_STAB
        ));
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
        assert_eq!(
            combat.state.enemies[2]
                .entity
                .status(crate::status_ids::sid::STRENGTH),
            5
        );
        assert_eq!(
            combat.state.enemies[2].entity.block, 0,
            "Encourage never gives the Leader block"
        );
        for minion in &combat.state.enemies[..2] {
            assert_eq!(minion.entity.status(crate::status_ids::sid::STRENGTH), 5);
            assert_eq!(minion.entity.block, 10);
        }
        assert_eq!(
            combat.ai_rng.counter,
            ai_before + 2,
            "Encourage quote and queued RollMove each consume one tick"
        );

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
        assert!(summons
            .iter()
            .all(|enemy| enemy.is_minion && enemy.is_alive()));
        assert!(summons.iter().all(|enemy| matches!(
            enemy.id.as_str(),
            "GremlinWarrior" | "GremlinThief" | "GremlinFat" | "GremlinTsundere" | "GremlinWizard"
        )));
        assert_eq!(
            combat.state.enemies[2]
                .entity
                .status(crate::status_ids::sid::COUNT),
            2
        );
        assert_eq!(
            combat.ai_rng.counter,
            ai_before + 5,
            "two weighted type rolls, two init rolls, then Leader RollMove"
        );

        let mut death = RunEngine::new(42, 0);
        death.enter_specific_combat(vec!["GremlinLeader".to_string()]);
        let combat = death.combat_engine.as_mut().unwrap();
        combat.deal_damage_to_enemy(2, 999);
        assert_eq!(combat.state.enemies[2].entity.hp, 0);
        assert!(combat.state.enemies[..2]
            .iter()
            .all(|enemy| enemy.is_escaping && enemy.entity.hp == 0));
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
            assert_eq!(
                enemy.effect(crate::combat_types::mfx::FRAIL),
                frail.then_some(1)
            );
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut a17 = RunEngine::new(42, 17);
        a17.enter_specific_combat(vec!["GremlinFat".to_string()]);
        a17.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = a17.combat_engine.as_ref().unwrap();
        assert_eq!(
            combat.state.player.status(crate::status_ids::sid::WEAKENED),
            1
        );
        assert_eq!(combat.state.player.status(crate::status_ids::sid::FRAIL), 1);
        assert_eq!(
            combat.ai_rng.counter, 2,
            "the normal attack queues exactly one RollMoveAction"
        );

        let mut group = RunEngine::new(42, 0);
        group.enter_specific_combat(vec!["GremlinFat".to_string(), "GremlinThief".to_string()]);
        let combat = group.combat_engine.as_mut().unwrap();
        combat.state.enemies[1].entity.hp = 1;
        combat.deal_damage_to_enemy(1, 1);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ESCAPE
        );
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

            engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::GREMLIN_ATTACK
            );
            assert_eq!(combat.state.enemies[0].move_damage(), damage);
            assert_eq!(
                combat.ai_rng.counter, 1,
                "Puncture sets Puncture directly without RollMoveAction"
            );
        }

        let mut group = RunEngine::new(42, 0);
        group.enter_specific_combat(vec![
            "GremlinThief".to_string(),
            "GremlinWarrior".to_string(),
        ]);
        let combat = group.combat_engine.as_mut().unwrap();
        combat.state.enemies[1].entity.hp = 1;
        combat.deal_damage_to_enemy(1, 1);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ESCAPE
        );
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
                assert_eq!(
                    combat.state.enemies[0]
                        .entity
                        .status(crate::status_ids::sid::ANGRY),
                    angry
                );
                assert_eq!(combat.ai_rng.counter, 1);
                combat.deal_damage_to_enemy(0, 1);
                assert_eq!(
                    combat.state.enemies[0]
                        .entity
                        .status(crate::status_ids::sid::STRENGTH),
                    angry
                );
            }

            engine.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
            let combat = engine.combat_engine.as_ref().unwrap();
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::GREMLIN_ATTACK
            );
            assert_eq!(
                combat.ai_rng.counter, 1,
                "Scratch sets Scratch directly without RollMoveAction"
            );
        }

        let mut group = RunEngine::new(42, 0);
        group.enter_specific_combat(vec![
            "GremlinWarrior".to_string(),
            "GremlinThief".to_string(),
        ]);
        let combat = group.combat_engine.as_mut().unwrap();
        combat.state.enemies[1].entity.hp = 1;
        combat.deal_damage_to_enemy(1, 1);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ESCAPE
        );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                damage
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::COUNT), 1);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                marker
            );
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut low = RunEngine::new(42, 0);
        low.enter_specific_combat(vec!["GremlinWizard".to_string()]);
        low.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(
            low.combat_engine.as_ref().unwrap().state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_PROTECT
        );
        low.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        assert_eq!(
            low.combat_engine.as_ref().unwrap().state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ATTACK
        );
        low.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = low.combat_engine.as_ref().unwrap();
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_PROTECT
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::COUNT),
            0
        );
        assert_eq!(
            combat.ai_rng.counter, 1,
            "all charge/blast transitions use direct SetMoveAction calls"
        );

        let mut a17 = RunEngine::new(42, 17);
        a17.enter_specific_combat(vec!["GremlinWizard".to_string()]);
        a17.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        a17.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        a17.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = a17.combat_engine.as_ref().unwrap();
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ATTACK,
            "A17 repeats Ultimate Blast immediately"
        );
        assert_eq!(combat.ai_rng.counter, 1);

        let mut group = RunEngine::new(42, 0);
        group.enter_specific_combat(vec![
            "GremlinWizard".to_string(),
            "GremlinThief".to_string(),
        ]);
        let combat = group.combat_engine.as_mut().unwrap();
        combat.state.enemies[1].entity.hp = 1;
        combat.deal_damage_to_enemy(1, 1);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ESCAPE
        );
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
            assert_eq!(
                enemy.move_id,
                crate::enemies::move_ids::GREMLIN_TSUNDERE_PROTECT
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                damage
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                block
            );
            assert_eq!(
                enemy.effect(crate::combat_types::mfx::BLOCK_RANDOM_OTHER),
                Some(block as i16)
            );
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut solo = RunEngine::new(42, 0);
        solo.enter_specific_combat(vec!["GremlinTsundere".to_string()]);
        {
            let combat = solo.combat_engine.as_mut().unwrap();
            crate::combat_hooks::do_enemy_turns(combat);
            assert_eq!(combat.state.enemies[0].entity.block, 7);
            assert_eq!(
                combat.state.enemies[0].move_id,
                crate::enemies::move_ids::GREMLIN_TSUNDERE_BASH
            );
            assert_eq!(
                combat.ai_rng.counter, 1,
                "no eligible ally means self-block without an aiRng draw"
            );
        }

        let mut group = RunEngine::new(42, 17);
        group.enter_specific_combat(vec![
            "GremlinTsundere".to_string(),
            "GremlinThief".to_string(),
            "GremlinWarrior".to_string(),
        ]);
        let opening_ticks = group.combat_engine.as_ref().unwrap().ai_rng.counter;
        let combat = group.combat_engine.as_mut().unwrap();
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.block, 0);
        assert_eq!(
            combat.state.enemies[1..]
                .iter()
                .filter(|enemy| enemy.entity.block == 11)
                .count(),
            1
        );
        assert_eq!(
            combat.ai_rng.counter,
            opening_ticks + 1,
            "two eligible allies require exactly one random-target draw"
        );

        let mut escape = RunEngine::new(42, 0);
        escape.enter_specific_combat(vec![
            "GremlinTsundere".to_string(),
            "GremlinThief".to_string(),
        ]);
        let combat = escape.combat_engine.as_mut().unwrap();
        combat.state.enemies[1].entity.hp = 1;
        combat.deal_damage_to_enemy(1, 1);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GREMLIN_ESCAPE
        );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                rush
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), bash);
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::TURN_COUNT),
                enrage
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::BLOCK_AMT),
                marker
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ENRAGE), 0);
            assert_eq!(
                enemy.effect(crate::combat_types::mfx::ENRAGE),
                Some(enrage as i16)
            );
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut a18 = RunEngine::new(42, 18);
        a18.enter_specific_combat(vec!["GremlinNob".to_string()]);
        a18.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        {
            let combat = a18.combat_engine.as_ref().unwrap();
            let enemy = &combat.state.enemies[0];
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ENRAGE), 3);
            assert_eq!(
                enemy.move_id,
                crate::enemies::move_ids::NOB_SKULL_BASH,
                "A18 forces Bash when neither of the previous two moves was Bash"
            );
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
        a18.step_game(&GameAction::CombatAction(
            crate::actions::Action::PlayCard {
                card_idx: 0,
                target_idx: -1,
            },
        ));
        assert_eq!(
            a18.combat_engine.as_ref().unwrap().state.enemies[0]
                .entity
                .status(crate::status_ids::sid::STRENGTH),
            3,
            "the Enrage installed by Bellow triggers on a Skill"
        );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                damage
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), debuff);
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut natural = RunEngine::new(42, 18);
        natural.enter_specific_combat(vec!["Lagavulin".to_string()]);
        let combat = natural.combat_engine.as_mut().unwrap();
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_SLEEP
        );
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_SLEEP
        );
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_ATTACK
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::METALLICIZE),
            0
        );
        assert_eq!(
            combat.ai_rng.counter, 3,
            "the first two idle turns roll; natural wake on idle three does not"
        );

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_ATTACK
        );
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_SIPHON
        );
        assert_eq!(
            combat.state.enemies[0].effect(crate::combat_types::mfx::SIPHON_STR),
            Some(2)
        );
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.player.status(crate::status_ids::sid::STRENGTH),
            -2
        );
        assert_eq!(
            combat
                .state
                .player
                .status(crate::status_ids::sid::DEXTERITY),
            -2
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_ATTACK
        );
        assert_eq!(combat.ai_rng.counter, 6);

        let mut hit_wake = RunEngine::new(42, 0);
        hit_wake.enter_specific_combat(vec!["Lagavulin".to_string()]);
        let combat = hit_wake.combat_engine.as_mut().unwrap();
        combat.deal_damage_to_enemy(0, 8);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_SLEEP,
            "fully blocked damage does not change currentHealth or wake Lagavulin"
        );
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_STUN
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::METALLICIZE),
            0
        );
        let ticks = combat.ai_rng.counter;
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::LAGA_ATTACK
        );
        assert_eq!(
            combat.ai_rng.counter,
            ticks + 1,
            "the stunned wake turn queues one RollMoveAction"
        );
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
            assert_eq!(
                enemy.effect(crate::combat_types::mfx::DAZE),
                Some(daze as i16)
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STARTING_DMG),
                damage
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ARTIFACT), 1);
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut single = RunEngine::new(42, 0);
        single.enter_specific_combat(vec!["Sentry".to_string()]);
        single.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = single.combat_engine.as_ref().unwrap();
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Dazed")
                .count(),
            2
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SENTRY_BEAM
        );
        assert_eq!(combat.state.enemies[0].move_damage(), 9);
        assert_eq!(combat.ai_rng.counter, 2);

        let mut trio = RunEngine::new(42, 18);
        trio.enter_specific_combat(vec![
            "Sentry".to_string(),
            "Sentry".to_string(),
            "Sentry".to_string(),
        ]);
        {
            let combat = trio.combat_engine.as_ref().unwrap();
            assert_eq!(
                combat
                    .state
                    .enemies
                    .iter()
                    .map(|enemy| enemy.move_id)
                    .collect::<Vec<_>>(),
                vec![
                    crate::enemies::move_ids::SENTRY_BOLT,
                    crate::enemies::move_ids::SENTRY_BEAM,
                    crate::enemies::move_ids::SENTRY_BOLT,
                ]
            );
            assert_eq!(combat.ai_rng.counter, 3);
        }
        let hp_before = trio.combat_engine.as_ref().unwrap().state.player.hp;
        trio.step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn));
        let combat = trio.combat_engine.as_ref().unwrap();
        assert_eq!(combat.state.player.hp, hp_before - 10);
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Dazed")
                .count(),
            6
        );
        assert_eq!(
            combat
                .state
                .enemies
                .iter()
                .map(|enemy| enemy.move_id)
                .collect::<Vec<_>>(),
            vec![
                crate::enemies::move_ids::SENTRY_BEAM,
                crate::enemies::move_ids::SENTRY_BOLT,
                crate::enemies::move_ids::SENTRY_BEAM,
            ]
        );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::MODE_SHIFT),
                threshold
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::FIERCE_BASH_DMG),
                fierce
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::ROLL_DMG), roll);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), sharp);
            assert_eq!(enemy.move_id, crate::enemies::move_ids::GUARD_CHARGING_UP);
            assert_eq!(enemy.move_block(), 9);
            assert_eq!(
                combat.ai_rng.counter, 1,
                "the constructor's rollMove consumes exactly one aiRng value"
            );
        }

        let mut offensive = RunEngine::new(42, 0);
        offensive.enter_specific_combat(vec!["TheGuardian".to_string()]);
        let combat = offensive.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 200;
        combat.state.player.max_hp = 200;

        crate::combat_hooks::do_enemy_turns(combat); // Charge Up -> Fierce Bash
        assert_eq!(combat.state.enemies[0].entity.block, 9);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_FIERCE_BASH
        );
        assert_eq!(combat.state.enemies[0].move_damage(), 32);
        crate::combat_hooks::do_enemy_turns(combat); // Fierce Bash -> Vent Steam
        assert_eq!(combat.state.player.hp, 168);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_VENT_STEAM
        );
        crate::combat_hooks::do_enemy_turns(combat); // Vent Steam -> Whirlwind
        assert_eq!(
            combat.state.player.status(crate::status_ids::sid::WEAKENED),
            2
        );
        assert_eq!(
            combat
                .state
                .player
                .status(crate::status_ids::sid::VULNERABLE),
            2
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_WHIRLWIND
        );
        crate::combat_hooks::do_enemy_turns(combat); // Whirlwind -> Charge Up
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_CHARGING_UP
        );
        assert_eq!(
            combat.ai_rng.counter, 1,
            "Guardian's direct setMove cycle consumes no later aiRng values"
        );

        let mut defensive = RunEngine::new(42, 19);
        defensive.enter_specific_combat(vec!["TheGuardian".to_string()]);
        let combat = defensive.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 200;
        combat.state.player.max_hp = 200;

        combat.deal_damage_to_enemy(0, 39);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::DAMAGE_TAKEN_THIS_MODE),
            39
        );
        combat.state.enemies[0].entity.block = 1;
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_CHARGING_UP,
            "blocked damage does not count toward Mode Shift"
        );
        combat.deal_damage_to_enemy(0, 1);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::PHASE),
            1
        );
        assert_eq!(combat.state.enemies[0].entity.block, 20);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::MODE_SHIFT),
            50
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_CLOSE_UP
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::SHARP_HIDE),
            0,
            "Sharp Hide is applied by Close Up, not by the mode change"
        );

        crate::combat_hooks::do_enemy_turns(combat); // Close Up -> Roll Attack
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::SHARP_HIDE),
            4
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_ROLL_ATTACK
        );

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
        assert_eq!(
            combat.state.player.hp,
            hp_before - 3,
            "Tungsten Rod reduces Sharp Hide's THORNS HP damage by one"
        );

        combat.state.relics.retain(|id| id != "Tungsten Rod");
        combat.state.relics.push("Necronomicon".to_string());
        combat.state.enemies[0].entity.block = 200;
        combat.state.energy = 3;
        let bash = combat.card_registry.make_card("Bash");
        combat.state.hand.push(bash);
        let hp_before = combat.state.player.hp;
        combat.play_card(combat.state.hand.len() - 1, 0);
        assert_eq!(
            combat.state.player.hp,
            hp_before - 8,
            "Necronomicon's replay constructs a second Sharp Hide onUseCard hit"
        );

        crate::combat_hooks::do_enemy_turns(combat); // Roll Attack -> Twin Slam
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_TWIN_SLAM
        );
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::THORNS, 3);
        let guardian_hp_before = combat.state.enemies[0].entity.hp;
        crate::combat_hooks::do_enemy_turns(combat); // Offensive Mode, Twin Slam -> Whirlwind
        assert_eq!(combat.state.enemies[0].entity.hp, guardian_hp_before - 6);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::DAMAGE_TAKEN_THIS_MODE),
            6,
            "Offensive Mode resolves before Twin Slam, so both Thorns hits count"
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::PHASE),
            0
        );
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::SHARP_HIDE),
            0
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_WHIRLWIND
        );
        assert_eq!(combat.ai_rng.counter, 1);

        let mut lethal = RunEngine::new(42, 19);
        lethal.enter_specific_combat(vec!["TheGuardian".to_string()]);
        let combat = lethal.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].entity.hp = 40;
        combat.deal_damage_to_enemy(0, 40);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_ne!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::GUARD_CLOSE_UP,
            "TheGuardian.damage excludes lethal hits with !isDying"
        );
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::FIRE_TACKLE_DMG),
                tackle
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::INFERNO_DMG),
                inferno
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::STR_AMT),
                strength
            );
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::SEAR_BURN_COUNT),
                burns
            );
            assert_eq!(enemy.move_id, crate::enemies::move_ids::HEX_ACTIVATE);
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut engine = RunEngine::new(42, 0);
        engine.enter_specific_combat(vec!["Hexaghost".to_string()]);
        let combat = engine.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 77;
        combat.state.player.max_hp = 500;

        crate::combat_hooks::do_enemy_turns(combat); // Activate -> Divider
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_DIVIDER
        );
        assert_eq!(combat.state.enemies[0].move_damage(), 77 / 12 + 1);
        assert_eq!(combat.state.enemies[0].move_hits(), 6);
        assert_eq!(
            combat.ai_rng.counter, 1,
            "Activate sets Divider directly without RollMoveAction"
        );

        combat.state.player.hp = 500;
        let seed_burn = combat.card_registry.make_card("Burn");
        combat.state.draw_pile.push(seed_burn);
        crate::combat_hooks::do_enemy_turns(combat); // Divider -> Sear(0)
        assert_eq!(combat.state.player.hp, 458);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_SEAR
        );
        assert_eq!(
            combat.state.enemies[0].effect(crate::combat_types::mfx::BURN),
            Some(1)
        );
        assert_eq!(combat.ai_rng.counter, 2);

        crate::combat_hooks::do_enemy_turns(combat); // Sear(0) -> Tackle(1)
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_TACKLE
        );
        assert_eq!(combat.state.enemies[0].move_damage(), 5);
        crate::combat_hooks::do_enemy_turns(combat); // Tackle(1) -> Sear(2)
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_SEAR
        );
        crate::combat_hooks::do_enemy_turns(combat); // Sear(2) -> Inflame(3)
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_INFLAME
        );
        crate::combat_hooks::do_enemy_turns(combat); // Inflame(3) -> Tackle(4)
        assert_eq!(combat.state.enemies[0].entity.block, 12);
        assert_eq!(
            combat.state.enemies[0]
                .entity
                .status(crate::status_ids::sid::STRENGTH),
            2
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_TACKLE
        );
        crate::combat_hooks::do_enemy_turns(combat); // Tackle(4) -> Sear(5)
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_SEAR
        );
        crate::combat_hooks::do_enemy_turns(combat); // Sear(5) -> Inferno(6)
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_INFERNO
        );
        assert_eq!(combat.state.enemies[0].move_damage(), 2);
        assert_eq!(combat.state.enemies[0].move_hits(), 6);
        assert_eq!(combat.ai_rng.counter, 8);

        crate::combat_hooks::do_enemy_turns(combat); // Inferno -> Sear(0)
        let draw_names: Vec<&str> = combat
            .state
            .draw_pile
            .iter()
            .map(|card| combat.card_registry.card_name(card.def_id))
            .collect();
        let discard_names: Vec<&str> = combat
            .state
            .discard_pile
            .iter()
            .map(|card| combat.card_registry.card_name(card.def_id))
            .collect();
        assert!(draw_names.iter().all(|name| *name != "Burn"));
        assert!(discard_names.iter().all(|name| *name != "Burn"));
        assert_eq!(
            draw_names
                .iter()
                .chain(discard_names.iter())
                .filter(|name| **name == "Burn+")
                .count(),
            7,
            "three Sear Burns plus the seeded Burn are upgraded, then three Burn+ are added"
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::HEX_SEAR
        );
        assert_eq!(
            combat.state.enemies[0].effect(crate::combat_types::mfx::BURN_PLUS),
            Some(1)
        );
        assert_eq!(combat.ai_rng.counter, 9);

        crate::combat_hooks::do_enemy_turns(combat); // upgraded Sear -> Tackle
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Burn+")
                .count(),
            7
        );
        assert_eq!(
            combat
                .state
                .draw_pile
                .iter()
                .chain(combat.state.discard_pile.iter())
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Burn+")
                .count(),
            8
        );

        let mut a19 = RunEngine::new(42, 19);
        a19.enter_specific_combat(vec!["Hexaghost".to_string()]);
        let combat = a19.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 200;
        crate::combat_hooks::do_enemy_turns(combat);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.enemies[0].effect(crate::combat_types::mfx::BURN),
            Some(2)
        );
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
        assert!(!crate::enemies::known_enemy_ids()
            .iter()
            .any(|(id, _)| matches!(*id, "HexaghostBody" | "HexaghostOrb")));
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
            assert_eq!(
                enemy.entity.status(crate::status_ids::sid::FIRE_TACKLE_DMG),
                tackle
            );
            assert_eq!(enemy.entity.status(crate::status_ids::sid::SLAP_DMG), slam);
            assert_eq!(enemy.entity.status(crate::status_ids::sid::STR_AMT), slimed);
            assert_eq!(enemy.move_id, crate::enemies::move_ids::SB_STICKY);
            assert_eq!(
                enemy.effect(crate::combat_types::mfx::SLIMED),
                Some(slimed as i16)
            );
            assert_eq!(combat.ai_rng.counter, 1);
        }

        let mut cycle = RunEngine::new(42, 19);
        cycle.enter_specific_combat(vec!["SlimeBoss".to_string()]);
        let combat = cycle.combat_engine.as_mut().unwrap();
        combat.state.player.hp = 200;
        combat.state.player.max_hp = 200;
        crate::combat_hooks::do_enemy_turns(combat); // Sticky -> Prep Slam
        assert_eq!(
            combat
                .state
                .discard_pile
                .iter()
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Slimed")
                .count(),
            5
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SB_PREP_SLAM
        );
        crate::combat_hooks::do_enemy_turns(combat); // Prep Slam -> Slam
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SB_SLAM
        );
        assert_eq!(combat.state.enemies[0].move_damage(), 38);
        crate::combat_hooks::do_enemy_turns(combat); // Slam -> Sticky
        assert_eq!(combat.state.player.hp, 162);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SB_STICKY
        );
        assert_eq!(
            combat.state.enemies[0].effect(crate::combat_types::mfx::SLIMED),
            Some(5)
        );
        assert_eq!(
            combat.ai_rng.counter, 1,
            "Sticky, Prep Slam, and Slam all set the next move directly"
        );

        let mut split = RunEngine::new(42, 19);
        split.enter_specific_combat(vec!["SlimeBoss".to_string()]);
        let combat = split.combat_engine.as_mut().unwrap();
        combat.deal_damage_to_enemy(0, 75);
        assert_eq!(combat.state.enemies[0].entity.hp, 75);
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::SB_SPLIT
        );
        assert_eq!(
            combat.state.enemies[0].intent,
            crate::combat_types::Intent::Unknown,
            "SlimeBoss.damage assigns Java's UNKNOWN Split intent"
        );
        assert_eq!(
            combat.state.enemies.len(),
            1,
            "damage only interrupts the intent; Split has not taken its turn"
        );
        assert_eq!(combat.ai_rng.counter, 1);

        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_eq!(combat.state.enemies.len(), 3);
        assert_eq!(combat.state.enemies[1].id, "SpikeSlime_L");
        assert_eq!(combat.state.enemies[2].id, "AcidSlime_L");
        for child in &combat.state.enemies[1..] {
            assert_eq!((child.entity.hp, child.entity.max_hp), (75, 75));
        }
        assert_eq!(
            combat.state.enemies[1]
                .entity
                .status(crate::status_ids::sid::STARTING_DMG),
            18
        );
        assert_eq!(
            combat.state.enemies[1]
                .entity
                .status(crate::status_ids::sid::STR_AMT),
            3
        );
        assert_eq!(
            combat.state.enemies[2]
                .entity
                .status(crate::status_ids::sid::STARTING_DMG),
            12
        );
        assert_eq!(
            combat.state.enemies[2]
                .entity
                .status(crate::status_ids::sid::STR_AMT),
            18
        );
        assert_eq!(
            combat.state.enemies[1]
                .entity
                .status(crate::status_ids::sid::BLOCK_AMT),
            17
        );
        assert_eq!(
            combat.state.enemies[2]
                .entity
                .status(crate::status_ids::sid::BLOCK_AMT),
            17
        );
        assert_eq!(
            combat.ai_rng.counter, 3,
            "the spawned Spike then Acid each consume one opening aiRng value"
        );

        let mut lethal = RunEngine::new(42, 0);
        lethal.enter_specific_combat(vec!["SlimeBoss".to_string()]);
        let combat = lethal.combat_engine.as_mut().unwrap();
        combat.state.enemies[0].entity.hp = 10;
        combat.deal_damage_to_enemy(0, 10);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        assert_eq!(
            combat.state.enemies.len(),
            1,
            "SlimeBoss.damage excludes lethal hits with !isDying"
        );
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
            assert_eq!(
                combat.ai_rng.counter, 2,
                "rollMove and getMove.randomBoolean each consume one aiRng value"
            );
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
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::APOLOGY_DEBUFF
        );
        assert_eq!(
            combat.state.enemies[0].effect(crate::combat_types::mfx::WEAK),
            Some(1)
        );
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.player.status(crate::status_ids::sid::WEAKENED),
            1
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::APOLOGY_TACKLE
        );
        assert_eq!(
            combat.ai_rng.counter, 2,
            "the direct alternating cycle consumes no later aiRng values"
        );

        let mut debuff = RunEngine::new(debuff_seed.expect("debuff opener"), 0);
        debuff.enter_specific_combat(vec!["Apology Slime".to_string()]);
        let combat = debuff.combat_engine.as_mut().unwrap();
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(
            combat.state.player.status(crate::status_ids::sid::WEAKENED),
            1
        );
        assert_eq!(
            combat.state.enemies[0].move_id,
            crate::enemies::move_ids::APOLOGY_TACKLE
        );
        assert_eq!(combat.ai_rng.counter, 2);
    }

    #[test]
    fn test_run_engine_reset() {
        let ambient_state = (0x1234_5678_9ABC_DEF0, 0x0FED_CBA9_8765_4321);
        let collections_state = 0x1234_5678_9ABC;
        let mut engine =
            RunEngine::new_with_ambient_states(42, 20, ambient_state, collections_state);
        engine.run_state.current_hp = 10;
        engine.reset(99);
        let fresh = RunEngine::new(99, 20);
        assert_eq!(engine.run_state.current_hp, 68);
        assert_eq!(engine.seed, 99);
        assert_eq!(engine.ambient_math_rng_state(), ambient_state);
        assert_eq!(engine.java_collections_rng_state(), collections_state);
        assert_eq!(engine.persistent_rngs, fresh.persistent_rngs);
        assert_eq!(engine.floor_rngs, fresh.floor_rngs);
        assert_eq!(engine.map_rng, fresh.map_rng);
        assert_eq!(engine.neow_rng, fresh.neow_rng);
    }

    #[test]
    fn test_map_choice_actions() {
        let mut engine = RunEngine::new(42, 20);
        resolve_opening_neow(&mut engine);
        let actions = engine.get_legal_actions();
        assert!(!actions.is_empty(), "Should have map choice actions");
        for a in &actions {
            assert!(matches!(a, GameAction::ChoosePath(_)));
        }
    }

    #[test]
    fn test_first_floor_is_combat() {
        let mut engine = RunEngine::new(42, 20);
        resolve_opening_neow(&mut engine);
        let actions = engine.get_legal_actions();
        assert!(!actions.is_empty());

        // Take first path
        let outcome = engine.step_game(&actions[0]);
        assert!(outcome.accepted());
        assert!(!outcome.is_terminal());
        assert_eq!(engine.phase, RunPhase::Combat);
        assert_eq!(engine.run_state.floor, 1);
    }

    #[test]
    fn legal_action_rollout_reaches_a_terminal_state_before_the_step_cap() {
        // A deterministic legal-action rollout may win or die, but it must not
        // stall on an unexpressed decision or empty nonterminal action set.
        let mut engine = RunEngine::new(42, 20);
        let mut rng = crate::seed::StsRandom::new(42);
        let mut steps = 0;
        let max_steps = 50_000;

        while !engine.is_done() && steps < max_steps {
            let actions = engine.get_legal_actions();
            if actions.is_empty() {
                break;
            }
            let idx = rng.random_index(actions.len());
            engine.step_game(&actions[idx]);
            steps += 1;
        }

        assert!(
            engine.is_done(),
            "legal-action rollout stalled before a terminal state. Steps: {}, Floor: {}, Phase: {:?}",
            steps,
            engine.run_state.floor,
            engine.current_phase()
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
        assert!(actions.contains(&GameAction::CampfireRest));

        engine.step_game(&GameAction::CampfireRest);
        // Source: CampfireSleepEffect.java casts maxHealth * 0.3f to int,
        // so 72 max HP heals 21 rather than rounding 21.6 up to 22.
        assert_eq!(engine.run_state.current_hp, 71);
    }

    #[test]
    fn test_campfire_upgrade() {
        let mut engine = RunEngine::new(42, 0);
        engine.phase = RunPhase::Campfire;
        engine.run_state.deck = vec!["Strike".to_string(), "Eruption".to_string()];

        engine.step_game(&GameAction::CampfireUpgrade(1));
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
                .contains(&GameAction::CampfireUpgrade(0)));
            engine.step_game(&GameAction::CampfireUpgrade(0));
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
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);

        engine.enter_campfire();
        assert_eq!(
            engine.run_state.relic_flags.counters[crate::relic_flags::counter::ANCIENT_TEA_SET],
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
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);
        engine.current_event = crate::events::typed_events_for_act(2)
            .into_iter()
            .find(|event| event.name == "Forgotten Altar");
        engine.phase = RunPhase::Event;

        assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
        assert!(!engine
            .run_state
            .relics
            .iter()
            .any(|relic| relic == "Golden Idol"));
        assert!(engine
            .step_game(&GameAction::SelectRewardItem(0))
            .accepted());
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
        engine
            .run_state
            .relic_flags
            .set(crate::relic_flags::flag::ECTOPLASM);
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
        engine.run_state.reconcile_deck_card_states();
        engine.run_state.bottled_flame_card_instance_id =
            Some(engine.run_state.deck_card_states[9].instance_id);

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
        engine
            .run_state
            .relics
            .push("Bottled Lightning".to_string());
        engine.run_state.reconcile_deck_card_states();
        engine.run_state.bottled_lightning_card_instance_id =
            Some(engine.run_state.deck_card_states[9].instance_id);

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
        engine.run_state.reconcile_deck_card_states();
        engine.run_state.bottled_tornado_card_instance_id =
            Some(engine.run_state.deck_card_states[9].instance_id);

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
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);

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
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);

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
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);

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
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);

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
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);

        engine.enter_specific_combat(vec!["JawWorm".to_string()]);
        let combat = engine.combat_engine.as_ref().expect("combat should start");
        assert_eq!(combat.state.max_energy, 4);
        assert_eq!(combat.state.energy, 4);
    }

    #[test]
    fn mark_of_pain_increases_master_energy_and_adds_two_wounds() {
        // Source-derived (verify relic/Mark of Pain): MarkOfPain.java::onEquip
        // increments energyMaster once and atBattleStart creates two Wounds.
        let mut engine = RunEngine::new(46, 0);
        engine.run_state.relics.push("Mark of Pain".to_string());
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);

        engine.enter_specific_combat(vec!["JawWorm".to_string()]);
        let combat = engine.combat_engine.as_ref().expect("combat should start");
        assert_eq!(combat.state.max_energy, 4);
        assert_eq!(combat.state.energy, 4);
        assert_eq!(
            combat
                .state
                .draw_pile
                .iter()
                .chain(&combat.state.hand)
                .filter(|card| combat.card_registry.card_name(card.def_id) == "Wound")
                .count(),
            2
        );
    }

    #[test]
    fn du_vu_doll_counts_actual_master_deck_curses_for_combat_strength() {
        // Source-derived (verify relic/Du-Vu Doll): DuVuDoll.java counts every
        // CURSE-typed master-deck card, then grants that count at battle start.
        let mut engine = RunEngine::new(47, 20);
        engine.run_state.relics.push("Du-Vu Doll".to_string());
        engine
            .run_state
            .deck
            .extend(["Regret".to_string(), "CurseOfTheBell".to_string()]);

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

        engine.step_game(&GameAction::SelectRewardItem(0));
        assert_eq!(
            engine
                .current_reward_screen()
                .as_ref()
                .and_then(|screen| screen.active_item),
            Some(0)
        );
        assert!(engine.get_legal_actions().iter().all(|action| matches!(
            action,
            GameAction::ChooseRewardOption { item_index: 0, .. } | GameAction::SkipRewardItem(0)
        )));
        engine.step_game(&GameAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 1,
        });
        assert_eq!(engine.run_state.deck.len(), deck_before + 1);
        assert_eq!(engine.run_state.deck.last().unwrap(), "Vigilance");
    }

    #[test]
    fn test_card_reward_skip() {
        let mut engine = RunEngine::new(42, 0);
        engine.debug_set_card_reward_screen(vec!["Eruption".to_string(), "Vigilance".to_string()]);
        let deck_before = engine.run_state.deck.len();

        engine.step_game(&GameAction::SkipRewardItem(0));
        assert_eq!(engine.run_state.deck.len(), deck_before);
    }

    #[test]
    fn test_event_choices() {
        let mut engine = RunEngine::new(42, 0);
        engine.enter_event();
        assert_eq!(engine.phase, RunPhase::Event);

        let actions = engine.get_legal_actions();
        assert!(!actions.is_empty());
        assert!(actions
            .iter()
            .all(|a| matches!(a, GameAction::EventChoice(_))));
    }

    #[test]
    fn test_shop_mechanics() {
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.gold = 500;
        engine.enter_shop();
        assert_eq!(engine.phase, RunPhase::Shop);

        let actions = engine.get_legal_actions();
        assert!(actions.contains(&GameAction::ShopLeave));

        // Leave shop
        engine.step_game(&GameAction::ShopLeave);
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
            !engine_low
                .run_state
                .deck
                .contains(&"AscendersBane".to_string()),
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

        assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
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
        assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
        assert_eq!(engine.run_state.deck.len(), deck_before + 1);
        assert_eq!(
            engine.run_state.deck.last().map(String::as_str),
            Some("Injury")
        );

        let mut damage = RunEngine::new(43, 0);
        damage.run_state.max_hp = 72;
        damage.run_state.current_hp = 72;
        damage.debug_set_typed_event_state(event.clone());
        damage.step_game(&GameAction::EventChoice(0));
        damage.step_game(&GameAction::EventChoice(1));
        assert_eq!(damage.run_state.current_hp, 54);

        let mut asc15 = RunEngine::new(44, 15);
        asc15.run_state.max_hp = 100;
        asc15.run_state.current_hp = 100;
        asc15.debug_set_typed_event_state(event.clone());
        asc15.step_game(&GameAction::EventChoice(0));
        asc15.step_game(&GameAction::EventChoice(2));
        assert_eq!(asc15.run_state.max_hp, 90);
        assert_eq!(asc15.run_state.current_hp, 90);

        let mut duplicate = RunEngine::new(45, 0);
        duplicate.run_state.relics.push("Golden Idol".to_string());
        duplicate
            .run_state
            .relic_flags
            .rebuild(&duplicate.run_state.relics);
        duplicate.debug_set_typed_event_state(event);
        duplicate.step_game(&GameAction::EventChoice(0));
        assert_eq!(
            duplicate
                .run_state
                .relics
                .iter()
                .filter(|relic| relic.as_str() == "Golden Idol")
                .count(),
            1
        );
        assert!(duplicate
            .run_state
            .relics
            .iter()
            .any(|relic| relic == "Circlet"));
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
                engine
                    .run_state
                    .relic_flags
                    .rebuild(&engine.run_state.relics);
            }
            let gold_before = engine.run_state.gold;
            engine.debug_enter_specific_combat(&["Cultist"]);
            engine.debug_force_current_combat_outcome(true);
            engine.debug_resolve_current_combat_outcome();
            engine.step_game(&GameAction::SelectRewardItem(0));
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
    fn boss_gold_roll_is_one_misc_tick_even_when_the_value_is_hidden() {
        // Act 3/4 suppress the reward item only after AbstractRoom constructs
        // it. This focused primitive proof complements the Act 3 engine-path
        // test, whose transition immediately replaces the floor RNG stream.
        // Re-verified: AbstractRoom.java:286-331.
        let mut engine = RunEngine::new(44, 0);
        let mut oracle = engine.floor_rngs.misc.clone();
        let expected = 100 + oracle.random_int_range(-5, 5);
        assert_eq!(engine.roll_boss_gold_reward(), expected);
        assert_eq!(engine.floor_rngs.misc, oracle);
    }

    #[test]
    fn chest_type_and_reward_share_java_treasure_stream_order() {
        // Re-verified: chest type is 50/33/17 from one inclusive draw; the
        // constructor's second draw jointly chooses optional gold and tier.
        // Java: AbstractDungeon.java:489-498 and rewards/chests/*.java.
        let mut saw = [false; 3];
        for seed in 0..512 {
            let mut engine = RunEngine::new(44, 0);
            engine.persistent_rngs.treasure = crate::seed::StsRandom::new(seed);
            let mut oracle = engine.persistent_rngs.treasure.clone();
            let size_roll = oracle.random_int(99);
            let reward_roll = oracle.random_int(99);
            let expected = if size_roll < 50 {
                saw[0] = true;
                GeneratedChest {
                    size: ChestSize::Small,
                    gold_reward: reward_roll < 50,
                    relic_tier: if reward_roll < 75 {
                        RelicTier::Common
                    } else {
                        RelicTier::Uncommon
                    },
                }
            } else if size_roll < 83 {
                saw[1] = true;
                GeneratedChest {
                    size: ChestSize::Medium,
                    gold_reward: reward_roll < 35,
                    relic_tier: if reward_roll < 35 {
                        RelicTier::Common
                    } else if reward_roll < 85 {
                        RelicTier::Uncommon
                    } else {
                        RelicTier::Rare
                    },
                }
            } else {
                saw[2] = true;
                GeneratedChest {
                    size: ChestSize::Large,
                    gold_reward: reward_roll < 50,
                    relic_tier: if reward_roll < 75 {
                        RelicTier::Uncommon
                    } else {
                        RelicTier::Rare
                    },
                }
            };
            assert_eq!(engine.generate_chest(), expected);
            assert_eq!(engine.persistent_rngs.treasure, oracle);
        }
        assert_eq!(saw, [true, true, true]);
    }

    #[test]
    fn cursed_key_uses_java_hashmap_curse_order_on_the_card_stream() {
        // Re-verified: CardLibrary.getCurse iterates the Java 8 curses HashMap,
        // excludes four non-random curses, then indexes this exact ten-card
        // order with cardRng.random(0, 9).
        // Java: CardLibrary.java:1012-1019, CursedKey.java:47.
        for (seed, expected) in [(0, "Shame"), (2, "Clumsy")] {
            let mut engine = RunEngine::new(44, 0);
            engine.run_state.relics.push("Cursed Key".to_string());
            engine
                .run_state
                .relic_flags
                .rebuild(&engine.run_state.relics);
            engine.persistent_rngs.card = crate::seed::StsRandom::new(seed);
            let before = engine.run_state.deck.len();

            engine.build_treasure_reward_screen();

            assert_eq!(engine.run_state.deck.len(), before + 1);
            assert_eq!(
                engine.run_state.deck.last().map(String::as_str),
                Some(expected)
            );
            assert_eq!(engine.persistent_rngs.card.counter, 1);
        }
    }

    #[test]
    fn neow_three_potions_use_initialized_watcher_order() {
        // NeowReward.THREE_SMALL_POTIONS calls PotionHelper.getRandomPotion
        // three times. PotionHelper indexes this exact initialized Watcher
        // order with potionRng and performs no rarity roll.
        // Java: PotionHelper.java::getPotions, NeowReward.java::activate.
        assert_eq!(
            WATCHER_POTIONS
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>(),
            vec![
                "BottledMiracle",
                "StancePotion",
                "Ambrosia",
                "Block Potion",
                "Dexterity Potion",
                "Energy Potion",
                "Explosive Potion",
                "Fire Potion",
                "Strength Potion",
                "Swift Potion",
                "Weak Potion",
                "FearPotion",
                "AttackPotion",
                "SkillPotion",
                "PowerPotion",
                "ColorlessPotion",
                "SteroidPotion",
                "SpeedPotion",
                "BlessingOfTheForge",
                "Regen Potion",
                "Ancient Potion",
                "LiquidBronze",
                "GamblersBrew",
                "EssenceOfSteel",
                "DuplicationPotion",
                "DistilledChaos",
                "LiquidMemories",
                "CultistPotion",
                "Fruit Juice",
                "SneckoOil",
                "FairyPotion",
                "SmokeBomb",
                "EntropicBrew",
            ]
        );

        let mut engine = RunEngine::new(44, 0);
        engine.persistent_rngs.potion = crate::seed::StsRandom::new((-97i64) as u64);
        let mut oracle = engine.persistent_rngs.potion.clone();
        let expected = (0..3)
            .map(|_| {
                WATCHER_POTIONS[oracle.random_index(WATCHER_POTIONS.len())]
                    .0
                    .to_string()
            })
            .collect::<Vec<_>>();

        engine.build_neow_potion_reward_screen();

        let actual = engine
            .reward_screen
            .as_ref()
            .expect("Neow potion rewards")
            .items
            .iter()
            .map(|item| item.label.clone())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
        assert_eq!(engine.persistent_rngs.potion, oracle);
    }

    #[test]
    fn match_and_keep_a0_uses_java_pool_and_stream_contract() {
        // GremlinMatchGame selects rare/uncommon/common/curse on cardRng,
        // shuffles the complete colorless pool on shuffleRng, then shuffles
        // the duplicated 12-card board from miscRng.randomLong().
        // Java: GremlinMatchGame.java::initializeCards,
        // AbstractDungeon.java::returnColorlessCard/returnRandomCurse.
        assert_eq!(COLORLESS_CARD_POOL.len(), 35);
        assert_eq!(
            &COLORLESS_CARD_POOL[7..11],
            &["Blind", "Bandage Up", "Secret Technique", "Deep Breath"]
        );

        let mut engine = RunEngine::new(44, 0);
        engine.persistent_rngs.card = crate::seed::StsRandom::new(101);
        engine.floor_rngs.shuffle = crate::seed::StsRandom::new(202);
        engine.floor_rngs.misc = crate::seed::StsRandom::new(303);
        let mut card_oracle = engine.persistent_rngs.card.clone();
        let mut shuffle_oracle = engine.floor_rngs.shuffle.clone();
        let mut misc_oracle = engine.floor_rngs.misc.clone();
        let mut colorless_oracle = engine.card_pools.colorless.clone();

        let mut unique = vec![
            engine.card_pools.rare[card_oracle.random_index(engine.card_pools.rare.len())].clone(),
            engine.card_pools.uncommon[card_oracle.random_index(engine.card_pools.uncommon.len())]
                .clone(),
            engine.card_pools.common[card_oracle.random_index(engine.card_pools.common.len())]
                .clone(),
        ];
        let colorless_seed = shuffle_oracle.random_long_unbounded();
        crate::seed::java_util_shuffle(&mut colorless_oracle, colorless_seed);
        unique.push(
            colorless_oracle
                .iter()
                .find(|card| event_card_rarity(card) == Some(EventCardRarity::Uncommon))
                .expect("colorless uncommon")
                .clone(),
        );
        unique.push(
            RANDOM_OBTAINABLE_CURSES[card_oracle.random_index(RANDOM_OBTAINABLE_CURSES.len())]
                .to_string(),
        );
        unique.push("Eruption".to_string());
        let mut expected = unique.clone();
        expected.extend(unique);
        let board_seed = misc_oracle.random_long_unbounded();
        crate::seed::java_util_shuffle(&mut expected, board_seed);

        let board = engine.build_match_and_keep_board();

        assert_eq!(board, expected);
        assert_eq!(engine.card_pools.colorless, colorless_oracle);
        assert_eq!(engine.persistent_rngs.card, card_oracle);
        assert_eq!(engine.floor_rngs.shuffle, shuffle_oracle);
        assert_eq!(engine.floor_rngs.misc, misc_oracle);
    }

    #[test]
    fn tiny_house_uses_misc_for_shuffle_and_potion_even_without_upgrade_targets() {
        // TinyHouse.onEquip always seeds Collections.shuffle from one
        // miscRng.randomLong(), even for an empty eligible list, then indexes
        // PotionHelper with one further miscRng draw. potionRng is untouched.
        // Java: relics/TinyHouse.java::onEquip.
        let mut engine = RunEngine::new(44, 0);
        engine.run_state.deck = vec!["Eruption+".to_string(), "Vigilance+".to_string()];
        engine.floor_rngs.misc = crate::seed::StsRandom::new(73);
        let potion_before = engine.persistent_rngs.potion.clone();
        let mut oracle = engine.floor_rngs.misc.clone();
        let shuffle_seed = oracle.random_long_unbounded();
        let mut empty: Vec<usize> = Vec::new();
        crate::seed::java_util_shuffle(&mut empty, shuffle_seed);
        let expected_potion = WATCHER_POTIONS[oracle.random_index(WATCHER_POTIONS.len())].0;

        engine.add_relic_reward("Tiny House");
        engine.build_tiny_house_reward_screen();

        let screen = engine.reward_screen.as_ref().expect("Tiny House rewards");
        assert_eq!(screen.items[1].label, expected_potion);
        assert_eq!(engine.floor_rngs.misc, oracle);
        assert_eq!(engine.persistent_rngs.potion, potion_before);
        assert_eq!(engine.run_state.deck, vec!["Eruption+", "Vigilance+"]);
    }

    #[test]
    fn fixed_tier_relic_routes_pop_persistent_pools_without_identity_draws() {
        // Neow, Calling Bell, and fixed-tier event rewards delegate directly
        // to returnRandomRelic for a known tier. Identity is the front of the
        // persistent shuffled pool; no fresh relicRng draw occurs.
        // Java: AbstractDungeon.java::returnRandomRelicKey,
        // NeowReward.java::activate, CallingBell.java::onEquip.
        let mut engine = RunEngine::new(44, 0);
        engine.relic_pools.common = vec!["Anchor".to_string(), "Boot".to_string()];
        engine.relic_pools.uncommon = vec!["Sundial".to_string(), "Kunai".to_string()];
        engine.relic_pools.rare = vec!["Ginger".to_string(), "Mango".to_string()];
        let relic_rng = engine.persistent_rngs.relic.clone();

        assert_eq!(engine.roll_neow_common_relic(), "Anchor");
        assert_eq!(engine.roll_neow_rare_relic(), "Ginger");
        assert_eq!(engine.roll_uncommon_event_relic_id(), "Sundial");
        assert_eq!(engine.persistent_rngs.relic, relic_rng);

        engine.build_calling_bell_reward_screen();
        let screen = engine.reward_screen.as_ref().expect("Calling Bell rewards");
        assert_eq!(screen.items[1].label, "Boot");
        assert_eq!(screen.items[2].label, "Kunai");
        assert_eq!(screen.items[3].label, "Mango");
        assert_eq!(engine.persistent_rngs.relic, relic_rng);
        assert!(engine.relic_pools.common.is_empty());
        assert!(engine.relic_pools.uncommon.is_empty());
        assert!(engine.relic_pools.rare.is_empty());
    }

    #[test]
    fn singleton_event_fallbacks_still_consume_their_java_misc_draws() {
        // FaceTrader always shuffles its candidate list after inserting a
        // Circlet fallback. CursedTome always calls random(size - 1) after
        // inserting the same fallback, including random(0).
        // Java: FaceTrader.java::getRandomFace,
        // CursedTome.java::randomBook.
        let mut face = RunEngine::new(44, 0);
        face.run_state.relics.extend([
            "CultistMask".to_string(),
            "FaceOfCleric".to_string(),
            "GremlinMask".to_string(),
            "NlothsMask".to_string(),
            "SsserpentHead".to_string(),
        ]);
        face.floor_rngs.misc = crate::seed::StsRandom::new(19);
        let mut face_oracle = face.floor_rngs.misc.clone();
        let seed = face_oracle.random_long_unbounded();
        let mut fallback = vec!["Circlet"];
        crate::seed::java_util_shuffle(&mut fallback, seed);
        let mut rewards = Vec::new();
        face.apply_event_program_op(&EventProgramOp::ObtainRandomFace, &mut rewards);
        assert_eq!(face.floor_rngs.misc, face_oracle);
        assert_eq!(
            face.run_state.relics.last().map(String::as_str),
            Some("Circlet")
        );

        let mut tome = RunEngine::new(44, 0);
        tome.run_state.relics.extend([
            "Necronomicon".to_string(),
            "Enchiridion".to_string(),
            "Nilry's Codex".to_string(),
        ]);
        tome.floor_rngs.misc = crate::seed::StsRandom::new(23);
        let mut tome_oracle = tome.floor_rngs.misc.clone();
        assert_eq!(tome_oracle.random_int(0), 0);
        assert_eq!(tome.roll_cursed_tome_book_id(), "Circlet");
        assert_eq!(tome.floor_rngs.misc, tome_oracle);
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
        let has_remove = actions
            .iter()
            .any(|a| matches!(a, GameAction::ShopRemoveCard(_)));
        assert!(has_remove, "Should offer card removal before first use");

        // Remove a card
        engine.step_game(&GameAction::ShopRemoveCard(0));
        assert_eq!(engine.run_state.deck.len(), 9);

        // After removal, should NOT have removal options
        let actions_after = engine.get_legal_actions();
        let has_remove_after = actions_after
            .iter()
            .any(|a| matches!(a, GameAction::ShopRemoveCard(_)));
        assert!(
            !has_remove_after,
            "Should not offer card removal after first use"
        );
    }

    #[test]
    fn pandora_box_replaces_every_starter_tagged_card_one_for_one() {
        // PandorasBox.java removes STARTER_STRIKE/STARTER_DEFEND cards, rolls
        // one returnTrulyRandomCard per removal, previews relic callbacks, and
        // obtains all replacements through FastCardObtainEffect.
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.deck = vec![
            "Strike_P".to_string(),
            "Strike_P+".to_string(),
            "Defend_P".to_string(),
            "Defend_P+".to_string(),
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
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);
        engine.add_relic_reward("Pandora's Box");
        assert_eq!(engine.run_state.deck.len(), 7);
        assert_eq!(
            &engine.run_state.deck[..3],
            &["Perfected Strike", "Eruption", "Vigilance"]
        );
        assert!(engine
            .run_state
            .deck
            .iter()
            .all(|card| { !matches!(card.as_str(), "Strike" | "Strike+" | "Defend" | "Defend+") }));
        assert!(engine.run_state.deck[3..]
            .iter()
            .all(|card| card.ends_with('+')));
        assert!(engine.run_state.deck[3..].iter().all(|card| {
            let base = card.trim_end_matches('+');
            WATCHER_COMMON_CARDS.contains(&base)
                || WATCHER_UNCOMMON_CARDS.contains(&base)
                || WATCHER_RARE_CARDS.contains(&base)
        }));
        assert_eq!(engine.run_state.gold, 136);
        assert_eq!(
            engine.run_state.deck.len(),
            engine.run_state.deck_card_states.len(),
        );
        let checkpoint = crate::checkpoint::CoreCheckpoint::capture(&engine)
            .expect("Pandora's Box result must be checkpoint-safe");
        checkpoint
            .restore()
            .expect("Pandora's Box checkpoint must restore");
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
    fn necronomicurse_is_not_exposed_to_normal_master_deck_removal() {
        // CardGroup.getPurgeableCards excludes Necronomicurse, so no player
        // removal screen can reach onRemoveFromMasterDeck. Necronomicon's
        // onUnequip removes directly from group and intentionally bypasses it.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java:967-973
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Necronomicon.java:43-52
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.deck = vec!["Necronomicurse".to_string()];
        engine.run_state.reconcile_deck_card_states();

        assert_eq!(engine.remove_master_deck_card(0), None);
        assert_eq!(engine.run_state.deck, vec!["Necronomicurse"]);
    }
}
