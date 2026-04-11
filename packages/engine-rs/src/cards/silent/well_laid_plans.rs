use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Well-Laid Plans ---- (cost 1, power, retain 1 card/turn; +1)
    insert(cards, CardDef {
                id: "Well-Laid Plans", name: "Well-Laid Plans", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["well_laid_plans"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::WELL_LAID_PLANS, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Well-Laid Plans+", name: "Well-Laid Plans+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["well_laid_plans"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::WELL_LAID_PLANS, A::Magic)),
                ], complex_hook: None,
            });
}
