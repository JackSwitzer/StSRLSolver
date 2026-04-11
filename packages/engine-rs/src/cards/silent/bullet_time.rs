use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Bullet Time ---- (cost 3, cards cost 0 this turn, no more draw; upgrade: cost 2)
    insert(cards, CardDef {
                id: "Bullet Time", name: "Bullet Time", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["bullet_time"], effect_data: &[
                    E::Simple(SE::SetFlag(BF::BulletTime)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Bullet Time+", name: "Bullet Time+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["bullet_time"], effect_data: &[
                    E::Simple(SE::SetFlag(BF::BulletTime)),
                ], complex_hook: None,
            });
}
