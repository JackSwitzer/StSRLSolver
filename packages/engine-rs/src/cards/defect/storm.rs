use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Storm: 1 cost, power, channel 1 Lightning on power play (upgrade: innate)
    insert(cards, CardDef {
                id: "Storm", name: "Storm", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &[],
                effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::STORM, A::Magic))],
                complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Storm+", name: "Storm+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["innate"],
                effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::STORM, A::Magic))],
                complex_hook: None,
            });
}
