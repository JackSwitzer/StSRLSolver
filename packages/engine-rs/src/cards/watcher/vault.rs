use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Vault ---- (cost 3, skill, exhaust, skip enemy turn, end turn; upgrade: cost 2)
    insert(cards, CardDef {
                id: "Vault", name: "Vault", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["skip_enemy_turn", "end_turn"], effect_data: &[
                    E::Simple(SE::SetFlag(BF::SkipEnemyTurn)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Vault+", name: "Vault+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["skip_enemy_turn", "end_turn"], effect_data: &[
                    E::Simple(SE::SetFlag(BF::SkipEnemyTurn)),
                ], complex_hook: None,
            });
}
