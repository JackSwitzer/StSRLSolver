use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Signature Move ---- (cost 2, 30 dmg, only playable if no other attacks in hand; +10 dmg upgrade)
    insert(cards, CardDef {
                id: "SignatureMove", name: "Signature Move", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 30, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "SignatureMove+", name: "Signature Move+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 40, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
}
