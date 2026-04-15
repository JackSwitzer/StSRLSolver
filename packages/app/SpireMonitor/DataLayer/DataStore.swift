import Foundation

@MainActor @Observable
final class DataStore {
    var status: TrainingStatus?
    var recentEpisodes: [Episode] = []
    var topEpisodes: [Episode] = []
    var runManifest: TrainingRunArtifactManifest?
    var frontierReport: FrontierReportArtifact?
    var benchmarkReports: [LocatedBenchmarkReport] = []
    var artifactEpisodes: [LocatedEpisodeLog] = []
    var eventStream: [TrainingEventRecord] = []
    var metricStream: [TrainingMetricRecord] = []
    var floorCurve: [Double] = []
    var workers: [WorkerStatus] = []
    var lastStatusUpdate: Date?

    // Loss history (accumulated from polling, max 500 points)
    var lossHistory: [LossPoint] = []
    // Diagnostics history (EV, KL, advantage — max 500 points)
    var diagnosticsHistory: [DiagnosticsPoint] = []
    // System stats (accumulated, max 720 = 30 min at 2.5s)
    var systemStats: SystemStats?
    var systemHistory: [SystemStats] = []
    // Persistent metrics history from metrics_history.jsonl
    var metricsHistory: [MetricsSnapshot] = []

    var configHistory: [String: ConfigStats] = [:]

    var isLive: Bool {
        guard let last = lastStatusUpdate else { return false }
        return Date().timeIntervalSince(last) < 10
    }

    var peakFloor: Int {
        if let peak = status?.peakFloor ?? status?.replayBestFloor { return peak }
        return (recentEpisodes + topEpisodes)
            .max(by: { $0.effectiveFloor < $1.effectiveFloor })?.effectiveFloor ?? 0
    }

    var latestEvent: TrainingEventRecord? {
        eventStream.last
    }

    var latestMetric: TrainingMetricRecord? {
        metricStream.last
    }

    var currentBenchmarkReport: BenchmarkReportArtifact? {
        benchmarkReports.first?.report
    }

    var currentFrontierRanking: [FrontierPointArtifact] {
        guard let frontierReport else { return [] }
        let lookup = Dictionary(uniqueKeysWithValues: frontierReport.points.map { ($0.label, $0) })
        return frontierReport.ranking.compactMap { lookup[$0] }
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

    func appendDiagnostics(from status: TrainingStatus) {
        let step = status.trainSteps ?? diagnosticsHistory.count
        let point = DiagnosticsPoint(
            step: step,
            explainedVariance: status.explainedVariance ?? 0,
            meanValue: status.meanValue ?? 0,
            klDivergence: status.klDivergence ?? 0,
            meanAdvantage: status.meanAdvantage ?? 0,
            meanReturn: status.meanReturn ?? 0
        )
        if diagnosticsHistory.last?.step != point.step {
            diagnosticsHistory.append(point)
            if diagnosticsHistory.count > 500 { diagnosticsHistory.removeFirst() }
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

struct DiagnosticsPoint: Identifiable {
    let id = UUID()
    let step: Int
    let explainedVariance: Double
    let meanValue: Double
    let klDivergence: Double
    let meanAdvantage: Double
    let meanReturn: Double
}

struct RoomPerformance {
    let count: Int
    let avgTurns: Double
    let avgHpLost: Double
    let potionRate: Double
}

struct ConfigStats: Identifiable {
    let id = UUID()
    let name: String
    var games: Int
    var avgFloor: Double
    var peakFloor: Int
    var lastLoss: Double?
    var phase: String?
    var isActive: Bool
}
