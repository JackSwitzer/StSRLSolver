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

        async let manifest = ManifestLoader.load(from: logsURL)
        async let frontier = FrontierReportLoader.load(from: logsURL)
        async let benchmarkReports = BenchmarkReportLoader.loadAll(from: logsURL)
        async let artifactEpisodes = ArtifactEpisodeLogLoader.loadAll(from: logsURL)
        async let events = EventStreamLoader.load(from: logsURL)
        async let metricStream = MetricStreamLoader.load(from: logsURL)

        let loadedManifest = await manifest
        let loadedFrontier = await frontier
        let loadedBenchmarkReports = await benchmarkReports
        let loadedArtifactEpisodes = await artifactEpisodes
        let loadedEvents = await events
        let loadedMetricStream = await metricStream

        await MainActor.run {
            store.runManifest = loadedManifest
            store.frontierReport = loadedFrontier
            store.benchmarkReports = loadedBenchmarkReports
            store.artifactEpisodes = loadedArtifactEpisodes
            store.eventStream = loadedEvents
            store.metricStream = loadedMetricStream
        }
    }
}
