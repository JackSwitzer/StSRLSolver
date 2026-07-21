use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/MasterReality.java
    // MakeTempCard* actions/effects upgrade created non-Status/non-Curse cards
    // while this non-stacking power is present; the card upgrade changes cost.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/vfx/cardManip/ShowCardAndAddToDiscardEffect.java
    insert(cards, CardDef {
        id: "MasterReality", name: "Master Reality", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::SetStatus(T::Player, sid::MASTER_REALITY, A::Fixed(1))),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "MasterReality+", name: "Master Reality+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::SetStatus(T::Player, sid::MASTER_REALITY, A::Fixed(1))),
        ], complex_hook: None,
    });
}
