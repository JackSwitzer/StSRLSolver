use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Static Discharge: 1 cost, power, channel 1 Lightning whenever you take unblocked damage
    insert(cards, CardDef {
                id: "Static Discharge", name: "Static Discharge", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["channel_lightning_on_damage"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Static Discharge+", name: "Static Discharge+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["channel_lightning_on_damage"], effect_data: &[], complex_hook: None,
            });
}
