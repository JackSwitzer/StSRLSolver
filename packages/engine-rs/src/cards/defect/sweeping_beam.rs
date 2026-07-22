use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Sweeping Beam: 1 cost, 6 dmg AoE, draw 1
    // DamageAllEnemiesAction resolves before DrawCardAction; a full-board kill
    // clears that queued draw. The upgrade changes only damage from 6 to 9.
    // Java: reference/extracted/methods/card/SweepingBeam.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAllEnemiesAction.java
    insert(
        cards,
        CardDef {
            id: "Sweeping Beam",
            name: "Sweeping Beam",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 1,
            base_damage: 6,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawCards(A::Magic))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Sweeping Beam+",
            name: "Sweeping Beam+",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 1,
            base_damage: 9,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawCards(A::Magic))],
            complex_hook: None,
        },
    );
}
