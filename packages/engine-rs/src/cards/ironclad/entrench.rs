use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Ironclad Uncommon: Entrench ---- (cost 2, double block; upgrade: cost 1)
    insert(cards, CardDef {
        id: "Entrench",
        name: "Entrench",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: 2,
        base_damage: -1,
        base_block: -1,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::PlayerBlock))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Entrench+",
        name: "Entrench+",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: 1,
        base_damage: -1,
        base_block: -1,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::PlayerBlock))],
        complex_hook: None,
    });
}
