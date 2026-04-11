use super::prelude::*;

/// Fairy in a Bottle: passive death trigger, not manually activated.
/// The existing check_fairy_revive / consume_fairy functions handle this.
/// The EntityDef is a placeholder with no triggers (passive only).
pub static DEF: EntityDef = EntityDef {
    id: "FairyPotion",
    name: "Fairy in a Bottle",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: None,
};
