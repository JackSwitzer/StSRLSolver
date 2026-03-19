import Foundation

@MainActor @Observable
final class DataStore {
    var status: TrainingStatus?
    var recentEpisodes: [Episode] = []
    var topEpisodes: [Episode] = []
    var floorCurve: [Double] = []
    var workers: [WorkerStatus] = []
    var lastStatusUpdate: Date?

    // Per-config loss history loaded from perf_log.jsonl
    var configLossHistory: [String: [LossPoint]] = [:]

    // Legacy single-stream loss (fallback when no perf_log)
    var lossHistory: [LossPoint] = []

    // System stats (accumulated, max 720 = 30 min at 2.5s)
    var systemStats: SystemStats?
    var systemHistory: [SystemStats] = []

    var isLive: Bool {
        guard let last = lastStatusUpdate else { return false }
        return Date().timeIntervalSince(last) < 10
    }

    var peakFloor: Int {
        if let peak = status?.peakFloor ?? status?.replayBestFloor { return peak }
        return (recentEpisodes + topEpisodes)
            .max(by: { $0.effectiveFloor < $1.effectiveFloor })?.effectiveFloor ?? 0
    }

    /// All config names that have loss data, sorted alphabetically
    var configNames: [String] {
        configLossHistory.keys.sorted()
    }

    /// Whether we have per-config data (from perf_log.jsonl)
    var hasConfigData: Bool {
        !configLossHistory.isEmpty
    }

    func appendLoss(from status: TrainingStatus) {
        guard let total = status.totalLoss else { return }
        let point = LossPoint(
            step: status.trainSteps ?? lossHistory.count,
            total: total,
            policy: status.policyLoss ?? 0,
            value: status.valueLoss ?? 0,
            avgFloor: status.avgFloor100 ?? 0,
            configName: status.configName ?? ""
        )
        if lossHistory.last?.total != point.total || lossHistory.last?.step != point.step {
            lossHistory.append(point)
            if lossHistory.count > 500 { lossHistory.removeFirst() }
        }
    }

    /// Load per-config loss history from a single perf_log.jsonl
    func loadPerfLog(from url: URL) {
        loadPerfLogs(from: [url])
    }

    /// Load and merge per-config loss history from multiple perf_log.jsonl files.
    /// Used to combine the active run with archived runs for comparison charts.
    func loadPerfLogs(from urls: [URL]) {
        var newHistory: [String: [LossPoint]] = [:]
        let decoder = JSONDecoder()

        for url in urls {
            guard let data = try? Data(contentsOf: url),
                  let text = String(data: data, encoding: .utf8) else { continue }

            for line in text.split(separator: "\n") {
                guard let lineData = line.data(using: .utf8),
                      let entry = try? decoder.decode(PerfLogEntry.self, from: lineData) else {
                    continue
                }
                let config = entry.configName ?? "unknown"
                let point = LossPoint(
                    step: entry.trainStep ?? 0,
                    total: entry.totalLoss ?? 0,
                    policy: entry.policyLoss ?? 0,
                    value: entry.valueLoss ?? 0,
                    avgFloor: entry.avgFloor ?? 0,
                    configName: config
                )
                newHistory[config, default: []].append(point)
            }
        }

        // Sort each config's points by step for consistent charting
        for key in newHistory.keys {
            newHistory[key]?.sort { $0.step < $1.step }
        }

        configLossHistory = newHistory
    }
}

struct LossPoint: Identifiable {
    let id = UUID()
    let step: Int
    let total: Double
    let policy: Double
    let value: Double
    let avgFloor: Double
    let configName: String

    init(step: Int, total: Double, policy: Double, value: Double,
         avgFloor: Double = 0, configName: String = "") {
        self.step = step
        self.total = total
        self.policy = policy
        self.value = value
        self.avgFloor = avgFloor
        self.configName = configName
    }
}

/// JSON shape matching perf_log.jsonl entries
private struct PerfLogEntry: Codable {
    let configName: String?
    let trainStep: Int?
    let totalLoss: Double?
    let policyLoss: Double?
    let valueLoss: Double?
    let avgFloor: Double?

    enum CodingKeys: String, CodingKey {
        case configName = "config_name"
        case trainStep = "train_step"
        case totalLoss = "total_loss"
        case policyLoss = "policy_loss"
        case valueLoss = "value_loss"
        case avgFloor = "avg_floor"
    }
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
