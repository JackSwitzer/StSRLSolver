use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Hemokinesis ---- (cost 1, 15 dmg, lose 2 HP; +5 dmg)
    insert(cards, CardDef {
                id: "Hemokinesis", name: "Hemokinesis", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ModifyHp(A::Fixed(-2))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Hemokinesis+", name: "Hemokinesis+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 20, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ModifyHp(A::Fixed(-2))),
                ], complex_hook: None,
            });
}
