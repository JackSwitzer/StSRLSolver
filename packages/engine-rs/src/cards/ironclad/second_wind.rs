use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Ironclad Uncommon: Second Wind ----
    // Fully typed: exhaust all non-attacks, then gain block equal to the
    // number of exhausted cards times the card's block value. The shared
    // count-return primitive is carried by `AmountSource::LastBulkCount`.
    insert(cards, CardDef {
        id: "Second Wind",
        name: "Second Wind",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: 1,
        base_damage: -1,
        base_block: 5,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
        effects: &["second_wind"],
        effect_data: &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::NonAttacks,
                action: crate::effects::declarative::BulkAction::Exhaust,
            },
            E::Simple(SE::GainBlock(A::LastBulkCountTimesBlock)),
        ],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Second Wind+",
        name: "Second Wind+",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: 1,
        base_damage: -1,
        base_block: 7,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
        effects: &["second_wind"],
        effect_data: &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::NonAttacks,
                action: crate::effects::declarative::BulkAction::Exhaust,
            },
            E::Simple(SE::GainBlock(A::LastBulkCountTimesBlock)),
        ],
        complex_hook: None,
    });
}
