use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Rare: Deva Form ---- (cost 3, power, ethereal, gain 1 energy each turn (stacks); upgrade: no ethereal)
    insert(cards, CardDef {
        id: "DevaForm", name: "Deva Form", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::DEVA_FORM, A::Magic)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "DevaForm+", name: "Deva Form+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::DEVA_FORM, A::Magic)),
        ], complex_hook: None,
    });
}
