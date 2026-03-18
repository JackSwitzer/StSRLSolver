import Foundation

struct WorkerStatus: Codable, Identifiable {
    var id: String { name }

    let name: String
    let workerID: Int?
    let seed: String?
    let floor: Int?
    let phase: String?
    let hp: Int?
    let maxHp: Int?
    let enemy: String?
    let ts: Double?

    var hpFraction: Double {
        guard let hp, let maxHp, maxHp > 0 else { return 0 }
        return Double(hp) / Double(maxHp)
    }

    enum CodingKeys: String, CodingKey {
        case name
        case workerID = "id"
        case seed, floor, phase, hp
        case maxHp = "max_hp"
        case enemy, ts
    }
}
