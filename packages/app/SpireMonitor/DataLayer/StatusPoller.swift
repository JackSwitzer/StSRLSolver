import Foundation

actor StatusPoller {
    private let config: AppConfig
    private let store: DataStore
    private var isRunning = false
    private var pollTask: Task<Void, Never>?
    private var pollCount = 0

    init(config: AppConfig, store: DataStore) {
        self.config = config
        self.store = store
    }

    func start() {
        guard !isRunning else { return }
        isRunning = true
        pollTask = Task { await pollLoop() }
    }

    func stop() {
        isRunning = false
        pollTask?.cancel()
        pollTask = nil
    }

    private func pollLoop() async {
        while isRunning {
            await poll()
            try? await Task.sleep(for: .seconds(2.5))
        }
    }

    private func poll() async {
        // Always use the latest timestamped run directory
        let logsURL = Self.latestRunDir(config: config)

        // Read status.json -- check file mtime to detect stale data
        let statusURL = logsURL.appending(path: "status.json")
        if let data = try? Data(contentsOf: statusURL),
           let status = try? JSONDecoder().decode(TrainingStatus.self, from: data) {
            let mtime = (try? FileManager.default.attributesOfItem(atPath: statusURL.path())[.modificationDate] as? Date) ?? .distantPast
            await MainActor.run {
                store.status = status
                store.lastStatusUpdate = mtime
                store.appendLoss(from: status)
            }
        }

        // Read floor_curve.json
        if let data = try? Data(contentsOf: logsURL.appending(path: "floor_curve.json")),
           let curve = try? JSONDecoder().decode([Double].self, from: data) {
            await MainActor.run {
                store.floorCurve = curve
            }
        }

        // Reload episodes every 4th poll (~10s) to pick up new data
        pollCount += 1
        if pollCount % 4 == 0 {
            let recent = await EpisodeLoader.loadRecent(from: logsURL)
            let top = await EpisodeLoader.loadTop(from: logsURL)
            await MainActor.run {
                if !recent.isEmpty { store.recentEpisodes = recent }
                if !top.isEmpty { store.topEpisodes = top }
            }
        }

        // Keep config in sync
        if logsURL != config.logsPath {
            await MainActor.run { config.logsPath = logsURL }
        }

        // Scan workers/*.json
        let workersDir = logsURL.appending(path: "workers")
        if let files = try? FileManager.default.contentsOfDirectory(at: workersDir, includingPropertiesForKeys: nil) {
            var newWorkers: [WorkerStatus] = []
            for file in files where file.pathExtension == "json" {
                if let data = try? Data(contentsOf: file),
                   let worker = try? JSONDecoder().decode(WorkerStatus.self, from: data) {
                    newWorkers.append(worker)
                }
            }
            let sorted = newWorkers.sorted { ($0.workerID ?? 0) < ($1.workerID ?? 0) }
            await MainActor.run {
                store.workers = sorted
            }
        }
    }

    /// Find the latest run directory by scanning known locations.
    /// Picks the most recent directory that has a status.json.
    private static func latestRunDir(config: AppConfig) -> URL {
        let fm = FileManager.default
        let logsBase = config.logsPath.deletingLastPathComponent()

        // Candidate directories: logs/overnight, logs/weekend-run, logs/runs/run_*, logs/training/run_*
        var best: (url: URL, mtime: Date)?

        for dir in ["overnight", "weekend-run"] {
            let candidate = logsBase.appending(path: dir)
            let statusFile = candidate.appending(path: "status.json")
            if let attrs = try? fm.attributesOfItem(atPath: statusFile.path()),
               let mtime = attrs[.modificationDate] as? Date {
                if best == nil || mtime > best!.mtime { best = (candidate, mtime) }
            }
        }

        for subdir in ["runs", "training"] {
            let scanDir = logsBase.appending(path: subdir)
            guard let contents = try? fm.contentsOfDirectory(at: scanDir, includingPropertiesForKeys: nil) else { continue }
            for dir in contents where dir.lastPathComponent.hasPrefix("run_") {
                let statusFile = dir.appending(path: "status.json")
                if let attrs = try? fm.attributesOfItem(atPath: statusFile.path()),
                   let mtime = attrs[.modificationDate] as? Date {
                    if best == nil || mtime > best!.mtime { best = (dir, mtime) }
                }
            }
        }

        return best?.url ?? config.logsPath
    }
}
