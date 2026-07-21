use crate::cards::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/BowlingBash.java
//   ctor: cost 1 ATTACK targeting ENEMY with baseDamage 7.
//   use(): queues one damage hit on the selected target per non-dead/non-escaped monster.
//   upgrade(): upgradeDamage(3), producing 10 damage per hit.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Common Watcher Cards ----
    insert(cards, CardDef {
                id: "BowlingBash", name: "Bowling Bash", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::LivingEnemyCount),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "BowlingBash+", name: "Bowling Bash+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::LivingEnemyCount),
                ], complex_hook: None,
            });
}
