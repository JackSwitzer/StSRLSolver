import Foundation

struct Combat: Codable, Identifiable {
    var id: String { "\(floor)_\(encounterName ?? roomType ?? "unknown")" }

    let floor: Int
    let roomType: String?
    let encounterName: String?
    let hpLost: Int?
    let turns: Int?
    let cardsPlayed: Int?
    let potionsUsed: Int?
    let durationMs: Int?
    let turnsDetail: [TurnDetail]?

    var enemy: String { encounterName ?? roomType ?? "Unknown" }

    var roomCategory: RoomCategory {
        guard let rt = roomType?.lowercased() else { return .monster }
        if rt.contains("boss") { return .boss }
        if rt.contains("elite") { return .elite }
        return .monster
    }

    enum CodingKeys: String, CodingKey {
        case floor
        case roomType = "room_type"
        case encounterName = "encounter_name"
        case hpLost = "hp_lost"
        case turns
        case cardsPlayed = "cards_played"
        case potionsUsed = "potions_used"
        case durationMs = "duration_ms"
        case turnsDetail = "turns_detail"
    }
}

struct TurnDetail: Codable {
    let cardsPlayed: [String]?
    let damage: Int?
    let block: Int?
    let stance: String?

    enum CodingKeys: String, CodingKey {
        case cardsPlayed = "cards_played"
        case damage, block, stance
    }
}

enum RoomCategory: String, CaseIterable {
    case monster = "Monster"
    case elite = "Elite"
    case boss = "Boss"
}
