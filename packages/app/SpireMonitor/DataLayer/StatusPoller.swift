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
        while isRunning && !Task.isCancelled {
            await poll()
            try? await Task.sleep(for: .seconds(2.5))
        }
    }

    /// Resolve the logs URL, following symlinks so reads work even when
    /// `logs/active` is a relative symlink (e.g. `runs/run_XXXXXX`).
    private func resolvedLogsURL() -> URL {
        let raw = config.logsPath
        let resolved = raw.path().isEmpty ? raw : URL(filePath: (raw.path() as NSString).resolvingSymlinksInPath)
        return resolved
    }

    private func poll() async {
        let logsURL = resolvedLogsURL()

        // Read status.json -- check file mtime to detect stale data.
        // Use resolved path so FileManager attribute lookups hit the real file.
        let statusURL = logsURL.appending(path: "status.json")
        let resolvedStatusPath = (statusURL.path() as NSString).resolvingSymlinksInPath
        if let data = try? Data(contentsOf: URL(filePath: resolvedStatusPath)),
           let status = try? JSONDecoder().decode(TrainingStatus.self, from: data) {
            let mtime = (try? FileManager.default.attributesOfItem(atPath: resolvedStatusPath)[.modificationDate] as? Date) ?? .distantPast
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

        // Reload episodes + perf_log every 4th poll (~10s)
        pollCount += 1
        if pollCount % 4 == 0 {
            let recent = await EpisodeLoader.loadRecent(from: logsURL)
            let top = await EpisodeLoader.loadTop(from: logsURL)

            // Collect perf_log.jsonl URLs: active run + all archived runs
            var perfLogURLs: [URL] = []

            // Active run perf_log
            let activePerfLog = logsURL.appending(path: "perf_log.jsonl")
            if FileManager.default.fileExists(atPath: activePerfLog.path()) {
                perfLogURLs.append(activePerfLog)
            }

            // Scan archived runs: logs/runs/*/perf_log.jsonl
            let runsDir = config.archivedRunsPath
            if let runDirs = try? FileManager.default.contentsOfDirectory(
                at: runsDir,
                includingPropertiesForKeys: [.isDirectoryKey],
                options: [.skipsHiddenFiles]
            ) {
                for runDir in runDirs {
                    let perfLog = runDir.appending(path: "perf_log.jsonl")
                    if FileManager.default.fileExists(atPath: perfLog.path()) {
                        perfLogURLs.append(perfLog)
                    }
                }
            }

            let collectedPerfLogURLs = perfLogURLs
            await MainActor.run {
                if !recent.isEmpty { store.recentEpisodes = recent }
                if !top.isEmpty { store.topEpisodes = top }
                store.loadPerfLogs(from: collectedPerfLogURLs)
            }
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
}
