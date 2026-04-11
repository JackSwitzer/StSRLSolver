use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Seeing Red ---- (cost 1, gain 2 energy, exhaust; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Seeing Red", name: "Seeing Red", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["gain_energy"], effect_data: &[
                    E::Simple(SE::GainEnergy(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Seeing Red+", name: "Seeing Red+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["gain_energy"], effect_data: &[
                    E::Simple(SE::GainEnergy(A::Magic)),
                ], complex_hook: None,
            });
}
