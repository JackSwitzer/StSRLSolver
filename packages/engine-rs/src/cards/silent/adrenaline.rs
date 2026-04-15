use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Silent Rare: Adrenaline ---- (cost 0, gain 1 energy, draw 2, exhaust; upgrade: gain 2 energy)
    insert(cards, CardDef {
        id: "Adrenaline", name: "Adrenaline", card_type: CardType::Skill,
        target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: true, enter_stance: None,
                effect_data: &[
            E::Simple(SE::GainEnergy(A::Magic)),
            E::Simple(SE::DrawCards(A::Fixed(2))),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Adrenaline+", name: "Adrenaline+", card_type: CardType::Skill,
        target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 2, exhaust: true, enter_stance: None,
                effect_data: &[
            E::Simple(SE::GainEnergy(A::Magic)),
            E::Simple(SE::DrawCards(A::Fixed(2))),
        ], complex_hook: None,
    });
}

#[cfg(test)]
#[path = "../../tests/test_card_runtime_silent_wave7.rs"]
mod test_card_runtime_silent_wave7;
