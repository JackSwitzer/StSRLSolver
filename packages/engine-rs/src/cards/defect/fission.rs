use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Fission: 0 cost, remove all orbs, gain energy+draw per orb, exhaust (upgrade: evoke instead of remove)
    insert(cards, CardDef {
                id: "Fission", name: "Fission", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effects: &["fission"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Fission+", name: "Fission+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effects: &["fission_evoke"], effect_data: &[], complex_hook: None,
            });
}
