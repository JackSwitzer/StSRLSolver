use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: reference/extracted/methods/card/DaggerThrow.java deals 9 damage,
    // draws one card, then discards one card; upgrading adds 3 damage only.
    insert(cards, CardDef {
                id: "Dagger Throw", name: "Dagger Throw", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Fixed(1))), E::ChooseCards { source: P::Hand, filter: CardFilter::All, action: ChoiceAction::Discard, min_picks: A::Fixed(1), max_picks: A::Fixed(1), post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0) }], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Dagger Throw+", name: "Dagger Throw+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Fixed(1))), E::ChooseCards { source: P::Hand, filter: CardFilter::All, action: ChoiceAction::Discard, min_picks: A::Fixed(1), max_picks: A::Fixed(1), post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0) }], complex_hook: None,
            });
}
