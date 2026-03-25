import SwiftUI
import Charts

struct DiagnosticsChartsView: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        let data = appState.store.diagnosticsHistory

        VStack(spacing: 16) {
            // Explained Variance chart
            VStack(alignment: .leading, spacing: 4) {
                Text("Explained Variance")
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsTextMuted)

                Chart(data) { point in
                    LineMark(
                        x: .value("Step", point.step),
                        y: .value("EV", point.explainedVariance)
                    )
                    .foregroundStyle(Color.stsAccent)
                }
                .frame(height: 120)
                .chartYAxis {
                    AxisMarks(position: .leading)
                }
            }
            .padding(12)
            .background(Color.stsCard)
            .clipShape(RoundedRectangle(cornerRadius: 8))

            // KL Divergence chart
            VStack(alignment: .leading, spacing: 4) {
                Text("KL Divergence")
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsTextMuted)

                Chart(data) { point in
                    LineMark(
                        x: .value("Step", point.step),
                        y: .value("KL", point.klDivergence)
                    )
                    .foregroundStyle(Color.stsOrange)
                }
                .frame(height: 120)
                .chartYAxis {
                    AxisMarks(position: .leading)
                }
            }
            .padding(12)
            .background(Color.stsCard)
            .clipShape(RoundedRectangle(cornerRadius: 8))

            // Mean Advantage chart
            VStack(alignment: .leading, spacing: 4) {
                Text("Mean Advantage")
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsTextMuted)

                Chart(data) { point in
                    LineMark(
                        x: .value("Step", point.step),
                        y: .value("Adv", point.meanAdvantage)
                    )
                    .foregroundStyle(Color.stsGreen)
                }
                .frame(height: 120)
                .chartYAxis {
                    AxisMarks(position: .leading)
                }
            }
            .padding(12)
            .background(Color.stsCard)
            .clipShape(RoundedRectangle(cornerRadius: 8))
        }
    }
}
