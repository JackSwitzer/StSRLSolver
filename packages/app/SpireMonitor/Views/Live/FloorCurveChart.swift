import SwiftUI
import Charts

struct FloorCurveChart: View {
    let data: [Double]
    var title: String = "Floor Curve"

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: title)

            if data.isEmpty {
                Text("No floor data yet")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsTextMuted)
                    .frame(maxWidth: .infinity, minHeight: 150)
            } else {
                Chart {
                    ForEach(Array(data.enumerated()), id: \.offset) { index, value in
                        LineMark(
                            x: .value("Game", index),
                            y: .value("Floor", value)
                        )
                        .foregroundStyle(Color.stsAccent)
                        .interpolationMethod(.catmullRom)

                        AreaMark(
                            x: .value("Game", index),
                            y: .value("Floor", value)
                        )
                        .foregroundStyle(Color.stsAccent.opacity(0.06))
                        .interpolationMethod(.catmullRom)
                    }

                    // Reference lines
                    RuleMark(y: .value("F5", 5))
                        .foregroundStyle(Color.stsTextMuted.opacity(0.3))
                        .lineStyle(StrokeStyle(dash: [4, 4]))
                    RuleMark(y: .value("F10", 10))
                        .foregroundStyle(Color.stsTextMuted.opacity(0.3))
                        .lineStyle(StrokeStyle(dash: [4, 4]))
                    RuleMark(y: .value("F15", 15))
                        .foregroundStyle(Color.stsTextMuted.opacity(0.3))
                        .lineStyle(StrokeStyle(dash: [4, 4]))
                }
                .chartYAxis {
                    AxisMarks(position: .leading) { value in
                        AxisValueLabel()
                            .font(.stsLabel)
                            .foregroundStyle(Color.stsTextDim)
                        AxisGridLine()
                            .foregroundStyle(Color.stsBorderDim)
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
