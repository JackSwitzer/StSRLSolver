use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Coolheaded: 1 cost, channel Frost, draw 1
    insert(cards, CardDef {
                id: "Coolheaded", name: "Coolheaded", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["channel_frost", "draw"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Frost, A::Fixed(1))),
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Coolheaded+", name: "Coolheaded+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["channel_frost", "draw"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Frost, A::Fixed(1))),
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
}
