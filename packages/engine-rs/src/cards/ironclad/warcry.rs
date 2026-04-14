use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
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
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
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
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
}
