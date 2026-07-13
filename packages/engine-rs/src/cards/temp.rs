use crate::cards::prelude::*;

pub fn register_temp(cards: &mut HashMap<&'static str, CardDef>) {
        // Beta: registered in watcher/beta.rs (with effect_data for AddCard Omega)
        // Insight: registered in watcher/insight.rs (with effect_data for DrawCards)

        // Omega.java installs 50 OmegaPower for 3 energy; upgrade adds 10
        // magic only. OmegaPower deals source-less THORNS damage to all living
        // enemies at the end of every player turn.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Omega.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/OmegaPower.java
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
        // Expunger.setX stores Conjure Blade's X in card-owned state, and use
        // queues exactly that many 9-damage actions. The upgrade adds 6 damage.
        // Java: reference/extracted/methods/card/Expunger.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Expunger.java
        insert(cards, CardDef {
            id: "Expunger", name: "Expunger", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                E::ExtraHits(A::CardMisc),
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Expunger+", name: "Expunger+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                E::ExtraHits(A::CardMisc),
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ], complex_hook: None,
        });
        // Insight: registered in watcher/insight.rs (with effect_data for DrawCards)

        // Safety.java costs 1, gains 12 block, self-retains, and exhausts;
        // upgradeBlock(4) raises only its block to 16.
        // Java: reference/extracted/methods/card/Safety.java
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
        // Shiv.java constructs a zero-cost, 4-damage exhausting Attack;
        // upgradeDamage(2) is its only upgrade change.
        // Java: reference/extracted/methods/card/Shiv.java
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
