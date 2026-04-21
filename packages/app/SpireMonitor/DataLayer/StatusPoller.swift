import Foundation

actor StatusPoller {
    private let config: AppConfig
    private let store: DataStore
    private var isRunning = false
    private var pollTask: Task<Void, Never>?

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

    private func poll() async {
        let logsURL = config.logsPath
        let bundle = await MonitorArtifactLoader.load(from: logsURL)

        await MainActor.run {
            store.runManifest = bundle.manifest
            store.frontierReport = bundle.frontier
            store.seedValidationReports = bundle.seedValidationReports
            store.checkpointComparisons = bundle.checkpointComparisons
            store.benchmarkReports = bundle.benchmarkReports
            store.artifactEpisodes = bundle.artifactEpisodes
            store.eventStream = bundle.events
            store.metricStream = bundle.metricStream
            store.recordedRunReplay = bundle.recordedRunReplay
        }
    }
}
