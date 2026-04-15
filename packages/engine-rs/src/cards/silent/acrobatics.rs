use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Acrobatics ---- (cost 1, draw 3, discard 1; +1 draw)
    insert(cards, CardDef {
                id: "Acrobatics", name: "Acrobatics", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Magic)), E::ChooseCards { source: P::Hand, filter: CardFilter::All, action: ChoiceAction::Discard, min_picks: A::Fixed(1), max_picks: A::Fixed(1), post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0) }], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Acrobatics+", name: "Acrobatics+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Magic)), E::ChooseCards { source: P::Hand, filter: CardFilter::All, action: ChoiceAction::Discard, min_picks: A::Fixed(1), max_picks: A::Fixed(1), post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0) }], complex_hook: None,
            });
}
