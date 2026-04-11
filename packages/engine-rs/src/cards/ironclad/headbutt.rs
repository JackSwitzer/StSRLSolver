use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
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
}
