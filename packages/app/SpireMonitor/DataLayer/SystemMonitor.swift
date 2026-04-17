import Foundation
#if canImport(Darwin)
import Darwin
#endif

actor SystemMonitor {
    private let store: DataStore
    private var isRunning = false
    private var monitorTask: Task<Void, Never>?
    // Previous CPU ticks for delta calculation
    private var prevUserTicks: UInt64 = 0
    private var prevSystemTicks: UInt64 = 0
    private var prevIdleTicks: UInt64 = 0

    init(store: DataStore) {
        self.store = store
    }

    func start() {
        guard !isRunning else { return }
        isRunning = true
        monitorTask = Task { await pollLoop() }
    }

    func stop() {
        isRunning = false
        monitorTask?.cancel()
        monitorTask = nil
    }

    private func pollLoop() async {
        while isRunning && !Task.isCancelled {
            let stats = await collectStats()
            await MainActor.run {
                store.systemStats = stats
                store.systemHistory.append(stats)
                if store.systemHistory.count > 720 { // 30 min at 2.5s
                    store.systemHistory.removeFirst()
                }
            }
            try? await Task.sleep(for: .seconds(2.5))
        }
    }

    private func collectStats() async -> SystemStats {
        let cpu = getCPUUsage()
        let (usedGB, totalGB) = getMemory()
        return SystemStats(
            timestamp: Date(),
            cpuPercent: cpu,
            memoryUsedGB: usedGB,
            memoryTotalGB: totalGB,
            gpuPercent: nil
        )
    }

    private func getCPUUsage() -> Double {
        var loadInfo = host_cpu_load_info_data_t()
        var count = mach_msg_type_number_t(MemoryLayout<host_cpu_load_info_data_t>.size / MemoryLayout<integer_t>.size)
        let result = withUnsafeMutablePointer(to: &loadInfo) {
            $0.withMemoryRebound(to: integer_t.self, capacity: Int(count)) { ptr in
                host_statistics(mach_host_self(), HOST_CPU_LOAD_INFO, ptr, &count)
            }
        }
        guard result == KERN_SUCCESS else { return 0 }
        let user = UInt64(loadInfo.cpu_ticks.0)
        let system = UInt64(loadInfo.cpu_ticks.1)
        let idle = UInt64(loadInfo.cpu_ticks.2)

        let dUser = user - prevUserTicks
        let dSystem = system - prevSystemTicks
        let dIdle = idle - prevIdleTicks
        let dTotal = dUser + dSystem + dIdle

        prevUserTicks = user
        prevSystemTicks = system
        prevIdleTicks = idle

        // First poll: no delta yet, return 0
        guard dTotal > 0 else { return 0 }
        return Double(dUser + dSystem) / Double(dTotal) * 100
    }

    private nonisolated func getMemory() -> (used: Double, total: Double) {
        let totalBytes = ProcessInfo.processInfo.physicalMemory
        let totalGB = Double(totalBytes) / 1_073_741_824

        var stats = vm_statistics64_data_t()
        var count = mach_msg_type_number_t(MemoryLayout<vm_statistics64_data_t>.size / MemoryLayout<integer_t>.size)
        let result = withUnsafeMutablePointer(to: &stats) {
            $0.withMemoryRebound(to: integer_t.self, capacity: Int(count)) { ptr in
                host_statistics64(mach_host_self(), HOST_VM_INFO64, ptr, &count)
            }
        }
        guard result == KERN_SUCCESS else { return (0, totalGB) }
        let pageSize = Double(vm_kernel_page_size)
        let active = Double(stats.active_count) * pageSize
        let wired = Double(stats.wire_count) * pageSize
        let compressed = Double(stats.compressor_page_count) * pageSize
        let usedGB = (active + wired + compressed) / 1_073_741_824
        return (usedGB, totalGB)
    }
}
