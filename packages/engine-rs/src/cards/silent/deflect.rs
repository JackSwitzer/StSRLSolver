use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Silent Common: Deflect ---- (cost 0, 4 block; +3)
    insert(cards, CardDef {
        id: "Deflect", name: "Deflect", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block))], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Deflect+", name: "Deflect+", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 7,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block))], complex_hook: None,
    });
}
