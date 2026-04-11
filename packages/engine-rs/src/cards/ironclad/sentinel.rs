use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Sentinel ---- (cost 1, 5 block, gain 2 energy on exhaust; +3 block, 3 energy)
    insert(cards, CardDef {
                id: "Sentinel", name: "Sentinel", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["energy_on_exhaust"], effect_data: &[], complex_hook: None,
                // Block handled by preamble (base_block). Energy-on-exhaust is a passive trigger.
            });
    insert(cards, CardDef {
                id: "Sentinel+", name: "Sentinel+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["energy_on_exhaust"], effect_data: &[], complex_hook: None,
                // Block handled by preamble (base_block). Energy-on-exhaust is a passive trigger.
            });
}
