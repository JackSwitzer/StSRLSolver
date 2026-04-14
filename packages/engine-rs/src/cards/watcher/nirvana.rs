use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Uncommon: Nirvana ---- (cost 1, power, gain 3 block whenever you Scry; +1 magic upgrade)
    insert(cards, CardDef {
        id: "Nirvana", name: "Nirvana", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 3, exhaust: false, enter_stance: None,
        effects: &["on_scry_block"], effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::NIRVANA, A::Magic)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Nirvana+", name: "Nirvana+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 4, exhaust: false, enter_stance: None,
        effects: &["on_scry_block"], effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::NIRVANA, A::Magic)),
        ], complex_hook: None,
    });
}
