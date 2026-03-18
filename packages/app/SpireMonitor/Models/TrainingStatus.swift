import Foundation

struct TrainingStatus: Codable {
    let timestamp: String?
    let elapsedHours: Double?
    let totalGames: Int?
    let totalWins: Int?
    let winRate100: Double?
    let avgFloor100: Double?
    let gamesPerMin: Double?
    let trainSteps: Int?
    let totalLoss: Double?
    let policyLoss: Double?
    let valueLoss: Double?
    let entropy: Double?
    let sweepPhase: String?
    let configName: String?
    let bufferSize: Int?

    enum CodingKeys: String, CodingKey {
        case timestamp
        case elapsedHours = "elapsed_hours"
        case totalGames = "total_games"
        case totalWins = "total_wins"
        case winRate100 = "win_rate_100"
        case avgFloor100 = "avg_floor_100"
        case gamesPerMin = "games_per_min"
        case trainSteps = "train_steps"
        case totalLoss = "total_loss"
        case policyLoss = "policy_loss"
        case valueLoss = "value_loss"
        case entropy
        case sweepPhase = "sweep_phase"
        case configName = "config_name"
        case bufferSize = "buffer_size"
    }
}
