use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Conserve Battery: 1 cost, 7 block, next turn gain 1 energy (via Energized)
    insert(cards, CardDef {
                id: "Conserve Battery", name: "Conserve Battery", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["next_turn_energy"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ENERGIZED, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Conserve Battery+", name: "Conserve Battery+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 10,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["next_turn_energy"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ENERGIZED, A::Fixed(1))),
                ], complex_hook: None,
            });
}
