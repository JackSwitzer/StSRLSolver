import SwiftUI
import Charts

struct SystemStatsBar: View {
    let current: SystemStats?
    let history: [SystemStats]

    var body: some View {
        HStack(spacing: 16) {
            // CPU chart
            miniGauge("CPU", value: current?.cpuPercent ?? 0, unit: "%",
                       data: history.map(\.cpuPercent), color: .stsBlue)

            // GPU chart
            miniGauge("GPU", value: current?.gpuPercent ?? 0, unit: "%",
                       data: history.compactMap(\.gpuPercent), color: .stsGold)

            // RAM chart
            miniGauge("RAM", value: current?.memoryUsedGB ?? 0,
                       unit: String(format: "/%.0fGB", current?.memoryTotalGB ?? 24),
                       data: history.map(\.memoryUsedGB), color: .stsGreen,
                       maxValue: current?.memoryTotalGB ?? 24,
                       formatter: { String(format: "%.1f", $0) })
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .background(Color.stsCard)
    }

    private func miniGauge(_ label: String, value: Double, unit: String,
                           data: [Double], color: Color,
                           maxValue: Double = 100,
                           formatter: ((Double) -> String)? = nil) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 6) {
                Text(label)
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsTextDim)
                Text(formatter?(value) ?? String(format: "%.0f", value))
                    .font(.stsValue)
                    .foregroundStyle(color)
                Text(unit)
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsTextDim)
            }

            if data.count > 2 {
                Chart {
                    ForEach(Array(data.enumerated()), id: \.offset) { i, val in
                        AreaMark(x: .value("T", i), y: .value("V", val))
                            .foregroundStyle(color.opacity(0.15))
                            .interpolationMethod(.catmullRom)
                        LineMark(x: .value("T", i), y: .value("V", val))
                            .foregroundStyle(color)
                            .interpolationMethod(.catmullRom)
                    }
                }
                .chartXAxis(.hidden)
                .chartYAxis(.hidden)
                .frame(height: 30)
            } else {
                // Progress bar fallback when no history yet
                GeometryReader { geo in
                    ZStack(alignment: .leading) {
                        RoundedRectangle(cornerRadius: 2).fill(Color.stsBorderDim)
                        RoundedRectangle(cornerRadius: 2).fill(color)
                            .frame(width: geo.size.width * min(value / maxValue, 1.0))
                    }
                }
                .frame(height: 4)
            }
        }
        .frame(maxWidth: .infinity)
    }
}
