use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Silent Special: Shiv ---- (cost 0, 4 dmg, exhaust; +2 dmg)
    insert(cards, CardDef {
        id: "Shiv", name: "Shiv", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
        effects: &[], effect_data: &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Shiv+", name: "Shiv+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
        effects: &[], effect_data: &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ], complex_hook: None,
    });
}
