use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Berserk ---- (cost 0, power, 2 vuln to self, +1 energy/turn; -1 vuln)
    insert(cards, CardDef {
                id: "Berserk", name: "Berserk", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::VULNERABLE, A::Magic)),
                    E::Simple(SE::AddStatus(T::Player, sid::BERSERK, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Berserk+", name: "Berserk+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::VULNERABLE, A::Magic)),
                    E::Simple(SE::AddStatus(T::Player, sid::BERSERK, A::Fixed(1))),
                ], complex_hook: None,
            });
}
