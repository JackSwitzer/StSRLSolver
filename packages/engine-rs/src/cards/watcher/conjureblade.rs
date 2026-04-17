use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Conjure Blade ---- (cost X, skill, exhaust, create Expunger with X hits; upgrade: X+1 hits)
    insert(cards, CardDef {
                id: "ConjureBlade", name: "Conjure Blade", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCardWithMisc("Expunger", P::Draw, A::Fixed(1), A::XCostPlus(0))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "ConjureBlade+", name: "Conjure Blade+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCardWithMisc("Expunger", P::Draw, A::Fixed(1), A::XCostPlus(1))),
                ], complex_hook: None,
            });
}
