use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // NoxiousFumes.java installs 2 Noxious Fumes for one energy; upgrade adds
    // one magic only. NoxiousFumesPower stacks and poisons every living enemy
    // in atStartOfTurnPostDraw.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/NoxiousFumes.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/NoxiousFumesPower.java
    insert(cards, CardDef {
                id: "Noxious Fumes", name: "Noxious Fumes", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::NOXIOUS_FUMES, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Noxious Fumes+", name: "Noxious Fumes+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::NOXIOUS_FUMES, A::Magic)),
                ], complex_hook: None,
            });
}
