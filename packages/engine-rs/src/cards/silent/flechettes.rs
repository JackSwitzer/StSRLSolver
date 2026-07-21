use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // FlechetteAction iterates the remaining hand and queues exactly one
        // hit per Skill, including zero hits when none remain. Upgrading adds
        // 2 damage to every hit.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/FlechetteAction.java
    insert(cards, CardDef {
                id: "Flechettes", name: "Flechettes", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 4, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::SkillsInHand),
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ],
                complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Flechettes+", name: "Flechettes+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::SkillsInHand),
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ],
                complex_hook: None,
            });
}
