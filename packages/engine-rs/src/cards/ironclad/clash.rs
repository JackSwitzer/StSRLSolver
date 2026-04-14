use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: Clash ---- (cost 0, 14 dmg, only if hand is all attacks; +4 dmg)
    insert(cards, CardDef {
                id: "Clash", name: "Clash", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 14, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["only_attacks_in_hand"], effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Clash+", name: "Clash+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 18, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["only_attacks_in_hand"], effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
            });
}
