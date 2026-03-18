import Foundation

actor StatusPoller {
    private let config: AppConfig
    private let store: DataStore
    private let decoder: JSONDecoder = {
        let d = JSONDecoder()
        d.keyDecodingStrategy = .convertFromSnakeCase
        return d
    }()

    private var isRunning = false

    init(config: AppConfig, store: DataStore) {
        self.config = config
        self.store = store
    }

    func start() {
        guard !isRunning else { return }
        isRunning = true
        Task { await pollLoop() }
    }

    func stop() {
        isRunning = false
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
