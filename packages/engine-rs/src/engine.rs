//! Combat engine — slim orchestrator for MCTS simulations.
//!
//! Core turn loop that delegates to:
//! - card_effects: card play effect execution
//! - status_effects: end-of-turn hand card triggers
//! - combat_hooks: enemy turns, boss damage hooks

use pyo3::prelude::*;
use rand::seq::SliceRandom;

use crate::actions::{Action, PyAction};
use crate::cards::{CardDef, CardRegistry, CardTarget, CardType};
use crate::combat_hooks;
use crate::damage;
use crate::enemies;
use crate::orbs::{EvokeEffect, PassiveEffect};
use crate::potions;
use crate::powers;
use crate::relics;
use crate::state::{CombatState, EnemyCombatState, PyCombatState, Stance};
use crate::status_effects;

/// Combat phase enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatPhase {
    NotStarted,
    PlayerTurn,
    EnemyTurn,
    CombatOver,
}

/// The Rust combat engine. Wraps CombatState + card registry + RNG.
#[derive(Clone)]
pub struct CombatEngine {
    pub state: CombatState,
    pub phase: CombatPhase,
    pub card_registry: CardRegistry,
    pub(crate) rng: crate::seed::StsRandom,
}

impl CombatEngine {
    /// Create a new combat engine.
    pub fn new(state: CombatState, seed: u64) -> Self {
        Self {
            state,
            phase: CombatPhase::NotStarted,
            card_registry: CardRegistry::new(),
            rng: crate::seed::StsRandom::new(seed),
        }
    }

    // =======================================================================
    // Core API
    // =======================================================================

    /// Start combat: apply relic effects, shuffle draw pile, draw initial hand.
    pub fn start_combat(&mut self) {
        if self.phase != CombatPhase::NotStarted {
            return;
        }

        // Apply combat-start relic effects
        relics::apply_combat_start_relics(&mut self.state);

        // Shuffle draw pile
        self.state.draw_pile.shuffle(&mut self.rng);

        // Start first player turn
        self.start_player_turn();
    }

    /// Check if combat is over.
    pub fn is_combat_over(&self) -> bool {
        self.state.combat_over
    }

    /// Get all legal actions from the current state.
    pub fn get_legal_actions(&self) -> Vec<Action> {
        if self.phase != CombatPhase::PlayerTurn || self.state.combat_over {
            return Vec::new();
        }

        let mut actions = Vec::new();
        let living = self.state.targetable_enemy_indices();

        // Card plays
        for (hand_idx, card_id) in self.state.hand.iter().enumerate() {
            let card = self.card_registry.get_or_default(card_id);
            if self.can_play_card(&card, card_id) {
                match card.target {
                    CardTarget::Enemy => {
                        for &enemy_idx in &living {
                            actions.push(Action::PlayCard {
                                card_idx: hand_idx,
                                target_idx: enemy_idx as i32,
                            });
                        }
                    }
                    _ => {
                        actions.push(Action::PlayCard {
                            card_idx: hand_idx,
                            target_idx: -1,
                        });
                    }
                }
            }
        }

        // Potion uses
        for (pot_idx, potion_id) in self.state.potions.iter().enumerate() {
            if !potion_id.is_empty() {
                if potions::potion_requires_target(potion_id) {
                    // Targeted potion: one action per living enemy
                    for &enemy_idx in &living {
                        actions.push(Action::UsePotion {
                            potion_idx: pot_idx,
                            target_idx: enemy_idx as i32,
                        });
                    }
                } else {
                    actions.push(Action::UsePotion {
                        potion_idx: pot_idx,
                        target_idx: -1,
                    });
                }
            }
        }

        // End turn is always legal
        actions.push(Action::EndTurn);

        actions
    }

    /// Execute an action.
    pub fn execute_action(&mut self, action: &Action) {
        match action {
            Action::EndTurn => self.end_turn(),
            Action::PlayCard {
                card_idx,
                target_idx,
            } => self.play_card(*card_idx, *target_idx),
            Action::UsePotion {
                potion_idx,
                target_idx,
            } => self.use_potion(*potion_idx, *target_idx),
        }
    }

    /// Deep clone for MCTS tree search.
    pub fn clone_state(&self) -> CombatEngine {
        CombatEngine {
            state: self.state.clone(),
            phase: self.phase,
            card_registry: CardRegistry::new(), // Registry is stateless, cheap to recreate
            rng: self.rng.clone(),
        }
    }

    // =======================================================================
    // Turn Flow
    // =======================================================================

    fn start_player_turn(&mut self) {
        self.state.turn += 1;
        self.phase = CombatPhase::PlayerTurn;

        // Blasphemy: die at start of turn
        if self.state.blasphemy_active {
            self.state.blasphemy_active = false;
            self.state.player.hp = 0;
            self.state.combat_over = true;
            self.state.player_won = false;
            self.phase = CombatPhase::CombatOver;
            return;
        }

        // Reset energy — Ice Cream preserves unspent energy
        if relics::has_ice_cream(&self.state) {
            self.state.energy += self.state.max_energy;
        } else {
            self.state.energy = self.state.max_energy;
        }

        // Reset turn counters
        self.state.cards_played_this_turn = 0;
        self.state.attacks_played_this_turn = 0;
        self.state.last_card_type = None;

        // Reset per-turn statuses
        self.state.player.set_status("Wave of the Hand", 0);

        // Necronomicon reset
        relics::necronomicon_reset(&mut self.state);

        // All turn-start relic effects (Lantern, Happy Flower, Mercury Hourglass, etc.)
        relics::apply_turn_start_relics(&mut self.state);

        // Divinity auto-exit at start of turn
        if self.state.stance == Stance::Divinity {
            self.change_stance(Stance::Neutral);
        }

        // Block decay — Calipers retains up to 15, Barricade retains all
        // Skip block reset on turn 1 to preserve combat-start relic effects (Anchor)
        if self.state.turn > 1 {
            let barricade = self.state.player.status("Barricade") > 0;
            let blur = self.state.player.status("Blur") > 0;
            if barricade || blur {
                // Keep all block
            } else {
                let retained = relics::calipers_block_retention(&self.state, self.state.player.block);
                self.state.player.block = retained;
            }
        }

        // LoseStrength/LoseDexterity at end of previous turn
        let lose_str = self.state.player.status("LoseStrength");
        if lose_str > 0 {
            self.state.player.add_status("Strength", -lose_str);
            self.state.player.set_status("LoseStrength", 0);
        }
        let lose_dex = self.state.player.status("LoseDexterity");
        if lose_dex > 0 {
            self.state.player.add_status("Dexterity", -lose_dex);
            self.state.player.set_status("LoseDexterity", 0);
        }

        // Deva Form: gain energy at start of turn (increasing)
        let deva_form = self.state.player.status("DevaForm");
        if deva_form > 0 {
            self.state.energy += deva_form;
            // Increase for next turn
            self.state.player.set_status("DevaForm", deva_form + 1);
        }

        // Devotion: gain Mantra at start of turn
        let devotion = self.state.player.status("Devotion");
        if devotion > 0 {
            self.gain_mantra(devotion);
        }

        // ---- Start-of-turn orb passives (Plasma) ----
        self.apply_orb_start_of_turn();

        // BattleHymn: add Smite(s) to hand at start of turn
        let battle_hymn = self.state.player.status("BattleHymn");
        if battle_hymn > 0 {
            for _ in 0..battle_hymn {
                let smite_id = self.temp_card_id("Smite");
                if self.state.hand.len() < 10 {
                    self.state.hand.push(smite_id);
                }
            }
        }

        // Draw cards (default 5)
        self.draw_cards(5);
    }

    fn end_turn(&mut self) {
        if self.phase != CombatPhase::PlayerTurn {
            return;
        }

        // Clear Entangled (only lasts one turn)
        self.state.player.set_status("Entangled", 0);

        // ---- STS end-of-turn order: relics -> powers/buffs -> status cards -> discard ----

        // 1. End-of-turn relic triggers
        relics::apply_turn_end_relics(&mut self.state);

        // 2. End-of-turn power triggers
        // Metallicize, Plated Armor
        powers::apply_metallicize(&mut self.state.player);
        powers::apply_plated_armor(&mut self.state.player);

        // Like Water: if in Calm, gain block
        let like_water = self.state.player.status("LikeWater");
        if like_water > 0 && self.state.stance == Stance::Calm {
            self.state.player.block += like_water;
        }

        // Study: add Insight to draw pile
        let study = self.state.player.status("Study");
        if study > 0 {
            for _ in 0..study {
                let insight_id = self.temp_card_id("Insight");
                self.state.draw_pile.push(insight_id);
            }
        }

        // Omega: deal damage to all living enemies
        let omega = self.state.player.status("Omega");
        if omega > 0 {
            let living = self.state.living_enemy_indices();
            for idx in living {
                self.deal_damage_to_enemy(idx, omega);
            }
        }

        // 3. End-of-turn hand card triggers (Burn, Decay, Regret, Doubt, Shame)
        let player_died = status_effects::process_end_turn_hand_cards(
            &mut self.state,
            &self.card_registry,
        );
        if player_died {
            self.phase = CombatPhase::CombatOver;
            return;
        }

        // 4. Discard hand — Runic Pyramid keeps ALL cards in hand (including Status/Curse).
        //    Only Ethereal cards exhaust at end of turn regardless of Runic Pyramid.
        let explicitly_retained = std::mem::take(&mut self.state.retained_cards);
        let mut ethereal_exhausted = 0i32;
        if relics::has_runic_pyramid(&self.state) {
            // Runic Pyramid: keep ALL cards except ethereal (which exhaust)
            let hand = std::mem::take(&mut self.state.hand);
            let mut kept = Vec::new();
            for card_id in hand {
                let card = self.card_registry.get_or_default(&card_id);
                if card.effects.contains(&"ethereal") {
                    self.state.exhaust_pile.push(card_id);
                    ethereal_exhausted += 1;
                } else {
                    kept.push(card_id);
                }
            }
            self.state.hand = kept;
        } else {
            // Normal: retain tagged cards + explicitly retained (Meditate), exhaust ethereal, discard rest
            let hand = std::mem::take(&mut self.state.hand);
            let mut retained = Vec::new();
            for card_id in hand {
                let card = self.card_registry.get_or_default(&card_id);
                if card.effects.contains(&"retain") || explicitly_retained.contains(&card_id) {
                    retained.push(card_id);
                } else if card.effects.contains(&"ethereal") {
                    self.state.exhaust_pile.push(card_id);
                    ethereal_exhausted += 1;
                } else {
                    self.state.discard_pile.push(card_id);
                }
            }
            self.state.hand = retained;
        }

        // Trigger exhaust hooks for ethereal cards exhausted at end of turn
        for _ in 0..ethereal_exhausted {
            self.trigger_on_exhaust();
        }

        // ---- End-of-turn orb passives ----
        self.apply_orb_end_of_turn();
        if self.state.combat_over {
            return;
        }

        // Player poison tick (before enemy turns)
        let player_poison = self.state.player.status("Poison");
        if player_poison > 0 {
            let intangible = self.state.player.status("Intangible") > 0;
            let tungsten_rod = self.state.has_relic("Tungsten Rod");
            let hp_loss = damage::apply_hp_loss(player_poison, intangible, tungsten_rod);
            self.state.player.hp -= hp_loss;
            self.state.total_damage_taken += hp_loss;
            // Decrement poison by 1
            let new_poison = player_poison - 1;
            self.state.player.set_status("Poison", new_poison);

            if self.state.player.hp <= 0 {
                // Check Fairy in a Bottle
                let revive_hp = potions::check_fairy_revive(&self.state);
                if revive_hp > 0 {
                    potions::consume_fairy(&mut self.state);
                    self.state.player.hp = revive_hp;
                } else {
                    self.state.player.hp = 0;
                    self.state.combat_over = true;
                    self.state.player_won = false;
                    self.phase = CombatPhase::CombatOver;
                    return;
                }
            }
        }

        // Check combat end (Omega may have killed enemies)
        if self.check_combat_end() {
            return;
        }

        // Enemy turns (skip if Vault was played)
        if self.state.skip_enemy_turn {
            self.state.skip_enemy_turn = false;
        } else {
            combat_hooks::do_enemy_turns(self);
        }

        // End of round: decrement debuffs on player
        powers::decrement_debuffs(&mut self.state.player);

        // End of round: decrement debuffs on enemies
        for enemy in &mut self.state.enemies {
            if !enemy.entity.is_dead() {
                powers::decrement_debuffs(&mut enemy.entity);
            }
        }

        // Decrement player Intangible
        let intangible = self.state.player.status("Intangible");
        if intangible > 0 {
            self.state.player.set_status("Intangible", intangible - 1);
        }

        // Check combat end
        if !self.check_combat_end() {
            self.start_player_turn();
        }
    }

    // =======================================================================
    // Card Play
    // =======================================================================

    fn can_play_card(&self, card: &CardDef, card_id: &str) -> bool {
        // Unplayable cards
        if card.cost == -2 || card.effects.contains(&"unplayable") {
            return false;
        }

        // Velvet Choker: max 6 cards per turn
        if !relics::velvet_choker_can_play(&self.state) {
            return false;
        }

        // Energy check
        let cost = self.effective_cost(card, card_id);
        if cost > self.state.energy {
            return false;
        }

        // Entangled: can't play attacks
        if self.state.player.status("Entangled") > 0 && card.card_type == CardType::Attack {
            return false;
        }

        // Signature Move: only playable if no other attacks in hand
        if card.effects.contains(&"only_attack_in_hand") {
            let other_attacks = self.state.hand.iter().filter(|c| {
                let other_card = self.card_registry.get_or_default(c);
                other_card.card_type == CardType::Attack && c.as_str() != card_id
            }).count();
            if other_attacks > 0 {
                return false;
            }
        }

        // Clash: only playable if hand contains only attacks
        if card.effects.contains(&"only_attacks_in_hand") {
            let has_non_attack = self.state.hand.iter().any(|c| {
                let other_card = self.card_registry.get_or_default(c);
                other_card.card_type != CardType::Attack
            });
            if has_non_attack {
                return false;
            }
        }

        // Grand Finale: only playable if draw pile is empty
        if card.effects.contains(&"only_empty_draw") && !self.state.draw_pile.is_empty() {
            return false;
        }

        true
    }

    fn effective_cost(&self, card: &CardDef, _card_id: &str) -> i32 {
        // X-cost cards: always playable if energy >= 0
        if card.cost == -1 {
            return 0;
        }

        // NextAttackFree: next attack costs 0
        if card.card_type == CardType::Attack && self.state.player.status("NextAttackFree") > 0 {
            return 0;
        }

        card.cost
    }

    fn play_card(&mut self, hand_idx: usize, target_idx: i32) {
        if hand_idx >= self.state.hand.len() {
            return;
        }

        let card_id = self.state.hand[hand_idx].clone();
        let card = self.card_registry.get_or_default(&card_id);

        if !self.can_play_card(&card, &card_id) {
            return;
        }

        // Pay energy
        let cost = self.effective_cost(&card, &card_id);
        self.state.energy -= cost;

        // Remove from hand
        self.state.hand.remove(hand_idx);

        // Track counters
        self.state.cards_played_this_turn += 1;
        self.state.total_cards_played += 1;
        if card.card_type == CardType::Attack {
            self.state.attacks_played_this_turn += 1;
        }

        // Execute effects (last_card_type refers to card played BEFORE this one)
        crate::card_effects::execute_card_effects(self, &card, &card_id, target_idx);

        // Update last_card_type AFTER effects (so next card sees this one)
        self.state.last_card_type = Some(card.card_type);

        // Stance change from card
        if let Some(stance_name) = card.enter_stance {
            let new_stance = Stance::from_str(stance_name);
            self.change_stance(new_stance);
        }

        // Gremlin Nob Enrage: gains Strength when player plays non-Attack
        if card.card_type != CardType::Attack {
            for enemy in &mut self.state.enemies {
                if enemy.is_alive() {
                    let enrage = enemy.entity.status("Enrage");
                    if enrage > 0 {
                        enemy.entity.add_status("Strength", enrage);
                    }
                }
            }
        }

        // Pain curse: deal 1 HP loss per Pain card in hand on every card play
        let pain_killed = status_effects::process_pain_on_card_play(
            &mut self.state,
            &self.card_registry,
        );
        if pain_killed {
            self.phase = CombatPhase::CombatOver;
            return;
        }

        // All on-card-play relic effects (Fan, Kunai, Shuriken, Nunchaku, etc.)
        relics::on_card_played(&mut self.state, card.card_type);

        // ---- Power hooks on card play ----

        // After Image: gain block per card played
        let after_image_block = powers::get_after_image_block(&self.state.player);
        if after_image_block > 0 {
            self.state.player.block += after_image_block;
        }

        // A Thousand Cuts: deal damage to ALL living enemies per card played
        let thousand_cuts_dmg = powers::get_thousand_cuts_damage(&self.state.player);
        if thousand_cuts_dmg > 0 {
            let living = self.state.living_enemy_indices();
            for idx in living {
                self.deal_damage_to_enemy(idx, thousand_cuts_dmg);
            }
        }

        // Rage: gain block when playing an Attack
        if card.card_type == CardType::Attack {
            let rage_block = powers::get_rage_block(&self.state.player);
            if rage_block > 0 {
                self.state.player.block += rage_block;
            }
        }

        // Beat of Death: enemies with this power deal damage to player per card played
        for ei in 0..self.state.enemies.len() {
            if self.state.enemies[ei].is_alive() {
                let bod = powers::get_beat_of_death_damage(&self.state.enemies[ei].entity);
                if bod > 0 {
                    let intangible = self.state.player.status("Intangible") > 0;
                    let has_torii = self.state.has_relic("Torii");
                    let has_tungsten = self.state.has_relic("Tungsten Rod");
                    let result = damage::calculate_incoming_damage(
                        bod,
                        self.state.player.block,
                        self.state.stance == Stance::Wrath,
                        self.state.player.is_vulnerable(),
                        intangible,
                        has_torii,
                        has_tungsten,
                    );
                    self.state.player.block = result.block_remaining;
                    if result.hp_loss > 0 {
                        self.state.player.hp -= result.hp_loss;
                        self.state.total_damage_taken += result.hp_loss;
                    }
                }
            }
        }

        // Slow: increment counter on enemies with Slow power
        for ei in 0..self.state.enemies.len() {
            if self.state.enemies[ei].is_alive() {
                powers::increment_slow(&mut self.state.enemies[ei].entity);
            }
        }

        // TimeWarp: increment card counter; at 12 end turn + enemy gains Strength
        for ei in 0..self.state.enemies.len() {
            if self.state.enemies[ei].is_alive() {
                let triggered = powers::increment_time_warp(&mut self.state.enemies[ei].entity);
                if triggered {
                    self.state.enemies[ei].entity.add_status("Strength", 2);
                    self.end_turn();
                    return;
                }
            }
        }

        // Panache: every 5 cards played, deal damage to all enemies
        let panache_dmg = powers::check_panache(&mut self.state.player);
        if panache_dmg > 0 {
            let living = self.state.living_enemy_indices();
            for idx in living {
                self.deal_damage_to_enemy(idx, panache_dmg);
            }
        }

        // Consume NextAttackFree after playing an attack
        if card.card_type == CardType::Attack && self.state.player.status("NextAttackFree") > 0 {
            self.state.player.set_status("NextAttackFree", 0);
        }

        // Power card: install status effect instead of going to discard
        if card.card_type == CardType::Power {
            self.install_power(&card);
            // Storm: channel Lightning when playing a Power
            if powers::should_storm_channel(&self.state.player) {
                self.channel_orb(crate::orbs::OrbType::Lightning);
            }
            // Heatsink: draw cards when playing a Power
            let heatsink_draw = powers::get_heatsink_draw(&self.state.player);
            if heatsink_draw > 0 {
                self.draw_cards(heatsink_draw);
            }
            // Powers don't go to any pile
        } else if card.effects.contains(&"shuffle_self_into_draw") {
            // Tantrum: goes to draw pile instead of discard
            // (already handled in execute_card_effects, don't double-add)
        } else if card.exhaust {
            self.state.exhaust_pile.push(card_id.clone());
            self.trigger_on_exhaust();
        } else {
            self.state.discard_pile.push(card_id.clone());
        }

        // Conclude: end the turn immediately after playing
        // Let end_turn() handle the remaining hand (respects retain/ethereal)
        if card.effects.contains(&"end_turn") {
            self.end_turn();
            return;
        }

        // Check combat end after card play
        self.check_combat_end();
    }

    /// Install a power card as a permanent status effect.
    fn install_power(&mut self, card: &CardDef) {
        for effect in card.effects {
            match *effect {
                "on_wrath_draw" => {
                    // Rushdown: draw cards when entering Wrath
                    let current = self.state.player.status("Rushdown");
                    self.state
                        .player
                        .set_status("Rushdown", current + card.base_magic.max(2));
                }
                "on_stance_change_block" => {
                    // MentalFortress: gain block on any stance change
                    let current = self.state.player.status("MentalFortress");
                    self.state
                        .player
                        .set_status("MentalFortress", current + card.base_magic);
                }
                "battle_hymn" => {
                    // BattleHymn: at start of turn, add Smite(s) to hand
                    let current = self.state.player.status("BattleHymn");
                    self.state
                        .player
                        .set_status("BattleHymn", current + card.base_magic.max(1));
                }
                "like_water" => {
                    // LikeWater: at end of turn, if in Calm, gain block
                    let current = self.state.player.status("LikeWater");
                    self.state
                        .player
                        .set_status("LikeWater", current + card.base_magic.max(1));
                }
                "on_scry_block" => {
                    // Nirvana: gain block on Scry
                    let current = self.state.player.status("Nirvana");
                    self.state
                        .player
                        .set_status("Nirvana", current + card.base_magic.max(1));
                }
                "study" => {
                    // Study: at end of turn, add Insight to draw
                    let current = self.state.player.status("Study");
                    self.state
                        .player
                        .set_status("Study", current + card.base_magic.max(1));
                }
                "deva_form" => {
                    // DevaForm: at start of turn, gain energy (increasing)
                    let current = self.state.player.status("DevaForm");
                    self.state
                        .player
                        .set_status("DevaForm", current + card.base_magic.max(1));
                }
                "devotion" => {
                    // Devotion: at start of turn, gain Mantra
                    let current = self.state.player.status("Devotion");
                    self.state
                        .player
                        .set_status("Devotion", current + card.base_magic.max(1));
                }
                "establishment" => {
                    // Establishment: retained cards cost 1 less
                    let current = self.state.player.status("Establishment");
                    self.state
                        .player
                        .set_status("Establishment", current + card.base_magic.max(1));
                }
                "fasting" => {
                    // Fasting: gain Str and Dex, lose 1 max energy
                    let amount = card.base_magic.max(1);
                    self.state.player.add_status("Strength", amount);
                    self.state.player.add_status("Dexterity", amount);
                    self.state.max_energy -= 1;
                    self.state.energy = self.state.energy.min(self.state.max_energy);
                }
                "master_reality" => {
                    // MasterReality: all temp cards created are upgraded
                    self.state.player.set_status("MasterReality", 1);
                }
                "omega" => {
                    // Omega: deal damage to all enemies at end of turn
                    let current = self.state.player.status("Omega");
                    self.state
                        .player
                        .set_status("Omega", current + card.base_magic.max(1));
                }
                "after_image" => {
                    let current = self.state.player.status("After Image");
                    self.state.player.set_status("After Image", current + card.base_magic.max(1));
                }
                "draw_on_power_play" => {
                    let current = self.state.player.status("Heatsink");
                    self.state.player.set_status("Heatsink", current + card.base_magic.max(1));
                }
                "channel_lightning_on_power" => {
                    let current = self.state.player.status("Storm");
                    self.state.player.set_status("Storm", current + card.base_magic.max(1));
                }
                "buffer" => {
                    let current = self.state.player.status("Buffer");
                    self.state.player.set_status("Buffer", current + card.base_magic.max(1));
                }
                "extra_draw_each_turn" => {
                    let current = self.state.player.status("Draw");
                    self.state.player.set_status("Draw", current + card.base_magic.max(1));
                }
                _ => {}
            }
        }
    }

    // =======================================================================
    // Potion Use
    // =======================================================================

    fn use_potion(&mut self, potion_idx: usize, target_idx: i32) {
        if potion_idx >= self.state.potions.len() {
            return;
        }
        if self.state.potions[potion_idx].is_empty() {
            return;
        }

        let potion_id = self.state.potions[potion_idx].clone();

        // Apply potion effect
        let success = potions::apply_potion(&mut self.state, &potion_id, target_idx);

        if success {
            // Consume the potion slot
            self.state.potions[potion_idx] = String::new();
        }

        // Check combat end (potions can kill enemies)
        self.check_combat_end();
    }

    // =======================================================================
    // Damage Dealing / Taking
    // =======================================================================

    pub fn deal_damage_to_enemy(&mut self, enemy_idx: usize, damage: i32) {
        let enemy = &mut self.state.enemies[enemy_idx];
        let blocked = enemy.entity.block.min(damage);
        let hp_damage = damage - blocked;
        enemy.entity.block -= blocked;
        enemy.entity.hp -= hp_damage;
        self.state.total_damage_dealt += hp_damage;

        if enemy.entity.hp <= 0 {
            enemy.entity.hp = 0;
        }

        // Boss damage hooks
        combat_hooks::on_enemy_damaged(self, enemy_idx, hp_damage);
    }

    #[allow(dead_code)]
    pub fn deal_damage_to_player(&mut self, damage: i32) {
        let player = &mut self.state.player;
        let blocked = player.block.min(damage);
        let hp_damage = damage - blocked;
        player.block -= blocked;
        player.hp -= hp_damage;
        self.state.total_damage_taken += hp_damage;

        if player.hp <= 0 {
            // Check Fairy in a Bottle
            let revive_hp = potions::check_fairy_revive(&self.state);
            if revive_hp > 0 {
                potions::consume_fairy(&mut self.state);
                self.state.player.hp = revive_hp;
            } else {
                self.state.player.hp = 0;
            }
        }
    }

    // =======================================================================
    // Orb Effects
    // =======================================================================

    /// Pick a random living enemy index using the combat RNG.
    fn random_living_enemy(&mut self) -> Option<usize> {
        let living = self.state.living_enemy_indices();
        if living.is_empty() {
            return None;
        }
        if living.len() == 1 {
            return Some(living[0]);
        }
        let roll = self.rng.random(living.len() as i32 - 1) as usize;
        Some(living[roll])
    }

    /// Pick the living enemy with the lowest HP.
    fn lowest_hp_enemy(&self) -> Option<usize> {
        let living = self.state.living_enemy_indices();
        living
            .iter()
            .copied()
            .min_by_key(|&i| self.state.enemies[i].entity.hp)
    }

    /// Apply a single evoke effect to the game state.
    pub(crate) fn apply_evoke_effect(&mut self, effect: EvokeEffect) {
        match effect {
            EvokeEffect::LightningDamage(dmg) => {
                if let Some(idx) = self.random_living_enemy() {
                    self.deal_damage_to_enemy(idx, dmg);
                }
            }
            EvokeEffect::FrostBlock(block) => {
                self.state.player.block += block;
            }
            EvokeEffect::DarkDamage(dmg) => {
                if let Some(idx) = self.lowest_hp_enemy() {
                    self.deal_damage_to_enemy(idx, dmg);
                }
            }
            EvokeEffect::PlasmaEnergy(energy) => {
                self.state.energy += energy;
            }
            EvokeEffect::None => {}
        }
    }

    /// Apply a single passive effect to the game state.
    fn apply_passive_effect(&mut self, effect: PassiveEffect) {
        match effect {
            PassiveEffect::LightningDamage(dmg) => {
                if let Some(idx) = self.random_living_enemy() {
                    self.deal_damage_to_enemy(idx, dmg);
                }
            }
            PassiveEffect::FrostBlock(block) => {
                self.state.player.block += block;
            }
            PassiveEffect::PlasmaEnergy(energy) => {
                self.state.energy += energy;
            }
            PassiveEffect::None => {}
        }
    }

    /// Trigger orb end-of-turn passives and apply their effects.
    fn apply_orb_end_of_turn(&mut self) {
        if !self.state.orb_slots.has_orbs() {
            return;
        }
        let focus = self.state.player.focus();
        let effects = self.state.orb_slots.trigger_end_of_turn_passives(focus);
        for effect in effects {
            self.apply_passive_effect(effect);
            if self.state.player.is_dead() || self.check_combat_end() {
                return;
            }
        }
    }

    /// Trigger orb start-of-turn passives (Plasma) and apply their effects.
    fn apply_orb_start_of_turn(&mut self) {
        if !self.state.orb_slots.has_orbs() {
            return;
        }
        let effects = self.state.orb_slots.trigger_start_of_turn_passives();
        for effect in effects {
            self.apply_passive_effect(effect);
        }
    }

    // =======================================================================
    // Enemy Turns
    // =======================================================================

    fn do_enemy_turns(&mut self) {
        self.phase = CombatPhase::EnemyTurn;

        let num_enemies = self.state.enemies.len();
        for i in 0..num_enemies {
            if !self.state.enemies[i].is_alive() {
                continue;
            }

            // Block decays at start of enemy turn
            self.state.enemies[i].entity.block = 0;

            // Metallicize: enemy gains block
            powers::apply_metallicize(&mut self.state.enemies[i].entity);

            // Poison tick
            let poison_dmg = powers::tick_poison(&mut self.state.enemies[i].entity);
            if poison_dmg > 0 {
                self.state.total_damage_dealt += poison_dmg;
                if self.state.enemies[i].entity.is_dead() {
                    self.state.enemies[i].entity.hp = 0;
                    continue;
                }
            }

            // Ritual strength gain (not first turn)
            if !self.state.enemies[i].first_turn {
                powers::apply_ritual(&mut self.state.enemies[i].entity);
            }

            // Execute enemy move
            self.execute_enemy_move(i);

            // Check player death
            if self.state.player.is_dead() {
                self.state.player.hp = 0;
                self.state.combat_over = true;
                self.state.player_won = false;
                self.phase = CombatPhase::CombatOver;
                return;
            }

            // Mark first turn complete
            self.state.enemies[i].first_turn = false;
        }
    }

    fn execute_enemy_move(&mut self, enemy_idx: usize) {
        let enemy = &self.state.enemies[enemy_idx];
        if enemy.move_id == -1 {
            return;
        }

        // Attack
        if enemy.move_damage > 0 {
            let enemy_strength = enemy.entity.strength();
            let enemy_weak = enemy.entity.is_weak();
            let base_damage = enemy.move_damage + enemy_strength;

            // Apply Weak to enemy's attack
            let mut damage_f = base_damage as f64;
            if enemy_weak {
                damage_f *= damage::WEAK_MULT;
            }

            // Floor the per-hit base (before stance/vuln/intangible)
            let per_hit_base = (damage_f as i32).max(0);

            let is_wrath = self.state.stance == Stance::Wrath;
            let player_vuln = self.state.player.is_vulnerable();
            let player_intangible = self.state.player.status("Intangible") > 0;
            let has_torii = self.state.has_relic("Torii");
            let has_tungsten = self.state.has_relic("Tungsten Rod");

            let hits = enemy.move_hits;
            for _ in 0..hits {
                let result = damage::calculate_incoming_damage(
                    per_hit_base,
                    self.state.player.block,
                    is_wrath,
                    player_vuln,
                    player_intangible,
                    has_torii,
                    has_tungsten,
                );

                self.state.player.block = result.block_remaining;
                if result.hp_loss > 0 {
                    self.state.player.hp -= result.hp_loss;
                    self.state.total_damage_taken += result.hp_loss;

                    // Plated Armor decrements on unblocked HP damage
                    let plated = self.state.player.status("Plated Armor");
                    if plated > 0 {
                        let new_plated = plated - 1;
                        self.state.player.set_status("Plated Armor", new_plated);
                    }
                }

                if self.state.player.hp <= 0 {
                    // Check Fairy in a Bottle
                    let revive_hp = potions::check_fairy_revive(&self.state);
                    if revive_hp > 0 {
                        potions::consume_fairy(&mut self.state);
                        self.state.player.hp = revive_hp;
                    } else {
                        self.state.player.hp = 0;
                    }
                }

                if self.state.player.is_dead() {
                    return;
                }
            }
        }

        // Block
        if self.state.enemies[enemy_idx].move_block > 0 {
            let block = self.state.enemies[enemy_idx].move_block;
            self.state.enemies[enemy_idx].entity.block += block;
        }

        // Apply move effects
        let effects = self.state.enemies[enemy_idx].move_effects.clone();
        if let Some(&amt) = effects.get("weak") {
            powers::apply_debuff(&mut self.state.player, "Weakened", amt);
        }
        if let Some(&amt) = effects.get("vulnerable") {
            powers::apply_debuff(&mut self.state.player, "Vulnerable", amt);
        }
        if let Some(&amt) = effects.get("frail") {
            powers::apply_debuff(&mut self.state.player, "Frail", amt);
        }
        if let Some(&amt) = effects.get("strength") {
            self.state.enemies[enemy_idx]
                .entity
                .add_status("Strength", amt);
        }
        if let Some(&amt) = effects.get("ritual") {
            self.state.enemies[enemy_idx]
                .entity
                .set_status("Ritual", amt);
        }
        if let Some(&amt) = effects.get("entangle") {
            if amt > 0 {
                self.state.player.set_status("Entangled", 1);
            }
        }
        if let Some(&amt) = effects.get("slimed") {
            // Add Slimed cards to discard pile
            for _ in 0..amt {
                self.state.discard_pile.push("Slimed".to_string());
            }
        }
        if let Some(&amt) = effects.get("daze") {
            // Add Daze cards to discard pile
            for _ in 0..amt {
                self.state.discard_pile.push("Daze".to_string());
            }
        }
        if let Some(&amt) = effects.get("burn") {
            for _ in 0..amt {
                self.state.discard_pile.push("Burn".to_string());
            }
        }
        if let Some(&amt) = effects.get("burn+") {
            for _ in 0..amt {
                self.state.discard_pile.push("Burn".to_string());
            }
        }
        // Lagavulin Siphon Soul: reduce player Strength and Dexterity
        if let Some(&amt) = effects.get("siphon_str") {
            self.state.player.add_status("Strength", -(amt));
        }
        if let Some(&amt) = effects.get("siphon_dex") {
            self.state.player.add_status("Dexterity", -(amt));
        }

        // Advance enemy to next move for next turn
        enemies::roll_next_move(&mut self.state.enemies[enemy_idx]);
    }

    // =======================================================================
    // Draw / Shuffle
    // =======================================================================

    pub fn draw_cards(&mut self, count: i32) {
        for _ in 0..count {
            if self.state.hand.len() >= 10 {
                break; // Hand size limit
            }

            if self.state.draw_pile.is_empty() {
                // Shuffle discard into draw
                if self.state.discard_pile.is_empty() {
                    break; // No cards left anywhere
                }
                let mut shuffled = std::mem::take(&mut self.state.discard_pile);
                shuffled.shuffle(&mut self.rng);
                self.state.draw_pile = shuffled;
            }

            if let Some(card) = self.state.draw_pile.pop() {
                self.state.hand.push(card);
            }
        }
    }

    /// Shuffle the draw pile (pub(crate) for card_effects).
    pub(crate) fn shuffle_draw_pile(&mut self) {
        self.state.draw_pile.shuffle(&mut self.rng);
    }

    /// Generate a random range using the engine RNG (pub(crate) for card_effects).
    pub(crate) fn rng_gen_range(&mut self, range: std::ops::Range<usize>) -> usize {
        use rand::Rng;
        self.rng.gen_range(range)
    }

    // =======================================================================
    // Stance
    // =======================================================================

    pub fn change_stance(&mut self, new_stance: Stance) {
        let old_stance = self.state.stance;
        if old_stance == new_stance {
            return;
        }

        // Exit Calm: gain 2 energy (+ Violet Lotus bonus)
        if old_stance == Stance::Calm {
            let bonus = relics::violet_lotus_calm_exit_bonus(&self.state);
            self.state.energy += 2 + bonus;
        }

        // Enter Divinity: gain 3 energy
        if new_stance == Stance::Divinity {
            self.state.energy += 3;
        }

        self.state.stance = new_stance;

        // -- Power triggers on stance change --

        // MentalFortress: gain block on ANY stance change
        let mental_fortress = self.state.player.status("MentalFortress");
        if mental_fortress > 0 {
            self.state.player.block += mental_fortress;
        }

        // Rushdown: draw cards when entering Wrath
        if new_stance == Stance::Wrath {
            let rushdown = self.state.player.status("Rushdown");
            if rushdown > 0 {
                self.draw_cards(rushdown);
            }
        }
    }

    /// Gain mantra and check for Divinity entry (10+ mantra).
    pub fn gain_mantra(&mut self, amount: i32) {
        self.state.mantra += amount;
        self.state.mantra_gained += amount;
        if self.state.mantra >= 10 {
            self.state.mantra -= 10;
            self.change_stance(Stance::Divinity);
        }
    }

    // =======================================================================
    // Helpers: Temp Card Creation (respects Master Reality)
    // =======================================================================

    /// Get the ID for a temporary card, upgrading if Master Reality is active.
    pub fn temp_card_id(&self, base_id: &str) -> String {
        if self.state.player.status("MasterReality") > 0 {
            format!("{}+", base_id)
        } else {
            base_id.to_string()
        }
    }

    /// Perform Scry: approximate for MCTS by discarding bottom N cards from draw pile.
    /// Triggers Nirvana (on_scry block) and Weave (return_on_scry).
    pub fn do_scry(&mut self, amount: i32) {
        // For MCTS: approximate scry as discarding bottom cards from draw pile.
        let to_scry = (amount as usize).min(self.state.draw_pile.len());
        if to_scry > 0 {
            // Remove from the bottom (front) of draw pile as an approximation
            let scryed: Vec<String> = self.state.draw_pile.drain(0..to_scry).collect();
            // In MCTS, we discard all scryed cards (simplification)
            for card in scryed {
                self.state.discard_pile.push(card);
            }
        }

        // Nirvana: gain block when scrying
        let nirvana = self.state.player.status("Nirvana");
        if nirvana > 0 {
            self.state.player.block += nirvana;
        }

        // Weave: return from discard to hand on Scry
        let mut weave_indices = Vec::new();
        for (i, card_id) in self.state.discard_pile.iter().enumerate() {
            if card_id.starts_with("Weave") {
                weave_indices.push(i);
            }
        }
        // Remove in reverse order to maintain indices
        for &i in weave_indices.iter().rev() {
            let card = self.state.discard_pile.remove(i);
            if self.state.hand.len() < 10 {
                self.state.hand.push(card);
            }
        }
    }

    // =======================================================================
    // Exhaust Hooks
    // =======================================================================

    /// Trigger all on-exhaust power and relic hooks.
    pub fn trigger_on_exhaust(&mut self) {
        // Feel No Pain: gain block per exhaust
        let fnp = powers::get_feel_no_pain_block(&self.state.player);
        if fnp > 0 {
            self.state.player.block += fnp;
        }

        // Dark Embrace: draw 1 card per exhaust
        let de = powers::get_dark_embrace_draw(&self.state.player);
        if de > 0 {
            self.draw_cards(de);
        }

        // Dead Branch (relic): add a random card to hand on exhaust
        if relics::dead_branch_on_exhaust(&self.state) {
            let temp_card = self.temp_card_id("Strike");
            if self.state.hand.len() < 10 {
                self.state.hand.push(temp_card);
            }
        }
    }

    // =======================================================================
    // Orb Channel / Evoke (public API for card_effects)
    // =======================================================================

    /// Channel an orb. If slots are full, evokes the front orb first.
    pub fn channel_orb(&mut self, orb_type: crate::orbs::OrbType) {
        let focus = self.state.player.focus();
        let evoke_effect = self.state.orb_slots.channel(orb_type, focus);
        self.apply_evoke_effect(evoke_effect);
    }

    /// Evoke the front orb.
    pub fn evoke_front_orb(&mut self) {
        let focus = self.state.player.focus();
        let effect = self.state.orb_slots.evoke_front(focus);
        self.apply_evoke_effect(effect);
    }

    /// Evoke the front orb N times.
    pub fn evoke_front_orb_n(&mut self, n: usize) {
        let focus = self.state.player.focus();
        let effects = self.state.orb_slots.evoke_front_n(n, focus);
        for effect in effects {
            self.apply_evoke_effect(effect);
            if self.state.combat_over {
                return;
            }
        }
    }

    /// Evoke all orbs.
    pub fn evoke_all_orbs(&mut self) {
        let focus = self.state.player.focus();
        let effects = self.state.orb_slots.evoke_all(focus);
        for effect in effects {
            self.apply_evoke_effect(effect);
            if self.state.combat_over {
                return;
            }
        }
    }

    /// Initialize Defect orb slots (3 by default).
    pub fn init_defect_orbs(&mut self, num_slots: usize) {
        self.state.orb_slots = crate::orbs::OrbSlots::new(num_slots);
    }

    // =======================================================================
    // Combat End Check
    // =======================================================================

    pub fn check_combat_end(&mut self) -> bool {
        if self.state.combat_over {
            return true;
        }

        // Victory: all enemies dead
        if self.state.is_victory() {
            self.state.combat_over = true;
            self.state.player_won = true;
            self.phase = CombatPhase::CombatOver;
            return true;
        }

        // Defeat: player dead
        if self.state.is_defeat() {
            self.state.combat_over = true;
            self.state.player_won = false;
            self.phase = CombatPhase::CombatOver;
            return true;
        }

        false
    }
}

// ===========================================================================
// PyO3 Bindings — RustCombatEngine exposed to Python
// ===========================================================================

#[pyclass(name = "RustCombatEngine")]
pub struct RustCombatEngine {
    engine: CombatEngine,
}

#[pymethods]
impl RustCombatEngine {
    /// Create a new Rust combat engine.
    ///
    /// Args:
    ///     player_hp: Player's current HP
    ///     player_max_hp: Player's maximum HP
    ///     energy: Starting energy per turn
    ///     deck: List of card IDs (strings)
    ///     enemies: List of (id, hp, max_hp, move_damage, move_hits) tuples
    ///     seed: RNG seed for shuffling
    ///     relics: Optional list of relic IDs
    #[new]
    #[pyo3(signature = (player_hp, player_max_hp, energy, deck, enemies, seed=42, relics=None))]
    fn new_py(
        player_hp: i32,
        player_max_hp: i32,
        energy: i32,
        deck: Vec<String>,
        enemies: Vec<(String, i32, i32, i32, i32)>,
        seed: u64,
        relics: Option<Vec<String>>,
    ) -> Self {
        let enemy_states: Vec<EnemyCombatState> = enemies
            .into_iter()
            .map(|(id, hp, max_hp, move_damage, move_hits)| {
                let mut e = EnemyCombatState::new(&id, hp, max_hp);
                e.move_damage = move_damage;
                e.move_hits = move_hits;
                e
            })
            .collect();

        let mut state = CombatState::new(player_hp, player_max_hp, enemy_states, deck, energy);
        if let Some(r) = relics {
            state.relics = r;
        }

        RustCombatEngine {
            engine: CombatEngine::new(state, seed),
        }
    }

    /// Start combat (shuffle + draw initial hand).
    fn start_combat(&mut self) {
        self.engine.start_combat();
    }

    /// Get legal actions as a list of Action objects.
    fn get_legal_actions(&self) -> Vec<PyAction> {
        self.engine
            .get_legal_actions()
            .into_iter()
            .map(|a| PyAction { inner: a })
            .collect()
    }

    /// Execute an action.
    fn take_action(&mut self, action: &PyAction) {
        self.engine.execute_action(&action.inner);
    }

    /// Check if combat is over.
    fn is_combat_over(&self) -> bool {
        self.engine.is_combat_over()
    }

    /// Check if player won.
    fn is_victory(&self) -> bool {
        self.engine.state.player_won
    }

    /// Get the current combat state as a Python CombatState object.
    fn get_state(&self) -> PyCombatState {
        PyCombatState {
            inner: self.engine.state.clone(),
        }
    }

    /// Get player HP.
    #[getter]
    fn player_hp(&self) -> i32 {
        self.engine.state.player.hp
    }

    /// Get player block.
    #[getter]
    fn player_block(&self) -> i32 {
        self.engine.state.player.block
    }

    /// Get current energy.
    #[getter]
    fn energy(&self) -> i32 {
        self.engine.state.energy
    }

    /// Get current turn number.
    #[getter]
    fn turn(&self) -> i32 {
        self.engine.state.turn
    }

    /// Get the current hand as list of card IDs.
    #[getter]
    fn hand(&self) -> Vec<String> {
        self.engine.state.hand.clone()
    }

    /// Get stance as string.
    #[getter]
    fn stance(&self) -> &str {
        self.engine.state.stance.as_str()
    }

    /// Set an enemy's next move (for external AI control).
    fn set_enemy_move(
        &mut self,
        enemy_idx: usize,
        move_id: i32,
        damage: i32,
        hits: i32,
        block: i32,
    ) {
        if enemy_idx < self.engine.state.enemies.len() {
            self.engine.state.enemies[enemy_idx].set_move(move_id, damage, hits, block);
        }
    }

    /// Set an enemy move effect (e.g., "weak" -> 2).
    fn set_enemy_move_effect(&mut self, enemy_idx: usize, effect: &str, amount: i32) {
        if enemy_idx < self.engine.state.enemies.len() {
            self.engine.state.enemies[enemy_idx]
                .move_effects
                .insert(effect.to_string(), amount);
        }
    }

    /// Deep clone for MCTS (returns a new independent engine).
    fn clone_for_mcts(&self) -> RustCombatEngine {
        RustCombatEngine {
            engine: self.engine.clone_state(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "RustCombatEngine(hp={}/{}, energy={}, turn={}, hand={}, enemies={}, over={})",
            self.engine.state.player.hp,
            self.engine.state.player.max_hp,
            self.engine.state.energy,
            self.engine.state.turn,
            self.engine.state.hand.len(),
            self.engine.state.enemies.len(),
            self.engine.state.combat_over,
        )
    }
}

// ===========================================================================
// Rust-only tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_state() -> CombatState {
        let deck = vec![
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

        let mut enemy = EnemyCombatState::new("JawWorm", 44, 44);
        enemy.set_move(1, 11, 1, 0); // 11 damage attack

        CombatState::new(80, 80, vec![enemy], deck, 3)
    }

    #[test]
    fn test_start_combat_draws_hand() {
        let state = make_test_state();
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        assert_eq!(engine.state.hand.len(), 5);
        assert_eq!(engine.state.draw_pile.len(), 5);
        assert_eq!(engine.state.turn, 1);
        assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    }

    #[test]
    fn test_legal_actions_include_end_turn() {
        let state = make_test_state();
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let actions = engine.get_legal_actions();
        assert!(actions.contains(&Action::EndTurn));
        assert!(!actions.is_empty());
    }

    #[test]
    fn test_play_strike_deals_damage() {
        let state = make_test_state();
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;

        // Find a Strike in hand and play it
        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c.starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });

        // Strike deals 6 damage
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 6);
        assert_eq!(engine.state.energy, 2); // Spent 1 energy
        assert_eq!(engine.state.hand.len(), 4); // Card removed from hand
    }

    #[test]
    fn test_play_defend_gives_block() {
        let state = make_test_state();
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let defend_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c.starts_with("Defend"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: defend_idx,
            target_idx: -1,
        });

        // Defend gives 5 block
        assert_eq!(engine.state.player.block, 5);
    }

    #[test]
    fn test_eruption_enters_wrath() {
        let mut state = make_test_state();
        // Ensure Eruption is in the deck and will be drawn
        state.draw_pile = vec![
            "Eruption".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let eruption_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c == "Eruption")
            .unwrap();

        engine.execute_action(&Action::PlayCard {
            card_idx: eruption_idx,
            target_idx: 0,
        });

        assert_eq!(engine.state.stance, Stance::Wrath);
    }

    #[test]
    fn test_vigilance_enters_calm() {
        let mut state = make_test_state();
        state.draw_pile = vec![
            "Vigilance".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let vig_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c == "Vigilance")
            .unwrap();

        engine.execute_action(&Action::PlayCard {
            card_idx: vig_idx,
            target_idx: -1,
        });

        assert_eq!(engine.state.stance, Stance::Calm);
    }

    #[test]
    fn test_calm_exit_grants_energy() {
        let mut state = make_test_state();
        state.stance = Stance::Calm;
        state.draw_pile = vec![
            "Eruption".to_string(), // Will enter Wrath, exiting Calm for +2 energy
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_energy = engine.state.energy; // Should be 3

        let eruption_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c == "Eruption")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: eruption_idx,
            target_idx: 0,
        });

        // Eruption costs 2, but exiting Calm grants +2, net 0 change
        assert_eq!(engine.state.energy, initial_energy - 2 + 2);
        assert_eq!(engine.state.stance, Stance::Wrath);
    }

    #[test]
    fn test_end_turn_enemy_attacks() {
        let state = make_test_state();
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.player.hp;

        engine.execute_action(&Action::EndTurn);

        // Enemy attacks for 11 damage (Jaw Worm)
        assert_eq!(engine.state.player.hp, initial_hp - 11);
        assert_eq!(engine.state.turn, 2); // New turn started
    }

    #[test]
    fn test_block_absorbs_damage() {
        let state = make_test_state();
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Play Defend first
        let defend_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c.starts_with("Defend"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: defend_idx,
            target_idx: -1,
        });

        assert_eq!(engine.state.player.block, 5);
        let initial_hp = engine.state.player.hp;

        engine.execute_action(&Action::EndTurn);

        // 11 damage - 5 block = 6 HP lost
        assert_eq!(engine.state.player.hp, initial_hp - 6);
    }

    #[test]
    fn test_enemy_death_ends_combat() {
        let mut state = make_test_state();
        state.enemies[0].entity.hp = 5; // Low HP enemy

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c.starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });

        // 6 damage kills 5 HP enemy
        assert!(engine.state.combat_over);
        assert!(engine.state.player_won);
    }

    #[test]
    fn test_player_death_ends_combat() {
        let mut state = make_test_state();
        state.enemies[0].move_damage = 100; // Lethal damage

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        engine.execute_action(&Action::EndTurn);

        assert!(engine.state.combat_over);
        assert!(!engine.state.player_won);
        assert_eq!(engine.state.player.hp, 0);
    }

    #[test]
    fn test_wrath_doubles_outgoing_damage() {
        let mut state = make_test_state();
        state.stance = Stance::Wrath;

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;

        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c.starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });

        // Divinity auto-exits at turn start, so stance is Neutral.
        // But we set Wrath which doesn't auto-exit.
        // Wait -- Divinity auto-exits, Wrath does not. Let me check.
        // start_player_turn only exits Divinity, not Wrath. Good.
        // Strike in Wrath: 6 * 2.0 = 12
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 12);
    }

    #[test]
    fn test_wrath_doubles_incoming_damage() {
        let mut state = make_test_state();
        state.stance = Stance::Wrath;

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Wrath doesn't auto-exit at turn start (only Divinity does)
        assert_eq!(engine.state.stance, Stance::Wrath);

        let initial_hp = engine.state.player.hp;
        engine.execute_action(&Action::EndTurn);

        // Enemy: 11 * 2.0 (Wrath incoming) = 22 damage
        assert_eq!(engine.state.player.hp, initial_hp - 22);
    }

    #[test]
    fn test_shuffle_on_empty_draw() {
        let mut state = make_test_state();
        state.draw_pile = vec!["Strike_P".to_string()]; // Only 1 card
        state.discard_pile = vec![
            "Defend_P".to_string(),
            "Defend_P".to_string(),
            "Defend_P".to_string(),
            "Defend_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Should draw 1 from draw, then shuffle 4 from discard, draw 4 more
        assert_eq!(engine.state.hand.len(), 5);
        assert!(engine.state.discard_pile.is_empty());
    }

    #[test]
    fn test_vulnerability_increases_damage() {
        let mut state = make_test_state();
        state.enemies[0].entity.set_status("Vulnerable", 2);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;

        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c.starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });

        // 6 * 1.5 = 9
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 9);
    }

    #[test]
    fn test_strength_adds_damage() {
        let mut state = make_test_state();
        state.player.set_status("Strength", 3);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;

        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c.starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });

        // 6 + 3 = 9
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 9);
    }

    #[test]
    fn test_debuff_decrement_on_end_round() {
        let mut state = make_test_state();
        state.player.set_status("Weakened", 2);
        state.player.set_status("Vulnerable", 1);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();
        engine.execute_action(&Action::EndTurn);

        // After one round: Weakened 2->1, Vulnerable 1->removed
        assert_eq!(engine.state.player.status("Weakened"), 1);
        assert_eq!(engine.state.player.status("Vulnerable"), 0);
    }

    #[test]
    fn test_poison_ticks_on_enemies() {
        let mut state = make_test_state();
        state.enemies[0].entity.set_status("Poison", 5);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;
        engine.execute_action(&Action::EndTurn);

        // Poison deals 5 HP to enemy, then decrements to 4.
        // Enemy also attacks player for 11, but that doesn't affect enemy HP.
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 5);
        assert_eq!(engine.state.enemies[0].entity.status("Poison"), 4);
    }

    #[test]
    fn test_no_actions_when_combat_over() {
        let mut state = make_test_state();
        state.combat_over = true;

        let engine = CombatEngine::new(state, 42);
        let actions = engine.get_legal_actions();
        assert!(actions.is_empty());
    }

    // =================================================================
    // Phase A: Power trigger tests
    // =================================================================

    #[test]
    fn test_rushdown_draws_on_wrath_entry() {
        let mut state = make_test_state();
        // Give player Rushdown power (draw 2 on Wrath entry)
        state.player.set_status("Rushdown", 2);
        state.draw_pile = vec![
            "Eruption".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Defend_P".to_string(),
            // Extra cards for Rushdown draw
            "Defend_P".to_string(),
            "Defend_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Ensure Eruption is in hand (RNG may not have drawn it)
        if !engine.state.hand.contains(&"Eruption".to_string()) {
            engine.state.hand.push("Eruption".to_string());
        }

        let hand_size_before = engine.state.hand.len();

        // Find and play Eruption (enters Wrath)
        let eruption_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c == "Eruption")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: eruption_idx,
            target_idx: 0,
        });

        assert_eq!(engine.state.stance, Stance::Wrath);
        // Rushdown drew 2 cards: hand was (N-1) after playing Eruption, now N-1+2
        assert_eq!(engine.state.hand.len(), hand_size_before - 1 + 2);
    }

    #[test]
    fn test_mental_fortress_blocks_on_stance_change() {
        let mut state = make_test_state();
        // Give player MentalFortress power (4 block on stance change)
        state.player.set_status("MentalFortress", 4);
        state.draw_pile = vec![
            "Eruption".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Defend_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        assert_eq!(engine.state.player.block, 0);

        // Play Eruption -> enters Wrath, MentalFortress triggers
        let eruption_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c == "Eruption")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: eruption_idx,
            target_idx: 0,
        });

        assert_eq!(engine.state.stance, Stance::Wrath);
        assert_eq!(engine.state.player.block, 4); // MentalFortress block
    }

    #[test]
    fn test_mantra_accumulation_to_divinity() {
        let mut state = make_test_state();
        state.draw_pile = vec![
            "Prostrate".to_string(),  // 2 mantra + 4 block
            "Prostrate".to_string(),
            "Prostrate".to_string(),
            "Prostrate".to_string(),
            "Prostrate".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        assert_eq!(engine.state.mantra, 0);
        assert_eq!(engine.state.stance, Stance::Neutral);

        // Play Prostrate cards to accumulate mantra
        // Each gives 2 mantra (base_magic=2)
        for i in 0..5 {
            if let Some(idx) = engine
                .state
                .hand
                .iter()
                .position(|c| c == "Prostrate")
            {
                engine.execute_action(&Action::PlayCard {
                    card_idx: idx,
                    target_idx: -1,
                });

                if i < 4 {
                    // After 4 plays: 8 mantra, not yet Divinity
                    assert_ne!(engine.state.stance, Stance::Divinity);
                } else {
                    // After 5 plays: 10 mantra -> Divinity!
                    assert_eq!(engine.state.stance, Stance::Divinity);
                    assert_eq!(engine.state.mantra, 0); // Reset after entering Divinity
                }
            }
        }
    }

    #[test]
    fn test_violet_lotus_extra_energy_on_calm_exit() {
        let mut state = make_test_state();
        state.stance = Stance::Calm;
        state.relics.push("Violet Lotus".to_string());
        state.draw_pile = vec![
            "Eruption".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_energy = engine.state.energy; // 3

        let eruption_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c == "Eruption")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: eruption_idx,
            target_idx: 0,
        });

        // Eruption costs 2, Calm exit gives +2, Violet Lotus gives +1, net +1
        assert_eq!(engine.state.energy, initial_energy - 2 + 2 + 1);
    }

    #[test]
    fn test_divinity_auto_exit_gives_energy() {
        let mut state = make_test_state();
        // Pre-set mantra to 5 so only one Worship needed to enter Divinity
        state.mantra = 5;
        state.draw_pile = vec![
            "Worship".to_string(),  // 5 mantra, cost 2
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let energy_before = engine.state.energy; // 3

        // Play Worship: 5 + 5 = 10 mantra -> Divinity + 3 energy
        if let Some(idx) = engine.state.hand.iter().position(|c| c == "Worship") {
            engine.execute_action(&Action::PlayCard {
                card_idx: idx,
                target_idx: -1,
            });
        }
        assert_eq!(engine.state.stance, Stance::Divinity);
        assert_eq!(engine.state.mantra, 0);
        // Energy: 3 base - 2 Worship cost + 3 Divinity bonus = 4
        assert_eq!(engine.state.energy, energy_before - 2 + 3);

        // End turn + start new turn: Divinity auto-exits to Neutral
        engine.execute_action(&Action::EndTurn);
        assert_eq!(engine.state.stance, Stance::Neutral);
    }

    #[test]
    fn test_potion_use_fire_potion() {
        let mut state = make_test_state();
        state.potions[0] = "Fire Potion".to_string();

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;

        engine.execute_action(&Action::UsePotion {
            potion_idx: 0,
            target_idx: 0,
        });

        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 20);
        assert!(engine.state.potions[0].is_empty()); // Consumed
    }

    #[test]
    fn test_potion_use_energy_potion() {
        let mut state = make_test_state();
        state.potions[0] = "Energy Potion".to_string();

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_energy = engine.state.energy;

        engine.execute_action(&Action::UsePotion {
            potion_idx: 0,
            target_idx: -1,
        });

        assert_eq!(engine.state.energy, initial_energy + 2);
    }

    #[test]
    fn test_relic_vajra_at_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Vajra".to_string());

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        assert_eq!(engine.state.player.strength(), 1);

        // Strike should now deal 7 damage (6 + 1 Str)
        let initial_hp = engine.state.enemies[0].entity.hp;
        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c.starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 7);
    }

    #[test]
    fn test_relic_lantern_first_turn_energy() {
        let mut state = make_test_state();
        state.relics.push("Lantern".to_string());

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Turn 1: should have 3 + 1 = 4 energy
        assert_eq!(engine.state.energy, 4);
    }

    #[test]
    fn test_fairy_auto_revive() {
        let mut state = make_test_state();
        state.enemies[0].move_damage = 200; // Lethal damage
        state.potions[0] = "FairyPotion".to_string();

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        engine.execute_action(&Action::EndTurn);

        // Fairy should revive at 30% of 80 = 24 HP
        assert_eq!(engine.state.player.hp, 24);
        assert!(!engine.state.combat_over);
        assert!(engine.state.potions[0].is_empty()); // Consumed
    }

    #[test]
    fn test_enemy_slimed_cards_to_discard() {
        let mut state = make_test_state();
        state.enemies[0].move_damage = 0;
        state.enemies[0]
            .move_effects
            .insert("slimed".to_string(), 3);

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        engine.execute_action(&Action::EndTurn);

        // 5 cards from hand + 3 Slimed cards from enemy
        let slimed_count = engine
            .state
            .discard_pile
            .iter()
            .filter(|c| *c == "Slimed")
            .count();
        assert_eq!(slimed_count, 3);
    }

    #[test]
    fn test_entangle_prevents_attacks() {
        let mut state = make_test_state();
        state.player.set_status("Entangled", 1);
        state.draw_pile = vec![
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Defend_P".to_string(),
            "Defend_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let actions = engine.get_legal_actions();
        // Should NOT contain any Strike plays (attacks blocked by Entangle)
        let attack_actions: Vec<_> = actions
            .iter()
            .filter(|a| {
                if let Action::PlayCard { card_idx, .. } = a {
                    engine.state.hand[*card_idx].starts_with("Strike")
                } else {
                    false
                }
            })
            .collect();
        assert!(attack_actions.is_empty());

        // But Defend should still be playable
        let defend_actions: Vec<_> = actions
            .iter()
            .filter(|a| {
                if let Action::PlayCard { card_idx, .. } = a {
                    engine.state.hand[*card_idx].starts_with("Defend")
                } else {
                    false
                }
            })
            .collect();
        assert!(!defend_actions.is_empty());
    }

    #[test]
    fn test_miracle_gives_energy() {
        let mut state = make_test_state();
        state.draw_pile = vec![
            "Miracle".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_energy = engine.state.energy;
        let miracle_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c == "Miracle")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: miracle_idx,
            target_idx: -1,
        });

        // Miracle costs 0, gives 1 energy
        assert_eq!(engine.state.energy, initial_energy + 1);
        // Miracle exhausts
        assert!(engine.state.exhaust_pile.contains(&"Miracle".to_string()));
    }

    #[test]
    fn test_inner_peace_in_calm_draws() {
        let mut state = make_test_state();
        state.stance = Stance::Calm;
        state.draw_pile = vec![
            "InnerPeace".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            // Extra for Inner Peace draw
            "Defend_P".to_string(),
            "Defend_P".to_string(),
            "Defend_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        // Ensure InnerPeace is in hand (RNG may not have drawn it)
        if !engine.state.hand.contains(&"InnerPeace".to_string()) {
            engine.state.hand.push("InnerPeace".to_string());
        }

        let hand_before = engine.state.hand.len();

        let ip_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c == "InnerPeace")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: ip_idx,
            target_idx: -1,
        });

        // In Calm: draws 3 (base_magic=3). Hand was (N-1) after playing, now N-1+3
        assert_eq!(engine.state.hand.len(), hand_before - 1 + 3);
        // Stays in Calm (doesn't change stance when drawing)
        assert_eq!(engine.state.stance, Stance::Calm);
    }

    #[test]
    fn test_inner_peace_not_calm_enters_calm() {
        let mut state = make_test_state();
        state.stance = Stance::Neutral;
        state.draw_pile = vec![
            "InnerPeace".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let hand_before = engine.state.hand.len();

        let ip_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c == "InnerPeace")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: ip_idx,
            target_idx: -1,
        });

        // Not in Calm: enters Calm, no draw
        assert_eq!(engine.state.stance, Stance::Calm);
        assert_eq!(engine.state.hand.len(), hand_before - 1); // Only removed the played card
    }

    #[test]
    fn test_power_card_not_in_discard() {
        let mut state = make_test_state();
        state.draw_pile = vec![
            "MentalFortress".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let mf_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c == "MentalFortress")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: mf_idx,
            target_idx: -1,
        });

        // Power card should NOT be in discard pile
        assert!(!engine
            .state
            .discard_pile
            .contains(&"MentalFortress".to_string()));
        // MentalFortress status installed
        assert_eq!(engine.state.player.status("MentalFortress"), 4);
    }

    #[test]
    fn test_vigor_consumed_on_attack() {
        let mut state = make_test_state();
        state.player.set_status("Vigor", 8); // Akabeko

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_hp = engine.state.enemies[0].entity.hp;

        let strike_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c.starts_with("Strike"))
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: strike_idx,
            target_idx: 0,
        });

        // Strike deals 6 + 8 vigor = 14 damage
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 14);
        // Vigor consumed
        assert_eq!(engine.state.player.status("Vigor"), 0);
    }

    #[test]
    fn test_enemy_advances_moves_each_turn() {
        let state = make_test_state();
        // Jaw Worm starts with Chomp (11 damage)
        assert_eq!(state.enemies[0].move_id, 1); // CHOMP

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        engine.execute_action(&Action::EndTurn);

        // After first enemy turn, enemy should have rolled next move
        // Jaw Worm after Chomp -> Bellow (no damage, gains strength)
        assert_ne!(engine.state.enemies[0].move_id, -1);
        // The move should have changed from Chomp
        assert!(engine.state.enemies[0].move_history.contains(&1));
    }

    #[test]
    fn test_targeted_potion_in_legal_actions() {
        let mut state = make_test_state();
        state.potions[0] = "Fire Potion".to_string();

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let actions = engine.get_legal_actions();
        let potion_actions: Vec<_> = actions
            .iter()
            .filter(|a| matches!(a, Action::UsePotion { .. }))
            .collect();

        // Fire Potion requires target, so should have 1 action per living enemy
        assert_eq!(potion_actions.len(), 1); // One enemy alive
        if let Action::UsePotion { target_idx, .. } = potion_actions[0] {
            assert_eq!(*target_idx, 0); // Targets enemy 0
        }
    }

    #[test]
    fn test_halt_extra_block_in_wrath() {
        let mut state = make_test_state();
        state.stance = Stance::Wrath;
        state.draw_pile = vec![
            "Halt".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let halt_idx = engine
            .state
            .hand
            .iter()
            .position(|c| c == "Halt")
            .unwrap();
        engine.execute_action(&Action::PlayCard {
            card_idx: halt_idx,
            target_idx: -1,
        });

        // Halt: 3 base block + 9 extra in Wrath = 12 total
        assert_eq!(engine.state.player.block, 12);
    }
}
