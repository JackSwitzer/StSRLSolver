use crate::cards::prelude::*;

#[cfg(test)]
#[path = "../../tests/test_card_runtime_silent_wave4.rs"]
mod test_card_runtime_silent_wave4;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Silent Uncommon: Bouncing Flask ---- (cost 2, 3 poison x3 to random; +1 hit)
    insert(cards, CardDef {
        id: "Bouncing Flask", name: "Bouncing Flask", card_type: CardType::Skill,
        target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
        base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::RandomEnemy, sid::POISON, A::Fixed(3))),
            E::Simple(SE::AddStatus(T::RandomEnemy, sid::POISON, A::Fixed(3))),
            E::Simple(SE::AddStatus(T::RandomEnemy, sid::POISON, A::Fixed(3))),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Bouncing Flask+", name: "Bouncing Flask+", card_type: CardType::Skill,
        target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
        base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::RandomEnemy, sid::POISON, A::Fixed(3))),
            E::Simple(SE::AddStatus(T::RandomEnemy, sid::POISON, A::Fixed(3))),
            E::Simple(SE::AddStatus(T::RandomEnemy, sid::POISON, A::Fixed(3))),
            E::Simple(SE::AddStatus(T::RandomEnemy, sid::POISON, A::Fixed(3))),
        ], complex_hook: None,
    });
}
