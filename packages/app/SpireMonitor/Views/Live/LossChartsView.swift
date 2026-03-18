import SwiftUI
import Charts

struct LossChartsView: View {
    let history: [LossPoint]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Training Loss")

            if history.count < 2 {
                Text("Accumulating loss data...")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsTextMuted)
                    .frame(maxWidth: .infinity, minHeight: 120)
            } else {
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

                // Legend
                HStack(spacing: 16) {
                    legendItem("Total", color: .stsRed)
                    legendItem("Policy", color: .stsBlue)
                    legendItem("Value", color: .stsGold)
                }
                .padding(.top, 4)
            }
        }
    }

    private func legendItem(_ label: String, color: Color) -> some View {
        HStack(spacing: 4) {
            Circle().fill(color).frame(width: 6, height: 6)
            Text(label).font(.stsLabel).foregroundStyle(Color.stsTextDim)
        }
    }
}
