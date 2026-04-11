use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Uppercut ---- (cost 2, 13 dmg, 1 weak + 1 vuln; +1/+1)
    insert(cards, CardDef {
                id: "Uppercut", name: "Uppercut", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["weak", "vulnerable"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Uppercut+", name: "Uppercut+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["weak", "vulnerable"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
                ], complex_hook: None,
            });
}
