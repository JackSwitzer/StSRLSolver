use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Accuracy ---- (cost 1, power, Shivs +4 dmg; +2)
    insert(cards, CardDef {
                id: "Accuracy", name: "Accuracy", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["accuracy"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ACCURACY, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Accuracy+", name: "Accuracy+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 6, exhaust: false, enter_stance: None,
                effects: &["accuracy"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ACCURACY, A::Magic)),
                ], complex_hook: None,
            });
}
