use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, CardFilter, Effect as E, Pile as P, SimpleEffect as SE};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Violence: 0 cost, put 3 random Attacks from draw pile into hand, exhaust
    insert(cards, CardDef {
                id: "Violence", name: "Violence", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["draw_attacks_from_draw"], effect_data: &[
                    E::Simple(SE::DrawRandomCardsFromPileToHand(P::Draw, CardFilter::Attacks, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Violence+", name: "Violence+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: true, enter_stance: None,
                effects: &["draw_attacks_from_draw"], effect_data: &[
                    E::Simple(SE::DrawRandomCardsFromPileToHand(P::Draw, CardFilter::Attacks, A::Magic)),
                ], complex_hook: None,
            });
}
