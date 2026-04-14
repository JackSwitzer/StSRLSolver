use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Ritual Dagger already has a typed damage primary body.
        // The remaining blocker is kill-context / misc propagation inside the hook.
    insert(cards, CardDef {
                id: "RitualDagger", name: "Ritual Dagger", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["ritual_dagger"], effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
                complex_hook: Some(crate::effects::hooks_complex::hook_ritual_dagger),
            });
    insert(cards, CardDef {
                id: "RitualDagger+", name: "Ritual Dagger+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
                base_magic: 5, exhaust: true, enter_stance: None,
                effects: &["ritual_dagger"], effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
                complex_hook: Some(crate::effects::hooks_complex::hook_ritual_dagger),
            });
}
