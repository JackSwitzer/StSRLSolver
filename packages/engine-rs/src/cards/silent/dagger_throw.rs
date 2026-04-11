use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Dagger Throw ---- (cost 1, 9 dmg, draw 1, discard 1; +3 dmg)
    insert(cards, CardDef {
                id: "Dagger Throw", name: "Dagger Throw", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["draw", "discard"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Dagger Throw+", name: "Dagger Throw+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["draw", "discard"], effect_data: &[], complex_hook: None,
            });
}
