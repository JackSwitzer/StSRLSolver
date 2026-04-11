use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Empty Mind ---- (cost 1, draw 2, exit stance; +1 draw upgrade)
    insert(cards, CardDef {
                id: "EmptyMind", name: "Empty Mind", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: Some("Neutral"),
                effects: &["draw", "exit_stance"], effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "EmptyMind+", name: "Empty Mind+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: Some("Neutral"),
                effects: &["draw", "exit_stance"], effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
}
