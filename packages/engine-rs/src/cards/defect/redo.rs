use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Recursion (Java ID: Redo): 1 cost, evoke frontmost, channel it back (upgrade: cost 0)
    insert(cards, CardDef {
                id: "Redo", name: "Recursion", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["evoke_orb", "channel_evoked"], effect_data: &[
                    E::Simple(SE::EvokeAndRechannelFrontOrb),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Redo+", name: "Recursion+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["evoke_orb", "channel_evoked"], effect_data: &[
                    E::Simple(SE::EvokeAndRechannelFrontOrb),
                ], complex_hook: None,
            });
}
