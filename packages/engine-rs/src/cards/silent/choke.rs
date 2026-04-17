use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Choke ---- (cost 2, 12 dmg, deal 3 dmg per card played this turn; +2 magic)
    insert(cards, CardDef {
                id: "Choke", name: "Choke", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::CONSTRICTED, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Choke+", name: "Choke+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::CONSTRICTED, A::Magic)),
                ], complex_hook: None,
            });
}
