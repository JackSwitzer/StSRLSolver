//! Observation encoding — matches Python's 480-dim RunStateEncoder output.
//!
//! Layout (from state_encoders.py):
//!   [0..6]     HP/resources (6 dims)
//!   [6..9]     Keys (3 dims)
//!   [9..25]    Deck functional aggregate (16 dims)
//!   [25..206]  Relic binary flags (181 dims)
//!   [206..226] Potion slots (5 x 4 = 20 dims)
//!   [226..247] Map lookahead (3 x 7 = 21 dims)
//!   [247..251] Progress features (4 dims)
//!   [251..254] HP deficit + floor type flags (3 dims)
//!   [254..260] Decision phase type (6 dims one-hot)
//!   [260..480] Action encoding (10 x 22 = 220 dims)

use crate::cards::{CardRegistry, CardType, CardTarget};
use crate::run::{RunEngine, RunPhase, RunAction};
use crate::status_ids::sid;

pub const RUN_DIM: usize = 480;
pub const STATE_DIM: usize = 260;
pub const ACTION_DIM: usize = 220;
pub const ACTION_SLOTS: usize = 10;
pub const ACTION_FEAT_DIM: usize = 22;

// Relic catalog — sorted list matching Python's sorted(ALL_RELICS.keys())
// Generated from: sorted(ALL_RELICS.keys()) in packages/engine/content/relics.py
const N_RELICS: usize = 181;

const RELIC_CATALOG: [&str; N_RELICS] = [
    "Akabeko", "Anchor", "Ancient Tea Set", "Art of War", "Astrolabe",
    "Bag of Marbles", "Bag of Preparation", "Bird Faced Urn", "Black Blood", "Black Star",
    "Blood Vial", "Bloody Idol", "Blue Candle", "Boot", "Bottled Flame",
    "Bottled Lightning", "Bottled Tornado", "Brimstone", "Bronze Scales", "Burning Blood",
    "Busted Crown", "Cables", "Calipers", "Calling Bell", "CaptainsWheel",
    "Cauldron", "Centennial Puzzle", "CeramicFish", "Champion Belt", "Charon's Ashes",
    "Chemical X", "Circlet", "CloakClasp", "ClockworkSouvenir", "Coffee Dripper",
    "Cracked Core", "CultistMask", "Cursed Key", "Damaru", "Darkstone Periapt",
    "DataDisk", "Dead Branch", "Discerning Monocle", "DollysMirror", "Dream Catcher",
    "Du-Vu Doll", "Ectoplasm", "Emotion Chip", "Empty Cage", "Enchiridion",
    "Eternal Feather", "FaceOfCleric", "FossilizedHelix", "Frozen Egg 2", "Frozen Eye",
    "FrozenCore", "Fusion Hammer", "Gambling Chip", "Ginger", "Girya",
    "Golden Idol", "GoldenEye", "Gremlin Horn", "GremlinMask", "HandDrill",
    "Happy Flower", "HolyWater", "HornCleat", "HoveringKite", "Ice Cream",
    "Incense Burner", "InkBottle", "Inserter", "Juzu Bracelet", "Kunai",
    "Lantern", "Lee's Waffle", "Letter Opener", "Lizard Tail", "Magic Flower",
    "Mango", "Mark of Pain", "Mark of the Bloom", "Matryoshka", "MawBank",
    "MealTicket", "Meat on the Bone", "Medical Kit", "Melange", "Membership Card",
    "Mercury Hourglass", "Molten Egg 2", "Mummified Hand", "MutagenicStrength", "Necronomicon",
    "NeowsBlessing", "Nilry's Codex", "Ninja Scroll", "Nloth's Gift", "NlothsMask",
    "Nuclear Battery", "Nunchaku", "Odd Mushroom", "Oddly Smooth Stone", "Old Coin",
    "Omamori", "OrangePellets", "Orichalcum", "Ornamental Fan", "Orrery",
    "Pandora's Box", "Pantograph", "Paper Crane", "Paper Frog", "Peace Pipe",
    "Pear", "Pen Nib", "Philosopher's Stone", "Pocketwatch", "Potion Belt",
    "Prayer Wheel", "PreservedInsect", "PrismaticShard", "PureWater", "Question Card",
    "Red Circlet", "Red Mask", "Red Skull", "Regal Pillow", "Ring of the Serpent",
    "Ring of the Snake", "Runic Capacitor", "Runic Cube", "Runic Dome", "Runic Pyramid",
    "SacredBark", "Self Forming Clay", "Shovel", "Shuriken", "Singing Bowl",
    "SlaversCollar", "Sling", "Smiling Mask", "Snake Skull", "Snecko Eye",
    "Sozu", "Spirit Poop", "SsserpentHead", "StoneCalendar", "Strange Spoon",
    "Strawberry", "StrikeDummy", "Sundial", "Symbiotic Virus", "TeardropLocket",
    "The Courier", "The Specimen", "TheAbacus", "Thread and Needle", "Tingsha",
    "Tiny Chest", "Tiny House", "Toolbox", "Torii", "Tough Bandages",
    "Toxic Egg 2", "Toy Ornithopter", "TungstenRod", "Turnip", "TwistedFunnel",
    "Unceasing Top", "Vajra", "Velvet Choker", "VioletLotus", "War Paint",
    "WarpedTongs", "Whetstone", "White Beast Statue", "WingedGreaves", "WristBlade",
    "Yang",
];

/// Boss ID mapping matching Python's _BOSS_ID_MAP.
fn boss_id_index(name: &str) -> i32 {
    match name {
        "The Guardian" | "TheGuardian" => 0,
        "Hexaghost" => 1,
        "Slime Boss" | "SlimeBoss" => 2,
        "Automaton" => 3,
        "Collector" => 4,
        "Champ" => 5,
        "Awakened One" => 6,
        "Time Eater" => 7,
        "Donu and Deca" => 8,
        "Corrupt Heart" => 9,
        _ => -1,
    }
}

/// Phase type index matching Python's PHASE_TYPE_MAP.
fn phase_type_index(phase: RunPhase) -> usize {
    match phase {
        RunPhase::MapChoice => 0,        // "path"
        RunPhase::CardReward => 1,       // "card_pick"
        RunPhase::Campfire => 2,         // "rest"
        RunPhase::Shop => 3,             // "shop"
        RunPhase::Event => 4,            // "event"
        RunPhase::Combat | RunPhase::GameOver => 5, // "other"
    }
}

/// Room type index for path action encoding.
fn room_type_action_index(room_type: &str) -> Option<usize> {
    match room_type {
        "monster" => Some(4),
        "elite" => Some(5),
        "rest" => Some(6),
        "shop" => Some(7),
        "event" => Some(8),
        "treasure" => Some(9),
        "boss" => Some(10),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Card effect vector (18 dims) — simplified version matching Python
// ---------------------------------------------------------------------------

/// Encode a card's static data into an 18-dim effect vector.
/// Matches Python's _card_effect_vector() layout.
fn card_effect_vector(card_id: &str, registry: &CardRegistry) -> [f32; 18] {
    let mut v = [0.0f32; 18];
    let card = registry.get_or_default(card_id);
    card_effect_vector_from_def(&card, card_id, &mut v);
    v
}

/// Encode a card's static data into an 18-dim effect vector from a CardInstance.
fn card_effect_vector_inst(card_inst: &crate::combat_types::CardInstance, registry: &CardRegistry) -> [f32; 18] {
    let mut v = [0.0f32; 18];
    let card_id = registry.card_name(card_inst.def_id);
    let card = registry.card_def_by_id(card_inst.def_id);
    card_effect_vector_from_def(card, card_id, &mut v);
    v
}

fn card_effect_vector_from_def(card: &crate::cards::CardDef, _card_id: &str, v: &mut [f32; 18]) {

    // [0] energy cost normalized
    v[0] = if card.cost == -1 {
        -1.0
    } else {
        card.cost as f32 / 4.0
    };

    // [1] base damage normalized
    if card.base_damage >= 0 {
        v[1] = card.base_damage as f32 / 40.0;
    }

    // [2] base block normalized
    if card.base_block >= 0 {
        v[2] = card.base_block as f32 / 30.0;
    }

    // [3] draw (from effects)
    if card.effects.contains(&"draw") && card.base_magic > 0 {
        v[3] = card.base_magic as f32 / 5.0;
    }

    // [4] discard
    // (simplified — not many watcher cards discard)

    // [5] aoe
    if card.target == CardTarget::AllEnemy {
        v[5] = 1.0;
    }

    // [6] exhaust
    v[6] = if card.exhaust { 1.0 } else { 0.0 };

    // [7] ethereal (simplified: status cards)
    if card.card_type == CardType::Status {
        v[7] = 1.0;
    }

    // [8-10] type one-hot
    match card.card_type {
        CardType::Attack => v[8] = 1.0,
        CardType::Skill => v[9] = 1.0,
        CardType::Power => v[10] = 1.0,
        _ => {}
    }

    // [11-14] stance
    if let Some(stance) = card.enter_stance {
        match stance {
            "Wrath" => v[11] = 1.0,
            "Calm" => v[12] = 1.0,
            "Divinity" => v[13] = 1.0,
            _ => {}
        }
    }
    // exit_stance not tracked in CardDef, skip v[14]

    // [15-17] power embedding (simplified)
    for effect in card.effects {
        let e = effect.to_lowercase();
        if e.contains("strength") || e.contains("rushdown") || e.contains("on_wrath") {
            v[15] = 1.0;
        }
        if e.contains("dexterity") || e.contains("mental") || e.contains("on_stance") {
            v[16] = 1.0;
        }
    }
    if card.card_type == CardType::Power && card.base_magic > 0 {
        v[17] = card.base_magic as f32 / 10.0;
    }
}

// ---------------------------------------------------------------------------
// Power index for combat encoding
// ---------------------------------------------------------------------------

use crate::ids::StatusId;

const POWER_STATUS_IDS: &[StatusId] = &[
    sid::STRENGTH, sid::DEXTERITY, sid::VULNERABLE, sid::WEAKENED, sid::FRAIL,
    sid::MENTAL_FORTRESS, sid::RUSHDOWN, sid::VIGOR, sid::MANTRA,
    sid::PLATED_ARMOR, sid::METALLICIZE, sid::THORNS, sid::RITUAL,
    sid::RETAIN_CARDS, sid::ARTIFACT, sid::INTANGIBLE, sid::BARRICADE,
    sid::RAGE, sid::ANGRY, sid::REGENERATION,
];

fn power_index(id: StatusId) -> Option<usize> {
    POWER_STATUS_IDS.iter().position(|&p| p == id)
}

// ---------------------------------------------------------------------------
// Run state encoding (260 dims)
// ---------------------------------------------------------------------------

/// Encode the run state portion of the observation (260 dims).
pub fn encode_run_state(engine: &RunEngine, obs: &mut [f32; RUN_DIM]) {
    let rs = &engine.run_state;
    let registry = CardRegistry::new();
    let mut off = 0;

    // --- HP/resources (6 dims) ---
    let max_hp = rs.max_hp.max(1) as f32;
    obs[off] = rs.current_hp as f32 / max_hp;
    obs[off + 1] = max_hp / 100.0;
    obs[off + 2] = rs.gold as f32 / 500.0;
    obs[off + 3] = rs.floor as f32 / 55.0;
    obs[off + 4] = rs.act as f32 / 3.0;
    obs[off + 5] = rs.ascension as f32 / 20.0;
    off += 6;

    // --- Keys (3 dims) ---
    obs[off] = if rs.has_ruby_key { 1.0 } else { 0.0 };
    obs[off + 1] = if rs.has_emerald_key { 1.0 } else { 0.0 };
    obs[off + 2] = if rs.has_sapphire_key { 1.0 } else { 0.0 };
    off += 3;

    // --- Deck functional aggregate (16 dims) ---
    let n_deck = rs.deck.len();
    if n_deck > 0 {
        let mut effect_sum = [0.0f32; 18];
        let mut n_attacks = 0.0f32;
        let mut n_skills = 0.0f32;
        let mut n_powers = 0.0f32;
        let mut n_upgraded = 0.0f32;

        for card_id in &rs.deck {
            let ev = card_effect_vector(card_id, &registry);
            for i in 0..18 {
                effect_sum[i] += ev[i];
            }
            n_attacks += ev[8];
            n_skills += ev[9];
            n_powers += ev[10];
            if card_id.ends_with('+') {
                n_upgraded += 1.0;
            }
        }

        let nd = n_deck as f32;
        // Average of first 8 dims
        for i in 0..8 {
            obs[off + i] = effect_sum[i] / nd;
        }
        // Deck composition
        obs[off + 8] = nd / 40.0;
        obs[off + 9] = n_attacks / nd;
        obs[off + 10] = n_skills / nd;
        obs[off + 11] = n_powers / nd;
        // Upgrade ratio + stance density
        obs[off + 12] = n_upgraded / nd;
        obs[off + 13] = effect_sum[11] / nd; // wrath density
        obs[off + 14] = effect_sum[12] / nd; // calm density
        obs[off + 15] = (effect_sum[13] + effect_sum[14]) / nd; // divinity/exit
    }
    off += 16;

    // --- Relic binary flags (181 dims) ---
    // Uses sorted catalog matching Python's sorted(ALL_RELICS.keys())
    for relic in &rs.relics {
        if let Some(idx) = relic_catalog_index(relic) {
            obs[off + idx] = 1.0;
        }
    }
    off += N_RELICS;

    // --- Potion slots (5 x 4 = 20 dims) ---
    for i in 0..5.min(rs.potions.len()) {
        let base = off + i * 4;
        let potion = &rs.potions[i];
        if !potion.is_empty() {
            obs[base] = 1.0; // has potion
            let pid = potion.to_lowercase();
            if pid.contains("fire") || pid.contains("explosive") || pid.contains("attack") || pid.contains("poison") {
                obs[base + 1] = 1.0; // damage
            }
            if pid.contains("fairy") || pid.contains("fruit") || pid.contains("blood") || pid.contains("regen") {
                obs[base + 2] = 1.0; // heal
            }
            if pid.contains("block") || pid.contains("ghost") || pid.contains("ancient") {
                obs[base + 3] = 1.0; // defensive
            }
        }
    }
    off += 20;

    // --- Map lookahead (3 x 7 = 21 dims) ---
    encode_map_lookahead(engine, obs, off);
    off += 21;

    // --- Progress features (4 dims) ---
    obs[off] = rs.combats_won as f32 / 20.0;
    obs[off + 1] = rs.elites_killed as f32 / 5.0;
    obs[off + 2] = rs.bosses_killed as f32 / 3.0;
    let boss_id = boss_id_index(engine.boss_name());
    obs[off + 3] = if boss_id >= 0 { (boss_id + 1) as f32 / 11.0 } else { 0.0 };
    off += 4;

    // --- HP deficit + floor type flags (3 dims) ---
    obs[off] = 1.0 - (rs.current_hp as f32 / max_hp);
    let rt = engine.current_room_type().to_lowercase();
    obs[off + 1] = if rt.contains("boss") { 1.0 } else { 0.0 };
    obs[off + 2] = if rt == "elite" { 1.0 } else { 0.0 };
    off += 3;

    // --- Phase type (6 dims one-hot) ---
    let phase_idx = phase_type_index(engine.current_phase());
    obs[off + phase_idx] = 1.0;
    // off += 6; // = 260
}

fn encode_map_lookahead(engine: &RunEngine, obs: &mut [f32; RUN_DIM], off: usize) {
    let rs = &engine.run_state;
    let room_type_map: [&str; 7] = ["monster", "elite", "rest", "shop", "event", "treasure", "boss"];

    let current_floor = if rs.map_y >= 0 { rs.map_y as usize } else { 0 };

    for row_i in 0..3 {
        let target_floor = current_floor + row_i + 1;
        let base = off + row_i * 7;

        if target_floor >= engine.map.height {
            continue;
        }

        let mut counts = [0.0f32; 7];
        let nodes = engine.map.get_nodes_at_floor(target_floor);
        for node in &nodes {
            let rt_str = node.room_type.as_str();
            for (rt_idx, &rt_name) in room_type_map.iter().enumerate() {
                if rt_str == rt_name {
                    counts[rt_idx] += 1.0;
                    break;
                }
            }
        }
        let total: f32 = counts.iter().sum();
        if total > 0.0 {
            for i in 0..7 {
                obs[base + i] = counts[i] / total;
            }
        }
    }
}

/// Look up relic index in the sorted catalog (matches Python's sorted(ALL_RELICS.keys())).
/// Returns None for unknown relics.
fn relic_catalog_index(relic: &str) -> Option<usize> {
    RELIC_CATALOG.iter().position(|&r| r == relic)
}

// ---------------------------------------------------------------------------
// Action encoding (220 dims, appended at offset 260)
// ---------------------------------------------------------------------------

/// Encode available actions into the observation vector.
pub fn encode_actions(engine: &RunEngine, actions: &[RunAction], obs: &mut [f32; RUN_DIM]) {
    let registry = CardRegistry::new();
    let off = STATE_DIM;
    let n = actions.len().min(ACTION_SLOTS);

    for i in 0..n {
        let base = off + i * ACTION_FEAT_DIM;
        let action = &actions[i];

        match engine.current_phase() {
            RunPhase::CardReward => {
                obs[base] = 1.0; // is_card_pick
                match action {
                    RunAction::PickCard(idx) => {
                        let rewards = engine.get_card_rewards();
                        if *idx < rewards.len() {
                            let ev = card_effect_vector(&rewards[*idx], &registry);
                            for j in 0..18 {
                                obs[base + 4 + j] = ev[j];
                            }
                        }
                    }
                    RunAction::SkipCardReward => {
                        obs[base + 3] = 1.0; // skip marker
                    }
                    _ => {}
                }
            }
            RunPhase::MapChoice => {
                obs[base + 1] = 1.0; // is_path
                if let RunAction::ChoosePath(idx) = action {
                    // Encode destination room type
                    let next_nodes = if engine.run_state.map_y < 0 {
                        engine.map.get_start_nodes()
                    } else {
                        let x = engine.run_state.map_x as usize;
                        let y = engine.run_state.map_y as usize;
                        engine.map.get_next_nodes(x, y)
                    };
                    if *idx < next_nodes.len() {
                        let rt = next_nodes[*idx].room_type.as_str();
                        if let Some(rt_idx) = room_type_action_index(rt) {
                            obs[base + rt_idx] = 1.0;
                        }
                    }
                }
            }
            RunPhase::Campfire => {
                obs[base + 2] = 1.0; // is_rest
                match action {
                    RunAction::CampfireRest => obs[base + 4] = 1.0,
                    RunAction::CampfireUpgrade(_) => obs[base + 5] = 1.0,
                    _ => {}
                }
            }
            RunPhase::Shop | RunPhase::Event => {
                obs[base + 3] = 1.0; // is_other
                obs[base + 4] = (i as f32 + 1.0) / n.max(1) as f32;
            }
            _ => {
                // Combat or other — action index
                if matches!(action, RunAction::SkipCardReward) {
                    obs[base + 3] = 0.5;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Combat state encoding (298 dims) — separate from run encoding
// ---------------------------------------------------------------------------

pub const COMBAT_DIM: usize = 298;

/// Encode combat state into a 298-dim vector matching Python's CombatStateEncoder.
pub fn encode_combat_state(engine: &RunEngine) -> [f32; COMBAT_DIM] {
    let mut obs = [0.0f32; COMBAT_DIM];
    let registry = CardRegistry::new();

    let combat = match engine.get_combat_engine() {
        Some(e) => e,
        None => return obs,
    };

    let state = &combat.state;
    let player = &state.player;
    let mut off = 0;

    // --- Energy/block/turn/stance (9 dims) ---
    obs[off] = state.energy as f32 / 4.0;
    obs[off + 1] = player.block as f32 / 50.0;
    obs[off + 2] = state.turn as f32 / 20.0;
    obs[off + 3] = state.hand.len() as f32 / 10.0;
    obs[off + 4] = state.draw_pile.len() as f32 / 30.0;
    obs[off + 5] = state.discard_pile.len() as f32 / 30.0;
    obs[off + 6] = state.exhaust_pile.len() as f32 / 20.0;
    // Stance encoding
    match state.stance {
        crate::state::Stance::Wrath => obs[off + 7] = 1.0,
        crate::state::Stance::Calm => obs[off + 7] = -1.0,
        crate::state::Stance::Divinity => obs[off + 8] = 1.0,
        _ => {}
    }
    off += 9;

    // --- Mantra (1 dim) ---
    obs[off] = state.mantra as f32 / 10.0;
    off += 1;

    // --- Active powers 20 x 2 (40 dims) ---
    for (i, &val) in player.statuses.iter().enumerate() {
        if val != 0 {
            let status_id = crate::ids::StatusId(i as u16);
            if let Some(idx) = power_index(status_id) {
                if idx < 20 {
                    let base = off + idx * 2;
                    obs[base] = 1.0;
                    obs[base + 1] = val as f32 / 10.0;
                }
            }
        }
    }
    off += 40;

    // --- Hand cards: 10 x 18 (180 dims) ---
    for i in 0..state.hand.len().min(10) {
        let ev = card_effect_vector_inst(&state.hand[i], &registry);
        let base = off + i * 18;
        for j in 0..18 {
            obs[base + j] = ev[j];
        }
    }
    off += 180;

    // --- Enemy features: 5 x 12 (60 dims) ---
    for i in 0..state.enemies.len().min(5) {
        let enemy = &state.enemies[i];
        let base = off + i * 12;
        let emax = enemy.entity.max_hp.max(1) as f32;
        obs[base] = enemy.entity.hp as f32 / emax;
        obs[base + 1] = emax / 300.0;
        obs[base + 2] = enemy.entity.block as f32 / 50.0;
        obs[base + 3] = enemy.move_damage as f32 / 40.0;
        obs[base + 4] = enemy.move_hits as f32 / 5.0;
        obs[base + 5] = if enemy.entity.hp > 0 { 1.0 } else { 0.0 };

        // Enemy statuses
        obs[base + 6] = enemy.entity.status(sid::VULNERABLE) as f32 / 5.0;
        obs[base + 7] = enemy.entity.status(sid::WEAKENED) as f32 / 5.0;
        obs[base + 8] = enemy.entity.status(sid::STRENGTH) as f32 / 10.0;
        obs[base + 9] = enemy.entity.status(sid::RITUAL) as f32 / 5.0;
        obs[base + 10] = enemy.entity.status(sid::ARTIFACT) as f32 / 3.0;
        obs[base + 11] = enemy.entity.status(sid::INTANGIBLE) as f32 / 3.0;
    }
    off += 60;

    // --- Draw pile summary (6 dims) ---
    let draw = &state.draw_pile;
    if !draw.is_empty() {
        let n_draw = draw.len() as f32;
        let mut draw_atk = 0.0f32;
        let mut draw_skl = 0.0f32;
        let mut draw_dmg = 0.0f32;
        let mut draw_blk = 0.0f32;
        let mut draw_stance = 0.0f32;

        for card_inst in draw {
            let ev = card_effect_vector_inst(card_inst, &registry);
            if ev[8] > 0.0 { draw_atk += 1.0; }
            if ev[9] > 0.0 { draw_skl += 1.0; }
            draw_dmg += ev[1];
            draw_blk += ev[2];
            if ev[11] > 0.0 || ev[12] > 0.0 || ev[13] > 0.0 || ev[14] > 0.0 {
                draw_stance += 1.0;
            }
        }

        obs[off] = n_draw / 30.0;
        obs[off + 1] = draw_atk / n_draw;
        obs[off + 2] = draw_skl / n_draw;
        obs[off + 3] = draw_dmg / n_draw;
        obs[off + 4] = draw_blk / n_draw;
        obs[off + 5] = draw_stance / n_draw;
    }
    off += 6;

    // --- Discard summary (2 dims) ---
    let discard = &state.discard_pile;
    obs[off] = discard.len() as f32 / 30.0;
    if !discard.is_empty() {
        let disc_dmg: f32 = discard.iter()
            .map(|c| card_effect_vector_inst(c, &registry)[1])
            .sum();
        obs[off + 1] = disc_dmg / discard.len().max(1) as f32;
    }
    // off += 2; // = 298

    obs
}

// ---------------------------------------------------------------------------
// Full observation (480 dims)
// ---------------------------------------------------------------------------

/// Get the full 480-dim observation vector for the current state.
pub fn get_observation(engine: &RunEngine) -> [f32; RUN_DIM] {
    let mut obs = [0.0f32; RUN_DIM];
    encode_run_state(engine, &mut obs);

    let actions = engine.get_legal_actions();
    encode_actions(engine, &actions, &mut obs);

    obs
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_dim() {
        let engine = RunEngine::new(42, 20);
        let obs = get_observation(&engine);
        assert_eq!(obs.len(), 480);
    }

    #[test]
    fn test_observation_not_all_zeros() {
        let engine = RunEngine::new(42, 20);
        let obs = get_observation(&engine);
        let nonzero = obs.iter().filter(|&&v| v != 0.0).count();
        assert!(nonzero > 10, "Obs should have many non-zero values, got {}", nonzero);
    }

    #[test]
    fn test_hp_encoding() {
        let engine = RunEngine::new(42, 20);
        let obs = get_observation(&engine);
        // HP ratio should be 1.0 at start (full health)
        assert!((obs[0] - 1.0).abs() < 0.01, "HP ratio should be ~1.0, got {}", obs[0]);
    }

    #[test]
    fn test_phase_encoding() {
        let engine = RunEngine::new(42, 20);
        let obs = get_observation(&engine);
        // Phase = MapChoice = index 0
        assert_eq!(obs[254], 1.0, "Phase dim 0 (path) should be 1.0");
        assert_eq!(obs[255], 0.0);
    }

    #[test]
    fn test_combat_encoding_dims() {
        let mut engine = RunEngine::new(42, 20);
        // Enter combat
        let actions = engine.get_legal_actions();
        engine.step(&actions[0]);
        assert_eq!(engine.current_phase(), RunPhase::Combat);

        let combat_obs = encode_combat_state(&engine);
        assert_eq!(combat_obs.len(), COMBAT_DIM);

        let nonzero = combat_obs.iter().filter(|&&v| v != 0.0).count();
        assert!(nonzero > 5, "Combat obs should have non-zero values, got {}", nonzero);
    }

    #[test]
    fn test_card_effect_vector_strike() {
        let registry = CardRegistry::new();
        let ev = card_effect_vector("Strike_P", &registry);
        assert!(ev[1] > 0.0, "Strike should have damage > 0");
        assert_eq!(ev[8], 1.0, "Strike should be attack type");
    }

    #[test]
    fn test_relic_encoding_matches_python_order() {
        // Verify that relic encoding uses the correct sorted position
        // matching Python's sorted(ALL_RELICS.keys())
        assert_eq!(relic_catalog_index("Akabeko"), Some(0));
        assert_eq!(relic_catalog_index("PureWater"), Some(123));
        assert_eq!(relic_catalog_index("Vajra"), Some(171));
        assert_eq!(relic_catalog_index("Yang"), Some(180));
        assert_eq!(relic_catalog_index("NonexistentRelic"), None);

        // Verify observation vector encodes PureWater at the right position
        let engine = RunEngine::new(42, 20);
        let obs = get_observation(&engine);
        // PureWater is at index 123 in catalog, relic section starts at offset 25
        assert_eq!(obs[25 + 123], 1.0, "PureWater should be at obs[148]");
    }

    #[test]
    fn test_deterministic_obs() {
        let engine1 = RunEngine::new(42, 20);
        let engine2 = RunEngine::new(42, 20);
        let obs1 = get_observation(&engine1);
        let obs2 = get_observation(&engine2);
        assert_eq!(obs1, obs2, "Same seed should produce same observation");
    }
}
