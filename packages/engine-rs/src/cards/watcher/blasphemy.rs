use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Blasphemy ---- (cost 1, skill, exhaust, enter Divinity, die next turn; upgrade: retain)
    insert(cards, CardDef {
                id: "Blasphemy", name: "Blasphemy", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: Some("Divinity"),
                effects: &["die_next_turn"], effect_data: &[
                    E::Simple(SE::SetFlag(BF::Blasphemy)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Blasphemy+", name: "Blasphemy+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: Some("Divinity"),
                effects: &["die_next_turn", "retain"], effect_data: &[
                    E::Simple(SE::SetFlag(BF::Blasphemy)),
                ], complex_hook: None,
            });
}
