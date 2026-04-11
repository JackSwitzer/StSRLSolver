use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Turbo: 0 cost, gain 2 energy, add Void to discard
    insert(cards, CardDef {
                id: "Turbo", name: "Turbo", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["gain_energy", "add_void_to_discard"], effect_data: &[
                    E::Simple(SE::GainEnergy(A::Magic)),
                    E::Simple(SE::AddCard("Void", P::Discard, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Turbo+", name: "Turbo+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["gain_energy", "add_void_to_discard"], effect_data: &[
                    E::Simple(SE::GainEnergy(A::Magic)),
                    E::Simple(SE::AddCard("Void", P::Discard, A::Fixed(1))),
                ], complex_hook: None,
            });
}
