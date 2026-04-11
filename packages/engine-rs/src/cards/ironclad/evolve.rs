use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Evolve ---- (cost 1, power, draw 1 when Status drawn; upgrade: draw 2)
    insert(cards, CardDef {
                id: "Evolve", name: "Evolve", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["evolve"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::EVOLVE, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Evolve+", name: "Evolve+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["evolve"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::EVOLVE, A::Magic)),
                ], complex_hook: None,
            });
}
