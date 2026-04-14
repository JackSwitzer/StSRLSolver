use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Common: Crescendo ---- (cost 1, enter Wrath, exhaust, retain; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Crescendo", name: "Crescendo", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["retain"], effect_data: &[
                    E::Simple(SE::ChangeStance(Stance::Wrath)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Crescendo+", name: "Crescendo+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["retain"], effect_data: &[
                    E::Simple(SE::ChangeStance(Stance::Wrath)),
                ], complex_hook: None,
            });
}
