use crate::cards::prelude::*;

pub fn register_temp(cards: &mut HashMap<&'static str, CardDef>) {
        // Beta: registered in watcher/beta.rs (with effect_data for AddCard Omega)
        // Insight: registered in watcher/insight.rs (with effect_data for DrawCards)

        // Omega: 3 cost, power, deal 50 dmg to all enemies at end of each turn
        insert(cards, CardDef {
            id: "Omega", name: "Omega", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 50, exhaust: false, enter_stance: None,
                effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::OMEGA, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Omega+", name: "Omega+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 60, exhaust: false, enter_stance: None,
                effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::OMEGA, A::Magic)),
            ], complex_hook: None,
        });
        // Expunger: 1 cost, 9 dmg x magic (from Conjure Blade)
        insert(cards, CardDef {
            id: "Expunger", name: "Expunger", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
                effect_data: &[
                E::ExtraHits(A::CardMisc),
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Expunger+", name: "Expunger+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
                effect_data: &[
                E::ExtraHits(A::CardMisc),
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ], complex_hook: None,
        });
        // Insight: registered in watcher/insight.rs (with effect_data for DrawCards)

        // Safety: 1 cost, 12 block, retain, exhaust
        insert(cards, CardDef {
            id: "Safety", name: "Safety", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                E::Simple(SE::GainBlock(A::Block)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Safety+", name: "Safety+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 16,
            base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                E::Simple(SE::GainBlock(A::Block)),
            ], complex_hook: None,
        });
        // Through Violence: 0 cost, 20 dmg, retain, exhaust
        insert(cards, CardDef {
            id: "ThroughViolence", name: "Through Violence", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 20, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "ThroughViolence+", name: "Through Violence+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 30, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ], complex_hook: None,
        });
        // Shiv: 0 cost, 4 dmg, exhaust
        insert(cards, CardDef {
            id: "Shiv", name: "Shiv", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Shiv+", name: "Shiv+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ], complex_hook: None,
        });
}
