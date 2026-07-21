use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/CalculatedGamble.java
    // and actions/unique/CalculatedGambleAction.java. Both versions pass `false`
    // to the action, so the upgrade only removes Exhaust; it does not draw extra.
    insert(cards, CardDef {
        id: "Calculated Gamble", name: "Calculated Gamble", card_type: CardType::Skill,
        target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::All,
                action: BulkAction::Discard,
            },
            E::Simple(SE::DrawCards(A::HandSizeAtPlay)),
        ],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Calculated Gamble+", name: "Calculated Gamble+", card_type: CardType::Skill,
        target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::All,
                action: BulkAction::Discard,
            },
            E::Simple(SE::DrawCards(A::HandSizeAtPlay)),
        ],
        complex_hook: None,
    });
}
