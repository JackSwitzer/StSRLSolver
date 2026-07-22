use crate::cards::prelude::*;

fn apply_energy_down(
    engine: &mut crate::engine::CombatEngine,
    _ctx: &crate::effects::types::CardPlayContext,
) {
    crate::powers::apply_debuff(&mut engine.state.player, sid::ENERGY_DOWN, 1);
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java applies Strength, Dexterity, then the debuff EnergyDownPower(1).
    // Energy Down is Artifact-blockable and subtracts energy on turn start;
    // it does not permanently modify the character's max energy.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Fasting.java
    // decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/EnergyDownPower.java
    // ---- Rare (listed): Fasting ---- (Java: Uncommon, cost 2, power, +3 str/dex, -1 energy; +1 magic upgrade)
    insert(
        cards,
        CardDef {
            id: "Fasting2",
            name: "Fasting",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                E::Simple(SE::AddStatus(T::Player, sid::DEXTERITY, A::Magic)),
            ],
            complex_hook: Some(apply_energy_down),
        },
    );
    insert(
        cards,
        CardDef {
            id: "Fasting2+",
            name: "Fasting+",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: -1,
            base_magic: 4,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                E::Simple(SE::AddStatus(T::Player, sid::DEXTERITY, A::Magic)),
            ],
            complex_hook: Some(apply_energy_down),
        },
    );
}
