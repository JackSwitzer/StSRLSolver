use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/BulletTime.java
    // Cost 3 (2 upgraded); applies No Draw and sets the current hand's
    // non-X costs to zero for this turn.
    insert(cards, CardDef {
                id: "Bullet Time", name: "Bullet Time", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetFlag(BF::BulletTime)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Bullet Time+", name: "Bullet Time+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetFlag(BF::BulletTime)),
                ], complex_hook: None,
            });
}
