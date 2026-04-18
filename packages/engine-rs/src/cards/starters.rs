//! Universal starter basics: `Strike` and `Defend`.
//!
//! Java ships 4 class-specific copies (`Strike_R`/`Strike_G`/`Strike_B`/
//! `Strike_P`, same for `Defend`) that are byte-identical except for the
//! `id` string and the class-color suffix. We're Watcher-only for the
//! foreseeable training horizon, so we collapse to a single pair here and
//! use substring-based class affinity (`is_strike(name)`, `is_defend(name)`
//! helpers) where the engine still needs to distinguish "basic starter"
//! from "anything named Strike" (Perfected Strike, Windmill Strike, etc).
//!
//! When we need multi-class training, we'll add a `CharacterClass` enum +
//! `CardDef.class: Option<CharacterClass>` and can re-split if the cost of
//! the single-ID approach becomes visible. Until then: one Strike, one
//! Defend.

use crate::cards::prelude::*;

// Preserve the Defect wave-6 test hook originally attached to strike_b.rs.
// The test file itself is unchanged; the hook just needs to mount somewhere
// so `cargo test` discovers it.
#[cfg(test)]
#[path = "../tests/test_card_runtime_defect_wave6.rs"]
mod test_card_runtime_defect_wave6;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(
        cards,
        CardDef {
            id: "Strike",
            name: "Strike",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 6,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Strike+",
            name: "Strike+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 9,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Defend",
            name: "Defend",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 5,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::Block))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Defend+",
            name: "Defend+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 8,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::Block))],
            complex_hook: None,
        },
    );
}
