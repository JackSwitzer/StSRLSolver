use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: Sword Boomerang ---- (cost 1, 3 dmg x3 random; +1 magic)
    insert(cards, CardDef {
                id: "Sword Boomerang", name: "Sword Boomerang", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 3, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["damage_random_x_times"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Sword Boomerang+", name: "Sword Boomerang+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 3, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["damage_random_x_times"], effect_data: &[], complex_hook: None,
            });
}
