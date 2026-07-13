use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Sucker Punch ---- (cost 1, 7 dmg, 1 weak; +2/+1)
    // DamageAction is queued before ApplyPowerAction, so a combat-ending hit
    // clears the later Weak application.
    // Java: reference/extracted/methods/card/SuckerPunch.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAction.java
    insert(cards, CardDef {
                id: "Sucker Punch", name: "Sucker Punch", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Sucker Punch+", name: "Sucker Punch+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
}
