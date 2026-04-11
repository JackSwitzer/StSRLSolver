use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Barricade ---- (cost 3, power, block not removed at end of turn; upgrade: cost 2)
    insert(cards, CardDef {
                id: "Barricade", name: "Barricade", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["barricade"], effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::BARRICADE, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Barricade+", name: "Barricade+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["barricade"], effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::BARRICADE, A::Fixed(1))),
                ], complex_hook: None,
            });
}
