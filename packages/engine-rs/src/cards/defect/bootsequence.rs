use crate::cards::prelude::*;

#[cfg(test)]
#[path = "../../tests/test_card_runtime_defect_wave4.rs"]
mod test_card_runtime_defect_wave4;

#[cfg(test)]
#[path = "../../tests/test_card_runtime_defect_wave7.rs"]
mod test_card_runtime_defect_wave7;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Boot Sequence: 0 cost, 10 block, innate, exhaust.
    insert(cards, CardDef {
        id: "BootSequence", name: "Boot Sequence", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 10,
        base_magic: -1, exhaust: true, enter_stance: None,
        effects: &["innate"], effect_data: &[E::Simple(SE::GainBlock(A::Block))], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "BootSequence+", name: "Boot Sequence+", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 13,
        base_magic: -1, exhaust: true, enter_stance: None,
        effects: &["innate"], effect_data: &[E::Simple(SE::GainBlock(A::Block))], complex_hook: None,
    });
}
