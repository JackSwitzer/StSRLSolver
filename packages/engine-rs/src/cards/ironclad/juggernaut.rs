use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Juggernaut ---- (cost 2, power, deal 5 dmg to random enemy on block; +2 magic)
    insert(cards, CardDef {
                id: "Juggernaut", name: "Juggernaut", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::JUGGERNAUT, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Juggernaut+", name: "Juggernaut+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 7, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::JUGGERNAUT, A::Magic)),
                ], complex_hook: None,
            });
}
