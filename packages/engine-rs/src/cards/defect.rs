use std::collections::HashMap;
use super::{CardDef, CardType, CardTarget};

pub fn register_defect(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Defect Basic Cards ----
        insert(cards, CardDef {
            id: "Strike_B", name: "Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Strike_B+", name: "Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Defend_B", name: "Defend", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Defend_B+", name: "Defend+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Zap", name: "Zap", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Zap+", name: "Zap+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Dualcast", name: "Dualcast", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["evoke_orb", "evoke_orb"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Dualcast+", name: "Dualcast+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["evoke_orb", "evoke_orb"], effect_data: &[], complex_hook: None,
        });

        // ---- Defect Common Cards ----
        // Ball Lightning: 1 cost, 7 dmg, channel 1 Lightning
        insert(cards, CardDef {
            id: "Ball Lightning", name: "Ball Lightning", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Ball Lightning+", name: "Ball Lightning+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning"], effect_data: &[], complex_hook: None,
        });
        // Barrage: 1 cost, 4 dmg x orbs
        insert(cards, CardDef {
            id: "Barrage", name: "Barrage", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_per_orb"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Barrage+", name: "Barrage+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_per_orb"], effect_data: &[], complex_hook: None,
        });
        // Beam Cell: 0 cost, 3 dmg, 1 vuln
        insert(cards, CardDef {
            id: "Beam Cell", name: "Beam Cell", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["apply_vulnerable"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Beam Cell+", name: "Beam Cell+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["apply_vulnerable"], effect_data: &[], complex_hook: None,
        });
        // Cold Snap: 1 cost, 6 dmg, channel 1 Frost
        insert(cards, CardDef {
            id: "Cold Snap", name: "Cold Snap", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_frost"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Cold Snap+", name: "Cold Snap+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_frost"], effect_data: &[], complex_hook: None,
        });
        // Compile Driver: 1 cost, 7 dmg, draw 1 per unique orb
        insert(cards, CardDef {
            id: "Compile Driver", name: "Compile Driver", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw_per_unique_orb"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Compile Driver+", name: "Compile Driver+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw_per_unique_orb"], effect_data: &[], complex_hook: None,
        });
        // Conserve Battery: 1 cost, 7 block, next turn gain 1 energy (via Energized)
        insert(cards, CardDef {
            id: "Conserve Battery", name: "Conserve Battery", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_turn_energy"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Conserve Battery+", name: "Conserve Battery+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 10,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_turn_energy"], effect_data: &[], complex_hook: None,
        });
        // Coolheaded: 1 cost, channel Frost, draw 1
        insert(cards, CardDef {
            id: "Coolheaded", name: "Coolheaded", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_frost", "draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Coolheaded+", name: "Coolheaded+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["channel_frost", "draw"], effect_data: &[], complex_hook: None,
        });
        // Go for the Eyes: 0 cost, 3 dmg, apply Weak if attacking
        insert(cards, CardDef {
            id: "Go for the Eyes", name: "Go for the Eyes", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["weak_if_attacking"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Go for the Eyes+", name: "Go for the Eyes+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["weak_if_attacking"], effect_data: &[], complex_hook: None,
        });
        // Hologram: 1 cost, 3 block, put card from discard into hand, exhaust (upgrade: no exhaust)
        insert(cards, CardDef {
            id: "Hologram", name: "Hologram", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 3,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["return_from_discard"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Hologram+", name: "Hologram+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["return_from_discard"], effect_data: &[], complex_hook: None,
        });
        // Leap: 1 cost, 9 block
        insert(cards, CardDef {
            id: "Leap", name: "Leap", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 9,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Leap+", name: "Leap+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        // Rebound: 1 cost, 9 dmg, next card drawn goes to top of draw pile
        insert(cards, CardDef {
            id: "Rebound", name: "Rebound", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_card_to_top"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Rebound+", name: "Rebound+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_card_to_top"], effect_data: &[], complex_hook: None,
        });
        // Stack: 1 cost, block = discard pile size (upgrade: +3)
        insert(cards, CardDef {
            id: "Stack", name: "Stack", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 0,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_from_discard"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Stack+", name: "Stack+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 3,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_from_discard"], effect_data: &[], complex_hook: None,
        });
        // Steam Barrier (SteamBarrier): 0 cost, 6 block, loses 1 block each play
        insert(cards, CardDef {
            id: "Steam", name: "Steam Barrier", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 6,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["lose_block_each_play"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Steam+", name: "Steam Barrier+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["lose_block_each_play"], effect_data: &[], complex_hook: None,
        });
        // Streamline: 2 cost, 15 dmg, costs 1 less each play
        insert(cards, CardDef {
            id: "Streamline", name: "Streamline", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["reduce_cost_each_play"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Streamline+", name: "Streamline+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["reduce_cost_each_play"], effect_data: &[], complex_hook: None,
        });
        // Sweeping Beam: 1 cost, 6 dmg AoE, draw 1
        insert(cards, CardDef {
            id: "Sweeping Beam", name: "Sweeping Beam", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Sweeping Beam+", name: "Sweeping Beam+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[], complex_hook: None,
        });
        // Turbo: 0 cost, gain 2 energy, add Void to discard
        insert(cards, CardDef {
            id: "Turbo", name: "Turbo", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["gain_energy", "add_void_to_discard"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Turbo+", name: "Turbo+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["gain_energy", "add_void_to_discard"], effect_data: &[], complex_hook: None,
        });
        // Claw (Java ID: Gash): 0 cost, 3 dmg, all Claw dmg +2 for rest of combat
        insert(cards, CardDef {
            id: "Gash", name: "Claw", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["claw_scaling"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Gash+", name: "Claw+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 5, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["claw_scaling"], effect_data: &[], complex_hook: None,
        });

        // ---- Defect Uncommon Cards ----
        // Aggregate: 1 cost, gain 1 energy per 4 cards in draw pile (upgrade: per 3)
        insert(cards, CardDef {
            id: "Aggregate", name: "Aggregate", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["energy_per_cards_in_draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Aggregate+", name: "Aggregate+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["energy_per_cards_in_draw"], effect_data: &[], complex_hook: None,
        });
        // Auto Shields: 1 cost, 11 block only if no block
        insert(cards, CardDef {
            id: "Auto Shields", name: "Auto-Shields", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 11,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_if_no_block"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Auto Shields+", name: "Auto-Shields+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 15,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_if_no_block"], effect_data: &[], complex_hook: None,
        });
        // Blizzard: 1 cost, dmg = 2 * frost channeled this combat, AoE
        insert(cards, CardDef {
            id: "Blizzard", name: "Blizzard", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 0, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["damage_per_frost_channeled"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Blizzard+", name: "Blizzard+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 0, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["damage_per_frost_channeled"], effect_data: &[], complex_hook: None,
        });
        // Boot Sequence: 0 cost, 10 block, innate, exhaust
        insert(cards, CardDef {
            id: "BootSequence", name: "Boot Sequence", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 10,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["innate"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "BootSequence+", name: "Boot Sequence+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 13,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["innate"], effect_data: &[], complex_hook: None,
        });
        // Capacitor: 1 cost, power, gain 2 orb slots
        insert(cards, CardDef {
            id: "Capacitor", name: "Capacitor", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["gain_orb_slots"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Capacitor+", name: "Capacitor+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["gain_orb_slots"], effect_data: &[], complex_hook: None,
        });
        // Chaos: 1 cost, channel 1 random orb (upgrade: 2)
        insert(cards, CardDef {
            id: "Chaos", name: "Chaos", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_random"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Chaos+", name: "Chaos+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["channel_random"], effect_data: &[], complex_hook: None,
        });
        // Chill: 0 cost, channel 1 Frost per enemy, exhaust (upgrade: innate)
        insert(cards, CardDef {
            id: "Chill", name: "Chill", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["channel_frost_per_enemy"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Chill+", name: "Chill+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["channel_frost_per_enemy", "innate"], effect_data: &[], complex_hook: None,
        });
        // Consume: 2 cost, remove 1 orb slot, gain 2 focus
        insert(cards, CardDef {
            id: "Consume", name: "Consume", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["gain_focus", "lose_orb_slot"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Consume+", name: "Consume+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["gain_focus", "lose_orb_slot"], effect_data: &[], complex_hook: None,
        });
        // Darkness: 1 cost, channel 1 Dark (upgrade: also trigger Dark passive)
        insert(cards, CardDef {
            id: "Darkness", name: "Darkness", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_dark"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Darkness+", name: "Darkness+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_dark", "trigger_dark_passive"], effect_data: &[], complex_hook: None,
        });
        // Defragment: 1 cost, power, gain 1 focus
        insert(cards, CardDef {
            id: "Defragment", name: "Defragment", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["gain_focus"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Defragment+", name: "Defragment+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["gain_focus"], effect_data: &[], complex_hook: None,
        });
        // Doom and Gloom: 2 cost, 10 dmg AoE, channel 1 Dark
        insert(cards, CardDef {
            id: "Doom and Gloom", name: "Doom and Gloom", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_dark"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Doom and Gloom+", name: "Doom and Gloom+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 14, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_dark"], effect_data: &[], complex_hook: None,
        });
        // Double Energy: 1 cost, double your energy, exhaust (upgrade: cost 0)
        insert(cards, CardDef {
            id: "Double Energy", name: "Double Energy", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["double_energy"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Double Energy+", name: "Double Energy+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["double_energy"], effect_data: &[], complex_hook: None,
        });
        // Equilibrium (Java ID: Undo): 2 cost, 13 block, retain hand this turn
        insert(cards, CardDef {
            id: "Undo", name: "Equilibrium", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 13,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["retain_hand"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Undo+", name: "Equilibrium+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 16,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["retain_hand"], effect_data: &[], complex_hook: None,
        });
        // Force Field: 4 cost, 12 block, costs 1 less per power played
        insert(cards, CardDef {
            id: "Force Field", name: "Force Field", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 4, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["reduce_cost_per_power"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Force Field+", name: "Force Field+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 4, base_damage: -1, base_block: 16,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["reduce_cost_per_power"], effect_data: &[], complex_hook: None,
        });
        // FTL: 0 cost, 5 dmg, draw 1 if <3 cards played this turn
        insert(cards, CardDef {
            id: "FTL", name: "FTL", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 5, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["draw_if_few_cards_played"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "FTL+", name: "FTL+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["draw_if_few_cards_played"], effect_data: &[], complex_hook: None,
        });
        // Fusion: 2 cost, channel 1 Plasma (upgrade: cost 1)
        insert(cards, CardDef {
            id: "Fusion", name: "Fusion", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_plasma"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Fusion+", name: "Fusion+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_plasma"], effect_data: &[], complex_hook: None,
        });
        // Genetic Algorithm: 1 cost, block from misc (starts 0), grows +2 per combat, exhaust
        insert(cards, CardDef {
            id: "Genetic Algorithm", name: "Genetic Algorithm", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 0,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["genetic_algorithm"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Genetic Algorithm+", name: "Genetic Algorithm+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 0,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["genetic_algorithm"], effect_data: &[], complex_hook: None,
        });
        // Glacier: 2 cost, 7 block, channel 2 Frost
        insert(cards, CardDef {
            id: "Glacier", name: "Glacier", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 7,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["channel_frost"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Glacier+", name: "Glacier+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 10,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["channel_frost"], effect_data: &[], complex_hook: None,
        });
        // Heatsinks: 1 cost, power, whenever you play a power draw 1 card
        insert(cards, CardDef {
            id: "Heatsinks", name: "Heatsinks", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw_on_power_play"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Heatsinks+", name: "Heatsinks+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw_on_power_play"], effect_data: &[], complex_hook: None,
        });
        // Hello World: 1 cost, power, add random common card to hand each turn (upgrade: innate)
        insert(cards, CardDef {
            id: "Hello World", name: "Hello World", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["hello_world"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Hello World+", name: "Hello World+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["hello_world", "innate"], effect_data: &[], complex_hook: None,
        });
        // Impulse: 1 cost, trigger all orb passives, exhaust (upgrade: no exhaust)
        insert(cards, CardDef {
            id: "Impulse", name: "Impulse", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["trigger_all_passives"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Impulse+", name: "Impulse+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["trigger_all_passives"], effect_data: &[], complex_hook: None,
        });
        // Lock-On (Java ID: Lockon): 1 cost, 8 dmg, apply 2 Lock-On
        insert(cards, CardDef {
            id: "Lockon", name: "Lock-On", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["apply_lock_on"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Lockon+", name: "Lock-On+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["apply_lock_on"], effect_data: &[], complex_hook: None,
        });
        // Loop: 1 cost, power, trigger frontmost orb passive at start of turn
        insert(cards, CardDef {
            id: "Loop", name: "Loop", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["loop_orb"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Loop+", name: "Loop+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["loop_orb"], effect_data: &[], complex_hook: None,
        });
        // Melter: 1 cost, 10 dmg, remove all enemy block
        insert(cards, CardDef {
            id: "Melter", name: "Melter", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["remove_enemy_block"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Melter+", name: "Melter+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 14, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["remove_enemy_block"], effect_data: &[], complex_hook: None,
        });
        // Overclock (Java ID: Steam Power): 0 cost, draw 2, add Burn to discard
        insert(cards, CardDef {
            id: "Steam Power", name: "Overclock", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw", "add_burn_to_discard"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Steam Power+", name: "Overclock+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["draw", "add_burn_to_discard"], effect_data: &[], complex_hook: None,
        });
        // Recycle: 1 cost, exhaust a card, gain energy equal to its cost (upgrade: cost 0)
        insert(cards, CardDef {
            id: "Recycle", name: "Recycle", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["recycle"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Recycle+", name: "Recycle+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["recycle"], effect_data: &[], complex_hook: None,
        });
        // Recursion (Java ID: Redo): 1 cost, evoke frontmost, channel it back (upgrade: cost 0)
        insert(cards, CardDef {
            id: "Redo", name: "Recursion", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["evoke_orb", "channel_evoked"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Redo+", name: "Recursion+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["evoke_orb", "channel_evoked"], effect_data: &[], complex_hook: None,
        });
        // Reinforced Body: X cost, gain 7 block X times
        insert(cards, CardDef {
            id: "Reinforced Body", name: "Reinforced Body", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_x_times"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Reinforced Body+", name: "Reinforced Body+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: 9,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_x_times"], effect_data: &[], complex_hook: None,
        });
        // Reprogram: 1 cost, lose 1 focus, gain 1 str and 1 dex
        insert(cards, CardDef {
            id: "Reprogram", name: "Reprogram", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["reprogram"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Reprogram+", name: "Reprogram+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["reprogram"], effect_data: &[], complex_hook: None,
        });
        // Rip and Tear: 1 cost, deal 7 dmg twice to random enemies
        insert(cards, CardDef {
            id: "Rip and Tear", name: "Rip and Tear", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["damage_random_x_times"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Rip and Tear+", name: "Rip and Tear+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["damage_random_x_times"], effect_data: &[], complex_hook: None,
        });
        // Scrape: 1 cost, 7 dmg, draw 4 then discard non-0-cost cards drawn
        insert(cards, CardDef {
            id: "Scrape", name: "Scrape", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["draw_discard_non_zero"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Scrape+", name: "Scrape+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["draw_discard_non_zero"], effect_data: &[], complex_hook: None,
        });
        // Self Repair: 1 cost, power, heal 7 HP at end of combat
        insert(cards, CardDef {
            id: "Self Repair", name: "Self Repair", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: false, enter_stance: None,
            effects: &["heal_end_of_combat"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Self Repair+", name: "Self Repair+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 10, exhaust: false, enter_stance: None,
            effects: &["heal_end_of_combat"], effect_data: &[], complex_hook: None,
        });
        // Skim: 1 cost, draw 3 cards
        insert(cards, CardDef {
            id: "Skim", name: "Skim", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Skim+", name: "Skim+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[], complex_hook: None,
        });
        // Static Discharge: 1 cost, power, channel 1 Lightning whenever you take unblocked damage
        insert(cards, CardDef {
            id: "Static Discharge", name: "Static Discharge", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning_on_damage"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Static Discharge+", name: "Static Discharge+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["channel_lightning_on_damage"], effect_data: &[], complex_hook: None,
        });
        // Storm: 1 cost, power, channel 1 Lightning on power play (upgrade: innate)
        insert(cards, CardDef {
            id: "Storm", name: "Storm", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning_on_power"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Storm+", name: "Storm+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning_on_power", "innate"], effect_data: &[], complex_hook: None,
        });
        // Sunder: 3 cost, 24 dmg, gain 3 energy if this kills
        insert(cards, CardDef {
            id: "Sunder", name: "Sunder", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 3, base_damage: 24, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["energy_on_kill"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Sunder+", name: "Sunder+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 3, base_damage: 32, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["energy_on_kill"], effect_data: &[], complex_hook: None,
        });
        // Tempest: X cost, channel X Lightning orbs, exhaust (upgrade: +1)
        insert(cards, CardDef {
            id: "Tempest", name: "Tempest", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["channel_lightning_x"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Tempest+", name: "Tempest+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["channel_lightning_x_plus_1"], effect_data: &[], complex_hook: None,
        });
        // White Noise: 1 cost, add random Power to hand, exhaust (upgrade: cost 0)
        insert(cards, CardDef {
            id: "White Noise", name: "White Noise", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_random_power"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "White Noise+", name: "White Noise+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_random_power"], effect_data: &[], complex_hook: None,
        });

        // ---- Defect Rare Cards ----
        // All For One: 2 cost, 10 dmg, return all 0-cost cards from discard to hand
        insert(cards, CardDef {
            id: "All For One", name: "All For One", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["return_zero_cost_from_discard"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "All For One+", name: "All For One+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 14, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["return_zero_cost_from_discard"], effect_data: &[], complex_hook: None,
        });
        // Amplify: 1 cost, next power played this turn is played twice
        insert(cards, CardDef {
            id: "Amplify", name: "Amplify", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["amplify_power"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Amplify+", name: "Amplify+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["amplify_power"], effect_data: &[], complex_hook: None,
        });
        // Biased Cognition: 1 cost, power, gain 4 focus, lose 1 focus each turn
        insert(cards, CardDef {
            id: "Biased Cognition", name: "Biased Cognition", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["gain_focus", "lose_focus_each_turn"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Biased Cognition+", name: "Biased Cognition+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["gain_focus", "lose_focus_each_turn"], effect_data: &[], complex_hook: None,
        });
        // Buffer: 2 cost, power, prevent next X HP loss
        insert(cards, CardDef {
            id: "Buffer", name: "Buffer", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["buffer"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Buffer+", name: "Buffer+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["buffer"], effect_data: &[], complex_hook: None,
        });
        // Core Surge: 1 cost, 11 dmg, gain 1 Artifact, exhaust
        insert(cards, CardDef {
            id: "Core Surge", name: "Core Surge", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["gain_artifact"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Core Surge+", name: "Core Surge+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["gain_artifact"], effect_data: &[], complex_hook: None,
        });
        // Creative AI: 3 cost, power, add random Power to hand each turn (upgrade: cost 2)
        insert(cards, CardDef {
            id: "Creative AI", name: "Creative AI", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["creative_ai"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Creative AI+", name: "Creative AI+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["creative_ai"], effect_data: &[], complex_hook: None,
        });
        // Echo Form: 3 cost, power, ethereal, first card each turn played twice (upgrade: no ethereal)
        insert(cards, CardDef {
            id: "Echo Form", name: "Echo Form", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["echo_form", "ethereal"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Echo Form+", name: "Echo Form+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["echo_form"], effect_data: &[], complex_hook: None,
        });
        // Electrodynamics: 2 cost, power, Lightning hits all enemies, channel 2 Lightning
        insert(cards, CardDef {
            id: "Electrodynamics", name: "Electrodynamics", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["lightning_hits_all", "channel_lightning"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Electrodynamics+", name: "Electrodynamics+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["lightning_hits_all", "channel_lightning"], effect_data: &[], complex_hook: None,
        });
        // Fission: 0 cost, remove all orbs, gain energy+draw per orb, exhaust (upgrade: evoke instead of remove)
        insert(cards, CardDef {
            id: "Fission", name: "Fission", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["fission"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Fission+", name: "Fission+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["fission_evoke"], effect_data: &[], complex_hook: None,
        });
        // Hyperbeam: 2 cost, 26 dmg AoE, lose 3 focus
        insert(cards, CardDef {
            id: "Hyperbeam", name: "Hyperbeam", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 26, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["lose_focus"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Hyperbeam+", name: "Hyperbeam+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 34, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["lose_focus"], effect_data: &[], complex_hook: None,
        });
        // Machine Learning: 1 cost, power, draw 1 extra card each turn (upgrade: innate)
        insert(cards, CardDef {
            id: "Machine Learning", name: "Machine Learning", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["extra_draw_each_turn"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Machine Learning+", name: "Machine Learning+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["extra_draw_each_turn", "innate"], effect_data: &[], complex_hook: None,
        });
        // Meteor Strike: 5 cost, 24 dmg, channel 3 Plasma
        insert(cards, CardDef {
            id: "Meteor Strike", name: "Meteor Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 5, base_damage: 24, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["channel_plasma"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Meteor Strike+", name: "Meteor Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 5, base_damage: 30, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["channel_plasma"], effect_data: &[], complex_hook: None,
        });
        // Multi-Cast: X cost, evoke frontmost orb X times (upgrade: X+1)
        insert(cards, CardDef {
            id: "Multi-Cast", name: "Multi-Cast", card_type: CardType::Skill,
            target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["evoke_orb_x"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Multi-Cast+", name: "Multi-Cast+", card_type: CardType::Skill,
            target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["evoke_orb_x_plus_1"], effect_data: &[], complex_hook: None,
        });
        // Rainbow: 2 cost, channel Lightning+Frost+Dark, exhaust (upgrade: no exhaust)
        insert(cards, CardDef {
            id: "Rainbow", name: "Rainbow", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["channel_lightning", "channel_frost", "channel_dark"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Rainbow+", name: "Rainbow+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning", "channel_frost", "channel_dark"], effect_data: &[], complex_hook: None,
        });
        // Reboot: 0 cost, shuffle hand+discard into draw, draw 4, exhaust
        insert(cards, CardDef {
            id: "Reboot", name: "Reboot", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["reboot"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Reboot+", name: "Reboot+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 6, exhaust: true, enter_stance: None,
            effects: &["reboot"], effect_data: &[], complex_hook: None,
        });
        // Seek: 0 cost, choose 1 card from draw pile and put into hand, exhaust
        insert(cards, CardDef {
            id: "Seek", name: "Seek", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["seek"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Seek+", name: "Seek+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["seek"], effect_data: &[], complex_hook: None,
        });
        // Thunder Strike: 3 cost, deal 7 dmg for each Lightning channeled this combat
        insert(cards, CardDef {
            id: "Thunder Strike", name: "Thunder Strike", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 3, base_damage: 7, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["damage_per_lightning_channeled"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Thunder Strike+", name: "Thunder Strike+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 3, base_damage: 9, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["damage_per_lightning_channeled"], effect_data: &[], complex_hook: None,
        });
}

fn insert(map: &mut HashMap<&'static str, CardDef>, card: CardDef) {
    map.insert(card.id, card);
}
