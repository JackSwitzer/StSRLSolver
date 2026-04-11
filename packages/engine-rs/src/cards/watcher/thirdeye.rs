use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Common Watcher Cards (continued) ----
    insert(cards, CardDef {
                id: "ThirdEye", name: "Third Eye", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["scry"], effect_data: &[
                    E::Simple(SE::Scry(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "ThirdEye+", name: "Third Eye+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 9,
                base_magic: 5, exhaust: false, enter_stance: None,
                effects: &["scry"], effect_data: &[
                    E::Simple(SE::Scry(A::Magic)),
                ], complex_hook: None,
            });
}
