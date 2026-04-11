use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Defect Uncommon Cards ----
        // Aggregate: 1 cost, gain 1 energy per 4 cards in draw pile (upgrade: per 3)
    insert(cards, CardDef {
                id: "Aggregate", name: "Aggregate", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["energy_per_cards_in_draw"], effect_data: &[
                    E::Simple(SE::GainEnergy(A::DrawPileDivN(4))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Aggregate+", name: "Aggregate+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["energy_per_cards_in_draw"], effect_data: &[
                    E::Simple(SE::GainEnergy(A::DrawPileDivN(3))),
                ], complex_hook: None,
            });
}
