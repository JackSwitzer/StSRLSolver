use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Common: Sash Whip ---- (cost 1, 8 dmg, weak 1 if last attack; +2 dmg +1 magic upgrade)
    insert(cards, CardDef {
                id: "SashWhip", name: "Sash Whip", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["weak_if_last_attack"], effect_data: &[
                    E::Conditional(Cond::LastCardType(CardType::Attack), &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic))], &[]),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "SashWhip+", name: "Sash Whip+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["weak_if_last_attack"], effect_data: &[
                    E::Conditional(Cond::LastCardType(CardType::Attack), &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic))], &[]),
                ], complex_hook: None,
            });
}
