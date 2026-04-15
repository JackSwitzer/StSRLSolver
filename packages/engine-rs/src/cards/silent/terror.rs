use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Terror ---- (cost 1, 99 vuln, exhaust; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Terror", name: "Terror", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 99, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Terror+", name: "Terror+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 99, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
                ], complex_hook: None,
            });
}
