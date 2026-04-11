use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: Clothesline ---- (cost 2, 12 dmg, 2 weak; +2/+1)
    insert(cards, CardDef {
                id: "Clothesline", name: "Clothesline", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["weak"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Clothesline+", name: "Clothesline+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 14, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["weak"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
}
