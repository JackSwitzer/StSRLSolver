use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Silent Uncommon: Calculated Gamble ---- (cost 0, discard hand draw that many, exhaust; upgrade: no exhaust)
    insert(cards, CardDef {
        id: "Calculated Gamble", name: "Calculated Gamble", card_type: CardType::Skill,
        target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
        effects: &["calculated_gamble"],
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
        effects: &["calculated_gamble"],
        effect_data: &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::All,
                action: BulkAction::Discard,
            },
            E::Simple(SE::DrawCards(A::HandSizeAtPlayPlus(1))),
        ],
        complex_hook: None,
    });
}
