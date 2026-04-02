//! Hook-dispatch power system — static dispatch tables for power triggers.
//!
//! Each power declares its hooks in one place. The engine loops the table
//! instead of scattering `status(sid::THING)` checks across engine.rs.
//!
//! Effect structs are returned by dispatch functions. The engine applies
//! the effects after dispatch (draw cards, deal damage, etc.).

use crate::ids::StatusId;
use crate::state::EntityState;
use crate::status_ids::sid;

// ===========================================================================
// Effect Structs — one per trigger type
// ===========================================================================

/// Effects produced by start-of-turn power hooks.
#[derive(Debug, Default)]
pub struct TurnStartEffect {
    pub energy: i32,
    pub draw: i32,
    pub hp_loss: i32,
    pub poison_all_enemies: i32,
    pub strength_gain: i32,
    pub dexterity_loss: i32,
    pub enter_divinity: bool,
    pub add_smites: i32,
    pub add_shivs: i32,
    pub add_strikes: i32,
    pub mantra_gain: i32,
    pub add_creative_ai_cards: i32,
    pub doppelganger_draw: i32,
    pub doppelganger_energy: i32,
    pub mayhem_draw: i32,
    pub tools_of_the_trade_draw: i32,
    pub tools_of_the_trade_discard: i32,
}

impl TurnStartEffect {
    pub fn merge(&mut self, other: Self) {
        self.energy += other.energy;
        self.draw += other.draw;
        self.hp_loss += other.hp_loss;
        self.poison_all_enemies += other.poison_all_enemies;
        self.strength_gain += other.strength_gain;
        self.dexterity_loss += other.dexterity_loss;
        self.enter_divinity = self.enter_divinity || other.enter_divinity;
        self.add_smites += other.add_smites;
        self.add_shivs += other.add_shivs;
        self.add_strikes += other.add_strikes;
        self.mantra_gain += other.mantra_gain;
        self.add_creative_ai_cards += other.add_creative_ai_cards;
        self.doppelganger_draw += other.doppelganger_draw;
        self.doppelganger_energy += other.doppelganger_energy;
        self.mayhem_draw += other.mayhem_draw;
        self.tools_of_the_trade_draw += other.tools_of_the_trade_draw;
        self.tools_of_the_trade_discard += other.tools_of_the_trade_discard;
    }
}

/// Effects produced by end-of-turn power hooks.
#[derive(Debug, Default)]
pub struct TurnEndEffect {
    pub block_gain: i32,
    pub omega_damage: i32,
    pub combust_damage: i32,
    pub combust_hp_loss: i32,
    pub add_insights: i32,
    pub clear_rage: bool,
}

impl TurnEndEffect {
    pub fn merge(&mut self, other: Self) {
        self.block_gain += other.block_gain;
        self.omega_damage += other.omega_damage;
        self.combust_damage += other.combust_damage;
        self.combust_hp_loss += other.combust_hp_loss;
        self.add_insights += other.add_insights;
        self.clear_rage = self.clear_rage || other.clear_rage;
    }
}

/// Effects produced by on-card-played power hooks.
#[derive(Debug, Default)]
pub struct OnCardPlayedEffect {
    pub block_gain: i32,
}

impl OnCardPlayedEffect {
    pub fn merge(&mut self, other: Self) {
        self.block_gain += other.block_gain;
    }
}

/// Effects produced by on-exhaust power hooks.
#[derive(Debug, Default)]
pub struct OnExhaustEffect {
    pub block_gain: i32,
    pub draw: i32,
}

impl OnExhaustEffect {
    pub fn merge(&mut self, other: Self) {
        self.block_gain += other.block_gain;
        self.draw += other.draw;
    }
}

/// Effects produced by on-stance-change power hooks.
#[derive(Debug, Default)]
pub struct OnStanceChangeEffect {
    pub block_gain: i32,
    pub draw: i32,
}

impl OnStanceChangeEffect {
    pub fn merge(&mut self, other: Self) {
        self.block_gain += other.block_gain;
        self.draw += other.draw;
    }
}

/// Effects produced by enemy-turn-start power hooks.
#[derive(Debug, Default)]
pub struct EnemyTurnStartEffect {
    pub block_gain: i32,
    pub heal: i32,
    pub strength_gain: i32,
    pub block_from_growth: i32,
    pub faded: bool,
    pub bomb_damage: i32,
    pub ritual_strength: i32,
}

impl EnemyTurnStartEffect {
    pub fn merge(&mut self, other: Self) {
        self.block_gain += other.block_gain;
        self.heal += other.heal;
        self.strength_gain += other.strength_gain;
        self.block_from_growth += other.block_from_growth;
        self.faded = self.faded || other.faded;
        self.bomb_damage += other.bomb_damage;
        self.ritual_strength += other.ritual_strength;
    }
}

// ===========================================================================
// Hook Tables — static arrays mapping status key -> hook function
// ===========================================================================

type TurnStartHookFn = fn(i32, &mut EntityState) -> TurnStartEffect;
type TurnEndHookFn = fn(i32, &mut EntityState) -> TurnEndEffect;
type OnCardPlayedHookFn = fn(i32, &EntityState) -> OnCardPlayedEffect;
type OnExhaustHookFn = fn(i32, &EntityState) -> OnExhaustEffect;
type OnStanceChangeHookFn = fn(i32, &EntityState, bool) -> OnStanceChangeEffect;
type EnemyTurnStartHookFn = fn(i32, &mut EntityState) -> EnemyTurnStartEffect;

// ---- Turn Start hooks ----

static TURN_START_HOOKS: &[(StatusId, TurnStartHookFn)] = &[
    (sid::DEMON_FORM, hook_demon_form),
    (sid::NOXIOUS_FUMES, hook_noxious_fumes),
    (sid::BRUTALITY, hook_brutality),
    (sid::BERSERK, hook_berserk),
    (sid::INFINITE_BLADES, hook_infinite_blades),
    (sid::HELLO_WORLD, hook_hello_world),
    (sid::BATTLE_HYMN, hook_battle_hymn),
    (sid::WRAITH_FORM, hook_wraith_form),
    (sid::CREATIVE_AI, hook_creative_ai),
    (sid::DEVA_FORM, hook_deva_form),
    (sid::MAGNETISM, hook_magnetism),
    (sid::DOPPELGANGER_DRAW, hook_doppelganger_draw),
    (sid::DOPPELGANGER_ENERGY, hook_doppelganger_energy),
    (sid::ENTER_DIVINITY, hook_enter_divinity),
    (sid::MAYHEM, hook_mayhem),
    (sid::TOOLS_OF_THE_TRADE, hook_tools_of_the_trade),
    (sid::DEVOTION, hook_devotion),
];

// ---- Turn End hooks ----

static TURN_END_HOOKS: &[(StatusId, TurnEndHookFn)] = &[
    (sid::METALLICIZE, hook_end_metallicize),
    (sid::PLATED_ARMOR, hook_end_plated_armor),
    (sid::LIKE_WATER, hook_end_like_water),
    (sid::STUDY, hook_end_study),
    (sid::OMEGA, hook_end_omega),
    (sid::COMBUST, hook_end_combust),
    (sid::RAGE, hook_end_rage),
    (sid::TEMP_STRENGTH, hook_end_temp_strength),
    // NOTE: Regeneration is intentionally NOT here. It fires AFTER Constricted
    // damage and orb passives in the original Java order. Kept inline in engine.rs.
];

// ---- On Card Played hooks (pre-effects: Java onUseCard) ----

static ON_CARD_PLAYED_PRE_HOOKS: &[(StatusId, OnCardPlayedHookFn)] = &[
    (sid::AFTER_IMAGE, hook_play_after_image),
];

// ---- On Card Played hooks (post-effects: Java onAfterUseCard) ----

static ON_CARD_PLAYED_POST_HOOKS: &[(StatusId, OnCardPlayedHookFn)] = &[
    (sid::RAGE, hook_play_rage),
];

// ---- On Exhaust hooks ----

static ON_EXHAUST_HOOKS: &[(StatusId, OnExhaustHookFn)] = &[
    (sid::FEEL_NO_PAIN, hook_exhaust_feel_no_pain),
    (sid::DARK_EMBRACE, hook_exhaust_dark_embrace),
];

// ---- On Stance Change hooks ----

static ON_STANCE_CHANGE_HOOKS: &[(StatusId, OnStanceChangeHookFn)] = &[
    (sid::MENTAL_FORTRESS, hook_stance_mental_fortress),
    (sid::RUSHDOWN, hook_stance_rushdown),
];

// ---- Enemy Turn Start hooks ----

static ENEMY_TURN_START_HOOKS: &[(StatusId, EnemyTurnStartHookFn)] = &[
    (sid::METALLICIZE, hook_enemy_metallicize),
    (sid::REGENERATION, hook_enemy_regeneration),
    (sid::GROWTH, hook_enemy_growth),
    (sid::FADING, hook_enemy_fading),
    (sid::THE_BOMB, hook_enemy_the_bomb),
    (sid::RITUAL, hook_enemy_ritual),
];

// ===========================================================================
// Dispatch Functions
// ===========================================================================

/// Dispatch all turn-start power hooks for the player.
/// Some hooks mutate the entity directly (DemonForm adds Strength) AND return effects.
/// The caller applies returned effects that need engine context (draw, damage, etc.).
pub fn dispatch_turn_start(entity: &mut EntityState) -> TurnStartEffect {
    let mut out = TurnStartEffect::default();
    for &(key, hook_fn) in TURN_START_HOOKS {
        let amt = entity.status(key);
        if amt != 0 {
            let effect = hook_fn(amt, entity);
            out.merge(effect);
        }
    }
    out
}

/// Dispatch all turn-end power hooks for the player.
/// `in_calm` is needed for LikeWater check.
pub fn dispatch_turn_end(entity: &mut EntityState, in_calm: bool) -> TurnEndEffect {
    let mut out = TurnEndEffect::default();
    for &(key, hook_fn) in TURN_END_HOOKS {
        let amt = entity.status(key);
        if amt != 0 {
            // LikeWater needs stance context -- skip if not in Calm
            if key == sid::LIKE_WATER && !in_calm {
                continue;
            }
            let effect = hook_fn(amt, entity);
            out.merge(effect);
        }
    }
    out
}

/// Dispatch pre-effects card-played hooks (AfterImage — Java onUseCard).
pub fn dispatch_on_card_played_pre(entity: &EntityState) -> OnCardPlayedEffect {
    let mut out = OnCardPlayedEffect::default();
    for &(key, hook_fn) in ON_CARD_PLAYED_PRE_HOOKS {
        let amt = entity.status(key);
        if amt > 0 {
            let effect = hook_fn(amt, entity);
            out.merge(effect);
        }
    }
    out
}

/// Dispatch post-effects card-played hooks (Rage — Java onAfterUseCard).
/// `is_attack` filters Rage (attack-only).
pub fn dispatch_on_card_played_post(entity: &EntityState, is_attack: bool) -> OnCardPlayedEffect {
    let mut out = OnCardPlayedEffect::default();
    for &(key, hook_fn) in ON_CARD_PLAYED_POST_HOOKS {
        let amt = entity.status(key);
        if amt > 0 {
            // Rage only fires on Attacks
            if key == sid::RAGE && !is_attack {
                continue;
            }
            let effect = hook_fn(amt, entity);
            out.merge(effect);
        }
    }
    out
}

/// Dispatch on-exhaust hooks.
pub fn dispatch_on_exhaust(entity: &EntityState) -> OnExhaustEffect {
    let mut out = OnExhaustEffect::default();
    for &(key, hook_fn) in ON_EXHAUST_HOOKS {
        let amt = entity.status(key);
        if amt > 0 {
            let effect = hook_fn(amt, entity);
            out.merge(effect);
        }
    }
    out
}

/// Dispatch on-stance-change hooks. `entering_wrath` enables Rushdown draw.
pub fn dispatch_on_stance_change(entity: &EntityState, entering_wrath: bool) -> OnStanceChangeEffect {
    let mut out = OnStanceChangeEffect::default();
    for &(key, hook_fn) in ON_STANCE_CHANGE_HOOKS {
        let amt = entity.status(key);
        if amt > 0 {
            let effect = hook_fn(amt, entity, entering_wrath);
            out.merge(effect);
        }
    }
    out
}

/// Dispatch enemy-turn-start hooks. `is_first_turn` skips Ritual on first turn.
pub fn dispatch_enemy_turn_start(entity: &mut EntityState, is_first_turn: bool) -> EnemyTurnStartEffect {
    let mut out = EnemyTurnStartEffect::default();
    for &(key, hook_fn) in ENEMY_TURN_START_HOOKS {
        let amt = entity.status(key);
        if amt != 0 {
            // Ritual skips first turn
            if key == sid::RITUAL && is_first_turn {
                continue;
            }
            let effect = hook_fn(amt, entity);
            out.merge(effect);
        }
    }
    out
}

// ===========================================================================
// Hook Implementations — Turn Start
// ===========================================================================

fn hook_demon_form(amt: i32, entity: &mut EntityState) -> TurnStartEffect {
    // DemonForm: gain Strength each turn (mutate directly)
    entity.add_status(sid::STRENGTH, amt);
    TurnStartEffect::default()
}

fn hook_noxious_fumes(amt: i32, _entity: &mut EntityState) -> TurnStartEffect {
    TurnStartEffect { poison_all_enemies: amt, ..Default::default() }
}

fn hook_brutality(amt: i32, _entity: &mut EntityState) -> TurnStartEffect {
    // Brutality: draw cards AND lose HP
    TurnStartEffect { draw: amt, hp_loss: amt, ..Default::default() }
}

fn hook_berserk(amt: i32, _entity: &mut EntityState) -> TurnStartEffect {
    TurnStartEffect { energy: amt, ..Default::default() }
}

fn hook_infinite_blades(amt: i32, _entity: &mut EntityState) -> TurnStartEffect {
    TurnStartEffect { add_shivs: amt, ..Default::default() }
}

fn hook_hello_world(amt: i32, _entity: &mut EntityState) -> TurnStartEffect {
    // HelloWorld: add Strike(s) as MCTS approximation for random common card
    TurnStartEffect { add_strikes: amt, ..Default::default() }
}

fn hook_battle_hymn(amt: i32, _entity: &mut EntityState) -> TurnStartEffect {
    TurnStartEffect { add_smites: amt, ..Default::default() }
}

fn hook_wraith_form(_amt: i32, entity: &mut EntityState) -> TurnStartEffect {
    // WraithForm: lose 1 Dexterity each turn (mutate directly)
    entity.add_status(sid::DEXTERITY, -1);
    TurnStartEffect::default()
}

fn hook_creative_ai(_amt: i32, _entity: &mut EntityState) -> TurnStartEffect {
    // CreativeAI: add random Power card to hand (MCTS: add "Smite")
    TurnStartEffect { add_creative_ai_cards: 1, ..Default::default() }
}

fn hook_deva_form(amt: i32, entity: &mut EntityState) -> TurnStartEffect {
    // DevaForm: gain energy (escalating), then increase for next turn
    let energy = amt;
    entity.set_status(sid::DEVA_FORM, amt + 1);
    TurnStartEffect { energy, ..Default::default() }
}

fn hook_magnetism(_amt: i32, _entity: &mut EntityState) -> TurnStartEffect {
    // Magnetism: add random card to hand (MCTS: add "Strike")
    TurnStartEffect { add_strikes: 1, ..Default::default() }
}

fn hook_doppelganger_draw(amt: i32, entity: &mut EntityState) -> TurnStartEffect {
    // One-shot: consume after use
    entity.set_status(sid::DOPPELGANGER_DRAW, 0);
    TurnStartEffect { doppelganger_draw: amt, ..Default::default() }
}

fn hook_doppelganger_energy(amt: i32, entity: &mut EntityState) -> TurnStartEffect {
    // One-shot: consume after use
    entity.set_status(sid::DOPPELGANGER_ENERGY, 0);
    TurnStartEffect { doppelganger_energy: amt, ..Default::default() }
}

fn hook_enter_divinity(_amt: i32, entity: &mut EntityState) -> TurnStartEffect {
    // Damaru relic flag: enter Divinity stance, then clear
    entity.set_status(sid::ENTER_DIVINITY, 0);
    TurnStartEffect { enter_divinity: true, ..Default::default() }
}

fn hook_mayhem(amt: i32, _entity: &mut EntityState) -> TurnStartEffect {
    TurnStartEffect { mayhem_draw: amt, ..Default::default() }
}

fn hook_tools_of_the_trade(amt: i32, _entity: &mut EntityState) -> TurnStartEffect {
    // ToolsOfTheTrade: draw N then discard N (discard needs RNG, handled by engine)
    TurnStartEffect {
        tools_of_the_trade_draw: amt,
        tools_of_the_trade_discard: amt,
        ..Default::default()
    }
}

fn hook_devotion(amt: i32, _entity: &mut EntityState) -> TurnStartEffect {
    TurnStartEffect { mantra_gain: amt, ..Default::default() }
}

// ===========================================================================
// Hook Implementations — Turn End
// ===========================================================================

fn hook_end_metallicize(amt: i32, _entity: &mut EntityState) -> TurnEndEffect {
    TurnEndEffect { block_gain: amt, ..Default::default() }
}

fn hook_end_plated_armor(amt: i32, _entity: &mut EntityState) -> TurnEndEffect {
    TurnEndEffect { block_gain: amt, ..Default::default() }
}

fn hook_end_like_water(amt: i32, _entity: &mut EntityState) -> TurnEndEffect {
    // Only called when in_calm is true (filtered at dispatch level)
    TurnEndEffect { block_gain: amt, ..Default::default() }
}

fn hook_end_study(amt: i32, _entity: &mut EntityState) -> TurnEndEffect {
    TurnEndEffect { add_insights: amt, ..Default::default() }
}

fn hook_end_omega(amt: i32, _entity: &mut EntityState) -> TurnEndEffect {
    TurnEndEffect { omega_damage: amt, ..Default::default() }
}

fn hook_end_combust(amt: i32, _entity: &mut EntityState) -> TurnEndEffect {
    // Combust: lose 1 HP, deal damage to all enemies
    TurnEndEffect { combust_damage: amt, combust_hp_loss: 1, ..Default::default() }
}

fn hook_end_rage(_amt: i32, entity: &mut EntityState) -> TurnEndEffect {
    entity.set_status(sid::RAGE, 0);
    TurnEndEffect { clear_rage: true, ..Default::default() }
}

fn hook_end_temp_strength(amt: i32, entity: &mut EntityState) -> TurnEndEffect {
    // Revert temporary Strength (mutate directly)
    entity.add_status(sid::STRENGTH, -amt);
    entity.set_status(sid::TEMP_STRENGTH, 0);
    TurnEndEffect::default()
}

// NOTE: Regeneration is kept inline in engine.rs (fires after Constricted/orb passives)

// ===========================================================================
// Hook Implementations — On Card Played
// ===========================================================================

fn hook_play_after_image(amt: i32, _entity: &EntityState) -> OnCardPlayedEffect {
    OnCardPlayedEffect { block_gain: amt }
}

fn hook_play_rage(amt: i32, _entity: &EntityState) -> OnCardPlayedEffect {
    // Only fires on Attacks (filtered at dispatch level)
    OnCardPlayedEffect { block_gain: amt }
}

// ===========================================================================
// Hook Implementations — On Exhaust
// ===========================================================================

fn hook_exhaust_feel_no_pain(amt: i32, _entity: &EntityState) -> OnExhaustEffect {
    OnExhaustEffect { block_gain: amt, ..Default::default() }
}

fn hook_exhaust_dark_embrace(amt: i32, _entity: &EntityState) -> OnExhaustEffect {
    OnExhaustEffect { draw: amt, ..Default::default() }
}

// ===========================================================================
// Hook Implementations — On Stance Change
// ===========================================================================

fn hook_stance_mental_fortress(amt: i32, _entity: &EntityState, _entering_wrath: bool) -> OnStanceChangeEffect {
    OnStanceChangeEffect { block_gain: amt, ..Default::default() }
}

fn hook_stance_rushdown(amt: i32, _entity: &EntityState, entering_wrath: bool) -> OnStanceChangeEffect {
    if entering_wrath {
        OnStanceChangeEffect { draw: amt, ..Default::default() }
    } else {
        OnStanceChangeEffect::default()
    }
}

// ===========================================================================
// Hook Implementations — Enemy Turn Start
// ===========================================================================

fn hook_enemy_metallicize(amt: i32, _entity: &mut EntityState) -> EnemyTurnStartEffect {
    EnemyTurnStartEffect { block_gain: amt, ..Default::default() }
}

fn hook_enemy_regeneration(amt: i32, entity: &mut EntityState) -> EnemyTurnStartEffect {
    // Heal and decrement
    entity.add_status(sid::REGENERATION, -1);
    EnemyTurnStartEffect { heal: amt, ..Default::default() }
}

fn hook_enemy_growth(amt: i32, entity: &mut EntityState) -> EnemyTurnStartEffect {
    // Growth: gain Strength AND Block equal to amount
    entity.add_status(sid::STRENGTH, amt);
    EnemyTurnStartEffect { block_from_growth: amt, ..Default::default() }
}

fn hook_enemy_fading(amt: i32, entity: &mut EntityState) -> EnemyTurnStartEffect {
    // Fading: decrement counter, die at 0
    let new_val = amt - 1;
    entity.set_status(sid::FADING, new_val);
    if new_val <= 0 {
        EnemyTurnStartEffect { faded: true, ..Default::default() }
    } else {
        EnemyTurnStartEffect::default()
    }
}

fn hook_enemy_the_bomb(amt: i32, entity: &mut EntityState) -> EnemyTurnStartEffect {
    // TheBomb: decrement turns counter, detonate on 0
    let turns = entity.status(sid::THE_BOMB_TURNS);
    let new_turns = turns - 1;
    entity.set_status(sid::THE_BOMB_TURNS, new_turns);
    if new_turns <= 0 {
        EnemyTurnStartEffect { bomb_damage: amt, ..Default::default() }
    } else {
        EnemyTurnStartEffect::default()
    }
}

fn hook_enemy_ritual(amt: i32, entity: &mut EntityState) -> EnemyTurnStartEffect {
    // Ritual: gain Strength (skipped on first turn, filtered at dispatch level)
    entity.add_status(sid::STRENGTH, amt);
    EnemyTurnStartEffect::default()
}
