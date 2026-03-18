import Foundation

@MainActor @Observable
final class DataStore {
    var status: TrainingStatus?
    var recentEpisodes: [Episode] = []
    var topEpisodes: [Episode] = []
    var floorCurve: [Double] = []
    var workers: [WorkerStatus] = []
    var lastStatusUpdate: Date?

    // Loss history (accumulated from polling, max 500 points)
    var lossHistory: [LossPoint] = []
    // System stats (accumulated, max 720 = 30 min at 2.5s)
    var systemStats: SystemStats?
    var systemHistory: [SystemStats] = []

    var isLive: Bool {
        guard let last = lastStatusUpdate else { return false }
        return Date().timeIntervalSince(last) < 10
    }

    // MARK: - Computed stats

    var topRunsSorted: [Episode] {
        let source = recentEpisodes.isEmpty ? topEpisodes : recentEpisodes
        return source.sorted { $0.effectiveFloor > $1.effectiveFloor }
    }

    var deathStats: [(enemy: String, count: Int)] {
        let source = recentEpisodes.isEmpty ? topEpisodes : recentEpisodes
        var counts: [String: Int] = [:]
        for ep in source where !ep.won {
            let enemy = ep.deathEnemy ?? "Unknown"
            counts[enemy, default: 0] += 1
        }
        return counts.sorted { $0.value > $1.value }.map { (enemy: $0.key, count: $0.value) }
    }

    var performanceByRoom: [RoomCategory: RoomPerformance] {
        let source = recentEpisodes.isEmpty ? topEpisodes : recentEpisodes
        var grouped: [RoomCategory: [Combat]] = [:]
        for ep in source {
            for combat in ep.combats ?? [] {
                grouped[combat.roomCategory, default: []].append(combat)
            }
        }
        var result: [RoomCategory: RoomPerformance] = [:]
        for (cat, combats) in grouped {
            let avgTurns = combats.compactMap(\.turns).average
            let avgHpLost = combats.compactMap(\.hpLost).average
            let potionRate = combats.count > 0
                ? Double(combats.filter { ($0.potionsUsed ?? 0) > 0 }.count) / Double(combats.count)
                : 0
            result[cat] = RoomPerformance(
                count: combats.count,
                avgTurns: avgTurns,
                avgHpLost: avgHpLost,
                potionRate: potionRate
            )
        }
        return result
    }

    var peakFloor: Int {
        status?.peakFloor ?? status?.replayBestFloor ?? topRunsSorted.first?.effectiveFloor ?? 0
    }

    func appendLoss(from status: TrainingStatus) {
        guard let total = status.totalLoss else { return }
        let point = LossPoint(
            step: status.trainSteps ?? lossHistory.count,
            total: total,
            policy: status.policyLoss ?? 0,
            value: status.valueLoss ?? 0
        )
        // Append if value changed or enough time passed (new poll)
        if lossHistory.last?.total != point.total || lossHistory.last?.step != point.step {
            lossHistory.append(point)
            if lossHistory.count > 500 { lossHistory.removeFirst() }
        }
    }
}

struct LossPoint: Identifiable {
    let id = UUID()
    let step: Int
    let total: Double
    let policy: Double
    let value: Double
}

struct SystemStats: Identifiable {
    let id = UUID()
    let timestamp: Date
    let cpuPercent: Double
    let memoryUsedGB: Double
    let memoryTotalGB: Double
    let gpuPercent: Double?
}

struct RoomPerformance {
    let count: Int
    let avgTurns: Double
    let avgHpLost: Double
    let potionRate: Double
}

private extension Array where Element == Int {
    var average: Double {
        isEmpty ? 0 : Double(reduce(0, +)) / Double(count)
    }
}
