use crate::cards::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Beta.java
//   ctor: cost 2 COLORLESS SPECIAL skill targeting NONE; exhausts and previews Omega.
//   use(): adds one stat-equivalent Omega to a random position in the draw pile.
//   upgrade(): upgradeBaseCost(1) only; the generated card and exhaust stay unchanged.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Special Generated Cards ----
        // Beta (from Alpha chain): cost 2, skill, exhaust, add Omega to draw
    insert(cards, CardDef {
                id: "Beta", name: "Beta", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Omega", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Beta+", name: "Beta+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Omega", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
}
