use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Wave of the Hand ---- (cost 1, skill, whenever you gain block this turn apply 1 Weak; +1 magic upgrade)
    insert(cards, CardDef {
                id: "WaveOfTheHand", name: "Wave of the Hand", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::WAVE_OF_THE_HAND, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "WaveOfTheHand+", name: "Wave of the Hand+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::WAVE_OF_THE_HAND, A::Magic)),
                ], complex_hook: None,
            });
}
