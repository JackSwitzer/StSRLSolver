use std::collections::HashMap;
use super::{CardDef, CardType, CardTarget};
use crate::effects::declarative::{Effect as E, SimpleEffect as SE, Target as T, AmountSource as A, Pile as P, BoolFlag as BF};
use crate::status_ids::sid;

pub fn register_ironclad(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Basic: Bash ---- (cost 2, 8 dmg, 2 vuln; +2/+1)
        insert(cards, CardDef {
            id: "Bash", name: "Bash", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 8, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["vulnerable"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Bash+", name: "Bash+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["vulnerable"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Common: Anger ---- (cost 0, 6 dmg, add copy to discard; +2 dmg)
        insert(cards, CardDef {
            id: "Anger", name: "Anger", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["copy_to_discard"], effect_data: &[
                E::Simple(SE::CopyThisCardTo(P::Discard)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Anger+", name: "Anger+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["copy_to_discard"], effect_data: &[
                E::Simple(SE::CopyThisCardTo(P::Discard)),
            ], complex_hook: None,
        });

        // ---- Ironclad Common: Armaments ---- (cost 1, 5 block, upgrade 1 card in hand; upgrade: all cards)
        insert(cards, CardDef {
            id: "Armaments", name: "Armaments", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["upgrade_one_card"], effect_data: &[
                E::ChooseCards {
                    source: P::Hand,
                    filter: crate::effects::declarative::CardFilter::Upgradeable,
                    action: crate::effects::declarative::ChoiceAction::Upgrade,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                },
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Armaments+", name: "Armaments+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["upgrade_all_cards"], effect_data: &[
                E::ForEachInPile {
                    pile: P::Hand,
                    filter: crate::effects::declarative::CardFilter::Upgradeable,
                    action: crate::effects::declarative::BulkAction::Upgrade,
                },
            ], complex_hook: None,
        });

        // ---- Ironclad Common: Body Slam ---- (cost 1, dmg = current block; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Body Slam", name: "Body Slam", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 0, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_equals_block"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Body Slam+", name: "Body Slam+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 0, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_equals_block"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Common: Clash ---- (cost 0, 14 dmg, only if hand is all attacks; +4 dmg)
        insert(cards, CardDef {
            id: "Clash", name: "Clash", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 14, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["only_attacks_in_hand"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Clash+", name: "Clash+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 18, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["only_attacks_in_hand"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Common: Cleave ---- (cost 1, 8 dmg AoE; +3 dmg)
        insert(cards, CardDef {
            id: "Cleave", name: "Cleave", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Cleave+", name: "Cleave+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 11, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Common: Clothesline ---- (cost 2, 12 dmg, 2 weak; +2/+1)
        insert(cards, CardDef {
            id: "Clothesline", name: "Clothesline", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["weak"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Clothesline+", name: "Clothesline+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 14, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["weak"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Common: Flex ---- (cost 0, +2 str this turn; +2 magic)
        insert(cards, CardDef {
            id: "Flex", name: "Flex", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["temp_strength"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                E::Simple(SE::AddStatus(T::Player, sid::TEMP_STRENGTH, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Flex+", name: "Flex+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["temp_strength"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                E::Simple(SE::AddStatus(T::Player, sid::TEMP_STRENGTH, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Common: Havoc ---- (cost 1, play top card of draw pile; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Havoc", name: "Havoc", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["play_top_card"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Havoc+", name: "Havoc+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["play_top_card"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Common: Headbutt ---- (cost 1, 9 dmg, put card from discard on top of draw; +3 dmg)
        insert(cards, CardDef {
            id: "Headbutt", name: "Headbutt", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["discard_to_top_of_draw"], effect_data: &[
                E::ChooseCards {
                    source: P::Discard,
                    filter: crate::effects::declarative::CardFilter::All,
                    action: crate::effects::declarative::ChoiceAction::PutOnTopOfDraw,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                },
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Headbutt+", name: "Headbutt+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["discard_to_top_of_draw"], effect_data: &[
                E::ChooseCards {
                    source: P::Discard,
                    filter: crate::effects::declarative::CardFilter::All,
                    action: crate::effects::declarative::ChoiceAction::PutOnTopOfDraw,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                },
            ], complex_hook: None,
        });

        // ---- Ironclad Common: Heavy Blade ---- (cost 2, 14 dmg, 3x str scaling; upgrade: 5x str)
        insert(cards, CardDef {
            id: "Heavy Blade", name: "Heavy Blade", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 14, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["heavy_blade"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Heavy Blade+", name: "Heavy Blade+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 14, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["heavy_blade"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Common: Iron Wave ---- (cost 1, 5 dmg + 5 block; +2/+2)
        insert(cards, CardDef {
            id: "Iron Wave", name: "Iron Wave", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Iron Wave+", name: "Iron Wave+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Common: Perfected Strike ---- (cost 2, 6 dmg + 2/strike in deck; +1 magic)
        insert(cards, CardDef {
            id: "Perfected Strike", name: "Perfected Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 6, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["perfected_strike"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Perfected Strike+", name: "Perfected Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 6, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["perfected_strike"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Common: Pommel Strike ---- (cost 1, 9 dmg, draw 1; +1/+1)
        insert(cards, CardDef {
            id: "Pommel Strike", name: "Pommel Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Pommel Strike+", name: "Pommel Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Common: Shrug It Off ---- (cost 1, 8 block, draw 1; +3 block)
        insert(cards, CardDef {
            id: "Shrug It Off", name: "Shrug It Off", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Fixed(1))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Shrug It Off+", name: "Shrug It Off+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 11,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Fixed(1))),
            ], complex_hook: None,
        });

        // ---- Ironclad Common: Sword Boomerang ---- (cost 1, 3 dmg x3 random; +1 magic)
        insert(cards, CardDef {
            id: "Sword Boomerang", name: "Sword Boomerang", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 3, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["damage_random_x_times"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Sword Boomerang+", name: "Sword Boomerang+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 3, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["damage_random_x_times"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Common: Thunderclap ---- (cost 1, 4 dmg AoE + 1 vuln all; +3 dmg)
        insert(cards, CardDef {
            id: "Thunderclap", name: "Thunderclap", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 4, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["vulnerable_all"], effect_data: &[
                E::Simple(SE::AddStatus(T::AllEnemies, sid::VULNERABLE, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Thunderclap+", name: "Thunderclap+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["vulnerable_all"], effect_data: &[
                E::Simple(SE::AddStatus(T::AllEnemies, sid::VULNERABLE, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Common: True Grit ---- (cost 1, 7 block, exhaust random card; upgrade: +2 block, choose)
        insert(cards, CardDef {
            id: "True Grit", name: "True Grit", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["exhaust_random"], effect_data: &[], complex_hook: None,
            // exhaust_random is complex (RNG-based), leave for old path
        });
        insert(cards, CardDef {
            id: "True Grit+", name: "True Grit+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 9,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["exhaust_choose"], effect_data: &[
                E::ChooseCards {
                    source: P::Hand,
                    filter: crate::effects::declarative::CardFilter::All,
                    action: crate::effects::declarative::ChoiceAction::Exhaust,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                },
            ], complex_hook: None,
        });

        // ---- Ironclad Common: Twin Strike ---- (cost 1, 5 dmg x2; +2 dmg)
        insert(cards, CardDef {
            id: "Twin Strike", name: "Twin Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Twin Strike+", name: "Twin Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Common: Warcry ---- (cost 0, draw 1, put 1 on top, exhaust; +1 draw)
        insert(cards, CardDef {
            id: "Warcry", name: "Warcry", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["draw", "put_card_on_top"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
                E::ChooseCards {
                    source: P::Hand,
                    filter: crate::effects::declarative::CardFilter::All,
                    action: crate::effects::declarative::ChoiceAction::PutOnTopOfDraw,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                },
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Warcry+", name: "Warcry+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["draw", "put_card_on_top"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
                E::ChooseCards {
                    source: P::Hand,
                    filter: crate::effects::declarative::CardFilter::All,
                    action: crate::effects::declarative::ChoiceAction::PutOnTopOfDraw,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                },
            ], complex_hook: None,
        });

        // ---- Ironclad Common: Wild Strike ---- (cost 1, 12 dmg, shuffle Wound into draw; +5 dmg)
        insert(cards, CardDef {
            id: "Wild Strike", name: "Wild Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_wound_to_draw"], effect_data: &[
                E::Simple(SE::AddCard("Wound", P::Draw, A::Fixed(1))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Wild Strike+", name: "Wild Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 17, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_wound_to_draw"], effect_data: &[
                E::Simple(SE::AddCard("Wound", P::Draw, A::Fixed(1))),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Battle Trance ---- (cost 0, draw 3, no more draw; +1)
        insert(cards, CardDef {
            id: "Battle Trance", name: "Battle Trance", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["draw", "no_draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
                E::Simple(SE::SetFlag(BF::NoDraw)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Battle Trance+", name: "Battle Trance+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["draw", "no_draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
                E::Simple(SE::SetFlag(BF::NoDraw)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Blood for Blood ---- (cost 4, 18 dmg, -1 cost per HP loss; upgrade: cost 3, +4 dmg)
        insert(cards, CardDef {
            id: "Blood for Blood", name: "Blood for Blood", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 4, base_damage: 18, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["cost_reduce_on_hp_loss"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Blood for Blood+", name: "Blood for Blood+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 3, base_damage: 22, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["cost_reduce_on_hp_loss"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Bloodletting ---- (cost 0, lose 3 HP, gain 2 energy; +1 energy)
        insert(cards, CardDef {
            id: "Bloodletting", name: "Bloodletting", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["lose_hp_gain_energy"], effect_data: &[
                E::Simple(SE::ModifyHp(A::Fixed(-3))),
                E::Simple(SE::GainEnergy(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Bloodletting+", name: "Bloodletting+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["lose_hp_gain_energy"], effect_data: &[
                E::Simple(SE::ModifyHp(A::Fixed(-3))),
                E::Simple(SE::GainEnergy(A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Burning Pact ---- (cost 1, exhaust 1 card, draw 2; +1 draw)
        insert(cards, CardDef {
            id: "Burning Pact", name: "Burning Pact", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["exhaust_choose", "draw"], effect_data: &[], complex_hook: None,
            // exhaust_choose triggers AwaitingChoice; draw follows -- leave for old path
        });
        insert(cards, CardDef {
            id: "Burning Pact+", name: "Burning Pact+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["exhaust_choose", "draw"], effect_data: &[], complex_hook: None,
            // exhaust_choose triggers AwaitingChoice; draw follows -- leave for old path
        });

        // ---- Ironclad Uncommon: Carnage ---- (cost 2, 20 dmg, ethereal; +8 dmg)
        insert(cards, CardDef {
            id: "Carnage", name: "Carnage", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["ethereal"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Carnage+", name: "Carnage+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 28, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["ethereal"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Combust ---- (cost 1, power, lose 1 HP/turn, deal 5 dmg to all; +2 magic)
        insert(cards, CardDef {
            id: "Combust", name: "Combust", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["combust"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::COMBUST, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Combust+", name: "Combust+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: false, enter_stance: None,
            effects: &["combust"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::COMBUST, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Dark Embrace ---- (cost 2, power, draw 1 on exhaust; upgrade: cost 1)
        insert(cards, CardDef {
            id: "Dark Embrace", name: "Dark Embrace", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["dark_embrace"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::DARK_EMBRACE, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Dark Embrace+", name: "Dark Embrace+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["dark_embrace"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::DARK_EMBRACE, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Disarm ---- (cost 1, -2 str to enemy, exhaust; +1 magic)
        insert(cards, CardDef {
            id: "Disarm", name: "Disarm", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["reduce_strength"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Disarm+", name: "Disarm+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["reduce_strength"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Dropkick ---- (cost 1, 5 dmg, if vuln: +1 energy + draw 1; +3 dmg)
        insert(cards, CardDef {
            id: "Dropkick", name: "Dropkick", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["if_vulnerable_energy_draw"], effect_data: &[
                E::Conditional(
                    crate::effects::declarative::Condition::EnemyHasStatus(sid::VULNERABLE),
                    &[E::Simple(SE::GainEnergy(A::Fixed(1))), E::Simple(SE::DrawCards(A::Fixed(1)))],
                    &[],
                ),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Dropkick+", name: "Dropkick+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["if_vulnerable_energy_draw"], effect_data: &[
                E::Conditional(
                    crate::effects::declarative::Condition::EnemyHasStatus(sid::VULNERABLE),
                    &[E::Simple(SE::GainEnergy(A::Fixed(1))), E::Simple(SE::DrawCards(A::Fixed(1)))],
                    &[],
                ),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Dual Wield ---- (cost 1, copy 1 attack/power in hand; upgrade: 2 copies)
        insert(cards, CardDef {
            id: "Dual Wield", name: "Dual Wield", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["dual_wield"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Dual Wield+", name: "Dual Wield+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["dual_wield"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Entrench ---- (cost 2, double block; upgrade: cost 1)
        insert(cards, CardDef {
            id: "Entrench", name: "Entrench", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["double_block"], effect_data: &[
                E::Simple(SE::GainBlock(A::PlayerBlock)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Entrench+", name: "Entrench+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["double_block"], effect_data: &[
                E::Simple(SE::GainBlock(A::PlayerBlock)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Evolve ---- (cost 1, power, draw 1 when Status drawn; upgrade: draw 2)
        insert(cards, CardDef {
            id: "Evolve", name: "Evolve", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["evolve"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::EVOLVE, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Evolve+", name: "Evolve+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["evolve"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::EVOLVE, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Feel No Pain ---- (cost 1, power, 3 block on exhaust; +1 magic)
        insert(cards, CardDef {
            id: "Feel No Pain", name: "Feel No Pain", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["feel_no_pain"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::FEEL_NO_PAIN, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Feel No Pain+", name: "Feel No Pain+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["feel_no_pain"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::FEEL_NO_PAIN, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Fire Breathing ---- (cost 1, power, 6 dmg on Status/Curse draw; +4 magic)
        insert(cards, CardDef {
            id: "Fire Breathing", name: "Fire Breathing", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 6, exhaust: false, enter_stance: None,
            effects: &["fire_breathing"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::FIRE_BREATHING, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Fire Breathing+", name: "Fire Breathing+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 10, exhaust: false, enter_stance: None,
            effects: &["fire_breathing"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::FIRE_BREATHING, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Flame Barrier ---- (cost 2, 12 block + 4 fire dmg when hit; +4/+2)
        insert(cards, CardDef {
            id: "Flame Barrier", name: "Flame Barrier", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 12,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["flame_barrier"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::FLAME_BARRIER, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Flame Barrier+", name: "Flame Barrier+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 16,
            base_magic: 6, exhaust: false, enter_stance: None,
            effects: &["flame_barrier"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::FLAME_BARRIER, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Ghostly Armor ---- (cost 1, 10 block, ethereal; +3 block)
        insert(cards, CardDef {
            id: "Ghostly Armor", name: "Ghostly Armor", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 10,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["ethereal"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Ghostly Armor+", name: "Ghostly Armor+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 13,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["ethereal"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Hemokinesis ---- (cost 1, 15 dmg, lose 2 HP; +5 dmg)
        insert(cards, CardDef {
            id: "Hemokinesis", name: "Hemokinesis", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["lose_hp"], effect_data: &[
                E::Simple(SE::ModifyHp(A::Fixed(-2))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Hemokinesis+", name: "Hemokinesis+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 20, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["lose_hp"], effect_data: &[
                E::Simple(SE::ModifyHp(A::Fixed(-2))),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Infernal Blade ---- (cost 1, exhaust, add random attack to hand at cost 0; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Infernal Blade", name: "Infernal Blade", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["random_attack_to_hand"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Infernal Blade+", name: "Infernal Blade+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["random_attack_to_hand"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Inflame ---- (cost 1, power, +2 str; +1)
        insert(cards, CardDef {
            id: "Inflame", name: "Inflame", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["gain_strength"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Inflame+", name: "Inflame+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["gain_strength"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Intimidate ---- (cost 0, 1 weak to all, exhaust; +1 magic)
        insert(cards, CardDef {
            id: "Intimidate", name: "Intimidate", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["weak_all"], effect_data: &[
                E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Intimidate+", name: "Intimidate+", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["weak_all"], effect_data: &[
                E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Metallicize ---- (cost 1, power, +3 block/turn; +1)
        insert(cards, CardDef {
            id: "Metallicize", name: "Metallicize", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["metallicize"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::METALLICIZE, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Metallicize+", name: "Metallicize+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["metallicize"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::METALLICIZE, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Power Through ---- (cost 1, 15 block, add 2 Wounds to hand; +5 block)
        insert(cards, CardDef {
            id: "Power Through", name: "Power Through", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 15,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_wounds_to_hand"], effect_data: &[
                E::Simple(SE::AddCard("Wound", P::Hand, A::Fixed(2))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Power Through+", name: "Power Through+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 20,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_wounds_to_hand"], effect_data: &[
                E::Simple(SE::AddCard("Wound", P::Hand, A::Fixed(2))),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Pummel ---- (cost 1, 2 dmg x4, exhaust; +1 hit)
        insert(cards, CardDef {
            id: "Pummel", name: "Pummel", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 2, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Pummel+", name: "Pummel+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 2, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Rage ---- (cost 0, gain 3 block per attack played this turn; +2 magic)
        insert(cards, CardDef {
            id: "Rage", name: "Rage", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["rage"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::RAGE, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Rage+", name: "Rage+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["rage"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::RAGE, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Rampage ---- (cost 1, 8 dmg, +5 dmg each play; +3 magic)
        insert(cards, CardDef {
            id: "Rampage", name: "Rampage", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["rampage"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Rampage+", name: "Rampage+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 8, exhaust: false, enter_stance: None,
            effects: &["rampage"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Reckless Charge ---- (cost 0, 7 dmg, shuffle Dazed into draw; +3 dmg)
        insert(cards, CardDef {
            id: "Reckless Charge", name: "Reckless Charge", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_dazed_to_draw"], effect_data: &[
                E::Simple(SE::AddCard("Dazed", P::Draw, A::Fixed(1))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Reckless Charge+", name: "Reckless Charge+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_dazed_to_draw"], effect_data: &[
                E::Simple(SE::AddCard("Dazed", P::Draw, A::Fixed(1))),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Rupture ---- (cost 1, power, +1 str when lose HP from card; +1 magic)
        insert(cards, CardDef {
            id: "Rupture", name: "Rupture", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["rupture"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::RUPTURE, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Rupture+", name: "Rupture+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["rupture"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::RUPTURE, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Searing Blow ---- (cost 2, 12 dmg, can upgrade infinitely; +4+N per upgrade)
        insert(cards, CardDef {
            id: "Searing Blow", name: "Searing Blow", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["searing_blow"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Searing Blow+", name: "Searing Blow+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 16, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["searing_blow"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Second Wind ---- (cost 1, exhaust all non-attack, gain block per; +2 block)
        insert(cards, CardDef {
            id: "Second Wind", name: "Second Wind", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["second_wind"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Second Wind+", name: "Second Wind+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["second_wind"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Seeing Red ---- (cost 1, gain 2 energy, exhaust; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Seeing Red", name: "Seeing Red", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["gain_energy"], effect_data: &[
                E::Simple(SE::GainEnergy(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Seeing Red+", name: "Seeing Red+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["gain_energy"], effect_data: &[
                E::Simple(SE::GainEnergy(A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Sentinel ---- (cost 1, 5 block, gain 2 energy on exhaust; +3 block, 3 energy)
        insert(cards, CardDef {
            id: "Sentinel", name: "Sentinel", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["energy_on_exhaust"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Sentinel+", name: "Sentinel+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["energy_on_exhaust"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Sever Soul ---- (cost 2, 16 dmg, exhaust all non-attacks in hand; +6 dmg)
        insert(cards, CardDef {
            id: "Sever Soul", name: "Sever Soul", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 16, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["exhaust_non_attacks"], effect_data: &[
                E::ForEachInPile {
                    pile: P::Hand,
                    filter: crate::effects::declarative::CardFilter::NonAttacks,
                    action: crate::effects::declarative::BulkAction::Exhaust,
                },
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Sever Soul+", name: "Sever Soul+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 22, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["exhaust_non_attacks"], effect_data: &[
                E::ForEachInPile {
                    pile: P::Hand,
                    filter: crate::effects::declarative::CardFilter::NonAttacks,
                    action: crate::effects::declarative::BulkAction::Exhaust,
                },
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Shockwave ---- (cost 2, 3 weak+vuln to all, exhaust; +2 magic)
        insert(cards, CardDef {
            id: "Shockwave", name: "Shockwave", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["weak_all", "vulnerable_all"], effect_data: &[
                E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Magic)),
                E::Simple(SE::AddStatus(T::AllEnemies, sid::VULNERABLE, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Shockwave+", name: "Shockwave+", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["weak_all", "vulnerable_all"], effect_data: &[
                E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Magic)),
                E::Simple(SE::AddStatus(T::AllEnemies, sid::VULNERABLE, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Spot Weakness ---- (cost 1, +3 str if enemy attacking; +1 magic)
        insert(cards, CardDef {
            id: "Spot Weakness", name: "Spot Weakness", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["spot_weakness"], effect_data: &[
                E::Conditional(
                    crate::effects::declarative::Condition::EnemyAttacking,
                    &[E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic))],
                    &[],
                ),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Spot Weakness+", name: "Spot Weakness+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["spot_weakness"], effect_data: &[
                E::Conditional(
                    crate::effects::declarative::Condition::EnemyAttacking,
                    &[E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic))],
                    &[],
                ),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Uppercut ---- (cost 2, 13 dmg, 1 weak + 1 vuln; +1/+1)
        insert(cards, CardDef {
            id: "Uppercut", name: "Uppercut", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["weak", "vulnerable"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Uppercut+", name: "Uppercut+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["weak", "vulnerable"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Uncommon: Whirlwind ---- (cost X, 5 dmg AoE per X; +3 dmg)
        insert(cards, CardDef {
            id: "Whirlwind", name: "Whirlwind", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: -1, base_damage: 5, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["x_cost"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Whirlwind+", name: "Whirlwind+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: -1, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["x_cost"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Rare: Barricade ---- (cost 3, power, block not removed at end of turn; upgrade: cost 2)
        insert(cards, CardDef {
            id: "Barricade", name: "Barricade", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["barricade"], effect_data: &[
                E::Simple(SE::SetStatus(T::Player, sid::BARRICADE, A::Fixed(1))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Barricade+", name: "Barricade+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["barricade"], effect_data: &[
                E::Simple(SE::SetStatus(T::Player, sid::BARRICADE, A::Fixed(1))),
            ], complex_hook: None,
        });

        // ---- Ironclad Rare: Berserk ---- (cost 0, power, 2 vuln to self, +1 energy/turn; -1 vuln)
        insert(cards, CardDef {
            id: "Berserk", name: "Berserk", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["berserk"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::VULNERABLE, A::Magic)),
                E::Simple(SE::AddStatus(T::Player, sid::BERSERK, A::Fixed(1))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Berserk+", name: "Berserk+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["berserk"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::VULNERABLE, A::Magic)),
                E::Simple(SE::AddStatus(T::Player, sid::BERSERK, A::Fixed(1))),
            ], complex_hook: None,
        });

        // ---- Ironclad Rare: Bludgeon ---- (cost 3, 32 dmg; +10 dmg)
        insert(cards, CardDef {
            id: "Bludgeon", name: "Bludgeon", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 3, base_damage: 32, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Bludgeon+", name: "Bludgeon+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 3, base_damage: 42, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Rare: Brutality ---- (cost 0, power, lose 1 HP + draw 1 at turn start; upgrade: innate)
        insert(cards, CardDef {
            id: "Brutality", name: "Brutality", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["brutality"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::BRUTALITY, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Brutality+", name: "Brutality+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["brutality", "innate"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::BRUTALITY, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Rare: Corruption ---- (cost 3, power, skills cost 0 but exhaust; upgrade: cost 2)
        insert(cards, CardDef {
            id: "Corruption", name: "Corruption", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["corruption"], effect_data: &[
                E::Simple(SE::SetStatus(T::Player, sid::CORRUPTION, A::Fixed(1))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Corruption+", name: "Corruption+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["corruption"], effect_data: &[
                E::Simple(SE::SetStatus(T::Player, sid::CORRUPTION, A::Fixed(1))),
            ], complex_hook: None,
        });

        // ---- Ironclad Rare: Demon Form ---- (cost 3, power, +2 str/turn; +1 magic)
        insert(cards, CardDef {
            id: "Demon Form", name: "Demon Form", card_type: CardType::Power,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["demon_form"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::DEMON_FORM, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Demon Form+", name: "Demon Form+", card_type: CardType::Power,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["demon_form"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::DEMON_FORM, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Rare: Double Tap ---- (cost 1, next attack played twice; upgrade: 2 attacks)
        insert(cards, CardDef {
            id: "Double Tap", name: "Double Tap", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["double_tap"], effect_data: &[
                E::Simple(SE::SetStatus(T::Player, sid::DOUBLE_TAP, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Double Tap+", name: "Double Tap+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["double_tap"], effect_data: &[
                E::Simple(SE::SetStatus(T::Player, sid::DOUBLE_TAP, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Rare: Exhume ---- (cost 1, exhaust, put card from exhaust pile into hand; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Exhume", name: "Exhume", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["exhume"], effect_data: &[
                E::ChooseCards {
                    source: P::Exhaust,
                    filter: crate::effects::declarative::CardFilter::All,
                    action: crate::effects::declarative::ChoiceAction::MoveToHand,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                },
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Exhume+", name: "Exhume+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["exhume"], effect_data: &[
                E::ChooseCards {
                    source: P::Exhaust,
                    filter: crate::effects::declarative::CardFilter::All,
                    action: crate::effects::declarative::ChoiceAction::MoveToHand,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                },
            ], complex_hook: None,
        });

        // ---- Ironclad Rare: Feed ---- (cost 1, 10 dmg, exhaust, +3 max HP on kill; +2/+1)
        insert(cards, CardDef {
            id: "Feed", name: "Feed", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["feed"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Feed+", name: "Feed+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["feed"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Rare: Fiend Fire ---- (cost 2, exhaust, 7 dmg per card in hand exhausted; +3 dmg)
        insert(cards, CardDef {
            id: "Fiend Fire", name: "Fiend Fire", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["fiend_fire"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Fiend Fire+", name: "Fiend Fire+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["fiend_fire"], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Rare: Immolate ---- (cost 2, 21 AoE dmg, add Burn to discard; +7 dmg)
        insert(cards, CardDef {
            id: "Immolate", name: "Immolate", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 21, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_burn_to_discard"], effect_data: &[
                E::Simple(SE::AddCard("Burn", P::Discard, A::Fixed(1))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Immolate+", name: "Immolate+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 28, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_burn_to_discard"], effect_data: &[
                E::Simple(SE::AddCard("Burn", P::Discard, A::Fixed(1))),
            ], complex_hook: None,
        });

        // ---- Ironclad Rare: Impervious ---- (cost 2, 30 block, exhaust; +10 block)
        insert(cards, CardDef {
            id: "Impervious", name: "Impervious", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 30,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Impervious+", name: "Impervious+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 40,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Ironclad Rare: Juggernaut ---- (cost 2, power, deal 5 dmg to random enemy on block; +2 magic)
        insert(cards, CardDef {
            id: "Juggernaut", name: "Juggernaut", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["juggernaut"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::JUGGERNAUT, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Juggernaut+", name: "Juggernaut+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: false, enter_stance: None,
            effects: &["juggernaut"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::JUGGERNAUT, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Rare: Limit Break ---- (cost 1, double str, exhaust; upgrade: no exhaust)
        insert(cards, CardDef {
            id: "Limit Break", name: "Limit Break", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["double_strength"], effect_data: &[
                E::Simple(SE::MultiplyStatus(T::Player, sid::STRENGTH, 2)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Limit Break+", name: "Limit Break+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["double_strength"], effect_data: &[
                E::Simple(SE::MultiplyStatus(T::Player, sid::STRENGTH, 2)),
            ], complex_hook: None,
        });

        // ---- Ironclad Rare: Offering ---- (cost 0, lose 6 HP, gain 2 energy, draw 3, exhaust; +2 draw)
        insert(cards, CardDef {
            id: "Offering", name: "Offering", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["offering"], effect_data: &[
                E::Simple(SE::ModifyHp(A::Fixed(-6))),
                E::Simple(SE::GainEnergy(A::Fixed(2))),
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Offering+", name: "Offering+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["offering"], effect_data: &[
                E::Simple(SE::ModifyHp(A::Fixed(-6))),
                E::Simple(SE::GainEnergy(A::Fixed(2))),
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });

        // ---- Ironclad Rare: Reaper ---- (cost 2, 4 AoE dmg, heal for unblocked, exhaust; +1 dmg)
        insert(cards, CardDef {
            id: "Reaper", name: "Reaper", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["reaper"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Reaper+", name: "Reaper+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 5, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["reaper"], effect_data: &[], complex_hook: None,
        });
}

fn insert(map: &mut HashMap<&'static str, CardDef>, card: CardDef) {
    map.insert(card.id, card);
}
