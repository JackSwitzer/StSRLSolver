use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: reference/extracted/methods/card/Dash.java queues 10 Block before
    // 10 damage; upgrading adds 3 to both values and does not change cost.
    insert(cards, CardDef {
        id: "Dash", name: "Dash", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: 10,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::GainBlock(A::Block)),
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Dash+", name: "Dash+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: 13,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::GainBlock(A::Block)),
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ], complex_hook: None,
    });
}
