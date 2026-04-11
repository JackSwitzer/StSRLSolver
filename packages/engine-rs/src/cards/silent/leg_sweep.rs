use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Leg Sweep ---- (cost 2, 2 weak, 11 block; +1/+3)
    insert(cards, CardDef {
                id: "Leg Sweep", name: "Leg Sweep", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 2, base_damage: -1, base_block: 11,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["weak"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Leg Sweep+", name: "Leg Sweep+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 2, base_damage: -1, base_block: 14,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["weak"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
}
