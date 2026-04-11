use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Basic: Neutralize ---- (cost 0, 3 dmg, 1 weak; +1/+1)
    insert(cards, CardDef {
                id: "Neutralize", name: "Neutralize", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["weak"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Neutralize+", name: "Neutralize+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["weak"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
}
