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
        while isRunning {
            await poll()
            try? await Task.sleep(for: .seconds(2.5))
        }
    }

    private func poll() async {
        let logsURL = config.logsPath

        // Read status.json
        if let data = try? Data(contentsOf: logsURL.appending(path: "status.json")),
           let status = try? JSONDecoder().decode(TrainingStatus.self, from: data) {
            await MainActor.run {
                store.status = status
                store.lastStatusUpdate = Date()
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
