use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect, SimpleEffect as SE};

// Source: reference/extracted/methods/card/Reboot.java. ShuffleAllAction moves
// the remaining hand directly to draw, shuffles discard, and fires relic hooks;
// Reboot then explicitly shuffles the combined draw pile and draws magicNumber.
static REBOOT_EFFECTS: [Effect; 1] = [Effect::Simple(SE::ShuffleAllAndDraw(A::Magic))];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Reboot: 0 cost, shuffle every non-played card into draw, draw N, exhaust.
    insert(
        cards,
        CardDef {
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
        },
    );
    insert(
        cards,
        CardDef {
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
        },
    );
}

#[cfg(test)]
#[path = "../../tests/test_card_runtime_defect_wave3.rs"]
mod test_card_runtime_defect_wave3;
