use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Secret Weapon", name: "Secret Weapon", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["search_attack"], effect_data: &[
                    E::ChooseCards {
                        source: P::Draw, filter: CardFilter::Attacks, action: ChoiceAction::MoveToHand,
                        min_picks: A::Fixed(1), max_picks: A::Fixed(1),
                    },
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Secret Weapon+", name: "Secret Weapon+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["search_attack"], effect_data: &[
                    E::ChooseCards {
                        source: P::Draw, filter: CardFilter::Attacks, action: ChoiceAction::MoveToHand,
                        min_picks: A::Fixed(1), max_picks: A::Fixed(1),
                    },
                ], complex_hook: None,
            });
}
