import Foundation

struct Episode: Codable, Identifiable {
    let id = UUID()

    let seed: String
    let won: Bool
    let episode: Int?

    // Dual-format fields (Python writes snake_case, mapped format differs)
    let floor: Int?
    let floorsReached: Int?
    let hp: Int?
    let hpRemaining: Int?
    let maxHp: Int?
    let durationS: Double?
    let duration: Double?

    let combats: [Combat]?
    let deckFinal: [String]?
    let relicsFinal: [String]?
    let deathEnemy: String?
    let deathRoom: String?
    let neowChoice: String?
    let hpHistory: [Int]?
    let totalReward: Double?
    let pbrsReward: Double?
    let eventReward: Double?

    // Computed accessors that handle dual formats
    var effectiveFloor: Int { floorsReached ?? floor ?? 0 }
    var effectiveHp: Int { hpRemaining ?? hp ?? 0 }
    var effectiveMaxHp: Int { maxHp ?? 80 }
    var effectiveDuration: Double { duration ?? durationS ?? 0 }

    enum CodingKeys: String, CodingKey {
        case seed, won, episode, floor, hp, duration, combats
        case floorsReached = "floors_reached"
        case hpRemaining = "hp_remaining"
        case maxHp = "max_hp"
        case durationS = "duration_s"
        case deckFinal = "deck_final"
        case relicsFinal = "relics_final"
        case deathEnemy = "death_enemy"
        case deathRoom = "death_room"
        case neowChoice = "neow_choice"
        case hpHistory = "hp_history"
        case totalReward = "total_reward"
        case pbrsReward = "pbrs_reward"
        case eventReward = "event_reward"
    }
}

struct TopEpisodesFile: Codable {
    let meta: TopEpisodesMeta?
    let episodes: [Episode]
}

struct TopEpisodesMeta: Codable {
    let totalEpisodes: Int?
    let avgFloor: Double?

    enum CodingKeys: String, CodingKey {
        case totalEpisodes = "total_episodes"
        case avgFloor = "avg_floor"
    }
}
