use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
        id: "Eruption", name: "Eruption", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 2, base_damage: 9, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::ChangeStance(Stance::Wrath)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Eruption+", name: "Eruption+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::ChangeStance(Stance::Wrath)),
        ], complex_hook: None,
    });
}

#[cfg(test)]
#[path = "../../tests/test_card_runtime_watcher_wave4.rs"]
mod test_card_runtime_watcher_wave4;
