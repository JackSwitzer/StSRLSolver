use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Sadistic Nature: 0 cost, power, deal 5 dmg whenever you apply debuff
    insert(cards, CardDef {
        id: "Sadistic Nature", name: "Sadistic Nature", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 5, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::SADISTIC, A::Magic))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Sadistic Nature+", name: "Sadistic Nature+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 7, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::SADISTIC, A::Magic))],
        complex_hook: None,
    });
}
