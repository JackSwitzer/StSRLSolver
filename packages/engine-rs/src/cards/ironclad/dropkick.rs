use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Dropkick ---- (cost 1, 5 dmg, if vuln: +1 energy + draw 1; +3 dmg)
    insert(cards, CardDef {
                id: "Dropkick", name: "Dropkick", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(
                        crate::effects::declarative::Condition::EnemyHasStatus(sid::VULNERABLE),
                        &[E::Simple(SE::GainEnergy(A::Fixed(1))), E::Simple(SE::DrawCards(A::Fixed(1)))],
                        &[],
                    ),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Dropkick+", name: "Dropkick+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(
                        crate::effects::declarative::Condition::EnemyHasStatus(sid::VULNERABLE),
                        &[E::Simple(SE::GainEnergy(A::Fixed(1))), E::Simple(SE::DrawCards(A::Fixed(1)))],
                        &[],
                    ),
                ], complex_hook: None,
            });
}
