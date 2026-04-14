#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/HolyWater.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/NinjaScroll.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/MutagenicStrength.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/EmotionChip.java

#[test]
#[ignore = "blocked on combat-start temp-card materialization timing; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/HolyWater.java"]
fn dead_cleanup_wave10_holy_water_stays_queued_until_temp_card_runtime_is_authoritative() {}

#[test]
#[ignore = "blocked on combat-start temp-card materialization timing; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/NinjaScroll.java"]
fn dead_cleanup_wave10_ninja_scroll_stays_queued_until_temp_card_runtime_is_authoritative() {}

#[test]
#[ignore = "blocked on combat-start temporary strength runtime parity; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/MutagenicStrength.java"]
fn dead_cleanup_wave10_mutagenic_strength_stays_queued_until_start_of_combat_runtime_is_authoritative() {}

#[test]
#[ignore = "blocked on the next-turn orb passive timing primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/EmotionChip.java"]
fn dead_cleanup_wave10_emotion_chip_stays_queued_until_timing_primitive_exists() {}
