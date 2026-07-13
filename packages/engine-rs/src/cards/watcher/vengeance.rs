use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Java installs separate powers: WrathNextTurnPower is non-stacking and
        // triggers atStartOfTurn, while DrawCardNextTurnPower stacks and draws
        // atStartOfTurnPostDraw. Upgrade adds one draw.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SimmeringFury.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/WrathNextTurnPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DrawCardNextTurnPower.java
    insert(cards, CardDef {
                id: "Vengeance", name: "Simmering Fury", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::WRATH_NEXT_TURN, A::Fixed(1))),
                    E::Simple(SE::AddStatus(T::Player, sid::DRAW_CARD, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Vengeance+", name: "Simmering Fury+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::WRATH_NEXT_TURN, A::Fixed(1))),
                    E::Simple(SE::AddStatus(T::Player, sid::DRAW_CARD, A::Magic)),
                ], complex_hook: None,
            });
}
