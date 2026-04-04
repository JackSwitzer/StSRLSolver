use std::collections::HashMap;
use super::{CardDef, CardType, CardTarget};

pub fn register_ironclad(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Basic: Bash ---- (cost 2, 8 dmg, 2 vuln; +2/+1)
        insert(cards, CardDef {
            id: "Bash", name: "Bash", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 8, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["vulnerable"],
        });
        insert(cards, CardDef {
            id: "Bash+", name: "Bash+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["vulnerable"],
        });

        // ---- Ironclad Common: Anger ---- (cost 0, 6 dmg, add copy to discard; +2 dmg)
        insert(cards, CardDef {
            id: "Anger", name: "Anger", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["copy_to_discard"],
        });
        insert(cards, CardDef {
            id: "Anger+", name: "Anger+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["copy_to_discard"],
        });

        // ---- Ironclad Common: Armaments ---- (cost 1, 5 block, upgrade 1 card in hand; upgrade: all cards)
        insert(cards, CardDef {
            id: "Armaments", name: "Armaments", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["upgrade_one_card"],
        });
        insert(cards, CardDef {
            id: "Armaments+", name: "Armaments+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["upgrade_all_cards"],
        });

        // ---- Ironclad Common: Body Slam ---- (cost 1, dmg = current block; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Body Slam", name: "Body Slam", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 0, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_equals_block"],
        });
        insert(cards, CardDef {
            id: "Body Slam+", name: "Body Slam+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 0, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_equals_block"],
        });

        // ---- Ironclad Common: Clash ---- (cost 0, 14 dmg, only if hand is all attacks; +4 dmg)
        insert(cards, CardDef {
            id: "Clash", name: "Clash", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 14, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["only_attacks_in_hand"],
        });
        insert(cards, CardDef {
            id: "Clash+", name: "Clash+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 18, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["only_attacks_in_hand"],
        });

        // ---- Ironclad Common: Cleave ---- (cost 1, 8 dmg AoE; +3 dmg)
        insert(cards, CardDef {
            id: "Cleave", name: "Cleave", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Cleave+", name: "Cleave+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 11, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });

        // ---- Ironclad Common: Clothesline ---- (cost 2, 12 dmg, 2 weak; +2/+1)
        insert(cards, CardDef {
            id: "Clothesline", name: "Clothesline", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["weak"],
        });
        insert(cards, CardDef {
            id: "Clothesline+", name: "Clothesline+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 14, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["weak"],
        });

        // ---- Ironclad Common: Flex ---- (cost 0, +2 str this turn; +2 magic)
        insert(cards, CardDef {
            id: "Flex", name: "Flex", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["temp_strength"],
        });
        insert(cards, CardDef {
            id: "Flex+", name: "Flex+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["temp_strength"],
        });

        // ---- Ironclad Common: Havoc ---- (cost 1, play top card of draw pile; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Havoc", name: "Havoc", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["play_top_card"],
        });
        insert(cards, CardDef {
            id: "Havoc+", name: "Havoc+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["play_top_card"],
        });

        // ---- Ironclad Common: Headbutt ---- (cost 1, 9 dmg, put card from discard on top of draw; +3 dmg)
        insert(cards, CardDef {
            id: "Headbutt", name: "Headbutt", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["discard_to_top_of_draw"],
        });
        insert(cards, CardDef {
            id: "Headbutt+", name: "Headbutt+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["discard_to_top_of_draw"],
        });

        // ---- Ironclad Common: Heavy Blade ---- (cost 2, 14 dmg, 3x str scaling; upgrade: 5x str)
        insert(cards, CardDef {
            id: "Heavy Blade", name: "Heavy Blade", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 14, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["heavy_blade"],
        });
        insert(cards, CardDef {
            id: "Heavy Blade+", name: "Heavy Blade+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 14, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["heavy_blade"],
        });

        // ---- Ironclad Common: Iron Wave ---- (cost 1, 5 dmg + 5 block; +2/+2)
        insert(cards, CardDef {
            id: "Iron Wave", name: "Iron Wave", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Iron Wave+", name: "Iron Wave+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });

        // ---- Ironclad Common: Perfected Strike ---- (cost 2, 6 dmg + 2/strike in deck; +1 magic)
        insert(cards, CardDef {
            id: "Perfected Strike", name: "Perfected Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 6, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["perfected_strike"],
        });
        insert(cards, CardDef {
            id: "Perfected Strike+", name: "Perfected Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 6, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["perfected_strike"],
        });

        // ---- Ironclad Common: Pommel Strike ---- (cost 1, 9 dmg, draw 1; +1/+1)
        insert(cards, CardDef {
            id: "Pommel Strike", name: "Pommel Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        insert(cards, CardDef {
            id: "Pommel Strike+", name: "Pommel Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });

        // ---- Ironclad Common: Shrug It Off ---- (cost 1, 8 block, draw 1; +3 block)
        insert(cards, CardDef {
            id: "Shrug It Off", name: "Shrug It Off", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        insert(cards, CardDef {
            id: "Shrug It Off+", name: "Shrug It Off+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 11,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });

        // ---- Ironclad Common: Sword Boomerang ---- (cost 1, 3 dmg x3 random; +1 magic)
        insert(cards, CardDef {
            id: "Sword Boomerang", name: "Sword Boomerang", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 3, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["damage_random_x_times"],
        });
        insert(cards, CardDef {
            id: "Sword Boomerang+", name: "Sword Boomerang+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 3, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["damage_random_x_times"],
        });

        // ---- Ironclad Common: Thunderclap ---- (cost 1, 4 dmg AoE + 1 vuln all; +3 dmg)
        insert(cards, CardDef {
            id: "Thunderclap", name: "Thunderclap", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 4, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["vulnerable_all"],
        });
        insert(cards, CardDef {
            id: "Thunderclap+", name: "Thunderclap+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["vulnerable_all"],
        });

        // ---- Ironclad Common: True Grit ---- (cost 1, 7 block, exhaust random card; upgrade: +2 block, choose)
        insert(cards, CardDef {
            id: "True Grit", name: "True Grit", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["exhaust_random"],
        });
        insert(cards, CardDef {
            id: "True Grit+", name: "True Grit+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 9,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["exhaust_choose"],
        });

        // ---- Ironclad Common: Twin Strike ---- (cost 1, 5 dmg x2; +2 dmg)
        insert(cards, CardDef {
            id: "Twin Strike", name: "Twin Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit"],
        });
        insert(cards, CardDef {
            id: "Twin Strike+", name: "Twin Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit"],
        });

        // ---- Ironclad Common: Warcry ---- (cost 0, draw 1, put 1 on top, exhaust; +1 draw)
        insert(cards, CardDef {
            id: "Warcry", name: "Warcry", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["draw", "put_card_on_top"],
        });
        insert(cards, CardDef {
            id: "Warcry+", name: "Warcry+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["draw", "put_card_on_top"],
        });

        // ---- Ironclad Common: Wild Strike ---- (cost 1, 12 dmg, shuffle Wound into draw; +5 dmg)
        insert(cards, CardDef {
            id: "Wild Strike", name: "Wild Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_wound_to_draw"],
        });
        insert(cards, CardDef {
            id: "Wild Strike+", name: "Wild Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 17, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_wound_to_draw"],
        });

        // ---- Ironclad Uncommon: Battle Trance ---- (cost 0, draw 3, no more draw; +1)
        insert(cards, CardDef {
            id: "Battle Trance", name: "Battle Trance", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["draw", "no_draw"],
        });
        insert(cards, CardDef {
            id: "Battle Trance+", name: "Battle Trance+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["draw", "no_draw"],
        });

        // ---- Ironclad Uncommon: Blood for Blood ---- (cost 4, 18 dmg, -1 cost per HP loss; upgrade: cost 3, +4 dmg)
        insert(cards, CardDef {
            id: "Blood for Blood", name: "Blood for Blood", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 4, base_damage: 18, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["cost_reduce_on_hp_loss"],
        });
        insert(cards, CardDef {
            id: "Blood for Blood+", name: "Blood for Blood+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 3, base_damage: 22, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["cost_reduce_on_hp_loss"],
        });

        // ---- Ironclad Uncommon: Bloodletting ---- (cost 0, lose 3 HP, gain 2 energy; +1 energy)
        insert(cards, CardDef {
            id: "Bloodletting", name: "Bloodletting", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["lose_hp_gain_energy"],
        });
        insert(cards, CardDef {
            id: "Bloodletting+", name: "Bloodletting+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["lose_hp_gain_energy"],
        });

        // ---- Ironclad Uncommon: Burning Pact ---- (cost 1, exhaust 1 card, draw 2; +1 draw)
        insert(cards, CardDef {
            id: "Burning Pact", name: "Burning Pact", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["exhaust_choose", "draw"],
        });
        insert(cards, CardDef {
            id: "Burning Pact+", name: "Burning Pact+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["exhaust_choose", "draw"],
        });

        // ---- Ironclad Uncommon: Carnage ---- (cost 2, 20 dmg, ethereal; +8 dmg)
        insert(cards, CardDef {
            id: "Carnage", name: "Carnage", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["ethereal"],
        });
        insert(cards, CardDef {
            id: "Carnage+", name: "Carnage+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 28, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["ethereal"],
        });

        // ---- Ironclad Uncommon: Combust ---- (cost 1, power, lose 1 HP/turn, deal 5 dmg to all; +2 magic)
        insert(cards, CardDef {
            id: "Combust", name: "Combust", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["combust"],
        });
        insert(cards, CardDef {
            id: "Combust+", name: "Combust+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: false, enter_stance: None,
            effects: &["combust"],
        });

        // ---- Ironclad Uncommon: Dark Embrace ---- (cost 2, power, draw 1 on exhaust; upgrade: cost 1)
        insert(cards, CardDef {
            id: "Dark Embrace", name: "Dark Embrace", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["dark_embrace"],
        });
        insert(cards, CardDef {
            id: "Dark Embrace+", name: "Dark Embrace+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["dark_embrace"],
        });

        // ---- Ironclad Uncommon: Disarm ---- (cost 1, -2 str to enemy, exhaust; +1 magic)
        insert(cards, CardDef {
            id: "Disarm", name: "Disarm", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["reduce_strength"],
        });
        insert(cards, CardDef {
            id: "Disarm+", name: "Disarm+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["reduce_strength"],
        });

        // ---- Ironclad Uncommon: Dropkick ---- (cost 1, 5 dmg, if vuln: +1 energy + draw 1; +3 dmg)
        insert(cards, CardDef {
            id: "Dropkick", name: "Dropkick", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["if_vulnerable_energy_draw"],
        });
        insert(cards, CardDef {
            id: "Dropkick+", name: "Dropkick+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["if_vulnerable_energy_draw"],
        });

        // ---- Ironclad Uncommon: Dual Wield ---- (cost 1, copy 1 attack/power in hand; upgrade: 2 copies)
        insert(cards, CardDef {
            id: "Dual Wield", name: "Dual Wield", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["dual_wield"],
        });
        insert(cards, CardDef {
            id: "Dual Wield+", name: "Dual Wield+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["dual_wield"],
        });

        // ---- Ironclad Uncommon: Entrench ---- (cost 2, double block; upgrade: cost 1)
        insert(cards, CardDef {
            id: "Entrench", name: "Entrench", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["double_block"],
        });
        insert(cards, CardDef {
            id: "Entrench+", name: "Entrench+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["double_block"],
        });

        // ---- Ironclad Uncommon: Evolve ---- (cost 1, power, draw 1 when Status drawn; upgrade: draw 2)
        insert(cards, CardDef {
            id: "Evolve", name: "Evolve", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["evolve"],
        });
        insert(cards, CardDef {
            id: "Evolve+", name: "Evolve+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["evolve"],
        });

        // ---- Ironclad Uncommon: Feel No Pain ---- (cost 1, power, 3 block on exhaust; +1 magic)
        insert(cards, CardDef {
            id: "Feel No Pain", name: "Feel No Pain", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["feel_no_pain"],
        });
        insert(cards, CardDef {
            id: "Feel No Pain+", name: "Feel No Pain+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["feel_no_pain"],
        });

        // ---- Ironclad Uncommon: Fire Breathing ---- (cost 1, power, 6 dmg on Status/Curse draw; +4 magic)
        insert(cards, CardDef {
            id: "Fire Breathing", name: "Fire Breathing", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 6, exhaust: false, enter_stance: None,
            effects: &["fire_breathing"],
        });
        insert(cards, CardDef {
            id: "Fire Breathing+", name: "Fire Breathing+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 10, exhaust: false, enter_stance: None,
            effects: &["fire_breathing"],
        });

        // ---- Ironclad Uncommon: Flame Barrier ---- (cost 2, 12 block + 4 fire dmg when hit; +4/+2)
        insert(cards, CardDef {
            id: "Flame Barrier", name: "Flame Barrier", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 12,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["flame_barrier"],
        });
        insert(cards, CardDef {
            id: "Flame Barrier+", name: "Flame Barrier+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 16,
            base_magic: 6, exhaust: false, enter_stance: None,
            effects: &["flame_barrier"],
        });

        // ---- Ironclad Uncommon: Ghostly Armor ---- (cost 1, 10 block, ethereal; +3 block)
        insert(cards, CardDef {
            id: "Ghostly Armor", name: "Ghostly Armor", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 10,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["ethereal"],
        });
        insert(cards, CardDef {
            id: "Ghostly Armor+", name: "Ghostly Armor+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 13,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["ethereal"],
        });

        // ---- Ironclad Uncommon: Hemokinesis ---- (cost 1, 15 dmg, lose 2 HP; +5 dmg)
        insert(cards, CardDef {
            id: "Hemokinesis", name: "Hemokinesis", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["lose_hp"],
        });
        insert(cards, CardDef {
            id: "Hemokinesis+", name: "Hemokinesis+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 20, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["lose_hp"],
        });

        // ---- Ironclad Uncommon: Infernal Blade ---- (cost 1, exhaust, add random attack to hand at cost 0; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Infernal Blade", name: "Infernal Blade", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["random_attack_to_hand"],
        });
        insert(cards, CardDef {
            id: "Infernal Blade+", name: "Infernal Blade+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["random_attack_to_hand"],
        });

        // ---- Ironclad Uncommon: Inflame ---- (cost 1, power, +2 str; +1)
        insert(cards, CardDef {
            id: "Inflame", name: "Inflame", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["gain_strength"],
        });
        insert(cards, CardDef {
            id: "Inflame+", name: "Inflame+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["gain_strength"],
        });

        // ---- Ironclad Uncommon: Intimidate ---- (cost 0, 1 weak to all, exhaust; +1 magic)
        insert(cards, CardDef {
            id: "Intimidate", name: "Intimidate", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["weak_all"],
        });
        insert(cards, CardDef {
            id: "Intimidate+", name: "Intimidate+", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["weak_all"],
        });

        // ---- Ironclad Uncommon: Metallicize ---- (cost 1, power, +3 block/turn; +1)
        insert(cards, CardDef {
            id: "Metallicize", name: "Metallicize", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["metallicize"],
        });
        insert(cards, CardDef {
            id: "Metallicize+", name: "Metallicize+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["metallicize"],
        });

        // ---- Ironclad Uncommon: Power Through ---- (cost 1, 15 block, add 2 Wounds to hand; +5 block)
        insert(cards, CardDef {
            id: "Power Through", name: "Power Through", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 15,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_wounds_to_hand"],
        });
        insert(cards, CardDef {
            id: "Power Through+", name: "Power Through+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 20,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_wounds_to_hand"],
        });

        // ---- Ironclad Uncommon: Pummel ---- (cost 1, 2 dmg x4, exhaust; +1 hit)
        insert(cards, CardDef {
            id: "Pummel", name: "Pummel", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 2, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["multi_hit"],
        });
        insert(cards, CardDef {
            id: "Pummel+", name: "Pummel+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 2, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["multi_hit"],
        });

        // ---- Ironclad Uncommon: Rage ---- (cost 0, gain 3 block per attack played this turn; +2 magic)
        insert(cards, CardDef {
            id: "Rage", name: "Rage", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["rage"],
        });
        insert(cards, CardDef {
            id: "Rage+", name: "Rage+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["rage"],
        });

        // ---- Ironclad Uncommon: Rampage ---- (cost 1, 8 dmg, +5 dmg each play; +3 magic)
        insert(cards, CardDef {
            id: "Rampage", name: "Rampage", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["rampage"],
        });
        insert(cards, CardDef {
            id: "Rampage+", name: "Rampage+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 8, exhaust: false, enter_stance: None,
            effects: &["rampage"],
        });

        // ---- Ironclad Uncommon: Reckless Charge ---- (cost 0, 7 dmg, shuffle Dazed into draw; +3 dmg)
        insert(cards, CardDef {
            id: "Reckless Charge", name: "Reckless Charge", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_dazed_to_draw"],
        });
        insert(cards, CardDef {
            id: "Reckless Charge+", name: "Reckless Charge+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_dazed_to_draw"],
        });

        // ---- Ironclad Uncommon: Rupture ---- (cost 1, power, +1 str when lose HP from card; +1 magic)
        insert(cards, CardDef {
            id: "Rupture", name: "Rupture", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["rupture"],
        });
        insert(cards, CardDef {
            id: "Rupture+", name: "Rupture+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["rupture"],
        });

        // ---- Ironclad Uncommon: Searing Blow ---- (cost 2, 12 dmg, can upgrade infinitely; +4+N per upgrade)
        insert(cards, CardDef {
            id: "Searing Blow", name: "Searing Blow", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["searing_blow"],
        });
        insert(cards, CardDef {
            id: "Searing Blow+", name: "Searing Blow+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 16, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["searing_blow"],
        });

        // ---- Ironclad Uncommon: Second Wind ---- (cost 1, exhaust all non-attack, gain block per; +2 block)
        insert(cards, CardDef {
            id: "Second Wind", name: "Second Wind", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["second_wind"],
        });
        insert(cards, CardDef {
            id: "Second Wind+", name: "Second Wind+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["second_wind"],
        });

        // ---- Ironclad Uncommon: Seeing Red ---- (cost 1, gain 2 energy, exhaust; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Seeing Red", name: "Seeing Red", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["gain_energy"],
        });
        insert(cards, CardDef {
            id: "Seeing Red+", name: "Seeing Red+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["gain_energy"],
        });

        // ---- Ironclad Uncommon: Sentinel ---- (cost 1, 5 block, gain 2 energy on exhaust; +3 block, 3 energy)
        insert(cards, CardDef {
            id: "Sentinel", name: "Sentinel", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["energy_on_exhaust"],
        });
        insert(cards, CardDef {
            id: "Sentinel+", name: "Sentinel+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["energy_on_exhaust"],
        });

        // ---- Ironclad Uncommon: Sever Soul ---- (cost 2, 16 dmg, exhaust all non-attacks in hand; +6 dmg)
        insert(cards, CardDef {
            id: "Sever Soul", name: "Sever Soul", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 16, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["exhaust_non_attacks"],
        });
        insert(cards, CardDef {
            id: "Sever Soul+", name: "Sever Soul+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 22, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["exhaust_non_attacks"],
        });

        // ---- Ironclad Uncommon: Shockwave ---- (cost 2, 3 weak+vuln to all, exhaust; +2 magic)
        insert(cards, CardDef {
            id: "Shockwave", name: "Shockwave", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["weak_all", "vulnerable_all"],
        });
        insert(cards, CardDef {
            id: "Shockwave+", name: "Shockwave+", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["weak_all", "vulnerable_all"],
        });

        // ---- Ironclad Uncommon: Spot Weakness ---- (cost 1, +3 str if enemy attacking; +1 magic)
        insert(cards, CardDef {
            id: "Spot Weakness", name: "Spot Weakness", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["spot_weakness"],
        });
        insert(cards, CardDef {
            id: "Spot Weakness+", name: "Spot Weakness+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["spot_weakness"],
        });

        // ---- Ironclad Uncommon: Uppercut ---- (cost 2, 13 dmg, 1 weak + 1 vuln; +1/+1)
        insert(cards, CardDef {
            id: "Uppercut", name: "Uppercut", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["weak", "vulnerable"],
        });
        insert(cards, CardDef {
            id: "Uppercut+", name: "Uppercut+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["weak", "vulnerable"],
        });

        // ---- Ironclad Uncommon: Whirlwind ---- (cost X, 5 dmg AoE per X; +3 dmg)
        insert(cards, CardDef {
            id: "Whirlwind", name: "Whirlwind", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: -1, base_damage: 5, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["x_cost"],
        });
        insert(cards, CardDef {
            id: "Whirlwind+", name: "Whirlwind+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: -1, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["x_cost"],
        });

        // ---- Ironclad Rare: Barricade ---- (cost 3, power, block not removed at end of turn; upgrade: cost 2)
        insert(cards, CardDef {
            id: "Barricade", name: "Barricade", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["barricade"],
        });
        insert(cards, CardDef {
            id: "Barricade+", name: "Barricade+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["barricade"],
        });

        // ---- Ironclad Rare: Berserk ---- (cost 0, power, 2 vuln to self, +1 energy/turn; -1 vuln)
        insert(cards, CardDef {
            id: "Berserk", name: "Berserk", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["berserk"],
        });
        insert(cards, CardDef {
            id: "Berserk+", name: "Berserk+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["berserk"],
        });

        // ---- Ironclad Rare: Bludgeon ---- (cost 3, 32 dmg; +10 dmg)
        insert(cards, CardDef {
            id: "Bludgeon", name: "Bludgeon", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 3, base_damage: 32, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Bludgeon+", name: "Bludgeon+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 3, base_damage: 42, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });

        // ---- Ironclad Rare: Brutality ---- (cost 0, power, lose 1 HP + draw 1 at turn start; upgrade: innate)
        insert(cards, CardDef {
            id: "Brutality", name: "Brutality", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["brutality"],
        });
        insert(cards, CardDef {
            id: "Brutality+", name: "Brutality+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["brutality", "innate"],
        });

        // ---- Ironclad Rare: Corruption ---- (cost 3, power, skills cost 0 but exhaust; upgrade: cost 2)
        insert(cards, CardDef {
            id: "Corruption", name: "Corruption", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["corruption"],
        });
        insert(cards, CardDef {
            id: "Corruption+", name: "Corruption+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["corruption"],
        });

        // ---- Ironclad Rare: Demon Form ---- (cost 3, power, +2 str/turn; +1 magic)
        insert(cards, CardDef {
            id: "Demon Form", name: "Demon Form", card_type: CardType::Power,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["demon_form"],
        });
        insert(cards, CardDef {
            id: "Demon Form+", name: "Demon Form+", card_type: CardType::Power,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["demon_form"],
        });

        // ---- Ironclad Rare: Double Tap ---- (cost 1, next attack played twice; upgrade: 2 attacks)
        insert(cards, CardDef {
            id: "Double Tap", name: "Double Tap", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["double_tap"],
        });
        insert(cards, CardDef {
            id: "Double Tap+", name: "Double Tap+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["double_tap"],
        });

        // ---- Ironclad Rare: Exhume ---- (cost 1, exhaust, put card from exhaust pile into hand; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Exhume", name: "Exhume", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["exhume"],
        });
        insert(cards, CardDef {
            id: "Exhume+", name: "Exhume+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["exhume"],
        });

        // ---- Ironclad Rare: Feed ---- (cost 1, 10 dmg, exhaust, +3 max HP on kill; +2/+1)
        insert(cards, CardDef {
            id: "Feed", name: "Feed", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["feed"],
        });
        insert(cards, CardDef {
            id: "Feed+", name: "Feed+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["feed"],
        });

        // ---- Ironclad Rare: Fiend Fire ---- (cost 2, exhaust, 7 dmg per card in hand exhausted; +3 dmg)
        insert(cards, CardDef {
            id: "Fiend Fire", name: "Fiend Fire", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["fiend_fire"],
        });
        insert(cards, CardDef {
            id: "Fiend Fire+", name: "Fiend Fire+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["fiend_fire"],
        });

        // ---- Ironclad Rare: Immolate ---- (cost 2, 21 AoE dmg, add Burn to discard; +7 dmg)
        insert(cards, CardDef {
            id: "Immolate", name: "Immolate", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 21, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_burn_to_discard"],
        });
        insert(cards, CardDef {
            id: "Immolate+", name: "Immolate+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 28, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_burn_to_discard"],
        });

        // ---- Ironclad Rare: Impervious ---- (cost 2, 30 block, exhaust; +10 block)
        insert(cards, CardDef {
            id: "Impervious", name: "Impervious", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 30,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Impervious+", name: "Impervious+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 40,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[],
        });

        // ---- Ironclad Rare: Juggernaut ---- (cost 2, power, deal 5 dmg to random enemy on block; +2 magic)
        insert(cards, CardDef {
            id: "Juggernaut", name: "Juggernaut", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["juggernaut"],
        });
        insert(cards, CardDef {
            id: "Juggernaut+", name: "Juggernaut+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: false, enter_stance: None,
            effects: &["juggernaut"],
        });

        // ---- Ironclad Rare: Limit Break ---- (cost 1, double str, exhaust; upgrade: no exhaust)
        insert(cards, CardDef {
            id: "Limit Break", name: "Limit Break", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["double_strength"],
        });
        insert(cards, CardDef {
            id: "Limit Break+", name: "Limit Break+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["double_strength"],
        });

        // ---- Ironclad Rare: Offering ---- (cost 0, lose 6 HP, gain 2 energy, draw 3, exhaust; +2 draw)
        insert(cards, CardDef {
            id: "Offering", name: "Offering", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["offering"],
        });
        insert(cards, CardDef {
            id: "Offering+", name: "Offering+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["offering"],
        });

        // ---- Ironclad Rare: Reaper ---- (cost 2, 4 AoE dmg, heal for unblocked, exhaust; +1 dmg)
        insert(cards, CardDef {
            id: "Reaper", name: "Reaper", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["reaper"],
        });
        insert(cards, CardDef {
            id: "Reaper+", name: "Reaper+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 5, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["reaper"],
        });
}

fn insert(map: &mut HashMap<&'static str, CardDef>, card: CardDef) {
    map.insert(card.id, card);
}
