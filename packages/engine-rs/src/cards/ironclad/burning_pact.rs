use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Ironclad Uncommon: Burning Pact ----
    // Typed choice body is in place; the deferred draw after exhaustion still
    // stays hook-backed until the shared post-choice sequencing primitive lands.
    insert(cards, CardDef {
        id: "Burning Pact",
        name: "Burning Pact",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 1,
        base_damage: -1,
        base_block: -1,
        base_magic: 2,
        exhaust: false,
        enter_stance: None,
        effects: &["exhaust_choose", "draw"],
        effect_data: &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::Exhaust,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
        }],
        complex_hook: Some(crate::effects::hooks_complex::hook_burning_pact),
    });
    insert(cards, CardDef {
        id: "Burning Pact+",
        name: "Burning Pact+",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 1,
        base_damage: -1,
        base_block: -1,
        base_magic: 3,
        exhaust: false,
        enter_stance: None,
        effects: &["exhaust_choose", "draw"],
        effect_data: &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::Exhaust,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
        }],
        complex_hook: Some(crate::effects::hooks_complex::hook_burning_pact),
    });
}
