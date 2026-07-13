use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // NirvanaPower.onScry queues raw block before ScryAction checks whether the
    // draw pile is empty; the upgrade adds one to the stacked power amount.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Nirvana.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/NirvanaPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/ScryAction.java
    insert(cards, CardDef {
        id: "Nirvana", name: "Nirvana", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::NIRVANA, A::Magic)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Nirvana+", name: "Nirvana+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::NIRVANA, A::Magic)),
        ], complex_hook: None,
    });
}
