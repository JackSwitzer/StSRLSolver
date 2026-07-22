use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // SadisticNature.java applies magic 5 (7 upgraded) SadisticPower for zero
    // energy. SadisticPower reacts to player-sourced, non-Artifact-blocked enemy
    // debuffs except Shackled, dealing DamageInfo.THORNS.
    // Java: reference/extracted/methods/card/SadisticNature.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/SadisticPower.java
    insert(
        cards,
        CardDef {
            id: "Sadistic Nature",
            name: "Sadistic Nature",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 5,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::SADISTIC, A::Magic))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Sadistic Nature+",
            name: "Sadistic Nature+",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 7,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::SADISTIC, A::Magic))],
            complex_hook: None,
        },
    );
}
