//! Combat engine — core turn loop for MCTS simulations.
//!
//! Mirrors the Python CombatEngine. Handles the full Watcher card set:
//! - State management (energy, HP, block, hand, draw/discard)
//! - Damage/block calculation with all modifiers
//! - All Watcher card plays (75+ cards)
//! - End turn (enemy attacks, block decay, debuff ticks)
//! - get_legal_actions()
//!
//! Complex effects (scry choices, discover) are approximated for MCTS speed.

use pyo3::prelude::*;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::actions::{Action, PyAction};
use crate::cards::{CardDef, CardRegistry, CardTarget, CardType};
use crate::damage;
use crate::powers;
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

    /// Start combat: shuffle draw pile, draw initial hand.
    pub fn start_combat(&mut self) {
        if self.phase != CombatPhase::NotStarted {
            return;
        }

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
                actions.push(Action::UsePotion {
                    potion_idx: pot_idx,
                    target_idx: -1,
                });
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

        // Blasphemy: die at start of next turn
        if self.state.die_next_turn {
            self.state.player.hp = 0;
            self.state.combat_over = true;
            self.state.player_won = false;
            self.phase = CombatPhase::CombatOver;
            return;
        }

        // Divinity auto-exit at start of turn
        if self.state.stance == Stance::Divinity {
            self.change_stance(Stance::Neutral);
        }

        // Reset energy (includes Deva Form stacking)
        self.state.energy = self.state.max_energy + self.state.deva_form_energy;

        // Reset turn counters
        self.state.cards_played_this_turn = 0;
        self.state.attacks_played_this_turn = 0;
        self.state.last_card_type = None;
        self.state.wreath_of_flame_bonus = 0;

        // Block decay (simplified: no Barricade/Blur/Calipers in fast path)
        self.state.player.block = 0;

        // Devotion: gain mantra each turn
        let devotion_amount = *self.state.powers_applied.get("Devotion").unwrap_or(&0);
        if devotion_amount > 0 {
            self.gain_mantra(devotion_amount);
        }

        // Draw cards (default 5)
        self.draw_cards(5);
    }

    fn end_turn(&mut self) {
        if self.phase != CombatPhase::PlayerTurn {
            return;
        }

        // Discard hand (respecting retain)
        let hand = std::mem::take(&mut self.state.hand);
        for card_id in hand {
            let card = self.card_registry.get_or_default(&card_id);
            if card.retain {
                self.state.hand.push(card_id);
            } else {
                self.state.discard_pile.push(card_id);
            }
        }

        // Like Water: if in Calm, gain block
        let like_water = *self.state.powers_applied.get("LikeWater").unwrap_or(&0);
        if like_water > 0 && self.state.stance == Stance::Calm {
            let dex = self.state.player.dexterity();
            let frail = self.state.player.is_frail();
            let block = damage::calculate_block(like_water, dex, frail);
            self.state.player.block += block;
        }

        // Omega: deal damage to all enemies at end of turn
        let omega_damage = *self.state.powers_applied.get("Omega").unwrap_or(&0);
        if omega_damage > 0 {
            let living = self.state.living_enemy_indices();
            for enemy_idx in living {
                self.deal_damage_to_enemy(enemy_idx, omega_damage);
            }
            if self.check_combat_end() {
                return;
            }
        }

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

        // Signature Move: only playable if this is the only attack in hand
        if card.effects.contains(&"only_attack_in_hand") {
            let attack_count = self.state.hand.iter().filter(|c| {
                let def = self.card_registry.get_or_default(c);
                def.card_type == CardType::Attack
            }).count();
            if attack_count > 1 {
                return false;
            }
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

        // Pay energy (X-cost pays all energy)
        let cost = if card.cost == -1 {
            self.state.energy
        } else {
            self.effective_cost(&card, &card_id)
        };
        let x_amount = if card.cost == -1 { self.state.energy } else { 0 };
        self.state.energy -= cost;

        // Remove from hand
        self.state.hand.remove(hand_idx);

        // Track counters
        self.state.cards_played_this_turn += 1;
        self.state.total_cards_played += 1;
        if card.card_type == CardType::Attack {
            self.state.attacks_played_this_turn += 1;
        }

        // Execute effects
        self.execute_card_effects(&card, &card_id, target_idx, x_amount);

        // Stance change (from card definition, not from effects)
        if let Some(stance_name) = card.enter_stance {
            let new_stance = Stance::from_str(stance_name);
            self.change_stance(new_stance);
        }

        // Track last card type for conditional effects (Follow-Up, Sash Whip, etc.)
        self.state.last_card_type = Some(card.card_type);

        // Card destination: exhaust, shuffle back, or discard
        // Power cards don't go anywhere (they're "played")
        if card.card_type != CardType::Power {
            if card.exhaust {
                self.state.exhaust_pile.push(card_id);
            } else if card.shuffle_back {
                self.state.draw_pile.push(card_id);
                self.state.draw_pile.shuffle(&mut self.rng);
            } else {
                self.state.discard_pile.push(card_id);
            }
        }

        // Check combat end after card play
        self.check_combat_end();

        // Conclude effect: end turn after playing
        if card.effects.contains(&"end_turn") && !self.state.combat_over {
            self.end_turn();
        }
    }

    fn execute_card_effects(&mut self, card: &CardDef, _card_id: &str, target_idx: i32, x_amount: i32) {
        let effects = card.effects;

        // ---- Pre-damage effects ----

        // Exit stance (Empty Fist, Empty Body, Empty Mind)
        if effects.contains(&"exit_stance") && card.enter_stance.is_none() {
            // Cards with exit_stance but no enter_stance: just go to Neutral
            self.change_stance(Stance::Neutral);
        }

        // ---- Damage ----
        if card.base_damage >= 0 {
            let is_multi_hit = effects.contains(&"multi_hit");
            let hits = if is_multi_hit && card.base_magic > 0 {
                card.base_magic
            } else {
                1
            };

            let player_strength = self.state.player.strength();
            let player_weak = self.state.player.is_weak();
            let stance_mult = self.state.stance.outgoing_mult();

            // Wreath of Flame bonus (only first attack after playing it)
            let wreath_bonus = self.state.wreath_of_flame_bonus;
            if wreath_bonus > 0 && card.card_type == CardType::Attack {
                self.state.wreath_of_flame_bonus = 0;
            }

            // Brilliance: add total mantra gained to damage
            let brilliance_bonus = if effects.contains(&"damage_plus_mantra") {
                self.state.total_mantra_gained
            } else {
                0
            };

            let effective_strength = player_strength + wreath_bonus + brilliance_bonus;

            // Bowling Bash: deal damage once per living enemy
            let bowling_bash_hits = if effects.contains(&"damage_per_enemy") {
                self.state.living_enemy_indices().len() as i32
            } else {
                0
            };

            // Ragnarok: damage random enemies magic times
            if effects.contains(&"damage_random_x_times") {
                let living = self.state.living_enemy_indices();
                if !living.is_empty() {
                    for _ in 0..card.base_magic {
                        let living_now = self.state.living_enemy_indices();
                        if living_now.is_empty() {
                            break;
                        }
                        let &rand_idx = living_now.choose(&mut self.rng).unwrap();
                        let enemy_vuln = self.state.enemies[rand_idx].entity.is_vulnerable();
                        let dmg = damage::calculate_damage(
                            card.base_damage, effective_strength, player_weak,
                            stance_mult, enemy_vuln, false,
                        );
                        self.deal_damage_to_enemy(rand_idx, dmg);
                    }
                }
                // Skip normal damage path
            } else {
                match card.target {
                    CardTarget::Enemy => {
                        if target_idx >= 0 && (target_idx as usize) < self.state.enemies.len() {
                            let enemy_vuln = self.state.enemies[target_idx as usize]
                                .entity
                                .is_vulnerable();
                            let dmg = damage::calculate_damage(
                                card.base_damage, effective_strength, player_weak,
                                stance_mult, enemy_vuln, false,
                            );

                            let total_hits = if bowling_bash_hits > 0 {
                                bowling_bash_hits
                            } else {
                                hits
                            };

                            for _ in 0..total_hits {
                                let actual_dmg = self.deal_damage_to_enemy(target_idx as usize, dmg);

                                // Wallop: gain block equal to unblocked damage dealt
                                if effects.contains(&"gain_block_equal_damage") && actual_dmg > 0 {
                                    self.state.player.block += actual_dmg;
                                }

                                if self.state.enemies[target_idx as usize].entity.is_dead() {
                                    break;
                                }
                            }

                            // Talk to the Hand: apply Block Return
                            if effects.contains(&"apply_block_return") && card.base_magic > 0 {
                                self.state.enemies[target_idx as usize]
                                    .entity
                                    .add_status("BlockReturn", card.base_magic);
                            }

                            // (Mark application moved outside damage block for Pressure Points)
                        }
                    }
                    CardTarget::AllEnemy => {
                        let living = self.state.living_enemy_indices();
                        for enemy_idx in living {
                            let enemy_vuln = self.state.enemies[enemy_idx].entity.is_vulnerable();
                            let dmg = damage::calculate_damage(
                                card.base_damage, effective_strength, player_weak,
                                stance_mult, enemy_vuln, false,
                            );
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
        }

        // Pressure Points: apply Mark to target (outside damage block since it's a Skill)
        if effects.contains(&"apply_mark") && card.base_magic > 0 {
            if target_idx >= 0 && (target_idx as usize) < self.state.enemies.len() {
                self.state.enemies[target_idx as usize]
                    .entity
                    .add_status("Mark", card.base_magic);
            }
        }

        // Pressure Points: trigger all Marks (after applying new marks)
        if effects.contains(&"trigger_marks") {
            let living = self.state.living_enemy_indices();
            for enemy_idx in living {
                let mark = self.state.enemies[enemy_idx].entity.status("Mark");
                if mark > 0 {
                    self.deal_damage_to_enemy(enemy_idx, mark);
                }
            }
        }

        // ---- Block ----
        if card.base_block >= 0 {
            let dex = self.state.player.dexterity();
            let frail = self.state.player.is_frail();
            let block = damage::calculate_block(card.base_block, dex, frail);
            self.state.player.block += block;
        }

        // Extra block in Wrath (Halt)
        if effects.contains(&"extra_block_in_wrath") && self.state.stance == Stance::Wrath {
            if card.base_magic > 0 {
                let dex = self.state.player.dexterity();
                let frail = self.state.player.is_frail();
                let extra = damage::calculate_block(card.base_magic, dex, frail);
                self.state.player.block += extra;
            }
        }

        // Spirit Shield: gain block per card in hand
        if effects.contains(&"block_per_card_in_hand") && card.base_magic > 0 {
            let cards_in_hand = self.state.hand.len() as i32;
            let dex = self.state.player.dexterity();
            let frail = self.state.player.is_frail();
            let block = damage::calculate_block(card.base_magic * cards_in_hand, dex, frail);
            self.state.player.block += block;
        }

        // ---- Draw ----
        if effects.contains(&"draw_1") {
            self.draw_cards(1);
        }
        if effects.contains(&"draw_2") {
            self.draw_cards(2);
        }
        if effects.contains(&"draw_cards") && card.base_magic > 0 {
            self.draw_cards(card.base_magic);
        }
        if effects.contains(&"draw_until_full") {
            let to_draw = (10 - self.state.hand.len() as i32).max(0);
            if to_draw > 0 {
                self.draw_cards(to_draw);
            }
        }

        // ---- Energy effects ----
        if effects.contains(&"gain_energy") && card.base_magic > 0 {
            self.state.energy += card.base_magic;
        }
        if effects.contains(&"energy_if_last_attack") {
            if self.state.last_card_type == Some(CardType::Attack) {
                self.state.energy += 1;
            }
        }

        // ---- Debuff effects ----
        if effects.contains(&"vuln_if_last_skill") {
            if self.state.last_card_type == Some(CardType::Skill) && card.base_magic > 0 {
                if target_idx >= 0 && (target_idx as usize) < self.state.enemies.len() {
                    powers::apply_debuff(
                        &mut self.state.enemies[target_idx as usize].entity,
                        "Vulnerable",
                        card.base_magic,
                    );
                }
            }
        }
        if effects.contains(&"weak_if_last_attack") {
            if self.state.last_card_type == Some(CardType::Attack) && card.base_magic > 0 {
                if target_idx >= 0 && (target_idx as usize) < self.state.enemies.len() {
                    powers::apply_debuff(
                        &mut self.state.enemies[target_idx as usize].entity,
                        "Weakened",
                        card.base_magic,
                    );
                }
            }
        }

        // Indignation: if in Wrath apply Vuln to all, else enter Wrath
        if effects.contains(&"if_wrath_vuln_all_else_wrath") {
            if self.state.stance == Stance::Wrath {
                let living = self.state.living_enemy_indices();
                for enemy_idx in living {
                    powers::apply_debuff(
                        &mut self.state.enemies[enemy_idx].entity,
                        "Vulnerable",
                        card.base_magic.max(1),
                    );
                }
            } else {
                self.change_stance(Stance::Wrath);
            }
        }

        // ---- Conditional stance effects ----

        // Fear No Evil: if target is attacking, enter Calm
        if effects.contains(&"calm_if_enemy_attacking") {
            if target_idx >= 0 && (target_idx as usize) < self.state.enemies.len() {
                if self.state.enemies[target_idx as usize].is_attacking() {
                    self.change_stance(Stance::Calm);
                }
            }
        }

        // Inner Peace: if in Calm draw, else enter Calm
        if effects.contains(&"if_calm_draw_else_calm") {
            if self.state.stance == Stance::Calm {
                let draw_amount = card.base_magic.max(1);
                self.draw_cards(draw_amount);
            } else {
                self.change_stance(Stance::Calm);
            }
        }

        // Sanctity: draw 2 if last card was a Skill
        if effects.contains(&"draw_2_if_last_skill") {
            if self.state.last_card_type == Some(CardType::Skill) {
                self.draw_cards(2);
            }
        }

        // ---- Mantra effects ----
        if effects.contains(&"mantra") && card.base_magic > 0 {
            self.gain_mantra(card.base_magic);
        }

        // ---- Power application effects ----
        if effects.contains(&"on_stance_change_block") && card.base_magic > 0 {
            let current = *self.state.powers_applied.get("MentalFortress").unwrap_or(&0);
            self.state.powers_applied.insert("MentalFortress".to_string(), current + card.base_magic);
        }
        if effects.contains(&"on_wrath_draw") && card.base_magic > 0 {
            let current = *self.state.powers_applied.get("Rushdown").unwrap_or(&0);
            self.state.powers_applied.insert("Rushdown".to_string(), current + card.base_magic);
        }
        if effects.contains(&"on_scry_block") && card.base_magic > 0 {
            let current = *self.state.powers_applied.get("Nirvana").unwrap_or(&0);
            self.state.powers_applied.insert("Nirvana".to_string(), current + card.base_magic);
        }
        if effects.contains(&"calm_end_turn_block") && card.base_magic > 0 {
            let current = *self.state.powers_applied.get("LikeWater").unwrap_or(&0);
            self.state.powers_applied.insert("LikeWater".to_string(), current + card.base_magic);
        }
        if effects.contains(&"gain_mantra_each_turn") && card.base_magic > 0 {
            let current = *self.state.powers_applied.get("Devotion").unwrap_or(&0);
            self.state.powers_applied.insert("Devotion".to_string(), current + card.base_magic);
        }
        if effects.contains(&"gain_energy_stacking") {
            self.state.deva_form_energy += card.base_magic.max(1);
        }
        if effects.contains(&"deal_damage_end_turn") && card.base_magic > 0 {
            let current = *self.state.powers_applied.get("Omega").unwrap_or(&0);
            self.state.powers_applied.insert("Omega".to_string(), current + card.base_magic);
        }

        // Fasting: gain Strength and Dexterity
        if effects.contains(&"gain_str_dex") && card.base_magic > 0 {
            self.state.player.add_status("Strength", card.base_magic);
            self.state.player.add_status("Dexterity", card.base_magic);
        }

        // Wreath of Flame: next attack deals +magic damage
        if effects.contains(&"next_attack_plus_damage") && card.base_magic > 0 {
            self.state.wreath_of_flame_bonus += card.base_magic;
        }

        // Wave of the Hand: apply Weak to all enemies (simplified)
        if effects.contains(&"block_gain_applies_weak") && card.base_magic > 0 {
            let living = self.state.living_enemy_indices();
            for enemy_idx in living {
                powers::apply_debuff(
                    &mut self.state.enemies[enemy_idx].entity,
                    "Weakened",
                    card.base_magic,
                );
            }
        }

        // Blasphemy: die next turn
        if effects.contains(&"die_next_turn") {
            self.state.die_next_turn = true;
        }

        // Judgement: if enemy HP <= magic, kill it
        if effects.contains(&"if_hp_below_kill") && card.base_magic > 0 {
            if target_idx >= 0 && (target_idx as usize) < self.state.enemies.len() {
                let enemy = &mut self.state.enemies[target_idx as usize];
                if enemy.entity.hp <= card.base_magic && enemy.is_alive() {
                    let hp = enemy.entity.hp;
                    enemy.entity.hp = 0;
                    self.state.total_damage_dealt += hp;
                }
            }
        }

        // Collect: put X Miracles on top of draw pile
        if effects.contains(&"put_x_miracles_on_draw") && x_amount > 0 {
            for _ in 0..x_amount {
                self.state.draw_pile.push("Miracle".to_string());
            }
        }

        // Add generated cards to hand/draw
        if effects.contains(&"add_smite_to_hand") {
            if self.state.hand.len() < 10 {
                self.state.hand.push("Smite".to_string());
            }
        }
        if effects.contains(&"add_safety_to_hand") {
            if self.state.hand.len() < 10 {
                self.state.hand.push("Safety".to_string());
            }
        }
        if effects.contains(&"add_insight_to_draw") {
            self.state.draw_pile.push("Insight".to_string());
        }
        if effects.contains(&"add_through_violence_to_draw") {
            self.state.draw_pile.push("ThroughViolence".to_string());
        }
        if effects.contains(&"shuffle_beta_into_draw") {
            self.state.draw_pile.push("Beta".to_string());
            self.state.draw_pile.shuffle(&mut self.rng);
        }
        if effects.contains(&"shuffle_omega_into_draw") {
            self.state.draw_pile.push("Omega".to_string());
            self.state.draw_pile.shuffle(&mut self.rng);
        }
    }

    // =======================================================================
    // Mantra
    // =======================================================================

    fn gain_mantra(&mut self, amount: i32) {
        self.state.mantra += amount;
        self.state.total_mantra_gained += amount;

        // At 10+ mantra, enter Divinity
        if self.state.mantra >= 10 {
            self.state.mantra -= 10;
            self.change_stance(Stance::Divinity);
        }
    }

    // =======================================================================
    // Potion Use
    // =======================================================================

    fn use_potion(&mut self, potion_idx: usize, _target_idx: i32) {
        if potion_idx >= self.state.potions.len() {
            return;
        }
        if self.state.potions[potion_idx].is_empty() {
            return;
        }

        // Remove potion (consume the slot)
        let _potion_id = std::mem::take(&mut self.state.potions[potion_idx]);

        // Potion effects are complex — stub for now.
        // The Python engine handles full potion effects.
    }

    // =======================================================================
    // Damage Dealing / Taking
    // =======================================================================

    /// Deal damage to an enemy. Returns actual HP damage dealt (after block).
    fn deal_damage_to_enemy(&mut self, enemy_idx: usize, damage: i32) -> i32 {
        let enemy = &mut self.state.enemies[enemy_idx];
        let blocked = enemy.entity.block.min(damage);
        let hp_damage = damage - blocked;
        enemy.entity.block -= blocked;
        enemy.entity.hp -= hp_damage;
        self.state.total_damage_dealt += hp_damage;

        if enemy.entity.hp <= 0 {
            enemy.entity.hp = 0;
        }

        hp_damage
    }

    fn deal_damage_to_player(&mut self, damage: i32) {
        let player = &mut self.state.player;
        let blocked = player.block.min(damage);
        let hp_damage = damage - blocked;
        player.block -= blocked;
        player.hp -= hp_damage;
        self.state.total_damage_taken += hp_damage;

        if player.hp <= 0 {
            player.hp = 0;
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
            for _ in 0..hits {
                let mut hit_damage = damage_f * stance_mult;

                // Vulnerable
                if player_vuln {
                    hit_damage *= damage::VULN_MULT;
                }

                // Floor
                let final_damage = (hit_damage as i32).max(0);

                self.deal_damage_to_player(final_damage);

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

        // Apply move effects (simplified)
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

        // Exit Calm: gain 2 energy
        if old_stance == Stance::Calm {
            self.state.energy += 2;
        }

        self.state.stance = new_stance;

        // Mental Fortress: gain block on stance change
        let mental_fortress = *self.state.powers_applied.get("MentalFortress").unwrap_or(&0);
        if mental_fortress > 0 {
            self.state.player.block += mental_fortress;
        }

        // Rushdown: draw cards on entering Wrath
        if new_stance == Stance::Wrath {
            let rushdown = *self.state.powers_applied.get("Rushdown").unwrap_or(&0);
            if rushdown > 0 {
                self.draw_cards(rushdown);
            }
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

    /// Helper: create engine with specific cards in hand.
    fn make_engine_with_hand(hand: Vec<&str>, enemy_hp: i32) -> CombatEngine {
        let mut state = CombatState::new(80, 80, vec![], vec![], 3);
        state.hand = hand.iter().map(|s| s.to_string()).collect();
        let mut enemy = EnemyCombatState::new("Test", enemy_hp, enemy_hp);
        enemy.set_move(1, 10, 1, 0);
        state.enemies = vec![enemy];
        let mut engine = CombatEngine::new(state, 42);
        engine.phase = CombatPhase::PlayerTurn;
        engine.state.turn = 1;
        engine
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
            "Eruption".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
            "Strike_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        let initial_energy = engine.state.energy;

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
        state.enemies[0].entity.hp = 5;

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

        assert!(engine.state.combat_over);
        assert!(engine.state.player_won);
    }

    #[test]
    fn test_player_death_ends_combat() {
        let mut state = make_test_state();
        state.enemies[0].move_damage = 100;

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

        // Strike in Wrath: 6 * 2.0 = 12
        assert_eq!(engine.state.enemies[0].entity.hp, initial_hp - 12);
    }

    #[test]
    fn test_wrath_doubles_incoming_damage() {
        let mut state = make_test_state();
        state.stance = Stance::Wrath;

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

        assert_eq!(engine.state.stance, Stance::Wrath);

        let initial_hp = engine.state.player.hp;
        engine.execute_action(&Action::EndTurn);

        // Enemy: 11 * 2.0 (Wrath incoming) = 22 damage
        assert_eq!(engine.state.player.hp, initial_hp - 22);
    }

    #[test]
    fn test_shuffle_on_empty_draw() {
        let mut state = make_test_state();
        state.draw_pile = vec!["Strike_P".to_string()];
        state.discard_pile = vec![
            "Defend_P".to_string(),
            "Defend_P".to_string(),
            "Defend_P".to_string(),
            "Defend_P".to_string(),
        ];

        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();

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

    // ======================================================================
    // New card effect tests
    // ======================================================================

    #[test]
    fn test_consecrate_hits_all_enemies() {
        let mut engine = make_engine_with_hand(vec!["Consecrate"], 50);
        // Add a second enemy
        let mut enemy2 = EnemyCombatState::new("Test2", 50, 50);
        enemy2.set_move(1, 5, 1, 0);
        engine.state.enemies.push(enemy2);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // Both enemies take 5 damage
        assert_eq!(engine.state.enemies[0].entity.hp, 45);
        assert_eq!(engine.state.enemies[1].entity.hp, 45);
        assert_eq!(engine.state.energy, 3); // 0 cost
    }

    #[test]
    fn test_flying_sleeves_hits_twice() {
        let mut engine = make_engine_with_hand(vec!["FlyingSleeves"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        // 4 damage * 2 hits = 8 total
        assert_eq!(engine.state.enemies[0].entity.hp, 42);
    }

    #[test]
    fn test_tantrum_hits_multiple_and_wrath() {
        let mut engine = make_engine_with_hand(vec!["Tantrum"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        // 3 damage * 3 hits = 9 total, enters Wrath
        assert_eq!(engine.state.enemies[0].entity.hp, 41);
        assert_eq!(engine.state.stance, Stance::Wrath);
        // Tantrum shuffles back into draw pile
        assert!(engine.state.draw_pile.contains(&"Tantrum".to_string()));
    }

    #[test]
    fn test_bowling_bash_damage_per_enemy() {
        let mut engine = make_engine_with_hand(vec!["BowlingBash"], 50);
        // Add second and third enemies
        let mut e2 = EnemyCombatState::new("E2", 50, 50);
        e2.set_move(1, 5, 1, 0);
        let mut e3 = EnemyCombatState::new("E3", 50, 50);
        e3.set_move(1, 5, 1, 0);
        engine.state.enemies.push(e2);
        engine.state.enemies.push(e3);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        // 7 damage * 3 enemies = 21 damage to target
        assert_eq!(engine.state.enemies[0].entity.hp, 29);
    }

    #[test]
    fn test_halt_extra_block_in_wrath() {
        let mut engine = make_engine_with_hand(vec!["Halt"], 50);
        engine.state.stance = Stance::Wrath;

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // 3 base block + 9 extra in Wrath = 12 total
        assert_eq!(engine.state.player.block, 12);
    }

    #[test]
    fn test_halt_no_extra_outside_wrath() {
        let mut engine = make_engine_with_hand(vec!["Halt"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // 3 base block only
        assert_eq!(engine.state.player.block, 3);
    }

    #[test]
    fn test_follow_up_energy_if_last_attack() {
        let mut engine = make_engine_with_hand(vec!["Strike_P", "FollowUp"], 50);

        // Play Strike first
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });
        assert_eq!(engine.state.energy, 2); // 3 - 1 = 2

        // Play Follow-Up: last card was attack, so gain 1 energy
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });
        // 2 - 1 (cost) + 1 (bonus) = 2
        assert_eq!(engine.state.energy, 2);
    }

    #[test]
    fn test_follow_up_no_energy_if_last_skill() {
        let mut engine = make_engine_with_hand(vec!["Defend_P", "FollowUp"], 50);

        // Play Defend first (Skill)
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });
        assert_eq!(engine.state.energy, 2);

        // Play Follow-Up: last card was Skill, no bonus
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });
        assert_eq!(engine.state.energy, 1); // 2 - 1 = 1
    }

    #[test]
    fn test_crush_joints_vuln_if_last_skill() {
        let mut engine = make_engine_with_hand(vec!["Defend_P", "CrushJoints"], 50);

        // Play Defend (Skill)
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // Play Crush Joints: last card was Skill, apply 1 Vulnerable
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });
        assert_eq!(engine.state.enemies[0].entity.status("Vulnerable"), 1);
    }

    #[test]
    fn test_sash_whip_weak_if_last_attack() {
        let mut engine = make_engine_with_hand(vec!["Strike_P", "SashWhip"], 50);

        // Play Strike (Attack)
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        // Play Sash Whip: last card was Attack, apply 1 Weak
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });
        assert_eq!(engine.state.enemies[0].entity.status("Weakened"), 1);
    }

    #[test]
    fn test_prostrate_mantra_and_block() {
        let mut engine = make_engine_with_hand(vec!["Prostrate"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        assert_eq!(engine.state.player.block, 4);
        assert_eq!(engine.state.mantra, 2);
    }

    #[test]
    fn test_mantra_triggers_divinity() {
        let mut engine = make_engine_with_hand(vec!["Prostrate", "Prostrate", "Prostrate", "Prostrate", "Prostrate"], 50);

        // Play Prostrate 5 times (2 mantra each = 10 total -> Divinity)
        for _ in 0..5 {
            engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });
        }

        assert_eq!(engine.state.stance, Stance::Divinity);
        assert_eq!(engine.state.mantra, 0); // 10 consumed
    }

    #[test]
    fn test_crescendo_enter_wrath_retain_exhaust() {
        let mut engine = make_engine_with_hand(vec!["Crescendo"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        assert_eq!(engine.state.stance, Stance::Wrath);
        assert!(engine.state.exhaust_pile.contains(&"Crescendo".to_string()));
        assert_eq!(engine.state.energy, 2); // Cost 1
    }

    #[test]
    fn test_tranquility_enter_calm() {
        let mut engine = make_engine_with_hand(vec!["ClearTheMind"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        assert_eq!(engine.state.stance, Stance::Calm);
        assert!(engine.state.exhaust_pile.contains(&"ClearTheMind".to_string()));
    }

    #[test]
    fn test_empty_fist_exit_stance() {
        let mut engine = make_engine_with_hand(vec!["EmptyFist"], 50);
        engine.state.stance = Stance::Calm;

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        // Exit Calm -> Neutral, gain +2 energy
        assert_eq!(engine.state.stance, Stance::Neutral);
        // 3 - 1 (cost) + 2 (calm exit) = 4
        assert_eq!(engine.state.energy, 4);
        // Deals 9 damage
        assert_eq!(engine.state.enemies[0].entity.hp, 41);
    }

    #[test]
    fn test_empty_mind_draw_and_exit() {
        let mut engine = make_engine_with_hand(vec!["EmptyMind"], 50);
        engine.state.stance = Stance::Calm;
        engine.state.draw_pile = vec!["Strike_P".to_string(), "Defend_P".to_string()];

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        assert_eq!(engine.state.stance, Stance::Neutral);
        // Drew 2 cards
        assert_eq!(engine.state.hand.len(), 2);
        // Got +2 energy from calm exit
        assert_eq!(engine.state.energy, 4); // 3 - 1 + 2
    }

    #[test]
    fn test_conclude_hits_all_and_ends_turn() {
        let mut engine = make_engine_with_hand(vec!["Conclude"], 50);
        let mut e2 = EnemyCombatState::new("E2", 50, 50);
        e2.set_move(1, 5, 1, 0);
        engine.state.enemies.push(e2);
        engine.state.draw_pile = vec![
            "Strike_P".to_string(), "Strike_P".to_string(),
            "Strike_P".to_string(), "Strike_P".to_string(),
            "Strike_P".to_string(),
        ];

        let initial_hp = engine.state.player.hp;
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // Both enemies take 12 damage
        assert_eq!(engine.state.enemies[0].entity.hp, 38);
        assert_eq!(engine.state.enemies[1].entity.hp, 38);
        // Turn ended (enemies attacked)
        assert!(engine.state.player.hp < initial_hp);
        assert_eq!(engine.state.turn, 2);
    }

    #[test]
    fn test_fear_no_evil_calm_if_attacking() {
        let mut engine = make_engine_with_hand(vec!["FearNoEvil"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        // Enemy is attacking (move_damage > 0), so enter Calm
        assert_eq!(engine.state.stance, Stance::Calm);
        assert_eq!(engine.state.enemies[0].entity.hp, 42); // 8 damage
    }

    #[test]
    fn test_fear_no_evil_no_calm_if_not_attacking() {
        let mut engine = make_engine_with_hand(vec!["FearNoEvil"], 50);
        engine.state.enemies[0].set_move(1, 0, 0, 10); // Block move, no attack

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        // Enemy not attacking, stay Neutral
        assert_eq!(engine.state.stance, Stance::Neutral);
    }

    #[test]
    fn test_inner_peace_draw_in_calm() {
        let mut engine = make_engine_with_hand(vec!["InnerPeace"], 50);
        engine.state.stance = Stance::Calm;
        engine.state.draw_pile = vec![
            "Strike_P".to_string(), "Defend_P".to_string(), "Eruption".to_string(),
        ];

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // In Calm: draw 3 cards
        assert_eq!(engine.state.hand.len(), 3);
        assert_eq!(engine.state.stance, Stance::Calm); // Stays in Calm
    }

    #[test]
    fn test_inner_peace_enter_calm_if_not() {
        let mut engine = make_engine_with_hand(vec!["InnerPeace"], 50);
        engine.state.stance = Stance::Neutral;

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // Not in Calm: enter Calm, don't draw
        assert_eq!(engine.state.stance, Stance::Calm);
        assert_eq!(engine.state.hand.len(), 0);
    }

    #[test]
    fn test_wheel_kick_draws_2() {
        let mut engine = make_engine_with_hand(vec!["WheelKick"], 50);
        engine.state.draw_pile = vec!["Strike_P".to_string(), "Defend_P".to_string()];

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        assert_eq!(engine.state.enemies[0].entity.hp, 35); // 50 - 15
        assert_eq!(engine.state.hand.len(), 2); // Drew 2
    }

    #[test]
    fn test_wallop_gains_block_equal_damage() {
        let mut engine = make_engine_with_hand(vec!["Wallop"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        // 9 damage dealt to enemy, gain 9 block
        assert_eq!(engine.state.enemies[0].entity.hp, 41);
        assert_eq!(engine.state.player.block, 9);
    }

    #[test]
    fn test_wallop_block_respects_enemy_block() {
        let mut engine = make_engine_with_hand(vec!["Wallop"], 50);
        engine.state.enemies[0].entity.block = 5;

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        // 9 damage - 5 enemy block = 4 HP damage, gain 4 block
        assert_eq!(engine.state.enemies[0].entity.hp, 46);
        assert_eq!(engine.state.player.block, 4);
    }

    #[test]
    fn test_mental_fortress_block_on_stance_change() {
        let mut engine = make_engine_with_hand(vec!["Crescendo"], 50);
        engine.state.powers_applied.insert("MentalFortress".to_string(), 4);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // Entering Wrath triggers Mental Fortress: +4 block
        assert_eq!(engine.state.player.block, 4);
    }

    #[test]
    fn test_rushdown_draws_on_wrath() {
        let mut engine = make_engine_with_hand(vec!["Crescendo"], 50);
        engine.state.powers_applied.insert("Rushdown".to_string(), 2);
        engine.state.draw_pile = vec!["Strike_P".to_string(), "Defend_P".to_string()];

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // Entering Wrath triggers Rushdown: draw 2
        assert_eq!(engine.state.hand.len(), 2);
    }

    #[test]
    fn test_judgement_kills_below_threshold() {
        let mut engine = make_engine_with_hand(vec!["Judgement"], 50);
        engine.state.enemies[0].entity.hp = 25; // Below 30 threshold

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        assert_eq!(engine.state.enemies[0].entity.hp, 0);
        assert!(engine.state.combat_over);
        assert!(engine.state.player_won);
    }

    #[test]
    fn test_judgement_does_nothing_above_threshold() {
        let mut engine = make_engine_with_hand(vec!["Judgement"], 50);
        engine.state.enemies[0].entity.hp = 35; // Above 30 threshold

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        assert_eq!(engine.state.enemies[0].entity.hp, 35);
        assert!(!engine.state.combat_over);
    }

    #[test]
    fn test_signature_move_only_if_only_attack() {
        let engine = make_engine_with_hand(vec!["SignatureMove", "Strike_P"], 50);

        let actions = engine.get_legal_actions();
        // SignatureMove should NOT be playable (Strike is also in hand)
        let sig_actions: Vec<_> = actions.iter().filter(|a| {
            matches!(a, Action::PlayCard { card_idx: 0, .. })
        }).collect();
        assert!(sig_actions.is_empty());
    }

    #[test]
    fn test_signature_move_playable_alone() {
        let engine = make_engine_with_hand(vec!["SignatureMove", "Defend_P"], 50);

        let actions = engine.get_legal_actions();
        // SignatureMove IS playable (only attack)
        let sig_actions: Vec<_> = actions.iter().filter(|a| {
            matches!(a, Action::PlayCard { card_idx: 0, .. })
        }).collect();
        assert!(!sig_actions.is_empty());
    }

    #[test]
    fn test_blasphemy_enters_divinity_die_next_turn() {
        let mut engine = make_engine_with_hand(vec!["Blasphemy"], 50);
        engine.state.draw_pile = vec![
            "Strike_P".to_string(), "Strike_P".to_string(),
            "Strike_P".to_string(), "Strike_P".to_string(),
            "Strike_P".to_string(),
        ];

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        assert_eq!(engine.state.stance, Stance::Divinity);
        assert!(engine.state.die_next_turn);

        // End turn -> enemy attacks -> next turn starts -> die
        engine.execute_action(&Action::EndTurn);

        assert!(engine.state.combat_over);
        assert!(!engine.state.player_won);
        assert_eq!(engine.state.player.hp, 0);
    }

    #[test]
    fn test_wreath_of_flame_bonus() {
        let mut engine = make_engine_with_hand(vec!["WreathOfFlame", "Strike_P"], 50);

        // Play Wreath of Flame (+5 to next attack)
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // Play Strike: 6 + 5 = 11 damage
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });
        assert_eq!(engine.state.enemies[0].entity.hp, 39);
        // Bonus consumed
        assert_eq!(engine.state.wreath_of_flame_bonus, 0);
    }

    #[test]
    fn test_spirit_shield_block_per_card() {
        let mut engine = make_engine_with_hand(
            vec!["SpiritShield", "Strike_P", "Defend_P", "Eruption"], 50
        );

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // 3 cards remaining in hand * 3 block each = 9 block
        assert_eq!(engine.state.player.block, 9);
    }

    #[test]
    fn test_scrawl_draws_until_full() {
        let mut engine = make_engine_with_hand(vec!["Scrawl", "Strike_P"], 50);
        // 8 cards needed to fill hand to 10
        for _ in 0..10 {
            engine.state.draw_pile.push("Defend_P".to_string());
        }

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // Hand should be full (10 cards): 1 remaining (Strike_P) + 9 drawn
        // But Scrawl was played (removed from hand), so 1 card + draw until 10
        assert_eq!(engine.state.hand.len(), 10);
    }

    #[test]
    fn test_miracle_gains_energy() {
        let mut engine = make_engine_with_hand(vec!["Miracle"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // 0 cost + gain 1 energy
        assert_eq!(engine.state.energy, 4);
        // Exhausted
        assert!(engine.state.exhaust_pile.contains(&"Miracle".to_string()));
    }

    #[test]
    fn test_indignation_wrath_when_not_in_wrath() {
        let mut engine = make_engine_with_hand(vec!["Indignation"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        assert_eq!(engine.state.stance, Stance::Wrath);
    }

    #[test]
    fn test_indignation_vuln_all_when_in_wrath() {
        let mut engine = make_engine_with_hand(vec!["Indignation"], 50);
        engine.state.stance = Stance::Wrath;
        let mut e2 = EnemyCombatState::new("E2", 50, 50);
        e2.set_move(1, 5, 1, 0);
        engine.state.enemies.push(e2);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        assert_eq!(engine.state.enemies[0].entity.status("Vulnerable"), 3);
        assert_eq!(engine.state.enemies[1].entity.status("Vulnerable"), 3);
    }

    #[test]
    fn test_fasting_gains_str_dex() {
        let mut engine = make_engine_with_hand(vec!["Fasting2"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        assert_eq!(engine.state.player.status("Strength"), 3);
        assert_eq!(engine.state.player.status("Dexterity"), 3);
    }

    #[test]
    fn test_deva_form_stacking_energy() {
        let mut engine = make_engine_with_hand(vec!["DevaForm"], 50);
        engine.state.draw_pile = (0..10).map(|_| "Strike_P".to_string()).collect();

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        assert_eq!(engine.state.deva_form_energy, 1);

        // End turn -> new turn should have max_energy + 1
        engine.execute_action(&Action::EndTurn);

        assert_eq!(engine.state.energy, 4); // 3 + 1
    }

    #[test]
    fn test_pressure_points_mark_and_trigger() {
        let mut engine = make_engine_with_hand(vec!["PathToVictory", "PathToVictory"], 50);

        // First play: apply 8 mark, trigger 8 damage
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });
        assert_eq!(engine.state.enemies[0].entity.status("Mark"), 8);
        assert_eq!(engine.state.enemies[0].entity.hp, 42); // 50 - 8

        // Second play: apply 8 more mark (total 16), trigger 16 damage
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });
        assert_eq!(engine.state.enemies[0].entity.status("Mark"), 16);
        assert_eq!(engine.state.enemies[0].entity.hp, 26); // 42 - 16
    }

    #[test]
    fn test_just_lucky_scry_block_damage() {
        let mut engine = make_engine_with_hand(vec!["JustLucky"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        // 3 damage, 2 block, 0 cost
        assert_eq!(engine.state.enemies[0].entity.hp, 47);
        assert_eq!(engine.state.player.block, 2);
        assert_eq!(engine.state.energy, 3);
    }

    #[test]
    fn test_protect_retain() {
        let reg = CardRegistry::new();
        let protect = reg.get("Protect").unwrap();
        assert!(protect.retain);
        assert_eq!(protect.base_block, 12);
        assert_eq!(protect.cost, 2);
    }

    #[test]
    fn test_like_water_end_turn_block() {
        let mut engine = make_engine_with_hand(vec![], 50);
        engine.state.stance = Stance::Calm;
        engine.state.powers_applied.insert("LikeWater".to_string(), 5);
        engine.state.draw_pile = (0..5).map(|_| "Strike_P".to_string()).collect();

        engine.execute_action(&Action::EndTurn);

        // Like Water grants 5 block in Calm at end of turn
        // But block decays at start of new turn, so check turn 2 block is 0
        // Actually block is applied at end of turn, then enemy turns happen,
        // then new turn starts with block = 0. So the block helps absorb enemy damage.
        // The enemy does 10 damage, 5 blocked = 5 HP lost
        assert_eq!(engine.state.player.hp, 75);
    }

    #[test]
    fn test_devotion_mantra_each_turn() {
        let mut engine = make_engine_with_hand(vec![], 50);
        engine.state.powers_applied.insert("Devotion".to_string(), 3);
        engine.state.draw_pile = (0..20).map(|_| "Strike_P".to_string()).collect();

        // End turn -> starts new turn -> Devotion grants 3 mantra
        engine.execute_action(&Action::EndTurn);

        assert_eq!(engine.state.mantra, 3);
        assert_eq!(engine.state.total_mantra_gained, 3);
    }

    #[test]
    fn test_evaluate_adds_insight() {
        let mut engine = make_engine_with_hand(vec!["Evaluate"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        assert_eq!(engine.state.player.block, 6);
        assert!(engine.state.draw_pile.contains(&"Insight".to_string()));
    }

    #[test]
    fn test_carve_reality_adds_smite() {
        let mut engine = make_engine_with_hand(vec!["CarveReality"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        assert_eq!(engine.state.enemies[0].entity.hp, 44); // 6 damage
        assert!(engine.state.hand.contains(&"Smite".to_string()));
    }

    #[test]
    fn test_deceive_reality_adds_safety() {
        let mut engine = make_engine_with_hand(vec!["DeceiveReality"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        assert_eq!(engine.state.player.block, 4);
        assert!(engine.state.hand.contains(&"Safety".to_string()));
    }

    #[test]
    fn test_reach_heaven_adds_through_violence() {
        let mut engine = make_engine_with_hand(vec!["ReachHeaven"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        assert_eq!(engine.state.enemies[0].entity.hp, 40); // 10 damage
        assert!(engine.state.draw_pile.contains(&"ThroughViolence".to_string()));
    }

    #[test]
    fn test_alpha_shuffles_beta() {
        let mut engine = make_engine_with_hand(vec!["Alpha"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        assert!(engine.state.draw_pile.contains(&"Beta".to_string()));
        assert!(engine.state.exhaust_pile.contains(&"Alpha".to_string()));
    }

    #[test]
    fn test_collect_x_cost_miracles() {
        let mut engine = make_engine_with_hand(vec!["Collect"], 50);
        engine.state.energy = 3;

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // Spent all 3 energy, added 3 Miracles to draw pile
        assert_eq!(engine.state.energy, 0);
        let miracle_count = engine.state.draw_pile.iter().filter(|c| *c == "Miracle").count();
        assert_eq!(miracle_count, 3);
    }

    #[test]
    fn test_brilliance_damage_plus_mantra() {
        let mut engine = make_engine_with_hand(vec!["Brilliance"], 50);
        engine.state.total_mantra_gained = 8;

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        // 12 base + 8 mantra = 20 damage
        assert_eq!(engine.state.enemies[0].entity.hp, 30);
    }

    #[test]
    fn test_talk_to_the_hand_applies_block_return() {
        let mut engine = make_engine_with_hand(vec!["TalkToTheHand"], 50);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: 0 });

        assert_eq!(engine.state.enemies[0].entity.hp, 45); // 5 damage
        assert_eq!(engine.state.enemies[0].entity.status("BlockReturn"), 2);
        assert!(engine.state.exhaust_pile.contains(&"TalkToTheHand".to_string()));
    }

    #[test]
    fn test_wave_of_the_hand_applies_weak() {
        let mut engine = make_engine_with_hand(vec!["WaveOfTheHand"], 50);
        let mut e2 = EnemyCombatState::new("E2", 50, 50);
        e2.set_move(1, 5, 1, 0);
        engine.state.enemies.push(e2);

        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        assert_eq!(engine.state.enemies[0].entity.status("Weakened"), 1);
        assert_eq!(engine.state.enemies[1].entity.status("Weakened"), 1);
    }

    #[test]
    fn test_sanctity_draw_if_last_skill() {
        let mut engine = make_engine_with_hand(vec!["Defend_P", "Sanctity"], 50);
        engine.state.draw_pile = vec!["Strike_P".to_string(), "Eruption".to_string()];

        // Play Defend (Skill)
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });
        // Play Sanctity: last was Skill, draw 2
        engine.execute_action(&Action::PlayCard { card_idx: 0, target_idx: -1 });

        // Block from Sanctity (6) + Defend (5) = 11
        assert_eq!(engine.state.player.block, 11);
        // Drew 2 cards
        assert_eq!(engine.state.hand.len(), 2);
    }

    #[test]
    fn test_omega_deals_damage_end_turn() {
        let mut engine = make_engine_with_hand(vec![], 50);
        engine.state.powers_applied.insert("Omega".to_string(), 50);
        engine.state.draw_pile = (0..5).map(|_| "Strike_P".to_string()).collect();

        engine.execute_action(&Action::EndTurn);

        // Omega deals 50 to all enemies at end of turn
        assert_eq!(engine.state.enemies[0].entity.hp, 0);
        assert!(engine.state.player_won);
    }

    #[test]
    fn test_retain_cards_stay_in_hand() {
        let mut engine = make_engine_with_hand(vec!["Protect", "Strike_P"], 50);
        engine.state.draw_pile = (0..5).map(|_| "Defend_P".to_string()).collect();

        engine.execute_action(&Action::EndTurn);

        // Protect (retain) stays in hand, Strike is discarded
        assert!(engine.state.hand.contains(&"Protect".to_string()));
        assert!(!engine.state.hand.contains(&"Strike_P".to_string()));
    }
}
