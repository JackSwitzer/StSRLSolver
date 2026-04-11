use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Darkness: 1 cost, channel 1 Dark (upgrade: also trigger Dark passive)
    insert(cards, CardDef {
                id: "Darkness", name: "Darkness", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["channel_dark"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Dark, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Darkness+", name: "Darkness+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["channel_dark", "trigger_dark_passive"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Dark, A::Magic)),
                ], complex_hook: Some(crate::effects::hooks_orb::hook_trigger_dark_passive),
            });
}
