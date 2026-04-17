use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Through Violence (from Reach Heaven): cost 0, 20 dmg, retain
    insert(cards, CardDef {
        id: "ThroughViolence", name: "Through Violence", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 20, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "ThroughViolence+", name: "Through Violence+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 30, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ], complex_hook: None,
    });
}
