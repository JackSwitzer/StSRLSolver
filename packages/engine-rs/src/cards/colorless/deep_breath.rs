use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Deep Breath: 0 cost, shuffle discard into draw, draw 1
    insert(cards, CardDef {
                id: "Deep Breath", name: "Deep Breath", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["shuffle_discard_into_draw", "draw"], effect_data: &[E::Simple(SE::ShuffleDiscardIntoDraw), E::Simple(SE::DrawCards(A::Magic))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Deep Breath+", name: "Deep Breath+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["shuffle_discard_into_draw", "draw"], effect_data: &[E::Simple(SE::ShuffleDiscardIntoDraw), E::Simple(SE::DrawCards(A::Magic))], complex_hook: None,
            });
}
