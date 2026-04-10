use std::collections::HashMap;
use super::{CardDef, CardType, CardTarget};

pub fn register_colorless(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Colorless basics (Strike/Defend aliases for other characters) ----
        insert(cards, CardDef {
            id: "Strike_R", name: "Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Strike_R+", name: "Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Defend_R", name: "Defend", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Defend_R+", name: "Defend+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Colorless Uncommon ----
        // Bandage Up: 0 cost, heal 4, exhaust
        insert(cards, CardDef {
            id: "Bandage Up", name: "Bandage Up", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["heal"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Bandage Up+", name: "Bandage Up+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 6, exhaust: true, enter_stance: None,
            effects: &["heal"], effect_data: &[], complex_hook: None,
        });
        // Blind: 0 cost, apply 2 Weak to enemy (upgrade: target all)
        insert(cards, CardDef {
            id: "Blind", name: "Blind", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["apply_weak"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Blind+", name: "Blind+", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["apply_weak"], effect_data: &[], complex_hook: None,
        });
        // Dark Shackles: 0 cost, reduce enemy str by 9 for one turn, exhaust
        insert(cards, CardDef {
            id: "Dark Shackles", name: "Dark Shackles", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 9, exhaust: true, enter_stance: None,
            effects: &["reduce_str_this_turn"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Dark Shackles+", name: "Dark Shackles+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 15, exhaust: true, enter_stance: None,
            effects: &["reduce_str_this_turn"], effect_data: &[], complex_hook: None,
        });
        // Deep Breath: 0 cost, shuffle discard into draw, draw 1
        insert(cards, CardDef {
            id: "Deep Breath", name: "Deep Breath", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["shuffle_discard_into_draw", "draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Deep Breath+", name: "Deep Breath+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["shuffle_discard_into_draw", "draw"], effect_data: &[], complex_hook: None,
        });
        // Discovery: 1 cost, choose 1 of 3 cards to add to hand, exhaust (upgrade: no exhaust)
        insert(cards, CardDef {
            id: "Discovery", name: "Discovery", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["discovery"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Discovery+", name: "Discovery+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["discovery"], effect_data: &[], complex_hook: None,
        });
        // Dramatic Entrance: 0 cost, 8 dmg AoE, innate, exhaust
        insert(cards, CardDef {
            id: "Dramatic Entrance", name: "Dramatic Entrance", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 0, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["innate"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Dramatic Entrance+", name: "Dramatic Entrance+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 0, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["innate"], effect_data: &[], complex_hook: None,
        });
        // Enlightenment: 0 cost, reduce cost of all cards in hand to 1 (this turn, upgrade: permanent)
        insert(cards, CardDef {
            id: "Enlightenment", name: "Enlightenment", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["enlightenment_this_turn"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Enlightenment+", name: "Enlightenment+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["enlightenment_permanent"], effect_data: &[], complex_hook: None,
        });
        // Finesse: 0 cost, 2 block, draw 1
        insert(cards, CardDef {
            id: "Finesse", name: "Finesse", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 2,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Finesse+", name: "Finesse+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[], complex_hook: None,
        });
        // Flash of Steel: 0 cost, 3 dmg, draw 1
        insert(cards, CardDef {
            id: "Flash of Steel", name: "Flash of Steel", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Flash of Steel+", name: "Flash of Steel+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[], complex_hook: None,
        });
        // Forethought: 0 cost, put card from hand to bottom of draw pile at 0 cost
        insert(cards, CardDef {
            id: "Forethought", name: "Forethought", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["forethought"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Forethought+", name: "Forethought+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["forethought_all"], effect_data: &[], complex_hook: None,
        });
        // Good Instincts: 0 cost, 6 block
        insert(cards, CardDef {
            id: "Good Instincts", name: "Good Instincts", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 6,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Good Instincts+", name: "Good Instincts+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 9,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        // Impatience: 0 cost, draw 2 if no attacks in hand
        insert(cards, CardDef {
            id: "Impatience", name: "Impatience", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw_if_no_attacks"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Impatience+", name: "Impatience+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["draw_if_no_attacks"], effect_data: &[], complex_hook: None,
        });
        // Jack of All Trades: 0 cost, add 1 random colorless card to hand, exhaust
        insert(cards, CardDef {
            id: "Jack Of All Trades", name: "Jack Of All Trades", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["add_random_colorless"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Jack Of All Trades+", name: "Jack Of All Trades+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["add_random_colorless"], effect_data: &[], complex_hook: None,
        });
        // Madness: 1 cost, reduce random card in hand to 0 cost, exhaust (upgrade: cost 0)
        insert(cards, CardDef {
            id: "Madness", name: "Madness", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["madness"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Madness+", name: "Madness+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["madness"], effect_data: &[], complex_hook: None,
        });
        // Mind Blast: 2 cost, dmg = draw pile size, innate (upgrade: cost 1)
        insert(cards, CardDef {
            id: "Mind Blast", name: "Mind Blast", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 0, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_from_draw_pile", "innate"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Mind Blast+", name: "Mind Blast+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 0, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_from_draw_pile", "innate"], effect_data: &[], complex_hook: None,
        });
        // Panacea: 0 cost, gain 1 Artifact, exhaust
        insert(cards, CardDef {
            id: "Panacea", name: "Panacea", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["gain_artifact"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Panacea+", name: "Panacea+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["gain_artifact"], effect_data: &[], complex_hook: None,
        });
        // Panic Button: 0 cost, 30 block, no block next 2 turns, exhaust
        insert(cards, CardDef {
            id: "PanicButton", name: "Panic Button", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 30,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["no_block_next_turns"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "PanicButton+", name: "Panic Button+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 40,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["no_block_next_turns"], effect_data: &[], complex_hook: None,
        });
        // Purity: 0 cost, exhaust up to 3 cards from hand, exhaust
        insert(cards, CardDef {
            id: "Purity", name: "Purity", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["exhaust_from_hand"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Purity+", name: "Purity+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["exhaust_from_hand"], effect_data: &[], complex_hook: None,
        });
        // Swift Strike: 0 cost, 7 dmg
        insert(cards, CardDef {
            id: "Swift Strike", name: "Swift Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Swift Strike+", name: "Swift Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        // Trip: 0 cost, apply 2 Vulnerable (upgrade: target all)
        insert(cards, CardDef {
            id: "Trip", name: "Trip", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["apply_vulnerable"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Trip+", name: "Trip+", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["apply_vulnerable"], effect_data: &[], complex_hook: None,
        });

        // ---- Colorless Rare ----
        // Apotheosis: 2 cost, upgrade all cards in deck, exhaust (upgrade: cost 1)
        insert(cards, CardDef {
            id: "Apotheosis", name: "Apotheosis", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["upgrade_all_cards"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Apotheosis+", name: "Apotheosis+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["upgrade_all_cards"], effect_data: &[], complex_hook: None,
        });
        // Chrysalis: 2 cost, shuffle 3 random upgraded Skills into draw pile, exhaust
        insert(cards, CardDef {
            id: "Chrysalis", name: "Chrysalis", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["add_random_skills_to_draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Chrysalis+", name: "Chrysalis+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["add_random_skills_to_draw"], effect_data: &[], complex_hook: None,
        });
        // Hand of Greed: 2 cost, 20 dmg, if kill gain 20 gold
        insert(cards, CardDef {
            id: "HandOfGreed", name: "Hand of Greed", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
            base_magic: 20, exhaust: false, enter_stance: None,
            effects: &["gold_on_kill"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "HandOfGreed+", name: "Hand of Greed+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 25, base_block: -1,
            base_magic: 25, exhaust: false, enter_stance: None,
            effects: &["gold_on_kill"], effect_data: &[], complex_hook: None,
        });
        // Magnetism: 2 cost, power, add random colorless card to hand each turn (upgrade: cost 1)
        insert(cards, CardDef {
            id: "Magnetism", name: "Magnetism", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["magnetism"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Magnetism+", name: "Magnetism+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["magnetism"], effect_data: &[], complex_hook: None,
        });
        // Master of Strategy: 0 cost, draw 3, exhaust
        insert(cards, CardDef {
            id: "Master of Strategy", name: "Master of Strategy", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Master of Strategy+", name: "Master of Strategy+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["draw"], effect_data: &[], complex_hook: None,
        });
        // Mayhem: 2 cost, power, auto-play top card of draw pile each turn (upgrade: cost 1)
        insert(cards, CardDef {
            id: "Mayhem", name: "Mayhem", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["mayhem"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Mayhem+", name: "Mayhem+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["mayhem"], effect_data: &[], complex_hook: None,
        });
        // Metamorphosis: 2 cost, shuffle 3 random upgraded Attacks into draw pile, exhaust
        insert(cards, CardDef {
            id: "Metamorphosis", name: "Metamorphosis", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["add_random_attacks_to_draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Metamorphosis+", name: "Metamorphosis+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["add_random_attacks_to_draw"], effect_data: &[], complex_hook: None,
        });
        // Panache: 0 cost, power, deal 10 dmg to all every 5th card played per turn
        insert(cards, CardDef {
            id: "Panache", name: "Panache", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 10, exhaust: false, enter_stance: None,
            effects: &["panache"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Panache+", name: "Panache+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 14, exhaust: false, enter_stance: None,
            effects: &["panache"], effect_data: &[], complex_hook: None,
        });
        // Sadistic Nature: 0 cost, power, deal 5 dmg whenever you apply debuff
        insert(cards, CardDef {
            id: "Sadistic Nature", name: "Sadistic Nature", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["sadistic_nature"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Sadistic Nature+", name: "Sadistic Nature+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: false, enter_stance: None,
            effects: &["sadistic_nature"], effect_data: &[], complex_hook: None,
        });
        // Secret Technique: 0 cost, choose Skill from draw pile, put in hand, exhaust (upgrade: no exhaust)
        insert(cards, CardDef {
            id: "Secret Technique", name: "Secret Technique", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["search_skill"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Secret Technique+", name: "Secret Technique+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["search_skill"], effect_data: &[], complex_hook: None,
        });
        // Secret Weapon: 0 cost, choose Attack from draw pile, put in hand, exhaust (upgrade: no exhaust)
        insert(cards, CardDef {
            id: "Secret Weapon", name: "Secret Weapon", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["search_attack"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Secret Weapon+", name: "Secret Weapon+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["search_attack"], effect_data: &[], complex_hook: None,
        });
        // The Bomb: 2 cost, deal 40 dmg to all enemies in 3 turns
        insert(cards, CardDef {
            id: "The Bomb", name: "The Bomb", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 40, exhaust: false, enter_stance: None,
            effects: &["the_bomb"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "The Bomb+", name: "The Bomb+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 50, exhaust: false, enter_stance: None,
            effects: &["the_bomb"], effect_data: &[], complex_hook: None,
        });
        // Thinking Ahead: 0 cost, draw 2, put 1 card from hand on top of draw, exhaust (upgrade: no exhaust)
        insert(cards, CardDef {
            id: "Thinking Ahead", name: "Thinking Ahead", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["thinking_ahead"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Thinking Ahead+", name: "Thinking Ahead+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["thinking_ahead"], effect_data: &[], complex_hook: None,
        });
        // Transmutation: X cost, add X random colorless cards to hand, exhaust
        insert(cards, CardDef {
            id: "Transmutation", name: "Transmutation", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["transmutation"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Transmutation+", name: "Transmutation+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["transmutation"], effect_data: &[], complex_hook: None,
        });
        // Violence: 0 cost, put 3 random Attacks from draw pile into hand, exhaust
        insert(cards, CardDef {
            id: "Violence", name: "Violence", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["draw_attacks_from_draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Violence+", name: "Violence+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["draw_attacks_from_draw"], effect_data: &[], complex_hook: None,
        });

        // ---- Colorless Special ----
        // Apparition (Java ID: Ghostly): 1 cost, gain 1 Intangible, exhaust, ethereal
        insert(cards, CardDef {
            id: "Ghostly", name: "Apparition", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["intangible", "ethereal"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Ghostly+", name: "Apparition+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["intangible"], effect_data: &[], complex_hook: None,
        });
        // Bite: 1 cost, 7 dmg, heal 2
        insert(cards, CardDef {
            id: "Bite", name: "Bite", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["heal_on_play"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Bite+", name: "Bite+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["heal_on_play"], effect_data: &[], complex_hook: None,
        });
        // J.A.X.: 0 cost, lose 3 HP, gain 2 str
        insert(cards, CardDef {
            id: "J.A.X.", name: "J.A.X.", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["lose_hp_gain_str"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "J.A.X.+", name: "J.A.X.+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["lose_hp_gain_str"], effect_data: &[], complex_hook: None,
        });
        // Ritual Dagger: 1 cost, dmg from misc, gain 3 per kill, exhaust
        insert(cards, CardDef {
            id: "RitualDagger", name: "Ritual Dagger", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["ritual_dagger"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "RitualDagger+", name: "Ritual Dagger+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["ritual_dagger"], effect_data: &[], complex_hook: None,
        });
}

fn insert(map: &mut HashMap<&'static str, CardDef>, card: CardDef) {
    map.insert(card.id, card);
}
