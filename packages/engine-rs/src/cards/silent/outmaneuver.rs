use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Outmaneuver ---- (cost 1, +2 energy next turn; +1 energy)
    insert(cards, CardDef {
                id: "Outmaneuver", name: "Outmaneuver", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ENERGIZED, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Outmaneuver+", name: "Outmaneuver+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ENERGIZED, A::Magic)),
                ], complex_hook: None,
            });
}
