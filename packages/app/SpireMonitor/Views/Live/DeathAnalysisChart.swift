import SwiftUI
import Charts

struct DeathAnalysisChart: View {
    let deaths: [(enemy: String, count: Int)]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Top Killers")

            if deaths.isEmpty {
                Text("No death data yet")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsTextMuted)
                    .frame(maxWidth: .infinity, minHeight: 150)
            } else {
                Chart {
                    ForEach(Array(deaths.enumerated()), id: \.offset) { index, item in
                        BarMark(
                            x: .value("Deaths", item.count),
                            y: .value("Enemy", item.enemy)
                        )
                        .foregroundStyle(index == 0 ? Color.stsRed : Color.stsAccent.opacity(0.7))
                    }
                }
                .chartYAxis {
                    AxisMarks { value in
                        AxisValueLabel()
                            .font(.stsLabel)
                            .foregroundStyle(Color.stsTextDim)
                    }
                }
                .chartXAxis {
                    AxisMarks { value in
                        AxisValueLabel()
                            .font(.stsLabel)
                            .foregroundStyle(Color.stsTextDim)
                    }
                }
                .chartPlotStyle { plotArea in
                    plotArea.background(Color.stsBg.opacity(0.5))
                }
            }
        }
    }
}
