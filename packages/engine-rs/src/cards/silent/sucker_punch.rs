use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Sucker Punch ---- (cost 1, 7 dmg, 1 weak; +2/+1)
    insert(cards, CardDef {
                id: "Sucker Punch", name: "Sucker Punch", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["weak"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Sucker Punch+", name: "Sucker Punch+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["weak"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
}
