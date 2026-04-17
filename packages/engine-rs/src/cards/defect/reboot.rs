use crate::cards::prelude::*;
use crate::effects::declarative::{BulkAction, CardFilter, Effect, Pile as P, SimpleEffect as SE};

static REBOOT_EFFECTS: [Effect; 3] = [
    Effect::ForEachInPile {
        pile: P::Hand,
        filter: CardFilter::All,
        action: BulkAction::Discard,
    },
    Effect::Simple(SE::ShuffleDiscardIntoDraw),
    Effect::Simple(SE::DrawCards(crate::effects::declarative::AmountSource::Magic)),
];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Reboot: 0 cost, move remaining hand to discard, shuffle discard into draw, draw N, exhaust.
    insert(cards, CardDef {
        id: "Reboot",
        name: "Reboot",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: 0,
        base_damage: -1,
        base_block: -1,
        base_magic: 4,
        exhaust: true,
        enter_stance: None,
                effect_data: &REBOOT_EFFECTS,
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Reboot+",
        name: "Reboot+",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: 0,
        base_damage: -1,
        base_block: -1,
        base_magic: 6,
        exhaust: true,
        enter_stance: None,
                effect_data: &REBOOT_EFFECTS,
        complex_hook: None,
    });
}

#[cfg(test)]
#[path = "../../tests/test_card_runtime_defect_wave3.rs"]
mod test_card_runtime_defect_wave3;
