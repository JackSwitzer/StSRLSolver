use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: cards/red/Bash.java costs 2 and queues 8 damage before
    // applying 2 Vulnerable; the upgrade adds 2 damage and 1 Vulnerable.
    insert(
        cards,
        CardDef {
            id: "Bash",
            name: "Bash",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 2,
            base_damage: 8,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::SelectedEnemy,
                sid::VULNERABLE,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Bash+",
            name: "Bash+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 2,
            base_damage: 10,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::SelectedEnemy,
                sid::VULNERABLE,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
