import SwiftUI
import Charts

struct SystemStatsBar: View {
    let current: SystemStats?
    let history: [SystemStats]
    @State private var showHistory = false

    var body: some View {
        VStack(spacing: 0) {
            // Compact bar
            HStack(spacing: 20) {
                statGauge("CPU", value: current?.cpuPercent ?? 0, max: 100, color: .stsBlue)
                statGauge("GPU", value: current?.gpuPercent ?? 0, max: 100, color: .stsGold)
                statGauge("RAM",
                    value: current?.memoryUsedGB ?? 0,
                    max: current?.memoryTotalGB ?? 24,
                    color: .stsGreen,
                    label: String(format: "%.1f/%.0fGB", current?.memoryUsedGB ?? 0, current?.memoryTotalGB ?? 24))

                Spacer()

                Button(action: { showHistory.toggle() }) {
                    Image(systemName: showHistory ? "chart.line.downtrend.xyaxis" : "chart.line.uptrend.xyaxis")
                        .font(.system(size: 12))
                        .foregroundStyle(Color.stsTextDim)
                }
                .buttonStyle(.plain)
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 6)
            .background(Color.stsCard)

            // Expandable history charts
            if showHistory && history.count > 2 {
                Divider().background(Color.stsBorder)
                HStack(spacing: 12) {
                    miniChart("CPU %", data: history.map(\.cpuPercent), color: .stsBlue)
                    miniChart("RAM GB", data: history.map(\.memoryUsedGB), color: .stsGreen)
                    if history.compactMap(\.gpuPercent).count > 0 {
                        miniChart("GPU %", data: history.compactMap(\.gpuPercent), color: .stsGold)
                    }
                }
                .padding(12)
                .background(Color.stsCard)
                .frame(height: 120)
            }
        }
    }

    private func statGauge(_ label: String, value: Double, max: Double, color: Color, label customLabel: String? = nil) -> some View {
        HStack(spacing: 8) {
            Text(label)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
                .frame(width: 28, alignment: .trailing)

            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: 2)
                        .fill(Color.stsBorderDim)
                    RoundedRectangle(cornerRadius: 2)
                        .fill(color)
                        .frame(width: geo.size.width * min(value / max, 1.0))
                }
            }
            .frame(width: 80, height: 6)

            Text(customLabel ?? String(format: "%.0f%%", value))
                .font(.stsLabel)
                .foregroundStyle(Color.stsText)
                .frame(width: 60, alignment: .leading)
        }
    }

    private func miniChart(_ title: String, data: [Double], color: Color) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title).font(.stsLabel).foregroundStyle(Color.stsTextDim)
            Chart {
                ForEach(Array(data.enumerated()), id: \.offset) { i, val in
                    LineMark(x: .value("T", i), y: .value("V", val))
                        .foregroundStyle(color)
                        .interpolationMethod(.catmullRom)
                }
            }
            .chartXAxis(.hidden)
            .chartYAxis(.hidden)
        }
        .frame(maxWidth: .infinity)
    }
}
