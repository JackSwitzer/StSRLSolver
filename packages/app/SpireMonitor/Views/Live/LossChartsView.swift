import SwiftUI
import Charts

/// Toggle between loss metric types
enum LossMetric: String, CaseIterable {
    case total = "Total Loss"
    case policy = "Policy"
    case value = "Value"
    case avgFloor = "Avg Floor"
}

struct LossChartsView: View {
    let history: [LossPoint]
    let configHistory: [String: [LossPoint]]
    @State private var metric: LossMetric = .total

    private var hasConfigData: Bool { !configHistory.isEmpty }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                SectionHeader(title: hasConfigData ? "Per-Config Comparison" : "Training Loss")
                Spacer()
                Picker("Metric", selection: $metric) {
                    ForEach(LossMetric.allCases, id: \.self) { m in
                        Text(m.rawValue).tag(m)
                    }
                }
                .pickerStyle(.segmented)
                .frame(maxWidth: 300)
            }

            if hasConfigData {
                configChart
            } else if history.count < 2 {
                Text("Accumulating loss data...")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsTextMuted)
                    .frame(maxWidth: .infinity, minHeight: 120)
            } else {
                singleChart
            }
        }
    }

    // MARK: - Per-config chart (from perf_log.jsonl)

    private var configChart: some View {
        VStack(alignment: .leading, spacing: 4) {
            Chart {
                ForEach(configHistory.keys.sorted(), id: \.self) { config in
                    if let points = configHistory[config] {
                        ForEach(points) { pt in
                            LineMark(
                                x: .value("Step", pt.step),
                                y: .value(metric.rawValue, metricValue(pt))
                            )
                            .foregroundStyle(by: .value("Config", config))
                            .interpolationMethod(.catmullRom)
                        }
                    }
                }
            }
            .chartForegroundStyleScale(configColorScale)
            .chartYAxis {
                AxisMarks(position: .leading) { _ in
                    AxisValueLabel().font(.stsLabel).foregroundStyle(Color.stsTextDim)
                    AxisGridLine().foregroundStyle(Color.stsBorderDim)
                }
            }
            .chartXAxis {
                AxisMarks { _ in
                    AxisValueLabel().font(.stsLabel).foregroundStyle(Color.stsTextDim)
                }
            }
            .chartPlotStyle { plotArea in
                plotArea.background(Color.stsBg.opacity(0.5))
            }

            // Legend
            HStack(spacing: 12) {
                ForEach(configHistory.keys.sorted(), id: \.self) { config in
                    legendItem(config, color: colorForConfig(config))
                }
            }
            .padding(.top, 4)
        }
    }

    // MARK: - Single-stream chart (legacy fallback)

    private var singleChart: some View {
        VStack(alignment: .leading, spacing: 4) {
            Chart {
                ForEach(history) { pt in
                    LineMark(
                        x: .value("Step", pt.step),
                        y: .value("Loss", pt.total)
                    )
                    .foregroundStyle(by: .value("Series", "Total"))
                    .interpolationMethod(.catmullRom)
                }
                ForEach(history) { pt in
                    LineMark(
                        x: .value("Step", pt.step),
                        y: .value("Loss", pt.policy)
                    )
                    .foregroundStyle(by: .value("Series", "Policy"))
                    .interpolationMethod(.catmullRom)
                }
                ForEach(history) { pt in
                    LineMark(
                        x: .value("Step", pt.step),
                        y: .value("Loss", pt.value)
                    )
                    .foregroundStyle(by: .value("Series", "Value"))
                    .interpolationMethod(.catmullRom)
                }
            }
            .chartForegroundStyleScale([
                "Total": Color.stsRed,
                "Policy": Color.stsBlue,
                "Value": Color.stsGold,
            ])
            .chartYAxis {
                AxisMarks(position: .leading) { _ in
                    AxisValueLabel().font(.stsLabel).foregroundStyle(Color.stsTextDim)
                    AxisGridLine().foregroundStyle(Color.stsBorderDim)
                }
            }
            .chartXAxis {
                AxisMarks { _ in
                    AxisValueLabel().font(.stsLabel).foregroundStyle(Color.stsTextDim)
                }
            }
            .chartPlotStyle { plotArea in
                plotArea.background(Color.stsBg.opacity(0.5))
            }

            HStack(spacing: 16) {
                legendItem("Total", color: .stsRed)
                legendItem("Policy", color: .stsBlue)
                legendItem("Value", color: .stsGold)
            }
            .padding(.top, 4)
        }
    }

    // MARK: - Helpers

    private func metricValue(_ pt: LossPoint) -> Double {
        switch metric {
        case .total: return pt.total
        case .policy: return pt.policy
        case .value: return pt.value
        case .avgFloor: return pt.avgFloor
        }
    }

    /// Stable colors per config name
    private static let configColors: [Color] = [
        .stsBlue, .stsRed, .stsGreen, .stsGold, .stsOrange,
        Color(hex: 0xbc8cff), Color(hex: 0xff7eb6), Color(hex: 0x42be65),
    ]

    private func colorForConfig(_ config: String) -> Color {
        let configs = configHistory.keys.sorted()
        guard let idx = configs.firstIndex(of: config) else { return .stsTextDim }
        return Self.configColors[idx % Self.configColors.count]
    }

    private var configColorScale: KeyValuePairs<String, Color> {
        // Swift Charts requires KeyValuePairs — build from sorted keys
        let sorted = configHistory.keys.sorted()
        // Can't dynamically build KeyValuePairs, so use array of tuples
        // and pass via chartForegroundStyleScale domain+range instead
        return KeyValuePairs(dictionaryLiteral:
            (sorted.count > 0 ? sorted[0] : "", sorted.count > 0 ? colorForConfig(sorted[0]) : .clear),
            (sorted.count > 1 ? sorted[1] : "_1", sorted.count > 1 ? colorForConfig(sorted[1]) : .clear),
            (sorted.count > 2 ? sorted[2] : "_2", sorted.count > 2 ? colorForConfig(sorted[2]) : .clear),
            (sorted.count > 3 ? sorted[3] : "_3", sorted.count > 3 ? colorForConfig(sorted[3]) : .clear),
            (sorted.count > 4 ? sorted[4] : "_4", sorted.count > 4 ? colorForConfig(sorted[4]) : .clear)
        )
    }

    private func legendItem(_ label: String, color: Color) -> some View {
        HStack(spacing: 4) {
            Circle().fill(color).frame(width: 6, height: 6)
            Text(label).font(.stsLabel).foregroundStyle(Color.stsTextDim)
        }
    }
}
