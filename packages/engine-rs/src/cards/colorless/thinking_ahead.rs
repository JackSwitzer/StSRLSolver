use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Thinking Ahead", name: "Thinking Ahead", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Fixed(2))),
                    E::ChooseCards {
                        source: P::Hand, filter: CardFilter::All, action: ChoiceAction::PutOnTopOfDraw,
                        min_picks: A::Fixed(1), max_picks: A::Fixed(1),
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Thinking Ahead+", name: "Thinking Ahead+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Fixed(2))),
                    E::ChooseCards {
                        source: P::Hand, filter: CardFilter::All, action: ChoiceAction::PutOnTopOfDraw,
                        min_picks: A::Fixed(1), max_picks: A::Fixed(1),
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
}
