use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ImpulseAction invokes every orb's start- and end-of-turn callbacks once.
    // Cables repeats both callbacks for the non-empty front orb. Base Exhausts;
    // upgrading removes only Exhaust.
    // Java: reference/extracted/methods/card/Impulse.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ImpulseAction.java
    insert(
        cards,
        CardDef {
            id: "Impulse",
            name: "Impulse",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::TriggerAllOrbPassives)],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Impulse+",
            name: "Impulse+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::TriggerAllOrbPassives)],
            complex_hook: None,
        },
    );
}
