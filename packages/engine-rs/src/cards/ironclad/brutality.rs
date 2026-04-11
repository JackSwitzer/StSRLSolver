use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Brutality ---- (cost 0, power, lose 1 HP + draw 1 at turn start; upgrade: innate)
    insert(cards, CardDef {
                id: "Brutality", name: "Brutality", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["brutality"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::BRUTALITY, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Brutality+", name: "Brutality+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["brutality", "innate"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::BRUTALITY, A::Magic)),
                ], complex_hook: None,
            });
}
