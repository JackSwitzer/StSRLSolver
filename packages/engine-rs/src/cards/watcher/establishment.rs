use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Java applies EstablishmentPower(amount 1); at end of turn it reduces
        // only cards whose retain or selfRetain flag is set. Upgrade is innate.
        // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Establishment.java
        // decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/EstablishmentPower.java
        // decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EstablishmentPowerAction.java
        // ---- Rare: Establishment ---- (cost 1, power, retained cards cost 1 less; upgrade: innate)
    insert(cards, CardDef {
                id: "Establishment", name: "Establishment", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ESTABLISHMENT, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Establishment+", name: "Establishment+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ESTABLISHMENT, A::Magic)),
                ], complex_hook: None,
            });
}
