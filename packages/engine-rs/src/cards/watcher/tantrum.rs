use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Tantrum", name: "Tantrum", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 3, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["multi_hit", "shuffle_self_into_draw"], effect_data: &[
                    E::Simple(SE::ChangeStance(Stance::Wrath)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Tantrum+", name: "Tantrum+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 3, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["multi_hit", "shuffle_self_into_draw"], effect_data: &[
                    E::Simple(SE::ChangeStance(Stance::Wrath)),
                ], complex_hook: None,
            });
}
