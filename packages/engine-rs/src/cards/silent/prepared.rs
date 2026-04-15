use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Prepared ---- (cost 0, draw 1, discard 1; upgrade: draw 2 discard 2)
    insert(cards, CardDef {
                id: "Prepared", name: "Prepared", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Magic)), E::ChooseCards { source: P::Hand, filter: CardFilter::All, action: ChoiceAction::Discard, min_picks: A::Magic, max_picks: A::Magic, post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0) }], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Prepared+", name: "Prepared+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Magic)), E::ChooseCards { source: P::Hand, filter: CardFilter::All, action: ChoiceAction::Discard, min_picks: A::Magic, max_picks: A::Magic, post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0) }], complex_hook: None,
            });
}
