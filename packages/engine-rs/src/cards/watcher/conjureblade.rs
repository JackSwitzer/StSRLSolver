use crate::cards::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ConjureBlade.java
// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/ConjureBladeAction.java
//   X-cost creates one Expunger in the draw pile with X hits (+2 Chemical X).
//   upgrade() adds one hit without changing cost or exhaust.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Rare: Conjure Blade ---- (cost X, skill, exhaust, create Expunger with X hits; upgrade: X+1 hits)
    insert(
        cards,
        CardDef {
            id: "ConjureBlade",
            name: "Conjure Blade",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: -1,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddCardWithMisc(
                "Expunger",
                P::Draw,
                A::Fixed(1),
                A::XCostPlus(0),
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "ConjureBlade+",
            name: "Conjure Blade+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: -1,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddCardWithMisc(
                "Expunger",
                P::Draw,
                A::Fixed(1),
                A::XCostPlus(1),
            ))],
            complex_hook: None,
        },
    );
}
