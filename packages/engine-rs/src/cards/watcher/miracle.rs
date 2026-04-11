use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Special Cards ----
    insert(cards, CardDef {
                id: "Miracle", name: "Miracle", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effects: &["gain_energy"], effect_data: &[
                    E::Simple(SE::GainEnergy(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Miracle+", name: "Miracle+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["gain_energy"], effect_data: &[
                    E::Simple(SE::GainEnergy(A::Magic)),
                ], complex_hook: None,
            });
}
