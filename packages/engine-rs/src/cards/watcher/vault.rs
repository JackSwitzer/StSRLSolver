use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Vault.java
    // SkipEnemiesTurnAction precedes PressEndTurnButtonAction; the post-play
    // EndTurn trait supplies the latter after this flag is set.
    insert(cards, CardDef {
                id: "Vault", name: "Vault", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetFlag(BF::SkipEnemyTurn)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Vault+", name: "Vault+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetFlag(BF::SkipEnemyTurn)),
                ], complex_hook: None,
            });
}
