use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Blind: 0 cost, apply 2 Weak to enemy (upgrade: target all)
    insert(cards, CardDef {
                id: "Blind", name: "Blind", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Blind+", name: "Blind+", card_type: CardType::Skill,
                target: CardTarget::AllEnemy, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic))], complex_hook: None,
            });
}
