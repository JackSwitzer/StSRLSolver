//! Relic flags -- bitfield for O(1) relic checks in hot paths.
//!
//! Avoids Vec<String> scanning for relics that just need boolean or counter checks.

/// Bitfield flags for boolean relics. Checked via `flags & FLAG != 0`.
pub mod flag {
    pub const ECTOPLASM: u64        = 1 << 0;   // No gold gain
    pub const GOLDEN_IDOL: u64      = 1 << 1;   // +25% gold from combats
    pub const COFFEE_DRIPPER: u64   = 1 << 2;   // Can't rest at campfire
    pub const FUSION_HAMMER: u64    = 1 << 3;   // Can't upgrade at campfire
    pub const SOZU: u64             = 1 << 4;   // Can't gain potions
    pub const MEMBERSHIP_CARD: u64  = 1 << 5;   // 50% shop discount
    pub const SACRED_BARK: u64      = 1 << 6;   // Double potion effects
    pub const CURSED_KEY: u64       = 1 << 7;   // Gain curse on chest open
    pub const BLACK_STAR: u64       = 1 << 8;   // Double elite relic rewards
    pub const PRISMATIC_SHARD: u64  = 1 << 9;   // Can see all color cards
    pub const REGAL_PILLOW: u64     = 1 << 10;  // +15 campfire heal
    pub const ICE_CREAM: u64        = 1 << 11;  // Preserve energy between turns
    pub const TOY_ORNITHOPTER: u64  = 1 << 12;  // Heal 5 on potion use
    pub const OMAMORI: u64          = 1 << 13;  // Negate 2 curses
    pub const SMILING_MASK: u64     = 1 << 14;  // Card removal costs 50g
    pub const SINGING_BOWL: u64     = 1 << 15;  // +2 max HP option at card reward
    pub const QUESTION_CARD: u64    = 1 << 16;  // +1 card choice at reward
    pub const PRAYER_WHEEL: u64     = 1 << 17;  // +1 card reward after combat
    pub const MAW_BANK: u64         = 1 << 18;  // +12g per non-shop floor
    pub const OLD_COIN: u64         = 1 << 19;  // +300g on pickup (already applied)
    pub const CERAMIC_FISH: u64     = 1 << 20;  // +9g on card add
    pub const MEAL_TICKET: u64      = 1 << 21;  // Heal 15 at shop
    pub const DREAM_CATCHER: u64    = 1 << 22;  // Card reward at rest
    pub const JUZU_BRACELET: u64    = 1 << 23;  // No ? room monsters
    pub const SSSERPENT_HEAD: u64   = 1 << 24;  // +50g on event card add
    pub const THE_COURIER: u64      = 1 << 25;  // Shop has card removal + discount
    pub const MATRYOSHKA: u64       = 1 << 26;  // 2 free relics from first 2 chests
    pub const MARK_OF_BLOOM: u64    = 1 << 27;  // No healing
    pub const MAGIC_FLOWER: u64     = 1 << 28;  // 1.5x healing
    pub const WHITE_BEAST: u64      = 1 << 29;  // Heal on potion use (ToyOrnithopter alias)
    pub const TINY_CHEST: u64       = 1 << 30;  // Every 4th ? room has treasure
}

/// Counter indices for cross-combat persistent counters.
pub mod counter {
    pub const NUNCHAKU: usize       = 0;  // 10 attacks -> +1 energy
    pub const INCENSE_BURNER: usize = 1;  // 6 turns -> intangible
    pub const INK_BOTTLE: usize     = 2;  // 10 cards -> +1 draw
    pub const HAPPY_FLOWER: usize   = 3;  // 3 turns -> +1 energy
    pub const MAW_BANK_GOLD: usize  = 4;  // Accumulated Maw Bank gold
    pub const OMAMORI_USES: usize   = 5;  // Remaining curse negations (starts at 2)
    pub const MATRYOSHKA_USES: usize = 6; // Remaining free chest relics
    pub const NUM_COUNTERS: usize   = 8;
}

/// Relic flags for a run. Populated from Vec<String> relics on add/remove.
#[derive(Debug, Clone, Default)]
pub struct RelicFlags {
    pub flags: u64,
    pub counters: [i16; counter::NUM_COUNTERS],
}

impl RelicFlags {
    pub fn has(&self, flag: u64) -> bool {
        self.flags & flag != 0
    }

    pub fn set(&mut self, flag: u64) {
        self.flags |= flag;
    }

    pub fn clear(&mut self, flag: u64) {
        self.flags &= !flag;
    }

    /// Rebuild flags from a relic name list. Call after any relic add/remove.
    pub fn rebuild(&mut self, relics: &[String]) {
        self.flags = 0;
        for name in relics {
            let f = match name.as_str() {
                "Ectoplasm" => flag::ECTOPLASM,
                "GoldenIdol" | "Golden Idol" => flag::GOLDEN_IDOL,
                "CoffeeDripper" | "Coffee Dripper" => flag::COFFEE_DRIPPER,
                "FusionHammer" | "Fusion Hammer" => flag::FUSION_HAMMER,
                "Sozu" => flag::SOZU,
                "MembershipCard" | "Membership Card" => flag::MEMBERSHIP_CARD,
                "SacredBark" | "Sacred Bark" => flag::SACRED_BARK,
                "CursedKey" | "Cursed Key" => flag::CURSED_KEY,
                "BlackStar" | "Black Star" => flag::BLACK_STAR,
                "PrismaticShard" | "Prismatic Shard" => flag::PRISMATIC_SHARD,
                "RegalPillow" | "Regal Pillow" => flag::REGAL_PILLOW,
                "IceCream" | "Ice Cream" => flag::ICE_CREAM,
                "ToyOrnithopter" | "Toy Ornithopter" => flag::TOY_ORNITHOPTER,
                "Omamori" => flag::OMAMORI,
                "SmilingMask" | "Smiling Mask" => flag::SMILING_MASK,
                "SingingBowl" | "Singing Bowl" => flag::SINGING_BOWL,
                "QuestionCard" | "Question Card" => flag::QUESTION_CARD,
                "PrayerWheel" | "Prayer Wheel" => flag::PRAYER_WHEEL,
                "MawBank" | "Maw Bank" => flag::MAW_BANK,
                "OldCoin" | "Old Coin" => flag::OLD_COIN,
                "CeramicFish" | "Ceramic Fish" => flag::CERAMIC_FISH,
                "MealTicket" | "Meal Ticket" => flag::MEAL_TICKET,
                "DreamCatcher" | "Dream Catcher" => flag::DREAM_CATCHER,
                "JuzuBracelet" | "Juzu Bracelet" => flag::JUZU_BRACELET,
                "SsserpentHead" | "Ssserpent Head" => flag::SSSERPENT_HEAD,
                "TheCourier" | "The Courier" => flag::THE_COURIER,
                "Matryoshka" => flag::MATRYOSHKA,
                "MarkOfTheBloom" | "Mark of the Bloom" => flag::MARK_OF_BLOOM,
                "MagicFlower" | "Magic Flower" => flag::MAGIC_FLOWER,
                "TinyChest" | "Tiny Chest" => flag::TINY_CHEST,
                _ => 0,
            };
            self.flags |= f;
        }
    }

    /// Initialize counters when a new relic is added.
    pub fn init_relic_counter(&mut self, name: &str) {
        match name {
            "Omamori" => self.counters[counter::OMAMORI_USES] = 2,
            "Matryoshka" => self.counters[counter::MATRYOSHKA_USES] = 2,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_set_check() {
        let mut rf = RelicFlags::default();
        assert!(!rf.has(flag::ECTOPLASM));
        rf.set(flag::ECTOPLASM);
        assert!(rf.has(flag::ECTOPLASM));
        assert!(!rf.has(flag::GOLDEN_IDOL));
    }

    #[test]
    fn test_rebuild_from_relics() {
        let relics = vec![
            "Ectoplasm".to_string(),
            "GoldenIdol".to_string(),
            "PureWater".to_string(), // Not a flag relic
        ];
        let mut rf = RelicFlags::default();
        rf.rebuild(&relics);
        assert!(rf.has(flag::ECTOPLASM));
        assert!(rf.has(flag::GOLDEN_IDOL));
        assert!(!rf.has(flag::SOZU));
    }

    #[test]
    fn test_counters_default_zero() {
        let rf = RelicFlags::default();
        assert_eq!(rf.counters[counter::NUNCHAKU], 0);
        assert_eq!(rf.counters[counter::INCENSE_BURNER], 0);
    }

    #[test]
    fn test_omamori_init() {
        let mut rf = RelicFlags::default();
        rf.init_relic_counter("Omamori");
        assert_eq!(rf.counters[counter::OMAMORI_USES], 2);
    }
}
