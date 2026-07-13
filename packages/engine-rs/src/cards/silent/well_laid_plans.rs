use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // RetainCardPower opens an optional up-to-one end-turn retain selection;
    // the upgrade raises that amount to two. The source ID uses spaces and
    // the card target is NONE.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/WellLaidPlans.java
    insert(cards, CardDef {
                id: "Well Laid Plans", name: "Well Laid Plans", card_type: CardType::Power,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::WELL_LAID_PLANS, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Well Laid Plans+", name: "Well Laid Plans+", card_type: CardType::Power,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::WELL_LAID_PLANS, A::Magic)),
                ], complex_hook: None,
            });
}
