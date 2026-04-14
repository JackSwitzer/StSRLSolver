#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeusExMachina.java

#[test]
#[ignore = "Deus Ex Machina remains empty effect_data on purpose: its behavior is a draw-trigger hook, and moving it into declarative card effect_data would misrepresent the card without adding a real on-draw card-definition primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeusExMachina.java"]
fn watcher_wave23_deus_ex_machina_remains_empty_effect_data_blocker() {
    assert!(crate::cards::global_registry()
        .get("DeusExMachina")
        .expect("Deus Ex Machina should be registered")
        .effect_data
        .is_empty());
}
