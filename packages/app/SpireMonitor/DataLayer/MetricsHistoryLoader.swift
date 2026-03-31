import Foundation

/// A single metrics snapshot from metrics_history.jsonl
struct MetricsSnapshot: Codable, Identifiable {
    let id = UUID()

    let step: Int?
    let totalGames: Int?
    let avgFloor: Double?
    let peakFloor: Int?
    let totalLoss: Double?
    let policyLoss: Double?
    let valueLoss: Double?
    let entropy: Double?
    let explainedVariance: Double?
    let klDivergence: Double?
    let meanValue: Double?
    let meanAdvantage: Double?
    let meanReturn: Double?
    let timestamp: String?

    enum CodingKeys: String, CodingKey {
        case step
        case totalGames = "total_games"
        case avgFloor = "avg_floor"
        case peakFloor = "peak_floor"
        case totalLoss = "total_loss"
        case policyLoss = "policy_loss"
        case valueLoss = "value_loss"
        case entropy
        case explainedVariance = "explained_variance"
        case klDivergence = "kl_divergence"
        case meanValue = "mean_value"
        case meanAdvantage = "mean_advantage"
        case meanReturn = "mean_return"
        case timestamp
    }
}

enum MetricsHistoryLoader {
    /// Load metrics_history.jsonl from the logs directory.
    /// Returns snapshots sorted by step, deduplicated.
    static func load(from logsURL: URL) async -> [MetricsSnapshot] {
        let url = logsURL.appending(path: "metrics_history.jsonl")
        guard let data = try? String(contentsOf: url, encoding: .utf8) else { return [] }

        var snapshots: [MetricsSnapshot] = []
        var seenSteps = Set<Int>()

        for line in data.split(separator: "\n") {
            let trimmed = line.trimmingCharacters(in: .whitespacesAndNewlines)
            guard !trimmed.isEmpty else { continue }
            guard let jsonData = trimmed.data(using: .utf8) else { continue }

            if let snapshot = try? JSONDecoder().decode(MetricsSnapshot.self, from: jsonData) {
                let step = snapshot.step ?? snapshots.count
                if seenSteps.insert(step).inserted {
                    snapshots.append(snapshot)
                }
            }
        }

        return snapshots.sorted { ($0.step ?? 0) < ($1.step ?? 0) }
    }
}
