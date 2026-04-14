use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Consume: 2 cost, remove 1 orb slot, gain 2 focus
    insert(cards, CardDef {
                id: "Consume", name: "Consume", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["gain_focus", "lose_orb_slot"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FOCUS, A::Magic)),
                    E::Simple(SE::RemoveOrbSlot),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Consume+", name: "Consume+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["gain_focus", "lose_orb_slot"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FOCUS, A::Magic)),
                    E::Simple(SE::RemoveOrbSlot),
                ], complex_hook: None,
            });
}
