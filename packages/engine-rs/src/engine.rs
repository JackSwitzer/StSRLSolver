//! Combat engine — core turn loop for MCTS simulations.
//!
//! Mirrors the Python CombatEngine with full power triggers:
//! - State management (energy, HP, block, hand, draw/discard)
//! - Damage/block calculation with all modifiers
//! - Card play with power triggers (Rushdown, MentalFortress, Mantra)
//! - Enemy AI (Act 1 enemies + bosses)
//! - Relic effects at combat start and during play
//! - Potion effects
//! - End turn (enemy attacks, block decay, debuff ticks)
//! - get_legal_actions()

use pyo3::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::actions::{Action, PyAction};
use crate::cards::{CardDef, CardRegistry, CardTarget, CardType};
use crate::damage;
use crate::enemies;
use crate::potions;
use crate::powers;
use crate::relics;
use crate::state::{CombatState, EnemyCombatState, PyCombatState, Stance};

/// Combat phase enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatPhase {
    NotStarted,
    PlayerTurn,
    EnemyTurn,
    CombatOver,
}

/// The Rust combat engine. Wraps CombatState + card registry + RNG.
pub struct CombatEngine {
    pub state: CombatState,
    pub phase: CombatPhase,
    pub card_registry: CardRegistry,
    rng: rand::rngs::SmallRng,
}

impl CombatEngine {
    /// Create a new combat engine.
    pub fn new(state: CombatState, seed: u64) -> Self {
        Self {
            state,
            phase: CombatPhase::NotStarted,
            card_registry: CardRegistry::new(),
            rng: rand::rngs::SmallRng::seed_from_u64(seed),
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
        let living = self.state.living_enemy_indices();

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

        // Reset energy
        self.state.energy = self.state.max_energy;

        // Reset turn counters
        self.state.cards_played_this_turn = 0;
        self.state.attacks_played_this_turn = 0;
        self.state.last_card_type = None;

        // Lantern: +1 energy on turn 1
        relics::apply_lantern_turn_start(&mut self.state);

        // Divinity auto-exit at start of turn
        if self.state.stance == Stance::Divinity {
            self.change_stance(Stance::Neutral);
        }

        // Block decay (simplified: no Barricade/Blur/Calipers in fast path)
        self.state.player.block = 0;

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

        // Draw cards (default 5)
        self.draw_cards(5);
    }

    fn end_turn(&mut self) {
        if self.phase != CombatPhase::PlayerTurn {
            return;
        }

        // Clear Entangled (only lasts one turn)
        self.state.player.set_status("Entangled", 0);

        // Discard hand — respecting Retain and Ethereal
        let hand = std::mem::take(&mut self.state.hand);
        let mut retained = Vec::new();
        for card_id in hand {
            let card = self.card_registry.get_or_default(&card_id);
            if card.effects.contains(&"retain") {
                // Retained cards stay in hand
                retained.push(card_id);
            } else if card.effects.contains(&"ethereal") {
                // Ethereal cards exhaust instead of discarding
                self.state.exhaust_pile.push(card_id);
            } else {
                self.state.discard_pile.push(card_id);
            }
        }
        self.state.hand = retained;

        // End-of-turn player powers: Metallicize, Plated Armor
        powers::apply_metallicize(&mut self.state.player);
        powers::apply_plated_armor(&mut self.state.player);

        // Enemy turns
        self.do_enemy_turns();

        // End of round: decrement debuffs on player
        powers::decrement_debuffs(&mut self.state.player);

        // End of round: decrement debuffs on enemies
        for enemy in &mut self.state.enemies {
            if !enemy.entity.is_dead() {
                powers::decrement_debuffs(&mut enemy.entity);
            }
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

        // Energy check
        let cost = self.effective_cost(card, card_id);
        if cost > self.state.energy {
            return false;
        }

        // Entangled: can't play attacks
        if self.state.player.status("Entangled") > 0 && card.card_type == CardType::Attack {
            return false;
        }

        true
    }

    fn effective_cost(&self, card: &CardDef, _card_id: &str) -> i32 {
        // X-cost cards: always playable if energy >= 0
        if card.cost == -1 {
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
        self.execute_card_effects(&card, &card_id, target_idx);

        // Update last_card_type AFTER effects (so next card sees this one)
        self.state.last_card_type = Some(card.card_type);

        // Stance change from card
        if let Some(stance_name) = card.enter_stance {
            let new_stance = Stance::from_str(stance_name);
            self.change_stance(new_stance);
        }

        // Ornamental Fan: gain 4 block every 3 cards
        relics::check_ornamental_fan(&mut self.state);

        // Power card: install status effect instead of going to discard
        if card.card_type == CardType::Power {
            self.install_power(&card);
            // Powers don't go to any pile
        } else if card.exhaust {
            self.state.exhaust_pile.push(card_id.clone());
        } else {
            self.state.discard_pile.push(card_id.clone());
        }

        // Conclude: discard remaining hand and end the turn
        if card.effects.contains(&"end_turn") {
            let hand = std::mem::take(&mut self.state.hand);
            for c in hand {
                self.state.discard_pile.push(c);
            }
            // Actually end the turn (enemy turns, debuff ticks, etc.)
            self.end_turn();
            return;
        }

        // Check combat end after card play
        self.check_combat_end();
    }

    fn execute_card_effects(&mut self, card: &CardDef, _card_id: &str, target_idx: i32) {
        // ---- Pen Nib check (before damage) ----
        let pen_nib_active = if card.card_type == CardType::Attack {
            relics::check_pen_nib(&mut self.state)
        } else {
            false
        };

        // ---- Vigor (consumed on first attack hit) ----
        let vigor = if card.card_type == CardType::Attack {
            let v = self.state.player.status("Vigor");
            if v > 0 {
                self.state.player.set_status("Vigor", 0);
            }
            v
        } else {
            0
        };

        // ---- Damage ----
        if card.base_damage >= 0 {
            let is_multi_hit = card.effects.contains(&"multi_hit");
            let hits = if is_multi_hit && card.base_magic > 0 {
                card.base_magic
            } else {
                1
            };

            let player_strength = self.state.player.strength();
            let player_weak = self.state.player.is_weak();
            let stance_mult = self.state.stance.outgoing_mult();

            match card.target {
                CardTarget::Enemy => {
                    if target_idx >= 0 && (target_idx as usize) < self.state.enemies.len() {
                        let enemy_vuln = self.state.enemies[target_idx as usize]
                            .entity
                            .is_vulnerable();
                        let mut dmg = damage::calculate_damage(
                            card.base_damage,
                            player_strength + vigor,
                            player_weak,
                            stance_mult,
                            enemy_vuln,
                            false,
                        );
                        if pen_nib_active {
                            dmg *= 2;
                        }
                        for _ in 0..hits {
                            self.deal_damage_to_enemy(target_idx as usize, dmg);
                            if self.state.enemies[target_idx as usize].entity.is_dead() {
                                break;
                            }
                        }
                    }
                }
                CardTarget::AllEnemy => {
                    let living = self.state.living_enemy_indices();
                    for enemy_idx in living {
                        let enemy_vuln = self.state.enemies[enemy_idx].entity.is_vulnerable();
                        let mut dmg = damage::calculate_damage(
                            card.base_damage,
                            player_strength + vigor,
                            player_weak,
                            stance_mult,
                            enemy_vuln,
                            false,
                        );
                        if pen_nib_active {
                            dmg *= 2;
                        }
                        for _ in 0..hits {
                            self.deal_damage_to_enemy(enemy_idx, dmg);
                            if self.state.enemies[enemy_idx].entity.is_dead() {
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // ---- Block ----
        if card.base_block >= 0 {
            let dex = self.state.player.dexterity();
            let frail = self.state.player.is_frail();
            let block = damage::calculate_block(card.base_block, dex, frail);
            self.state.player.block += block;
        }

        // ---- Halt: extra block in Wrath ----
        if card.effects.contains(&"extra_block_in_wrath") && self.state.stance == Stance::Wrath {
            if card.base_magic > 0 {
                let dex = self.state.player.dexterity();
                let frail = self.state.player.is_frail();
                let extra = damage::calculate_block(card.base_magic, dex, frail);
                self.state.player.block += extra;
            }
        }

        // ---- Draw ----
        if card.effects.contains(&"draw") && card.base_magic > 0 {
            self.draw_cards(card.base_magic);
        }

        // ---- Mantra ----
        if card.effects.contains(&"mantra") && card.base_magic > 0 {
            self.gain_mantra(card.base_magic);
        }

        // ---- Scry (simplified for MCTS: peek at top cards) ----
        if card.effects.contains(&"scry") && card.base_magic > 0 {
            // For MCTS: approximate scry as a no-op on the draw pile.
            // The real effect is choosing which cards to discard from top X,
            // but for fast simulation we skip this selection step.
        }

        // ---- Gain Energy (Miracle) ----
        if card.effects.contains(&"gain_energy") && card.base_magic > 0 {
            self.state.energy += card.base_magic;
        }

        // ---- Inner Peace: if in Calm, draw 3; else enter Calm ----
        if card.effects.contains(&"if_calm_draw_else_calm") {
            if self.state.stance == Stance::Calm {
                self.draw_cards(card.base_magic);
            } else {
                self.change_stance(Stance::Calm);
            }
        }

        // ---- BowlingBash: damage per living enemy (extra hits) ----
        if card.effects.contains(&"damage_per_enemy") {
            let living_count = self.state.living_enemy_indices().len() as i32;
            // BowlingBash deals base_damage * living_count to the target.
            // The base hit was already dealt above (1 hit). Deal (living_count - 1) more.
            if living_count > 1 && target_idx >= 0 && (target_idx as usize) < self.state.enemies.len() {
                let player_strength = self.state.player.strength();
                let player_weak = self.state.player.is_weak();
                let stance_mult = self.state.stance.outgoing_mult();
                let enemy_vuln = self.state.enemies[target_idx as usize].entity.is_vulnerable();
                let dmg = damage::calculate_damage(
                    card.base_damage,
                    player_strength + vigor,
                    player_weak,
                    stance_mult,
                    enemy_vuln,
                    false,
                );
                for _ in 0..(living_count - 1) {
                    if self.state.enemies[target_idx as usize].entity.is_dead() {
                        break;
                    }
                    self.deal_damage_to_enemy(target_idx as usize, dmg);
                }
            }
        }

        // ---- CrushJoints: apply Vulnerable if last card played was a Skill ----
        if card.effects.contains(&"vuln_if_last_skill") {
            if self.state.last_card_type == Some(CardType::Skill) {
                if target_idx >= 0 && (target_idx as usize) < self.state.enemies.len() {
                    let vuln_amount = card.base_magic.max(1);
                    self.state.enemies[target_idx as usize]
                        .entity
                        .add_status("Vulnerable", vuln_amount);
                }
            }
        }

        // ---- FollowUp: gain 1 energy if last card played was an Attack ----
        if card.effects.contains(&"energy_if_last_attack") {
            if self.state.last_card_type == Some(CardType::Attack) {
                self.state.energy += 1;
            }
        }

        // ---- TalkToTheHand: apply BlockReturn to target ----
        if card.effects.contains(&"apply_block_return") {
            if target_idx >= 0 && (target_idx as usize) < self.state.enemies.len() {
                let amount = card.base_magic.max(1);
                self.state.enemies[target_idx as usize]
                    .entity
                    .add_status("BlockReturn", amount);
            }
        }

        // ---- Ragnarok: deal damage to random enemies X times ----
        if card.effects.contains(&"damage_random_x_times") && card.base_magic > 0 {
            let player_strength = self.state.player.strength();
            let player_weak = self.state.player.is_weak();
            let stance_mult = self.state.stance.outgoing_mult();
            // The AllEnemy base hit was already dealt above. Ragnarok deals
            // base_magic total hits to random enemies. We already did 1 pass
            // (AllEnemy loop), so do (base_magic - 1) more random hits.
            for _ in 0..(card.base_magic - 1) {
                let living = self.state.living_enemy_indices();
                if living.is_empty() {
                    break;
                }
                let idx = living[self.rng.gen_range(0..living.len())];
                let enemy_vuln = self.state.enemies[idx].entity.is_vulnerable();
                let dmg = damage::calculate_damage(
                    card.base_damage,
                    player_strength + vigor,
                    player_weak,
                    stance_mult,
                    enemy_vuln,
                    false,
                );
                self.deal_damage_to_enemy(idx, dmg);
            }
        }
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

    fn deal_damage_to_enemy(&mut self, enemy_idx: usize, damage: i32) {
        let enemy = &mut self.state.enemies[enemy_idx];
        let blocked = enemy.entity.block.min(damage);
        let hp_damage = damage - blocked;
        enemy.entity.block -= blocked;
        enemy.entity.hp -= hp_damage;
        self.state.total_damage_dealt += hp_damage;

        if enemy.entity.hp <= 0 {
            enemy.entity.hp = 0;
        }
    }

    fn deal_damage_to_player(&mut self, damage: i32) {
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

            // Stance multiplier for incoming
            let stance_mult = self.state.stance.incoming_mult();
            let player_vuln = self.state.player.is_vulnerable();

            let hits = enemy.move_hits;
            let block_return = enemy.entity.status("BlockReturn");
            for _ in 0..hits {
                let mut hit_damage = damage_f * stance_mult;

                // Vulnerable
                if player_vuln {
                    hit_damage *= damage::VULN_MULT;
                }

                // Floor
                let final_damage = (hit_damage as i32).max(0);

                self.deal_damage_to_player(final_damage);

                // Talk to the Hand: player gains block when this enemy attacks
                if block_return > 0 {
                    self.state.player.block += block_return;
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

        // Advance enemy to next move for next turn
        enemies::roll_next_move(&mut self.state.enemies[enemy_idx]);
    }

    // =======================================================================
    // Draw / Shuffle
    // =======================================================================

    fn draw_cards(&mut self, count: i32) {
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

    // =======================================================================
    // Stance
    // =======================================================================

    fn change_stance(&mut self, new_stance: Stance) {
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
    fn gain_mantra(&mut self, amount: i32) {
        self.state.mantra += amount;
        if self.state.mantra >= 10 {
            self.state.mantra -= 10;
            self.change_stance(Stance::Divinity);
        }
    }

    // =======================================================================
    // Combat End Check
    // =======================================================================

    fn check_combat_end(&mut self) -> bool {
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

        let hand_size_before = engine.state.hand.len(); // Should be 5

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
        // Rushdown drew 2 cards: hand was 4 (after playing Eruption), now 6
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

        let hand_before = engine.state.hand.len(); // 5

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

        // In Calm: draws 3 (base_magic=3). Hand was 4 after playing, now 7
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
