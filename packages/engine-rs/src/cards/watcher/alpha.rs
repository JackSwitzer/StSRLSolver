use crate::cards::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Alpha.java
//   ctor: cost 1 SKILL targeting NONE, exhausts, and previews Beta.
//   use(): adds one stat-equivalent Beta to a random position in the draw pile.
//   upgrade(): makes Alpha innate; cost and effect stay unchanged.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Alpha ---- (cost 1, skill, exhaust, shuffle Beta into draw; upgrade: innate)
    insert(cards, CardDef {
                id: "Alpha", name: "Alpha", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Beta", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Alpha+", name: "Alpha+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Beta", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
}
