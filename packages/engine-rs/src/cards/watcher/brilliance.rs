use crate::cards::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Brilliance.java
//   ctor: cost 1 ATTACK targeting ENEMY, baseDamage 12, baseMagicNumber 0.
//   applyPowers()/calculateCardDamage(): add total mantra gained this combat.
//   upgrade(): upgradeDamage(4), producing baseDamage 16.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Brilliance ---- (cost 1, 12 dmg + mantra gained this combat; +4 dmg upgrade)
    insert(cards, CardDef {
                id: "Brilliance", name: "Brilliance", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
                base_magic: 0, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Brilliance+", name: "Brilliance+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 16, base_block: -1,
                base_magic: 0, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
}
