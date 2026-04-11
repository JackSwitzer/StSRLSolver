use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Adrenaline ---- (cost 0, gain 1 energy, draw 2, exhaust; +1 draw)
    insert(cards, CardDef {
                id: "Adrenaline", name: "Adrenaline", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["gain_energy_1", "draw"], effect_data: &[
                    E::Simple(SE::GainEnergy(A::Fixed(1))),
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Adrenaline+", name: "Adrenaline+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["gain_energy_1", "draw"], effect_data: &[
                    E::Simple(SE::GainEnergy(A::Fixed(1))),
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
}
