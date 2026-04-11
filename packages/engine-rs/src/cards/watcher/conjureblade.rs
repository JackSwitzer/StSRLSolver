use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Conjure Blade ---- (cost X, skill, exhaust, create Expunger with X hits; upgrade: X+1 hits)
    insert(cards, CardDef {
                id: "ConjureBlade", name: "Conjure Blade", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["conjure_blade"], effect_data: &[], complex_hook: None, // TODO: full X-cost + Expunger creation
            });
    insert(cards, CardDef {
                id: "ConjureBlade+", name: "Conjure Blade+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["conjure_blade"], effect_data: &[], complex_hook: None,
            });
}
