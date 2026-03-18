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

    var peakFloor: Int {
        let topFloor = (recentEpisodes + topEpisodes)
            .max(by: { $0.effectiveFloor < $1.effectiveFloor })?.effectiveFloor ?? 0
        return status?.peakFloor ?? status?.replayBestFloor ?? topFloor
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
