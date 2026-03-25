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
    let peakFloor: Int?
    let replayBestFloor: Int?
    let entropyCoeff: Double?
    let clipFraction: Double?
    let gpuPercent: Double?
    // Diagnostic fields from strategic_trainer
    let explainedVariance: Double?
    let meanValue: Double?
    let klDivergence: Double?
    let meanAdvantage: Double?
    let meanReturn: Double?

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
        case peakFloor = "peak_floor"
        case replayBestFloor = "replay_best_floor"
        case entropyCoeff = "entropy_coeff"
        case clipFraction = "clip_fraction"
        case gpuPercent = "gpu_percent"
        case explainedVariance = "explained_variance"
        case meanValue = "mean_value"
        case klDivergence = "kl_divergence"
        case meanAdvantage = "mean_advantage"
        case meanReturn = "mean_return"
    }
}
