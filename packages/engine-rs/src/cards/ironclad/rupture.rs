use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Rupture.java applies RupturePower for magic 1 (2 upgraded) at cost 1.
    // RupturePower.wasHPLost grants that much Strength only when positive HP
    // is lost from DamageInfo owned by the player.
    // Java: reference/extracted/methods/card/Rupture.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/RupturePower.java
    insert(
        cards,
        CardDef {
            id: "Rupture",
            name: "Rupture",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::RUPTURE, A::Magic))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Rupture+",
            name: "Rupture+",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::RUPTURE, A::Magic))],
            complex_hook: None,
        },
    );
}
