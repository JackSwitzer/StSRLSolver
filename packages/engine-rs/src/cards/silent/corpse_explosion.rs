use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Corpse Explosion ---- (cost 2, 6 poison, on death deal dmg = max HP to all; +3 poison)
    insert(cards, CardDef {
                id: "Corpse Explosion", name: "Corpse Explosion", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 6, exhaust: false, enter_stance: None,
                effects: &["corpse_explosion"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::POISON, A::Magic)),
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::CORPSE_EXPLOSION, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Corpse Explosion+", name: "Corpse Explosion+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 9, exhaust: false, enter_stance: None,
                effects: &["corpse_explosion"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::POISON, A::Magic)),
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::CORPSE_EXPLOSION, A::Fixed(1))),
                ], complex_hook: None,
            });
}
