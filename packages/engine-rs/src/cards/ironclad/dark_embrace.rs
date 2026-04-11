use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Dark Embrace ---- (cost 2, power, draw 1 on exhaust; upgrade: cost 1)
    insert(cards, CardDef {
                id: "Dark Embrace", name: "Dark Embrace", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["dark_embrace"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DARK_EMBRACE, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Dark Embrace+", name: "Dark Embrace+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["dark_embrace"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DARK_EMBRACE, A::Magic)),
                ], complex_hook: None,
            });
}
