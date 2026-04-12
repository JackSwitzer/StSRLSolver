use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Steam Barrier (SteamBarrier): 0 cost, 6 block, loses 1 block each play
    insert(cards, CardDef {
                id: "Steam", name: "Steam Barrier", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 6,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["lose_block_each_play"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::STEAM_BARRIER_LOSS, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Steam+", name: "Steam Barrier+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 8,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["lose_block_each_play"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::STEAM_BARRIER_LOSS, A::Fixed(1))),
                ], complex_hook: None,
            });
}
